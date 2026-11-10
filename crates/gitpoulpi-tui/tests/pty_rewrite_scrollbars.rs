//! The Secrets tab of the history rewrite workbench with many findings,
//! through a pty (`xtask::pty`): the list has a scrollbar, the thumb can be
//! dragged, the wheel scrolls the list under the pointer, and the keys keep
//! the selected finding in view.
//!
//! The rules are `pty_ui.rs`'s: nothing sleeps, every wait describes the
//! state it waits for. The bar is found by its end arrows on the list's own
//! rows, never by a hard-coded coordinate.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const FILES: usize = 40;

/// A fake token of the shape of a classic GitHub one, different for each `i`.
fn token(i: usize) -> String {
    const SET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let body: String = (0..36)
        .map(|k| SET[(i * 7 + k * 13 + k * k + i * k) % SET.len()] as char)
        .collect();
    format!("ghp_{body}")
}

fn fixture() -> FixtureRepo {
    let mut builder = FixtureRepoBuilder::new().unwrap();
    for i in 0..FILES {
        builder = builder
            .commit_file(
                &format!("secrets/cfg{i:02}.env"),
                &format!("GITHUB_TOKEN={}\n", token(i)),
                &format!("add cfg{i:02}"),
            )
            .unwrap();
    }
    builder.build()
}

/// The rows of the findings list: the ones that name a GitHub token.
fn list_rows(s: &Screen) -> Vec<u16> {
    (0..s.rows())
        .filter(|&r| {
            s.line(r).contains("[x] GitHub token") || s.line(r).contains("[ ] GitHub token")
        })
        .collect()
}

/// The column of `glyph` on `row`, if the row has it.
fn glyph_col(s: &Screen, row: u16, glyph: &str) -> Option<u16> {
    s.find_all(glyph)
        .into_iter()
        .find(|&(_, r)| r == row)
        .map(|(c, _)| c)
}

/// The list's own rows with the bar's arrows at both ends:
/// `(first row, last row, bar column)`.
fn bar_ends(s: &Screen) -> Option<(u16, u16, u16)> {
    let rows = list_rows(s);
    let (&first, &last) = (rows.first()?, rows.last()?);
    let col = glyph_col(s, first, "\u{2191}")?;
    (glyph_col(s, last, "\u{2193}") == Some(col)).then_some((first, last, col))
}

/// What the list shows: each row up to the bar.
fn shown(s: &Screen) -> Vec<String> {
    let (_, _, col) = bar_ends(s).expect("the bar");
    list_rows(s)
        .into_iter()
        .map(|r| {
            (0..col)
                .filter_map(|c| s.cell(c, r).map(|cell| cell.display().to_string()))
                .collect::<String>()
        })
        .collect()
}

/// The path of the finding the cursor is on: the detail line right under
/// the list.
fn detail_path(s: &Screen) -> String {
    let (_, last, col) = bar_ends(s).expect("the list");
    let line: String = (0..col)
        .filter_map(|c| s.cell(c, last + 1).map(|cell| cell.display().to_string()))
        .collect();
    line.split_whitespace()
        .find(|w| w.contains("cfg"))
        .unwrap_or("")
        .to_string()
}

fn open_secrets_tab(term: &mut Terminal) -> Screen {
    term.wait_for("the fixture loaded", STARTUP, |s| s.contains("add cfg39"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-g").unwrap();
    term.send_text("H").unwrap();
    term.wait_for("the workbench", STEP, |s| s.contains("Add a rule to start"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Tab").unwrap();
    for _ in 0..4 {
        term.send_keys("Right").unwrap();
    }
    term.wait_for("the Secrets tab", STEP, |s| s.contains("Scan for secrets"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("s").unwrap();
    // The bar and the detail line under the list: a torn frame can show the
    // first before the second.
    term.wait_for("the findings, their bar and the detail line", STEP, |s| {
        bar_ends(s).is_some() && !detail_path(s).is_empty()
    })
    .unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn the_secrets_list_has_a_bar_the_thumb_drags_the_wheel_scrolls_and_the_keys_follow() {
    let repo = fixture();
    let config = tempfile::tempdir().expect("temp config dir");
    let mut term = common::launch(repo.root())
        .session("pty-rewrite-scrollbars")
        .config_dir(config.path())
        .size(160, 50)
        .spawn();
    let start = open_secrets_tab(&mut term);
    let (first, last, col) = bar_ends(&start).expect("the bar");
    let before = shown(&start);
    assert!(before.len() < FILES, "the list overflows: {}", start.text());

    // Drag the thumb (on its top, right under the up arrow) to the bottom:
    // the list moves, the selection does not.
    // Retried: a drag that starts on a torn frame can miss the thumb.
    let dragged = common::retry_until(
        &mut term,
        "the list scrolled by the thumb",
        4,
        Duration::from_secs(10),
        |t| t.drag((col, first + 1), (col, last + 3)).unwrap(),
        |s| bar_ends(s).is_some() && !detail_path(s).is_empty() && shown(s) != before,
    );
    assert_eq!(
        detail_path(&dragged),
        detail_path(&start),
        "dragging the bar leaves the selection alone"
    );
    assert!(
        shown(&dragged).iter().any(|l| l.contains("cfg39")),
        "dragged to the end\n{}",
        dragged.text()
    );

    // The wheel over the list scrolls it back up.
    for _ in 0..30 {
        term.scroll(false, col - 10, first + 1).unwrap();
    }
    let wheeled = term
        .wait_for("the list scrolled back by the wheel", STEP, |s| {
            bar_ends(s).is_some() && shown(s) == before && detail_path(s) == detail_path(&start)
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(detail_path(&wheeled), detail_path(&start));

    // Keys: the selection moves down past the window; the list follows it.
    for _ in 0..8 {
        term.send_keys("PageDown").unwrap();
    }
    let keyed = term
        .wait_for("the selected finding in view", STEP, |s| {
            let Some(_) = bar_ends(s) else { return false };
            let path = detail_path(s);
            path.contains("cfg3") && shown(s).iter().any(|l| l.contains(&path))
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert_ne!(shown(&keyed), before, "the window followed the selection");
}
