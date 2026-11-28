//! The desktop's settings: `desktop.toml` through the shared `gitpoulpi_config::DesktopConfig` (theme,
//! language, graph colours, layout, window, pane and column widths, every per-interface `[ui]` key and
//! the key bindings), the common `config.toml` for what both interfaces share, and the recent list of
//! `gitpoulpi-session`.
//!
//! Nothing here defines a file format. [`PrefsStore`] is the application global that holds what was
//! read, writes through the shared setters, and notices an edit made from outside (another
//! frontend, `gitpoulpi config set desktop.ui...`, a text editor): [`watch`] compares a cheap
//! fingerprint of the files and [`load`] re-reads them off the UI thread.

mod columns;
mod load;
mod store;
mod watch;

pub use columns::{resolve, Col, ColumnLayout, ColumnWidths, MESSAGE_FLOOR, MESSAGE_MIN};
pub use load::Loaded;
pub use store::{Overrides, PrefsStore};

use gitpoulpi_config::{pixel_range, DesktopUiConfig};

/// File name inside the config directory.
pub const FILE_NAME: &str = "desktop.toml";

fn range(key: &str) -> (f32, f32) {
    pixel_range(key).map_or((0.0, f32::MAX), |(lo, hi)| (lo as f32, hi as f32))
}

/// Which side panes are shown and how wide, in logical pixels.
#[derive(Clone, Debug, PartialEq)]
pub struct LayoutPrefs {
    pub show_left: bool,
    pub show_right: bool,
    pub left_width: f32,
    pub right_width: f32,
}

impl LayoutPrefs {
    /// The panes' limits are the schema's (`desktop.ui.left_pane_width`, `right_pane_width`).
    pub fn left_range() -> (f32, f32) {
        range("left_pane_width")
    }

    pub fn right_range() -> (f32, f32) {
        range("right_pane_width")
    }

    pub fn of(ui: &DesktopUiConfig) -> Self {
        Self {
            show_left: ui.left_pane_visible,
            show_right: ui.right_pane_visible,
            left_width: ui.left_pane_width as f32,
            right_width: ui.right_pane_width as f32,
        }
    }
}

/// The main window's remembered geometry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowPrefs {
    pub width: f32,
    pub height: f32,
    /// Top-left corner; `None` lets the window manager (or the centring) place it.
    pub position: Option<(f32, f32)>,
    pub maximized: bool,
}

impl WindowPrefs {
    /// Smallest window the layout is designed for; also the floor of a restored size.
    pub fn min() -> (f32, f32) {
        (range("window_width").0, range("window_height").0)
    }

    pub fn of(ui: &DesktopUiConfig) -> Self {
        Self {
            width: ui.window_width as f32,
            height: ui.window_height as f32,
            position: ui
                .window_x
                .zip(ui.window_y)
                .map(|(x, y)| (x as f32, y as f32)),
            maximized: ui.window_maximized,
        }
    }
}
