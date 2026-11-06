//! Which setting lives on which Settings page.
//!
//! Every plain `config.toml` row is built once in a pool (`general_rows`,
//! `ui_rows`, `editor_rows`) and each page below lists the keys it shows, in
//! order, with the headings between them. Moving a setting is a one-line
//! change here; `tests::pages` pins the mapping.

use super::*;
use gitpoulpi_i18n::msg;

/// One entry of a page table.
#[derive(Clone, Copy)]
pub(super) enum Row {
    /// The pool's row for this config key (skipped when this build or
    /// language does not offer it).
    Key(&'static str),
    /// A sub-heading.
    Heading(fn() -> &'static str),
}
use Row::{Heading, Key};

pub(super) const GENERAL: &[Row] = &[
    Key("ui.language"),
    Key("ui.translate_git_terms"),
    Key("ui.git_terms_english"),
    Key("general.remember_tabs"),
    Key("general.radar_enabled"),
    Key("general.startup_tab_in_repo"),
    Key("general.startup_tab_outside_repo"),
    Key("general.startup_tab_launcher"),
    Key("general.group_worktree_tabs"),
    Key("repos.clone_root"),
    Key("repos.clone_start_source"),
    Key("repos.browse_root"),
    Key("session.per_repo_state_dir"),
    Key("general.default_branch_name"),
    Key("general.conflict_detection"),
    Heading(|| msg::settings_general::HEADING_UNDO.get()),
    Key("general.undo_enabled"),
    Key("general.undo_depth"),
    Key("general.journal_outside"),
    Heading(|| msg::settings_general::HEADING_LOG.get()),
    Key("log.level"),
    Key("log.retention_days"),
    Key("log.max_file_mb"),
    Key("log.timings"),
    Key("log.extended"),
    // Last on the page: the launcher's per-place rows are appended after
    // these (see `general_fields`), so nothing above them shifts.
    Heading(|| msg::launcher::HEADING.get()),
    Key("ui.launch_terminal"),
    Key("ui.launcher_build"),
    Key("ui.relaunch_in_terminal"),
    Key("ui.reuse_window"),
    Key("ui.command_alias"),
    Key("ui.command_alias_add"),
];

pub(super) const SYNC: &[Row] = &[
    Key("general.auto_fetch_interval_minutes"),
    Key("general.fetch_on_tab_open"),
    Key("general.auto_prune"),
    Key("general.auto_prune_tags"),
    Key("general.keep_submodules_up_to_date"),
    Key("general.delete_orig_files_after_merge"),
    Heading(|| msg::settings_general::HEADING_AUTO_STASH.get()),
    Key("general.auto_stash"),
    Key("general.include_untracked_in_autostash"),
    Heading(|| gitpoulpi_i18n::msg::updates::HEADING.get()),
    Key("updates.mode"),
    Key("updates.channel"),
];

/// The History workbench's script limits, below [`SYNC`]'s rows; the page
/// only has them in a build with the workbench.
pub(super) const SYNC_REWRITE: &[Row] = &[
    Heading(|| msg::settings_general::HEADING_REWRITE.get()),
    Key("rewrite.script_timeout_ms"),
    Key("rewrite.script_total_timeout_s"),
];

/// The generic rows above the Integrations page's own body.
pub(super) const INTEGRATIONS: &[Row] = &[
    Key("general.hosting_refresh_minutes"),
    Key("ui.pr_terminology"),
    Key("ui.hide_finished_pr_chip"),
];

/// The two MCP server rows only exist in a build with the server (S5):
/// `picked` skips a key that is not in the pool.
pub(super) const AGENTS: &[Row] = &[
    Key("mcp.enabled"),
    Key("mcp.port"),
    Key("ui.allow_agent_view_control"),
    Key("mcp.allow_ui_propose"),
    Key("ui.launch"),
    Key("ui.launch_command"),
    Heading(|| msg::mcp_security::HEADING.get()),
    Key("mcp.http_auth"),
    Key("mcp.token"),
    Key("mcp.tls"),
    Key("mcp.tls_cert"),
    Key("mcp.tls_key"),
    Key("mcp.behind_proxy"),
];

pub(super) const APPEARANCE: &[Row] = &[
    Key("ui.theme"),
    Key("ui.avatars"),
    Key("ui.avatar_sources"),
    Key("ui.avatar_fallback"),
    Key("ui.input_color"),
    Key("ui.input_color_custom"),
    Key("ui.color_depth"),
    Key("ui.graph_color_theme"),
    Key("ui.graph_lane_background"),
    Key("ui.graph_row_tint_strength"),
    Heading(|| msg::settings_ui::HEADING_LAYOUT.get()),
    Key("ui.toolbar_position"),
    Key("ui.toolbar_align"),
    Key("ui.toolbar_pr_pinned"),
    Key("ui.statusbar_position"),
    Key("ui.tabs_position"),
    Heading(|| msg::settings_ui::HEADING_DATE_TIME.get()),
    Key("ui.date_locale"),
    Key("ui.date_format"),
    Key("ui.date_time_format"),
    Key("ui.date_verbose_format"),
];

pub(super) const LOG_GRAPH: &[Row] = &[
    Key("ui.graph_renderer"),
    Key("ui.image_protocol"),
    Key("ui.graph_show_stashes"),
    Key("ui.graph_ghost_ref"),
    Key("ui.ref_connector"),
    Key("ui.column_resize"),
    Key("ui.log_columns_layout"),
    Key("ui.log_wip_counts_in_message"),
    Key("ui.ref_tree"),
    Key("ui.sidebar_sections_scope"),
    Key("ui.reveal_on_ref_click"),
    Key("ui.composer_files"),
    Key("general.initial_commits_in_graph"),
    Key("general.lazy_load_commits_in_graph"),
    Key("general.commits_per_lazy_page"),
    Key("general.max_commits_in_graph"),
    Key("log.commit_count"),
    Key("log.use_mailmap"),
];

pub(super) const DIFF_EDITOR: &[Row] = &[
    Key("editor.tab_size"),
    Key("editor.eol"),
    Key("editor.syntax_highlighting"),
    Key("editor.line_numbers"),
    Key("editor.follow_selection"),
    Key("ui.diff_word_wrap"),
    Key("ui.diff_layout"),
    Key("ui.diff_hunks_only"),
    Key("ui.diff_show_whitespace"),
    Key("ui.minimap"),
    Key("ui.blame_side"),
    Key("ui.blame_width"),
    Key("merge.side_by_side_min_width"),
    Heading(|| msg::settings_general::HEADING_EXTERNAL_TOOLS.get()),
    Key("general.external_editor"),
    Key("general.external_diff_tool"),
    Key("general.external_merge_tool"),
    Key("general.file_explorer"),
];

/// Added to the Open in page (before its own list).
pub(super) const OPEN_IN: &[Row] = &[Key("ui.tab_dir_follow")];

/// The viewer rows of the Images page, above its editor list.
pub(super) const IMAGES: &[Row] = &[
    Heading(|| msg::settings_images::HEADING_VIEWER.get()),
    Key("ui.image_view"),
    Key("ui.image_view_text_fallback"),
    Key("ui.image_view_max_mib"),
    Key("ui.image_view_pixelated_from"),
    Key("ui.image_view_sync"),
    Heading(|| msg::settings_images::HEADING_EDITORS.get()),
];

/// The Images page of a build without `image-view`: no viewer, so only the
/// editors part (Edit in still works on a file the viewer cannot show).
pub(super) const IMAGES_EDITORS_ONLY: &[Row] =
    &[Heading(|| msg::settings_images::HEADING_EDITORS.get())];

/// The generic rows above the Keyboard & mouse page's shortcut editor.
pub(super) const KEYBOARD: &[Row] = &[
    Key("ui.kitty_keyboard"),
    Key("ui.mouse_enabled"),
    Key("ui.mouse_scroll_lines"),
];

/// The config keys every plain row is registered under, for the pools'
/// completeness test.
#[cfg(test)]
pub(super) fn tables() -> [(PrefSection, &'static [Row]); 11] {
    [
        (PrefSection::General, GENERAL),
        (PrefSection::Sync, SYNC),
        (PrefSection::Sync, SYNC_REWRITE),
        (PrefSection::Integrations, INTEGRATIONS),
        (PrefSection::AgentsMcp, AGENTS),
        (PrefSection::Appearance, APPEARANCE),
        (PrefSection::LogGraph, LOG_GRAPH),
        (PrefSection::DiffEditor, DIFF_EDITOR),
        (PrefSection::OpenIn, OPEN_IN),
        (PrefSection::Images, IMAGES),
        (PrefSection::Keybindings, KEYBOARD),
    ]
}

impl PreferencesState {
    /// Every plain `config.toml` row, unplaced, plus the date patterns'
    /// live previews (each right after its pattern).
    pub(super) fn row_pool(&self) -> Vec<Field> {
        let mut pool = self.general_rows();
        pool.extend(self.ui_rows());
        pool.extend(self.editor_rows());
        pool.extend(self.launcher_rows());
        #[cfg(feature = "mcp")]
        {
            pool.extend(self.mcp_rows());
            pool.extend(self.mcp_security_rows());
        }
        if !crate::build_features::SUBMODULE {
            pool.retain(|f| {
                !matches!(
                    f.target,
                    Target::Global("general.keep_submodules_up_to_date")
                )
            });
        }
        if !crate::build_features::RADAR {
            pool.retain(|f| !matches!(f.target, Target::Global("general.radar_enabled")));
        }
        pool
    }

    /// The rows of `table`, in its order, taken from a fresh pool.
    pub(super) fn picked(&self, table: &[Row]) -> Vec<Field> {
        let mut pool = self.row_pool();
        let mut out = Vec::new();
        for row in table {
            match row {
                Heading(text) => out.push(Field::heading(text())),
                Key(key) => {
                    let Some(at) = pool
                        .iter()
                        .position(|f| matches!(f.target, Target::Global(k) if k == *key))
                    else {
                        continue;
                    };
                    out.push(pool.remove(at));
                    // A date pattern's preview follows it.
                    if pool.get(at).is_some_and(|f| {
                        matches!(f.target, Target::None)
                            && matches!(f.control, Control::Info(_) | Control::Swatches(_))
                    }) {
                        out.push(pool.remove(at));
                    }
                }
            }
        }
        out
    }

    pub(super) fn general_fields(&self) -> Vec<Field> {
        let mut fields = self.picked(GENERAL);
        // The alias and PATH buttons sit right under the alias box; the
        // per-place shortcut rows follow.
        let alias_at = fields
            .iter()
            .position(|f| f.target == Target::Global("ui.command_alias_add"))
            .map_or(fields.len(), |i| i + 1);
        fields.splice(alias_at..alias_at, self.tools_action_rows());
        fields.extend(self.launcher_action_rows());
        fields
    }

    pub(super) fn sync_fields(&self) -> Vec<Field> {
        let mut fields = self.picked(SYNC);
        if crate::build_features::HISTORY_REWRITE {
            fields.extend(self.picked(SYNC_REWRITE));
        }
        fields
    }

    /// The agent-facing rows: the MCP server's switch and port (when this
    /// build has the server), the view-control and proposal switches and the
    /// "launch" pair.
    pub(super) fn agents_fields(&self) -> Vec<Field> {
        #[cfg_attr(not(feature = "mcp"), allow(unused_mut))]
        let mut fields = self.picked(AGENTS);
        // The fingerprint sits under the key path (`mcp_security`).
        #[cfg(feature = "mcp")]
        {
            let after = |fields: &[Field], key: &str| {
                fields
                    .iter()
                    .position(|f| matches!(f.target, Target::Global(k) if k == key))
                    .map(|i| i + 1)
            };
            if let Some(at) = after(&fields, "mcp.tls_key") {
                fields.splice(at..at, self.mcp_fingerprint_rows());
            }
        }
        // No server in this build: no security heading either.
        #[cfg(not(feature = "mcp"))]
        fields.retain(|f| f.label != msg::mcp_security::HEADING.get());
        fields
    }

    pub(super) fn appearance_fields(&self) -> Vec<Field> {
        self.picked(APPEARANCE)
    }

    pub(super) fn log_graph_fields(&self) -> Vec<Field> {
        self.picked(LOG_GRAPH)
    }

    pub(super) fn diff_editor_fields(&self) -> Vec<Field> {
        self.picked(DIFF_EDITOR)
    }

    /// Selectable rows shown above a page that draws its own body.
    pub(super) fn strip_fields(&self, section: PrefSection) -> Vec<Field> {
        match section {
            PrefSection::Integrations => self.picked(INTEGRATIONS),
            _ => Vec::new(),
        }
    }
}

impl PreferencesState {
    /// Whether the keyboard is in the generic rows of the current page: always
    /// on an ordinary page, only after moving up into the strip on a page
    /// that draws its own body below it.
    pub(super) fn list_focused(&self) -> bool {
        self.focus == PrefFocus::Fields && (!self.current_section().has_strip() || self.strip)
    }

    /// The strip's rows have the keyboard (not the page's own body).
    pub(super) fn strip_active(&self) -> bool {
        self.strip && self.focus == PrefFocus::Fields && self.current_section().has_strip()
    }

    /// Up from the top of a custom page's body moves into its strip.
    /// `true` when it did.
    pub(super) fn enter_strip_on(&mut self, key: &KeyEvent) -> bool {
        let section = self.current_section();
        if !section.has_strip()
            || self.focus != PrefFocus::Fields
            || self.strip
            || !matches!(key.code, KeyCode::Up)
            || !key.modifiers.is_empty()
        {
            return false;
        }
        if !self.integrations.at_top() {
            return false;
        }
        let fields = self.fields();
        let Some(last) = (0..fields.len()).rev().find(|i| fields[*i].selectable()) else {
            return false;
        };
        self.strip = true;
        self.field = last;
        true
    }

    /// Down from the strip's last row hands the keyboard to the page body.
    pub(super) fn leave_strip(&mut self) {
        self.strip = false;
        self.integrations.zone = crate::integrations_page::Zone::Types;
    }
}

impl PrefSection {
    /// `true` for the pages that draw their own body below a strip of
    /// generic rows (`strip_fields`).
    pub(super) fn has_strip(self) -> bool {
        self == PrefSection::Integrations
    }
}
