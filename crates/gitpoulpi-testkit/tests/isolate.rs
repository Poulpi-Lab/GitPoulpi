//! `isolate_binary!` must have cut this process off from the real user state
//! before any test thread started. The tests never call
//! `isolate_process_env` themselves: only the pre-main hook can have run.

use std::path::PathBuf;

gitpoulpi_testkit::isolate_binary!();

fn var(name: &str) -> PathBuf {
    PathBuf::from(std::env::var_os(name).unwrap_or_else(|| panic!("{name} is not set")))
}

fn root() -> PathBuf {
    let root = var("GITPOULPI_CONFIG_DIR").parent().unwrap().to_path_buf();
    assert!(
        root.file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("gitpoulpi-isolate-"),
        "{root:?}"
    );
    root
}

#[test]
fn the_process_environment_points_into_the_isolation_dir() {
    let root = root();
    let mut names = vec![
        "GITPOULPI_CONFIG_DIR",
        "XDG_CONFIG_HOME",
        "XDG_DATA_HOME",
        "GIT_CONFIG_GLOBAL",
    ];
    // Windows keeps the real home and app data dirs (see `isolate.rs`).
    if cfg!(not(windows)) {
        names.extend(["HOME", "USERPROFILE", "APPDATA", "LOCALAPPDATA"]);
    }
    for name in names {
        assert!(
            var(name).starts_with(&root),
            "{name} = {:?} is outside {root:?}",
            var(name)
        );
    }
    assert_eq!(std::env::var("GIT_CONFIG_NOSYSTEM").as_deref(), Ok("1"));
}

#[test]
fn the_credential_helper_is_a_file_inside_the_isolation_dir() {
    let root = root();
    let out = std::process::Command::new("git")
        .args(["config", "--global", "--get-all", "credential.helper"])
        .output()
        .expect("run git");
    let helpers = String::from_utf8_lossy(&out.stdout).replace('\\', "/");
    let creds = root
        .join("git-credentials")
        .to_string_lossy()
        .replace('\\', "/");
    assert_eq!(helpers.lines().count(), 1, "{helpers}");
    assert!(
        helpers.contains("store --file") && helpers.contains(&creds),
        "{helpers}"
    );
}
