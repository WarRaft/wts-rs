//! WTS's token kinds: line roles (a header, a comment, a brace, a line of
//! text), not the shared `TokenKind` vocabulary. They still flow through
//! the shared [`Token`] plumbing.

use std::fmt;

pub use parser_rs::{ParseError, Token, TokenKindExt};

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    /// `STRING` at the start of a header line.
    StringKw,
    /// The id after `STRING`, as written: `0`, `011`.
    Id(String),
    /// `// ...` outside a string, to the end of the line.
    Comment(String),
    /// `{` opening a string's text.
    LBrace,
    /// One line of a string's text, as written.
    Text(String),
    /// A color code in a text line: `|cffff0000` (`|c` and eight hex
    /// digits), `|r` (back to the default color), `|n` (a line break).
    ColorCode(String),
    /// `}` closing a string's text.
    RBrace,
    /// A line, or the rest of one, that fits no form.
    Junk(String),
    Newline,
    Eof,
}

impl TokenKindExt for Tok {
    fn is_eof(&self) -> bool {
        matches!(self, Tok::Eof)
    }
}

impl fmt::Display for Tok {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tok::StringKw => write!(f, "'STRING'"),
            Tok::Id(id) => write!(f, "id {id}"),
            Tok::Comment(_) => write!(f, "comment"),
            Tok::LBrace => write!(f, "'{{'"),
            Tok::Text(_) => write!(f, "text"),
            Tok::ColorCode(c) => write!(f, "'{c}'"),
            Tok::RBrace => write!(f, "'}}'"),
            Tok::Junk(t) => write!(f, "'{t}'"),
            Tok::Newline => write!(f, "end of line"),
            Tok::Eof => write!(f, "end of file"),
        }
    }
}
