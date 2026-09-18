//! `gitpoulpi split-source` and `split-commit`, end to end through the real
//! binary, the daemon and the backend. Assertions are on deserialized JSON.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_core::domain::{SplitOutcome, SplitSource};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::{json, Value};

fn config_dir() -> &'static Path {
    static DIR: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    DIR.get_or_init(|| tempfile::tempdir().unwrap()).path()
}

/// Runs the CLI; returns (exit code, parsed stdout).
fn run(repo: &FixtureRepo, session: &str, stdin: Option<&str>, args: &[&str]) -> (i32, Value) {
    let mut command = Command::cargo_bin("gitpoulpi").unwrap();
    command
        .env("GITPOULPI_CONFIG_DIR", config_dir())
        .args(["--json", "--session", session])
        .args(["--repo", &repo.root().to_string_lossy()])
        .args(args)
        .current_dir(repo.root());
    if let Some(input) = stdin {
        command.write_stdin(input.to_string());
    }
    let output = command.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "not JSON ({e}): {stdout}\n{}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output.status.code().unwrap_or(-1), value)
}

fn ok(repo: &FixtureRepo, session: &str, args: &[&str]) -> Value {
    let (code, value) = run(repo, session, None, args);
    assert_eq!(code, 0, "{args:?}: {value}");
    value
}

fn twenty_lines(second: &str, eighteenth: &str) -> String {
    (1..=20)
        .map(|n| match n {
            2 => format!("{second}\n"),
            18 => format!("{eighteenth}\n"),
            _ => format!("line {n}\n"),
        })
        .collect()
}

/// A commit touching `a.txt` in two distant hunks and adding `b.txt`.
fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", &twenty_lines("line 2", "line 18"), "add a")
        .unwrap()
        .commit_file("a.txt", &twenty_lines("two", "eighteen"), "edit a")
        .unwrap()
        .commit("top")
        .unwrap()
        .build()
}

fn rev_parse(repo: &FixtureRepo, rev: &str) -> String {
    repo.git(&["rev-parse", rev]).trim().to_string()
}

fn source(repo: &FixtureRepo, session: &str) -> SplitSource {
    serde_json::from_value(ok(repo, session, &["split-source", "HEAD~1"])).unwrap()
}

fn hunk_ids(source: &SplitSource) -> Vec<Value> {
    let a = source
        .files
        .iter()
        .find(|f| f.path.to_string_lossy() == "a.txt")
        .unwrap();
    a.hunks.iter().map(|h| json!(h.id)).collect()
}

#[test]
fn split_source_numbers_hunks_and_marks_whole_only_files() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", &twenty_lines("line 2", "line 18"), "add a")
        .unwrap()
        .commit_file("a.txt", &twenty_lines("two", "eighteen"), "edit a")
        .unwrap()
        .commit_bytes("b.bin", &[0, 1, 2, 3], "add binary")
        .unwrap()
        .build();
    let session = "cli-split-source";
    let source: SplitSource =
        serde_json::from_value(ok(&repo, session, &["split-source", "HEAD~1"])).unwrap();
    assert_eq!(source.commit.to_hex(), rev_parse(&repo, "HEAD~1"));
    assert_eq!(source.message.trim(), "edit a");
    assert_eq!(source.files.len(), 1);
    assert_eq!(source.files[0].hunks.len(), 2);
    assert!(source.files[0].whole_only.is_none());

    let top: SplitSource =
        serde_json::from_value(ok(&repo, session, &["split-source", "HEAD"])).unwrap();
    assert_eq!(top.files.len(), 1);
    assert!(top.files[0].whole_only.is_some());
    assert!(top.files[0].hunks.is_empty());
}

#[test]
fn split_commit_applies_a_plan_then_undo_restores_the_tip() {
    let repo = fixture();
    let session = "cli-split-apply";
    let ids = hunk_ids(&source(&repo, session));
    assert_eq!(ids.len(), 2);
    let old_head = rev_parse(&repo, "HEAD");
    let old_tree = rev_parse(&repo, "HEAD^{tree}");
    let plan = json!({"parts": [
        {"message": "feat: two", "pieces": [{"path": "a.txt", "hunk": ids[0]}]},
        {"message": "feat: eighteen", "pieces": [{"path": "a.txt", "hunk": ids[1]}]},
    ]});
    let plan_file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(plan_file.path(), plan.to_string()).unwrap();

    let outcome: SplitOutcome = serde_json::from_value(ok(
        &repo,
        session,
        &[
            "split-commit",
            "HEAD~1",
            "--plan",
            &plan_file.path().to_string_lossy(),
        ],
    ))
    .unwrap();
    assert_eq!(outcome.created.len(), 2);
    assert_eq!(outcome.replayed, 1);
    assert_eq!(outcome.previous_head.to_hex(), old_head);
    assert_eq!(outcome.new_head.to_hex(), rev_parse(&repo, "HEAD"));
    assert_eq!(rev_parse(&repo, "HEAD^{tree}"), old_tree);
    assert_eq!(
        repo.git(&["log", "--format=%s", "-4"])
            .lines()
            .collect::<Vec<_>>(),
        ["top", "feat: eighteen", "feat: two", "add a"]
    );

    let undo = ok(&repo, session, &["undo"]);
    assert_eq!(undo["applied"], true, "{undo}");
    assert_eq!(rev_parse(&repo, "HEAD"), old_head);
}

#[test]
fn split_commit_dry_run_and_rest_message_from_stdin() {
    let repo = fixture();
    let session = "cli-split-rest";
    let ids = hunk_ids(&source(&repo, session));
    let old_head = rev_parse(&repo, "HEAD");
    let partial = json!({"parts": [
        {"message": "feat: two", "pieces": [{"path": "a.txt", "hunk": ids[0]}]},
    ]})
    .to_string();

    // A dry run with the rest assigned is valid; without it, it is not.
    let (code, dry) = run(
        &repo,
        session,
        Some(&partial),
        &["split-commit", "HEAD~1", "--plan", "-", "--dry-run"],
    );
    assert_eq!(code, 2, "{dry}");
    assert_eq!(dry["valid"], false);
    assert_eq!(dry["refusal"], "too_few_parts");
    assert_eq!(dry["unassigned"].as_array().unwrap().len(), 1, "{dry}");
    assert_eq!(rev_parse(&repo, "HEAD"), old_head);

    let (code, dry) = run(
        &repo,
        session,
        Some(&partial),
        &[
            "split-commit",
            "HEAD~1",
            "--plan",
            "-",
            "--rest-message",
            "chore: rest",
            "--dry-run",
        ],
    );
    assert_eq!(code, 0, "{dry}");
    assert_eq!(dry["valid"], true);
    assert_eq!(dry["parts"], 2);
    assert_eq!(rev_parse(&repo, "HEAD"), old_head);

    let (code, done) = run(
        &repo,
        session,
        Some(&partial),
        &[
            "split-commit",
            "HEAD~1",
            "--plan",
            "-",
            "--rest-message",
            "chore: rest",
        ],
    );
    assert_eq!(code, 0, "{done}");
    let outcome: SplitOutcome = serde_json::from_value(done).unwrap();
    assert_eq!(outcome.created.len(), 2);
    assert_eq!(
        repo.git(&["log", "--format=%s", "-3"])
            .lines()
            .collect::<Vec<_>>(),
        ["top", "chore: rest", "feat: two"]
    );
}

#[test]
fn split_commit_refusals_are_json_with_exit_code_2_and_change_nothing() {
    let repo = fixture();
    let session = "cli-split-refuse";
    let ids = hunk_ids(&source(&repo, session));
    let old_head = rev_parse(&repo, "HEAD");
    let refuse = |plan: Value, reason: &str, rev: &str| {
        let (code, value) = run(
            &repo,
            session,
            Some(&plan.to_string()),
            &["split-commit", rev, "--plan", "-"],
        );
        assert_eq!(code, 2, "{value}");
        assert_eq!(value["refused"]["reason"], reason, "{value}");
        assert!(value["refused"]["message"].is_string());
        assert_eq!(rev_parse(&repo, "HEAD"), old_head);
    };
    let one = json!({"parts": [{"message": "all", "pieces": [
        {"path": "a.txt", "hunk": ids[0]}, {"path": "a.txt", "hunk": ids[1]}]}]});
    refuse(one, "too_few_parts", "HEAD~1");
    let missing = json!({"parts": [
        {"message": "one", "pieces": [{"path": "a.txt", "hunk": ids[0]}]},
        {"message": "two", "pieces": [{"path": "a.txt", "hunk": 99}]}]});
    refuse(missing, "unknown_piece", "HEAD~1");
    let twice = json!({"parts": [
        {"message": "one", "pieces": [{"path": "a.txt"}]},
        {"message": "two", "pieces": [{"path": "a.txt", "hunk": ids[1]}]}]});
    refuse(twice, "assigned_twice", "HEAD~1");
    let empty_message = json!({"parts": [
        {"message": " ", "pieces": [{"path": "a.txt", "hunk": ids[0]}]},
        {"message": "two", "pieces": [{"path": "a.txt", "hunk": ids[1]}]}]});
    refuse(empty_message, "empty_message", "HEAD~1");

    // A malformed plan is a plain failure (exit 1, `error`), not a refusal.
    let (code, value) = run(
        &repo,
        session,
        Some("not json"),
        &["split-commit", "HEAD~1", "--plan", "-"],
    );
    assert_eq!(code, 1, "{value}");
    assert!(value["error"].is_string());
}

/// `ai split-plan` prints a plan `split-commit --plan` accepts and applies
/// nothing, with a scripted local agent (no socket).
#[cfg(feature = "ai")]
#[test]
fn ai_split_plan_prints_an_acceptable_plan_and_changes_nothing() {
    use gitpoulpi_testkit::fake_agent::{agent_calls, ai_config_toml, write_fake_agent};

    let repo = fixture();
    let tools = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let agent = write_fake_agent(
        tools.path(),
        &[
            r#"[{"message": "feat: two", "pieces": ["H1"]}, {"message": "feat: eighteen", "pieces": ["H2"]}]"#,
        ],
    );
    std::fs::write(config.path().join("config.toml"), ai_config_toml(&agent)).unwrap();
    let old_head = rev_parse(&repo, "HEAD");

    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config.path())
        .args(["--json", "--session", "cli-split-ai"])
        .args(["--repo", &repo.root().to_string_lossy()])
        .args(["ai", "split-plan", "HEAD~1"])
        .current_dir(repo.root())
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["applied"], false);
    assert_eq!(value["parts"].as_array().unwrap().len(), 2, "{value}");
    assert_eq!(agent_calls(&agent), 1);
    assert_eq!(rev_parse(&repo, "HEAD"), old_head);

    // The printed plan is what `split-commit --plan` takes. The default
    // config dir of `run` has no AI, which a dry run does not need.
    let (code, done) = run(
        &repo,
        "cli-split-ai",
        Some(&json!({ "parts": value["parts"] }).to_string()),
        &["split-commit", "HEAD~1", "--plan", "-", "--dry-run"],
    );
    assert_eq!(code, 0, "{done}");
    assert_eq!(done["valid"], true);
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
