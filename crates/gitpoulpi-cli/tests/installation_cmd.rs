//! `gitpoulpi installation show|set` and the installation each alias belongs to
//! in `gitpoulpi alias status`. JSON assertions; HOME, the config folder and
//! PATH point into a scratch directory, so nothing real is touched.

#![cfg(unix)]

use std::path::Path;

use assert_cmd::Command;
use serde_json::Value;

fn command(exe: Option<&Path>, scratch: &Path) -> Command {
    let config = scratch.join("cfg");
    std::fs::create_dir_all(&config).unwrap();
    let home = scratch.join("home");
    std::fs::create_dir_all(&home).unwrap();
    let mut cmd = match exe {
        Some(exe) => Command::new(exe),
        None => Command::cargo_bin("gitpoulpi").unwrap(),
    };
    cmd.env("GITPOULPI_CONFIG_DIR", &config)
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .env("HOME", &home)
        .env("XDG_DATA_HOME", scratch.join("data"))
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("PATH", "/usr/bin:/bin")
        .env_remove("LOCALAPPDATA")
        .env_remove("GITPOULPI_INSTALL_ID")
        .env_remove("GITPOULPI_ENV")
        .current_dir(scratch)
        .arg("--json");
    cmd
}

fn json_of(mut cmd: Command, args: &[&str]) -> (bool, Value) {
    let output = cmd.args(args).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let json = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "not JSON ({e}): {stdout} / {}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output.status.success(), json)
}

fn run(scratch: &Path, args: &[&str]) -> (bool, Value) {
    json_of(command(None, scratch), args)
}

#[test]
fn installation_show_json_has_id_kind_path_env_primary() {
    let tmp = tempfile::tempdir().unwrap();
    let mut cmd = command(None, tmp.path());
    cmd.env("GITPOULPI_INSTALL_ID", "tester");
    let (ok, show) = json_of(cmd, &["installation", "show"]);
    assert!(ok, "{show}");
    assert_eq!(show["id"], "p-tester");
    assert_eq!(show["kind"], "pinned");
    assert!(
        show["path"].as_str().is_some_and(|p| !p.is_empty()),
        "{show}"
    );
    assert_eq!(show["env"], "p-tester", "its own environment by default");
    assert_eq!(show["primary"], false, "nobody is primary by default");
    assert!(show["short"].as_str().unwrap().starts_with('i'));
    assert!(show["settings"]["name"]
        .as_str()
        .unwrap()
        .starts_with("GitPoulpi ("));
    assert!(show["others"].as_array().unwrap().is_empty());

    // The environment can be shared by a setting, and forced by GITPOULPI_ENV.
    let mut cmd = command(None, tmp.path());
    cmd.env("GITPOULPI_INSTALL_ID", "tester");
    let (ok, set) = json_of(cmd, &["installation", "set", "env", "shared"]);
    assert!(ok, "{set}");
    let mut cmd = command(None, tmp.path());
    cmd.env("GITPOULPI_INSTALL_ID", "tester");
    let (_, show) = json_of(cmd, &["installation", "show"]);
    assert_eq!(show["env"], "shared");
    assert_eq!(show["settings"]["env"], "shared");
    let mut cmd = command(None, tmp.path());
    cmd.env("GITPOULPI_INSTALL_ID", "tester")
        .env("GITPOULPI_ENV", "forced");
    let (_, show) = json_of(cmd, &["installation", "show"]);
    assert_eq!(show["env"], "forced");

    // A bad field or value is refused.
    let (ok, err) = run(tmp.path(), &["installation", "set", "nope", "x"]);
    assert!(!ok && err["error"].as_str().unwrap().contains("unknown field"));
    let (ok, _) = run(tmp.path(), &["installation", "set", "command_alias", "a/b"]);
    assert!(!ok);
}

#[test]
fn alias_status_json_names_each_aliases_installation() {
    let tmp = tempfile::tempdir().unwrap();
    // A second installation: a copy of the program in another folder.
    let other_dir = tmp.path().join("other");
    std::fs::create_dir_all(&other_dir).unwrap();
    let other_exe = other_dir.join("gitpoulpi");
    std::fs::copy(assert_cmd::cargo::cargo_bin("gitpoulpi"), &other_exe).unwrap();

    let (ok, theirs) = json_of(
        command(Some(&other_exe), tmp.path()),
        &["installation", "show"],
    );
    assert!(ok, "{theirs}");
    let other_id = theirs["id"].as_str().unwrap().to_string();
    assert_eq!(theirs["kind"], "portable");
    let (ok, made) = json_of(
        command(Some(&other_exe), tmp.path()),
        &["alias", "create", "gpo"],
    );
    assert!(ok, "{made}");

    let (ok, made) = run(tmp.path(), &["alias", "create", "gp"]);
    assert!(ok, "{made}");
    let (ok, status) = run(tmp.path(), &["alias", "status"]);
    assert!(ok, "{status}");
    let aliases = status["aliases"].as_array().unwrap();
    let by_name = |name: &str| aliases.iter().find(|a| a["name"] == name).unwrap();
    let mine = by_name("gp");
    assert_eq!(mine["state"], "installed");
    assert_eq!(mine["installation"]["mine"], true);
    let theirs = by_name("gpo");
    assert_eq!(theirs["state"], "other", "never outdated: {theirs}");
    assert_eq!(theirs["installation"]["id"], other_id);
    assert_eq!(theirs["installation"]["mine"], false);
    assert_ne!(mine["installation"]["id"], theirs["installation"]["id"]);

    // Create and remove refuse the other's name; remove-all leaves it.
    let (ok, err) = run(tmp.path(), &["alias", "create", "gpo"]);
    assert!(
        !ok && err["error"]
            .as_str()
            .unwrap()
            .contains("another installation")
    );
    let (ok, removed) = run(tmp.path(), &["alias", "remove"]);
    assert!(ok, "{removed}");
    assert_eq!(removed["removed"], serde_json::json!(["gp"]));
    assert!(std::fs::symlink_metadata(tmp.path().join("home/.local/bin/gpo")).is_ok());

    // `alias create` recorded the name in the installation's settings.
    let (_, show) = json_of(
        command(Some(&other_exe), tmp.path()),
        &["installation", "show"],
    );
    assert_eq!(show["settings"]["command_alias"], "gpo");
    assert!(show["others"]
        .as_array()
        .unwrap()
        .iter()
        .any(|o| o["name"].is_string()));
}
