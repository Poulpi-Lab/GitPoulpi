//! `gitpoulpi flow start|finish|sync|promote|set-parent`: plan, confirmation,
//! run, conflicts, hooks, merge requests. Fixture repositories, deserialized
//! JSON; no network.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_core::workflow::{FlowOutcome, FlowPlan, FlowStatus, StepStatus, WorkflowReport};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;

fn config_dir() -> &'static Path {
    static DIR: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    DIR.get_or_init(|| tempfile::tempdir().unwrap()).path()
}

fn run_in(config: &Path, root: &Path, session: &str, args: &[&str]) -> std::process::Output {
    Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config)
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .env("GITPOULPI_TOKEN_H", "test-token")
        .arg("--json")
        .args(["--session", session])
        .args(["--repo", &root.to_string_lossy()])
        .args(args)
        .current_dir(root)
        .output()
        .unwrap()
}

fn run(root: &Path, session: &str, args: &[&str]) -> std::process::Output {
    run_in(config_dir(), root, session, args)
}

fn json_of(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "not JSON ({e}): {} / {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn ok(root: &Path, session: &str, args: &[&str]) -> Value {
    let output = run(root, session, args);
    assert!(
        output.status.success(),
        "{args:?} failed ({:?}): {} {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    json_of(&output)
}

fn error_of(root: &Path, session: &str, args: &[&str]) -> String {
    let output = run(root, session, args);
    assert!(!output.status.success(), "{args:?} unexpectedly worked");
    json_of(&output)["error"].as_str().unwrap().to_string()
}

/// main <- develop (+ feature/x with a commit), the git-flow workflow written
/// by `flow config init --write` and hidden from status.
fn repo(session: &str) -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "base\n", "init")
        .unwrap()
        .branch("develop")
        .unwrap()
        .branch("feature/x")
        .unwrap()
        .commit_file("f.txt", "f\n", "f1")
        .unwrap()
        .checkout("develop")
        .unwrap()
        .commit_file("d.txt", "d\n", "d1")
        .unwrap()
        .checkout("main")
        .unwrap()
        .build();
    init_workflow(&repo, session, "git-flow");
    repo
}

fn init_workflow(repo: &FixtureRepo, session: &str, template: &str) {
    let exclude = repo.root().join(".git").join("info").join("exclude");
    std::fs::create_dir_all(exclude.parent().unwrap()).unwrap();
    std::fs::write(exclude, "/.gitpoulpi/\n").unwrap();
    let out = ok(
        repo.root(),
        session,
        &["flow", "config", "init", "--template", template, "--write"],
    );
    assert_eq!(out["written"], true);
}

fn head(repo: &FixtureRepo) -> String {
    repo.git(&["rev-parse", "--abbrev-ref", "HEAD"])
        .trim()
        .to_string()
}

fn plan_of(value: &Value) -> FlowPlan {
    serde_json::from_value(value["plan"].clone()).unwrap()
}

#[test]
fn dry_run_prints_the_plan_as_json_and_changes_nothing() {
    let session = "cli-flow-dry";
    let repo = repo(session);
    let out = ok(
        repo.root(),
        session,
        &["flow", "start", "feature", "login-form", "--dry-run"],
    );
    let plan = plan_of(&out);
    assert!(plan.can_run(), "{:?}", plan.blockers);
    assert_eq!(plan.branch.as_deref(), Some("feature/login-form"));
    let ops: Vec<&str> = out["plan"]["steps"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["op"].as_str().unwrap())
        .collect();
    assert_eq!(ops, ["create_branch", "checkout", "record_parent"]);
    assert_eq!(head(&repo), "main");
    assert!(repo
        .try_git(&["rev-parse", "--verify", "-q", "feature/login-form"])
        .is_none());

    // A blocked plan is still printed, with a non-zero exit.
    let output = run(
        repo.root(),
        session,
        &["flow", "start", "nope", "x", "--dry-run"],
    );
    assert_eq!(output.status.code(), Some(1));
    let blocked = plan_of(&json_of(&output));
    assert!(!blocked.can_run());
}

#[test]
fn without_yes_a_script_gets_the_plan_and_exit_code_3() {
    let session = "cli-flow-confirm";
    let repo = repo(session);
    repo.git(&["checkout", "-q", "feature/x"]);
    let output = run(repo.root(), session, &["flow", "finish"]);
    assert_eq!(output.status.code(), Some(3), "{:?}", json_of(&output));
    let value = json_of(&output);
    let plan: FlowPlan =
        serde_json::from_value(value["needs_confirmation"]["plan"].clone()).unwrap();
    assert_eq!(plan.title, "Finish feature/x");
    assert!(value["needs_confirmation"]["prompt"].is_string());
    assert_eq!(head(&repo), "feature/x", "nothing changed");
    assert!(repo
        .try_git(&["rev-parse", "--verify", "-q", "feature/x"])
        .is_some());
}

#[test]
fn start_records_the_parent_shows_in_status_and_undo_takes_it_back() {
    let session = "cli-flow-start";
    let repo = repo(session);
    let out = ok(
        repo.root(),
        session,
        &["flow", "start", "feature", "slug=Login form", "--yes"],
    );
    let outcome: FlowOutcome = serde_json::from_value(out).unwrap();
    assert_eq!(outcome.status, FlowStatus::Done);
    assert_eq!(outcome.branch.as_deref(), Some("feature/login-form"));
    assert_eq!(head(&repo), "feature/login-form");

    let report: WorkflowReport =
        serde_json::from_value(ok(repo.root(), session, &["flow", "status"])).unwrap();
    let branch = report
        .branches
        .iter()
        .find(|b| b.name == "feature/login-form" && b.remote.is_none())
        .unwrap();
    let parent = branch.parent.as_ref().unwrap();
    assert_eq!(parent.name, "develop");
    assert_eq!(
        serde_json::to_value(parent.source).unwrap(),
        "recorded",
        "the recorded parent, not the type's"
    );

    ok(repo.root(), session, &["undo"]);
    assert_eq!(head(&repo), "main");
    assert!(repo
        .try_git(&["rev-parse", "--verify", "-q", "feature/login-form"])
        .is_none());
}

#[test]
fn finish_merges_and_deletes_and_undo_restores_it() {
    let session = "cli-flow-finish";
    let repo = repo(session);
    repo.git(&["checkout", "-q", "feature/x"]);
    let feature = repo.git(&["rev-parse", "feature/x"]).trim().to_string();
    let out = ok(repo.root(), session, &["flow", "finish", "--yes"]);
    let outcome: FlowOutcome = serde_json::from_value(out).unwrap();
    assert_eq!(outcome.status, FlowStatus::Done, "{outcome:?}");
    assert_eq!(head(&repo), "develop");
    assert!(repo
        .try_git(&["rev-parse", "--verify", "-q", "feature/x"])
        .is_none());
    assert_eq!(
        std::fs::read_to_string(repo.root().join("f.txt")).unwrap(),
        "f\n"
    );
    ok(repo.root(), session, &["undo"]);
    assert_eq!(head(&repo), "feature/x");
    assert_eq!(repo.git(&["rev-parse", "feature/x"]).trim(), feature);
    // Nothing is waiting to continue.
    let error = error_of(
        repo.root(),
        session,
        &["flow", "finish", "--continue", "--yes"],
    );
    assert!(error.contains("no flow"), "{error}");
}

#[test]
fn a_conflict_exits_with_2_and_continues_or_aborts() {
    let session = "cli-flow-conflict";
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\ntwo\nthree\n", "init")
        .unwrap()
        .branch("develop")
        .unwrap()
        .branch("feature/c")
        .unwrap()
        .commit_file("a.txt", "one\nFEATURE\nthree\n", "feature edit")
        .unwrap()
        .checkout("develop")
        .unwrap()
        .commit_file("a.txt", "one\nDEVELOP\nthree\n", "develop edit")
        .unwrap()
        .checkout("feature/c")
        .unwrap()
        .build();
    init_workflow(&repo, session, "git-flow");

    let output = run(repo.root(), session, &["flow", "finish", "--yes"]);
    assert_eq!(output.status.code(), Some(2));
    let outcome: FlowOutcome = serde_json::from_value(json_of(&output)).unwrap();
    assert_eq!(outcome.status, FlowStatus::Conflicted);
    assert_eq!(outcome.conflicts, ["a.txt"]);
    assert_eq!(head(&repo), "develop");

    let error = error_of(
        repo.root(),
        session,
        &["flow", "finish", "--continue", "--yes"],
    );
    assert!(error.contains("a.txt"), "{error}");

    std::fs::write(repo.root().join("a.txt"), "one\nBOTH\nthree\n").unwrap();
    repo.git(&["add", "a.txt"]);
    let out = ok(
        repo.root(),
        session,
        &["flow", "finish", "--continue", "--yes"],
    );
    let outcome: FlowOutcome = serde_json::from_value(out).unwrap();
    assert_eq!(outcome.status, FlowStatus::Done, "{outcome:?}");
    assert_eq!(head(&repo), "develop");
    assert!(repo
        .try_git(&["rev-parse", "--verify", "-q", "feature/c"])
        .is_none());
}

#[test]
fn a_stopped_flow_can_be_aborted() {
    let session = "cli-flow-abort";
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\ntwo\nthree\n", "init")
        .unwrap()
        .branch("develop")
        .unwrap()
        .branch("feature/c")
        .unwrap()
        .commit_file("a.txt", "one\nFEATURE\nthree\n", "feature edit")
        .unwrap()
        .checkout("develop")
        .unwrap()
        .commit_file("a.txt", "one\nDEVELOP\nthree\n", "develop edit")
        .unwrap()
        .checkout("feature/c")
        .unwrap()
        .build();
    init_workflow(&repo, session, "git-flow");
    let output = run(repo.root(), session, &["flow", "finish", "--yes"]);
    assert_eq!(output.status.code(), Some(2));
    // Aborting needs no confirmation.
    let out = ok(repo.root(), session, &["flow", "finish", "--abort"]);
    assert_eq!(out["status"], "aborted");
    assert_eq!(head(&repo), "feature/c");
    assert!(repo
        .try_git(&["rev-parse", "--verify", "-q", "MERGE_HEAD"])
        .is_none());
}

#[test]
fn hooks_run_only_when_asked_and_their_output_is_in_the_result() {
    let session = "cli-flow-hooks";
    let repo = repo(session);
    let path = repo.state_dir().join("workflow.toml");
    let mut text = std::fs::read_to_string(&path).unwrap();
    let hook = if cfg!(windows) {
        "echo hi> hook.txt && echo from-hook"
    } else {
        "echo hi > hook.txt; echo from-hook"
    };
    text.push_str(&format!("\n[hooks]\nafter_start = [{hook:?}]\n"));
    std::fs::write(&path, text).unwrap();

    // Not asked: the plan lists it, nothing runs.
    let out = ok(
        repo.root(),
        session,
        &["flow", "start", "feature", "h", "--yes"],
    );
    let outcome: FlowOutcome = serde_json::from_value(out).unwrap();
    assert!(!repo.root().join("hook.txt").exists());
    assert!(outcome.steps.iter().all(|s| s.exit_code.is_none()));
    ok(repo.root(), session, &["undo"]);

    let out = ok(
        repo.root(),
        session,
        &["flow", "start", "feature", "h", "--run-hooks", "--yes"],
    );
    let outcome: FlowOutcome = serde_json::from_value(out).unwrap();
    let last = outcome.steps.last().unwrap();
    assert_eq!(last.status, StepStatus::Done);
    assert_eq!(last.exit_code, Some(0));
    assert!(last.output.as_deref().unwrap().contains("from-hook"));
    assert!(repo.root().join("hook.txt").exists());

    // The plan shows the exact command.
    ok(repo.root(), session, &["undo"]);
    let out = ok(
        repo.root(),
        session,
        &["flow", "start", "feature", "h2", "--run-hooks", "--dry-run"],
    );
    let plan = plan_of(&out);
    assert!(plan.hooks_run);
    assert_eq!(plan.hooks[0].command, hook);
    let output = run(
        repo.root(),
        session,
        &[
            "flow",
            "start",
            "feature",
            "h3",
            "--run-hooks",
            "--no-hooks",
        ],
    );
    assert!(!output.status.success(), "the two flags exclude each other");
}

#[test]
fn set_parent_and_sync_and_promote() {
    let session = "cli-flow-misc";
    let repo = repo(session);
    // set-parent needs no confirmation and is undoable.
    let out = ok(
        repo.root(),
        session,
        &["flow", "set-parent", "feature/x", "main"],
    );
    assert_eq!(out["status"], "done");
    assert_eq!(
        repo.git(&["config", "branch.feature/x.gitpoulpi-parent"])
            .trim(),
        "main"
    );
    let error = error_of(
        repo.root(),
        session,
        &["flow", "set-parent", "main", "feature/x"],
    );
    assert!(error.contains("cycle"), "{error}");
    ok(
        repo.root(),
        session,
        &["flow", "set-parent", "feature/x", "--clear"],
    );

    // sync: feature/x is behind develop by one commit.
    let out = ok(
        repo.root(),
        session,
        &["flow", "sync", "feature/x", "--dry-run"],
    );
    let plan = plan_of(&out);
    let ops: Vec<String> = plan.lines();
    assert_eq!(ops.len(), 3, "{ops:?}");
    let out = ok(
        repo.root(),
        session,
        &["flow", "sync", "feature/x", "--yes"],
    );
    assert_eq!(out["status"], "done");
    assert_eq!(head(&repo), "main", "back where it started");
    assert!(repo
        .try_git(&["merge-base", "--is-ancestor", "develop", "feature/x"])
        .is_some());

    // promote: develop already has everything of main, so nothing is done.
    let out = ok(
        repo.root(),
        session,
        &["flow", "promote", "develop", "--yes"],
    );
    assert_eq!(out["status"], "nothing_to_do", "{out}");
}

#[test]
fn a_github_flow_finish_pushes_and_opens_the_merge_request() {
    let session = "cli-flow-mr";
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "base\n", "init")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .branch("feature/mr")
        .unwrap()
        .commit_file("m.txt", "m\n", "work")
        .unwrap()
        .build();
    // The host's name for the remote, the bare repository for pushing; no
    // integration is configured, so the request falls back to its web form.
    let bare = repo
        .remote_path()
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/");
    repo.git(&[
        "remote",
        "set-url",
        "origin",
        "https://github.com/me/thing.git",
    ]);
    repo.git(&["config", "remote.origin.pushurl", &bare]);
    init_workflow(&repo, session, "github-flow");

    // The plan names the request and says Undo cannot take it back.
    let value = ok(repo.root(), session, &["flow", "finish", "--dry-run"]);
    let plan = plan_of(&value);
    assert!(!plan.undoable);
    assert_eq!(plan.merge_request.as_ref().unwrap().target, "main");
    assert!(plan
        .irreversible
        .iter()
        .any(|i| i.contains("merge request")));

    let value = ok(
        repo.root(),
        session,
        &["flow", "finish", "--yes", "--title", "Ship it"],
    );
    assert_eq!(value["status"], "done", "{value}");
    let url = value["opened"]["url"].as_str().expect("the compare URL");
    assert!(
        url.starts_with("https://github.com/me/thing/compare/main...feature/mr?"),
        "{url}"
    );
    assert!(url.contains("title=Ship%20it"), "{url}");
    assert!(repo.git_in_remote(&["branch"]).contains("feature/mr"));
    // The branch stays, the merge request needs it.
    assert_eq!(head(&repo), "feature/mr");
    assert!(repo
        .try_git(&["rev-parse", "--verify", "-q", "feature/mr"])
        .is_some());
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
