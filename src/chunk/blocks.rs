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
// todo models
pub enum ModelType {
    /// Empty model
    Empty,
    /// Standart block type
    Meshable(AssetPath<'static>),
    Crossed(AssetPath<'static>),
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
    Crossed(Handle<Image>),
    Custom(Handle<CustomModel>)
}

impl Model {
    pub fn load(assets: &AssetServer, t: ModelType) -> Self {
        match t {
            ModelType::Empty => Self::Empty,
            ModelType::Meshable(path) => Self::Meshable(assets.load(path)),
            ModelType::Crossed(path) => Self::Crossed(assets.load(path)),
            ModelType::Custom(path) => Self::Custom(assets.load(path))
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
    pub fn texture(&self) -> Option<Handle<Image>> {
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
    pub fn new(assets: &AssetServer, t: BlockType) -> Self {
        Self {
            model: Model::load(assets, t.model),
            collision: t.collision
        }
    }
}

#[derive(Clone)]
/// Contains all blocks assets
pub struct BlocksHandler(Arc<OrderMap<String, Block>>);

impl BlocksHandler {
    pub fn new(assets: &AssetServer, blocks: Blocks) -> Self {
        let data = blocks.0.into_iter()
            .map(|(name, t)| (name, Block::new(assets, t)));

        Self(Arc::new(OrderMap::from_iter(data)))
    }

    /// Return block id (0 if not exists) by name
    pub fn block(&self, name: impl Into<String>) -> u16 {
        self.0.get_index_of(&name.into())
        .and_then(|i| Some(i as u16)).unwrap_or(0)
    }

    /// Get all meshable blocks textures
    pub fn textures(&self) -> Vec<Option<Handle<Image>>> {
        self.0.iter().map(|(_, b)| b.model.texture()).collect()
    }

    /// Is texture drawable with default way?
    pub fn is_meshable(&self, block: u16) -> bool {
        match self.0.get_index(block as usize) {
            Some((_, t)) => t.model.is_meshable(),
            _ => false
        }
    }

    /// Returns all blocks vec
    pub fn all(&self) -> Vec<u16> {
        self.0.keys().enumerate().map(|(i, _)| i as u16).collect()
    }
}