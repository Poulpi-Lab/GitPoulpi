use super::intraline::{hunk_emphasis, CharRange};
use super::styled::{self, content_spans};
use super::*;
use gitpoulpi_core::domain::{
    DiffLine, FileDiff, FileStatus, Hunk, HunkId, LfsChange, LfsPointer, LineOrigin,
};
use std::path::PathBuf;

fn line(origin: LineOrigin, content: &str) -> DiffLine {
    DiffLine {
        origin,
        content: content.to_string(),
        staged: false,
    }
}

fn hunk(lines: Vec<DiffLine>) -> Hunk {
    Hunk {
        id: HunkId(3),
        old_start: 10,
        old_lines: 3,
        new_start: 20,
        new_lines: 3,
        lines,
    }
}

fn file(hunks: Vec<Hunk>) -> FileDiff {
    FileDiff {
        renamed_from: None,
        similarity: None,
        path: PathBuf::from("a.rs"),
        status: FileStatus::Modified,
        hunks,
    }
}

use LineOrigin::{Add, Context, Delete};

#[test]
fn line_numbers_run_per_side() {
    let h = hunk(vec![
        line(Context, "a"),
        line(Delete, "b"),
        line(Add, "c"),
        line(Context, "d"),
    ]);
    assert_eq!(
        line_numbers(&h),
        vec![
            (Some(10), Some(20)),
            (Some(11), None),
            (None, Some(21)),
            (Some(12), Some(22)),
        ]
    );
}

#[test]
fn whitespace_off_collapses_blank_changes_and_keeps_the_origin() {
    let f = file(vec![hunk(vec![
        line(Context, "a"),
        line(Delete, "x  "),
        line(Add, "x"),
        line(Delete, "y"),
        line(Add, "z"),
    ])]);
    let on = effective_hunks(&f, true);
    assert_eq!(on[0].origin, vec![0, 1, 2, 3, 4]);
    let off = effective_hunks(&f, false);
    // The whitespace-only pair becomes context, ahead of the real change.
    let kinds: Vec<_> = off[0].hunk.lines.iter().map(|l| l.origin.clone()).collect();
    assert_eq!(kinds, vec![Context, Context, Delete, Add]);
    assert_eq!(off[0].origin, vec![0, 2, 3, 4]);
}

#[test]
fn a_whitespace_only_hunk_disappears() {
    let f = file(vec![hunk(vec![line(Delete, "x "), line(Add, "x")])]);
    assert!(effective_hunks(&f, false).is_empty());
    assert_eq!(effective_hunks(&f, true).len(), 1);
}

#[test]
fn split_pairs_pair_runs_by_position() {
    let h = hunk(vec![
        line(Context, "a"),
        line(Delete, "b"),
        line(Delete, "c"),
        line(Add, "d"),
        line(Context, "e"),
    ]);
    let pairs = split_pairs(&h);
    assert_eq!(
        pairs,
        vec![
            (Some((0, 10)), Some((0, 20))),
            (Some((1, 11)), Some((3, 21))),
            (Some((2, 12)), None),
            (Some((4, 13)), Some((4, 22))),
        ]
    );
    assert_eq!(split_change(&h, pairs[1].0, pairs[1].1), RowChange::Both);
    assert_eq!(split_change(&h, pairs[2].0, pairs[2].1), RowChange::Del);
    assert_eq!(split_change(&h, pairs[0].0, pairs[0].1), RowChange::None);
}

#[test]
fn selected_lines_report_a_spill_into_another_hunk() {
    let rows = [
        DiffRow::header(HunkId(1)),
        DiffRow::body(HunkId(1), [Some(2), None]),
        DiffRow::header(HunkId(2)),
    ];
    assert_eq!(
        selected_lines(&rows, 0, 1),
        Some((HunkId(1), vec![2], false))
    );
    assert_eq!(selected_lines(&rows, 0, 2).map(|r| r.2), Some(true));
    assert_eq!(hunk_at(&rows, 2), Some(HunkId(2)));
}

#[test]
fn a_unified_line_is_gutter_marker_text() {
    let l = styled::unified_line(&Add, "let x = 2;", (None, Some(7)), &[], true);
    assert_eq!(l.plain(), "          7 +let x = 2;");
    let roles: Vec<_> = l.spans.iter().map(|s| s.role).collect();
    assert_eq!(roles, [DiffRole::Gutter, DiffRole::Add, DiffRole::Add]);
    assert!(l.spans.iter().all(|s| !s.emph && s.syntax.is_none()));
    // Line numbers off: a blank gutter of the same width.
    let l = styled::unified_line(&Delete, "x", (Some(1), None), &[], false);
    assert_eq!(l.spans[0].text, " ".repeat(12));
}

#[test]
fn intraline_emphasis_splits_the_changed_words() {
    let lines = vec![
        line(Delete, "let value = compute(a, b);"),
        line(Add, "let value = compute(a, c);"),
    ];
    let e = hunk_emphasis(&lines, |s| s.to_string());
    let l = styled::unified_line(&Add, &lines[1].content, (None, Some(1)), &e[1], true);
    let emph: Vec<&str> = l
        .spans
        .iter()
        .filter(|s| s.emph)
        .map(|s| s.text.as_str())
        .collect();
    assert_eq!(emph, ["c"]);
    assert_eq!(l.plain().trim_start(), "1 +let value = compute(a, c);");
    // Context lines never carry emphasis, whatever ranges they are given.
    let c = styled::unified_line(&Context, "abc", (Some(1), Some(1)), &[(0, 1)], true);
    assert!(c.spans.iter().all(|s| !s.emph));
}

#[test]
fn content_spans_cut_on_char_ranges() {
    let marked: [CharRange; 2] = [(1, 2), (4, 5)];
    let spans = content_spans("a\u{e9}cdef", &marked, DiffRole::Delete);
    let parts: Vec<(&str, bool)> = spans.iter().map(|s| (s.text.as_str(), s.emph)).collect();
    assert_eq!(
        parts,
        [
            ("a", false),
            ("\u{e9}", true),
            ("cd", false),
            ("e", true),
            ("f", false)
        ]
    );
}

#[test]
fn a_split_half_has_number_marker_text() {
    let l = styled::split_side_line(&Delete, "old", 4, &[(0, 1)], true);
    assert_eq!(l.plain(), "    4 -old");
    assert!(l.spans[2].emph);
    let l = styled::split_side_line(&Context, "same", 5, &[], false);
    assert_eq!(l.plain(), "       same");
}

#[test]
fn a_hunk_header_and_a_notice_have_their_own_roles() {
    let h = hunk(vec![line(Context, "a")]);
    let l = styled::hunk_header(&h);
    assert_eq!(l.plain(), "@@ -10,3 +20,3 @@");
    assert_eq!(l.spans[0].role, DiffRole::HunkHeader);
    let n = styled::notice("Nothing to show");
    assert_eq!(n, StyledLine::single("Nothing to show", DiffRole::Notice));
}

#[test]
fn an_lfs_card_is_a_title_then_text_then_muted_lines() {
    let ptr = |c: char, size| LfsPointer {
        oid: c.to_string().repeat(64),
        size,
    };
    let card = styled::lfs_card(
        &LfsChange {
            old: Some(ptr('a', 1024)),
            new: Some(ptr('b', 2048)),
        },
        Some(true),
    );
    assert_eq!(card[0].spans[0].role, DiffRole::Title);
    assert_eq!(card[1].spans[0].role, DiffRole::Text);
    assert!(card[1].plain().contains("->"));
    assert!(card[2].plain().starts_with("sha256 aaaaaaa"));
    assert!(card[2..].iter().all(|l| l.spans[0].role == DiffRole::Muted));
    let added = styled::lfs_card(
        &LfsChange {
            old: None,
            new: Some(ptr('b', 10)),
        },
        None,
    );
    assert!(added[2].plain().starts_with("sha256 bbbbbbb"));
    assert_eq!(added[2].spans[0].role, DiffRole::Muted);
}
