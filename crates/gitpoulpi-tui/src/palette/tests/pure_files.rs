//! The vocabulary files the palette's payloads and availability types live in
//! (now thin re-exports of `gitpoulpi_app::palette::vocab`) stay free of terminal types, so a frontend other than the terminal can
//! share them: they name neither `ratatui` nor `crossterm` and pull nothing in
//! through `use super::*`.

use std::path::Path;

/// Source files (relative to `src/`) that must stay terminal-free.
const PURE_FILES: &[&str] = &[
    "activity_log/cmd.rs",
    "commit_column/vocab.rs",
    "diff_view/mode.rs",
    "editor/vocab.rs",
    "hosting_command.rs",
    "hscroll/vocab.rs",
    "jobs_ui/cmd.rs",
    "log_menu/vocab.rs",
    "log_table/vocab.rs",
    "merge_view/cmd.rs",
    "pane_layout/vocab.rs",
    "ref_menu/vocab.rs",
    "refs_sidebar/vocab.rs",
    "reword_ui/vocab.rs",
    "rewrite_view/vocab.rs",
    "safety_panel/vocab.rs",
    "status_pane/vocab.rs",
    "tree_fold/vocab.rs",
];

#[test]
fn vocabulary_files_name_no_terminal_type() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut bad = Vec::new();
    for rel in PURE_FILES {
        let text = std::fs::read_to_string(root.join(rel))
            .unwrap_or_else(|e| panic!("{rel} must exist: {e}"));
        for (i, line) in text.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            for needle in ["ratatui", "crossterm", "use super::*"] {
                if line.contains(needle) {
                    bad.push(format!("{rel}:{}: {needle}", i + 1));
                }
            }
        }
    }
    assert!(bad.is_empty(), "terminal-free files reach for: {bad:#?}");
}
