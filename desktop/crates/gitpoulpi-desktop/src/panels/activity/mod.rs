//! Undo history, Activity Log, background jobs and grouped pushes: the panels the terminal UI opens over
//! its panes, as dialogs of the window.
//!
//! | Module | Role |
//! |--------|------|
//! | `safety_model` | the undo history, Reflog and Lost work views as plain data: rows, selection, what each row can do |
//! | `safety` | the undo-safety panel's state and what its actions do (confirmations, steps, restores) |
//! | `safety_render` | painting the panel and its keys, and opening it |
//! | `log_model` | the Activity Log's filters (level, scope) and line texts |
//! | `activity_log` | the Activity Log's state: the Git operations feed and the application log |
//! | `activity_log_render` | painting the Activity Log and its keys, and opening it |
//! | `jobs` | job wording, cancelling, and the jobs panel |
//! | `group` | the progress of a grouped push, per remote |
//! | `resolve` | the questions the link raises: a rejected push, a risky undo, forgetting a blocked entry |
//! | `families` | the palette's generated `Jobs: Cancel <job>` rows and the last-error facts |
//! | `widgets` | the button and tab chips these panels share |

pub mod activity_log;
mod activity_log_render;
mod families;
pub mod group;
pub mod jobs;
pub mod log_model;
pub mod resolve;
pub mod safety;
pub mod safety_model;
mod safety_render;
mod widgets;

use gpui_kit::App;

pub use activity_log::ActivityView;
pub use activity_log_render::open as open_activity;
pub use jobs::JobsView;
pub use safety::SafetyView;
pub use safety_model::Tab as SafetyTab;
pub use safety_render::open as open_safety;

/// Registers the palette rows this area generates. Call once from `app::init`.
pub fn init(_cx: &mut App) {
    families::install();
}

/// The view the open undo-safety panel is on (the palette's availability rules read it).
pub fn open_safety_tab(cx: &App) -> Option<SafetyTab> {
    safety::current(cx).map(|v| v.read(cx).tab)
}

/// What the palette needs to know about the grouped push the link last reported.
pub fn group_state(
    backend: &crate::daemon::Shared,
) -> gitpoulpi_app::palette::vocab::push_group::GroupState {
    match backend.host().group_run() {
        None => Default::default(),
        Some(run) => gitpoulpi_app::palette::vocab::push_group::GroupState {
            run: true,
            paused: false,
            ready: false,
            retry: !run.going() && run.has_retryable(),
        },
    }
}
