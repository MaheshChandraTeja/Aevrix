use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NodeId(pub u32);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeKind {
    Document,
    Element(ElementData),
    Text(String),
    Style(String),
    Script { _raw: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TagName {
    Html,
    Head,
    Body,
    Div,
    P,
    Span,
    Link,
    Style,
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementData {
    pub tag: TagName,
    pub attrs: BTreeMap<String, String>,
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
    pub fn new_document() -> Self {
        let root = Node {
            id: NodeId(0),
            parent: None,
            children: Vec::new(),
            kind: NodeKind::Document,
        };
        Self { nodes: vec![root], root: NodeId(0) }
    }

    pub fn append_child(&mut self, parent: NodeId, kind: NodeKind) -> NodeId {
        let id = NodeId(self.nodes.len() as u32);
        self.nodes.push(Node { id, parent: Some(parent), children: Vec::new(), kind });
        if let Some(p) = self.nodes.get_mut(parent.0 as usize) {
            p.children.push(id);
        }
        id
    }
}

pub fn parse_html_min(input: &str) -> crate::error::Result<Dom> {
    use crate::error::{EngineError, Result};

    let mut dom = Dom::new_document();

    let html_id = dom.append_child(dom.root, NodeKind::Element(ElementData {
        tag: TagName::Html,
        attrs: BTreeMap::new(),
    }));
    let head_id = dom.append_child(html_id, NodeKind::Element(ElementData {
        tag: TagName::Head,
        attrs: BTreeMap::new(),
    }));
    let body_id = dom.append_child(html_id, NodeKind::Element(ElementData {
        tag: TagName::Body,
        attrs: BTreeMap::new(),
    }));

    let mut stack: Vec<NodeId> = vec![body_id];
    for chunk in input.split('<') {
        if chunk.is_empty() {
            continue;
        }
        if let Some(pos) = chunk.find('>') {
            let inside = &chunk[..pos];
            let tail = &chunk[pos + 1..];

            if inside.starts_with('/') {
                let name = inside.trim_start_matches('/').trim().to_lowercase();
                let targets = ["div", "p", "span", "body", "head", "html", "style", "link"];
                if targets.contains(&name.as_str()) {
                    if stack.len() > 1 {
                        stack.pop();
                    }
                }
                let text = tail.trim();
                if !text.is_empty() {
                    push_text(&mut dom, *stack.last().unwrap(), text);
                }
                continue;
            }

            let (tag_name, attrs, self_closing) = parse_tag(inside);
            let tag = canon_tag(&tag_name);
            let node_id = match tag {
                TagName::Style => {
                    let (style_text, rest) = split_until_end(tail, "/style");
                    let id = dom.append_child(*stack.last().unwrap(), NodeKind::Style(style_text.to_string()));
                    let text_after = rest.trim();
                    if !text_after.is_empty() {
                        push_text(&mut dom, *stack.last().unwrap(), text_after);
                    }
                    id
                }
                TagName::Link => {
                    let sorted = attrs;
                    let id = dom.append_child(
                        *stack.last().unwrap(),
                        NodeKind::Element(ElementData { tag: TagName::Link, attrs: sorted }),
                    );
                    id
                }
                _ => {
                    let id = dom.append_child(
                        *stack.last().unwrap(),
                        NodeKind::Element(ElementData { tag: tag.clone(), attrs }),
                    );
                    if !self_closing {
                        stack.push(id);
                    }
                    id
                }
            };

            let text = tail.trim();
            if !text.is_empty() && !matches!(dom.nodes[node_id.0 as usize].kind, NodeKind::Style(_)) {
                push_text(&mut dom, *stack.last().unwrap(), text);
            }
        } else {
            let text = chunk.trim();
            if !text.is_empty() {
                push_text(&mut dom, *stack.last().unwrap(), text);
            }
        }
    }

    Ok(dom)
}

fn push_text(dom: &mut Dom, parent: NodeId, s: &str) {
    let collapsed = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if !collapsed.is_empty() {
        dom.append_child(parent, NodeKind::Text(collapsed));
    }
}

fn parse_tag(src: &str) -> (String, BTreeMap<String, String>, bool) {
    let mut parts = src.trim().split_whitespace();
    let name = parts.next().unwrap_or("div").trim_matches('/').to_lowercase();
    let mut attrs = BTreeMap::new();
    let mut self_closing = src.trim().ends_with("/");

    for part in parts {
        let mut it = part.splitn(2, '=');
        let k = it.next().unwrap_or("").trim().trim_matches('/').to_lowercase();
        let mut v = it.next().unwrap_or("").trim();
        if v.starts_with('"') && v.ends_with('"') && v.len() >= 2 {
            v = &v[1..v.len() - 1];
        } else if v.starts_with('\'') && v.ends_with('\'') && v.len() >= 2 {
            v = &v[1..v.len() - 1];
        }
        if !k.is_empty() {
            attrs.insert(k.to_string(), v.to_string());
        }
        if part.ends_with('/') {
            self_closing = true;
        }
    }

    (name, attrs, self_closing)
}

fn canon_tag(name: &str) -> TagName {
    match name {
        "html" => TagName::Html,
        "head" => TagName::Head,
        "body" => TagName::Body,
        "div" => TagName::Div,
        "p" => TagName::P,
        "span" => TagName::Span,
        "style" => TagName::Style,
        "link" => TagName::Link,
        other => TagName::Unknown(other.to_string()),
    }
}

fn split_until_end<'a>(src: &'a str, end: &str) -> (&'a str, &'a str) {
    let needle = format!("<{}", end);
    if let Some(i) = src.to_lowercase().find(&needle) {
        let (left, right) = src.split_at(i);
        
        if let Some(j) = right.find('>') {
            (left, &right[j + 1..])
        } else {
            (left, "")
        }
    } else {
        (src, "")
    }
}
