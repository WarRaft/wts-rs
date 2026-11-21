# wts-rs

A parser, editor and semantic highlighter for **WTS** (`war3map.wts`,
`war3campaign.wts`), the string table of Warcraft III maps: the text that
object data, triggers and the map's info name as `TRIGSTR_<n>`. Structured
like the other language crates; everything not WTS-specific comes from
[`parser-rs`](../parser-rs/README.md).

```rust
let doc = wts_rs::parse("STRING 3\n{\nKnight\n}\n");
assert_eq!(doc.lookup("TRIGSTR_003"), Some("Knight"));
assert_eq!(doc.resolve("Name: TRIGSTR_3"), "Name: Knight");

// Write a string: the text edits that do it, applied to the source.
let edits = wts_rs::set_string(&doc, 3, "Knight of the Hand").unwrap();
```

## The format

```text
STRING 0
// Units: h000 (Knight), Name (unam)
{
Knight
}

STRING 1
{
A |cffffcc00holy|r warrior.|nStrong against undead.
}
```

- **Blocks.** `STRING` and a number, then `// comments` (World Editor says
  there what uses the string), then the text between `{` and `}`, each on
  a line of its own. Blank lines between blocks.
- **Numbers.** `TRIGSTR_003` and `TRIGSTR_3` both name string 3: the number
  may be padded, in the file and in references.
- **Text** is every line between `{` and the first line that is `}` alone
  (spaces around it allowed), as written: a line reading `STRING 4` or
  `// ...` in there is text. So a string can't hold a line that is only
  `}`.
- **Color codes** inside text: `|c` and eight hex digits (alpha, red,
  green, blue) start a color, `|r` ends it, `|n` is a line break.
- **Files** start with a UTF-8 byte-order mark and use `\r\n`, as World
  Editor writes them; both are optional.

## Parser status

Parsing never fails as a whole: a line with a problem is recorded in
`Document::errors` and the rest still parses. Checked: a header is
`STRING` and a number, alone on its line; a header is followed by `{`, and
`{` follows a header; a string ends with `}`; outside strings a line is a
header, a brace, a comment or blank.

Checked on 209 files from the game's maps and campaigns and custom maps:
26,539 strings, no problems; every text matches a separate reading.

### Lint

| rule               | finds |
|--------------------|-------|
| `duplicate-string` | a string number defined twice |

### Lookups

`Document::get(n)`, `lookup("TRIGSTR_003")` and `resolve(text)`, which
replaces every `TRIGSTR_<n>` the file has with its text.

### Editing

`set_string(&doc, n, text)` gives the text edits that write string `n`:
its lines between the braces replaced, or a new block at the end in World
Editor's layout; `remove_string(&doc, n)` drops a block and the blank line
after it. Everything else stays as it was, line breaks included;
`apply(src, &edits)` applies them. Spans count after a byte-order mark:
`split_bom` takes it off, and a tool puts it back when it writes the file.

Checked over the corpus: 3,574 strings set and removed and a new one
added per file, each changing that string and nothing else.

## Highlighting

Straight from the scanner, so it works on any file, broken or not:
`STRING` `keyword`, its number `number`, comments `comment`, braces
`operator`, text `string`, color codes `regexp`.

## CLI

```text
wts-rs [--ast|--dot|--html] <war3map.wts>
wts-rs --get <number> <war3map.wts>      print one string's text
wts-rs --resolve <text> <war3map.wts>    replace TRIGSTR_<n> in the text
```

## Tests

```sh
cargo test
WTS_CORPUS=path/to/wts/files cargo test --release --test corpus -- --ignored
UPDATE_SNAPSHOTS=1 cargo test --test html_snapshot   # after an intended change
```
