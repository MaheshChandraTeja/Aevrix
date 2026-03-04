use std::collections::BTreeMap;

use crate::fonts::atlas::FontAtlas;
use crate::layout_tree::{DomForLayout, LayoutConfig, LayoutBox, BoxKind, Rect, Point, Size, PaintCmd};
use hb_css::cascade::{ComputedStyle, Display};



pub fn layout_element_block<D: DomForLayout>(
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
    if matches!(style.display, Display::None) {
        return LayoutBox {
            node: Some(node),
            kind: BoxKind::Anonymous,
            rect: Rect { origin: crate::layout_tree::Point { x, y: *y }, size: Size { w: 0.0, h: 0.0 } },
            children: Vec::new(),
        };
    }

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
            let lb = crate::inline::layout_text_block(child, &text, &style, x, y, max_w, cfg, atlas, paints);
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
