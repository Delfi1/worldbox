use std::collections::VecDeque;
use bevy::{
    math::*,
    prelude::*,
    input::mouse::{MouseMotion, MouseWheel},
    utils::HashSet
};
use bevy::window::{CursorGrabMode, PrimaryWindow};
use std::f32::consts::PI;

use super::{
    world::WorldController,
    utils::*
};

/// Render distance
#[derive(Debug, Clone)]
pub struct RenderDistance {
    width: usize,
    height: usize,
    chunk_offsets: Vec<IVec3>,
    mesh_offsets: Vec<IVec3>,
}

impl RenderDistance {
    fn make_chunk_offsets(half_width: usize) -> Vec<IVec3> {
        let k_w = (half_width * 2) + 1;

        let mut sampling_offsets = HashSet::with_capacity(k_w.pow(2));
        for i in 0..k_w * k_w * k_w  {
            let mut pos = index_to_ivec3_bounds(i, k_w);
            let w = (k_w as f32 / 2.0) as i32;
            pos -= IVec3::new(w, w, w);

            sampling_offsets.insert(pos);
        }
        let mut offsets = Vec::from_iter(sampling_offsets);

        offsets.sort_by(|a, b| {
            a.distance_squared(IVec3::ZERO)
                .cmp(&b.distance_squared(IVec3::ZERO))
        });

        offsets
    }

    fn make_mesh_offsets(half_width: usize, half_height: usize) -> Vec<IVec3> {
        let k_w = (half_width * 2) + 1;
        let k_h = (half_height * 2) + 1;
        let mut sampling_offsets = Vec::with_capacity(k_w.pow(2) * k_h);
        for i in 0..k_w.pow(3) {
            let mut pos = index_to_ivec3_bounds(i, k_w);

            let w = (k_w as f32 / 2.0) as i32;
            let h = (k_h as f32 / 2.0) as i32;

            pos -= IVec3::new(w, w.min(h), w);

            sampling_offsets.push(pos);
        }
        sampling_offsets.sort_by(|a, b| {
            a.distance_squared(IVec3::ZERO)
                .cmp(&b.distance_squared(IVec3::ZERO))
        });

        sampling_offsets
    }

    pub fn new(width: usize, height: usize) -> Self {
        let chunk_offsets = Self::make_chunk_offsets(width + 1);
        let mesh_offsets = Self::make_mesh_offsets(width, height);

        Self {width, height, chunk_offsets, mesh_offsets}
    }

    pub fn update(&mut self, width: usize, height: usize) {
        self.chunk_offsets = Self::make_chunk_offsets(width + 1);
        self.mesh_offsets = Self::make_mesh_offsets(width, height);
    }
}

pub struct Controller {
    pub speed: f32,
    pub sensitivity: f32,
    pub yaw: f32,
    pub pitch: f32,
}

impl Controller {
    fn new() -> Self {
        Self { speed: 12.0, sensitivity: 0.012, yaw: 0.0, pitch: 0.0 }
    }
}

#[derive(Component)]
pub struct MainCamera {
    pub controller: Controller,
    pub render_distance: RenderDistance,
    pub prev_chunk_pos: IVec3,

    pub unresolved_chunk_load: Vec<IVec3>,
    pub unresolved_mesh_load: Vec<IVec3>,

    pub unresolved_chunk_unload: VecDeque<IVec3>,
    pub unresolved_mesh_unload: VecDeque<IVec3>,
}

impl MainCamera {
    pub fn new(render_distance: RenderDistance) -> Self {
        Self {
            controller: Controller::new(),
            render_distance,
            prev_chunk_pos: ivec3(512, 512, 512),

            unresolved_chunk_load: Vec::new(),
            unresolved_chunk_unload: VecDeque::new(),

            unresolved_mesh_load: Vec::new(),
            unresolved_mesh_unload: VecDeque::new()
        }
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (detect_move, scan_chunk, scan_mesh, scan_chunk_unload, scan_mesh_unload)
                .run_if(any_with_component::<MainCamera>)
        ).add_systems(
            Last, camera_control.run_if(any_with_component::<PrimaryWindow>)
        );
    }
}

fn camera_control(
    mut primary_window: Query<Mut<Window>, With<PrimaryWindow>>,
    mut cameras: Query<(Mut<MainCamera>, Mut<Transform>)>,
    time: Res<Time>,
    kbd: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut evr_motion: EventReader<MouseMotion>,
    mut evr_scroll: EventReader<MouseWheel>,
) {
    let mut window = primary_window.get_single_mut().unwrap();
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
        let speed = camera.controller.speed;
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

        if mouse_buttons.pressed(MouseButton::Right) {
            let contr = &mut camera.controller;

            window.cursor_options.visible = false;
            window.cursor_options.grab_mode = CursorGrabMode::Locked;

            contr.yaw += motion.x.to_radians() * contr.sensitivity * delta_time * 1000.0;
            contr.pitch += motion.y.to_radians() * contr.sensitivity * delta_time * 1000.0;

            contr.pitch = contr.pitch.clamp(-PI/2., PI/2.);
        } else {
            window.cursor_options.visible = true;
            window.cursor_options.grab_mode = CursorGrabMode::None;
        }

        transform.rotation = Quat::from_euler(
            EulerRot::YXZ,
            camera.controller.yaw,
            camera.controller.pitch,
            0.0
        );
    }
}

fn detect_move(
    mut world: ResMut<WorldController>,
    mut cameras: Query<(Mut<MainCamera>, Ref<GlobalTransform>)>
) {
    for (mut camera, g_transform) in cameras.iter_mut() {
        let chunk_pos = get_chunk_pos(g_transform.translation());

        let previous_chunk_pos = camera.prev_chunk_pos;
        let chunk_pos_changed = chunk_pos != camera.prev_chunk_pos;
        camera.prev_chunk_pos = chunk_pos;
        if !chunk_pos_changed {
            return;
        }

        let load_chunk_area = camera
            .render_distance.chunk_offsets
            .iter()
            .map(|offset| chunk_pos + *offset)
            .collect::<HashSet<IVec3>>();

        let unload_chunk_area = camera
            .render_distance.chunk_offsets
            .iter()
            .map(|offset| previous_chunk_pos + *offset)
            .collect::<HashSet<IVec3>>();

        let load_mesh_area = camera
            .render_distance.mesh_offsets
            .iter()
            .map(|offset| chunk_pos + *offset)
            .collect::<HashSet<IVec3>>();

        let unload_mesh_area = camera
            .render_distance.mesh_offsets
            .iter()
            .map(|offset| previous_chunk_pos + *offset)
            .collect::<HashSet<IVec3>>();

        let chunk_load = load_chunk_area.difference(&unload_chunk_area);
        let chunk_unload = unload_chunk_area.difference(&load_chunk_area);
        let mesh_load = load_mesh_area.difference(&unload_mesh_area);
        let mesh_unload = unload_mesh_area.difference(&load_mesh_area);

        camera.unresolved_chunk_load.extend(chunk_load);
        camera.unresolved_chunk_unload.extend(chunk_unload);
        camera.unresolved_mesh_unload.extend(mesh_unload);
        camera.unresolved_mesh_load.extend(mesh_load);

        let MainCamera {
            unresolved_chunk_load,
            unresolved_mesh_load,
            unresolved_chunk_unload,
            unresolved_mesh_unload,
            ..
        } = camera.as_mut();

        for p in unresolved_mesh_unload.iter() {
            if let Some((i, _)) = world
                .load_meshes
                .iter()
                .enumerate()
                .find(|(_i, k)| *k == p)
            {
                world.load_meshes.remove(i);
            }
        }
        for p in unresolved_chunk_unload.iter() {
            if let Some((i, _)) = world
                .load_chunks
                .iter()
                .enumerate()
                .find(|(_i, k)| *k == p)
            {
                world.load_chunks.remove(i);
            }
        }

        unresolved_mesh_load.retain(|p| {
            let want_unload = unresolved_mesh_unload.contains(p);
            !want_unload
        });

        unresolved_chunk_load.retain(|p| {
            let want_unload = unresolved_chunk_unload.contains(p);
            !want_unload
        });

        camera.unresolved_mesh_load.sort_by(|a, b| {
            a.distance_squared(chunk_pos)
                .cmp(&b.distance_squared(chunk_pos))
        });
        camera.unresolved_chunk_load.sort_by(|a, b| {
            a.distance_squared(chunk_pos)
                .cmp(&b.distance_squared(chunk_pos))
        });
    }
}

fn scan_chunk(
    mut cameras: Query<Mut<MainCamera>>,
    mut world: ResMut<WorldController>,
) {
    for mut camera in cameras.iter_mut() {
        if world.chunk_tasks.len() >= CHUNK_TASKS {
            return;
        }

        for chunk_pos in camera.unresolved_chunk_load.drain(..) {
            let is_busy = world.chunks.contains_key(&chunk_pos)
                || world.load_chunks.contains(&chunk_pos)
                || world.chunk_tasks.contains_key(&chunk_pos);

            if !is_busy {
                world.load_chunks.push(chunk_pos);

                let index_of_unloading =
                    world.unload_chunks.iter().enumerate().find_map(
                        |(i, pos)| match pos == &chunk_pos {
                            true => Some(i),
                            false => None,
                        },
                    );

                if let Some(i) = index_of_unloading {
                    world.unload_chunks.remove(i);
                }
            }
        }
    }
}

fn scan_chunk_unload(
    mut cameras: Query<Mut<MainCamera>>,
    mut world: ResMut<WorldController>,
) {
    for mut camera in cameras.iter_mut() {
        for chunk_pos in camera.unresolved_chunk_unload.drain(..) {
            let is_busy = !world.chunks.contains_key(&chunk_pos);
            if !is_busy {
                world.unload_chunks.push(chunk_pos);
            }
        }
    }
}

fn scan_mesh_unload(mut cameras: Query<&mut MainCamera>, mut world: ResMut<WorldController>) {
    for mut camera in cameras.iter_mut() {
        for chunk_pos in camera.unresolved_mesh_unload.drain(..) {
            world.unload_meshes.push(chunk_pos);
        }
    }
}

fn scan_mesh(mut cameras: Query<Mut<MainCamera>>, mut world: ResMut<WorldController>) {
    for mut camera in cameras.iter_mut() {
        if world.mesh_tasks.len() >= MESH_TASKS {
            return;
        }

        for chunk_pos in camera.unresolved_mesh_load.drain(..) {
            let busy = world.load_meshes.contains(&chunk_pos);

            if !busy {
                world.load_meshes.push(chunk_pos);

                let index_of_unloading =
                    world.unload_meshes.iter().enumerate().find_map(
                        |(i, pos)| match pos == &chunk_pos {
                            true => Some(i),
                            false => None,
                        },
                    );
                if let Some(i) = index_of_unloading {
                    world.unload_meshes.remove(i);
                }
            }
        }
    }
}