//! `general.auto_stash` in the real TUI: a checkout blocked by local
//! changes opens the "stash changes and continue?" dialog when the
//! setting is `ask` (the default), and the answer is carried out by the
//! daemon -- stash, checkout, pop. Driven through a pty by `xtask::pty`.
//! Rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps.

mod common;

use common::git_trimmed as git;
use common::git_try;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);
const SIDEBAR_RIGHT: u16 = 40;

/// `main` and `topic` differ on line 1 of a.txt; the working tree edits
/// line 5, so checking `topic` out is blocked and the stash pops cleanly.
fn blocked_fixture() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "1\n2\n3\n4\n5\n", "base commit")
        .unwrap()
        .branch("topic")
        .unwrap()
        .checkout("topic")
        .unwrap()
        .commit_file("a.txt", "1 topic\n2\n3\n4\n5\n", "topic edit")
        .unwrap()
        .checkout("main")
        .unwrap()
        .build();
    std::fs::write(repo.root().join("a.txt"), "1\n2\n3\n4\n5 local\n").unwrap();
    repo
}

fn launch(repo: &Path, session: &str, config_toml: &str) -> (Terminal, tempfile::TempDir) {
    let (term, config) = common::launch(repo)
        .session(session)
        .config(config_toml)
        .size(200, 50)
        .ready_when("the fixture's log", |s| {
            s.contains("topic edit") && s.contains("base commit")
        })
        .start();
    (term, config)
}

fn sidebar_topic(screen: &Screen) -> Option<(u16, u16)> {
    screen
        .find_all("topic")
        .into_iter()
        .find(|(col, _)| *col < SIDEBAR_RIGHT)
}

fn double_click_topic(term: &mut Terminal) {
    let screen = term
        .wait_for("`topic` in the Refs sidebar", STEP, |s| {
            sidebar_topic(s).is_some()
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = sidebar_topic(&screen).unwrap();
    term.double_click(col + 1, row).unwrap();
}

#[test]
fn a_blocked_checkout_asks_and_yes_stashes_checks_out_and_pops() {
    let repo = blocked_fixture();
    let (mut term, _config) = launch(repo.root(), "pty-autostash-ask", "");

    double_click_topic(&mut term);
    let screen = term
        .wait_for("the auto-stash dialog", STEP, |s| {
            s.contains("Local changes block the checkout")
                && s.contains("Stash changes and continue")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    // Nothing happened yet.
    assert_eq!(
        git(repo.root(), &["rev-parse", "--abbrev-ref", "HEAD"]),
        "main"
    );

    let (col, row) = screen.find("Stash changes and continue").unwrap();
    term.click(col + 1, row).unwrap();
    term.wait_for(
        "the dialog gone, topic checked out, stash popped",
        STEP,
        |s| {
            !s.contains("Local changes block the checkout")
                && git_try(repo.root(), &["rev-parse", "--abbrev-ref", "HEAD"]).as_deref()
                    == Some("topic")
                && git_try(repo.root(), &["stash", "list"]).is_some_and(|l| l.is_empty())
                && std::fs::read_to_string(repo.root().join("a.txt"))
                    .is_ok_and(|t| t.contains("local"))
        },
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap(),
        "1 topic\n2\n3\n4\n5 local\n"
    );
    assert_eq!(git(repo.root(), &["stash", "list"]), "");
}

#[test]
fn canceling_the_dialog_changes_nothing_and_the_keyboard_answers_it_too() {
    let repo = blocked_fixture();
    let (mut term, _config) = launch(repo.root(), "pty-autostash-cancel", "");

    double_click_topic(&mut term);
    term.wait_for("the auto-stash dialog", STEP, |s| {
        s.contains("Local changes block the checkout")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Esc").unwrap();
    term.wait_for_text_to_go("Local changes block the checkout", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    // A double-click may have sent the request twice; the dialog must not
    // come back for the duplicate.
    term.wait_for_stable(Duration::from_secs(2), STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!term.text().contains("Local changes block the checkout"));
    assert_eq!(
        git(repo.root(), &["rev-parse", "--abbrev-ref", "HEAD"]),
        "main"
    );
    assert_eq!(git(repo.root(), &["stash", "list"]), "");

    // The keyboard twin: open it again, Enter takes the first entry.
    double_click_topic(&mut term);
    term.wait_for("the auto-stash dialog again", STEP, |s| {
        s.contains("Local changes block the checkout")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("topic checked out", STEP, |s| {
        !s.contains("Local changes block the checkout")
            && git_try(repo.root(), &["rev-parse", "--abbrev-ref", "HEAD"]).as_deref()
                == Some("topic")
            && git_try(repo.root(), &["stash", "list"]).is_some_and(|l| l.is_empty())
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn auto_stash_always_skips_the_dialog_and_never_refuses() {
    let repo = blocked_fixture();
    let (mut term, _config) = launch(
        repo.root(),
        "pty-autostash-always",
        "[general]\nauto_stash = \"always\"\n",
    );
    double_click_topic(&mut term);
    term.wait_for("topic checked out and the stash popped", STEP, |s| {
        !s.contains("Local changes block the checkout")
            && git_try(repo.root(), &["rev-parse", "--abbrev-ref", "HEAD"]).as_deref()
                == Some("topic")
            && git_try(repo.root(), &["stash", "list"]).is_some_and(|l| l.is_empty())
            && std::fs::read_to_string(repo.root().join("a.txt")).is_ok_and(|t| t.contains("local"))
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap(),
        "1 topic\n2\n3\n4\n5 local\n"
    );
    term.stop().ok();

    let repo = blocked_fixture();
    let (mut term, _config) = launch(
        repo.root(),
        "pty-autostash-never",
        "[general]\nauto_stash = \"never\"\n",
    );
    double_click_topic(&mut term);
    term.wait_for("the plain refusal, no dialog", STEP, |s| {
        s.contains("prevent") || s.contains("overwritten") || s.contains("error")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(!term.text().contains("Local changes block the checkout"));
    assert_eq!(
        git(repo.root(), &["rev-parse", "--abbrev-ref", "HEAD"]),
        "main"
    );
}

/// `origin/main` is one commit ahead and edits line 1 of a.txt; the working
/// tree edits line 5, so the pull is blocked and the stash pops cleanly.
fn blocked_pull_fixture() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "1\n2\n3\n4\n5\n", "base commit")
        .unwrap()
        .commit_file("a.txt", "1 upstream\n2\n3\n4\n5\n", "upstream edit")
        .unwrap()
        .with_origin()
        .unwrap()
        .build();
    repo.git(&["push", "-u", "origin", "main"]);
    repo.git(&["reset", "--hard", "HEAD~1"]);
    std::fs::write(repo.root().join("a.txt"), "1\n2\n3\n4\n5 local\n").unwrap();
    repo
}

/// The dialog's "always" answer: the pull goes through (stash, pull, pop)
/// and `general.auto_stash = "always"` is written, so the Settings row and
/// the next blocked operation follow it.
#[test]
fn a_blocked_pull_asks_and_always_pulls_and_remembers_the_answer() {
    let repo = blocked_pull_fixture();
    let (mut term, config) = common::launch(repo.root())
        .session("pty-autostash-pull-always")
        .size(200, 50)
        .ready_when("the fixture's log", |s| {
            s.contains("upstream edit") && s.contains("base commit")
        })
        .start();

    let screen = term.screen();
    let (col, row) = screen.find("Pull").expect("the toolbar's Pull button");
    term.click(col + 1, row).unwrap();
    let screen = term
        .wait_for("the auto-stash dialog for the pull", STEP, |s| {
            s.contains("Local changes block the pull") && s.contains("Always stash automatically")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let upstream = git(repo.root(), &["rev-parse", "origin/main"]);
    assert_ne!(git(repo.root(), &["rev-parse", "HEAD"]), upstream);

    let (col, row) = screen.find("Always stash automatically").unwrap();
    term.click(col + 1, row).unwrap();
    term.wait_for("pulled and the stash popped", STEP, |s| {
        !s.contains("Local changes block the pull")
            && git_try(repo.root(), &["rev-parse", "HEAD"]).as_deref() == Some(upstream.as_str())
            && git_try(repo.root(), &["stash", "list"]).is_some_and(|l| l.is_empty())
            && std::fs::read_to_string(repo.root().join("a.txt")).is_ok_and(|t| t.contains("local"))
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap(),
        "1 upstream\n2\n3\n4\n5 local\n"
    );
    let saved = std::fs::read_to_string(config.path().join("config.toml")).unwrap_or_default();
    assert!(
        saved.contains("auto_stash = \"always\""),
        "the answer was not remembered:\n{saved}"
    );
}

/// The maintainer's trap: an auto-stashed pull whose pop conflicts. The
/// conflicted file must be on screen (WIP selected), the banner must offer
/// a way out that keeps the work - *Finish*, refused while a conflict is
/// left, then: changes kept (unstaged), stash dropped, banner gone.
#[test]
fn a_conflicted_auto_stash_pop_shows_the_conflict_and_finish_keeps_the_work() {
    let repo = blocked_pull_fixture();
    // Line 1 on both sides: the pop after the pull conflicts.
    std::fs::write(repo.root().join("a.txt"), "1 mine\n2\n3\n4\n5\n").unwrap();
    let (mut term, _config) = common::launch(repo.root())
        .session("pty-autostash-pop-conflict")
        .config("[general]\nauto_stash = \"always\"\n")
        .size(200, 50)
        .ready_when("the fixture's log", |s| {
            s.contains("upstream edit") && s.contains("base commit")
        })
        .start();
    let upstream = git(repo.root(), &["rev-parse", "origin/main"]);

    // On a commit, not on WIP: the right pane shows its details, where the
    // conflict would not be seen.
    let screen = term.screen();
    let (col, row) = screen.find("base commit").unwrap();
    term.click(col + 1, row).unwrap();
    let screen = term
        .wait_for("the commit's details", STEP, |s| {
            !s.contains("Unstaged Files")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = screen.find("Pull").expect("the toolbar's Pull button");
    term.click(col + 1, row).unwrap();
    let screen = term
        .wait_for(
            "the conflicted banner, Finish, and the file in WIP",
            STEP,
            |s| s.contains("Keep result") && s.contains("a.txt") && s.contains("Unstaged Files"),
        )
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(git(repo.root(), &["rev-parse", "HEAD"]), upstream, "pulled");
    assert_eq!(git(repo.root(), &["stash", "list"]).lines().count(), 1);

    // A conflict is left: Finish refuses and drops nothing.
    let (col, row) = screen.find("Keep result").unwrap();
    term.click(col + 1, row).unwrap();
    term.wait_for("Finish refused", STEP, |s| {
        s.contains("resolve the conflicted files first")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(git(repo.root(), &["stash", "list"]).lines().count(), 1);

    // Resolved (here outside the app): Finish keeps the tree, drops the stash.
    std::fs::write(repo.root().join("a.txt"), "1 upstream mine\n2\n3\n4\n5\n").unwrap();
    git(repo.root(), &["add", "a.txt"]);
    term.wait_for("the conflict gone from the list", STEP, |s| {
        !s.contains("resolve the conflicted files first")
            && git_try(repo.root(), &["diff", "--name-only", "--diff-filter=U"])
                .is_some_and(|l| l.is_empty())
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let screen = term
        .wait_for_stable(Duration::from_secs(3), STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = screen.find("Keep result").unwrap();
    term.click(col + 1, row).unwrap();
    term.wait_for("banner gone, stash dropped", STEP, |s| {
        !s.contains("Keep result")
            && git_try(repo.root(), &["stash", "list"]).is_some_and(|l| l.is_empty())
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        std::fs::read_to_string(repo.root().join("a.txt")).unwrap(),
        "1 upstream mine\n2\n3\n4\n5\n",
        "the resolved work is kept"
    );
    assert_eq!(
        git(repo.root(), &["diff", "--cached", "--name-only"]),
        "",
        "unstaged, as a clean pop leaves it"
    );
    assert_eq!(git(repo.root(), &["diff", "--name-only"]), "a.txt");
    let state =
        std::fs::read_to_string(repo.state_dir().join("repo-state.toml")).unwrap_or_default();
    assert!(
        !state.contains("pending_auto_stash"),
        "record cleared:\n{state}"
    );
}
