use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::fonts::atlas::FontAtlas;
use crate::block::layout_element_block;
use crate::inline::layout_text_block;
use hb_css::cascade::{ComputedStyle, Display};


#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Size { pub w: f32, pub h: f32 }
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point { pub x: f32, pub y: f32 }
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rect { pub origin: Point, pub size: Size }


#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BoxKind { Block, InlineText, Anonymous, Document }


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutBox<N: Copy + Ord> {
    pub node: Option<N>,   
    pub kind: BoxKind,
    pub rect: Rect,
    pub children: Vec<LayoutBox<N>>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaintCmd {
    FillRect { rect: Rect, rgba: [u8;4] },
    DrawText { pos: Point, text: String, rgba: [u8;4], size_px: f32 },
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderPlan<N: Copy + Ord> {
    pub viewport: Size,
    pub layout_root: LayoutBox<N>,
    pub paint_list: Vec<PaintCmd>,
}


#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LayoutConfig {
    pub viewport: Size,
    
    pub line_height: f32,
}

impl Default for LayoutConfig {
    fn default() -> Self { Self { viewport: Size { w: 800.0, h: 600.0 }, line_height: 1.25 } }
}


pub trait DomForLayout {
    type NodeId: Copy + Ord;

    fn root(&self) -> Self::NodeId;
    fn parent(&self, n: Self::NodeId) -> Option<Self::NodeId>;
    fn children(&self, n: Self::NodeId) -> Vec<Self::NodeId>;

    fn is_element(&self, n: Self::NodeId) -> bool;
    fn is_text(&self, n: Self::NodeId) -> bool;
    fn text(&self, n: Self::NodeId) -> Option<String>;
}


pub fn layout<D: DomForLayout>(
    dom: &D,
    styles: &BTreeMap<D::NodeId, ComputedStyle>,
    cfg: &LayoutConfig,
) -> RenderPlan<D::NodeId> {
    let mut atlas = FontAtlas::new();
    let mut paints = Vec::new();

    let root = dom.root();
    let mut y_cursor = 0.0f32;

    let root_box = layout_node(dom, styles, &mut atlas, cfg, root, 0.0, &mut y_cursor, cfg.viewport.w, &mut paints);

    RenderPlan { viewport: cfg.viewport, layout_root: root_box, paint_list: paints }
}

fn layout_node<D: DomForLayout>(
    dom: &D,
    styles: &BTreeMap<D::NodeId, ComputedStyle>,
    atlas: &mut FontAtlas,
    cfg: &LayoutConfig,
    node: D::NodeId,
    x: f32,
    y: &mut f32,
    max_w: f32,
    paints: &mut Vec<PaintCmd>,
) -> LayoutBox<D::NodeId> {
    let style = styles.get(&node).cloned().unwrap_or_default();

    if dom.is_text(node) {
        
        let text = dom.text(node).unwrap_or_default();
        return layout_text_block(node, &text, &style, x, y, max_w, cfg, atlas, paints);
    }

    
    match style.display {
        Display::None => LayoutBox {
            node: Some(node),
            kind: BoxKind::Anonymous,
            rect: Rect { origin: Point { x, y: *y }, size: Size { w: 0.0, h: 0.0 } },
            children: Vec::new(),
        },
        _ => {
            
            let mut bg_idx: Option<usize> = None;
            let start_y = *y;

            if let Some(bg) = style.background {
                let rect = Rect { origin: Point { x, y: start_y }, size: Size { w: max_w, h: 0.0 } };
                bg_idx = Some(paints.len());
                paints.push(PaintCmd::FillRect { rect, rgba: bg });
            }

            let mut children_boxes = Vec::new();
            for child in dom.children(node) {
                if dom.is_text(child) {
                    
                    let text = dom.text(child).unwrap_or_default();
                    let lb = layout_text_block(child, &text, &style, x, y, max_w, cfg, atlas, paints);
                    children_boxes.push(lb);
                } else {
                    
                    let lb = layout_element_block(dom, styles, atlas, cfg, child, x, y, max_w, paints);
                    children_boxes.push(lb);
                }
            }

            let height = (*y - start_y).max(style.font_size * cfg.line_height);
            if let Some(i) = bg_idx {
                if let PaintCmd::FillRect { rect, .. } = &mut paints[i] {
                    rect.size.h = height;
                }
            }

            LayoutBox {
                node: Some(node),
                kind: BoxKind::Block,
                rect: Rect { origin: Point { x, y: start_y }, size: Size { w: max_w, h: height } },
                children: children_boxes,
            }
        }
    }
}
