use bevy::prelude::*;
use bevy_egui::{egui, *};
use super::*;

#[derive(Resource)]
pub struct WorldsList {
    pub worlds: Vec<Handle<WorldData>>,
    pub need_reload: bool
}

impl WorldsList {
    pub fn reload(&mut self) {
        self.need_reload = true;
    }

    pub fn iter(&self) -> Vec<Handle<WorldData>> {
        self.worlds.clone()
    }
}

pub fn setup(
    assets: Res<AssetServer>,
    mut commands: Commands,
) {
    let list = WorldData::load_list();

    commands.insert_resource(
        WorldsList {
            worlds: list.into_iter().map(|p| assets.load(p)).collect(),
            need_reload: true
        }
    );
}

#[derive(Resource)]
pub struct WorldName(pub String);

impl Default for WorldName {
    fn default() -> Self { Self("World".to_string()) }
}

/// Update menu ui
pub fn update(
    mut contexts: EguiContexts,
    mut commands: Commands,
    assets: Res<AssetServer>,
    worlds: Res<Assets<WorldData>>,
    mut materials: ResMut<Assets<ChunkMaterial>>,
    mut next_state: ResMut<NextState<MainState>>,

    mut world_name: ResMut<WorldName>,
    mut list: ResMut<WorldsList>
) {
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
                WorldData::init(world_name.0.clone());
                *world_name = WorldName::default();
                
                list.reload();
            }
            
            // Worlds list (load-remove)
            cen.add_space(10.0);
            cen.label("Worlds list:");
            egui::Grid::new("grid").show(cen, |grid| {
                for handle in list.iter() {
                    let Some(data) = worlds.get(&handle) else { continue; };

                    if grid.button(format!("{}", data.name)).clicked() {
                        println!("Opening {};", data.name);
                        
                        let blocks = BlocksHandler::new(&assets, data.blocks.clone());
                        let material = materials.add(ChunkMaterial::new(&blocks));
                        let res = WorldRes {
                            name: data.name.clone(),
                            handler: handle.clone(),
                            blocks,
                            main_material: material,
                            entities: Vec::new()
                        };

                        commands.insert_resource(res);
                        next_state.set(MainState::Loading);
                    };

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

/// Reload worlds list
pub fn reload(
    events: EventReader<AssetEvent<WorldData>>,
    assets: Res<AssetServer>,
    mut worlds: ResMut<WorldsList>
) {
    if worlds.need_reload | !events.is_empty() {
        let list = WorldData::load_list();
        worlds.worlds = list.into_iter().map(|p| assets.load(p)).collect();
        
        worlds.need_reload = false;
    }
}

/// Load all assets if loaded - enter world
pub fn process(
    world: ResMut<WorldRes>,
    assets: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut next_state: ResMut<NextState<MainState>>,
) {
    // todo loading check logic
    let textures: Vec<_> = world.blocks.textures().iter().cloned().filter(|t| t.is_some())
        .map(|t| t.unwrap()).collect();
    
    for texture in textures {
        if assets.is_loaded(&texture) {
            let image = images.get(&texture).unwrap();
            if image.texture_descriptor.array_layer_count() != 1 { continue; }

            // If image isn't proceeded yet - reinterpret
            let image = images.get_mut(&texture).unwrap();
            image.reinterpret_stacked_2d_as_array(6);
        } else { 
            // Wait for all assets loading
            return;
        }
    }

    next_state.set(MainState::InGame);
}