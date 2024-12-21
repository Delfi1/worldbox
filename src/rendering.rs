// Rendering graph for chunk mesh; Main rendering data

use super::{
    chunks::*,
    //
};

/// Main graphics logic structure
#[derive(Debug, Clone)]
pub struct Tile {
    
}

// Block = Tiles -> Mesh
#[derive(Debug, Clone)]
pub struct ChunkMesh(Vec<Tile>);

impl ChunkMesh {
    pub async fn new(refs: ChunksRefs) -> Self {
        // todo
        Self(Vec::new())
    }
}