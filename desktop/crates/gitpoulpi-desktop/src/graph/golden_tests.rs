//! Golden test: the rows of a few hand-built histories (`fixtures.rs`) against `golden.txt`.
//!
//! The expected rows were first checked equal, cell for cell, to the private copy of the terminal
//! UI's lane algorithm that this crate carried before it used `gitpoulpi-app`. A diff here means the
//! drawn graph changed. After an intended change, regenerate with
//! `UPDATE_GOLDEN=1 cargo test -p gitpoulpi-desktop --lib golden` and review the diff.

use std::fmt::Write as _;

use gitpoulpi_app::graph::Sides;

use super::fixtures::{all, Fixture};
use super::{Entry, GraphInput, GraphLayout};

const GOLDEN: &str = include_str!("golden.txt");

fn sides(s: Sides) -> String {
    let mut out = String::new();
    for (on, c) in [(s.up, 'u'), (s.down, 'd'), (s.left, 'l'), (s.right, 'r')] {
        if on {
            out.push(c);
        }
    }
    if out.is_empty() {
        out.push('-');
    }
    out
}

fn render(f: &Fixture, palette_len: usize) -> String {
    let refs: Vec<_> = f
        .ref_targets
        .iter()
        .map(|t| gitpoulpi_core::domain::Ref {
            kind: gitpoulpi_core::domain::RefKind::LocalBranch { name: "x".into() },
            target: *t,
            visibility: Default::default(),
            is_head: false,
            upstream: None,
            tag_sync: None,
            multi_remote: None,
        })
        .collect();
    let layout = GraphLayout::build(&GraphInput {
        nodes: &f.nodes,
        refs: &refs,
        stashes: &f.stashes,
        dirty: f.dirty,
        head: f.head,
        palette_len,
    });
    let mut out = String::new();
    writeln!(out, "## {} (palette {palette_len})", f.name).unwrap();
    for entry in layout.entries() {
        let label = match entry {
            Entry::Wip => "wip".to_string(),
            Entry::Commit(i) => format!("c{i}"),
            Entry::Stash(i) => format!("s{i}"),
        };
        let row = &layout.graphic(*entry).expect("row").0;
        write!(out, "{label} lane={} color={:?} |", row.lane, row.color).unwrap();
        for cell in &row.cells {
            write!(
                out,
                " {:?}/{:?}/{}",
                cell.part,
                cell.color,
                sides(cell.dashed)
            )
            .unwrap();
        }
        out.push('\n');
    }
    out
}

fn render_all() -> String {
    let mut out = String::new();
    for f in all() {
        for palette_len in [1, 3, 8] {
            out.push_str(&render(&f, palette_len));
        }
    }
    out
}

#[test]
fn rows_match_the_stored_golden() {
    let now = render_all();
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/graph/golden.txt");
        std::fs::write(path, &now).unwrap();
        return;
    }
    assert_eq!(
        now, GOLDEN,
        "the drawn graph changed; see the module docs to regenerate"
    );
}

#[test]
fn fixtures_exercise_the_interesting_rows() {
    let text = render_all();
    for needle in ["wip ", "s0 ", "MergeIn", "BranchOut"] {
        assert!(text.contains(needle), "no `{needle}` in the fixtures");
    }
}
