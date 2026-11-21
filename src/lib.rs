//! A parser, editor and semantic highlighter for WTS (`war3map.wts`,
//! `war3campaign.wts`), the string table of Warcraft III maps: the text
//! that object data, triggers and the map's info name as `TRIGSTR_<n>`.
//! Structured like the other language crates; everything not
//! WTS-specific comes from `parser-rs` (see its README for the shared
//! design).
//!
//! ```
//! let doc = wts_rs::parse("STRING 3\n{\nKnight\n}\n");
//! assert_eq!(doc.lookup("TRIGSTR_003"), Some("Knight"));
//! assert_eq!(doc.resolve("Name: TRIGSTR_3"), "Name: Knight");
//! ```

pub mod ast;
pub mod edit;
pub mod html;
pub mod lexer;
pub mod lint;
pub mod parser;
pub mod printer;
pub mod semantic;
pub mod token;

pub use ast::{ColumnEncoding, Document, Entry, Span, split_bom};
pub use edit::{apply, check_text, remove_string, set_string};
pub use html::render_html;
pub use lint::{Diagnostic, Level, lint};
pub use parser::Parser;
pub use parser_rs::ParseError;
pub use printer::{render_dot, render_tree};
pub use semantic::{encoded_semantic_tokens, semantic_tokens};

/// Parses WTS source (a leading byte-order mark is skipped). This never
/// fails as a whole: problems are listed in [`Document::errors`] and the
/// rest still parses.
pub fn parse(src: &str) -> Document {
    Parser::parse_str(src)
}
