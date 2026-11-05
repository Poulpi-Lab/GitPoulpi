//! The event loop: `run_in_repo` builds one repository window's
//! [`init::WindowState`] and runs `tick` -> `draw_frame` -> `drain_input` ->
//! `after_input` -> `apply_outcomes` on it (`window/`). The input handlers
//! live in `input/`, the draw passes in `draw/`. Also: the panic hook that
//! restores the terminal and writes a crash report, and applying settings
//! changed live from Preferences.

use super::*;
use crate::keymap::Action as Bind;

pub(crate) mod draw;
pub(super) mod fetch_policy;
mod hover;
mod init;
mod input;
use crate::palette::StashTarget;
use input::commands::{self, CommandCtx, Resize};
mod open_menu;
mod window;
use draw::central::{draw_central, CentralDrawCtx};
use draw::chrome::{draw_chrome, ChromeAreas, ChromeDrawCtx};
use draw::footer::{draw_footer, prefix_hint_rows, FooterDrawCtx};
use draw::log::{draw_log, LogDrawCtx};
use draw::overlay_menus::{draw_overlay_menus, OverlayMenusDrawCtx};
use draw::overlays::{draw_overlays, OverlaysDrawCtx};
use draw::right_panel::{draw_right_panel, RightPanelDrawCtx};
use draw::sidebar::{draw_sidebar, SidebarDrawCtx};
use hover::hover;
use open_menu::OpenMenu;

/// Makes a panic inside the alternate screen readable.
///
/// The terminal is still in raw mode on the alternate screen, so the panic
/// message would be painted into a buffer discarded when the process dies.
/// So: put the terminal back first, then write the whole report to a file
/// and say where. The default hook still runs afterwards.
///
/// `RUST_BACKTRACE` is forced on: a backtrace that has to be asked for is one
/// nobody has when it matters, and the cost is paid only on the way out.
pub(crate) fn install_panic_hook() {
    if std::env::var_os("RUST_BACKTRACE").is_none() {
        std::env::set_var("RUST_BACKTRACE", "1");
    }
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // Best-effort, and deliberately unconditional: the alternate
        // screen and raw mode may or may not be active depending on how
        // far startup got, and every one of these is a no-op if it was
        // not. Failing here would replace the report with nothing.
        let mut out = io::stdout();
        // Popped first: the keyboard mode stack belongs to the screen
        // it was pushed on (the alternate one).
        crate::kitty_keyboard::disable(&mut out);
        crate::cursor::reset_pointer(&mut out);
        let _ = execute!(
            out,
            DisableMouseCapture,
            crossterm::event::DisableFocusChange,
            LeaveAlternateScreen
        );
        let _ = disable_raw_mode();
        // The embedded daemon dies with the process: no stale discovery file.
        crate::daemon_link::remove_published_port_files();

        let report = format!(
            "gitpoulpi-tui panicked.

{info}

backtrace:
{}
",
            std::backtrace::Backtrace::force_capture()
        );
        tracing::error!(target: "panic", "{info}");
        if let Some(log) = gitpoulpi_log::global() {
            log.flush();
        }
        let path = panic_report_path();
        match std::fs::write(&path, &report) {
            Ok(()) => eprintln!(
                "\ngitpoulpi-tui: crash report written to {}", // i18n-english,
                path.display()
            ),
            Err(err) => eprintln!(
                "\ngitpoulpi-tui: could not write {}: {err}", // i18n-english,
                path.display()
            ),
        }
        previous(info);
    }));
}

/// Where the crash report goes: beside the config, and the temp directory
/// if that cannot be resolved.
pub(crate) fn panic_report_path() -> std::path::PathBuf {
    let dir = gitpoulpi_config::ConfigPaths::discover()
        .map(|paths| paths.config_dir().to_path_buf())
        .unwrap_or_else(std::env::temp_dir);
    let _ = std::fs::create_dir_all(&dir);
    dir.join("gitpoulpi-tui-panic.txt")
}

/// What `run_in_repo` wants the outer loop to do next.
///
/// Three outcomes rather than `Option<RepoRef>`: "there is no repository any
/// more" and "the user quit" are different, and conflating them left the
/// window showing a closed repository under a `no repository open` tab bar.
pub(crate) enum NextWindow {
    Quit,
    Repository(RepoRef),
    /// Every tab is closed. The window stays open but must not pretend to show
    /// a repository, so it comes back as the Repository Management tab, the
    /// same shape as a TUI started outside one.
    None,
    /// The user went to the Repository Management tab (the `⌂`,
    /// `Ctrl+O`, a palette entry), possibly asking for one of its
    /// dialogs at the same time.
    Management(ManagementEntry),
}

/// One line of the open-path trace (`GITPOULPI_TIMING=1`, written to the
/// timing log file, never the terminal): what happened and how long after
/// the window began opening. See `docs/developer/open-time.md`.
fn open_trace(what: &str, since: Instant) {
    gitpoulpi_ipc::timing::record("tui-open", || {
        format!("{what} (+{})", gitpoulpi_ipc::timing::ms(since.elapsed()))
    });
}

/// A UI-thread step long enough to be felt (input is read on this thread,
/// so it is a keystroke left waiting). Logged under `GITPOULPI_TIMING`.
pub(crate) const STALL: std::time::Duration = std::time::Duration::from_millis(40);

pub(crate) fn stall_trace(what: &str, since: Instant) {
    let took = since.elapsed();
    if took >= STALL {
        gitpoulpi_ipc::timing::record("tui-stall", || {
            format!("{what} {}", gitpoulpi_ipc::timing::ms(took))
        });
    }
}

/// One repository's worth of TUI. Returns the repository to switch to,
/// or `None` when the user quit.
pub(crate) fn run_in_repo(
    session: &str,
    repo: RepoRef,
    start_dir: &std::path::Path,
    entry: ManagementEntry,
) -> io::Result<NextWindow> {
    let mut window = init::init(session, &repo, start_dir, entry)?;
    open_trace("setup done, entering the frame loop", window.open_t0);
    let mut frame: Option<window::FrameInputs> = None;
    loop {
        window.wait_for_work()?;
        let draws = window.draws_before_input();
        let t = Instant::now();
        let inputs = match frame.take() {
            Some(held) if !draws => held,
            _ => window.tick(),
        };
        stall_trace("tick", t);
        if draws {
            let t = Instant::now();
            window.draw_frame(&inputs)?;
            stall_trace("draw", t);
        }
        let t = Instant::now();
        window.drain_input(&inputs)?;
        // While something animates the drain's first `poll` waits one frame
        // for input (the spinner's pacing). An expired wait handled no input
        // and is not a stalled UI thread: any keystroke ends it at once.
        if window.redraw.input_handled {
            stall_trace("input", t);
        }
        if window.after_input() {
            break;
        }
        let t = Instant::now();
        window.apply_outcomes();
        stall_trace("apply_outcomes", t);
        window.end_iteration();
        frame = Some(inputs);
    }
    window.finish()
}

/// Which way an Undo / Redo toolbar button runs.
fn undo_redo_direction(button: ToolbarButton) -> gitpoulpi_core::domain::Direction {
    if button == ToolbarButton::Undo {
        gitpoulpi_core::domain::Direction::Undo
    } else {
        gitpoulpi_core::domain::Direction::Redo
    }
}

/// The LFS rows of a file's menu (they are not run by
/// `apply_file_context_action`, which has no remote to lock against).
pub(crate) fn is_lfs_file_action(action: FileContextAction) -> bool {
    matches!(
        action,
        FileContextAction::LfsTrackFile
            | FileContextAction::LfsTrackExtension
            | FileContextAction::LfsLock
            | FileContextAction::LfsUnlock
            | FileContextAction::LfsForceUnlock
    )
}

/// Runs one LFS row of a file's menu (or its palette twin) on `path`.
pub(crate) fn apply_lfs_file_action(
    action: FileContextAction,
    path: &std::path::Path,
    remote: Option<String>,
    link: &DaemonLink,
    input_mode: &mut InputMode,
    last_message: &mut Option<String>,
) {
    let need_remote = |last_message: &mut Option<String>| -> Option<String> {
        if remote.is_none() {
            *last_message = Some(gitpoulpi_i18n::msg::input::NO_DEFAULT_REMOTE.get().into());
        }
        remote.clone()
    };
    match action {
        FileContextAction::LfsTrackFile => {
            // Also re-stage the file if it is already committed, so its
            // next commit stores a pointer.
            link.submit_action(Action::LfsTrack {
                patterns: vec![lfs_ui::pattern_for_file(path)],
                opts: gitpoulpi_core::domain::LfsTrackOptions {
                    lockable: false,
                    renormalize: true,
                },
            });
            *last_message = Some(gitpoulpi_i18n::msg::input::lfs_tracking(path.display()));
        }
        FileContextAction::LfsTrackExtension => match lfs_ui::extension_pattern(path) {
            Some(pattern) => {
                link.submit_action(Action::LfsTrack {
                    patterns: vec![pattern.clone()],
                    opts: gitpoulpi_core::domain::LfsTrackOptions {
                        lockable: false,
                        renormalize: true,
                    },
                });
                *last_message = Some(gitpoulpi_i18n::msg::input::lfs_tracking(&pattern));
            }
            None => {
                *last_message = Some(gitpoulpi_i18n::msg::lfs_status::NO_EXTENSION.get().into())
            }
        },
        FileContextAction::LfsLock => {
            if let Some(remote) = need_remote(last_message) {
                link.submit_action(Action::LfsLock {
                    remote,
                    path: path.to_path_buf(),
                });
                *last_message = Some(gitpoulpi_i18n::msg::lfs_status::locking(path.display()));
            }
        }
        FileContextAction::LfsUnlock => {
            if let Some(remote) = need_remote(last_message) {
                link.submit_action(Action::LfsUnlock {
                    remote,
                    path: path.to_path_buf(),
                    force: false,
                });
                *last_message = Some(gitpoulpi_i18n::msg::lfs_status::unlocking(path.display()));
            }
        }
        // Breaking somebody else's lock is confirmed first.
        FileContextAction::LfsForceUnlock => {
            if let Some(remote) = need_remote(last_message) {
                *input_mode = InputMode::Confirm {
                    prompt: gitpoulpi_i18n::msg::lfs_status::break_lock(path.display()),
                    action: ConfirmAction::Destructive(PendingDestructive::LfsForceUnlock {
                        remote,
                        path: path.to_path_buf(),
                    }),
                };
            }
        }
        _ => {}
    }
}

/// Opens the git-lfs install page (the banner's click, and the palette's
/// `LFS: Open install page`) and says what happened on the status line.
pub(crate) fn open_lfs_install_page() -> String {
    match repo_menu::open_url(gitpoulpi_core::domain::LFS_INSTALL_URL) {
        Ok(()) => gitpoulpi_i18n::msg::lfs_status::opened(gitpoulpi_core::domain::LFS_INSTALL_URL),
        Err(e) => crate::activity_log::report_problem_clear(
            crate::activity_log::ProblemSource::Tool,
            &gitpoulpi_i18n::msg::lfs_status::install_hint(
                e,
                gitpoulpi_core::domain::LFS_INSTALL_URL,
            ),
        ),
    }
}

/// Re-reads `config.toml` and republishes the settings the TUI acts on, so a
/// Preferences change applies to the running session without a restart.
///
/// Exactly the keys in `gitpoulpi_config::LIVE_KEYS` that can change without
/// tearing something down. Log paging settings come back to the caller as a
/// `LogLoadSettings` because they belong to one `graph::LazyLog` per window;
/// the window's own flags (`ui.ref_tree`, `ui.graph_show_stashes`,
/// `ui.ref_connector`, `ui.avatars`) go through the `&mut` parameters.
///
/// `general.default_backend` is deliberately not here: a live daemon's
/// backend cannot be swapped under its subscribed clients.
/// `ui.mouse_enabled` is not either: capture mode is entered once, outside
/// the loop. Both say so in their field's help text.
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_live_settings(
    last_message: &mut Option<String>,
    hierarchical_refs: &mut bool,
    graph_show_stashes: &mut bool,
    ref_connector: &mut bool,
    avatar_placement: &mut gitpoulpi_config::AvatarPlacement,
    avatar_live: &mut avatar::LiveSettings,
    lane_background: &mut graph::LaneBackground,
    graph_theme: &mut crate::graph::LaneTheme,
    tint_strength: &mut gitpoulpi_config::GraphRowTintStrength,
) -> Option<graph::LogLoadSettings> {
    match gitpoulpi_config::Settings::resolve_ambient() {
        Ok(settings) => {
            let config = settings.config();
            daemon_link::set_log_page_limit(config.general.initial_commits_in_graph_clamped());
            live_settings::install_language(&config.ui);
            live_settings::install_date_patterns(&config.ui);
            live_settings::install_auto_prune(config.general.auto_prune);
            live_settings::install_auto_prune_tags(config.general.auto_prune_tags);
            crate::build_features::install_radar_enabled(config.general.radar_enabled);
            live_settings::install_minimap(config.ui.minimap);
            live_settings::install_column_resize(config.ui.column_resize);
            live_settings::install_log_columns_layout(config.ui.log_columns_layout);
            live_settings::install_chrome_layout(&config.ui);
            live_settings::install_mermaid(&config.ui);
            live_settings::install_blame(&config.ui);
            live_settings::install_image_view(&config.ui);
            crate::tab_dir::install(&config.ui);
            crate::image_editor::install(&config.ui);
            live_settings::install_log_wip_counts_in_message(config.ui.log_wip_counts_in_message);
            live_settings::install_reveal_on_ref_click(config.ui.reveal_on_ref_click);
            live_settings::install_sidebar_sections_global(
                config.ui.sidebar_sections_scope == gitpoulpi_config::SidebarSectionsScope::Global,
            );
            live_settings::install_graph_ghost_ref(config.ui.graph_ghost_ref);
            live_settings::install_hide_finished_pr_chip(config.ui.hide_finished_pr_chip);
            live_settings::install_group_worktree_tabs(config.general.group_worktree_tabs);
            crate::editor_prefs::install(&config.editor);
            crate::merge_view::layout::install(&config.merge);
            install_input_color(&config.ui.input_color, &config.ui.input_color_custom);
            install_theme_from_config(config.ui.theme_slug());
            crate::color_depth::install(config.ui.color_depth);
            lfs_ui::install_default(config.git.lfs_enabled);
            live_settings::install_delete_orig_after_merge(
                config.general.delete_orig_files_after_merge,
            );
            live_settings::install_active_profile(config);
            live_settings::install_commit_count(config.log.commit_count);
            crate::about::install(&config.ui);
            crate::launcher_sync::install(config);
            external_tools::install(config);
            if let Some(message) = keymap::issues_message(&keymap::install_from_config(config)) {
                *last_message = Some(message);
            }
            *hierarchical_refs = config.ui.ref_tree;
            *graph_show_stashes = config.ui.graph_show_stashes;
            *ref_connector = config.ui.ref_connector;
            *avatar_placement = config.ui.avatar_placement();
            *avatar_live = avatar::LiveSettings::from_config(config);
            *lane_background = config.ui.graph_lane_background.into();
            *graph_theme = graph::LaneTheme::from_slug(config.ui.graph_color_slug());
            *tint_strength = config.ui.graph_row_tint_strength;
            Some(graph::LogLoadSettings::from_config(&config.general))
        }
        Err(err) => {
            *last_message = Some(gitpoulpi_i18n::msg::lfs_status::config_reread(err));
            None
        }
    }
}

/// The square pixel box every decoded avatar is resampled to for a
/// terminal with `cell` pixels per cell: the larger of the commit
/// detail's author box and the graph node's picture
/// ([`graph::avatar_picture_px`]), so one decode serves both and is 1:1
/// with the screen at any zoom.
fn avatar_box_px(cell: terminal_image::CellSize) -> u32 {
    let (cols, rows) = avatar::detail_size(cell);
    ((u32::from(cols) * cell.width).max(u32::from(rows) * cell.height)
        * terminal_image::supersample())
    .max(graph::avatar_picture_px(
        cell,
        terminal_image::supersample(),
    ))
}
