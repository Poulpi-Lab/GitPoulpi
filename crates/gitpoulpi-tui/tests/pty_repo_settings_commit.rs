//! Settings > Repository > Commit, through a pty: the real binary, daemon and
//! git, a local bare remote, no network.
//!
//! - "Push on every commit": the commit lands on the remote too, and a branch
//!   with no upstream says that nothing was pushed.
//! - "Bypass git hooks": a failing `pre-commit` hook blocks a commit,
//!   the setting starts the composer's Skip hooks checked, and the commit goes
//!   through; the palette twin flips the checkbox.
//!
//! No hard-coded coordinates: every position is found on the screen.

mod common;

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

/// A repository with an `origin` (a bare repository next to it) the branch
/// tracks, and one staged edit to commit.
fn with_upstream() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "pty-commit-initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .build();
    repo.git(&["push", "-q", "-u", "origin", "main"]);
    stage_edit(&repo);
    repo
}

fn without_upstream() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "pty-commit-initial")
        .unwrap()
        .build();
    stage_edit(&repo);
    repo
}

fn stage_edit(repo: &FixtureRepo) {
    std::fs::write(repo.root().join("a.txt"), "one\ntwo\n").unwrap();
    repo.git(&["add", "a.txt"]);
}

fn launch(repo: &Path, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session(session)
        .size(200, 60)
        .no_exit_pause()
        .ready_when("the fixture and its composer", |s| {
            s.contains("pty-commit-initial") && s.contains("Skip hooks")
        })
        .mouse_capture()
        .start()
}

fn open_page(term: &mut Terminal, title: &str) {
    let (c, r) = term
        .wait_for_text("\u{2699}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c, r).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(term, title);
}

fn close_settings(term: &mut Terminal) {
    common::retry_until(
        term,
        "Settings closed",
        6,
        Duration::from_secs(3),
        |t| t.send_keys("Escape").unwrap(),
        |s| !s.contains("Repo-specific settings"),
    );
}

/// The repository's own saved choices (the repo data directory).
fn saved(repo: &FixtureRepo) -> String {
    std::fs::read_to_string(repo.root().join(".git/gitpoulpi/repo-state.toml")).unwrap_or_default()
}

/// Types `summary` in the composer and presses Commit.
fn commit_through_the_composer(term: &mut Terminal, summary: &str) {
    let (c, r) = term
        .wait_for_text("Summary", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 1, r).unwrap();
    term.send_text(summary).unwrap();
    term.wait_for("the summary typed", STEP, |s| s.contains(summary))
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = term
        .wait_for_text("Commit changes to 1 file", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 1, r).expect("click Commit");
}

fn wait_for(term: &Terminal, what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = Instant::now() + STEP;
    while !ok() {
        assert!(Instant::now() < deadline, "never: {what}\n{}", term.text());
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn install_failing_pre_commit(repo: &FixtureRepo) {
    let hook = repo.root().join(".git/hooks/pre-commit");
    std::fs::create_dir_all(hook.parent().unwrap()).unwrap();
    common::write_sh_script(&hook, "#!/bin/sh\necho pty-hook-refused >&2\nexit 1\n");
}

/// The page lists both switches off; ticking "Push on every commit" saves
/// it, and the next commit is pushed to the upstream, so the remote has it.
#[test]
fn push_after_each_commit_pushes_the_commit_to_the_upstream() {
    let repo = with_upstream();
    let (mut term, _config) = launch(repo.root(), "pty-repo-commit-push");
    open_page(&mut term, "Commit");
    let screen = term
        .wait_for("the Commit page", Duration::from_secs(20), |s| {
            s.contains("Push on every commit") && s.contains("Bypass git hooks")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (_, row) = screen.find("Push on every commit").unwrap();
    assert!(screen.line(row).contains("[ ]"), "off by default");
    let (c, r) = screen.find("Push on every commit").unwrap();
    term.click(c, r).unwrap();
    term.wait_for("ticked and saved", STEP, |s| {
        s.line(row).contains("[x]") && saved(&repo).contains("push_after_commit = true")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}\n{}", saved(&repo), term.text()));
    close_settings(&mut term);

    let before = repo.git_in_remote(&["rev-parse", "main"]);
    commit_through_the_composer(&mut term, "feat: pushed along");
    wait_for(&term, "the remote has the commit", || {
        repo.git_in_remote(&["log", "-1", "--format=%s", "main"])
            .trim()
            == "feat: pushed along"
    });
    assert_ne!(repo.git_in_remote(&["rev-parse", "main"]), before);
    assert_eq!(
        repo.git(&["rev-parse", "HEAD"]),
        repo.git_in_remote(&["rev-parse", "main"]),
        "the same commit"
    );
    let _ = term.stop();
}

/// A branch with no upstream is committed but not pushed, and the notice says
/// so (the first push of a new branch stays the Push button's).
#[test]
fn a_branch_without_an_upstream_is_committed_and_the_notice_says_nothing_was_pushed() {
    let repo = without_upstream();
    let state = repo.root().join(".git/gitpoulpi");
    std::fs::create_dir_all(&state).unwrap();
    std::fs::write(
        state.join("repo-state.toml"),
        "[preferences]\npush_after_commit = true\n",
    )
    .unwrap();
    let (mut term, _config) = launch(repo.root(), "pty-repo-commit-nopush");
    commit_through_the_composer(&mut term, "feat: local only");
    term.wait_for("the notice", STEP, |s| {
        s.contains("nothing was pushed because this branch has no upstream")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(
        repo.git(&["log", "-1", "--format=%s"]).trim(),
        "feat: local only"
    );
    let _ = term.stop();
}

/// A failing `pre-commit` hook blocks the commit; "Bypass git hooks"
/// starts the composer's Skip hooks checked and the commit then goes through.
/// The palette's "Toggle Skip Hooks" flips the checkbox like the click.
#[test]
fn the_skip_hooks_default_starts_the_composer_checked_and_the_commit_passes_the_hook() {
    let repo = without_upstream();
    install_failing_pre_commit(&repo);
    let (mut term, _config) = launch(repo.root(), "pty-repo-commit-hooks");
    let screen = term.screen();
    assert!(screen.contains("[ ] Skip hooks"), "off by default");

    // The hook blocks a commit made with the option off.
    commit_through_the_composer(&mut term, "feat: blocked");
    term.wait_for("the hook's refusal", STEP, |s| {
        s.contains("pty-hook-refused")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(
        repo.git(&["log", "-1", "--format=%s"]).trim(),
        "pty-commit-initial",
        "nothing was committed"
    );

    // Esc closes the failure card (it would cover the Commit button).
    term.send_keys("Esc").unwrap();
    term.wait_for("the card gone", STEP, |s| !s.contains("Commit failed"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The palette twin of the checkbox.
    term.send_keys("C-p").unwrap();
    term.send_text("toggle skip hooks").unwrap();
    term.wait_for("the palette entry", STEP, |s| {
        s.contains("Toggle Skip Hooks")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the checkbox ticked", STEP, |s| {
        s.contains("[x] Skip hooks")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // A click on it unticks it again.
    let (c, r) = term
        .wait_for_text("[x] Skip hooks", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    term.click(c + 4, r).unwrap();
    term.wait_for("the checkbox unticked", STEP, |s| {
        s.contains("[ ] Skip hooks")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The setting makes it the default.
    open_page(&mut term, "Commit");
    let screen = term
        .wait_for("the Commit page", Duration::from_secs(20), |s| {
            s.contains("Bypass git hooks")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = screen.find("Bypass git hooks").unwrap();
    term.click(c, r).unwrap();
    term.wait_for("saved", STEP, |s| {
        s.line(r).contains("[x]") && saved(&repo).contains("skip_hooks_by_default = true")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}\n{}", saved(&repo), term.text()));
    close_settings(&mut term);
    term.wait_for("the composer starts with Skip hooks checked", STEP, |s| {
        s.contains("[x] Skip hooks")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    commit_through_the_composer(&mut term, "feat: past the hook");
    wait_for(&term, "the commit past the hook", || {
        repo.git(&["log", "-1", "--format=%s"]).trim() == "feat: past the hook"
    });
    // Per commit: the option goes back to the default after it.
    term.wait_for("the composer ready again", STEP, |s| {
        s.contains("[x] Skip hooks")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let _ = term.stop();
}
