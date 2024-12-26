use core::f32;

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

fn setup(mut commands: Commands, mut assets: ResMut<AssetServer>, mut windows: Query<Mut<Window>>) {
    if let Ok(mut window) = windows.get_single_mut() {
        window.title = String::from("WorldBox");
    }

    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(16.0, 36.0, 16.0))
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 4000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            f32::consts::PI / 3.,
            -f32::consts::PI / 4.,
        )),
    ));
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
        .run();
}