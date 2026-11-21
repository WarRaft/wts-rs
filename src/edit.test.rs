use super::*;
use crate::parse;

const FILE: &str =
    "STRING 1\r\n// Units: h000\r\n{\r\nKnight\r\n}\r\n\r\nSTRING 2\r\n{\r\nold\r\nlines\r\n}\r\n";

fn set(src: &str, id: u32, text: &str) -> String {
    let out = apply(src, &set_string(&parse(src), id, text).unwrap());
    assert_eq!(parse(&out).errors, vec![], "{out}");
    out
}

#[test]
fn replace_add_remove() {
    // Only the text's lines change; the comment and the rest stay.
    assert_eq!(
        set(FILE, 2, "new"),
        FILE.replace("old\r\nlines\r\n", "new\r\n")
    );
    assert_eq!(
        set(FILE, 1, "a\nb\n\nc"),
        FILE.replace("Knight\r\n", "a\r\nb\r\n\r\nc\r\n")
    );
    assert_eq!(set(FILE, 1, ""), FILE.replace("Knight\r\n", "\r\n"));
    assert!(set_string(&parse(FILE), 1, "Knight").unwrap().is_empty());
    // A new one at the end, after a blank line.
    assert_eq!(
        set(FILE, 7, "Footman"),
        format!("{FILE}\r\nSTRING 7\r\n{{\r\nFootman\r\n}}\r\n")
    );
    assert_eq!(
        set("STRING 1\n{\nx\n}", 2, "y"),
        "STRING 1\n{\nx\n}\n\nSTRING 2\n{\ny\n}\n"
    );
    // Removing takes the blank line after it.
    let out = apply(FILE, &remove_string(&parse(FILE), 1).unwrap());
    assert_eq!(out, "STRING 2\r\n{\r\nold\r\nlines\r\n}\r\n");
    assert!(remove_string(&parse(FILE), 9).is_err());
}

#[test]
fn what_can_not_be_written() {
    assert!(
        set_string(&parse(FILE), 1, "a\n }\nb")
            .unwrap_err()
            .contains("only `}`")
    );
    assert!(
        set_string(&parse("STRING 1\n{\nx\n"), 1, "y")
            .unwrap_err()
            .contains("isn't closed")
    );
}
