//! The generated `Settings: ...` rows are pinned: their count and the config
//! key of each, in order. The rows come from the shared Settings model
//! (`gitpoulpi_app::settings`); this fails when a setting is added, removed or
//! moved, so that is decided on purpose (and the parity table with it).

#![cfg(feature = "full")]

use gitpoulpi_app::palette::{PaletteCommand, SettingOp};

/// Every setting of a full build in English, in the Settings order.
const KEYS: &[&str] = &[
    "general.auto_fetch_interval_minutes",
    "general.fetch_on_tab_open",
    "general.auto_prune",
    "general.auto_prune_tags",
    "general.keep_submodules_up_to_date",
    "general.default_branch_name",
    "general.conflict_detection",
    "general.delete_orig_files_after_merge",
    "general.initial_commits_in_graph",
    "general.lazy_load_commits_in_graph",
    "general.commits_per_lazy_page",
    "general.max_commits_in_graph",
    "general.undo_enabled",
    "general.undo_depth",
    "general.journal_outside",
    "log.level",
    "log.retention_days",
    "log.max_file_mb",
    "log.timings",
    "log.extended",
    "log.commit_count",
    "log.use_mailmap",
    "general.remember_tabs",
    "general.radar_enabled",
    "general.startup_tab_in_repo",
    "general.startup_tab_outside_repo",
    "general.startup_tab_launcher",
    "general.group_worktree_tabs",
    "ui.allow_agent_view_control",
    "mcp.allow_ui_propose",
    "repos.clone_root",
    "repos.clone_start_source",
    "repos.browse_root",
    "session.per_repo_state_dir",
    "general.auto_stash",
    "general.include_untracked_in_autostash",
    "updates.mode",
    "updates.channel",
    "general.hosting_refresh_minutes",
    "general.hosting_post_as",
    "rewrite.script_timeout_ms",
    "rewrite.script_total_timeout_s",
    "ui.pr_terminology",
    "ui.hide_finished_pr_chip",
    "general.external_editor",
    "general.external_diff_tool",
    "general.external_merge_tool",
    "general.file_explorer",
    "ui.launch",
    "ui.launch_command",
    "ui.tab_dir_follow",
    "ui.language",
    "ui.translate_git_terms",
    "ui.theme",
    "ui.graph_lane_background",
    "ui.graph_row_tint_strength",
    "ui.toolbar_position",
    "ui.toolbar_align",
    "ui.dialog_geometry",
    "ui.toolbar_pr_pinned",
    "ui.statusbar_position",
    "ui.tabs_position",
    "ui.graph_color_theme",
    "ui.graph_renderer",
    "ui.image_protocol",
    "ui.color_depth",
    "ui.text_cursor",
    "ui.kitty_keyboard",
    "ui.graph_show_stashes",
    "ui.graph_ghost_ref",
    "ui.ref_connector",
    "ui.log_wip_counts_in_message",
    "ui.avatars",
    "ui.avatar_sources",
    "ui.avatar_fallback",
    "ui.composer_files",
    "ui.ref_tree",
    "ui.tree_folders_first",
    "ui.reveal_on_ref_click",
    "ui.mouse_enabled",
    "ui.mouse_scroll_lines",
    "ui.date_locale",
    "ui.date_format",
    "ui.date_time_format",
    "ui.date_verbose_format",
    "ui.diff_layout",
    "ui.diff_hunks_only",
    "ui.diff_show_whitespace",
    "ui.input_color",
    "ui.input_color_custom",
    "ui.minimap",
    "ui.sidebar_sections_scope",
    "ui.column_resize",
    "ui.log_columns_layout",
    "ui.blame_side",
    "ui.blame_width",
    "ui.image_view",
    "ui.image_view_text_fallback",
    "ui.image_view_max_mib",
    "ui.image_view_pixelated_from",
    "ui.image_view_sync",
    "editor.tab_size",
    "editor.eol",
    "editor.syntax_highlighting",
    "editor.line_numbers",
    "editor.follow_selection",
    "ui.diff_word_wrap",
    "ui.diff_intraline",
    "merge.side_by_side_min_width",
    "installation.name",
    "installation.name_with_version",
    "installation.logo_color",
    "installation.primary",
    "installation.env",
    "installation.folder_menu_submenu",
    "ui.launch_terminal",
    "ui.relaunch_in_terminal",
    "ui.reuse_window",
    "installation.command_alias",
    "installation.command_alias_add",
    "mcp.enabled",
    "mcp.port",
    "mcp.suggest_port",
    "mcp.http_auth",
    "mcp.token",
    "mcp.tls",
    "mcp.tls_cert",
    "mcp.tls_key",
    "mcp.behind_proxy",
];

#[test]
fn the_settings_rows_are_the_pinned_list() {
    gitpoulpi_i18n::with_lang(gitpoulpi_i18n::Lang::En, || {
        let keys: Vec<&str> = gitpoulpi_app::settings::rows()
            .iter()
            .map(|r| r.key)
            .collect();
        assert_eq!(keys, KEYS);
    });
}

#[test]
fn the_palette_lists_one_entry_per_setting_and_one_per_choice() {
    gitpoulpi_i18n::with_lang(gitpoulpi_i18n::Lang::En, || {
        let entries = gitpoulpi_app::settings::palette_rows::entries();
        assert_eq!(entries.len(), 257);
        let chosen = entries
            .iter()
            .filter(|e| matches!(e.command, PaletteCommand::Setting(SettingOp::Choose(..))))
            .count();
        assert_eq!(entries.len() - chosen, KEYS.len());
        assert_eq!(
            gitpoulpi_app::settings::palette_rows::listed_count(),
            KEYS.len()
        );
    });
}

/// The model the terminal UI's own pages build agrees with the shared one:
/// the same keys, labels and kinds, in the same order. The rows of the machine
/// and the installation are described by `gitpoulpi_app::settings::host`; this
/// is what catches a label or a kind changing on one side only.
#[test]
fn the_pages_rows_and_the_shared_rows_agree() {
    gitpoulpi_i18n::with_lang(gitpoulpi_i18n::Lang::En, || {
        let tui = crate::preferences::palette_settings::rows();
        let shared = gitpoulpi_app::settings::rows();
        let describe = |rows: &[gitpoulpi_app::settings::SettingRow]| {
            rows.iter()
                .map(|r| format!("{} | {} | {:?}", r.key, r.label, r.kind))
                .collect::<Vec<_>>()
        };
        assert_eq!(describe(&tui), describe(&shared));
    });
}
