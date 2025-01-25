use std::sync::*;
use bevy::{
    asset::*, prelude::*
};
use serde::{Serialize, Deserialize};
use ordermap::OrderMap;

/// Contains block/model collision box
#[derive(Clone, Serialize, Deserialize)]
pub struct CollisionBox {
    min: Vec3,
    max: Vec3
}

impl CollisionBox {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }
}

impl Default for CollisionBox {
    /// Default meshable block collision
    fn default() -> Self {
        Self { min: Vec3::ZERO, max: Vec3::ONE }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
// Models can be two types:
// Meshable or non-Meshable.
pub enum ModelType {
    /// Block without textures
    Empty,
    /// Standart block type
    Meshable(AssetPath<'static>),
    // Custom textures, non-meshable block type
    Custom(AssetPath<'static>)
}

impl ModelType {
    pub fn collision(&self) -> Option<CollisionBox> {
        match self {
            Self::Meshable(_) => Some(CollisionBox::default()),
            _ => None
        }
    }
}

// todo: blocks "schemes" for worldgen
// pub struct Scheme 

#[derive(Clone, Serialize, Deserialize)]
pub struct BlockType {
    model: ModelType,
    collision: Option<CollisionBox>,
}

impl BlockType {
    /// Default block type from ModelType
    pub fn new(model: ModelType) -> Self {
        Self {
            collision: model.collision(),
            model,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
// Contains all blocks data by name
pub struct Blocks(pub OrderMap<String, BlockType>);

impl Default for Blocks {
    fn default() -> Self {
        Self(OrderMap::from([
            ("Air".into(), BlockType::new(ModelType::Empty)),
            ("Dirt".into(), BlockType::new(ModelType::Meshable("dirt.png".into()))),
            ("Grass".into(), BlockType::new(ModelType::Meshable("grass.png".into()))),
            ("Stone".into(), BlockType::new(ModelType::Meshable("stone.png".into()))),
        ]))
    }
}

/// todo: Custom model load logic, rendering etc
#[derive(Clone, Asset, TypePath)]
pub struct CustomModel {
    //todo
}

/// Contains all models textures & data
pub enum Model {
    Empty,
    Meshable(Handle<Image>),
    Custom(Handle<CustomModel>)
}

impl Model {
    // Load block model asset
    pub fn load(loader: &mut LoadContext<'_>, t: ModelType) -> Self {
        match t {
            ModelType::Empty => Self::Empty,
            ModelType::Meshable(path) => Self::Meshable(loader.load(path)),
            ModelType::Custom(path) => Self::Custom(loader.load(path))
        }
    }

    /// Is model meshable?
    pub fn is_meshable(&self) -> bool {
        match self {
            Self::Meshable(_) => true,
            _ => false
        }
    }

    /// Get meshable block texture if exists
    pub fn meshable_texture(&self) -> Option<Handle<Image>> {
        match self {
            Self::Meshable(h) => Some(h.clone()),
            _ => None
        }
    }
}

// Block data like model, collision, etc
pub struct Block {
    pub model: Model,
    pub collision: Option<CollisionBox>
}

impl Block {
    pub fn new(loader: &mut LoadContext<'_>, t: BlockType) -> Self {
        Self {
            model: Model::load(loader, t.model),
            collision: t.collision
        }
    }
}

#[derive(Clone)]
/// Contains all blocks assets
pub struct BlocksHandler(Arc<OrderMap<String, Block>>);

impl BlocksHandler {
    pub fn new(loader: &mut LoadContext<'_>, blocks: Blocks) -> Self {
        let data = blocks.0.into_iter()
            .map(|(name, t)| (name, Block::new(loader, t)));

        Self(Arc::new(OrderMap::from_iter(data)))
    }

    pub fn is_meshable(&self, index: u16) -> bool {
        match self.0.get_index(index as usize) {
            Some((_, b)) => b.model.is_meshable(),
            None => false
        }
    }

    /// Return block by name
    pub fn block(&self, name: impl Into<String>) -> Option<&Block> {
        self.0.get(&name.into())
    }

    /// Return block id by name
    pub fn get(&self, name: impl Into<String>) -> u16 {
        self.0.get_index_of(&name.into()).unwrap() as u16
    }

    /// Get all meshable blocks textures
    pub fn meshable_textures(&self) -> Vec<Option<Handle<Image>>> {
        self.0.iter().map(|(_, b)| b.model.meshable_texture()).collect()
    }

    /// Returns all blocks vec
    pub fn all(&self) -> Vec<&Block> {
        self.0.values().collect()
    }

    pub fn ids(&self) -> Vec<u16> {
        self.0.iter().enumerate().map(|(i, _)| i as u16).collect()
    }
}