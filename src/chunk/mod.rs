//! Main chunks objects data;

mod blocks;
use std::sync::*;
use bevy::prelude::*;
use rand::seq::SliceRandom;
pub use blocks::*;

fn _random<T>(vec: &Vec<T>) -> &T {
    vec.choose(&mut rand::thread_rng()).unwrap()
}

#[derive(Debug, Clone)]
#[repr(transparent)]
/// Chunks contains data in YXZ coordinate
pub struct RawChunk(Vec<u16>);

impl RawChunk {
    pub const SIZE: usize = 32;
    pub const SIZE_I32: i32 = Self::SIZE as i32;
    pub const SIZE_F32: f32 = Self::SIZE as f32;
    pub const SIZE_P3: usize = Self::SIZE.pow(3);

    /// Get chunk global pos
    pub fn global(pos: Vec3) -> IVec3 {
        (pos / Self::SIZE_F32).floor().as_ivec3()
    }

    /// Get current block pos relative to current chunk
    pub fn relative(pos: Vec3) -> IVec3 {
        (pos.floor().as_ivec3() % Self::SIZE_I32).map(|v| {
            if v < 0 { return v + Self::SIZE_I32 }
            return v
        })
    }

    /// XZY coord system
    pub fn block_index(pos: IVec3) -> usize {
        let x = pos.x % Self::SIZE_I32;
        let z = pos.z * Self::SIZE_I32;
        let y = pos.y * Self::SIZE_I32.pow(2);

        (x + y + z) as usize
    }

    /// Main generate function - WIP
    pub async fn generate(_blocks: BlocksHandler, pos: IVec3) -> Self {
        if pos.y == 0 {
            let mut chunk = Self::empty();
            for i in 0..Self::SIZE.pow(2) {
                chunk.get_mut()[i] = _blocks.block("Grass");
            }

            chunk
        } else {
            Self::empty()
        }
    }

    /// Get all blocks above cursore by radius, absolute pos and vector u (camera forward)
    pub fn under_cursor(mut current: Vec3, u: Vec3, r: usize) -> Vec<Vec3> {        
        let delta = 0.01;
        let mut stored = Vec::with_capacity(r);
        let mut blocks = Vec::with_capacity(r);
        while blocks.len() < r {
            current.x += u.x*delta;
            current.y += u.y*delta;
            current.z += u.z*delta;

            let next = current.as_ivec3();
            if !stored.contains(&next) {
                stored.push(next);
                blocks.push(current);
            }
        }
        blocks
    }

    /// Create a chunk filled with block
    pub fn filled(block: u16) -> Self {
        Self(std::iter::repeat_n(block, Self::SIZE_P3).collect())
    }

    // Same as RawChunk::filled(0)
    pub fn empty() -> Self {
        Self::filled(0)
    }

    pub fn get(&self) -> &Vec<u16> { &self.0 }

    pub fn get_mut(&mut self) -> &mut Vec<u16> { &mut self.0 }
}

// All voxels pocket data
#[derive(Debug, Clone)]
pub struct Chunk(Arc<RwLock<RawChunk>>);

impl Chunk {
    pub fn new(raw: RawChunk) -> Self {
        Self(Arc::new(RwLock::new(raw)))
    }

    pub fn read(&self) -> RwLockReadGuard<RawChunk> {
        self.0.read().unwrap()
    }

    pub fn write(&self) -> RwLockWriteGuard<RawChunk> {
        self.0.write().unwrap()
    }
}

#[derive(Debug, Clone)]
#[repr(transparent)]
/// Contains all near chunks:
/// 
/// Current; Left; Right; Down; Up; Back; Forward;
pub struct ChunksRefs([Chunk; 7]);

impl ChunksRefs {
    pub const OFFSETS: [IVec3; 7] = [
        IVec3::ZERO,  // current
        IVec3::NEG_Y, // down
        IVec3::Y,     // up
        IVec3::NEG_X, // left
        IVec3::X,     // right
        IVec3::NEG_Z, // forward
        IVec3::Z,     // back
    ];

    pub const SIZE: usize = RawChunk::SIZE;
    pub const SIZE_I32: i32 = RawChunk::SIZE_I32;

    pub fn offsets(pos: IVec3) -> Vec<IVec3> {
        Self::OFFSETS.iter().map(|o| pos + o).collect()
    }

    pub fn new(data: [Chunk; 7]) -> Self {
        Self(data)
    }

    fn offset_index(v: IVec3) -> usize {
        Self::OFFSETS.iter().position(|p| p==&v).unwrap()
    }

    fn chunk_index(x: usize, y: usize, z: usize) -> usize {
        let (cx, cy, cz) = (
            (x / Self::SIZE) as i32,
            (y / Self::SIZE) as i32, 
            (z / Self::SIZE) as i32
        );
        
        Self::offset_index(IVec3::new(cx, cy, cz) - IVec3::ONE)
    }
    
    fn block_index(x: usize, y: usize, z: usize) -> usize {
        let (bx, by, bz) = (
            (x % Self::SIZE) as i32,
            (y % Self::SIZE) as i32,
            (z % Self::SIZE) as i32
        );

        RawChunk::block_index(IVec3::new(bx, by, bz))
    }

    pub fn get_block(&self, pos: IVec3) -> u16 {
        let x = (pos.x + Self::SIZE_I32) as usize;
        let y = (pos.y + Self::SIZE_I32) as usize;
        let z = (pos.z + Self::SIZE_I32) as usize;
        let chunk = Self::chunk_index(x, y, z);
        let block = Self::block_index(x, y, z);

        self.0[chunk].read().get()[block]
    }
}