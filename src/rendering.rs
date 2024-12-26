//todo: Rendering graph for chunk mesh; Main rendering data

use bevy::{
    asset::*,
    prelude::*,
    render::{
        mesh::*, 
        render_resource::*
    }
};

use crate::utils;

use super::{
    blocks::*,
    chunks::*,
    utils::{index_to_ivec3, CHUNK_SIZE_P3},
};

/// Block face direction
#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Left,
    Right,
    Down,
    Up,
    Back,
    Forward,
}

impl Direction {
    pub fn normal(&self) -> usize {
        match self {
            Self::Left => 0,
            Self::Right => 1,
            Self::Down => 2,
            Self::Up => 3,
            Self::Back => 4,
            Self::Forward => 5,
        }
    }
}

/// Block face
pub struct Face {
    pub corners: [[f32; 3]; 4],
    pub uvs: [[f32; 2]; 4]
}

impl Face {
    pub fn new(pos: Vec3, block: &Block, dir: Direction) -> Self {
        let texture = block.texture().unwrap();

        match dir {
            Direction::Left => Self {
                corners: [
                    [pos.x, pos.y, pos.z],
                    [pos.x, pos.y, pos.z + 1.],
                    [pos.x, pos.y + 1., pos.z + 1.],
                    [pos.x, pos.y + 1., pos.z],
                ],
                uvs: block.face(texture, 0),
            },
            Direction::Right => Self {
                corners: [
                    [pos.x, pos.y + 1.0, pos.z],
                    [pos.x, pos.y + 1.0, pos.z + 1.0],
                    [pos.x, pos.y, pos.z + 1.0],
                    [pos.x, pos.y, pos.z],
                ],
                uvs: block.face(texture, 1),
            },
            Direction::Down => Self {
                corners: [
                    [pos.x, pos.y, pos.z],
                    [pos.x + 1.0, pos.y, pos.z],
                    [pos.x + 1.0, pos.y, pos.z + 1.0],
                    [pos.x, pos.y, pos.z + 1.0],
                ],
                uvs: block.face(texture, 2),
            },
            Direction::Up => Self {
                corners: [
                    [pos.x, pos.y, pos.z + 1.0],
                    [pos.x + 1.0, pos.y, pos.z + 1.0],
                    [pos.x + 1.0, pos.y, pos.z],
                    [pos.x, pos.y, pos.z],
                ],
                uvs: block.face(texture, 3),
            },
            Direction::Back => Self {
                corners: [
                    [pos.x, pos.y, pos.z],
                    [pos.x, pos.y + 1.0, pos.z],
                    [pos.x + 1.0, pos.y + 1.0, pos.z],
                    [pos.x + 1.0, pos.y, pos.z],
                ],
                uvs: block.face(texture, 4),
            },
            Direction::Forward => Self {
                corners: [
                    [pos.x + 1.0, pos.y, pos.z],
                [pos.x + 1.0, pos.y + 1.0, pos.z],
                [pos.x, pos.y + 1.0, pos.z],
                [pos.x, pos.y, pos.z],
                ],
                uvs: block.face(texture, 5),
            },
        }
    }
}

/// Main graphics logic structure
#[derive(Debug, Clone)]
pub struct Vertex {
    position: Vec3,
    uv: Vec2,
    // (also a normal id)
    side: usize
}

#[derive(Debug, Clone, Default)]
pub struct ChunkMesh { 
    vertices: Vec<Vertex>,
    indices: Vec<u32>
}

impl ChunkMesh {
    pub fn push_face(&mut self, pos: Vec3, block: &Block, dir: Direction) {
        let face = Face::new(pos, block, dir);
        for i in 0..4 {
            self.vertices.push(Vertex {
                position: Vec3::from(face.corners[i]),
                side: dir.normal(),
                uv: Vec2::from(face.uvs[i])
            });
        }
    }

    pub async fn build(refs: ChunksRefs) -> Option<Self> {
        let mut mesh = Self::default();
        for i in 0..CHUNK_SIZE_P3 {
            // Get near blocks
            let pos = index_to_ivec3(i);
            let (current, back, left, down) = refs.get_adjacent_blocks(pos);
            let local = pos.as_vec3();

            if current.meshable() {
                println!("{:?}", current);

                if !left.meshable() {
                    mesh.push_face(local, &current, Direction::Left);
                }
                if !back.meshable() {
                    mesh.push_face(local, &current, Direction::Back);
                }
                if !down.meshable() {
                    mesh.push_face(local, &current, Direction::Down);
                }
            } else {
                if left.meshable() {
                    mesh.push_face(local, &left, Direction::Right);
                }
                if back.meshable() {
                    mesh.push_face(local, &back, Direction::Forward);
                }
                if down.meshable() {
                    mesh.push_face(local, &down, Direction::Up);
                }
            }
        }

        print!("Builing mesh...");
        // Return Some if not empty
        if mesh.vertices.len() != 0 {
            mesh.indices = utils::generate_indices(mesh.vertices.len());
            Some(mesh)
        } else {
            None
        }
    }

    // Collect all mesh positions
    pub fn positions(&self) -> Vec<Vec3> {
        self.vertices.iter().map(|v| v.position).collect()
    }

    pub fn normals(&self) -> Vec<Vec3> {
        self.vertices.iter().map(|v| utils::get_normal(v.side)).collect()
    }

    pub fn uvs(&self) -> Vec<Vec2> {
        self.vertices.iter().map(|v| v.uv).collect()
    }

    pub fn spawn(self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions());
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals());
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs());
        mesh.insert_indices(Indices::U32(self.indices.into()));

        mesh
    }
}
