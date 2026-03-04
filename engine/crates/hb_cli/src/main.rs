use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use clap::Parser;
use hb_css::{
    cascade::compute_styles_for_tree,
    parser::parse_css,
};
use hb_graphics::render_to_surface;
use hb_html::{tree_builder as html, Dom as HtmlDom, NodeKind as HtmlNodeKind, TagName as HtmlTag};
use hb_layout::{
    layout_tree::{LayoutConfig, Size as LSize},
    DomForLayout,
};

#[derive(Debug, Parser)]
#[command(name="hb-cli", about="Aevrix headless renderer")]
struct Args {
    
    #[arg(short='i', long="in")]
    input: Option<PathBuf>,

    
    #[arg(short='o', long="out")]
    output: Option<PathBuf>,

    
    #[arg(long, default_value_t=800)]
    width: u32,

    
    #[arg(long, default_value_t=600)]
    height: u32,

    
    #[arg(long)]
    raw: bool,
}



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



fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let html_src = if let Some(path) = &args.input {
        fs::read_to_string(path)?
    } else {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        buf
    };

    let dom = hb_html::parse_to_dom(&html_src);
    let sheet = collect_author_css(&dom);

    let css_adapter = CssDomAdapter { dom: &dom };
    let styles = compute_styles_for_tree(&css_adapter, &sheet);

    let lay_adapter = LayoutDomAdapter { dom: &dom };
    let cfg = LayoutConfig { viewport: LSize { w: args.width as f32, h: args.height as f32 }, line_height: 1.25 };
    let plan = hb_layout::layout(&lay_adapter, &styles, &cfg);

    let surf = render_to_surface(&plan);
    if args.raw || args.output.is_none() {
        
        let mut stdout = io::stdout().lock();
        stdout.write_all(&surf.pixels)?;
    } else {
        let path = args.output.unwrap();
        let file = fs::File::create(path)?;
        let w = surf.size.w;
        let h = surf.size.h;

        let mut enc = png::Encoder::new(file, w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header()?;
        writer.write_image_data(&surf.pixels)?;
        writer.finish()?;
    }

    Ok(())
}
