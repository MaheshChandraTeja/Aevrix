use blake3::Hasher;
use hb_css::{
    cascade::{compute_styles_for_tree, DomAccessor as CssDom},
    parser::parse_css,
};
use hb_graphics::render_to_surface;
use hb_html::{tree_builder as html, Dom as HtmlDom, NodeKind as HtmlNodeKind, TagName as HtmlTag};
use hb_layout::{
    layout,
    layout_tree::{LayoutConfig, Size as LSize},
    DomForLayout,
};
use serde::Serialize;
use serde_json::to_vec;


struct CssDomAdapter<'a> { dom: &'a HtmlDom }
impl<'a> hb_css::cascade::DomAccessor for CssDomAdapter<'a> {
    type NodeId = html::NodeId;
    fn root(&self) -> Self::NodeId { self.dom.root }
    fn parent(&self, n: Self::NodeId) -> Option<Self::NodeId> { self.dom.nodes[n.0 as usize].parent }
    fn children(&self, n: Self::NodeId) -> Vec<Self::NodeId> { self.dom.nodes[n.0 as usize].children.clone() }
    fn is_element(&self, n: Self::NodeId) -> bool { matches!(self.dom.nodes[n.0 as usize].kind, HtmlNodeKind::Element(_)) }
    fn tag_name(&self, n: Self::NodeId) -> Option<String> {
        match &self.dom.nodes[n.0 as usize].kind {
            HtmlNodeKind::Element(ed) => Some(match ed.tag {
                HtmlTag::Html => "html",
                HtmlTag::Head => "head",
                HtmlTag::Body => "body",
                HtmlTag::Div => "div",
                HtmlTag::P => "p",
                HtmlTag::Span => "span",
                HtmlTag::Link => "link",
                HtmlTag::Style => "style",
                HtmlTag::Meta => "meta",
                HtmlTag::Br => "br",
                HtmlTag::Hr => "hr",
                HtmlTag::Img => "img",
                HtmlTag::Unknown(_) => "div",
            }.to_string()),
            _ => None,
        }
    }
    fn id(&self, n: Self::NodeId) -> Option<String> {
        match &self.dom.nodes[n.0 as usize].kind {
            HtmlNodeKind::Element(ed) => ed.attrs.get("id").cloned(),
            _ => None,
        }
    }
    fn classes(&self, n: Self::NodeId) -> Vec<String> {
        match &self.dom.nodes[n.0 as usize].kind {
            HtmlNodeKind::Element(ed) => {
                if let Some(cls) = ed.attrs.get("class") {
                    cls.split_whitespace().map(|s| s.to_string()).collect()
                } else { vec![] }
            }
            _ => vec![],
        }
    }
    fn inline_style(&self, n: Self::NodeId) -> Option<String> {
        match &self.dom.nodes[n.0 as usize].kind {
            HtmlNodeKind::Element(ed) => ed.attrs.get("style").cloned(),
            _ => None,
        }
    }
}


struct LayoutDomAdapter<'a> { dom: &'a HtmlDom }
impl<'a> DomForLayout for LayoutDomAdapter<'a> {
    type NodeId = html::NodeId;
    fn root(&self) -> Self::NodeId { self.dom.root }
    fn parent(&self, n: Self::NodeId) -> Option<Self::NodeId> { self.dom.nodes[n.0 as usize].parent }
    fn children(&self, n: Self::NodeId) -> Vec<Self::NodeId> { self.dom.nodes[n.0 as usize].children.clone() }
    fn is_element(&self, n: Self::NodeId) -> bool { matches!(self.dom.nodes[n.0 as usize].kind, HtmlNodeKind::Element(_)) }
    fn is_text(&self, n: Self::NodeId) -> bool { matches!(self.dom.nodes[n.0 as usize].kind, HtmlNodeKind::Text(_)) }
    fn text(&self, n: Self::NodeId) -> Option<String> {
        match &self.dom.nodes[n.0 as usize].kind {
            HtmlNodeKind::Text(s) => Some(s.clone()),
            _ => None,
        }
    }
}

fn pipeline(html_src: &str, w: u32, h: u32) -> (hb_layout::RenderPlan<html::NodeId>, Vec<u8>) {
    let dom = hb_html::parse_to_dom(html_src);
    let sheet = collect_author_css(&dom);

    let css_adapter = CssDomAdapter { dom: &dom };
    let styles = compute_styles_for_tree(&css_adapter, &sheet);

    let lay_adapter = LayoutDomAdapter { dom: &dom };
    let cfg = LayoutConfig { viewport: LSize { w: w as f32, h: h as f32 }, line_height: 1.25 };
    let plan = layout(&lay_adapter, &styles, &cfg);

    let surf = hb_graphics::render_to_surface(&plan);
    (plan, surf.pixels)
}

fn collect_author_css(dom: &HtmlDom) -> hb_css::parser::Stylesheet {
    let mut css_src = String::new();
    for node in &dom.nodes {
        if let HtmlNodeKind::Element(ed) = &node.kind {
            if matches!(ed.tag, HtmlTag::Style) {
                for &child in &node.children {
                    if let HtmlNodeKind::Text(t) = &dom.nodes[child.0 as usize].kind {
                        css_src.push_str(t);
                        css_src.push('\n');
                    }
                }
            }
        }
    }
    hb_css::parser::parse_css(&css_src)
}

fn hash_bytes<B: AsRef<[u8]>>(bytes: B) -> [u8; 32] {
    let mut h = Hasher::new();
    h.update(bytes.as_ref());
    *h.finalize().as_bytes()
}

fn hash_json<T: Serialize>(t: &T) -> [u8; 32] {
    let bytes = to_vec(t).expect("json");
    hash_bytes(bytes)
}

#[test]
fn first_paint_deterministic_static_content() {
    let html = r#"
        <style>
           body { background:#0b0b0f; color:#e6e6e6; }
           .wrap { background:#181a1f }
           p { color:#a0a0ff; font-size:18px }
        </style>
        <div class="wrap">
            <p>Hello <span>World</span></p>
        </div>
    "#;

    let (plan1, px1) = pipeline(html, 640, 360);
    let (plan2, px2) = pipeline(html, 640, 360);

    let h1 = hash_json(&plan1);
    let h2 = hash_json(&plan2);
    assert_eq!(h1, h2, "RenderPlan hashes differ");

    assert_eq!(px1.len(), px2.len(), "pixel buffer size differs");
    assert_eq!(hash_bytes(&px1), hash_bytes(&px2), "pixel content differs");
}

#[test]
fn paint_list_has_text_and_background() {
    let html = r#"
      <style> .bg { background:#202020 } p { font-size:16px; color:#ffffff } </style>
      <div class="bg"><p>Line one two three four five.</p></div>
    "#;

    let (plan, _px) = pipeline(html, 400, 200);

    let mut saw_text = 0usize;
    let mut saw_fill = 0usize;
    for cmd in &plan.paint_list {
        match cmd {
            hb_layout::PaintCmd::DrawText { .. } => saw_text += 1,
            hb_layout::PaintCmd::FillRect { .. } => saw_fill += 1,
        }
    }
    assert!(saw_text > 0, "expected DrawText paint ops");
    assert!(saw_fill > 0, "expected FillRect paint ops");
}
