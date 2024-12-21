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

// WorldController plugin
mod world;
use world::*;

fn setup(mut windows: Query<Mut<Window>>) {
    if let Ok(mut window) = windows.get_single_mut() {
        window.title = String::from("WorldBox");
    }
}

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_plugins(WorldPlugin)
        .run();
}