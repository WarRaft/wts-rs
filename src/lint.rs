//! Lint rules for WTS. Malformed lines are parse errors, in
//! [`Document::errors`].
//!
//! - `duplicate-string`: a string number defined twice.

use std::collections::HashMap;

use crate::ast::Document;

use parser_rs::rule;
pub use parser_rs::{Diagnostic, Level};

/// Runs every lint rule over a document, sorted by line.
pub fn lint(doc: &Document) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let mut seen: HashMap<u32, usize> = HashMap::new();
    for entry in &doc.strings {
        let Some(id) = entry.id else { continue };
        if let Some(first) = seen.get(&id) {
            out.push(
                Diagnostic::warning(
                    rule::DUPLICATE_STRING,
                    format!("STRING {id} is already defined at line {first}"),
                    entry.line,
                )
                .at(entry.header),
            );
        } else {
            seen.insert(id, entry.line);
        }
    }
    out
}

#[cfg(test)]
#[path = "lint.test.rs"]
mod tests;
