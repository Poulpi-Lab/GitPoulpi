use super::*;

fn job() -> UpdateJob {
    UpdateJob {
        schema: SCHEMA,
        action: Action::Apply,
        install_dir: PathBuf::from("/opt/gp"),
        from_version: "1.4.1".into(),
        to_version: "1.4.2".into(),
        staged: Staged::InPlace,
        manager_argv: None,
        elevation: JobElevation::None,
        relaunch: Relaunch {
            exe: PathBuf::from("/opt/gp/gitpoulpi-tui"),
            args: vec!["--session".into(), "work".into()],
            cwd: PathBuf::from("/home/u/repo"),
        },
        requester_pid: 4242,
        config_dir: PathBuf::from("/home/u/.config/gitpoulpi"),
        status_path: PathBuf::from("/home/u/.local/share/gitpoulpi/updates/status.json"),
    }
}

#[test]
fn a_job_round_trips() {
    for staged in [
        Staged::InPlace,
        Staged::Archive {
            archive: PathBuf::from("/home/u/.cache/gp/gitpoulpi-1.4.2.zip"),
            sha256_url: "https://example.test/a.zip.sha256".into(),
        },
    ] {
        let j = UpdateJob { staged, ..job() };
        assert_eq!(UpdateJob::parse(&j.render()).unwrap(), j);
    }
    let m = UpdateJob {
        action: Action::Manager,
        manager_argv: Some(vec!["choco".into(), "upgrade".into(), "gitpoulpi".into()]),
        elevation: JobElevation::Uac,
        ..job()
    };
    assert_eq!(UpdateJob::parse(&m.render()).unwrap(), m);
}

#[test]
fn the_wire_format_is_the_documented_one() {
    let v: serde_json::Value = serde_json::from_str(&job().render()).unwrap();
    assert_eq!(v["schema"], 1);
    assert_eq!(v["action"], "apply");
    assert_eq!(v["staged"], "in_place");
    assert_eq!(v["elevation"], "none");
    assert!(v.get("manager_argv").is_none());
    let archive = UpdateJob {
        staged: Staged::Archive {
            archive: PathBuf::from("/c/a.zip"),
            sha256_url: "u".into(),
        },
        ..job()
    };
    let v: serde_json::Value = serde_json::from_str(&archive.render()).unwrap();
    assert_eq!(v["staged"]["archive"], "/c/a.zip");
    assert_eq!(v["staged"]["sha256_url"], "u");
}

#[test]
fn another_schema_is_refused_before_anything_else_is_read() {
    // A future job may have a different shape entirely.
    let future = r#"{"schema": 2, "whatever": true}"#;
    assert_eq!(
        UpdateJob::parse(future).unwrap_err(),
        JobError::Schema { found: 2 }
    );
    let mut v: serde_json::Value = serde_json::from_str(&job().render()).unwrap();
    v["schema"] = 0.into();
    assert_eq!(
        UpdateJob::parse(&v.to_string()).unwrap_err(),
        JobError::Schema { found: 0 }
    );
}

#[test]
fn garbage_and_missing_fields_are_invalid() {
    assert!(matches!(
        UpdateJob::parse("not json"),
        Err(JobError::Invalid(_))
    ));
    assert!(matches!(
        UpdateJob::parse(r#"{"schema": 1}"#),
        Err(JobError::Invalid(_))
    ));
    assert!(matches!(UpdateJob::parse("{}"), Err(JobError::Invalid(_))));
    let mut v: serde_json::Value = serde_json::from_str(&job().render()).unwrap();
    v["staged"] = "somewhere".into();
    assert!(matches!(
        UpdateJob::parse(&v.to_string()),
        Err(JobError::Invalid(_))
    ));
}

#[test]
fn relative_paths_are_refused() {
    let j = UpdateJob {
        install_dir: PathBuf::from("app"),
        ..job()
    };
    assert_eq!(
        UpdateJob::parse(&j.render()).unwrap_err(),
        JobError::RelativePath("install_dir")
    );
    let j = UpdateJob {
        status_path: PathBuf::from("status.json"),
        ..job()
    };
    assert_eq!(
        UpdateJob::parse(&j.render()).unwrap_err(),
        JobError::RelativePath("status_path")
    );
}

#[test]
fn a_manager_job_needs_a_command() {
    let j = UpdateJob {
        action: Action::Manager,
        ..job()
    };
    assert_eq!(
        UpdateJob::parse(&j.render()).unwrap_err(),
        JobError::NoManagerCommand
    );
}

#[test]
fn a_status_round_trips_and_a_foreign_one_is_ignored() {
    let s = UpdateStatus::new(Phase::Failed, "the checksum does not match", 1_700_000_000);
    assert_eq!(UpdateStatus::parse(&s.render()), Some(s.clone()));
    assert!(s.is_final());
    assert!(!UpdateStatus::new(Phase::Working, "", 0).is_final());
    assert_eq!(UpdateStatus::parse("garbage"), None);
    let other = s.render().replace("\"schema\":1", "\"schema\":9");
    assert_eq!(UpdateStatus::parse(&other), None);
}

#[test]
fn the_exit_codes_are_the_documented_ones() {
    assert_eq!(
        [
            exit::DONE,
            exit::FAILED,
            exit::ELEVATION_DECLINED,
            exit::NOTHING_TO_DO,
            exit::REFUSED_BY_POLICY,
            exit::WOULD_NOT_STOP,
            exit::BAD_JOB
        ],
        [0, 1, 2, 3, 4, 5, 6]
    );
}
