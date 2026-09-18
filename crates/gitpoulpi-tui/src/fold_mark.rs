//! The fold marker: one pair of glyphs for every expand / collapse arrow of
//! the app (Refs, Working Changes, the commit files, the hosting panes, the
//! management lists, the Radar, the Activity log, the review and compose
//! views). Both are one cell wide, text-presentation triangles (no emoji
//! presentation), so rows keep their alignment whichever state they show.
//!
//! A fold arrow is a control: the pointer is the hand over it (its owner
//! registers the cell through `pointer_map::hand` / its hit list), and its row
//! lights as a whole.

/// A collapsed (closed) group: points right.
pub(crate) const COLLAPSED: &str = "\u{25B8}";
/// An expanded (open) group: points down.
pub(crate) const EXPANDED: &str = "\u{25BE}";

/// The marker for a group that is `collapsed` or not.
pub(crate) fn glyph(collapsed: bool) -> &'static str {
    if collapsed {
        COLLAPSED
    } else {
        EXPANDED
    }
}

/// The marker for a group that is `expanded` or not.
pub(crate) fn glyph_open(expanded: bool) -> &'static str {
    glyph(!expanded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui_kit::display_width;

    #[test]
    fn both_glyphs_are_one_cell_text_triangles() {
        assert_eq!(display_width(COLLAPSED), 1);
        assert_eq!(display_width(EXPANDED), 1);
        assert_ne!(COLLAPSED, EXPANDED);
        assert_eq!(glyph(true), COLLAPSED);
        assert_eq!(glyph_open(true), EXPANDED);
    }

    /// No screen draws its own fold triangle: the glyphs live here only
    /// (a literal `\u{25B8}` / `\u{25BE}` outside this module, tests and
    /// non-fold uses listed below, is a hand-drawn marker).
    #[test]
    fn no_surface_hand_draws_a_fold_marker() {
        // Files whose triangles are not fold markers (dropdown carets, drag
        // arrows).
        const NOT_FOLD: &[&str] = &[
            "combo.rs",
            "ref_drop.rs",
            "refs_sidebar.rs",
            "composer.rs",
            "chrome/toolbar.rs",
            "diff_view/mod.rs",
            "folder_picker/render.rs",
            "preferences/field_layout.rs",
            "preferences/list_editor.rs",
            "preferences/quantity.rs",
            "rebase_ui/render.rs",
            "rewrite_view/render.rs",
            "split_ui/render.rs",
            "hosting_ui/rows.rs",
        ];
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut stack = vec![root.clone()];
        let mut offenders = Vec::new();
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).unwrap().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                let rel = path
                    .strip_prefix(&root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                let name = rel.rsplit('/').next().unwrap_or("");
                if !rel.ends_with(".rs")
                    || rel == "fold_mark.rs"
                    || NOT_FOLD.contains(&rel.as_str())
                    || rel.contains("tests")
                    || name.ends_with("_tests.rs")
                    || name == "tests.rs"
                {
                    continue;
                }
                let text = std::fs::read_to_string(&path).unwrap();
                for (n, line) in text.lines().enumerate() {
                    let code = line.split("//").next().unwrap_or("");
                    let lower = code.to_lowercase();
                    if lower.contains("u{25b8}") || lower.contains("u{25be}") {
                        offenders.push(format!("{rel}:{}", n + 1));
                    }
                }
            }
        }
        assert!(
            offenders.is_empty(),
            "hand-drawn fold markers: {offenders:?}"
        );
    }
}
