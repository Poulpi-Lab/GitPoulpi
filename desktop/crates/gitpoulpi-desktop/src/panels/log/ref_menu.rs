//! Ref pills as a place for the ref menu: a right click opens it, a double click checks the branch out,
//! and a click picks the pill, so the menu key opens the same menu from the keyboard (`picked_pill`).

use gpui_kit::{
    div, px, App, ClickEvent, InteractiveElement as _, IntoElement, MouseButton,
    ParentElement as _, Pixels, Point, StatefulInteractiveElement as _, Styled as _, Window,
};

use super::pills::{Pill, PillKind};
use gitpoulpi_core::domain::Oid;

use super::LogPanel;
use crate::app::workspace_of;
use crate::ref_target::RefTarget;

/// The target a pill stands for.
pub fn target_of(p: &Pill) -> RefTarget {
    match p.kind {
        PillKind::Local => RefTarget::Local(p.full.clone()),
        PillKind::Remote => RefTarget::Remote(p.full.clone()),
        PillKind::Tag => RefTarget::Tag(p.full.clone()),
    }
}

/// Wraps a pill so it answers the mouse: right click for the menu, double click to check out.
pub(super) fn interactive(
    ix: usize,
    commit: Oid,
    pill: &Pill,
    element: impl IntoElement,
) -> impl IntoElement {
    let target = target_of(pill);
    let double = target.clone();
    let picked = target.clone();
    div()
        .min_w_0()
        .id(("log-pill", ix))
        .cursor_pointer()
        .on_mouse_down(MouseButton::Right, move |ev, window, cx| {
            cx.stop_propagation();
            let (target, at) = (target.clone(), ev.position);
            if let Some(ws) = workspace_of(cx) {
                ws.update(cx, |ws, cx| ws.open_ref_menu(target, at, window, cx));
            }
        })
        .on_click(move |ev: &ClickEvent, window: &mut Window, cx: &mut App| {
            if ev.click_count() < 2 {
                if let Some(ws) = workspace_of(cx) {
                    let picked = picked.clone();
                    ws.update(cx, |ws, cx| ws.pick_log_pill(commit, picked, cx));
                }
            } else {
                let target = double.clone();
                if let Some(ws) = workspace_of(cx) {
                    ws.update(cx, |ws, cx| {
                        ws.run_ref_op(
                            crate::commands::refs_ops::RefOp::Checkout,
                            Some(target),
                            window,
                            cx,
                        )
                    });
                }
            }
        })
        .child(element)
}

impl LogPanel {
    /// Remembers the clicked pill of `commit`.
    pub fn pick_pill(&mut self, commit: Oid, target: RefTarget) {
        self.pill_ix = (self.selected_commit() == Some(commit))
            .then(|| {
                self.pills_of_selected()
                    .iter()
                    .position(|p| target_of(p) == target)
            })
            .flatten();
        self.pill = Some((commit, target));
    }

    /// The pills of the selected commit, in the order the Ref column lists them.
    pub fn pills_of_selected(&self) -> Vec<Pill> {
        let Some(oid) = self.selected_commit() else {
            return Vec::new();
        };
        self.data
            .refs_by_commit
            .get(&oid)
            .map(|idxs| super::pills::pills_for(&self.data.refs, idxs))
            .unwrap_or_default()
    }

    /// The commit and pill the pill cursor is on, for painting.
    pub fn pill_cursor(&self) -> Option<(Oid, usize)> {
        let oid = self.selected_commit()?;
        self.pill_ix.map(|i| (oid, i))
    }

    /// Moves the pill cursor to the next (or previous) ref of the selected commit; it wraps. The menu key
    /// then opens that ref's menu, as a right click on the pill does.
    pub fn step_pill(&mut self, forward: bool, cx: &mut gpui_kit::Context<Self>) {
        let Some(oid) = self.selected_commit() else {
            return;
        };
        let pills = self.pills_of_selected();
        if pills.is_empty() {
            self.say(crate::strings::NO_REFS_ON_COMMIT.to_string(), cx);
            return;
        }
        let n = pills.len();
        let next = match (self.pill_ix.filter(|i| *i < n), forward) {
            (None, true) => 0,
            (None, false) => n - 1,
            (Some(i), true) => (i + 1) % n,
            (Some(i), false) => (i + n - 1) % n,
        };
        self.pill_ix = Some(next);
        self.pill = Some((oid, target_of(&pills[next])));
        cx.notify();
    }

    /// Drops the pill cursor (Escape).
    pub fn clear_pill_cursor(&mut self, cx: &mut gpui_kit::Context<Self>) {
        self.pill_ix = None;
        self.pill = None;
        cx.notify();
    }

    /// The pill picked on the selected row, if any.
    pub fn picked_pill(&self) -> Option<RefTarget> {
        let selected = self.selected_commit()?;
        self.pill
            .as_ref()
            .filter(|(oid, _)| *oid == selected)
            .map(|(_, t)| t.clone())
    }

    /// Where a menu opened from the keyboard appears (near the Ref column of the Log).
    pub fn selected_row_origin(&self) -> Point<Pixels> {
        Point::new(px(320.0), px(160.0))
    }
}
