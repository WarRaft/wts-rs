//! Semantic highlighting for WTS, straight from the scanner's tokens, so
//! it works on any file, broken or not:
//!
//! - `STRING`: `keyword`; its number: `number`;
//! - comments: `comment`; braces: `operator`;
//! - text: `string`; color codes (`|cffff0000`, `|r`, `|n`): `regexp`.

use crate::lexer::tokenize;
use crate::token::Tok;
use parser_rs::semantic;
use parser_rs::{Modifiers, SemanticToken, SemanticType as T};

/// Every semantic token in `src`, unencoded.
pub fn semantic_tokens(src: &str) -> Vec<SemanticToken> {
    tokenize(src)
        .iter()
        .filter_map(|t| {
            let ty = match &t.kind {
                Tok::StringKw => T::Keyword,
                Tok::Id(_) => T::Number,
                Tok::Comment(_) => T::Comment,
                Tok::LBrace | Tok::RBrace => T::Operator,
                Tok::Text(_) => T::String,
                Tok::ColorCode(_) => T::Regexp,
                _ => return None,
            };
            Some(SemanticToken::at(t, ty, Modifiers::NONE))
        })
        .collect()
}

/// [`semantic_tokens`] in the editor's wire format.
pub fn encoded_semantic_tokens(src: &str) -> Vec<u32> {
    semantic::encode(semantic_tokens(src), src)
}

#[cfg(test)]
#[path = "semantic.test.rs"]
mod tests;
