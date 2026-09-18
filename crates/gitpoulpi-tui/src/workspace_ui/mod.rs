//! Workspaces in the TUI: named groups of repositories, each with its own
//! tab list and an optional default commit profile (plan:
//! the workspaces design).
//!
//! - [`store`]: `workspaces.toml` and the session's active workspace and
//!   per-workspace tab lists (file I/O, no IPC).
//! - [`task`]: what needs the daemon (a repository's `origin`, Fetch all,
//!   Pull all, Clone missing), off the input loop.
//! - [`watch`]: notices `workspaces.toml` edited from outside the window.
//! - [`chip`]: the tab bar's workspace chip and its combo.
//! - [`dialog`]: the New / Edit workspace form.
//! - the Home tab's Workspaces page lives in `repo_manager::workspaces`.

pub(crate) mod ai_box;
pub(crate) mod apply;
pub(crate) mod chip;
pub(crate) mod dialog;
pub(crate) mod menu;
pub(crate) mod store;
pub(crate) mod task;
pub(crate) mod watch;

use gitpoulpi_i18n::{msg::workspaces as m, M};
use ratatui::style::Color;

use crate::th;

/// The colors a workspace can wear: theme roles by name, so a theme change
/// recolors them. `accent` is the default for a workspace with none.
pub(crate) const COLORS: [(&str, M); 6] = [
    ("accent", m::COLOR_ACCENT),
    ("green", m::COLOR_GREEN),
    ("yellow", m::COLOR_YELLOW),
    ("red", m::COLOR_RED),
    ("blue", m::COLOR_BLUE),
    ("purple", m::COLOR_PURPLE),
];

/// The color named `name` (a stored `color`), the accent for `None` or a
/// name nothing here knows (a hand-edited or newer file).
pub(crate) fn color_of(name: Option<&str>) -> Color {
    match name.unwrap_or("accent") {
        "green" => th().status.success,
        "yellow" => th().status.warning,
        "red" => th().status.danger,
        "blue" => th().status.info,
        "purple" => th().status.ai,
        _ => th().ui.accent,
    }
}

/// The glyph before a workspace's name. U+25CF is text-presentation by
/// default, one column in the fonts a terminal falls back to.
pub(crate) const DOT: &str = "\u{25CF}";
