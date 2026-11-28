//! Sideways scrolling of the pane that has the keyboard (`crate::hscroll`):
//! the keys (Alt+Left / Alt+Right by default, one action per pane so each can
//! be rebound) and the palette's twin land here.

use super::CommandCtx;
use crate::event_loop::*;
use crate::hscroll::{self, Dir, HPane};
use crate::keymap::term::TermKeymap;
use gitpoulpi_i18n::msg::pane_hscroll as msg;

/// The pane the keyboard is in, if it is one that scrolls sideways.
fn focused_pane(cx: &CommandCtx<'_>) -> Option<HPane> {
    match *cx.focus {
        Focus::Refs => Some(HPane::Refs(*cx.active_ref_section)),
        Focus::Status if cx.wip_all.shown() && *cx.right_panel == RightPanel::Status => None,
        Focus::Status => Some(HPane::Status(*cx.active_status_section)),
        // With a file open in the central column the Alt+arrows are that
        // view's (the image divider, ...): the Log's panes keep to the Log.
        Focus::Log if *cx.central_view != CentralView::Log => None,
        Focus::Log if *cx.commit_file_focus => Some(HPane::CommitFiles),
        Focus::Log => Some(HPane::CommitInfo),
    }
}

/// Scrolls `pane` one step; says so when it has nothing to scroll.
fn scroll(cx: &mut CommandCtx<'_>, pane: HPane, dir: Dir) {
    let step = hscroll::STEP as isize;
    let moved = hscroll::scroll_by(
        pane,
        match dir {
            Dir::Left => -step,
            Dir::Right => step,
        },
    );
    if !moved {
        *cx.last_message = Some(msg::NOTHING.get().into());
    }
}

/// The palette command: the focused pane, or a note when none scrolls.
pub(in crate::event_loop) fn scroll_focused_pane(cx: &mut CommandCtx<'_>, dir: Dir) {
    match focused_pane(cx) {
        Some(pane) => scroll(cx, pane, dir),
        None => *cx.last_message = Some(msg::NOTHING.get().into()),
    }
}

/// A scroll key in the pane that has the keyboard. `true` when it was one.
pub(in crate::event_loop) fn hscroll_key(
    cx: &mut CommandCtx<'_>,
    km: &crate::keymap::Keymap,
    key: &KeyEvent,
) -> bool {
    let Some(pane) = focused_pane(cx) else {
        return false;
    };
    let dir = if km.is(pane.key(Dir::Left), key) {
        Dir::Left
    } else if km.is(pane.key(Dir::Right), key) {
        Dir::Right
    } else {
        return false;
    };
    scroll(cx, pane, dir);
    true
}
