//! `gitpoulpi log --count`: the commit total the TUI's status line shows
//! after the slash, through the daemon. Asserts on the deserialized JSON.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_testkit::FixtureRepoBuilder;
use serde_json::Value;

fn run_json(root: &Path, config: &Path, args: &[&str]) -> Value {
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config)
        .args(["--json", "--session", "log-count"])
        .args(["--repo", &root.to_string_lossy()])
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{args:?} failed: {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn log_count_is_the_whole_walk_whatever_limit_and_skip_say() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("one")
        .unwrap()
        .branch("side")
        .unwrap()
        .commit("side one")
        .unwrap()
        .checkout("-")
        .unwrap()
        .commit("two")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let root = repo.root();

    // HEAD's ancestry: one, two. The graph walk adds the side branch.
    assert_eq!(
        run_json(root, config.path(), &["log", "--count"])["total"],
        2
    );
    assert_eq!(
        run_json(root, config.path(), &["log", "--count", "--graph"])["total"],
        3
    );
    assert_eq!(
        run_json(
            root,
            config.path(),
            &["log", "--count", "--graph", "-n", "1", "--skip", "1"]
        )["total"],
        3,
        "paging flags do not shrink the total"
    );
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
