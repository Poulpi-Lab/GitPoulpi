//! All seven context menus (Status file, Status folder, commit file, diff
//! hunk, ref (sidebar), Log commit and WIP) are opened by the mouse, closed
//! by `Esc`, closed by a click away, and closed by the `⋯` overflow menu.
//! They also open from the keyboard (`.`) where the focused pane has a menu.
//!
//! Same rules as `pty_ui.rs`. These tests pin what the code does today; the
//! one known oddity (the overflow menu leaving some menus open) is asserted
//! as it is, with a comment.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const OVERFLOW: &str = "\u{22EF}";

/// Which menu; `marker` is a row only that menu has.
#[derive(Clone, Copy, Debug)]
enum Kind {
    File,
    Folder,
    CommitFile,
    DiffHunk,
    Ref,
    Log,
    Wip,
}

impl Kind {
    const ALL: [Kind; 7] = [
        Kind::File,
        Kind::Folder,
        Kind::CommitFile,
        Kind::DiffHunk,
        Kind::Ref,
        Kind::Log,
        Kind::Wip,
    ];

    fn marker(self) -> &'static str {
        match self {
            Kind::File => "Stash file",
            Kind::Folder => "Stash folder",
            Kind::CommitFile => "Open Diff",
            Kind::DiffHunk => "Copy hunk",
            Kind::Ref => "Check out feature",
            Kind::Log => "Create branch here",
            Kind::Wip => "Stash all changes",
        }
    }
}

fn fixture() -> FixtureRepo {
    let many: String = (1..=30).map(|n| format!("l{n}\n")).collect();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("src/a.txt", "one\n", "add a")
        .unwrap()
        .commit_file("src/b.txt", "two\n", "add b")
        .unwrap()
        .commit_file("c.txt", &many, "add c")
        .unwrap()
        .write_worktree("src/a.txt", b"one changed\n")
        .unwrap()
        .write_worktree("src/b.txt", b"two changed\n")
        .unwrap()
        .write_worktree("c.txt", many.replace("l15", "CHANGED-MIDDLE").as_bytes())
        .unwrap()
        .build();
    repo.git(&["branch", "feature", "HEAD~1"]);
    repo
}

fn launch(repo: &FixtureRepo, session: &str, cols: u16) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        // Pinned to the top toolbar (the old layout): a context menu opened
        // low in the window covers a bottom toolbar, and this file's
        // overflow test clicks the `⋯` while the menu is open.
        .config("[ui]\ntoolbar_position = \"top\"\n")
        .size(cols, 50)
        .ready_when("the fixture's log", |s| {
            s.contains("add c") && s.contains("add a")
        })
        .mouse_capture()
        .start()
}

/// Selects the WIP row (the click lands on its `◌`, not its text field).
fn select_wip(term: &mut Terminal) {
    let (col, row) = term.wait_for_text("\u{25CC}", STEP).unwrap();
    term.click(col, row).unwrap();
    term.wait_for("the working changes", STEP, |s| {
        s.contains("Unstaged Files") && s.contains("\u{270E} c.txt") && s.contains("src/")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

fn wait_menu(term: &Terminal, kind: Kind) {
    term.wait_for(&format!("the {kind:?} menu"), STEP, |s| {
        s.contains(kind.marker())
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

fn wait_closed(term: &Terminal, kind: Kind) {
    term.wait_for(&format!("the {kind:?} menu to close"), STEP, |s| {
        !s.contains(kind.marker())
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// Right-clicks where `find` locates the target until `kind`'s menu is
/// drawn. A right-click sent right after its row appears can be dropped (the
/// row is drawn before its hit target is registered, and a refresh in
/// between redraws the pane), so the target is re-located and the click
/// repeated until the result shows.
fn right_click_until_menu(
    term: &mut Terminal,
    kind: Kind,
    find: impl Fn(&Screen) -> Option<(u16, u16)>,
) {
    common::retry_until(
        term,
        &format!("the {kind:?} menu"),
        8,
        Duration::from_secs(4),
        |t| {
            if let Some((c, r)) = find(&t.screen()) {
                t.right_click(c, r).unwrap();
            }
        },
        |s| s.contains(kind.marker()),
    );
}

fn first(s: &Screen, needle: &str, dx: u16) -> Option<(u16, u16)> {
    s.find(needle).map(|(c, r)| (c + dx, r))
}

/// Brings the screen to where `kind`'s menu can be opened, then opens it
/// with the mouse. Returns with the menu drawn.
fn open_by_mouse(term: &mut Terminal, kind: Kind) {
    match kind {
        Kind::File => {
            select_wip(term);
            right_click_until_menu(term, kind, |s| first(s, "\u{270E} c.txt", 1));
        }
        Kind::Folder => {
            select_wip(term);
            right_click_until_menu(term, kind, |s| first(s, "\u{25BE} src/", 3));
        }
        Kind::Wip => {
            term.wait_for_text("\u{25CC}", STEP).unwrap();
            right_click_until_menu(term, kind, |s| first(s, "\u{25CC}", 0));
        }
        Kind::Log => {
            term.wait_for_text("add b", STEP).unwrap();
            right_click_until_menu(term, kind, |s| first(s, "add b", 1));
        }
        Kind::CommitFile => {
            let (c, r) = term.wait_for_text("add c", STEP).unwrap();
            term.click(c + 1, r).unwrap();
            term.wait_for("the commit's file list", STEP, |s| {
                s.contains("c.txt") && !s.contains("Unstaged Files")
            })
            .unwrap_or_else(|e| panic!("{e}"));
            right_click_until_menu(term, kind, |s| {
                s.find_all("c.txt")
                    .into_iter()
                    .min_by_key(|(c, _)| *c)
                    .map(|(c, r)| (c + 1, r))
            });
        }
        Kind::Ref => {
            term.wait_for("the sidebar's branch", STEP, |s| s.contains("feature"))
                .unwrap_or_else(|e| panic!("{e}"));
            right_click_until_menu(term, kind, |s| {
                s.find_all("feature")
                    .into_iter()
                    .min_by_key(|(c, _)| *c)
                    .map(|(c, r)| (c + 1, r))
            });
        }
        Kind::DiffHunk => {
            select_wip(term);
            let (c, r) = term.wait_for_text("\u{270E} c.txt", STEP).unwrap();
            term.click(c + 1, r).unwrap();
            term.wait_for_text("CHANGED-MIDDLE", STEP).unwrap();
            right_click_until_menu(term, kind, |s| first(s, "CHANGED-MIDDLE", 1));
        }
    }
    wait_menu(term, kind);
}

/// Every menu opens on a right-click and `Esc` closes it (and only it:
/// the Log is still there afterwards).
#[test]
fn every_context_menu_opens_by_mouse_and_closes_on_escape() {
    let repo = fixture();
    for kind in Kind::ALL {
        let (mut term, _c) = launch(&repo, &format!("pty-menus-esc-{kind:?}"), 200);
        open_by_mouse(&mut term, kind);
        term.send_keys("Esc").unwrap();
        wait_closed(&term, kind);
        // The diff view covers the Log, so its survivor is the diff body.
        let survivor = if matches!(kind, Kind::DiffHunk) {
            "CHANGED-MIDDLE"
        } else {
            "add c"
        };
        assert!(
            term.screen().contains(survivor),
            "{kind:?}: Esc closed more than the menu:\n{}",
            term.text()
        );
        term.stop().unwrap();
    }
}

/// A left click anywhere outside the menu closes it. The click is on the
/// toolbar's blank right-hand end, away from every menu and every row.
#[test]
fn every_context_menu_closes_on_a_click_away() {
    let repo = fixture();
    for kind in Kind::ALL {
        let (mut term, _c) = launch(&repo, &format!("pty-menus-away-{kind:?}"), 200);
        open_by_mouse(&mut term, kind);
        // The title-bar row, last column: nothing lives there that a
        // click could run.
        let screen = term.screen();
        term.click(screen.cols() - 1, 0).unwrap();
        wait_closed(&term, kind);
        term.stop().unwrap();
    }
}

/// A second right-click on another target while a menu is open replaces
/// it: opening any menu closes the others, so only the WIP menu is drawn.
#[test]
fn a_second_right_click_replaces_the_open_menu() {
    let repo = fixture();
    let (mut term, _c) = launch(&repo, "pty-menus-replace", 200);
    open_by_mouse(&mut term, Kind::Log);
    term.wait_for_text("\u{25CC}", STEP).unwrap();
    right_click_until_menu(&mut term, Kind::Wip, |s| first(s, "\u{25CC}", 0));
    wait_menu(&term, Kind::Wip);
    wait_closed(&term, Kind::Log);
    term.stop().unwrap();
}

/// The menus that open from the keyboard (`.`, the Context Menu binding,
/// on the focused pane's selection) close on `Esc` too.
#[test]
fn the_dot_key_opens_the_log_and_wip_menus_and_escape_closes_them() {
    let repo = fixture();
    let (mut term, _c) = launch(&repo, "pty-menus-dot", 200);
    // The Log has focus at start with a commit selected.
    let (c, r) = term.wait_for_text("add b", STEP).unwrap();
    term.click(c + 1, r).unwrap();
    term.send_keys(".").unwrap();
    wait_menu(&term, Kind::Log);
    term.send_keys("Esc").unwrap();
    wait_closed(&term, Kind::Log);

    let (c, r) = term.wait_for_text("\u{25CC}", STEP).unwrap();
    term.click(c, r).unwrap();
    term.send_keys(".").unwrap();
    wait_menu(&term, Kind::Wip);
    term.send_keys("Esc").unwrap();
    wait_closed(&term, Kind::Wip);
    term.stop().unwrap();
}

/// Arrow keys move the highlight and Enter runs the row: the Log menu's
/// first row is a harmless prompt-opener.
#[test]
fn a_menu_row_is_chosen_with_the_arrows_and_enter() {
    let repo = fixture();
    let (mut term, _c) = launch(&repo, "pty-menus-arrows", 200);
    open_by_mouse(&mut term, Kind::Log);
    // Walk down then back up: whatever row is first is chosen again.
    term.send_keys("Down").unwrap();
    term.send_keys("Up").unwrap();
    let before = term.screen();
    term.send_keys("Enter").unwrap();
    term.wait_for("the menu to run its row and close", STEP, |s| {
        !s.contains(Kind::Log.marker())
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", before.text()));
    term.stop().unwrap();
}

/// The `⋯` overflow menu with another menu open: on today's code it
/// closes every context menu (checked per kind, at 90 columns where the
/// toolbar overflows yet every pane still shows); a single `OpenMenu`
/// holds this by construction.
#[test]
fn the_overflow_menu_versus_each_open_context_menu() {
    let repo = fixture();
    let mut still_open = Vec::new();
    for kind in Kind::ALL {
        let (mut term, _c) = launch(&repo, &format!("pty-menus-overflow-{kind:?}"), 90);
        open_by_mouse(&mut term, kind);
        let (c, r) = term.wait_for_text(OVERFLOW, STEP).unwrap();
        term.click(c, r).unwrap();
        // Whatever happens, the overflow menu itself must open.
        term.wait_for("the overflow menu", STEP, |s| s.contains("\u{2315} Search"))
            .unwrap_or_else(|e| panic!("{kind:?}: {e}\n{}", term.text()));
        // A frame read mid-write can still show the old menu's cells next
        // to the new popup, so "still open" means it never goes away.
        if term
            .wait_for("the context menu to close", Duration::from_secs(3), |s| {
                s.contains("\u{2315} Search") && !s.contains(kind.marker())
            })
            .is_err()
        {
            still_open.push(format!("{kind:?}"));
        }
        term.stop().unwrap();
    }
    // Observed on today's code (see the assertion message when it moves).
    assert_eq!(
        still_open,
        Vec::<String>::new(),
        "the overflow menu left these context menus open"
    );
}

/// Menus do not stack (see `a_second_right_click_replaces_the_open_menu`):
/// one click away closes the single open menu.
#[test]
fn one_click_away_closes_the_replacing_menu() {
    let repo = fixture();
    let (mut term, _c) = launch(&repo, "pty-menus-chain", 200);
    open_by_mouse(&mut term, Kind::Log);
    term.wait_for_text("\u{25CC}", STEP).unwrap();
    right_click_until_menu(&mut term, Kind::Wip, |s| first(s, "\u{25CC}", 0));
    // One whole state (a torn frame can show the old menu next to the new).
    term.wait_for("only the Wip menu", STEP, |s| {
        s.contains(Kind::Wip.marker()) && !s.contains(Kind::Log.marker())
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let cols = term.screen().cols();
    term.click(cols - 1, 0).unwrap();
    wait_closed(&term, Kind::Wip);
    term.stop().unwrap();
}
