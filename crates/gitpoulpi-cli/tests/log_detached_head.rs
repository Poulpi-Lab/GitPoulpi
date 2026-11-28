//! `gitpoulpi log --graph` and `--count` on a detached HEAD holding commits
//! no ref reaches: the walk includes them, once, and the total agrees.
//! Asserts on the deserialized JSON.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_testkit::FixtureRepoBuilder;
use serde_json::Value;

fn run_json(root: &Path, config: &Path, args: &[&str]) -> Value {
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config)
        .args(["--json", "--session", "log-detached"])
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
fn the_graph_walk_includes_commits_only_a_detached_head_reaches() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("one")
        .unwrap()
        .build();
    repo.git(&["checkout", "-q", "--detach", "HEAD"]);
    for name in ["lone alpha", "lone beta"] {
        repo.git(&["commit", "-q", "--allow-empty", "-m", name]);
    }
    let config = tempfile::tempdir().unwrap();
    let root = repo.root();

    let count = run_json(root, config.path(), &["log", "--count", "--graph"]);
    assert_eq!(count["total"], 3);

    let log = run_json(root, config.path(), &["log", "--graph"]);
    let nodes = log["nodes"].as_array().expect("nodes");
    let summaries: Vec<&str> = nodes
        .iter()
        .map(|n| n["summary"].as_str().unwrap())
        .collect();
    assert_eq!(summaries.len(), 3, "{summaries:?}");
    for name in ["lone alpha", "lone beta", "one"] {
        assert_eq!(
            summaries.iter().filter(|s| **s == name).count(),
            1,
            "{name} once: {summaries:?}"
        );
    }
}
