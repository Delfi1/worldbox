use std::sync::*;
use bevy::math::*;
use super::{
    blocks::*,
    utils::*
};

/// Raw chunk data (blocks)
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct RawChunk(Vec<Block>);

impl RawChunk {
    pub fn filled(block: Block) -> Self {
        Self(std::iter::repeat_n(block, CHUNK_SIZE_P3).collect())
    }
    
    // Same as RawChunk::filled(Block::Air)
    pub fn empty() -> Self {
        Self::filled(Block::Air)
    }

    pub async fn generate(pos: IVec3) -> Self {
        //todo: worldgen
        let mut chunk = Self::empty();
        for i in 0..32 {
            chunk.0[i] = Block::Grass;
        }

        chunk
    }

    pub fn get(&self) -> &Vec<Block> {
        &self.0
    }

    pub fn get_mut(&mut self) -> &mut Vec<Block> {
        &mut self.0
    }
}

#[derive(Debug, Clone)]
#[repr(transparent)]
/// Main logic structure, contains blocks data with RwLock
pub struct Chunk(Arc<RwLock<RawChunk>>);

impl Chunk {
    pub fn new(raw: RawChunk) -> Self {
        Self(Arc::new(RwLock::new(raw)))
    }

    /// Get mut access with ReadWrite-locker
    pub fn write(&self) -> RwLockWriteGuard<RawChunk> {
        self.0.write().unwrap()
    }

    /// Get read access with ReadWrite-locker
    pub fn read(&self) -> RwLockReadGuard<RawChunk> {
        self.0.read().unwrap()
    }
}

// Current; Down; Up; Left; Right; Forward; Back;
#[derive(Debug, Clone)]
pub struct ChunksRefs([Chunk; 7]);

impl ChunksRefs {
    pub fn new(data: [Chunk; 7]) -> Self {
        Self(data)
    }

    /// current, back, left, down
    pub fn get_adjacent_blocks(
        &self,
        pos: IVec3,
    ) -> (Block, Block, Block, Block) {
        let current = self.get_block(pos);
        let back = self.get_block(pos + IVec3::NEG_Z);
        let left = self.get_block(pos + IVec3::NEG_X);
        let down = self.get_block(pos + IVec3::NEG_Y);
        (current, back, left, down)
    }

    fn chunk_index(x: usize, y: usize, z: usize) -> usize {
        let (x, y, z) = (
            (x / CHUNK_SIZE) as i32,
            (y / CHUNK_SIZE) as i32, 
            (z / CHUNK_SIZE) as i32
        );
        
        index_from_offset(IVec3::new(x-1, y-1, z-1))
    }
    
    fn block_index(x: usize, y: usize, z: usize) -> usize {
        let (x, y, z) = (
            (x % CHUNK_SIZE) as i32,
            (y % CHUNK_SIZE) as i32, 
            (z % CHUNK_SIZE) as i32
        );

        vec3_to_index(IVec3::new(x, y, z), CHUNK_SIZE_I32)
    }

    pub fn get_block(&self, pos: IVec3) -> Block {
        let x = (pos.x + CHUNK_SIZE_I32) as usize;
        let y = (pos.y + CHUNK_SIZE_I32) as usize;
        let z = (pos.z + CHUNK_SIZE_I32) as usize;
        let chunk = Self::chunk_index(x, y, z);
        let block = Self::block_index(x, y, z);

        self.0[chunk].read().get()[block]
    }
}