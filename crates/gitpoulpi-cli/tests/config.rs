//! `gitpoulpi config get|set|path`, the precedence ladder behind them,
//! and the one thing `general.default_backend` is for: making
//! `Git2Backend` reachable in a real run instead of only from the
//! contract suite.
//!
//! Assertions are on **deserialized JSON**: a `contains("git2")` test
//! passes just as happily when the word turns up in an error message.
//!
//! Every case points the binary at its own throwaway `GITPOULPI_CONFIG_DIR`
//! (`crates/gitpoulpi-config/src/paths.rs`), so no test reads or writes
//! the developer's real `config.toml`.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_core::domain::WorkingTree;
use gitpoulpi_testkit::FixtureRepoBuilder;
use serde_json::Value;

/// Runs `gitpoulpi --json ...` with an isolated config dir and the given
/// extra environment, returning (exit-success, parsed stdout).
fn run(config_dir: &Path, cwd: &Path, env: &[(&str, &str)], args: &[&str]) -> (bool, Value) {
    let mut command = Command::cargo_bin("gitpoulpi").unwrap();
    command
        .env("GITPOULPI_CONFIG_DIR", config_dir)
        .current_dir(cwd)
        .arg("--json")
        .args(args);
    for (key, value) in env {
        command.env(key, value);
    }
    let output = command.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let json = serde_json::from_str(&stdout).unwrap_or_else(|err| {
        panic!(
            "not JSON ({err}): stdout={stdout} stderr={}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output.status.success(), json)
}

fn ok(config_dir: &Path, cwd: &Path, env: &[(&str, &str)], args: &[&str]) -> Value {
    let (success, json) = run(config_dir, cwd, env, args);
    assert!(success, "expected success, got {json}");
    json
}

#[test]
fn config_path_points_inside_the_config_dir_and_reports_whether_it_exists() {
    let config = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();

    let before = ok(config.path(), cwd.path(), &[], &["config", "path"]);
    assert_eq!(before["exists"], Value::Bool(false));
    // Compared as paths, not as strings: Windows hands back the same
    // directory with a different spelling often enough that a string
    // equality here would be a platform-specific failure.
    assert_eq!(
        Path::new(before["path"].as_str().unwrap()),
        config.path().join("config.toml")
    );

    ok(
        config.path(),
        cwd.path(),
        &[],
        &["config", "set", "general.default_backend", "system"],
    );
    let after = ok(config.path(), cwd.path(), &[], &["config", "path"]);
    assert_eq!(after["exists"], Value::Bool(true));
}

#[test]
fn a_value_written_by_config_set_is_what_config_get_reads_back() {
    let config = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();

    let default = ok(
        config.path(),
        cwd.path(),
        &[],
        &["config", "get", "general.default_backend"],
    );
    assert_eq!(default["value"], Value::from("git2"));
    assert_eq!(default["source"], Value::from("default"));

    ok(
        config.path(),
        cwd.path(),
        &[],
        &["config", "set", "general.default_backend", "system"],
    );
    let stored = ok(
        config.path(),
        cwd.path(),
        &[],
        &["config", "get", "general.default_backend"],
    );
    assert_eq!(stored["value"], Value::from("system"));
    assert_eq!(stored["source"], Value::from("file"));
}

/// The precedence rule this wiring settled: defaults -> config.toml ->
/// `GITPOULPI_*` -> command-line flags, each layer overriding the one
/// below and `source` naming which one actually spoke.
#[test]
fn each_layer_overrides_the_one_below_it() {
    let config = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    ok(
        config.path(),
        cwd.path(),
        &[],
        &["config", "set", "general.default_backend", "git2"],
    );

    let from_env = ok(
        config.path(),
        cwd.path(),
        &[("GITPOULPI_GENERAL_DEFAULT_BACKEND", "system")],
        &["config", "get", "general.default_backend"],
    );
    assert_eq!(from_env["value"], Value::from("system"));
    assert_eq!(from_env["source"], Value::from("env"));

    // The flag beats the env var, not merely the file -- otherwise an
    // exported variable in a shell profile would quietly win over what
    // the user just typed.
    let from_flag = ok(
        config.path(),
        cwd.path(),
        &[("GITPOULPI_GENERAL_DEFAULT_BACKEND", "system")],
        &[
            "--backend",
            "git2",
            "config",
            "get",
            "general.default_backend",
        ],
    );
    assert_eq!(from_flag["value"], Value::from("git2"));
    assert_eq!(from_flag["source"], Value::from("flag"));
}

/// `config set` must write the *file* layer, and only the key it was
/// given: writing back the resolved value would bake a one-off
/// `GITPOULPI_*` override into config.toml, and writing back a full
/// `Config` would make every untouched default look deliberate.
#[test]
fn config_set_records_the_file_layer_not_whatever_the_environment_said() {
    let config = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();

    ok(
        config.path(),
        cwd.path(),
        &[("GITPOULPI_GENERAL_DEFAULT_BACKEND", "system")],
        &["config", "set", "ui.mouse_enabled", "false"],
    );

    let stored = ok(
        config.path(),
        cwd.path(),
        &[],
        &["config", "get", "general.default_backend"],
    );
    assert_eq!(stored["value"], Value::from("git2"));
    assert_eq!(stored["source"], Value::from("default"));
}

/// The whole point of this command group: a user can tell an ignored key
/// from a misspelled one without running an experiment.
#[test]
fn config_get_says_whether_anything_actually_reads_the_key() {
    let config = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();

    for key in [
        "general.default_backend",
        "ui.mouse_enabled",
        "ui.theme",
        "mcp.enabled",
        "general.auto_stash",
        "credentials.https_mode",
        "session.per_repo_state_dir",
    ] {
        let json = ok(config.path(), cwd.path(), &[], &["config", "get", key]);
        assert_eq!(json["implemented"], Value::Bool(true), "{key}: {json}");
    }
}

#[test]
fn an_unknown_key_fails_instead_of_reporting_null() {
    let config = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();

    let (success, json) = run(
        config.path(),
        cwd.path(),
        &[],
        &["config", "get", "general.no_such_key"],
    );
    assert!(!success, "{json}");
    assert!(json["error"].is_string(), "{json}");
}

#[test]
fn a_rejected_value_leaves_the_stored_config_untouched() {
    let config = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    ok(
        config.path(),
        cwd.path(),
        &[],
        &["config", "set", "general.default_backend", "system"],
    );

    let (success, json) = run(
        config.path(),
        cwd.path(),
        &[],
        &["config", "set", "general.default_backend", "mercurial"],
    );
    assert!(!success, "{json}");

    let stored = ok(
        config.path(),
        cwd.path(),
        &[],
        &["config", "get", "general.default_backend"],
    );
    assert_eq!(stored["value"], Value::from("system"));
}

/// A `GITPOULPI_*` variable with a value the schema does not accept is an
/// error, not a silent fall-through to the file layer - those two look
/// identical from outside, and the second teaches the user that the
/// variable is ignored.
#[test]
fn a_bad_environment_override_is_reported_rather_than_ignored() {
    let config = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();

    let (success, json) = run(
        config.path(),
        cwd.path(),
        &[("GITPOULPI_GENERAL_DEFAULT_BACKEND", "mercurial")],
        &["config", "get", "general.default_backend"],
    );
    assert!(!success, "{json}");
    let message = json["error"].as_str().unwrap();
    assert!(
        message.contains("GITPOULPI_GENERAL_DEFAULT_BACKEND"),
        "{message}"
    );
}

/// The proof that `general.default_backend` reaches a real daemon.
///
/// The discriminator is a `PATH` with no `git` on it: `Git2Backend` reads
/// through libgit2 and answers anyway, `SystemGitBackend` shells out and
/// cannot. Both halves are asserted, so a substituted `PATH` that failed
/// to hide git fails loudly instead of passing for the wrong reason.
///
/// `PATH` is a real empty directory, not `""`, which the C library on Unix
/// and `CreateProcess` on Windows interpret differently.
///
/// It is `status` rather than `log` because `SystemGitBackend::log` turns
/// any `git log` failure into an empty graph (an empty repository makes
/// it fail legitimately), which swallows a failure to spawn git at all.
/// `status` propagates.
#[test]
fn the_configured_backend_is_the_one_the_daemon_actually_serves_from() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    // Something for the git2 read to actually find, so a working answer
    // cannot be confused with an empty one.
    std::fs::write(repo.root().join("new.txt"), "hello").unwrap();

    let config = tempfile::tempdir().unwrap();
    let no_git = tempfile::tempdir().unwrap();
    let path_without_git = no_git.path().to_string_lossy().into_owned();
    let root = repo.root().to_string_lossy().into_owned();

    let with_git2 = ok(
        config.path(),
        repo.root(),
        &[("PATH", &path_without_git)],
        &[
            "--backend",
            "git2",
            "--session",
            "cli-test-backend-git2",
            "--repo",
            &root,
            "status",
        ],
    );
    let tree: WorkingTree = serde_json::from_value(with_git2).unwrap();
    assert_eq!(tree.files.len(), 1);
    assert_eq!(tree.files[0].path.to_str(), Some("new.txt"));

    let (success, json) = run(
        config.path(),
        repo.root(),
        &[("PATH", &path_without_git)],
        &[
            "--backend",
            "system",
            "--session",
            "cli-test-backend-system",
            "--repo",
            &root,
            "status",
        ],
    );
    assert!(
        !success,
        "the system-git backend answered with no git on PATH, so this test \
         proves nothing about which backend served the git2 run: {json}"
    );
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
