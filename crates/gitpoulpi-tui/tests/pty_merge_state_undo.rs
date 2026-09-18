//! A merge stopped on three conflicted files: resolve one, unstage it from
//! the Status pane, and the other two must still be conflicted (a click on
//! them opens the merge editor, not a Diff of raw markers) with Undo
//! offered; and aborting the merge can be undone with Ctrl+Z, bringing the
//! whole stopped merge back.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::ConflictFixture;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

fn launch(repo: &Path, session: &str) -> (Terminal, tempfile::TempDir) {
    let (term, config) = common::launch(repo)
        .session(session)
        .size(200, 50)
        .no_exit_pause()
        .ready_when("the conflicted files in the Status pane", |s| {
            s.contains("Unstaged") && s.contains("Merge in progress")
        })
        .mouse_capture()
        .start();
    common::wait_for_viewer(config.path(), session);
    // Past the "the user just pressed a key" deferral window.
    std::thread::sleep(Duration::from_millis(1700));
    (term, config)
}

/// `ResolveConflict` (whole file, given text) sent to the session's daemon.
fn daemon_resolve(config: &tempfile::TempDir, session: &str, fx: &ConflictFixture, path: &str) {
    use gitpoulpi_core::domain::OperationKind;
    let port_file = config
        .path()
        .join("sessions")
        .join(format!("{session}.port"));
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let mut client = gitpoulpi_ipc::Client::connect(&port_file)
            .await
            .expect("connect to the session's daemon");
        let response = client
            .call(&gitpoulpi_ipc::Request {
                id: 1,
                session: session.into(),
                repo: Some(fx.repo_ref().id),
                command: OperationKind::ResolveConflict,
                args: serde_json::json!({
                    "path": path,
                    "resolution": { "UseContent": "resolved by hand\n" },
                }),
            })
            .await
            .expect("call");
        response.result.expect("the resolution is accepted");
    });
}

fn unmerged(fx: &ConflictFixture) -> Vec<String> {
    common::git(fx.root(), &["diff", "--name-only", "--diff-filter=U"])
        .lines()
        .map(str::to_string)
        .collect()
}

#[test]
fn unstaging_a_resolved_file_keeps_the_other_conflicts_and_undo_is_offered() {
    let fx = ConflictFixture::multi_conflict();
    let (mut term, config) = launch(fx.root(), "pty-merge-unstage");
    daemon_resolve(&config, "pty-merge-unstage", &fx, "a.txt");
    term.wait_for("a.txt staged, two files left conflicted", STEP, |s| {
        s.contains("Staged Files (1)") && s.contains("Unstage all changes")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Unstage it with the Staged section's button.
    let (c, r) = term.wait_for_text("Unstage all changes", STEP).unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("nothing staged any more", STEP, |s| {
        !s.contains("Staged Files (1)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The two files that were never resolved are still conflicted: git
    // says so, and the Status pane still marks them `!` (not a pencil).
    assert_eq!(unmerged(&fx), ["b.txt", "docs/c.txt"]);
    let screen = term.screen();
    let row_of = |name: &str| {
        screen
            .find_all(name)
            .into_iter()
            .map(|(_, r)| screen.line(r))
            .find(|line| line.contains(name))
            .unwrap_or_else(|| panic!("{name} not on screen\n{}", term.text()))
    };
    assert!(
        row_of("b.txt").contains('!'),
        "b.txt still marked conflicted: {}",
        row_of("b.txt")
    );

    // A click on a conflicted row opens the merge editor, not a raw Diff.
    let (c, r) = term.wait_for_text("b.txt", STEP).unwrap();
    term.click(c, r).unwrap();
    term.wait_for("the merge editor on b.txt", STEP, |s| {
        s.contains("Conflict 1 /")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("the editor closed", STEP, |s| !s.contains("Conflict 1 /"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Undo takes the unstaging back: a.txt is staged again.
    term.send_keys("C-z").unwrap();
    term.wait_for("a.txt staged again", STEP, |_| {
        common::git(fx.root(), &["diff", "--cached", "--name-only"]).contains("a.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(unmerged(&fx), ["b.txt", "docs/c.txt"]);
    term.stop().unwrap();
}

#[test]
fn undoing_an_abort_brings_the_whole_merge_back() {
    let fx = ConflictFixture::multi_conflict();
    let merge_head = common::git_trimmed(fx.root(), &["rev-parse", "MERGE_HEAD"]);
    let markers_b = fx.worktree_bytes("b.txt");
    let (mut term, config) = launch(fx.root(), "pty-merge-abort-undo");
    daemon_resolve(&config, "pty-merge-abort-undo", &fx, "a.txt");
    term.wait_for("a.txt staged", STEP, |s| s.contains("Staged Files (1)"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    let (c, r) = term.wait_for_text("Abort merge", STEP).unwrap();
    term.click(c + 2, r).unwrap();
    term.wait_for("the confirmation", STEP, |s| {
        s.contains("Abort the merge in progress?")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("y").unwrap();
    term.wait_for("the merge gone", STEP, |s| !s.contains("Merge in progress"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(common::git_try(fx.root(), &["rev-parse", "-q", "--verify", "MERGE_HEAD"]).is_none());

    // Undo: the merge, its conflicts and the resolved file come back.
    term.send_keys("C-z").unwrap();
    term.wait_for("the merge in progress again", STEP, |s| {
        s.contains("Merge in progress")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(
        common::git_trimmed(fx.root(), &["rev-parse", "MERGE_HEAD"]),
        merge_head
    );
    assert_eq!(unmerged(&fx), ["b.txt", "docs/c.txt"]);
    assert_eq!(fx.worktree_bytes("b.txt"), markers_b);
    assert!(common::git(fx.root(), &["diff", "--cached", "--name-only"]).contains("a.txt"));
    term.stop().unwrap();
}
