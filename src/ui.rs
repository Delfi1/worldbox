use std::path::*;
use bevy::{
    prelude::*,
    asset::*
};
use bevy_egui::*;
use super::*;

#[derive(Resource)]
struct WorldName(pub String);

impl Default for WorldName {
    fn default() -> Self {
        Self("World".into())
    }
}

#[derive(Resource)]
struct WorldsList {
    pub worlds: Vec<Handle<WorldInfo>>,
    pub need_reload: bool
}

impl WorldsList {
    pub fn reload(&mut self) {
        self.need_reload = true;
    }

    pub fn iter(&self) -> Vec<Handle<WorldInfo>> {
        self.worlds.clone()
    }
}

/// Get all worlds paths
fn load_list() -> Vec<AssetPath<'static>> {
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

/// Setup ui
fn setup(
    assets: Res<AssetServer>,
    mut commands: Commands,
) {
    let list = load_list();

    commands.insert_resource(WorldName::default());
    commands.insert_resource(
        WorldsList {
            worlds: list.into_iter().map(|p| assets.load(p)).collect(),
            need_reload: true
        }
    );
}

fn menu(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut list: ResMut<WorldsList>,
    mut world_name: ResMut<WorldName>,
    assets: Res<AssetServer>,
    world_assets: Res<Assets<WorldInfo>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let task_pool = IoTaskPool::get();
    let panel = egui::CentralPanel::default();

    panel.show(contexts.ctx_mut(), |ui| {
        ui.vertical_centered(|cen| {
            cen.label(egui::RichText::new("Main Menu").size(50.0));

            // Create new world
            cen.label("New World:");
            cen.add(egui::TextEdit::singleline(&mut world_name.0).char_limit(32).desired_width(64.0));
            world_name.0 = world_name.0.replace(" ", "_");
            
            cen.add_space(10.0);
            if cen.button("Create").clicked() {
                // Spawn io create-task
                task_pool.spawn(RawWorld::init(world_name.0.clone())).detach();
                *world_name = WorldName::default();
                
                list.reload();
            }
            
            // Worlds list (load-remove)
            cen.add_space(10.0);
            cen.label("Worlds list:");
            egui::Grid::new("grid").show(cen, |grid| {
                for handle in list.iter() {
                    let Some(world) = world_assets.get(&handle) else { continue; };

                    if grid.button(format!("{}", world.name)).clicked() {
                        commands.insert_resource(Controller::new(handle.clone()));
                        next_state.set(AppState::Game);
                    }

                    if grid.button("Remove").clicked() {
                        let asset_path = assets.get_path(&handle).unwrap();
                        let dir = asset_path.path().parent().unwrap();
                        let path = PathBuf::from("./worlds").join(dir);
                        std::fs::remove_dir_all(path).unwrap();                        
                        list.reload();
                    }
                    grid.end_row();
                }
            });
        });
    });
}

/// Update worlds list
fn reload(
    mut list: ResMut<WorldsList>,
    assets: Res<AssetServer>,
    events: EventReader<AssetEvent<WorldInfo>>,
) {
    if list.need_reload | !events.is_empty() {
        let paths = load_list();
        list.worlds = paths.into_iter().map(|p| assets.load(p)).collect();
        
        list.need_reload = false;
    }
}

// Game menu UI
fn game_menu(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>
) {
    // Main modal menu
    let _modal = egui::Modal::new(egui::Id::new("Menu")).show(contexts.ctx_mut(), |ui| {
        ui.set_width(400.0);
        ui.set_height(460.0);
        ui.heading("Menu");

        ui.label("WORK IN PROGRESS");
       
        // World save and exit button
        ui.add_space(30.0);
        
        if ui.button("Exit").clicked() {
            next_state.set(AppState::MainMenu)
        }
    });
}

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                reload,
                menu
            )
            .chain()
            .run_if(in_state(AppState::MainMenu))
        )
        .add_systems(
            Update,
            game_menu.run_if(in_state(AppState::GameMenu))
        );
    }
}