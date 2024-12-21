use bevy::{
    prelude::*,
    utils::*,
    tasks::*
};

use super::{
    chunks::*,
    rendering::*,
    utils::{self, CHUNK_TASKS, MESH_TASKS}
};


#[derive(Resource)]
pub struct WorldController {
    chunks: HashMap<IVec3, Chunk>,
    load_chunks: Vec<IVec3>,
    chunk_tasks: HashMap<IVec3, Task<RawChunk>>,

    meshes: HashMap<IVec3, ChunkMesh>,
    load_meshes: Vec<IVec3>,
    mesh_tasks: HashMap<IVec3, Task<ChunkMesh>>
}

impl WorldController {
    pub fn chunk_refs(&self, pos: IVec3) -> Option<ChunksRefs> {
        let mut data = Vec::<Chunk>::with_capacity(7);
        for n in 0..7 {
            data.push(self.chunks.get(&(pos + utils::CHUNKS_OFFSETS[n])).cloned()?)
        }
        ChunksRefs::new(data)
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
            chunk_tasks: HashMap::with_capacity(32),
            meshes: HashMap::with_capacity(1024),
            //load_meshes: Vec::with_capacity(512),
            load_meshes,
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

    let data = controller.load_chunks.drain(..).collect::<Vec<_>>();
    for pos in data {
        if controller.chunk_tasks.len() >= CHUNK_TASKS {
            controller.load_chunks.push(pos);
            continue;
        } 
        
        let task = task_pool.spawn(RawChunk::generate());
        controller.chunk_tasks.insert(pos, task);
    }

    let data = controller.load_meshes.drain(..).collect::<Vec<_>>();
    for pos in data {
        if controller.mesh_tasks.len() >= MESH_TASKS {
            controller.load_meshes.push(pos);
            continue;
        }

        if let Some(chunk) = controller.chunk_refs(pos) {
            let task = task_pool.spawn(ChunkMesh::new(chunk));
            controller.mesh_tasks.insert(pos, task);
        } else {
            controller.load_meshes.push(pos);
        }
    }
}

/// Join finished tasks;
fn finish(mut controller: ResMut<WorldController>) {
    if controller.load_chunks.len() != 0 {
        println!("Load chunks: {}; Tasks: {};", controller.load_chunks.len(), controller.chunk_tasks.len());
    }

    if controller.load_meshes.len() != 0 {
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
    let data: Vec<(IVec3, Task<ChunkMesh>)> = controller.mesh_tasks.drain().collect::<Vec<_>>();
    for (pos, task) in data {
        if task.is_finished() {
            if let Some(mesh) = block_on(poll_once(task)) {
                controller.meshes.insert(pos, mesh);
            }
        } else {
            controller.mesh_tasks.insert(pos, task);
        }
    }
}

pub struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldController>()
            .add_systems(Update, prepare)
            .add_systems(Last, finish);
    }
}   