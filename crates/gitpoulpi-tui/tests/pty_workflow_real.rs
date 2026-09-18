//! Branching workflow actions (plan `branching-workflows.md`, slices W2 + W3)
//! through a pty on the REAL path: the dialogs talk to the real daemon, which
//! runs real `git` on a fixture repository. No fake (`GITPOULPI_FLOW_FAKE`
//! stays unset), no network.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps, and every wait describes the whole state it
//! is waiting for.

mod common;

use std::time::{Duration, Instant};

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Color, Terminal};

const STEP: Duration = Duration::from_secs(30);

/// Git Flow with a squash finish for features (so the finish is one commit
/// on develop) and the usual two-target hotfix.
const WORKFLOW: &str = r#"template = "git-flow"

[[base]]
name = "main"
names = ["master"]
protected = true

[[base]]
name = "develop"
names = ["dev"]
parent = "main"
merge = "merge"

[[topic]]
id = "feature"
label = "Feature"
pattern = "feature/{slug}"
parent = "develop"
finish = [{ into = "develop", merge = "squash" }]
delete_after_finish = true

[[topic]]
id = "hotfix"
label = "Hotfix"
pattern = "hotfix/{version}"
parent = "main"
finish = [{ into = "main", merge = "merge", tag = "v{version}" }, { into = "develop", merge = "merge" }]
delete_after_finish = true
"#;

fn fixture() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .tag("v1.4.2")
        .unwrap()
        .branch("develop")
        .unwrap()
        .build();
    std::fs::create_dir_all(repo.state_dir()).unwrap();
    std::fs::write(repo.state_dir().join("workflow.toml"), WORKFLOW).unwrap();
    repo
}

fn launch(repo: &FixtureRepo, session: &str) -> Terminal {
    let config = tempfile::tempdir().expect("temp config dir");
    std::fs::write(
        config.path().join("config.toml"),
        "[ui]\ngraph_renderer = \"text\"\n",
    )
    .unwrap();
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(200, 60)
        .no_exit_pause()
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GITPOULPI_NO_BROWSER", "1")
        // The browser stub: every opened URL is appended to this file.
        .env("GITPOULPI_OPEN_LOG", opened_log(repo))
        .mouse_capture()
        .spawn();
    // The config dir lives as long as the test: leak it with the terminal.
    std::mem::forget(config);
    term.wait_for("the Flow button", STEP, |s| s.contains("Flow \u{25BE}"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term
}

fn opened_log(repo: &FixtureRepo) -> std::path::PathBuf {
    repo.root().with_file_name("opened.log")
}

fn palette(term: &mut Terminal, query: &str, entry: &str) {
    term.send_keys("C-p").unwrap();
    term.wait_for_text("Command Palette", STEP).unwrap();
    term.send_text(query).unwrap();
    term.wait_for("the palette entry", STEP, |s| s.contains(entry))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
}

fn git(repo: &FixtureRepo, args: &[&str]) -> String {
    common::git_trimmed(repo.root(), args)
}

fn wait_until(what: &str, mut f: impl FnMut() -> bool) {
    let deadline = Instant::now() + xtask::pty::scaled(STEP);
    while !f() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// Commits `file` with `content` on the checked-out branch, like a user
/// would from a shell.
fn commit(repo: &FixtureRepo, file: &str, content: &str, message: &str) {
    std::fs::write(repo.root().join(file), content).unwrap();
    git(repo, &["add", file]);
    git(
        repo,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.com",
            "commit",
            "-m",
            message,
        ],
    );
}

fn current(repo: &FixtureRepo) -> String {
    git(repo, &["branch", "--show-current"])
}

/// Runs a Start dialog through its preview and run; `preview` is a step the
/// preview must list. Leaves the dialog closed.
fn run_dialog(term: &mut Terminal, preview: &str, done: &str) {
    term.wait_for("the step list", STEP, |s| {
        s.contains("This will:") && s.contains(preview)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the run done", STEP, |s| {
        s.contains("Done.") && s.contains(done)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Esc").unwrap();
    term.wait_for_text_to_go("Done.", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
}

fn button(term: &Terminal, label: &str) -> (u16, u16) {
    term.screen()
        .find_all(label)
        .into_iter()
        .min_by_key(|(_, row)| *row)
        .unwrap_or_else(|| panic!("no {label} button:\n{}", term.text()))
}

fn fg(term: &Terminal, label: &str) -> Option<Color> {
    let (col, row) = button(term, label);
    term.screen().cell(col, row).map(|c| c.fg)
}

#[test]
fn a_feature_starts_then_finishes_as_a_squash_and_one_ctrl_z_takes_it_all_back() {
    let repo = fixture();
    let mut term = launch(&repo, "pty-wfreal-feature");
    // The Redo button is grayed here: the colour Undo must differ from once
    // the journal holds an entry.
    let grayed = fg(&term, "Redo");

    palette(&mut term, "Start Feature", "Flow: Start Feature...");
    term.wait_for("the Start Feature form", STEP, |s| {
        s.contains("Slug") && s.contains("Start from")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("login").unwrap();
    term.wait_for("the name preview", STEP, |s| s.contains("feature/login"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    run_dialog(
        &mut term,
        "Create branch feature/login from develop",
        "[x] Check out feature/login",
    );
    wait_until("the new branch checked out", || {
        current(&repo) == "feature/login"
    });
    // The recorded parent is the type's parent.
    assert_eq!(
        git(&repo, &["config", "branch.feature/login.gitpoulpi-parent"]),
        "develop"
    );

    // Two commits on the branch, from a shell.
    commit(&repo, "login.txt", "a\n", "Add the form");
    commit(&repo, "login.txt", "a\nb\n", "Validate it");
    let develop_before = git(&repo, &["rev-parse", "develop"]);
    let head_before = git(&repo, &["rev-parse", "HEAD"]);

    palette(
        &mut term,
        "Finish Current",
        "Flow: Finish Current Branch...",
    );
    run_dialog(
        &mut term,
        "Squash feature/login into develop as one commit",
        "[x] Delete the local branch feature/login",
    );
    wait_until("develop checked out and the branch deleted", || {
        current(&repo) == "develop" && git(&repo, &["branch", "--list", "feature/login"]).is_empty()
    });
    // One squash commit on develop, no merge commit, the work is there.
    assert_eq!(git(&repo, &["rev-list", "--count", "develop"]), "2");
    assert_eq!(
        git(&repo, &["rev-list", "--merges", "--count", "develop"]),
        "0"
    );
    assert_eq!(git(&repo, &["show", "develop:login.txt"]), "a\nb");
    assert_eq!(
        git(&repo, &["status", "--porcelain", "--untracked-files=no"]),
        ""
    );

    // One Ctrl+Z takes the whole finish back: the branch returns at its
    // tip, develop where it was, HEAD on the branch.
    term.wait_for("Undo to light up", STEP, |_| fg(&term, "Undo") != grayed)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("C-z").unwrap();
    wait_until("the finish undone", || {
        current(&repo) == "feature/login"
            && git(&repo, &["rev-parse", "develop"]) == develop_before
            && git(&repo, &["rev-parse", "HEAD"]) == head_before
    });
    assert_eq!(
        git(&repo, &["status", "--porcelain", "--untracked-files=no"]),
        ""
    );
    // The entry reads well in the Undo history: "Flow finish <branch>".
    assert!(
        git(&repo, &["config", "branch.feature/login.gitpoulpi-parent"]) == "develop",
        "the recorded parent came back with the branch"
    );
    term.stop().unwrap();
}

#[test]
fn a_hotfix_finishes_into_main_with_its_tag_and_into_develop() {
    let repo = fixture();
    let mut term = launch(&repo, "pty-wfreal-hotfix");

    palette(&mut term, "Start Hotfix", "Flow: Start Hotfix...");
    term.wait_for("the Start Hotfix form with the next patch", STEP, |s| {
        s.contains("Version") && s.contains("1.4.3")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    run_dialog(
        &mut term,
        "Create branch hotfix/1.4.3 from main",
        "[x] Check out hotfix/1.4.3",
    );
    wait_until("the hotfix checked out", || {
        current(&repo) == "hotfix/1.4.3"
    });
    commit(&repo, "fix.txt", "fixed\n", "Fix the crash");

    palette(
        &mut term,
        "Finish Current",
        "Flow: Finish Current Branch...",
    );
    run_dialog(
        &mut term,
        "Tag main as v1.4.3",
        "[x] Delete the local branch hotfix/1.4.3",
    );
    wait_until("the hotfix finished", || {
        git(&repo, &["branch", "--list", "hotfix/1.4.3"]).is_empty()
            && !git(&repo, &["tag", "--list", "v1.4.3"]).is_empty()
    });
    // Two targets: both have the fix, main carries the tag.
    assert_eq!(git(&repo, &["show", "main:fix.txt"]), "fixed");
    assert_eq!(git(&repo, &["show", "develop:fix.txt"]), "fixed");
    assert_eq!(
        git(&repo, &["rev-parse", "v1.4.3^{commit}"]),
        git(&repo, &["rev-parse", "main"])
    );
    term.stop().unwrap();
}

#[test]
fn a_conflict_mid_finish_offers_continue_and_the_finish_completes_after_resolving() {
    let repo = fixture();
    // develop and the feature both change a.txt.
    git(&repo, &["checkout", "-q", "-b", "feature/clash", "develop"]);
    commit(&repo, "a.txt", "feature\n", "Feature edit");
    git(&repo, &["checkout", "-q", "develop"]);
    commit(&repo, "a.txt", "develop\n", "Develop edit");
    git(&repo, &["checkout", "-q", "feature/clash"]);
    let mut term = launch(&repo, "pty-wfreal-conflict");

    palette(
        &mut term,
        "Finish Current",
        "Flow: Finish Current Branch...",
    );
    term.wait_for("the step list", STEP, |s| s.contains("This will:"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the conflict with Continue and Abort", STEP, |s| {
        s.contains("Resolve the conflicts, then Continue")
            && s.contains("Continue")
            && s.contains("Abort")
            && s.contains("a.txt")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(current(&repo), "develop", "the merge stopped on develop");

    // Continue before resolving is refused and the dialog keeps both buttons.
    term.send_keys("Enter").unwrap();
    term.wait_for("the refusal, still stopped", STEP, |s| {
        s.contains("still in conflict") && s.contains("Abort")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // Resolve like a user would, then Continue.
    std::fs::write(repo.root().join("a.txt"), "resolved\n").unwrap();
    git(&repo, &["add", "a.txt"]);
    term.send_keys("Enter").unwrap();
    term.wait_for("the finish done", STEP, |s| s.contains("Done."))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    wait_until("the branch deleted and develop merged", || {
        git(&repo, &["branch", "--list", "feature/clash"]).is_empty()
    });
    assert_eq!(current(&repo), "develop");
    assert_eq!(git(&repo, &["show", "develop:a.txt"]), "resolved");
    assert_eq!(
        git(&repo, &["status", "--porcelain", "--untracked-files=no"]),
        ""
    );
    term.stop().unwrap();
}

#[test]
fn a_closed_dialog_is_reopened_by_the_continue_command_after_resolving_elsewhere() {
    let repo = fixture();
    git(&repo, &["checkout", "-q", "-b", "feature/clash", "develop"]);
    commit(&repo, "a.txt", "feature\n", "Feature edit");
    git(&repo, &["checkout", "-q", "develop"]);
    commit(&repo, "a.txt", "develop\n", "Develop edit");
    git(&repo, &["checkout", "-q", "feature/clash"]);
    let mut term = launch(&repo, "pty-wfreal-resume");

    palette(
        &mut term,
        "Finish Current",
        "Flow: Finish Current Branch...",
    );
    term.wait_for("the step list", STEP, |s| s.contains("This will:"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the conflict", STEP, |s| {
        s.contains("Resolve the conflicts, then Continue")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Close the dialog: the conflict tools are free to use.
    term.send_keys("Esc").unwrap();
    term.wait_for_text_to_go("Resolve the conflicts, then Continue", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    std::fs::write(repo.root().join("a.txt"), "resolved\n").unwrap();
    git(&repo, &["add", "a.txt"]);

    // The palette twin brings the stopped action back, on its failed step.
    palette(&mut term, "Continue or Abort", "Flow: Continue or Abort");
    term.wait_for("the stopped action", STEP, |s| {
        s.contains("Stopped workflow action") && s.contains("[!]")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the finish done", STEP, |s| s.contains("Done."))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    wait_until("the branch deleted", || {
        git(&repo, &["branch", "--list", "feature/clash"]).is_empty()
    });
    assert_eq!(git(&repo, &["show", "develop:a.txt"]), "resolved");
    term.stop().unwrap();
}

#[test]
fn a_merge_request_finish_without_a_linked_host_shows_the_compare_url() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .build();
    let bare = repo.remote_path().expect("the bare origin").to_path_buf();
    // The host is GitHub by name, the push goes to the local bare repository.
    git(
        &repo,
        &[
            "remote",
            "set-url",
            "origin",
            "https://github.com/acme/app.git",
        ],
    );
    git(
        &repo,
        &["config", "remote.origin.pushurl", bare.to_str().unwrap()],
    );
    git(
        &repo,
        &[
            "fetch",
            "-q",
            bare.to_str().unwrap(),
            "main:refs/remotes/origin/main",
        ],
    );
    git(&repo, &["checkout", "-q", "-b", "work/add-login"]);
    commit(&repo, "login.txt", "a\n", "Add the form");
    std::fs::create_dir_all(repo.state_dir()).unwrap();
    std::fs::write(
        repo.state_dir().join("workflow.toml"),
        r#"template = "github-flow"

[[base]]
name = "main"
protected = true

[[topic]]
id = "feature"
label = "Feature"
pattern = "work/{slug}"
parent = "main"
finish = [{ into = "main", merge = "squash" }]
finish_mode = "merge-request"
delete_after_finish = true
"#,
    )
    .unwrap();
    let mut term = launch(&repo, "pty-wfreal-compare");
    palette(
        &mut term,
        "Finish Current",
        "Flow: Finish Current Branch...",
    );
    term.wait_for("the step list", STEP, |s| {
        s.contains("This will:") && s.contains("Open a merge request from work/add-login into main")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    wait_until("the compare URL opened", || {
        std::fs::read_to_string(opened_log(&repo))
            .is_ok_and(|t| t.contains("github.com/acme/app") && t.contains("work"))
    });
    assert!(
        !repo
            .git_in_remote(&["branch", "--list", "work/add-login"])
            .is_empty(),
        "pushed first"
    );
    term.stop().unwrap();
}
