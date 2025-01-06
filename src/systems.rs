use bevy::{
    prelude::*,
    core_pipeline::Skybox,
    render::{
        primitives::*,
        render_resource::*
    },
    tasks::*
};

#[derive(Resource)]
pub struct SkyBoxHandler(pub Handle<Image>);

use super::*;
pub fn setup(
    assets: Res<AssetServer>,
    mut commands: Commands
) {
    commands.spawn((
        DirectionalLight {
            illuminance: 4000.0,
            ..default()
        },
        
        Transform::from_rotation(Quat::from_euler(
        EulerRot::XYZ,
            -3.14/2.5,
            0.0,
            0.0
        ))
    ));

    commands.insert_resource(AmbientLight {
        color: Color::srgb_u8(210, 220, 240),
        brightness: 1.0,
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
    camera_query: Query<Ref<Transform>, With<Camera3d>>
) {
    let task_pool = ComputeTaskPool::get();
    let camera_chunk = RawChunk::global(camera_query.single().translation);

    if controller.load.len() != 0 || controller.build.len() != 0 {
        println!("Load: {}; Build: {};", controller.load.len(), controller.build.len());
    }

    // Sort chunks & meshes
    controller.load.sort_by(|a, b| 
        a.distance_squared(camera_chunk).cmp(&b.distance_squared(camera_chunk)));
    controller.build.sort_by(|a, b| 
        a.distance_squared(camera_chunk).cmp(&b.distance_squared(camera_chunk)));

    // chunks queue
    let l = (MAX_TASKS - controller.load_tasks.len()).min(controller.load.len());
    let data: Vec<_> = controller.load.drain(0..l).collect();
    for pos in data {
        controller.load_tasks.insert(pos, task_pool.spawn(RawChunk::generate(pos)));
    }

    // meshes queue
    let b = (MAX_TASKS - controller.build_tasks.len()).min(controller.build.len());
    let data: Vec<_> = controller.build.drain(0..b).collect();
    for pos in data {
        if let Some(refs) = controller.refs(pos) {
            controller.build_tasks.insert(pos, task_pool.spawn(ChunkMesh::build(refs)));
        } else {
            controller.build.push(pos);
        }
    }
}

pub fn join(
    mut commands: Commands,
    mut controller: ResMut<Controller>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ChunkMaterial>>,
    global_texture: ResMut<GlobalTexture>
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
                MeshMaterial3d(materials.add(ChunkMaterial::new(global_texture.clone()))),
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
    mut commands: Commands,
    mut images: EventReader<AssetEvent<Image>>,
) {
    if !images.is_empty() {
        for (pos, entity) in controller.meshes.drain().collect::<Vec<_>>() {
            commands.entity(entity).despawn();
            controller.build.push(pos);
        }

        images.clear();
    }
}