//! `gitpoulpi conflicts check`: the proactive conflict prediction as JSON,
//! through the real binary (the MCP tool `conflicts_check` is the same leaf).

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_core::domain::conflict_prevention::{ConflictPrediction, PredictionStatus};
use gitpoulpi_testkit::prediction_fixture::{conflicting_feature, fix_the_conflict};
use gitpoulpi_testkit::FixtureRepo;
use serde_json::Value;

fn config_dir() -> &'static Path {
    static DIR: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    DIR.get_or_init(|| tempfile::tempdir().expect("temp config dir"))
        .path()
}

fn run(repo: &FixtureRepo, session: &str, args: &[&str]) -> (i32, Value) {
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config_dir())
        .env("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false")
        .arg("--json")
        .args(["--session", session])
        .args(["--repo", &repo.root().to_string_lossy()])
        .args(args)
        .current_dir(repo.root())
        .output()
        .expect("failed to run gitpoulpi");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let json = serde_json::from_str(&stdout).unwrap_or(Value::Null);
    (output.status.code().unwrap_or(-1), json)
}

fn check(repo: &FixtureRepo, session: &str) -> ConflictPrediction {
    let (code, json) = run(repo, session, &["conflicts", "check"]);
    assert_eq!(code, 0, "{json}");
    serde_json::from_value(json).expect("a ConflictPrediction")
}

#[test]
fn check_names_the_files_then_goes_clean_once_the_branch_fixes_it() {
    let repo = conflicting_feature();
    let session = "cli-conflicts-check";
    let prediction = check(&repo, session);
    assert_eq!(prediction.status, PredictionStatus::Conflicts);
    assert_eq!(prediction.base_ref.as_deref(), Some("origin/main"));
    assert_eq!(prediction.files, ["a.txt"]);
    assert_eq!(
        prediction.head.as_deref(),
        Some(repo.git(&["rev-parse", "HEAD"]).trim())
    );

    fix_the_conflict(&repo);
    let prediction = check(&repo, session);
    assert_eq!(prediction.status, PredictionStatus::Clean);
    assert!(prediction.files.is_empty());
}

#[test]
fn check_is_disabled_by_the_global_setting() {
    let repo = conflicting_feature();
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config_dir())
        .env("GITPOULPI_GENERAL_JOURNAL_OUTSIDE", "false")
        .env("GITPOULPI_GENERAL_CONFLICT_DETECTION", "false")
        .arg("--json")
        .args(["--session", "cli-conflicts-check-off"])
        .args(["--repo", &repo.root().to_string_lossy()])
        .args(["conflicts", "check"])
        .current_dir(repo.root())
        .output()
        .unwrap();
    let prediction: ConflictPrediction =
        serde_json::from_slice(&output.stdout).expect("a ConflictPrediction");
    assert_eq!(prediction.status, PredictionStatus::Disabled);
    assert!(prediction.files.is_empty());
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
