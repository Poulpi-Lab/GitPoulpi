use super::*;

/// A path that is absolute on the host: `/x` as is, `C:\x` on Windows.
fn abs(unix: &str) -> PathBuf {
    if cfg!(windows) {
        PathBuf::from(format!("C:{}", unix.replace('/', "\\")))
    } else {
        PathBuf::from(unix)
    }
}

fn job() -> UpdateJob {
    UpdateJob {
        schema: SCHEMA,
        action: Action::Apply,
        install_dir: abs("/opt/gp"),
        from_version: "1.4.1".into(),
        to_version: "1.4.2".into(),
        staged: Staged::InPlace,
        manager_argv: None,
        elevation: JobElevation::None,
        relaunch: Relaunch {
            exe: abs("/opt/gp/gitpoulpi-tui"),
            args: vec!["--session".into(), "work".into()],
            cwd: abs("/home/u/repo"),
        },
        requester_pid: 4242,
        config_dir: abs("/home/u/.config/gitpoulpi"),
        status_path: abs("/home/u/.local/share/gitpoulpi/updates/status.json"),
        environment: Some("env1".into()),
        install_id: Some("abc123".into()),
    }
}

#[test]
fn a_job_round_trips() {
    for staged in [
        Staged::InPlace,
        Staged::Archive {
            archive: abs("/home/u/.cache/gp/gitpoulpi-1.4.2.zip"),
            sha256_url: "https://example.test/a.zip.sha256".into(),
            minisig_url: "https://example.test/a.zip.minisig".into(),
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
            minisig_url: "m".into(),
        },
        ..job()
    };
    let v: serde_json::Value = serde_json::from_str(&archive.render()).unwrap();
    assert_eq!(v["staged"]["archive"], "/c/a.zip");
    assert_eq!(v["staged"]["sha256_url"], "u");
    assert_eq!(v["staged"]["minisig_url"], "m");
}

#[test]
fn a_staged_archive_job_without_a_signature_url_is_refused() {
    let archive = UpdateJob {
        staged: Staged::Archive {
            archive: PathBuf::from("/c/a.zip"),
            sha256_url: "u".into(),
            minisig_url: "m".into(),
        },
        ..job()
    };
    let mut v: serde_json::Value = serde_json::from_str(&archive.render()).unwrap();
    v["staged"].as_object_mut().unwrap().remove("minisig_url");
    assert!(matches!(
        UpdateJob::parse(&v.to_string()),
        Err(JobError::Invalid(_))
    ));
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

#[test]
fn a_job_carries_its_environment_and_installation_id() {
    let v: serde_json::Value = serde_json::from_str(&job().render()).unwrap();
    assert_eq!(v["environment"], "env1");
    assert_eq!(v["install_id"], "abc123");
    let back = UpdateJob::parse(&job().render()).unwrap();
    assert_eq!(back.environment.as_deref(), Some("env1"));
    assert_eq!(back.install_id.as_deref(), Some("abc123"));
}

#[test]
fn an_older_job_without_them_still_parses() {
    let mut v: serde_json::Value = serde_json::from_str(&job().render()).unwrap();
    let obj = v.as_object_mut().unwrap();
    obj.remove("environment");
    obj.remove("install_id");
    let parsed = UpdateJob::parse(&v.to_string()).unwrap();
    assert_eq!(parsed.environment, None);
    assert_eq!(parsed.install_id, None);
}
