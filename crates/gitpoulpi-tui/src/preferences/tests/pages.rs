//! Which setting lives on which Settings page (the thematic regrouping).

use super::*;
use std::collections::BTreeMap;

fn keys_of(st: &PreferencesState, section: PrefSection) -> Vec<&'static str> {
    st.fields_of(section)
        .iter()
        .filter_map(|f| match f.target {
            Target::Global(k) => Some(k),
            _ => None,
        })
        .collect()
}

#[test]
fn each_setting_is_on_the_page_the_owner_chose() {
    let dir = tempfile::tempdir().unwrap();
    let st = state(dir.path());
    let expect: &[(PrefSection, &[&str])] = &[
        (
            PrefSection::General,
            &[
                "ui.language",
                "general.remember_tabs",
                "general.radar_enabled",
                "repos.clone_root",
                "repos.browse_root",
                "general.default_branch_name",
                "general.undo_enabled",
                "general.undo_depth",
                "log.level",
                "log.retention_days",
                "log.max_file_mb",
                "log.timings",
                "log.extended",
                "general.journal_outside",
            ],
        ),
        (PrefSection::ConflictGlobal, &["general.conflict_detection"]),
        (
            PrefSection::Launcher,
            &[
                "ui.launch_terminal",
                "ui.relaunch_in_terminal",
                "ui.reuse_window",
                "ui.command_alias",
                "ui.command_alias_add",
            ],
        ),
        (
            PrefSection::Sync,
            &[
                "general.auto_fetch_interval_minutes",
                "general.fetch_on_tab_open",
                "general.auto_prune",
                "general.auto_prune_tags",
                "general.keep_submodules_up_to_date",
                "general.auto_stash",
                "general.include_untracked_in_autostash",
                "general.delete_orig_files_after_merge",
                "updates.mode",
                "updates.channel",
                "rewrite.script_timeout_ms",
                "rewrite.script_total_timeout_s",
            ],
        ),
        (
            PrefSection::Ssh,
            &[
                "ssh.private_key_path",
                "ssh.public_key_path",
                "ssh.key_type",
                "credentials.https_mode",
            ],
        ),
        (
            PrefSection::Integrations,
            &[
                "general.hosting_refresh_minutes",
                "ui.pr_terminology",
                "ui.hide_finished_pr_chip",
            ],
        ),
        (
            PrefSection::AgentsMcp,
            &[
                #[cfg(feature = "mcp")]
                "mcp.enabled",
                #[cfg(feature = "mcp")]
                "mcp.port",
                #[cfg(feature = "mcp")]
                "mcp.http_auth",
                #[cfg(feature = "mcp")]
                "mcp.token",
                #[cfg(feature = "mcp")]
                "mcp.tls",
                #[cfg(feature = "mcp")]
                "mcp.tls_cert",
                #[cfg(feature = "mcp")]
                "mcp.tls_key",
                #[cfg(feature = "mcp")]
                "mcp.behind_proxy",
                "ui.allow_agent_view_control",
                "mcp.allow_ui_propose",
                "ui.launch",
                "ui.launch_command",
            ],
        ),
        (
            PrefSection::Appearance,
            &[
                "ui.theme",
                "ui.avatars",
                "ui.avatar_sources",
                "ui.avatar_fallback",
                "ui.input_color",
                "ui.input_color_custom",
                "ui.color_depth",
                "ui.text_cursor",
                "ui.graph_color_theme",
                "ui.graph_lane_background",
                "ui.graph_row_tint_strength",
                "ui.date_locale",
                "ui.date_format",
                "ui.date_time_format",
                "ui.date_verbose_format",
            ],
        ),
        (
            PrefSection::LogGraph,
            &[
                "ui.graph_renderer",
                "ui.image_protocol",
                "ui.graph_show_stashes",
                "ui.ref_connector",
                "ui.log_wip_counts_in_message",
                "ui.ref_tree",
                "ui.reveal_on_ref_click",
                "ui.composer_files",
                "general.initial_commits_in_graph",
                "general.lazy_load_commits_in_graph",
                "general.commits_per_lazy_page",
                "general.max_commits_in_graph",
                "log.commit_count",
                "log.use_mailmap",
            ],
        ),
        (
            PrefSection::DiffEditor,
            &[
                "editor.tab_size",
                "editor.eol",
                "editor.syntax_highlighting",
                "editor.line_numbers",
                "editor.follow_selection",
                "ui.diff_word_wrap",
                "ui.diff_layout",
                "ui.diff_hunks_only",
                "ui.diff_show_whitespace",
                "ui.minimap",
                "ui.blame_side",
                "ui.blame_width",
                "merge.side_by_side_min_width",
                "general.external_editor",
                "general.external_diff_tool",
                "general.external_merge_tool",
                "general.file_explorer",
            ],
        ),
        (
            PrefSection::OpenIn,
            &["ui.tab_dir_follow", "ui.open_in_default"],
        ),
        (
            PrefSection::Images,
            &[
                "ui.image_view",
                "ui.image_view_text_fallback",
                "ui.image_view_max_mib",
                "ui.image_view_pixelated_from",
                "ui.image_view_sync",
                "ui.image_editor_default",
            ],
        ),
        (
            PrefSection::Keybindings,
            &[
                "ui.kitty_keyboard",
                "ui.mouse_enabled",
                "ui.mouse_scroll_lines",
            ],
        ),
    ];
    for (section, keys) in expect {
        let have = keys_of(&st, *section);
        for key in *keys {
            let not_built = match *key {
                "general.keep_submodules_up_to_date" => !crate::build_features::SUBMODULE,
                "ui.avatar_sources" => !crate::build_features::AVATAR_FETCH,
                "general.radar_enabled" => !crate::build_features::RADAR,
                k if k.starts_with("ui.image_view") => !crate::build_features::IMAGE_VIEW,
                k if k.starts_with("rewrite.") => !crate::build_features::HISTORY_REWRITE,
                _ => false,
            };
            if not_built {
                continue;
            }
            assert!(have.contains(key), "{section:?} lacks {key}: {have:?}");
        }
    }
}

#[test]
fn no_setting_is_on_two_pages_and_the_removed_rows_are_gone() {
    let dir = tempfile::tempdir().unwrap();
    let st = state(dir.path());
    let mut seen: BTreeMap<&str, PrefSection> = BTreeMap::new();
    for section in PrefSection::ALL {
        for key in keys_of(&st, *section) {
            if let Some(other) = seen.insert(key, *section) {
                panic!("{key} is on both {other:?} and {section:?}");
            }
        }
    }
    for gone in ["ui.keymap_preset", "ui.graph_colorization"] {
        assert!(!seen.contains_key(gone), "{gone} was removed");
    }
}

#[test]
fn every_pooled_row_is_placed_on_a_page() {
    let dir = tempfile::tempdir().unwrap();
    let st = state(dir.path());
    let placed: Vec<&str> = crate::preferences::pages::tables()
        .iter()
        .flat_map(|(_, rows)| rows.iter())
        .filter_map(|r| match r {
            crate::preferences::pages::Row::Key(k) => Some(*k),
            _ => None,
        })
        .collect();
    for field in st.row_pool() {
        if let Target::Global(key) = field.target {
            assert!(placed.contains(&key), "{key} is built but on no page");
        }
    }
    // Every table entry resolves to a row (git_terms_english exists only
    // for some languages).
    let pool: Vec<&str> = st
        .row_pool()
        .iter()
        .filter_map(|f| match f.target {
            Target::Global(k) => Some(k),
            _ => None,
        })
        .collect();
    for key in placed {
        assert!(
            pool.contains(&key)
                // Dev mode only: built when `general.dev_mode` is on.
                || matches!(key, "ui.git_terms_english" | "ui.launcher_build")
                || cfg!(not(feature = "full")),
            "{key} is in a page table but nothing builds it"
        );
    }
}

#[test]
fn the_legacy_page_ids_still_resolve() {
    use gitpoulpi_ipc::ui::SettingsPage as P;
    assert_eq!(
        PrefSection::from_page(P::UiCustomization),
        PrefSection::Appearance
    );
    assert_eq!(PrefSection::from_page(P::Editor), PrefSection::DiffEditor);
    assert_eq!(
        PrefSection::from_page(P::KeyboardShortcuts),
        PrefSection::Keybindings
    );
    for section in PrefSection::ALL {
        let page = P::parse(section.id()).unwrap_or_else(|| panic!("{}", section.id()));
        assert_eq!(PrefSection::from_page(page), *section);
    }
    // A moved row is still found through the old page id.
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    st.show_page(Some(P::UiCustomization), Some("ui.minimap"))
        .unwrap();
    assert_eq!(st.current_section(), PrefSection::DiffEditor);
    st.show_page(Some(P::General), Some("general.auto_prune"))
        .unwrap();
    assert_eq!(st.current_section(), PrefSection::Sync);
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

/// The Integrations page draws its own body below a strip of generic rows:
/// Up from the body's top reaches it, Down from its last row hands the
/// keyboard back, and a row of it edits the config.
#[test]
fn up_from_a_custom_page_reaches_its_strip_and_down_leaves_it() {
    for section in [PrefSection::Integrations] {
        if !section.is_built() {
            continue;
        }
        let dir = tempfile::tempdir().unwrap();
        let mut st = state(dir.path());
        go(&mut st, section);
        st.focus = PrefFocus::Fields;
        assert!(!st.strip_active(), "{section:?}: the body has the keyboard");
        st.on_key(key(KeyCode::Up));
        assert!(st.strip_active(), "{section:?}: Up reaches the strip");
        let rows = st.fields().iter().filter(|f| f.selectable()).count();
        assert!(rows >= 2, "{section:?}");
        for _ in 0..rows + 2 {
            st.on_key(key(KeyCode::Down));
        }
        assert!(!st.strip_active(), "{section:?}: Down leaves the strip");
    }
}

/// The Keyboard & mouse page's generic rows are its first tab; the
/// keybindings editor is the second.
#[test]
fn a_keyboard_row_toggles_its_setting_and_the_editor_has_its_own_tab() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    go_tab(&mut st, PrefSection::Keybindings, SubTab::General);
    st.focus = PrefFocus::Fields;
    assert_eq!(st.sub_tab(), SubTab::General);
    assert!(!st.keys_active(), "the rows own the keyboard");
    st.field = st
        .fields()
        .iter()
        .position(|f| matches!(f.target, Target::Global("ui.mouse_enabled")))
        .unwrap();
    let before = st.config.ui.mouse_enabled;
    st.on_key(key(KeyCode::Enter));
    assert_ne!(st.config.ui.mouse_enabled, before);
    let screen = draw_screen(&mut st);
    assert!(screen.contains("Mouse enabled"), "{screen}");

    st.on_key(KeyEvent::new(KeyCode::Right, KeyModifiers::ALT));
    assert_eq!(st.sub_tab(), SubTab::List);
    assert!(st.keys_active(), "the editor owns the keyboard");
    let screen = draw_screen(&mut st);
    assert!(screen.contains("Keybindings"), "{screen}");
    assert!(!screen.contains("Mouse enabled"), "{screen}");

    st.on_key(KeyEvent::new(KeyCode::Left, KeyModifiers::ALT));
    assert_eq!(st.sub_tab(), SubTab::General);
}

/// The palette's `App: Keybindings...` and an agent's `keys.<id>` land on
/// the editor tab; the settings search finds both the tab and the rows.
#[test]
fn the_keybindings_tab_is_reached_by_the_palette_the_agent_and_the_search() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    st.show_keybindings(false);
    assert_eq!(st.sub_tab(), SubTab::List);
    assert!(st.keys_active());

    let mut st = state(dir.path());
    let id = crate::keymap::ACTIONS[0].id;
    st.show_page(None, Some(&format!("keys.{id}"))).unwrap();
    assert_eq!(st.sub_tab(), SubTab::List);
    let mut st = state(dir.path());
    st.show_page(None, Some("ui.mouse_enabled")).unwrap();
    assert_eq!(st.current_section(), PrefSection::Keybindings);
    assert_eq!(st.sub_tab(), SubTab::General);

    let entries = st.search_entries();
    let tab = entries
        .iter()
        .find(|e| e.label == "Keybindings" && e.tab == SubTab::List)
        .expect("the tab is searchable");
    let hit = crate::preferences::search::matches("keybindings", &entries);
    assert!(hit
        .iter()
        .any(|h| h.tab == SubTab::List && h.label == tab.label));
    let hit = hit.into_iter().find(|h| h.tab == SubTab::List).unwrap();
    st.jump_to(&hit);
    assert_eq!(st.sub_tab(), SubTab::List);
    assert!(st.keys_active());
    assert!(
        entries
            .iter()
            .any(|e| e.label.contains("Mouse enabled") && e.tab == SubTab::General),
        "the generic rows are still found"
    );
}

/// The History workbench's settings (the script limits on the Sync page, the
/// two AI blocks of its scripts and rules) are on the pages exactly when the
/// build has the workbench.
#[test]
fn the_workbench_settings_follow_the_feature() {
    let dir = tempfile::tempdir().unwrap();
    let st = state(dir.path());
    let sync = keys_of(&st, PrefSection::Sync);
    assert_eq!(
        sync.contains(&"rewrite.script_timeout_ms"),
        crate::build_features::HISTORY_REWRITE
    );
    assert_eq!(
        sync.contains(&"rewrite.script_total_timeout_s"),
        crate::build_features::HISTORY_REWRITE
    );
    let sync_headings: Vec<String> = st
        .fields_of(PrefSection::Sync)
        .iter()
        .filter(|f| matches!(f.control, Control::Heading))
        .map(|f| f.label.clone())
        .collect();
    assert_eq!(
        sync_headings
            .iter()
            .any(|h| h == gitpoulpi_i18n::msg::settings_general::HEADING_REWRITE.get()),
        crate::build_features::HISTORY_REWRITE
    );
    #[cfg(feature = "ai")]
    {
        let ai_titles: Vec<String> = st
            .ai_general_fields()
            .iter()
            .map(|f| f.label.clone())
            .collect();
        for title in [
            gitpoulpi_i18n::msg::rewrite_ai::TITLE,
            gitpoulpi_i18n::msg::rewrite_rules_ai::TITLE,
        ] {
            assert_eq!(
                ai_titles.iter().any(|t| t == title.get()),
                crate::build_features::HISTORY_REWRITE,
                "{}",
                title.get()
            );
        }
    }
}

#[test]
fn the_launcher_and_conflict_prevention_have_their_own_pages_the_palette_and_ids_reach() {
    let dir = tempfile::tempdir().unwrap();
    for (key, section, id) in [
        ("ui.launch_terminal", PrefSection::Launcher, "launcher"),
        (
            "general.conflict_detection",
            PrefSection::ConflictGlobal,
            "conflict-prevention-global",
        ),
    ] {
        let mut st = state(dir.path());
        // The palette's `Settings: ...` row opens the row by its key.
        st.show_page(None, Some(key)).unwrap();
        assert_eq!(st.current_section(), section, "{key}");
        assert_eq!(section.id(), id);
        assert!(!section.is_repo_scoped());
        let page = gitpoulpi_ipc::ui::SettingsPage::parse(id).unwrap();
        assert_eq!(PrefSection::from_page(page), section);
        // ... and the page's id opens the page.
        let mut st = state(dir.path());
        st.show_page(Some(page), None).unwrap();
        assert_eq!(st.current_section(), section);
        // General no longer lists the row.
        assert!(!keys_of(&st, PrefSection::General).contains(&key));
    }
}
