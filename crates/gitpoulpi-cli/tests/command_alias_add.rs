//! `gitpoulpi alias create <name> --add`: the `gpa` variant that runs
//! `gitpoulpi --add`. JSON assertions; HOME, XDG_* and PATH point into a
//! scratch directory so the real home and `~/.local/bin` are never touched.

#![cfg(unix)]

use std::path::Path;

use assert_cmd::Command;
use serde_json::Value;

fn run(scratch: &Path, args: &[&str]) -> (bool, Value) {
    let config = scratch.join("cfg");
    std::fs::create_dir_all(&config).unwrap();
    let home = scratch.join("home");
    std::fs::create_dir_all(&home).unwrap();
    let output = Command::cargo_bin("gitpoulpi")
        .unwrap()
        .env("GITPOULPI_CONFIG_DIR", &config)
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .env("HOME", &home)
        .env("XDG_DATA_HOME", scratch.join("data"))
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("PATH", "/usr/bin:/bin")
        .env_remove("LOCALAPPDATA")
        .current_dir(scratch)
        .arg("--json")
        .args(args)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let json = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "not JSON ({e}): {stdout} / {}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output.status.success(), json)
}

#[test]
fn the_add_variant_is_a_script_that_runs_the_tui_with_add_and_is_listed_by_kind() {
    let tmp = tempfile::tempdir().unwrap();
    let script = tmp.path().join("home/.local/bin/gpa");

    let (ok, created) = run(tmp.path(), &["alias", "create", "gpa", "--add"]);
    assert!(ok, "{created}");
    assert_eq!(created["created"], "gpa");
    assert_eq!(created["kind"], "add");
    let text = std::fs::read_to_string(&script).unwrap();
    let target = created["target"].as_str().unwrap();
    assert!(
        text.contains(&format!("exec '{target}' --add \"$@\"")),
        "{text}"
    );
    assert!(text.contains("X-GitPoulpi=1"), "{text}");

    let (ok, plain) = run(tmp.path(), &["alias", "create", "gp"]);
    assert!(ok, "{plain}");
    assert_eq!(plain["kind"], "open");

    let (_, status) = run(tmp.path(), &["alias", "status"]);
    let aliases = status["aliases"].as_array().unwrap();
    let find = |name: &str| aliases.iter().find(|a| a["name"] == name).unwrap();
    assert_eq!(find("gp")["kind"], "open");
    assert_eq!(find("gpa")["kind"], "add");
    assert_eq!(find("gpa")["state"], "installed");

    // Remove with no name takes out both kinds.
    let (_, removed) = run(tmp.path(), &["alias", "remove"]);
    let mut names: Vec<_> = removed["removed"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_str().unwrap().to_string())
        .collect();
    names.sort();
    assert_eq!(names, ["gp", "gpa"]);
    assert!(!script.exists());
}

#[test]
fn the_configured_add_name_is_reported_by_status() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join("cfg")).unwrap();
    std::fs::write(
        tmp.path().join("cfg/config.toml"),
        "[ui]\ncommand_alias = \"gp\"\ncommand_alias_add = \"gpa\"\n",
    )
    .unwrap();
    let (ok, status) = run(tmp.path(), &["alias", "status"]);
    assert!(ok, "{status}");
    assert_eq!(status["configured"]["name"], "gp");
    assert_eq!(status["configured_add"]["name"], "gpa");
    assert_eq!(status["configured_add"]["kind"], "add");
    assert_eq!(status["configured_add"]["state"], "not_installed");
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
