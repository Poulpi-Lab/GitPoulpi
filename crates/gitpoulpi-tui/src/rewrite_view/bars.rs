//! Scrollbars of the workbench's lists.
//!
//! Every list that can be longer than its room paints the shared bar
//! (`scrollbar.rs`) in the pane's right border column, beside its rows, and
//! registers it here: a press and a drag on it move the list's first row, the
//! wheel over the pane scrolls it, and the keys keep moving the cursor (the
//! list follows it). One registry per paint, so the bar a user sees and the
//! one a click hits are the same `Track`.
//!
//! A list shows `top..top + room`. By default `top` follows the cursor; once the
//! bar or the wheel moved it, it stays where it was put for as long as the
//! cursor stays where it was (`Paint::anchors`): any key that moves the cursor
//! brings the list back to it, with no hook in each of those keys.

use ratatui::layout::Rect;
use ratatui::style::Style;

use super::render::wrap;
use super::render::{RewriteHit, Ui};
use super::{Focus, RewriteView};
use crate::scrollbar::{self, Track, TrackPress};

/// A list with a bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BarId {
    Rules,
    Detail,
    Commits,
    /// The Refs tab's two lists: the refs that move and the protected ones.
    RefsMoving,
    RefsProtected,
    Secrets,
    Identities,
    Size,
    Push,
    /// A tab that is wrapped text (Summary, Impact, a refusal).
    Text,
    /// The blob diff overlay's file list and diff.
    DrillFiles,
    DrillDiff,
    /// The purge screen's body.
    Purge,
    /// The host how-to's text.
    Howto,
    /// The script editor's text, parameter rows and result table.
    ScriptText,
    ScriptParams,
    ScriptRows,
    /// An open dropdown's rows.
    Popup,
    /// The "Merge into..." dialog's candidates.
    IdentPick,
    /// The script AI dialog's "what is sent" pane.
    AiSent,
}

/// One bar as painted this frame.
#[derive(Clone, Copy)]
pub(super) struct BarReg {
    pub id: BarId,
    pub track: Track,
    pub content: usize,
    pub viewport: usize,
    pub cursor: Option<usize>,
}

impl BarId {
    /// The pane whose wheel scrolls this list.
    pub(crate) fn pane(self) -> Focus {
        match self {
            BarId::Rules => Focus::Rules,
            BarId::Detail => Focus::Detail,
            BarId::DrillFiles | BarId::DrillDiff => Focus::Preview,
            _ => Focus::Preview,
        }
    }
}

impl RewriteView {
    /// Whether `id`'s list was scrolled by its bar or the wheel and its cursor
    /// has not moved since: it then stays where it was put. Otherwise the
    /// held position is dropped and the list follows the cursor again.
    pub(super) fn bar_free(&self, id: BarId, cursor: Option<usize>) -> bool {
        let mut anchors = self.paint.anchors.borrow_mut();
        let held = anchors.iter().position(|(a, _)| *a == id);
        let free = match (held, cursor) {
            (Some(i), c) => anchors[i].1 == c.unwrap_or(usize::MAX),
            (None, None) => true,
            _ => false,
        };
        if !free {
            if let Some(i) = held {
                anchors.remove(i);
            }
        }
        free
    }

    /// The first row of `id`'s list for this paint: kept where the bar or the
    /// wheel left it while `cursor` has not moved, else following `cursor`.
    /// `cursor == None` is a list without one (it never follows).
    pub(super) fn settle(
        &self,
        id: BarId,
        cursor: Option<usize>,
        top: usize,
        room: usize,
        len: usize,
    ) -> usize {
        let free = self.bar_free(id, cursor);
        let mut top = top;
        if !free {
            if let Some(cursor) = cursor {
                if cursor < top {
                    top = cursor;
                } else if room > 0 && cursor >= top + room {
                    top = cursor + 1 - room;
                }
            }
        }
        top.min(len.saturating_sub(room))
    }

    /// Paints `id`'s bar over `x`, rows `y..y + h` (the pane's border column),
    /// when `len` rows do not fit `viewport`, and registers it.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn paint_bar(
        &self,
        ui: &mut Ui<'_>,
        id: BarId,
        column: (u16, u16, u16),
        len: usize,
        viewport: usize,
        top: usize,
        cursor: Option<usize>,
    ) {
        let hot = ui.hovered == Some(RewriteHit::Bar(id));
        self.paint_bar_on(
            ui.buf,
            &mut ui.fine,
            hot,
            id,
            column,
            len,
            viewport,
            top,
            cursor,
        );
    }

    /// [`Self::paint_bar`] for an overlay painted on a bare buffer, which
    /// collects its own hit list.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn paint_bar_on(
        &self,
        buf: &mut ratatui::buffer::Buffer,
        hits: &mut Vec<(Rect, RewriteHit)>,
        hot: bool,
        id: BarId,
        column: (u16, u16, u16),
        len: usize,
        viewport: usize,
        top: usize,
        cursor: Option<usize>,
    ) {
        let (x, y, h) = column;
        let Some(track) = Track::column(Rect::new(x, y, 1, h), len, viewport, top) else {
            return;
        };
        let hit = RewriteHit::Bar(id);
        scrollbar::render_buf(buf, track, true, hot);
        hits.push((track.rect, hit));
        self.paint.bars.borrow_mut().push(BarReg {
            id,
            track,
            content: len,
            viewport,
            cursor,
        });
    }

    fn bar_reg(&self, id: BarId) -> Option<BarReg> {
        self.paint
            .bars
            .borrow()
            .iter()
            .find(|r| r.id == id)
            .copied()
    }

    /// Where `id`'s list is now: the live value, not what the last paint saw,
    /// so a burst of wheel notches between two paints adds up.
    fn live_top(&self, reg: &BarReg) -> usize {
        match reg.id {
            BarId::Rules => self.paint.rules_top.get(),
            BarId::Detail => self.paint.detail_top.get(),
            BarId::Commits => self.paint.rows_top.get(),
            BarId::RefsMoving => self.paint.refs_moving_top.get(),
            BarId::RefsProtected => self.paint.refs_top.get(),
            BarId::Secrets => self.paint.secrets_top.get(),
            BarId::Identities => self.identities.top.get(),
            BarId::Size => self.paint.size_top.get(),
            BarId::Push => self.paint.push_top.get(),
            BarId::Text => match self.paint.text_top.get() {
                Some((tab, top)) if tab == self.tab => top,
                _ => 0,
            },
            BarId::DrillFiles => self.paint.drill_top.get(),
            BarId::DrillDiff => self.drill.as_ref().map_or(0, |d| d.scroll),
            BarId::Purge => self.purge.as_ref().map_or(0, |s| s.scroll.get()),
            BarId::Howto => self.host.howto.as_ref().map_or(0, |o| o.scroll),
            BarId::ScriptText => self
                .script
                .editor
                .as_ref()
                .map_or(0, |ed| ed.caret.top.get()),
            BarId::ScriptParams => self.paint.script_params_top.get(),
            BarId::ScriptRows => self.script.editor.as_ref().map_or(0, |ed| ed.top),
            BarId::Popup => self.paint.popup_top.get(),
            BarId::IdentPick => self.paint.ident_pick_top.get(),
            BarId::AiSent => self.script.ai.dialog.as_ref().map_or(0, |d| d.scroll),
        }
    }

    /// Puts `id`'s list at `top` and keeps it there while the cursor stays.
    fn set_bar_top(&mut self, reg: &BarReg, top: usize) {
        let top = top.min(reg.content.saturating_sub(reg.viewport));
        match reg.id {
            BarId::Rules => self.paint.rules_top.set(top),
            BarId::Detail => self.paint.detail_top.set(top),
            BarId::Commits => self.paint.rows_top.set(top),
            BarId::RefsMoving => self.paint.refs_moving_top.set(top),
            BarId::RefsProtected => self.paint.refs_top.set(top),
            BarId::Secrets => self.paint.secrets_top.set(top),
            BarId::Identities => self.identities.top.set(top),
            BarId::Size => self.paint.size_top.set(top),
            BarId::Push => self.paint.push_top.set(top),
            BarId::Text => self.paint.text_top.set(Some((self.tab, top))),
            BarId::DrillFiles => self.paint.drill_top.set(top),
            BarId::ScriptText => {
                let text = self.script_text().map(str::to_string);
                if let (Some(text), Some(ed)) = (text, self.script.editor.as_mut()) {
                    ed.caret.scroll_lines_to(&text, top, reg.viewport);
                }
            }
            BarId::AiSent => {
                if let Some(d) = self.script.ai.dialog.as_mut() {
                    d.scroll = top;
                }
            }
            BarId::IdentPick => self.paint.ident_pick_top.set(top),
            BarId::Popup => self.paint.popup_top.set(top),
            BarId::ScriptParams => self.paint.script_params_top.set(top),
            BarId::ScriptRows => {
                if let Some(ed) = self.script.editor.as_mut() {
                    ed.top = top;
                }
            }
            BarId::Howto => {
                if let Some(o) = self.host.howto.as_mut() {
                    o.scroll = top;
                }
            }
            BarId::Purge => {
                if let Some(s) = self.purge.as_ref() {
                    s.scroll.set(top);
                }
            }
            BarId::DrillDiff => {
                if let Some(d) = self.drill.as_mut() {
                    d.scroll = top;
                }
            }
        }
        let mut anchors = self.paint.anchors.borrow_mut();
        anchors.retain(|(a, _)| *a != reg.id);
        anchors.push((reg.id, reg.cursor.unwrap_or(usize::MAX)));
    }

    /// A press on `id`'s bar: an arrow steps, the thumb is held, the bare
    /// track jumps the thumb under the pointer (and keeps dragging).
    pub(super) fn bar_press(&mut self, id: BarId, row: u16) {
        let Some(reg) = self.bar_reg(id) else {
            return;
        };
        self.focus = reg.id.pane();
        let pos = row.saturating_sub(reg.track.rect.y);
        match reg.track.press(pos) {
            TrackPress::Arrow(step) => {
                let top = (self.live_top(&reg) as isize + step).max(0) as usize;
                self.set_bar_top(&reg, top);
            }
            TrackPress::Grab { grab, jump } => {
                self.paint.bar_drag.set(Some((id, grab)));
                if jump {
                    self.bar_drag(row);
                }
            }
        }
    }

    /// The pointer moved with the button down: moves the held thumb. `false`
    /// when no thumb is held.
    pub(crate) fn bar_drag(&mut self, row: u16) -> bool {
        let Some((id, grab)) = self.paint.bar_drag.get() else {
            return false;
        };
        let Some(reg) = self.bar_reg(id) else {
            self.paint.bar_drag.set(None);
            return false;
        };
        let pos = i32::from(row) - i32::from(reg.track.rect.y);
        let top = reg.track.drag_offset(reg.content, reg.viewport, grab, pos);
        self.set_bar_top(&reg, top);
        true
    }

    /// The button went up: the thumb is let go. `true` if one was held.
    pub(crate) fn bar_release(&mut self) -> bool {
        self.paint.bar_drag.replace(None).is_some()
    }

    /// The wheel over `pane`: scrolls its list by `delta` rows without
    /// touching the selection. `false` when the pane's list fits (no bar), so
    /// the caller falls back to moving the cursor.
    /// The wheel over `id`'s own bar or list.
    pub(crate) fn wheel_id(&mut self, id: BarId, delta: isize) -> bool {
        let Some(reg) = self.bar_reg(id) else {
            return false;
        };
        self.set_bar_top(&reg, (self.live_top(&reg) as isize + delta).max(0) as usize);
        true
    }

    pub(crate) fn wheel_bar(&mut self, pane: Focus, row: u16, delta: isize) -> bool {
        let reg = {
            let bars = self.paint.bars.borrow();
            let mut of_pane = bars.iter().filter(|r| r.id.pane() == pane);
            // Two lists in one pane (the Refs tab): the one under the pointer.
            let under = bars
                .iter()
                .filter(|r| r.id.pane() == pane)
                .find(|r| row >= r.track.rect.y && row < r.track.rect.bottom())
                .copied();
            under.or_else(|| of_pane.next().copied())
        };
        let Some(reg) = reg else {
            return false;
        };
        let top = (self.live_top(&reg) as isize + delta).max(0) as usize;
        self.set_bar_top(&reg, top);
        true
    }
}

impl RewriteView {
    /// `lines` (indent, text, style) wrapped to the room beside the bar and
    /// shown from the scroll the keys, the wheel or the bar set, with the bar
    /// when they do not all fit. Returns the row after the last line shown.
    pub(super) fn paint_text(
        &self,
        ui: &mut Ui<'_>,
        body: Rect,
        lines: &[(u16, String, Style)],
    ) -> u16 {
        let width = (body.width as usize).saturating_sub(2);
        let mut rows: Vec<(u16, String, Style)> = Vec::new();
        for (indent, text, style) in lines {
            let room = width.saturating_sub(*indent as usize).max(1);
            for line in wrap(text, room) {
                rows.push((*indent, line, *style));
            }
        }
        let room = body.height as usize;
        let held = match self.paint.text_top.get() {
            Some((tab, top)) if tab == self.tab => top,
            _ => 0,
        };
        let top = self.settle(BarId::Text, None, held, room, rows.len());
        self.paint.text_top.set(Some((self.tab, top)));
        self.paint_bar(
            ui,
            BarId::Text,
            (body.right(), body.y, body.height),
            rows.len(),
            room,
            top,
            None,
        );
        let mut y = body.y;
        for (indent, line, style) in rows.iter().skip(top).take(room) {
            ui.text(
                body.x + 1 + indent,
                y,
                body.width.saturating_sub(1 + indent),
                line,
                *style,
            );
            y += 1;
        }
        y
    }

    /// The keys' twin of the wheel on a text tab: scrolls by `delta` lines.
    pub(super) fn scroll_text(&self, delta: isize) {
        let held = match self.paint.text_top.get() {
            Some((tab, top)) if tab == self.tab => top,
            _ => 0,
        };
        let top = (held as isize + delta).max(0) as usize;
        self.paint.text_top.set(Some((self.tab, top)));
    }
}

impl RewriteView {
    /// Scrolls `id`'s list by `pages` pages (a key). `false` when it has no bar
    /// on screen.
    pub(super) fn page_bar(&mut self, id: BarId, pages: isize) -> bool {
        let Some(reg) = self.bar_reg(id) else {
            return false;
        };
        let step = pages * reg.viewport.max(1) as isize;
        self.set_bar_top(&reg, (self.live_top(&reg) as isize + step).max(0) as usize);
        true
    }
}
