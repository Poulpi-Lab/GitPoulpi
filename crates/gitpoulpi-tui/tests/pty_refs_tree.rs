//! End-to-end checks of the Refs sidebar's branch tree against a
//! repository with **several remotes**, one of them named `grp/sub`,
//! driven through a pty by `xtask::pty` -- the real binary, the real
//! daemon, real `git`.
//!
//! Kept apart from `pty_ui.rs` because every test here needs the same
//! multi-remote fixture. The rules are that file's.
//!
//! The fixture's REMOTE section, as the tree draws it:
//!
//! ```text
//! ▾ 🌐 gh
//!       main            gh/main
//! ▾ 🌐 grp/sub
//!   ▾ 📂 feature/
//!         x             grp/sub/feature/x
//! ▾ 🌐 origin
//!   ▾ 📂 feature/
//!         login         origin/feature/login
//!         signup        origin/feature/signup
//!       main            origin/main   (tracked by the local `main`)
//! ```

mod common;

use common::git_trimmed as git;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Color, Screen, Terminal};

const STEP: Duration = Duration::from_secs(20);
const COLS: u16 = 200;
const ROWS: u16 = 50;

/// The Refs pane is 20% of 200 columns; everything this file looks for
/// lives left of this.
const SIDEBAR_RIGHT: u16 = 40;

/// `SELECTION_BG` in `gitpoulpi_tui`.
const SELECTION_BG: Color = Color::Rgb(52, 58, 90);

/// `HIDDEN_IN_GRAPH_MARK` in `gitpoulpi_tui` - what a hidden ref's gutter
/// shows. Not the eye (the eye is the *action*: hover, menu entry); this is
/// the *state*, the circled slash.
const HIDDEN: &str = "\u{2298}";

/// Two commits on `main`, three remotes (never fetched from - their
/// URLs point nowhere; the remote-tracking refs are written directly),
/// and `main` tracking `origin/main`. `extra` runs more git commands on
/// top, for the one test that needs a differently-named tracking branch.
fn fixture(extra: &[&[&str]]) -> gitpoulpi_testkit::FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial")
        .unwrap()
        .commit_file("b.txt", "two\n", "second")
        .unwrap()
        .build();
    let root = repo.root().to_path_buf();
    for args in [
        &["remote", "add", "origin", "../nowhere-origin"][..],
        &["remote", "add", "gh", "../nowhere-gh"],
        &["remote", "add", "grp/sub", "../nowhere-grp"],
        &["update-ref", "refs/remotes/origin/main", "HEAD"],
        &["update-ref", "refs/remotes/origin/feature/login", "HEAD~1"],
        &["update-ref", "refs/remotes/origin/feature/signup", "HEAD~1"],
        &["update-ref", "refs/remotes/gh/main", "HEAD"],
        &["update-ref", "refs/remotes/grp/sub/feature/x", "HEAD~1"],
        &["branch", "--set-upstream-to=origin/main", "main"],
    ] {
        git(&root, args);
    }
    for args in extra {
        git(&root, args);
    }
    repo
}

fn launch(repo: &Path, session: &str, config: &Path) -> Terminal {
    common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(COLS, ROWS)
        .ready_when("the fixture's log and its REMOTE tree", |s| {
            s.contains("second") && s.contains("initial") && s.contains("grp/sub")
        })
        .spawn()
}

/// The row of a section's header (`LOCAL (`, `REMOTE (`).
fn header_row(screen: &Screen, header: &str) -> Option<u16> {
    screen
        .find_all(header)
        .into_iter()
        .find(|(col, _)| *col < SIDEBAR_RIGHT)
        .map(|(_, row)| row)
}

/// Where `label` is drawn in the Refs sidebar, below the section header
/// `header` and below the first `under` node after it - `under` is how
/// `origin`'s `main` is told from `gh`'s. The first match wins; `None`
/// when it is not on screen.
fn sidebar_label(screen: &Screen, header: &str, under: &[&str], label: &str) -> Option<(u16, u16)> {
    let mut from = header_row(screen, header)?;
    let next_header = ["LOCAL (", "REMOTE (", "TAGS ("]
        .iter()
        .filter_map(|h| header_row(screen, h))
        .filter(|row| *row > from)
        .min()
        .unwrap_or(u16::MAX);
    let first_below = |needle: &str, from: u16| {
        screen
            .find_all(needle)
            .into_iter()
            .filter(|(col, row)| *col < SIDEBAR_RIGHT && *row > from && *row < next_header)
            .min_by_key(|(_, row)| *row)
    };
    for node in under {
        from = first_below(node, from)?.1;
    }
    first_below(label, from)
}

/// Waits for `label` to be drawn where `sidebar_label` looks, and
/// returns where. A wait rather than a one-off read: a snapshot can be
/// half of one repaint (`launch`'s predicate is satisfied by the first
/// half of a frame as readily as by a whole one).
fn expect_label(term: &Terminal, header: &str, under: &[&str], label: &str) -> (u16, u16) {
    let screen = term
        .wait_for(
            &format!("{label:?} under {header:?} {under:?}"),
            STEP,
            |s| sidebar_label(s, header, under, label).is_some(),
        )
        .unwrap_or_else(|e| panic!("{e}"));
    sidebar_label(&screen, header, under, label).expect("just waited for it")
}

/// Whether the LOCAL section shows `label` with the checked-out mark.
/// `CHECKED_OUT_MARK` in `gitpoulpi_tui` - `\u{2713}` (CHECK MARK), the
/// same glyph as the Log's ref pills.
fn local_checked_out(screen: &Screen, label: &str) -> bool {
    sidebar_label(screen, "LOCAL (", &[], label)
        .is_some_and(|(_, row)| screen.line(row).contains('\u{2713}'))
}

/// The user's own words: double-clicking a remote branch nobody tracks
/// offers to create the local branch, and yes creates it - tracking the
/// remote branch, checked out. The proposed name is the part after the
/// remote, not after the first `/`.
#[test]
fn double_clicking_a_remote_branch_nobody_tracks_offers_to_create_it_and_yes_checks_it_out() {
    let repo = fixture(&[]);
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "refs-tree-create", config.path());

    let (col, row) = expect_label(&term, "REMOTE (", &["origin"], "login");
    term.double_click(col + 1, row).unwrap();
    term.wait_for("the create-tracking-branch question", STEP, |s| {
        s.contains("No local branch tracks origin/feature/login.")
            && s.contains("Create feature/login tracking it")
            && s.contains("Create branch")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // `y` is the Refs pane's key: Tab twice goes Log -> Status -> Refs.
    term.send_keys("Tab Tab y").unwrap();
    term.wait_for("feature/login checked out in LOCAL", STEP, |s| {
        !s.contains("No local branch tracks") && local_checked_out(s, "login")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    let root = repo.root();
    assert_eq!(
        git(root, &["rev-parse", "--abbrev-ref", "HEAD"]),
        "feature/login"
    );
    assert_eq!(
        git(
            root,
            &["rev-parse", "--abbrev-ref", "feature/login@{upstream}"]
        ),
        "origin/feature/login"
    );
}

/// Cancelling creates nothing.
#[test]
fn canceling_the_create_question_leaves_the_repository_alone() {
    let repo = fixture(&[]);
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "refs-tree-cancel", config.path());

    let (col, row) = expect_label(&term, "REMOTE (", &["grp/sub"], "x");
    term.double_click(col + 1, row).unwrap();
    term.wait_for("the question for grp/sub's branch", STEP, |s| {
        s.contains("No local branch tracks grp/sub/feature/x.")
            // `feature/x`, not `sub/feature/x`: the remote's whole name
            // is stripped.
            && s.contains("Create feature/x tracking it")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Esc").unwrap();
    term.wait_for_text_to_go("No local branch tracks", STEP)
        .unwrap_or_else(|e| panic!("{e}"));

    let root = repo.root();
    assert_eq!(git(root, &["rev-parse", "--abbrev-ref", "HEAD"]), "main");
    assert_eq!(git(root, &["branch", "--list", "feature/x"]), "");
}

/// `Enter` is the keyboard's double-click, and "the mapped local branch"
/// is whichever one *tracks* the remote branch - here one called `work`,
/// nothing like the remote branch's name. No question is asked.
#[test]
fn enter_on_a_remote_branch_checks_out_the_local_branch_that_tracks_it() {
    let repo = fixture(&[
        &["branch", "work", "HEAD~1"],
        &["branch", "--set-upstream-to=origin/feature/login", "work"],
    ]);
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "refs-tree-enter", config.path());

    let (col, row) = expect_label(&term, "REMOTE (", &["origin"], "login");
    // A single click selects the row (and focuses the sidebar); Enter
    // does the rest.
    term.click(col + 1, row).unwrap();
    term.wait_for("the login row selected", STEP, |s| {
        s.cell(col, row).is_some_and(|c| c.bg == SELECTION_BG)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("work checked out", STEP, |s| local_checked_out(s, "work"))
        .unwrap_or_else(|e| panic!("{e}"));

    assert!(!term.screen().contains("No local branch tracks"));
    assert_eq!(
        git(repo.root(), &["rev-parse", "--abbrev-ref", "HEAD"]),
        "work"
    );
}

/// `gh/main`'s obvious local name is taken by a `main` that tracks
/// `origin/main`. Neither checking that `main` out nor inventing another
/// name is what was asked: it says so, and changes nothing.
#[test]
fn a_remote_branch_whose_local_name_is_taken_says_so_and_changes_nothing() {
    let repo = fixture(&[&["checkout", "-q", "-b", "elsewhere"]]);
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "refs-tree-taken", config.path());

    let (col, row) = expect_label(&term, "REMOTE (", &["gh"], "main");
    term.double_click(col + 1, row).unwrap();
    term.wait_for("the name-taken explanation", STEP, |s| {
        s.contains("main already exists and tracks origin/main, not gh/main")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    assert!(!term.screen().contains("No local branch tracks"));
    assert_eq!(
        git(repo.root(), &["rev-parse", "--abbrev-ref", "HEAD"]),
        "elsewhere"
    );
}

/// Folding a folder is remembered per repository, across a restart.
#[test]
fn a_folded_remote_folder_stays_folded_after_a_restart() {
    let repo = fixture(&[]);
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "refs-tree-fold-1", config.path());

    let (col, row) = expect_label(&term, "REMOTE (", &["origin"], "feature/");
    term.click(col + 1, row).unwrap();
    term.wait_for("origin/feature folded", STEP, |s| {
        sidebar_label(s, "REMOTE (", &["origin"], "login").is_none()
            // The captured screen (a real terminal, not ratatui's buffer)
            // shows a wide glyph's continuation cell as a blank space of
            // its own, so a 2-column icon reads as two spaces before the
            // text that follows it -- one from that continuation cell,
            // one from the literal space `ref_folder_label` writes.
            && sidebar_label(s, "REMOTE (", &["origin"], "\u{25B8} \u{1F4C1}  feature/").is_some()
            // grp/sub's own `feature/` is a different folder.
            && sidebar_label(s, "REMOTE (", &["grp/sub"], "x").is_some()
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();

    let state = std::fs::read_to_string(repo.state_dir().join("repo-state.toml"))
        .expect("repo-state.toml written");
    assert!(state.contains("origin/feature"), "{state}");

    let term = launch(repo.root(), "refs-tree-fold-2", config.path());
    term.wait_for("origin/feature still folded after the restart", STEP, |s| {
        sidebar_label(s, "REMOTE (", &["origin"], "\u{25B8} \u{1F4C1}  feature/").is_some()
            && sidebar_label(s, "REMOTE (", &["origin"], "login").is_none()
            && sidebar_label(s, "REMOTE (", &["origin"], "main").is_some()
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// `y` (Refs pane) is the Preferences checkbox's shortcut, not a session-only
/// flag: it flattens the tree *and* writes `ui.ref_tree = false`, so the
/// next launch comes up flat too.
#[test]
fn y_flattens_the_tree_and_the_choice_survives_a_restart() {
    let repo = fixture(&[]);
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "refs-tree-flat-1", config.path());

    // `y` is the Refs pane's key: Tab twice goes Log -> Status -> Refs.
    term.send_keys("Tab Tab y").unwrap();
    term.wait_for("the flat REMOTE list", STEP, |s| {
        sidebar_label(s, "REMOTE (", &[], "origin/feature/login").is_some()
            && sidebar_label(s, "REMOTE (", &[], "grp/sub/feature/x").is_some()
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();

    let written =
        std::fs::read_to_string(config.path().join("config.toml")).expect("config.toml written");
    assert!(written.contains("ref_tree = false"), "{written}");

    let term = launch(repo.root(), "refs-tree-flat-2", config.path());
    expect_label(&term, "REMOTE (", &[], "origin/feature/login");
}

/// Clicking the gutter hides the ref, and the gutter then shows the hidden
/// *state* mark (the circled slash), not the eye - on the selected row
/// too, and never bold there. The name on the selected row is still bold.
#[test]
fn a_hidden_ref_shows_the_circled_slash_never_bold_even_when_selected() {
    let repo = fixture(&[]);
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "refs-tree-eye", config.path());

    // `signup`: no pill of its own on the selected commit, nothing else
    // on its row.
    let (col, row) = expect_label(&term, "REMOTE (", &["origin"], "signup");
    // The gutter: checkmark, space, eye, space, name -- the eye is two
    // cells left of the name.
    let eye_col = col - 2;
    term.click(eye_col, row).unwrap();
    term.wait_for("signup hidden, with the circled slash", STEP, |s| {
        s.cell(eye_col, row).is_some_and(|c| c.text == HIDDEN)
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Select the row by its name; move the pointer away so the hover
    // tint cannot stand in for the selection.
    term.click(col + 1, row).unwrap();
    term.move_to(COLS - 5, ROWS - 5).unwrap();
    let screen = term
        .wait_for("signup selected, hidden mark still there", STEP, |s| {
            s.cell(col, row)
                .is_some_and(|c| c.bg == SELECTION_BG && c.bold)
                && s.cell(eye_col, row).is_some_and(|c| c.text == HIDDEN)
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let eye = screen.cell(eye_col, row).unwrap();
    assert!(
        !eye.bold,
        "the hidden mark on the selected row is bold: {eye:?}"
    );
    assert_eq!(
        eye.bg, SELECTION_BG,
        "the selection band still covers the gutter"
    );
}

/// The hide mark: U+2299, a text symbol (it replaced the emoji-capable eye).
const EYE: &str = "\u{2299}";

/// Dragging a branch in the Refs pane must not turn the hover eye into its
/// color emoji: no eye cell is ever bold, whatever the drag paints over the
/// rows.
#[test]
fn the_hide_eye_is_never_bold_while_dragging_a_ref() {
    let repo = fixture(&[]);
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "refs-tree-eye-drag", config.path());
    let (col, row) = expect_label(&term, "REMOTE (", &["origin"], "signup");
    // Hover first, as a real pointer does on its way to the press.
    term.move_to(col + 1, row).unwrap();
    term.wait_for("the hover eye", STEP, |s| {
        s.cell(col - 2, row).is_some_and(|c| c.text == EYE)
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.press(col + 1, row).unwrap();
    term.drag_to(col + 2, row).unwrap();
    term.drag_to(col + 3, row).unwrap();
    let eye_cells = |s: &xtask::pty::Screen| {
        (0..ROWS)
            .flat_map(|r| (0..COLS).map(move |c| (c, r)))
            .filter_map(|(c, r)| s.cell(c, r).filter(|cell| cell.text == EYE).cloned())
            .collect::<Vec<_>>()
    };
    let screen = term
        .wait_for("an eye drawn during the drag", STEP, |s| {
            !eye_cells(s).is_empty()
        })
        .unwrap_or_else(|e| panic!("{e}"));
    for cell in eye_cells(&screen) {
        assert!(!cell.bold, "a bold eye during a drag: {cell:?}");
    }
    term.release(col + 3, row).unwrap();
}
