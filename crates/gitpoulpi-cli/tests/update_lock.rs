//! While an update runs (`<data>/updates/updating.lock`), a command that
//! would start the session's daemon refuses with one English JSON error
//! instead of starting a daemon from files that are being replaced. A stale
//! lock (older than ten minutes) is ignored.

use assert_cmd::Command;
use gitpoulpi_config::{update_lock, ConfigPaths};
use gitpoulpi_testkit::FixtureRepoBuilder;
use serde_json::Value;

fn status(config: &std::path::Path, repo: &std::path::Path) -> (bool, Value) {
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config)
        .args(["--json", "--session", "cli-test-update-lock"])
        .args(["--repo", &repo.to_string_lossy()])
        .arg("status")
        .current_dir(repo)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let value =
        serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("not JSON ({e}): {stdout}"));
    (output.status.success(), value)
}

fn lock(config: &std::path::Path, at: u64) {
    let paths = ConfigPaths::for_dir(config);
    std::fs::create_dir_all(paths.updates_dir()).unwrap();
    std::fs::write(paths.update_lock_file(), update_lock::render(1, at)).unwrap();
}

#[test]
fn a_live_update_lock_makes_daemon_auto_start_refuse_with_a_json_error() {
    let config = tempfile::tempdir().unwrap();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    lock(config.path(), update_lock::now_secs());
    let (ok, value) = status(config.path(), repo.root());
    assert!(!ok, "{value}");
    assert_eq!(value["error"], update_lock::UPDATING_MESSAGE, "{value}");
}

#[test]
fn a_stale_update_lock_is_ignored() {
    let config = tempfile::tempdir().unwrap();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    lock(
        config.path(),
        update_lock::now_secs() - update_lock::STALE_AFTER_SECS - 5,
    );
    let (ok, value) = status(config.path(), repo.root());
    assert!(ok, "{value}");
}
