//! The History view's list: paint, and the hit rects of that same walk.

use gitpoulpi_config::AvatarFallback;
use gitpoulpi_core::domain::{short, FileChangeKind, FileHistoryEntry};
use gitpoulpi_i18n::msg::history_view as msg;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

use super::{HistoryView, ROW_H};
use crate::avatar::{self, AvatarService, LogAvatarImages, LogAvatarMode};
use crate::chrome::HitTarget;
use crate::ui_kit::{display_width, fit};
use crate::{th, ClippedRender};

/// What a hit on the list means.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HistoryHit {
    Row(usize),
    /// The splitter between the list and the diff.
    Divider,
    /// The list's close cross: hides it (`Alt+C` is the keyboard twin).
    Close,
    /// The strip left of the right side while the list is hidden.
    Reopen,
}

/// The settings a paint reads, gathered by the caller.
pub(crate) struct ListPaint {
    pub(crate) fallback: AvatarFallback,
    /// How the avatar is drawn (the same verdict the compare cards and the
    /// Blame view use): pixels, a glyph, or not at all.
    pub(crate) mode: LogAvatarMode,
    pub(crate) hovered_row: Option<usize>,
    pub(crate) close_hot: bool,
    pub(crate) divider_hot: bool,
    /// Seconds since the epoch, for the relative dates.
    pub(crate) now: i64,
}

/// The pixel avatars, when the terminal draws them.
pub(crate) type Pictures<'a> = Option<(&'a mut AvatarService, &'a mut LogAvatarImages)>;

const STATS_COLS: usize = 9;
/// A rename shows on the second line when at least this many cells are left
/// for it (cut with an ellipsis below its full length).
const MIN_RENAME: usize = 10;
/// The stats appear from this inner width up, the summary always staying.
const SHOW_STATS_AT: usize = 30;
/// The separator between the second line's parts.
const SEP: &str = " \u{b7} ";

/// Cells the avatar slot takes in the left gutter (0: no avatar).
fn slot_cols(mode: LogAvatarMode) -> u16 {
    match mode {
        LogAvatarMode::Off => 0,
        LogAvatarMode::Glyph => 1,
        LogAvatarMode::Picture => avatar::SLOT_COLS,
    }
}

/// The left gutter: the badge on the first line and the avatar on the second
/// share it, so both lines' text starts in the same column.
pub(crate) fn gutter(mode: LogAvatarMode) -> usize {
    usize::from(slot_cols(mode)).max(1) + 1
}

/// Paints the list into `area` and returns its hit rects: rows, then the
/// splitter along the diff's left edge (below the diff's own bar rows).
pub(crate) fn render_list(
    frame: &mut Frame,
    area: Rect,
    view: &mut HistoryView,
    paint: &ListPaint,
    mut pictures: Pictures<'_>,
) -> Vec<(Rect, HitTarget)> {
    let focused = view.pane == super::Pane::List;
    let title = if view.is_range() {
        let (lo, hi) = view.range();
        msg::title_range(hi - lo + 1, view.path.display())
    } else {
        msg::title(view.path.display())
    };
    let block = Block::default()
        .title(format!(" {title} "))
        .border_type(BorderType::Rounded)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if focused {
            th().ui.accent
        } else {
            th().ui.border
        }));
    let inner = block.inner(area);
    frame.render_clipped(block, area);
    view.list_area = area;
    view.rows_area = inner;
    view.strip = Rect::default();
    let mut hits = Vec::new();
    if let Some(cross) =
        crate::chrome::close_cross::paint(frame.buffer_mut(), area, paint.close_hot)
    {
        hits.push((cross, HitTarget::History(HistoryHit::Close)));
    }
    if inner.height == 0 || inner.width == 0 {
        return hits;
    }
    let viewport = view.viewport_rows();
    view.clamp(viewport);

    let dim = Style::default().fg(th().ui.muted);
    let mut notice: Vec<Line<'static>> = Vec::new();
    if let Some(why) = &view.error {
        notice.push(Line::styled(
            msg::failed(why),
            Style::default().fg(th().status.danger),
        ));
    } else if view.first_loaded && view.entries.is_empty() {
        notice.push(Line::styled(msg::EMPTY.get(), dim));
    } else if view.entries.is_empty() {
        notice.push(Line::styled(msg::LOADING.get(), dim));
    }
    // One more row than fits whole when the height is odd: its first line
    // shows.
    let shown = view.offset..(view.offset + viewport + 1).min(view.rows());
    let mut slots: Vec<(Rect, usize)> = Vec::new();
    for i in shown {
        let y = view.row_y(i);
        if y >= inner.bottom() {
            break;
        }
        let Some(entry) = view.row_at(i) else {
            // The loading row, past the last commit.
            frame.render_clipped(
                Paragraph::new(Line::styled(msg::LOADING_MORE.get(), dim)),
                Rect::new(inner.x, y, inner.width, 1),
            );
            continue;
        };
        let look = RowLook {
            cursor: i == view.cursor,
            in_range: view.in_selection(i),
            hot: paint.hovered_row == Some(i),
        };
        let h = ROW_H.min(inner.bottom() - y);
        let lines = row_lines(entry, inner.width as usize, look, paint);
        frame.render_clipped(
            Paragraph::new(lines.to_vec()),
            Rect::new(inner.x, y, inner.width, h),
        );
        if h > 1 && paint.mode == LogAvatarMode::Picture {
            slots.push((Rect::new(inner.x, y + 1, avatar::SLOT_COLS, 1), i));
        }
        hits.push((
            Rect::new(inner.x, y, inner.width, h),
            HitTarget::History(HistoryHit::Row(i)),
        ));
    }
    if !notice.is_empty() {
        frame.render_clipped(Paragraph::new(notice), inner);
    }
    // Pixel avatars, over the initials the text path just drew.
    if let Some((service, images)) = pictures.as_mut() {
        if !slots.is_empty() {
            let shas: Vec<String> = slots
                .iter()
                .filter_map(|(_, i)| view.row_at(*i))
                .map(|e| e.commit.to_hex())
                .collect();
            let queries: Vec<avatar::AvatarQuery> = slots
                .iter()
                .filter_map(|(_, i)| view.row_at(*i))
                .zip(&shas)
                .map(|(e, sha)| avatar::AvatarQuery {
                    email: e.author_email.as_str(),
                    sha,
                })
                .collect();
            let found = service.pictures(&queries);
            let draw: Vec<avatar::AvatarSlot> = slots
                .iter()
                .zip(found)
                .filter_map(|((area, i), picture)| {
                    let (author, bitmap) = picture?;
                    let selected = view.in_selection(*i);
                    let hot = paint.hovered_row == Some(*i);
                    Some(avatar::AvatarSlot {
                        area: *area,
                        key: avatar::PictureKey {
                            author: author.to_string(),
                            tint: Some(avatar::RowTint::of(selected, hot)),
                            dimmed: false,
                        },
                        bitmap,
                    })
                })
                .collect();
            images.render(frame.buffer_mut(), &draw);
        }
    }

    if let Some(track) = crate::scrollbar::Track::vertical(
        area,
        view.rows() * usize::from(ROW_H),
        view.offset * usize::from(ROW_H),
    ) {
        crate::scrollbar::render(frame, track, true, false);
    }
    // The splitter: the diff's left border, under its bar rows. Wide enough
    // to grab, as exactly the cell that lights up.
    if area.height > super::HEADER_ROWS {
        let rect = Rect::new(
            area.right(),
            area.y + super::HEADER_ROWS,
            1,
            area.height - super::HEADER_ROWS,
        );
        if paint.divider_hot {
            let buf = frame.buffer_mut();
            for y in rect.y..rect.bottom() {
                if let Some(cell) = buf.cell_mut((rect.x, y)) {
                    cell.set_style(Style::default().fg(th().ui.accent));
                }
            }
        }
        hits.push((rect, HitTarget::History(HistoryHit::Divider)));
    }
    hits
}

#[derive(Clone, Copy)]
pub(crate) struct RowLook {
    pub(crate) cursor: bool,
    pub(crate) in_range: bool,
    pub(crate) hot: bool,
}

/// One commit's two lines, `width` cells wide.
///
/// Line 1: the change badge, the summary and the colored `+a -d`. Line 2
/// (muted): the avatar's initials (a picture is laid over them), the author,
/// the relative date and the short sha, then `old -> new` on a rename. When
/// the width runs out the rename, then the sha, then the author's tail go;
/// the author and the date are never dropped.
pub(crate) fn row_lines(
    entry: &FileHistoryEntry,
    width: usize,
    look: RowLook,
    paint: &ListPaint,
) -> [Line<'static>; 2] {
    let dim = Style::default().fg(th().ui.muted);
    let gutter = gutter(paint.mode);
    let text_w = width.saturating_sub(gutter).max(1);

    // ---- line 1
    let (letter, color) = badge(&entry.change);
    let mut first: Vec<Span<'static>> = vec![
        Span::styled(
            letter,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" ".repeat(gutter.saturating_sub(1))),
    ];
    let show_stats = width >= SHOW_STATS_AT;
    let stats_w = if show_stats { STATS_COLS + 1 } else { 0 };
    let summary_w = text_w.saturating_sub(stats_w).max(1);
    let summary = fit(&entry.summary, summary_w);
    let used = display_width(&summary);
    first.push(Span::raw(summary));
    if show_stats {
        first.push(Span::raw(" ".repeat(summary_w.saturating_sub(used) + 1)));
        first.extend(stats(entry));
    }

    // ---- line 2
    let mut second: Vec<Span<'static>> = Vec::new();
    let slot = usize::from(slot_cols(paint.mode));
    if slot > 0 {
        let (glyph, color) = crate::avatar::fallback_text(
            paint.fallback,
            &entry.author_name,
            &entry.author_email,
            slot,
        );
        let pad = " ".repeat(gutter.saturating_sub(display_width(&glyph)));
        second.push(Span::styled(
            glyph,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ));
        second.push(Span::raw(pad));
    } else {
        second.push(Span::raw(" ".repeat(gutter)));
    }
    let date = crate::compare::relative_age(paint.now, entry.timestamp);
    let sha = short(entry.commit);
    let rename = match &entry.change {
        FileChangeKind::Renamed { from } => Some(format!(
            "{} \u{2192} {}",
            from.display(),
            entry.path.display()
        )),
        _ => None,
    };
    let sep = display_width(SEP);
    let author_w = display_width(&entry.author_name);
    let date_w = display_width(&date);
    // What fits after the author and the date: the rename first (cut to the
    // room left, it is what the row is about), then the short sha.
    let mut tail = String::new();
    let mut room = text_w.saturating_sub(author_w + sep + date_w);
    if let Some(rename) = rename.as_deref() {
        if room >= sep + MIN_RENAME {
            let text = fit(rename, room - sep);
            room -= sep + display_width(&text);
            tail.push_str(SEP);
            tail.push_str(&text);
        }
    }
    if sep + display_width(&sha) <= room {
        tail.push_str(SEP);
        tail.push_str(&sha);
    }
    // The author and the date always stay: the author shrinks to what is left.
    let author = fit(
        &entry.author_name,
        text_w.saturating_sub(sep + date_w).max(1),
    );
    let date_room = text_w.saturating_sub(display_width(&author) + sep).max(1);
    second.push(Span::styled(author, Style::default().fg(th().ui.text)));
    second.push(Span::styled(format!("{SEP}{}", fit(&date, date_room)), dim));
    if !tail.is_empty() {
        second.push(Span::styled(tail, dim));
    }

    let mut lines = [Line::from(first), Line::from(second)];
    let fill = if look.cursor || look.in_range {
        Some(th().ui.selection)
    } else if look.hot {
        Some(th().ui.hover)
    } else {
        None
    };
    if let Some(bg) = fill {
        for line in &mut lines {
            *line = std::mem::take(line).style(Style::default().bg(bg));
        }
    }
    if look.cursor {
        lines[0] = std::mem::take(&mut lines[0]).patch_style(Modifier::BOLD);
    }
    lines
}

/// The change badge's letter and color. Letters, like git's own status.
fn badge(change: &FileChangeKind) -> (&'static str, ratatui::style::Color) {
    match change {
        FileChangeKind::Added => ("A", th().status.success),
        FileChangeKind::Modified => ("M", th().status.warning),
        FileChangeKind::Deleted => ("D", th().status.danger),
        FileChangeKind::Renamed { .. } => ("R", th().status.info),
    }
}

/// `+a -d`, right-aligned in `STATS_COLS`; a binary file has no line counts.
fn stats(entry: &FileHistoryEntry) -> Vec<Span<'static>> {
    match (entry.added, entry.removed) {
        (Some(a), Some(d)) => {
            let (plus, minus) = (format!("+{a}"), format!("-{d}"));
            let used = display_width(&plus) + 1 + display_width(&minus);
            let mut out = vec![Span::raw(" ".repeat(STATS_COLS.saturating_sub(used)))];
            out.push(Span::styled(
                plus,
                Style::default().fg(th().chrome.stat_add),
            ));
            out.push(Span::raw(" "));
            out.push(Span::styled(
                minus,
                Style::default().fg(th().chrome.stat_del),
            ));
            out
        }
        _ => vec![Span::styled(
            format!("{:>w$}", msg::BINARY.get(), w = STATS_COLS),
            Style::default().fg(th().ui.muted),
        )],
    }
}

/// The strip that brings a hidden list back, in the first `STRIP_COLS` cells
/// of `area` (the central column's left): the commit column's own strip
/// (`commit_column::paint_strip`), with the History's hit.
pub(crate) fn render_strip(
    frame: &mut Frame,
    area: Rect,
    view: &mut HistoryView,
    hovered: bool,
) -> Vec<(Rect, HitTarget)> {
    view.forget_list();
    view.strip = crate::commit_column::paint_strip(frame, area, hovered);
    if view.strip.is_empty() {
        return Vec::new();
    }
    vec![(view.strip, HitTarget::History(HistoryHit::Reopen))]
}
