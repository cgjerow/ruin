use anyhow::Result;
use image::GenericImageView;
use wgpu::{
    AddressMode, BindGroup, Device, Extent3d, FilterMode, Origin3d, Queue, Sampler,
    SamplerDescriptor, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor,
};

use crate::Asset;

#[derive(Debug)]
pub struct ImageBindGroup {
    bind_group: BindGroup, // store this so child references stay alive
}

impl Asset for ImageBindGroup {}

impl ImageBindGroup {
    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    pub fn from_image_texture(
        device: &Device,
        img: ImageTexture,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        Self {
            bind_group: device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(img.view()),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(img.sampler()),
                    },
                ],
                label: Some("Texture Bind Group"),
            }),
        }
    }
}

#[derive(Debug)]
pub struct ImageTexture {
    _texture: Texture, // store this so child references stay alive
    sampler: Sampler,
    view: TextureView,
}

impl Asset for ImageTexture {}

impl ImageTexture {
    pub fn sampler(&self) -> &Sampler {
        &self.sampler
    }

    pub fn view(&self) -> &TextureView {
        &self.view
    }

    pub fn from_image(
        device: &Device,
        queue: &Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
    ) -> Result<Self> {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let size = Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            TexelCopyTextureInfo {
                aspect: TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
            },
            &rgba,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            _texture: texture,
            view,
            sampler,
        })
    }
}
