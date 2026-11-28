//! The application shell: startup, the main window and its actions.

mod actions;
mod chrome;
mod live;
mod workspace;

use gitpoulpi_config::ConfigPaths;
use gpui_kit::component::TitleBar;
use gpui_kit::{point, px, size, App, AppContext as _, Bounds, WindowBounds, WindowOptions};

pub use crate::commands::status::Notice;
pub(crate) use actions::workspace as workspace_of;
pub use actions::{set_lane_theme, set_language, set_theme};
pub use workspace::{BannerButton, Workspace};

use crate::commands;
use crate::daemon::Shared;
pub use crate::prefs::Overrides;
use crate::prefs::{PrefsStore, WindowPrefs};
use crate::{fonts, panels};

/// Application id the window manager groups the windows under.
pub const APP_ID: &str = "gitpoulpi-desktop";

/// Registers fonts, settings, theme, language, shortcuts and action handlers. `paths` is the config
/// folder (`desktop.toml`, `config.toml`; `None` keeps the settings in memory). Call once, before the
/// first window.
pub fn init(cx: &mut App, paths: Option<ConfigPaths>, overrides: &Overrides) -> anyhow::Result<()> {
    gpui_kit::init(cx);
    fonts::register(cx)?;
    cx.set_global(PrefsStore::open(paths, overrides.clone()));
    let issues = PrefsStore::install(cx);

    actions::init(cx);
    commands::keys::install(&issues, cx);
    panels::log::init(cx);
    panels::refs::init(cx);
    panels::diff::init(cx);
    panels::activity::init(cx);
    PrefsStore::watch(cx);
    Ok(())
}

/// Options of the main window: the remembered size, centred, with the app's own title bar.
pub fn window_options(cx: &App) -> WindowOptions {
    let saved = PrefsStore::get(cx).window();
    let (min_w, min_h) = WindowPrefs::min();
    let bounds = match saved.position {
        Some((x, y)) => Bounds::new(point(px(x), px(y)), size(px(saved.width), px(saved.height))),
        None => Bounds::centered(None, size(px(saved.width), px(saved.height)), cx),
    };
    WindowOptions {
        window_bounds: Some(if saved.maximized {
            WindowBounds::Maximized(bounds)
        } else {
            WindowBounds::Windowed(bounds)
        }),
        window_min_size: Some(size(px(min_w), px(min_h))),
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
