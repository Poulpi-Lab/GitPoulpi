use super::*;

/// The field round-trips to the file, and
/// `RepoDialogs`' own resolution (`browse_start_for`) is what
/// actually reads it back - see `repo_dialogs.rs`'s test of that
/// fallback chain for the other half of this contract.
#[test]
fn the_browse_root_field_saves_to_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    state.focus = PrefFocus::Fields;
    let index = state
        .general_fields()
        .iter()
        .position(|f| f.label == "Default browse folder")
        .unwrap();
    state.commit_text(index, r"D:\Projects");

    assert_eq!(state.config.repos.browse_root, r"D:\Projects");
    let raw = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(raw.contains(r"browse_root = 'D:\Projects'"), "{raw}");
}

#[test]
fn toggling_a_global_setting_reaches_the_file_and_the_reloaded_config() {
    // `activate` publishes `AUTO_PRUNE` process-wide: hold the lock while it is on
    // and put it back, or a concurrent `Fetch` golden sees `--prune`.
    let _lock = crate::live_settings::AUTO_PRUNE_TEST_LOCK.blocking_lock();
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            crate::live_settings::install_auto_prune(false);
        }
    }
    let _reset = Reset;
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    go(&mut state, PrefSection::Sync);
    state.focus = PrefFocus::Fields;
    state.field = state
        .sync_fields()
        .iter()
        .position(|f| f.label == "Auto-prune")
        .unwrap();
    state.activate();

    assert!(
        !state.config.general.auto_prune,
        "on by default, so the toggle turns it off"
    );
    let raw = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(raw.contains("auto_prune = false"), "{raw}");
}

/// `general.active_profile`'s round trip: picking a profile republishes
/// `live_settings::active_profile_author`, which `daemon_link::run_action`'s
/// `Commit` arm reads. A profile with a blank author name or email must not
/// become a half-identity override; the commit falls back to `git config user.*`.
#[test]
fn picking_a_profile_publishes_its_author_for_the_next_commit() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    state.config.profiles.push(Profile {
        id: "work".into(),
        name: "Work".into(),
        author_name: "Ada Lovelace".into(),
        author_email: "ada@example.com".into(),
    });
    state.write_profiles();
    go_tab(&mut state, PrefSection::Profiles, SubTab::General);
    state.focus = PrefFocus::Fields;
    state.field = 0; // "Active profile" is the General tab's only row.
    assert_eq!(state.fields()[0].label, "Active profile");
    // Options are ["(none)", "Work <...>"], starting at index 0;
    // A choice opens a dropdown; `choose_next` picks the option after
    // the current one through it.
    state.choose_next();

    assert_eq!(state.config.general.active_profile, "work");
    let author = active_profile_author().expect("a profile with a full identity");
    assert_eq!(author.name, "Ada Lovelace");
    assert_eq!(author.email, "ada@example.com");

    // A profile naming no identity must not publish a half-override.
    state.config.profiles[0].author_email = String::new();
    state.write_profiles();
    assert!(
        active_profile_author().is_none(),
        "an empty email must fall back to git config user.*, not override with half an \
         identity"
    );

    // Leave the process-wide static as the schema default so a test
    // that runs after this one in the same binary is not surprised
    // by a profile author this test installed.
    install_active_profile(&Config::default());
}

/// The repo scope must land in the repo's own file, not in
/// `config.toml` - two repos disagreeing about line endings is the
/// entire point of having a repo scope.
#[cfg(feature = "lfs")]
#[test]
fn a_repo_setting_is_written_to_the_repos_own_state_file() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    state.section = PrefSection::ALL
        .iter()
        .position(|s| *s == PrefSection::GitAndEncoding)
        .unwrap();
    state.focus = PrefFocus::Fields;
    state.field = state
        .git_fields()
        .iter()
        .position(|f| f.label == "Git LFS")
        .unwrap();
    state.activate();

    let path = RepoState::path_for(dir.path(), ".gitpoulpi");
    assert!(path.exists(), "the repo state file was not written");
    let back = RepoState::load(&path).unwrap();
    // `git.lfs_enabled` defaults to on, so activating the row turns
    // it *off* for this repository.
    assert_eq!(back.preferences.lfs_enabled, Some(false));
    assert!(
        !dir.path().join("config.toml").exists(),
        "a repo setting must not touch the global config"
    );
}

/// `(inherited)` is index 0 and means *clear the override*, not
/// "write the string `(inherited)` into the repo's config". `core.autocrlf` /
/// `core.eol` are git's own config: a pick becomes a request for the daemon
/// (journaled there), never a value in `repo-state.toml`.
#[test]
fn choosing_inherited_clears_the_override_rather_than_storing_a_word() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    let git_values = std::collections::BTreeMap::from([(
        "core.autocrlf".to_string(),
        Some("input".to_string()),
    )]);
    state.set_repo_config(&git_values, 1);
    state.section = PrefSection::ALL
        .iter()
        .position(|s| *s == PrefSection::GitAndEncoding)
        .unwrap();
    let row = |state: &PreferencesState| {
        state
            .fields()
            .into_iter()
            .find(|f| f.label == "core.autocrlf")
            .unwrap()
    };
    // The row shows what git reports, not what a file remembers.
    assert!(matches!(
        &row(&state).control,
        Control::Choice { index: 2, .. }
    ));

    state.commit_choice(&Target::Repo(RepoField::AutoCrlf), 0);
    assert_eq!(
        state.take_repo_config_requests(),
        vec![("core.autocrlf".to_string(), None)]
    );
    assert!(matches!(
        &row(&state).control,
        Control::Choice { index: 0, .. }
    ));
    assert_eq!(state.repo_state.preferences.autocrlf, None);

    // `true` is the first value after "(inherited)".
    state.commit_choice(&Target::Repo(RepoField::AutoCrlf), 1);
    state.commit_choice(&Target::Repo(RepoField::Eol), 3);
    assert_eq!(
        state.take_repo_config_requests(),
        vec![
            ("core.autocrlf".to_string(), Some("true".to_string())),
            ("core.eol".to_string(), Some("native".to_string())),
        ]
    );
    assert!(state.take_repo_config_requests().is_empty(), "consumed");
    // The next refresh's answer replaces the optimistic value.
    state.set_repo_config(&git_values, 2);
    assert!(matches!(
        &row(&state).control,
        Control::Choice { index: 2, .. }
    ));
}

/// The five rows that used to say "coming" now do their job: none carries a
/// note, every one is live, and the date locale is a real choice.
#[test]
fn the_former_coming_rows_are_live_without_a_note() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    let mut seen = 0;
    for section in [
        PrefSection::Ssh,
        PrefSection::Appearance,
        PrefSection::GitAndEncoding,
    ] {
        state.section = PrefSection::ALL.iter().position(|s| *s == section).unwrap();
        for label in [
            "Private key path",
            "Public key path",
            "Date/time locale",
            "core.autocrlf",
            "core.eol",
        ] {
            if let Some(f) = state.fields().into_iter().find(|f| f.label == label) {
                assert!(f.live, "{label}");
                assert!(f.coming.is_none(), "{label}");
                assert!(f.help.is_some(), "{label} explains itself");
                seen += 1;
            }
        }
        assert!(
            !state.fields().iter().any(|f| f.label == "Date word Format"),
            "the dead Date word Format row is gone"
        );
    }
    assert_eq!(seen, 5, "all five rows were found");
    state.section = PrefSection::ALL
        .iter()
        .position(|s| *s == PrefSection::Appearance)
        .unwrap();
    let locale = state
        .fields()
        .into_iter()
        .find(|f| f.label == "Date/time locale")
        .unwrap();
    assert!(matches!(locale.control, Control::Choice { .. }));
}

/// Out-of-range numbers are refused, not clamped silently: the user
/// typed a number and has to learn it was not taken.
#[test]
fn a_number_outside_its_range_is_refused_and_says_so() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    state.focus = PrefFocus::Fields;
    go(&mut state, PrefSection::LogGraph);
    let index = state
        .log_graph_fields()
        .iter()
        .position(|f| f.label == "Initial commits in graph")
        .unwrap();
    state.commit_text(index, "12");
    assert_eq!(
        state.config.general.initial_commits_in_graph,
        gitpoulpi_config::DEFAULT_INITIAL_COMMITS_IN_GRAPH
    );
    assert!(state.message.as_deref().unwrap().contains("outside"));

    state.commit_text(index, "900");
    assert_eq!(state.config.general.initial_commits_in_graph, 900);
}

#[test]
fn the_generated_global_ssh_key_follows_the_key_type_setting() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    let ssh = dir.path().join("dot-ssh");
    state.config.ssh.key_type = "ecdsa".into();
    state.generate_ssh_key_in(&ssh);
    let public = std::fs::read_to_string(ssh.join("gitpoulpi_ecdsa.pub")).unwrap();
    assert!(public.starts_with("ecdsa-sha2-nistp256 "), "{public}");
    assert!(!ssh.join("gitpoulpi_ed25519").exists());
    assert!(state
        .config
        .ssh
        .private_key_path
        .as_deref()
        .is_some_and(|p| p.ends_with("gitpoulpi_ecdsa")));
}

#[test]
fn the_global_ssh_key_is_generated_in_process_and_recorded() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    let ssh = dir.path().join("dot-ssh");
    state.generate_ssh_key_in(&ssh);
    assert!(ssh.join("gitpoulpi_ed25519").is_file());
    assert!(ssh.join("gitpoulpi_ed25519.pub").is_file());
    assert!(state
        .config
        .ssh
        .private_key_path
        .as_deref()
        .is_some_and(|p| p.ends_with("gitpoulpi_ed25519")));
    // Never overwrites what is already there.
    state.generate_ssh_key_in(&ssh);
    assert!(state
        .message
        .as_deref()
        .is_some_and(|m| m.contains("already exists")));
}

#[test]
fn the_tab_folder_rows_are_comboboxes_and_persist() {
    let _serial = crate::tab_dir::TEST_SERIAL
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    go_tab(&mut state, PrefSection::OpenIn, SubTab::General);
    let fields = state.fields();
    let follow = fields
        .iter()
        .position(|f| matches!(f.target, Target::Global("ui.tab_dir_follow")))
        .expect("the follow row");
    assert_eq!(fields[follow].label, "Follow the active tab's folder");
    let Control::Choice { options, index } = &fields[follow].control else {
        panic!("ui.tab_dir_follow must be a combobox");
    };
    assert_eq!(options, &["buttons", "auto"]);
    assert_eq!(*index, 0, "buttons by default");
    assert!(
        !fields.iter().any(|f| matches!(
            f.target,
            Target::Global("ui.terminal_launch" | "ui.terminal_launch_command")
        )),
        "the terminal rows moved to the Open in page"
    );

    state.field = follow;
    state.commit_choice(&Target::Global("ui.tab_dir_follow"), 1);
    assert_eq!(state.config.ui.tab_dir_follow, TabDirFollow::Auto);
    let raw = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(raw.contains("tab_dir_follow = \"auto\""), "{raw}");
    crate::tab_dir::reset_for_tests();
}
