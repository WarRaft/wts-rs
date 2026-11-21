use super::*;

#[test]
fn every_part_has_a_color() {
    let got: Vec<(usize, usize, usize, T)> =
        semantic_tokens("STRING 1\n// note\n{\nhi |cffff0000x|r\n}\n")
            .into_iter()
            .map(|t| (t.line, t.start_col, t.end_col, t.ty))
            .collect();
    assert_eq!(
        got,
        vec![
            (1, 1, 7, T::Keyword),
            (1, 8, 9, T::Number),
            (2, 1, 8, T::Comment),
            (3, 1, 2, T::Operator),
            (4, 1, 4, T::String),
            (4, 4, 14, T::Regexp),
            (4, 14, 15, T::String),
            (4, 15, 17, T::Regexp),
            (5, 1, 2, T::Operator),
        ]
    );
}
