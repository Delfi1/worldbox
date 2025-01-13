mod menu;

use std::path::PathBuf;
use bevy::{
    prelude::*,
    asset::*,
};
use serde::{Serialize, Deserialize};
use super::*;

#[derive(Resource, Clone)]
/// Contains all main world objects
pub struct WorldRes {
    pub name: String,
    /// Current world data (for hot-reloading)
    pub handler: Handle<WorldData>,
    /// All world's blocks
    pub blocks: BlocksHandler,
    pub main_material: Handle<ChunkMaterial>,
    /// Remove this after exit world
    pub entities: Vec<Entity>
}

/// World data in ./worlds/
#[derive(Clone, TypePath, Asset)]
#[derive(Serialize, Deserialize)]
pub struct WorldData {
    pub name: String,
    pub skybox: AssetPath<'static>,
    pub blocks: Blocks
}

impl WorldData {
    /// Create new world folder
    pub fn init(name: String) {
        let dir = PathBuf::from(format!("./worlds/{}", name));
        if std::fs::exists(&dir).unwrap() {
            println!("World already exists");
            return;
        }

        // Create all folders
        std::fs::create_dir_all(&dir).unwrap();
        let data = WorldData { name, ..default() };
        let file = dir.join("world.yaml");
        std::fs::write(&file, serde_yaml::to_string(&data).unwrap()).unwrap();
    }

    /// Load all folders list
    pub fn load_list() -> Vec<AssetPath<'static>> {
        let mut result = Vec::new();
        for entry in glob::glob("./worlds/*/world.yaml").expect("Failed to read glob pattern") {
            if let Ok(path) = entry {
                let mut components = path.components();
                components.next();
                result.push(PathBuf::from_iter(components));
            }
        }

        result.into_iter().map(|p| AssetPath::from_static(p).with_source("worlds")).collect()
    }
}

impl Default for WorldData {
    fn default() -> Self {
        Self {
            name: "World".into(),
            skybox: "skybox.png".into(),
            blocks: Blocks::default()
        }
    }
}

#[derive(Default)]
pub struct WorldLoader;
impl AssetLoader for WorldLoader {
    type Asset = WorldData;
    type Error = String;
    type Settings = ();

    async fn load(
            &self,
            reader: &mut dyn io::Reader,
            _settings: &Self::Settings,
            _load_context: &mut LoadContext<'_>,
        ) -> Result<Self::Asset, Self::Error> {
        let mut buf = String::with_capacity(512);
        if let Ok(_) = reader.read_to_string(&mut buf).await {
            if let Ok(data) = serde_yaml::from_str(&buf) {
                Ok(data)
            } else {
                Err("Serialize error".to_string())
            }
        } else {
            Err("Read Error".to_string())
        }
    }

    fn extensions(&self) -> &[&str] {
        &["yaml"]
    }
}

#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, States)]
pub enum MainState {
    /// Menu screen, choose world
    #[default]
    InMenu,
    /// World loading state
    Loading,
    /// When world is loaded
    InGame,
}

/// Main game plugin
pub struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(menu::WorldName::default())
            .add_systems(OnEnter(MainState::InMenu), menu::setup)
            .add_systems(Startup, systems::setup)
            .add_systems(Update, 
                (menu::update, menu::reload).run_if(in_state(MainState::InMenu))
            ).add_systems(OnEnter(MainState::Loading), systems::load_world)
            .add_systems(Update, menu::process.run_if(in_state(MainState::Loading)));
    }
}