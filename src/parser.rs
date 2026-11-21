//! Reads the scanner's tokens into a [`Document`], line by line. A line
//! with a problem is recorded and the rest still parses. What's checked:
//!
//! - a header is `STRING` and a number;
//! - a header is followed by `{` (comments between are fine), and `{` by
//!   a header;
//! - a string's text ends with `}`;
//! - outside strings, a line is a header, `{`, `}`, a `// comment` or
//!   blank.

use crate::ast::*;
use crate::lexer::tokenize;
use crate::token::{Tok, Token};

pub struct Parser;

fn span(t: &Token<Tok>) -> Span {
    Span::new(t.line, t.col, t.end_line, t.end_col)
}

fn error(message: impl Into<String>, span: Span) -> ParseError {
    ParseError {
        message: message.into(),
        line: span.start_line,
        col: span.start_col,
        end_col: span.end_col.max(span.start_col + 1),
    }
}

impl Parser {
    pub fn parse_str(src: &str) -> Document {
        let tokens = tokenize(src);
        let mut doc = Document {
            newline: if src.contains("\r\n") { "\r\n" } else { "\n" },
            ..Document::default()
        };
        // The string being read, and whether its text has begun.
        let mut current: Option<Entry> = None;
        let mut text: Option<Vec<String>> = None;
        if let Some(eof) = tokens.last() {
            doc.end = (eof.line, eof.col);
        }
        for (i, tokens) in tokens.split(|t| t.kind == Tok::Newline).enumerate() {
            let Some(first) = tokens.first().filter(|t| t.kind != Tok::Eof) else {
                // A blank line: inside text, an empty line of it.
                match text.as_mut() {
                    Some(lines) => lines.push(String::new()),
                    None => doc.blank_lines.push(i + 1),
                }
                continue;
            };
            match &first.kind {
                Tok::Text(_) | Tok::ColorCode(_) => {
                    let joined = tokens
                        .iter()
                        .filter_map(|t| match &t.kind {
                            Tok::Text(s) | Tok::ColorCode(s) => Some(s.as_str()),
                            _ => None,
                        })
                        .collect::<String>();
                    if let Some(lines) = text.as_mut() {
                        lines.push(joined);
                    }
                }
                Tok::RBrace => {
                    if let (Some(mut entry), Some(lines)) = (current.take(), text.take()) {
                        entry.close = Some(first.line);
                        entry.text = lines.join("\n");
                        doc.strings.push(entry);
                    } else {
                        doc.errors
                            .push(error("'}' without a string before it", span(first)));
                    }
                }
                Tok::StringKw => {
                    if let Some(entry) = current.take() {
                        doc.errors.push(error(
                            format!("{} has no text: '{{' missing", name(&entry)),
                            entry.header,
                        ));
                        doc.strings.push(entry);
                    }
                    current = Some(header(tokens, &mut doc.errors));
                }
                Tok::Comment(c) => {
                    if let Some(entry) = current.as_mut() {
                        entry.comments.push(c.clone());
                    }
                }
                Tok::LBrace => match current.as_mut() {
                    Some(entry) if entry.open.is_none() => {
                        entry.open = Some(first.line);
                        text = Some(Vec::new());
                    }
                    _ => doc
                        .errors
                        .push(error("'{' without a STRING header before it", span(first))),
                },
                Tok::Junk(t) => doc.errors.push(error(
                    format!(
                        "expected `STRING <number>`, a `// comment`, '{{' or '}}', found '{t}'"
                    ),
                    span(first),
                )),
                Tok::Id(_) | Tok::Newline | Tok::Eof => {}
            }
        }
        if let Some(mut entry) = current {
            let message = match text {
                Some(lines) => {
                    entry.text = lines.join("\n");
                    format!("{} isn't closed: '}}' missing", name(&entry))
                }
                None => format!("{} has no text: '{{' missing", name(&entry)),
            };
            doc.errors.push(error(message, entry.header));
            doc.strings.push(entry);
        }
        // In source order, whichever line found them.
        doc.errors.sort_by_key(|e| (e.line, e.col));
        doc
    }
}

/// `STRING 3`, or `STRING` without a number.
fn name(entry: &Entry) -> String {
    if entry.id_text.is_empty() {
        "STRING".into()
    } else {
        format!("STRING {}", entry.id_text)
    }
}

/// A header line's tokens: the entry it starts.
fn header(tokens: &[Token<Tok>], errors: &mut Vec<ParseError>) -> Entry {
    let kw = &tokens[0];
    let id = tokens.get(1).filter(|t| matches!(t.kind, Tok::Id(_)));
    let id_text = match id.map(|t| &t.kind) {
        Some(Tok::Id(s)) => s.clone(),
        _ => String::new(),
    };
    let id_span = id.map_or(Span::new(kw.line, kw.end_col, kw.line, kw.end_col), span);
    let number = id_text
        .parse::<u32>()
        .ok()
        .filter(|_| id_text.bytes().all(|b| b.is_ascii_digit()));
    match id {
        None => errors.push(error("expected the string's number after STRING", span(kw))),
        Some(t) if number.is_none() => errors.push(error(
            format!("a string's id is a number, not '{id_text}'"),
            span(t),
        )),
        _ => {}
    }
    if let Some(junk) = tokens.iter().find(|t| matches!(t.kind, Tok::Junk(_))) {
        errors.push(error(
            "expected only the string's number after STRING",
            span(junk),
        ));
    }
    let end = tokens.last().map_or(kw.end_col, |t| t.end_col);
    Entry {
        id: number,
        id_text,
        line: kw.line,
        header: Span::new(kw.line, kw.col, kw.line, end),
        id_span,
        comments: Vec::new(),
        open: None,
        close: None,
        text: String::new(),
    }
}

#[cfg(test)]
#[path = "parser.test.rs"]
mod tests;
