//! A failed command says which operation failed: the `--json` error object
//! keeps its `error` (and `detail`) and adds a `context` with the operation,
//! its main arguments, the repository, who asked (`CLI`) and the request's id;
//! the plain error lists the same facts under the message.

use assert_cmd::Command;
use gitpoulpi_testkit::FixtureRepoBuilder;
use serde_json::Value;

fn run(args: &[&str], json: bool) -> (std::process::Output, String) {
    let config = tempfile::tempdir().unwrap();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    let mut command = Command::cargo_bin("gitpoulpi").unwrap();
    command
        .env("GITPOULPI_CONFIG_DIR", config.path())
        .args(["--session", "cli-test-failure-context"]);
    if json {
        command.arg("--json");
    }
    let output = command
        .args(["--repo", &repo.root().to_string_lossy()])
        .args(args)
        .current_dir(repo.root())
        .output()
        .unwrap();
    (output, repo.root().to_string_lossy().into_owned())
}

#[test]
fn a_failed_remote_removal_adds_its_operation_to_the_json_error() {
    let (output, root) = run(&["remote", "remove", "cm6"], true);
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let value: Value = serde_json::from_str(&stdout).expect("error output was not JSON");
    // What was there stays.
    assert!(value["error"].as_str().unwrap().contains("cm6"), "{value}");
    // What is added.
    let context = &value["context"];
    assert_eq!(context["label"], "Remove remote", "{value}");
    assert_eq!(context["operation"], "RemoteRemove", "{value}");
    assert_eq!(context["args"], "remote=cm6", "{value}");
    assert_eq!(context["frontend"], "CLI", "{value}");
    assert_eq!(context["repo"], root.as_str(), "{value}");
    assert!(context["request"].as_u64().unwrap() > 0, "{value}");
}

#[test]
fn the_plain_error_lists_the_operation_under_the_message() {
    let (output, _) = run(&["remote", "remove", "cm6"], false);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("gitpoulpi: "), "{stderr}");
    assert!(
        stderr.contains("operation: Remove remote (RemoteRemove)"),
        "{stderr}"
    );
    assert!(stderr.contains("arguments: remote=cm6"), "{stderr}");
    assert!(stderr.contains("requested by: CLI"), "{stderr}");
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
