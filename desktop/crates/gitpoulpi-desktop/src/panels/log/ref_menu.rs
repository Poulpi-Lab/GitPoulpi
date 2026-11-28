//! Ref pills as a place for the ref menu: a right click opens it, a double click checks the branch out,
//! and the keyboard reaches the same menu for the selected commit's first pill (`first_ref_target`).

use gpui_kit::{
    div, px, App, ClickEvent, InteractiveElement as _, IntoElement, MouseButton,
    ParentElement as _, Pixels, Point, StatefulInteractiveElement as _, Styled as _, Window,
};

use super::pills::{pills_for, Pill, PillKind};
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
pub(super) fn interactive(ix: usize, pill: &Pill, element: impl IntoElement) -> impl IntoElement {
    let target = target_of(pill);
    let double = target.clone();
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
            if ev.click_count() >= 2 {
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
    /// The first ref pill of the selected commit, as a target.
    pub fn first_ref_target(&self) -> Option<RefTarget> {
        let oid = self.selected_commit()?;
        let idxs = self.data.refs_by_commit.get(&oid)?;
        pills_for(&self.data.refs, idxs).first().map(target_of)
    }

    /// Where a menu opened from the keyboard appears (near the Ref column of the Log).
    pub fn selected_row_origin(&self) -> Point<Pixels> {
        Point::new(px(320.0), px(160.0))
    }
}
