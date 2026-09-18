//! "Split commit" end to end through a pty (`xtask::pty`): the real binary,
//! daemon and git. The editor opens from the Log menu and from the palette,
//! changes are assigned by keys and by mouse (the `[N v]` list), the split
//! rewrites the history, Undo puts it back, and a daemon refusal keeps the
//! editor open on the same plan.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps, every wait describes the state it waits for.

mod common;

use std::time::{Duration, Instant};

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

fn numbered(change_first: bool, change_last: bool) -> String {
    (1..=20)
        .map(|n| match (n, change_first, change_last) {
            (1, true, _) => "line 01 changed\n".to_string(),
            (20, _, true) => "line 20 changed\n".to_string(),
            _ => format!("line {n:02}\n"),
        })
        .collect()
}

/// `base`, then `parser and flag` (two hunks of `a.txt`, a new `b.txt` and a
/// new file named after the commit: three of the four assignable units are
/// whole files), then `tip on top`.
fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", &numbered(false, false), "base")
        .unwrap()
        .write_worktree("a.txt", numbered(true, true).as_bytes())
        .unwrap()
        .stage("a.txt")
        .unwrap()
        .write_worktree("b.txt", b"bee\n")
        .unwrap()
        .stage("b.txt")
        .unwrap()
        .commit("parser and flag")
        .unwrap()
        .commit("tip on top")
        .unwrap()
        .build()
}

fn subjects(repo: &FixtureRepo) -> Vec<String> {
    repo.git(&["log", "--format=%s"])
        .lines()
        .map(str::to_string)
        .collect()
}

fn wait_for_subjects(repo: &FixtureRepo, wanted: &[&str]) {
    let deadline = Instant::now() + STEP;
    while subjects(repo) != wanted {
        assert!(
            Instant::now() < deadline,
            "history is {:?}, wanted {wanted:?}",
            subjects(repo)
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn launch(repo: &FixtureRepo, session: &str) -> (Terminal, tempfile::TempDir) {
    let config = tempfile::tempdir().expect("temp config dir");
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| {
        s.contains("tip on top") && s.contains("parser and flag")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    (term, config)
}

/// Right-click `parser and flag` -> Split commit...
fn open_from_menu(term: &mut Terminal) {
    let (col, row) = term.wait_for_text("parser and flag", STEP).unwrap();
    term.right_click(col + 1, row)
        .expect("right-click the commit");
    let menu = term
        .wait_for("the commit menu with Split commit...", STEP, |s| {
            s.contains("Split commit...")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Split commit...").unwrap();
    term.click(c + 1, r).expect("click Split commit...");
    wait_editor(term);
}

fn wait_editor(term: &Terminal) {
    term.wait_for("the split editor with its changes", STEP, |s| {
        s.contains("Split into 2") && s.contains("b.txt") && s.contains("New commits")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// How many `[ n v]` assignment fields show the number `n`.
fn owned_by(term: &Terminal, n: u8) -> usize {
    term.screen().find_all(&format!("[ {n} \u{25be}]")).len()
}

/// Gives the second hunk of `a.txt`, `b.txt` and the commit-named file to a
/// second commit, by keys, and names it.
fn plan_by_keys(term: &mut Terminal) {
    // The cursor opens on a.txt: down to its second hunk.
    term.send_keys("Down").unwrap();
    term.send_keys("Down").unwrap();
    term.send_text("2").unwrap();
    term.send_keys("Down").unwrap();
    term.send_text("2").unwrap();
    term.wait_for("two changes now in the second commit", STEP, |s| {
        s.find_all("[ 2 \u{25be}]").len() == 2
    })
    .unwrap_or_else(|e| panic!("{e}"));
    name_second(term, "second part");
}

/// Tab to the commits, select the second, edit its message.
fn name_second(term: &mut Terminal, message: &str) {
    term.send_keys("Tab").unwrap();
    term.send_text("j").unwrap();
    term.send_text("e").unwrap();
    term.wait_for("the message editor", STEP, |s| {
        s.contains("Reword commit message")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text(message).unwrap();
    term.send_keys("C-s").unwrap();
    term.wait_for("the message in the commits pane", STEP, |s| {
        !s.contains("Reword commit message") && s.contains(&format!("2 {message}"))
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// Menu entry, keys, Ctrl+S, the history, then Ctrl+Z.
#[test]
fn a_commit_is_split_by_keys_from_the_log_menu_and_undo_restores_it() {
    let repo = fixture();
    let original = subjects(&repo);
    let tip = repo.git(&["rev-parse", "HEAD"]);
    let tree = repo.git(&["rev-parse", "HEAD^{tree}"]);
    let (mut term, _config) = launch(&repo, "pty-split-keys");
    open_from_menu(&mut term);
    plan_by_keys(&mut term);
    term.send_keys("C-s").unwrap();
    term.wait_for("the editor closed", STEP, |s| !s.contains("Split into 2"))
        .unwrap_or_else(|e| panic!("{e}"));
    wait_for_subjects(
        &repo,
        &["tip on top", "second part", "parser and flag", "base"],
    );
    assert_eq!(repo.git(&["rev-parse", "HEAD^{tree}"]), tree);
    // The first commit keeps the first hunk and the file named after it;
    // the second takes the other hunk and b.txt.
    assert_eq!(
        repo.git(&["show", "--format=", "--name-only", "HEAD~2"]),
        "a.txt\nparser and flag.txt"
    );
    assert_eq!(
        repo.git(&["show", "--format=", "--name-only", "HEAD~1"]),
        "a.txt\nb.txt"
    );
    assert_eq!(repo.git(&["status", "--porcelain"]), "");
    term.wait_for("the new commits in the log", STEP, |s| {
        s.contains("second part")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Ctrl+Z puts the original tip back.
    term.send_keys("C-z").unwrap();
    wait_for_subjects(
        &repo,
        &original.iter().map(String::as_str).collect::<Vec<_>>(),
    );
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), tip);
}

/// Palette entry, the `[N v]` list opened and picked with the mouse, a
/// double click on the commit to name it, the Split button.
#[test]
fn a_commit_is_split_by_mouse_from_the_palette_through_the_assignment_list() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "pty-split-mouse");
    let (col, row) = term.wait_for_text("parser and flag", STEP).unwrap();
    term.click(col + 1, row).expect("select the commit");
    term.send_keys("C-p").unwrap();
    term.send_text("split selected").unwrap();
    term.wait_for("the palette entry", STEP, |s| {
        s.contains("Git: Split Selected Commit...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    wait_editor(&term);
    assert_eq!(
        owned_by(&term, 1),
        5,
        "everything starts in the first commit"
    );

    // The third `[1 v]` is the second hunk of a.txt.
    let mut fields = term.screen().find_all("[ 1 \u{25be}]");
    fields.sort_by_key(|&(_, r)| r);
    let (c, r) = fields[2];
    term.click(c + 1, r).expect("open the list");
    let screen = term
        .wait_for("the list of commits", STEP, |s| {
            s.contains("Unassigned") && s.contains("+ New commit")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    // "2 (no message)" shows in the commits pane and in the list (the one
    // further right).
    let mut items = screen.find_all("2 (no message)");
    items.sort_by_key(|&(c, _)| c);
    let (ic, ir) = *items.last().unwrap();
    assert!(items.len() >= 2, "{items:?}");
    term.click(ic + 1, ir).expect("pick commit 2");
    term.wait_for("one change in the second commit", STEP, |s| {
        s.find_all("[ 2 \u{25be}]").len() == 1 && !s.contains("Unassigned")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // A double click on the commit names it.
    let (nc, nr) = *items.first().unwrap();
    term.double_click(nc + 4, nr)
        .expect("double click the commit");
    term.wait_for("the message editor", STEP, |s| {
        s.contains("Reword commit message")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("tail hunk").unwrap();
    term.send_keys("C-s").unwrap();
    let screen = term
        .wait_for("the Split button ready", STEP, |s| {
            !s.contains("Reword commit message") && s.contains("2 tail hunk")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (bc, br) = screen.find("Split into 2").unwrap();
    term.click(bc + 2, br).expect("click Split into 2");
    wait_for_subjects(
        &repo,
        &["tip on top", "tail hunk", "parser and flag", "base"],
    );
    // The second hunk went to the new commit: it changes only a.txt.
    assert_eq!(
        repo.git(&["show", "--format=", "--name-only", "HEAD~1"]),
        "a.txt"
    );
}

/// A refusal at apply keeps the editor and the plan; the reason shows.
#[test]
fn a_refusal_keeps_the_editor_open_on_the_same_plan() {
    let repo = fixture();
    let base = repo.git(&["rev-parse", "HEAD~2"]);
    let (mut term, _config) = launch(&repo, "pty-split-refused");
    open_from_menu(&mut term);
    plan_by_keys(&mut term);
    // Another client moves the checkout off the commit being split: the
    // daemon will refuse.
    repo.git(&["checkout", "-q", "-b", "elsewhere", &base]);
    term.send_keys("C-s").unwrap();
    term.wait_for("the refusal in the editor", STEP, |s| {
        s.contains("cannot split that commit") && s.contains("Split into 2")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // The plan survived: two changes still belong to the second commit.
    assert_eq!(owned_by(&term, 2), 2);
    assert_eq!(repo.git(&["rev-parse", "elsewhere"]), base);
    term.send_keys("Escape").unwrap();
    term.wait_for("the editor closed", STEP, |s| !s.contains("Split into 2"))
        .unwrap_or_else(|e| panic!("{e}"));
}

/// Outside the frame: a pristine plan closes, an edited one swallows the
/// click; the cross still closes it.
#[test]
fn an_outside_click_is_swallowed_once_the_plan_is_edited_and_the_cross_closes() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "pty-split-dismiss");
    open_from_menu(&mut term);
    let outside = |term: &Terminal| {
        let screen = term.screen();
        let (c, r) = screen.find("Split commit").expect("the title");
        // Two columns left of the frame's corner, a few rows down.
        (c.saturating_sub(4), r + 6)
    };
    // Edited: assign a change to the second commit.
    term.send_keys("Down").unwrap();
    term.send_keys("Down").unwrap();
    term.send_text("2").unwrap();
    term.wait_for("one change moved", STEP, |s| {
        s.find_all("[ 2 \u{25be}]").len() == 1
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (oc, or) = outside(&term);
    term.click(oc, or).expect("click outside");
    // Still there after the click has been handled: undo the edit and
    // check the frame is the same editor.
    term.send_text("1").unwrap();
    term.wait_for("the editor still open, plan reset by hand", STEP, |s| {
        s.find_all("[ 1 \u{25be}]").len() == 5 && s.contains("Split into 2")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Pristine again: the same click now closes.
    term.click(oc, or).expect("click outside");
    term.wait_for("the editor closed", STEP, |s| !s.contains("Split into 2"))
        .unwrap_or_else(|e| panic!("{e}"));
    // Reopen, edit, close with the cross.
    open_from_menu(&mut term);
    term.send_keys("Down").unwrap();
    term.send_keys("Down").unwrap();
    term.send_text("2").unwrap();
    let screen = term
        .wait_for("one change moved", STEP, |s| {
            s.find_all("[ 2 \u{25be}]").len() == 1
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let top = screen
        .text()
        .lines()
        .find(|l| l.contains("Split commit") && l.contains('\u{2715}'))
        .map(str::to_string)
        .expect("the frame's top border carries the cross");
    assert!(!top.contains("Esc"), "{top}");
    let (xc, xr) = screen
        .find_all("\u{2715}")
        .into_iter()
        .find(|&(_, r)| screen.line(r).contains("Split commit"))
        .unwrap();
    term.click(xc + 1, xr).expect("click the cross");
    term.wait_for("the editor closed", STEP, |s| !s.contains("Split into 2"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(subjects(&repo), ["tip on top", "parser and flag", "base"]);
}

/// Right-click `base` -> Interactively rebase from here...
fn open_rebase(term: &mut Terminal) {
    let (col, row) = term.wait_for_text("base", STEP).unwrap();
    term.right_click(col + 1, row)
        .expect("right-click the base commit");
    let menu = term
        .wait_for("the commit menu", STEP, |s| {
            s.contains("Interactively rebase from here...")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Interactively rebase from here...").unwrap();
    term.click(c + 1, r).expect("click the rebase row");
    term.wait_for("the rebase popup", STEP, |s| s.contains("Start rebase"))
        .unwrap_or_else(|e| panic!("{e}"));
}

/// The "split..." button of the `parser and flag` row.
fn click_row_split(term: &mut Terminal) {
    let screen = term.screen();
    let (_, r) = screen
        .find_all("parser and flag")
        .into_iter()
        .find(|&(_, r)| screen.line(r).contains("split..."))
        .expect("a split... button on the parser and flag row");
    let (c, _) = screen
        .find_all("split...")
        .into_iter()
        .find(|&(_, row)| row == r)
        .unwrap();
    term.click(c + 1, r).expect("click split...");
}

/// The rebase popup's "split..." opens the editor on that commit; leaving
/// it returns to the popup.
#[test]
fn the_rebase_popup_split_button_opens_the_editor_and_cancel_returns_to_the_popup() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "pty-split-rebase-cancel");
    open_rebase(&mut term);
    click_row_split(&mut term);
    wait_editor(&term);
    assert!(!term.screen().contains("Start rebase"));
    term.send_keys("Escape").unwrap();
    term.wait_for("the rebase popup is back", STEP, |s| {
        s.contains("Start rebase") && !s.contains("Split into 2")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(subjects(&repo), ["tip on top", "parser and flag", "base"]);
}

/// Splitting from the popup rewrites the history and the popup comes back
/// listing the new commits.
#[test]
fn splitting_from_the_rebase_popup_returns_to_it_on_the_new_history() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "pty-split-rebase-done");
    open_rebase(&mut term);
    // The keyboard twin of the button: the cursor starts on the newest row.
    term.send_keys("Down").unwrap();
    term.send_text("t").unwrap();
    wait_editor(&term);
    plan_by_keys(&mut term);
    term.send_keys("C-s").unwrap();
    wait_for_subjects(
        &repo,
        &["tip on top", "second part", "parser and flag", "base"],
    );
    term.wait_for("the popup back on the new history", STEP, |s| {
        s.contains("Start rebase")
            && s.contains("second part")
            && s.contains("parser and flag")
            && s.contains("tip on top")
            && !s.contains("Split into 2")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

/// An edited plan is not thrown away by one press: the first says so, the
/// second splits.
#[test]
fn an_edited_plan_asks_for_a_second_press_before_it_is_discarded() {
    let repo = fixture();
    let (mut term, _config) = launch(&repo, "pty-split-rebase-confirm");
    open_rebase(&mut term);
    // Edit the plan: drop the newest row.
    term.send_text("d").unwrap();
    click_row_split(&mut term);
    term.wait_for("the warning", STEP, |s| {
        s.contains("press split again") && s.contains("Start rebase")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(!term.screen().contains("Split into 2"));
    click_row_split(&mut term);
    wait_editor(&term);
}
