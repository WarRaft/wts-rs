use std::env;
use std::fs;
use std::process::ExitCode;

use wts_rs::{Level, lint, render_dot, render_html, render_tree};

enum Mode {
    Lint,
    Ast,
    Dot,
    Html,
    Get(u32),
    Resolve(String),
}

fn usage() {
    eprintln!("usage: wts-rs [--ast|--dot|--html] <war3map.wts>");
    eprintln!("       wts-rs --get <number> <war3map.wts>      print one string's text");
    eprintln!("       wts-rs --resolve <text> <war3map.wts>    replace TRIGSTR_<n> in the text");
    eprintln!("  (no flag)  check the file and print its problems");
    eprintln!("  --ast      print the strings as an indented tree");
    eprintln!("  --dot      print the strings as a Graphviz DOT graph");
    eprintln!("  --html     print a self-contained viewer page (source + tree)");
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let (mode, path) = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        [path] if !path.starts_with('-') => (Mode::Lint, path.to_string()),
        ["--ast", path] => (Mode::Ast, path.to_string()),
        ["--dot", path] => (Mode::Dot, path.to_string()),
        ["--html", path] => (Mode::Html, path.to_string()),
        ["--get", n, path] => match n.parse() {
            Ok(n) => (Mode::Get(n), path.to_string()),
            Err(_) => {
                usage();
                return ExitCode::FAILURE;
            }
        },
        ["--resolve", text, path] => (Mode::Resolve(text.to_string()), path.to_string()),
        ["-h" | "--help"] => {
            usage();
            return ExitCode::SUCCESS;
        }
        _ => {
            usage();
            return ExitCode::FAILURE;
        }
    };
    let src = match fs::read(&path) {
        Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        Err(err) => {
            eprintln!("{path}: {err}");
            return ExitCode::FAILURE;
        }
    };
    // Parsing never fails as a whole.
    let doc = wts_rs::parse(&src);
    match mode {
        Mode::Ast => print!("{}", render_tree(&doc)),
        Mode::Dot => print!("{}", render_dot(&doc)),
        Mode::Html => print!("{}", render_html(&doc, &src, &path)),
        Mode::Get(n) => match doc.get(n) {
            Some(entry) => println!("{}", entry.text),
            None => {
                eprintln!("{path}: no STRING {n}");
                return ExitCode::FAILURE;
            }
        },
        Mode::Resolve(text) => println!("{}", doc.resolve(&text)),
        Mode::Lint => {
            let diagnostics = lint(&doc);
            for err in &doc.errors {
                println!("{path}:{}:{}: error: {}", err.line, err.col, err.message);
            }
            for d in &diagnostics {
                println!(
                    "{path}:{}: {} [{}]: {}",
                    d.line,
                    d.level.as_str(),
                    d.rule,
                    d.message
                );
            }
            if doc.errors.is_empty() && diagnostics.is_empty() {
                println!("{path}: {} strings, no issues found", doc.strings.len());
            }
            if !doc.errors.is_empty() || diagnostics.iter().any(|d| d.level == Level::Error) {
                return ExitCode::FAILURE;
            }
        }
    }
    ExitCode::SUCCESS
}
