//! The "which parent is the mainline?" dropdown of reverting a merge commit.
//!
//! A merge commit has several parents, and `git revert` needs to know which
//! side stays (`-m <n>`). Reverting a merge commit from the Log (the context
//! menu, the `Ctrl+G` chord, the palette's `Git: Revert Selected Commit...`:
//! all one function, `apply_log_context_action`) opens this popup instead of
//! the plain confirm; picking a parent goes on to the confirm modal, which
//! then submits `Action::Revert` with that mainline.
//!
//! The popup is a [`Combo`] hung from a small centered box, modal like the
//! other comboboxes (`open_in_bar`): it owns the keyboard (Up / Down / Enter /
//! Esc) and the mouse (a click away closes it and is consumed). The first
//! parent, the branch the merge was made on, starts highlighted.

use std::sync::Mutex;

use crossterm::event::{KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::Frame;

use crate::combo::{Combo, ComboOption, ComboOutcome};
use crate::dialog_kit;
use crate::th;
use gitpoulpi_core::domain::{CommitGraph, Oid};
use gitpoulpi_i18n::msg::revert_mainline as msg;

struct Open {
    combo: Combo,
    commit: Oid,
    close: dialog_kit::CloseHit,
}

static POPUP: Mutex<Option<Open>> = Mutex::new(None);

/// What a pick chose: revert `commit` keeping parent `parent` (1-based).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Pick {
    pub(crate) commit: Oid,
    pub(crate) parent: u32,
}

/// The parents of `commit` in `graph`, when it is a merge commit.
pub(crate) fn merge_parents(graph: &CommitGraph, commit: Oid) -> Option<Vec<Oid>> {
    let node = graph.nodes.iter().find(|n| n.id == commit)?;
    (node.parents.len() > 1).then(|| node.parents.clone())
}

/// One row per parent: its number, short SHA and summary (the SHA alone
/// when the parent is not in the loaded graph). The value is the number.
pub(crate) fn parent_options(graph: &CommitGraph, parents: &[Oid]) -> Vec<ComboOption> {
    parents
        .iter()
        .enumerate()
        .map(|(i, parent)| {
            let short = &parent.to_hex()[..crate::SHORT_SHA_LEN];
            let summary = graph
                .nodes
                .iter()
                .find(|n| n.id == *parent)
                .map_or("", |n| n.summary.as_str());
            ComboOption::new(msg::parent(i + 1, short, summary), (i + 1).to_string())
        })
        .collect()
}

/// Opens the dropdown for `commit`, whose parents are `parents`.
pub(crate) fn open(graph: &CommitGraph, commit: Oid, parents: &[Oid]) {
    let mut combo = Combo::new("");
    combo.set_options(parent_options(graph, parents));
    combo.select(Some("1".to_string()));
    combo.open();
    if let Ok(mut p) = POPUP.lock() {
        *p = Some(Open {
            combo,
            commit,
            close: Default::default(),
        });
    }
}

pub(crate) fn is_open() -> bool {
    POPUP
        .lock()
        .is_ok_and(|p| p.as_ref().is_some_and(|o| o.combo.is_open()))
}

/// Settles an outcome: a pick ends the popup, a closed popup is dropped.
fn settle(slot: &mut Option<Open>, outcome: ComboOutcome) -> Option<Pick> {
    let pick = match (&*slot, outcome) {
        (Some(open), ComboOutcome::Picked(value)) => value.parse().ok().map(|parent| Pick {
            commit: open.commit,
            parent,
        }),
        _ => None,
    };
    if slot.as_ref().is_some_and(|o| !o.combo.is_open()) {
        *slot = None;
    }
    pick
}

/// A key while the popup is open (it swallows them all).
pub(crate) fn on_key(key: KeyEvent) -> Option<Pick> {
    let mut slot = POPUP.lock().ok()?;
    let outcome = slot.as_mut()?.combo.on_key(key);
    settle(&mut slot, outcome)
}

/// A mouse event while the popup is open (it swallows them all).
pub(crate) fn on_mouse(mouse: MouseEvent) -> Option<Pick> {
    let mut slot = POPUP.lock().ok()?;
    let open = slot.as_mut()?;
    match mouse.kind {
        MouseEventKind::Moved | MouseEventKind::Drag(_) => {
            open.close.set_hover(mouse.column, mouse.row);
        }
        MouseEventKind::Down(MouseButton::Left) if open.close.hit(mouse.column, mouse.row) => {
            *slot = None;
            return None;
        }
        _ => {}
    }
    let outcome = open.combo.on_mouse(mouse);
    settle(&mut slot, outcome)
}

/// Paints the box and the open dropdown, last, over the panes.
pub(crate) fn render(frame: &mut Frame) {
    let Ok(mut slot) = POPUP.lock() else {
        return;
    };
    let Some(open) = slot.as_mut() else {
        return;
    };
    let area = frame.area();
    let width = 64u16.min(area.width.saturating_sub(4));
    if width < 16 || area.height < 8 {
        return;
    }
    let outer = Rect::new(
        area.x + (area.width - width) / 2,
        area.y + area.height / 3,
        width,
        4,
    );
    let buf = frame.buffer_mut();
    open.close.panel(buf, outer, msg::TITLE.get());
    dialog_kit::put(
        buf,
        outer.x + 2,
        outer.y + 1,
        msg::QUESTION.get(),
        outer.width.saturating_sub(4),
        Style::default().fg(th().ui.text).bg(th().ui.panel),
    );
    let field = Rect::new(outer.x + 2, outer.y + 2, outer.width.saturating_sub(4), 1);
    open.combo.render_field(buf, field, true, false);
    open.combo.render_popup(buf, area);
}

#[cfg(test)]
mod tests;
