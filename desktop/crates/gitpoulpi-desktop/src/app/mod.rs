//! The application shell: startup, the main window and its actions.

mod actions;
mod chrome;
mod workspace;

use std::path::PathBuf;

use gitpoulpi_i18n::Lang;
use gpui_kit::component::TitleBar;
use gpui_kit::{px, size, App, AppContext as _, Bounds, WindowBounds, WindowOptions};

pub(crate) use actions::workspace as workspace_of;
pub use actions::{set_lane_theme, set_language, set_theme};
pub use workspace::Workspace;

use crate::commands;
use crate::daemon::Shared;
use crate::prefs::{PrefsStore, WindowPrefs};
use crate::theme::{self, LaneTheme};
use crate::{fonts, panels};

/// Application id the window manager groups the windows under.
pub const APP_ID: &str = "gitpoulpi-desktop";

/// Command-line choices that override the stored preferences for this run only.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct Overrides {
    pub theme: Option<String>,
    pub lane_theme: Option<String>,
    pub lang: Option<Lang>,
}

/// Registers fonts, preferences, theme, language, shortcuts and action handlers. `prefs_dir` is where
/// `desktop.toml` lives (`None` keeps the preferences in memory). Call once, before the first window.
pub fn init(cx: &mut App, prefs_dir: Option<PathBuf>, overrides: &Overrides) -> anyhow::Result<()> {
    gpui_kit::init(cx);
    fonts::register(cx)?;
    let config_dir = prefs_dir.clone();
    cx.set_global(PrefsStore::open(prefs_dir));
    let prefs = PrefsStore::get(cx).ui.clone();

    let lang = overrides
        .lang
        .or_else(|| Lang::parse(&prefs.language))
        .unwrap_or(Lang::En);
    gitpoulpi_i18n::set(lang);

    cx.set_global(LaneTheme(overrides.lane_theme.clone().or(prefs.lane_theme)));
    let spec = theme::find_preset(overrides.theme.as_deref().unwrap_or(&prefs.theme));
    theme::apply(&spec, cx);

    actions::init(cx);
    commands::keys::install(config_dir.as_deref(), cx);
    panels::log::init(cx);
    panels::diff::init(cx);
    Ok(())
}

/// Options of the main window: the remembered size, centred, with the app's own title bar.
pub fn window_options(cx: &App) -> WindowOptions {
    let WindowPrefs { width, height } = PrefsStore::get(cx).window;
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
            None,
            size(px(width), px(height)),
            cx,
        ))),
        window_min_size: Some(size(px(WindowPrefs::MIN.0), px(WindowPrefs::MIN.1))),
        app_id: Some(APP_ID.into()),
        #[cfg(target_os = "linux")]
        window_decorations: Some(gpui_kit::WindowDecorations::Client),
        ..TitleBar::window_options()
    }
}

/// Opens the main window on a running daemon link; returns its handle and the workspace view.
pub fn open_main_window(
    backend: Shared,
    cx: &mut App,
) -> anyhow::Result<(gpui_kit::AnyWindowHandle, gpui_kit::Entity<Workspace>)> {
    let opened = gpui_kit::open_window(window_options(cx), cx, move |window, cx| {
        window.set_window_title("GitPoulpi");
        cx.new(|cx| Workspace::new(backend, window, cx))
    })?;
    cx.activate(true);
    Ok(opened)
}
