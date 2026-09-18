use super::*;

/// A field nothing reads has to say so. The flag comes from
/// `LIVE_KEYS`, so this also pins the two crates together.
#[test]
fn a_fields_live_flag_comes_from_the_config_crates_own_list() {
    let dir = tempfile::tempdir().unwrap();
    let state = state(dir.path());
    let general = {
        let mut all = state.log_graph_fields();
        all.extend(state.sync_fields());
        all.extend(state.general_fields());
        all
    };
    let graph = general
        .iter()
        .find(|f| f.label == "Initial commits in graph")
        .expect("the field exists");
    assert!(
        graph.live,
        "general.initial_commits_in_graph is in LIVE_KEYS"
    );
    let prune = general
        .iter()
        .find(|f| f.label == "Auto-prune")
        .expect("the field exists");
    assert!(
        prune.live,
        "general.auto_prune governs Fetch/FetchAll's --prune now"
    );
    let submodules = general
        .iter()
        .find(|f| f.label == "Keep submodules up to date");
    assert_eq!(submodules.is_some(), crate::build_features::SUBMODULE);
    if let Some(submodules) = submodules {
        assert!(
            submodules.live && submodules.coming.is_none(),
            "general.keep_submodules_up_to_date is read by the daemon after checkout/pull"
        );
    }
    let browse_root = general
        .iter()
        .find(|f| f.label == "Default browse folder")
        .expect("the field exists");
    assert!(
        browse_root.live,
        "repos.browse_root is in LIVE_KEYS - RepoDialogs::open reads it"
    );
}

/// The Git & Encoding page carries the LFS rows: the program's state
/// (read-only, with the install link in its help), and an "Initialize
/// LFS" button that asks the event loop to run `git lfs install --local`.
#[cfg(feature = "lfs")]
#[test]
fn the_git_page_shows_the_lfs_program_and_the_initialize_button_asks_for_an_install() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    state.section = PrefSection::ALL
        .iter()
        .position(|s| *s == PrefSection::GitAndEncoding)
        .unwrap();
    state.set_lfs_program(Some("not installed (https://git-lfs.com)".into()));
    let fields = state.git_fields();
    let program = fields
        .iter()
        .find(|f| f.label == "git-lfs program")
        .expect("the program row");
    assert!(matches!(&program.control, Control::Info(t) if t.contains("not installed")));
    assert!(program
        .help
        .as_deref()
        .is_some_and(|h| h.contains("git-lfs.com")));

    state.focus = PrefFocus::Fields;
    state.field = fields
        .iter()
        .position(|f| f.label == "Initialize LFS for this repository")
        .unwrap();
    assert!(!state.take_lfs_init_request());
    state.activate();
    assert!(
        state.take_lfs_init_request(),
        "pressing the button asks the event loop to set LFS up"
    );
    assert!(!state.take_lfs_init_request(), "consumed");
}

/// **The guard for the `coming` state.** Every row that edits a `config.toml`
/// key or a repo-state field is either `live` (`LIVE_KEYS` says something
/// reads it) or explicitly `coming` with a reason - never a silent control
/// that commits to disk and is read by nothing.
///
/// Every section is walked with a config exercising its non-empty branches
/// (one profile, one integration).
#[test]
fn every_editable_row_is_live_or_explicitly_coming() {
    let dir = tempfile::tempdir().unwrap();
    let mut config = Config::default();
    config.profiles.push(Profile::new("p1", "Test Profile"));
    config.integrations.push(Integration::new(
        "gh1",
        IntegrationKind::Github,
        "https://github.com",
    ));
    let mut state = PreferencesState::with_parts(
        config,
        Some(dir.path().join("config.toml")),
        dir.path(),
        RepoState::default(),
        RepoState::path_for(dir.path(), ".gitpoulpi"),
        std::sync::Arc::new(InMemoryTokens::default()),
    );

    let mut offenders = Vec::new();
    for section in PrefSection::ALL {
        state.section = PrefSection::ALL.iter().position(|s| s == section).unwrap();
        for field in state.fields() {
            let edits_config = matches!(field.target, Target::Global(_) | Target::Repo(_));
            if edits_config && !field.live && field.coming.is_none() {
                offenders.push(format!("{:?}: {}", section, field.label));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "row(s) edit a config field nothing reads and are not marked `coming`: \
         {offenders:?}"
    );
}

#[test]
fn the_launch_rows_are_a_dropdown_of_presets_and_a_text_template() {
    let _serial = crate::tab_dir::TEST_SERIAL
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    go(&mut state, PrefSection::AgentsMcp);
    let fields = state.fields();
    let launch = fields
        .iter()
        .position(|f| matches!(f.target, Target::Global("ui.launch")))
        .expect("the launch choice row");
    let Control::Choice { options, index } = &fields[launch].control else {
        panic!("ui.launch must be a combobox, not a toggle or text");
    };
    assert_eq!(options.first().map(String::as_str), Some("off"));
    assert_eq!(options.last().map(String::as_str), Some("custom"));
    assert_eq!(*index, 0, "off by default");
    assert!(options.len() > 3, "{options:?}");
    assert!(matches!(
        fields[launch + 1].control,
        Control::Text(ref t) if t.is_empty()
    ));
    assert!(matches!(
        fields[launch + 1].target,
        Target::Global("ui.launch_command")
    ));

    // Picking a preset and typing a template both persist.
    state.commit_choice(&Target::Global("ui.launch"), 0);
    state.field = launch;
    state.commit_choice(&Target::Global("ui.launch"), options.len() - 1);
    assert_eq!(state.config.ui.launch, "custom");
    state.commit_text(launch + 1, "my-term {repo} {gitpoulpi}");
    assert_eq!(state.config.ui.launch_command, "my-term {repo} {gitpoulpi}");
    let raw = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(raw.contains("launch = \"custom\""), "{raw}");
}

#[test]
fn the_lane_background_combo_shows_friendly_labels_and_stores_the_names() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    state.section = PrefSection::ALL
        .iter()
        .position(|s| *s == PrefSection::Appearance)
        .unwrap();
    state.focus = PrefFocus::Fields;
    state.field = state
        .fields()
        .iter()
        .position(|f| f.label == "Graph lane background")
        .unwrap();
    assert!(draw_screen(&mut state)
        .lines()
        .any(|l| l.contains("Off") && l.contains('\u{25BE}')));
    state.choose_next();
    let file = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(
        file.contains("graph_lane_background = \"row_tint\""),
        "{file}"
    );
    assert!(draw_screen(&mut state)
        .lines()
        .any(|l| l.contains("Row tint") && l.contains('\u{25BE}')));
}

#[test]
fn the_hosting_refresh_row_follows_the_pr_terminology_setting() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    state.section = PrefSection::ALL
        .iter()
        .position(|s| *s == PrefSection::Integrations)
        .unwrap();
    state.focus = PrefFocus::Fields;
    state.strip = true;
    state.field = state
        .fields()
        .iter()
        .position(|f| f.label == "Pull Requests and issues refresh")
        .unwrap();
    crate::terminology::install(gitpoulpi_config::PrTerminology::MergeRequest);
    let screen = draw_screen(&mut state);
    crate::terminology::install(gitpoulpi_config::PrTerminology::Auto);
    assert!(screen.contains("Merge Requests and issues"), "{screen}");
    assert!(!screen.contains("Pull Requests and issues"), "{screen}");
}

#[test]
fn the_diff_and_editor_page_leads_with_the_editor_settings_all_wired() {
    let dir = tempfile::tempdir().unwrap();
    let state = state(dir.path());
    let fields = state.fields_of(PrefSection::DiffEditor);
    let labels: Vec<&str> = fields.iter().map(|f| f.label.as_str()).collect();
    assert_eq!(
        labels[..6],
        [
            "Tab size",
            "End of line",
            "Syntax highlighting",
            "Line numbers",
            "Follow the selection in the editor",
            "Word wrap",
        ]
    );
    assert!(fields.iter().all(|f| f.live && f.coming.is_none()));
}

/// Without `avatar-fetch` there is no source to choose; the placement and
/// the fallback (initials / identicon) rows stay in every build.
#[test]
fn the_avatar_sources_row_needs_avatar_fetch() {
    let dir = tempfile::tempdir().unwrap();
    let state = state(dir.path());
    let ui = state.appearance_fields();
    let has = |label: &str| ui.iter().any(|f| f.label == label);
    assert_eq!(
        has(gitpoulpi_i18n::msg::settings_ui::AVATAR_SOURCES.get()),
        crate::build_features::AVATAR_FETCH
    );
    assert!(has(gitpoulpi_i18n::msg::settings_ui::AVATARS.get()));
    assert!(has(gitpoulpi_i18n::msg::settings_ui::AVATAR_FALLBACK.get()));
}

fn combo_of(state: &PreferencesState, key: &'static str) -> (Vec<(String, String)>, String) {
    match state
        .general_fields()
        .into_iter()
        .find(|f| f.target == Target::Global(key))
        .map(|f| f.control)
    {
        Some(Control::Combo { options, value, .. }) => (options, value),
        other => panic!("{key} is not a combobox: {}", other.is_some()),
    }
}

#[test]
fn the_repo_data_location_is_a_combobox_and_the_legacy_value_maps_to_the_worktree() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    assert_eq!(combo_of(&st, "session.per_repo_state_dir").1, "in_git_dir");
    st.config.session.per_repo_state_dir = ".gitpoulpi".into();
    assert_eq!(combo_of(&st, "session.per_repo_state_dir").1, "in_worktree");
    st.commit_combo(&Target::Global("session.per_repo_state_dir"), "user_data");
    let stored = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(
        stored.contains("per_repo_state_dir = \"user_data\""),
        "{stored}"
    );
}

#[test]
fn commit_count_is_a_dropdown_with_three_methods_that_applies_live() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    state.section = PrefSection::ALL
        .iter()
        .position(|s| *s == PrefSection::LogGraph)
        .unwrap();
    state.focus = PrefFocus::Fields;
    state.field = state
        .fields()
        .iter()
        .position(|f| f.target == Target::Global("log.commit_count"))
        .expect("the Log/Graph page lists the commit count");
    match &state.fields()[state.field].control {
        Control::Choice { options, index } => {
            assert_eq!(options, &["system-git", "libgit2", "off"]);
            assert_eq!(*index, 0, "default is system git");
        }
        _ => panic!("commit_count must be a dropdown"),
    }
    assert!(draw_screen(&mut state)
        .lines()
        .any(|l| l.contains("System git") && l.contains('\u{25BE}')));
    state.choose_next();
    let file = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(file.contains("commit_count = \"libgit2\""), "{file}");
    assert_eq!(
        crate::live_settings::commit_count(),
        gitpoulpi_config::CommitCountMethod::Libgit2,
        "the pick is published at once, no restart"
    );
    state.choose_next();
    assert_eq!(
        crate::live_settings::commit_count(),
        gitpoulpi_config::CommitCountMethod::Off
    );
    crate::live_settings::install_commit_count(gitpoulpi_config::CommitCountMethod::SystemGit);
}

#[test]
fn the_update_settings_are_dropdowns_that_apply_live() {
    let _lock = crate::updates::TEST_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    state.section = PrefSection::ALL
        .iter()
        .position(|s| *s == PrefSection::Sync)
        .unwrap();
    state.focus = PrefFocus::Fields;
    let at = |state: &PreferencesState, key: &'static str| {
        state
            .fields()
            .iter()
            .position(|f| f.target == Target::Global(key))
            .unwrap_or_else(|| panic!("the Synchronization page lists {key}"))
    };
    state.field = at(&state, "updates.mode");
    match &state.fields()[state.field].control {
        Control::Choice { options, index } => {
            assert_eq!(options, &["auto", "notify", "off"]);
            assert_eq!(*index, 1, "default is notify");
        }
        _ => panic!("updates.mode must be a dropdown"),
    }
    assert!(draw_screen(&mut state)
        .lines()
        .any(|l| l.contains("Notify") && l.contains('\u{25BE}')));
    state.choose_next();
    let file = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(file.contains("mode = \"off\""), "{file}");
    assert_eq!(
        crate::updates::snapshot().mode,
        gitpoulpi_config::UpdateMode::Off,
        "the pick is published at once, no restart"
    );
    state.field = at(&state, "updates.channel");
    match &state.fields()[state.field].control {
        Control::Choice { options, index } => {
            assert_eq!(options, &["stable", "prerelease"]);
            assert_eq!(*index, 0);
        }
        _ => panic!("updates.channel must be a dropdown"),
    }
    state.choose_next();
    let file = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(file.contains("channel = \"prerelease\""), "{file}");
    crate::updates::configure(&gitpoulpi_config::Config::default(), None);
}

/// The three start-tab settings are dropdowns on the General page, next to
/// Remember tabs; picking one is written to `config.toml`.
#[test]
#[cfg(feature = "radar")]
fn the_start_tab_settings_are_dropdowns_that_save() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    state.section = PrefSection::ALL
        .iter()
        .position(|s| *s == PrefSection::General)
        .unwrap();
    state.focus = PrefFocus::Fields;
    let at = |state: &PreferencesState, key: &'static str| {
        state
            .fields()
            .iter()
            .position(|f| f.target == Target::Global(key))
            .unwrap_or_else(|| panic!("the General page lists {key}"))
    };
    let remember = at(&state, "general.remember_tabs");
    assert_eq!(at(&state, "general.radar_enabled"), remember + 1);
    let in_repo = at(&state, "general.startup_tab_in_repo");
    match &state.fields()[in_repo].control {
        Control::Choice { options, index } => {
            let mut want = vec!["current-repo", "last", "home"];
            if crate::build_features::RADAR {
                want.push("radar");
            }
            assert_eq!(options, &want);
            assert_eq!(*index, 0, "default is the current repository");
        }
        _ => panic!("startup_tab_in_repo must be a dropdown"),
    }
    match &state.fields()[at(&state, "general.startup_tab_outside_repo")].control {
        Control::Choice { options, index } => {
            assert!(!options.contains(&"current-repo".to_string()));
            assert_eq!(options[*index], "home", "default is Home");
        }
        _ => panic!("startup_tab_outside_repo must be a dropdown"),
    }
    state.field = in_repo;
    state.choose_next();
    let file = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(file.contains("startup_tab_in_repo = \"last\""), "{file}");
}

/// A stored `radar` with the Radar off reads as Home, and the
/// choice is not offered.
#[test]
fn the_radar_choice_is_hidden_while_the_radar_is_off() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    state.config.general.radar_enabled = false;
    state.config.general.startup_tab_launcher = gitpoulpi_config::StartupTab::Radar;
    crate::build_features::install_radar_enabled(false);
    let fields = state.general_rows();
    let field = fields
        .iter()
        .find(|f| f.target == Target::Global("general.startup_tab_launcher"))
        .unwrap();
    match &field.control {
        Control::Choice { options, index } => {
            assert!(!options.contains(&"radar".to_string()), "{options:?}");
            assert_eq!(options[*index], "home");
        }
        _ => panic!("a dropdown"),
    }
    crate::build_features::install_radar_enabled(true);
}

/// An absent `general.remember_tabs` means on: the row is checked whether the
/// file lacks the key, the `[general]` table, or the whole file.
#[test]
fn remember_tabs_shows_checked_when_the_key_is_absent() {
    let row = |toml: &str| {
        let dir = tempfile::tempdir().unwrap();
        let mut st = state(dir.path());
        st.config = gitpoulpi_config::Config::from_toml_str(toml).unwrap().0;
        let field = st
            .fields_of(PrefSection::General)
            .into_iter()
            .find(|f| matches!(f.target, Target::Global("general.remember_tabs")))
            .expect("the row");
        field.value_text()
    };
    assert_eq!(row(""), "[x]", "no file content");
    assert_eq!(row("[ui]\nminimap = false\n"), "[x]", "no [general] table");
    assert_eq!(
        row("[general]\nauto_prune = false\nstartup_tab_outside_repo = \"last\"\n"),
        "[x]",
        "no key"
    );
    assert_eq!(
        row("[general]\nremember_tabs = false\n"),
        "[ ]",
        "explicit off"
    );
}
