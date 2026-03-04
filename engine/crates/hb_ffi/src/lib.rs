


use hb_css::{
    cascade::{compute_styles_for_tree, DomAccessor as CssDom, ComputedStyle},
    parser::parse_css,
};
use hb_graphics::render_to_surface;
use hb_html::{tree_builder as html, Dom as HtmlDom, NodeKind as HtmlNodeKind, TagName as HtmlTag};
use hb_layout::{
    layout,
    layout_tree::{LayoutConfig, Size as LSize},
    DomForLayout,
};
use once_cell::sync::OnceCell;
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Mutex;



#[derive(Default)]
struct EngineState {
    viewport_w: u32,
    viewport_h: u32,
    
    active_html: Option<String>,
}

static STATE: OnceCell<Mutex<EngineState>> = OnceCell::new();

fn state() -> &'static Mutex<EngineState> {
    STATE.get_or_init(|| Mutex::new(EngineState::default()))
}



struct CssDomAdapter<'a> {
    dom: &'a HtmlDom,
}
impl<'a> CssDom for CssDomAdapter<'a> {
    type NodeId = html::NodeId;

    fn root(&self) -> Self::NodeId { self.dom.root }
    fn parent(&self, n: Self::NodeId) -> Option<Self::NodeId> { self.dom.nodes[n.0 as usize].parent }
    fn children(&self, n: Self::NodeId) -> Vec<Self::NodeId> { self.dom.nodes[n.0 as usize].children.clone() }
    fn is_element(&self, n: Self::NodeId) -> bool {
        matches!(self.dom.nodes[n.0 as usize].kind, HtmlNodeKind::Element(_))
    }
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

struct LayoutDomAdapter<'a> {
    dom: &'a HtmlDom,
}
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
    parse_css(&css_src)
}



fn render_html_to_rgba(
    html_utf8: &str,
    viewport_w: u32,
    viewport_h: u32,
) -> Vec<u8> {
    
    let dom = hb_html::parse_to_dom(html_utf8);

    
    let sheet = collect_author_css(&dom);
    let css_adapter = CssDomAdapter { dom: &dom };
    let style_map = compute_styles_for_tree(&css_adapter, &sheet);

    
    let lay_adapter = LayoutDomAdapter { dom: &dom };
    let cfg = LayoutConfig { viewport: LSize { w: viewport_w as f32, h: viewport_h as f32 }, line_height: 1.25 };
    let plan = layout(&lay_adapter, &style_map, &cfg);

    
    let surf = render_to_surface(&plan);
    surf.pixels
}



#[no_mangle]
pub extern "C" fn hb_version() -> *const c_char {
    static mut PTR: *const c_char = std::ptr::null();
    unsafe {
        if PTR.is_null() {
            let s = format!("aevrix-hb v{}", option_env!("AEVRIX_BUILD_VERSION").unwrap_or("dev"));
            PTR = CString::new(s).unwrap().into_raw();
        }
        PTR
    }
}

#[no_mangle]
pub extern "C" fn hb_init(viewport_width: u32, viewport_height: u32) -> i32 {
    let mut s = state().lock().unwrap();
    s.viewport_w = viewport_width.max(1);
    s.viewport_h = viewport_height.max(1);
    0
}

#[no_mangle]
pub extern "C" fn hb_load_html(html_utf8: *const c_char) -> u32 {
    if html_utf8.is_null() { return 0; }
    let c = unsafe { CStr::from_ptr(html_utf8) };
    match c.to_str() {
        Ok(s) => {
            let mut st = state().lock().unwrap();
            st.active_html = Some(s.to_string());
            1 
        }
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn hb_load_url(_url_utf8: *const c_char) -> u32 {
    
    0
}

#[repr(C)]
pub struct hb_surface {
    pixels: *mut u8,
    width: u32,
    height: u32,
    stride: u32,
    len: usize,
}

#[no_mangle]
pub extern "C" fn hb_render(doc: u32, viewport_width: u32, viewport_height: u32, out: *mut hb_surface) -> i32 {
    if out.is_null() { return 1; }
    if doc != 1 {
        return 3; 
    }
    let (vw, vh) = {
        let s = state().lock().unwrap();
        let w = if viewport_width == 0 { s.viewport_w.max(1) } else { viewport_width };
        let h = if viewport_height == 0 { s.viewport_h.max(1) } else { viewport_height };
        (w, h)
    };
    let html = {
        let s = state().lock().unwrap();
        match &s.active_html {
            Some(h) => h.clone(),
            None => "<div style=\"font-size:18px\">Aevrix (no document loaded)</div>".to_string(),
        }
    };

    let mut pixels = render_html_to_rgba(&html, vw, vh);
    let len = pixels.len();
    let ptr = pixels.as_mut_ptr();
    std::mem::forget(pixels);

    unsafe {
        (*out).pixels = ptr;
        (*out).width = vw;
        (*out).height = vh;
        (*out).stride = vw * 4;
        (*out).len = len;
    }
    0
}

#[no_mangle]
pub extern "C" fn hb_render_html(html_utf8: *const c_char, viewport_width: u32, viewport_height: u32, out: *mut hb_surface) -> i32 {
    if out.is_null() || html_utf8.is_null() { return 1; }
    let c = unsafe { CStr::from_ptr(html_utf8) };
    let Ok(s) = c.to_str() else { return 1; };

    let vw = viewport_width.max(1);
    let vh = viewport_height.max(1);
    let mut pixels = render_html_to_rgba(s, vw, vh);

    let len = pixels.len();
    let ptr = pixels.as_mut_ptr();
    std::mem::forget(pixels);

    unsafe {
        (*out).pixels = ptr;
        (*out).width = vw;
        (*out).height = vh;
        (*out).stride = vw * 4;
        (*out).len = len;
    }
    0
}

#[no_mangle]
pub extern "C" fn hb_surface_release(surf: *mut hb_surface) {
    if surf.is_null() { return; }
    unsafe {
        let ptr = (*surf).pixels;
        let len = (*surf).len;
        if !ptr.is_null() && len > 0 {
            
            let _ = Vec::from_raw_parts(ptr, len, len);
        }
        (*surf).pixels = std::ptr::null_mut();
        (*surf).width = 0;
        (*surf).height = 0;
        (*surf).stride = 0;
        (*surf).len = 0;
    }
}
