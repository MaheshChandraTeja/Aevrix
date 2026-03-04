use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::dom_types::{Dom, NodeKind, NodeId, TagName};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Size {
    pub w: f32,
    pub h: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Display {
    Block,
    Inline,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Style {
    pub display: Display,
    pub color: Option<[u8; 4]>,
    pub background: Option<[u8; 4]>,
    pub font_size: f32,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            display: Display::Block,
            color: None,
            background: None,
            font_size: 16.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyledNode {
    pub node: NodeId,
    pub style: Style,
    pub children: Vec<StyledNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutBox {
    pub node: NodeId,
    pub rect: Rect,
    pub children: Vec<LayoutBox>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaintCmd {
    FillRect { rect: Rect, rgba: [u8; 4] },
    DrawText { pos: Point, text: String, rgba: [u8; 4], size_px: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderPlan {
    pub viewport: Size,
    pub layout_root: LayoutBox,
    pub paint_list: Vec<PaintCmd>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    pub viewport: Size,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self { viewport: Size { w: 800.0, h: 600.0 } }
    }
}

pub fn style_tree(dom: &Dom) -> StyledNode {
    use crate::dom_types::{ElementData, Node};
    fn parse_color(v: &str) -> Option<[u8; 4]> {
        let vv = v.trim();
        if let Some(hex) = vv.strip_prefix('#') {
            if hex.len() == 6 {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                return Some([r, g, b, 255]);
            }
        }
        None
    }

    fn style_from_attrs(attrs: &BTreeMap<String, String>) -> Style {
        let mut s = Style::default();
        if let Some(inline) = attrs.get("style") {
            for decl in inline.split(';') {
                let mut it = decl.splitn(2, ':');
                let k = it.next().unwrap_or("").trim().to_lowercase();
                let v = it.next().unwrap_or("").trim();
                match k.as_str() {
                    "display" => {
                        s.display = match v {
                            "none" => Display::None,
                            "inline" => Display::Inline,
                            _ => Display::Block,
                        }
                    }
                    "color" => s.color = parse_color(v),
                    "background" | "background-color" => s.background = parse_color(v),
                    "font-size" => {
                        if let Some(px) = v.strip_suffix("px") {
                            if let Ok(n) = px.trim().parse::<f32>() {
                                s.font_size = n.max(8.0).min(64.0);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        s
    }

    fn build(dom: &Dom, id: NodeId) -> StyledNode {
        let node = &dom.nodes[id.0 as usize];
        match &node.kind {
            NodeKind::Element(ElementData { tag: _, attrs }) => {
                let style = style_from_attrs(attrs);
                let children = node.children.iter().map(|c| build(dom, *c)).collect();
                StyledNode { node: id, style, children }
            }
            NodeKind::Text(_)
            | NodeKind::Document
            | NodeKind::Style(_)
            | NodeKind::Script { .. } => {
                let children = node.children.iter().map(|c| build(dom, *c)).collect();
                StyledNode { node: id, style: Style::default(), children }
            }
        }
    }

    build(dom, dom.root)
}

pub fn layout_tree(dom: &Dom, styled: &StyledNode, cfg: &LayoutConfig) -> (LayoutBox, Vec<PaintCmd>) {
    let mut paints = Vec::new();

    fn layout_node(
        dom: &Dom,
        styled: &StyledNode,
        x: f32,
        y: &mut f32,
        max_w: f32,
        paints: &mut Vec<PaintCmd>,
    ) -> LayoutBox {
        let node = &dom.nodes[styled.node.0 as usize];

        match &node.kind {
            NodeKind::Document | NodeKind::Element(_) => {
                if let Some(bg) = styled.style.background {
                    let rect = Rect { origin: Point { x, y: *y }, size: Size { w: max_w, h: 0.0 } };
                    let idx = paints.len();
                    paints.push(PaintCmd::FillRect { rect, rgba: bg });

                    let start_y = *y;
                    let mut children_boxes = Vec::new();
                    for c in &styled.children {
                        let child = layout_node(dom, c, x, y, max_w, paints);
                        children_boxes.push(child);
                    }
                    let height = (*y - start_y).max(styled.style.font_size * 1.2);
                    if let PaintCmd::FillRect { rect, rgba } = &mut paints[idx] {
                        rect.size.h = height;
                    }

                    LayoutBox {
                        node: styled.node,
                        rect: Rect { origin: Point { x, y: start_y }, size: Size { w: max_w, h: height } },
                        children: children_boxes,
                    }
                } else {
                    let start_y = *y;
                    let mut children_boxes = Vec::new();
                    for c in &styled.children {
                        let child = layout_node(dom, c, x, y, max_w, paints);
                        children_boxes.push(child);
                    }
                    let height = (*y - start_y).max(styled.style.font_size * 1.2);
                    LayoutBox {
                        node: styled.node,
                        rect: Rect { origin: Point { x, y: start_y }, size: Size { w: max_w, h: height } },
                        children: children_boxes,
                    }
                }
            }
            NodeKind::Text(text) => {
                let fs = styled.style.font_size;
                let w = 0.6 * fs * (text.chars().count() as f32);
                let h = fs * 1.2;
                let rect = Rect { origin: Point { x, y: *y }, size: Size { w, h } };

                let color = styled.style.color.unwrap_or([0, 0, 0, 255]);
                paints.push(PaintCmd::DrawText { pos: rect.origin, text: text.clone(), rgba: color, size_px: fs });

                *y += h;

                LayoutBox { node: styled.node, rect, children: Vec::new() }
            }
            NodeKind::Style(_) | NodeKind::Script { .. } => {
                LayoutBox {
                    node: styled.node,
                    rect: Rect { origin: Point { x, y: *y }, size: Size { w: 0.0, h: 0.0 } },
                    children: Vec::new(),
                }
            }
        }
    }

    let mut y = 0.0f32;
    let root_box = layout_node(dom, styled, 0.0, &mut y, cfg.viewport.w, &mut paints);

    (root_box, paints)
}



pub fn build_render_plan(dom: &Dom, styled: &StyledNode, cfg: &LayoutConfig) -> RenderPlan {
    let (root_box, paints) = layout_tree(dom, styled, cfg);
    RenderPlan {
        viewport: cfg.viewport,
        layout_root: root_box,
        paint_list: paints,
    }
}
