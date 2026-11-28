//! GPUI actions the window's menus carry, and the handles the command layer reaches the workspace by.
//!
//! Keys do not come through here: `commands::keys` resolves every keystroke with the shared keymap and
//! runs the command through `commands::run`. What stays is the small set of actions that carry data
//! (a theme, a language, a recent repository, a command id) for the menus built by the toolkit.

use gitpoulpi_i18n::Lang;
use gpui_kit::{Action, App, Context, Entity, Global, SharedString, WeakEntity, Window};
use serde::Deserialize;

use super::workspace::Workspace;
use crate::commands;
use crate::prefs::PrefsStore;
use crate::theme::{self, CurrentTheme, LaneTheme};

#[derive(Action, Clone, PartialEq, Eq, Deserialize)]
#[action(namespace = gp, no_json)]
pub struct SetTheme(pub SharedString);

#[derive(Action, Clone, PartialEq, Eq, Deserialize)]
#[action(namespace = gp, no_json)]
pub struct SetLang(pub SharedString);

#[derive(Action, Clone, PartialEq, Eq, Deserialize)]
#[action(namespace = gp, no_json)]
pub struct SetLaneTheme(pub SharedString);

#[derive(Action, Clone, PartialEq, Eq, Deserialize)]
#[action(namespace = gp, no_json)]
pub struct OpenRecent(pub SharedString);

/// Runs a command of the registry by its id (`global.palette`, `palette.refresh`, ...).
#[derive(Action, Clone, PartialEq, Eq, Deserialize)]
#[action(namespace = gp, no_json)]
pub struct RunCommand(pub SharedString);

/// Weak handle to the workspace of the main window.
pub(super) struct WorkspaceHandle(pub WeakEntity<Workspace>);
impl Global for WorkspaceHandle {}

/// The workspace of the main window, once it exists.
pub(crate) fn workspace(cx: &App) -> Option<Entity<Workspace>> {
    cx.try_global::<WorkspaceHandle>()
        .and_then(|h| h.0.upgrade())
}

/// Runs `f` on the workspace from a global action handler. Deferred: an action fires inside the
/// window's own key dispatch, so updating the window again right there would fail.
pub(super) fn with_workspace(
    cx: &mut App,
    f: impl FnOnce(&mut Workspace, &mut Window, &mut Context<Workspace>) + 'static,
) {
    let Some(ws) = workspace(cx) else {
        return;
    };
    let Some(win) = cx.windows().first().copied() else {
        return;
    };
    cx.defer(move |cx| {
        let _ = win.update(cx, move |_, window, cx| {
            ws.update(cx, |ws, cx| f(ws, window, cx));
        });
    });
}

/// Re-applies the current theme (after the lane palette changed, for instance).
fn reapply_theme(cx: &mut App) {
    let slug = cx.global::<CurrentTheme>().0.clone();
    theme::apply(&theme::find_preset(&slug), cx);
}

/// Switches to a theme and remembers the choice.
pub fn set_theme(slug: &str, cx: &mut App) {
    let spec = theme::find_preset(slug);
    theme::apply(&spec, cx);
    PrefsStore::update(cx, |p| p.ui.theme = spec.slug.clone());
    cx.refresh_windows();
}

/// Switches the UI language and remembers the choice.
pub fn set_language(lang: Lang, cx: &mut App) {
    gitpoulpi_i18n::set(lang);
    PrefsStore::update(cx, |p| p.ui.language = lang.code().to_string());
    cx.refresh_windows();
}

/// Switches the graph colour palette (`"theme"` follows the theme) and remembers the choice.
pub fn set_lane_theme(slug: &str, cx: &mut App) {
    let choice = if slug == "theme" {
        None
    } else {
        Some(slug.to_string())
    };
    cx.set_global(LaneTheme(choice.clone()));
    PrefsStore::update(cx, |p| p.ui.lane_theme = choice);
    reapply_theme(cx);
    // The lane colour indices come from the layout, so a repaint is all that is needed.
    cx.refresh_windows();
}

/// Registers the global handlers. Call once at startup.
pub fn init(cx: &mut App) {
    cx.on_action(|a: &SetTheme, cx| set_theme(a.0.as_ref(), cx));
    cx.on_action(|a: &SetLaneTheme, cx| set_lane_theme(a.0.as_ref(), cx));
    cx.on_action(|a: &SetLang, cx| {
        if let Some(l) = gitpoulpi_i18n::SHIPPED
            .iter()
            .copied()
            .find(|l| l.native_name() == a.0.as_ref())
        {
            set_language(l, cx);
        }
    });
    cx.on_action(|a: &OpenRecent, cx| {
        let p = std::path::PathBuf::from(a.0.as_ref());
        with_workspace(cx, move |ws, _, cx| ws.open_repo(&p, cx));
    });
    cx.on_action(|a: &RunCommand, cx| {
        let id = a.0.to_string();
        let Some(win) = cx.windows().first().copied() else {
            return;
        };
        cx.defer(move |cx| {
            let _ = win.update(cx, move |_, window, cx| {
                commands::run_id(&id, window, cx);
            });
        });
    });
}
