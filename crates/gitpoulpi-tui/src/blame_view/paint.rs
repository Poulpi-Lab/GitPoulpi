//! Painting the Blame view into the frame buffer, and the hit rects that
//! come out of the same walk (the project rule: never recompute geometry
//! elsewhere).

use std::time::{Duration, Instant};

use gitpoulpi_core::domain::{Blame, BlameContent, BlameLine};
use gitpoulpi_i18n::msg::blame_view as msg;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;

use super::hunks::{heat_color, Hunk};
use super::layout::{self, Layout};
use super::{BlameHit, BlameView};
use crate::avatar::{self, AvatarService, LogAvatarImages, LogAvatarMode};
use crate::cell_text::{cells, char_cells};
use crate::editor_highlight::Run;
use crate::file_view::{FileInsight, FileInsightData};
use crate::keymap::Keymap;
use crate::th;

/// Time the syntax highlighter may use per frame.
const HIGHLIGHT_BUDGET: Duration = Duration::from_millis(20);

/// What the painter needs besides the insight.
pub(crate) struct BlameEnv<'a> {
    /// What the pointer is on (last frame's hit).
    pub(crate) hovered: Option<BlameHit>,
    pub(crate) now: Instant,
    pub(crate) now_secs: i64,
    pub(crate) mode: LogAvatarMode,
    pub(crate) fallback: gitpoulpi_config::AvatarFallback,
    pub(crate) keys: &'a Keymap,
    /// The pixel avatars; `None` draws the initials only.
    pub(crate) pictures: Option<(&'a mut AvatarService, &'a mut LogAvatarImages)>,
}

/// What a frame produced: hit rects of the rows, and the popup's (which
/// the caller puts first in the hit list).
#[derive(Debug, Default)]
pub(crate) struct Drawn {
    pub(crate) hits: Vec<(Rect, BlameHit)>,
    pub(crate) popup_hits: Vec<(Rect, BlameHit)>,
}

/// The title's legend: the age heat ramp, old to new.
pub(crate) fn legend() -> Vec<Span<'static>> {
    let mut spans = vec![Span::styled(
        format!(" {} ", msg::LEGEND_OLD.get()),
        Style::default().fg(th().ui.muted),
    )];
    for i in 0..5 {
        spans.push(Span::styled(
            "\u{2588}",
            Style::default().fg(heat_color(i as f32 / 4.0)),
        ));
    }
    spans.push(Span::styled(
        format!(" {} ", msg::LEGEND_NEW.get()),
        Style::default().fg(th().ui.muted),
    ));
    spans
}

/// Cells wide the legend takes.
pub(crate) fn legend_width() -> usize {
    legend().iter().map(|s| cells(&s.content)).sum()
}

/// Draws the Blame view in `inner`. Does nothing (and returns no hits)
/// when `insight` holds no blame lines.
pub(crate) fn render(
    buf: &mut Buffer,
    inner: Rect,
    insight: &mut FileInsight,
    env: &mut BlameEnv<'_>,
) -> Drawn {
    let mut drawn = Drawn::default();
    if inner.width == 0 || inner.height == 0 {
        return drawn;
    }
    insight.clamp(inner.height as usize);
    let FileInsight {
        data,
        blame,
        selected,
        offset,
        h_offset,
        highlight,
        ..
    } = insight;
    let (
        Some(view),
        FileInsightData::Blame(Blame {
            content: BlameContent::Lines(lines),
            ..
        }),
    ) = (blame.as_mut(), data)
    else {
        return drawn;
    };
    let (selected, first, h_offset) = (*selected, *offset, *h_offset);
    let height = inner.height as usize;
    let last = (first + height).min(lines.len());
    let left = crate::live_settings::blame_left();
    let lay = layout::layout(
        inner,
        lines.len(),
        left,
        crate::live_settings::blame_width(),
    );
    view.inner = inner;
    view.left = left;
    view.track_hover(env.hovered, env.now);

    // Syntax colors for what is about to be drawn, within the budget.
    view.hl_pending = false;
    if crate::editor_prefs::syntax_highlighting() && last > first {
        let ok = view.hl.advance_window(
            |i| lines[i].content.as_str(),
            lines.len(),
            first,
            last,
            HIGHLIGHT_BUDGET,
        );
        view.hl_pending = !ok;
    }

    let dim = Style::default().fg(th().ui.muted);
    let hovered_hunk = match env.hovered {
        Some(BlameHit::Annot(l)) => view.hunk_at(l),
        _ => None,
    };
    let selected_hunk = view.hunk_at(selected);

    #[allow(clippy::needless_range_loop)]
    for i in first..last {
        let y = inner.y + (i - first) as u16;
        let agent = highlight.is_some_and(|(a, b)| (a..=b).contains(&i));
        let row_style = if agent {
            Some(
                Style::default()
                    .bg(th().diff.search_hit_bg)
                    .add_modifier(Modifier::BOLD),
            )
        } else if i == selected {
            Some(
                Style::default()
                    .bg(th().ui.selection)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            None
        };
        if let Some(style) = row_style {
            buf.set_style(Rect::new(inner.x, y, inner.width, 1), style);
        }
        let num = format!("{:>w$}", i + 1, w = lay.num_w as usize);
        buf.set_stringn(lay.num_x, y, &num, lay.num_w as usize, dim);
        paint_code(
            buf,
            lay.text_x,
            y,
            lay.text_w as usize,
            &lines[i].content,
            view.hl.line(i),
            h_offset,
        );
        let hunk = &view.hunks[view.hunk_of[i] as usize];
        let (heat, bar) = strip_styles(hunk);
        buf.set_stringn(lay.heat_x, y, heat.0, 1, heat.1);
        buf.set_stringn(lay.bar_x, y, bar.0, 1, bar.1);
        if lay.ann_w > 0 {
            let ann = Rect::new(lay.ann_x, y, lay.ann_w, 1);
            let tinted = (Some(view.hunk_of[i] as usize) == selected_hunk
                || Some(view.hunk_of[i] as usize) == hovered_hunk)
                && i != selected
                && !agent;
            if tinted {
                buf.set_style(ann, Style::default().bg(th().ui.hover));
            }
            drawn.hits.push((ann, BlameHit::Annot(i)));
        }
    }

    // The border, drawn over the whole height so it reads as a column.
    if lay.ann_w > 0 {
        let hot = view.drag || env.hovered == Some(BlameHit::Handle);
        let style = if hot {
            Style::default().fg(th().ui.accent)
        } else {
            dim
        };
        for y in inner.y..inner.y + inner.height {
            buf.set_stringn(lay.handle_x, y, "\u{2502}", 1, style);
        }
        drawn.hits.push((
            Rect::new(lay.handle_x, inner.y, 1, inner.height),
            BlameHit::Handle,
        ));
    }

    if lay.ann_w > 0 && last > first {
        paint_annotations(buf, inner, &lay, lines, view, (first, last), selected, env);
    }

    // The detail card (hover dwell or pinned) or the right-click menu.
    if lay.ann_w > 0 || view.menu.is_some() || view.pinned {
        super::card::paint(
            buf,
            inner,
            &lay,
            lines,
            view,
            (selected, first, last),
            env,
            &mut drawn,
        );
    }
    if let Some(typed) = &view.goto {
        paint_goto(buf, inner, typed);
    }
    drawn
}

/// The heat and bar glyphs of a hunk: colored half blocks, or dotted
/// muted ones for uncommitted lines.
fn strip_styles(hunk: &Hunk) -> ((&'static str, Style), (&'static str, Style)) {
    if hunk.uncommitted {
        let s = Style::default().fg(th().status.warning);
        (("\u{250A}", s), ("\u{250A}", s))
    } else {
        (
            ("\u{258C}", Style::default().fg(heat_color(hunk.heat))),
            ("\u{258C}", Style::default().fg(hunk.color)),
        )
    }
}

/// One line of code: syntax colors, tabs as one space, scrolled by
/// `h_offset` characters (the first cell then a `«`, as in every File
/// Insight view).
fn paint_code(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    width: usize,
    text: &str,
    runs: Option<&[Run]>,
    h_offset: usize,
) {
    if width == 0 {
        return;
    }
    let mut col = 0usize;
    let mut chars = text.char_indices().peekable();
    if h_offset > 0 {
        let mut skipped = 0;
        while skipped < h_offset {
            if chars.next().is_none() {
                break;
            }
            skipped += 1;
        }
        buf.set_stringn(x, y, "\u{ab}", 1, Style::default().fg(th().ui.muted));
        col = 1;
        if skipped < h_offset {
            return;
        }
    }
    // The run the current byte is in: (end byte, style).
    let mut run_idx = 0usize;
    let mut run_end = 0usize;
    let mut style = Style::default();
    let mut tmp = [0u8; 4];
    for (byte, ch) in chars {
        if col >= width {
            break;
        }
        if let Some(runs) = runs {
            while byte >= run_end && run_idx < runs.len() {
                style = crate::editor_highlight::style_of(runs[run_idx].style);
                run_end += runs[run_idx].len;
                run_idx += 1;
            }
            if byte >= run_end {
                style = Style::default();
            }
        }
        let ch = if ch == '\t' { ' ' } else { ch };
        let w = char_cells(ch);
        if w == 0 {
            continue;
        }
        if col + w > width {
            break;
        }
        buf.set_stringn(x + col as u16, y, ch.encode_utf8(&mut tmp), w, style);
        col += w;
    }
}

/// The annotation text of every hunk on screen, the avatars, and the
/// separator under each hunk's last line.
#[allow(clippy::too_many_arguments)]
fn paint_annotations(
    buf: &mut Buffer,
    inner: Rect,
    lay: &Layout,
    lines: &[BlameLine],
    view: &BlameView,
    (first, last): (usize, usize),
    selected: usize,
    env: &mut BlameEnv<'_>,
) {
    let dim = Style::default().fg(th().ui.muted);
    let strong = Style::default()
        .fg(th().ui.text)
        .add_modifier(Modifier::BOLD);
    let slot_cols: u16 = match env.mode {
        LogAvatarMode::Off => 0,
        LogAvatarMode::Glyph => 1,
        LogAvatarMode::Picture => avatar::SLOT_COLS,
    };
    let indent = if slot_cols == 0 { 0 } else { slot_cols + 1 };
    // Cells the text may use: the last one is left empty, so the text
    // never touches the author bar of a left-hand column.
    let room = lay.ann_w.saturating_sub(1);
    let mut slots: Vec<(Rect, usize)> = Vec::new();
    let mut hi = view.hunk_at(first).unwrap_or(0);
    while hi < view.hunks.len() && view.hunks[hi].start < last {
        let h = &view.hunks[hi];
        let a = h.start.max(first);
        let b = h.end.min(last);
        let ya = inner.y + (a - first) as u16;
        let line = &lines[h.start];
        let visible = b - a;
        if h.uncommitted {
            buf.set_stringn(
                lay.ann_x,
                ya,
                crate::wip_view::NOT_COMMITTED.get(),
                room as usize,
                dim.add_modifier(Modifier::ITALIC),
            );
        } else {
            let age = crate::compare::relative_age(env.now_secs, line.timestamp);
            if slot_cols > 0 {
                let (glyph, color) = avatar::fallback_text(
                    env.fallback,
                    &line.author_name,
                    &line.author_email,
                    usize::from(slot_cols),
                );
                buf.set_stringn(
                    lay.ann_x,
                    ya,
                    &glyph,
                    usize::from(slot_cols),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                );
                if env.mode == LogAvatarMode::Picture {
                    slots.push((Rect::new(lay.ann_x, ya, avatar::SLOT_COLS, 1), hi));
                }
            }
            let text_x = lay.ann_x + indent;
            let text_room = room.saturating_sub(indent) as usize;
            if visible >= 2 && lay.ann_w >= layout::FULL_ANN {
                // Header: author, and the age at the right edge.
                let age_w = cells(&age);
                let name_room = if text_room > age_w + 8 {
                    text_room - age_w - 1
                } else {
                    text_room
                };
                let name = fit(&line.author_name, name_room);
                buf.set_stringn(text_x, ya, &name, name_room, strong);
                if text_room > age_w + 8 {
                    buf.set_stringn(text_x + (text_room - age_w) as u16, ya, &age, age_w, dim);
                }
                buf.set_stringn(
                    text_x,
                    ya + 1,
                    fit(&line.summary, text_room),
                    text_room,
                    Style::default().fg(th().ui.text),
                );
                // A tall hunk would leave the rest of its column empty: the
                // short sha and the full date fill the first free line.
                if visible >= 3 {
                    let meta = format!(
                        "{}  {}",
                        gitpoulpi_core::domain::short(line.commit),
                        crate::live_settings::format_date_time(line.timestamp)
                    );
                    buf.set_stringn(text_x, ya + 2, fit(&meta, text_room), text_room, dim);
                }
            } else {
                // One row: author, age and summary as far as they fit.
                let compact = format!("{}  {}  {}", line.author_name, age, line.summary);
                buf.set_stringn(text_x, ya, fit(&compact, text_room), text_room, strong);
                // The author alone in bold, the rest quieter.
                let name_w = cells(&line.author_name).min(text_room);
                let rest = Rect::new(text_x + name_w as u16, ya, (text_room - name_w) as u16, 1);
                buf.set_style(rest, dim.remove_modifier(Modifier::BOLD));
            }
        }
        // The thin separator under the hunk's own last line.
        if b == h.end && hi + 1 < view.hunks.len() {
            let yl = inner.y + (b - 1 - first) as u16;
            buf.set_style(
                Rect::new(lay.ann_x, yl, lay.ann_w, 1),
                Style::default().add_modifier(Modifier::UNDERLINED),
            );
        }
        hi += 1;
    }
    // Pixel avatars, over the initials the text path just drew.
    if let Some((service, images)) = env.pictures.as_mut() {
        if !slots.is_empty() {
            let shas: Vec<String> = slots
                .iter()
                .map(|(_, h)| view.hunks[*h].commit.to_hex())
                .collect();
            let queries: Vec<avatar::AvatarQuery> = slots
                .iter()
                .zip(&shas)
                .map(|((_, h), sha)| avatar::AvatarQuery {
                    email: lines[view.hunks[*h].start].author_email.as_str(),
                    sha,
                })
                .collect();
            let pictures = service.pictures(&queries);
            let selected_hunk = view.hunk_at(selected);
            let draw: Vec<avatar::AvatarSlot> = slots
                .iter()
                .zip(pictures)
                .filter_map(|((area, h), picture)| {
                    let (author, bitmap) = picture?;
                    Some(avatar::AvatarSlot {
                        area: *area,
                        key: avatar::PictureKey {
                            author: author.to_string(),
                            tint: Some(avatar::RowTint::of(selected_hunk == Some(*h), false)),
                            dimmed: false,
                        },
                        bitmap,
                    })
                })
                .collect();
            images.render(buf, &draw);
        }
    }
}

/// The minimap beside the code, colored by the age heat (the legend's
/// ramp): `rect` is `minimap::rect_in` of the pane's body.
pub(crate) fn paint_minimap(frame: &mut ratatui::Frame, rect: Rect, insight: &mut FileInsight) {
    let FileInsight {
        data,
        blame,
        offset,
        ..
    } = insight;
    let (
        Some(view),
        FileInsightData::Blame(Blame {
            content: BlameContent::Lines(lines),
            ..
        }),
    ) = (blame.as_mut(), data)
    else {
        return;
    };
    let viewport = view.inner.height as usize;
    let key = view.id;
    let mut map = std::mem::take(&mut view.minimap);
    let hunks = &view.hunks;
    let hunk_of = &view.hunk_of;
    map.draw_colored(frame, rect, key, lines.len(), (*offset, viewport), |i| {
        let color = hunk_of
            .get(i)
            .and_then(|h| hunks.get(*h as usize))
            .map(|h| {
                if h.uncommitted {
                    th().status.warning
                } else {
                    heat_color(h.heat)
                }
            });
        (std::borrow::Cow::Borrowed(lines[i].content.as_str()), color)
    });
    view.minimap = map;
}

/// `text` clipped to `width` cells with an ellipsis.
fn fit(text: &str, width: usize) -> String {
    if cells(text) <= width {
        return text.to_string();
    }
    crate::ui_kit::truncate_with_ellipsis(text, width)
}

/// The "Go to line" prompt over the pane's last row.
fn paint_goto(buf: &mut Buffer, inner: Rect, typed: &str) {
    let text = format!(" {} {typed}\u{2581} ", msg::GOTO_PROMPT.get());
    let w = (cells(&text) as u16).min(inner.width);
    let area = Rect::new(inner.x, inner.y + inner.height - 1, w, 1);
    buf.set_style(
        area,
        Style::default()
            .bg(th().ui.accent)
            .fg(th().ui.on_accent)
            .add_modifier(Modifier::BOLD),
    );
    buf.set_stringn(area.x, area.y, &text, w as usize, Style::default());
}
