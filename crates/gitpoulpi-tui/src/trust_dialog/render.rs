//! Painting the dialog: every hit rect comes out of this walk.

use gitpoulpi_i18n::msg::trust_dialog as msg;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use super::body::{self, Tone};
use super::{Dialog, Painted};
use crate::dialog_kit;

const WIN: dialog_kit::Win = dialog_kit::Win::movable("trust");
use crate::scrollbar::{self, Track};
use crate::th;

/// The widest the dialog gets.
const MAX_WIDTH: u16 = 88;
/// Fewest body rows kept before the dialog gives up shrinking.
const MIN_BODY: u16 = 3;

pub(super) fn paint(d: &mut Dialog, buf: &mut Buffer, area: Rect) {
    let Some(problem) = d.current().cloned() else {
        return;
    };
    let width = MAX_WIDTH.min(area.width.saturating_sub(4));
    if width < 30 || area.height < 10 {
        d.painted = Painted::default();
        return;
    }
    let text_w = usize::from(width.saturating_sub(5));
    let lines = body::lines(&problem, text_w);
    let choices = d.choices();
    let more = d.queue.len().saturating_sub(1);
    // border, pad, body, pad, answers, pad, hint (+ the queue note)
    let fixed = 2 + 1 + 1 + choices.len() as u16 + 1 + 1 + u16::from(more > 0);
    let room = area.height.saturating_sub(2).saturating_sub(fixed);
    let body_rows = (lines.len() as u16).min(room.max(MIN_BODY));
    let height = (fixed + body_rows).min(area.height.saturating_sub(2));
    let outer = WIN.place(area, dialog_kit::centered(area, width, height));
    let body_rows = body_rows.min(outer.height.saturating_sub(fixed));

    d.close.framed(buf, outer, msg::TITLE.get(), &WIN, true);
    let plain = Style::default().fg(th().ui.text).bg(th().ui.panel);
    let muted = Style::default().fg(th().ui.muted).bg(th().ui.panel);
    let heading = Style::default()
        .fg(th().ui.accent)
        .bg(th().ui.panel)
        .add_modifier(Modifier::BOLD);

    // The body, scrolled.
    let body_rect = Rect::new(outer.x + 2, outer.y + 2, text_w as u16, body_rows);
    let max = lines.len().saturating_sub(usize::from(body_rows));
    d.scroll = d.scroll.min(max);
    for (i, l) in lines
        .iter()
        .skip(d.scroll)
        .take(usize::from(body_rows))
        .enumerate()
    {
        let style = match l.tone {
            Tone::Plain => plain,
            Tone::Heading => heading,
            Tone::Muted => muted,
        };
        dialog_kit::put(
            buf,
            body_rect.x,
            body_rect.y + i as u16,
            &l.text,
            body_rect.width,
            style,
        );
    }
    let bar_rect = Rect::new(outer.right().saturating_sub(2), body_rect.y, 1, body_rows);
    let track = Track::column(bar_rect, lines.len(), usize::from(body_rows), d.scroll);
    if let Some(track) = track {
        scrollbar::with_pointer(d.pointer, || scrollbar::render_buf(buf, track, true, false));
    }

    // The numbered answers: list rows, no default highlight.
    let mut y = body_rect.bottom() + 1;
    let mut answers = Vec::new();
    for (i, choice) in choices.iter().enumerate() {
        let selected = d.cursor == Some(i);
        let style = if selected {
            Style::default()
                .fg(th().ui.text)
                .bg(th().ui.selection)
                .add_modifier(Modifier::BOLD)
        } else {
            plain
        };
        let row = Rect::new(outer.x + 1, y, outer.width.saturating_sub(2), 1);
        dialog_kit::fill(buf, row, style);
        dialog_kit::put(
            buf,
            outer.x + 2,
            y,
            &format!("{}  {}", i + 1, body::label(&problem, *choice)),
            outer.width.saturating_sub(4),
            style,
        );
        crate::pointer_map::plain(row);
        answers.push(row);
        y += 1;
    }
    y += 1;
    dialog_kit::put(
        buf,
        outer.x + 2,
        y,
        msg::HINT.get(),
        outer.width.saturating_sub(4),
        muted,
    );
    if more > 0 {
        dialog_kit::put(
            buf,
            outer.x + 2,
            y + 1,
            &msg::more_waiting(more),
            outer.width.saturating_sub(4),
            muted,
        );
    }
    d.painted = Painted {
        window: outer,
        answers,
        body: body_rect,
        track,
    };
}
