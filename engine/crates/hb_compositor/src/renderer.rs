#![cfg_attr(not(feature = "text"), allow(dead_code, unused_imports, unused_variables))]

use thiserror::Error;
use smallvec::SmallVec;

use hb_layout::{RenderPlan, PaintCmd};
use crate::surface::{GpuContext, RenderTarget};
use font8x8::UnicodeFonts;

#[derive(Debug, Error)]
pub enum CompositorError {
    #[error("pipeline: {0}")]
    Pipeline(String),
    #[error("encode: {0}")]
    Encode(String),
}

#[derive(Debug, Clone, Copy)]
pub struct CompositorConfig {
    
    pub clear_rgba: [f32; 4],
}

impl Default for CompositorConfig {
    fn default() -> Self { Self { clear_rgba: [1.0, 1.0, 1.0, 0.0] } }
}

pub struct Compositor {
    cfg: CompositorConfig,
    rect: RectPipelines,
    #[cfg(feature = "text")]
    text: TextPipelines,
}

impl Compositor {
    pub async fn new(ctx: &GpuContext) -> Result<Self, CompositorError> {
        let rect = RectPipelines::new(ctx).map_err(|e| CompositorError::Pipeline(format!("rect: {e}")))?;
        #[cfg(feature = "text")]
        let text = TextPipelines::new(ctx).map_err(|e| CompositorError::Pipeline(format!("text: {e}")))?;
        Ok(Self {cfg: CompositorConfig::default(), rect, #[cfg(feature = "text")] text,})
    }

    pub fn config_mut(&mut self) -> &mut CompositorConfig { &mut self.cfg }

    pub fn encode<N: Copy + Ord>(
        &mut self,
        ctx: &GpuContext,
        enc: &mut wgpu::CommandEncoder,
        target: &RenderTarget,
        plan: &RenderPlan<N>,
    ) -> Result<(), CompositorError> {
        
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("aevrix.firstpaint.pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: self.cfg.clear_rgba[0] as f64,
                        g: self.cfg.clear_rgba[1] as f64,
                        b: self.cfg.clear_rgba[2] as f64,
                        a: self.cfg.clear_rgba[3] as f64,
                    }),
                    store: wgpu::StoreOp::Store,
                }
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        
        {
            self.rect.bind(&mut pass, ctx, target, plan.viewport.w, plan.viewport.h);
            let mut instances: SmallVec<[RectInstance; 256]> = SmallVec::new();
            for cmd in &plan.paint_list {
                if let PaintCmd::FillRect { rect, rgba } = cmd {
                    instances.push(RectInstance {
                        xywh: [rect.origin.x, rect.origin.y, rect.size.w, rect.size.h],
                        rgba: [rgba[0] as f32 / 255.0, rgba[1] as f32 / 255.0, rgba[2] as f32 / 255.0, rgba[3] as f32 / 255.0],
                    });
                }
            }
            if !instances.is_empty() {
                self.rect.draw_instances(&mut pass, ctx, &instances);
            }
        }

        
        {
            drop(pass); 
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("aevrix.text.pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target.view,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store }
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            for cmd in &plan.paint_list {
                #[cfg(feature = "text")]
                if let PaintCmd::DrawText { pos, text, rgba, size_px } = cmd {
                    let (tw, th, tex, view, smp) = self.text.build_run_texture(ctx, text, *size_px, *rgba)?;
                    let (vs_bg, fs_bg) = self.text.make_bind_groups(
                        ctx,
                        plan.viewport.w,
                        plan.viewport.h,
                        &view,
                        &smp,
                    );
                    self.text.bind(&mut pass, &vs_bg, &fs_bg);

                    self.text.draw_quad(&mut pass, ctx, pos.x, pos.y, tw as f32, th as f32);
                    
                    drop(tex);
                }
            }
        }

        Ok(())
    }
}



#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct RectInstance {
    xywh: [f32; 4],
    rgba: [f32; 4],
}

struct RectPipelines {
    vs_uniform: wgpu::Buffer, 
    vs_bind: wgpu::BindGroupLayout,
    vs_bg: wgpu::BindGroup,
    vbuf: wgpu::Buffer,       
    pibuf: wgpu::Buffer,      
    pipeline: wgpu::RenderPipeline,
    cap_instances: usize,
}

impl RectPipelines {
    fn new(ctx: &GpuContext) -> Result<Self, String> {
        
        let vs_bind = ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("rect.vs.bind"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                }
            ],
        });

        let vs_uniform = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rect.uniform.viewport"),
            size: 8,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let vs_bg = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rect.vs.bg"),
            layout: &vs_bind,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: vs_uniform.as_entire_binding() }],
        });

        
        let shader = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("rect.shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("rect.wgsl").into()),
        });

        
        let pipeline_layout = ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("rect.pipeline.layout"),
            bind_group_layouts: &[&vs_bind],
            push_constant_ranges: &[],
        });

        let pipeline = ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("rect.pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[
                    
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<RectInstance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttribute { shader_location: 0, offset: 0, format: wgpu::VertexFormat::Float32x4 }, 
                            wgpu::VertexAttribute { shader_location: 1, offset: 16, format: wgpu::VertexFormat::Float32x4 }, 
                        ],
                    }
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let vbuf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rect.vertex.dummy"),
            size: 4, usage: wgpu::BufferUsages::VERTEX, mapped_at_creation: false
        });
        let pibuf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rect.instance.init"),
            size: 1024, usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false
        });

        Ok(Self {
            vs_uniform, vs_bind, vs_bg, vbuf, pibuf, pipeline, cap_instances: 1024 / std::mem::size_of::<RectInstance>(),
        })
    }

    fn bind<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        ctx: &'a GpuContext,
        target: &'a RenderTarget,
        vw: f32, vh: f32,
    ) {
        ctx.queue.write_buffer(&self.vs_uniform, 0, &vw.to_le_bytes());
        ctx.queue.write_buffer(&self.vs_uniform, 4, &vh.to_le_bytes());
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.vs_bg, &[]);
        pass.set_vertex_buffer(0, self.pibuf.slice(..));
    }

    pub fn draw_instances<'a>(
    &'a self,
    pass: &mut wgpu::RenderPass<'a>,
    _ctx: &GpuContext,
    instances: &[RectInstance],
    ) {
        if instances.is_empty() {
            return;
        }

        pass.set_pipeline(&self.pipeline);

        
        
        for _ in instances {
            pass.draw(0..6, 0..1);
        }

        
        
        
        
        
        
    }
}



#[cfg(feature = "text")]
struct TextPipelines {
    vs_uniform: wgpu::Buffer,       
    vs_bind: wgpu::BindGroupLayout,
    fs_bind: wgpu::BindGroupLayout, 
    pipeline: wgpu::RenderPipeline,
}

#[cfg(feature = "text")]
impl TextPipelines {
    
    pub fn bind<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        vs_bg: &'a wgpu::BindGroup,
        fs_bg: &'a wgpu::BindGroup,
    ) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, vs_bg, &[]);
        pass.set_bind_group(1, fs_bg, &[]);
    }

    
    pub fn make_bind_groups<'a>(
        &'a self,
        ctx: &'a GpuContext,
        viewport_w: f32,
        viewport_h: f32,
        tex_view: &'a wgpu::TextureView,
        sampler: &'a wgpu::Sampler,
    ) -> (wgpu::BindGroup, wgpu::BindGroup) {
        
        ctx.queue.write_buffer(&self.vs_uniform, 0, &viewport_w.to_le_bytes());
        ctx.queue.write_buffer(&self.vs_uniform, 4, &viewport_h.to_le_bytes());

        let vs_bg = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("text.vs.bg"),
            layout: &self.vs_bind,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.vs_uniform.as_entire_binding(),
                },
            ],
        });

        let fs_bg = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("text.fs.bg"),
            layout: &self.fs_bind,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(tex_view),
                },
            ],
        });

        (vs_bg, fs_bg)
    }
}







const _: &str = include_str!("rect.wgsl");


const _: &str = include_str!("text.wgsl");