//! End-to-end checks of the Repository Management tab (the `⌂`) and the
//! repository tab menu, driven through a pty by `xtask::pty`: the real
//! binary, the real daemon, real `git`.
//!
//! The rules are `pty_ui.rs`'s (see its header and
//! `docs/developer/visual-verification.md`): nothing is located by a
//! hard-coded coordinate, nothing sleeps, and every wait describes the
//! whole state it is waiting for.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const COLS: u16 = 180;
const ROWS: u16 = 45;

/// Launches the TUI in `repo` with a fresh config dir, seeded with a
/// recent list naming `recent` - the way a machine that has used
/// GitPoulpi before looks - and waits for `wait_for`.
fn launch(
    repo: &Path,
    session: &str,
    recent: &[(&Path, &str)],
    wait_for: &str,
) -> (Terminal, tempfile::TempDir) {
    let config = tempfile::tempdir().expect("temp config dir");
    let mut toml = String::new();
    for (path, name) in recent {
        toml.push_str(&format!(
            "[[repos]]\npath = '{}'\nname = '{name}'\nlast_opened = 1\n\n",
            path.display()
        ));
    }
    std::fs::write(config.path().join("recent.toml"), toml).expect("seed recent.toml");
    let term = common::launch(repo)
        .session(session)
        .config_dir(config.path())
        .size(COLS, ROWS)
        .spawn();
    let needle = wait_for.to_string();
    term.wait_for("the first window", STARTUP, move |s| s.contains(&needle))
        .unwrap_or_else(|e| panic!("{e}"));
    (term, config)
}

/// `Ctrl+O` goes to the management tab, which lists the open repository
/// and the recent one; typing in its search and pressing Enter opens the
/// match as a tab. Then the `⌂` (a click) brings the tab back with both
/// repositories under *Open repositories*, and `f` stars the selection
/// into `favorites.toml`.
#[test]
fn the_management_tab_opens_a_recent_repository_and_stars_one() {
    let first = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "alpha-only-commit")
        .unwrap()
        .build();
    let second = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("b.txt", "b\n", "bravo-only-commit")
        .unwrap()
        .build();
    let (mut term, config) = launch(
        first.root(),
        "pty-repo-management",
        &[(second.root(), "bravo-repo")],
        "alpha-only-commit",
    );

    term.send_keys("C-o").expect("go to the management tab");
    term.wait_for("the management tab, listing both", STEP, |s| {
        s.contains("Repository Management")
            && s.contains("OPEN REPOSITORIES")
            && s.contains("RECENT REPOSITORIES")
            && s.contains("bravo-repo")
            && !s.contains("alpha-only-commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("/").expect("focus the search");
    term.send_text("bravo").expect("type a filter");
    term.wait_for("the filter applied", STEP, |s| {
        s.contains("bravo") && s.contains("Enter open best match")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").expect("open the best match");
    term.wait_for("the second repository open as a tab", STARTUP, |s| {
        s.contains("bravo-only-commit") && !s.contains("Repository Management")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Back by mouse: the `⌂` at the far left of the tab bar.
    let (col, row) = term.wait_for_text("\u{2302}", STEP).unwrap();
    term.click(col, row).expect("click the management tab");
    let screen = term
        .wait_for("the management tab, both repositories open", STEP, |s| {
            s.contains("Repository Management") && s.contains("OPEN REPOSITORIES  2")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!screen.contains("bravo-only-commit"), "nothing behind it");

    // `f` on the selected row (the first open repository) stars it.
    term.send_keys("f").expect("star the selection");
    term.wait_for("the favorites section filled", STEP, |s| {
        s.contains("FAVORITES  1") && s.contains("to favorites")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let favorites = std::fs::read_to_string(config.path().join("favorites.toml"))
        .expect("favorites.toml written");
    assert!(favorites.contains("[[repos]]"), "{favorites}");
}

/// Right-click a repository tab: its menu, with the close group, the
/// favorite toggle and the path group; *Close other tabs...* asks once for
/// all of them, and confirming leaves only the clicked tab.
#[test]
fn right_clicking_a_tab_offers_its_menu_and_close_others_closes_the_rest() {
    let first = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "charlie-only-commit")
        .unwrap()
        .build();
    let second = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("b.txt", "b\n", "delta-only-commit")
        .unwrap()
        .build();
    let (mut term, _config) = launch(
        first.root(),
        "pty-tab-menu",
        &[(second.root(), "delta-repo")],
        "charlie-only-commit",
    );

    // Open the second one too, from the management tab.
    term.send_keys("C-o").unwrap();
    term.wait_for("the management tab", STEP, |s| s.contains("delta-repo"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("/").unwrap();
    term.send_text("delta").unwrap();
    term.wait_for("filtered", STEP, |s| s.contains("Enter open best match"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the second repository", STARTUP, |s| {
        s.contains("delta-only-commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // The tab bar's label for the second tab -- its folder name, which
    // is what opening it recorded. The fixtures' folders share a name,
    // so it is the *second* such label on the bar's row.
    // Trailing space, not " x": the close cross only shows on the active
    // and the hovered tab.
    let second_label = format!(" {} ", second.root().file_name().unwrap().to_string_lossy());
    let bar_row = term.wait_for_text("\u{2302}", STEP).unwrap().1;
    let labels = label_columns(&term.screen().line(bar_row), &second_label);
    assert_eq!(labels.len(), 2, "two tabs: {}", term.screen().line(bar_row));
    term.right_click(labels[1] + 2, bar_row)
        .expect("right-click the tab");
    let menu = term
        .wait_for("the tab menu", STEP, |s| {
            s.contains("Close tab")
                && s.contains("Close other tabs...")
                && s.contains("Add to favorites")
                && s.contains("Copy path")
                && s.contains("Open in ›")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Close other tabs...").unwrap();
    term.click(c + 1, r).expect("click Close other tabs...");
    // One other tab: the ordinary single-tab question.
    term.wait_for("the confirmation", STEP, |s| {
        s.contains("Close the tab for") && s.contains("not touched on disk")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("y").expect("confirm");
    let screen = term
        .wait_for("only the clicked tab left", STEP, |s| {
            s.contains("delta-only-commit") && !s.contains("Close the tab for")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let left = label_columns(&screen.line(bar_row), &second_label).len();
    assert_eq!(left, 1, "one tab left: {}", screen.line(bar_row));
}

/// Every column `needle` starts at in `line` (`Screen::find_all` reports
/// one per row). The tab bar holds only single-width glyphs, so a char
/// count is a cell count.
fn label_columns(line: &str, needle: &str) -> Vec<u16> {
    line.match_indices(needle)
        .map(|(byte, _)| line[..byte].chars().count() as u16)
        .collect()
}

/// `Ctrl+T` is the keyboard half of right-clicking a tab, and with three
/// tabs *Close other tabs...* asks once, naming how many.
#[test]
fn ctrl_t_opens_the_tab_menu_and_closing_several_asks_once() {
    let repos: Vec<_> = ["echo-one", "echo-two", "echo-three"]
        .iter()
        .map(|msg| {
            FixtureRepoBuilder::new()
                .unwrap()
                .commit_file("f.txt", "f\n", msg)
                .unwrap()
                .build()
        })
        .collect();
    let (mut term, _config) = launch(
        repos[0].root(),
        "pty-tab-menu-keys",
        &[(repos[1].root(), "echo-2"), (repos[2].root(), "echo-3")],
        "echo-one",
    );
    for (filter, commit) in [("echo-2", "echo-two"), ("echo-3", "echo-three")] {
        term.send_keys("C-o").unwrap();
        term.wait_for("the management tab", STEP, |s| {
            s.contains("Repository Management")
        })
        .unwrap_or_else(|e| panic!("{e}"));
        term.send_keys("/").unwrap();
        term.send_text(filter).unwrap();
        term.wait_for("filtered", STEP, |s| s.contains("Enter open best match"))
            .unwrap_or_else(|e| panic!("{e}"));
        term.send_keys("Enter").unwrap();
        term.wait_for("the repository", STARTUP, move |s| s.contains(commit))
            .unwrap_or_else(|e| panic!("{e}"));
    }

    term.send_keys("C-t").expect("open the tab menu");
    let menu = term
        .wait_for("the tab menu, from the keyboard", STEP, |s| {
            s.contains("Close other tabs...") && s.contains("Open in ›")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    // Walk to "Close other tabs..." with the arrows, run it with Enter.
    let (_, close_row) = menu.find("Close other tabs...").unwrap();
    // The menu opens on its first row, "New tab": one Down reaches "Close tab".
    let (_, first_row) = menu.find("Close tab").unwrap();
    term.send_keys("Down").unwrap();
    for _ in first_row..close_row {
        term.send_keys("Down").unwrap();
    }
    term.send_keys("Enter").unwrap();
    term.wait_for("one question for both", STEP, |s| {
        s.contains("Close 2 tabs") && s.contains("not touched on disk")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("y").unwrap();
    term.wait_for("closed both", STEP, |s| {
        s.contains("echo-three") && s.contains("closed 2 tabs")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// A stored recent list holding a folder that no longer exists and
/// one repository spelled twice (`<root>` and `<root>/.`) shows the gone
/// folder nowhere and the repository once per section; the stale entries
/// are migrated out of `recent.toml`; and `x` removes a row from the
/// list without touching the repository on disk.
#[test]
fn gone_and_duplicate_recent_entries_are_cleaned_and_x_removes_a_row() {
    let first = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "alpha-only-commit")
        .unwrap()
        .build();
    let second = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("b.txt", "b\n", "bravo-only-commit")
        .unwrap()
        .build();
    let spelled_again = second.root().join(".");
    let ghost = second.root().join("no-such-folder");
    let (mut term, config) = launch(
        first.root(),
        "pty-b9-clean-lists",
        &[
            (second.root(), "bravo-repo"),
            (&spelled_again, "bravo-repo"),
            (&ghost, "ghost-repo"),
        ],
        "alpha-only-commit",
    );

    term.send_keys("C-o").expect("go to the management tab");
    let screen = term
        .wait_for("the management tab with the cleaned lists", STEP, |s| {
            // Once in RECENT and once in ALL: the count is part of the
            // wait, the two lists are not drawn in the same instant.
            let text = (0..s.rows()).map(|r| s.line(r)).collect::<String>();
            s.contains("ALL REPOSITORIES") && text.matches("bravo-repo").count() == 2
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let screen: String = (0..screen.rows())
        .map(|r| screen.line(r))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!screen.contains("ghost-repo"), "{screen}");
    assert!(!screen.contains("gone from disk"), "{screen}");
    assert_eq!(
        screen.matches("bravo-repo").count(),
        2,
        "once in RECENT, once in ALL:\n{screen}"
    );
    let stored = std::fs::read_to_string(config.path().join("recent.toml")).unwrap();
    assert!(!stored.contains("ghost-repo"), "{stored}");
    assert_eq!(stored.matches("bravo-repo").count(), 1, "{stored}");

    // Filter to the repository, leave the field (Down keeps the query), `x`.
    term.send_keys("/").expect("focus the search");
    term.send_text("bravo").expect("type a filter");
    term.send_keys("Down")
        .expect("leave the search field, keeping the query");
    term.wait_for("the search field left", STEP, |s| {
        s.contains("bravo") && !s.contains("type filter")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("x").expect("remove the selected row");
    term.wait_for("the repository gone from the lists", STEP, |s| {
        s.contains("removed") && !s.contains("bravo-repo  ")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let stored = std::fs::read_to_string(config.path().join("recent.toml")).unwrap();
    assert!(!stored.contains("bravo-repo"), "{stored}");
    assert!(second.root().join(".git").exists(), "disk untouched");
}
