use bevy::{
    prelude::*,
    render::primitives::Aabb, 
    tasks::*, 
    utils::*,
};

use std::f32::consts::PI;
use super::{
    blocks,
    chunks::*,
    camera::*,
    rendering::*,
    utils::{self, near_chunks, get_chunk_pos, CHUNK_SIZE, CHUNK_SIZE_P3, CHUNK_TASKS, MESH_TASKS}
};

/// Main blocks textures map
#[derive(Resource)]
pub struct Texture(pub Handle<Image>);

#[derive(Resource)]
pub struct WorldController {
    pub chunks: HashMap<IVec3, Chunk>,
    pub meshes: HashMap<IVec3, Entity>,

    // Load-unload queue
    pub load_chunks: HashSet<IVec3>,
    pub load_meshes: HashSet<IVec3>,
    pub unload: Vec<IVec3>,

    pub chunk_tasks: HashMap<IVec3, Task<RawChunk>>,
    pub mesh_tasks: HashMap<IVec3, Task<Option<ChunkMesh>>>
}

impl WorldController {
    pub fn chunk_refs(&self, pos: IVec3) -> Option<ChunksRefs> {
        let mut data = Vec::<Chunk>::with_capacity(7);
        for n in 0..7 {
            data.push(self.chunks.get(&(pos + utils::CHUNKS_OFFSETS[n])).cloned()?)
        }
        Some(ChunksRefs::new(utils::to_array(data)))
    }
}

impl Default for WorldController {
    fn default() -> Self {
        Self {
            chunks: HashMap::with_capacity(1024),
            meshes: HashMap::with_capacity(1024),
            load_chunks: HashSet::with_capacity(512),
            load_meshes: HashSet::with_capacity(512),
            unload: Vec::new(),
            chunk_tasks: HashMap::with_capacity(32),
            mesh_tasks: HashMap::with_capacity(32)
        }
    }
}

fn keybind(
    mut controller: ResMut<WorldController>,
    cameras: Query<Ref<Transform>, With<MainCamera>>,
    kbd: Res<ButtonInput<KeyCode>>,
) {
    let transform = cameras.get_single().unwrap();
    // tesg fill chunk function
    if kbd.just_pressed(KeyCode::KeyF) {
        let pos = utils::get_chunk_pos(transform.translation);
        if let Some(chunk) = controller.chunks.get(&pos) {
            let mut guard = chunk.write();
            let data = guard.get_mut();
            for i in 0..CHUNK_SIZE_P3 {
                data[i] = blocks::Block::Grass;
            }
        } else { return; }
        controller.load_meshes.extend(near_chunks(pos));
    }
}

/// Start generate chunks and meshes building;
fn prepare(
    mut controller: ResMut<WorldController>,
    cameras: Query<Ref<Transform>, With<MainCamera>>
) {
    let task_pool = ComputeTaskPool::get();

    // Sort chunks
    let camera = cameras.get_single().unwrap();
    let camera_chunk = get_chunk_pos(camera.translation);
    
    // Start chunks generate tasks
    let mut data = controller.load_chunks.iter().copied().collect::<Vec<_>>();
    data.sort_by(|a, b| 
        a.distance_squared(camera_chunk).cmp(&b.distance_squared(camera_chunk)));

    for pos in data {
        if controller.chunk_tasks.len() >= CHUNK_TASKS {
            continue;
        }

        controller.load_chunks.remove(&pos);
        let task = task_pool.spawn(RawChunk::generate(pos));
        controller.chunk_tasks.insert(pos, task);
    }

    // Start meshes build tasks
    let mut data = controller.load_meshes.iter().copied().collect::<Vec<_>>();
    data.sort_by(|a, b| 
        a.distance_squared(camera_chunk).cmp(&b.distance_squared(camera_chunk)));

    for pos in data {
        if controller.mesh_tasks.len() >= MESH_TASKS {
            continue;
        }

        controller.load_meshes.remove(&pos);
        if let Some(chunk) = controller.chunk_refs(pos) {
            let task = task_pool.spawn(ChunkMesh::build(chunk));
            controller.mesh_tasks.insert(pos, task);
        } else {
            controller.load_meshes.insert(pos);
        }
    }
}

fn unload(
    mut commands: Commands,
    mut controller: ResMut<WorldController>
) {
    let data = controller.unload.drain(..).collect::<Vec<_>>();
    for pos in data {
        controller.chunks.remove(&pos);
        controller.chunk_tasks.remove(&pos);
        if let Some(mesh) = controller.meshes.remove(&pos) {
            commands.entity(mesh).despawn();
        }
    }
}

/// Join finished tasks;
fn finish(
    mut commands: Commands,
    texture: Res<Texture>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut controller: ResMut<WorldController>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Debug (todo: change to bevy diag)
    if controller.chunk_tasks.len() != 0 {
        println!("Load chunks: {}; Tasks: {};", controller.load_chunks.len(), controller.chunk_tasks.len());
    }

    if controller.mesh_tasks.len() != 0 {
        println!("Load meshes: {}; Tasks: {};", controller.load_meshes.len(), controller.mesh_tasks.len());
    }

    // Join all chunk generate (load) tasks
    let data = controller.chunk_tasks.drain().collect::<Vec<_>>();
    for (pos, task) in data {
        if task.is_finished() {
            if let Some(raw) = block_on(poll_once(task)) {
                controller.chunks.insert(pos, Chunk::new(raw));
            }
        } else {
            controller.chunk_tasks.insert(pos, task);
        }
    }

    // Join all mesh build tasks
    let data = controller.mesh_tasks.drain().collect::<Vec<_>>();
    for (pos, task) in data {
        if task.is_finished() {
            if let Some(chunk_mesh) = block_on(task) {
                let mesh = chunk_mesh.spawn();
                let handle = meshes.add(mesh);
                
                //todo: custom shader
                let entity = commands.spawn((
                    Aabb::from_min_max(Vec3::ZERO, Vec3::splat(CHUNK_SIZE as f32)),
                    Mesh3d(handle),
                    Transform::from_translation(pos.as_vec3() * Vec3::splat(CHUNK_SIZE as f32)),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color_texture: Some(texture.0.clone()),
                        ..default()
                    })),
                )).id();
                if let Some(old) = controller.meshes.insert(pos, entity) {
                    commands.entity(old).despawn();
                }
            }
        } else {
            controller.mesh_tasks.insert(pos, task);
        }
    }
}

// Simple hot-reloading
pub fn handle_events (
    mut controller: ResMut<WorldController>,
    mut images: EventReader<AssetEvent<Image>>,
) {
    if !images.is_empty() {
        let data = controller.meshes.keys().copied().collect::<Vec<_>>();
        controller.load_meshes.extend(data);
    }
}

pub fn setup(mut commands: Commands) {
    commands.spawn((
        MainCamera::new(RenderDistance::new(8, 4)),
        Camera3d::default(),
        Transform::from_xyz(0.0, 42.0, 0.0)
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 2000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            PI / 2.,
            -PI / 3.,
        )),
    ));
    
}

pub struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldController>()
            .add_plugins(CameraPlugin)
            .add_systems(Startup, setup)
            .add_systems(Update, (prepare, keybind, handle_events))
            .add_systems(Last, (unload, finish).chain());
    }
}