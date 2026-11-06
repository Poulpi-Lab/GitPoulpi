use super::*;

#[derive(Default)]
struct Fake {
    file: Option<String>,
    registry: Option<BTreeMap<String, String>>,
}

impl PolicySource for Fake {
    fn file_text(&self) -> Option<String> {
        self.file.clone()
    }
    fn registry_values(&self) -> Option<BTreeMap<String, String>> {
        self.registry.clone()
    }
}

fn file(text: &str) -> Fake {
    Fake {
        file: Some(text.into()),
        ..Fake::default()
    }
}

fn reg(pairs: &[(&str, &str)]) -> Option<BTreeMap<String, String>> {
    Some(
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    )
}

#[test]
fn every_key_parses() {
    let loaded = load(&file(
        r#"
[updates]
mode = "auto"
release_channel = "prerelease"
pin = "1.4"
check = "nuget"
check_url = "https://nexus.example/repository/choco-internal/"
repo = "Acme/GitPoulpi"
download_base = "https://mirror.example/gitpoulpi/"
allow_self_update = false
message = "Updates are deployed by IT."
"#,
    ));
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    let p = loaded.policy;
    assert_eq!(p.mode, Some(UpdateMode::Auto));
    assert_eq!(p.release_channel, Some(UpdateChannel::Prerelease));
    assert_eq!(p.pin, Pin::parse("1.4"));
    assert_eq!(p.check, Some(CheckKind::Nuget));
    assert_eq!(
        p.check_url.as_deref(),
        Some("https://nexus.example/repository/choco-internal/")
    );
    assert_eq!(p.repo.as_deref(), Some("Acme/GitPoulpi"));
    assert_eq!(
        p.download_base.as_deref(),
        Some("https://mirror.example/gitpoulpi/")
    );
    assert_eq!(p.allow_self_update, Some(false));
    assert_eq!(p.message.as_deref(), Some("Updates are deployed by IT."));
    assert!(!p.failed_closed);
}

#[test]
fn no_policy_means_no_restriction() {
    let loaded = load(&Fake::default());
    assert_eq!(loaded.policy, Policy::default());
    let eff = merge(&loaded.policy, &UpdatesConfig::default());
    assert_eq!(eff.mode, UpdateMode::Notify);
    assert!(eff.allow_self_update);
    assert!(!eff.mode_locked);
}

#[test]
fn the_mode_is_a_ceiling_the_user_may_only_lower() {
    let user = |mode| UpdatesConfig {
        mode,
        ..UpdatesConfig::default()
    };
    let policy = |mode| Policy {
        mode: Some(mode),
        ..Policy::default()
    };
    // Policy notify: user auto is held down, user off stays off.
    let p = policy(UpdateMode::Notify);
    assert_eq!(merge(&p, &user(UpdateMode::Auto)).mode, UpdateMode::Notify);
    assert_eq!(merge(&p, &user(UpdateMode::Off)).mode, UpdateMode::Off);
    // Policy off beats everything.
    let p = policy(UpdateMode::Off);
    assert_eq!(merge(&p, &user(UpdateMode::Auto)).mode, UpdateMode::Off);
    // Policy auto does not force the user up.
    let p = policy(UpdateMode::Auto);
    assert_eq!(
        merge(&p, &user(UpdateMode::Notify)).mode,
        UpdateMode::Notify
    );
    assert!(merge(&p, &user(UpdateMode::Notify)).mode_locked);
}

#[test]
fn a_policy_release_channel_and_repo_replace_the_users() {
    let policy = load(&file(
        "[updates]\nrelease_channel = \"stable\"\nrepo = \"Acme/G\"\n",
    ))
    .policy;
    let user = UpdatesConfig {
        channel: UpdateChannel::Prerelease,
        ..UpdatesConfig::default()
    };
    let eff = merge(&policy, &user);
    assert_eq!(eff.release_channel, UpdateChannel::Stable);
    assert_eq!(eff.repo, "Acme/G");
    assert!(eff.release_channel_locked && eff.repo_locked);
    let free = merge(&Policy::default(), &user);
    assert_eq!(free.release_channel, UpdateChannel::Prerelease);
}

#[test]
fn a_two_part_pin_allows_any_patch_of_that_minor() {
    let pin = Pin::parse("1.4").unwrap();
    assert!(pin.allows("1.4.0"));
    assert!(pin.allows("1.4.9"));
    assert!(pin.allows("v1.4.10"));
    assert!(!pin.allows("1.5.0"));
    assert!(!pin.allows("1.3.9"), "a pin is a family, not a ceiling");
    assert!(!pin.allows("2.0.0"));
}

#[test]
fn a_full_pin_is_an_upper_bound() {
    let pin = Pin::parse("1.4.2").unwrap();
    assert!(pin.allows("1.4.2"));
    assert!(pin.allows("1.4.1"));
    assert!(pin.allows("1.3.9"));
    assert!(!pin.allows("1.4.3"));
    assert!(!pin.allows("1.5.0"));
    assert!(!pin.allows("not-a-version"));
}

#[test]
fn a_one_part_pin_and_bad_pins() {
    assert!(Pin::parse("1").unwrap().allows("1.9.9"));
    assert!(!Pin::parse("1").unwrap().allows("2.0.0"));
    for bad in ["", "x", "1.x", "1.2.3.4", "1..2"] {
        assert_eq!(Pin::parse(bad), None, "{bad:?}");
    }
}

#[test]
fn registry_beats_file_beats_user() {
    let source = Fake {
        file: Some(
            "[updates]\nmode = \"auto\"\nrepo = \"File/Repo\"\nmessage = \"from file\"\n".into(),
        ),
        registry: reg(&[("mode", "notify"), ("repo", "Reg/Repo")]),
    };
    let policy = load(&source).policy;
    assert_eq!(policy.mode, Some(UpdateMode::Notify), "registry over file");
    assert_eq!(policy.repo.as_deref(), Some("Reg/Repo"));
    assert_eq!(
        policy.message.as_deref(),
        Some("from file"),
        "a key only the file sets still applies"
    );
    let user = UpdatesConfig {
        mode: UpdateMode::Auto,
        repo: "User/Repo".into(),
        ..UpdatesConfig::default()
    };
    let eff = merge(&policy, &user);
    assert_eq!(eff.repo, "Reg/Repo", "policy over user");
    assert_eq!(eff.mode, UpdateMode::Notify);
}

#[test]
fn a_parse_error_fails_closed_and_logs_one_line() {
    let loaded = load(&file("[updates\nmode = \"auto\""));
    assert_eq!(loaded.warnings.len(), 1, "{:?}", loaded.warnings);
    let eff = merge(
        &loaded.policy,
        &UpdatesConfig {
            mode: UpdateMode::Auto,
            ..UpdatesConfig::default()
        },
    );
    assert_eq!(eff.mode, UpdateMode::Notify);
    assert!(!eff.allow_self_update);
}

#[test]
fn a_bad_value_fails_closed_like_a_parse_error() {
    for text in [
        "[updates]\nmode = \"always\"\n",
        "[updates]\npin = \"latest\"\n",
        "[updates]\ncheck = \"ftp\"\n",
        "[updates]\nallow_self_update = \"maybe\"\n",
        "[updates]\nmode = 3\n",
    ] {
        let loaded = load(&file(text));
        assert!(loaded.policy.failed_closed, "{text}");
        assert!(!merge(&loaded.policy, &UpdatesConfig::default()).allow_self_update);
    }
}

#[test]
fn a_broken_registry_never_raises_a_mode_the_file_lowered() {
    let source = Fake {
        file: Some("[updates]\nmode = \"off\"\n".into()),
        registry: reg(&[("pin", "banana")]),
    };
    let eff = merge(&load(&source).policy, &UpdatesConfig::default());
    assert_eq!(eff.mode, UpdateMode::Off);
    assert!(!eff.allow_self_update);
}

#[test]
fn an_unknown_key_is_ignored_with_a_warning() {
    let loaded = load(&file("[updates]\nmode = \"off\"\nfuture_knob = \"x\"\n"));
    assert_eq!(loaded.policy.mode, Some(UpdateMode::Off));
    assert!(!loaded.policy.failed_closed);
    assert_eq!(loaded.warnings.len(), 1);
    assert!(loaded.warnings[0].contains("future_knob"));
}

#[test]
fn a_file_without_an_updates_table_sets_nothing() {
    assert_eq!(load(&file("# empty\n")).policy, Policy::default());
}

#[test]
fn registry_booleans_accept_the_msi_spellings() {
    let policy = load(&Fake {
        registry: reg(&[("allow_self_update", "0")]),
        ..Fake::default()
    })
    .policy;
    assert_eq!(policy.allow_self_update, Some(false));
}

#[test]
fn policy_paths_per_os() {
    use std::path::Path;
    assert_eq!(
        policy_path_for("windows", Some(r"D:\PD")),
        Path::new(r"D:\PD").join("GitPoulpi").join("policy.toml")
    );
    assert_eq!(
        policy_path_for("windows", None),
        Path::new(r"C:\ProgramData")
            .join("GitPoulpi")
            .join("policy.toml")
    );
    assert_eq!(
        policy_path_for("linux", None),
        Path::new("/etc/gitpoulpi/policy.toml")
    );
    assert_eq!(
        policy_path_for("macos", None),
        Path::new("/Library/Application Support/GitPoulpi/policy.toml")
    );
    assert_eq!(policy_paths().len(), 1);
}

#[test]
fn the_file_source_reads_a_real_file_and_a_missing_one_is_no_policy() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("policy.toml");
    assert_eq!(
        load(&FilePolicySource(path.clone())).policy,
        Policy::default()
    );
    std::fs::write(&path, "[updates]\nmode = \"off\"\n").unwrap();
    assert_eq!(
        load(&FilePolicySource(path)).policy.mode,
        Some(UpdateMode::Off)
    );
}

#[test]
fn a_pin_displays_as_the_text_it_was_parsed_from() {
    for text in ["2", "1.4", "1.4.2"] {
        assert_eq!(Pin::parse(text).unwrap().to_string(), text);
    }
}
