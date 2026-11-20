use super::*;
use gitpoulpi_update::job::UpdateJob;

const TARGET: &str = "x86_64-unknown-linux-gnu";

fn inputs(dir: &Path) -> Inputs {
    Inputs {
        action: Action::Apply,
        install_dir: dir.join("install"),
        from_version: "0.1.0".into(),
        to_version: "0.2.0".into(),
        staged: Staged::InPlace,
        manager_argv: None,
        elevation: Elevation::None,
        own_exe: dir.join("install").join("gitpoulpi-tui"),
        args: vec!["--repo".into(), "x".into()],
        cwd: dir.to_path_buf(),
        pid: 4242,
        config_dir: dir.join("config"),
        status_path: dir.join("data").join("updates").join("status.json"),
        environment: None,
        install_id: None,
    }
}

#[test]
fn the_job_carries_every_field_the_updater_reads() {
    let dir = tempfile::tempdir().unwrap();
    let job = build_job(Inputs {
        manager_argv: Some(vec!["brew".into(), "upgrade".into(), "gitpoulpi".into()]),
        action: Action::Manager,
        elevation: Elevation::Uac,
        ..inputs(dir.path())
    });
    // The wire format round-trips through the updater's own parser.
    let parsed = UpdateJob::parse(&job.render()).unwrap();
    assert_eq!(parsed, job);
    assert_eq!(parsed.action, Action::Manager);
    assert_eq!(parsed.elevation, JobElevation::Uac);
    assert_eq!(parsed.requester_pid, 4242);
    assert_eq!(parsed.relaunch.args, ["--repo", "x"]);
    assert_eq!(parsed.relaunch.cwd, dir.path());
    assert_eq!(parsed.from_version, "0.1.0");
    assert_eq!(parsed.to_version, "0.2.0");
}

#[test]
fn a_staged_new_updater_is_preferred_over_the_installed_one() {
    let dir = Path::new("/opt/gp");
    let has = |names: &'static [&'static str]| {
        move |p: &Path| names.iter().any(|n| p == dir.join(n).as_path())
    };
    assert_eq!(
        pick_updater(
            dir,
            TARGET,
            &has(&["gitpoulpi-updater", "gitpoulpi-updater.new"])
        ),
        Some(dir.join("gitpoulpi-updater.new"))
    );
    assert_eq!(
        pick_updater(dir, TARGET, &has(&["gitpoulpi-updater"])),
        Some(dir.join("gitpoulpi-updater"))
    );
    assert_eq!(pick_updater(dir, TARGET, &has(&[])), None);
    assert_eq!(
        updater_name("x86_64-pc-windows-msvc"),
        "gitpoulpi-updater.exe"
    );
}

#[test]
fn prepare_writes_the_job_and_copies_the_updater_out_of_the_install_folder() {
    let _guard = crate::updates::TEST_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let dir = tempfile::tempdir().unwrap();
    let install = dir.path().join("install");
    std::fs::create_dir_all(&install).unwrap();
    std::fs::write(install.join("gitpoulpi-updater"), b"installed").unwrap();
    std::fs::write(install.join("gitpoulpi-updater.new"), b"new one").unwrap();
    let paths = paths_in(&dir.path().join("data"));
    let job = build_job(inputs(dir.path()));
    let prepared = prepare(&job, &paths, TARGET).unwrap();
    assert_eq!(prepared.updater, paths.bin.join("gitpoulpi-updater"));
    assert_eq!(std::fs::read(&prepared.updater).unwrap(), b"new one");
    let written = std::fs::read_to_string(&paths.job).unwrap();
    assert_eq!(UpdateJob::parse(&written).unwrap(), job);
    // The install folder keeps its own files untouched.
    assert_eq!(
        std::fs::read(install.join("gitpoulpi-updater")).unwrap(),
        b"installed"
    );
    // A second handoff replaces the copy even though the first still exists.
    std::fs::write(install.join("gitpoulpi-updater.new"), b"newer").unwrap();
    prepare(&job, &paths, TARGET).unwrap();
    assert_eq!(std::fs::read(&prepared.updater).unwrap(), b"newer");
}

#[test]
fn an_install_without_an_updater_says_so_and_changes_nothing() {
    let _guard = crate::updates::TEST_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("install")).unwrap();
    let paths = paths_in(&dir.path().join("data"));
    let err = prepare(&build_job(inputs(dir.path())), &paths, TARGET).unwrap_err();
    assert!(err.contains("no updater"), "{err}");
    assert!(!paths.job.exists());
}

#[test]
fn the_status_becomes_one_line_and_a_final_one_is_read_once() {
    let done = UpdateStatus::new(Phase::Done, "ok", 1000);
    assert_eq!(
        status_line(&done, "0.2.0", 1010).as_deref(),
        Some(msg::msg_updated("0.2.0").as_str())
    );
    let failed = UpdateStatus::new(Phase::Failed, "the checksum did not match", 1000);
    let line = status_line(&failed, "0.1.0", 1010).unwrap();
    assert!(line.contains("the checksum did not match"), "{line}");
    assert!(
        line.contains("previous version is still installed"),
        "{line}"
    );
    assert_eq!(
        status_line(&UpdateStatus::new(Phase::Declined, "", 1000), "0.1.0", 1010).as_deref(),
        Some(msg::MSG_UPDATE_DECLINED.get())
    );
    // Still running, or a day old: no line.
    assert_eq!(
        status_line(&UpdateStatus::new(Phase::Working, "", 1000), "0.1.0", 1010),
        None
    );
    assert_eq!(status_line(&done, "0.2.0", 1000 + 2 * 86_400), None);

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("status.json");
    std::fs::write(&path, done.render()).unwrap();
    assert!(startup_status(&path, "0.2.0", 1010).is_some());
    assert!(!path.exists(), "consumed");
    assert_eq!(startup_status(&path, "0.2.0", 1010), None);
    // A running update's status stays for the updater to finish.
    std::fs::write(&path, UpdateStatus::new(Phase::Working, "", 1000).render()).unwrap();
    assert_eq!(startup_status(&path, "0.1.0", 1010), None);
    assert!(path.exists());
}
