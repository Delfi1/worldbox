use bevy::math::*;

// Tasks consts
pub const CHUNK_TASKS: usize = 32;
pub const MESH_TASKS: usize = 8;

// Chunks consts
pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_SIZE_P3: usize = CHUNK_SIZE.pow(3);

pub const CHUNKS_OFFSETS: [IVec3; 7] = [
    IVec3::ZERO,  // current
    IVec3::NEG_Y, // down
    IVec3::Y,  // up
    IVec3::NEG_X, // left
    IVec3::X,  // right
    IVec3::NEG_Z, // forward
    IVec3::Z,  // back
];