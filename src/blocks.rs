#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Block {
    Air,
    Grass,
    Dirt,
}

impl Block {
    pub const MESHABLES: &[Block] = &[
        Self::Grass,
        Self::Dirt
    ];

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

    fn size(offset: f32) -> f32 {
        Self::MESHABLES.len() as f32 * offset
    }

    pub fn face(&self, texture: usize, id: usize) -> [[f32; 2]; 4] {
        let texture = texture as f32;
        let id = id as f32;
        
        // textures coords
        let (x0, y0) = (0.0, texture*32.0);
        let (x1, y1) = (6.0*32.0, y0+32.0);
        let offset = 32.0;
        let size = Self::size(offset);

        [
         [(x0+offset*id)/x1, (y0)/size],
         [(x0+offset*(id+1.))/x1, (y0)/size],
         [(x0+offset*(id+1.))/x1, (y1)/size], 
         [(x0+offset*id)/x1, (y1)/size],
        ]
    }
}