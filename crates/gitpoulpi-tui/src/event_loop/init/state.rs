//! The window's state struct, built by [`super::init`].

use super::super::*;

/// Everything one repository window owns, built by [`init`].
pub(in crate::event_loop) struct WindowState {
    pub(in crate::event_loop) open_t0: Instant,
    pub(in crate::event_loop) settings: gitpoulpi_config::Settings,
    pub(in crate::event_loop) graph_image: graph::GraphImage,
    /// The commit column's Graph picture, beside the Log's.
    pub(in crate::event_loop) commit_graph_image: graph::GraphImage,
    pub(in crate::event_loop) ui_lane_background: graph::LaneBackground,
    pub(in crate::event_loop) ui_graph_theme: crate::graph::LaneTheme,
    pub(in crate::event_loop) ui_tint_strength: gitpoulpi_config::GraphRowTintStrength,
    pub(in crate::event_loop) graph_show_stashes: bool,
    pub(in crate::event_loop) ref_connector: bool,
    pub(in crate::event_loop) avatar_placement: gitpoulpi_config::AvatarPlacement,
    pub(in crate::event_loop) avatar_capability: terminal_image::Capability,
    pub(in crate::event_loop) detail_avatar_cols: u16,
    pub(in crate::event_loop) detail_avatar_rows: u16,
    pub(in crate::event_loop) avatar_service: avatar::AvatarService,
    pub(in crate::event_loop) avatar_live: avatar::LiveSettings,
    pub(in crate::event_loop) zoom_check_size: Size,
    pub(in crate::event_loop) detail_avatar_image: avatar::DetailAvatarImage,
    pub(in crate::event_loop) log_avatar_images: avatar::LogAvatarImages,
    pub(in crate::event_loop) log_scroll_lines: i32,
    pub(in crate::event_loop) diff_config_path: Option<PathBuf>,
    pub(in crate::event_loop) auto_fetch_interval: Option<Duration>,
    /// `general.fetch_on_tab_open`, consumed by the first tick.
    pub(in crate::event_loop) fetch_on_open_pending: bool,
    pub(in crate::event_loop) hosting: HostingUi,
    pub(in crate::event_loop) hosting_prefs_was_open: bool,
    pub(in crate::event_loop) terminal: Terminal<crate::quiet_backend::QuietBackend<io::Stdout>>,
    pub(in crate::event_loop) repo_root: PathBuf,
    pub(in crate::event_loop) title: String,
    pub(in crate::event_loop) cwd: PathBuf,
    pub(in crate::event_loop) switch_to: Option<RepoRef>,
    pub(in crate::event_loop) close_window: bool,
    pub(in crate::event_loop) quit_how: Option<&'static str>,
    pub(in crate::event_loop) input_errors: u32,
    pub(in crate::event_loop) no_repository: bool,
    pub(in crate::event_loop) repo_dialogs: RepoDialogs,
    pub(in crate::event_loop) management: Option<repo_manager::RepoManager>,
    pub(in crate::event_loop) management_probe: Option<repo_probe::Probe>,
    pub(in crate::event_loop) show_management: Option<ManagementEntry>,
    pub(in crate::event_loop) management_panel: Option<(Rect, repo_manager::ManagerRender)>,
    pub(in crate::event_loop) repo_menu: Option<RepoMenuState>,
    pub(in crate::event_loop) repo_menu_rects: Vec<Rect>,
    pub(in crate::event_loop) repo_tabs: Vec<repo_switcher::RepoTab>,
    pub(in crate::event_loop) active_tab: usize,
    pub(in crate::event_loop) tab_hits: Vec<repo_switcher::TabHit>,
    pub(in crate::event_loop) link: DaemonLink,
    /// Press-and-hold on a scrollbar end arrow (`scrollbar_repeat.rs`).
    pub(in crate::event_loop) arrow_repeat: crate::scrollbar_repeat::ArrowRepeat,
    pub(in crate::event_loop) ui_viewer: crate::ui_viewer::UiViewer,
    pub(in crate::event_loop) link_failure: Option<String>,
    pub(in crate::event_loop) retry_link_requested: bool,
    pub(in crate::event_loop) log_layout: graph::LogLayoutCache,
    pub(in crate::event_loop) log_gen: u64,
    pub(in crate::event_loop) refs_gen: u64,
    pub(in crate::event_loop) read_hashes: daemon_link::ReadHashes,
    pub(in crate::event_loop) stashes_gen: u64,
    pub(in crate::event_loop) log_result:
        gitpoulpi_core::Result<gitpoulpi_core::domain::CommitGraph>,
    pub(in crate::event_loop) status_result:
        gitpoulpi_core::Result<gitpoulpi_core::domain::WorkingTree>,
    pub(in crate::event_loop) optimistic_backup: Option<gitpoulpi_core::domain::WorkingTree>,
    pub(in crate::event_loop) refs_result: gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Ref>>,
    pub(in crate::event_loop) stashes_result:
        gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Stash>>,
    pub(in crate::event_loop) worktrees_result:
        gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Worktree>>,
    pub(in crate::event_loop) submodules_result: Vec<gitpoulpi_core::domain::Submodule>,
    pub(in crate::event_loop) submodule_open: Option<std::path::PathBuf>,
    pub(in crate::event_loop) still_loading: bool,
    pub(in crate::event_loop) repo_open_started_at: Instant,
    pub(in crate::event_loop) conflicts: Vec<gitpoulpi_core::domain::ConflictEntry>,
    pub(in crate::event_loop) rebase_state: Option<gitpoulpi_core::domain::RebaseState>,
    /// The repo's pending auto-stash record (`autostash_banner`).
    pub(in crate::event_loop) pending_auto_stash: Option<gitpoulpi_session::PendingAutoStash>,
    pub(in crate::event_loop) lfs: lfs_ui::LfsUi,
    pub(in crate::event_loop) log_len: usize,
    pub(in crate::event_loop) lazy_log: graph::LazyLog,
    /// The status bar's `loaded / total` (see `graph::total`).
    pub(in crate::event_loop) commit_total: graph::CommitTotal,
    pub(in crate::event_loop) focus: Focus,
    pub(in crate::event_loop) right_panel: RightPanel,
    pub(in crate::event_loop) central_view: CentralView,
    pub(in crate::event_loop) diff_view_state: Option<DiffViewState>,
    pub(in crate::event_loop) diff_bar_area: Rect,
    pub(in crate::event_loop) diff_body_area: Rect,
    pub(in crate::event_loop) diff_rendered_len: usize,
    pub(in crate::event_loop) diff_hunk_starts: Vec<usize>,
    pub(in crate::event_loop) diff_longest_line: usize,
    pub(in crate::event_loop) diff_minimap: minimap::Minimap,
    pub(in crate::event_loop) diff_rows: std::rc::Rc<[diff_view::DiffRow]>,
    pub(in crate::event_loop) diff_content: diff_content::DiffContent,
    pub(in crate::event_loop) hovered_diff_row: Option<usize>,
    pub(in crate::event_loop) diff_line_drag: Option<usize>,
    pub(in crate::event_loop) text_drag: Option<text_select::TextDrag>,
    pub(in crate::event_loop) diff_render: Option<std::rc::Rc<diff_view::DiffRender>>,
    pub(in crate::event_loop) working_diff: Option<daemon_link::WorkingDiffOutcome>,
    pub(in crate::event_loop) log_state: TableState,
    pub(in crate::event_loop) log_multi: LogSelection,
    /// Files multi-selected in the right pane's lists (Ctrl/Shift+click, Space,
    /// Shift+Up/Down), next to each list's own cursor.
    pub(in crate::event_loop) file_sel: FileSelection,
    pub(in crate::event_loop) log_follow: bool,
    pub(in crate::event_loop) sidebar_states: SidebarStates,
    pub(in crate::event_loop) active_ref_section: SidebarSection,
    pub(in crate::event_loop) commit_file_state: ListState,
    pub(in crate::event_loop) commit_files_follow: bool,
    pub(in crate::event_loop) commit_files_visible_rows: usize,
    pub(in crate::event_loop) commit_file_focus: bool,
    pub(in crate::event_loop) pending_count: Option<u32>,
    /// Events read ahead while telling a paste from typing (`paste_burst`),
    /// handled before the next read.
    pub(in crate::event_loop) queued_events: std::collections::VecDeque<crossterm::event::Event>,
    pub(in crate::event_loop) status_list_mode: FileListMode,
    pub(in crate::event_loop) commit_list_mode: FileListMode,
    pub(in crate::event_loop) collapsed_status_folders: HashSet<PathBuf>,
    pub(in crate::event_loop) collapsed_status_sections: HashSet<StatusSection>,
    pub(in crate::event_loop) collapsed_commit_folders: HashSet<PathBuf>,
    pub(in crate::event_loop) commit_files_filter: String,
    pub(in crate::event_loop) commit_summary: String,
    pub(in crate::event_loop) commit_description: String,
    pub(in crate::event_loop) commit_amend: bool,
    /// The composer's Skip hooks option (`--no-verify`), per commit.
    pub(in crate::event_loop) commit_skip_hooks: bool,
    /// This repository's commit behavior (Settings > Commit and Commit
    /// template), reloaded when Settings closes and when git's config is read.
    pub(in crate::event_loop) commit_settings: crate::commit_settings::CommitSettings,
    /// Whether the settings were read at least once (the template's file is
    /// only known once the daemon reported `commit.template`).
    pub(in crate::event_loop) commit_settings_ready: bool,
    /// The `repo_config_version` the settings were last read at, and whether
    /// Settings was open last tick (its closing reloads them).
    pub(in crate::event_loop) commit_settings_version: u64,
    pub(in crate::event_loop) prefs_open_seen: bool,
    /// The empty composer was already offered the template (or the user
    /// emptied it): not pre-filled again until the next commit.
    pub(in crate::event_loop) commit_template_applied: bool,
    /// The `SQUASH_MSG` already offered to the composer, by modification time.
    pub(in crate::event_loop) commit_squash_seen: Option<std::time::SystemTime>,
    /// Proactive conflict detection: the last answer and the request in flight
    /// (`conflict_badge`), and how often Settings was closed (the key's part
    /// that makes a changed switch or Base branches ask again).
    pub(in crate::event_loop) conflict_badge: crate::conflict_badge::ConflictBadge,
    pub(in crate::event_loop) conflict_settings_gen: u64,
    pub(in crate::event_loop) hovered_status_row: Option<(StatusSection, usize)>,
    pub(in crate::event_loop) composer_focus: Option<ComposerField>,
    pub(in crate::event_loop) compose_stash: Option<Box<ComposeForm>>,
    pub(in crate::event_loop) compose_move_requested: bool,
    pub(in crate::event_loop) ai_ui: AiUi,
    pub(in crate::event_loop) status_section_lines: Vec<(StatusSection, Vec<StatusPaneLine>)>,
    pub(in crate::event_loop) status_section_areas: Vec<(StatusSection, Rect)>,
    pub(in crate::event_loop) status_states: StatusStates,
    pub(in crate::event_loop) active_status_section: StatusSection,
    pub(in crate::event_loop) commit_file_rows: Vec<TreeRow>,
    pub(in crate::event_loop) commit_paths_cache: Vec<PathBuf>,
    pub(in crate::event_loop) commit_message_scroll: usize,
    pub(in crate::event_loop) commit_ref_urls: Vec<(String, bool)>,
    pub(in crate::event_loop) commit_message_for: Option<Oid>,
    pub(in crate::event_loop) commit_message_user_rows: u16,
    pub(in crate::event_loop) commit_message_area: Rect,
    pub(in crate::event_loop) commit_message_total: usize,
    pub(in crate::event_loop) commit_message_rows_now: u16,
    pub(in crate::event_loop) commit_detail_status_height: u16,
    pub(in crate::event_loop) dragging_commit_message: bool,
    pub(in crate::event_loop) commit_sort_desc: bool,
    pub(in crate::event_loop) view_all_files: bool,
    /// What the Log's multi-select compares (`compare::resolve`), refreshed
    /// every frame; `None` below two selected rows.
    pub(in crate::event_loop) compare: Option<crate::compare::CompareSpec>,
    /// The user swapped the two ends of the comparison of `compare_keys`.
    pub(in crate::event_loop) compare_swapped: bool,
    /// The selected rows the comparison was last built from: a different
    /// set forgets the swap.
    pub(in crate::event_loop) compare_keys: Vec<RowKey>,
    pub(in crate::event_loop) wip_all: WipAllFiles,
    pub(in crate::event_loop) commit_filter_focus: bool,
    pub(in crate::event_loop) log_area: Rect,
    pub(in crate::event_loop) status_area: Rect,
    pub(in crate::event_loop) refs_area: Rect,
    pub(in crate::event_loop) refs_filter_area: Rect,
    pub(in crate::event_loop) log_search_bar_area: Option<Rect>,
    pub(in crate::event_loop) sidebar_section_areas: Vec<(SidebarSection, Rect)>,
    pub(in crate::event_loop) sidebar_heights: Vec<u16>,
    /// Left-pane sections hidden in this repository: its own choice, else the
    /// global default (`ui.sidebar_sections_hidden`); see `sections_scope`.
    pub(in crate::event_loop) hidden_sidebar_sections: HashSet<SidebarSection>,
    pub(in crate::event_loop) sidebar_scope: crate::sections_scope::SectionsScope,
    pub(in crate::event_loop) sidebar_layout_specs: Vec<sidebar_layout::SectionSpec>,
    pub(in crate::event_loop) sidebar_layout_avail: u16,
    pub(in crate::event_loop) dragging_sidebar_section: Option<SidebarSection>,
    pub(in crate::event_loop) panes_area: Rect,
    pub(in crate::event_loop) commit_files_area: Rect,
    pub(in crate::event_loop) commit_meta_area: Rect,
    pub(in crate::event_loop) hit_targets: Vec<(Rect, HitTarget)>,
    pub(in crate::event_loop) nav_badge_targets: Vec<(char, FocusTarget)>,
    pub(in crate::event_loop) hovered_commit:
        Option<(gitpoulpi_core::domain::Oid, LogColumn, u16, u16)>,
    pub(in crate::event_loop) tooltip: tooltip::TooltipLifecycle,
    pub(in crate::event_loop) hovered_wip_row: bool,
    pub(in crate::event_loop) hovered_tab_index: Option<usize>,
    pub(in crate::event_loop) hovered_close_tab: Option<usize>,
    pub(in crate::event_loop) hovered_repo_menu_row: Option<usize>,
    pub(in crate::event_loop) hovered_target: Option<HitTarget>,
    pub(in crate::event_loop) overflow_slots: OverflowSlots,
    pub(in crate::event_loop) preferences: Option<PreferencesState>,
    pub(in crate::event_loop) hovered_resize_strip: Option<Rect>,
    pub(in crate::event_loop) hovered_ref_row: Option<String>,
    pub(in crate::event_loop) hovered_ref_folder: Option<String>,
    pub(in crate::event_loop) ref_drag: Option<RefDrag>,
    /// A repository tab being carried to a new place (`tab_drag`).
    pub(in crate::event_loop) tab_drag: Option<crate::tab_drag::TabDrag>,
    /// Reveal in graph: the pending request and the flashing row.
    pub(in crate::event_loop) reveal: crate::ref_reveal::RevealState,
    pub(in crate::event_loop) ref_drop_source: Option<(String, RefRowKind)>,
    /// The ref marked as the source of a keyboard "mark then drop" (`Refs: Mark as Drop Source`).
    pub(in crate::event_loop) ref_drop_mark: Option<(String, RefRowKind)>,
    pub(in crate::event_loop) ref_popup: Option<RefPopupState>,
    pub(in crate::event_loop) ref_popup_rect: Option<Rect>,
    pub(in crate::event_loop) divider1_pct: u16,
    pub(in crate::event_loop) divider2_pct: u16,
    pub(in crate::event_loop) left_pane_collapsed: bool,
    pub(in crate::event_loop) left_auto_collapse: AutoCollapse,
    pub(in crate::event_loop) right_pane_collapsed: bool,
    /// The merge editor collapses the right panel too (full width),
    /// restoring it on close unless the user opened it meanwhile.
    pub(in crate::event_loop) right_auto_collapse: AutoCollapse,
    /// The Status pane had the focus when the editor collapsed it: it gets
    /// it back when the pane is restored.
    pub(in crate::event_loop) status_focus_restore: bool,
    pub(in crate::event_loop) dragging_divider: Option<u8>,
    pub(in crate::event_loop) wip_row_was_selected: bool,
    /// The view under the editor as it was while it showed the edited file
    /// (`editor.follow_selection`'s Cancel puts it back).
    pub(in crate::event_loop) editor_host_snapshot: Option<window::HostSnapshot>,
    pub(in crate::event_loop) last_saved_log_columns: LogColumnPrefs,
    pub(in crate::event_loop) last_saved_folds: crate::fold_state::Folds,
    pub(in crate::event_loop) log_column_order: Vec<LogColumn>,
    /// The Log column the keyboard has picked (`Shift+C`, then `Tab`, `<` / `>`,
    /// `[` / `]`): `Some` while that mode is on.
    pub(in crate::event_loop) log_col_pick: Option<LogColumn>,
    pub(in crate::event_loop) log_column_hidden: std::collections::HashSet<LogColumn>,
    pub(in crate::event_loop) log_column_overrides: std::collections::HashMap<LogColumn, u16>,
    /// The `scroll` layout's own widths (`ui.log_column_widths_scroll`).
    pub(in crate::event_loop) log_column_scroll_widths: std::collections::HashMap<LogColumn, u16>,
    /// How far the Log's columns are scrolled sideways, in cells (the
    /// `scroll` layout; 0 in the `fit` one). Per window, not saved; clamped
    /// against the geometry on every frame.
    pub(in crate::event_loop) log_h_offset: u16,
    pub(in crate::event_loop) dragging_log_boundary: Option<LogBoundaryDrag>,
    /// What each Log column needs to show its content untruncated, measured
    /// by the last paint (`NaturalWidths`): what "fit to content" reads.
    pub(in crate::event_loop) log_natural_widths: NaturalWidths,
    pub(in crate::event_loop) commit_changes:
        std::collections::HashMap<Oid, gitpoulpi_core::domain::CommitStat>,
    pub(in crate::event_loop) changes_asked: std::collections::HashSet<Oid>,
    pub(in crate::event_loop) visible_commit_ids: Vec<Oid>,
    pub(in crate::event_loop) dragging_scrollbar: Option<(ScrollBar, i32)>,
    pub(in crate::event_loop) last_cursor_shape: CursorShape,
    pub(in crate::event_loop) dragging_log_header: Option<LogHeaderDrag>,
    pub(in crate::event_loop) last_click: Option<(Instant, u16, u16)>,
    pub(in crate::event_loop) input_mode: InputMode,
    pub(in crate::event_loop) ref_name_error: Option<String>,
    pub(in crate::event_loop) ref_name_revealed: bool,
    /// Which painter holds the name field (inline spot or the name box), kept
    /// until the prompt ends so the field never jumps (`ref_name_box.rs`).
    pub(in crate::event_loop) ref_name_slot: NameSlot,
    pub(in crate::event_loop) last_message: Option<String>,
    pub(in crate::event_loop) refs_filter: String,
    pub(in crate::event_loop) collapsed_ref_sections: std::collections::HashSet<String>,
    pub(in crate::event_loop) hierarchical_refs: bool,
    pub(in crate::event_loop) collapsed_ref_folders: CollapsedFolders,
    /// While the refs name filter is active, `collapsed_ref_folders` is the
    /// filter's own all-open view and the user's set is parked in here
    /// (`tree_fold::FilterOverlay`); what is saved is always the user's.
    pub(in crate::event_loop) ref_fold_overlay: crate::tree_fold::FilterOverlay<CollapsedFolders>,
    pub(in crate::event_loop) last_saved_ref_folders: CollapsedFolders,
    /// The same for the commit's changed files and their filter.
    pub(in crate::event_loop) commit_fold_overlay:
        crate::tree_fold::FilterOverlay<std::collections::HashSet<std::path::PathBuf>>,
    pub(in crate::event_loop) remote_names: Vec<String>,
    pub(in crate::event_loop) remote_ui: RemoteUi,
    /// The repository-local `core.autocrlf` / `core.eol` as git reports them
    /// (Settings > Git & Encoding shows these), and a counter bumped on each
    /// refresh that read them.
    pub(in crate::event_loop) repo_config: std::collections::BTreeMap<String, Option<String>>,
    pub(in crate::event_loop) repo_config_version: u64,
    /// Whether the working tree is a sparse checkout (the footer badge); the
    /// Sparse checkout dialog gets the same state through `repo_dialogs`.
    pub(in crate::event_loop) sparse: gitpoulpi_core::domain::SparseState,
    pub(in crate::event_loop) hidden_from_graph: HashSet<String>,
    pub(in crate::event_loop) solo_refs: HashSet<String>,
    pub(in crate::event_loop) log_menu_refs: pill_menu::RefTable,
    pub(in crate::event_loop) open_menu: OpenMenu,
    pub(in crate::event_loop) clipboard: clipboard::Clipboard,
    pub(in crate::event_loop) worktree_seed: Option<WorktreeSeed>,
    /// Keeps the workflow report fresh (`flow::source`).
    pub(in crate::event_loop) flow_source: crate::flow::FlowSource,
    /// Watched issues whose branch a fetch brought (`flow::issue_watch`).
    #[cfg(feature = "hosting")]
    pub(in crate::event_loop) issue_watch: crate::flow::issue_watch::IssueWatch,
    pub(in crate::event_loop) pending_worktree_tab: Option<PathBuf>,
    /// The newest commit a split created, selected in the Log as soon as a
    /// refresh lists it (given up after a few seconds).
    /// An agent's split proposal waiting for the commit's numbered changes:
    /// the commit and the plan to open the editor with.
    pub(in crate::event_loop) pending_split_proposal:
        Option<(Oid, Vec<gitpoulpi_core::domain::SplitPart>)>,
    pub(in crate::event_loop) pending_split_select: Option<(Oid, Instant, Option<Oid>)>,
    pub(in crate::event_loop) pull_request_seed: Option<PullRequestSeed>,
    pub(in crate::event_loop) patch_seed: Option<crate::repo_dialogs::PatchSeed>,
    pub(in crate::event_loop) session: String,
    /// The session file as this window last saw it
    /// (`session_store::session_stamp`): a change that is not ours is a
    /// `gitpoulpi repo open|close` from outside.
    pub(in crate::event_loop) session_stamp: Option<(std::time::SystemTime, u64)>,
    pub(in crate::event_loop) repo: RepoRef,
    pub(in crate::event_loop) palette_popup: Option<PalettePopup>,
    pub(in crate::event_loop) tab_picker_popup: Option<PalettePopup>,
    pub(in crate::event_loop) log_order: graph::LogRows,
    pub(in crate::event_loop) show_help: bool,
    pub(in crate::event_loop) help_scroll: u16,
    pub(in crate::event_loop) help_filter: String,
    pub(in crate::event_loop) help_popup_inner_height: u16,
    pub(in crate::event_loop) file_insight: Option<FileInsight>,
    /// The File History split view; its diff is `diff_view_state`.
    pub(in crate::event_loop) history: Option<crate::history_view::HistoryView>,
    /// The file view the remembered `File` list was last applied to (so a
    /// History that cannot open does not retry every tick); cleared while no
    /// commit's file is open. See `WindowState::apply_remembered_list`.
    pub(in crate::event_loop) list_applied_to: Option<(std::path::PathBuf, Oid)>,
    /// The commit list left of the Diff / File View / Blame / Edit views.
    pub(in crate::event_loop) commit_col: crate::commit_column::CommitColumn,
    pub(in crate::event_loop) editor: Option<crate::editor::Editor>,
    /// The merge conflict editor, when open (`CentralView::Merge`).
    pub(in crate::event_loop) merge: Option<crate::merge_view::MergeView>,
    /// The editor's side of the window: opening, saving, the file cache.
    /// The remembered merge sizes were put on the open editor (reset when
    /// it closes, so the next one adopts them again).
    pub(in crate::event_loop) merge_layout_applied: bool,
    /// What was last written for the merge editor: column weights and
    /// Output percent (`0` = default). Compared per frame.
    pub(in crate::event_loop) merge_layout_saved: (Option<[u16; 3]>, u16),
    pub(in crate::event_loop) merge_flow: crate::merge_flow::MergeFlow,
    /// The merge, cherry-pick, revert or rebase in progress, whichever it
    /// is: what the operation banner draws from.
    pub(in crate::event_loop) operation_state: Option<gitpoulpi_core::domain::OperationState>,
    /// The repository's `index.lock`, when one exists (the stale-lock strip).
    pub(in crate::event_loop) index_lock: Option<gitpoulpi_core::domain::IndexLock>,
    pub(in crate::event_loop) file_insight_area: Rect,
    pub(in crate::event_loop) requested_commit_files: Option<gitpoulpi_core::domain::Oid>,
    pub(in crate::event_loop) requested_commit_tree: Option<gitpoulpi_core::domain::Oid>,
    pub(in crate::event_loop) commit_files: Option<daemon_link::CommitFilesOutcome>,
    pub(in crate::event_loop) commit_files_len: usize,
    pub(in crate::event_loop) last_auto_fetch: Instant,
    pub(in crate::event_loop) field_editor: text_edit::FieldEditor<MainField>,
    pub(in crate::event_loop) pending_field_click: Option<(MainField, u16, u16, Instant)>,
    pub(in crate::event_loop) dragging_list_bar: Option<(ListBar, i32)>,
    pub(in crate::event_loop) open_first_frame_logged: bool,
    pub(in crate::event_loop) open_first_outcome_logged: bool,
    pub(in crate::event_loop) visible_sidebar: Vec<SidebarSection>,
    pub(in crate::event_loop) should_quit: bool,
    /// The status message as last seen by `note_message`, and when it
    /// first appeared: a refresh only clears a message that has been up for
    /// `MESSAGE_MIN_LIFETIME`.
    pub(in crate::event_loop) message_seen: Option<String>,
    pub(in crate::event_loop) message_set_at: Instant,
    /// Redraw on demand: see `window/idle.rs`.
    pub(in crate::event_loop) redraw: super::window::RedrawState,
}
