







mod glyph_cache;
pub mod image_decode;

use glyph_cache::GlyphCache;
use hb_layout::{PaintCmd, RenderPlan};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Size { pub w: u32, pub h: u32 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Surface {
    pub size: Size,
    
    pub pixels: Vec<u8>,
}

impl Surface {
    pub fn new(w: u32, h: u32) -> Self {
        let cap = (w as usize) * (h as usize) * 4;
        Self { size: Size { w, h }, pixels: vec![0u8; cap] }
    }

    #[inline]
    fn idx(&self, x: u32, y: u32) -> Option<usize> {
        if x >= self.size.w || y >= self.size.h { return None; }
        Some(((y as usize) * (self.size.w as usize) + (x as usize)) * 4)
    }

    
    #[inline]
    fn blend_px(dst: &mut [u8], src: [u8; 4]) {
        let da = dst[3] as f32 / 255.0;
        let sa = src[3] as f32 / 255.0;

        let out_a = sa + da * (1.0 - sa);
        if out_a <= f32::EPSILON {
            dst.copy_from_slice(&[0,0,0,0]);
            return;
        }

        let blend = |dc: u8, sc: u8| -> u8 {
            let dc = dc as f32 / 255.0;
            let sc = sc as f32 / 255.0;
            let out = (sc * sa + dc * da * (1.0 - sa)) / out_a;
            (out * 255.0 + 0.5) as u8
        };

        let r = blend(dst[0], src[0]);
        let g = blend(dst[1], src[1]);
        let b = blend(dst[2], src[2]);
        let a = (out_a * 255.0 + 0.5) as u8;

        dst[0] = r; dst[1] = g; dst[2] = b; dst[3] = a;
    }

    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, rgba: [u8;4]) {
        if w <= 0 || h <= 0 { return; }
        let x0 = x.max(0) as u32;
        let y0 = y.max(0) as u32;
        let x1 = (x + w).min(self.size.w as i32) as u32;
        let y1 = (y + h).min(self.size.h as i32) as u32;
        for yy in y0..y1 {
            for xx in x0..x1 {
                if let Some(i) = self.idx(xx, yy) {
                    Self::blend_px(&mut self.pixels[i..i+4], rgba);
                }
            }
        }
    }

    fn blit_glyph_mask(
        &mut self,
        gx: i32,
        gy: i32,
        mask_w: u32,
        mask_h: u32,
        mask: &[u8],            
        color: [u8;4],          
    ) {
        let w = mask_w as i32;
        let h = mask_h as i32;
        for j in 0..h {
            let y = gy + j;
            if y < 0 || y >= self.size.h as i32 { continue; }
            for i in 0..w {
                let x = gx + i;
                if x < 0 || x >= self.size.w as i32 { continue; }
                let a = mask[(j as u32 * mask_w + i as u32) as usize];
                if a == 0 { continue; }
                if let Some(idx) = self.idx(x as u32, y as u32) {
                    let src = [color[0], color[1], color[2], a];
                    Self::blend_px(&mut self.pixels[idx..idx+4], src);
                }
            }
        }
    }
}


pub fn render_to_surface<N: Copy + Ord>(plan: &RenderPlan<N>) -> Surface {
    let w = plan.viewport.w.max(1.0).round() as u32;
    let h = plan.viewport.h.max(1.0).round() as u32;
    let mut surf = Surface::new(w, h);
    let mut cache = GlyphCache::new();

    for cmd in &plan.paint_list {
        match cmd {
            PaintCmd::FillRect { rect, rgba } => {
                let x = rect.origin.x.round() as i32;
                let y = rect.origin.y.round() as i32;
                let ww = rect.size.w.round() as i32;
                let hh = rect.size.h.round() as i32;
                surf.fill_rect(x, y, ww, hh, *rgba);
            }
            PaintCmd::DrawText { pos, text, rgba, size_px } => {
                
                let glyph_h = (size_px * 0.90).clamp(6.0, 128.0).floor() as u32;
                let baseline = pos.y.round() as i32; 

                let mut pen_x = pos.x.round() as i32;
                for ch in text.chars() {
                    let (mask_w, mask_h, mask) = cache.get(ch, glyph_h);
                    
                    surf.blit_glyph_mask(pen_x, baseline, mask_w, mask_h, &mask, *rgba);
                    pen_x += mask_w as i32 + 1;
                }
            }
        }
    }

    surf
}

#[cfg(test)]
mod tests {
    use super::*;
    use hb_layout::{PaintCmd, layout_tree::{Size as LSize, Point as LPoint, Rect as LRect}, layout};

    #[test]
    fn raster_smoke() {
        
        let plan = RenderPlan::<u32> {
            viewport: hb_layout::layout_tree::Size { w: 320.0, h: 120.0 },
            layout_root: hb_layout::layout_tree::LayoutBox {
                node: None, kind: hb_layout::layout_tree::BoxKind::Document,
                rect: hb_layout::layout_tree::Rect { origin: hb_layout::layout_tree::Point { x:0.0, y:0.0}, size: hb_layout::layout_tree::Size { w:320.0, h:120.0 } },
                children: vec![]
            },
            paint_list: vec![
                PaintCmd::FillRect { rect: LRect { origin: LPoint { x: 0.0, y: 0.0 }, size: LSize { w: 320.0, h: 120.0 }}, rgba: [240,240,240,255] },
                PaintCmd::DrawText { pos: LPoint { x: 8.0, y: 8.0 }, text: "Hello Aevrix".into(), rgba: [20,20,20,255], size_px: 18.0 },
            ],
        };

        let surf = render_to_surface(&plan);
        assert_eq!(surf.size.w, 320);
        assert_eq!(surf.size.h, 120);
        
        assert!(surf.pixels.iter().any(|&p| p != 0));
    }
}
