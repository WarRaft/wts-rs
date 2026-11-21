use crate::ast::*;
pub use parser_rs::Node;

/// Builds a visualizable tree out of a parsed document: its strings, each
/// with its comments and text.
pub fn build_tree(doc: &Document) -> Node {
    let children = doc
        .strings
        .iter()
        .map(|e| {
            let mut kids: Vec<Node> = e
                .comments
                .iter()
                .map(|c| Node::leaf(c.clone()).tag("lit", None))
                .collect();
            kids.push(Node::leaf(format!("{:?}", e.text)).tag("lit", None));
            Node::with_children(format!("STRING {}", e.id_text), kids).tag("decl", Some(e.header))
        })
        .collect();
    Node::with_children("Document", children).tag("root", None)
}

/// Renders the document as an indented tree.
pub fn render_tree(doc: &Document) -> String {
    parser_rs::render_tree(&build_tree(doc))
}

/// Renders the document as a Graphviz DOT graph.
pub fn render_dot(doc: &Document) -> String {
    parser_rs::render_dot(&build_tree(doc))
}
