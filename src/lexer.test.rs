use super::*;

fn kinds(src: &str) -> Vec<Tok> {
    tokenize(src).into_iter().map(|t| t.kind).collect()
}

#[test]
fn a_block() {
    let got: Vec<(Tok, usize, usize, usize)> =
        tokenize("\u{FEFF}STRING 3\r\n// a note\r\n{\r\nKnight\r\n}\r\n")
            .into_iter()
            .map(|t| (t.kind, t.line, t.col, t.end_col))
            .collect();
    assert_eq!(
        got,
        vec![
            (Tok::StringKw, 1, 1, 7),
            (Tok::Id("3".into()), 1, 8, 9),
            (Tok::Newline, 1, 9, 10),
            (Tok::Comment("// a note".into()), 2, 1, 10),
            (Tok::Newline, 2, 10, 11),
            (Tok::LBrace, 3, 1, 2),
            (Tok::Newline, 3, 2, 3),
            (Tok::Text("Knight".into()), 4, 1, 7),
            (Tok::Newline, 4, 7, 8),
            (Tok::RBrace, 5, 1, 2),
            (Tok::Newline, 5, 2, 3),
            (Tok::Eof, 6, 1, 1),
        ]
    );
}

#[test]
fn inside_text_everything_is_text() {
    let toks = kinds("STRING 1\n{\nSTRING 2\n// not a comment\n {\n  }\n");
    assert_eq!(
        toks,
        vec![
            Tok::StringKw,
            Tok::Id("1".into()),
            Tok::Newline,
            Tok::LBrace,
            Tok::Newline,
            Tok::Text("STRING 2".into()),
            Tok::Newline,
            Tok::Text("// not a comment".into()),
            Tok::Newline,
            Tok::Text(" {".into()),
            Tok::Newline,
            // `}` alone, spaces around it, ends the text.
            Tok::RBrace,
            Tok::Newline,
            Tok::Eof,
        ]
    );
}

#[test]
fn color_codes() {
    let toks = kinds("STRING 1\n{\na|cffFF0000red|r|nb|cbad\n}");
    assert_eq!(
        toks[5..10],
        [
            Tok::Text("a".into()),
            Tok::ColorCode("|cffFF0000".into()),
            Tok::Text("red".into()),
            Tok::ColorCode("|r".into()),
            Tok::ColorCode("|n".into()),
        ]
    );
    // `|cbad` isn't a color: text.
    assert_eq!(toks[10], Tok::Text("b|cbad".into()));
}

#[test]
fn headers_and_junk() {
    assert_eq!(kinds("STRING")[..1], [Tok::StringKw]);
    assert_eq!(
        kinds("STRING 4 x")[..3],
        [Tok::StringKw, Tok::Id("4".into()), Tok::Junk("x".into())]
    );
    assert_eq!(kinds("STRINGS 4")[0], Tok::Junk("STRINGS 4".into()));
    assert_eq!(kinds("hello")[0], Tok::Junk("hello".into()));
}
