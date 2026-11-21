//! The WTS document: its strings, in order.

pub use parser_rs::{ColumnEncoding, ParseError, Span};

/// A parsed file. Parsing never fails as a whole: a line with a problem is
/// recorded in `errors` and the rest still parses.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Document {
    pub strings: Vec<Entry>,
    /// In source order.
    pub errors: Vec<ParseError>,
    /// The line break the file uses: `\r\n` or `\n`.
    pub newline: &'static str,
    /// Where the text ends: the last line and the column past its end.
    pub end: (usize, usize),
    /// Blank lines outside strings.
    pub blank_lines: Vec<usize>,
}

/// `src` without a leading byte-order mark, and whether it had one. Spans
/// and edits count from after it: give them the text without it, and put
/// it back when writing the file.
pub fn split_bom(src: &str) -> (&str, bool) {
    match src.strip_prefix('\u{FEFF}') {
        Some(rest) => (rest, true),
        None => (src, false),
    }
}

/// `STRING 3`, its comments, and its text between `{` and `}`.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// The number after `STRING`: `TRIGSTR_003` and `TRIGSTR_3` both name
    /// string 3. `None` when it isn't one.
    pub id: Option<u32>,
    /// The id as written: `003`.
    pub id_text: String,
    /// The header's line.
    pub line: usize,
    /// `STRING 3`.
    pub header: Span,
    /// The id, or the end of `STRING` without one.
    pub id_span: Span,
    /// `// ...` lines between the header and `{`, as written. World Editor
    /// says there what uses the string.
    pub comments: Vec<String>,
    /// The line of `{`, if any.
    pub open: Option<usize>,
    /// The line of `}`, if any.
    pub close: Option<usize>,
    /// The text: the lines between `{` and `}`, joined with `\n`.
    pub text: String,
}

impl Entry {
    /// Whether it has both braces.
    pub fn is_closed(&self) -> bool {
        self.open.is_some() && self.close.is_some()
    }
}

impl Document {
    /// The first string with this number.
    pub fn get(&self, id: u32) -> Option<&Entry> {
        self.strings.iter().find(|e| e.id == Some(id))
    }

    /// The text of the string a `TRIGSTR_...` reference names: `TRIGSTR_3`,
    /// `TRIGSTR_003` (the number may be padded), or `None`.
    pub fn lookup(&self, reference: &str) -> Option<&str> {
        let id = reference.strip_prefix("TRIGSTR_")?.parse().ok()?;
        self.get(id).map(|e| e.text.as_str())
    }

    /// `text` with every `TRIGSTR_<number>` that names a string of the
    /// file replaced by that string's text; others stay as written.
    pub fn resolve(&self, text: &str) -> String {
        let mut out = String::new();
        let mut rest = text;
        while let Some(at) = rest.find("TRIGSTR_") {
            out += &rest[..at];
            let after = &rest[at + 8..];
            let digits = after.len() - after.trim_start_matches(|c: char| c.is_ascii_digit()).len();
            let reference = &rest[at..at + 8 + digits];
            match self.lookup(reference).filter(|_| digits > 0) {
                Some(found) => out += found,
                None => out += reference,
            }
            rest = &rest[at + 8 + digits..];
        }
        out + rest
    }
}
