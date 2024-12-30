#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Block {
    Air,
    Grass,
    Dirt,
}

impl Block {
    pub fn meshable(&self) -> bool {
        self.texture().is_some()
    }
    
    /// Returns block texture id
    pub fn texture(&self) -> Option<usize> {
        match self {
            Self::Grass => Some(0),
            Self::Dirt => Some(1),
            _ => None
        }
    }

    pub fn face(&self, texture: usize, id: usize) -> [[f32; 2]; 4] {
        let texture = texture as f32;
        let id = id as f32;
        
        // textures coords
        let offset = 0.03125;
        let y0 = texture*offset;
        let y1 = y0+offset;

        [
         [(offset*id), y0],
         [(offset*(id+1.)), y0],
         [(offset*(id+1.)), y1], 
         [(offset*id), y1],
        ]
    }
}