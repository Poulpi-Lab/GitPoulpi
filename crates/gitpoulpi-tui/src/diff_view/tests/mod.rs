use gitpoulpi_core::domain::{DiffLine, FileDiff, Hunk, HunkId, LineOrigin, Oid};

use super::interaction::{
    apply_cursor, apply_highlight, discard_hunk_label, hunk_button, line_discard_label,
    line_stage_label, line_unstage_label, row_buttons,
};
use super::layout::{text_viewport, GUTTER_WIDTH};
use super::model::{lfs_card, render_diff, render_diff_scrolled, RowChange};
use super::*;
use crate::display_width;
use crate::text_select::{self, TextSelection};
use std::path::PathBuf;

fn test_oid() -> Oid {
    Oid([7u8; 20])
}

fn line(origin: LineOrigin, content: &str) -> DiffLine {
    DiffLine {
        origin,
        content: content.to_string(),
        staged: false,
    }
}

fn hunk(lines: Vec<DiffLine>) -> Hunk {
    Hunk {
        id: gitpoulpi_core::domain::HunkId(0),
        old_start: 10,
        old_lines: 3,
        new_start: 10,
        new_lines: 3,
        lines,
    }
}

fn file(hunks: Vec<Hunk>) -> FileDiff {
    FileDiff {
        path: PathBuf::from("src/lib.rs"),
        status: gitpoulpi_core::domain::FileStatus::Modified,
        hunks,
    }
}

fn plain(lines: &[Line<'_>]) -> Vec<String> {
    lines
        .iter()
        .map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect())
        .collect()
}

fn sample() -> FileDiff {
    file(vec![hunk(vec![
        line(LineOrigin::Context, "fn main() {"),
        line(LineOrigin::Delete, "    let x = 1;"),
        line(LineOrigin::Add, "    let x = 2;"),
        line(LineOrigin::Context, "}"),
    ])])
}

fn wide_sample() -> FileDiff {
    file(vec![hunk(vec![
        line(LineOrigin::Context, "0123456789abcdefghijklmnopqrstuvwxyz"),
        line(
            LineOrigin::Delete,
            "0123456789abcdefghijklmnopqrstuvwxyz-old",
        ),
        line(LineOrigin::Add, "0123456789abcdefghijklmnopqrstuvwxyz-new"),
    ])])
}

fn two_hunk_file() -> FileDiff {
    let mut a = hunk(vec![
        line(LineOrigin::Context, "ctx a"),
        line(LineOrigin::Delete, "old a"),
        line(LineOrigin::Add, "new a"),
    ]);
    a.id = gitpoulpi_core::domain::HunkId(0);
    // Deliberately a different SHAPE from the first hunk: its changed
    // lines sit at indices 3 and 4, not 1 and 2. A selection that
    // wrongly reached across the hunk boundary would otherwise
    // produce the first hunk's indices by coincidence, and the test
    // for that confinement would pass against broken code.
    let mut b = hunk(vec![
        line(LineOrigin::Context, "ctx b1"),
        line(LineOrigin::Context, "ctx b2"),
        line(LineOrigin::Context, "ctx b3"),
        line(LineOrigin::Add, "new b1"),
        line(LineOrigin::Add, "new b2"),
    ]);
    b.id = gitpoulpi_core::domain::HunkId(1);
    file(vec![a, b])
}

mod bar;
mod cjk;
mod copy_select;
mod cursor;
mod h_scroll;
mod i18n;
mod layout;
mod lfs;
mod paint;
mod row_map;
mod ruler;
mod state;
