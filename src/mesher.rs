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

pub struct Face {x: i32, y: i32}
impl Face {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub const UVS: [UVec2; 4] = [
        UVec2::new(1, 1),
        UVec2::new(0, 1),
        UVec2::new(0, 0),
        UVec2::new(1, 0)
    ];

    pub fn vertices(self, dir: Direction, axis: i32, block: u8) -> Vec<Vertex> {
        let axis = axis + dir.negate_axis();
        
        let v1 = Vertex::new(
            dir.world_sample(axis, self.x, self.y), 
            dir,
            block as u32,
            &Self::UVS[0]
        );

        let v2 = Vertex::new(
            dir.world_sample(axis, self.x + 1, self.y), 
            dir,
            block as u32,
            &Self::UVS[1]
        );

        let v3 = Vertex::new(
            dir.world_sample(axis, self.x + 1, self.y + 1), 
            dir,
            block as u32,
            &Self::UVS[2]
        );

        let v4 = Vertex::new(
            dir.world_sample(axis, self.x, self.y + 1), 
            dir,
            block as u32,
            &Self::UVS[3]
        );
        
        let mut new = std::collections::VecDeque::from([v1, v2, v3, v4]);
        if dir.reverse_order() {
            let o = new.split_off(1);
            o.into_iter().rev().for_each(|i| new.push_back(i));
        }

        Vec::from(new)
    }
}

/// Pocket of vertex data
/// [6]bits - X
/// [6]bits - Y
/// [6]bits - Z
/// [3]bits - Face
/// [7]bits - texture_x
#[derive(Debug, Clone, Copy)]
pub struct Vertex(u32);

impl Vertex {
    pub fn new(local: IVec3, dir: Direction, block: u32, uv: &UVec2) -> Self {
        let data = local.x as u32
        | (local.y as u32) << 6u32
        | (local.z as u32) << 12u32
        | (dir.to_u32()) << 18u32
        | (block) << 21u32 // Block id also texture id in binding array 
        | (uv.x) << 28u32  // uv May be only 0 or 1
        | (uv.y) << 29u32; 
        
        Self(data)
    }
}   

/// All mesh vertices
#[derive(Debug, Default, Clone)]
#[repr(transparent)]
pub struct ChunkMesh {
    vertices: Vec<Vertex>
}

impl ChunkMesh {
    fn make_vertices(dir: Direction, handler: &BlocksHandler, refs: &ChunksRefs) -> Vec<Vertex> {
        let mut vertices = Vec::with_capacity(512);
        let size = RawChunk::SIZE_I32;

        // Culled meshser
        for axis in 0..size {
            for i in 0..size.pow(2) {
                let row = i % size;
                let column = i / size;
                let pos = dir.world_sample(axis, row, column);
                let (current, neg_z) =
                    (refs.get_block(pos), refs.get_block(pos + dir.air_sample()));

                if handler.is_meshable(current) && !handler.is_meshable(neg_z) {
                    let face = Face::new(row, column);
                    vertices.extend(face.vertices(dir, axis, current));
                }
            }
        }

        vertices
    }

    pub async fn build(handler: BlocksHandler, refs: ChunksRefs) -> Option<Mesh> {
        let mut mesh = Self::default();

        // Apply all directions
        for dir in Direction::iter() {
            mesh.vertices.extend(Self::make_vertices(dir, &handler, &refs));
        }           
        
        if !mesh.vertices.is_empty() {
            Some(mesh.spawn())
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

    pub fn spawn(self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );

        let indices = self.generate_indices();
        let data: Vec<_> = self.vertices.into_iter().map(|v| v.0).collect();
        mesh.insert_attribute(ATTRIBUTE_DATA, data);
        mesh.insert_indices(Indices::U32(indices));

        mesh
    }
}
