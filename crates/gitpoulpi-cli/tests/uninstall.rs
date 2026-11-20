//! `gitpoulpi uninstall`: JSON assertions in a scratch HOME. The exe is
//! copied next to a marker so it counts as a release install; the dev build
//! (no marker) is refused.

#![cfg(unix)]

use std::path::Path;

use assert_cmd::Command;
use serde_json::Value;

fn run(exe: &Path, scratch: &Path, args: &[&str]) -> (bool, Value) {
    let home = scratch.join("home");
    std::fs::create_dir_all(&home).unwrap();
    // The exe was just copied by this process: while a sibling test thread
    // forks, the child briefly inherits the still-open write fd and an exec
    // of the copy fails with ETXTBSY ("Text file busy"). That is the harness
    // racing itself, not the product, so only that error is retried.
    let mut attempt = 0;
    let output = loop {
        let mut cmd = Command::new(exe);
        cmd.env("GITPOULPI_CONFIG_DIR", scratch.join("cfg"))
            .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
            .env("HOME", &home)
            .env("XDG_DATA_HOME", scratch.join("data"))
            .env("XDG_CONFIG_HOME", home.join(".config"))
            .env("PATH", "/usr/bin:/bin")
            .env_remove("LOCALAPPDATA")
            .current_dir(scratch)
            .arg("--json")
            .args(args);
        match cmd.output() {
            Ok(output) => break output,
            Err(e) if e.raw_os_error() == Some(26) && attempt < 50 => {
                attempt += 1;
                std::thread::sleep(std::time::Duration::from_millis(40));
            }
            Err(e) => panic!("running {}: {e}", exe.display()),
        }
    };
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let json = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "not JSON ({e}): {stdout} / {}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output.status.success(), json)
}

fn built() -> std::path::PathBuf {
    assert_cmd::cargo::cargo_bin("gitpoulpi")
}

#[test]
fn a_development_build_is_refused() {
    let tmp = tempfile::tempdir().unwrap();
    let (ok, out) = run(&built(), tmp.path(), &["uninstall"]);
    assert!(!ok);
    assert!(out["error"].as_str().unwrap().contains("not a release"));
}

#[test]
fn a_release_install_loses_its_alias_path_block_and_files() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("opt");
    std::fs::create_dir_all(&dir).unwrap();
    let exe = dir.join("gitpoulpi");
    std::fs::copy(built(), &exe).unwrap();
    std::fs::write(dir.join("gitpoulpi-tui"), b"tui").unwrap();
    std::fs::write(dir.join("gitpoulpi.old"), b"old").unwrap();
    std::fs::write(dir.join("notes.txt"), b"mine").unwrap();
    std::fs::write(
        dir.join(".gitpoulpi-dist"),
        gitpoulpi_config::release::Marker::new("0.0.1", gitpoulpi_config::release::host_target())
            .render(),
    )
    .unwrap();
    let rc = tmp.path().join("home/.bashrc");
    std::fs::create_dir_all(rc.parent().unwrap()).unwrap();
    std::fs::write(&rc, "export A=1\n").unwrap();

    let (ok, made) = run(&exe, tmp.path(), &["alias", "create", "gp"]);
    assert!(ok, "{made}");
    let (ok, added) = run(&exe, tmp.path(), &["path", "add"]);
    assert!(ok, "{added}");
    assert!(std::fs::read_to_string(&rc).unwrap().len() > "export A=1\n".len());
    let link = tmp.path().join("home/.local/bin/gp");
    assert!(std::fs::symlink_metadata(&link).is_ok());

    let (ok, out) = run(&exe, tmp.path(), &["uninstall"]);
    assert!(ok, "{out}");
    assert_eq!(out["aliases_removed"][0], "gp");
    assert!(!out["path_changed"].as_array().unwrap().is_empty());
    assert_eq!(out["kept_files"], false);
    let removed: Vec<&str> = out["files_removed"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    for name in [
        "gitpoulpi",
        "gitpoulpi-tui",
        "gitpoulpi.old",
        ".gitpoulpi-dist",
    ] {
        assert!(removed.contains(&name), "{name} in {removed:?}");
    }
    assert_eq!(out["dir_removed"], false, "notes.txt keeps the folder");
    assert!(dir.join("notes.txt").is_file());
    assert!(!exe.exists());
    assert!(std::fs::symlink_metadata(&link).is_err());
    assert_eq!(std::fs::read_to_string(&rc).unwrap(), "export A=1\n");
}

#[test]
fn keep_files_leaves_the_program_alone() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("opt");
    std::fs::create_dir_all(&dir).unwrap();
    let exe = dir.join("gitpoulpi");
    std::fs::copy(built(), &exe).unwrap();
    std::fs::write(dir.join(".gitpoulpi-dist"), "version=0.0.1\ntarget=x\n").unwrap();
    let (ok, out) = run(&exe, tmp.path(), &["uninstall", "--keep-files"]);
    assert!(ok, "{out}");
    assert_eq!(out["kept_files"], true);
    assert!(exe.exists());
}

// ------------------------------------------------- several installations

/// One scratch machine with a shared config, home and a fake `nautilus` on the
/// PATH, and any number of installations (a copy of the binary beside a
/// release marker, each its own folder and so its own installation).
struct Machine {
    tmp: tempfile::TempDir,
}

impl Machine {
    fn new() -> Machine {
        let tmp = tempfile::tempdir().unwrap();
        let bin = tmp.path().join("fakebin");
        std::fs::create_dir_all(&bin).unwrap();
        let nautilus = bin.join("nautilus");
        std::fs::write(&nautilus, "#!/bin/sh\n").unwrap();
        make_executable(&nautilus);
        std::fs::create_dir_all(tmp.path().join("cfg")).unwrap();
        std::fs::write(
            tmp.path().join("cfg/config.toml"),
            "[ui]\nlaunch_terminal = \"kitty\"\n",
        )
        .unwrap();
        Machine { tmp }
    }

    fn scratch(&self) -> &Path {
        self.tmp.path()
    }

    /// An install in `<scratch>/<name>`; `marker`: it carries the release marker.
    fn install(&self, name: &str, marker: bool) -> std::path::PathBuf {
        let dir = self.tmp.path().join(name);
        std::fs::create_dir_all(&dir).unwrap();
        let exe = dir.join("gitpoulpi");
        std::fs::copy(built(), &exe).unwrap();
        if marker {
            std::fs::write(dir.join(".gitpoulpi-dist"), "version=0.0.1\ntarget=x\n").unwrap();
        }
        exe
    }

    fn run(&self, exe: &Path, args: &[&str]) -> (bool, Value) {
        let mut attempt = 0;
        let home = self.tmp.path().join("home");
        std::fs::create_dir_all(&home).unwrap();
        let path = format!("{}/fakebin:/usr/bin:/bin", self.tmp.path().display());
        let output = loop {
            let mut cmd = Command::new(exe);
            cmd.env("GITPOULPI_CONFIG_DIR", self.tmp.path().join("cfg"))
                .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
                .env("HOME", &home)
                .env("XDG_DATA_HOME", self.tmp.path().join("data"))
                .env("XDG_CONFIG_HOME", home.join(".config"))
                .env("PATH", &path)
                .env_remove("LOCALAPPDATA")
                .env_remove("GITPOULPI_INSTALL_ID")
                .env_remove("GITPOULPI_ENV")
                .current_dir(self.tmp.path())
                .arg("--json")
                .args(args);
            match cmd.output() {
                Ok(output) => break output,
                Err(e) if e.raw_os_error() == Some(26) && attempt < 50 => {
                    attempt += 1;
                    std::thread::sleep(std::time::Duration::from_millis(40));
                }
                Err(e) => panic!("running {}: {e}", exe.display()),
            }
        };
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let json = serde_json::from_str(&stdout).unwrap_or_else(|e| {
            panic!(
                "not JSON ({e}): {stdout} / {}",
                String::from_utf8_lossy(&output.stderr)
            )
        });
        (output.status.success(), json)
    }

    fn ok(&self, exe: &Path, args: &[&str]) -> Value {
        let (ok, out) = self.run(exe, args);
        assert!(ok, "{args:?}: {out}");
        out
    }

    fn applications(&self) -> std::path::PathBuf {
        self.tmp.path().join("data/applications")
    }

    fn scripts(&self) -> std::path::PathBuf {
        self.tmp.path().join("data/nautilus/scripts")
    }

    fn bin(&self) -> std::path::PathBuf {
        self.tmp.path().join("home/.local/bin")
    }

    fn desktop_files(&self) -> Vec<String> {
        names(&self.applications())
            .into_iter()
            .filter(|n| n.ends_with(".desktop"))
            .collect()
    }
}

fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

/// The sorted entries of `dir`, folders as `name/` and their files as `name/file`.
fn names(dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let Ok(read) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in read.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if entry.path().is_dir() {
            for inner in names(&entry.path()) {
                out.push(format!("{name}/{inner}"));
            }
            out.push(format!("{name}/"));
        } else {
            out.push(name);
        }
    }
    out.sort();
    out
}

fn folders(list: &[String]) -> usize {
    list.iter().filter(|n| n.ends_with('/')).count()
}

fn set_up(m: &Machine, exe: &Path, alias: &str, submenu: Option<&str>) {
    m.ok(exe, &["alias", "create", alias]);
    m.ok(exe, &["desktop", "shortcut", "install"]);
    if let Some(name) = submenu {
        m.ok(exe, &["installation", "set", "folder_menu_submenu", name]);
    }
    m.ok(exe, &["desktop", "folder-menu", "install"]);
}

#[test]
fn uninstall_keep_files_removes_only_this_installations_launchers_aliases_and_folder_entries() {
    let m = Machine::new();
    let a = m.install("a", true);
    let b = m.install("b", true);
    set_up(&m, &a, "gpa", None);
    set_up(&m, &b, "gpb", None);
    m.ok(&a, &["path", "add"]);
    m.ok(&b, &["path", "add"]);

    let launchers = m.desktop_files();
    assert_eq!(launchers.len(), 2, "{launchers:?}");
    assert_eq!(names(&m.scripts()).len(), 2, "{:?}", names(&m.scripts()));
    assert!(m.bin().join("gpa").exists() && m.bin().join("gpb").exists());

    let out = m.ok(&a, &["uninstall", "--keep-files"]);
    assert_eq!(out["kept_files"], true);
    assert_eq!(out["aliases_removed"], serde_json::json!(["gpa"]), "{out}");

    // A's alias, launcher and folder entry are gone; B keeps all three.
    assert!(std::fs::symlink_metadata(m.bin().join("gpa")).is_err());
    assert!(std::fs::symlink_metadata(m.bin().join("gpb")).is_ok());
    let left = m.desktop_files();
    assert_eq!(left.len(), 1, "{left:?}");
    assert!(launchers.contains(&left[0]));
    assert_eq!(names(&m.scripts()).len(), 1, "{:?}", names(&m.scripts()));
    assert!(a.exists(), "--keep-files deletes nothing");
    // The alias folder stays on the PATH while B has an alias there.
    let rc: String = [".bashrc", ".zshrc", ".profile"]
        .iter()
        .filter_map(|f| std::fs::read_to_string(m.scratch().join("home").join(f)).ok())
        .collect();
    assert!(rc.contains(".local/bin"), "PATH kept for B: {rc}");

    // B was untouched: its own uninstall finds and removes exactly its own.
    let out = m.ok(&b, &["uninstall", "--keep-files"]);
    assert_eq!(out["aliases_removed"], serde_json::json!(["gpb"]), "{out}");
    assert!(std::fs::symlink_metadata(m.bin().join("gpb")).is_err());
    assert!(m.desktop_files().is_empty());
    assert!(names(&m.scripts()).is_empty());
}

#[test]
fn uninstall_keep_files_works_without_a_marker_and_files_still_need_one() {
    let m = Machine::new();
    let dev = m.install("dev", false);
    set_up(&m, &dev, "gpd", None);
    assert!(std::fs::symlink_metadata(m.bin().join("gpd")).is_ok());

    // Deleting files is refused for a build without a marker, nothing is touched.
    let (ok, out) = m.run(&dev, &["uninstall"]);
    assert!(!ok);
    assert!(
        out["error"].as_str().unwrap().contains("not a release"),
        "{out}"
    );
    assert!(std::fs::symlink_metadata(m.bin().join("gpd")).is_ok());
    assert!(dev.exists());

    // Cleaning up the launchers and aliases needs no marker.
    let out = m.ok(&dev, &["uninstall", "--keep-files"]);
    assert_eq!(out["aliases_removed"], serde_json::json!(["gpd"]), "{out}");
    assert!(std::fs::symlink_metadata(m.bin().join("gpd")).is_err());
    assert!(m.desktop_files().is_empty());
    assert!(names(&m.scripts()).is_empty());
    assert!(dev.exists(), "the program stays");
}

#[test]
fn uninstall_leaves_a_shared_submenu_used_by_another_installation() {
    let m = Machine::new();
    let a = m.install("a", true);
    let b = m.install("b", true);
    set_up(&m, &a, "gpa", Some("Dev tools"));
    set_up(&m, &b, "gpb", Some("dev tools "));
    let shared = names(&m.scripts());
    assert_eq!(folders(&shared), 1, "one shared submenu folder: {shared:?}");
    assert_eq!(shared.len() - 1, 2, "{shared:?}");

    m.ok(&a, &["uninstall", "--keep-files"]);
    let after = names(&m.scripts());
    assert_eq!(folders(&after), 1, "{after:?}");
    assert_eq!(after.len() - 1, 1, "{after:?}");

    // The last one out takes the empty container with it.
    m.ok(&b, &["uninstall", "--keep-files"]);
    assert!(names(&m.scripts()).is_empty(), "{:?}", names(&m.scripts()));
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
