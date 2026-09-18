//! A failed network command reports what went wrong, not just that it did:
//! the `--json` error object carries a `detail` with the operation, the
//! command line that ran, its exit code and git's stderr, and no password.

use assert_cmd::Command;
use gitpoulpi_testkit::FixtureRepoBuilder;
use serde_json::Value;

#[test]
fn a_failed_push_reports_its_command_exit_code_and_stderr_as_json() {
    let config = tempfile::tempdir().unwrap();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    // A port nobody listens on: fails at once, and offline.
    repo.git(&[
        "remote",
        "add",
        "origin",
        "http://me:topsecret@127.0.0.1:1/r.git",
    ]);

    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config.path())
        .args(["--json", "--session", "cli-test-failure-detail"])
        .args(["--repo", &repo.root().to_string_lossy()])
        .args(["push", "--remote", "origin"])
        .current_dir(repo.root())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let value: Value = serde_json::from_str(&stdout).expect("error output was not JSON");
    assert!(value["error"].is_string(), "{value}");
    let detail = &value["detail"];
    assert_eq!(detail["operation"], "Push", "{value}");
    assert!(
        detail["command"].as_str().unwrap().starts_with("git push"),
        "{value}"
    );
    assert!(detail["exit_code"].as_i64().unwrap() != 0, "{value}");
    assert!(
        !detail["stderr"].as_str().unwrap().trim().is_empty(),
        "{value}"
    );
    assert!(!stdout.contains("topsecret"), "{stdout}");
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
