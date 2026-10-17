//! A push the host refuses (S23), end to end through a pty (`xtask::pty`):
//! the real binary, daemon and git. The remote is a local bare repository
//! whose hook says what GitHub says about a protected branch, while `origin`
//! is *named* a GitHub URL, so no network is involved and nothing is asked of
//! a host (the repository is not linked). The rejected row offers "How to get
//! through"; the overlay explains the three ways with the exact settings page
//! and does nothing by itself.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps, every wait describes the state it waits for.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

fn old_commit(repo: &FixtureRepo, message: &str) {
    repo.git(&[
        "-c",
        "user.name=Old Name",
        "-c",
        "user.email=old@example.com",
        "commit",
        "--allow-empty",
        "-q",
        "-m",
        message,
    ]);
}

/// A pushed `feature` whose remote refuses every push like a protected
/// GitHub branch, with `origin` named a GitHub URL.
fn fixture() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_origin()
        .unwrap()
        .commit_file("a.txt", "one\n", "base")
        .unwrap()
        .build();
    repo.git(&["push", "-q", "origin", "main"]);
    repo.git(&["checkout", "-q", "-b", "feature"]);
    old_commit(&repo, "first by old");
    old_commit(&repo, "second by old");
    repo.git(&["push", "-q", "origin", "feature"]);
    let bare = repo.remote_path().unwrap().to_path_buf();
    let hook = bare.join("hooks").join("pre-receive");
    std::fs::create_dir_all(hook.parent().unwrap()).unwrap();
    std::fs::write(
        &hook,
        "#!/bin/sh\necho 'remote: error: GH006: Protected branch update failed for refs/heads/feature.' >&2\nexit 1\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    repo.git(&[
        "config",
        "remote.origin.url",
        "https://github.com/octo/thing.git",
    ]);
    repo.git(&["config", "remote.origin.pushurl", &bare.to_string_lossy()]);
    repo
}

fn fill_identity_by_keys(term: &mut Terminal) {
    term.send_text("a").unwrap();
    term.wait_for("the Add menu", STEP, |s| s.contains("Replace an author"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the rule's fields", STEP, |s| {
        s.contains("Email is") && s.contains("New email")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    term.send_text("old@example.com").unwrap();
    term.send_keys("Enter").unwrap();
    for _ in 0..3 {
        term.send_keys("Down").unwrap();
    }
    term.send_keys("Enter").unwrap();
    term.send_text("new@example.org").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("both emails in the fields", STEP, |s| {
        s.contains("old@example.com") && s.contains("new@example.org")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn a_host_refusal_offers_the_three_ways_with_the_exact_settings_page() {
    let repo = fixture();
    let old_feature = repo.git_in_remote(&["rev-parse", "refs/heads/feature"]);
    let config = tempfile::tempdir().expect("temp config dir");
    let mut term = common::launch(repo.root())
        .session("pty-rewrite-host")
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| {
        s.contains("second by old") && s.contains("first by old")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-g").unwrap();
    term.send_text("H").unwrap();
    term.wait_for("the workbench", STEP, |s| {
        s.contains("Rewrite history") && s.contains("Add a rule to start")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    fill_identity_by_keys(&mut term);
    term.send_keys("F5").unwrap();
    term.wait_for("the summary", STEP, |s| s.contains("keep their id"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-s").unwrap();
    let screen = term
        .wait_for("the pushed-history question", STEP, |s| {
            s.contains("Rewrite pushed history?") && s.contains("Rewrite anyway")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let _ = screen;
    term.send_keys("1").unwrap();
    term.wait_for("the result panel with the push list", STEP, |s| {
        s.contains("Rewrote 2") && s.contains("Push rewritten branches")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // The push is sent; the host says no.
    term.send_keys("p").unwrap();
    let refused = term
        .wait_for("the refusal and the way out", STEP, |s| {
            s.contains("protected by the host") && s.contains("How to get through")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        repo.git_in_remote(&["rev-parse", "refs/heads/feature"]),
        old_feature,
        "the remote kept its history:\n{}",
        refused.text()
    );

    // By key (and by mouse: both reach the same overlay).
    term.send_keys("M-h").unwrap();
    let howto = term
        .wait_for("the three ways", STEP, |s| {
            s.contains("1. Temporarily lift the protection")
                && s.contains("2. Push the rewrite under a new branch name")
                && s.contains("3. Last resort: delete and recreate")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    // Way 1's two pages: the classic rules and the rulesets.
    for page in ["settings/branches", "settings/rules"] {
        assert!(
            howto.contains(&format!("https://github.com/octo/thing/{page}")),
            "the exact settings page {page}:\n{}",
            howto.text()
        );
    }
    assert!(
        howto.contains("not linked to a host"),
        "an unlinked repository says why nothing was read:\n{}",
        howto.text()
    );
    term.send_keys("Escape").unwrap();
    term.wait_for("the overlay closed", STEP, |s| {
        !s.contains("1. Temporarily lift the protection") && s.contains("Rewrite history")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        repo.git_in_remote(&["rev-parse", "refs/heads/feature"]),
        old_feature,
        "explaining changes nothing on the host"
    );
    let _ = term.stop();
}
