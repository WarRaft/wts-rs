use crate::ast::Document;
use crate::printer::build_tree;
use crate::semantic::semantic_tokens;

/// Renders a self-contained HTML page that visualizes a parsed document
/// next to its source, colored from the same semantic tokens the editor
/// gets; see [`parser_rs::html`].
pub fn render_html(doc: &Document, source: &str, title: &str) -> String {
    parser_rs::render_html_tokens(&build_tree(doc), source, title, &semantic_tokens(source))
}
