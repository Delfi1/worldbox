use bevy::{
    prelude::*,
    render::primitives::Aabb, 
    tasks::*, 
    utils::*,
};

use super::{
    chunks::*,
    camera::*,
    rendering::*,
    utils::{self, CHUNK_SIZE, CHUNK_TASKS, MESH_TASKS}
};

/// Main blocks textures map
#[derive(Resource)]
pub struct Texture(pub Handle<Image>);

#[derive(Resource)]
pub struct WorldController {
    pub chunks: HashMap<IVec3, Chunk>,
    pub load_chunks: Vec<IVec3>,
    pub unload_chunks: Vec<IVec3>,
    pub chunk_tasks: HashMap<IVec3, Task<RawChunk>>,

    pub meshes: HashMap<IVec3, Entity>,
    pub load_meshes: Vec<IVec3>,
    pub unload_meshes: Vec<IVec3>,
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
        let n: i32 = 4;
        let mut load_chunks = Vec::with_capacity(((n as usize)*2).pow(3));
        for x in -n..n {
            for y in -n..n {
                for z in -n..n {
                    load_chunks.push(IVec3::new(x, y, z));
                }
            }
        }

        let k = n - 1;
        let mut load_meshes = Vec::with_capacity(((k as usize)*2).pow(3));
        for x in -k..k {
            for y in -k..k {
                for z in -k..k {
                    load_meshes.push(IVec3::new(x, y, z));
                }
            }
        }

        Self {
            chunks: HashMap::with_capacity(1024),
            //load_chunks: Vec::with_capacity(512),
            load_chunks,
            unload_chunks: Vec::new(),
            chunk_tasks: HashMap::with_capacity(32),
            meshes: HashMap::with_capacity(1024),
            //load_meshes: Vec::with_capacity(512),
            load_meshes,
            unload_meshes: Vec::new(),
            mesh_tasks: HashMap::with_capacity(32)
        }
    }
}

/// Start generate chunks and meshes building;
fn prepare(mut controller: ResMut<WorldController>) {
    let task_pool = ComputeTaskPool::get();

    // Sort chunks and meshes
    // todo: change ZERO to "Camera Chunk"
    controller.load_chunks.sort_by(|a, b| 
        a.distance_squared(IVec3::ZERO).cmp(&b.distance_squared(IVec3::ZERO)));
    controller.load_meshes.sort_by(|a, b| 
            a.distance_squared(IVec3::ZERO).cmp(&b.distance_squared(IVec3::ZERO)));

    // Start chunks generate tasks
    let data = controller.load_chunks.drain(..).collect::<Vec<_>>();
    for pos in data {
        if controller.chunk_tasks.len() >= CHUNK_TASKS {
            controller.load_chunks.push(pos);
            continue;
        } 
        
        let task = task_pool.spawn(RawChunk::generate(pos));
        controller.chunk_tasks.insert(pos, task);
    }

    // Start meshes build tasks
    let data = controller.load_meshes.drain(..).collect::<Vec<_>>();
    for pos in data {
        if controller.mesh_tasks.len() >= MESH_TASKS {
            controller.load_meshes.push(pos);
            continue;
        }

        if let Some(chunk) = controller.chunk_refs(pos) {
            let task = task_pool.spawn(ChunkMesh::build(chunk));
            controller.mesh_tasks.insert(pos, task);
        } else {
            controller.load_meshes.push(pos);
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
            if let Some(Some(chunk_mesh)) = block_on(poll_once(task)) {
                let mesh = chunk_mesh.spawn();
                let handle = meshes.add(mesh);

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

pub fn setup(mut commands: Commands) {
    commands.spawn((
        MainCamera::new(RenderDistance::new(8, 4)),
        Camera3d::default(),
        Transform::from_xyz(0.0, 42.0, 0.0)
    ));
}

pub struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldController>()
            .add_plugins(CameraPlugin)
            .add_systems(Startup, setup)
            .add_systems(Update, prepare)
            .add_systems(Last, finish);
    }
}   