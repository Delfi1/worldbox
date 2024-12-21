use bevy::asset::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Block {
    Air,
    Grass,
    Dirt,
}

impl Block {
    /// Returns block texture option by id
    pub fn texture(&self) -> Option<AssetPath<'static>> {
        match self {
            Self::Grass => Some("grass.png".into()),
            Self::Dirt => Some("dirt.png".into()),
            _ => None
        }
    }
}