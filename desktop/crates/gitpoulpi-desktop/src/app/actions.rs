//! The window's actions: what menus, the command palette and shortcuts trigger.
//!
//! Handlers are registered globally and reach the workspace through a weak handle, deferred: an action
//! fires inside the window's own key dispatch, so updating the window again right there would fail.

use gitpoulpi_i18n::Lang;
use gpui_kit::{
    actions, Action, App, Context, Global, KeyBinding, SharedString, WeakEntity, Window,
};
use serde::Deserialize;

use super::workspace::Workspace;
use crate::prefs::PrefsStore;
use crate::theme::{self, CurrentTheme, LaneTheme};

actions!(
    gp,
    [
        OpenPalette,
        FocusRefs,
        FocusLog,
        FocusChanges,
        FocusFilter,
        OpenRepo,
        Quit,
        ToggleLeft,
        ToggleRight,
        FitColumns,
        ResetColumns,
        ToggleSearch
    ]
);

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

/// Weak handle to the workspace of the main window.
pub(super) struct WorkspaceHandle(pub WeakEntity<Workspace>);
impl Global for WorkspaceHandle {}

/// Runs `f` on the workspace from a global action handler.
pub(super) fn with_workspace(
    cx: &mut App,
    f: impl FnOnce(&mut Workspace, &mut Window, &mut Context<Workspace>) + 'static,
) {
    let Some(ws) = cx.try_global::<WorkspaceHandle>().map(|h| h.0.clone()) else {
        return;
    };
    let Some(win) = cx.windows().first().copied() else {
        return;
    };
    cx.defer(move |cx| {
        let _ = win.update(cx, move |_, window, cx| {
            let _ = ws.update(cx, |ws, cx| f(ws, window, cx));
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

/// Registers shortcuts and global handlers. Call once at startup.
pub fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("ctrl-p", OpenPalette, None),
        KeyBinding::new("ctrl-shift-p", OpenPalette, None),
        KeyBinding::new("ctrl-1", FocusRefs, None),
        KeyBinding::new("ctrl-2", FocusLog, None),
        KeyBinding::new("ctrl-3", FocusChanges, None),
        KeyBinding::new("ctrl-f", ToggleSearch, None),
        KeyBinding::new("ctrl-o", OpenRepo, None),
        KeyBinding::new("ctrl-q", Quit, None),
    ]);
    cx.on_action(|_: &Quit, cx| cx.quit());
    cx.on_action(|_: &OpenPalette, cx| with_workspace(cx, |ws, w, cx| ws.open_palette(w, cx)));
    cx.on_action(|_: &FocusRefs, cx| {
        with_workspace(cx, |ws, w, cx| {
            ws.refs.update(cx, |r, cx| r.focus_filter(w, cx))
        })
    });
    cx.on_action(|_: &FocusLog, cx| with_workspace(cx, |ws, w, cx| ws.focus_log(w, cx)));
    cx.on_action(|_: &ToggleSearch, cx| {
        with_workspace(cx, |ws, w, cx| {
            ws.log.update(cx, |l, cx| l.toggle_search(w, cx))
        })
    });
    cx.on_action(|_: &FocusFilter, cx| {
        with_workspace(cx, |ws, w, cx| {
            ws.log.update(cx, |l, cx| l.open_search(w, cx))
        })
    });
    cx.on_action(|_: &FocusChanges, cx| {
        use gpui_kit::Focusable as _;
        with_workspace(cx, |ws, w, cx| {
            ws.right.read(cx).focus_handle(cx).focus(w, cx)
        })
    });
    cx.on_action(|_: &FitColumns, cx| {
        with_workspace(cx, |ws, w, cx| ws.log.update(cx, |l, cx| l.fit_all(w, cx)))
    });
    cx.on_action(|_: &ResetColumns, cx| {
        with_workspace(cx, |ws, _, cx| {
            ws.log.update(cx, |l, cx| l.reset_widths(cx))
        })
    });
    cx.on_action(|_: &ToggleLeft, cx| with_workspace(cx, |ws, _, cx| ws.toggle_left(cx)));
    cx.on_action(|_: &ToggleRight, cx| with_workspace(cx, |ws, _, cx| ws.toggle_right(cx)));
    cx.on_action(|a: &SetTheme, cx| set_theme(a.0.as_ref(), cx));
    cx.on_action(|a: &SetLaneTheme, cx| {
        let choice = if a.0.as_ref() == "theme" {
            None
        } else {
            Some(a.0.to_string())
        };
        cx.set_global(LaneTheme(choice.clone()));
        PrefsStore::update(cx, |p| p.ui.lane_theme = choice);
        reapply_theme(cx);
        // The lane colour indices come from the layout, so a repaint is all that is needed.
        cx.refresh_windows();
    });
    cx.on_action(|a: &SetLang, cx| {
        if let Some(l) = gitpoulpi_i18n::SHIPPED
            .iter()
            .copied()
            .find(|l| l.native_name() == a.0.as_ref())
        {
            set_language(l, cx);
        }
    });
    cx.on_action(|_: &OpenRepo, cx| {
        let rx = cx.prompt_for_paths(gpui_kit::PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: None,
        });
        cx.spawn(async move |cx| {
            if let Ok(Ok(Some(paths))) = rx.await {
                if let Some(p) = paths.into_iter().next() {
                    cx.update(|cx| with_workspace(cx, move |ws, _, cx| ws.open_repo(&p, cx)));
                }
            }
        })
        .detach();
    });
    cx.on_action(|a: &OpenRecent, cx| {
        let p = std::path::PathBuf::from(a.0.as_ref());
        with_workspace(cx, move |ws, _, cx| ws.open_repo(&p, cx));
    });
}
