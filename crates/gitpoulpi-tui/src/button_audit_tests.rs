//! Guard: buttons are real buttons, never bracketed text (`[ Edit ]`,
//! `[+ Add]`, `[ 1 ▾]`). A clickable control is drawn through the button
//! widget (`dialog_kit::button` / `button_style`: padded, filled, hover and
//! disabled looks, the hand pointer), so nothing in the screen code builds a
//! label with a bracket around it, and no catalog label carries one.
//!
//! Checkboxes (`[x]` / `[ ]`, `dialog_kit::checkbox_mark`) are marks, not
//! buttons, and are not flagged; they are hand controls like any other (the
//! hover audit, `pty_hover_audit`, and the pointer-map tests cover that).
//!
//! The scan reads the source text of every non-test file, so a regression
//! fails here before any screen is drawn.

use std::path::{Path, PathBuf};

/// Bracketed-button shapes in a string literal: a label wrapped in `[ ... ]`
/// or `[+ ...]`, a dropdown `[.. ▾]`.
const FORBIDDEN: &[&str] = &[
    "\"[ {",
    "\"[+ {",
    "\"[+ \"",
    "\"[ \"",
    "\"[{} \\u{25BE}",
    "\\u{25BE}]",
    " {text} ]",
    " {label} ]",
    " {} ]\"",
];

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n == "tests") {
                continue;
            }
            walk(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            if name == "tests.rs" || name.ends_with("_tests.rs") || name.starts_with("tests_") {
                continue;
            }
            out.push(path);
        }
    }
}

/// The file's own code, without its inline `#[cfg(test)]` module.
fn code_of(source: &str) -> &str {
    source
        .rfind("#[cfg(test)]\nmod ")
        .map_or(source, |at| &source[..at])
}

/// Lines that are not comments (a doc comment may draw a diagram).
fn code_lines(source: &str) -> impl Iterator<Item = (usize, &str)> {
    source
        .lines()
        .enumerate()
        .filter(|(_, l)| !l.trim_start().starts_with("//"))
}

/// Red if any screen code builds a label wrapped in brackets.
#[test]
fn no_screen_code_draws_a_bracketed_button_label() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    walk(&root, &mut files);
    let mut found = Vec::new();
    for file in files {
        let source = std::fs::read_to_string(&file).unwrap();
        for (n, line) in code_lines(code_of(&source)) {
            if line.contains("//") && !line.contains('"') {
                continue;
            }
            for pattern in FORBIDDEN {
                if line.contains(pattern) {
                    found.push(format!(
                        "{}:{}: {}",
                        file.strip_prefix(&root).unwrap().display(),
                        n + 1,
                        line.trim()
                    ));
                }
            }
        }
    }
    assert!(
        found.is_empty(),
        "bracketed button labels (draw `dialog_kit::button` instead):\n{}",
        found.join("\n")
    );
}

/// Red if a catalog label (any language) is a bracketed word.
#[test]
fn no_catalog_label_is_wrapped_in_brackets() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../gitpoulpi-i18n/locales");
    let mut checked = 0;
    let mut found = Vec::new();
    for entry in std::fs::read_dir(&dir).unwrap().flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "toml") {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        for (n, line) in source.lines().enumerate() {
            let Some((key, value)) = line.split_once(" = ") else {
                continue;
            };
            let value = value.trim();
            let bracketed = value.starts_with("\"[ ")
                || value.starts_with("\"[+")
                || (value.starts_with("\"[") && value.ends_with("]\"") && value.contains(' '));
            // The `[base]` / `[base, protected]` tags are badges, not buttons.
            if bracketed && !key.starts_with("badge_") {
                found.push(format!(
                    "{}:{}: {}",
                    path.file_name().unwrap().to_string_lossy(),
                    n + 1,
                    line.trim()
                ));
            }
            checked += 1;
        }
    }
    assert!(checked > 1000, "the scan read only {checked} lines");
    assert!(found.is_empty(), "{}", found.join("\n"));
}

/// The scan itself must be able to fail: a bracketed label is caught.
#[test]
fn the_scan_catches_a_bracketed_label() {
    let bad = "let shown = format!(\"[ {text} ]\");";
    assert!(FORBIDDEN.iter().any(|p| bad.contains(p)));
    let bad = "let shown = format!(\"[+ {}]\", t);";
    assert!(FORBIDDEN.iter().any(|p| bad.contains(p)));
    let ok = "let shown = format!(\" {text} \");";
    assert!(!FORBIDDEN.iter().any(|p| ok.contains(p)));
}
