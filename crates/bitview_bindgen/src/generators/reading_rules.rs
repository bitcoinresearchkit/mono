//! How to read a series' values, written once for people and agents alike: markdown for the
//! skill and the MCP instructions, HTML for the support page's block.

use std::fmt::Write;

const RULES: &[&str] = &[
    "`end` is exclusive: one block is start 900000, end 900001; one day is start 2025-01-01, end 2025-01-02.",
    "A `Ratio` or `Multiple` is a plain number (0.55, not 55%); a `Percent` is already a percent (55 means 55%).",
    "`null` means missing or undefined (before the data exists, or a period with no block), never zero.",
    "`price` reads 0, not null, before block 74,641 (August 2010): there was no price yet.",
];
const _: () = assert!(code_spans_closed(RULES), "a reading rule has an unclosed code span");

/// Whether every rule has an even number of backticks.
const fn code_spans_closed(rules: &[&str]) -> bool {
    let mut rule = 0;
    while rule < rules.len() {
        let (bytes, mut i, mut ticks) = (rules[rule].as_bytes(), 0, 0);
        while i < bytes.len() {
            ticks += (bytes[i] == b'`') as usize;
            i += 1;
        }
        if ticks % 2 == 1 {
            return false;
        }
        rule += 1;
    }
    true
}

/// The rules as markdown list items.
pub(crate) fn markdown() -> impl Iterator<Item = String> {
    RULES.iter().map(|rule| format!("- {rule}\n"))
}

/// The rules as an HTML list at `indent`, their `code` spans as `<code>`, the rest escaped.
pub(crate) fn html(indent: &str) -> String {
    let mut html = format!("{indent}<ul>\n");
    for rule in RULES {
        let item = rule
            .split('`')
            .enumerate()
            .map(|(i, part)| {
                let part = part.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
                match i % 2 {
                    1 => format!("<code>{part}</code>"),
                    _ => part,
                }
            })
            .collect::<String>();
        writeln!(html, "{indent}  <li>{item}</li>").unwrap();
    }
    writeln!(html, "{indent}</ul>").unwrap();
    html
}
