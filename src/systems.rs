use super::*;
use bevy::{
    core_pipeline::*, prelude::*, render::{primitives::*, render_resource::*}, tasks::*, window::*
};

/// Setup application
pub fn setup(mut windows: Query<Mut<Window>, With<PrimaryWindow>>) {
    for mut window in windows.iter_mut() {
        window.title = "WorldBox".into();
        window.present_mode = PresentMode::AutoVsync;
    }
}

// On world load system
pub fn load_world(assets: Res<AssetServer>, mut commands: Commands, mut world: ResMut<WorldRes>) {
    commands.insert_resource(Controller::default());
    commands.insert_resource(ViewBlocks::empty());
    commands.insert_resource(SelectedBlock(0));

    commands.insert_resource(AmbientLight {
        color: Color::Srgba(Srgba::rgb_u8(210, 220, 240)),
        brightness: 500.0,
        ..default()
    });

    let light = commands
        .spawn((
            DirectionalLight {
                illuminance: 1200.0,
                ..default()
            },
            Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -3.14 / 2.8, 0.0, 0.0)),
        ))
        .id();

    world.entities.push(light);

    let camera = commands
        .spawn((
            Camera3d::default(),
            MainCamera::new(),
            LoadArea::new(16, 6),
            Frustum::default(),
            Transform::from_xyz(16.0, 36.0, 16.0),
        ))
        .id();

    world.entities.push(camera);
}

pub fn skybox(
    mut commands: Commands,
    cameras: Query<Entity, With<Camera3d>>,
    mut images: ResMut<Assets<Image>>,
    world: Res<WorldRes>
) {
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

/// Max thread tasks;
pub const MAX_TASKS: usize = 4;

// Begin tasks
pub fn begin(
    mut controller: ResMut<Controller>,
    cameras: Query<Ref<Transform>, With<Camera3d>>,
    world: Res<WorldRes>,
) {
    let task_pool = AsyncComputeTaskPool::get();

    // Sort load-build queues
    if controller.need_sort {
        let current = RawChunk::global(cameras.single().translation);
        controller.load.sort_by(|a, b| {
            a.distance_squared(current)
                .cmp(&b.distance_squared(current))
        });
        controller.build.sort_by(|a, b| {
            a.distance_squared(current)
                .cmp(&b.distance_squared(current))
        });

        controller.need_sort = false;
    }

    // Chunks queue
    let l = (MAX_TASKS - controller.load_tasks.len()).min(controller.load.len());

    for i in 0..l {
        let Some(pos) = controller.load.get_index(i).cloned() else {
            continue;
        };
        // Remove pos
        controller.load.remove(&pos);

        // Begin task
        controller.load_tasks.insert(
            pos,
            task_pool.spawn(RawChunk::load(world.clone(), pos)),
        );
    }

    // Meshes queue
    let b = (MAX_TASKS - controller.build_tasks.len()).min(controller.build.len());

    for i in 0..b {
        let Some(pos) = controller.build.get_index(i).cloned() else {
            continue;
        };
        if let Some(refs) = controller.refs(pos) {
            // Clear queue
            controller.build.remove(&pos);

            // Create mesh build task
            controller.build_tasks.insert(
                pos,
                task_pool.spawn(ChunkMesh::build(world.blocks.clone(), refs)),
            );
        }
    }
}

pub fn unload(mut controller: ResMut<Controller>, world: Res<WorldRes>, mut commands: Commands) {
    let data: Vec<_> = controller.unload.drain(..).collect();
    for pos in data {
        if let Some(chunk) = controller.chunks.remove(&pos) {
            if chunk.is_modified() {
                let stored = StoredChunk::new(world.blocks.clone(), chunk);

                stored.store(&world.name, pos);
            }
        }

        if let Some(mesh) = controller.meshes.remove(&pos) {
            controller.despawn.push(mesh);
        };
    }

    for entity in controller.despawn.drain(..) {
        commands.entity(entity).despawn();
    }
}

pub fn join(
    mut commands: Commands,
    mut controller: ResMut<Controller>,
    mut meshes: ResMut<Assets<Mesh>>,
    world: Res<WorldRes>,
) {
    // join chunks;
    let data: Vec<_> = controller.load_tasks.drain().collect();
    for (pos, task) in data {
        if !task.is_finished() {
            controller.load_tasks.insert(pos, task);
            continue;
        }

        let raw = block_on(task);
        controller.chunks.insert(pos, Chunk::new(raw));
    }

    // join meshes
    let data: Vec<_> = controller.build_tasks.drain().collect();
    for (pos, task) in data {
        if !task.is_finished() {
            controller.build_tasks.insert(pos, task);
            continue;
        }

        // Remove current mesh first chunks
        if let Some(old) = controller.meshes.remove(&pos) {
            controller.despawn.push(old);
        };

        // Spawn new mesh
        if let Some(mesh) = block_on(task) {
            let handler = meshes.add(mesh);
            let entity = commands
                .spawn((
                    Aabb::from_min_max(
                        Vec3::splat(-RawChunk::SIZE_F32 / 2.0),
                        Vec3::splat(RawChunk::SIZE_F32 * 1.5),
                    ),
                    Mesh3d(handler),
                    MeshMaterial3d(world.main_material.clone()),
                    Transform::from_translation(pos.as_vec3() * Vec3::splat(RawChunk::SIZE_F32)),
                ))
                .id();

            controller.meshes.insert(pos, entity);
        }
    }
}

pub fn hot_reload(
    mut controller: ResMut<Controller>,
    mut images: EventReader<AssetEvent<Image>>,
    mut worlds_events: EventReader<AssetEvent<WorldData>>,

    assets: Res<AssetServer>,
    worlds: Res<Assets<WorldData>>,
    mut materials: ResMut<Assets<ChunkMaterial>>,
    mut world: ResMut<WorldRes>,
) {
    for ev in worlds_events.read() {
        if ev.is_modified(&world.handler) {
            // Get updated world data
            let data = worlds.get(&world.handler).unwrap();
            let blocks = BlocksHandler::new(&assets, data.blocks.clone());
            // Recreate material
            let material = materials.add(ChunkMaterial::new(&blocks));

            world.skybox = assets.load("skybox.png");
            world.blocks = blocks;
            world.main_material = material;
            controller.reload();
        }
    }

    if !images.is_empty() {
        controller.reload();
        images.clear();
    }
}

struct ViewBlockData {
    chunk: IVec3,
    block: usize,
    data: u16,
}

impl ViewBlockData {
    pub fn empty() -> Self {
        Self {
            chunk: IVec3::ZERO,
            block: 0,
            data: 0,
        }
    }

    pub fn set(&mut self, chunk: IVec3, block: usize, data: u16) {
        (self.chunk, self.block, self.data) = (chunk, block, data);
    }
}

#[derive(Resource)]
pub struct ViewBlocks {
    previous: ViewBlockData,
    current: ViewBlockData,
}

impl ViewBlocks {
    pub fn empty() -> Self {
        Self {
            previous: ViewBlockData::empty(),
            current: ViewBlockData::empty(),
        }
    }

    pub fn reset(&mut self) {
        *self = Self::empty();
    }
}

pub fn update_view_blocks(
    cameras: Query<Ref<GlobalTransform>, With<Camera3d>>,
    controller: Res<Controller>,
    mut view_blocks: ResMut<ViewBlocks>,
) {
    let camera = cameras.single();
    let current = camera.translation();
    let u = camera.forward().normalize();
    let blocks = RawChunk::under_cursor(current, u, 32);

    // Reset view_blocks blocks
    view_blocks.reset();
    for block in blocks {
        let chunk_pos = RawChunk::global(block);
        let index = RawChunk::block_index(RawChunk::relative(block));

        if let Some(chunk) = controller.chunks.get(&chunk_pos).cloned() {
            let guard = chunk.read();
            let data = guard.get()[index];
            if data == 0 {
                view_blocks.previous.set(chunk_pos, index, data);
            } else {
                view_blocks.current.set(chunk_pos, index, data);
                break;
            }
        }
    }
}

/// Current block.
#[derive(Debug, Resource)]
pub struct SelectedBlock(u16);

pub fn keybind(
    mut controller: ResMut<Controller>,
    kbd: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut primary_window: Query<Mut<Window>, With<PrimaryWindow>>,
    mut evr_scroll: EventReader<bevy::input::mouse::MouseWheel>,
    cameras: Query<Ref<GlobalTransform>, With<Camera3d>>,
    view_blocks: Res<ViewBlocks>,
    mut next_state: ResMut<NextState<MainState>>,
    mut selected: ResMut<SelectedBlock>,
    world: Res<WorldRes>,
) {
    if let Some(mut window) = primary_window.get_single_mut().ok() {
        window.cursor_options.visible = false;
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
    }

    let camera = cameras.single();

    if kbd.just_pressed(KeyCode::KeyR) {
        controller.reload();
    }

    if kbd.just_pressed(KeyCode::Escape) {
        next_state.set(MainState::GameMenu);
    }

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
        let blocks = world.blocks.all();

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

    if kbd.just_pressed(KeyCode::KeyF) {
        let current = camera.translation();
        let u = camera.forward().normalize();
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
