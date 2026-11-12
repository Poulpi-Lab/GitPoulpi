//! Window initialisation: everything `run_in_repo` sets up before its main
//! loop.

mod state;

use super::*;

pub(super) use state::WindowState;

/// Opens the terminal, resolves the settings and builds the window's state.
pub(super) fn init(
    session: &str,
    repo: &RepoRef,
    start_dir: &std::path::Path,
    entry: ManagementEntry,
) -> io::Result<WindowState> {
    // The only place `config.toml` reaches the TUI. Resolved *before* the
    // alternate screen so a config that will not parse is a readable line on a
    // normal terminal. `[ui] mouse_enabled = false` never enters mouse-capture
    // mode, which hands click-drag selection back to the terminal; teardown
    // sends `DisableMouseCapture` unconditionally (a no-op when capture was
    // never enabled).
    let open_t0 = Instant::now();
    open_trace(&format!("window opening: {}", repo.root.display()), open_t0);
    let settings = gitpoulpi_config::Settings::resolve_ambient()
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))?;
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    // `ui.theme = system`: the terminal's own background decides Light or
    // Dark (OSC 11, then COLORFGBG, then Dark), once, bounded to ~100 ms,
    // its reply fully read. Before the keyboard probe and anything that
    // reads input.
    if settings.config().ui.theme_slug() == gitpoulpi_config::THEME_SYSTEM {
        crate::theme::set_system_is_light(crate::terminal_bg::detect_is_light().unwrap_or(false));
    }
    crate::color_depth::install(settings.config().ui.color_depth);
    crate::text_cursor::install(settings.config().ui.text_cursor);
    // The kitty keyboard protocol (`ui.kitty_keyboard`): a bounded query,
    // then a push the teardown pops. Before anything reads the keyboard.
    crate::kitty_keyboard::enable(settings.config().ui.kitty_keyboard, &mut stdout);
    if settings.config().ui.mouse_enabled {
        execute!(stdout, EnableMouseCapture)?;
    }
    // Bracketed paste: a paste arrives as one `Event::Paste`, so its line
    // breaks never submit and its letters never fire shortcuts.
    crate::term_modes::enable_bracketed_paste(&mut stdout);
    {
        // Focus loss hides any tooltip (`tooltip.rs`); best-effort.
        let _ = execute!(stdout, crossterm::event::EnableFocusChange);
    }
    // Whether the Graph column can be drawn as pixels, and how big one cell
    // is. Side-effect free and input-free (it must never read stdin: see
    // `terminal_image`'s module header); done once because neither the
    // environment nor the setting changes under a running process.
    // `ui.image_protocol` picks iTerm2 or Kitty outright; `auto` lets
    // detection choose (Kitty inside a Kitty-enabled herdr).
    let mut graph_image = graph::GraphImage::new(terminal_image::detect(
        settings.config().ui.graph_renderer.into(),
        settings.config().ui.image_protocol,
    ));
    // `[keybindings]`: the effective keymap, installed before the first
    // key is read; entries it could not honour are reported once on the
    // status line (and again by Settings > Keybindings).
    let startup_keymap_message =
        keymap::issues_message(&keymap::install_from_config(settings.config()));
    let ui_lane_background: graph::LaneBackground =
        settings.config().ui.graph_lane_background.into();
    let ui_graph_theme =
        crate::graph::LaneTheme::from_slug(settings.config().ui.graph_color_slug());
    let ui_tint_strength = settings.config().ui.graph_row_tint_strength;
    // `ui.graph_show_stashes`: a row per stash in the Log, on by default.
    // Re-read when the Preferences screen closes (`apply_live_settings`).
    let graph_show_stashes = settings.config().ui.graph_show_stashes;
    // `ui.ref_connector`: the thin line from a ref pill to its commit's
    // node, on by default. Re-read with the others.
    let ref_connector = settings.config().ui.ref_connector;
    // Author avatars (`ui.show_avatars`, off by default; see `avatar`'s module
    // header for the privacy reasoning). The service is started unconditionally
    // (idle is cheap: it only sends a job when asked about an author), so the
    // Preferences toggle works live, unlike `ui.graph_renderer`.
    // `avatar_capability` reuses the detection `graph_image` already ran.
    // Live: re-read by `apply_live_settings` when Preferences closes.
    // `ui.avatars` (graph / author / off), the legacy `ui.show_avatars`
    // folded in by `avatar_placement`.
    let avatar_placement = settings.config().ui.avatar_placement();
    // Sixel on Windows has no way to read the cell size from the system:
    // ask the terminal once, before any input thread-like reader exists.
    if terminal_image::measure_cell_px(graph_image.capability().protocol) {
        graph_image.set_cell(terminal_image::cell_size());
    }
    let avatar_capability = graph_image.capability();
    let commit_graph_image = graph::GraphImage::new(avatar_capability);
    graph_image.set_avatars(avatar::in_graph(avatar_placement, avatar_capability));
    graph::set_avatars_in_graph(avatar::in_graph(avatar_placement, avatar_capability));
    // The bitmap is decoded once, at the size of the biggest picture drawn
    // from it: the commit detail's square author box (at the terminal's
    // own pixel size) or the graph node's picture box
    // (`graph::avatar_picture_px`), whichever is larger -- never a single
    // cell upscaled into a blurred, squashed picture.
    let (detail_avatar_cols, detail_avatar_rows) = avatar::detail_size(avatar_capability.cell);
    let avatar_cell_px = avatar_box_px(avatar_capability.cell);
    let avatar_cache_dir = gitpoulpi_config::ConfigPaths::discover()
        // `avatars-v2`: the pictures cached under `avatars` were fetched at
        // the (tiny) node size; they would keep being served for the
        // cache's lifetime, so the bigger fetch needs a fresh directory.
        .map(|paths| paths.cache_dir().join("avatars-v2"))
        .unwrap_or_else(std::env::temp_dir);
    // The GitHub token (a github.com `[[integrations]]` entry's) is
    // looked up on the worker, once, only if the GitHub API is ever
    // asked -- never here on the input thread.
    let avatar_service = avatar::AvatarService::spawn(
        avatar_cache_dir,
        avatar_cell_px,
        avatar::github_token_provider(&settings.config().integrations),
        avatar::gitlab_token_provider(&settings.config().integrations),
    );
    // `ui.avatar_sources`, `ui.avatar_fallback` and the connected
    // integrations: pushed into the service on the first frame, and again
    // whenever Preferences closes or the remotes change.
    let avatar_live = avatar::LiveSettings::from_config(settings.config());
    let detail_avatar_image = avatar::DetailAvatarImage::new(avatar_capability);
    // One round picture per visible Log row, in the Author column
    // (`avatar::log_images`): same capability verdict as the Graph
    // column, so herdr/tmux/`ui.graph_renderer = "text"` keep the
    // initials.
    let log_avatar_images = avatar::LogAvatarImages::new(avatar_capability);
    // Rows per mouse-wheel notch in the Log (`ui.mouse_scroll_lines`, default 3).
    let log_scroll_lines = i32::from(settings.config().ui.mouse_scroll_lines_clamped());
    // The live settings this process acts on, published before the
    // first refresh goes out so the very first `Log` request already
    // carries the configured page size rather than a default that a
    // later frame quietly replaces.
    daemon_link::set_log_page_limit(settings.config().general.initial_commits_in_graph_clamped());
    live_settings::install_language(&settings.config().ui);
    live_settings::install_date_patterns(&settings.config().ui);
    live_settings::install_auto_prune(settings.config().general.auto_prune);
    live_settings::install_auto_prune_tags(settings.config().general.auto_prune_tags);
    crate::build_features::install_radar_enabled(settings.config().general.radar_enabled);
    crate::build_features::install_radar_closed(session_store::radar_closed(session));
    crate::build_features::install_workspaces_open(session_store::workspaces_open(session));
    live_settings::install_minimap(settings.config().ui.minimap);
    live_settings::install_column_resize(settings.config().ui.column_resize);
    live_settings::install_log_columns_layout(settings.config().ui.log_columns_layout);
    live_settings::install_chrome_layout(&settings.config().ui);
    live_settings::install_dialog_geometry(settings.config().ui.dialog_geometry);
    live_settings::install_mermaid(&settings.config().ui);
    live_settings::install_blame(&settings.config().ui);
    live_settings::install_image_view(&settings.config().ui);
    crate::tab_dir::install(&settings.config().ui);
    crate::image_editor::install(&settings.config().ui);
    live_settings::install_log_wip_counts_in_message(
        settings.config().ui.log_wip_counts_in_message,
    );
    live_settings::install_reveal_on_ref_click(settings.config().ui.reveal_on_ref_click);
    live_settings::install_sidebar_sections_global(
        settings.config().ui.sidebar_sections_scope
            == gitpoulpi_config::SidebarSectionsScope::Global,
    );
    live_settings::install_graph_ghost_ref(settings.config().ui.graph_ghost_ref);
    live_settings::install_hide_finished_pr_chip(settings.config().ui.hide_finished_pr_chip);
    live_settings::install_group_worktree_tabs(settings.config().general.group_worktree_tabs);
    live_settings::request_regroup();
    crate::editor_prefs::install(&settings.config().editor);
    crate::merge_view::layout::install(&settings.config().merge);
    terminology::install(settings.config().ui.pr_terminology);
    install_input_color(
        &settings.config().ui.input_color,
        &settings.config().ui.input_color_custom,
    );
    crate::install_theme_from_config(settings.config().ui.theme_slug());
    lfs_ui::install_default(settings.config().git.lfs_enabled);
    live_settings::install_delete_orig_after_merge(
        settings.config().general.delete_orig_files_after_merge,
    );
    live_settings::install_active_profile(settings.config());
    live_settings::install_commit_count(settings.config().log.commit_count);
    crate::updates::configure(settings.config(), crate::updates::state_path());
    crate::updates::start_background_check();
    workspace_ui::chip::refresh(session);
    profile_bar::load_repo(&repo.root, session);
    external_tools::install(settings.config());
    // The Diff view's last-used layout/whitespace/wrap -- published here
    // so the very first diff opened in this window already resumes
    // whatever `config.toml` says, and kept in scope through the whole
    // loop below so `persist_diff_view_toggle` can write a change back to
    // the same file every other Preferences edit uses (`set_in_file`'s
    // load-modify-save, never a full `Config::save` that would clobber
    // keys this process never touched).
    live_settings::install_diff_view_prefs(live_settings::DiffViewPrefs {
        layout: settings.config().ui.diff_layout,
        hunks_only: settings.config().ui.diff_hunks_only,
        show_whitespace: settings.config().ui.diff_show_whitespace,
        word_wrap: settings.config().ui.diff_word_wrap,
    });
    let diff_config_path = settings.file().map(Path::to_path_buf);
    // The review's diff pane writes the same toggles from outside the event loop.
    live_settings::install_diff_config_path(diff_config_path.clone());
    let auto_fetch_interval = settings.config().general.auto_fetch_interval();
    let fetch_on_open_pending = settings.config().general.fetch_on_tab_open;
    // Pull/merge requests and issues of this window's repository
    // (`hosting_ui`): its own worker thread, its own refresh policy.
    let hosting = HostingUi::new(
        &repo.root,
        settings.config().general.hosting_refresh_interval(),
    );
    let mut hosting = hosting;
    let restored_folds = crate::fold_state::load(&repo.root);
    hosting.restore_folded_groups(restored_folds.pr_groups.clone());

    let backend = crate::quiet_backend::QuietBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    // Kept because `repo` is moved into the daemon link below. The Preferences
    // repo scope is keyed on the working-tree root, never the cwd: the TUI and
    // the CLI must key the same way or one daemon sees two repos.
    let repo_root = repo.root.clone();

    let title = super::draw::chrome::header_title(&repo.root);
    // Where a typed relative path and a clone destination are resolved
    // against. Captured once rather than read per use: the process'
    // directory does not change, and re-reading it in a key handler is
    // an `io::Result` in a place that has nothing useful to do with one.
    let cwd = std::env::current_dir()?;
    // True for a window that has no repository to show -- started
    // outside one, or left without one by closing the last tab. It
    // opens the overlay by itself and keeps the Log from shouting a
    // backend error about a path that was never a repository.
    let no_repository = repo.root == std::path::Path::new(NO_REPOSITORY);
    // The management tab's Browse / Init / Clone dialogs (`repo_dialogs`),
    // with the folder reads and the init / clone tasks they run in the
    // background.
    let mut repo_dialogs = RepoDialogs::new(session);
    // A window with no repository *is* the Repository Management tab
    // (`repo_manager.rs`): the list of every repository known, with Browse /
    // Clone / Init. No repository sits behind it to steal a click or wheel
    // notch. Its probe fills in branches and counters in the background.
    let mut management: Option<repo_manager::RepoManager> = None;
    let mut management_probe: Option<repo_probe::Probe> = None;
    // A bad `--repo` is explained on the status line / Home's footer.
    let launch_notice = crate::tab_dir::take_launch_notice();
    if no_repository {
        let (mut opened, probe) = open_management(session);
        opened.message = launch_notice.clone();
        if entry == ManagementEntry::NewTab {
            opened.new_tab = true;
            opened.search_focused = true;
        }
        if entry == ManagementEntry::Workspaces {
            opened.show_page(repo_manager::HomePage::Workspaces);
        }
        if entry == ManagementEntry::Radar {
            opened.radar.open(session);
        }
        if entry == ManagementEntry::ReleaseNotes {
            opened.notes.open();
        }
        management = Some(opened);
        management_probe = Some(probe);
        repo_dialogs.open(entry, start_dir);
    }
    let repo_tabs: Vec<repo_switcher::RepoTab> = session_store::tabs(session);
    // The management window has no tab of its own in the list: it keeps
    // the session's active tab, so `Esc` goes back to where you were.
    let active_tab = repo_tabs
        .iter()
        .position(|t| t.path == repo.root)
        .or_else(|| session_store::active_tab_index(session))
        .unwrap_or(0);
    let tab_hits: Vec<repo_switcher::TabHit> = Vec::new();
    // `mut`: a stopped link is replaced by a fresh one on Retry (`r`
    // while `link_failure` is set), never left for the window to wait on.
    let link: DaemonLink = daemon_link::start(session.to_string(), repo.clone());
    open_trace("daemon link started", open_t0);
    // The TUI as a viewer for an agent's `ui` intents (`ui_viewer`).
    let ui_viewer = crate::ui_viewer::UiViewer::new(settings.config().ui.allow_agent_view_control);

    let log_result: gitpoulpi_core::Result<gitpoulpi_core::domain::CommitGraph> = Err(
        gitpoulpi_core::Error::Backend(gitpoulpi_i18n::msg::window::CONNECTING.get().to_string()),
    );
    let status_result: gitpoulpi_core::Result<gitpoulpi_core::domain::WorkingTree> = Err(
        gitpoulpi_core::Error::Backend(gitpoulpi_i18n::msg::window::CONNECTING.get().to_string()),
    );
    let refs_result: gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Ref>> = Err(
        gitpoulpi_core::Error::Backend(gitpoulpi_i18n::msg::window::CONNECTING.get().to_string()),
    );
    let stashes_result: gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Stash>> = Err(
        gitpoulpi_core::Error::Backend(gitpoulpi_i18n::msg::window::CONNECTING.get().to_string()),
    );
    let worktrees_result: gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Worktree>> = Err(
        gitpoulpi_core::Error::Backend(gitpoulpi_i18n::msg::window::CONNECTING.get().to_string()),
    );
    // True until the *first* `RefreshOutcome` arrives on `link.outcome_rx`.
    // The five `Result`s above are seeded with a placeholder `Err` so early
    // reads behave like "nothing yet"; rendering reads `still_loading`, never
    // the placeholder's text (see `docs/architecture/session-and-state.md`,
    // "Loading state, not an error").
    let still_loading = !no_repository;
    let repo_open_started_at = Instant::now();
    let pending_auto_stash = crate::autostash_banner::load(&repo.root);
    // Incremental loading of older history. Every decision (when to ask, what
    // to ask for, whether an arriving page is still valid, when history ends)
    // lives in `graph::LazyLog`, unit-tested there without a terminal; this
    // function only holds the value.
    let lazy_log = graph::LazyLog::new(graph::LogLoadSettings::from_config(
        &settings.config().general,
    ));

    // The minimap's pixel picture goes through the terminal image
    // protocol, like the commit-detail avatar.
    minimap::init(avatar_capability);
    crate::about::init(avatar_capability);
    crate::title_logo::init(avatar_capability);
    crate::about::install(&settings.config().ui);
    crate::launcher_sync::install(settings.config());
    crate::image_view::pane::init(avatar_capability);
    crate::image_view::split::init(avatar_capability);
    // The diff render pass's row map: which hunk and which of its lines each
    // rendered row shows. It must come from the walk that drew the rows, not a
    // second one over the same hunks: drift here stages the wrong lines.
    let diff_rows: std::rc::Rc<[diff_view::DiffRow]> = std::rc::Rc::from(Vec::new());
    let collapsed_status_sections: HashSet<StatusSection> = restored_folds.status.clone();
    let commit_message_user_rows: u16 = session_store::layout(session).commit_message_rows;
    // The heights the user dragged the sidebar sections to (`0` = automatic),
    // restored from the session's layout and saved when a drag or a keyboard
    // resize ends. `sidebar_layout_specs` / `sidebar_layout_avail` are what
    // the last frame laid out from, kept so a drag can clamp against the same
    // numbers the renderer used.
    let sidebar_heights: Vec<u16> = session_store::layout(session).sidebar_heights;
    // The Preferences screen, when it is open. Modal, like the File
    // Insight overlay: while it is up it owns the keyboard and the
    // mouse, and nothing underneath is reachable.
    let preferences: Option<PreferencesState> = None;
    // The three-pane split, restored from the session file like the tab list
    // (`session_store::layout`); a session that never saved one gets
    // `PaneLayout::default()` (20/65, nothing collapsed).
    let restored_layout = session_store::layout(session);
    // The commit column's shown parts and widths are `[ui]` settings. A
    // session written before them moved holds its own: used once, copied to
    // the config, then forgotten by the session.
    let mut column_prefs = crate::commit_column::ColumnPrefs::from_ui(&settings.config().ui);
    if column_prefs.is_default() {
        let legacy = crate::commit_column::ColumnPrefs::from_legacy(&restored_layout);
        if !legacy.is_default() {
            column_prefs = legacy;
            if let Some(path) = &diff_config_path {
                let saved = gitpoulpi_config::set_commit_column_in_file(
                    path,
                    &column_prefs.visible,
                    &column_prefs.order,
                    &column_prefs.width_map(),
                );
                if saved.is_ok() {
                    session_store::forget_legacy_commit_column(session);
                }
            }
        }
    }
    let divider1_pct: u16 = restored_layout.divider1_pct;
    let divider2_pct: u16 = restored_layout.divider2_pct;
    let left_pane_collapsed: bool = restored_layout.left_collapsed;
    let right_pane_collapsed: bool = restored_layout.right_collapsed;
    crate::folder_picker::install_split(restored_layout.picker_split_permille);
    // Log table column layout: display order (drag-to-reorder), hidden
    // set (right-click header menu), and manual width overrides
    // (drag-to-resize a column boundary) -- see `LogColumnWidths` and
    // `HitTarget::{LogColumnBoundary,LogColumnHeader,ColumnMenuItem}`.
    // Global (one state for every tab) and persisted in `[ui]`
    // (`log_columns_hidden/order`, `log_column_widths`): loaded here,
    // saved from the top of the frame loop whenever it changed.
    let initial_log_columns = LogColumnPrefs::from_ui(&settings.config().ui);
    let last_saved_log_columns = initial_log_columns.clone();
    // Left-pane sections hidden by the user: global, `[ui]`, saved from the tick.
    let (hidden_sidebar_sections, sidebar_scope) = crate::sections_scope::SectionsScope::resolve(
        crate::refs_sidebar::hidden_from_names(&settings.config().ui.sidebar_sections_hidden),
        crate::sections_scope::load_own(&crate::fold_state::state_path(&repo.root)).as_deref(),
        crate::live_settings::sidebar_sections_global(),
    );
    let last_saved_folds = crate::fold_state::Folds::capture(
        &restored_folds.sidebar,
        &restored_folds.status,
        hosting.folded_groups(),
    )
    .with_trees(
        &mut restored_folds.status_set(),
        &mut restored_folds.commit_set(),
        &crate::fold_state::Folds {
            status_folders: restored_folds.status_folders.clone(),
            commit_folders: restored_folds.commit_folders.clone(),
            ..Default::default()
        },
    );
    let log_column_order: Vec<LogColumn> = initial_log_columns.order;
    let log_column_hidden: std::collections::HashSet<LogColumn> = initial_log_columns.hidden;
    let log_column_overrides: std::collections::HashMap<LogColumn, u16> =
        initial_log_columns.widths;
    let log_column_scroll_widths: std::collections::HashMap<LogColumn, u16> =
        initial_log_columns.scroll_widths;
    let dragging_log_boundary: Option<LogBoundaryDrag> = None;
    // The Log "Changes" column's numbers, by commit (immutable, so never
    // stale): filled from the daemon for the rows on screen only. The
    // frame records which commits are visible; the loop asks for the ones
    // not held or already asked, after the paint (`request_visible_changes`).
    let commit_changes: std::collections::HashMap<Oid, gitpoulpi_core::domain::CommitStat> =
        std::collections::HashMap::new();
    let changes_asked: std::collections::HashSet<Oid> = std::collections::HashSet::new();
    // A tab switch builds a new window: an unread failure stays on the line.
    let last_message: Option<String> = launch_notice
        .or(startup_keymap_message)
        .or_else(crate::activity_log::status_error::carry);
    let collapsed_ref_sections: std::collections::HashSet<String> = restored_folds.sidebar.clone();
    // Folder hierarchy for LOCAL/REMOTE branch names that
    // contain `/`, on by default. `ui.ref_tree` is the setting; `y` in the Refs
    // pane and the palette entry toggle it live *and write it back*. Tags never
    // go hierarchical (see `refs_tree::ref_display_rows`).
    let hierarchical_refs = gitpoulpi_config::Settings::resolve_ambient()
        .map(|s| s.config().ui.ref_tree)
        .unwrap_or(true);
    // Per repository, not per config: a folder only exists in the repo
    // whose branch names make it. Loaded once here and written back on
    // every toggle (`persist_collapsed_ref_folders`).
    let collapsed_ref_folders: CollapsedFolders = refs_tree::load_collapsed(&repo.root);
    // Remotes with their URLs, the chosen default remote, and the one
    // open remote menu or Add/Edit popup (`remotes_ui.rs`).
    let remote_ui = RemoteUi::load(&repo.root);
    // Refs hidden from the Log graph, and the soloed set
    // (`refs_tree::load_ref_visibility`/`save_ref_visibility`, persisted in
    // `<repo>/.gitpoulpi/repo-state.toml`). Hidden refs still show, dimmed, in
    // the Refs sidebar but get no pill and do not count toward `has_ref`. Solo
    // shows only the soloed refs' ancestry; it accumulates (soloing a second
    // ref adds it) and stores each ref's full name (`ref_full_name`), resolved
    // to an oid each frame via `refs_result` since branches move. Hiding or
    // soloing a folder or remote (`refs_sidebar::refs_under_folder`/
    // `toggle_folder_membership`) writes into these same two sets: a branch
    // created later under a hidden folder stays visible until the folder is
    // hidden again.
    let (hidden_from_graph, solo_refs) = refs_tree::load_ref_visibility(&repo.root);
    let log_order = log_rows_now(
        &log_result,
        &status_result,
        &stashes_result,
        stash_rows_shown(graph_show_stashes, &solo_refs),
        &solo_refs,
        &refs_result,
    );
    // A new window starts without the previous repository's workflow.
    crate::flow::FlowSource::reset();
    Ok(WindowState {
        open_t0,
        settings,
        graph_image,
        commit_graph_image,
        ui_lane_background,
        ui_graph_theme,
        ui_tint_strength,
        graph_show_stashes,
        ref_connector,
        avatar_placement,
        avatar_capability,
        detail_avatar_cols,
        detail_avatar_rows,
        avatar_service,
        avatar_live,
        // The frame size the cell pixel size was last measured at.
        zoom_check_size: ratatui::layout::Size::default(),
        detail_avatar_image,
        log_avatar_images,
        log_scroll_lines,
        diff_config_path,
        auto_fetch_interval,
        fetch_on_open_pending,
        hosting,
        hosting_prefs_was_open: false,
        terminal,
        repo_root,
        title,
        cwd,
        // Set by the repository overlay or the tab bar; returned at the end
        // so the outer `run` loop rebuilds everything for the new repo.
        switch_to: None,
        // Set when the last tab was closed: the window ends with no
        // repository rather than quitting (see `NextWindow`).
        close_window: false,
        // How the user asked to quit (`q`, the close button, ...), written to the
        // exit log with what was on screen. `None` at the end of a quitting window
        // means nobody asked, which the log then says rather than recording a quit
        // as if it were one.
        quit_how: None,
        // Consecutive failed terminal reads; see `exit_log::tolerate_input_error`.
        // Outside the frame loop so a read that fails on every frame is
        // counted as the run it is.
        input_errors: 0,
        no_repository,
        repo_dialogs,
        management,
        management_probe,
        // Set to leave this window for the management tab.
        show_management: None,
        // Where the management view was drawn, and its hit rects.
        management_panel: None,
        // The tab menu / a management row's `⋮` menu, and its row rects.
        repo_menu: None,
        repo_menu_rects: Vec::new(),
        repo_tabs,
        session_stamp: session_store::session_stamp(session),
        active_tab,
        tab_hits,
        link,
        arrow_repeat: Default::default(),
        ui_viewer,
        // Why the link stopped, once it has (`DaemonLink::failure_rx`). While set,
        // the panes show the reason and "press r to retry" instead of "Loading
        // repository..." forever.
        link_failure: None,
        // Set by `r` / the palette's Refresh while `link_failure` is set; the
        // restart itself happens where the link's channels are drained.
        retry_link_requested: false,
        // The Log's layout and search matches, rebuilt only when one of these
        // generations (bumped wherever the matching result is replaced or
        // extended) or another input changes -- see `graph::LogLayoutCache`.
        log_layout: graph::LogLayoutCache::default(),
        log_gen: 0,
        refs_gen: 0,
        read_hashes: daemon_link::ReadHashes::default(),
        stashes_gen: 0,
        log_result,
        status_result,
        // The status as it was before the first optimistic stage/unstage since
        // the last real refresh: put back if the daemon refuses the action.
        optimistic_backup: None,
        refs_result,
        stashes_result,
        worktrees_result,
        // The SUBMODULES section's rows. A plain list, not a `Result`: a read
        // that fails (or is skipped by the refresh's scope) keeps the last one.
        submodules_result: Vec::new(),
        // "Open in new tab" from a submodule's menu, waiting for the loop to
        // hand it to the tab switcher.
        submodule_open: None,
        still_loading,
        repo_open_started_at,
        // The unmerged paths, refreshed with everything else. Held as the
        // plain `Vec` rather than a `Result` because there is nothing useful
        // to show for "we could not ask": the Status pane's conflict
        // affordances simply stay off, which is also what "no conflicts"
        // looks like.
        conflicts: Vec::new(),
        // The rebase (interactive or plain) currently in progress, if any --
        // drawn as a strip above the Log pane by `rebase_banner`. Same
        // "nothing useful to show for `Err`" reasoning as `conflicts`.
        rebase_state: None,
        pending_auto_stash,
        // What is known about Git LFS in this repository (`lfs_ui.rs`): the
        // LFS tag on rows, the diff card, the missing-program banner.
        // `enabled` is known from the first frame: the toolbar's `LFS`
        // button is laid out from it, and a button that appeared a refresh
        // later would shift the centered toolbar under the pointer.
        lfs: lfs_ui::LfsUi::new(lfs_ui::resolved_enabled(&repo.root)),
        log_len: 0usize,
        lazy_log,
        commit_total: graph::CommitTotal::default(),
        focus: Focus::Log,
        right_panel: RightPanel::CommitDetail,
        // The central column's mode, plus the open diff's own state. `Some`
        // exactly when `central_view == CentralView::Diff` -- kept as two
        // fields rather than `Option` alone so the log-table handlers can ask
        // the cheap question ("am I still the central view?") without caring
        // about diff internals.
        central_view: CentralView::Log,
        diff_view_state: None,
        diff_bar_area: Rect::default(),
        diff_body_area: Rect::default(),
        // Row count of the currently-rendered diff and where each hunk starts,
        // published by the render pass so the scroll clamp and the next/prev
        // hunk jumps work off exactly what was drawn (same discipline as
        // `ref_display_rows` driving both the Refs rows and their count).
        diff_rendered_len: 0usize,
        diff_hunk_starts: Vec::new(),
        // The open file's longest raw line, published by the same render pass
        // for the same reason `diff_rendered_len` is: the horizontal
        // scrollbar's mouse handling runs outside `terminal.draw`'s closure
        // (`open_file` doesn't survive past it), so this is what a scrollbar
        // click/drag geometry is computed against.
        diff_longest_line: 0usize,
        diff_minimap: minimap::Minimap::default(),
        diff_rows,
        // The whole file behind the open diff, and the render cache (a whole-file
        // diff is rendered once per change, not per frame). See `diff_content.rs`.
        diff_content: diff_content::DiffContent::default(),
        // Which rendered diff row the mouse is over, so that hunk's header
        // reveals its Stage/Unstage button -- the third pane's own hover
        // affordance, applied to hunks.
        hovered_diff_row: None,
        // Where a left-button drag inside the diff started. A plain click
        // only moves the cursor; it is the drag that creates a line
        // selection, so the anchor is held here until the mouse actually
        // moves with the button down.
        diff_line_drag: None,
        // The same press, read as the start of a *text* selection (Diff view,
        // File View, Blame, History). The drag that sweeps the line selection above
        // also sweeps this one -- see `text_select`'s module doc and
        // `docs/user/mouse-and-gestures.md`.
        text_drag: None,
        // The last diff render drawn, kept between frames for the text
        // selection's mouse mapping and for copying (the rows map above is
        // kept for the same reason).
        diff_render: None,
        // The working tree's own diff, for the two halves that hunk- and
        // line-level staging can act on. Re-requested on every refresh while
        // such a diff is open: staging a hunk *is* a mutation, so a cached
        // copy would be stale exactly when it matters most.
        working_diff: None,
        log_state: TableState::default(),
        // Commits multi-selected in the Log itself (Ctrl/Shift+click, Space,
        // Shift+Up/Down); the cursor above stays the single 'current' commit.
        log_multi: LogSelection::default(),
        file_sel: FileSelection::default(),
        // True whenever the Log's offset should keep following `log_state`'s
        // selection on render (a keyboard move or a click just happened). Only a
        // mouse-wheel scroll sets it false (it moves the offset and leaves the
        // selection); every place that changes the selection sets it back to
        // true. See `ui_kit::table_render_state`.
        log_follow: true,
        // Three separate collapsible sub-panes (LOCAL/REMOTE/TAGS), each with its
        // own scroll/selection state, plus which one keyboard input targets while
        // `focus == Focus::Refs`.
        sidebar_states: SidebarStates::default(),
        active_ref_section: SidebarSection::Local,
        commit_file_state: ListState::default(),
        // The same "follow" flag as `log_follow`, for the commit-detail file
        // list. `commit_files_visible_rows` is refreshed from the real layout each
        // draw, so the wheel handler (which runs before that frame's draw) clamps
        // against last frame's capacity: one frame of staleness at most.
        commit_files_follow: true,
        commit_files_visible_rows: 1usize,
        commit_file_focus: false,
        // Vim-style count prefix (`4j`, `10k`, ...) for the Log/Refs/Status
        // motions below -- see `apply_count` and the digit-accumulation arm
        // at the top of the `InputMode::Normal` match. `None` means "no count
        // pending", not zero; `0` alone never starts one (`count_digit_value`
        // in `chrome.rs`).
        pending_count: None,
        // --- third pane, presentation state ---
        // The Path/Tree toggle is per-mode: which layout suits the working
        // tree isn't necessarily what suits one commit's handful of files.
        status_list_mode: FileListMode::Tree,
        commit_list_mode: FileListMode::Path,
        collapsed_status_folders: restored_folds.status_set(),
        collapsed_status_sections,
        collapsed_commit_folders: restored_folds.commit_set(),
        commit_files_filter: String::new(),
        // The commit composer at the bottom of the working-changes pane:
        // summary + description + amend, as one box. Kept
        // separate from `InputMode::ComposingCommitMessage` (the older
        // footer-line composer reached by `Ctrl+G c`), which still works.
        commit_summary: String::new(),
        commit_description: String::new(),
        commit_amend: false,
        commit_skip_hooks: false,
        commit_settings: Default::default(),
        commit_settings_ready: false,
        commit_settings_version: 0,
        prefs_open_seen: false,
        commit_template_applied: false,
        commit_squash_seen: None,
        conflict_badge: Default::default(),
        conflict_settings_gen: 0,
        // Which status-pane row the mouse is over, so its Stage/Unstage
        // button can be revealed on hover.
        hovered_status_row: None,
        composer_focus: None,
        // The composer review while the command palette (Ctrl+P inside it) is
        // open, and a "Move file to commit..." the palette asked for.
        compose_stash: None,
        compose_move_requested: false,
        // AI (off by default): the worker and the `[ai]` config. Every entry
        // point below checks `ai_ui.enabled()`.
        ai_ui: {
            let mut ui = AiUi::new();
            ui.adopt(&repo.root);
            ui
        },
        // Rebuilt every frame from `status_result`; the event handlers read it
        // to map a selection/click back to a file, so it must be declared out
        // here rather than inside the draw closure.
        status_section_lines: Vec::new(),
        // The single (section, area) list every mouse site reuses.
        status_section_areas: Vec::new(),
        status_states: StatusStates::default(),
        active_status_section: StatusSection::Unstaged,
        commit_file_rows: Vec::new(),
        commit_paths_cache: Vec::new(),
        // The commit message block of the right panel: its scroll, the commit it
        // was scrolled for, the height the user dragged it to (0 = automatic,
        // persisted in the session layout), and what the last frame drew so the
        // mouse and key handlers clamp against the same numbers.
        commit_message_scroll: 0,
        // The URL behind each `HitTarget::CommitReference(i)` painted this frame.
        commit_ref_urls: Vec::new(),
        commit_message_for: None,
        commit_message_user_rows,
        commit_message_area: Rect::default(),
        commit_message_total: 0,
        commit_message_rows_now: 1,
        commit_detail_status_height: 0,
        dragging_commit_message: false,
        // Descending path order for the commit's file list (the sort button).
        commit_sort_desc: false,
        // `View all files`: show the commit's whole tree rather than the
        // paths it changed. Off by default -- "what did this commit do" is
        // the question the pane exists to answer, and listing a large
        // repository's entire tree is work nobody asked for until they ask.
        view_all_files: false,
        compare: None,
        compare_swapped: false,
        compare_keys: Vec::new(),
        // The same toggle on the WIP row: HEAD's tree, the list's selection and
        // where it was drawn.
        wip_all: WipAllFiles::default(),
        commit_filter_focus: false,
        log_area: Rect::default(),
        status_area: Rect::default(),
        refs_area: Rect::default(),
        refs_filter_area: Rect::default(),
        // The Log's find bar (`InputMode::SearchingLog`), same purpose as
        // `refs_filter_area`: the click handler needs last frame's rect to tell
        // "inside the field" from "outside it". `None` while the bar isn't drawn
        // (not searching, or the Log too narrow for it: see the `bar_w > 12`
        // guard around its render).
        log_search_bar_area: None,
        // One list per frame instead of three fixed Rects: the visible
        // section set is dynamic (worktrees/stashes appear only when
        // non-empty), and every click/hover/scroll site iterates this, so a
        // new section can't be missed at one site and work at another.
        sidebar_section_areas: Vec::new(),
        sidebar_heights,
        sidebar_layout_specs: Vec::new(),
        sidebar_layout_avail: 0,
        dragging_sidebar_section: None,
        panes_area: Rect::default(),
        commit_files_area: Rect::default(),
        commit_meta_area: Rect::default(),
        hit_targets: Vec::new(),
        // `Ctrl+N` navigation mode's current badge assignment -- rebuilt every
        // frame that mode is open (see the render pass in the draw closure
        // below) and consulted by the `InputMode::Navigation` key handler,
        // the same "captured while drawing, read on the next key" shape as
        // `hit_targets`/`frame_hits`.
        nav_badge_targets: Vec::new(),
        hovered_commit: None,
        // The single owner of tooltip visibility (`tooltip.rs`).
        tooltip: crate::tooltip::TooltipLifecycle::default(),
        // The WIP row has no `Oid` for `hovered_commit` to carry, so its hover
        // is a flag of its own.
        hovered_wip_row: false,
        // Which control is lit (gear, close, menu rows, the composer's buttons,
        // pane chevrons, ...) is read from `hovered_target` through
        // `hover::hover`; only hovers keyed by geometry rather than a
        // `HitTarget` keep their own local below.
        // A tab in the repository tab bar (not the leading folder button,
        // which is the `OpenRepositoryButton` target) -- indexes `tab_hits`.
        hovered_tab_index: None,
        // The tab whose close cross the pointer is exactly over.
        hovered_close_tab: None,
        // The tab menu / a management row's `\u{22EE}` menu -- kept apart
        // from `hover::menu_row` since its rects (`repo_menu_rects`) are a
        // separate registry from `hit_targets`, the same way the tab bar's
        // own `tab_hits` are.
        hovered_repo_menu_row: None,
        // ---------------------------------------------------------------
        // Single source of truth for "what is the pointer over right now": set
        // from the one `find_hit_target` call the `Moved` arm makes. Every
        // "is this lit?" question is a `hover::hover` helper over it; a new
        // hoverable element adds a helper there, never another local.
        hovered_target: None,
        // Responsive button bars (`button_bar.rs`). The `⋯` buttons the last
        // frame drew, with what each hides; the open overflow menu, if any; and
        // the hit a chosen menu row stands for. A choice is re-dispatched as a
        // left click at the `⋯`'s own cell with `forced_hit` in place of the hit
        // test, so it runs through the exact code the hidden button's click runs.
        overflow_slots: Vec::new(),
        preferences,
        // The 1-cell strip of a resizable boundary (a pane divider or a Log
        // column boundary) the mouse is over, highlighted so the drag zone is
        // discoverable. This works on every terminal; the `OSC 22` pointer hint in
        // `cursor.rs` only helps where it is implemented (kitty and foot yes,
        // Windows Terminal no).
        hovered_resize_strip: None,
        // Which Refs-sidebar leaf row (by its full ref name) the mouse is
        // currently over -- drives the hide/show-in-graph eye icon's reserved
        // gutter: blank for a visible ref unless hovered (then a plain eye),
        // always a crossed eye for an already-hidden ref regardless of hover.
        hovered_ref_row: None,
        // The folder/remote-root row (its collapse-key path) the mouse is
        // currently over, in the same section-scoped shape `hovered_ref_row`
        // uses for leaves -- drives that row's own hide/solo eye gutter.
        hovered_ref_folder: None,
        // A ref being carried from one ref to another (see `RefDrag`), the
        // mouse half of merge/rebase/cherry-pick. `None` most of the time:
        // it is armed by a press on a ref and only becomes a real drag once
        // the pointer leaves the cell it was armed on.
        ref_drag: None,
        tab_drag: None,
        reveal: Default::default(),
        // The ref a drop menu (`ref_drop`) was opened for, alongside
        // `ref_context_menu` (whose own name/section are the *target*): what
        // its `RefContextAction::Drop` rows act on. Only ever read by those
        // rows, which only that menu has.
        ref_drop_source: None,
        ref_drop_mark: None,
        // The "other refs on this commit" reveal popup. Not derived from
        // `hovered_commit` (see `RefPopupState`): it has its own
        // open/retarget/close transitions, so it behaves like a menu.
        ref_popup: None,
        ref_popup_rect: None,
        divider1_pct,
        divider2_pct,
        left_pane_collapsed,
        // A diff / file view in the center collapses the Refs pane, closing it
        // restores it; never persisted (`AutoCollapse`).
        left_auto_collapse: AutoCollapse::default(),
        right_pane_collapsed,
        right_auto_collapse: AutoCollapse::default(),
        status_focus_restore: false,
        dragging_divider: None,
        // Edge-detection for `wip_selection_should_expand_status`: whether the
        // WIP row was selected as of the previous frame.
        wip_row_was_selected: false,
        editor_host_snapshot: None,
        last_saved_log_columns,
        hidden_sidebar_sections,
        sidebar_scope,
        last_saved_folds,
        merge_layout_applied: false,
        merge_layout_saved: (None, 0),
        log_column_order,
        log_col_pick: None,
        log_column_hidden,
        log_column_overrides,
        log_column_scroll_widths,
        log_h_offset: 0,
        dragging_log_boundary,
        log_natural_widths: NaturalWidths::default(),
        commit_changes,
        changes_asked,
        visible_commit_ids: Vec::new(),
        // A scrollbar thumb drag on one of the scrollable central views (the
        // diff, or the File Insight overlay), started by a mouse-down on the
        // thumb itself -- a click on the *track* (outside the thumb) is a
        // one-shot page jump and never sets this. `grab` is the track-relative
        // row/col, within the thumb, where the drag began (see
        // `scrollbar::Track::drag_offset`'s own doc comment for why).
        dragging_scrollbar: None,
        // The OS pointer shape last requested via OSC 22 (see `cursor.rs`) --
        // tracked so the escape sequence is only re-emitted when the desired
        // shape actually changes, not on every `Moved` event.
        last_cursor_shape: CursorShape::Default,
        dragging_log_header: None,
        last_click: None,
        input_mode: InputMode::Normal,
        // The inline branch/tag name box (`log_inline_ref.rs`): the message
        // shown on the box's row while the typed name is refused, and whether
        // the target commit was already scrolled into view for this prompt.
        ref_name_error: None,
        // Why the Create File prompt refused the typed name. Apart from
        // `ref_name_error`, which the per-frame inline-ref bookkeeping resets
        // for every prompt that is not an inline ref box.
        new_file_error: None,
        ref_name_revealed: false,
        last_message,
        refs_filter: String::new(),
        collapsed_ref_sections,
        hierarchical_refs,
        last_saved_ref_folders: collapsed_ref_folders.clone(),
        ref_fold_overlay: Default::default(),
        commit_fold_overlay: Default::default(),
        collapsed_ref_folders,
        // The repository's remotes, as the backend reports them. The REMOTE
        // section's tree needs them to know where a remote's name ends and a
        // branch's begins -- `origin/feature/login` versus a remote actually
        // named `grp/sub` -- and guessing from the string is exactly the bug
        // `split_remote_ref` exists to prevent. Empty until the first
        // refresh answers, which `ref_display_rows` handles by falling back
        // to the first separator rather than drawing nothing.
        remote_names: Vec::new(),
        remote_ui,
        repo_config: Default::default(),
        repo_config_version: 0,
        sparse: Default::default(),
        hidden_from_graph,
        solo_refs,
        // The refs the open Log menu's `LogContextAction::Ref(_, i)` rows point
        // into (`pill_menu`): a ref pill's merged menu and a commit's per-ref
        // submenus both carry ref rows. Replaced every time the menu opens.
        log_menu_refs: Vec::new(),
        // The one popup menu open over the panes, if any.
        open_menu: OpenMenu::default(),
        // One clipboard for the whole run, never dropped between copies --
        // on X11/Wayland dropping it is what makes a copied value vanish.
        // See `clipboard`'s module header.
        clipboard: clipboard::Clipboard::new(),
        // Create worktree: a menu row / palette command asks for the dialog
        // through `worktree_seed`, and a submitted worktree to open as a tab waits
        // in `pending_worktree_tab` for the refresh that lists it.
        worktree_seed: None,
        flow_source: Default::default(),
        #[cfg(feature = "hosting")]
        issue_watch: Default::default(),
        pending_worktree_tab: None,
        pending_split_proposal: None,
        pending_split_select: None,
        // Start a pull request: the ref menu / palette asks for the dialog through
        // `pull_request_seed` the same way Create worktree does, only when a token
        // is available -- with no token it opens a browser URL directly and never
        // touches this.
        pull_request_seed: None,
        // Create patch: the same shape as `worktree_seed` -- a menu row / palette
        // command asks for the Create patch dialog through `patch_seed`, which
        // `repo_dialogs` opens.
        patch_seed: None,
        log_order,
        session: session.to_string(),
        repo: repo.clone(),
        palette_popup: None,
        tab_picker_popup: None,
        show_help: false,
        help_scroll: 0,
        help_filter: String::new(),
        help_popup_inner_height: 10,
        file_insight: None,
        history: None,
        list_applied_to: None,
        commit_col: crate::commit_column::CommitColumn::new(
            restored_layout.commit_column,
            restored_layout.commit_column_cols,
        )
        .with_saved(restored_layout.commit_column_mode, &column_prefs.widths)
        .with_visible(&column_prefs.visible)
        .with_order(&column_prefs.order),
        editor: None,
        merge: None,
        merge_flow: Default::default(),
        operation_state: None,
        index_lock: None,
        file_insight_area: Rect::new(0, 0, 0, 0),
        requested_commit_files: None,
        requested_commit_tree: None,
        commit_files: None,
        commit_files_len: 0,
        last_auto_fetch: Instant::now(),
        field_editor: text_edit::FieldEditor::default(),
        pending_field_click: None,
        dragging_list_bar: None,
        open_first_frame_logged: false,
        open_first_outcome_logged: false,
        visible_sidebar: Vec::new(),
        should_quit: false,
        message_seen: None,
        message_set_at: Instant::now(),
        redraw: Default::default(),
    })
}
