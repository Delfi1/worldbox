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

// todo: fix render textures outline bug;

/// Config Data
#[derive(Resource)]
#[derive(Serialize, Deserialize)]
pub struct MainConfig {
    pub vsync: bool,
    pub light_direct: Vec3,
    pub illuminance: f32,
    /// World ambient color
    pub ambient_color: Srgba,
    pub ambient_brightness: f32,
    pub blocks: Blocks
}

impl Default for MainConfig {
    fn default() -> Self {
        Self {
            vsync: true,
            light_direct: Vec3::new(-3.14/2.5, 0.0, 0.0),
            illuminance: 2000.0,
            ambient_color: Srgba::rgb_u8(210, 220, 240),
            ambient_brightness: 1.0,
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
pub struct Controller {
    pub chunks: HashMap<IVec3, chunk::Chunk>,
    pub meshes: HashMap<IVec3, Entity>,
    /// load chunks queue; build meshes queue
    pub load: OrderSet<IVec3>,
    pub build: OrderSet<IVec3>,

    /// unload queue
    pub unload: Vec<IVec3>,

    /// Compute tasks
    load_tasks: HashMap<IVec3, Task<RawChunk>>,
    build_tasks: HashMap<IVec3, Task<Option<Mesh>>>,
}

impl Default for Controller {
    fn default() -> Self {
        let n = 6;
        let k = n-1;

        // Test generate chunks area
        let mut load = OrderSet::new();
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

            load_tasks: HashMap::new(),
            build_tasks: HashMap::new(),
        }
    }
}

impl Controller {
    fn to_array<T: std::fmt::Debug, const N: usize>(data: Vec<T>) -> [T; N] {
        data.try_into().expect("Wrong size")
    }
    
    // sort load and build queues
    pub fn sort(&mut self, current: IVec3) {
        self.load.sort_by(|a, b| 
            a.distance_squared(current).cmp(&b.distance_squared(current)));

        self.build.sort_by(|a, b| 
            a.distance_squared(current).cmp(&b.distance_squared(current)));
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
            .add_systems(Update, systems::skybox)
            .add_systems(PostUpdate, (systems::hot_reload, systems::begin).chain())
            .add_systems(Last, systems::join);
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