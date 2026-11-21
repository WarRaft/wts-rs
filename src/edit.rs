//! Changing a string table through its text: [`set_string`] and
//! [`remove_string`] give the text edits that write or drop one string,
//! changing only its lines, so an editor (or a tool) keeps everything else
//! as it was. [`apply`] applies them.

use parser_rs::{Edit, Span};

use crate::ast::Document;

/// Whether `text` can be a string's text: no line of it may be `}` alone,
/// which would end it.
pub fn check_text(text: &str) -> Result<(), String> {
    if text.lines().any(|l| l.trim() == "}") {
        return Err("a line that is only `}` would end the string early".into());
    }
    Ok(())
}

/// The edits that set string `id` to `text` (`\n` between lines): its
/// lines between the braces replaced, or, when there's no string `id`, a
/// new one added at the end in World Editor's layout. The file's line
/// breaks are kept.
pub fn set_string(doc: &Document, id: u32, text: &str) -> Result<Vec<Edit>, String> {
    check_text(text)?;
    let nl = doc.newline;
    let body: String = text
        .replace("\r\n", "\n")
        .split('\n')
        .map(|l| format!("{l}{nl}"))
        .collect();
    match doc.strings.iter().rfind(|e| e.id == Some(id)) {
        Some(entry) => {
            let (Some(open), Some(close)) = (entry.open, entry.close) else {
                return Err(format!(
                    "STRING {} isn't closed: fix its braces first",
                    entry.id_text
                ));
            };
            if entry.text == text.replace("\r\n", "\n") {
                return Ok(Vec::new());
            }
            Ok(vec![Edit {
                span: Span::new(open + 1, 1, close, 1),
                text: body,
            }])
        }
        None => {
            // After the last line; a blank line before it, as World Editor
            // writes them.
            let (line, col, lead) = match doc_end(doc) {
                (line, col, true) => (line, col, nl.to_string()),
                (line, col, false) => (line, col, format!("{nl}{nl}")),
            };
            let block = format!("{lead}STRING {id}{nl}{{{nl}{body}}}{nl}");
            Ok(vec![Edit {
                span: Span::new(line, col, line, col),
                text: block,
            }])
        }
    }
}

/// Where the file ends (line, column) and whether its last line is blank
/// (the file ends with a line break).
fn doc_end(doc: &Document) -> (usize, usize, bool) {
    (doc.end.0, doc.end.1, doc.end.1 == 1)
}

/// The edits that drop string `id`: its header, comments and text, and
/// the blank line after it.
pub fn remove_string(doc: &Document, id: u32) -> Result<Vec<Edit>, String> {
    let entry = doc
        .strings
        .iter()
        .rfind(|e| e.id == Some(id))
        .ok_or(format!("no STRING {id}"))?;
    let close = entry.close.ok_or(format!(
        "STRING {} isn't closed: fix its braces first",
        entry.id_text
    ))?;
    let mut end = close + 1;
    if doc.blank_lines.contains(&end) {
        end += 1;
    }
    Ok(vec![Edit {
        span: Span::new(entry.line, 1, end, 1),
        text: String::new(),
    }])
}

/// `src` with `edits` applied.
pub fn apply(src: &str, edits: &[Edit]) -> String {
    parser_rs::apply_edits(src, edits)
}

#[cfg(test)]
#[path = "edit.test.rs"]
mod tests;
