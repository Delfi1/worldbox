//! Just simple debug info

use super::MainState;
use bevy::diagnostic::DiagnosticsStore;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;

#[derive(Component)]
struct DebugText;

fn setup(
    mut commands: Commands,
) {
    commands.spawn((
        Text::new("N/A"),
        DebugText,
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
    mut query: Query<Mut<Text>, With<DebugText>>,
) {
    if let Some(value) = diagnostics
    .get(&FrameTimeDiagnosticsPlugin::FPS).and_then(|fps| fps.smoothed()) {
        let camera = camera_query.single();
        let mut text = query.get_single_mut().unwrap();

        text.0 = format!(
            "Fps: {}; \nPosition: {}; \nView: {};",
            value.round() as u32,
            camera.translation.as_ivec3(),
            camera.forward().normalize()
        );
    }
}

fn hide(
    mut q: Query<Mut<Visibility>, With<DebugText>>,
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

fn destroy(
    mut commands: Commands,
    query: Query<Entity, With<DebugText>>
) {
    for e in &query {
        commands.entity(e).despawn();
    }
}

pub struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            .add_systems(OnEnter(MainState::InGame), setup)
            .add_systems(FixedUpdate,
                (update, hide).chain().run_if(in_state(MainState::InGame))
            ).add_systems(OnExit(MainState::InGame), destroy);
    }
}