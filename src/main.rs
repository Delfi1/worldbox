use bevy::{
    prelude::*,
    tasks::*,
    utils::*, 
    window::*
};

mod rendering;
mod blocks;
mod chunks;
mod utils;
mod camera;
mod fps;

// WorldController plugin
mod world;
use world::*;

fn setup(mut commands: Commands, mut assets: ResMut<AssetServer>, mut windows: Query<Mut<Window>>) {
    if let Ok(mut window) = windows.get_single_mut() {
        window.title = String::from("WorldBox");
    }
    
    commands.insert_resource(world::Texture(assets.load("textures.png")));
}
    
pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes_override: Some(true),
            ..Default::default()
        }))
        .add_systems(Startup, setup)
        .add_plugins(WorldPlugin)
        .add_plugins(fps::FpsPlugin)
        .run();
}