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
    pub fn texture(&self) -> Option<[usize; 6]> {
        match self {
            Self::Grass => Some([0, 1, 2, 3, 4, 5]),
            Self::Dirt => Some([1024, 1025, 1026, 1027, 1028, 1029]),
            _ => None
        }
    }

    pub fn face(&self, texture: [usize; 6], id: usize) -> [[f32; 2]; 4] {
        // textures coords
        let offset = 0.03125;
        let (x, y) = (texture[id]%1024, texture[id]/1024);
        let (x0, y0) = ((x as f32)*offset, (y as f32)*offset);

        [
         [x0, y0],
         [x0+offset, y0],
         [x0+offset, y0+offset], 
         [x0, y0+offset],
        ]
    }
}