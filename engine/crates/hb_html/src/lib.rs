















pub mod tokenizer;
pub mod tree_builder;

pub use tokenizer::{Token, Tokenizer};
pub use tree_builder::{
    Dom, ElementData, Node, NodeId, NodeKind, TagName, parse_to_dom
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_tree() {
        let html = r#"
          <!DOCTYPE html>
          <div class="wrap" style="background:#eee">
            <p style="color:#333;font-size:18px"> Hello <span>World</span> </p>
            <link rel="stylesheet" href="/x.css" />
          </div>
        "#;

        let dom = parse_to_dom(html);
        
        assert!(dom.nodes.len() >= 4);

        
        let texts: Vec<String> = dom.nodes.iter().filter_map(|n| {
            if let NodeKind::Text(t) = &n.kind { Some(t.clone()) } else { None }
        }).collect();

        assert!(texts.iter().any(|t| t.contains("Hello")));
        assert!(texts.iter().any(|t| t.contains("World")));
    }

    #[test]
    fn attributes_are_sorted() {
        let html = r#"<div b="2" a="1" c=3></div>"#;
        let dom = parse_to_dom(html);
        let elem = dom.nodes.iter().find(|n| matches!(n.kind, NodeKind::Element(_))).unwrap();
        if let NodeKind::Element(ed) = &elem.kind {
            let keys: Vec<_> = ed.attrs.keys().cloned().collect();
            assert_eq!(keys, vec!["a","b","c"]);
        }
    }
}
