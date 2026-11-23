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

/// A config a 1.0 dev build left: `ui.*` holds `gpd` / `gpda` and the dev
/// installation's entry (its folder exists) claims both.
fn write_legacy_dev_config(scratch: &Path, primary: Option<&str>) {
    let cfg = scratch.join("cfg");
    std::fs::create_dir_all(&cfg).unwrap();
    let dev_dir = scratch.join("dev");
    std::fs::create_dir_all(&dev_dir).unwrap();
    let primary = primary
        .map(|p| format!("primary = \"{p}\"\n"))
        .unwrap_or_default();
    std::fs::write(
        cfg.join("config.toml"),
        format!(
            "[ui]\ncommand_alias = \"gpd\"\ncommand_alias_add = \"gpda\"\n\n[installation]\n{primary}\n[installation.settings.p-dev]\npath = \"{}\"\nkind = \"portable\"\ncommand_alias = \"gpd\"\ncommand_alias_add = \"gpda\"\n\n[installation.settings.p-choco]\npath = \"/opt/choco\"\nkind = \"portable\"\n",
            dev_dir.display()
        ),
    )
    .unwrap();
}

fn as_choco(scratch: &Path) -> Command {
    let mut cmd = command(None, scratch);
    cmd.env("GITPOULPI_INSTALL_ID", "choco");
    cmd
}

#[test]
fn a_non_primary_has_no_alias_of_its_own_until_alias_create_records_one() {
    let tmp = tempfile::tempdir().unwrap();
    write_legacy_dev_config(tmp.path(), Some("p-dev"));
    let (ok, show) = json_of(as_choco(tmp.path()), &["installation", "show"]);
    assert!(ok, "{show}");
    assert_eq!(show["primary"], false);
    assert_eq!(show["settings"]["command_alias"], "");
    assert_eq!(show["settings"]["command_alias_add"], "");
    let (ok, made) = json_of(as_choco(tmp.path()), &["alias", "create", "gp"]);
    assert!(ok, "{made}");
    let (_, show) = json_of(as_choco(tmp.path()), &["installation", "show"]);
    assert_eq!(show["settings"]["command_alias"], "gp", "{show}");
    assert_eq!(show["settings"]["command_alias_add"], "");
    // The shared keys older builds read are left alone.
    let text = std::fs::read_to_string(tmp.path().join("cfg/config.toml")).unwrap();
    assert!(text.contains("command_alias = \"gpd\""), "{text}");
}

#[test]
fn the_primary_does_not_take_a_legacy_alias_the_dev_installation_claims() {
    let tmp = tempfile::tempdir().unwrap();
    write_legacy_dev_config(tmp.path(), Some("p-choco"));
    // The untagged add script a 1.0 dev build made, running the dev's program.
    let bin = tmp.path().join("home/.local/bin");
    std::fs::create_dir_all(&bin).unwrap();
    let legacy = "#!/bin/sh\n# X-GitPoulpi=1\nexec '/dev/gitpoulpi' --add \"$@\"\n";
    std::fs::write(bin.join("gpda"), legacy).unwrap();
    let (ok, show) = json_of(as_choco(tmp.path()), &["installation", "show"]);
    assert!(ok, "{show}");
    assert_eq!(show["primary"], true);
    assert_eq!(
        show["settings"]["command_alias_add"], "",
        "the dev's name is not the primary's: {show}"
    );
    let (ok, status) = json_of(as_choco(tmp.path()), &["alias", "status"]);
    assert!(ok, "{status}");
    if let Some(gpda) = status["aliases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["name"] == "gpda")
    {
        assert_eq!(gpda["state"], "other", "{gpda}");
        assert_eq!(gpda["installation"]["id"], "p-dev", "{gpda}");
    }
    let (ok, err) = json_of(as_choco(tmp.path()), &["alias", "create", "gpda", "--add"]);
    assert!(
        !ok && err["error"]
            .as_str()
            .unwrap()
            .contains("another installation"),
        "{err}"
    );
    assert_eq!(std::fs::read_to_string(bin.join("gpda")).unwrap(), legacy);
}
