//! Main chunks objects data;

use std::sync::*;
use bevy::{
    prelude::*,
    utils::*
};
use strum::IntoEnumIterator;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[derive(strum::EnumIter)]
#[repr(u8)]
pub enum Block {
    Air, 
    Dirt,
    Grass, 
    Stone
}

impl Block {
    /// Return all meshsable block types
    pub fn meshables() -> Vec<Block> {
        Block::iter().filter(|b| b.is_meshable()).collect()
    }

    pub fn is_meshable(&self) -> bool {
        match self {
            Self::Air => false,
            _ => true
        }
    }

    pub fn uvs(self, dir: u32) -> [[f32; 2]; 4] {
        let id = (self as u8) as f32;
        let dir = dir as f32;
        // "0-index" positions
        let (x0, y0) = (id, dir);
        // Sizes
        let (sx, sy) = (256.0, 6.0);

        [
         [(x0+1.0)/sx, (y0+1.0)/sy],
         [x0/sx, (y0+1.0)/sy],
         [x0/sx, y0/sy],
         [(x0+1.0)/sx, y0/sy],
        ]
    }
}

#[derive(Debug, Clone)]
#[repr(transparent)]
/// Chunks contains data in YXZ coordinate
pub struct RawChunk(Vec<Block>);

impl RawChunk {
    pub const SIZE: usize = 32;
    pub const SIZE_I32: i32 = Self::SIZE as i32;
    pub const SIZE_F32: f32 = Self::SIZE as f32;
    pub const SIZE_P3: usize = Self::SIZE.pow(3);

    /// YXZ coord system
    fn block_index(pos: IVec3) -> usize {
        let x = pos.x % Self::SIZE_I32;
        let z = pos.z * Self::SIZE_I32;
        let y = pos.y * Self::SIZE_I32.pow(2);

        (x + y + z) as usize
    }

    pub async fn generate(pos: IVec3) -> Self {
        let mut chunk = Self::empty();
        
        for x in 0..Self::SIZE_I32 {
            for z in 0..Self::SIZE_I32 {
                if x % 2 == 0 && z % 2 == 0 {
                    let i = Self::block_index(IVec3::new(x, 16, z));
                    chunk.get_mut()[i] = Block::Grass;
                }
            }
        }

        return chunk
    }   

    /// Create a chunk full filled with block
    pub fn filled(block: Block) -> Self {
        Self(std::iter::repeat_n(block, Self::SIZE_P3).collect())
    }

    // Same as RawChunk::filled(Block::Air)
    pub fn empty() -> Self {
        Self::filled(Block::Air)
    }

    pub fn get(&self) -> &Vec<Block> { &self.0 }

    pub fn get_mut(&mut self) -> &mut Vec<Block> { &mut self.0 }
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

    pub fn get_block(&self, pos: IVec3) -> Block {
        let x = (pos.x + Self::SIZE_I32) as usize;
        let y = (pos.y + Self::SIZE_I32) as usize;
        let z = (pos.z + Self::SIZE_I32) as usize;
        let chunk = Self::chunk_index(x, y, z);
        let block = Self::block_index(x, y, z);

        self.0[chunk].read().get()[block]
    }
}