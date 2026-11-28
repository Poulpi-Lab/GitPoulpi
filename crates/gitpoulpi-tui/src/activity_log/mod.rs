//! Telling the user what went wrong, and keeping the record.
//!
//! - [`feed`]: the session's messages (failures with the facts the backend
//!   left, and the status messages worth keeping).
//! - [`status_error`]: a failed operation has no card; the status line words
//!   it in red and a click (or the palette) opens its entry.
//! - [`panel`]: the Activity Log window (status bar button, `View: Activity
//!   Log`), two tabs, **Git operations** (failures, messages and timings) and
//!   **Application** (the log of the window and the daemon, level box, filter,
//!   horizontal scroll), and a Scope box that narrows both to one repository,
//!   a workspace or everything.
//! - [`timing`]: how long operations take (`log.timings`, `log.extended`).
//! - [`report`]: the text the Copy buttons produce, made to be pasted to an AI.
//! - [`friendly`]: git's failures in plain words, and what could fix them (the
//!   remedy buttons of an entry: Pull first, Stash and retry, Retry, Open
//!   Integrations).
//!
//! The facts come from `gitpoulpi_core::ErrorDetail` (operation, redacted
//! command line, exit code, stderr tail, or libgit2's class and code); the
//! file side of the log is the `gitpoulpi-log` crate.

mod capabilities;
mod cmd;
mod feed;
pub(crate) mod friendly;
pub(crate) mod identity;
mod panel;
mod problem;
mod report;
pub(crate) mod status_error;
mod timing;

/// The window and the status error keep their state in statics: tests that
/// paint them take this lock.
#[cfg(test)]
pub(crate) static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

use crossterm::event::MouseEvent;
use ratatui::Frame;

use gitpoulpi_core::domain::PushRejection;
use gitpoulpi_core::Error as CoreError;

pub(crate) use panel::{
    clear_status_button, is_open, level_cmd_open as level_command, nav_press, nav_targets,
    open as open_panel, open_global as open_global_panel, paint_status_button, pointer_hand,
    pointer_text, set_home, status_button_width, LevelCmd,
};
pub(crate) use panel::{last_error_command, last_error_has_remedy};
#[cfg(feature = "hosting")]
pub(crate) use problem::remember_kind;
#[cfg_attr(not(feature = "hosting"), allow(unused_imports))]
pub(crate) use problem::report_as as report_problem_as;
pub(crate) use problem::{
    report as report_problem, report_and_clear as report_problem_clear,
    report_in as report_problem_in, Source as ProblemSource,
};
pub(crate) use timing::{
    background as timing_background, background_fetch_pending, background_fetch_skipped,
    expect_background_fetch, install as install_timings,
    install_from_settings as install_timings_from_settings, record as timing_record,
};

/// The details of the newest failure recorded (tests of launches read it).
#[cfg(test)]
pub(crate) fn newest_failure_details() -> Option<String> {
    feed::snapshot()
        .into_iter()
        .find(|e| e.severity == feed::Severity::Error)
        .map(|e| e.message)
}

/// What an entry's remedy button asks the window to do (the Activity Log
/// cannot reach the link).
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Command {
    Submit(Action),
    OpenIntegrations,
    /// Open this repository's SSH & credentials page.
    OpenRepoAuth,
    Message(String),
}

/// Copies the report of the failure the status line shows (the palette's
/// `Activity Log: Copy the Last Error`); the status line says what went to
/// the clipboard.
pub(crate) fn copy_last_error() -> Option<Command> {
    let entry = status_error::shown_entry()?;
    Some(Command::Message(crate::clipboard::copy_shared(
        gitpoulpi_i18n::msg::activity_log::WHAT_ERROR.get(),
        &report::entry_block(&entry),
    )))
}

use crate::daemon_link::Action;
use feed::{Entry, Severity};
use gitpoulpi_i18n::msg::activity_log as msg;

/// Whether the error is a failure worth a card (and the badge), as opposed
/// to a refusal the status line already words (nothing to undo, a dialog
/// that asks first).
fn is_failure(error: &CoreError) -> bool {
    matches!(
        error,
        CoreError::Backend(_)
            | CoreError::Git { .. }
            | CoreError::Io(_)
            | CoreError::NotFastForward
            | CoreError::Conflict { .. }
            | CoreError::DirtyWorkingTree { .. }
    )
}

/// Records an action that failed: an entry in the Activity Log, the status
/// line's error unless it was only a refusal, and a line in the application
/// log. Called from the link's worker thread.
pub(crate) fn report_failure(
    action: &Action,
    error: &CoreError,
    repo: &std::path::Path,
    elapsed: std::time::Duration,
    context: gitpoulpi_core::OpContext,
) {
    let kind = friendly::kind_of(error);
    let failure = is_failure(error);
    let id = feed::reserve_id();
    let context = complete_context(context, repo, id);
    let mut title = friendly::title_for(action);
    // An operation with no title of its own is named by what it was.
    if title == gitpoulpi_i18n::msg::git_error::TITLE_OTHER.get() {
        title = msg::failed_title(&context.label);
    }
    let mut entry = Entry::new(
        if failure {
            Severity::Error
        } else {
            Severity::Warning
        },
        title,
        gitpoulpi_core::redact(&error.to_string()),
    );
    entry.id = id;
    entry.context = Some(context.clone());
    entry.kind = kind;
    entry.detail = error.detail().cloned();
    entry.retry = Some(action.clone());
    entry.action_text = Some(friendly::action_text(action));
    let typed_repo_auth = friendly::is_repo_auth_error(error);
    entry.auth = friendly::auth_context(
        kind,
        typed_repo_auth,
        repo != std::path::Path::new(crate::tabs::NO_REPOSITORY),
        identity::get(repo).as_ref(),
    );
    let text = match error.detail() {
        Some(d) => format!("{}\n{error}", d.stderr),
        None => error.to_string(),
    };
    match friendly::wording(kind, &text) {
        Some((summary, hint)) => {
            entry.summary = Some(summary);
            entry.hint = Some(hint);
        }
        // Typed refusals have their own sentence (localised).
        None if error.typed().is_some() => {
            entry.summary = Some(crate::daemon_link::error_text::error_text(error));
            if typed_repo_auth {
                entry.hint = Some(
                    gitpoulpi_i18n::msg::git_error::HINT_REPO_AUTH
                        .get()
                        .to_string(),
                );
            }
            if kind == gitpoulpi_core::FailureKind::LocalChangesBlockAbort {
                entry.hint = Some(
                    gitpoulpi_i18n::msg::git_error::HINT_ABORT_BLOCKED
                        .get()
                        .to_string(),
                );
            }
        }
        None => {}
    }
    entry.repo = Some(repo.to_path_buf());
    entry.duration_ms = timing::action_finished(action, repo, ms(elapsed), false);
    let repo_text = repo_text(repo);
    tracing::warn!(target: "tui", repo = repo_text.as_str(), "{}", context.failure_line(&entry.message));
    if let Some(detail) = &entry.detail {
        tracing::debug!(target: "tui", repo = repo_text.as_str(), "{}", detail.lines().join(" | "));
    }
    report::warm_up();
    // No card: the status line words a failure (a refusal keeps the line the
    // link sends), and the entry keeps the details and the remedy buttons.
    let status_line = failure.then(|| {
        let reason = entry
            .summary
            .clone()
            .unwrap_or_else(|| entry.message.lines().next().unwrap_or("").to_string());
        format!("{}: {reason}", entry.title)
    });
    let id = feed::push(entry);
    if let Some(reason) = status_line {
        status_error::arm(
            id,
            reason,
            Some(crate::daemon_link::error_text::error_text(error)),
        );
    }
}

/// An action that went through: a line in the Activity tab (the status line
/// says it went through).
pub(crate) fn report_success(
    action: &Action,
    repo: &std::path::Path,
    elapsed: std::time::Duration,
) {
    let duration = timing::action_finished(action, repo, ms(elapsed), true);
    let signs_in = identity::line_for(repo).filter(|_| identity::uses_network(action));
    let Some(title) = friendly::done_title(action) else {
        // Not worth a card: the Repository tab gets the timing line.
        if let (Some(label), Some(ms)) = (timing::label_for(action), duration) {
            let mut entry = Entry::operation(repo, label, ms, false);
            if let Some(line) = signs_in {
                entry.message = line;
            }
            feed::push(entry);
        }
        return;
    };
    tracing::info!(target: "tui", repo = repo_text(repo).as_str(), "{title}");
    let message = match &signs_in {
        Some(line) => format!("{title}\n{line}"),
        None => title.clone(),
    };
    let mut entry = Entry::new(Severity::Info, title.clone(), message);
    entry.repo = Some(repo.to_path_buf());
    entry.duration_ms = duration;
    entry.list_title = timing::label_for(action).filter(|_| duration.is_some());
    feed::push(entry);
}

/// A repository for the `repo` field of a log line (what the Activity Log's
/// scope filters the Application tab by); the Home tab's placeholder has none.
pub(crate) fn repo_text(repo: &std::path::Path) -> String {
    if repo == std::path::Path::new(crate::tabs::NO_REPOSITORY) {
        String::new()
    } else {
        repo.to_string_lossy().into_owned()
    }
}

/// What this window adds to the context of a failure: the repository it ran
/// in and "TUI" as the asker when the daemon did not say, and an operation id
/// (the entry's own) when the failure did not come from a request.
pub(crate) fn complete_context(
    mut context: gitpoulpi_core::OpContext,
    repo: &std::path::Path,
    entry_id: u64,
) -> gitpoulpi_core::OpContext {
    if context.repo.is_none() {
        let text = repo_text(repo);
        context.repo = (!text.is_empty()).then_some(text);
    }
    if context.frontend.is_none() {
        context.frontend = Some("TUI".to_string());
    }
    if context.request == 0 {
        context.request = entry_id;
    }
    context
}

fn ms(elapsed: std::time::Duration) -> u64 {
    u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
}

/// A push the remote refused because it moved on: the dialog asks what to
/// do, so there is no status error, but the Activity Log keeps the facts.
pub(crate) fn report_rejection(
    action: &Action,
    rejection: &PushRejection,
    repo: &std::path::Path,
    elapsed: std::time::Duration,
) {
    let mut entry = Entry::new(
        Severity::Warning,
        friendly::title_for(action),
        gitpoulpi_core::redact(&CoreError::PushRejected(rejection.clone()).to_string()),
    );
    entry.kind = gitpoulpi_core::FailureKind::NonFastForward;
    entry.detail = rejection.detail.as_deref().cloned();
    if let Some((summary, hint)) = friendly::wording(entry.kind, "") {
        entry.summary = Some(summary);
        entry.hint = Some(hint);
    }
    entry.retry = Some(action.clone());
    entry.action_text = Some(friendly::action_text(action));
    entry.repo = Some(repo.to_path_buf());
    entry.duration_ms = timing::action_finished(action, repo, ms(elapsed), false);
    let id = feed::reserve_id();
    entry.id = id;
    let context = complete_context(action.op_context(), repo, id);
    tracing::warn!(target: "tui", repo = repo_text(repo).as_str(), "{}", context.failure_line(&entry.message));
    entry.context = Some(context);
    feed::push(entry);
}

/// How a workspace-wide task (fetch all, pull all, clone missing) ended, for
/// the Global tab: it belongs to no open repository.
pub(crate) fn note_workspace(message: &str, workspace: &str) {
    let flat = message.lines().next().unwrap_or("").trim();
    if flat.is_empty() {
        return;
    }
    tracing::info!(target: "tui", "{flat}");
    let mut entry = Entry::new(Severity::Info, flat.to_string(), flat.to_string());
    entry.workspace = Some(workspace.to_string());
    feed::push(entry);
}

/// A status message worth keeping in the Activity tab.
pub(crate) fn note_message(message: &str, repo: &std::path::Path) {
    let flat = message.lines().next().unwrap_or("").trim();
    if flat.is_empty() {
        return;
    }
    tracing::info!(target: "tui", repo = repo_text(repo).as_str(), "{flat}");
    let mut entry = Entry::new(Severity::Info, flat.to_string(), flat.to_string());
    entry.repo = Some(repo.to_path_buf());
    feed::push(entry);
}

/// What a mouse event did to the Activity Log window or the status line's
/// error. `Pass` means none of them wanted it.
pub(crate) enum Routed {
    Pass,
    Consumed(Option<Command>),
}

/// Routes a mouse event to the Activity Log window, the status line's error
/// or the status bar button.
pub(crate) fn route_mouse(mouse: MouseEvent) -> Routed {
    use crossterm::event::{MouseButton, MouseEventKind};
    panel::note_pointer(mouse.column, mouse.row);
    if panel::is_open() {
        panel::on_mouse(mouse);
        return Routed::Consumed(panel::take_command());
    }
    if mouse.kind == MouseEventKind::Down(MouseButton::Left)
        && status_error::hit(mouse.column, mouse.row)
    {
        status_error::open_entry();
        return Routed::Consumed(None);
    }
    if mouse.kind == MouseEventKind::Down(MouseButton::Left)
        && panel::status_button_hit(mouse.column, mouse.row)
    {
        panel::open();
        return Routed::Consumed(None);
    }
    Routed::Pass
}

/// A key while the Activity Log is open: it takes them all. A remedy button's
/// key returns what it asks the window to do.
pub(crate) fn on_panel_key(key: crossterm::event::KeyEvent) -> Option<Command> {
    panel::on_key(key);
    panel::take_command()
}

/// `log.level` and `log.max_file_mb` changed in Preferences: the running log
/// follows (the retention applies when the app starts).
pub(crate) fn apply_log_settings(log: &gitpoulpi_config::LogConfig) {
    gitpoulpi_log::set_level(gitpoulpi_log::level_of(log.level));
    if let Some(logger) = gitpoulpi_log::global() {
        logger.set_max_file_bytes(log.max_file_bytes());
    }
}

/// The repository of the tab in front: the Repository tab lists its
/// operations. Called by the window's tick, so it is current whenever the
/// Activity Log opens.
pub(crate) fn set_active_repo(root: &std::path::Path) {
    panel::set_active_repo(root);
}

/// Paints the Activity Log window, then the status error on the bottom row
/// of a surface that covers the footer.
pub(crate) fn render(frame: &mut Frame) {
    panel::render(frame);
    status_error::paint_fallback(frame);
}

/// The Activity Log's "copy diagnostics" for the palette.
pub(crate) fn copy_diagnostics() -> String {
    panel::copy_diagnostics_now()
}
