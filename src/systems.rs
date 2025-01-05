use bevy::{
    prelude::*, render::primitives::*, tasks::*
};

use super::*;

pub fn setup(
    mut commands: Commands,
) {
    commands.spawn((
        DirectionalLight {
            illuminance: 800.0,
            ..default()
        },
        
        Transform::from_rotation(Quat::from_euler(
        EulerRot::XYZ,
            -3.14/2.5,
            0.0,
            0.0
        ))
    ));
    commands.spawn((
        Camera3d::default(),
        MainCamera::new(),
        Frustum::default(),
        Transform::from_xyz(0.0, 32.0, 0.0)
    ));
}

/// Max thread tasks;
pub const MAX_TASKS: usize = 2;
// Begin tasks
pub fn begin(mut controller: ResMut<Controller>) {
    let task_pool = ComputeTaskPool::get();

    if controller.load.len() != 0 {
        println!("Load: {}", controller.load.len())
    }
    if controller.build.len() != 0 {
        println!("Build: {};", controller.build.len())
    }

    // chunks queue
    let mut data = controller.load.iter().copied().collect::<Vec<_>>();
    data.sort_by(|a, b| 
        a.distance_squared(IVec3::ZERO).cmp(&b.distance_squared(IVec3::ZERO)));
    for pos in data {
        if controller.load_tasks.len() < MAX_TASKS {
            controller.load_tasks.insert(pos, task_pool.spawn(RawChunk::generate(pos)));
            controller.load.remove(&pos);
        }
    }

    // meshes queue
    let mut data: Vec<_> = controller.build.iter().copied().collect();
    data.sort_by(|a, b| 
        a.distance_squared(IVec3::ZERO).cmp(&b.distance_squared(IVec3::ZERO)));
    for pos in data {
        if controller.build_tasks.len() < MAX_TASKS {
            if let Some(refs) = controller.refs(pos) {
                controller.build_tasks.insert(pos, task_pool.spawn(ChunkMesh::build(refs)));
                controller.build.remove(&pos);
            }
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
            let handler = meshes.add(mesh.spawn());
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