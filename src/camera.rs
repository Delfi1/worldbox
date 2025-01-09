use bevy::{
    math::*,
    prelude::*,
    input::mouse::{MouseMotion, MouseWheel},
};
use bevy::window::*;
use std::f32::consts::PI;

pub struct CameraController {
    pub speed: f32,
    pub sensitivity: f32,
    pub yaw: f32,
    pub pitch: f32,
}

impl CameraController {
    fn new() -> Self {
        Self { speed: 12.0, sensitivity: 0.2, yaw: 0.0, pitch: 0.0 }
    }
}

#[derive(Component)]
pub struct MainCamera {
    pub controller: CameraController,
}

impl MainCamera {
    pub fn new() -> Self {
        Self {controller: CameraController::new()}
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Last, camera_control.run_if(any_with_component::<PrimaryWindow>));
    }
}

fn camera_control(
    mut cameras: Query<(Mut<MainCamera>, Mut<Transform>)>,
    primary_window: Query<Ref<Window>, With<PrimaryWindow>>,
    time: Res<Time>,
    kbd: Res<ButtonInput<KeyCode>>,
    mut evr_motion: EventReader<MouseMotion>,
    mut evr_scroll: EventReader<MouseWheel>,
) {
    let window = primary_window.get_single().unwrap();
    let delta_time = time.delta().as_secs_f32();

    let mut motion = Vec2::ZERO;
    for event in evr_motion.read() {
        motion -= event.delta;
    }

    let mut scroll = 0.0;
    for event in evr_scroll.read() {
        scroll += event.y
    }

    if let Ok((mut camera, mut transform)) = cameras.get_single_mut() {
        let forward = transform.forward().normalize();
        let mut speed = camera.controller.speed;
        if kbd.pressed(KeyCode::ControlLeft) { speed *= 2.0 }
        let scroll_speed = speed * scroll * 4.0;

        transform.translation += forward * scroll_speed * delta_time;

        if kbd.pressed(KeyCode::KeyW) {
            transform.translation += forward * speed * delta_time;
        }
        if kbd.pressed(KeyCode::KeyD) {
            let right = transform.right().normalize();
            transform.translation += right * speed * delta_time;
        }
        if kbd.pressed(KeyCode::KeyS) {
            let back = transform.back().normalize();
            transform.translation += back * speed * delta_time;
        }
        if kbd.pressed(KeyCode::KeyA) {
            let left = transform.left().normalize();
            transform.translation += left * speed * delta_time;
        }

        if kbd.pressed(KeyCode::ShiftLeft) {
            transform.translation.y -= speed * delta_time;
        }
        if kbd.pressed(KeyCode::Space) {
            transform.translation.y += speed * delta_time;
        }

        if window.cursor_options.grab_mode != CursorGrabMode::None {
            // Rotate camera
            let contr = &mut camera.controller;
            contr.yaw += motion.x.to_radians() * contr.sensitivity;
            contr.pitch += motion.y.to_radians() * contr.sensitivity;
            contr.pitch = contr.pitch.clamp(-PI/2., PI/2.);
        }

        transform.rotation = Quat::from_euler(
            EulerRot::YXZ,
            camera.controller.yaw,
            camera.controller.pitch,
            0.0
        );
    }
}
