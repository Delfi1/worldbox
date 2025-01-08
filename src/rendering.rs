use std::num::NonZero;

use bevy::{
    prelude::*,
    pbr::*,
    render::{
        mesh::*,
        texture::*,
        render_resource::{
            binding_types::{sampler, texture_2d_array},
            *,
        },
        render_asset::*,
    }
};

use crate::BlocksHandler;

pub const ATTRIBUTE_DATA: MeshVertexAttribute =
    MeshVertexAttribute::new("data", 536618, VertexFormat::Uint32);

//todo: make custom bind group
#[derive(Clone, Asset, Reflect, Debug)]
pub struct ChunkMaterial {
    textures: Vec<Option<Handle<Image>>>,
}

/// Set max textures bind group lenght
pub const MAX_TEXTURES: usize = 64;

impl AsBindGroup for ChunkMaterial {
    type Data = ();
    type Param = (Res<'static, RenderAssets<GpuImage>>, Res<'static, FallbackImage>);

    fn as_bind_group(
        &self,
        layout: &BindGroupLayout,
        render_device: &bevy::render::renderer::RenderDevice,
        param: &mut bevy::ecs::system::SystemParamItem<'_, '_, Self::Param>,
    ) -> Result<PreparedBindGroup<Self::Data>, AsBindGroupError> {
        let mut images = vec![];

        for handle_opt in self.textures.iter() {
            let Some(handle) = handle_opt else {
                images.push(None);
                continue;
            };

            match param.0.get(handle) {
                Some(image) => {images.push(Some(image))},
                None => return Err(AsBindGroupError::RetryNextUpdate),
            }
        }
        
        let fallback_image = &param.1.d2_array;
        let textures = vec![&fallback_image.texture_view; MAX_TEXTURES];
        let mut textures: Vec<_> = textures.into_iter().map(|texture| &**texture).collect();
        for (id, image_opt) in images.into_iter().enumerate() {
            if let Some(image) = image_opt {
                textures[id] = &*image.texture_view;
            }
        }

        let bind_group = render_device.create_bind_group(
            "chunk_bind_group",
            layout,
            &BindGroupEntries::sequential((&textures[..], &fallback_image.sampler)),
        );

        Ok(PreparedBindGroup {
            bindings: vec![],
            bind_group,
            data: (),
        })
    }

    fn unprepared_bind_group(
        &self,
        _layout: &BindGroupLayout,
        _render_device: &bevy::render::renderer::RenderDevice,
        _param: &mut bevy::ecs::system::SystemParamItem<'_, '_, Self::Param>,
    ) -> Result<UnpreparedBindGroup<Self::Data>, AsBindGroupError> {
        Ok(UnpreparedBindGroup {
            bindings: vec![],
            data: ()
        })
    }

    fn bind_group_layout_entries(_render_device: &bevy::render::renderer::RenderDevice) -> Vec<BindGroupLayoutEntry>
        where Self: Sized {
            BindGroupLayoutEntries::with_indices(
            ShaderStages::FRAGMENT,
            (
                (0, texture_2d_array(TextureSampleType::Float { filterable: true })
                        .count(NonZero::<u32>::new(MAX_TEXTURES as u32).unwrap())),
                (1, sampler(SamplerBindingType::Filtering)),
            ),
        )
        .to_vec()
    }
}

impl ChunkMaterial {
    pub fn new(handler: &BlocksHandler) -> Self {
        Self {
            textures: handler.textures().into_iter().cloned().collect()
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

fn proceed_textures(
    assets: Res<AssetServer>,
    blocks: Res<BlocksHandler>,
    mut images: ResMut<Assets<Image>>,
) {
    let textures: Vec<_> = blocks.textures().iter().filter(|t| t.is_some())
        .map(|t| t.as_ref().unwrap()).cloned().collect();
    
    for texture in textures {
        if assets.is_loaded(&texture) {
            let image = images.get(&texture).unwrap();
            if image.texture_descriptor.array_layer_count() != 1 { continue; }

            // If image isn't proceeded yet - reinterpret
            let image = images.get_mut(&texture).unwrap();
            image.reinterpret_stacked_2d_as_array(6);
        }
    }
}

pub struct RenderingPlugin;
impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<ChunkMaterial>::default())
            .add_systems(PreUpdate, proceed_textures);
    }
}   
