//! The action registry: one `ActionDef` per bindable action, with its
//! stable config id, context, label and default chords. Data only.
//!
//! Only contexts whose dispatch reads the keymap are registered, so nothing
//! is listed here that the event loop does not consult.

/// Where a binding is looked up (the config table name).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Ctx {
    Global,
    GitPrefix,
    NavPrefix,
    Prompt,
    Refs,
    Log,
    /// The Log's column header, while a column is picked.
    LogColumns,
    LogFiles,
    Status,
    Home,
    Diff,
    FileInsight,
    History,
    /// The commit column left of a file's view, while it has the keyboard.
    CommitColumn,
    Image,
    Editor,
    EditorPrompt,
    Merge,
    MergeSides,
    MergeCard,
    Rebase,
    Compose,
    Split,
    Rewrite,
    Reword,
    Composer,
    /// The shared Markdown composer (Write / Preview / Split) of the
    /// review box, the review summary, the issue form and the PR body.
    MdEditor,
    Dialog,
    FolderPicker,
    Preferences,
    AiPopup,
    Integrations,
    /// The Activity Log window, while it is open.
    ActivityLog,
    /// The palette commands that have no key of their own: one bindable
    /// action each (`palette.<command id>`), dispatched like `Global`.
    Palette,
}

impl Ctx {
    pub(crate) const ALL: &'static [Ctx] = &[
        Ctx::Global,
        Ctx::GitPrefix,
        Ctx::NavPrefix,
        Ctx::Prompt,
        Ctx::Refs,
        Ctx::Log,
        Ctx::LogColumns,
        Ctx::LogFiles,
        Ctx::Status,
        Ctx::Home,
        Ctx::Diff,
        Ctx::FileInsight,
        Ctx::History,
        Ctx::CommitColumn,
        Ctx::Image,
        Ctx::Editor,
        Ctx::EditorPrompt,
        Ctx::Merge,
        Ctx::MergeSides,
        Ctx::MergeCard,
        Ctx::Rebase,
        Ctx::Compose,
        Ctx::Split,
        Ctx::Rewrite,
        Ctx::Reword,
        Ctx::Composer,
        Ctx::MdEditor,
        Ctx::Dialog,
        Ctx::FolderPicker,
        Ctx::Preferences,
        Ctx::AiPopup,
        Ctx::Integrations,
        Ctx::ActivityLog,
        Ctx::Palette,
    ];

    /// A focused pane or the Home tab: a child of `global`.
    pub(crate) const fn is_pane(self) -> bool {
        matches!(
            self,
            Ctx::Refs | Ctx::Log | Ctx::LogFiles | Ctx::Status | Ctx::Home
        )
    }

    /// A central view (diff, File View): a child of `global`, like a pane.
    pub(crate) const fn is_view(self) -> bool {
        matches!(
            self,
            Ctx::Diff
                | Ctx::FileInsight
                | Ctx::History
                | Ctx::CommitColumn
                | Ctx::LogColumns
                | Ctx::Image
        )
    }

    /// A popup or form that owns the keyboard: its table is its own, and
    /// Enter, Tab and the arrows stay fixed in it.
    pub(crate) const fn is_popup(self) -> bool {
        matches!(
            self,
            Ctx::Rebase
                | Ctx::Compose
                | Ctx::Split
                | Ctx::Rewrite
                | Ctx::Reword
                | Ctx::MdEditor
                | Ctx::Dialog
                | Ctx::FolderPicker
                | Ctx::Preferences
                | Ctx::AiPopup
                | Ctx::Integrations
                | Ctx::EditorPrompt
        )
    }

    /// A context where text is being typed: a bare printable key cannot
    /// be bound there.
    pub(crate) const fn is_typing(self) -> bool {
        matches!(
            self,
            Ctx::Editor
                | Ctx::Merge
                | Ctx::Compose
                | Ctx::Reword
                | Ctx::Composer
                | Ctx::MdEditor
                | Ctx::Dialog
                | Ctx::FolderPicker
                | Ctx::Preferences
        )
    }

    /// The name the Settings page shows for the context.
    pub(crate) const fn title(self) -> gitpoulpi_i18n::M {
        use gitpoulpi_i18n::msg::keymap_ctx as t;
        match self {
            Ctx::Global => t::GLOBAL,
            Ctx::GitPrefix => t::GIT_PREFIX,
            Ctx::NavPrefix => t::NAV_PREFIX,
            Ctx::Prompt => t::PROMPT,
            Ctx::Refs => t::REFS,
            Ctx::Log => t::LOG,
            Ctx::LogColumns => t::LOG_COLUMNS,
            Ctx::LogFiles => t::LOG_FILES,
            Ctx::Status => t::STATUS,
            Ctx::Home => t::HOME,
            Ctx::Diff => t::DIFF,
            Ctx::FileInsight => t::FILE_INSIGHT,
            Ctx::History => t::HISTORY,
            Ctx::CommitColumn => t::COMMIT_COLUMN,
            Ctx::Image => t::IMAGE,
            Ctx::Editor => t::EDITOR,
            Ctx::EditorPrompt => t::EDITOR_PROMPT,
            Ctx::Merge => t::MERGE,
            Ctx::MergeSides => t::MERGE_SIDES,
            Ctx::MergeCard => t::MERGE_CARD,
            Ctx::Rebase => t::REBASE,
            Ctx::Compose => t::COMPOSE,
            Ctx::Split => t::SPLIT,
            Ctx::Rewrite => t::REWRITE,
            Ctx::Reword => t::REWORD,
            Ctx::Composer => t::COMPOSER,
            Ctx::MdEditor => t::MD_EDITOR,
            Ctx::Dialog => t::DIALOG,
            Ctx::FolderPicker => t::FOLDER_PICKER,
            Ctx::Preferences => t::PREFERENCES,
            Ctx::AiPopup => t::AI_POPUP,
            Ctx::Integrations => t::INTEGRATIONS,
            Ctx::ActivityLog => t::ACTIVITY_LOG,
            Ctx::Palette => t::PALETTE,
        }
    }

    pub(crate) const fn name(self) -> &'static str {
        match self {
            Ctx::Global => "global",
            Ctx::GitPrefix => "git_prefix",
            Ctx::NavPrefix => "nav_prefix",
            Ctx::Prompt => "prompt",
            Ctx::Refs => "refs",
            Ctx::Log => "log",
            Ctx::LogColumns => "log_columns",
            Ctx::LogFiles => "log_files",
            Ctx::Status => "status",
            Ctx::Home => "home",
            Ctx::Diff => "diff",
            Ctx::FileInsight => "file_insight",
            Ctx::History => "history",
            Ctx::CommitColumn => "commit_column",
            Ctx::Image => "image",
            Ctx::Editor => "editor",
            Ctx::EditorPrompt => "editor_prompt",
            Ctx::Merge => "merge",
            Ctx::MergeSides => "merge_sides",
            Ctx::MergeCard => "merge_card",
            Ctx::Rebase => "rebase",
            Ctx::Compose => "compose",
            Ctx::Split => "split",
            Ctx::Rewrite => "rewrite",
            Ctx::Reword => "reword",
            Ctx::Composer => "composer",
            Ctx::MdEditor => "md_editor",
            Ctx::Dialog => "dialog",
            Ctx::FolderPicker => "folder_picker",
            Ctx::Preferences => "preferences",
            Ctx::AiPopup => "ai_popup",
            Ctx::Integrations => "integrations",
            Ctx::ActivityLog => "activity_log",
            Ctx::Palette => "palette",
        }
    }

    /// Whether dispatch for this context goes through the keymap.
    pub(crate) const fn is_live(self) -> bool {
        true
    }

    /// Prefix tables look at the second key's `code` alone, exactly as
    /// the `match key.code` they replace did: modifiers are ignored and
    /// an uppercase letter is `Shift+<lower>`.
    pub(crate) const fn ignores_modifiers(self) -> bool {
        matches!(self, Ctx::GitPrefix | Ctx::NavPrefix)
    }

    /// The context the key is looked up in: palette commands are global
    /// keys, so they clash with `global` and are hidden by a pane's.
    pub(crate) const fn scope(self) -> Ctx {
        match self {
            Ctx::Palette => Ctx::Global,
            other => other,
        }
    }
}

/// The label of every action: the keymap catalog's, plus the comparison's
/// two (their strings live in the `[compare]` section with the rest of that
/// feature).
mod labels {
    pub(super) use gitpoulpi_i18n::msg::blame_view::{
        OP_CARD as FILE_INSIGHT_BLAME_CARD, OP_COPY_SHA as FILE_INSIGHT_BLAME_COPY_SHA,
        OP_GOTO as FILE_INSIGHT_BLAME_GOTO, OP_HISTORY as FILE_INSIGHT_BLAME_HISTORY,
        OP_NARROWER as FILE_INSIGHT_BLAME_NARROWER, OP_NEXT as FILE_INSIGHT_BLAME_NEXT,
        OP_NEXT_SAME as FILE_INSIGHT_BLAME_NEXT_SAME, OP_PARENT as FILE_INSIGHT_BLAME_PARENT,
        OP_PREV as FILE_INSIGHT_BLAME_PREV, OP_PREV_SAME as FILE_INSIGHT_BLAME_PREV_SAME,
        OP_SHOW_DIFF as FILE_INSIGHT_BLAME_DIFF, OP_WIDER as FILE_INSIGHT_BLAME_WIDER,
    };
    pub(super) use gitpoulpi_i18n::msg::compare::{
        KEYMAP_COMPARE_SELECTION, KEYMAP_NEXT_CARD, KEYMAP_SWAP,
    };
    pub(super) use gitpoulpi_i18n::msg::keymap_action::*;
    pub(super) use gitpoulpi_i18n::msg::mermaid::KEYMAP_COPY as KEYMAP_COPY_MERMAID;
    pub(super) use gitpoulpi_i18n::msg::tree_fold::{
        KEY_COLLAPSE_ALL as TREE_COLLAPSE_ALL, KEY_COLLAPSE_BELOW as TREE_COLLAPSE_BELOW,
        KEY_EXPAND_ALL as TREE_EXPAND_ALL, KEY_EXPAND_BELOW as TREE_EXPAND_BELOW,
    };
}

macro_rules! actions {
    // An entry may end with `, @Feature` (e.g. `, @Ai`): the action then
    // needs that cargo feature and is left out of every list (Settings,
    // palette, help, conflict detection) when the build lacks it.
    ($( $variant:ident => ($ctx:ident, $id:literal, $label:ident, [$($chord:literal),*] $(, @$req:ident)?) ),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub(crate) enum Action {
            $($variant),*,
            /// A palette command without a key of its own: an index into
            /// `PALETTE_COMMANDS` (see [`palette_defs`]).
            Palette(u16),
        }

        pub(crate) const STATIC_ACTIONS: &[ActionDef] = &[
            $(ActionDef {
                action: Action::$variant,
                id: $id,
                ctx: Ctx::$ctx,
                label: labels::$label,
                defaults: &[$($chord),*],
                rebindable: true,
                shadows_parent: false,
                hint: None,
                requires: actions!(@req $($req)?),
            }),*
        ];
    };
    (@req) => { None };
    (@req $feature:ident) => { Some(crate::build_features::Feature::$feature) };
}

/// Every bindable action: the registered ones, then one per palette command
/// that has no key action of its own. Built once; behaves as a slice.
#[derive(Clone, Copy)]
pub(crate) struct ActionList;

pub(crate) const ACTIONS: ActionList = ActionList;

impl std::ops::Deref for ActionList {
    type Target = [ActionDef];
    fn deref(&self) -> &[ActionDef] {
        all_actions()
    }
}

impl IntoIterator for ActionList {
    type Item = &'static ActionDef;
    type IntoIter = std::slice::Iter<'static, ActionDef>;
    fn into_iter(self) -> Self::IntoIter {
        all_actions().iter()
    }
}

fn all_actions() -> &'static [ActionDef] {
    static ALL: std::sync::OnceLock<Vec<ActionDef>> = std::sync::OnceLock::new();
    ALL.get_or_init(|| {
        let mut all = STATIC_ACTIONS.to_vec();
        all.extend(crate::palette::palette_action_defs());
        all
    })
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ActionDef {
    pub(crate) action: Action,
    /// Stable config key `"<ctx>.<name>"`. Never renamed.
    pub(crate) id: &'static str,
    pub(crate) ctx: Ctx,
    /// Settings list and conflict messages (display text only: the stable
    /// `id` and the config keys stay English).
    pub(crate) label: gitpoulpi_i18n::M,
    /// Default chords, parsed by `KeyChord::parse`.
    pub(crate) defaults: &'static [&'static str],
    /// `false` = listed read-only, still part of conflict detection.
    pub(crate) rebindable: bool,
    /// A default that deliberately overrides a parent binding.
    pub(crate) shadows_parent: bool,
    pub(crate) hint: Option<&'static str>,
    /// The cargo feature this action belongs to (`@Ai` in `actions!`);
    /// `None` = always built.
    pub(crate) requires: Option<crate::build_features::Feature>,
}

impl ActionDef {
    /// Whether this build offers the action (its feature, if any, is built).
    pub(crate) fn built(&self) -> bool {
        crate::build_features::offered(self.requires)
            // Every action of the rewrite workbench's own keys belongs to the
            // History workbench, whatever else (`@Ai`) it also needs.
            && (!matches!(self.ctx, Ctx::Rewrite)
                || crate::build_features::offered(Some(
                    crate::build_features::Feature::HistoryRewrite,
                )))
    }

    /// Whether dispatch for this action reads the keymap: every registered
    /// action is.
    pub(crate) const fn live(&self) -> bool {
        self.ctx.is_live()
    }
}

actions! {
    // ---- global: window chords ----
    GlobalGitPrefix => (Global, "global.git_prefix", GLOBAL_GIT_PREFIX, ["Ctrl+G"]),
    GlobalNavPrefix => (Global, "global.nav_prefix", GLOBAL_NAV_PREFIX, ["Ctrl+N"]),
    GlobalPalette => (Global, "global.palette", GLOBAL_PALETTE, ["Ctrl+P"]),
    GlobalSettings => (Global, "global.settings", GLOBAL_SETTINGS, ["Ctrl+,"]),
    GlobalHome => (Global, "global.home", GLOBAL_HOME, ["Ctrl+O"]),
    GlobalClone => (Global, "global.clone", GLOBAL_CLONE, ["Ctrl+Shift+N"]),
    GlobalNewTab => (Global, "global.new_tab", GLOBAL_NEW_TAB, ["Alt+T"]),
    GlobalWorkspaceSwitch => (Global, "global.workspace_switch", GLOBAL_WORKSPACE_SWITCH, ["Alt+W"], @Workspaces),
    GlobalRadar => (Global, "global.radar", GLOBAL_RADAR, ["Alt+R"], @Radar),
    GlobalWorkspaces => (Global, "global.workspaces", GLOBAL_WORKSPACES, ["Alt+J"], @Workspaces),
    GlobalTabMenu => (Global, "global.tab_menu", GLOBAL_TAB_MENU, ["Ctrl+T"]),
    TabOpenInPopup => (Global, "global.tab_open_in_popup", GLOBAL_TAB_OPEN_IN_POPUP, ["Alt+O"]),
    TabOpenInDefault => (Global, "global.tab_open_in_default", GLOBAL_TAB_OPEN_IN_DEFAULT, ["Ctrl+Alt+O"]),
    GlobalCloseTab => (Global, "global.close_tab", GLOBAL_CLOSE_TAB, ["Ctrl+W"]),
    GlobalNextTab => (Global, "global.next_tab", GLOBAL_NEXT_TAB, ["Ctrl+PageDown", "Ctrl+Tab"]),
    GlobalPrevTab => (Global, "global.prev_tab", GLOBAL_PREV_TAB, ["Ctrl+PageUp", "Ctrl+Shift+Tab"]),
    // ---- global: plain keys ----
    GlobalHelp => (Global, "global.help", GLOBAL_HELP, ["?"]),
    GlobalQuit => (Global, "global.quit", GLOBAL_QUIT, ["q"]),
    GlobalRefresh => (Global, "global.refresh", GLOBAL_REFRESH, ["r"]),
    GlobalRefreshAll => (Global, "global.refresh_all", GLOBAL_REFRESH_ALL, ["F5"]),
    GlobalFindInLog => (Global, "global.find_in_log", GLOBAL_FIND_IN_LOG, ["f"]),
    GlobalWorkspaceSearch => (Global, "global.workspace_search", GLOBAL_WORKSPACE_SEARCH, ["Ctrl+Alt+F"], @GlobalSearch),
    GlobalSelectTab => (Global, "global.select_tab", GLOBAL_SELECT_TAB, ["t"]),
    GlobalUndo => (Global, "global.undo", GLOBAL_UNDO, ["Ctrl+Z"]),
    GlobalRedo => (Global, "global.redo", GLOBAL_REDO, ["Ctrl+Y", "Ctrl+Shift+Z"]),
    GlobalUndoHistory => (Global, "global.undo_history", GLOBAL_UNDO_HISTORY, ["Alt+Z"]),
    GlobalCommitColumn => (Global, "global.commit_column", GLOBAL_COMMIT_COLUMN, ["Alt+C"]),
    GlobalUndoOffer => (Global, "global.undo_offer", GLOBAL_UNDO_OFFER, ["Alt+Shift+Z"]),
    GlobalContextMenu => (Global, "global.context_menu", GLOBAL_CONTEXT_MENU, [".", "Menu", "Shift+F10"]),
    // ---- git_prefix: the key after Ctrl+G ----
    GitCompose => (GitPrefix, "git_prefix.compose_commit", GIT_PREFIX_COMPOSE_COMMIT, ["c"]),
    GitCommitNow => (GitPrefix, "git_prefix.commit_now", GIT_PREFIX_COMMIT_NOW, ["Enter"]),
    GitAiCompose => (GitPrefix, "git_prefix.ai_compose", GIT_PREFIX_AI_COMPOSE, ["Shift+C"], @Ai),
    GitAiExplain => (GitPrefix, "git_prefix.ai_explain", GIT_PREFIX_AI_EXPLAIN, ["w"], @Ai),
    GitStashMessage => (GitPrefix, "git_prefix.stash_message", GIT_PREFIX_STASH_MESSAGE, ["m"]),
    GitAiCommitMessage => (GitPrefix, "git_prefix.ai_commit_message", GIT_PREFIX_AI_COMMIT_MESSAGE, ["i"], @Ai),
    GitStageAll => (GitPrefix, "git_prefix.stage_all", GIT_PREFIX_STAGE_ALL, ["a"]),
    GitDiscardAll => (GitPrefix, "git_prefix.discard_all", GIT_PREFIX_DISCARD_ALL, ["x"]),
    GitStageFile => (GitPrefix, "git_prefix.stage_file", GIT_PREFIX_STAGE_FILE, ["s"]),
    GitUnstageFile => (GitPrefix, "git_prefix.unstage_file", GIT_PREFIX_UNSTAGE_FILE, ["u"]),
    GitPull => (GitPrefix, "git_prefix.pull", GIT_PREFIX_PULL, ["p"]),
    GitPush => (GitPrefix, "git_prefix.push", GIT_PREFIX_PUSH, ["Shift+P"]),
    GitOpenLinkedPr => (GitPrefix, "git_prefix.open_linked_pr", GIT_PREFIX_OPEN_LINKED_PR, ["g"], @Hosting),
    GitPushTags => (GitPrefix, "git_prefix.push_tags", GIT_PREFIX_PUSH_TAGS, ["Shift+U"]),
    GitFetchAll => (GitPrefix, "git_prefix.fetch_all", GIT_PREFIX_FETCH_ALL, ["f"]),
    GitNewTag => (GitPrefix, "git_prefix.new_tag", GIT_PREFIX_NEW_TAG, ["t"]),
    GitNewBranch => (GitPrefix, "git_prefix.new_branch", GIT_PREFIX_NEW_BRANCH, ["b"]),
    GitStashPush => (GitPrefix, "git_prefix.stash_push", GIT_PREFIX_STASH_PUSH, ["Shift+T"]),
    GitCherryPick => (GitPrefix, "git_prefix.cherry_pick", GIT_PREFIX_CHERRY_PICK, ["y"]),
    GitRevert => (GitPrefix, "git_prefix.revert", GIT_PREFIX_REVERT, ["v"]),
    GitEditMessage => (GitPrefix, "git_prefix.edit_message", GIT_PREFIX_EDIT_MESSAGE, ["e"]),
    GitAmend => (GitPrefix, "git_prefix.amend", GIT_PREFIX_AMEND, ["Shift+E"]),
    GitEditAuthor => (GitPrefix, "git_prefix.edit_author", GIT_PREFIX_EDIT_AUTHOR, ["Shift+A"]),
    GitResetMixed => (GitPrefix, "git_prefix.reset_mixed", GIT_PREFIX_RESET_MIXED, ["Shift+R"]),
    GitRebaseOnto => (GitPrefix, "git_prefix.rebase_onto", GIT_PREFIX_REBASE_ONTO, ["r"]),
    GitStashPop => (GitPrefix, "git_prefix.stash_pop", GIT_PREFIX_STASH_POP, ["o"]),
    GitDeleteRef => (GitPrefix, "git_prefix.delete_ref", GIT_PREFIX_DELETE_REF, ["d"]),
    GitLfsMenu => (GitPrefix, "git_prefix.lfs_menu", GIT_PREFIX_LFS_MENU, ["l"], @Lfs),
    GitFlowMenu => (GitPrefix, "git_prefix.flow_menu", GIT_PREFIX_FLOW_MENU, ["Shift+W"]),
    GitLfsFetchAll => (GitPrefix, "git_prefix.lfs_fetch_all", GIT_PREFIX_LFS_FETCH_ALL, ["Shift+F"], @Lfs),
    GitLfsCheckout => (GitPrefix, "git_prefix.lfs_checkout", GIT_PREFIX_LFS_CHECKOUT, ["Shift+K"], @Lfs),
    GitLfsPull => (GitPrefix, "git_prefix.lfs_pull", GIT_PREFIX_LFS_PULL, ["Shift+L"], @Lfs),
    GitLfsPushAll => (GitPrefix, "git_prefix.lfs_push_all", GIT_PREFIX_LFS_PUSH_ALL, ["Shift+J"], @Lfs),
    GitLfsPrune => (GitPrefix, "git_prefix.lfs_prune", GIT_PREFIX_LFS_PRUNE, ["Shift+V"], @Lfs),
    GitRewriteWorkbench => (GitPrefix, "git_prefix.rewrite_workbench", GIT_PREFIX_REWRITE_WORKBENCH, ["Shift+H"], @HistoryRewrite),
    // ---- nav_prefix: the key after Ctrl+N ----
    NavCollapseLeft => (NavPrefix, "nav_prefix.collapse_refs", NAV_PREFIX_COLLAPSE_REFS, ["Shift+H"]),
    NavCollapseRight => (NavPrefix, "nav_prefix.collapse_status", NAV_PREFIX_COLLAPSE_STATUS, ["Shift+L"]),
    NavOpenSettings => (NavPrefix, "nav_prefix.settings", NAV_PREFIX_SETTINGS, [","]),
    NavOpenAbout => (NavPrefix, "nav_prefix.about", NAV_PREFIX_ABOUT, ["a"]),
    NavOpenActivityLog => (NavPrefix, "nav_prefix.activity_log", NAV_PREFIX_ACTIVITY_LOG, ["Shift+A"]),
    NavOpenGlobalLog => (NavPrefix, "nav_prefix.global_log", NAV_PREFIX_GLOBAL_LOG, ["Shift+G"]),
    NavShowJobs => (NavPrefix, "nav_prefix.jobs", NAV_PREFIX_JOBS, ["Shift+J"]),
    NavShowUpdates => (NavPrefix, "nav_prefix.updates", NAV_PREFIX_UPDATES, ["u"]),
    NavToggleHelp => (NavPrefix, "nav_prefix.help", NAV_PREFIX_HELP, ["?"]),
    NavSelectTab => (NavPrefix, "nav_prefix.select_tab", NAV_PREFIX_SELECT_TAB, ["Tab"]),
    NavChangeTheme => (NavPrefix, "nav_prefix.change_theme", NAV_PREFIX_CHANGE_THEME, ["t"]),
    NavChangeLanguage => (NavPrefix, "nav_prefix.change_language", NAV_PREFIX_CHANGE_LANGUAGE, ["l"]),
    NavChangeGraphColorTheme => (NavPrefix, "nav_prefix.change_graph_color_theme", NAV_PREFIX_CHANGE_GRAPH_COLOR_THEME, ["Shift+T"], @Mermaid),
    NavToggleGraphRowTints => (NavPrefix, "nav_prefix.toggle_graph_row_tints", NAV_PREFIX_TOGGLE_GRAPH_ROW_TINTS, ["Shift+Y"]),
    NavGrowSection => (NavPrefix, "nav_prefix.grow_section", NAV_PREFIX_GROW_SECTION, ["+", "="]),
    NavShrinkSection => (NavPrefix, "nav_prefix.shrink_section", NAV_PREFIX_SHRINK_SECTION, ["-"]),
    NavGrowMessage => (NavPrefix, "nav_prefix.grow_message", NAV_PREFIX_GROW_MESSAGE, ["]"]),
    NavShrinkMessage => (NavPrefix, "nav_prefix.shrink_message", NAV_PREFIX_SHRINK_MESSAGE, ["["]),
    NavRefsWider => (NavPrefix, "nav_prefix.refs_wider", NAV_PREFIX_REFS_WIDER, ["Right"]),
    NavRefsNarrower => (NavPrefix, "nav_prefix.refs_narrower", NAV_PREFIX_REFS_NARROWER, ["Left"]),
    NavStatusWider => (NavPrefix, "nav_prefix.status_wider", NAV_PREFIX_STATUS_WIDER, ["Shift+Left"]),
    NavStatusNarrower => (NavPrefix, "nav_prefix.status_narrower", NAV_PREFIX_STATUS_NARROWER, ["Shift+Right"]),
    NavAgentShow => (NavPrefix, "nav_prefix.agent_show", NAV_PREFIX_AGENT_SHOW, ["Enter"]),
    NavAgentBack => (NavPrefix, "nav_prefix.agent_back", NAV_PREFIX_AGENT_BACK, ["Backspace"]),
    NavArrangeDialog => (NavPrefix, "nav_prefix.arrange_dialog", NAV_PREFIX_ARRANGE_DIALOG, ["m"]),
    // ---- refs pane ----
    RefsFilter => (Refs, "refs.filter", REFS_FILTER, ["/"]),
    RefsSolo => (Refs, "refs.solo", REFS_SOLO, ["o"]),
    RefsClearSolo => (Refs, "refs.clear_solo", REFS_CLEAR_SOLO, ["Shift+O"]),
    RefsToggleHierarchy => (Refs, "refs.toggle_hierarchy", REFS_TOGGLE_HIERARCHY, ["y"]),
    RefsShowAll => (Refs, "refs.show_all", REFS_SHOW_ALL, ["Shift+H"]),
    RefsReveal => (Refs, "refs.reveal", REFS_REVEAL, ["g"]),
    RefsMarkDrop => (Refs, "refs.mark_drop", REFS_MARK_DROP, ["m"]),
    RefsDropMarked => (Refs, "refs.drop_marked", REFS_DROP_MARKED, ["Shift+M"]),
    RefsExpandAll => (Refs, "refs.expand_all", TREE_EXPAND_ALL, ["="]),
    RefsCollapseAll => (Refs, "refs.collapse_all", TREE_COLLAPSE_ALL, ["-"]),
    RefsExpandBelow => (Refs, "refs.expand_below", TREE_EXPAND_BELOW, ["*", "Shift+Right"]),
    RefsCollapseBelow => (Refs, "refs.collapse_below", TREE_COLLAPSE_BELOW, ["_", "Shift+Left"]),
    // ---- log pane ----
    LogToggleFileMode => (Log, "log.toggle_file_mode", LOG_TOGGLE_FILE_MODE, ["m"]),
    LogViewAllFiles => (Log, "log.view_all_files", LOG_VIEW_ALL_FILES, ["a"]),
    LogCompareSelection => (Log, "log.compare_selection", KEYMAP_COMPARE_SELECTION, ["v"]),
    LogCompareSwap => (Log, "log.compare_swap", KEYMAP_SWAP, ["x"]),
    LogCompareNext => (Log, "log.compare_next", KEYMAP_NEXT_CARD, ["n"]),
    LogPickColumn => (Log, "log.pick_column", LOG_PICK_COLUMN, ["Shift+C"]),
    LogScrollLeft => (Log, "log.scroll_left", LOG_SCROLL_LEFT, ["Shift+Left"]),
    LogScrollRight => (Log, "log.scroll_right", LOG_SCROLL_RIGHT, ["Shift+Right"]),
    LogCopyMermaid => (Log, "log.copy_mermaid", KEYMAP_COPY_MERMAID, ["Shift+Y"], @Mermaid),
    // ---- the Log's column header, while a column is picked ----
    LogColumnsNext => (LogColumns, "log_columns.next", LOG_COLUMNS_NEXT, ["Tab"]),
    LogColumnsPrev => (LogColumns, "log_columns.prev", LOG_COLUMNS_PREV, ["Shift+BackTab"]),
    LogColumnsMoveLeft => (LogColumns, "log_columns.move_left", LOG_COLUMNS_MOVE_LEFT, ["<"]),
    LogColumnsMoveRight => (LogColumns, "log_columns.move_right", LOG_COLUMNS_MOVE_RIGHT, [">"]),
    LogColumnsWider => (LogColumns, "log_columns.wider", LOG_COLUMNS_WIDER, ["]"]),
    LogColumnsNarrower => (LogColumns, "log_columns.narrower", LOG_COLUMNS_NARROWER, ["["]),
    LogColumnsResetWidth => (LogColumns, "log_columns.reset_width", LOG_COLUMNS_RESET_WIDTH, ["0"]),
    LogColumnsFit => (LogColumns, "log_columns.fit", LOG_COLUMNS_FIT, ["="]),
    LogColumnsFitAll => (LogColumns, "log_columns.fit_all", LOG_COLUMNS_FIT_ALL, ["+"]),
    LogColumnsLayout => (LogColumns, "log_columns.layout", LOG_COLUMNS_LAYOUT, ["s"]),
    // ---- log pane, inside a commit's changed files ----
    LogFilesCollapseAll => (LogFiles, "log_files.collapse_all", LOG_FILES_COLLAPSE_ALL, ["z"]),
    LogFilesFilter => (LogFiles, "log_files.filter", LOG_FILES_FILTER, ["/"]),
    LogFilesExpandAll => (LogFiles, "log_files.expand_all", TREE_EXPAND_ALL, ["="]),
    LogFilesCollapseAllFolders => (LogFiles, "log_files.collapse_all_folders", TREE_COLLAPSE_ALL, ["-"]),
    LogFilesExpandBelow => (LogFiles, "log_files.expand_below", TREE_EXPAND_BELOW, ["*", "Shift+Right"]),
    LogFilesCollapseBelow => (LogFiles, "log_files.collapse_below", TREE_COLLAPSE_BELOW, ["_", "Shift+Left"]),
    // ---- status pane ----
    StatusCompose => (Status, "status.compose", STATUS_COMPOSE, ["c"]),
    StatusToggleFileMode => (Status, "status.toggle_file_mode", STATUS_TOGGLE_FILE_MODE, ["m"]),
    StatusViewAllFiles => (Status, "status.view_all_files", STATUS_VIEW_ALL_FILES, ["a"]),
    StatusAmend => (Status, "status.amend", STATUS_AMEND, ["Shift+A"]),
    StatusSkipHooks => (Status, "status.skip_hooks", STATUS_SKIP_HOOKS, ["Shift+H"]),
    StatusCollapseAll => (Status, "status.collapse_all", STATUS_COLLAPSE_ALL, ["z"]),
    StatusStageToggle => (Status, "status.stage_toggle", STATUS_STAGE_TOGGLE, ["Ctrl+Space"]),
    StatusOpenDiff => (Status, "status.open_diff", STATUS_OPEN_DIFF, ["d"]),
    StatusExpandAll => (Status, "status.expand_all", TREE_EXPAND_ALL, ["="]),
    StatusCollapseAllFolders => (Status, "status.collapse_all_folders", TREE_COLLAPSE_ALL, ["-"]),
    StatusExpandBelow => (Status, "status.expand_below", TREE_EXPAND_BELOW, ["*", "Shift+Right"]),
    StatusCollapseBelow => (Status, "status.collapse_below", TREE_COLLAPSE_BELOW, ["_", "Shift+Left"]),
    // ---- home tab ----
    HomeFavorite => (Home, "home.favorite", HOME_FAVORITE, ["f", "*"]),
    HomeCloseRepo => (Home, "home.close_repo", HOME_CLOSE_REPO, ["x", "Delete"]),
    HomeCopyPath => (Home, "home.copy_path", HOME_COPY_PATH, ["y"]),
    HomeOpenIn => (Home, "home.open_in", HOME_OPEN_IN, ["o"]),
    HomeBrowse => (Home, "home.browse", HOME_BROWSE, ["b"]),
    HomeClone => (Home, "home.clone", HOME_CLONE, ["c"]),
    HomeInit => (Home, "home.init", HOME_INIT, ["i"]),
    HomeCollapseAll => (Home, "home.collapse_all", HOME_COLLAPSE_ALL, ["-"]),
    HomeExpandAll => (Home, "home.expand_all", HOME_EXPAND_ALL, ["+", "="]),
    HomeSearch => (Home, "home.search", HOME_SEARCH, ["/"]),
    HomeWsNew => (Home, "home.workspace_new", HOME_WORKSPACE_NEW, ["n"], @Workspaces),
    HomeWsEdit => (Home, "home.workspace_edit", HOME_WORKSPACE_EDIT, ["e"], @Workspaces),
    HomeWsFromTabs => (Home, "home.workspace_from_tabs", HOME_WORKSPACE_FROM_TABS, ["a"], @Workspaces),
    HomeWsFetch => (Home, "home.workspace_fetch", HOME_WORKSPACE_FETCH, ["Shift+F"], @Workspaces),
    HomeWsPull => (Home, "home.workspace_pull", HOME_WORKSPACE_PULL, ["Shift+P"], @Workspaces),
    HomeWsCloneMissing => (Home, "home.workspace_clone_missing", HOME_WORKSPACE_CLONE_MISSING, ["m"], @Workspaces),
    HomeWsLocate => (Home, "home.workspace_locate", HOME_WORKSPACE_LOCATE, ["l"], @Workspaces),
    // ---- refs pane: pull requests, issues and remotes ----
    RefsHostingFilter => (Refs, "refs.hosting_filter", REFS_HOSTING_FILTER, ["v"], @Hosting),
    RefsHostingEdit => (Refs, "refs.hosting_edit", REFS_HOSTING_EDIT, ["e"], @Hosting),
    RefsNewPullRequest => (Refs, "refs.new_pull_request", REFS_NEW_PULL_REQUEST, ["n"], @Hosting),
    RefsAddRemote => (Refs, "refs.add_remote", REFS_ADD_REMOTE, ["+"]),
    RefsRemotes => (Refs, "refs.remotes", REFS_REMOTES, ["u"]),
    // ---- diff view ----
    DiffClose => (Diff, "diff.close", DIFF_CLOSE, ["q"]),
    DiffEdit => (Diff, "diff.edit", DIFF_EDIT, ["e"]),
    DiffLfsRaw => (Diff, "diff.lfs_raw", DIFF_LFS_RAW, ["r"]),
    DiffCopy => (Diff, "diff.copy", DIFF_COPY, ["y", "Ctrl+C"]),
    DiffStageHunk => (Diff, "diff.stage_hunk", DIFF_STAGE_HUNK, ["Space"]),
    DiffStageLines => (Diff, "diff.stage_lines", DIFF_STAGE_LINES, ["Enter"]),
    DiffDiscardHunk => (Diff, "diff.discard_hunk", DIFF_DISCARD_HUNK, ["x"]),
    DiffDiscardLines => (Diff, "diff.discard_lines", DIFF_DISCARD_LINES, ["Shift+X"]),
    DiffSelectLines => (Diff, "diff.select_lines", DIFF_SELECT_LINES, ["v"]),
    DiffNextHunk => (Diff, "diff.next_hunk", DIFF_NEXT_HUNK, ["n"]),
    DiffPrevHunk => (Diff, "diff.prev_hunk", DIFF_PREV_HUNK, ["p"]),
    DiffToggleLayout => (Diff, "diff.toggle_layout", DIFF_TOGGLE_LAYOUT, ["m"]),
    DiffToggleHunksOnly => (Diff, "diff.toggle_hunks_only", DIFF_TOGGLE_HUNKS_ONLY, ["Shift+H"]),
    DiffToggleMinimap => (Diff, "diff.toggle_minimap", DIFF_TOGGLE_MINIMAP, ["Shift+M"]),
    DiffChooseEncoding => (Diff, "diff.choose_encoding", DIFF_CHOOSE_ENCODING, ["Shift+C"]),
    DiffToggleWrap => (Diff, "diff.toggle_wrap", DIFF_TOGGLE_WRAP, ["w"]),
    DiffToggleWhitespace => (Diff, "diff.toggle_whitespace", DIFF_TOGGLE_WHITESPACE, ["s"]),
    DiffMergeEditor => (Diff, "diff.merge_editor", DIFF_MERGE_EDITOR, ["Shift+E"]),
    // ---- file insight: File View, Blame, File History ----
    FileInsightClose => (FileInsight, "file_insight.close", FILE_INSIGHT_CLOSE, ["q"]),
    FileInsightEdit => (FileInsight, "file_insight.edit", FILE_INSIGHT_EDIT, ["e"]),
    FileInsightCopy => (FileInsight, "file_insight.copy", FILE_INSIGHT_COPY, ["y", "Ctrl+C"]),
    FileInsightLastRow => (FileInsight, "file_insight.last_row", FILE_INSIGHT_LAST_ROW, ["Shift+G"]),
    FileInsightOpenCommit => (FileInsight, "file_insight.open_commit", FILE_INSIGHT_OPEN_COMMIT, ["Enter"]),
    // ---- file history: the split view (list on the left, diff on the right) ----
    HistoryShowInLog => (History, "history.show_in_log", HISTORY_SHOW_IN_LOG, ["Enter"]),
    HistoryCopySha => (History, "history.copy_sha", HISTORY_COPY_SHA, ["y"]),
    HistoryCopySummary => (History, "history.copy_summary", HISTORY_COPY_SUMMARY, ["Shift+Y"], @Mermaid),
    HistoryShowDiff => (History, "history.show_diff", HISTORY_SHOW_DIFF, ["d", "Alt+1"]),
    HistoryOpenFile => (History, "history.open_file", HISTORY_OPEN_FILE, ["o", "Alt+2"]),
    HistoryBlame => (History, "history.blame", HISTORY_BLAME, ["b", "Alt+3"]),
    HistoryCompareWorkingTree => (History, "history.compare_working_tree", HISTORY_COMPARE_WORKING_TREE, ["c"]),
    HistoryRestore => (History, "history.restore", HISTORY_RESTORE, ["Shift+R"]),
    HistoryRangeUp => (History, "history.range_up", HISTORY_RANGE_UP, ["Shift+Up"]),
    HistoryRangeDown => (History, "history.range_down", HISTORY_RANGE_DOWN, ["Shift+Down"]),
    HistorySwitchFocus => (History, "history.switch_focus", HISTORY_SWITCH_FOCUS, ["Tab"]),
    HistoryWidenList => (History, "history.widen_list", HISTORY_WIDEN_LIST, ["Alt+Right"]),
    HistoryNarrowList => (History, "history.narrow_list", HISTORY_NARROW_LIST, ["Alt+Left"]),
    HistoryResetSplit => (History, "history.reset_split", HISTORY_RESET_SPLIT, ["Alt+0"]),
    CommitColumnScrollLeft => (CommitColumn, "commit_column.scroll_left", COMMIT_COLUMN_SCROLL_LEFT, ["Shift+Left"]),
    CommitColumnScrollRight => (CommitColumn, "commit_column.scroll_right", COMMIT_COLUMN_SCROLL_RIGHT, ["Shift+Right"]),
    CommitColumnNextPart => (CommitColumn, "commit_column.next_part", COMMIT_COLUMN_NEXT_PART, ["Tab"]),
    CommitColumnMoveLeft => (CommitColumn, "commit_column.move_left", COMMIT_COLUMN_MOVE_LEFT, ["<"]),
    CommitColumnMoveRight => (CommitColumn, "commit_column.move_right", COMMIT_COLUMN_MOVE_RIGHT, [">"]),
    CommitColumnWiderPart => (CommitColumn, "commit_column.wider_part", COMMIT_COLUMN_WIDER_PART, ["]"]),
    CommitColumnNarrowerPart => (CommitColumn, "commit_column.narrower_part", COMMIT_COLUMN_NARROWER_PART, ["["]),
    CommitColumnResetWidths => (CommitColumn, "commit_column.reset_widths", COMMIT_COLUMN_RESET_WIDTHS, ["0"]),
    CommitColumnFit => (CommitColumn, "commit_column.fit", COMMIT_COLUMN_FIT, ["="]),
    CommitColumnFitAll => (CommitColumn, "commit_column.fit_all", COMMIT_COLUMN_FIT_ALL, ["+"]),
    FileInsightBlameCard => (FileInsight, "file_insight.blame_card", FILE_INSIGHT_BLAME_CARD, ["i"]),
    FileInsightBlameNext => (FileInsight, "file_insight.blame_next", FILE_INSIGHT_BLAME_NEXT, ["n"]),
    FileInsightBlamePrev => (FileInsight, "file_insight.blame_prev", FILE_INSIGHT_BLAME_PREV, ["p"]),
    FileInsightBlameNextSame => (FileInsight, "file_insight.blame_next_same", FILE_INSIGHT_BLAME_NEXT_SAME, ["Shift+N"]),
    FileInsightBlamePrevSame => (FileInsight, "file_insight.blame_prev_same", FILE_INSIGHT_BLAME_PREV_SAME, ["Shift+P"]),
    FileInsightBlameGoto => (FileInsight, "file_insight.blame_goto", FILE_INSIGHT_BLAME_GOTO, ["g"]),
    FileInsightBlameCopySha => (FileInsight, "file_insight.blame_copy_sha", FILE_INSIGHT_BLAME_COPY_SHA, ["Shift+Y"], @Mermaid),
    FileInsightBlameDiff => (FileInsight, "file_insight.blame_diff", FILE_INSIGHT_BLAME_DIFF, ["d"]),
    FileInsightBlameParent => (FileInsight, "file_insight.blame_parent", FILE_INSIGHT_BLAME_PARENT, ["b"]),
    FileInsightBlameHistory => (FileInsight, "file_insight.blame_history", FILE_INSIGHT_BLAME_HISTORY, ["Shift+H"]),
    FileInsightBlameWider => (FileInsight, "file_insight.blame_wider", FILE_INSIGHT_BLAME_WIDER, [">"]),
    FileInsightBlameNarrower => (FileInsight, "file_insight.blame_narrower", FILE_INSIGHT_BLAME_NARROWER, ["<"]),
    // ---- image view (File View on a picture) ----
    ImageZoomIn => (Image, "image.zoom_in", IMAGE_ZOOM_IN, ["+", "="], @ImageView),
    ImageZoomOut => (Image, "image.zoom_out", IMAGE_ZOOM_OUT, ["-"], @ImageView),
    ImageFit => (Image, "image.fit", IMAGE_FIT, ["0"], @ImageView),
    ImageActualSize => (Image, "image.actual_size", IMAGE_ACTUAL_SIZE, ["1"], @ImageView),
    ImageFlipSide => (Image, "image.flip_side", IMAGE_FLIP_SIDE, ["Tab"], @ImageView),
    ImageWidenBefore => (Image, "image.widen_before", IMAGE_WIDEN_BEFORE, ["Alt+Right"], @ImageView),
    ImageWidenAfter => (Image, "image.widen_after", IMAGE_WIDEN_AFTER, ["Alt+Left"], @ImageView),
    ImageResetSplit => (Image, "image.reset_split", IMAGE_RESET_SPLIT, ["Alt+0"], @ImageView),
    ImageEditIn => (Image, "image.edit_in", IMAGE_EDIT_IN, ["Shift+O"], @ImageView),
    ImageEditDefault => (Image, "image.edit_default", IMAGE_EDIT_DEFAULT, ["o"], @ImageView),
    // ---- editor ----
    EditorSave => (Editor, "editor.save", EDITOR_SAVE, ["Ctrl+S"]),
    EditorUndo => (Editor, "editor.undo", EDITOR_UNDO, ["Ctrl+Z"]),
    EditorRedo => (Editor, "editor.redo", EDITOR_REDO, ["Ctrl+Y", "Ctrl+Shift+Z"]),
    EditorFind => (Editor, "editor.find", EDITOR_FIND, ["Ctrl+F"]),
    EditorFindNext => (Editor, "editor.find_next", EDITOR_FIND_NEXT, ["F3"]),
    EditorPaste => (Editor, "editor.paste", EDITOR_PASTE, ["Ctrl+V"]),
    EditorToggleMinimap => (Editor, "editor.toggle_minimap", EDITOR_TOGGLE_MINIMAP, ["Alt+M"]),
    EditorPromptSave => (EditorPrompt, "editor_prompt.save", EDITOR_PROMPT_SAVE, ["s", "Shift+S"]),
    EditorPromptDiscard => (EditorPrompt, "editor_prompt.discard", EDITOR_PROMPT_DISCARD, ["d", "Shift+D"]),
    EditorPromptCancel => (EditorPrompt, "editor_prompt.cancel", EDITOR_PROMPT_CANCEL, ["c", "Shift+C"]),
    // ---- merge conflict editor ----
    MergeNextConflict => (Merge, "merge.next_conflict", MERGE_NEXT_CONFLICT, ["F7", "Alt+Down"]),
    MergePrevConflict => (Merge, "merge.prev_conflict", MERGE_PREV_CONFLICT, ["Shift+F7", "Alt+Up"]),
    MergeNextFile => (Merge, "merge.next_file", MERGE_NEXT_FILE, ["Alt+PageDown"]),
    MergePrevFile => (Merge, "merge.prev_file", MERGE_PREV_FILE, ["Alt+PageUp"]),
    MergeNextPane => (Merge, "merge.next_pane", MERGE_NEXT_PANE, ["F6"]),
    MergePrevPane => (Merge, "merge.prev_pane", MERGE_PREV_PANE, ["Shift+F6"]),
    MergeTogglePanes => (Merge, "merge.toggle_panes", MERGE_TOGGLE_PANES, ["F9"]),
    MergeToggleBase => (Merge, "merge.toggle_base", MERGE_TOGGLE_BASE, ["Alt+D"]),
    MergeToggleFullFile => (Merge, "merge.toggle_full_file", MERGE_TOGGLE_FULL_FILE, ["Alt+F"]),
    MergeUndo => (Merge, "merge.undo", MERGE_UNDO, ["Ctrl+Z"]),
    MergeRedo => (Merge, "merge.redo", MERGE_REDO, ["Ctrl+Y", "Ctrl+Shift+Z"]),
    MergeWidenCurrent => (Merge, "merge.widen_current", MERGE_WIDEN_CURRENT, ["Alt+Right"]),
    MergeWidenIncoming => (Merge, "merge.widen_incoming", MERGE_WIDEN_INCOMING, ["Alt+Left"]),
    MergeResetPanes => (Merge, "merge.reset_panes", MERGE_RESET_PANES, ["Alt+0"]),
    MergeSave => (Merge, "merge.save", MERGE_SAVE, ["Alt+S"]),
    MergeSaveResolve => (Merge, "merge.save_resolve", MERGE_SAVE_RESOLVE, ["Ctrl+S"]),
    MergeAiSuggest => (Merge, "merge.ai_suggest", MERGE_AI_SUGGEST, ["Alt+I"], @Ai),
    MergeUnpick => (Merge, "merge.unpick", MERGE_UNPICK, ["Alt+U"]),
    MergeResetAll => (Merge, "merge.reset_all", MERGE_RESET_ALL, ["Alt+R"]),
    MergeFocusA => (Merge, "merge.focus_a", MERGE_FOCUS_A, ["Alt+1"]),
    MergeFocusB => (Merge, "merge.focus_b", MERGE_FOCUS_B, ["Alt+2"]),
    MergeFocusOutput => (Merge, "merge.focus_output", MERGE_FOCUS_OUTPUT, ["Alt+3"]),
    MergeFocusBase => (Merge, "merge.focus_base", MERGE_FOCUS_BASE, ["Alt+4"]),
    MergeToggleLine => (MergeSides, "merge_sides.toggle_line", MERGE_SIDES_TOGGLE_LINE, ["Space"]),
    MergeToggleChunk => (MergeSides, "merge_sides.toggle_chunk", MERGE_SIDES_TOGGLE_CHUNK, ["Enter"]),
    MergeTakeA => (MergeSides, "merge_sides.take_a", MERGE_SIDES_TAKE_A, ["a"]),
    MergeTakeB => (MergeSides, "merge_sides.take_b", MERGE_SIDES_TAKE_B, ["b"]),
    MergeTakeBoth => (MergeSides, "merge_sides.take_both", MERGE_SIDES_TAKE_BOTH, ["c"]),
    MergeTakeBothReversed => (MergeSides, "merge_sides.take_both_reversed", MERGE_SIDES_TAKE_BOTH_REVERSED, ["Shift+C"]),
    MergeClear => (MergeSides, "merge_sides.clear", MERGE_SIDES_CLEAR, ["x"]),
    MergeSideUnpick => (MergeSides, "merge_sides.unpick", MERGE_SIDES_UNPICK, ["u"]),
    MergeSideResetAll => (MergeSides, "merge_sides.reset_all", MERGE_SIDES_RESET_ALL, ["Shift+U"]),
    MergeSwapOrder => (MergeSides, "merge_sides.swap_order", MERGE_SIDES_SWAP_ORDER, ["s"]),
    MergeSideNextConflict => (MergeSides, "merge_sides.next_conflict", MERGE_SIDES_NEXT_CONFLICT, ["n"]),
    MergeSidePrevConflict => (MergeSides, "merge_sides.prev_conflict", MERGE_SIDES_PREV_CONFLICT, ["Shift+N"]),
    MergeTakeAllA => (MergeSides, "merge_sides.take_all_a", MERGE_SIDES_TAKE_ALL_A, ["Shift+A"]),
    MergeTakeAllB => (MergeSides, "merge_sides.take_all_b", MERGE_SIDES_TAKE_ALL_B, ["Shift+B"]),
    MergeSideToggleBase => (MergeSides, "merge_sides.toggle_base", MERGE_SIDES_TOGGLE_BASE, ["d"]),
    MergeScrollLeft => (MergeSides, "merge_sides.scroll_left", MERGE_SIDES_SCROLL_LEFT, ["Shift+Left", "Shift+H"]),
    MergeScrollRight => (MergeSides, "merge_sides.scroll_right", MERGE_SIDES_SCROLL_RIGHT, ["Shift+Right", "Shift+L"]),
    MergeSideToggleFullFile => (MergeSides, "merge_sides.toggle_full_file", MERGE_SIDES_TOGGLE_FULL_FILE, ["f"]),
    MergeCycleHeight => (MergeSides, "merge_sides.cycle_height", MERGE_SIDES_CYCLE_HEIGHT, ["z"]),
    MergeCardA => (MergeCard, "merge_card.take_a", MERGE_CARD_TAKE_A, ["a"]),
    MergeCardB => (MergeCard, "merge_card.take_b", MERGE_CARD_TAKE_B, ["b"]),
    MergeCardBase => (MergeCard, "merge_card.take_base", MERGE_CARD_TAKE_BASE, ["o"]),
    MergeCardKeep => (MergeCard, "merge_card.keep", MERGE_CARD_KEEP, ["k"]),
    MergeCardDelete => (MergeCard, "merge_card.delete", MERGE_CARD_DELETE, ["Shift+D"]),
    MergeCardExternal => (MergeCard, "merge_card.external_tool", MERGE_CARD_EXTERNAL_TOOL, ["m"]),
    // ---- interactive rebase / cherry-pick view ----
    RebasePick => (Rebase, "rebase.pick", REBASE_PICK, ["p"]),
    RebaseReword => (Rebase, "rebase.reword", REBASE_REWORD, ["r"]),
    RebaseSquash => (Rebase, "rebase.squash", REBASE_SQUASH, ["s"]),
    RebaseFixup => (Rebase, "rebase.fixup", REBASE_FIXUP, ["f"]),
    RebaseDrop => (Rebase, "rebase.drop", REBASE_DROP, ["d"]),
    RebaseEdit => (Rebase, "rebase.edit", REBASE_EDIT, ["e"]),
    RebaseAutosquash => (Rebase, "rebase.autosquash", REBASE_AUTOSQUASH, ["a"]),
    RebaseStart => (Rebase, "rebase.start", REBASE_START, ["Ctrl+S", "Ctrl+Enter"]),
    RebaseReset => (Rebase, "rebase.reset", REBASE_RESET, ["Ctrl+R"]),
    RebaseSplit => (Rebase, "rebase.split", REBASE_SPLIT, ["t"]),
    RebaseMoveUp => (Rebase, "rebase.move_up", REBASE_MOVE_UP, ["Ctrl+Up", "Alt+Up"]),
    RebaseMoveDown => (Rebase, "rebase.move_down", REBASE_MOVE_DOWN, ["Ctrl+Down", "Alt+Down"]),
    // ---- forms and popups ----
    ComposeApply => (Compose, "compose.apply", COMPOSE_APPLY, ["Ctrl+S", "Ctrl+Enter", "Alt+Enter"]),
    SplitLineMode => (Split, "split.line_mode", SPLIT_LINE_MODE, ["v"]),
    SplitMoveUp => (Split, "split.move_up", SPLIT_MOVE_UP, ["Alt+Up", "Shift+K"]),
    SplitMoveDown => (Split, "split.move_down", SPLIT_MOVE_DOWN, ["Alt+Down", "Shift+J"]),
    SplitReset => (Split, "split.reset", SPLIT_RESET, ["Shift+R"]),
    RewritePreview => (Rewrite, "rewrite.preview", REWRITE_PREVIEW, ["F5"], @HistoryRewrite),
    RewriteApply => (Rewrite, "rewrite.apply", REWRITE_APPLY, ["Ctrl+Enter", "Ctrl+S"], @HistoryRewrite),
    RewriteCancel => (Rewrite, "rewrite.cancel", REWRITE_CANCEL, ["Ctrl+X"], @HistoryRewrite),
    RewriteAddRule => (Rewrite, "rewrite.add_rule", REWRITE_ADD_RULE, ["a", "Insert"], @HistoryRewrite),
    RewriteToggleRule => (Rewrite, "rewrite.toggle_rule", REWRITE_TOGGLE_RULE, ["Space"], @HistoryRewrite),
    RewriteMoveUp => (Rewrite, "rewrite.move_up", REWRITE_MOVE_UP, ["Alt+Up"], @HistoryRewrite),
    RewriteMoveDown => (Rewrite, "rewrite.move_down", REWRITE_MOVE_DOWN, ["Alt+Down"], @HistoryRewrite),
    RewriteDeleteRule => (Rewrite, "rewrite.delete_rule", REWRITE_DELETE_RULE, ["d", "Delete"], @HistoryRewrite),
    RewriteRestoreRule => (Rewrite, "rewrite.restore_rule", REWRITE_RESTORE_RULE, ["Ctrl+Z"], @HistoryRewrite),
    RewriteDuplicateRule => (Rewrite, "rewrite.duplicate_rule", REWRITE_DUPLICATE_RULE, ["Ctrl+D"], @HistoryRewrite),
    RewriteNextTab => (Rewrite, "rewrite.next_tab", REWRITE_NEXT_TAB, ["Alt+Right"], @HistoryRewrite),
    RewritePrevTab => (Rewrite, "rewrite.prev_tab", REWRITE_PREV_TAB, ["Alt+Left"], @HistoryRewrite),
    RewriteSaveShared => (Rewrite, "rewrite.save_shared", REWRITE_SAVE_SHARED, ["Alt+S"], @HistoryRewrite),
    RewriteSaveMine => (Rewrite, "rewrite.save_mine", REWRITE_SAVE_MINE, ["Alt+M"], @HistoryRewrite),
    RewriteLoad => (Rewrite, "rewrite.load", REWRITE_LOAD, ["Alt+L"], @HistoryRewrite),
    RewriteName => (Rewrite, "rewrite.name", REWRITE_NAME, ["F2"], @HistoryRewrite),
    RewriteProtectedAll => (Rewrite, "rewrite.protected_all", REWRITE_PROTECTED_ALL, ["Alt+A"], @HistoryRewrite),
    RewriteProtectedNone => (Rewrite, "rewrite.protected_none", REWRITE_PROTECTED_NONE, ["Alt+U"], @HistoryRewrite),
    RewriteCheckTools => (Rewrite, "rewrite.check_tools", REWRITE_CHECK_TOOLS, ["F6"], @HistoryRewrite),
    RewriteAnalyze => (Rewrite, "rewrite.analyze", REWRITE_ANALYZE, ["F7"], @HistoryRewrite),
    RewriteHostHowto => (Rewrite, "rewrite.host_howto", REWRITE_HOST_HOWTO, ["Alt+H"], @HistoryRewrite),
    RewriteAiRules => (Rewrite, "rewrite.ai_rules", REWRITE_AI_RULES, ["Alt+I"], @Ai),
    RewordApply => (Reword, "reword.apply", REWORD_APPLY, ["Ctrl+Enter", "Ctrl+S"]),
    RewordDeeper => (Reword, "reword.deeper", REWORD_DEEPER, ["Alt+H"], @HistoryRewrite),
    ComposerCommit => (Composer, "composer.commit", COMPOSER_COMMIT, ["Ctrl+Enter", "Alt+Enter"]),
    ComposerInsertIssueRef => (Composer, "composer.insert_issue_ref", COMPOSER_INSERT_ISSUE_REF, ["Alt+R"], @Hosting),
    MdEditorPreview => (MdEditor, "md_editor.preview", MD_EDITOR_PREVIEW, ["Alt+P"], @Hosting),
    MdEditorSplit => (MdEditor, "md_editor.split", MD_EDITOR_SPLIT, ["Alt+L"], @Hosting),
    MdEditorSubmit => (MdEditor, "md_editor.submit", MD_EDITOR_SUBMIT, ["Ctrl+Enter", "Ctrl+S", "Alt+Enter"], @Hosting),
    MdEditorAi => (MdEditor, "md_editor.ai", MD_EDITOR_AI, ["Alt+E"], @Ai),
    DialogBrowse => (Dialog, "dialog.browse", DIALOG_BROWSE, ["Ctrl+B"]),
    DialogSources => (Dialog, "dialog.sources", DIALOG_SOURCES, ["Alt+S"]),
    DialogSubmit => (Dialog, "dialog.submit", DIALOG_SUBMIT, ["Ctrl+S"]),
    DialogWeb => (Dialog, "dialog.web", DIALOG_WEB, ["Alt+W"]),
    FolderPickerChoose => (FolderPicker, "folder_picker.choose", FOLDER_PICKER_CHOOSE, ["Ctrl+S", "Ctrl+Enter"]),
    FolderPickerNewFolder => (FolderPicker, "folder_picker.new_folder", FOLDER_PICKER_NEW_FOLDER, ["Alt+N"]),
    FolderPickerFilter => (FolderPicker, "folder_picker.filter", FOLDER_PICKER_FILTER, ["Ctrl+F", "/"]),
    FolderPickerSiblings => (FolderPicker, "folder_picker.toggle_siblings", FOLDER_PICKER_TOGGLE_SIBLINGS, ["Alt+S"]),
    FolderPickerHidden => (FolderPicker, "folder_picker.toggle_hidden", FOLDER_PICKER_TOGGLE_HIDDEN, ["Alt+H"]),
    PreferencesSearch => (Preferences, "preferences.search", PREFERENCES_SEARCH, ["Ctrl+F", "/"]),
    PreferencesListAdd => (Preferences, "preferences.list_add", PREFERENCES_LIST_ADD, ["Insert"]),
    PreferencesListDuplicate => (Preferences, "preferences.list_duplicate", PREFERENCES_LIST_DUPLICATE, ["Ctrl+D"]),
    PreferencesListRemove => (Preferences, "preferences.list_remove", PREFERENCES_LIST_REMOVE, ["Delete"]),
    PreferencesListMoveUp => (Preferences, "preferences.list_move_up", PREFERENCES_LIST_MOVE_UP, ["Alt+Up"]),
    PreferencesListMoveDown => (Preferences, "preferences.list_move_down", PREFERENCES_LIST_MOVE_DOWN, ["Alt+Down"]),
    PreferencesPrevTab => (Preferences, "preferences.prev_tab", PREFERENCES_PREV_TAB, ["Alt+Left"]),
    PreferencesNextTab => (Preferences, "preferences.next_tab", PREFERENCES_NEXT_TAB, ["Alt+Right"]),
    AiPopupCopy => (AiPopup, "ai_popup.copy", AI_POPUP_COPY, ["c"], @Ai),
    ActivityLogScrollLeft => (ActivityLog, "activity_log.scroll_left", ACTIVITY_LOG_SCROLL_LEFT, ["Left"]),
    ActivityLogScrollRight => (ActivityLog, "activity_log.scroll_right", ACTIVITY_LOG_SCROLL_RIGHT, ["Right"]),
    IntegrationsAdd => (Integrations, "integrations.add", INTEGRATIONS_ADD, ["a"]),
    IntegrationsEdit => (Integrations, "integrations.edit", INTEGRATIONS_EDIT, ["e"]),
    IntegrationsOpenToken => (Integrations, "integrations.open_token", INTEGRATIONS_OPEN_TOKEN, ["o"]),
    IntegrationsRemove => (Integrations, "integrations.remove", INTEGRATIONS_REMOVE, ["d", "Delete"]),
    // ---- the palette itself ----
    PaletteConfigureShortcut => (Palette, "palette.configure_shortcut", PALETTE_CONFIGURE_SHORTCUT, ["Alt+K"]),
    PaletteArrange => (Palette, "palette.arrange", PALETTE_ARRANGE, ["Alt+G"]),
    // ---- prompt ----
    PromptAiStashMessage => (Prompt, "prompt.ai_stash_message", PROMPT_AI_STASH_MESSAGE, ["Ctrl+G"], @Ai),
}
