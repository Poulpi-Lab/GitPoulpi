//! `gitpoulpi log --format mermaid`: the Log as a Mermaid `gitGraph` script,
//! through the daemon like `log`. Asserts on the deserialized `--json`
//! payload; the generator itself is tested in `gitpoulpi-core`.
#![cfg(feature = "mermaid")]

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_testkit::FixtureRepoBuilder;
use serde_json::Value;

fn config_dir() -> &'static Path {
    static DIR: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    DIR.get_or_init(|| tempfile::tempdir().unwrap()).path()
}

fn run(root: &Path, session: &str, json: bool, args: &[&str]) -> std::process::Output {
    let mut cmd = Command::cargo_bin("gitpoulpi").unwrap();
    cmd.env("GITPOULPI_CONFIG_DIR", config_dir());
    if json {
        cmd.arg("--json");
    }
    cmd.args(["--session", session])
        .args(["--repo", &root.to_string_lossy()])
        .args(args)
        .current_dir(root)
        .output()
        .unwrap()
}

fn run_json(root: &Path, session: &str, args: &[&str]) -> Value {
    let output = run(root, session, true, args);
    assert!(
        output.status.success(),
        "{args:?} failed: {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn log_renders_a_merge_as_a_mermaid_gitgraph_at_each_detail() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .tag("v1")
        .unwrap()
        .branch("feature/x")
        .unwrap()
        .commit("feat one")
        .unwrap()
        .checkout("-")
        .unwrap()
        .commit("main two")
        .unwrap()
        .build();
    repo.git(&["merge", "--no-ff", "-m", "merge feature", "feature/x"]);
    let root = repo.root();
    let session = "cli-log-mermaid";

    let refs = run_json(root, session, &["log", "--format", "mermaid"]);
    assert_eq!(refs["format"], "mermaid");
    assert_eq!(refs["detail"], "refs");
    assert_eq!(refs["commits"].as_array().unwrap().len(), 4);
    let text = refs["text"].as_str().unwrap();
    assert!(text.starts_with("gitGraph LR:\n"), "{text}");
    assert!(text.contains("branch \"feature/x\""), "{text}");
    assert!(text.contains("merge \"feature/x\" id: "), "{text}");
    assert!(text.contains("tag: \"v1\""), "{text}");
    assert!(text.contains("type: HIGHLIGHT"), "{text}");

    let flat = run_json(
        root,
        session,
        &["log", "--format", "mermaid", "--detail", "structure"],
    );
    let flat = flat["text"].as_str().unwrap();
    assert!(
        flat.contains("branch b1") && !flat.contains("tag:"),
        "{flat}"
    );

    let subjects = run_json(
        root,
        session,
        &[
            "log",
            "--format",
            "mermaid",
            "--detail",
            "subjects",
            "--orientation",
            "tb",
            "--fence",
        ],
    );
    let subjects = subjects["text"].as_str().unwrap();
    assert!(
        subjects.starts_with("```mermaid\ngitGraph TB:\n"),
        "{subjects}"
    );
    assert!(subjects.contains("merge feature"), "{subjects}");

    // `--range A..B` is `git log A..B`: the merge and the feature commit.
    let ranged = run_json(
        root,
        session,
        &["log", "--format", "mermaid", "--range", "v1..HEAD"],
    );
    assert_eq!(ranged["commits"].as_array().unwrap().len(), 3);

    // The human output is the script itself.
    let human = run(
        root,
        session,
        false,
        &["log", "--format", "mermaid", "-n", "2"],
    );
    assert!(human.status.success());
    let human = String::from_utf8(human.stdout).unwrap();
    assert!(human.starts_with("gitGraph LR:"), "{human}");
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
