//! `gitpoulpi session list`, and the default-session rule both the CLI
//! and the TUI resolve through.
//!
//! Asserts on deserialized JSON: a `contains("live")` test passes just as
//! happily when the word appears in a column header.

use std::net::TcpListener;
use std::path::Path;

use assert_cmd::Command;
use serde_json::Value;

/// Every test here points the binary at its own throwaway config dir, so
/// none of them can see - or clobber - the developer's real live
/// sessions.
fn run(config_dir: &Path, cwd: &Path, args: &[&str]) -> Value {
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config_dir)
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "exit {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap_or_else(|err| {
        panic!(
            "not JSON ({err}): {}",
            String::from_utf8_lossy(&output.stdout)
        )
    })
}

/// A port that is bound for the duration of the test (returned listener
/// must be kept alive) and one that is definitely free, so "live" and
/// "stale" are real states rather than asserted constants.
fn bound_port() -> (TcpListener, u16) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    (listener, port)
}

fn free_port() -> u16 {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    port
}

fn write_port_file(config_dir: &Path, name: &str, port: u16) {
    let sessions = config_dir.join("sessions");
    std::fs::create_dir_all(&sessions).unwrap();
    std::fs::write(sessions.join(format!("{name}.port")), port.to_string()).unwrap();
}

#[test]
fn session_list_reports_liveness_per_session_rather_than_assuming_it() {
    let config = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let (_listener, live_port) = bound_port();
    write_port_file(config.path(), "alive", live_port);
    write_port_file(config.path(), "crashed", free_port());

    let json = run(config.path(), cwd.path(), &["session", "list", "--json"]);
    let sessions = json["sessions"].as_array().unwrap();
    assert_eq!(sessions.len(), 2, "{json}");

    let by_name = |name: &str| {
        sessions
            .iter()
            .find(|s| s["name"] == name)
            .unwrap_or_else(|| panic!("no {name} in {json}"))
            .clone()
    };
    assert_eq!(by_name("alive")["live"], Value::Bool(true));
    assert_eq!(by_name("alive")["port"], Value::from(live_port));
    // The whole point: a crash leaves the discovery file behind, and a
    // leftover file must not read as a running daemon.
    assert_eq!(by_name("crashed")["live"], Value::Bool(false));
}

#[test]
fn session_list_is_empty_when_no_daemon_has_ever_run() {
    let config = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let json = run(config.path(), cwd.path(), &["session", "list", "--json"]);
    assert_eq!(json["sessions"].as_array().unwrap().len(), 0, "{json}");
}

#[test]
fn the_default_session_is_shared_by_every_directory_inside_one_repo() {
    // Architecture principle 3: a CLI run deep in the tree and a TUI
    // opened at the root have to reach the same daemon, or a CLI mutation
    // stops showing up live in the open TUI.
    let config = tempfile::tempdir().unwrap();
    let repo = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(repo.path().join(".git")).unwrap();
    let deep = repo.path().join("crates").join("thing");
    std::fs::create_dir_all(&deep).unwrap();

    let at_root = run(config.path(), repo.path(), &["session", "list", "--json"]);
    let at_depth = run(config.path(), &deep, &["session", "list", "--json"]);
    assert_eq!(at_root["current"], at_depth["current"]);
    assert!(at_root["current"].as_str().unwrap().len() > 1, "{at_root}");
}

#[test]
fn two_repositories_get_two_default_sessions() {
    let config = tempfile::tempdir().unwrap();
    let alpha = tempfile::tempdir().unwrap();
    let beta = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(alpha.path().join(".git")).unwrap();
    std::fs::create_dir_all(beta.path().join(".git")).unwrap();

    let a = run(config.path(), alpha.path(), &["session", "list", "--json"]);
    let b = run(config.path(), beta.path(), &["session", "list", "--json"]);
    assert_ne!(a["current"], b["current"], "{a} vs {b}");
}

#[test]
fn an_explicit_session_flag_overrides_the_derived_default_and_is_sanitised() {
    let config = tempfile::tempdir().unwrap();
    let repo = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(repo.path().join(".git")).unwrap();

    let json = run(
        config.path(),
        repo.path(),
        &["-s", "feature/login", "session", "list", "--json"],
    );
    // Sanitised, because it becomes `sessions/<name>.port` -- unsanitised
    // this would be a path into a directory that does not exist.
    assert_eq!(json["current"], Value::from("feature-login"));
}

#[test]
fn an_explicit_session_flag_lets_two_repositories_share_one_daemon() {
    let config = tempfile::tempdir().unwrap();
    let alpha = tempfile::tempdir().unwrap();
    let beta = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(alpha.path().join(".git")).unwrap();
    std::fs::create_dir_all(beta.path().join(".git")).unwrap();

    let a = run(
        config.path(),
        alpha.path(),
        &["-s", "shared", "session", "list", "--json"],
    );
    let b = run(
        config.path(),
        beta.path(),
        &["-s", "shared", "session", "list", "--json"],
    );
    assert_eq!(a["current"], Value::from("shared"));
    assert_eq!(a["current"], b["current"]);
}

#[test]
fn the_current_session_is_flagged_in_the_listing() {
    let config = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let (_listener, port) = bound_port();
    write_port_file(config.path(), "mine", port);
    write_port_file(config.path(), "theirs", free_port());

    let json = run(
        config.path(),
        cwd.path(),
        &["-s", "mine", "session", "list", "--json"],
    );
    let sessions = json["sessions"].as_array().unwrap();
    let current: Vec<_> = sessions
        .iter()
        .filter(|s| s["current"] == Value::Bool(true))
        .map(|s| s["name"].clone())
        .collect();
    assert_eq!(current, vec![Value::from("mine")], "{json}");
}

/// Runs the binary and returns the whole output, whatever the exit status.
fn run_raw(config_dir: &Path, cwd: &Path, args: &[&str]) -> std::process::Output {
    Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config_dir)
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap()
}

fn error_of(output: &std::process::Output) -> String {
    assert!(!output.status.success(), "expected a failure");
    let json: Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "error was not JSON ({e}): {}",
            String::from_utf8_lossy(&output.stdout)
        )
    });
    json["error"].as_str().unwrap().to_string()
}

fn names(json: &Value) -> Vec<String> {
    json["sessions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["name"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn session_new_creates_a_saved_session_that_the_listing_shows() {
    let config = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();

    let created = run(
        config.path(),
        cwd.path(),
        &["session", "new", "work", "--json"],
    );
    assert_eq!(created["created"], Value::from("work"));
    assert!(config.path().join("sessions").join("work.toml").exists());

    let listing = run(config.path(), cwd.path(), &["session", "list", "--json"]);
    assert_eq!(names(&listing), vec!["work".to_string()]);
    let row = &listing["sessions"][0];
    assert_eq!(row["saved"], Value::Bool(true));
    assert_eq!(row["live"], Value::Bool(false));
    assert_eq!(row["port"], Value::Null);

    // It must not have started a daemon for the new name.
    assert!(!config.path().join("sessions").join("work.port").exists());

    let again = run_raw(
        config.path(),
        cwd.path(),
        &["session", "new", "work", "--json"],
    );
    assert!(error_of(&again).contains("already exists"));
}

#[test]
fn session_new_sanitises_the_name_like_the_session_flag() {
    let config = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let created = run(
        config.path(),
        cwd.path(),
        &["session", "new", "feature/login", "--json"],
    );
    assert_eq!(created["created"], Value::from("feature-login"));
}

#[test]
fn session_rename_keeps_tabs_and_layout() {
    let config = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(config.path().join("sessions")).unwrap();
    let saved = "[[tabs]]\nrepo_path = \"/tmp/some-repo\"\nactive = true\n\n[layout]\ndivider1_pct = 31\ndivider2_pct = 70\nleft_collapsed = true\nright_collapsed = false\nsidebar_heights = []\ncommit_message_rows = 4\n";
    std::fs::write(config.path().join("sessions").join("old.toml"), saved).unwrap();

    let json = run(
        config.path(),
        cwd.path(),
        &["session", "rename", "old", "fresh", "--json"],
    );
    assert_eq!(json["renamed"]["from"], Value::from("old"));
    assert_eq!(json["renamed"]["to"], Value::from("fresh"));
    assert!(!config.path().join("sessions").join("old.toml").exists());
    assert_eq!(
        std::fs::read_to_string(config.path().join("sessions").join("fresh.toml")).unwrap(),
        saved
    );

    let missing = run_raw(
        config.path(),
        cwd.path(),
        &["session", "rename", "old", "other", "--json"],
    );
    assert!(error_of(&missing).contains("no session named"));

    run(
        config.path(),
        cwd.path(),
        &["session", "new", "taken", "--json"],
    );
    let clash = run_raw(
        config.path(),
        cwd.path(),
        &["session", "rename", "fresh", "taken", "--json"],
    );
    assert!(error_of(&clash).contains("already exists"));
}

#[test]
fn session_delete_removes_the_session_and_its_stale_discovery_files() {
    let config = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    run(
        config.path(),
        cwd.path(),
        &["session", "new", "gone", "--json"],
    );
    write_port_file(config.path(), "gone", free_port());

    let json = run(
        config.path(),
        cwd.path(),
        &["session", "delete", "gone", "--json"],
    );
    assert_eq!(json["deleted"], Value::from("gone"));
    let listing = run(config.path(), cwd.path(), &["session", "list", "--json"]);
    assert!(names(&listing).is_empty(), "{listing}");
    assert!(!config.path().join("sessions").join("gone.port").exists());

    let missing = run_raw(
        config.path(),
        cwd.path(),
        &["session", "delete", "gone", "--json"],
    );
    assert!(error_of(&missing).contains("no session named"));
}

#[test]
fn session_delete_and_rename_refuse_while_a_daemon_is_live() {
    let config = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    run(
        config.path(),
        cwd.path(),
        &["session", "new", "busy", "--json"],
    );
    let (_listener, port) = bound_port();
    write_port_file(config.path(), "busy", port);

    let delete = run_raw(
        config.path(),
        cwd.path(),
        &["session", "delete", "busy", "--json"],
    );
    let message = error_of(&delete);
    assert!(message.contains("running daemon"), "{message}");
    assert!(message.contains("daemon stop"), "{message}");
    let rename = run_raw(
        config.path(),
        cwd.path(),
        &["session", "rename", "busy", "calm", "--json"],
    );
    assert!(error_of(&rename).contains("running daemon"));
    // Nothing was touched.
    assert!(config.path().join("sessions").join("busy.toml").exists());
    assert!(config.path().join("sessions").join("busy.port").exists());
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();

/// `run`, with `GITPOULPI_ENV` set in the child's environment only.
fn run_in_env(env: &str, config_dir: &Path, cwd: &Path, args: &[&str]) -> Value {
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", config_dir)
        .env("GITPOULPI_ENV", env)
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "exit {:?}: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn session_names(json: &Value) -> Vec<String> {
    json["sessions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["name"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn two_envs_list_only_their_own_sessions() {
    let config = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    run_in_env(
        "one",
        config.path(),
        cwd.path(),
        &["session", "new", "only-one", "--json"],
    );
    run_in_env(
        "two",
        config.path(),
        cwd.path(),
        &["session", "new", "only-two", "--json"],
    );

    let one = run_in_env(
        "one",
        config.path(),
        cwd.path(),
        &["session", "list", "--json"],
    );
    let two = run_in_env(
        "two",
        config.path(),
        cwd.path(),
        &["session", "list", "--json"],
    );
    assert_eq!(session_names(&one), vec!["only-one"], "{one}");
    assert_eq!(session_names(&two), vec!["only-two"], "{two}");
    assert!(config
        .path()
        .join("sessions/env/one/only-one.toml")
        .exists());
    assert!(config
        .path()
        .join("sessions/env/two/only-two.toml")
        .exists());
    // The flat layout is untouched and sees neither.
    let flat = run(config.path(), cwd.path(), &["session", "list", "--json"]);
    assert!(session_names(&flat).is_empty(), "{flat}");
}
