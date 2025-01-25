use super::*;
use bevy::{
    window::*,
    core_pipeline::Skybox,
    render::{
        render_resource::*,
        primitives::*,
    }
};

const MAX_TASKS: usize = 8;

pub fn exit_world(
    mut commands: Commands,
    world_assets: Res<Assets<WorldInfo>>,
    controller: Option<Res<Controller>>
) {
    if let Some(controller) = controller {
        let world = world_assets.get(&controller.world).unwrap();
        for entity in &world.entities {
            commands.entity(*entity).despawn();
        }

        commands.remove_resource::<Controller>();
    }
}

pub fn enter_world(
    mut commands: Commands,
    controller: Res<Controller>,
    mut worlds: ResMut<Assets<WorldInfo>>,
    assets: Res<AssetServer>,
) {
    let world = worlds.get_mut(&controller.world).unwrap();

    commands.insert_resource(ViewBlocks::empty());
    commands.insert_resource(SelectedBlock(0));

    commands.insert_resource(AmbientLight {
        color: Color::Srgba(Srgba::rgb_u8(210, 220, 240)),
        brightness: 500.0,
        ..default()
    });

    // Create centralized node with image node of cross image
    let cross = commands.spawn(
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            position_type: PositionType::Absolute,
            ..default()
        }
    ).with_child(ImageNode::new(assets.load("cross.png"))).id();

    let light = commands
        .spawn((
            DirectionalLight {
                illuminance: 1200.0,
                ..default()
            },
            Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -3.14 / 2.8, 0.0, 0.0)),
        ))
        .id();
    
    let player = commands.spawn((
        Player::new(),
        Camera3d::default(),
        Transform::from_translation(Vec3::new(0.0, 32.0, 0.0)),
        LoadArea::new(16, 8),
        MainCamera::new(),
    )).id();

    world.entities.extend([cross, light, player])
}

pub fn skybox(
    mut commands: Commands,
    cameras: Query<Entity, With<Camera3d>>,
    controller: Res<Controller>,
    mut images: ResMut<Assets<Image>>,
    world_assets: Res<Assets<WorldInfo>>
) {
    let world = world_assets.get(&controller.world).unwrap();

    let Some(image) = images.get(&world.skybox) else { return };
    if image.texture_descriptor.array_layer_count() == 1 {
        let image = images.get_mut(&world.skybox).unwrap();
        image.reinterpret_stacked_2d_as_array(image.height() / image.width());
        image.texture_view_descriptor = Some(TextureViewDescriptor {
            dimension: Some(TextureViewDimension::Cube),
            ..default()
        });
    }

    for camera in cameras.iter() {
        commands.entity(camera).insert(Skybox {
            image: world.skybox.clone(),
            brightness: 800.0,
            ..default()
        });
    }
}

/// Run generate chunks and meshes load tasks
pub fn run_tasks(
    mut controller: ResMut<Controller>,
    world_assets: Res<Assets<WorldInfo>>,
    players: Query<Ref<Transform>, With<Player>>
) {
    let task_pool = AsyncComputeTaskPool::get();
    let Some(world) = world_assets.get(&controller.world) else { return };

    // Sort load-build queues
    if controller.need_sort {
        let current = RawChunk::global(players.single().translation);
        controller.load.sort_by(|a, b| {
            a.distance_squared(current)
                .cmp(&b.distance_squared(current)).reverse()
        });

        controller.build.sort_by(|a, b| {
            a.distance_squared(current)
                .cmp(&b.distance_squared(current)).reverse()
        });

        controller.need_sort = false;
    }

    // Chunks queue
    let l = MAX_TASKS - controller.load_tasks.len();
    for _ in 0..l {
        let Some(pos) = controller.load.pop() else {
            continue;
        };

        // Begin task
        controller.load_tasks.insert(
            pos,
            task_pool.spawn(RawChunk::generate(world.blocks.clone(), pos)),
        );
    }

    // Meshes queue
    let b = MAX_TASKS - controller.build_tasks.len();
    for _ in 0..b {
        let Some(pos) = controller.build.pop() else {
            continue;
        };

        // Get chunks refs and start task
        if let Some(refs) = controller.refs(pos) {
            controller.build_tasks.insert(
                pos,
                task_pool.spawn(ChunkMesh::build(world.blocks.clone(), refs)),
            );
        } else {
            controller.build.insert(pos);
        }
    }
}

pub fn prepare_world(
    controller: Res<Controller>,
    world_assets: Res<Assets<WorldInfo>>,
    mut global_material: ResMut<GlobalMaterial>,
    mut materials: ResMut<Assets<ChunkMaterial>>
) {
    if global_material.0.is_none() {
        let Some(world) = world_assets.get(&controller.world) else { return };

        global_material.0 = Some(materials.add(
            ChunkMaterial::new(&world.blocks)
        ))
    }
}

pub fn join_tasks(
    mut controller: ResMut<Controller>,
    mut commands: Commands,
    global_material: Res<GlobalMaterial>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if global_material.0.is_none() { return };

    let tasks: Vec<_> = controller.load_tasks.drain().collect();
    for (pos, task) in tasks {
        if !task.is_finished() { 
            controller.load_tasks.insert(pos, task);
            continue;
        }

        controller.chunks.insert(pos, Chunk::new(block_on(task)));
    }

    let tasks: Vec<_> = controller.build_tasks.drain().collect();
    for (pos, task) in tasks {
        if !task.is_finished() { 
            controller.build_tasks.insert(pos, task);
            continue;
        }

        // Remove previous mesh first
        if let Some(mesh) = controller.meshes.remove(&pos) {
            controller.despawn.push(mesh);
        }

        if let Some(mesh) = block_on(task) {
            let handler = meshes.add(mesh);
            let entity = commands
                .spawn((
                    Aabb::from_min_max(
                        Vec3::splat(-RawChunk::SIZE_F32 / 2.0),
                        Vec3::splat(RawChunk::SIZE_F32 * 1.5),
                    ),
                    Mesh3d(handler),
                    MeshMaterial3d(global_material.0.clone().unwrap()),
                    Transform::from_translation(pos.as_vec3() * Vec3::splat(RawChunk::SIZE_F32)),
                ))
                .id();

            controller.meshes.insert(pos, entity);
        }
    }
}

/// Unload meshes
pub fn unload(
    mut controller: ResMut<Controller>
) {
    let unload: Vec<_> = controller.unload.drain(..).collect();
    for pos in unload {
        controller.chunks.remove(&pos);
        if let Some(mesh) = controller.meshes.remove(&pos) {
            controller.despawn.push(mesh);
        }
    }
}

pub fn despawn(
    mut commands: Commands,
    mut controller: ResMut<Controller>
) {
    let despawn: Vec<_> = controller.despawn.drain(..).collect();
    for entity in despawn {
        commands.entity(entity).despawn();
    }
}

pub fn prepare_textures(
    controller: Res<Controller>,
    mut images: ResMut<Assets<Image>>,
    world_assets: Res<Assets<WorldInfo>>
) {
    let Some(world) = world_assets.get(&controller.world) else { return };
    for texture in world.blocks.meshable_textures() {
        let Some(handle) = texture else { continue };
        let Some(image) = images.get(&handle) else { continue };
        if image.texture_descriptor.array_layer_count() != 1 { continue; }

        // If image isn't proceeded yet - reinterpret
        let image = images.get_mut(&handle).unwrap();
        image.reinterpret_stacked_2d_as_array(6);
    }
}

pub fn hot_reload(
    mut global_material: ResMut<GlobalMaterial>,
    mut worlds_events: EventReader<AssetEvent<WorldInfo>>,
    mut controller: ResMut<Controller>,
    mut materials: ResMut<Assets<ChunkMaterial>>,
    mut images: EventReader<AssetEvent<Image>>,
    worlds: Res<Assets<WorldInfo>>,
) {
    for ev in worlds_events.read() {
        if ev.is_modified(&controller.world) {
            // Get updated world data
            let world = worlds.get(&controller.world).unwrap();

            // Recreate material
            global_material.0 = Some(materials.add(ChunkMaterial::new(&world.blocks)));

            controller.reload();
        }
    }

    if !images.is_empty() {
        controller.reload();
        images.clear();
    }
}


pub fn keybinding(
    mut evr_scroll: EventReader<bevy::input::mouse::MouseWheel>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut controller: ResMut<Controller>,
    players: Query<Ref<GlobalTransform>, With<Player>>,
    kbd: Res<ButtonInput<KeyCode>>,
    mut selected: ResMut<SelectedBlock>,
    world_assets: Res<Assets<WorldInfo>>,
    view_blocks: Res<ViewBlocks>,
) {
    let player = players.get_single().unwrap();
    let world = world_assets.get(&controller.world).unwrap();

    // Destroy block
    if mouse_buttons.just_pressed(MouseButton::Left) {
        if view_blocks.current.data != 0 {
            if let Some(chunk) = controller.chunks.get(&view_blocks.current.chunk) {
                let mut guard = chunk.write();
                guard.get_mut()[view_blocks.current.block] = 0;
            }
            controller.rebuild(view_blocks.current.chunk);
        }
    }

    // Place block
    if mouse_buttons.just_pressed(MouseButton::Right) {
        if view_blocks.current.data != 0 {
            if let Some(chunk) = controller.chunks.get(&view_blocks.previous.chunk) {
                let mut guard = chunk.write();
                guard.get_mut()[view_blocks.previous.block] = selected.0;
            }
            controller.rebuild(view_blocks.previous.chunk);
        }
    }

    // Switch current block
    for scroll in evr_scroll.read() {   
        let blocks = world.blocks.ids();

        if scroll.y.is_sign_positive() {
            if selected.0 == blocks[blocks.len() - 1] {
                selected.0 = 0;
                continue;
            }
            selected.0 += 1;
        } else {
            if selected.0 == 0 {
                selected.0 = blocks[blocks.len() - 1];
                continue;
            }
            selected.0 -= 1;
        }

        println!("Selected: {:?}", selected.0);
    }

    if mouse_buttons.just_pressed(MouseButton::Middle) {
        selected.0 = view_blocks.current.data;
    }

    if kbd.just_pressed(KeyCode::KeyF) {
        let current = player.translation();
        let u = player.forward().normalize();
        let blocks = RawChunk::under_cursor(current, u, 320);

        for block in blocks {
            let chunk_pos = RawChunk::global(block);
            let index = RawChunk::block_index(RawChunk::relative(block));
            if let Some(chunk) = controller.chunks.get(&chunk_pos) {
                let mut guard = chunk.write();
                guard.get_mut()[index] = selected.0;
            }
            controller.rebuild(chunk_pos);
        }
    }
}

pub fn controls(
    kbd: Res<ButtonInput<KeyCode>>,
    mut window_query: Query<Mut<Window>, With<PrimaryWindow>>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>
) {
    let Ok(mut window) = window_query.get_single_mut() else { return };

    // Grub mouse
    window.cursor_options = match state.eq(&AppState::Game) {
        true => CursorOptions {
            grab_mode: CursorGrabMode::Locked,
            visible: false,
            ..Default::default()
        },
        false => CursorOptions::default()
    };

    // Switch menu state
    if kbd.just_pressed(KeyCode::Escape) {
        next_state.set(match state.get() {
            AppState::Game => AppState::GameMenu,
            AppState::GameMenu => AppState::Game,
            _ => AppState::MainMenu 
        });
    }
}