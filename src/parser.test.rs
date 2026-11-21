use crate::parse;

#[test]
fn strings_comments_and_text() {
    let doc = parse(
        "\u{FEFF}STRING 3\r\n// Units: h000\r\n{\r\nLine one\r\n\r\n|cffff0000Red|r\r\n}\r\n\r\nSTRING 011\r\n{\r\n}\r\n",
    );
    assert_eq!(doc.errors, vec![]);
    assert_eq!(doc.newline, "\r\n");
    assert_eq!(doc.strings.len(), 2);
    let a = &doc.strings[0];
    assert_eq!(
        (a.id, a.id_text.as_str(), a.line, a.open, a.close),
        (Some(3), "3", 1, Some(3), Some(7))
    );
    assert_eq!(a.comments, vec!["// Units: h000".to_string()]);
    assert_eq!(a.text, "Line one\n\n|cffff0000Red|r");
    let b = &doc.strings[1];
    assert_eq!((b.id, b.text.as_str()), (Some(11), ""));
    assert_eq!(doc.get(11).map(|e| e.line), Some(9));
    assert_eq!(doc.blank_lines, vec![8, 12]);
    assert_eq!(doc.end, (12, 1));
}

fn errors(src: &str) -> Vec<(usize, usize, String)> {
    parse(src)
        .errors
        .into_iter()
        .map(|e| (e.line, e.col, e.message))
        .collect()
}

#[test]
fn every_problem_is_reported() {
    assert_eq!(
        errors("hello\nSTRING\nSTRING x\n{\n}\nSTRING 5\nSTRING 6 extra\n{\nopen"),
        vec![
            (
                1,
                1,
                "expected `STRING <number>`, a `// comment`, '{' or '}', found 'hello'".into()
            ),
            (2, 1, "expected the string's number after STRING".into()),
            (2, 1, "STRING has no text: '{' missing".into()),
            (3, 8, "a string's id is a number, not 'x'".into()),
            (6, 1, "STRING 5 has no text: '{' missing".into()),
            (7, 1, "STRING 6 isn't closed: '}' missing".into()),
            (
                7,
                10,
                "expected only the string's number after STRING".into()
            ),
        ]
    );
    assert_eq!(
        errors("{\n}\n"),
        vec![
            (1, 1, "'{' without a STRING header before it".into()),
            (2, 1, "'}' without a string before it".into())
        ]
    );
}

#[test]
fn lookup_and_resolve() {
    let doc = parse("STRING 3\n{\nKnight\n}\nSTRING 4\n{\nof the Hand\n}\n");
    assert_eq!(doc.lookup("TRIGSTR_003"), Some("Knight"));
    assert_eq!(doc.lookup("TRIGSTR_9"), None);
    assert_eq!(doc.lookup("TRIGSTR_"), None);
    assert_eq!(
        doc.resolve("TRIGSTR_3 TRIGSTR_004, TRIGSTR_9 TRIGSTR_x"),
        "Knight of the Hand, TRIGSTR_9 TRIGSTR_x"
    );
}
