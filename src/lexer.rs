//! The WTS line scanner. A file is blocks:
//!
//! ```text
//! STRING 3
//! // Units: h000 (Knight), Name (unam)
//! {
//! Knight of the |cffffcc00Silver Hand|r
//! }
//! ```
//!
//! What a line is depends on where it stands: between `{` and `}` every
//! line is text, even one that reads `STRING 4` or `// ...`; outside,
//! a line is a header, a comment, a brace or blank. So WTS has its own
//! scanner, which keeps that state. It is the **only** lexer: the parser,
//! the highlighter and the viewer read its tokens.
//!
//! - A string's text ends at the first line that is `}` alone (spaces
//!   around it allowed); the text can't hold such a line.
//! - Inside text, `|c` and eight hex digits start a color, `|r` ends it,
//!   `|n` is a line break: [`Tok::ColorCode`], between [`Tok::Text`] parts.
//!
//! The scanner never fails: a line that fits no form is a [`Tok::Junk`]
//! and the parser reports it.

use crate::token::{Tok, Token};

/// Scans `src` into tokens: each line's tokens, then [`Tok::Newline`]
/// (none after the last line), then [`Tok::Eof`]. `\r\n` endings and a
/// leading byte-order mark are accepted.
pub fn tokenize(src: &str) -> Vec<Token<Tok>> {
    let src = src.strip_prefix('\u{FEFF}').unwrap_or(src);
    let mut out = Vec::new();
    let (mut line, mut end) = (1, 1);
    let mut inside = false;
    for (i, raw) in src.split('\n').enumerate() {
        if i > 0 {
            out.push(Token::new(Tok::Newline, line, end, end + 1));
            line += 1;
        }
        let text = raw.strip_suffix('\r').unwrap_or(raw);
        let chars: Vec<char> = text.chars().collect();
        end = chars.len() + 1;
        inside = scan_line(text, &chars, line, inside, &mut out);
    }
    out.push(Token::new(Tok::Eof, line, end, end));
    out
}

/// Columns are 1-based: char index `i` is column `i + 1`.
fn tok(kind: Tok, line: usize, start: usize, end: usize) -> Token<Tok> {
    Token::new(kind, line, start + 1, end + 1)
}

/// Scans one line; whether a string's text goes on after it.
fn scan_line(
    text: &str,
    chars: &[char],
    line: usize,
    inside: bool,
    out: &mut Vec<Token<Tok>>,
) -> bool {
    let first = chars.iter().position(|c| !c.is_whitespace());
    let last = chars
        .iter()
        .rposition(|c| !c.is_whitespace())
        .map_or(0, |i| i + 1);
    let trimmed = text.trim();
    if inside {
        if trimmed == "}" {
            let at = first.expect("a brace");
            out.push(tok(Tok::RBrace, line, at, at + 1));
            return false;
        }
        text_parts(chars, line, out);
        return true;
    }
    let Some(start) = first else {
        return false;
    };
    if trimmed == "{" {
        out.push(tok(Tok::LBrace, line, start, start + 1));
        return true;
    }
    if trimmed.starts_with("//") {
        out.push(tok(Tok::Comment(trimmed.to_string()), line, start, last));
        return false;
    }
    if let Some(rest) = trimmed.strip_prefix("STRING")
        && (rest.is_empty() || rest.starts_with(char::is_whitespace))
    {
        out.push(tok(Tok::StringKw, line, start, start + 6));
        let id_start = chars[start + 6..last]
            .iter()
            .position(|c| !c.is_whitespace())
            .map(|i| start + 6 + i);
        if let Some(s) = id_start {
            let id_end = chars[s..last]
                .iter()
                .position(|c| c.is_whitespace())
                .map_or(last, |i| s + i);
            out.push(tok(
                Tok::Id(chars[s..id_end].iter().collect()),
                line,
                s,
                id_end,
            ));
            if id_end < last {
                let junk_start = id_end
                    + chars[id_end..last]
                        .iter()
                        .position(|c| !c.is_whitespace())
                        .unwrap_or(0);
                out.push(tok(
                    Tok::Junk(chars[junk_start..last].iter().collect()),
                    line,
                    junk_start,
                    last,
                ));
            }
        }
        return false;
    }
    out.push(tok(Tok::Junk(trimmed.to_string()), line, start, last));
    false
}

/// A line of text: plain parts and the color codes between them.
fn text_parts(chars: &[char], line: usize, out: &mut Vec<Token<Tok>>) {
    let mut start = 0;
    let mut i = 0;
    let flush = |from: usize, to: usize, out: &mut Vec<Token<Tok>>| {
        if to > from {
            out.push(tok(
                Tok::Text(chars[from..to].iter().collect()),
                line,
                from,
                to,
            ));
        }
    };
    while i < chars.len() {
        let len = code_len(&chars[i..]);
        if len > 0 {
            flush(start, i, out);
            out.push(tok(
                Tok::ColorCode(chars[i..i + len].iter().collect()),
                line,
                i,
                i + len,
            ));
            i += len;
            start = i;
        } else {
            i += 1;
        }
    }
    flush(start, chars.len(), out);
}

/// The length of a color code at the start of `chars`, or 0: `|c` and
/// eight hex digits, `|r`, `|n`, in any case.
fn code_len(chars: &[char]) -> usize {
    if chars.first() != Some(&'|') {
        return 0;
    }
    match chars.get(1).map(char::to_ascii_lowercase) {
        Some('c') if chars.len() >= 10 && chars[2..10].iter().all(char::is_ascii_hexdigit) => 10,
        Some('r' | 'n') => 2,
        _ => 0,
    }
}

#[cfg(test)]
#[path = "lexer.test.rs"]
mod tests;
