use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Specificity(pub u32, pub u32, pub u32); 

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Combinator {
    Descendant,
    Child,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimpleSelector {
    pub tag: Option<String>,        
    pub id: Option<String>,
    pub classes: BTreeSet<String>,  
}

impl SimpleSelector {
    pub fn specificity(&self) -> Specificity {
        let a = self.id.is_some() as u32;
        let b = self.classes.len() as u32;
        let c = self.tag.is_some() as u32;
        Specificity(a, b, c)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectorPart {
    pub comb: Option<Combinator>, 
    pub simple: SimpleSelector,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selector {
    pub parts: Vec<SelectorPart>, 
}

impl Selector {
    pub fn specificity(&self) -> Specificity {
        self.parts.iter().fold(Specificity(0,0,0), |acc, p| {
            let s = p.simple.specificity();
            Specificity(acc.0 + s.0, acc.1 + s.1, acc.2 + s.2)
        })
    }
}






pub fn parse_selector_list(src: &str) -> Vec<Selector> {
    let mut out = Vec::new();
    for raw in src.split(',') {
        let s = raw.trim();
        if s.is_empty() { continue; }
        out.push(parse_selector(s));
    }
    out
}

fn parse_selector(s: &str) -> Selector {
    
    let mut parts_rev: Vec<SelectorPart> = Vec::new();

    
    
    let chunks: Vec<(Option<Combinator>, &str)> = Vec::new();
    let cur = s.trim();
    let _last = cur;
    
    
    
    let mut tokens: Vec<(Option<Combinator>, String)> = Vec::new();
    let mut i = 0usize;
    let bytes = cur.as_bytes();
    let mut buf = String::new();
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c == '>' {
            let chunk = buf.trim().to_string();
            if !chunk.is_empty() { tokens.push((None, chunk)); }
            tokens.push((Some(Combinator::Child), String::new()));
            buf.clear();
            i += 1;
        } else {
            buf.push(c);
            i += 1;
        }
    }
    if !buf.trim().is_empty() { tokens.push((None, buf.trim().to_string())); }

    
    let mut with_desc: Vec<(Option<Combinator>, String)> = Vec::new();
    for (comb, chunk) in tokens {
        if comb.is_some() {
            with_desc.push((comb, String::new()));
            continue;
        }
        let parts = chunk.split_whitespace().filter(|s| !s.is_empty());
        let mut first = true;
        for p in parts {
            if !first {
                with_desc.push((Some(Combinator::Descendant), p.to_string()));
            } else {
                with_desc.push((None, p.to_string()));
                first = false;
            }
        }
    }

    
    let mut pending_comb: Option<Combinator> = None;
    for (comb, chunk) in with_desc {
        if chunk.is_empty() {
            pending_comb = comb; 
            continue;
        }
        let simple = parse_simple(&chunk);
        
        parts_rev.push(SelectorPart { comb: pending_comb.take(), simple });
    }

    
    

    parts_rev.reverse();
    
    
    

    Selector { parts: parts_rev }
}

fn parse_simple(chunk: &str) -> SimpleSelector {
    let mut tag: Option<String> = None;
    let mut id: Option<String> = None;
    let mut classes: BTreeSet<String> = BTreeSet::new();

    let mut buf = String::new();
    let mut mode = 't'; 

    
    let mut chars = chunk.chars().peekable();
    if let Some('*') = chars.peek().cloned() {
        chars.next(); 
        tag = None;
    }

    while let Some(ch) = chars.next() {
        match ch {
            '#' => {
                if !buf.is_empty() && tag.is_none() { tag = Some(buf.clone()); }
                buf.clear();
                mode = 'i';
            }
            '.' => {
                if !buf.is_empty() && tag.is_none() { tag = Some(buf.clone()); }
                buf.clear();
                mode = 'c';
            }
            _ => { buf.push(ch.to_ascii_lowercase()); }
        }
    }

    
    match mode {
        'i' => if !buf.is_empty() { id = Some(buf) },
        'c' => if !buf.is_empty() { classes.insert(buf); },
        _ => if !buf.is_empty() { tag = Some(buf) },
    }

    SimpleSelector { tag, id, classes }
}


pub trait DomMatch {
    type NodeId: Copy + Ord;
    fn parent(&self, n: Self::NodeId) -> Option<Self::NodeId>;
    fn is_element(&self, n: Self::NodeId) -> bool;
    fn tag_name(&self, n: Self::NodeId) -> Option<String>; 
    fn id(&self, n: Self::NodeId) -> Option<String>;
    fn classes(&self, n: Self::NodeId) -> Vec<String>;
}

pub fn matches_selector<D: DomMatch>(sel: &Selector, dom: &D, n: D::NodeId) -> bool {
    if sel.parts.is_empty() { return false; }

    
    if !matches_simple(&sel.parts[0].simple, dom, n) { return false; }

    
    let mut current = n;
    for part in &sel.parts[1..] {
        match part.comb.unwrap_or(Combinator::Descendant) {
            Combinator::Child => {
                if let Some(p) = dom.parent(current) {
                    if matches_simple(&part.simple, dom, p) {
                        current = p;
                    } else {
                        return false;
                    }
                } else { return false; }
            }
            Combinator::Descendant => {
                
                let mut found = false;
                let mut p = dom.parent(current);
                while let Some(pp) = p {
                    if matches_simple(&part.simple, dom, pp) { found = true; current = pp; break; }
                    p = dom.parent(pp);
                }
                if !found { return false; }
            }
        }
    }

    true
}

fn matches_simple<D: DomMatch>(s: &SimpleSelector, dom: &D, n: D::NodeId) -> bool {
    if !dom.is_element(n) { return false; }

    if let Some(ref t) = s.tag {
        let tn = dom.tag_name(n);
        if tn.as_deref() != Some(t.as_str()) { return false; }
    }

    if let Some(ref want_id) = s.id {
        if dom.id(n).as_deref() != Some(want_id.as_str()) { return false; }
    }

    if !s.classes.is_empty() {
        let set: BTreeSet<String> = dom.classes(n).into_iter().collect();
        for c in &s.classes {
            if !set.contains(c) { return false; }
        }
    }

    true
}
