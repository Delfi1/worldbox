use std::path::PathBuf;
use bevy::{
    asset::*,
    prelude::*
};
use serde::{Serialize, Deserialize};
use super::*;

mod systems;
mod player;
pub use player::*;

/// Contains all World raw data
/// Blocks lists
#[derive(Clone, Default)]
#[derive(Serialize, Deserialize)]
pub struct RawWorld {
    pub blocks: Blocks
}

impl RawWorld {
    /// Create new world folder
    pub async fn init(name: String) {
        let path = PathBuf::from(format!("./worlds/{}", name));
        let _ = async_fs::create_dir_all(&path).await;

        let _ = async_fs::write(
            path.join("world.yaml"),
            serde_yaml::to_string(&Self::default()).unwrap()
        ).await;
    }
}

#[derive(Asset, TypePath, Resource)]
/// Contains all current world data
pub struct WorldInfo {
    pub name: String,
    pub path: PathBuf,
    pub blocks: BlocksHandler,
    pub skybox: Handle<Image>,
    pub entities: Vec<Entity>
}

// todo: world load-serialize error
pub enum WorldError {

}

/// Get path parent name
fn parent_name(path: &PathBuf) -> String {
    let parent = path.parent().unwrap();
    let name = parent.file_name().unwrap();
    String::from(name.to_string_lossy())
}

#[derive(Default)]
pub struct WorldLoader;
impl AssetLoader for WorldLoader {
    type Asset = WorldInfo;
    type Settings = ();
    type Error = String;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _: &Self::Settings,
        context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let path = PathBuf::from(context.asset_path().path());
        let mut buf = String::with_capacity(512);

        // Read and serialize file
        match reader.read_to_string(&mut buf).await {
            Err(err) => return Err(format!("Read file error: {}", err)),
            _ => ()
        };

        let raw: RawWorld = match serde_yaml::from_str(&buf) {
            Ok(data) => data,
            Err(_) => return Err("Deserialize error".to_string())
        };

        println!("World path: {:?}", path);
        // Get world name
        Ok(WorldInfo {
            name: parent_name(&path),
            path,
            skybox: context.load("skybox.png"),
            blocks: BlocksHandler::new(context, raw.blocks),
            entities: Vec::with_capacity(128)
        })
    }

    fn extensions(&self) -> &[&str] {
        &[".yaml"]
    }
}

#[derive(Resource)]
/// Main stored world chunks data
pub struct Controller {
    pub world: Handle<WorldInfo>,

    pub chunks: HashMap<IVec3, chunk::Chunk>,
    pub meshes: HashMap<IVec3, Entity>,
    
    /// load chunks queue; build meshes queue
    pub load: OrderSet<IVec3>,
    pub build: OrderSet<IVec3>,

    /// unload and despawn queue
    pub unload: Vec<IVec3>,
    pub despawn: Vec<Entity>,

    /// Compute tasks
    pub load_tasks: HashMap<IVec3, Task<RawChunk>>,
    pub build_tasks: HashMap<IVec3, Task<Option<Mesh>>>,
    pub need_sort: bool
}

impl Controller {
    pub fn new(world: Handle<WorldInfo>) -> Self {
        Self {
            world,
            chunks: HashMap::with_capacity(1024),
            meshes: HashMap::with_capacity(1024),

            load: OrderSet::with_capacity(1024),
            build: OrderSet::with_capacity(1024),
            
            unload: Vec::with_capacity(512),
            despawn: Vec::with_capacity(512),

            load_tasks: HashMap::new(),
            build_tasks: HashMap::new(),
            need_sort: true
        }
    }

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

    /// Unload all chunks & meshes
    pub fn unload(&mut self) {
        self.load_tasks.drain();
        self.build_tasks.drain();
        self.load.clear();
        self.build.clear();
        self.unload.extend(self.chunks.keys().copied());
    }

    // Rebuild chunk meshes
    pub fn rebuild(&mut self, chunk: IVec3) {
        self.build.extend(ChunksRefs::offsets(chunk));
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

struct ViewBlockData {
    chunk: IVec3,
    block: usize,
    data: u16,
}

impl ViewBlockData {
    pub fn empty() -> Self {
        Self {
            chunk: IVec3::ZERO,
            block: 0,
            data: 0,
        }
    }

    pub fn set(&mut self, chunk: IVec3, block: usize, data: u16) {
        (self.chunk, self.block, self.data) = (chunk, block, data);
    }
}

#[derive(Resource)]
pub struct ViewBlocks {
    previous: ViewBlockData,
    current: ViewBlockData,
}

impl ViewBlocks {
    pub fn empty() -> Self {
        Self {
            previous: ViewBlockData::empty(),
            current: ViewBlockData::empty(),
        }
    }

    pub fn reset(&mut self) {
        *self = Self::empty();
    }
}

pub fn update_view_blocks(
    cameras: Query<Ref<GlobalTransform>, With<Camera3d>>,
    controller: Res<Controller>,
    mut view_blocks: ResMut<ViewBlocks>,
) {
    let camera = cameras.single();
    let current = camera.translation();
    let u = camera.forward().normalize();
    let blocks = RawChunk::under_cursor(current, u, 128);

    // Reset view_blocks blocks
    view_blocks.reset();
    for block in blocks {
        let chunk_pos = RawChunk::global(block);
        let index = RawChunk::block_index(RawChunk::relative(block));

        if let Some(chunk) = controller.chunks.get(&chunk_pos).cloned() {
            let guard = chunk.read();
            let data = guard.get()[index];
            if data == 0 {
                view_blocks.previous.set(chunk_pos, index, data);
            } else {
                view_blocks.current.set(chunk_pos, index, data);
                break;
            }
        }
    }
}

/// Current block in hand
#[derive(Debug, Resource)]
pub struct SelectedBlock(u16);

pub struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_asset::<WorldInfo>()
        .init_asset_loader::<WorldLoader>()
        .add_plugins(ui::MenuPlugin)
        .add_systems(OnEnter(AppState::MainMenu), systems::exit_world)
        .add_systems(OnExit(AppState::MainMenu), systems::enter_world)
        .add_systems(Last, systems::controls)
        .add_systems(
            FixedUpdate,
            update_view_blocks
            .run_if(in_state(AppState::Game))
        ).add_systems(
            Update,
            (
                systems::join_tasks,
                systems::prepare_textures,
                systems::skybox,
                systems::prepare_world,
                systems::hot_reload,
                systems::keybinding,
                systems::run_tasks,
            ).chain()
            .run_if(in_state(AppState::Game))
        ).add_systems(
            Last,
            (
                systems::unload,
                systems::despawn,
            ).chain()
            .run_if(in_state(AppState::Game))
        );
    }
}