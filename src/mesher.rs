use super::*;
use strum::IntoEnumIterator;
use bevy::math::*;
use bevy::render::{
    mesh::*,
    render_asset::*
};

// Also normal
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumIter)]
pub enum Direction {
    Left, Right, Down, Up, Back, Forward
}

impl Direction {
    pub fn world_sample(&self, axis: i32, row: i32, column: i32) -> IVec3 {
        match self {
            Self::Up => IVec3::new(row, axis + 1, column),
            Self::Down => IVec3::new(row, axis, column),
            Self::Left => IVec3::new(axis, column, row),
            Self::Right => IVec3::new(axis + 1, column, row),
            Self::Forward => IVec3::new(row, column, axis),
            Self::Back => IVec3::new(row, column, axis + 1),
        }
    } 

    pub fn air_sample(&self) -> IVec3 {
        match self {
            Self::Up => IVec3::Y,
            Self::Down => IVec3::NEG_Y,
            Self::Left => IVec3::NEG_X,
            Self::Right => IVec3::X,
            Self::Forward => IVec3::NEG_Z,
            Self::Back => IVec3::Z,
        }
    }

    pub fn negate_axis(&self) -> i32 {
        match self {
            Self::Up => 1,
            Self::Down => 0,
            Self::Left => 0,
            Self::Right => 1,
            Self::Forward => 0,
            Self::Back => 1,
        }
    }

    pub fn reverse_order(&self) -> bool {
        match self {
            Self::Up => true,
            Self::Down => false,
            Self::Left => false,
            Self::Right => true,
            Self::Forward => true,
            Self::Back => false,
        }
    }
    
    pub fn to_u32(&self) -> u32 {
        match self {
            Self::Up => 0,
            Self::Left => 1,
            Self::Right => 2,
            Self::Forward => 3,
            Self::Back => 4,
            Self::Down => 5,
        }
    }
}

pub struct Quad {x: usize, y: usize, w: usize, h: usize}
impl Quad {
    pub fn new(x: usize, y: usize, w: usize, h: usize) -> Self {
        Self { x, y, w, h }
    }

    pub fn vertices(self, dir: Direction, axis: i32, block: Block) -> Vec<Vertex> {
        let axis = axis + dir.negate_axis();
        let face = block.uvs(dir.to_u32());
        let mut vertices = Vec::new();

        // Fixme: make normal culled mesher algoritm
        for i in 0..self.w {
            for j in 0..self.h {
                let x = self.x + i;
                let y = self.y + j;

                let v1 = Vertex::new(
                    dir.world_sample(axis, x as i32, y as i32), 
                    dir,
                    Vec2::from(face[0])
                );
        
                let v2 = Vertex::new(
                    dir.world_sample(axis, (x + 1) as i32, y as i32), 
                    dir,
                    Vec2::from(face[1])
                );
        
                let v3 = Vertex::new(
                    dir.world_sample(axis, (x + 1) as i32, (y + 1) as i32), 
                    dir,
                    Vec2::from(face[2])
                );
        
                let v4 = Vertex::new(
                    dir.world_sample(axis, x as i32, (y + 1) as i32), 
                    dir,
                    Vec2::from(face[3])
                );
                
                let mut new = std::collections::VecDeque::from([v1, v2, v3, v4]);
                if dir.reverse_order() {
                    let o = new.split_off(1);
                    o.into_iter().rev().for_each(|i| new.push_back(i));
                }
    
                vertices.extend(new.into_iter());
            }
        }

        vertices
    }
}

/// Pocket of vertex data
/// [6]bits - X
/// [6]bits - Y
/// [6]bits - Z
/// [3]bits - Face && Uy
/// [10]bits - UVx
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    data: u32,
    uv: Vec2
}

impl Vertex {
    pub fn new(local: IVec3, dir: Direction, uv: Vec2) -> Self {
        let data = local.x as u32
        | (local.y as u32) << 6u32
        | (local.z as u32) << 12u32
        | (dir.to_u32()) << 18u32;
        
        Self {data, uv}
    }
}   

/// All mesh vertices
#[derive(Debug, Default, Clone)]
#[repr(transparent)]
pub struct ChunkMesh {
    vertices: Vec<Vertex>
}

impl ChunkMesh {
    fn generate_quads(mut data: [u32; RawChunk::SIZE]) -> Vec<Quad> {
        let mut quads = Vec::new();
        let size = RawChunk::SIZE as u32;

        for row in 0..data.len() {
            let mut y = 0u32;
            while y < size {
                y += (data[row] >> y).trailing_zeros();
                if y >= size { continue; }

                let h = (data[row] >> y).trailing_ones();
                let h_mask = u32::checked_shl(1, h)
                    .map_or(!0, |v| v - 1);
                let mask = h_mask << y;
                
                let mut w = 1;
                while row + w < RawChunk::SIZE {
                    let next_row = (data[row+w] >> y) & h_mask;
                    if next_row != h_mask {
                        break;
                    }

                    data[row + w] = data[row + w] & !mask;
                    w += 1;
                }
                
                // x, y, w, h
                quads.push(Quad::new(row, y as usize, w, h as usize));
                y += h;
            }
        }

        quads
    }

    fn make_vertices(dir: Direction, refs: &ChunksRefs) -> Vec<Vertex> {
        let mut vertices = Vec::with_capacity(512);
        let size = RawChunk::SIZE_I32;

        for axis in 0..size {
            for block in Block::meshables() {
                let mut data = [0u32; 32];
                for i in 0..size.pow(2) {
                    let row = i % size;
                    let column = i / size;
                    let pos = dir.world_sample(axis, row, column);
                    let (current, neg_z) =
                        (refs.get_block(pos), refs.get_block(pos + dir.air_sample()));

                    if current != block { continue; }
                    let is_meshable = current.is_meshable() && !neg_z.is_meshable();
                    data[row as usize] = ((1 << column) * is_meshable as u32) | data[row as usize];
                }

                let quads = Self::generate_quads(data);
                quads.into_iter().for_each(|q| vertices.extend(q.vertices(dir, axis, block)));
            }
        }

        vertices
    }

    pub async fn build(refs: ChunksRefs) -> Option<Self> {
        let mut mesh = Self::default();

        // Apply all directions
        for dir in Direction::iter() {
            mesh.vertices.extend(Self::make_vertices(dir, &refs));
        }
        
        if !mesh.vertices.is_empty() {
            Some(mesh)
        } else {
            None
        }
    }

    pub fn generate_indices(&self) -> Vec<u32> {
        let indices_count = self.vertices.len() / 4;
        let mut indices = Vec::<u32>::with_capacity(indices_count);
        (0..indices_count).into_iter().for_each(|vert_index| {
            let vert_index = vert_index as u32 * 4u32;
            indices.push(vert_index);
            indices.push(vert_index + 1);
            indices.push(vert_index + 2);
            indices.push(vert_index);
            indices.push(vert_index + 2);
            indices.push(vert_index + 3);
        });
        indices
    }

    pub fn data(&self) -> Vec<u32> {
        self.vertices.iter().map(|v| v.data).collect()
    }

    pub fn uvs(&self) -> Vec<Vec2> {
        self.vertices.iter().map(|v| v.uv).collect()
    }

    pub fn spawn(self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );

        let indices = self.generate_indices();
        mesh.insert_attribute(ATTRIBUTE_DATA, self.data());
        mesh.insert_attribute(ATTRIBUTE_UV, self.uvs());
        mesh.insert_indices(Indices::U32(indices));

        mesh
    }
}
