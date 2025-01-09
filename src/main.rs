mod chunk;
mod mesher;
mod rendering;
mod systems;
mod camera;
mod fps;

use bevy::{
    prelude::*,
    tasks::*, 
    image::*,
    utils::*
};

use camera::*;
use mesher::*;
use chunk::*;
use rendering::*;
use fps::*;
use serde::{Serialize, Deserialize};
use ordermap::OrderSet;

// Todo:
// 1) World load-store system
// 2) Fix meshing system
// 3) Update Blocks Data (Add models, collisions, tags etc)
// 4) Player collisions with blocks
// 5) Add normal maps option for textures

/// Config Data
#[derive(Resource)]
#[derive(Serialize, Deserialize)]
pub struct MainConfig {
    pub vsync: bool,
    /// World ambient color
    pub ambient_color: Srgba,
    pub blocks: Blocks
}

impl Default for MainConfig {
    fn default() -> Self {
        Self {
            vsync: true,
            ambient_color: Srgba::rgb_u8(210, 220, 240),
            blocks: Blocks::default()
        }
    }
}

impl MainConfig {
    /// Load or init Config data
    pub fn load_init() -> Self {
        Self::load().unwrap_or(Self::default())
    }

    pub fn load() -> Option<Self> {
        std::fs::read_to_string("./assets/config.yaml").ok()
            .and_then(|s| serde_yaml::from_str(&s).ok())
    }

    pub fn save(&self) {
        if let Ok(data) = serde_yaml::to_string(&self) {
            let _ = std::fs::write("./assets/config.yaml", data);
        }
    }
}

#[derive(Resource)]
/// Main stored world chunks data
pub struct Controller {
    pub chunks: HashMap<IVec3, chunk::Chunk>,
    pub meshes: HashMap<IVec3, Entity>,
    /// load chunks queue; build meshes queue
    pub load: OrderSet<IVec3>,
    pub build: OrderSet<IVec3>,

    /// unload and despawn queue
    pub unload: Vec<IVec3>,
    pub despawn: Vec<Entity>,

    /// Compute tasks
    load_tasks: HashMap<IVec3, Task<RawChunk>>,
    build_tasks: HashMap<IVec3, Task<Option<Mesh>>>,
    need_sort: bool
}

impl Default for Controller {
    fn default() -> Self {
        let n = 8;
        let k = n-1;

        // Test generate chunks area
        let mut load: OrderSet<IVec3> = OrderSet::new();
        for x in -n..n {
            for y in -n..n {
                for z in -n..n {
                    load.insert(IVec3::new(x, y, z));
                }
            }
        }

        let mut build = OrderSet::new();
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
            
            unload: Vec::with_capacity(512),
            despawn: Vec::with_capacity(512),

            load_tasks: HashMap::new(),
            build_tasks: HashMap::new(),
            need_sort: true
        }
    }
}

impl Controller {
    fn to_array<T: std::fmt::Debug, const N: usize>(data: Vec<T>) -> [T; N] {
        data.try_into().expect("Wrong size")
    }
    
    // sort load and build queues
    pub fn sort(&mut self) {
        self.need_sort = true;
    }

    /// Reload all meshes & sort
    pub fn reload(&mut self) {
        self.build.extend(self.meshes.keys().copied());
        self.sort();
    }

    // Rebuild chunk meshes
    pub fn rebuild(&mut self, chunk: IVec3) {
        self.build.extend(ChunksRefs::offsets(chunk));
        self.sort();
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
            .insert_resource(MainConfig::load_init())
            .add_plugins((FpsPlugin, CameraPlugin, RenderingPlugin))
            .add_systems(Startup, systems::setup)
            .add_systems(Update, systems::keybind)
            .add_systems(FixedUpdate, systems::skybox)
            .add_systems(FixedPostUpdate, systems::update_selected)
            .add_systems(PostUpdate, (systems::hot_reload, systems::begin).chain())
            .add_systems(Last, (systems::unload, systems::join).chain());
    }

    fn cleanup(&self, app: &mut App) {
        // Save config data
        app.add_systems(PostStartup, |r: Res<MainConfig>| {
            r.save();
        });
    }
}

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(
            ImagePlugin {
                default_sampler: ImageSamplerDescriptor {
                    address_mode_u: ImageAddressMode::Repeat,
                    address_mode_v: ImageAddressMode::Repeat,
                    mag_filter: ImageFilterMode::Nearest,
                    min_filter: ImageFilterMode::Linear,
                    mipmap_filter: ImageFilterMode::Linear,
                    ..default()
                }
            }
        ))
        .add_plugins(WorldPlugin)
        .run();
}