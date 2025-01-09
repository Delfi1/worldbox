//! Just simple debug info

use bevy::diagnostic::DiagnosticsStore;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;

#[derive(Component)]
struct FpsText;

fn setup(
    mut commands: Commands,
) {
    commands.spawn((
        Text::new("N/A"),
        FpsText,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

fn update(
    diagnostics: Res<DiagnosticsStore>,
    camera_query: Query<Ref<Transform>, With<Camera3d>>,
    mut query: Query<Mut<Text>, With<FpsText>>,
) {
    if let Some(value) = diagnostics
    .get(&FrameTimeDiagnosticsPlugin::FPS).and_then(|fps| fps.smoothed()) {
        let camera = camera_query.single();
        let mut text = query.get_single_mut().unwrap();

        text.0 = format!(
            "Fps: {}; \nPosition: {}; \nView: {};",
            value.round() as u32,
            camera.translation,
            camera.forward().normalize()
        );
    }
}

fn hide(
    mut q: Query<Mut<Visibility>, With<FpsText>>,
    kbd: Res<ButtonInput<KeyCode>>,
) {
    if kbd.just_pressed(KeyCode::F12) {
        let mut vis = q.single_mut();
        *vis = match *vis {
            Visibility::Hidden => Visibility::Visible,
            _ => Visibility::Hidden,
        };
    }
}

pub struct FpsPlugin;
impl Plugin for FpsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            .add_systems(Startup, setup)
            .add_systems(FixedUpdate, (update, hide).chain());
    }
}