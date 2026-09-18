//! "Delete where?": the scope of a tag delete, when the tag lives on a
//! remote.
//!
//! The red Delete of a tag used to delete only the local one, and the next
//! fetch brought it back (git follows the remote's tags). When a remote has
//! the tag this dialog asks which delete is meant, as numbered rows with the
//! consequence of the highlighted one spelled out under them (master and
//! detail; nothing cycles on click):
//!
//! 1. **Delete locally**: this repository only. Runs at once (it loses no
//!    work: Undo restores it, and a fetch can bring it back), and the
//!    message afterwards says the remote still has it.
//! 2. **Delete on `<remote>`**: outward-facing, so it goes on to a y/n
//!    confirm *naming the remote*. Not undoable.
//! 3. **Delete locally and on `<remote>`**: the same confirm; Undo then
//!    restores the local tag only and the tag can be pushed back.
//!
//! One row per remote that has the tag (usually one). A tag that is only on
//! a remote has no local rows. Modal like `revert_mainline`: it owns the
//! keyboard (Up/Down, 1-9, Enter, Esc) and the mouse (a click on a row picks
//! it, a click away closes and is consumed).

use std::sync::Mutex;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::Frame;

use crate::daemon_link::{Action, DaemonLink};
use crate::dialog_kit;
use crate::modes::{ConfirmAction, InputMode, PendingDestructive};
use crate::th;
use gitpoulpi_i18n::msg::tag_where as msg;

/// Which delete a row stands for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Scope {
    Local,
    Remote(String),
    Both(String),
}

/// What a pick chose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Pick {
    pub(crate) name: String,
    pub(crate) scope: Scope,
    /// Every remote that has the tag, for the "still on" wording.
    pub(crate) on: Vec<String>,
}

struct Open {
    name: String,
    on: Vec<String>,
    options: Vec<Scope>,
    selected: usize,
    /// The rect of each option row, from the last paint.
    rows: Vec<Rect>,
    close: dialog_kit::CloseHit,
}

static POPUP: Mutex<Option<Open>> = Mutex::new(None);

/// The rows of the dialog, in order: local first, then per remote its
/// remote-only and its both-places delete. A tag with no local ref has no
/// local rows.
pub(crate) fn options(has_local: bool, remotes: &[String]) -> Vec<Scope> {
    let mut out = Vec::new();
    if has_local {
        out.push(Scope::Local);
    }
    for r in remotes {
        out.push(Scope::Remote(r.clone()));
        if has_local {
            out.push(Scope::Both(r.clone()));
        }
    }
    out
}

/// Starts the delete of tag `name` as the context menu, the keyboard and
/// the palette all do: a tag only here keeps the plain confirm; a tag on one
/// remote and nowhere local goes straight to that remote's confirm; every
/// other case opens the scope dialog. `want_remote` (the palette's "delete
/// from remote") starts with the remote row highlighted.
pub(crate) fn begin(
    name: &str,
    want_remote: bool,
    input_mode: &mut InputMode,
    last_message: &mut Option<String>,
) {
    let info = crate::tag_where_ui::lookup(name);
    let (has_local, on) = match &info {
        Some(i) => (
            i.place != gitpoulpi_core::domain::TagWhere::Remote,
            i.all_remotes(),
        ),
        None => (true, Vec::new()),
    };
    match (has_local, on.as_slice()) {
        (true, []) => {
            *input_mode = InputMode::Confirm {
                prompt: gitpoulpi_i18n::msg::menu_ref::confirm_delete_tag(name),
                action: ConfirmAction::Destructive(PendingDestructive::DeleteTag(name.into())),
            };
        }
        (false, []) => *last_message = Some(gitpoulpi_i18n::msg::menu_ref::no_remote_has(name)),
        (false, [only]) => confirm_remote(name, only, input_mode),
        _ => open(name, has_local, on, want_remote),
    }
}

fn confirm_remote(name: &str, remote: &str, input_mode: &mut InputMode) {
    *input_mode = InputMode::Confirm {
        prompt: msg::confirm_remote(name, remote),
        action: ConfirmAction::Destructive(PendingDestructive::DeleteRemoteTag {
            remote: remote.into(),
            name: name.into(),
        }),
    };
}

fn open(name: &str, has_local: bool, on: Vec<String>, want_remote: bool) {
    let options = options(has_local, &on);
    let selected = if want_remote {
        options
            .iter()
            .position(|o| matches!(o, Scope::Remote(_)))
            .unwrap_or(0)
    } else {
        0
    };
    if let Ok(mut p) = POPUP.lock() {
        *p = Some(Open {
            name: name.into(),
            on,
            options,
            selected,
            rows: Vec::new(),
            close: Default::default(),
        });
    }
}

pub(crate) fn is_open() -> bool {
    POPUP.lock().is_ok_and(|p| p.is_some())
}

fn pick_of(open: &Open, index: usize) -> Option<Pick> {
    Some(Pick {
        name: open.name.clone(),
        scope: open.options.get(index)?.clone(),
        on: open.on.clone(),
    })
}

/// A key while the dialog is open (it swallows them all).
pub(crate) fn on_key(key: KeyEvent) -> Option<Pick> {
    if key.kind != KeyEventKind::Press {
        return None;
    }
    let mut slot = POPUP.lock().ok()?;
    let open = slot.as_mut()?;
    let last = open.options.len().saturating_sub(1);
    match key.code {
        KeyCode::Esc => {
            *slot = None;
            None
        }
        KeyCode::Up | KeyCode::Char('k') => {
            open.selected = open.selected.saturating_sub(1);
            None
        }
        KeyCode::Down | KeyCode::Char('j') => {
            open.selected = (open.selected + 1).min(last);
            None
        }
        KeyCode::Char(c @ '1'..='9') => {
            let i = c as usize - '1' as usize;
            if i <= last {
                // A number picks the row, like the Enter of the row it names.
                let pick = pick_of(open, i);
                *slot = None;
                pick
            } else {
                None
            }
        }
        KeyCode::Enter => {
            let pick = pick_of(open, open.selected);
            *slot = None;
            pick
        }
        _ => None,
    }
}

/// A mouse event while the dialog is open (it swallows them all).
pub(crate) fn on_mouse(mouse: MouseEvent) -> Option<Pick> {
    let mut slot = POPUP.lock().ok()?;
    let open = slot.as_mut()?;
    let over = open
        .rows
        .iter()
        .position(|r| r.contains((mouse.column, mouse.row).into()));
    match mouse.kind {
        MouseEventKind::Moved | MouseEventKind::Drag(_) => {
            open.close.set_hover(mouse.column, mouse.row);
            if let Some(i) = over {
                open.selected = i;
            }
            None
        }
        MouseEventKind::Down(MouseButton::Left) => {
            if open.close.hit(mouse.column, mouse.row) {
                *slot = None;
                return None;
            }
            match over {
                Some(i) => {
                    let pick = pick_of(open, i);
                    *slot = None;
                    pick
                }
                None => {
                    // A click away closes the dialog and is consumed.
                    let inside = open
                        .rows
                        .first()
                        .zip(open.rows.last())
                        .is_some_and(|(a, b)| {
                            mouse.row >= a.y.saturating_sub(2) && mouse.row <= b.y + 8
                        });
                    if !inside {
                        *slot = None;
                    }
                    None
                }
            }
        }
        _ => None,
    }
}

/// What a pick leads to.
pub(crate) enum Decision {
    /// The local delete of a tag `on` still has: runs at once.
    DeleteLocal { name: String, on: Vec<String> },
    /// The y/n confirm that names the remote.
    Confirm {
        prompt: String,
        pending: PendingDestructive,
    },
}

/// What a pick does: the local delete runs at once, the others go on to the
/// confirm that names the remote.
pub(crate) fn decide(pick: Pick) -> Decision {
    match pick.scope {
        Scope::Local => Decision::DeleteLocal {
            name: pick.name,
            on: pick.on,
        },
        Scope::Remote(remote) => Decision::Confirm {
            prompt: msg::confirm_remote(&pick.name, &remote),
            pending: PendingDestructive::DeleteRemoteTag {
                remote,
                name: pick.name,
            },
        },
        Scope::Both(remote) => Decision::Confirm {
            prompt: msg::confirm_both(&pick.name, &remote),
            pending: PendingDestructive::DeleteTagBoth {
                remote,
                name: pick.name,
            },
        },
    }
}

pub(crate) fn apply(
    pick: Pick,
    link: &DaemonLink,
    input_mode: &mut InputMode,
    last_message: &mut Option<String>,
) {
    match decide(pick) {
        Decision::DeleteLocal { name, on } => {
            *last_message = Some(msg::msg_deleted_local_still(&name, on.join(", ")));
            link.submit_action(Action::TagDeleteKeepRemote { name, remotes: on });
        }
        Decision::Confirm { prompt, pending } => {
            *input_mode = InputMode::Confirm {
                prompt,
                action: ConfirmAction::Destructive(pending),
            };
        }
    }
}

fn label(scope: &Scope) -> String {
    match scope {
        Scope::Local => msg::OPT_LOCAL.get().to_string(),
        Scope::Remote(r) => msg::opt_remote(r),
        Scope::Both(r) => msg::opt_both(r),
    }
}

fn detail(scope: &Scope, on: &[String]) -> String {
    match scope {
        Scope::Local => msg::detail_local(on.join(", ")),
        Scope::Remote(r) => msg::detail_remote(r),
        Scope::Both(r) => msg::detail_both(r),
    }
}

/// Paints the dialog, last, over the panes.
pub(crate) fn render(frame: &mut Frame) {
    let Ok(mut slot) = POPUP.lock() else {
        return;
    };
    let Some(open) = slot.as_mut() else {
        return;
    };
    let area = frame.area();
    let width = 72u16.min(area.width.saturating_sub(4));
    if width < 30 || area.height < 12 {
        return;
    }
    let inner = width.saturating_sub(4) as usize;
    let detail_lines = crate::commit_detail::wrap_text(
        &detail(
            &open.options[open.selected.min(open.options.len() - 1)],
            &open.on,
        ),
        inner,
    );
    let height = (open.options.len() as u16 + detail_lines.len() as u16 + 6)
        .min(area.height.saturating_sub(2));
    let outer = dialog_kit::centered(area, width, height);
    let buf = frame.buffer_mut();
    open.close.panel(buf, outer, msg::DLG_TITLE.get());
    let text = Style::default().fg(th().ui.text).bg(th().ui.panel);
    dialog_kit::put(
        buf,
        outer.x + 2,
        outer.y + 1,
        &msg::dlg_question(&open.name),
        outer.width.saturating_sub(4),
        text,
    );
    open.rows.clear();
    let mut y = outer.y + 3;
    for (i, scope) in open.options.iter().enumerate() {
        let selected = i == open.selected;
        let style = if selected {
            Style::default()
                .fg(th().ui.text)
                .bg(th().ui.selection)
                .add_modifier(Modifier::BOLD)
        } else {
            text
        };
        let row = Rect::new(outer.x + 1, y, outer.width.saturating_sub(2), 1);
        dialog_kit::fill(buf, row, style);
        // The destructive rows read as such: the row text is the ordinary
        // ink, the number carries the danger colour when not selected.
        dialog_kit::put(
            buf,
            outer.x + 2,
            y,
            &format!("{}  {}", i + 1, label(scope)),
            outer.width.saturating_sub(4),
            style,
        );
        // An option row is a list row: the arrow, lit (the pointer selects it).
        crate::pointer_map::plain(row);
        open.rows.push(row);
        y += 1;
    }
    y += 1;
    let muted = Style::default().fg(th().ui.muted).bg(th().ui.panel);
    for line in detail_lines {
        dialog_kit::put(
            buf,
            outer.x + 2,
            y,
            &line,
            outer.width.saturating_sub(4),
            muted,
        );
        y += 1;
    }
    let hint_y = outer.y + outer.height.saturating_sub(2);
    dialog_kit::put(
        buf,
        outer.x + 2,
        hint_y,
        msg::DLG_HINT.get(),
        outer.width.saturating_sub(4),
        muted,
    );
}

#[cfg(test)]
mod tests;
