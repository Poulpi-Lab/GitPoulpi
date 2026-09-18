//! `git.default_encoding` through the real binary and daemon: `log --json`
//! decodes a non-UTF-8 commit message with the configured encoding.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_core::domain::CommitGraph;
use gitpoulpi_testkit::FixtureRepoBuilder;

fn log_summary(root: &Path, config: &Path, session: &str) -> String {
    let out = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config)
        .env_remove("GITPOULPI_GIT_DEFAULT_ENCODING")
        .arg("--json")
        .args(["--session", session])
        .args(["--repo", &root.to_string_lossy()])
        .args(["log", "--limit", "1"])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let graph: CommitGraph = serde_json::from_slice(&out.stdout).unwrap();
    graph.nodes[0].summary.clone()
}

#[test]
fn log_json_decodes_a_latin1_commit_message_with_the_configured_encoding() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "base")
        .unwrap()
        .build();
    repo.commit_raw_message(b"caf\xe9 cr\xe8me\n", None);

    let plain = tempfile::tempdir().unwrap();
    let lossy = log_summary(repo.root(), plain.path(), "cli-test-encoding-plain");
    assert!(lossy.contains('\u{fffd}'), "{lossy:?}");

    let configured = tempfile::tempdir().unwrap();
    std::fs::write(
        configured.path().join("config.toml"),
        "[git]\ndefault_encoding = \"windows-1252\"\n",
    )
    .unwrap();
    assert_eq!(
        log_summary(repo.root(), configured.path(), "cli-test-encoding-set"),
        "caf\u{e9} cr\u{e8}me"
    );
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
