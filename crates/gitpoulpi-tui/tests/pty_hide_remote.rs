//! Hiding a whole remote: the owner's round trip with a local `archive`
//! remote whose history is unrelated to the repository's own. The remote's
//! row in the REMOTE section hides it with every branch it has now or gets
//! later (a pattern, not a list), the Log loses the archive commits, and
//! showing it again brings them back.
//!
//! The real TUI in a pty, keys only (the keyboard twins of the eye), real
//! `git`.

mod common;

use std::time::Duration;

use common::git_trimmed as git;
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Screen;

const STEP: Duration = Duration::from_secs(30);
const COLS: u16 = 200;
const ROWS: u16 = 50;
const SIDEBAR_RIGHT: u16 = 40;
/// `HIDDEN_IN_GRAPH_MARK` in `gitpoulpi_tui`: the gutter mark of a hidden ref.
const HIDDEN: &str = "\u{2298}";

fn origin() -> FixtureRepo {
    let mut b = FixtureRepoBuilder::new().unwrap();
    for n in 1..=3 {
        b = b.commit(&format!("origin work {n}")).unwrap();
    }
    b.build()
}

fn archive() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("archive commit 1")
        .unwrap()
        .commit("archive commit 2")
        .unwrap()
        .branch("old-a")
        .unwrap()
        .commit("archive commit 3")
        .unwrap()
        .checkout("main")
        .unwrap()
        .branch("old-b")
        .unwrap()
        .commit("archive commit 4")
        .unwrap()
        .checkout("main")
        .unwrap()
        .build()
}

/// The loaded commit count from the status bar ("main - 3 commits - clean").
fn loaded_count(screen: &Screen) -> Option<usize> {
    let text = screen.text();
    let end = text.find(" commits \u{2022}")?;
    let segment = text[..end].rsplit('\u{2022}').next()?;
    segment.split(" / ").next()?.trim().parse().ok()
}

/// Whether a sidebar row (left of the Log) carries both `label` and the
/// hidden mark.
fn marked(screen: &Screen, label: &str) -> bool {
    let marks: Vec<u16> = screen
        .find_all(HIDDEN)
        .into_iter()
        .filter(|(col, _)| *col < SIDEBAR_RIGHT)
        .map(|(_, row)| row)
        .collect();
    screen
        .find_all(label)
        .into_iter()
        .any(|(col, row)| col < SIDEBAR_RIGHT && marks.contains(&row))
}

#[test]
fn the_remote_row_hides_the_archive_for_good_and_the_palette_shows_it_again() {
    let repo = origin();
    let archive = archive();
    let root = repo.root();
    git(
        root,
        &[
            "remote",
            "add",
            "archive",
            &archive.root().to_string_lossy(),
        ],
    );
    git(root, &["fetch", "-q", "archive"]);
    let config = tempfile::tempdir().unwrap();
    let mut term = common::launch(root)
        .session("hide-remote")
        .config_dir(config.path())
        .size(COLS, ROWS)
        .ready_when("both histories in the Log", |s| {
            s.contains("origin work 3") && s.contains("archive commit 4")
        })
        .spawn();

    // Keyboard only: the REMOTE section, its menu on the remote's own row.
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text("Focus Refs > Remote").unwrap();
    term.wait_for("the entry", STEP, |s| {
        s.contains("Navigation: Focus Refs > Remote")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the palette closed", STEP, |s| {
        !s.contains("Command Palette")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text(".").unwrap();
    term.wait_for("the remote's menu", STEP, |s| {
        s.contains("Hide remote from graph")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // The row above "Remove archive...".
    term.send_keys("End").unwrap();
    term.send_keys("Up").unwrap();
    term.send_keys("Enter").unwrap();

    let screen = term
        .wait_for("the archive gone from the Log, its row marked", STEP, |s| {
            s.contains("origin work 3")
                && !s.contains("archive commit")
                && loaded_count(s) == Some(3)
                && marked(s, "archive")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        screen.contains("archived") || screen.contains("hidden from the graph"),
        "the status line says what happened: {}",
        screen.text()
    );

    // A branch fetched later is hidden too: its row is marked, its commit
    // never shows up.
    archive.git(&["checkout", "-q", "-b", "late"]);
    archive.git(&["commit", "-q", "--allow-empty", "-m", "archive commit late"]);
    git(root, &["fetch", "-q", "archive"]);
    term.wait_for("the late branch listed, hidden with the rest", STEP, |s| {
        marked(s, "late") && !s.contains("archive commit late") && loaded_count(s) == Some(3)
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // The palette shows the remote again, late branch included.
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text("Show Remote in Graph").unwrap();
    term.wait_for("the entry", STEP, |s| {
        s.contains("Remote: Show Remote in Graph...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    // One remote: the command acts on it without asking which.
    term.wait_for("every archive commit back, the late one too", STEP, |s| {
        s.contains("archive commit late")
            && s.contains("archive commit 1")
            && s.contains("archive commit 4")
            && loaded_count(s) == Some(8)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}
