//! Main chunks objects data;

use std::sync::*;
use bevy::{
    prelude::*,
    asset::*,
};
use serde::{Serialize, Deserialize};
use ordermap::OrderMap;
use rand::seq::SliceRandom;

fn _random<T>(vec: &Vec<T>) -> &T {
    vec.choose(&mut rand::thread_rng()).unwrap()
}

#[derive(Clone, Serialize, Deserialize)]
// Contains all blocks data by id
// todo: Change option to Model Type
pub struct Blocks(pub OrderMap<u8, Option<AssetPath<'static>>>);

impl Default for Blocks {
    fn default() -> Self {
        Self(OrderMap::from([
            (0, None),
            (1, Some("dirt.png".into())),
            (2, Some("grass.png".into())),
            (3, Some("stone.png".into())),
            (4, Some("brick.png".into())),
        ]))
    }
}

#[derive(Resource, Clone)]
pub struct BlocksHandler(Arc<OrderMap<u8, Option<Handle<Image>>>>);

impl BlocksHandler {
    pub fn new(assets: &AssetServer, blocks: Blocks) -> Self {
        let data = blocks.0.into_iter()
            .map(|(i, o)| (i, o.and_then(|path| Some(assets.load(path)))));

        Self(Arc::new(OrderMap::from_iter(data)))
    }

    pub fn textures(&self) -> Vec<&Option<Handle<Image>>> {
        self.0.values().collect()
    }

    pub fn is_meshable(&self, block: u8) -> bool {
        self.0.get(&block).unwrap().is_some()
    }

    /// Returns all blocks vec
    pub fn all(&self) -> Vec<u8> {
        self.0.keys().copied().collect()
    }
}

#[derive(Debug, Clone)]
#[repr(transparent)]
/// Chunks contains data in YXZ coordinate
pub struct RawChunk(Vec<u8>);

impl RawChunk {
    pub const SIZE: usize = 32;
    pub const SIZE_I32: i32 = Self::SIZE as i32;
    pub const SIZE_F32: f32 = Self::SIZE as f32;
    pub const SIZE_P3: usize = Self::SIZE.pow(3);

    pub fn global(pos: Vec3) -> IVec3 {
        (pos / Self::SIZE_F32).floor().as_ivec3()
    }

    pub fn relative(pos: Vec3) -> IVec3 {
        pos.floor().as_ivec3() % Self::SIZE_I32
    }

    /// XZY coord system
    pub fn block_index(pos: IVec3) -> usize {
        let x = pos.x % Self::SIZE_I32;
        let z = pos.z * Self::SIZE_I32;
        let y = pos.y * Self::SIZE_I32.pow(2);

        (x + y + z) as usize
    }

    pub async fn generate(_blocks: BlocksHandler, pos: IVec3) -> Self {
        let mut chunk = Self::empty();
        if pos.y == 0 {
            for i in 0..Self::SIZE.pow(2)*2 {
                chunk.get_mut()[i] = 2;
            }
        }
        chunk
    }

    /// Create a chunk filled with block
    pub fn filled(block: u8) -> Self {
        Self(std::iter::repeat_n(block, Self::SIZE_P3).collect())
    }

    // Same as RawChunk::filled(0)
    pub fn empty() -> Self {
        Self::filled(0)
    }

    pub fn get(&self) -> &Vec<u8> { &self.0 }

    pub fn get_mut(&mut self) -> &mut Vec<u8> { &mut self.0 }
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

    pub fn get_block(&self, pos: IVec3) -> u8 {
        let x = (pos.x + Self::SIZE_I32) as usize;
        let y = (pos.y + Self::SIZE_I32) as usize;
        let z = (pos.z + Self::SIZE_I32) as usize;
        let chunk = Self::chunk_index(x, y, z);
        let block = Self::block_index(x, y, z);

        self.0[chunk].read().get()[block]
    }
}