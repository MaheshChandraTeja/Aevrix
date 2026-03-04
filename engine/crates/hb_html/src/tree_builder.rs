use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::tokenizer::{Token, Tokenizer};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NodeId(pub u32);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TagName {
    Html, Head, Body, Div, P, Span, Link, Style, Meta, Br, Hr, Img,
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementData {
    pub tag: TagName,
    pub attrs: BTreeMap<String, String>, 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeKind {
    Document,
    Element(ElementData),
    Text(String),
    
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub kind: NodeKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dom {
    pub nodes: Vec<Node>,
    pub root: NodeId,
}

impl Dom {
    fn new_document() -> Self {
        let root = Node {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            kind: NodeKind::Document,
        };
        Self { nodes: vec![root], root: NodeId(0) }
    }

    fn append_child(&mut self, parent: NodeId, kind: NodeKind) -> NodeId {
        let id = NodeId(self.nodes.len() as u32);
        self.nodes.push(Node { id, parent: Some(parent), children: Vec::new(), kind });
        let p = &mut self.nodes[parent.0 as usize];
        p.children.push(id);
        id
    }
}

#[derive(Debug, Clone)]
struct OpenElements(Vec<NodeId>);

impl OpenElements {
    fn new() -> Self { Self(Vec::new()) }
    fn push(&mut self, id: NodeId) { self.0.push(id) }
    fn pop_until<F: Fn(&NodeId) -> bool>(&mut self, pred: F) {
        while let Some(top) = self.0.last().cloned() {
            if pred(&top) { self.0.pop(); break; }
            self.0.pop();
        }
    }
    fn current(&self) -> Option<NodeId> { self.0.last().cloned() }
    fn is_empty(&self) -> bool { self.0.is_empty() }
}

fn canon_tag(name: &str) -> TagName {
    match name {
        "html" => TagName::Html,
        "head" => TagName::Head,
        "body" => TagName::Body,
        "div" => TagName::Div,
        "p" => TagName::P,
        "span" => TagName::Span,
        "link" => TagName::Link,
        "style" => TagName::Style,
        "meta" => TagName::Meta,
        "br" => TagName::Br,
        "hr" => TagName::Hr,
        "img" => TagName::Img,
        other => TagName::Unknown(other.to_string()),
    }
}

fn is_void(tag: &TagName) -> bool {
    matches!(tag, TagName::Br | TagName::Hr | TagName::Img | TagName::Meta | TagName::Link)
}


pub fn parse_to_dom(html: &str) -> Dom {
    let mut dom = Dom::new_document();

    
    let html_id = dom.append_child(dom.root, NodeKind::Element(ElementData {
        tag: TagName::Html, attrs: BTreeMap::new()
    }));
    let head_id = dom.append_child(html_id, NodeKind::Element(ElementData {
        tag: TagName::Head, attrs: BTreeMap::new()
    }));
    let body_id = dom.append_child(html_id, NodeKind::Element(ElementData {
        tag: TagName::Body, attrs: BTreeMap::new()
    }));

    let mut open = OpenElements::new();
    open.push(body_id); 

    let mut in_head = false;
    let mut seen_html = false;

    let mut tk = Tokenizer::new(html);
    while let Some(tok) = tk.next() {
        match tok {
            Token::EOF => break,
            Token::Doctype { .. } => {
                
            }
            Token::Comment(_) => {
                
            }
            Token::Character(text) => {
                if let Some(parent) = open.current() {
                    
                    if !text.is_empty() {
                        dom.append_child(parent, NodeKind::Text(text));
                    }
                }
            }
            Token::StartTag { name, attrs, self_closing } => {
                let tag = canon_tag(&name);
                
                let parent = if matches!(tag, TagName::Head) {
                    in_head = true;
                    html_id
                } else if matches!(tag, TagName::Meta | TagName::Link | TagName::Style) && !in_head {
                    
                    head_id
                } else {
                    open.current().unwrap_or(body_id)
                };

                let mut map = BTreeMap::new();
                for (k, v) in attrs {
                    map.insert(k, v);
                }

                let id = dom.append_child(parent, NodeKind::Element(ElementData { tag: tag.clone(), attrs: map }));
                if !self_closing && !is_void(&tag) {
                    
                    if matches!(tag, TagName::Head) {
                        
                        open.push(head_id);
                    } else {
                        open.push(id);
                    }
                }

                if matches!(tag, TagName::Html) {
                    seen_html = true;
                }
            }
            Token::EndTag { name } => {
                let tag = canon_tag(&name);

                match tag {
                    TagName::Head => {
                        in_head = false;
                        open.pop_until(|top| top == &head_id);
                    }
                    TagName::Body => {
                        
                        open.pop_until(|top| top == &body_id);
                    }
                    TagName::Html if seen_html => {
                        
                        while !open.is_empty() { open.0.pop(); }
                    }
                    _ => {
                        
                        let want = tag.clone();
                        open.pop_until(|top| {
                            let node = &dom.nodes[top.0 as usize];
                            if let NodeKind::Element(ed) = &node.kind {
                                return ed.tag == want;
                            }
                            false
                        });
                    }
                }
            }
        }
    }

    dom
}
