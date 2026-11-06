//! The real `gitpoulpi-updater` binary on real files: the command line, a
//! bad job, and (Unix) a whole update of a temp install whose "binaries" are
//! shell scripts, ending in the `exec` of the swapped-in file.
//!
//! The policy file is read from the OS location (`/etc/gitpoulpi/...`), which
//! a test machine does not have; no config or data folder of the user is
//! touched: every path comes from the job.

use std::path::Path;

use assert_cmd::Command;
use gitpoulpi_config::release::{self, InstallChannel, Marker, MARKER_FILE};
use gitpoulpi_update::job::{
    Action, JobElevation, Relaunch, Staged, UpdateJob, UpdateStatus, SCHEMA,
};

fn updater() -> Command {
    Command::cargo_bin("gitpoulpi-updater").unwrap()
}

#[test]
fn it_prints_its_version_and_refuses_unknown_commands() {
    let out = updater().arg("--version").output().unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).starts_with("gitpoulpi-updater "));
    assert_eq!(
        updater().arg("frobnicate").output().unwrap().status.code(),
        Some(6)
    );
    assert_eq!(
        updater().args(["run"]).output().unwrap().status.code(),
        Some(6)
    );
}

#[test]
fn a_missing_or_foreign_job_is_exit_6() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("none.json");
    let out = updater()
        .args(["run", "--job"])
        .arg(&missing)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(6));

    let future = dir.path().join("job.json");
    std::fs::write(&future, r#"{"schema": 99}"#).unwrap();
    let out = updater()
        .args(["run", "--job"])
        .arg(&future)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(6));
    assert!(String::from_utf8_lossy(&out.stderr).contains("schema"));
}

fn script(version: &str) -> String {
    format!("#!/bin/sh\necho \"gitpoulpi {version} (test)\"\n")
}

fn write_exec(path: &Path, text: &str) {
    std::fs::write(path, text).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
}

/// Version 1.0.0 installed, 2.0.0 staged, everything a script.
#[cfg(unix)]
#[test]
fn a_whole_update_swaps_verifies_and_execs_the_new_file() {
    let tmp = tempfile::tempdir().unwrap();
    let install = tmp.path().join("install");
    let updates = tmp.path().join("updates");
    std::fs::create_dir_all(&install).unwrap();
    std::fs::create_dir_all(&updates).unwrap();
    let target = release::host_target();
    for name in release::BINARIES {
        write_exec(&install.join(name), &script("1.0.0"));
        let new_text = if name == "gitpoulpi-tui" {
            // What the relaunch runs: proves the swapped-in file was executed.
            "#!/bin/sh\necho \"RELAUNCHED $*\"\n".to_string()
        } else {
            script("2.0.0")
        };
        write_exec(&install.join(format!("{name}.new")), &new_text);
    }
    let marker = |v: &str| Marker {
        channel: Some(InstallChannel::Archive),
        ..Marker::new(v, &target)
    };
    std::fs::write(install.join(MARKER_FILE), marker("1.0.0").render()).unwrap();
    std::fs::write(
        install.join(format!("{MARKER_FILE}.new")),
        marker("2.0.0").render(),
    )
    .unwrap();

    // A pid that is certainly gone.
    let mut gone = std::process::Command::new("true").spawn().unwrap();
    let requester_pid = gone.id();
    gone.wait().unwrap();

    let job = UpdateJob {
        schema: SCHEMA,
        action: Action::Apply,
        install_dir: install.clone(),
        from_version: "1.0.0".into(),
        to_version: "2.0.0".into(),
        staged: Staged::InPlace,
        manager_argv: None,
        elevation: JobElevation::None,
        relaunch: Relaunch {
            exe: install.join("gitpoulpi-tui"),
            args: vec!["--again".into()],
            cwd: tmp.path().to_path_buf(),
        },
        requester_pid,
        config_dir: tmp.path().join("config"),
        status_path: updates.join("status.json"),
    };
    let job_path = updates.join("job.json");
    std::fs::write(&job_path, job.render()).unwrap();

    let out = updater()
        .args(["run", "--job"])
        .arg(&job_path)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    // `exec` replaced the updater with the new `gitpoulpi-tui` script.
    assert!(
        stdout.contains("RELAUNCHED --again"),
        "{stdout}\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.status.success());

    let installed = std::fs::read_to_string(install.join("gitpoulpi")).unwrap();
    assert!(installed.contains("2.0.0"), "{installed}");
    assert!(std::fs::read_to_string(install.join("gitpoulpi.old"))
        .unwrap()
        .contains("1.0.0"));
    let status =
        UpdateStatus::parse(&std::fs::read_to_string(updates.join("status.json")).unwrap())
            .unwrap();
    assert!(
        status.is_final() && status.message.contains("2.0.0"),
        "{status:?}"
    );
    assert!(!updates.join("updating.lock").exists());
    assert!(!install.join(".gitpoulpi-updating").exists());
}
