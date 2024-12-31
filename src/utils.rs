use std::fmt::Debug;

use bevy::math::*;

// Tasks consts
pub const CHUNK_TASKS: usize = 8;
pub const MESH_TASKS: usize = CHUNK_TASKS/2;

// Chunks consts
pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_SIZE_I32: i32 = CHUNK_SIZE as i32;
pub const CHUNK_SIZE_P3: usize = CHUNK_SIZE.pow(3);

pub const CHUNKS_OFFSETS: [IVec3; 7] = [
    IVec3::ZERO,  // current
    IVec3::NEG_Y, // down
    IVec3::Y,     // up
    IVec3::NEG_X, // left
    IVec3::X,     // right
    IVec3::NEG_Z, // forward
    IVec3::Z,     // back
];

pub fn near_chunks(pos: IVec3) -> Vec<IVec3> {
    CHUNKS_OFFSETS.iter().map(|p| pos + p).collect::<Vec<_>>()
}

pub const NORMALS: [Vec3; 6] = [
    Vec3::NEG_X, // left
	Vec3::X,     // right
	Vec3::NEG_Y, // down
	Vec3::Y,     // up
	Vec3::NEG_Z, // forward
	Vec3::Z,     // back
];

pub fn index_from_offset(pos: IVec3) -> usize {
    CHUNKS_OFFSETS.iter().position(|p| p==&pos).unwrap()
}

pub fn get_normal(id: usize) -> Vec3 {
    NORMALS[id]
}

pub fn to_array<T: Debug, const N: usize>(data: Vec<T>) -> [T; N] {
    data.try_into().expect("Wrong size")
}

pub fn vec3_to_index(pos: IVec3, bounds: i32) -> usize {
    let x_i = pos.x % bounds;
    let y_i = pos.y * bounds;
    let z_i = pos.z * bounds.pow(2);
    
    (x_i + y_i + z_i) as usize
}

pub const fn index_to_ivec3(i: usize) -> IVec3 {
    index_to_ivec3_bounds(i, CHUNK_SIZE)
}

pub const fn index_to_ivec3_bounds(i: usize, bounds: usize) -> IVec3 {
    let x = i % bounds;
    let y = (i / bounds) % bounds;
    let z = i / (bounds * bounds);
    IVec3::new(x as i32, y as i32, z as i32)
}

pub fn generate_indices(verts: usize) -> Vec<u32> {
    let indices_count = verts / 4;
    let mut indices = Vec::<u32>::with_capacity(indices_count);
    (0..indices_count).into_iter().for_each(|vert_index| {
        let vert_index = vert_index as u32 * 4u32;
        indices.push(vert_index);
        indices.push(vert_index + 1);
        indices.push(vert_index + 2);
        indices.push(vert_index);
        indices.push(vert_index + 2);
        indices.push(vert_index + 3);
    });
    indices
}

pub fn get_chunk_pos(global: Vec3) -> IVec3 {
    global.as_ivec3() / IVec3::splat(CHUNK_SIZE_I32)
}