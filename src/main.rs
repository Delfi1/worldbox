mod chunk;
mod mesher;
mod rendering;
mod camera;
mod debug;
mod world;
pub mod ui;

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

/// Main engine logic
pub struct EnginePlugin;
impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_egui::EguiPlugin)
        .add_plugins((WorldPlugin, DebugPlugin, CameraPlugin, RenderingPlugin));
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

#[derive(Debug, Default, States, Copy, Clone, Hash, Eq, PartialEq)]
pub enum AppState {
    #[default]
    MainMenu,
    Game,
    GameMenu
}

pub fn main() {
    App::new()
        .register_asset_source(
            "worlds",
            AssetSourceBuilder::platform_default("worlds", None)
        )
        .add_plugins(DefaultPlugins
            .set(ImagePlugin { default_sampler: default_sampler() })
        )
        .init_state::<AppState>()
        .add_plugins(EnginePlugin)
        .run();
}