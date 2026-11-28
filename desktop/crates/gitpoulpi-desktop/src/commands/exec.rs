//! Runs commands. Every command goes through [`run`]: the ones the desktop implements run as their
//! [`Op`]; every other one answers "not available yet" in the status line (never a silent no-op).

use gitpoulpi_app::palette::vocab::diff_view::DiffViewMode;
use gitpoulpi_app::palette::vocab::tree_fold::TreeFold;
use gpui_kit::{App, Focusable as _, Window};

use super::catalog::{command_for_id, Command};
use super::keys;
use super::ops::{op_for, Op};
use super::staging;
use super::{palette, status};
use crate::app::{workspace_of, Workspace};
use crate::strings;

/// What running a command did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// The command ran.
    Done,
    /// The desktop does not implement it yet; the status line says so.
    NotAvailable,
}

/// The label of a command in the UI language, for messages.
pub fn label_of(command: Command) -> String {
    use gitpoulpi_app::keymap::STATIC_ACTIONS;
    match command {
        Command::Action(a) => STATIC_ACTIONS
            .iter()
            .find(|d| d.action == a)
            .map(|d| d.display_label().into_owned())
            .or_else(|| {
                gitpoulpi_app::palette::palette_entry(a).map(|e| e.display_label().into_owned())
            })
            .unwrap_or_default(),
        Command::Palette(c) => super::catalog::entry_of(c)
            .map(|e| e.display_label().into_owned())
            .unwrap_or_default(),
    }
}

/// Runs the command with this id (`global.quit`, `palette.refresh`, ...).
pub fn run_id(id: &str, window: &mut Window, cx: &mut App) -> Outcome {
    match command_for_id(id) {
        Some(command) => run(command, window, cx),
        None => {
            status::set(cx, strings::not_available_yet(id));
            Outcome::NotAvailable
        }
    }
}

/// Runs a command, or says it is not available yet.
pub fn run(command: Command, window: &mut Window, cx: &mut App) -> Outcome {
    match op_for(command) {
        Some(op) => {
            execute(op, window, cx);
            Outcome::Done
        }
        None => {
            status::set(cx, strings::not_available_yet(&label_of(command)));
            Outcome::NotAvailable
        }
    }
}

/// Runs `f` on the main window's workspace.
fn with_workspace(
    window: &mut Window,
    cx: &mut App,
    f: impl FnOnce(&mut Workspace, &mut Window, &mut gpui_kit::Context<Workspace>),
) {
    if let Some(ws) = workspace_of(cx) {
        ws.update(cx, |ws, cx| f(ws, window, cx));
    }
}

/// The folder chooser of the operating system, then open that repository.
fn browse_repository(cx: &mut App) {
    let rx = cx.prompt_for_paths(gpui_kit::PathPromptOptions {
        files: false,
        directories: true,
        multiple: false,
        prompt: None,
    });
    cx.spawn(async move |cx| {
        if let Ok(Ok(Some(paths))) = rx.await {
            if let Some(path) = paths.into_iter().next() {
                cx.update(|cx| {
                    if let Some(ws) = workspace_of(cx) {
                        ws.update(cx, |ws, cx| ws.open_repo(&path, cx));
                    }
                });
            }
        }
    })
    .detach();
}

/// Does what `op` says. Whatever the status line showed before is no longer true afterwards.
pub fn execute(op: Op, window: &mut Window, cx: &mut App) {
    if !matches!(op, Op::Prefix(_)) {
        status::clear(cx);
    }
    match op {
        Op::Quit => cx.quit(),
        Op::OpenPalette => palette::open(window, cx),
        Op::Prefix(ctx) => keys::begin_prefix(ctx, cx),
        Op::Keybindings => {
            crate::help::open(window, cx);
        }
        Op::ChangeTheme => palette::open_theme_picker(window, cx),
        Op::ChangeLanguage => palette::open_language_picker(window, cx),
        Op::ChangeGraphColors => palette::open_lane_picker(window, cx),
        Op::BrowseRepository => browse_repository(cx),
        Op::FindInLog => with_workspace(window, cx, |ws, window, cx| {
            ws.focus_log(window, cx);
            ws.log.update(cx, |l, cx| l.open_search(window, cx));
        }),
        Op::Refresh => with_workspace(window, cx, |ws, _, cx| {
            ws.refs.update(cx, |r, cx| r.reload(cx));
            ws.log.update(cx, |l, cx| l.reload(cx));
        }),
        Op::ToggleRefsPane => with_workspace(window, cx, |ws, _, cx| ws.toggle_left(cx)),
        Op::ToggleStatusPane => with_workspace(window, cx, |ws, _, cx| ws.toggle_right(cx)),
        Op::FocusRefs => with_workspace(window, cx, |ws, window, cx| {
            ws.refs.read(cx).focus_handle(cx).focus(window, cx)
        }),
        Op::FocusRefsFilter => with_workspace(window, cx, |ws, window, cx| {
            ws.refs.update(cx, |r, cx| r.focus_filter(window, cx))
        }),
        Op::FocusLog => with_workspace(window, cx, |ws, window, cx| ws.focus_log(window, cx)),
        Op::FocusStatus => with_workspace(window, cx, |ws, window, cx| {
            ws.right.read(cx).focus_handle(cx).focus(window, cx)
        }),
        Op::FitColumns => with_workspace(window, cx, |ws, window, cx| {
            ws.log.update(cx, |l, cx| l.fit_all(window, cx))
        }),
        Op::ResetColumns => with_workspace(window, cx, |ws, _, cx| {
            ws.log.update(cx, |l, cx| l.reset_widths(cx))
        }),
        Op::Git(what) => with_workspace(window, cx, move |ws, _, cx| ws.transfer(what, cx)),
        Op::RefsToggleHide => with_workspace(window, cx, |ws, _, cx| {
            ws.refs.update(cx, |r, cx| r.toggle_hidden_selected(cx))
        }),
        Op::RefsToggleSolo => with_workspace(window, cx, |ws, _, cx| {
            ws.refs.update(cx, |r, cx| r.toggle_solo_selected(cx))
        }),
        Op::RefsClearSolo => with_workspace(window, cx, |ws, _, cx| {
            ws.refs.update(cx, |r, cx| r.clear_solo(cx))
        }),
        Op::RefsShowAllHidden => with_workspace(window, cx, |ws, _, cx| {
            ws.refs.update(cx, |r, cx| r.show_all_hidden(cx))
        }),
        Op::RefsFold(fold) => with_workspace(window, cx, move |ws, _, cx| {
            ws.refs.update(cx, |r, cx| match fold {
                TreeFold::ExpandAll => r.expand_all(cx),
                TreeFold::CollapseAll => r.collapse_all(cx),
                TreeFold::ExpandBelow | TreeFold::CollapseBelow => {}
            })
        }),
        Op::RefsToggleSection => with_workspace(window, cx, |ws, _, cx| {
            ws.refs.update(cx, |r, cx| r.toggle_selected_section(cx))
        }),
        Op::DiffClose => with_workspace(window, cx, |ws, window, cx| ws.close_diff(window, cx)),
        Op::DiffLayout(mode) => with_workspace(window, cx, move |ws, _, cx| {
            ws.diff.update(cx, |d, cx| {
                if mode.is_none_or(|m| d.is_split() != (m == DiffViewMode::Split)) {
                    d.toggle_layout(cx)
                }
            })
        }),
        Op::DiffToggleWrap => with_workspace(window, cx, |ws, _, cx| {
            ws.diff.update(cx, |d, cx| d.toggle_wrap(cx))
        }),
        Op::DiffToggleWhitespace => with_workspace(window, cx, |ws, _, cx| {
            ws.diff.update(cx, |d, cx| d.toggle_whitespace(cx))
        }),
        Op::DiffStepHunk { forward } => with_workspace(window, cx, move |ws, _, cx| {
            ws.diff.update(cx, |d, cx| d.step_hunk(forward, cx))
        }),
        Op::Status(op) => with_workspace(window, cx, move |ws, window, cx| {
            ws.right
                .update(cx, |r, cx| staging::run_status(r, op, window, cx))
        }),
        Op::DiffPart(op) => with_workspace(window, cx, move |ws, _, cx| {
            if !ws.diff_is_open() {
                let why = gitpoulpi_i18n::msg::palette::WHY_THE_DIFF_VIEW_ISN_T_OPEN.get();
                return ws.say(why, cx);
            }
            ws.diff.update(cx, |d, cx| staging::run_part(d, op, cx))
        }),
    }
}
