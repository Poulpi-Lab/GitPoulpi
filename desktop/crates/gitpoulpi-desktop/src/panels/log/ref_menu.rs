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
        self.pill = Some((commit, target));
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
