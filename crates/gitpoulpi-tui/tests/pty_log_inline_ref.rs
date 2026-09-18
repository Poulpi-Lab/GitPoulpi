//! Naming a new branch / tag inline in the Log (a text box in the Ref
//! column, on the target commit's row), end to end through a pty. Same
//! rules as `pty_ui.rs`.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("older target commit")
        .unwrap()
        .commit("newer head commit")
        .unwrap()
        .build()
}

fn launch(repo: &FixtureRepo, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .ready_when("the fixture loaded", |s| {
            s.contains("older target commit") && s.contains("newer head commit")
        })
        .start()
}

fn wait_ref(repo: &FixtureRepo, args: &[&str]) -> String {
    for _ in 0..300 {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(repo.root())
            .output()
            .unwrap();
        if out.status.success() {
            return String::from_utf8_lossy(&out.stdout).trim().to_string();
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    panic!("git {args:?} never succeeded");
}

fn right_click_menu(term: &mut Terminal, target: &str, entry: &str) {
    let (col, row) = term.wait_for_text(target, STEP).unwrap();
    term.right_click(col + 1, row).unwrap();
    let menu = term
        .wait_for("the commit menu", STEP, |s| s.contains(entry))
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find(entry).unwrap();
    term.click(c + 1, r).unwrap();
}

/// The placeholder sits on the very row of the target commit (not in the
/// status bar), and Enter creates the branch there.
#[test]
fn create_branch_here_names_the_branch_inline_on_the_commits_row() {
    let repo = fixture();
    let target = repo.git(&["rev-parse", "HEAD~1"]);
    let (mut term, _c) = launch(&repo, "pty-inline-branch");
    right_click_menu(&mut term, "older target commit", "Create branch here...");
    let screen = term
        .wait_for("the inline box", STEP, |s| s.contains("enter branch name"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (_, row) = screen.find("enter branch name").unwrap();
    assert!(
        screen.line(row).contains("older target commit"),
        "the box must be on the target commit's row: {}",
        screen.line(row)
    );
    assert!(
        !screen.contains("New branch at"),
        "the modal/status-bar prompt is gone"
    );
    term.send_text("feat-inline").unwrap();
    term.wait_for("the typed name", STEP, |s| {
        s.contains("feat-inline") && !s.contains("enter branch name")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    assert_eq!(wait_ref(&repo, &["rev-parse", "feat-inline"]), target);
    term.wait_for("the box closed", STEP, |s| !s.contains("enter branch name"))
        .unwrap_or_else(|e| panic!("{e}"));
}

/// A refused name is reported on the row, the box stays open, Esc
/// cancels and nothing is created.
#[test]
fn an_invalid_name_shows_an_error_inline_and_esc_cancels() {
    let repo = fixture();
    let (mut term, _c) = launch(&repo, "pty-inline-error");
    right_click_menu(&mut term, "older target commit", "Create tag here...");
    term.wait_for("the tag box", STEP, |s| s.contains("enter tag name"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("bad name").unwrap();
    term.send_keys("Enter").unwrap();
    let screen = term
        .wait_for("the inline error", STEP, |s| {
            s.contains("can't contain spaces")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (_, row) = screen.find("can't contain spaces").unwrap();
    assert!(
        screen.line(row).contains("bad name"),
        "the error shares the box's row: {}",
        screen.line(row)
    );
    term.send_keys("Escape").unwrap();
    term.wait_for("the box closed", STEP, |s| {
        !s.contains("can't contain spaces") && !s.contains("bad name")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(repo.git(&["tag", "--list"]), "");
}

/// The keyboard path (command palette, commit selected first) opens the
/// same inline box, and an existing name is refused inline.
#[test]
fn the_palette_entry_uses_the_inline_box_and_refuses_an_existing_name() {
    let repo = fixture();
    repo.git(&["tag", "v1"]);
    let target = repo.git(&["rev-parse", "HEAD~1"]);
    let (mut term, _c) = launch(&repo, "pty-inline-palette");
    let (col, row) = term.wait_for_text("older target commit", STEP).unwrap();
    term.click(col + 1, row).unwrap();
    term.send_keys("C-p").unwrap();
    term.send_text("Create Tag at Selected").unwrap();
    term.wait_for("the palette narrowed", STEP, |s| {
        s.contains("Command Palette (1/")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the tag box", STEP, |s| s.contains("enter tag name"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("v1").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the duplicate is refused inline", STEP, |s| {
        s.contains("already exists")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Fix the name: Backspace, type "2", Enter.
    term.send_keys("Backspace").unwrap();
    term.send_text("2").unwrap();
    term.send_keys("Enter").unwrap();
    assert_eq!(wait_ref(&repo, &["rev-parse", "v2^{commit}"]), target);
}

/// `Ctrl+G b` (branch at HEAD) uses the box too, on HEAD's row.
#[test]
fn the_ctrl_g_chord_names_a_branch_at_head_inline() {
    let repo = fixture();
    let head = repo.git(&["rev-parse", "HEAD"]);
    let (mut term, _c) = launch(&repo, "pty-inline-chord");
    term.send_keys("C-g").unwrap();
    term.send_text("b").unwrap();
    let screen = term
        .wait_for("the inline box", STEP, |s| s.contains("enter branch name"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (_, row) = screen.find("enter branch name").unwrap();
    assert!(screen.line(row).contains("newer head commit"));
    term.send_text("from-chord").unwrap();
    term.send_keys("Enter").unwrap();
    assert_eq!(wait_ref(&repo, &["rev-parse", "from-chord"]), head);
}

/// Creating a branch checks it out: HEAD is the new branch, on the
/// chosen commit.
#[test]
fn creating_a_branch_checks_it_out() {
    let repo = fixture();
    let target = repo.git(&["rev-parse", "HEAD~1"]);
    let (mut term, _c) = launch(&repo, "pty-inline-checkout");
    right_click_menu(&mut term, "older target commit", "Create branch here...");
    term.wait_for("the inline box", STEP, |s| s.contains("enter branch name"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("feat-co").unwrap();
    term.send_keys("Enter").unwrap();
    assert_eq!(wait_ref(&repo, &["rev-parse", "feat-co"]), target);
    for _ in 0..300 {
        if repo.git(&["rev-parse", "--abbrev-ref", "HEAD"]) == "feat-co" {
            return;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    panic!("HEAD never moved to feat-co");
}

/// A name that clashes with an existing `name/...` ref (ref directory/file
/// conflict) is refused on the row, the box stays open, nothing changes.
#[test]
fn a_branch_namespace_clash_is_shown_and_keeps_the_box_open() {
    let repo = fixture();
    repo.git(&["branch", "test/my/branch"]);
    let (mut term, _c) = launch(&repo, "pty-inline-clash");
    term.send_keys("C-g").unwrap();
    term.send_text("b").unwrap();
    term.wait_for("the inline box", STEP, |s| s.contains("enter branch name"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("test").unwrap();
    term.send_keys("Enter").unwrap();
    let screen = term
        .wait_for("the clash is reported", STEP, |s| {
            s.contains("clashes with the existing 'test/my/branch'")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (_, row) = screen.find("clashes with").unwrap();
    assert!(
        screen.line(row).contains("test"),
        "box still open on the row"
    );
    assert_eq!(repo.git(&["branch", "--list", "test"]), "");
}

/// Same for tags: the clash is shown, the box stays, and a created tag
/// leaves HEAD where it was.
#[test]
fn a_tag_namespace_clash_is_shown_and_tags_are_not_checked_out() {
    let repo = fixture();
    repo.git(&["tag", "rel/1.0"]);
    let head_before = repo.git(&["rev-parse", "--abbrev-ref", "HEAD"]);
    let (mut term, _c) = launch(&repo, "pty-inline-tag-clash");
    term.send_keys("C-g").unwrap();
    term.send_text("t").unwrap();
    term.wait_for("the tag box", STEP, |s| s.contains("enter tag name"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("rel").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the clash is reported", STEP, |s| {
        s.contains("clashes with the existing 'rel/1.0'")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    for _ in 0..3 {
        term.send_keys("Backspace").unwrap();
    }
    term.send_text("v9").unwrap();
    term.send_keys("Enter").unwrap();
    wait_ref(&repo, &["rev-parse", "v9^{commit}"]);
    assert_eq!(
        repo.git(&["rev-parse", "--abbrev-ref", "HEAD"]),
        head_before
    );
}
