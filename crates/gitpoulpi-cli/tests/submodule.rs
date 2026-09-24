//! `gitpoulpi submodule list|init|update` through the real binary.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_core::domain::{Submodule, SubmoduleState};
use gitpoulpi_testkit::FixtureRepoBuilder;
use serde_json::Value;

fn config_dir() -> &'static Path {
    static DIR: std::sync::OnceLock<tempfile::TempDir> = std::sync::OnceLock::new();
    DIR.get_or_init(|| tempfile::tempdir().unwrap()).path()
}

/// A gitconfig that lets the daemon clone local submodules (git >= 2.38.1
/// refuses the `file` transport otherwise).
fn global_gitconfig() -> &'static Path {
    static FILE: std::sync::OnceLock<tempfile::NamedTempFile> = std::sync::OnceLock::new();
    FILE.get_or_init(|| {
        let file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(file.path(), "[protocol \"file\"]\n\tallow = always\n").unwrap();
        file
    })
    .path()
}

fn run_json(root: &Path, session: &str, args: &[&str]) -> Value {
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config_dir())
        .env("GIT_CONFIG_GLOBAL", global_gitconfig())
        .arg("--json")
        .args(["--session", session])
        .args(["--repo", &root.to_string_lossy()])
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "`gitpoulpi {}` failed: {}{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn submodules_are_listed_initialized_and_updated() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .with_submodule("libs/lib")
        .unwrap()
        .build();
    let session = "cli-test-submodule";

    let listed: Vec<Submodule> =
        serde_json::from_value(run_json(repo.root(), session, &["submodule", "list"])).unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].path, "libs/lib");
    assert_eq!(listed[0].state, SubmoduleState::UpToDate);

    // A fresh clone has it declared but not cloned.
    let holder = tempfile::tempdir().unwrap();
    let clone = holder.path().join("clone");
    let status = std::process::Command::new("git")
        .args(["clone", "-q"])
        .arg(repo.root())
        .arg(&clone)
        .env("GIT_CONFIG_GLOBAL", global_gitconfig())
        .status()
        .unwrap();
    assert!(status.success());
    let listed: Vec<Submodule> =
        serde_json::from_value(run_json(&clone, session, &["submodule", "list"])).unwrap();
    assert_eq!(listed[0].state, SubmoduleState::Uninitialized);

    run_json(&clone, session, &["submodule", "init"]);
    let updated: Vec<Submodule> = serde_json::from_value(run_json(
        &clone,
        session,
        &["submodule", "update", "--init", "--recursive"],
    ))
    .unwrap();
    assert_eq!(updated[0].state, SubmoduleState::UpToDate);
    assert!(clone.join("libs/lib/lib.txt").is_file());
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
