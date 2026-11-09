//! The Log's combined context menus, end to end through a pty: a ref
//! pill's menu (the ref's rows and the commit's rows in one list), the
//! `+N` overflow pill reaching every ref on a commit, branch deletion from
//! the graph, and a commit row's per-ref submenus. Same rules as
//! `pty_ui.rs`: nothing is located by a hard-coded coordinate.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

/// Two commits; `hello` shares the newest one with the checked-out
/// branch (so its pill hides behind `+1`), `to-delete` sits on the older.
fn fixture(with_origin: bool) -> FixtureRepo {
    let mut builder = FixtureRepoBuilder::new()
        .unwrap()
        .commit("older target commit")
        .unwrap()
        .commit("newer head commit")
        .unwrap();
    if with_origin {
        builder = builder.with_origin().unwrap();
    }
    let repo = builder.build();
    repo.git(&["branch", "hello"]);
    repo.git(&["branch", "to-delete", "HEAD~1"]);
    repo
}

fn launch(repo: &FixtureRepo, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .size(200, 60)
        .ready_when("the fixture loaded", |s| {
            s.contains("older target commit") && s.contains("newer head commit")
        })
        .start()
}

fn branches(repo: &FixtureRepo) -> String {
    repo.git(&["branch", "--list"])
}

fn wait_branch_gone(repo: &FixtureRepo, name: &str) {
    for _ in 0..300 {
        if !branches(repo).contains(name) {
            return;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    panic!("branch {name} was never deleted: {}", branches(repo));
}

/// Right-click the pill of `name` (the ref column, on `commit_text`'s row).
fn right_click_pill(term: &mut Terminal, commit_text: &str, name: &str) {
    let (_, row) = term.wait_for_text(commit_text, STEP).unwrap();
    let screen = term.screen();
    let col = screen
        .find_all(name)
        .into_iter()
        .find(|(_, r)| *r == row)
        .unwrap_or_else(|| panic!("no `{name}` pill on the row of {commit_text}"))
        .0;
    term.right_click(col + 1, row).unwrap();
}

/// Deleting a branch from its pill: the row is in the menu, it confirms,
/// and the branch is gone.
#[test]
fn deleting_a_branch_from_the_graph_confirms_and_deletes() {
    let repo = fixture(false);
    let (mut term, _c) = launch(&repo, "pty-pill-delete");
    right_click_pill(&mut term, "older target commit", "to-delete");
    let menu = term
        .wait_for("the pill menu", STEP, |s| s.contains("Delete to-delete"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Delete to-delete").unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("the confirm prompt", STEP, |s| {
        s.contains("Delete branch 'to-delete'?")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("y").unwrap();
    wait_branch_gone(&repo, "to-delete");
}

/// The checked-out branch is refused with a message, not deleted.
#[test]
fn deleting_the_checked_out_branch_is_refused_with_a_message() {
    let repo = fixture(false);
    let current = repo.git(&["branch", "--show-current"]);
    let (mut term, _c) = launch(&repo, "pty-pill-refuse");
    right_click_pill(&mut term, "newer head commit", &current);
    let menu = term
        .wait_for("the pill menu", STEP, |s| {
            s.contains(&format!("Delete {current}"))
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find(&format!("Delete {current}")).unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("the refusal", STEP, |s| {
        s.contains("it is checked out") && !s.contains("Delete branch '")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(branches(&repo).contains(&current));
}

/// A branch also on the remote gets the three delete rows; deleting the
/// remote one removes it on the remote and keeps the local branch.
#[test]
fn a_remote_branch_is_deleted_from_the_menu_and_the_local_one_stays() {
    let repo = fixture(true);
    repo.git(&["push", "origin", "to-delete"]);
    let (mut term, _c) = launch(&repo, "pty-pill-remote");
    right_click_pill(&mut term, "older target commit", "to-delete");
    let menu = term
        .wait_for("the delete rows", STEP, |s| {
            s.contains("Delete to-delete and origin/to-delete")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(menu.contains("Delete origin/to-delete"));
    let (c, r) = menu.find("Delete origin/to-delete").unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("the confirm prompt", STEP, |s| s.contains("from origin?"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("y").unwrap();
    for _ in 0..300 {
        if repo
            .git_in_remote(&["branch", "--list", "to-delete"])
            .is_empty()
        {
            assert!(branches(&repo).contains("to-delete"), "the local one stays");
            return;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    panic!("the remote branch was never deleted");
}

/// Deleting a branch on the remote is undone with Ctrl+Z: it asks first (the
/// undo re-creates the branch on the server), and after the confirm the
/// remote has the very same commit again. Falsified by breaking the undo of a
/// remote deletion (the branch stays gone).
#[test]
fn a_deleted_remote_branch_is_undone_with_ctrl_z_after_a_confirm() {
    let repo = fixture(true);
    repo.git(&["push", "origin", "to-delete"]);
    let tip = repo.git(&["rev-parse", "to-delete"]);
    let (mut term, _c) = launch(&repo, "pty-pill-remote-undo");
    right_click_pill(&mut term, "older target commit", "to-delete");
    let menu = term
        .wait_for("the delete rows", STEP, |s| {
            s.contains("Delete origin/to-delete")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Delete origin/to-delete").unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("the confirm prompt", STEP, |s| s.contains("from origin?"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("y").unwrap();
    for _ in 0..300 {
        if repo
            .git_in_remote(&["branch", "--list", "to-delete"])
            .is_empty()
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    assert!(
        repo.git_in_remote(&["branch", "--list", "to-delete"])
            .is_empty(),
        "the remote branch must be deleted first"
    );

    // Ctrl+Z: a network undo asks, and says what it does on the server.
    // The Undo button only enables once the daemon told the window about the
    // new entry, so the key is repeated until the question is up.
    let mut asked = false;
    for _ in 0..60 {
        term.send_keys("C-z").unwrap();
        if term
            .wait_for("the undo question", Duration::from_secs(1), |s| {
                s.contains("re-creates branch to-delete on origin")
            })
            .is_ok()
        {
            asked = true;
            break;
        }
    }
    assert!(asked, "the undo never asked:\n{}", term.text());
    assert!(
        repo.git_in_remote(&["branch", "--list", "to-delete"])
            .is_empty(),
        "asking changes nothing on the server"
    );
    term.send_keys("y").unwrap();
    for _ in 0..300 {
        if !repo
            .git_in_remote(&["branch", "--list", "to-delete"])
            .is_empty()
        {
            assert_eq!(
                repo.git_in_remote(&["rev-parse", "refs/heads/to-delete"]),
                tip,
                "the same commit is back on the remote"
            );
            return;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    panic!("the remote branch was never re-created:\n{}", term.text());
}

/// The ref's rows and the commit's rows are in one menu.
#[test]
fn the_pill_menu_carries_the_refs_rows_and_the_commits_rows() {
    let repo = fixture(false);
    let (mut term, _c) = launch(&repo, "pty-pill-merged");
    right_click_pill(&mut term, "older target commit", "to-delete");
    const ROWS: [&str; 14] = [
        "Delete to-delete",
        "Push to-delete",
        "Check out",
        "Create worktree from",
        "Create branch here...",
        "Revert commit...",
        "Edit commit message...",
        "Start a pull request",
        "Apply patch...",
        "Rename to-delete...",
        "Copy branch name",
        "Copy commit SHA",
        "Create tag here...",
        "Create annotated tag here...",
    ];
    // Rows fill in as the menu's sources answer (hosting state), so wait
    // for all of them rather than asserting on the first frame.
    term.wait_for("every row of the merged menu", STEP, |s| {
        ROWS.iter().all(|row| s.contains(row))
    })
    .unwrap_or_else(|e| {
        let text = term.text();
        let missing: Vec<_> = ROWS.iter().filter(|r| !text.contains(*r)).collect();
        panic!("{e}\nmissing: {missing:?}\n{text}")
    });
}

/// Right-clicking the `+1` overflow pill reaches the hidden ref: it has
/// its own submenu, and its delete row works.
#[test]
fn right_clicking_the_overflow_pill_reaches_the_hidden_ref() {
    let repo = fixture(false);
    let (mut term, _c) = launch(&repo, "pty-pill-overflow");
    let (_, row) = term.wait_for_text("newer head commit", STEP).unwrap();
    let (col, _) = term
        .screen()
        .find_all("+1")
        .into_iter()
        .find(|(_, r)| *r == row)
        .expect("the +1 overflow badge on the newest commit");
    term.right_click(col, row).unwrap();
    term.wait_for("the menu with the hidden ref's submenu", STEP, |s| {
        s.contains("hello \u{203A}")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Keyboard: the last row is the hidden ref's submenu; drill in.
    term.send_keys("End").unwrap();
    term.send_keys("Enter").unwrap();
    let menu = term
        .wait_for("the hidden ref's own rows", STEP, |s| {
            s.contains("Delete hello")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = menu.find("Delete hello").unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("the confirm prompt", STEP, |s| {
        s.contains("Delete branch 'hello'?")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("y").unwrap();
    wait_branch_gone(&repo, "hello");
}

/// A commit row's menu has one submenu per ref on the commit.
#[test]
fn the_commit_menu_has_one_submenu_per_ref() {
    let repo = fixture(false);
    let current = repo.git(&["branch", "--show-current"]);
    let (mut term, _c) = launch(&repo, "pty-commit-refs");
    let (col, row) = term.wait_for_text("newer head commit", STEP).unwrap();
    term.right_click(col + 1, row).unwrap();
    let menu = term
        .wait_for("the commit menu with per-ref submenus", STEP, |s| {
            s.contains("Check out this commit") && s.contains("hello \u{203A}")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        menu.contains(&format!("{current} \u{203A}")),
        "the checked-out branch has a submenu too"
    );
    assert!(menu.contains("Create annotated tag here..."));
}
