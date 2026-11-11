//! The Lost work and Reflog dialog as panes (commit list | header, changed
//! files, hunks) and as a movable, resizable window, end to end through a
//! pty: the real binary, daemon and `git`. Every wait describes the whole
//! state it expects (a terminal has no frame boundaries), and every gesture
//! that can land early is retried from the geometry it finds afresh.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const ATTEMPT: Duration = Duration::from_secs(5);

struct Rig {
    term: Terminal,
    _repo: FixtureRepo,
    _config: tempfile::TempDir,
}

fn launch(repo: FixtureRepo, session: &str, config: &str) -> Rig {
    let (term, config) = common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .no_exit_pause()
        .config(config)
        .ready_when("the toolbar and the log", |s| {
            s.contains("Undo") && s.contains("initial commit")
        })
        .mouse_capture()
        .start();
    Rig {
        term,
        _repo: repo,
        _config: config,
    }
}

/// `main` with an initial commit, and a deleted branch `doomed` whose one
/// commit changed `a.txt` and added `d.txt` (so it is lost work).
fn lost_repo() -> (FixtureRepo, String) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .build();
    let lost = lose_commit(
        &repo,
        "doomed",
        "doomed",
        &[
            ("a.txt", "one\nchanged-in-doomed\n"),
            ("d.txt", "from-doomed\n"),
        ],
    );
    (repo, lost)
}

/// Commits `files` on a new branch `name` with the message `message`, goes
/// back to `main` and deletes the branch: its commit is lost work.
fn lose_commit(
    repo: &FixtureRepo,
    name: &str,
    word: &str,
    files: &[(&str, &str)],
) -> String {
    let root = repo.root();
    common::git(root, &["checkout", "-b", name]);
    for (path, text) in files {
        std::fs::write(root.join(path), text).unwrap();
        common::git(root, &["add", path]);
    }
    common::git(root, &["commit", "-m", &format!("{word} work")]);
    let lost = common::git(root, &["rev-parse", name]).trim().to_string();
    common::git(root, &["checkout", "main"]);
    common::git(root, &["branch", "-D", name]);
    lost
}

/// `main` with a second commit that changed `a.txt` and added `d.txt`.
fn reflog_repo() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .build();
    let root = repo.root();
    std::fs::write(root.join("a.txt"), "one\nchanged-in-second\n").unwrap();
    std::fs::write(root.join("d.txt"), "from-second\n").unwrap();
    common::git(root, &["add", "a.txt", "d.txt"]);
    common::git(root, &["commit", "-m", "second commit"]);
    repo
}

fn wait(term: &mut Terminal, what: &str, f: impl FnMut(&Screen) -> bool) -> Screen {
    term.wait_for(what, STEP, f)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()))
}

/// Opens Lost work from the palette and waits for the whole first picture:
/// the list row, the header, the file list and the first file's hunks.
fn open_lost(rig: &mut Rig, lost: &str) {
    rig.term.send_keys("C-p").unwrap();
    rig.term.send_text("recover lost").unwrap();
    wait(&mut rig.term, "the palette row", |s| {
        s.contains("Recover Lost")
    });
    rig.term.send_keys("Enter").unwrap();
    let short = lost[..7].to_string();
    wait(&mut rig.term, "the lost commit with its first file", move |s| {
        s.contains(&short)
            && s.contains("2 files changed")
            && s.contains("Changes in a.txt")
            && s.contains("changed-in-doomed")
    });
}

/// Opens the Undo history (Alt+Z) and switches to Reflog with `2`, then
/// waits for the whole first picture of the newest position.
fn open_reflog(rig: &mut Rig) {
    rig.term.send_keys("M-z").unwrap();
    wait(&mut rig.term, "the Undo history panel", |s| {
        s.contains("can be undone") || s.contains("Undo history")
    });
    rig.term.send_keys("2").unwrap();
    wait(&mut rig.term, "the newest reflog position with its first file", |s| {
        s.contains("commit: second commit")
            && s.contains("2 files changed")
            && s.contains("Changes in a.txt")
            && s.contains("changed-in-second")
    });
}

/// The dialog's grip (the bottom right corner) or `None`.
fn grip(s: &Screen) -> Option<(u16, u16)> {
    s.find("\u{25e2}")
}

/// The title of the open dialog (the first row it appears on: the view tab
/// with the same words is below it).
fn title_at(s: &Screen, title: &str) -> Option<(u16, u16)> {
    s.find_all(title).into_iter().min_by_key(|(c, r)| (*r, *c))
}

/// Where the right pane starts: the column of the file box's title.
fn files_title_col(s: &Screen) -> Option<u16> {
    s.find("2 files changed").map(|(c, _)| c)
}

fn diff_title_row(s: &Screen) -> Option<u16> {
    s.find("Changes in").map(|(_, r)| r)
}

/// Retries `act` (it re-finds its target on every attempt) until `ok` holds.
fn act_until(
    term: &mut Terminal,
    what: &str,
    act: impl FnMut(&mut Terminal) -> xtask::pty::Result<()>,
    ok: impl FnMut(&Screen) -> bool,
) -> Screen {
    term.act_until(what, ATTEMPT, STEP, act, ok)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()))
}

#[test]
fn clicking_a_file_of_lost_work_shows_its_hunks() {
    let (repo, lost) = lost_repo();
    let mut rig = launch(repo, "pty-panes-lost-click", "");
    open_lost(&mut rig, &lost);
    act_until(
        &mut rig.term,
        "d.txt selected with its hunks and a.txt's gone",
        |t| {
            if let Some((c, r)) = t.screen().find("d.txt") {
                t.click(c + 1, r)?;
            }
            Ok(())
        },
        |s| {
            s.contains("Changes in d.txt")
                && s.contains("from-doomed")
                && !s.contains("changed-in-doomed")
        },
    );
    rig.term.stop().unwrap();
}

#[test]
fn clicking_a_file_of_the_reflog_shows_its_hunks() {
    let mut rig = launch(reflog_repo(), "pty-panes-reflog-click", "");
    open_reflog(&mut rig);
    act_until(
        &mut rig.term,
        "d.txt selected with its hunks and a.txt's gone",
        |t| {
            if let Some((c, r)) = t.screen().find("d.txt") {
                t.click(c + 1, r)?;
            }
            Ok(())
        },
        |s| {
            s.contains("Changes in d.txt")
                && s.contains("from-second")
                && !s.contains("changed-in-second")
        },
    );
    rig.term.stop().unwrap();
}

/// Tab walks list > files > diff; the arrows then act on the focused pane.
/// Ctrl+Tab (CSI u) and `1`/`2`/`3` switch the views.
#[test]
fn tab_walks_the_panes_and_ctrl_tab_walks_the_views() {
    let (repo, lost) = lost_repo();
    let mut rig = launch(repo, "pty-panes-focus", "");
    open_lost(&mut rig, &lost);
    // List -> Files: the arrows now pick a file.
    rig.term.send_keys("Tab Down").unwrap();
    wait(&mut rig.term, "d.txt's hunks after Tab then Down", |s| {
        s.contains("Changes in d.txt") && s.contains("from-doomed")
    });
    // Files -> Diff -> List: the arrow picks nothing in the diff, and in the
    // list there is only one commit; Up from the Files pane goes back.
    rig.term.send_keys("Tab Tab Tab Up").unwrap();
    wait(&mut rig.term, "a.txt again after the full cycle", |s| {
        s.contains("Changes in a.txt") && s.contains("changed-in-doomed")
    });
    // Ctrl+Tab: the next view (Undo history), and back with Shift.
    rig.term.send_text("\x1b[9;5u").unwrap();
    wait(&mut rig.term, "the Undo history after Ctrl+Tab", |s| {
        s.contains("can be undone") && !s.contains("2 files changed")
    });
    rig.term.send_text("\x1b[9;6u").unwrap();
    wait(&mut rig.term, "Lost work after Ctrl+Shift+Tab", |s| {
        s.contains("2 files changed")
    });
    rig.term.send_keys("1").unwrap();
    wait(&mut rig.term, "the history", |s| {
        s.contains("can be undone") && !s.contains("2 files changed")
    });
    rig.term.send_keys("3").unwrap();
    wait(&mut rig.term, "Lost work", |s| s.contains("2 files changed"));
    rig.term.stop().unwrap();
}

/// A press on the divider between the list and the right pane, a drag, and
/// the keyboard twin: Alt+Right / Alt+Left, Alt+0 resets, a double click too.
#[test]
fn the_vertical_divider_moves_by_drag_by_keys_and_resets() {
    let (repo, lost) = lost_repo();
    let mut rig = launch(repo, "pty-panes-divider-v", "");
    open_lost(&mut rig, &lost);
    let s = rig.term.screen();
    let home = files_title_col(&s).unwrap();
    // The gap column is two cells left of the file box's title text (the
    // box border, then one space of the title).
    let gap = home - 3;
    let row = diff_title_row(&s).unwrap() + 3;
    let target = gap + 18;
    act_until(
        &mut rig.term,
        "the right pane 18 columns further right",
        |t| {
            let s = t.screen();
            if let Some(now) = files_title_col(&s) {
                let gap = now - 3;
                t.drag((gap, row), (target, row))?;
            }
            Ok(())
        },
        |s| files_title_col(s) == Some(home + 18),
    );
    // The keyboard twin: Alt+Left moves it back a step, Alt+0 resets.
    rig.term.send_keys("M-Left").unwrap();
    wait(&mut rig.term, "a step to the left", |s| {
        files_title_col(s).is_some_and(|c| c < home + 18 && c > home)
    });
    rig.term.send_keys("M-Right M-Right").unwrap();
    wait(&mut rig.term, "the divider right of where the drag left it", |s| {
        files_title_col(s).is_some_and(|c| c > home + 18)
    });
    rig.term.send_keys("M-0").unwrap();
    wait(&mut rig.term, "the default place", |s| {
        files_title_col(s) == Some(home)
    });
    // A double click on the divider resets it too.
    rig.term.send_keys("M-Right").unwrap();
    wait(&mut rig.term, "moved", |s| files_title_col(s).is_some_and(|c| c > home));
    let s = rig.term.screen();
    let gap = files_title_col(&s).unwrap() - 3;
    act_until(
        &mut rig.term,
        "the double click back at the default place",
        |t| t.double_click(gap, row),
        |s| files_title_col(s) == Some(home),
    );
    rig.term.stop().unwrap();
}

/// The divider between the file list and the hunks: a drag and the keys.
#[test]
fn the_horizontal_divider_moves_by_drag_by_keys_and_resets() {
    let (repo, lost) = lost_repo();
    let mut rig = launch(repo, "pty-panes-divider-h", "");
    open_lost(&mut rig, &lost);
    let s = rig.term.screen();
    let home = diff_title_row(&s).unwrap();
    // The file box's bottom border is the row above the hunks' top border.
    let col = files_title_col(&s).unwrap() + 20;
    act_until(
        &mut rig.term,
        "the hunks box 6 rows lower",
        |t| {
            if let Some(now) = diff_title_row(&t.screen()) {
                t.drag((col, now - 1), (col, home + 5))?;
            }
            Ok(())
        },
        |s| diff_title_row(s) == Some(home + 6),
    );
    rig.term.send_keys("M-Up").unwrap();
    wait(&mut rig.term, "a step up", |s| {
        diff_title_row(s).is_some_and(|r| r < home + 6 && r > home - 3)
    });
    rig.term.send_keys("M-0").unwrap();
    wait(&mut rig.term, "the default place", |s| {
        diff_title_row(s) == Some(home)
    });
    rig.term.send_keys("M-Down").unwrap();
    wait(&mut rig.term, "a step down", |s| {
        diff_title_row(s).is_some_and(|r| r > home)
    });
    rig.term.stop().unwrap();
}

/// Dragging the title bar moves the dialog (the pointer leaves it and the
/// release is outside: the dialog stays open).
#[test]
fn dragging_the_title_bar_moves_the_dialog_and_never_closes_it() {
    let (repo, lost) = lost_repo();
    let mut rig = launch(repo, "pty-panes-move", "");
    open_lost(&mut rig, &lost);
    let s = rig.term.screen();
    let (tc, tr) = title_at(&s, " Lost work ").unwrap();
    let (gx, gy) = grip(&s).unwrap();
    // The pointer takes the dialog up and left, then leaves it altogether and
    // lets go in the window's top left corner: the dialog follows the pointer
    // there (clamped to the screen) and stays open.
    act_until(
        &mut rig.term,
        "the dialog at the top left, still open",
        |t| {
            let s = t.screen();
            if let Some((c, r)) = title_at(&s, " Lost work ") {
                t.press(c + 8, r)?;
                t.drag_to(c.saturating_sub(4), r.saturating_sub(2))?;
                t.drag_to(1, 0)?;
                t.release(1, 0)?;
            }
            Ok(())
        },
        |s| {
            title_at(s, " Lost work ") == Some((1, 0))
                && grip(s) == Some((gx - 20, gy - 3))
                && s.contains("2 files changed")
        },
    );
    assert_ne!((tc, tr), (1, 0));
    rig.term.stop().unwrap();
}

/// Dragging the bottom right corner resizes the dialog; the grip follows.
#[test]
fn dragging_the_corner_resizes_the_dialog() {
    let (repo, lost) = lost_repo();
    let mut rig = launch(repo, "pty-panes-resize", "");
    open_lost(&mut rig, &lost);
    let (gx, gy) = grip(&rig.term.screen()).unwrap();
    let target = (gx - 30, gy - 8);
    act_until(
        &mut rig.term,
        "the grip 30 columns left and 8 rows up",
        |t| {
            if let Some(g) = grip(&t.screen()) {
                t.drag(g, target)?;
            }
            Ok(())
        },
        |s| grip(s) == Some(target) && s.contains("2 files changed"),
    );
    rig.term.stop().unwrap();
}

/// `Ctrl+N` then `m`: the arrow keys move the dialog, Shift+arrows resize
/// it, `0` resets, Enter leaves; the border says so while it is on.
#[test]
fn ctrl_n_m_arranges_the_dialog_from_the_keyboard() {
    let (repo, lost) = lost_repo();
    let mut rig = launch(repo, "pty-panes-arrange", "");
    open_lost(&mut rig, &lost);
    let home_title = title_at(&rig.term.screen(), " Lost work ").unwrap();
    let home_grip = grip(&rig.term.screen()).unwrap();
    rig.term.send_keys("C-n").unwrap();
    rig.term.send_text("m").unwrap();
    wait(&mut rig.term, "the arrange banner", |s| s.contains("Arrange:"));
    rig.term.send_keys("Left Left Up").unwrap();
    wait(&mut rig.term, "the dialog 4 columns left and a row up", |s| {
        title_at(s, " Lost work ") == Some((home_title.0 - 4, home_title.1 - 1))
            && s.contains("Arrange:")
    });
    rig.term.send_keys("S-Left S-Left").unwrap();
    wait(&mut rig.term, "the grip 4 columns left of the moved one", |s| {
        grip(s) == Some((home_grip.0 - 4 - 4, home_grip.1 - 1))
    });
    rig.term.send_keys("0").unwrap();
    wait(&mut rig.term, "everything back", |s| {
        title_at(s, " Lost work ") == Some(home_title) && grip(s) == Some(home_grip)
    });
    rig.term.send_keys("Right Enter").unwrap();
    wait(&mut rig.term, "moved and left", |s| {
        title_at(s, " Lost work ") == Some((home_title.0 + 2, home_title.1)) && !s.contains("Arrange:")
    });
    rig.term.stop().unwrap();
}

/// `Ctrl+N` badges the view tabs and the three panes; a badge jumps there.
/// A badge is one character drawn over the first cell of what it names (the
/// title of a pane's border, the label of a view tab).
#[test]
fn ctrl_n_badges_jump_to_a_pane_and_to_a_view() {
    let (repo, lost) = lost_repo();
    let mut rig = launch(repo, "pty-panes-badges", "");
    open_lost(&mut rig, &lost);
    let s = rig.term.screen();
    let files_at = s.find("2 files changed").unwrap();
    let diff_at = s.find("Changes in").unwrap();
    let history_at = s.find("Undo history").unwrap();
    // The character drawn at a cell, when it is a badge (a letter or digit
    // that is not the one the title has there).
    let badge_at = |s: &Screen, (col, row): (u16, u16), plain: &str| -> Option<char> {
        let c = s.cell(col, row)?.text.chars().next()?;
        (c.is_ascii_alphanumeric() && plain.chars().next() != Some(c)).then_some(c)
    };
    // The hunks pane gets the keyboard: Down there scrolls, it does not pick
    // another file (a.txt stays), whereas on the files pane it picks d.txt.
    rig.term.send_keys("C-n").unwrap();
    let s = wait(&mut rig.term, "badges on the panes and the views", |s| {
        badge_at(s, files_at, "2").is_some()
            && badge_at(s, diff_at, "C").is_some()
            && badge_at(s, history_at, "U").is_some()
    });
    let diff_badge = badge_at(&s, diff_at, "C").unwrap();
    rig.term.send_text(&diff_badge.to_string()).unwrap();
    rig.term.send_keys("Down").unwrap();
    wait(&mut rig.term, "a.txt still shown after Down on the hunks", |s| {
        s.contains("Changes in a.txt")
            && s.contains("changed-in-doomed")
            && badge_at(s, files_at, "2").is_none()
    });
    rig.term.send_keys("C-n").unwrap();
    let s = wait(&mut rig.term, "badges again", |s| {
        badge_at(s, files_at, "2").is_some()
    });
    let files_badge = badge_at(&s, files_at, "2").unwrap();
    rig.term.send_text(&files_badge.to_string()).unwrap();
    rig.term.send_keys("Down").unwrap();
    wait(&mut rig.term, "d.txt after Down on the files", |s| {
        s.contains("Changes in d.txt") && s.contains("from-doomed")
    });
    // A view tab's badge opens that view.
    rig.term.send_keys("C-n").unwrap();
    let s = wait(&mut rig.term, "badges once more", |s| {
        badge_at(s, history_at, "U").is_some()
    });
    let history_badge = badge_at(&s, history_at, "U").unwrap();
    rig.term.send_text(&history_badge.to_string()).unwrap();
    wait(&mut rig.term, "the Undo history", |s| {
        s.contains("can be undone") && !s.contains("2 files changed")
    });
    rig.term.stop().unwrap();
}

/// A commit with many files and a long file: each list has a scrollbar that
/// can be pressed (the thumb jumps under the pointer) and dragged.
#[test]
fn the_files_and_hunks_scrollbars_can_be_pressed_and_dragged() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .build();
    let long: String = (0..150).map(|n| format!("row number {n:03}\n")).collect();
    let mut files: Vec<(String, String)> = (1..70)
        .map(|n| (format!("f{n:02}.txt"), "x\n".to_string()))
        .collect();
    files.push(("f00.txt".into(), long));
    let refs: Vec<(&str, &str)> = files.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
    lose_commit(&repo, "doomed", "many files", &refs);
    let mut rig = launch(repo, "pty-panes-bars", "");
    rig.term.send_keys("C-p").unwrap();
    rig.term.send_text("recover lost").unwrap();
    wait(&mut rig.term, "the palette row", |s| s.contains("Recover Lost"));
    rig.term.send_keys("Enter").unwrap();
    wait(&mut rig.term, "the first file and its hunks", |s| {
        s.contains("70 files changed") && s.contains("Changes in f00.txt") && s.contains("row number 000")
    });
    let s = rig.term.screen();
    let (gx, gy) = grip(&s).unwrap();
    // The boxes' right border (their scrollbar) is two columns left of the dialog's.
    let bar = gx - 2;
    let hunks_top = diff_title_row(&s).unwrap();
    let files_bottom = hunks_top - 1;
    // Files bar: a press near its bottom jumps the thumb there (the last
    // files show), a drag back up shows the first ones again.
    act_until(
        &mut rig.term,
        "the last files listed",
        |t| t.click(bar, files_bottom - 2),
        |s| s.contains("+ f6") && !s.contains("+ f00.txt") && !s.contains("+ f01.txt"),
    );
    act_until(
        &mut rig.term,
        "the first files listed again after a drag up",
        |t| t.drag((bar, files_bottom - 2), (bar, hunks_top.saturating_sub(10))),
        |s| s.contains("+ f01.txt") && !s.contains("+ f6"),
    );
    // Hunks bar: a press near the bottom of the box jumps to the end.
    let hunks_bottom = gy - 4;
    act_until(
        &mut rig.term,
        "the end of the hunks",
        |t| t.click(bar, hunks_bottom - 2),
        |s| s.contains("row number 14") && !s.contains("row number 000"),
    );
    act_until(
        &mut rig.term,
        "the start of the hunks again after a drag up",
        |t| t.drag((bar, hunks_bottom - 2), (bar, hunks_top + 1)),
        |s| s.contains("row number 000") && !s.contains("row number 1"),
    );
    rig.term.stop().unwrap();
}

/// `session` (the default) keeps a resized dialog for the next time it is
/// opened; `never` opens it at its default size again.
fn resize_close_reopen(config: &str, session: &str) -> ((u16, u16), (u16, u16)) {
    let (repo, lost) = lost_repo();
    let mut rig = launch(repo, session, config);
    open_lost(&mut rig, &lost);
    let (gx, gy) = grip(&rig.term.screen()).unwrap();
    let target = (gx - 24, gy - 6);
    act_until(
        &mut rig.term,
        "the dialog smaller",
        |t| {
            if let Some(g) = grip(&t.screen()) {
                t.drag(g, target)?;
            }
            Ok(())
        },
        |s| grip(s) == Some(target) && s.contains("2 files changed"),
    );
    rig.term.send_keys("Escape").unwrap();
    wait(&mut rig.term, "the dialog closed", |s| {
        !s.contains("2 files changed") && grip(s).is_none()
    });
    open_lost(&mut rig, &lost);
    let again = grip(&rig.term.screen()).unwrap();
    rig.term.stop().unwrap();
    (target, again)
}

#[test]
fn the_session_setting_keeps_the_size_when_the_dialog_reopens() {
    let (smaller, again) = resize_close_reopen("", "pty-panes-keep-session");
    assert_eq!(again, smaller);
}

#[test]
fn the_never_setting_resets_the_size_when_the_dialog_reopens() {
    let (smaller, again) = resize_close_reopen(
        "[ui]\ndialog_geometry = \"never\"\n",
        "pty-panes-keep-never",
    );
    assert_ne!(again, smaller, "back at the default size");
    assert!(again.0 > smaller.0 && again.1 > smaller.1);
}

/// Enter on a lost commit is not "Create branch here" (or anything else):
/// no dialog opens and no branch appears.
#[test]
fn enter_on_a_lost_commit_changes_nothing() {
    let (repo, lost) = lost_repo();
    let root = repo.root().to_path_buf();
    let mut rig = launch(repo, "pty-panes-enter", "");
    open_lost(&mut rig, &lost);
    rig.term.send_keys("Enter").unwrap();
    // The letter `t` opens the tag name prompt: it shows only if the Enter
    // before it opened nothing (a branch prompt would have swallowed it).
    rig.term.send_keys("t").unwrap();
    wait(&mut rig.term, "the tag prompt and no branch prompt", |s| {
        s.contains("restored-") && !s.contains("Name of the new branch")
    });
    rig.term.send_keys("Escape").unwrap();
    let branches = common::git(&root, &["branch", "--list"]);
    assert!(!branches.contains("restored-"), "{branches}");
    rig.term.stop().unwrap();
}
