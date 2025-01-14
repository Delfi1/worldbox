use bevy::{
    math::*,
    utils::*,
    prelude::*,
    input::mouse::*,
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
        app.add_systems(Last, 
            camera_control.run_if(any_with_component::<PrimaryWindow>)
                .run_if(any_with_component::<MainCamera>)
        );
    }
}

fn camera_control(
    mut cameras: Query<(Mut<MainCamera>, Mut<Transform>)>,
    primary_window: Query<Ref<Window>, With<PrimaryWindow>>,
    time: Res<Time>,
    kbd: Res<ButtonInput<KeyCode>>,
    mut evr_motion: EventReader<MouseMotion>,
) {
    let window = primary_window.get_single().unwrap();
    let delta_time = time.delta().as_secs_f32();

    let mut motion = Vec2::ZERO;
    for event in evr_motion.read() {
        motion -= event.delta;
    }

    if let Ok((mut camera, mut transform)) = cameras.get_single_mut() {
        let forward = transform.forward().normalize();
        let mut speed = camera.controller.speed;
        if kbd.pressed(KeyCode::ControlLeft) { speed *= 2.0 }

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

#[derive(Component)]
/// Procceds load and unload territory
pub struct LoadArea {
    area: HashSet<IVec3>,
    current: HashSet<IVec3>,
}

impl LoadArea {
    fn make_area(pos: IVec3, w: i32, h: i32) -> HashSet<IVec3> {
        let mut result = HashSet::with_capacity((w*w*h) as usize);

        for x in pos.x-w..pos.x+w {
            for z in pos.z-w..pos.z+w {
                for y in pos.y-w..pos.y+w {
                    result.insert(IVec3::new(x, y, z));
                }
            }
        }

        result
    }

    pub fn new(width: u32, height: u32) -> Self {
        let area = Self::make_area(IVec3::ZERO, width as i32, height as i32);

        Self { 
            area: area.clone(),
            current: area
        }
    }
}