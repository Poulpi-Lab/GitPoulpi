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
    "image_view/header.rs",
    "text_select.rs",
    "file_selection.rs",
    "log_multiselect.rs",
    // A menu taller than the screen follows its selection (no scroll state
    // to drag).
    "context_menu.rs",
    // Lists still to get their bar.
    "folder_picker/render.rs",
    "repo_switcher.rs",
    // Another change owns it.
    "rewrite_view/script_ai.rs",
];

const BAR_MARKERS: &[&str] = &[
    "scrollbar::",
    "Track::",
    "ListBar",
    "paint_bar",
    "BarDrag",
    "render_buf",
    "popup_list",
    "Combo",
    "combo::",
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
        if windows_a_list(&text)
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
        assert!(
            !BAR_MARKERS.iter().any(|m| text.contains(m)) || !windows_a_list(&text),
            "{rel} has its bar now: take it off PENDING"
        );
    }
}
