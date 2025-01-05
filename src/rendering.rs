use bevy::{
    prelude::*,
    image::*,
    pbr::*,
    render::{
        mesh::*,
        render_resource::*,
    }
};

#[derive(Resource, Clone)]
pub struct GlobalTexture(Handle<Image>);

impl GlobalTexture {
    pub fn inner(&self) -> Handle<Image> {
        self.0.clone()
    }
}

pub const ATTRIBUTE_DATA: MeshVertexAttribute =
    MeshVertexAttribute::new("data", 536618, VertexFormat::Uint32);

pub const ATTRIBUTE_UV: MeshVertexAttribute =
    MeshVertexAttribute::new("uv", 586618, VertexFormat::Float32x2);


#[derive(Clone, Asset, Reflect, AsBindGroup, Debug)]
pub struct ChunkMaterial {
    #[uniform(0)]
    pub reflectance: f32,
    #[uniform(0)]
    pub perceptual_roughness: f32,
    #[uniform(0)]
    pub metallic: f32,
    #[texture(1)]
    #[sampler(2)]
    texture: Handle<Image>,
}

impl ChunkMaterial {
    pub fn new(texture: GlobalTexture) -> Self {
        Self {
            reflectance: 0.5,
            perceptual_roughness: 1.0,
            metallic: 0.01,
            texture: texture.inner()
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
            ATTRIBUTE_DATA.at_shader_location(0),
            ATTRIBUTE_UV.at_shader_location(1),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

fn init(
    mut commands: Commands,
    assets: ResMut<AssetServer>,    
) {
    commands.insert_resource(
        GlobalTexture(assets.load_with_settings("textures.png", |s: &mut _| {
            *s = ImageLoaderSettings {
                sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
                    // rewriting mode to repeat image,
                    address_mode_u: ImageAddressMode::Repeat,
                    address_mode_v: ImageAddressMode::Repeat,
                    mag_filter: ImageFilterMode::Nearest,
                    min_filter: ImageFilterMode::Nearest,
                    mipmap_filter: ImageFilterMode::Linear,
                    ..default()
                }),
                ..default()
            }
        },)));
}

pub struct RenderingPlugin;
impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<ChunkMaterial>::default())
            .add_systems(Startup, init);
    }
}   
