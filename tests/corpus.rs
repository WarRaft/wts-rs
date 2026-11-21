//! Every `.wts` file under `$WTS_CORPUS` parses without problems, and
//! setting, adding or removing a string changes that string and nothing
//! else: `WTS_CORPUS=dir cargo test --release --test corpus -- --ignored`.

use std::path::{Path, PathBuf};

use wts_rs::{Document, apply, lint, parse, remove_string, set_string, split_bom};

fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            files(&path, out);
        } else if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("wts"))
        {
            out.push(path);
        }
    }
}

/// Every string as (id, text).
fn strings(doc: &Document) -> Vec<(Option<u32>, String)> {
    doc.strings.iter().map(|e| (e.id, e.text.clone())).collect()
}

fn after(path: &Path, src: &str, edits: &[parser_rs::Edit]) -> Document {
    let out = apply(src, edits);
    let doc = parse(&out);
    assert_eq!(doc.errors, vec![], "{}", path.display());
    doc
}

#[test]
#[ignore]
fn corpus() {
    let dir = std::env::var("WTS_CORPUS").expect("set WTS_CORPUS to a folder of .wts files");
    let mut paths = Vec::new();
    files(Path::new(&dir), &mut paths);
    assert!(!paths.is_empty(), "no .wts files in {dir}");
    let (mut count, mut edits) = (0, 0);
    for path in &paths {
        let bytes = std::fs::read(path).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        let (src, _) = split_bom(&text);
        let doc = parse(src);
        assert_eq!(doc.errors, vec![], "{}", path.display());
        assert!(lint(&doc).is_empty(), "{}", path.display());
        count += doc.strings.len();
        let before = strings(&doc);
        let ids: Vec<u32> = doc.strings.iter().filter_map(|e| e.id).collect();
        for &id in ids.iter().step_by(ids.len() / 10 + 1) {
            let new = "edited\n\n|cffff0000line|r three";
            let got = strings(&after(path, src, &set_string(&doc, id, new).unwrap()));
            let want: Vec<_> = before
                .iter()
                .map(|(i, t)| {
                    (
                        *i,
                        if *i == Some(id) {
                            new.to_string()
                        } else {
                            t.clone()
                        },
                    )
                })
                .collect();
            assert_eq!(got, want, "{}: setting {id}", path.display());
            let got = strings(&after(path, src, &remove_string(&doc, id).unwrap()));
            let want: Vec<_> = before
                .iter()
                .filter(|(i, _)| *i != Some(id))
                .cloned()
                .collect();
            assert_eq!(got, want, "{}: removing {id}", path.display());
            edits += 2;
        }
        let fresh = ids.iter().max().map_or(0, |m| m + 1);
        let got = strings(&after(path, src, &set_string(&doc, fresh, "new").unwrap()));
        let mut want = before.clone();
        want.push((Some(fresh), "new".into()));
        assert_eq!(got, want, "{}: adding", path.display());
    }
    eprintln!("{} files, {count} strings, {edits} edits", paths.len());
}
