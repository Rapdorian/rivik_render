use std::num::NonZeroU32;

use assets::{formats::img::Img, Format};
use image::{GenericImageView, ImageFormat};
use wgpu::{
    ImageCopyTexture, ImageDataLayout, Origin3d, Texture, TextureAspect, TextureDescriptor,
    TextureUsages, TextureViewDescriptor,
};

use crate::context::{device, queue};

pub struct GpuTexture(pub ImageFormat);

impl Format for GpuTexture {
    type Output = (Texture, TextureViewDescriptor<'static>);

    fn parse(&self, r: &assets::Path) -> reerror::Result<Self::Output> {
        let device = device();
        let image = (Img(self.0)).parse(r)?;

        let dimensions = image.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&TextureDescriptor {
            label: None,
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        });

        let desc = TextureViewDescriptor {
            label: None,
            format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: NonZeroU32::new(1),
            base_array_layer: 0,
            array_layer_count: NonZeroU32::new(1),
        };

        // write texture contents
        let img = image.to_rgba8();
        queue().write_texture(
            ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &img,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * texture_size.width),
                rows_per_image: NonZeroU32::new(texture_size.height),
            },
            texture_size,
        );
        Ok((texture, desc))
    }
}
