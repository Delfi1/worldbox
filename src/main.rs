mod chunk;
mod mesher;
mod rendering;
mod systems;
mod camera;

use bevy::{
    prelude::*,
    tasks::*, 
    utils::*
};

use camera::*;
use mesher::*;
use chunk::*;
use rendering::*;
use hashbrown::HashSet;

#[derive(Resource)]
pub struct Controller {
    chunks: HashMap<IVec3, chunk::Chunk>,
    meshes: HashMap<IVec3, Entity>,
    /// load chunks queue; build meshes queue
    load: HashSet<IVec3>,
    build: HashSet<IVec3>,

    /// unload queue
    unload: HashSet<IVec3>,

    /// Compute tasks
    load_tasks: HashMap<IVec3, Task<RawChunk>>,
    build_tasks: HashMap<IVec3, Task<Option<ChunkMesh>>>,
}

impl Default for Controller {
    fn default() -> Self {
        let n = 4;
        let k = n-1;

        let mut load = HashSet::new();
        for x in -n..n {
            for y in -n..n {
                for z in -n..n {
                    load.insert(IVec3::new(x, y, z));
                }
            }
        }

        let mut build = HashSet::new();
        for x in -k..k {
            for y in -k..k {
                for z in -k..k {
                    build.insert(IVec3::new(x, y, z));
                }
            }
        }

        Self {
            chunks: HashMap::with_capacity(1024),
            meshes: HashMap::with_capacity(1024),
            // Load-rebuild chunks 
            //load: HashSet::with_capacity(1024),
            //build: HashSet::with_capacity(1024),
            load, build,
            
            unload: HashSet::with_capacity(512),

            load_tasks: HashMap::new(),
            build_tasks: HashMap::new(),
        }
    }
}

impl Controller {
    fn to_array<T: std::fmt::Debug, const N: usize>(data: Vec<T>) -> [T; N] {
        data.try_into().expect("Wrong size")
    }
    
    // Get chunk refs
    pub fn refs(&self, pos: IVec3) -> Option<ChunksRefs> {
        let mut data = Vec::<Chunk>::with_capacity(7);
        for n in 0..7 {
            data.push(self.chunks.get(&(pos + ChunksRefs::OFFSETS[n])).cloned()?)
        }
        Some(ChunksRefs::new(Self::to_array(data)))
    }
}

/// Main game plugin
pub struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Controller>()
            .add_plugins((CameraPlugin, RenderingPlugin))
            .add_systems(Startup, systems::setup)
            .add_systems(PostUpdate, systems::begin)
            .add_systems(Last, systems::join);
    }
}


pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldPlugin)
        .run();
}