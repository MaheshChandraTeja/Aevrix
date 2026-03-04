















mod surface;
mod renderer;

pub use surface::{GpuContext, RenderTarget, OffscreenTargetDesc};
pub use renderer::{Compositor, CompositorConfig};

use hb_layout::layout_tree::Size as LSize;




pub fn render_plan_offscreen<N: Copy + Ord>(
    plan: &hb_layout::RenderPlan<N>,
) -> Result<(u32, u32, Vec<u8>), String> {
    let size = plan.viewport;
    let w = size.w.max(1.0).round() as u32;
    let h = size.h.max(1.0).round() as u32;

    let ctx = pollster::block_on(GpuContext::new_headless()) 
        .map_err(|e| format!("gpu init: {e}"))?;
    let mut comp = pollster::block_on(Compositor::new(&ctx))
        .map_err(|e| format!("compositor: {e}"))?;

    let target = pollster::block_on(RenderTarget::offscreen(&ctx, OffscreenTargetDesc { width: w, height: h }))
        .map_err(|e| format!("target: {e}"))?;

    let mut cmd = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("aevrix.firstpaint.encoder") });
    comp.encode(&ctx, &mut cmd, &target, plan).map_err(|e| format!("encode: {e}"))?;
    ctx.queue.submit(Some(cmd.finish()));

    let pixels = pollster::block_on(surface::read_rgba(&ctx, &target))
        .map_err(|e| format!("readback: {e}"))?;
    Ok((w, h, pixels))
}
