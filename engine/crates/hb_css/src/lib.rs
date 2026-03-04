









pub mod parser;
pub mod selectors;
pub mod cascade;

pub use parser::{Declaration, Stylesheet, Value, Color, parse_css, parse_inline_decls};
pub use selectors::{Selector, Specificity};
pub use cascade::{
    DomAccessor, Display, ComputedStyle, compute_styles_for_tree, default_user_agent_style
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
    struct N(u32);

    struct MockDom {
        
        parent: BTreeMap<N, Option<N>>,
        children: BTreeMap<N, Vec<N>>,
        tag: BTreeMap<N, String>,
        id: BTreeMap<N, String>,
        classes: BTreeMap<N, BTreeSet<String>>,
        inline: BTreeMap<N, String>,
    }

    impl MockDom {
        fn new() -> Self {
            use std::iter::FromIterator;
            let mut parent = BTreeMap::new();
            let mut children = BTreeMap::new();
            let mut tag = BTreeMap::new();
            let mut idm = BTreeMap::new();
            let mut classes = BTreeMap::new();
            let mut inline = BTreeMap::new();

            parent.insert(N(0), None);
            parent.insert(N(1), Some(N(0)));
            parent.insert(N(2), Some(N(1)));
            parent.insert(N(3), Some(N(2)));

            children.insert(N(0), vec![N(1)]);
            children.insert(N(1), vec![N(2)]);
            children.insert(N(2), vec![N(3)]);
            children.insert(N(3), vec![]);

            tag.insert(N(0), "html".into());
            tag.insert(N(1), "div".into());
            tag.insert(N(2), "p".into());
            tag.insert(N(3), "span".into());

            idm.insert(N(1), "app".into());

            classes.insert(N(1), BTreeSet::from_iter(["wrap".into()]));
            classes.insert(N(2), BTreeSet::new());
            classes.insert(N(3), BTreeSet::new());

            inline.insert(N(2), "color:#333333;font-size:18px".into());

            Self { parent, children, tag, id: idm, classes, inline }
        }
    }

    impl cascade::DomAccessor for MockDom {
        type NodeId = N;

        fn root(&self) -> Self::NodeId { N(0) }
        fn parent(&self, n: Self::NodeId) -> Option<Self::NodeId> { self.parent.get(&n).cloned().unwrap_or(None) }
        fn children(&self, n: Self::NodeId) -> Vec<Self::NodeId> { self.children.get(&n).cloned().unwrap_or_default() }
        fn is_element(&self, _n: Self::NodeId) -> bool { true }
        fn tag_name(&self, n: Self::NodeId) -> Option<String> { self.tag.get(&n).cloned() }
        fn id(&self, n: Self::NodeId) -> Option<String> { self.id.get(&n).cloned() }
        fn classes(&self, n: Self::NodeId) -> Vec<String> {
            self.classes.get(&n).map(|s| s.iter().cloned().collect()).unwrap_or_default()
        }
        fn inline_style(&self, n: Self::NodeId) -> Option<String> { self.inline.get(&n).cloned() }
    }

    #[test]
    fn cascade_applies_specificity_and_inline() {
        let css = r#"
            div { color:#000000 }
            #app { color:#111111 }
            .wrap { color:#222222 }
            div.wrap > p { color:#444444 }
        "#;
        let ss = parse_css(css);

        let dom = MockDom::new();
        let map = compute_styles_for_tree(&dom, &ss);

        let p_style = map.get(&N(2)).unwrap();
        
        assert_eq!(p_style.color.unwrap(), Color::rgba(0x33,0x33,0x33,0xFF).to_rgba());
        assert!((p_style.font_size - 18.0).abs() < f32::EPSILON);

        
        let span_style = map.get(&N(3)).unwrap();
        assert_eq!(span_style.color.unwrap(), default_user_agent_style().color.unwrap());
    }
}
