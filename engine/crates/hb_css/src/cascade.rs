
















use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::parser::{parse_inline_decls, Declaration, Stylesheet, Value};
use crate::selectors::{matches_selector, Specificity};





#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Display {
    Block,
    Inline,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputedStyle {
    pub display: Display,
    pub color: Option<[u8; 4]>,
    pub background: Option<[u8; 4]>,
    pub font_size: f32,
}


pub fn default_user_agent_style() -> ComputedStyle {
    ComputedStyle {
        display: Display::Block,
        color: Some([0x22, 0x22, 0x22, 0xFF]),
        background: None,
        font_size: 16.0,
    }
}


impl Default for ComputedStyle {
    fn default() -> Self {
        default_user_agent_style()
    }
}






pub trait DomAccessor {
    type NodeId: Copy + Ord;

    
    fn root(&self) -> Self::NodeId;
    fn parent(&self, n: Self::NodeId) -> Option<Self::NodeId>;
    fn children(&self, n: Self::NodeId) -> Vec<Self::NodeId>;

    
    fn is_element(&self, n: Self::NodeId) -> bool;
    
    fn tag_name(&self, n: Self::NodeId) -> Option<String>;
    fn id(&self, n: Self::NodeId) -> Option<String>;
    fn classes(&self, n: Self::NodeId) -> Vec<String>;

    
    fn inline_style(&self, n: Self::NodeId) -> Option<String>;
}


impl<T: DomAccessor> crate::selectors::DomMatch for T {
    type NodeId = <T as DomAccessor>::NodeId;

    #[inline]
    fn parent(&self, n: Self::NodeId) -> Option<Self::NodeId> {
        <T as DomAccessor>::parent(self, n)
    }
    #[inline]
    fn is_element(&self, n: Self::NodeId) -> bool {
        <T as DomAccessor>::is_element(self, n)
    }
    #[inline]
    fn tag_name(&self, n: Self::NodeId) -> Option<String> {
        <T as DomAccessor>::tag_name(self, n)
    }
    #[inline]
    fn id(&self, n: Self::NodeId) -> Option<String> {
        <T as DomAccessor>::id(self, n)
    }
    #[inline]
    fn classes(&self, n: Self::NodeId) -> Vec<String> {
        <T as DomAccessor>::classes(self, n)
    }
}










#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Rank {
    spec: Specificity,
    order: u32,
}

#[inline]
fn better(newr: Rank, old: Option<Rank>) -> bool {
    match old {
        None => true,
        Some(o) => {
            
            if newr.spec.0 != o.spec.0 {
                return newr.spec.0 > o.spec.0;
            }
            if newr.spec.1 != o.spec.1 {
                return newr.spec.1 > o.spec.1;
            }
            if newr.spec.2 != o.spec.2 {
                return newr.spec.2 > o.spec.2;
            }
            
            newr.order >= o.order
        }
    }
}

#[inline]
fn max_spec(a: Specificity, b: Specificity) -> Specificity {
    if a.0 != b.0 {
        return if a.0 > b.0 { a } else { b };
    }
    if a.1 != b.1 {
        return if a.1 > b.1 { a } else { b };
    }
    if a.2 != b.2 {
        return if a.2 > b.2 { a } else { b };
    }
    a
}






pub fn compute_styles_for_tree<D: DomAccessor>(dom: &D, sheet: &Stylesheet) -> BTreeMap<D::NodeId, ComputedStyle> {
    let mut out = BTreeMap::<D::NodeId, ComputedStyle>::new();
    compute_rec(dom, sheet, dom.root(), &mut out);
    out
}

fn compute_rec<D: DomAccessor>(
    dom: &D,
    sheet: &Stylesheet,
    n: D::NodeId,
    out: &mut BTreeMap<D::NodeId, ComputedStyle>,
) {
    
    if !dom.is_element(n) {
        out.entry(n).or_insert_with(default_user_agent_style);
        for c in dom.children(n) {
            compute_rec(dom, sheet, c, out);
        }
        return;
    }

    
    let mut style = default_user_agent_style();

    
    let mut r_color: Option<Rank> = None;
    let mut r_bg: Option<Rank> = None;
    let mut r_fs: Option<Rank> = None;
    let mut r_disp: Option<Rank> = None;

    
    for rule in &sheet.rules {
        if rule.selectors.is_empty() {
            continue;
        }

        
        let mut best_sel_spec: Option<Specificity> = None;
        for sel in &rule.selectors {
            if matches_selector(sel, dom, n) {
                let s = sel.specificity();
                best_sel_spec = Some(match best_sel_spec {
                    None => s,
                    Some(prev) => max_spec(prev, s),
                });
            }
        }

        if let Some(spec) = best_sel_spec {
            #[cfg(feature = "trace_css")]
            {
                
                tracing::trace!("rule applies: node={:?} spec={:?} order={}", n, spec, rule.source_order);
            }
            for decl in &rule.declarations {
                let rank = Rank {
                    spec,
                    order: rule.source_order,
                };
                apply_decl(decl, rank, &mut style, &mut r_color, &mut r_bg, &mut r_fs, &mut r_disp);
            }
        }
    }

    
    if let Some(style_attr) = dom.inline_style(n) {
        let decls = parse_inline_decls(&style_attr);
        
        let rank = Rank {
            spec: Specificity(1_000_000, 1_000_000, 1_000_000),
            order: u32::MAX,
        };
        for d in &decls {
            apply_decl(d, rank, &mut style, &mut r_color, &mut r_bg, &mut r_fs, &mut r_disp);
        }
    }

    out.insert(n, style);

    
    for c in dom.children(n) {
        compute_rec(dom, sheet, c, out);
    }
}

fn apply_decl(
    decl: &Declaration,
    rank: Rank,
    style: &mut ComputedStyle,
    r_color: &mut Option<Rank>,
    r_bg: &mut Option<Rank>,
    r_fs: &mut Option<Rank>,
    r_disp: &mut Option<Rank>,
) {
    let (n, v) = (&*decl.name, &decl.value);
    match (n, v) {
        (n, Value::Color(c)) if n.eq_ignore_ascii_case("color") => {
            if better(rank, *r_color) {
                style.color = Some(c.to_rgba());
                *r_color = Some(rank);
            }
        }
        (n, Value::Color(c))
            if n.eq_ignore_ascii_case("background") || n.eq_ignore_ascii_case("background-color") =>
        {
            if better(rank, *r_bg) {
                style.background = Some(c.to_rgba());
                *r_bg = Some(rank);
            }
        }
        (n, Value::LengthPx(px)) if n.eq_ignore_ascii_case("font-size") => {
            
            let clamped = px.max(8.0).min(96.0);
            if better(rank, *r_fs) {
                style.font_size = clamped;
                *r_fs = Some(rank);
            }
        }
        (n, Value::Ident(id)) if n.eq_ignore_ascii_case("display") => {
            let val = match id.as_str() {
                "none" => Display::None,
                "inline" => Display::Inline,
                
                _ => Display::Block,
            };
            if better(rank, *r_disp) {
                style.display = val;
                *r_disp = Some(rank);
            }
        }
        
        _ => {}
    }
}





#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rank_better_specificity_first() {
        let a = Rank { spec: Specificity(0, 1, 0), order: 1 };
        let b = Rank { spec: Specificity(0, 2, 0), order: 0 };
        assert!(better(b, Some(a)));
        assert!(!better(a, Some(b)));
    }

    #[test]
    fn rank_better_source_order_on_tie() {
        let a = Rank { spec: Specificity(0, 1, 0), order: 1 };
        let b = Rank { spec: Specificity(0, 1, 0), order: 2 };
        assert!(better(b, Some(a)));
    }
}
