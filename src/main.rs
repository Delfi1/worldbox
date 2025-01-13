mod chunk;
mod mesher;
mod rendering;
mod systems;
mod camera;
mod debug;
mod world;

use ordermap::OrderSet;
use bevy::{
    prelude::*,
    utils::*,
    tasks::*,
    asset::io::*,
    image::*,
};

use camera::*;
use mesher::*;
use chunk::*;
use rendering::*;
use debug::*;
use world::*;

// Todo:
// 1) World load-store system
// 2) Update Blocks Data (Add models, collisions, tags etc)
// 3) Player collisions with blocks
// 4) Add normal maps option for textures

#[derive(Resource)]
/// Main stored world chunks data
pub struct Controller {
    pub chunks: HashMap<IVec3, chunk::Chunk>,
    pub meshes: HashMap<IVec3, Entity>,
    /// load chunks queue; build meshes queue
    pub load: OrderSet<IVec3>,
    pub build: OrderSet<IVec3>,

    /// unload and despawn queue
    pub _unload: Vec<IVec3>,
    pub despawn: Vec<Entity>,

    /// Compute tasks
    pub load_tasks: HashMap<IVec3, Task<RawChunk>>,
    pub build_tasks: HashMap<IVec3, Task<Option<Mesh>>>,
    pub need_sort: bool
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
            //load: OrderSet::with_capacity(1024),
            //build: OrderSet::with_capacity(1024),
            load, build,
            
            _unload: Vec::with_capacity(512),
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

/// Main engine logic
pub struct EnginePlugin;
impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<MainState>()
        .add_plugins(bevy_egui::EguiPlugin)
        .add_plugins((WorldPlugin, DebugPlugin, CameraPlugin, RenderingPlugin))
        .add_systems(Update,
            (systems::keybind).run_if(in_state(MainState::InGame))
        ).add_systems(FixedUpdate,
            (systems::skybox).run_if(in_state(MainState::InGame))
        ).add_systems(FixedPostUpdate,
            systems::update_selected.run_if(in_state(MainState::InGame))
        ).add_systems(PostUpdate,
            (systems::hot_reload, systems::begin).chain().run_if(in_state(MainState::InGame))
        ).add_systems(Last,
            (systems::unload, systems::join).chain().run_if(in_state(MainState::InGame))
        );
    }
}

/// Default textures sampler
fn default_sampler() -> ImageSamplerDescriptor {
    ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        ..default()
    }
}

pub fn main() {
    App::new()
        .register_asset_source(
            "worlds",
            AssetSourceBuilder::platform_default("worlds", None)
        )
        .add_plugins(DefaultPlugins
            .set(ImagePlugin { default_sampler: default_sampler() })
        ).init_asset::<WorldData>()
        .init_asset_loader::<WorldLoader>()
        .add_plugins(EnginePlugin)
        .run();
}