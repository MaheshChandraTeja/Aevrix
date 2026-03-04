use crate::fonts::atlas::FontAtlas;
use crate::fonts::shaping::measure_text;
use crate::layout_tree::{LayoutBox, BoxKind, Rect, Point, Size, PaintCmd, LayoutConfig};
use hb_css::cascade::ComputedStyle;





pub fn layout_text_block<N: Copy + Ord>(
    node: N,
    text: &str,
    style: &ComputedStyle,
    x: f32,
    y: &mut f32,
    max_w: f32,
    cfg: &LayoutConfig,
    atlas: &mut FontAtlas,
    paints: &mut Vec<PaintCmd>,
) -> LayoutBox<N> {
    let color = style.color.unwrap_or([0,0,0,255]);
    let fs = style.font_size.max(8.0).min(64.0);
    let line_h = fs * cfg.line_height;

    
    let norm = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if norm.is_empty() {
        return LayoutBox {
            node: Some(node),
            kind: BoxKind::InlineText,
            rect: Rect { origin: Point { x, y: *y }, size: Size { w: 0.0, h: 0.0 } },
            children: Vec::new(),
        };
    }

    let words: Vec<&str> = norm.split(' ').collect();
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();

    for (i, w) in words.iter().enumerate() {
        let candidate = if cur.is_empty() { w.to_string() } else { format!("{} {}", cur, w) };
        let width = measure_text(&candidate, fs, atlas);

        if width <= max_w || cur.is_empty() {
            cur = candidate;
        } else {
            lines.push(cur);
            cur = (*w).to_string();
        }

        if i == words.len() - 1 {
            lines.push(cur.clone());
        }
    }

    let start_y = *y;
    for line in &lines {
        let pos = Point { x, y: *y };
        paints.push(PaintCmd::DrawText { pos, text: line.clone(), rgba: color, size_px: fs });
        *y += line_h;
    }

    let height = (*y - start_y).max(line_h);
    let max_line_w = lines
        .iter()
        .map(|s| measure_text(s, fs, atlas))
        .fold(0.0f32, |a, b| a.max(b));

    LayoutBox {
        node: Some(node),
        kind: BoxKind::InlineText,
        rect: Rect { origin: Point { x, y: start_y }, size: Size { w: max_line_w.min(max_w), h: height } },
        children: Vec::new(),
    }
}
