//! Painting the picker. Every hit rect is captured from the walk that paints
//! (`self.hits`), never recomputed.

use std::time::Instant;

use gitpoulpi_i18n::msg;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use super::{Focus, Hit, MetaPicker, Status, SPIN_AFTER};
use crate::dialog_kit::{self, ButtonState};
use crate::keymap::Action as Bind;
use crate::th;

impl MetaPicker {
    /// Replaces the line under the list (an error, or nothing).
    pub(crate) fn say(&mut self, text: String) {
        self.message = (!text.is_empty()).then_some(text);
    }

    /// The popup's rectangle inside `area`.
    pub(crate) fn rect_in(&self, area: Rect) -> Rect {
        let (w, h) = if self.field.is_single() {
            (56, 11)
        } else {
            (66, 22)
        };
        dialog_kit::centered(area, w.min(area.width.saturating_sub(2)), h)
    }

    /// Paints the popup centered in `area`. `sending` is when the edit went
    /// out, while it is on its way.
    pub(crate) fn render(&mut self, area: Rect, buf: &mut Buffer, sending: Option<Instant>) {
        self.edit.sync((!self.field.is_single()).then_some(()));
        self.edit.begin_frame();
        let rect = self.rect_in(area);
        self.frame = rect;
        self.close.panel(buf, rect, self.field.title());
        let mut hits: Vec<(Rect, Hit)> = Vec::new();
        let inner = Rect::new(
            rect.x + 2,
            rect.y + 1,
            rect.width.saturating_sub(4),
            rect.height.saturating_sub(2),
        );
        if inner.width < 24 || inner.height < 7 {
            self.hits = hits;
            return;
        }
        let muted = Style::default().fg(th().ui.muted).bg(th().ui.panel);
        let mut y = inner.y;
        dialog_kit::put(buf, inner.x, y, &self.target.caption, inner.width, muted);
        y += 2;

        let bottom = inner.bottom();
        let msg_y = bottom.saturating_sub(2);
        let button_y = bottom.saturating_sub(1);

        if self.field.is_single() {
            let field = Rect::new(inner.x, y, inner.width, 1);
            self.combo.render_field(
                buf,
                field,
                self.focus == Focus::Combo,
                self.hover == Some(Hit::Combo),
            );
            hits.push((field, Hit::Combo));
            self.bar = None;
        } else {
            let filter = Rect::new(inner.x, y, inner.width, 1);
            dialog_kit::text_input(
                buf,
                filter,
                &self.query,
                msg::meta_picker::FILTER_PLACEHOLDER.get(),
                self.focus == Focus::List,
                self.hover == Some(Hit::Filter),
                (&self.edit, ()),
            );
            hits.push((filter, Hit::Filter));
            y += 2;
            self.paint_rows(buf, inner, y, msg_y, &mut hits);
        }

        // The line under the list: the problem, the call in flight, or how
        // much is chosen.
        let (note, style) = if let Some(message) = &self.message {
            (
                message.clone(),
                Style::default().fg(th().status.danger).bg(th().ui.panel),
            )
        } else if let Some(since) = sending {
            let spin = if since.elapsed() >= SPIN_AFTER {
                format!(
                    "{} {}",
                    crate::activity::spinner_frame(since.elapsed()),
                    msg::meta_picker::SAVING.get()
                )
            } else {
                String::new()
            };
            (spin, Style::default().fg(th().ui.accent).bg(th().ui.panel))
        } else if !self.field.is_single() && self.status == Status::Ready {
            (msg::meta_picker::selected(self.checked.len() as u64), muted)
        } else {
            (String::new(), muted)
        };
        dialog_kit::put(buf, inner.x, msg_y, &note, inner.width, style);

        // Buttons, bottom right: Cancel, Apply.
        let cancel_label = msg::meta_picker::CANCEL.get();
        let apply_label = msg::meta_picker::APPLY.get();
        let apply_w = dialog_kit::button_width(apply_label);
        let cancel_w = dialog_kit::button_width(cancel_label);
        let apply_x = inner.right().saturating_sub(apply_w);
        let cancel_x = apply_x.saturating_sub(cancel_w + 1);
        let cancel = dialog_kit::button(
            buf,
            cancel_x,
            button_y,
            cancel_label,
            ButtonState::Normal {
                hovered: self.hover == Some(Hit::Cancel),
                focused: self.focus == Focus::Cancel,
            },
        );
        hits.push((cancel, Hit::Cancel));
        let apply = dialog_kit::button(
            buf,
            apply_x,
            button_y,
            apply_label,
            if !self.can_apply() || sending.is_some() {
                ButtonState::Disabled
            } else {
                ButtonState::Primary {
                    hovered: self.hover == Some(Hit::Apply),
                    focused: self.focus == Focus::Apply,
                }
            },
        );
        hits.push((apply, Hit::Apply));
        let apply_key = dialog_kit::key_label(Bind::DialogSubmit);
        let _ = apply_key;
        let mut hints: Vec<(&str, &str)> = Vec::new();
        if !self.field.is_single() {
            hints.push((
                msg::meta_picker::KEY_SPACE.get(),
                msg::meta_picker::HINT_TOGGLE.get(),
            ));
        }
        hints.push((
            msg::meta_picker::KEY_ENTER.get(),
            msg::meta_picker::HINT_APPLY.get(),
        ));
        dialog_kit::hints(
            buf,
            inner.x,
            button_y,
            cancel_x.saturating_sub(inner.x + 1),
            &hints,
        );
        crate::pointer_map::from_hits(&hits, |h| match h {
            Hit::Filter => Some(crate::pointer_map::Kind::Field),
            // A value row is registered by its own paint (`row_band`), under
            // its checkbox mark.
            Hit::Row(_) => None,
            _ => Some(crate::pointer_map::Kind::Hand),
        });
        self.hits = hits;
        // The milestone's open list last, over everything else.
        if self.field.is_single() {
            self.combo.render_popup(buf, buf.area);
        }
    }

    fn paint_rows(
        &mut self,
        buf: &mut Buffer,
        inner: Rect,
        top: u16,
        msg_y: u16,
        hits: &mut Vec<(Rect, Hit)>,
    ) {
        let muted = Style::default().fg(th().ui.muted).bg(th().ui.panel);
        let rows = (msg_y.saturating_sub(top) as usize).max(1);
        self.rows_visible = rows;
        if self.followed != (self.cursor, rows) {
            self.followed = (self.cursor, rows);
            self.keep_visible();
        }
        self.offset = self.offset.min(self.shown.len().saturating_sub(rows));
        // Leave the rightmost column to the scrollbar.
        let row_w = inner.width.saturating_sub(1);
        match &self.status {
            Status::Loading => {
                if self.since.elapsed() >= SPIN_AFTER {
                    dialog_kit::put(
                        buf,
                        inner.x,
                        top,
                        &format!(
                            "{} {}",
                            crate::activity::spinner_frame(self.since.elapsed()),
                            msg::meta_picker::LOADING.get()
                        ),
                        inner.width,
                        Style::default().fg(th().ui.accent).bg(th().ui.panel),
                    );
                }
                self.bar = None;
                return;
            }
            Status::Failed(_) => {
                self.bar = None;
                return;
            }
            Status::Ready => {}
        }
        if self.shown.is_empty() {
            let text = if self.items.is_empty() {
                msg::meta_picker::EMPTY.get()
            } else {
                msg::meta_picker::NO_MATCH.get()
            };
            dialog_kit::put(buf, inner.x, top, text, inner.width, muted);
            self.bar = None;
            return;
        }
        for (slot, shown_index) in (self.offset..self.shown.len()).take(rows).enumerate() {
            let item = &self.items[self.shown[shown_index]];
            let y = top + slot as u16;
            let band = Rect::new(inner.x, y, row_w, 1);
            let highlighted = shown_index == self.cursor && self.focus == Focus::List;
            let bg = dialog_kit::row_bg(highlighted, self.hover == Some(Hit::Row(shown_index)));
            dialog_kit::row_band(buf, band, bg);
            let checked = self.checked.contains(&item.value);
            let mut x = inner.x;
            let mark = dialog_kit::put(
                buf,
                x,
                y,
                if checked { "[x]" } else { "[ ]" },
                3,
                Style::default().fg(th().ui.text).bg(bg),
            );
            // The checkbox keeps the hand inside the arrow row.
            crate::pointer_map::hand(mark);
            x = mark.right() + 1;
            if let Some((text, color)) = &item.badge {
                let r = dialog_kit::put(
                    buf,
                    x,
                    y,
                    text,
                    4,
                    Style::default()
                        .fg(*color)
                        .bg(bg)
                        .add_modifier(Modifier::BOLD),
                );
                x = r.right() + 1;
            }
            if let Some(color) = item.color {
                dialog_kit::fill(buf, Rect::new(x, y, 2, 1), Style::default().bg(color));
                x += 3;
            }
            let room = band.right().saturating_sub(x);
            let label = dialog_kit::put(
                buf,
                x,
                y,
                &item.label,
                room,
                Style::default()
                    .fg(th().ui.text)
                    .bg(bg)
                    .add_modifier(if checked {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            );
            if !item.detail.is_empty() {
                let dx = label.right() + 2;
                dialog_kit::put(
                    buf,
                    dx,
                    y,
                    &item.detail,
                    band.right().saturating_sub(dx),
                    Style::default().fg(th().ui.muted).bg(bg),
                );
            }
            hits.push((band, Hit::Row(shown_index)));
        }
        // Overflowing rows get the shared scrollbar over the right border.
        let outer = Rect::new(
            inner.x - 2,
            top.saturating_sub(1),
            inner.width + 4,
            rows as u16 + 2,
        );
        self.bar = crate::scrollbar::Track::vertical(outer, self.shown.len(), self.offset);
        if let Some(track) = self.bar {
            crate::scrollbar::render_buf(buf, track, true, false);
        }
    }
}
