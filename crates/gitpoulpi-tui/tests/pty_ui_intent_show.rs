//! An agent's view intents (`ui show commit|ref|search|pr|issue|wip`) end
//! to end: the real binary and its embedded daemon in a
//! pty, and a plain IPC client standing in for the `gitpoulpi ui ...`
//! commands (`common::ui_show`). Hosting is only ever the unlinked case here
//! (no token, no network).
//!
//! Every wait names the state it waits for.

mod common;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use common::{git_ok, git_trimmed};
use gitpoulpi_core::domain::{Oid, RepoId};
use gitpoulpi_ipc::ui::{HostingItem, PaneId, UiIntent, UiOutcome, UiState};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const SIDEBAR_RIGHT: u16 = 40;

struct Rig {
    term: Terminal,
    config: tempfile::TempDir,
    session: String,
    root: PathBuf,
    _keep: Box<dyn std::any::Any>,
}

fn launch(
    session: &str,
    root: &Path,
    keep: Box<dyn std::any::Any>,
    config: &str,
    ready: &'static str,
) -> Rig {
    let (term, config) = common::launch(root)
        .session(session)
        .config(config)
        .size(200, 50)
        .no_exit_pause()
        .ready(ready)
        .start();
    common::wait_for_viewer(config.path(), session);
    // Past the "the user just pressed a key" deferral window.
    std::thread::sleep(Duration::from_millis(1700));
    Rig {
        term,
        config,
        session: session.into(),
        root: root.to_path_buf(),
        _keep: keep,
    }
}

impl Rig {
    fn repo(&self) -> RepoId {
        RepoId::from_path(&self.root)
    }

    fn show(&self, intent: UiIntent) -> UiOutcome {
        common::ui_show(self.config.path(), &self.session, intent)
    }

    fn state(&self) -> UiState {
        common::ui_state(self.config.path(), &self.session)
    }

    fn wait_for(&self, what: &str, f: impl Fn(&str) -> bool) {
        self.term
            .wait_for(what, STEP, |s| f(&s.text()))
            .unwrap_or_else(|e| panic!("{e}"));
    }

    fn selected_commit(&self) -> String {
        self.state().log.expect("log")["selected_commit"]
            .as_str()
            .unwrap_or_default()
            .to_string()
    }
}

fn oid(hex: &str) -> Oid {
    Oid::from_hex(hex).expect("a full object id")
}

fn applied(out: UiOutcome) -> UiState {
    match out {
        UiOutcome::Applied { state } => state,
        other => panic!("expected Applied, got {other:?}"),
    }
}

/// `main` with `n` commits `commit-0001`..`commit-NNNN`, built by one
/// `git fast-import`.
fn big_repo(n: usize) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("temp repo");
    git_ok(dir.path(), &["init", "-q", "-b", "main"]);
    let mut stream = String::new();
    for i in 1..=n {
        let msg = format!("commit-{i:04}");
        stream.push_str(&format!(
            "commit refs/heads/main\nmark :{i}\ncommitter T <t@example.com> {} +0000\ndata {}\n{msg}\n",
            1_600_000_000 + i * 60,
            msg.len()
        ));
        if i > 1 {
            stream.push_str(&format!("from :{}\n", i - 1));
        }
        stream.push_str(&format!(
            "M 100644 inline f.txt\ndata {}\n{i}\n\n",
            i.to_string().len()
        ));
    }
    let mut child = Command::new("git")
        .args(["fast-import", "--quiet"])
        .current_dir(dir.path())
        .stdin(Stdio::piped())
        .spawn()
        .expect("spawn fast-import");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stream.as_bytes())
        .unwrap();
    assert!(child.wait().unwrap().success());
    git_ok(dir.path(), &["reset", "-q", "--hard", "main"]);
    dir
}

/// A page size of 100: the Log starts with 100 commits and pages 100, 200,
/// 400 in.
const PAGED: &str =
    "[general]\ninitial_commits_in_graph = 100\nlazy_load_commits_in_graph = true\n\
                     commits_per_lazy_page = 100\nmax_commits_in_graph = 1000\n";

#[test]
fn show_commit_pages_in_the_450th_commit_and_selects_its_row() {
    let repo = big_repo(600);
    let wanted = git_trimmed(repo.path(), &["rev-parse", "main~150"]);
    assert_eq!(
        git_trimmed(repo.path(), &["log", "-1", "--format=%s", &wanted]),
        "commit-0450"
    );
    let root = repo.path().to_path_buf();
    let rig = launch(
        "pty-ui-show-paging",
        &root,
        Box::new(repo),
        PAGED,
        "commit-0600",
    );
    assert!(
        !rig.term.screen().contains("commit-0450"),
        "not loaded yet: the Log holds the newest 100"
    );
    let state = applied(rig.show(UiIntent::ShowCommit {
        repo: rig.repo(),
        commit: oid(&wanted),
        also: vec![],
        files: false,
        focus: None,
    }));
    assert_eq!(
        state.log.as_ref().unwrap()["selected_commit"],
        wanted.as_str(),
        "{state:?}"
    );
    // The Log row and the detail panel both name it.
    rig.wait_for("the row and the detail summary", |t| {
        t.matches("commit-0450").count() >= 2
    });
    assert_eq!(rig.selected_commit(), wanted);
}

/// `main`, plus `feature/deep/needle-branch` and `side` one commit ahead.
fn branchy() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial")
        .unwrap()
        .commit_file("b.txt", "two\n", "second")
        .unwrap()
        .build();
    let root = repo.root();
    for (branch, file, msg) in [
        ("feature/deep/needle-branch", "n.txt", "needle work"),
        ("side", "c.txt", "side work"),
    ] {
        git_ok(root, &["checkout", "-q", "-b", branch]);
        std::fs::write(root.join(file), "x\n").unwrap();
        git_ok(root, &["add", file]);
        git_ok(root, &["commit", "-q", "-m", msg]);
        git_ok(root, &["checkout", "-q", "main"]);
    }
    repo
}

fn in_sidebar(term: &Terminal, text: &str) -> bool {
    term.screen()
        .find_all(text)
        .into_iter()
        .any(|(col, _)| col < SIDEBAR_RIGHT)
}

#[test]
fn show_ref_unfolds_its_folder_and_selects_the_row_and_the_commit() {
    let repo = branchy();
    let tip = git_trimmed(repo.root(), &["rev-parse", "feature/deep/needle-branch"]);
    let root = repo.root().to_path_buf();
    let mut rig = launch("pty-ui-show-ref", &root, Box::new(repo), "", "needle work");
    // Fold the folder with the mouse: its row is `feature/`.
    let (col, row) = rig
        .term
        .screen()
        .find_all("feature/")
        .into_iter()
        .find(|(c, _)| *c < SIDEBAR_RIGHT)
        .expect("the feature/ folder row");
    rig.term.click(col + 1, row).unwrap();
    rig.term
        .wait_for("the folder folded", STEP, |s| {
            !s.find_all("needle-branch")
                .into_iter()
                .any(|(c, _)| c < SIDEBAR_RIGHT)
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(!in_sidebar(&rig.term, "needle-branch"));
    std::thread::sleep(Duration::from_millis(1700));

    let state = applied(rig.show(UiIntent::ShowRef {
        repo: rig.repo(),
        name: "refs/heads/feature/deep/needle-branch".into(),
    }));
    assert_eq!(state.focus.as_deref(), Some("refs"), "{state:?}");
    assert_eq!(
        state.log.as_ref().unwrap()["selected_commit"],
        tip.as_str(),
        "the commit it points at is selected"
    );
    rig.term
        .wait_for("the leaf in the sidebar", STEP, |s| {
            s.find_all("needle-branch")
                .into_iter()
                .any(|(c, _)| c < SIDEBAR_RIGHT)
        })
        .unwrap_or_else(|e| panic!("{e}"));

    let out = rig.show(UiIntent::ShowRef {
        repo: rig.repo(),
        name: "no-such-branch".into(),
    });
    assert!(matches!(out, UiOutcome::NotFound { .. }), "{out:?}");
}

#[test]
fn show_commit_hidden_by_a_solo_filter_is_partial_and_never_unhidden() {
    let repo = branchy();
    let side = git_trimmed(repo.root(), &["rev-parse", "side"]);
    let root = repo.root().to_path_buf();
    let rig = launch("pty-ui-show-hidden", &root, Box::new(repo), "", "side work");
    // Solo `main` through the CLI, like the user would.
    let cli = PathBuf::from(env!("CARGO_BIN_EXE_gitpoulpi-tui")).with_file_name("gitpoulpi");
    if !cli.exists() {
        let status = Command::new(env!("CARGO"))
            .args(["build", "-p", "gitpoulpi-cli"])
            .status()
            .expect("cargo build");
        assert!(status.success());
    }
    let out = Command::new(&cli)
        .env("GITPOULPI_CONFIG_DIR", rig.config.path())
        .arg("--json")
        .args(["--session", &rig.session])
        .args(["--repo", &root.to_string_lossy()])
        .args(["refs", "solo", "main"])
        .current_dir(&root)
        .output()
        .expect("run gitpoulpi");
    assert!(out.status.success(), "{:?}", out);
    rig.term
        .wait_for("only main's commits in the Log", STEP, |s| {
            !s.find_all("side work")
                .into_iter()
                .any(|(c, _)| (SIDEBAR_RIGHT..130).contains(&c))
        })
        .unwrap_or_else(|e| panic!("{e}"));
    std::thread::sleep(Duration::from_millis(1700));

    let before = rig.selected_commit();
    let out = rig.show(UiIntent::ShowCommit {
        repo: rig.repo(),
        commit: oid(&side),
        also: vec![],
        files: false,
        focus: None,
    });
    let UiOutcome::Partial { note, state } = out else {
        panic!("expected Partial, got {out:?}");
    };
    assert_eq!(note, "hidden_by_filter");
    assert_eq!(
        state.log.unwrap()["selected_commit"],
        before.as_str(),
        "the selection did not move"
    );
}

#[test]
fn show_wip_with_the_composer_focuses_the_summary_without_typing() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .write_worktree("a.txt", b"one\ntwo\n")
        .unwrap()
        .stage("a.txt")
        .unwrap()
        .build();
    let root = repo.root().to_path_buf();
    let mut rig = launch("pty-ui-show-wip", &root, Box::new(repo), "", "// WIP");
    let state = applied(rig.show(UiIntent::ShowWip {
        repo: rig.repo(),
        path: Some("a.txt".into()),
        staged: true,
        composer: true,
    }));
    assert_eq!(state.focus.as_deref(), Some("status"), "{state:?}");
    assert_eq!(state.right_panel.as_deref(), Some("status"));
    assert!(state.input_busy, "the composer owns the keyboard");
    // Wait until the composer's summary is painted focused (its "Summary"
    // placeholder is replaced by the caret) so the typed keys go to a field
    // that is really on screen, not to a frame still in flight.
    rig.wait_for("the focused, empty summary with its caret", |t| {
        !t.contains("Summary") && t.lines().any(|l| l.contains('_'))
    });
    // Nothing was typed for the agent: what the user types is all there is.
    // Count occurrences instead of asserting absence: a random temp path on
    // screen may itself contain "zq".
    let before = rig.term.screen().text().matches("zq").count();
    rig.term.send_text("zq").unwrap();
    rig.wait_for("the typed text in the summary", |t| {
        t.matches("zq").count() > before
    });
    // The caret sits in the composer's summary, right after what was typed.
    rig.wait_for("the caret in the summary field", |t| t.contains("zq_"));
}

#[test]
fn show_search_opens_the_find_bar_with_the_query_and_a_missing_path_is_not_found() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .commit_file("b.txt", "two\n", "second commit")
        .unwrap()
        .build();
    let second = git_trimmed(repo.root(), &["rev-parse", "HEAD"]);
    let root = repo.root().to_path_buf();
    let mut rig = launch(
        "pty-ui-show-search",
        &root,
        Box::new(repo),
        "",
        "second commit",
    );
    let state = applied(rig.show(UiIntent::ShowSearch {
        repo: rig.repo(),
        query: "initial".into(),
    }));
    assert!(
        state.input_busy,
        "the find bar owns the keyboard: {state:?}"
    );
    assert_ne!(rig.selected_commit(), second, "the first hit is selected");
    // The query is in the bar, not submitted: Esc closes it.
    rig.term.send_keys("Esc").unwrap();
    rig.wait_for("the bar closed", |_| !rig.state().input_busy);

    // No uncommitted changes: no WIP row to show.
    std::thread::sleep(Duration::from_millis(1700));
    let out = rig.show(UiIntent::ShowWip {
        repo: rig.repo(),
        path: None,
        staged: false,
        composer: false,
    });
    assert!(matches!(out, UiOutcome::NotFound { .. }), "{out:?}");

    // `also` extends the selection and `files` ticks "View all files".
    let first = git_trimmed(&root, &["rev-parse", "HEAD~1"]);
    std::thread::sleep(Duration::from_millis(1700));
    applied(rig.show(UiIntent::ShowCommit {
        repo: rig.repo(),
        commit: oid(&second),
        also: vec![oid(&first)],
        files: true,
        focus: None,
    }));
    assert_eq!(rig.selected_commit(), second);
    rig.wait_for("View all files ticked", |t| {
        t.contains("[x] View all files")
    });
    std::thread::sleep(Duration::from_millis(1700));

    // A commit that no ref reaches is not in the Log: an honest NotFound.
    let tree = git_trimmed(&root, &["rev-parse", "HEAD^{tree}"]);
    let lost = git_trimmed(&root, &["commit-tree", &tree, "-m", "dangling"]);
    let out = rig.show(UiIntent::ShowCommit {
        repo: rig.repo(),
        commit: oid(&lost),
        also: vec![],
        files: false,
        focus: Some(PaneId::Log),
    });
    assert!(matches!(out, UiOutcome::NotFound { .. }), "{out:?}");
}

#[test]
fn show_hosting_answers_hosting_not_linked_and_never_links() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .build();
    let root = repo.root().to_path_buf();
    let rig = launch(
        "pty-ui-show-pr",
        &root,
        Box::new(repo),
        "",
        "initial commit",
    );
    for item in [HostingItem::Pr(7), HostingItem::Issue(3)] {
        let out = rig.show(UiIntent::ShowHosting {
            repo: rig.repo(),
            item,
            view: Default::default(),
        });
        let UiOutcome::Refused { reason } = out else {
            panic!("expected Refused, got {out:?}");
        };
        assert!(reason.starts_with("hosting_not_linked"), "{reason}");
        std::thread::sleep(Duration::from_millis(500));
    }
}
