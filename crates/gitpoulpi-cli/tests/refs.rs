//! `gitpoulpi refs hide|show|solo`: the per-repo ref filter, changed
//! through the daemon and persisted in the repo's `repo-state.toml` (the
//! file the TUI reads). Asserts on deserialized JSON and on the state file.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_core::domain::RefVisibility;
use gitpoulpi_session::RepoState;
use gitpoulpi_testkit::FixtureRepoBuilder;
use serde_json::Value;

fn config_dir() -> &'static Path {
    static DIR: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    DIR.get_or_init(|| tempfile::tempdir().unwrap()).path()
}

fn run(root: &Path, session: &str, args: &[&str]) -> std::process::Output {
    Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config_dir())
        .arg("--json")
        .args(["--session", session])
        .args(["--repo", &root.to_string_lossy()])
        .args(args)
        .current_dir(root)
        .output()
        .unwrap()
}

fn run_json(root: &Path, session: &str, args: &[&str]) -> Value {
    let output = run(root, session, args);
    assert!(
        output.status.success(),
        "{args:?} failed: {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn hide_solo_show_and_clear_update_the_saved_filter() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .branch("feature/x")
        .unwrap()
        .branch("other")
        .unwrap()
        .build();
    let root = repo.root();
    let session = "cli-refs-filter";
    let state_of = || {
        let state_dir = root.join(".git").join("gitpoulpi").join("repo-state.toml");
        RepoState::load(&state_dir).unwrap().ref_visibility
    };

    let hid = run_json(root, session, &["refs", "hide", "feature/x", "other"]);
    assert_eq!(strings(&hid["hidden"]), ["feature/x", "other"]);
    assert!(hid["solo"].as_array().unwrap().is_empty());
    assert_eq!(state_of().get("feature/x"), Some(&RefVisibility::Hidden));

    let soloed = run_json(root, session, &["refs", "solo", "other"]);
    assert_eq!(strings(&soloed["solo"]), ["other"]);
    assert_eq!(strings(&soloed["hidden"]), ["feature/x"]);

    let shown = run_json(root, session, &["refs", "show", "feature/x"]);
    assert!(shown["hidden"].as_array().unwrap().is_empty());
    assert_eq!(strings(&shown["solo"]), ["other"]);

    run_json(root, session, &["refs", "hide", "feature/x"]);
    let cleared = run_json(root, session, &["refs", "show", "--clear"]);
    assert!(cleared["hidden"].as_array().unwrap().is_empty());
    assert!(cleared["solo"].as_array().unwrap().is_empty());
    assert!(state_of().is_empty());
}

#[test]
fn hiding_an_unknown_ref_is_refused_and_a_name_is_required_without_clear() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let session = "cli-refs-errors";
    let unknown = run(repo.root(), session, &["refs", "hide", "no-such-branch"]);
    assert!(!unknown.status.success());
    let json: Value = serde_json::from_slice(&unknown.stdout).unwrap();
    assert!(json["error"].as_str().unwrap().contains("no-such-branch"));

    let bare = run(repo.root(), session, &["refs", "hide"]);
    assert!(!bare.status.success(), "a name or --clear is required");
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
