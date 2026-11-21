use super::*;
use crate::parse;

#[test]
fn duplicates() {
    let doc = parse("STRING 1\n{\na\n}\nSTRING 001\n{\nb\n}\n");
    let found: Vec<(usize, &str, String)> = lint(&doc)
        .into_iter()
        .map(|d| (d.line, d.rule, d.message))
        .collect();
    assert_eq!(
        found,
        vec![(
            5,
            parser_rs::rule::DUPLICATE_STRING,
            "STRING 1 is already defined at line 1".into()
        )]
    );
}
