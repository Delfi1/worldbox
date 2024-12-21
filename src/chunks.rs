use std::sync::*;
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

    pub async fn generate() -> Self {
        //todo: worldgen
        Self::empty()
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
pub struct ChunksRefs {
    pub current: Chunk,
    pub down: Chunk,
    pub up: Chunk,
    pub left: Chunk,
    pub right: Chunk,
    pub forward: Chunk,
    pub back: Chunk
}

impl ChunksRefs {
    pub fn new(data: Vec<Chunk>) -> Option<Self> {
        Some(Self { 
            current: data.get(0).cloned()?,
            down: data.get(1).cloned()?,
            up: data.get(2).cloned()?,
            left: data.get(3).cloned()?,
            right: data.get(4).cloned()?, 
            forward: data.get(5).cloned()?,
            back: data.get(6).cloned()?
        })
    }
}