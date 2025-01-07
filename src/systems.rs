use std::thread::current;

use bevy::{
    core_pipeline::Skybox, 
    prelude::*,
    render::{
        primitives::*,
        render_resource::*
    },
    tasks::*, 
    window::*
};

#[derive(Resource)]
pub struct SkyBoxHandler(pub Handle<Image>);

use super::*;
pub fn setup(
    assets: Res<AssetServer>,
    mut windows: Query<Mut<Window>, With<PrimaryWindow>>,
    config: Res<MainConfig>,
    mut commands: Commands
) {
    commands.spawn((
        DirectionalLight {
            illuminance: config.illuminance,
            //shadows_enabled: true,
            ..default()
        },
        
        Transform::from_rotation(Quat::from_euler(
        EulerRot::XYZ,
            config.light_direct.x,
            config.light_direct.y,
            config.light_direct.z,
        ))
    ));

    for mut window in windows.iter_mut() {
        if config.vsync { window.present_mode = PresentMode::AutoVsync }
        else {window.present_mode = PresentMode::AutoNoVsync }
    }

    commands.insert_resource(BlocksHandler::new(&assets, config.blocks.clone()));
    commands.insert_resource(AmbientLight {
        color: Color::Srgba(config.ambient_color),
        brightness: config.ambient_brightness,
        ..default()
    });

    commands.insert_resource(SkyBoxHandler(assets.load("skybox.png")));
    commands.spawn((
        Camera3d::default(),
        MainCamera::new(),
        Frustum::default(),
        Transform::from_xyz(0.0, 32.0, 0.0)
    ));
}

pub fn skybox(
    mut commands: Commands,
    cameras: Query<Entity, With<Camera3d>>,
    assets: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    handler: Res<SkyBoxHandler>
) {
    if !assets.load_state(&handler.0).is_loaded() {return};
    let image = images.get(&handler.0).unwrap();
    if image.texture_descriptor.array_layer_count() != 1 {return;}
    
    let image = images.get_mut(&handler.0).unwrap();
    image.reinterpret_stacked_2d_as_array(image.height() / image.width());
    image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..default()
    });

    for camera in cameras.iter() {
        commands.entity(camera).insert(
            Skybox {
                image: handler.0.clone(),
                brightness: 1000.0,
                ..default()
            }
        );
    }
}

/// Max thread tasks;
pub const MAX_TASKS: usize = 4;
// Begin tasks
pub fn begin(
    mut controller: ResMut<Controller>,
    camera_query: Query<Ref<Transform>, With<Camera3d>>,
    handler: Res<BlocksHandler>
) {
    let task_pool = ComputeTaskPool::get();
    let camera_chunk = RawChunk::global(camera_query.single().translation);

    if controller.load.len() != 0 || controller.build.len() != 0 {
        println!("Load: {}; Build: {};", controller.load.len(), controller.build.len());
    }

    // chunks queue
    let l = (MAX_TASKS - controller.load_tasks.len()).min(controller.load.len());
    let mut to_remove = Vec::new();
    for i in 0..l {
        let pos = controller.load[i];
        controller.load_tasks.insert(pos, task_pool.spawn(RawChunk::generate(handler.clone(), pos)));
        to_remove.push(pos);
    }
    for pos in to_remove { controller.load.remove(&pos); }

    // meshes queue
    let b = (MAX_TASKS - controller.build_tasks.len()).min(controller.build.len());
    
    let mut to_remove = Vec::new();
    for i in 0..b {
        let pos = controller.build[i];
        if let Some(refs) = controller.refs(pos) {
            controller.build_tasks.insert(pos, task_pool.spawn(ChunkMesh::build(handler.clone(), refs)));
            to_remove.push(pos);
        }
    }
    
    for pos in to_remove { controller.build.remove(&pos); }
}

pub fn join(
    mut commands: Commands,
    mut controller: ResMut<Controller>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ChunkMaterial>>,
    blocks: Res<BlocksHandler>,
) {
    // join chunks; run mesh builder
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
        
        if let Some(mesh) = block_on(task) {
            let handler = meshes.add(mesh);
            let entity = commands.spawn((
                Aabb::from_min_max(Vec3::ZERO, Vec3::splat(RawChunk::SIZE_F32)),
                Mesh3d(handler),
                MeshMaterial3d(materials.add(ChunkMaterial::new(&blocks))),
                Transform::from_translation(pos.as_vec3() * Vec3::splat(RawChunk::SIZE_F32))
            )).id();

            if let Some(old) = controller.meshes.insert(pos, entity) {
                commands.entity(old).despawn();
            };
        }
    }
}

pub fn hot_reload(
    mut controller: ResMut<Controller>,
    cameras: Query<Ref<Transform>, With<Camera3d>>,
    mut commands: Commands,
    mut images: EventReader<AssetEvent<Image>>,
) {
    if !images.is_empty() {
        for (pos, entity) in controller.meshes.drain().collect::<Vec<_>>() {
            commands.entity(entity).despawn();
            controller.build.insert(pos);
        }
        let current = RawChunk::global(cameras.single().translation);
        controller.sort(current);

        images.clear();
    }
}