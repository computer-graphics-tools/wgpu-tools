use super::error::Error;
use super::texture::Texture;
use image::{DynamicImage, GenericImageView};
use nalgebra::SVector;
use wgpu;

pub struct ContextDescriptor<'a, 'b> {
    request_adapter_options: wgpu::RequestAdapterOptions<'a, 'b>,
    device_descriptor: wgpu::DeviceDescriptor<'a>,
    trace_path: Option<&'a std::path::Path>,
}

pub struct Context {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl Context {
    pub async fn new<'a, 'b>(
        instance: wgpu::Instance,
        descriptor: &'a ContextDescriptor<'a, 'b>,
    ) -> Result<Self, Error> {
        let adapter = instance
            .request_adapter(&descriptor.request_adapter_options)
            .await
            .ok_or(Error::RequestingAdapterFailed)?;
        let (device, queue) = adapter
            .request_device(&descriptor.device_descriptor, descriptor.trace_path)
            .await?;

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
        })
    }

    pub async fn default_with_surface<'a>(
        instance: wgpu::Instance,
        compatible_surface: Option<&wgpu::Surface<'a>>,
    ) -> Result<Self, Error> {
        let context = Self::new(
            instance,
            &ContextDescriptor {
                request_adapter_options: wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    force_fallback_adapter: false,
                    compatible_surface,
                },
                device_descriptor: wgpu::DeviceDescriptor::default(),
                trace_path: None,
            },
        )
        .await?;

        Ok(context)
    }

    pub async fn default() -> Result<Self, Error> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        Self::default_with_surface(instance, None).await
    }

    pub fn schedule<O>(&self, operations: O)
    where
        O: Fn(&mut wgpu::CommandEncoder) -> (),
    {
        let mut command_encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Command Encoder"),
                });

        operations(&mut command_encoder);

        let command_buffer = command_encoder.finish();
        self.queue.submit(std::iter::once(command_buffer));
    }

    pub fn depth_texture(&self, width: &u32, height: &u32, label: &str) -> Texture {
        let size = wgpu::Extent3d {
            width: *width,
            height: *height,
            depth_or_array_layers: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Texture::DEPTH_FORMAT,
            view_formats: &[wgpu::TextureFormat::Depth32Float],
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Texture {
            texture,
            view,
            sampler,
        }
    }

    pub fn texture_with_data(
        &self,
        data: &[u8],
        width: &u32,
        height: &u32,
        texture_format: &wgpu::TextureFormat,
        label: Option<&str>,
    ) -> Result<Texture, Error> {
        let size = wgpu::Extent3d {
            width: *width,
            height: *height,
            depth_or_array_layers: 1,
        };
        let bytes_per_row = 4 * size.width;
        let bytes_per_image = bytes_per_row * size.width;

        if bytes_per_row == 0 || data.len() < bytes_per_image as usize {
            return Err(Error::TextureCreationFailed);
        }

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: *texture_format,
            view_formats: &[*texture_format],
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(size.height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Ok(Texture {
            texture,
            view,
            sampler,
        })
    }

    pub fn texture_from_image(
        &self,
        image: &DynamicImage,
        texture_format: &wgpu::TextureFormat,
        label: Option<&str>,
    ) -> Result<Texture, Error> {
        let (width, height) = image.dimensions();
        let data = image.to_rgba8();
        self.texture_with_data(&data, &width, &height, texture_format, label)
    }

    pub fn texture_from_image_data(
        &self,
        data: &[u8],
        texture_format: &wgpu::TextureFormat,
        label: Option<&str>,
    ) -> Result<Texture, Error> {
        let image = image::load_from_memory(data)?;
        self.texture_from_image(&image, texture_format, label)
    }

    pub fn texture_from_color(
        &self,
        color: &SVector<f32, 4>,
        texture_format: &wgpu::TextureFormat,
        label: Option<&str>,
    ) -> Result<Texture, Error> {
        let data: [u8; 4] = [
            (color.x * 255_f32) as u8,
            (color.y * 255_f32) as u8,
            (color.z * 255_f32) as u8,
            (color.w * 255_f32) as u8,
        ];
        let (width, height) = (1, 1);
        self.texture_with_data(&data, &width, &height, texture_format, label)
    }
}
