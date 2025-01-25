use bevy::{
    math::*,
    prelude::*,
    input::mouse::*,
};
use bevy::window::*;
use std::f32::consts::PI;
use super::*;

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
    pub current_chunk: IVec3,
    need_update: bool,
}

impl MainCamera {
    pub fn new() -> Self {
        Self {
            controller: CameraController::new(),
            current_chunk: IVec3::ZERO,
            need_update: true,
        }
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Last, 
            camera_control
                .run_if(any_with_component::<PrimaryWindow>)
                .run_if(any_with_component::<MainCamera>)
                .run_if(in_state(AppState::Game))
        ).add_systems
        (PostUpdate,
             (
                on_move,
                calculate_area
            ).chain()
            .run_if(in_state(AppState::Game))
        );
    }
}

fn camera_control(
    mut cameras: Query<(Mut<MainCamera>, Mut<Transform>)>,
    time: Res<Time>,
    kbd: Res<ButtonInput<KeyCode>>,
    mut evr_motion: EventReader<MouseMotion>,
) {
    let delta_time = time.delta().as_secs_f32();

    let mut motion = Vec2::ZERO;
    for event in evr_motion.read() {
        motion -= event.delta;
    }

    if let Ok((mut camera, mut transform)) = cameras.get_single_mut() {
        let mut speed = camera.controller.speed;
        if kbd.pressed(KeyCode::ControlLeft) { speed *= 2.0 }

        if kbd.pressed(KeyCode::KeyW) {
            let mut forward = transform.forward().as_vec3();
            forward.y = 0.0;

            transform.translation += forward.normalize_or_zero() * speed * delta_time;
        }
        if kbd.pressed(KeyCode::KeyD) {
            let right = transform.right().normalize();
            transform.translation += right * speed * delta_time;
        }
        if kbd.pressed(KeyCode::KeyS) {
            let mut back = transform.back().as_vec3();
            back.y = 0.0;

            transform.translation += back.normalize_or_zero() * speed * delta_time;
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

        // Rotate camera
        let contr = &mut camera.controller;
        contr.yaw += motion.x.to_radians() * contr.sensitivity;
        contr.pitch += motion.y.to_radians() * contr.sensitivity;
        contr.pitch = contr.pitch.clamp(-PI/2.02, PI/2.02);
        
        transform.rotation = Quat::from_euler(
            EulerRot::YXZ,
            camera.controller.yaw,
            camera.controller.pitch,
            0.0
        );
    }
}

// Detect camera move
fn on_move(
    mut controller: ResMut<Controller>,
    mut cameras: Query<(Mut<MainCamera>, Mut<LoadArea>, Ref<Transform>)>
) {
    for (mut camera, mut load, transform) in cameras.iter_mut() {
        let global = super::RawChunk::global(transform.translation);
        // If need_update was overrided
        if camera.need_update {
            camera.current_chunk = global;
            load.current_chunks = load.area_chunks_pos(global);
            load.current_meshes = load.area_meshes_pos(global);
            
            controller.load.extend(load.current_chunks.iter().copied());
            controller.build.extend(load.current_meshes.iter().copied());
        }

        // If current chunk is changed
        if camera.current_chunk != global {
            camera.current_chunk = global;
            camera.need_update = true;
        }
    }
}

fn calculate_area(
    mut controller: ResMut<Controller>,
    mut cameras: Query<(Mut<MainCamera>, Mut<LoadArea>)>
) {
    let (mut camera, mut loadarea) = cameras.get_single_mut().unwrap();
    
    if camera.need_update {
        let pos = camera.current_chunk;
        let new_chunks = loadarea.area_chunks_pos(pos);
        let new_meshes = loadarea.area_meshes_pos(pos);

        let load = new_chunks.difference(&loadarea.current_chunks);
        let build = new_meshes.difference(&loadarea.current_meshes);
        let unload = loadarea.current_chunks.difference(&new_chunks);

        controller.load.extend(load);
        controller.build.extend(build);
        controller.unload.extend(unload);
        controller.sort();

        loadarea.current_chunks = new_chunks;
        loadarea.current_meshes = new_meshes;

        camera.need_update = false;
    }
}


#[derive(Clone, Component)]
/// Procceds load and unload territory
pub struct LoadArea {
    chunks_area: OrderSet<IVec3>,
    meshes_area: OrderSet<IVec3>,
    current_chunks: OrderSet<IVec3>,
    current_meshes: OrderSet<IVec3>
}

impl LoadArea {
    fn make_area(pos: IVec3, w: i32, h: i32) -> OrderSet<IVec3> {
        let mut result = OrderSet::with_capacity((w*w*h) as usize);

        for x in pos.x-w..pos.x+w {
            for z in pos.z-w..pos.z+w {
                for y in pos.y-w..pos.y+w {
                    result.insert(IVec3::new(x, y, z));
                }
            }
        }

        result
    }

    pub fn area_chunks_pos(&self, pos: IVec3) -> OrderSet<IVec3> {
        self.chunks_area.iter().copied().into_iter().map(|p| p + pos).collect()
    }

    pub fn area_meshes_pos(&self, pos: IVec3) -> OrderSet<IVec3> {
        self.meshes_area.iter().copied().into_iter().map(|p| p + pos).collect()
    }

    pub fn new(width: u32, height: u32) -> Self {
        let chunk_area = Self::make_area(IVec3::ZERO, (width+1) as i32, (height+1) as i32);
        let meshes_area = Self::make_area(IVec3::ZERO, width as i32, height as i32);

        Self {
            chunks_area: chunk_area,
            meshes_area: meshes_area,
            current_chunks: OrderSet::new(),
            current_meshes: OrderSet::new()
        }
    }

    pub fn _update(&mut self, new_w: u32, new_h: u32) {
        *self = Self::new(new_w, new_h);
    }
}