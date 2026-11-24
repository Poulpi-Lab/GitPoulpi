#![cfg(all(feature = "ai", feature = "worktree"))]
//! `gitpoulpi sequence list|show|validate|run|add|remove`, end to end through
//! the real binary, the daemon, a bare local remote and a **fake local agent**
//! (a script printing a canned answer): no socket is ever opened.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_testkit::fake_agent::{agent_calls, ai_config_toml, write_fake_agent};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;

const BACKUP: &str = r#"
[[sequences]]
id = "backup"
name = "Backup"

[[sequences.steps]]
run = "stage_all"

[[sequences.steps]]
run = "ai_generate_commit_message"

[[sequences.steps]]
run = "commit"

[[sequences.steps]]
run = "push"
"#;

/// `(exit code, parsed stdout)` of `gitpoulpi --json sequence ...`.
fn seq(config: &Path, root: &Path, session: &str, args: &[&str]) -> (i32, Value) {
    cli(config, root, session, &[&["sequence"], args].concat())
}

/// `(exit code, parsed stdout)` of `gitpoulpi --json <args>`.
fn cli(config: &Path, root: &Path, session: &str, args: &[&str]) -> (i32, Value) {
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config)
        .env("GITPOULPI_INSTALL_ID", "tester")
        .env("GITPOULPI_GENERAL_UNDO_DEPTH", "0")
        .env("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false")
        .arg("--json")
        .args(["--session", session])
        .args(["--repo", &root.to_string_lossy()])
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "not JSON ({e}): {stdout} / {}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output.status.code().unwrap_or(-1), value)
}

/// A repository with a bare `origin` that already tracks `main`, one
/// untracked file, a config dir with a fake AI agent, and `sequences_toml`
/// added through `sequence add`.
struct World {
    repo: FixtureRepo,
    config: tempfile::TempDir,
    agent: std::path::PathBuf,
    _tools: tempfile::TempDir,
    session: String,
}

fn world(session: &str, sequences_toml: &str) -> World {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_origin()
        .unwrap()
        .build();
    repo.git(&["push", "-u", "origin", "main"]);
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(
        tools.path(),
        &["feat: add the new file", "", "It holds a greeting."],
    );
    std::fs::write(config.path().join("config.toml"), ai_config_toml(&agent)).unwrap();
    std::fs::write(repo.root().join("new.txt"), "hello\n").unwrap();
    let w = World {
        repo,
        config,
        agent,
        _tools: tools,
        session: session.to_string(),
    };
    if !sequences_toml.is_empty() {
        let file = w.config.path().join("seq.toml");
        std::fs::write(&file, sequences_toml).unwrap();
        let (code, v) = w.run(&["add", "--file", &file.to_string_lossy()]);
        assert_eq!(code, 0, "{v}");
    }
    w
}

impl World {
    fn run(&self, args: &[&str]) -> (i32, Value) {
        seq(self.config.path(), self.repo.root(), &self.session, args)
    }
    fn undo(&self, args: &[&str]) -> (i32, Value) {
        cli(
            self.config.path(),
            self.repo.root(),
            &self.session,
            &[&["undo"], args].concat(),
        )
    }
    fn head(&self) -> String {
        self.repo.git(&["rev-parse", "HEAD"]).trim().to_string()
    }
    fn status(&self) -> String {
        self.repo.git(&["status", "--porcelain"])
    }
}

#[test]
fn list_and_validate_print_the_sequences_as_json() {
    let w = world(
        "seq-list",
        &format!(
            "{BACKUP}\n[[sequences]]\nid = \"ui-only\"\nname = \"UI only\"\n\n[[sequences.steps]]\nrun = \"tab_dir_terminal\"\n"
        ),
    );
    // The repository's own file is listed but never trusted.
    let dir = w.repo.root().join(".gitpoulpi");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("sequences.toml"),
        "[[sequences]]\nid = \"shared\"\nname = \"Shared\"\n\n[[sequences.steps]]\nrun = \"push\"\n",
    )
    .unwrap();

    let (code, v) = w.run(&["list"]);
    assert_eq!(code, 0, "{v}");
    let list = v["sequences"].as_array().unwrap();
    let by_id = |id: &str| list.iter().find(|s| s["id"] == id).unwrap();
    let backup = by_id("backup");
    assert_eq!(backup["name"], "Backup");
    assert_eq!(backup["scope"], "installation");
    assert_eq!(backup["owner"], "p-tester");
    assert_eq!(backup["trusted"], true);
    assert_eq!(backup["steps"].as_array().unwrap().len(), 4);
    assert_eq!(backup["steps"][0]["run"], "stage_all");
    assert_eq!(backup["problems"], serde_json::json!([]));
    assert_eq!(backup["runnable_headless"], true);
    let ui = by_id("ui-only");
    assert_eq!(ui["runnable_headless"], false);
    assert_eq!(ui["headless_problems"][0]["what"], "not_headless");
    let shared = by_id("shared");
    assert_eq!(shared["trusted"], false);
    assert_eq!(shared["scope"], "repo");
    assert_eq!(shared["runnable_headless"], false);

    let (code, v) = w.run(&["show", "backup"]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["details"][1]["run"], "ai_generate_commit_message");
    assert!(v["details"][1]["pauses"].is_string(), "{v}");

    let (code, v) = w.run(&["validate", "backup"]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["valid"], true);

    // An unknown step and a forbidden `never` are reported, exit code 2.
    let bad = w.config.path().join("bad.toml");
    std::fs::write(
        &bad,
        "[[sequences]]\nid = \"bad\"\nname = \"Bad\"\n\n[[sequences.steps]]\nrun = \"no_such_step\"\n\n[[sequences.steps]]\nrun = \"force_push_with_lease\"\npause = \"never\"\n",
    )
    .unwrap();
    let (code, v) = w.run(&["validate", "--file", &bad.to_string_lossy()]);
    assert_eq!(code, 2, "{v}");
    assert_eq!(v["valid"], false);
    let whats: Vec<&str> = v["sequences"][0]["problems"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["what"].as_str().unwrap())
        .collect();
    assert!(whats.contains(&"unknown_run"), "{whats:?}");
    assert!(whats.contains(&"never_forced"), "{whats:?}");
}

#[test]
fn a_backup_run_with_yes_commits_with_the_agents_message_and_moves_the_remote() {
    let w = world("seq-backup", BACKUP);
    let before = w.head();
    let (code, v) = w.run(&["run", "backup", "--yes"]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["applied"], true);
    assert_eq!(v["stopped_at"], Value::Null);
    let statuses: Vec<&str> = v["steps"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["status"].as_str().unwrap())
        .collect();
    assert_eq!(statuses, ["done", "done", "done", "done"], "{v}");
    assert_eq!(v["steps"][3]["run"], "push");

    let message = w.repo.git(&["log", "-1", "--format=%B"]);
    assert!(message.starts_with("feat: add the new file"), "{message}");
    assert!(message.contains("It holds a greeting."), "{message}");
    assert_ne!(w.head(), before);
    assert_eq!(
        w.repo.git_in_remote(&["rev-parse", "main"]).trim(),
        w.head()
    );
    assert_eq!(agent_calls(&w.agent), 1);
    assert_eq!(w.status(), "");
}

#[test]
fn a_pausing_step_needs_yes_and_nothing_changes_without_it() {
    let w = world("seq-pause", BACKUP);
    let before = w.head();
    let (code, v) = w.run(&["run", "backup"]);
    assert_eq!(code, 3, "{v}");
    assert_eq!(v["applied"], false);
    assert_eq!(v["needs_confirmation"][0]["step"], 1);
    assert_eq!(
        v["needs_confirmation"][0]["run"],
        "ai_generate_commit_message"
    );
    // Not even the stage that comes first ran.
    assert_eq!(w.head(), before);
    assert_eq!(w.status().trim(), "?? new.txt");
    assert_eq!(agent_calls(&w.agent), 0);
}

#[test]
fn a_step_that_needs_the_terminal_ui_refuses_the_whole_run() {
    let w = world(
        "seq-cli-false",
        "[[sequences]]\nid = \"mixed\"\nname = \"Mixed\"\n\n[[sequences.steps]]\nrun = \"stage_all\"\n\n[[sequences.steps]]\nrun = \"tab_dir_terminal\"\n",
    );
    let (code, v) = w.run(&["run", "mixed", "--yes"]);
    assert_eq!(code, 2, "{v}");
    assert_eq!(v["applied"], false);
    assert_eq!(v["problems"][0]["step"], 1);
    assert_eq!(v["problems"][0]["what"], "not_headless");
    assert!(v["problems"][0]["reason"].is_string());
    assert_eq!(
        w.status().trim(),
        "?? new.txt",
        "stage_all must not have run"
    );
}

#[test]
fn a_force_push_step_is_refused_even_with_yes() {
    let w = world(
        "seq-force",
        "[[sequences]]\nid = \"force\"\nname = \"Force\"\n\n[[sequences.steps]]\nrun = \"stage_all\"\n\n[[sequences.steps]]\nrun = \"force_push_with_lease\"\n",
    );
    let (code, v) = w.run(&["run", "force", "--yes"]);
    assert_eq!(code, 2, "{v}");
    assert_eq!(v["problems"][0]["what"], "forced_pause");
    assert_eq!(w.status().trim(), "?? new.txt");
}

#[test]
fn dry_run_prints_the_resolved_plan_and_changes_nothing() {
    let w = world(
        "seq-dry",
        &format!(
            "{BACKUP}\n[[sequences]]\nid = \"wt\"\nname = \"Worktree\"\n\n[[sequences.steps]]\nrun = \"create_worktree_from_current_branch\"\n[sequences.steps.options]\nname = \"wip-${{n}}\"\n"
        ),
    );
    let before = w.head();
    let (code, v) = w.run(&["run", "backup", "--dry-run"]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["dry_run"], true);
    assert_eq!(v["applied"], false);
    assert_eq!(v["steps"][0]["command"], "gitpoulpi stage --all");
    assert_eq!(v["steps"][3]["command"], "gitpoulpi push");
    assert_eq!(v["needs_yes"], true);
    assert_eq!(w.head(), before);
    assert_eq!(w.status().trim(), "?? new.txt");
    assert_eq!(agent_calls(&w.agent), 0);

    // The name template is expanded against what exists.
    let (code, v) = w.run(&["run", "wt", "--dry-run"]);
    assert_eq!(code, 0, "{v}");
    let command = v["steps"][0]["command"].as_str().unwrap();
    assert!(command.contains("worktree add"), "{command}");
    assert!(command.ends_with("-b wip-1"), "{command}");
}

#[test]
fn a_failing_step_stops_the_run_and_the_rest_is_skipped() {
    let w = world(
        "seq-stop",
        "[[sequences]]\nid = \"tags\"\nname = \"Tags\"\n\n[[sequences.steps]]\nrun = \"new_tag\"\npause = \"never\"\n[sequences.steps.options]\nname = \"v1\"\n\n[[sequences.steps]]\nrun = \"new_tag\"\npause = \"never\"\n[sequences.steps.options]\nname = \"v1\"\n\n[[sequences.steps]]\nrun = \"push\"\n",
    );
    let (code, v) = w.run(&["run", "tags"]);
    assert_eq!(code, 2, "{v}");
    assert_eq!(v["applied"], true);
    assert_eq!(v["stopped_at"], 1);
    assert_eq!(v["steps"][0]["status"], "done");
    assert_eq!(v["steps"][1]["status"], "failed");
    assert!(v["steps"][1]["error"].is_string(), "{v}");
    assert_eq!(v["steps"][2]["status"], "skipped");
    assert_eq!(w.repo.git(&["tag", "--list", "v1"]).trim(), "v1");
}

#[test]
fn add_and_remove_round_trip_and_keep_the_other_config_keys() {
    let w = world("seq-add", "");
    let file = w.config.path().join("one.toml");
    // A single sequence at the top level works too.
    std::fs::write(
        &file,
        "id = \"quick\"\nname = \"Quick\"\n\n[[steps]]\nrun = \"fetch\"\n",
    )
    .unwrap();
    let f = file.to_string_lossy().into_owned();

    let (code, v) = w.run(&["add", "--file", &f]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["saved"][0]["id"], "quick");

    // The same id again needs --replace.
    let (code, v) = w.run(&["add", "--file", &f]);
    assert_eq!(code, 2, "{v}");
    assert_eq!(v["problems"][0]["what"], "exists");
    let (code, _) = w.run(&["add", "--file", &f, "--replace"]);
    assert_eq!(code, 0);

    // An invalid sequence writes nothing.
    let bad = w.config.path().join("bad.toml");
    std::fs::write(
        &bad,
        "id = \"bad\"\nname = \"Bad\"\n\n[[steps]]\nrun = \"discard_all_changes\"\n",
    )
    .unwrap();
    let (code, v) = w.run(&["add", "--file", &bad.to_string_lossy()]);
    assert_eq!(code, 2, "{v}");
    assert_eq!(v["applied"], false);

    let (_, v) = w.run(&["list"]);
    let ids: Vec<&str> = v["sequences"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["quick"]);

    let (code, v) = w.run(&["remove", "quick"]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["removed"], true);
    let (code, v) = w.run(&["remove", "quick"]);
    assert_eq!(code, 2, "{v}");
    assert_eq!(v["removed"], false);

    // Everything else in config.toml survived both writes.
    let text = std::fs::read_to_string(w.config.path().join("config.toml")).unwrap();
    let doc: toml::Table = toml::from_str(&text).unwrap();
    assert_eq!(doc["ai"]["enabled"], toml::Value::Boolean(true));
    assert_eq!(doc["ai"]["profiles"].as_array().unwrap().len(), 1);
    assert!(doc.get("sequences").is_none(), "{text}");
}

#[test]
fn a_repository_sequence_is_never_run() {
    let w = world("seq-untrusted", "");
    let dir = w.repo.root().join(".gitpoulpi");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("sequences.toml"),
        "[[sequences]]\nid = \"shared\"\nname = \"Shared\"\n\n[[sequences.steps]]\nrun = \"stage_all\"\n",
    )
    .unwrap();
    let (code, v) = w.run(&["run", "shared", "--yes"]);
    assert_eq!(code, 2, "{v}");
    assert_eq!(v["problems"][0]["what"], "untrusted");
    assert!(!w.status().contains("A  "), "{}", w.status());
}

/// A run of `sequence run` marks its journal entries; `undo --sequence` then
/// undoes them newest first. The Backup commit is already on its upstream, so
/// the undo needs `--yes` (exit 3, nothing changed) before it goes through.
#[test]
fn undo_sequence_undoes_a_backup_run_and_asks_for_the_pushed_commit() {
    let w = world("seq-undo", BACKUP);
    let before = w.head();
    let (code, v) = w.run(&["run", "backup", "--yes"]);
    assert_eq!(code, 0, "{v}");
    let pushed = w.head();

    // The entries carry the run: a name, one run id, the step of each.
    let (code, v) = w.undo(&["--list"]);
    assert_eq!(code, 0, "{v}");
    let entries = v["undo"].as_array().unwrap();
    assert_eq!(entries.len(), 2, "{v}");
    let origin = |i: usize| &entries[i]["summary"]["origin"];
    assert_eq!(origin(0)["name"], "Backup");
    assert_eq!(origin(0)["run_id"], origin(1)["run_id"]);
    assert_eq!(origin(0)["step"], 1);
    assert_eq!(origin(1)["step"], 3);
    assert_eq!(origin(1)["of"], 4);

    let (code, v) = w.undo(&["--sequence", "--dry-run"]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["applied"], false);
    assert_eq!(v["entries"].as_array().unwrap().len(), 2);
    assert_eq!(w.head(), pushed);

    // The commit is on the remote: asks, changes nothing.
    let (code, v) = w.undo(&["--sequence"]);
    assert_eq!(code, 3, "{v}");
    assert_eq!(v["applied"], false);
    assert_eq!(v["stopped"]["step"], 3);
    assert!(
        v["stopped"]["needs_confirmation"]["prompt"].is_string(),
        "{v}"
    );
    assert_eq!(v["undone"], serde_json::json!([]));
    assert_eq!(w.head(), pushed);

    let (code, v) = w.undo(&["--sequence", "--yes"]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["applied"], true);
    assert_eq!(v["undone"].as_array().unwrap().len(), 2);
    assert!(
        v["note"].as_str().unwrap().contains("this repository"),
        "{v}"
    );
    assert_eq!(w.head(), before, "the commit is undone");
    assert_eq!(w.status().trim(), "?? new.txt", "and so is the stage");

    // Nothing of the run is left at the top.
    let (code, v) = w.undo(&["--sequence"]);
    assert_eq!(code, 2, "{v}");
    assert!(v["no_sequence"]["message"].is_string(), "{v}");
}

/// A refusal in the middle stops the undo, says which step, and leaves the
/// older entries of the run in the journal.
#[test]
fn undo_sequence_stops_at_the_first_refusal_and_leaves_the_rest() {
    let w = world(
        "seq-undo-stop",
        "[[sequences]]\nid = \"tagged\"\nname = \"Tagged\"\n\n[[sequences.steps]]\nrun = \"stage_all\"\n\n[[sequences.steps]]\nrun = \"new_tag\"\npause = \"never\"\n[sequences.steps.options]\nname = \"v1\"\n\n[[sequences.steps]]\nrun = \"commit\"\npause = \"never\"\n[sequences.steps.options]\nmessage = \"add the file\"\n",
    );
    let before = w.head();
    let (code, v) = w.run(&["run", "tagged", "--yes"]);
    assert_eq!(code, 0, "{v}");
    // Someone moves the tag behind the run's back.
    w.repo.git(&["tag", "-f", "v1", "HEAD"]);

    let (code, v) = w.undo(&["--sequence"]);
    assert_eq!(code, 2, "{v}");
    assert_eq!(v["applied"], true, "the commit was undone first: {v}");
    assert_eq!(v["undone"].as_array().unwrap().len(), 1);
    assert_eq!(v["stopped"]["step"], 2, "the tag is step 2: {v}");
    assert!(v["stopped"]["refused"]["message"].is_string(), "{v}");
    assert_eq!(
        v["left"].as_array().unwrap().len(),
        2,
        "tag and stage stay: {v}"
    );
    assert_eq!(w.head(), before);
    // The stage entry is still in the journal.
    let (_, v) = w.undo(&["--list"]);
    assert_eq!(v["undo"].as_array().unwrap().len(), 2, "{v}");
}
