//! The detail card and the right-click menu: one popup painter. The card
//! opens on hover after a short dwell (or pinned with its key) and shows the
//! hunk's commit in full; the menu lists the same actions where the
//! pointer is. Their hit rects go to the caller as `popup_hits`, which it
//! puts first in the hit list so nothing behind a popup hovers.

use crate::wide_edge::Clear;
use gitpoulpi_core::domain::BlameLine;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders, Widget};

use super::layout::Layout;
use super::paint::{BlameEnv, Drawn};
use super::{BlameHit, BlameOp, BlameView};
use crate::cell_text::cells;
use crate::th;

/// The card's width bounds, borders included.
const MIN_W: u16 = 38;
const MAX_W: u16 = 64;

struct Popup {
    /// Header lines (text, style); empty for the menu.
    header: Vec<(String, Style)>,
    ops: Vec<BlameOp>,
    /// The keyboard-selected row (the menu's).
    sel: Option<usize>,
}

/// Paints the card or the menu, if one is up, and records its hit rects.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint(
    buf: &mut Buffer,
    inner: Rect,
    lay: &Layout,
    lines: &[BlameLine],
    view: &BlameView,
    (selected, first, last): (usize, usize, usize),
    env: &BlameEnv<'_>,
    drawn: &mut Drawn,
) {
    if let Some(menu) = view.menu {
        if view.hunks.get(menu.hunk).is_none() {
            return;
        }
        let popup = Popup {
            header: Vec::new(),
            ops: BlameOp::POPUP.to_vec(),
            sel: Some(menu.sel),
        };
        let (x, y) = (menu.x, menu.y);
        draw(buf, inner, &popup, &move |_| x, y, false, env, drawn);
        return;
    }
    let Some(hi) = view.card_hunk(selected, env.now) else {
        return;
    };
    let Some(h) = view.hunks.get(hi) else {
        return;
    };
    let line = &lines[h.start];
    let popup = if h.uncommitted {
        Popup {
            header: vec![(
                crate::wip_view::NOT_COMMITTED.get().to_string(),
                Style::default()
                    .fg(th().ui.muted)
                    .add_modifier(Modifier::ITALIC),
            )],
            ops: Vec::new(),
            sel: None,
        }
    } else {
        Popup {
            header: header_lines(line, env),
            ops: BlameOp::POPUP.to_vec(),
            sel: None,
        }
    };
    // Anchored on the hunk's first visible row; to the side of the
    // annotation that faces the code.
    let a = h.start.max(first).min(last.saturating_sub(1));
    let y = inner.y + (a - first) as u16;
    let ann_x = lay.ann_x;
    let ann_w = lay.ann_w;
    let left = view.left;
    let pointer_x = ann_x;
    let place = move |w: u16| -> u16 {
        if ann_w == 0 {
            pointer_x
        } else if left {
            ann_x + ann_w
        } else {
            ann_x.saturating_sub(w)
        }
    };
    draw(buf, inner, &popup, &place, y, true, env, drawn);
}

fn header_lines(line: &BlameLine, env: &BlameEnv<'_>) -> Vec<(String, Style)> {
    let sha = line.commit.to_hex();
    let date = crate::live_settings::format_date_time(line.timestamp);
    let age = crate::compare::relative_age(env.now_secs, line.timestamp);
    let who = if line.author_email.is_empty() {
        line.author_name.clone()
    } else {
        format!("{} <{}>", line.author_name, line.author_email)
    };
    vec![
        (
            sha,
            Style::default()
                .fg(th().ui.accent)
                .add_modifier(Modifier::BOLD),
        ),
        (
            who,
            Style::default()
                .fg(th().ui.text)
                .add_modifier(Modifier::BOLD),
        ),
        (
            format!("{date}  ({age})"),
            Style::default().fg(th().ui.muted),
        ),
        (line.summary.clone(), Style::default().fg(th().ui.text)),
    ]
}

#[allow(clippy::too_many_arguments)]
/// Draws `popup` inside `bounds`, `place` giving its x for its width, `y`
/// the row it hangs from.
fn draw(
    buf: &mut Buffer,
    bounds: Rect,
    popup: &Popup,
    place: &dyn Fn(u16) -> u16,
    y: u16,
    closable: bool,
    env: &BlameEnv<'_>,
    drawn: &mut Drawn,
) {
    let rows: Vec<(String, String)> = popup
        .ops
        .iter()
        .map(|op| {
            (
                op.popup_label().get().to_string(),
                env.keys.shown_key(op.action()),
            )
        })
        .collect();
    let content_w = popup
        .header
        .iter()
        .map(|(t, _)| cells(t))
        .chain(rows.iter().map(|(l, k)| cells(l) + 3 + cells(k)))
        .max()
        .unwrap_or(0) as u16;
    let width = (content_w + 4).clamp(MIN_W, MAX_W).min(bounds.width);
    let sep = u16::from(!popup.header.is_empty() && !rows.is_empty());
    let height = (popup.header.len() as u16 + sep + rows.len() as u16 + 2).min(bounds.height);
    if width < 8 || height < 3 {
        return;
    }
    let x = place(width).clamp(bounds.x, bounds.x + bounds.width - width);
    let below = y + 1;
    let top = if below + height <= bounds.y + bounds.height {
        below
    } else {
        y.saturating_sub(height).max(bounds.y)
    };
    let rect = Rect::new(x, top, width, height);
    Clear.render(rect, buf);
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(th().ui.accent))
        .style(Style::default().bg(th().ui.popup_bg))
        .render(rect, buf);
    // The card (not the menu) carries the close cross; it is Esc.
    let cross = if closable {
        crate::chrome::close_cross::paint(buf, rect, env.hovered == Some(BlameHit::CardClose))
    } else {
        None
    };
    let text_w = (width - 4) as usize;
    let mut cy = rect.y + 1;
    for (text, style) in &popup.header {
        if cy >= rect.y + rect.height - 1 {
            break;
        }
        buf.set_stringn(rect.x + 2, cy, text, text_w, *style);
        cy += 1;
    }
    if sep == 1 {
        cy += 1;
    }
    for (i, (label, key)) in rows.iter().enumerate() {
        if cy >= rect.y + rect.height - 1 {
            break;
        }
        let row = Rect::new(rect.x + 1, cy, width - 2, 1);
        let op = popup.ops[i];
        let hot = env.hovered == Some(BlameHit::Op(op)) || popup.sel == Some(i);
        if hot {
            buf.set_style(
                row,
                Style::default()
                    .bg(th().ui.hover)
                    .add_modifier(Modifier::BOLD),
            );
        }
        buf.set_stringn(
            rect.x + 2,
            cy,
            label,
            text_w,
            Style::default().fg(th().ui.text),
        );
        let kw = cells(key);
        if kw > 0 && text_w > cells(label) + kw + 1 {
            buf.set_stringn(
                rect.x + 2 + (text_w - kw) as u16,
                cy,
                key,
                kw,
                Style::default().fg(th().ui.muted),
            );
        }
        drawn.popup_hits.push((row, BlameHit::Op(op)));
        cy += 1;
    }
    if let Some(cross) = cross {
        drawn.popup_hits.push((cross, BlameHit::CardClose));
    }
    // The body last: the rows above are inner-first.
    drawn.popup_hits.push((rect, BlameHit::Card));
}
