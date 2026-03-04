use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GpuError {
    #[error("adapter/device: {0}")]
    Device(String),
    #[error("surface: {0}")]
    Surface(String),
    #[error("readback: {0}")]
    Readback(String),
}

#[derive(Debug)]
pub struct GpuContext {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub limits: wgpu::Limits,
    pub features: wgpu::Features,
    pub downlevel: wgpu::DownlevelCapabilities,
}

impl GpuContext {
    
    pub async fn new_headless() -> Result<Self, GpuError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: wgpu::InstanceFlags::default(),
            dx12_shader_compiler: Default::default(),
            gles_minor_version: Default::default(),
        });

        
        let low = wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            force_fallback_adapter: false,
            compatible_surface: None,
        };
        let high = wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        };

        let adapter = if let Some(a) = instance.request_adapter(&low).await {
            a
        } else {
            instance
                .request_adapter(&high)
                .await
                .ok_or_else(|| GpuError::Device("no adapter".into()))?
        };

        let downlevel = adapter.get_downlevel_capabilities();

        let needed_features = wgpu::Features::empty();
        let limits = wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits());
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("aevrix.device"),
                    required_features: needed_features,
                    required_limits: limits.clone(),
                },
                None,
            )
            .await
            .map_err(|e| GpuError::Device(e.to_string()))?;

        Ok(Self { instance, adapter, device, queue, limits, features: needed_features, downlevel })
    }
}


pub struct RenderTarget {
    pub tex: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub size: (u32, u32),
    pub format: wgpu::TextureFormat,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OffscreenTargetDesc {
    pub width: u32,
    pub height: u32,
}

impl RenderTarget {
    pub async fn offscreen(ctx: &GpuContext, desc: OffscreenTargetDesc) -> Result<Self, GpuError> {
        let format = wgpu::TextureFormat::Rgba8Unorm; 

        let tex = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("aevrix.offscreen"),
            size: wgpu::Extent3d { width: desc.width, height: desc.height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[format],
        });
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());

        Ok(Self { tex, view, size: (desc.width, desc.height), format })
    }
}


pub async fn read_rgba(ctx: &GpuContext, target: &RenderTarget) -> Result<Vec<u8>, GpuError> {
    let (w, h) = target.size;
    let bpr = (w * 4) as u32;
    
    let padded_bpr = ((bpr + 255) / 256) * 256;
    let size = (padded_bpr * h) as usize;

    let buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("aevrix.readback"),
        size: size as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut enc = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("aevrix.readback.encoder")
    });

    enc.copy_texture_to_buffer(
        wgpu::ImageCopyTexture {
            texture: &target.tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::ImageCopyBuffer {
            buffer: &buf,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(padded_bpr),
                rows_per_image: Some(h),
            },
        },
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
    );

    ctx.queue.submit(Some(enc.finish()));

    let slice = buf.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    ctx.device.poll(wgpu::Maintain::Wait);

    let data = slice.get_mapped_range();
    
    let mut out = vec![0u8; (w as usize) * (h as usize) * 4];
    for y in 0..h as usize {
        let src = &data[(y * padded_bpr as usize)..(y * padded_bpr as usize + bpr as usize)];
        let dst = &mut out[(y * w as usize * 4)..((y + 1) * w as usize * 4)];
        dst.copy_from_slice(src);
    }
    drop(data);
    buf.unmap();

    Ok(out)
}
