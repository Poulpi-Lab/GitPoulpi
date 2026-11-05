//! The "What is sent" pane both ✦ AI dialogs of the workbench share (Create
//! Rules, Write a script): a read-only preview tinted apart from the editable
//! fields, **collapsed until the user opens it**, with its own pressable and
//! draggable scrollbar on its right edge. One implementation, so the two
//! dialogs cannot drift apart: geometry, tint, keys and the bar's mouse
//! handling live here; each dialog only supplies its lines and its hit
//! targets.

use crossterm::event::{KeyCode, MouseEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};

use gitpoulpi_i18n::msg::rewrite_ai as t;

use super::render::RewriteHit;
use crate::dialog_kit;
use crate::scrollbar::{BarDrag, Track};
use crate::th;

/// The widest an AI dialog grows; it is as tall as its content needs.
pub(super) const MAX_WIDTH: u16 = 100;
/// The least rows the open pane gets.
const MIN_PANE: u16 = 4;

/// The tint of the pane: the dialog's own panel, sunk a step, so the
/// read-only preview reads apart from the editable fields (which sit
/// lighter) in dark and light themes alike. A theme that follows the
/// terminal's own colors has nothing to sink and uses its hover tone.
pub(super) fn sent_bg() -> Color {
    let panel = th().ui.panel;
    match panel {
        Color::Rgb(..) => {
            let sunk = crate::lighten(panel, -14);
            if sunk == panel {
                crate::lighten(panel, 14)
            } else {
                sunk
            }
        }
        _ => th().ui.hover,
    }
}

/// The dialog's rectangle: centred in `room`, at most [`MAX_WIDTH`] wide and
/// `inner_h` rows of content tall (plus the two border rows).
pub(super) fn centered(room: Rect, inner_h: u16) -> Rect {
    let width = room.width.min(MAX_WIDTH);
    let height = (inner_h + 2).min(room.height);
    Rect::new(
        room.x + (room.width - width) / 2,
        room.y + (room.height - height) / 2,
        width,
        height,
    )
}

/// The state of the pane, in the dialog's state (`Default` = collapsed).
#[derive(Default)]
pub(super) struct SentPane {
    /// The body is shown.
    pub open: bool,
    /// First line shown.
    pub top: usize,
    rows: std::cell::Cell<usize>,
    height: std::cell::Cell<usize>,
    bar: std::cell::Cell<Option<Track>>,
    drag: BarDrag,
}

impl SentPane {
    pub(super) fn toggle(&mut self) {
        self.open = !self.open;
        self.top = 0;
    }

    /// Rows the open body asks for beyond the fixed ones, at least.
    pub(super) fn least_rows(&self) -> u16 {
        if self.open {
            MIN_PANE
        } else {
            0
        }
    }

    /// Rows the body takes: its lines, at least [`MIN_PANE`], at most `room`;
    /// none while collapsed.
    pub(super) fn body_rows(&self, lines: usize, room: u16) -> u16 {
        if self.open {
            (lines.min(u16::MAX as usize) as u16).max(MIN_PANE).min(room)
        } else {
            0
        }
    }

    /// Scrolls by `delta` lines (clamped to the last paint's rows).
    pub(super) fn scroll_by(&mut self, delta: isize) {
        let max = self.rows.get().saturating_sub(self.height.get()) as isize;
        self.top = (self.top as isize + delta).clamp(0, max) as usize;
    }

    /// The scroll a key asks for while the pane has the focus.
    pub(super) fn key_step(&self, code: KeyCode) -> Option<isize> {
        let page = self.height.get().max(2) as isize - 1;
        Some(match code {
            KeyCode::Up | KeyCode::Char('k') => -1,
            KeyCode::Down | KeyCode::Char('j') => 1,
            KeyCode::PageUp => -page,
            KeyCode::PageDown | KeyCode::Char(' ') => page,
            KeyCode::Home => isize::MIN / 2,
            KeyCode::End => isize::MAX / 2,
            _ => return None,
        })
    }

    /// A press, drag or release on the bar. Whether the pane scrolled (a
    /// press on the bar is then also a focus).
    pub(super) fn bar_mouse(&mut self, mouse: MouseEvent) -> bool {
        let new = self
            .drag
            .on_mouse(self.bar.get(), mouse, self.rows.get(), self.top);
        match new {
            Some(new) => {
                self.top = new;
                self.scroll_by(0);
                true
            }
            None => false,
        }
    }

    /// Paints the header row: the disclosure mark, the label, the key hint.
    pub(super) fn paint_head(&self, buf: &mut Buffer, head: Rect, focused: bool, hot: bool) {
        let bg = sent_bg();
        let head_bg = if hot { crate::lighten(bg, 10) } else { bg };
        dialog_kit::fill(buf, head, Style::default().bg(head_bg));
        let mark = if self.open {
            crate::fold_mark::EXPANDED
        } else {
            crate::fold_mark::COLLAPSED
        };
        let label_style = Style::default()
            .fg(if focused { th().ui.accent } else { th().ui.text })
            .bg(head_bg)
            .add_modifier(Modifier::BOLD);
        let lr = dialog_kit::put(
            buf,
            head.x + 1,
            head.y,
            &format!("{mark} {}", t::DLG_SENT_LABEL.get()),
            head.width.saturating_sub(2),
            label_style,
        );
        let hint = t::DLG_SENT_KEY.get();
        let hint_w = hint.chars().count() as u16;
        if head.width > lr.width + hint_w + 4 {
            dialog_kit::put(
                buf,
                head.right() - hint_w - 1,
                head.y,
                hint,
                hint_w,
                Style::default().fg(th().ui.muted).bg(head_bg),
            );
        }
    }

    /// Forgets the last paint (collapsed, or not shown).
    pub(super) fn paint_none(&self) {
        self.bar.set(None);
        self.rows.set(0);
        self.height.set(0);
    }

    /// Paints the scrolled lines in `pane` (text, heading) with the bar on
    /// its right column, and adds the bar's and the body's hit targets
    /// (bar first: inner-first). `tinted` is the preview look; the result
    /// list uses the plain dialog background.
    pub(super) fn paint_body(
        &self,
        buf: &mut Buffer,
        pane: Rect,
        lines: &[(String, bool)],
        tinted: bool,
        hits: &mut Vec<(Rect, RewriteHit)>,
        (bar_hit, body_hit): (RewriteHit, RewriteHit),
    ) {
        let rows = pane.height as usize;
        self.rows.set(lines.len());
        self.height.set(rows);
        let top = self.top.min(lines.len().saturating_sub(rows));
        let bg = if tinted { sent_bg() } else { th().ui.panel };
        dialog_kit::fill(buf, pane, Style::default().bg(bg));
        for (i, (text, heading)) in lines.iter().skip(top).take(rows).enumerate() {
            let mut style = Style::default().fg(th().ui.text).bg(bg);
            if *heading {
                style = style.add_modifier(Modifier::BOLD);
            }
            dialog_kit::put(
                buf,
                pane.x + 1,
                pane.y + i as u16,
                text,
                pane.width.saturating_sub(3),
                style,
            );
        }
        // Its own scrollbar: the pane's right column, rows of the body.
        let bar = Track::vertical(
            Rect::new(pane.x, pane.y.saturating_sub(1), pane.width, pane.height + 2),
            lines.len(),
            top,
        );
        self.bar.set(bar);
        if let Some(track) = bar {
            crate::scrollbar::render_buf(buf, track, true, false);
            hits.push((track.rect, bar_hit));
        }
        hits.push((pane, body_hit));
    }
}

/// The width the lines wrap to in a pane `wide` columns wide.
pub(super) fn wrap_width(wide: u16) -> usize {
    wide.saturating_sub(3) as usize
}
