use bevy::{
    prelude::*,
    pbr::*,
    render::{
        mesh::*,
        render_resource::*,
    }
};

#[derive(Resource)]
//todo:
pub struct _GlobalTexture(Handle<Image>);

pub const ATTRIBUTE_POSITION: MeshVertexAttribute =
    MeshVertexAttribute::new("position", 536618, VertexFormat::Float32x3);

pub const ATTRIBUTE_NORMAL: MeshVertexAttribute =
    MeshVertexAttribute::new("normal_index", 639118, VertexFormat::Uint32);

pub const ATTRIBUTE_COLOR: MeshVertexAttribute =
    MeshVertexAttribute::new("color", 687118, VertexFormat::Float32x3);

#[derive(Clone, Asset, Reflect, AsBindGroup, Debug)]
pub struct ChunkMaterial {
    #[uniform(0)]
    pub reflectance: f32,
    #[uniform(0)]
    pub perceptual_roughness: f32,
    #[uniform(0)]
    pub metallic: f32, 
}

impl Default for ChunkMaterial {
    fn default() -> Self {
        Self {
            reflectance: 0.5,
            perceptual_roughness: 1.0,
            metallic: 0.01,
        }
    }
}

impl Material for ChunkMaterial {
    fn vertex_shader() -> ShaderRef {
        "chunk.wgsl".into()
    }
    fn fragment_shader() -> ShaderRef {
        "chunk.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            ATTRIBUTE_POSITION.at_shader_location(0),
            ATTRIBUTE_NORMAL.at_shader_location(1),
            ATTRIBUTE_COLOR.at_shader_location(2)
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

pub struct RenderingPlugin;
impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<ChunkMaterial>::default());
    }
}   
