//! Guard: a window over a longer list (`.skip(top).take(rows)`) is a
//! scrollable area, and every scrollable area shows the shared scrollbar.
//! A source file that paints such a window must go through the scrollbar
//! code (`scrollbar::`, `Track::`, `ListBar`, `paint_bar`, `BarDrag`, the
//! combobox or popup list) or be on `PENDING`, which is the list of areas
//! still without one - it only ever shrinks. A new scrolling list in a new
//! file fails here until it draws its bar.

use std::path::Path;

/// Files with a windowed list and no bar yet (or no list the user sees:
/// byte windows of text, image cells), each with why.
const PENDING: &[&str] = &[
    // Not a list the user scrolls: pixel / text-cell windows.
    "commit_column/picture.rs",
    "text_select.rs",
];

const BAR_MARKERS: &[&str] = &[
    "scrollbar::",
    "Track::",
    "ListBar",
    "paint_bar",
    "BarDrag",
    "render_buf",
    "popup_list",
    "_bar.paint(",
];

fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
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

/// The file's own code: an inline `#[cfg(test)]` module windows lists of
/// test fixtures, not of the screen.
fn code_of(source: &str) -> &str {
    source
        .rfind("#[cfg(test)]\nmod ")
        .map_or(source, |at| &source[..at])
}

fn windows_a_list(source: &str) -> bool {
    let squeezed: String = source.split_whitespace().collect();
    let mut rest = squeezed.as_str();
    while let Some(at) = rest.find(".skip(") {
        let after = &rest[at + 6..];
        // The closing parenthesis of the skip, then `.take(`.
        let mut depth = 1;
        let mut end = None;
        for (i, c) in after.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }
        if let Some(end) = end {
            if after[end + 1..].starts_with(".take(") {
                return true;
            }
        }
        rest = after;
    }
    windows_a_range(&squeezed)
}

/// `(first..len).take(rows)`: a window cut from a range of indexes.
fn windows_a_range(squeezed: &str) -> bool {
    let bytes = squeezed.as_bytes();
    let mut from = 0;
    while let Some(at) = squeezed[from..].find(").take(") {
        let close = from + at;
        from = close + 1;
        // Walk back to the matching open parenthesis.
        let mut depth = 0usize;
        let mut open = None;
        for i in (0..=close).rev() {
            match bytes[i] {
                b')' => depth += 1,
                b'(' => {
                    depth -= 1;
                    if depth == 0 {
                        open = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }
        // A bare parenthesised range, not a call's arguments (`zip(a[..n]..)`);
        // `in(` is the `for .. in (..)` of a loop.
        let bare = |open: usize| {
            let word = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
            open == 0
                || !word(bytes[open - 1])
                || (open >= 2
                    && &bytes[open - 2..open] == b"in"
                    && (open < 3 || !word(bytes[open - 3])))
        };
        if let Some(open) = open.filter(|&o| bare(o)) {
            let inside = &squeezed[open + 1..close];
            // A range with a variable start: `offset..list.len()`.
            if let Some((start, _)) = inside.split_once("..") {
                if !start.is_empty() && start != "0" && !start.contains('(') {
                    return true;
                }
            }
        }
    }
    false
}

#[test]
fn every_windowed_list_draws_the_scrollbar() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    walk(&src, &mut files);
    let mut missing = Vec::new();
    for path in files {
        let rel = path
            .strip_prefix(&src)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let text = std::fs::read_to_string(&path).unwrap();
        let text = code_of(&text);
        if windows_a_list(text)
            && !BAR_MARKERS.iter().any(|m| text.contains(m))
            && !PENDING.contains(&rel.as_str())
        {
            missing.push(rel);
        }
    }
    assert!(
        missing.is_empty(),
        "these files window a list without the shared scrollbar: {missing:?}"
    );
    // The allow-list only shrinks: an entry that now has its bar goes.
    for rel in PENDING {
        let text = std::fs::read_to_string(src.join(rel)).unwrap();
        let text = code_of(&text);
        assert!(
            windows_a_list(text),
            "{rel} windows no list any more: take it off PENDING"
        );
        assert!(
            !BAR_MARKERS.iter().any(|m| text.contains(m)),
            "{rel} has its bar now: take it off PENDING"
        );
    }
}

/// A list drawn through ratatui's own scrolling (`List` / `Table` with a
/// state, a `Paragraph` with `.scroll(..)`) scrolls just as much as a
/// `.skip().take()` window: a file that renders one must go through the
/// scrollbar code too, or be listed here with why it has none.
const STATEFUL_PENDING: &[&str] = &[];

fn scrolls_through_ratatui(source: &str) -> bool {
    [
        "render_stateful_widget",
        "render_stateful_clipped",
        ".scroll((",
    ]
    .iter()
    .any(|m| source.contains(m))
}

#[test]
fn every_stateful_list_and_scrolled_paragraph_draws_the_scrollbar() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    walk(&src, &mut files);
    let mut missing = Vec::new();
    for path in files {
        let rel = path
            .strip_prefix(&src)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let text = std::fs::read_to_string(&path).unwrap();
        let text = code_of(&text);
        if scrolls_through_ratatui(text)
            && !BAR_MARKERS.iter().any(|m| text.contains(m))
            && !STATEFUL_PENDING.contains(&rel.as_str())
        {
            missing.push(rel);
        }
    }
    assert!(
        missing.is_empty(),
        "these files scroll a List, Table or Paragraph without the shared scrollbar: {missing:?}"
    );
}
