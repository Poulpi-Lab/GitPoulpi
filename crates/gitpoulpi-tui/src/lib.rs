//! The GitPoulpi terminal UI: a `gitpoulpi-ipc` client of the session's
//! `gitpoulpi-daemon` (started in-process when none is running), never a
//! direct caller of `GitBackend`.
//!
//! Layout: a toolbar row, three resizable panes, a breadcrumb footer, a help
//! overlay (`?`) and a command palette (`Ctrl+P`). The panes are the Refs
//! sidebar (LOCAL/REMOTE/TAGS sections with a name filter), the Log (a
//! per-branch colored commit graph, ref pills, and a pinned WIP row for
//! uncommitted work) and a right panel showing either the Status pane or the
//! selected commit's details with its own changed-files list.
//!
//! Keyboard: `Tab` cycles pane focus, `j`/`k`/arrows move, `h`/`l` switch the
//! Refs section or leave/enter a commit's changed-files list. Git actions
//! live behind the `Ctrl+G` leader prefix (see the `InputMode::Leader` arm);
//! fetch/pull/push run on a task inside `daemon_link` so network I/O never
//! stalls status polling.
//!
//! Mouse: click selects and focuses, the wheel scrolls, dragging near a pane
//! divider resizes it (min `MIN_PANE_PERCENT`), double-click on a ref checks
//! it out. Every clickable thing goes through one `HitTarget` /
//! `find_hit_target` mechanism. Log columns are drag-resizable,
//! drag-reorderable and hideable from the header menu (`Message` is always
//! shown). Hovering a Log row shows a per-column tooltip.
//!
//! Log ref pills and the Refs sidebar:
//! - "Checked out" is `Ref::is_head`, the backend's own fact, not inferred
//!   from log position.
//! - A local branch and its remote-tracking counterpart on the same commit
//!   render as one combined pill (`ref_pills_for_commit`; the blank cell
//!   between the two icons is explained at `ref_pill_spans`).
//! - Selection highlighting everywhere is the fixed muted `SELECTION_BG`, not
//!   `Modifier::REVERSED`, which turns a light-on-dark row stark white.
//! - A ref hidden from the graph (`hidden_from_graph`) stays dimmed in the
//!   sidebar but drops out of the pills and the graph's glyph choice.
//! - The "other refs on this commit" popup (`RefPopupState`) is a menu, not a
//!   tooltip: anchored under its row once, never to the live mouse position,
//!   and it stays open while the pointer is over the row or the popup. A
//!   keyboard-opened popup ignores the mouse.
//!
//! The sidebar's LOCAL/REMOTE sections can show names as a folder hierarchy
//! (`y`, `ui.ref_tree`; Tags never do). `ref_display_rows` is the one
//! function the render pass and every Up/Down/click/Enter site call for a
//! section's rows, so the row count behind `ListState` bounds cannot drift
//! from what is drawn.
//!
//! Live updates: the primary refresh trigger is the daemon pushing a
//! `RepoChanged` event, so a mutation from any client shows up without
//! waiting; a 2s poll stays as a safety net. See
//! `docs/architecture/session-and-state.md`.
//!
//! The commit graph lives in the `graph` module (layout, `TreePart`
//! vocabulary, every glyph); see `docs/developer/commit-graph.md` and
//! `graph::part`'s header before changing a marker. It walks history from
//! every ref (`LogOptions::all_refs`), and lane transitions render inside the
//! transitioning commit's own row, so the one-row-per-commit contract holds.
//! A freshly claimed lane index always gets a fresh palette color rather
//! than inheriting the color of whatever branch freed the slot.
//!
//! `LogColumnWidths` sizes the visible Log columns from the pane's width every
//! frame (protecting SHA/date/ref first, Message shrinks hardest, manual
//! widths win), and every cell that can overflow is truncated with an
//! ellipsis (`truncate_with_ellipsis`) because ratatui's own clipping shows
//! no truncation indicator.

#[cfg(all(
    feature = "hosting",
    not(any(
        feature = "github",
        feature = "gitlab",
        feature = "azure-devops",
        feature = "bitbucket",
        feature = "jira",
        feature = "trello",
        feature = "gitea",
        feature = "linear",
        feature = "youtrack",
        feature = "redmine",
        feature = "gerrit",
        feature = "local-issues"
    ))
))]
compile_error!(
    "the internal `hosting` feature needs a provider (`github`, `gitlab`, `azure-devops`, `bitbucket`, `jira`, `trello`, `gitea`, `linear`, `youtrack`, `redmine`, `gerrit` or `local-issues`)" // i18n-english
);

/// The hide-in-graph eye, as a literal so the context menu's labels can
/// `concat!` it: the menu entry and the Refs sidebar's gutter mark are one
/// glyph by construction.
///
/// Defined before the `mod` declarations below: a `macro_rules!` macro is
/// only in scope textually after its definition.
macro_rules! hide_in_graph_icon {
    // U+2299 (CIRCLED DOT OPERATOR): a plain text symbol with no emoji
    // presentation, so every terminal draws it from a text font. The eye
    // (U+1F441 + VS15) it replaced is emoji-capable: kitty and WezTerm drew it
    // with the colour emoji font whatever VS15 asked. Pairs with the hidden
    // mark U+2298 (circled slash).
    () => {
        "\u{2299}"
    };
}

mod about;
mod activity;
mod activity_log;
mod after_clone;
#[cfg_attr(not(feature = "ai"), path = "ai_limit_dialog_stub.rs")]
mod ai_limit_dialog;
#[cfg_attr(not(feature = "ai"), path = "ai_sent_dialog_stub.rs")]
mod ai_sent_dialog;
#[cfg_attr(not(feature = "ai"), path = "ai_ui_stub.rs")]
mod ai_ui;
mod avatar;
mod blame_view;
mod branch_remotes_dialog;
pub mod build_features;
#[cfg(test)]
mod button_audit_tests;
#[cfg(test)]
mod cjk_sweep_tests;
mod clipboard;
mod clipboard_route;
mod clone_dialog;
mod clone_task;
mod color_depth;
mod column_drag;
mod column_resize;
mod combo;
mod commit_column;
mod commit_settings;
mod conflict_badge;
mod conflict_popup;
mod context_menu;
mod cursor;
mod daemon_link;
pub mod desktop_shortcuts;
mod dialog_kit;
mod diff_content;
mod diff_encoding_menu;
mod diff_recode;
mod diff_view;
mod dir_picker;
mod editor;
mod editor_buffer;
mod editor_charset;
mod editor_highlight;
mod editor_prefs;
mod external_tools;
#[cfg(any(all(feature = "hosting", feature = "review"), feature = "ai"))]
mod file_browse;
mod file_selection;
mod file_view;
mod flow;
mod fold_mark;
mod fold_state;
mod folder_listing;
mod folder_picker;
#[cfg(test)]
mod footer_guard_tests;
mod graph;
mod guide_dialog;
mod help_guide;
mod help_view;
mod history_view;
mod hosting_notify;
#[cfg(feature = "hosting")]
mod hosting_task;
#[cfg(test)]
mod i18n_layout_tests;
mod image_decode;
#[cfg_attr(not(feature = "image-view"), path = "image_view_stub.rs")]
mod image_view;
mod init_dialog;
mod init_task;
#[cfg_attr(not(feature = "hosting"), path = "integrations_page_stub.rs")]
mod integrations_page;
#[cfg(feature = "hosting")]
mod integrations_ssh;
mod jobs_ui;
mod keymap;
mod kitty_keyboard;
mod launcher;
mod launcher_sync;
mod lfs_ui;
mod live_settings;
mod log_search;
mod logo_colors;
mod merge_flow;
mod merge_view;
mod minimap;
mod new_file;
mod off_branch_edit;
mod palette;
mod pane_menu;
mod paste_burst;
mod pointer_map;
mod popup_list;
mod preferences;
mod quiet_backend;
mod redraw;
mod refs_tree;
mod relaunch;
#[cfg(test)]
mod render_check_tests;
mod repo_manager;
mod repo_menu;
mod repo_probe;
mod repo_switcher;
mod safety_panel;
mod scroll_marks;
mod scrollbar;
#[cfg(test)]
mod scrollbar_guard_tests;
mod scrollbar_repeat;
mod sections_scope;
mod session_store;
mod sidebar_layout;
#[cfg(test)]
mod source_scan;
mod sparse_dialog;
pub mod standalone;
mod startup_probe;
mod startup_tab;
mod status_optimistic;
mod status_pane;
mod submodules_ui;
mod term_modes;
mod text_cursor;
mod title_logo;
mod ui_intent;
mod ui_viewer;
mod undo_ui;
mod updates;
mod wide_edge;
mod window_ctl;
mod wip_view;
mod workspace_ui;
// Pixels on a terminal: capability detection, the cell-metrics query,
// the iTerm2 encoder and the buffer-cell widget. The commit graph is its
// first caller; it is a layer rather than part of the graph so author
// avatars can use the same detection, metrics and encoder.
mod terminal_bg;
mod terminal_image;
mod terminology;
mod text_edit;
mod text_field;
mod text_history;
mod text_select;
#[cfg(feature = "worktree")]
mod worktree_dialog;
#[cfg(not(feature = "worktree"))]
#[path = "worktree_dialog_stub.rs"]
mod worktree_dialog;
#[cfg(feature = "worktree")]
mod worktree_remove_dialog;

// The main window: the event loop (`event_loop::run_in_repo`) and the
// helpers it calls, grouped by pane or concern. Each module starts with
// `use super::*;` and is glob-imported back here, so the set shares one
// namespace and items cross module lines as `pub(crate)`. See
// `docs/developer/where-we-are.md` for the map.
mod autostash_banner;
mod binary_hint;
mod board_cmd;
#[cfg(feature = "hosting")]
#[cfg_attr(not(feature = "board"), path = "board_view_stub.rs")]
mod board_view;
mod button_bar;
mod cell_text;
mod chrome;
mod commit_detail;
#[cfg(test)]
mod commit_files_bench;
mod commit_files_cache;
mod compare;
mod compare_cards;
mod compose_ui;
mod composer;
mod composer_cmd;
#[cfg(all(feature = "hosting", feature = "conversation"))]
mod conversation_ui;
mod diff_actions;
#[cfg(test)]
mod docs_tables;
mod event_loop;
mod exit_log;
mod file_panel;
mod file_rows;
mod git_actions;
#[cfg_attr(not(feature = "hosting"), path = "hosting_ui_stub.rs")]
mod hosting_ui;
mod image_editor;
#[cfg(feature = "hosting")]
mod issue_form;
#[cfg(feature = "hosting")]
mod issue_ref_picker;
#[cfg_attr(not(feature = "hosting"), path = "issue_tokens_stub.rs")]
mod issue_tokens;
mod item_cmd;
#[cfg(feature = "hosting")]
#[cfg_attr(not(feature = "conversation"), path = "item_view_stub.rs")]
mod item_view;
mod language_picker;
mod link_confirm;
mod log_inline_ref;
mod log_menu;
mod log_multiselect;
mod log_pills;
mod log_table;
mod md_editor;
mod md_render;
#[cfg_attr(not(feature = "mermaid"), path = "mermaid_popup_stub.rs")]
mod mermaid_popup;
mod message_links;
#[cfg(feature = "hosting")]
mod meta_picker;
mod modes;
mod multi_remote_badge;
mod open_in_bar;
#[cfg(test)]
mod open_time_bench;
mod open_with;
mod operation_banner;
mod pane_layout;
mod pane_split;
mod picker_view;
mod pill_menu;
mod profile_bar;
mod prompt_dialog;
#[cfg_attr(not(feature = "hosting"), path = "pull_request_stub.rs")]
mod pull_request;
mod push_group_ui;
#[cfg_attr(
    not(all(feature = "radar", feature = "hosting")),
    path = "radar_stub.rs"
)]
mod radar;
mod rebase_banner;
mod rebase_ui;
mod ref_drop;
mod ref_menu;
mod ref_name_box;
mod ref_reveal;
mod refs_sidebar;
mod remote_links;
mod remotes_ui;
mod rename_pairs;
mod repo_dialogs;
mod revert_mainline;
mod review_cmd;
#[cfg(feature = "hosting")]
#[cfg_attr(not(feature = "review"), path = "review_view_stub.rs")]
mod review_view;
mod reword_ui;
mod rewrite_view;
mod sections_menu;
mod sent_pane;
mod split_ui;
mod stage_guard_dialog;
mod status_list;
mod stuck_banner;
mod tab_column;
mod tab_dir;
mod tab_drag;
mod tab_scroll;
mod tabs;
mod tag_delete_dialog;
mod tag_where_ui;
#[cfg(test)]
mod test_support;
mod theme;
mod theme_picker;
mod theme_watch;
mod tooltip;
mod tree_fold;
#[cfg_attr(not(feature = "hosting"), path = "trust_dialog_stub.rs")]
mod trust_dialog;
mod ui_kit;
#[cfg_attr(not(feature = "global-search"), path = "workspace_search_stub.rs")]
mod workspace_search;

use ai_ui::{AiAnswer, AiOutcome, AiRequest, AiUi, ExplainTarget, PopupKey};
use autostash_banner::AutoStashBannerButton;
use button_bar::*;
use chrome::*;
use commit_detail::*;
use compose_ui::{compose_hit_at, ComposeForm, ComposeOutcome};
use composer::*;
use diff_actions::*;
use event_loop::*;
use file_panel::*;
use file_selection::*;
use git_actions::*;
use hosting_ui::{HostingCommand, HostingEffect, HostingHit, HostingList, HostingUi, ToolbarChip};
use log_inline_ref::*;
use log_menu::*;
use log_multiselect::*;
use log_pills::*;
use log_table::*;
use modes::*;
use pane_layout::*;
use pane_menu::*;
use pill_menu::*;
use pull_request::*;
use rebase_banner::*;
use rebase_ui::*;
use ref_drop::*;
use ref_menu::*;
use ref_name_box::*;
use refs_sidebar::*;
use remote_links::*;
use remotes_ui::*;
use repo_dialogs::*;
use reword_ui::*;
use split_ui::{SplitForm, SplitFormOutcome};
use status_list::*;
use submodules_ui::*;
use tabs::*;
use theme::*;
use ui_kit::*;

use context_menu::{Menu as ContextMenu, MenuOutcome};
use cursor::CursorShape;
use diff_view::{DiffBarButton, DiffSource, DiffStageAction, DiffViewState, DiffViewToggle};
use log_search::{LogSearch, SearchBarButton};
use palette::{palette_matches, PaletteCommand, PaletteContext};
use preferences::PreferencesState;
use refs_tree::{ref_display_rows, CollapsedFolders, RefRowKind, RefTreeRow};
use scrollbar::ScrollBar;
use status_pane::{
    all_folders, build_tree_rows, discard_confirmation_prompt, entry_status, file_context_entries,
    section_paths, status_marker, unstaged_paths_under, FileContextAction, FileListMode,
    FolderContextAction, StatusSection, TreeRow,
};
use worktree_dialog::WorktreeSeed;

use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::wide_edge::Clear;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
    KeyModifiers, MouseButton, MouseEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use daemon_link::{Action, DaemonLink, FileInsightKind, FileInsightRequest};
use file_view::{insight_rows, FileInsight};
use gitpoulpi_core::domain::{
    CommitNode, HunkId, LineSelector, Oid, RefKind, RepoId, RepoRef, TagSelection, WorkingTree,
};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::*;
use ratatui::widgets::{
    Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, TableState,
};

/// Runs the TUI for the given named session.
///
/// A session holds **N repositories** (ADR-0006) and one window shows one at
/// a time. Switching restarts the inner loop instead of re-subscribing:
/// `run_in_repo` owns a `DaemonLink` bound to one `RepoRef` for its whole
/// life, and rebuilding it is far cheaper than making every piece of
/// per-repo state swappable in place. The daemon is the same process
/// throughout (it lives on the process-lifetime runtime, see
/// `daemon_link::embedded_daemon_runtime`), so a switch costs one reconnect.
///
/// The tab list and the recent-repositories list are read and written
/// through `gitpoulpi-session` around that loop, which is why they survive
/// the switch.
pub fn run(session: &str) -> io::Result<()> {
    run_with(session, RunOptions::default())
}

/// What the command line hands the TUI besides the session.
#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    /// `--cwd-file`: written with the active tab's folder when the TUI quits
    /// ("Quit and cd here", or every quit under `ui.tab_dir_follow =
    /// "auto"`), for the wrapper `gitpoulpi shell-init` prints.
    pub cwd_file: Option<std::path::PathBuf>,
    /// `--repo <path>`: the repository this launch opens, instead of the
    /// current directory's. Handled exactly like a launch from that folder
    /// (made the active tab, merged with the remembered tabs); a path that
    /// is not a repository opens Home with a message.
    pub repo: Option<std::path::PathBuf>,
    /// `--launcher`: set by the desktop / Start menu shortcuts GitPoulpi
    /// writes; chooses the tab with `general.startup_tab_launcher`.
    pub launcher: bool,
}

/// Windows: the folder File Explorer's address bar started this program in
/// (typing `gitpoulpi` or the alias there), when this is such a start and the
/// user gave no argument; `None` everywhere else. Call it first thing in
/// `main`, before any argument handling: it also clears the shim's
/// `GITPOULPI_CMDCMDLINE` so that no child inherits it. The caller opens the
/// folder like `--repo <folder>` (or `--add <folder>` with
/// `ui.reuse_window`), and the terminal relaunch carries it along.
pub fn explorer_folder() -> Option<std::path::PathBuf> {
    #[cfg(windows)]
    {
        relaunch::explorer_folder()
    }
    #[cfg(not(windows))]
    {
        None
    }
}

/// [`run`] with the command-line options.
pub fn run_with(session: &str, options: RunOptions) -> io::Result<()> {
    // Every request this process sends says it comes from the window.
    gitpoulpi_ipc::auth::set_client_name("TUI");
    // User-only app directories (best effort, once per start).
    if let Some(paths) = gitpoulpi_config::ConfigPaths::discover() {
        gitpoulpi_config::secure_fs::harden_app_dirs(&paths);
    }
    // Windows: a start nobody typed (double-click, a pinned exe, a pinned
    // shortcut from before `--launcher`) is a launcher start.
    #[cfg(windows)]
    let options = RunOptions {
        launcher: relaunch::infer_launcher(
            options.launcher,
            std::env::var_os(relaunch::RELAUNCHED_ENV).is_some(),
            relaunch::console_processes(),
        ),
        ..options
    };
    // Windows double-click: reopen in the terminal `ui.launch_terminal` names.
    #[cfg(windows)]
    if relaunch::try_relaunch(options.repo.as_deref()) {
        return Ok(());
    }
    tab_dir::set_cwd_file(options.cwd_file);
    // The application log (the Activity Log's second tab); an embedded
    // daemon's `tracing` lines land in it too.
    gitpoulpi_log::init_from_settings("tui");
    activity_log::install_timings_from_settings();
    tracing::info!(session, "gitpoulpi-tui starting");
    // Windows: put this terminal window under the launcher's taskbar icon.
    window_ctl::group_in_background(options.launcher);
    install_panic_hook();
    // The UI runs on a thread of its own with a generous stack rather than on
    // `main`'s (1 MiB on Windows): `run_in_repo` holds a few hundred locals and
    // a draw closure capturing them all, and a stack overflow is the one way
    // this process can die without the panic hook or the exit log hearing of
    // it. The reservation is virtual. A panic on that thread comes back as a
    // join error and is recorded like any other way out.
    let owned = session.to_string();
    let repo_flag = options.repo;
    let launcher = options.launcher;
    let joined = std::thread::Builder::new()
        .name("gitpoulpi-ui".into())
        .stack_size(UI_THREAD_STACK)
        .spawn(move || run_windows(&owned, repo_flag, launcher))?
        .join();
    // The embedded daemon dies with this process: take its discovery files
    // down on every way out (quit, error, panic of the UI thread), before a
    // restart into the new binary could find them.
    daemon_link::remove_published_port_files();
    // The read-only copies "Edit in" handed to image editors.
    image_editor::clean_temp();
    let result = match joined {
        Ok(Ok(())) => {
            // "Update and restart" / "Revert": the job is written and the
            // terminal is back to normal; the updater takes over from here.
            updates::auto::launch_if_requested();
            return Ok(());
        }
        Ok(Err(err)) => {
            exit_log::record_exit(
                &exit_log::ExitReason::from_error(&err),
                session,
                &exit_log::recent_inputs(),
            );
            err
        }
        Err(payload) => {
            let message = payload
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "unknown panic payload".into()); // i18n-english
            exit_log::record_exit(
                &exit_log::ExitReason::Panic {
                    message: message.clone(),
                },
                session,
                &exit_log::recent_inputs(),
            );
            io::Error::other(format!("the UI thread panicked: {message}")) // i18n-english
        }
    };
    // An error left the loop through `?`, skipping its own teardown: put
    // the terminal back first, so what follows is readable, then keep it
    // on screen.
    exit_log::restore_terminal();
    exit_log::explain_error_and_pause(&result);
    Err(result)
}

/// See `run`: the UI thread's stack reservation.
const UI_THREAD_STACK: usize = 64 * 1024 * 1024;

/// The loop over repository windows - `run`'s body, on the UI thread.
fn run_windows(
    session: &str,
    repo_flag: Option<std::path::PathBuf>,
    launcher: bool,
) -> io::Result<()> {
    let process_cwd = std::env::current_dir()?;
    // `--repo <path>` stands in for the folder the TUI was launched from.
    let cwd = match &repo_flag {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => process_cwd.join(path),
        None => process_cwd,
    };
    if repo_flag.is_some() {
        let usable = cwd.is_dir()
            && session_store::is_repository(&find_repo_root(&cwd).unwrap_or(cwd.clone()));
        if !usable {
            tab_dir::set_launch_notice(gitpoulpi_i18n::msg::tab_bar::repo_not_repository(
                cwd.display(),
            ));
        }
    }

    // `general.remember_tabs` and the three `general.startup_tab_*` settings
    // are read once, here: they decide what a *launch* opens, not something
    // a running window's Preferences edit should retroactively undo. See
    // `session_store::restore_launch_tabs`: the remembered tabs live in the
    // last-used session, not necessarily this launch folder's.
    let config = gitpoulpi_config::Settings::resolve_ambient()
        .map(|s| s.config().clone())
        .unwrap_or_default();
    let remember = config.general.remember_tabs;
    let implicit_session = session == gitpoulpi_config::default_session_name(&cwd);
    tab_dir::set_launch_dir(&cwd, find_repo_root(&cwd).as_deref());

    let launch_root = {
        let root = find_repo_root(&cwd).unwrap_or(cwd.clone());
        session_store::is_repository(&root).then_some(root)
    };
    // The starting repository is recorded as an open tab, `open_tab` refusing
    // a path that is not a repository (which keeps a home directory out of
    // the recent list). It is only made the active one when the start opens
    // on it; otherwise the remembered active tab stays active.
    session_store::restore_launch_tabs(
        session,
        launch_root.as_deref(),
        remember,
        implicit_session,
        false,
    );
    let radar_offered = build_features::Feature::Radar.built() && config.general.radar_enabled;
    let facts = startup_tab::LaunchFacts {
        in_repo: launch_root,
        explicit_repo: repo_flag.is_some(),
        launcher,
        remember,
        last: session_store::active_tab_path(session)
            .filter(|path| session_store::is_repository(path)),
        radar_offered,
    };
    let start = startup_tab::decide(&facts, &config.general);
    let mut entry = ManagementEntry::Show;
    let mut repo = match start {
        startup_tab::Start::Repo(path) => {
            // Activates the tab (an existing one included).
            session_store::open_tab(session, &path, None);
            repo_ref_for(path)
        }
        startup_tab::Start::Home => no_repository_ref(),
        startup_tab::Start::Radar => {
            // Opening it brings the tab back when it was closed.
            tabs::reopen_radar_tab(session);
            entry = ManagementEntry::Radar;
            no_repository_ref()
        }
    };

    // The management tab *is* the no-repository window; `entry` says
    // whether it should also open its Browse / Clone / Init dialog (or the
    // Radar) as it arrives (the second half of "clone" asked from a
    // repository tab).
    loop {
        match run_in_repo(session, repo.clone(), &cwd, entry)? {
            NextWindow::Quit => {
                // "Quit and cd here": leave the active tab's folder for the
                // shell wrapper. A failure is shown, not swallowed.
                if repo.root.exists() && session_store::is_repository(&repo.root) {
                    if let Err(err) = tab_dir::write_cwd_on_quit(&repo.root) {
                        let what = "gitpoulpi: could not write --cwd-file"; // i18n-english
                        eprintln!("{what}: {err}");
                    }
                }
                return Ok(());
            }
            NextWindow::Repository(next) => {
                repo = next;
                entry = ManagementEntry::Show;
            }
            NextWindow::None => {
                repo = no_repository_ref();
                entry = ManagementEntry::Show;
            }
            NextWindow::Management(next) => {
                repo = no_repository_ref();
                entry = next;
            }
        }
    }
}
