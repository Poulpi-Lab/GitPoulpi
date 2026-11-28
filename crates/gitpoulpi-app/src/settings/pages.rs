//! The pages of Settings and which setting lives on which page.
//!
//! Every plain config row is built once in the pool ([`super::pool`]); each
//! page table below lists the keys it shows, in order, with the headings
//! between them. Moving a setting is a one-line change here.

use gitpoulpi_i18n::msg::{self, settings_open_in, settings_screen as screen};

/// One page of the screen. The order is the sidebar's order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    General,
    Launcher,
    Sync,
    Profiles,
    Ssh,
    Integrations,
    Ai,
    AgentsMcp,
    Appearance,
    Keybindings,
    /// The user's named action sequences (`sequences_page`).
    Sequences,
    LogGraph,
    DiffEditor,
    OpenIn,
    Images,
    ConflictGlobal,
    GitAndEncoding,
    RepoAi,
    Workflow,
    SparseCheckout,
    Lfs,
    Commit,
    MergeBehavior,
    CommitTemplate,
    ConflictPrevention,
    RepoAuth,
}

impl Page {
    /// A master-detail page: a `ListEditor` on the left (stacked above on a
    /// narrow body) and the selected item's rows on the right. Profiles, AI,
    /// Open in and Images so far; Workflow follows (plan M5).
    pub fn has_list(self) -> bool {
        matches!(
            self,
            Page::Profiles
                | Page::Ai
                | Page::OpenIn
                | Page::Images
                | Page::Workflow
                | Page::Sequences
        )
    }

    pub const ALL: &'static [Page] = &[
        Page::General,
        Page::Launcher,
        Page::Sync,
        Page::Profiles,
        Page::Ssh,
        Page::Integrations,
        Page::Ai,
        Page::AgentsMcp,
        Page::Appearance,
        // Right after the first core pages: it is read by everyone.
        Page::Keybindings,
        Page::Sequences,
        Page::LogGraph,
        Page::DiffEditor,
        Page::OpenIn,
        Page::Images,
        // Last of the global pages: it is specific.
        Page::ConflictGlobal,
        Page::GitAndEncoding,
        Page::RepoAuth,
        Page::RepoAi,
        Page::Workflow,
        Page::SparseCheckout,
        Page::Lfs,
        Page::Commit,
        Page::MergeBehavior,
        Page::CommitTemplate,
        Page::ConflictPrevention,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Page::General => screen::SECTION_GENERAL.get(),
            Page::Launcher => gitpoulpi_i18n::msg::launcher::SECTION.get(),
            Page::ConflictGlobal => gitpoulpi_i18n::msg::conflict_prevention::SECTION_GLOBAL.get(),
            Page::Sync => screen::SECTION_SYNC.get(),
            Page::Profiles => screen::SECTION_PROFILES.get(),
            Page::Ssh => screen::SECTION_SSH.get(),
            Page::Integrations => screen::SECTION_INTEGRATIONS.get(),
            Page::Ai => screen::SECTION_AI.get(),
            Page::AgentsMcp => screen::SECTION_AGENTS.get(),
            Page::Appearance => screen::SECTION_UI.get(),
            Page::LogGraph => screen::SECTION_LOG.get(),
            Page::Keybindings => screen::SECTION_KEYBINDINGS.get(),
            Page::Sequences => gitpoulpi_i18n::msg::settings_sequences::SECTION_TITLE.get(),
            Page::DiffEditor => screen::SECTION_EDITOR.get(),
            Page::OpenIn => settings_open_in::SECTION_TITLE.get(),
            Page::Images => gitpoulpi_i18n::msg::settings_images::SECTION_TITLE.get(),
            Page::GitAndEncoding => screen::SECTION_GIT.get(),
            Page::RepoAi => screen::SECTION_AI.get(),
            Page::Workflow => gitpoulpi_i18n::msg::settings_workflow::SECTION_TITLE.get(),
            Page::SparseCheckout => gitpoulpi_i18n::msg::settings_repo::SECTION_SPARSE.get(),
            Page::Lfs => gitpoulpi_i18n::msg::settings_repo::SECTION_LFS.get(),
            Page::Commit => gitpoulpi_i18n::msg::settings_repo::SECTION_COMMIT.get(),
            Page::MergeBehavior => gitpoulpi_i18n::msg::settings_repo::SECTION_MERGE.get(),
            Page::CommitTemplate => gitpoulpi_i18n::msg::settings_repo::SECTION_TEMPLATE.get(),
            Page::ConflictPrevention => gitpoulpi_i18n::msg::conflict_prevention::SECTION.get(),
            Page::RepoAuth => gitpoulpi_i18n::msg::repo_auth::SECTION.get(),
        }
    }

    /// The stable id the IPC/MCP `ui show settings` surface names this page
    /// by (`gitpoulpi_ipc::ui::SettingsPage::IDS`).
    pub const fn id(self) -> &'static str {
        match self {
            Page::General => "general",
            Page::Launcher => "launcher",
            Page::ConflictGlobal => "conflict-prevention-global",
            Page::Sync => "sync",
            Page::Profiles => "profiles",
            Page::Ssh => "ssh",
            Page::Integrations => "integrations",
            Page::Ai => "ai",
            Page::AgentsMcp => "agents-mcp",
            Page::Appearance => "appearance",
            Page::LogGraph => "log-graph",
            Page::DiffEditor => "diff-editor",
            Page::OpenIn => "open-in",
            Page::Images => "images",
            Page::Keybindings => "keyboard-mouse",
            Page::Sequences => "sequences",
            Page::GitAndEncoding => "git-encoding",
            Page::RepoAi => "repo-ai",
            Page::Workflow => "workflow",
            Page::SparseCheckout => "sparse-checkout",
            Page::Lfs => "lfs",
            Page::Commit => "commit",
            Page::MergeBehavior => "merge-behavior",
            Page::CommitTemplate => "commit-template",
            Page::ConflictPrevention => "conflict-prevention",
            Page::RepoAuth => "ssh-credentials",
        }
    }

    /// The cargo feature this page belongs to; `None` = always built.
    pub const fn requires(self) -> Option<crate::features::Feature> {
        match self {
            Page::Ai | Page::RepoAi => Some(crate::features::Feature::Ai),
            Page::Integrations => Some(crate::features::Feature::Hosting),
            Page::Lfs => Some(crate::features::Feature::Lfs),
            _ => None,
        }
    }

    /// Whether this build shows the page.
    pub fn is_built(self) -> bool {
        crate::features::offered(self.requires())
    }

    /// The page index reached from `from` by `delta` steps over the pages
    /// `built` accepts (stops at the first or last such page).
    pub fn step(from: usize, delta: isize, built: impl Fn(Page) -> bool) -> usize {
        let len = Self::ALL.len() as isize;
        let step = delta.signum();
        let mut at = from as isize;
        for _ in 0..delta.abs() {
            let mut probe = at + step;
            while (0..len).contains(&probe) && !built(Self::ALL[probe as usize]) {
                probe += step;
            }
            if !(0..len).contains(&probe) {
                break;
            }
            at = probe;
        }
        at as usize
    }

    /// `true` for the pages that draw their own body below a strip of
    /// generic rows.
    pub fn has_strip(self) -> bool {
        self == Page::Integrations
    }

    /// Whether the page is split into a General tab and a list tab.
    pub fn has_sub_tabs(self) -> bool {
        (self.has_list() && self != Page::Sequences) || self == Page::Keybindings
    }

    /// The list tab's label: the items' name, as the list titles it.
    pub fn list_tab_title(self) -> String {
        use gitpoulpi_i18n::msg as m;
        match self {
            Page::Profiles => m::settings_profiles::LIST_TITLE.get().to_string(),
            Page::Ai => m::ai_page::PROFILES_HEADING.get().to_string(),
            Page::OpenIn => m::settings_open_in::LIST_TITLE.get().to_string(),
            Page::Images => m::settings_images::LIST_TITLE.get().to_string(),
            Page::Workflow => m::settings_workflow::TAB_RULES.get().to_string(),
            Page::Keybindings => screen::TAB_KEYBINDINGS.get().to_string(),
            _ => String::new(),
        }
    }

    /// `true` for the pages under `Repo-Specific Settings`.
    pub fn is_repo_scoped(self) -> bool {
        matches!(
            self,
            Page::GitAndEncoding
                | Page::RepoAi
                | Page::Workflow
                | Page::SparseCheckout
                | Page::Lfs
                | Page::Commit
                | Page::MergeBehavior
                | Page::CommitTemplate
                | Page::ConflictPrevention
                | Page::RepoAuth
        )
    }
}

/// One entry of a page table.
#[derive(Clone, Copy)]
pub enum Entry {
    /// The pool's row for this config key (skipped when this build or
    /// language does not offer it).
    Key(&'static str),
    /// A sub-heading.
    Heading(fn() -> &'static str),
}
use Entry::{Heading, Key};

pub const GENERAL: &[Entry] = &[
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
];

/// The Launcher page: the launcher's own rows; its per-place rows are
/// appended after these (see `launcher_fields`), so nothing above them shifts.
pub const LAUNCHER: &[Entry] = &[
    Heading(|| msg::launcher::THIS_HEADING.get()),
    Key("installation.info"),
    Key("installation.name"),
    Key("installation.name_with_version"),
    Key("installation.logo_color"),
    Key("installation.primary"),
    Key("installation.env"),
    Key("installation.folder_menu_submenu"),
    Heading(|| msg::launcher::HEADING.get()),
    Key("ui.launch_terminal"),
    Key("ui.launcher_build"),
    Key("ui.relaunch_in_terminal"),
    Key("ui.reuse_window"),
    Key("installation.command_alias"),
    Key("installation.command_alias_add"),
];

/// The global part of proactive conflict detection: the default switch; the
/// default list and the named lists follow it (see `conflict_global_page_fields`).
pub const CONFLICT_GLOBAL: &[Entry] = &[
    Heading(|| msg::conflict_prevention::GLOBAL_HEADING.get()),
    Key("general.conflict_detection"),
];

pub const SYNC: &[Entry] = &[
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
pub const SYNC_REWRITE: &[Entry] = &[
    Heading(|| msg::settings_general::HEADING_REWRITE.get()),
    Key("rewrite.script_timeout_ms"),
    Key("rewrite.script_total_timeout_s"),
];

/// The generic rows above the Integrations page's own body.
pub const INTEGRATIONS: &[Entry] = &[
    Key("general.hosting_refresh_minutes"),
    Key("general.hosting_post_as"),
    Key("ui.pr_terminology"),
    Key("ui.hide_finished_pr_chip"),
];

/// The two MCP server rows only exist in a build with the server (S5):
/// `picked` skips a key that is not in the pool.
pub const AGENTS: &[Entry] = &[
    Key("mcp.enabled"),
    Key("mcp.port"),
    Key("mcp.suggest_port"),
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

pub const APPEARANCE: &[Entry] = &[
    Key("ui.theme"),
    Key("ui.avatars"),
    Key("ui.avatar_sources"),
    Key("ui.avatar_fallback"),
    Key("ui.input_color"),
    Key("ui.input_color_custom"),
    Key("ui.color_depth"),
    Key("ui.text_cursor"),
    Key("ui.graph_color_theme"),
    Key("ui.graph_lane_background"),
    Key("ui.graph_row_tint_strength"),
    Heading(|| msg::settings_ui::HEADING_LAYOUT.get()),
    Key("ui.toolbar_position"),
    Key("ui.toolbar_align"),
    Key("ui.dialog_geometry"),
    Key("ui.toolbar_pr_pinned"),
    Key("ui.statusbar_position"),
    Key("ui.tabs_position"),
    Heading(|| msg::settings_ui::HEADING_DATE_TIME.get()),
    Key("ui.date_locale"),
    Key("ui.date_format"),
    Key("ui.date_time_format"),
    Key("ui.date_verbose_format"),
];

pub const LOG_GRAPH: &[Entry] = &[
    Key("ui.graph_renderer"),
    Key("ui.image_protocol"),
    Key("ui.graph_show_stashes"),
    Key("ui.graph_ghost_ref"),
    Key("ui.ref_connector"),
    Key("ui.column_resize"),
    Key("ui.log_columns_layout"),
    Key("ui.log_wip_counts_in_message"),
    Key("ui.ref_tree"),
    Key("ui.tree_folders_first"),
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

pub const DIFF_EDITOR: &[Entry] = &[
    Key("editor.tab_size"),
    Key("editor.eol"),
    Key("editor.syntax_highlighting"),
    Key("editor.line_numbers"),
    Key("editor.follow_selection"),
    Key("ui.diff_word_wrap"),
    Key("ui.diff_intraline"),
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
pub const OPEN_IN: &[Entry] = &[Key("ui.tab_dir_follow")];

/// The viewer rows of the Images page, above its editor list.
pub const IMAGES: &[Entry] = &[
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
pub const IMAGES_EDITORS_ONLY: &[Entry] =
    &[Heading(|| msg::settings_images::HEADING_EDITORS.get())];

/// The generic rows above the Keyboard & mouse page's shortcut editor.
pub const KEYBOARD: &[Entry] = &[
    Key("ui.kitty_keyboard"),
    Key("ui.mouse_enabled"),
    Key("ui.mouse_scroll_lines"),
];

/// Every table with the page it belongs to (a page can have several: the
/// rewrite limits of Synchronization come after its own rows).
pub fn tables() -> [(Page, &'static [Entry]); 13] {
    [
        (Page::General, GENERAL),
        (Page::Launcher, LAUNCHER),
        (Page::ConflictGlobal, CONFLICT_GLOBAL),
        (Page::Sync, SYNC),
        (Page::Sync, SYNC_REWRITE),
        (Page::Integrations, INTEGRATIONS),
        (Page::AgentsMcp, AGENTS),
        (Page::Appearance, APPEARANCE),
        (Page::LogGraph, LOG_GRAPH),
        (Page::DiffEditor, DIFF_EDITOR),
        (Page::OpenIn, OPEN_IN),
        (Page::Images, IMAGES),
        (Page::Keybindings, KEYBOARD),
    ]
}

/// The rows of `table`, in its order, taken out of `pool`. `key_of` names the
/// config key a pool row writes, `is_preview` tells a read-only line with no
/// key (a date pattern's live preview, which follows its pattern) and
/// `heading` makes a sub-heading row. A key the pool does not have (this
/// build or language does not offer it) is skipped. A frontend with a row
/// type of its own calls this with its own accessors; [`page_rows`] is the
/// same over the shared [`Row`].
pub fn pick<R>(
    table: &[Entry],
    mut pool: Vec<R>,
    key_of: impl Fn(&R) -> Option<&'static str>,
    is_preview: impl Fn(&R) -> bool,
    heading: impl Fn(&str) -> R,
) -> Vec<R> {
    let mut out = Vec::new();
    for entry in table {
        match entry {
            Heading(text) => out.push(heading(text())),
            Key(key) => {
                let Some(at) = pool.iter().position(|f| key_of(f) == Some(*key)) else {
                    continue;
                };
                out.push(pool.remove(at));
                // A date pattern's preview follows it.
                if pool.get(at).is_some_and(&is_preview) {
                    out.push(pool.remove(at));
                }
            }
        }
    }
    out
}

/// The shared rows of one page table, from a fresh pool.
pub fn page_rows(table: &[Entry], cx: &super::PoolCtx) -> Vec<super::Row> {
    pick(
        table,
        super::pool(cx),
        |r| r.key(),
        |r| {
            matches!(r.target, super::Target::None)
                && matches!(
                    r.control,
                    super::Control::Info(_) | super::Control::Swatches(_)
                )
        },
        super::Row::heading,
    )
}
