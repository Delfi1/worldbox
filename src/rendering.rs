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
pub struct GlobalTexture(pub Handle<Image>);

impl GlobalTexture {
    pub fn inner(&self) -> Handle<Image> {
        self.0.clone()
    }
}

pub const ATTRIBUTE_DATA: MeshVertexAttribute =
    MeshVertexAttribute::new("data", 536618, VertexFormat::Uint32);

#[derive(Clone, Asset, Reflect, AsBindGroup, Debug)]
pub struct ChunkMaterial {
    #[uniform(0)]
    pub roughness: f32,
    #[texture(1)]
    #[sampler(2)]
    texture: Handle<Image>,
}

impl ChunkMaterial {
    pub fn new(texture: GlobalTexture) -> Self {
        Self {
            roughness: 0.3,
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
                    mag_filter: ImageFilterMode::Nearest,
                    min_filter: ImageFilterMode::Nearest,
                    mipmap_filter: ImageFilterMode::Nearest,
                    lod_max_clamp: 0.0,
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
