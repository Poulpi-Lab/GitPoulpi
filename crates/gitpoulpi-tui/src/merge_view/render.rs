//! The merge editor's paint walk (plan §2.2, §2.3, §2.7).
//!
//! One pass paints the frame *and* returns the hit rects, in the order
//! `find_hit_target` wants them: specific before broad, and nothing here
//! overlaps another target. The geometry comes from [`Geometry`], which
//! the paint reads and never recomputes (maintainer rule 6).
//!
//! All user-visible strings are plain literals in this file, so the i18n
//! extraction (slice I11) finds them in one place.

use crate::th;
use std::time::{Duration, Instant};

use gitpoulpi_core::domain::{ConflictSide, LineChange, LineOrigin, RowMark};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Widget};
use ratatui::Frame;

use gitpoulpi_i18n::msg::merge as tr;
use gitpoulpi_i18n::msg::merge_view as mv;

use crate::dialog_kit::{self, ButtonState};
use crate::diff_view::line_marker;
use crate::editor_buffer::char_cells;
use crate::editor_highlight::Run;
use crate::{display_width, fit, plain_button_style, scrollbar, tail_fit};

use super::layout::{
    self, choose_mode, group_width, place_items, toolbar_for, ButtonKind, DRow, Geometry, Mode,
    ORow, ToolItem, ToolbarInput,
};
use super::model::{ChunkPick, ConflictStatus, Origin};
use super::view::{slot, MergeView};
use super::{MergeHit, MergePane, MergeScroll};
use crate::keymap::Action;
use crate::scroll_marks::{self, Mark};

/// How long one frame may spend highlighting, shared by all panes.
const HIGHLIGHT_BUDGET: Duration = Duration::from_millis(20);
/// What each pane may spend per frame even when the shared budget is gone.
const MIN_PANE_BUDGET: Duration = Duration::from_millis(2);

fn side_color(side: ConflictSide) -> Color {
    match side {
        ConflictSide::Ours => th().merge.ours,
        ConflictSide::Theirs => th().merge.theirs,
        ConflictSide::Base => th().merge.base,
    }
}

fn side_pick_bg(side: ConflictSide) -> Color {
    match side {
        ConflictSide::Ours => th().merge.ours_pick,
        ConflictSide::Theirs => th().merge.theirs_pick,
        ConflictSide::Base => th().merge.base_pick,
    }
}

fn side_letter(side: ConflictSide) -> &'static str {
    match side {
        ConflictSide::Ours => "A",
        ConflictSide::Theirs => "B",
        ConflictSide::Base => "O",
    }
}

fn ordinal(n: usize) -> &'static str {
    match n {
        1 => mv::ORD_1.get(),
        2 => mv::ORD_2.get(),
        _ => mv::ORD_3.get(),
    }
}

fn digits(n: usize) -> usize {
    n.max(1).to_string().len()
}

/// The text of the placeholder row. Plain text, never saved.
pub(crate) fn placeholder_text(conflict: usize) -> String {
    format!("\u{2039} {} \u{203a}", tr::placeholder(conflict + 1))
}

/// The one-time hint after a pick replaced a typed edit. `undo_key` is the
/// keymap's label for undo: never a hard-coded key.
pub(crate) fn replaced_edit_text(undo_key: &str) -> String {
    tr::replaced_edit(undo_key)
}

/// The key hints that fit `width`, whole segments only. Each is the
/// effective key of its action (never a typed label) and a catalog word.
fn fit_hints(width: usize, in_output: bool) -> String {
    let km = crate::keymap::current();
    let keys = |actions: &[Action]| -> String {
        actions
            .iter()
            .map(|a| km.shown(*a))
            .filter(|k| !k.is_empty())
            .collect::<Vec<_>>()
            .join("/")
    };
    let esc = crate::keymap::KeyChord::from_code(crossterm::event::KeyCode::Esc)
        .map(|c| c.short())
        .unwrap_or_default();
    let mut segments: Vec<(String, &str)> = if in_output {
        vec![
            (keys(&[Action::MergeNextPane]), tr::HINT_PANE.get()),
            (keys(&[Action::MergeNextConflict]), tr::HINT_NEXT.get()),
            (keys(&[Action::MergeUndo]), tr::HINT_UNDO.get()),
            (keys(&[Action::MergeSaveResolve]), tr::HINT_SAVE.get()),
            (esc, tr::HINT_BACK.get()),
        ]
    } else {
        vec![
            (keys(&[Action::MergeToggleLine]), tr::HINT_LINE.get()),
            (keys(&[Action::MergeToggleChunk]), tr::HINT_CHUNK.get()),
            (
                keys(&[
                    Action::MergeTakeA,
                    Action::MergeTakeB,
                    Action::MergeTakeBoth,
                ]),
                tr::HINT_TAKE.get(),
            ),
            (keys(&[Action::MergeSwapOrder]), tr::HINT_ORDER.get()),
            (keys(&[Action::MergeNextConflict]), tr::HINT_NEXT.get()),
            (keys(&[Action::MergeNextPane]), tr::HINT_PANE.get()),
            (keys(&[Action::MergeSaveResolve]), tr::HINT_SAVE.get()),
            (esc, tr::HINT_CLOSE.get()),
        ]
    };
    segments.retain(|(k, _)| !k.is_empty());
    let mut out = String::new();
    for (key, word) in segments {
        let part = format!("{key} {word}");
        let need = display_width(&out) + display_width(&part) + if out.is_empty() { 0 } else { 3 };
        if need > width {
            break;
        }
        if !out.is_empty() {
            out.push_str(" \u{b7} ");
        }
        out.push_str(&part);
    }
    out
}

/// A text slice as spans, tabs expanded, the first `skip` columns scrolled
/// off, clipped to `max` columns and highlighted by `runs`. Returns the
/// spans and the columns used.
pub(super) fn text_spans(
    text: &str,
    runs: Option<&[Run]>,
    skip: usize,
    max: usize,
    bg: Option<Color>,
) -> (Vec<Span<'static>>, usize) {
    let mut out: Vec<Span<'static>> = Vec::new();
    let mut push = |s: &str, style: Style| match out.last_mut() {
        Some(last) if last.style == style => last.content.to_mut().push_str(s),
        _ => out.push(Span::styled(s.to_string(), style)),
    };
    let text_fg = th().merge.text;
    let mut run_idx = 0;
    let mut run_end = runs.and_then(|r| r.first()).map_or(usize::MAX, |r| r.len);
    // `col` counts from the start of the line (tabs stop at the same
    // places however far it is scrolled); `shown` from the pane's edge.
    let mut col = 0usize;
    let mut shown = 0usize;
    for (bi, c) in text.char_indices() {
        let w = char_cells(c, col);
        if shown + w > max {
            break;
        }
        if let Some(runs) = runs {
            while bi >= run_end && run_idx + 1 < runs.len() {
                run_idx += 1;
                run_end += runs[run_idx].len;
            }
        }
        if col + w <= skip {
            // Scrolled off.
            col += w;
            continue;
        }
        let mut style = runs
            .and_then(|r| r.get(run_idx))
            .map_or(Style::default().fg(text_fg), |r| r.style);
        if style.fg.is_none() {
            style = style.fg(text_fg);
        }
        if let Some(bg) = bg {
            style = style.bg(bg);
        }
        if col < skip {
            // A wide character cut by the scroll edge: a blank for the
            // half still in view, so the text keeps its real columns.
            let inside = (col + w - skip).min(max - shown);
            push(&" ".repeat(inside), style);
            col += w;
            shown += inside;
            continue;
        }
        if c == '\t' {
            push(&" ".repeat(w), style);
        } else if c.is_control() {
            push("\u{b7}", style.fg(th().ui.muted));
        } else {
            push(c.encode_utf8(&mut [0; 4]), style);
        }
        col += w;
        shown += w;
    }
    (out, shown)
}

/// The file line of side `s` on an aligned row, when it can be trusted to
/// be a line of the file (a degraded pane shows block lines instead).
fn number_of(row: &gitpoulpi_core::domain::AlignedRow, s: usize, degraded: bool) -> Option<usize> {
    if degraded {
        None
    } else {
        row.lines[s]
    }
}

fn set_line(buf: &mut Buffer, x: u16, y: u16, spans: Vec<Span<'static>>, width: u16) {
    if width == 0 || !buf.area.contains((x, y).into()) {
        return;
    }
    buf.set_line(x, y, &Line::from(spans), width);
}

/// Fills one row of a rect with a background.
fn paint_row_bg(buf: &mut Buffer, rect: Rect, y: u16, bg: Color) {
    dialog_kit::fill(
        buf,
        Rect::new(rect.x, y, rect.width, 1),
        Style::default().bg(bg),
    );
}

fn rule_style(strong: bool) -> Style {
    Style::default().fg(if strong {
        th().ui.accent
    } else {
        th().ui.muted
    })
}

/// A horizontal rule `─` across `x0..x1` on row `y`.
fn rule(buf: &mut Buffer, y: u16, x0: u16, x1: u16, style: Style) {
    for x in x0..x1 {
        if buf.area.contains((x, y).into()) {
            buf[(x, y)].set_symbol("\u{2500}").set_style(style);
        }
    }
}

fn put_symbol(buf: &mut Buffer, x: u16, y: u16, symbol: &str, style: Style) {
    if buf.area.contains((x, y).into()) {
        buf[(x, y)].set_symbol(symbol).set_style(style);
    }
}

impl MergeView {
    /// Paints the merge editor over the central column `area` and returns
    /// its hit rects.
    pub(crate) fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        hovered: Option<MergeHit>,
    ) -> Vec<(Rect, MergeHit)> {
        let mut hits: Vec<(Rect, MergeHit)> = Vec::new();
        if self.full_file {
            self.side_diffs();
        }
        let buf = frame.buffer_mut();
        dialog_kit::fill(buf, area, Style::default());
        if choose_mode(area) == Mode::TooSmall {
            self.render_too_small(buf, area, hovered, &mut hits);
            return hits;
        }
        if self.card.is_some() {
            let mut hits = self.render_card(frame.buffer_mut(), area, hovered);
            let mut front = self.paint_overlay(frame.buffer_mut(), area, hovered);
            front.append(&mut hits);
            return front;
        }
        let geo = Geometry::compute(area, self.split, self.cols, &self.wide_sides(), self.tab);
        self.compact = area.height < layout::COMPACT_BELOW_ROWS;
        self.heights = (
            geo.top_content.height as usize,
            geo.out_content.height as usize,
        );
        self.narrow = geo.mode == Mode::Narrow;
        self.areas = super::input::Areas {
            all: area,
            top: geo.top_content,
            out_title: geo.out_title,
            out: geo.out_content,
            pane_x0: geo.top_content.x,
            pane_w: {
                let mut w = [0u16; 3];
                if geo.mode == Mode::Wide {
                    for (slot, (_, r)) in w.iter_mut().zip(&geo.panes) {
                        *slot = r.width;
                    }
                }
                w
            },
            panes: if geo.mode == Mode::Wide {
                geo.panes.len() as u8
            } else {
                0
            },
            gutter: (digits(self.out_layout().lines).max(2) + 3) as u16,
        };
        let out = self.out_layout();
        self.top_scroll = self
            .top_scroll
            .min(self.top.rows.len().saturating_sub(self.heights.0.max(1)));
        self.out_scroll = self
            .out_scroll
            .min(out.total().saturating_sub(self.heights.1.max(1)));
        self.advance_highlights(&geo);

        // The frame.
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(th().ui.accent))
            .render(geo.outer, buf);
        self.paint_title(buf, &geo, hovered, &mut hits);
        self.paint_toolbar(buf, &geo, hovered, &mut hits);

        // Rules, separators and the pane titles.
        let inner_x0 = geo.outer.x + 1;
        let inner_x1 = geo.outer.right() - 1;
        let edge = Style::default().fg(th().ui.accent);
        for y in [geo.top_title.y, geo.out_title.y] {
            put_symbol(buf, geo.outer.x, y, "\u{251c}", edge);
            put_symbol(buf, geo.outer.right() - 1, y, "\u{2524}", edge);
        }
        let splitter_lit = hovered == Some(MergeHit::Splitter)
            || self.ui.drag == Some(super::input::Drag::Splitter);
        let divider_lit = |i: usize| {
            hovered == Some(MergeHit::Divider(i as u8))
                || self.ui.drag == Some(super::input::Drag::Divider(i as u8))
        };
        rule(
            buf,
            geo.out_title.y,
            inner_x0,
            inner_x1,
            rule_style(splitter_lit || self.focus == MergePane::Output),
        );
        match geo.mode {
            Mode::Wide => {
                rule(buf, geo.top_title.y, inner_x0, inner_x1, rule_style(false));
                for (i, (side, rect)) in geo.panes.iter().enumerate() {
                    let focused = self.focus == MergePane::of_side(*side);
                    let title = self.pane_title(*side);
                    let style = Style::default()
                        .fg(side_color(*side))
                        .add_modifier(if focused {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        });
                    let w = rect.width.saturating_sub(2);
                    dialog_kit::put(
                        buf,
                        rect.x + 1,
                        geo.top_title.y,
                        &format!(" {} ", fit(&title, w.saturating_sub(2) as usize)),
                        w,
                        style,
                    );
                    if i + 1 < geo.panes.len() {
                        let sx = rect.right();
                        let lit = divider_lit(i);
                        put_symbol(buf, sx, geo.top_title.y, "\u{252c}", rule_style(lit));
                        put_symbol(buf, sx, geo.out_title.y, "\u{2534}", rule_style(lit));
                    }
                }
            }
            _ => self.paint_tabs(buf, &geo, hovered, &mut hits),
        }
        for (i, (_, rect)) in geo.panes.iter().enumerate() {
            if i + 1 < geo.panes.len() {
                for y in rect.y..rect.bottom() {
                    let lit = divider_lit(i);
                    // A lit divider is heavier, so it reads on a terminal
                    // that shows the color difference faintly.
                    let glyph = if lit { "\u{2503}" } else { "\u{2502}" };
                    put_symbol(buf, rect.right(), y, glyph, rule_style(lit));
                }
            }
        }
        for (i, rect) in geo.dividers.iter().enumerate() {
            hits.push((*rect, MergeHit::Divider(i as u8)));
        }
        // The Output's title, also the splitter.
        self.paint_output_title(buf, &geo, splitter_lit);
        if geo.out_title.width > 0 {
            hits.push((geo.out_title, MergeHit::Splitter));
        }

        for (side, rect) in &geo.panes {
            self.paint_pane(buf, *side, *rect, hovered, &mut hits);
        }
        self.paint_output(buf, geo.out_content, &out, hovered, &mut hits);
        self.paint_scrollbars(buf, &geo, &out, hovered, &mut hits);
        self.paint_hints(buf, &geo);
        // Boxes, the menu and the strip go in front of everything else.
        let mut front = self.paint_overlay(frame.buffer_mut(), area, hovered);
        front.extend(hits);
        front
    }

    // -- too small ------------------------------------------------------------

    fn render_too_small(
        &self,
        buf: &mut Buffer,
        area: Rect,
        hovered: Option<MergeHit>,
        hits: &mut Vec<(Rect, MergeHit)>,
    ) {
        let close = " \u{2715} ";
        let cw = display_width(close) as u16;
        if area.width >= cw {
            let rect = Rect::new(area.right() - cw, area.y, cw, 1);
            dialog_kit::put(
                buf,
                rect.x,
                rect.y,
                close,
                cw,
                plain_button_style(hovered == Some(MergeHit::Close)),
            );
            hits.push((rect, MergeHit::Close));
        }
        let text = tr::TOO_SMALL.get();
        let wrap = area.width.max(1) as usize;
        let mut line = String::new();
        let mut row = 1u16;
        for word in text.split(' ') {
            let extra = usize::from(!line.is_empty());
            if !line.is_empty() && display_width(&line) + extra + display_width(word) > wrap {
                dialog_kit::put(
                    buf,
                    area.x,
                    area.y + row,
                    &line,
                    area.width,
                    Style::default().fg(th().ui.muted),
                );
                row += 1;
                line.clear();
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        if !line.is_empty() && area.y + row < area.bottom() {
            dialog_kit::put(
                buf,
                area.x,
                area.y + row,
                &line,
                area.width,
                Style::default().fg(th().ui.muted),
            );
        }
    }

    // -- header and toolbar ---------------------------------------------------

    fn pane_title(&self, side: ConflictSide) -> String {
        let l = &self.model.merge().labels;
        let (letter, label) = match side {
            ConflictSide::Ours => ("A", &l.current),
            ConflictSide::Theirs => ("B", &l.incoming),
            ConflictSide::Base => ("O", &l.base),
        };
        let label = super::labels::side(label, side);
        let approx = if self.aligned.degraded[slot(side)] {
            format!(" ({})", tr::APPROXIMATE.get())
        } else {
            String::new()
        };
        format!("{letter} \u{b7} {label}{approx}")
    }

    fn paint_title(
        &self,
        buf: &mut Buffer,
        geo: &Geometry,
        hovered: Option<MergeHit>,
        hits: &mut Vec<(Rect, MergeHit)>,
    ) {
        let y = geo.outer.y;
        let (k, n) = self.file_pos;
        let right_text = format!(" {} ", tr::file_of(k, n));
        let (prev, next) = (" \u{2039} ", " \u{203a} ");
        let rw = (display_width(&right_text) + 2 * display_width(prev) + 1) as u16;
        let right_x = (geo.outer.right() - 1).saturating_sub(rw);
        let left_x = geo.outer.x + 1;
        let room = right_x.saturating_sub(left_x + 1) as usize;
        let mut title = format!(
            " {}",
            tr::title(tail_fit(&self.path().to_string_lossy(), room))
        );
        let desc = super::labels::description(&self.model.merge().labels.description);
        if !desc.is_empty() {
            title.push_str(&format!(" \u{2500}\u{2500} {desc}"));
        }
        title.push(' ');
        dialog_kit::put(
            buf,
            left_x,
            y,
            &fit(&title, room),
            room as u16,
            Style::default()
                .fg(th().ui.text)
                .add_modifier(Modifier::BOLD),
        );
        if right_x <= left_x {
            return;
        }
        let mut x = right_x;
        x += dialog_kit::put(
            buf,
            x,
            y,
            &right_text,
            rw,
            Style::default().fg(th().ui.muted),
        )
        .width;
        for (label, hit) in [(prev, MergeHit::PrevFile), (next, MergeHit::NextFile)] {
            let w = display_width(label) as u16;
            let lit = hovered == Some(hit);
            let style = if lit {
                plain_button_style(true)
            } else if n > 1 {
                Style::default().fg(th().ui.text)
            } else {
                Style::default()
                    .fg(th().ui.muted)
                    .add_modifier(Modifier::DIM)
            };
            dialog_kit::put(buf, x, y, label, w, style);
            hits.push((Rect::new(x, y, w, 1), hit));
            x += w;
        }
    }

    fn paint_toolbar(
        &self,
        buf: &mut Buffer,
        geo: &Geometry,
        hovered: Option<MergeHit>,
        hits: &mut Vec<(Rect, MergeHit)>,
    ) {
        let m = &self.model;
        let c = m.counter();
        let input = ToolbarInput {
            counter_long: m.counter_label(),
            counter_short: if m.is_outside() {
                tr::EDITED_OUTSIDE.get().to_string()
            } else {
                format!("{}/{}", c.current, c.total)
            },
            base_on: self.show_base,
            base_available: true,
            full_file: self.full_file,
            ai: self.ai_enabled,
        };
        let plan = toolbar_for(geo.toolbar.width, &input);
        let y = geo.toolbar.y;
        let rw = group_width(&plan.right);
        let left = place_items(&plan.left, geo.toolbar.x + 1, y);
        let right = place_items(&plan.right, geo.toolbar.right().saturating_sub(1 + rw), y);
        let no_conflicts = m.conflict_count() == 0;
        for (rect, item) in left.into_iter().chain(right) {
            match item {
                ToolItem::Text(t) => {
                    let style = if t.trim().is_empty() || t.ends_with(':') {
                        Style::default().fg(th().ui.muted)
                    } else {
                        Style::default()
                            .fg(th().ui.text)
                            .add_modifier(Modifier::BOLD)
                    };
                    dialog_kit::put(buf, rect.x, rect.y, &t, rect.width, style);
                }
                ToolItem::Separator => {
                    dialog_kit::put(
                        buf,
                        rect.x,
                        rect.y,
                        " \u{2502} ",
                        rect.width,
                        Style::default().fg(th().ui.muted),
                    );
                }
                ToolItem::Button { label, hit, kind } => {
                    let lit = hovered == Some(hit);
                    let disabled = (kind == ButtonKind::Ai && (self.ai_busy || no_conflicts))
                        || no_conflicts
                            && matches!(
                                hit,
                                MergeHit::PrevConflict
                                    | MergeHit::NextConflict
                                    | MergeHit::TakeAllA
                                    | MergeHit::TakeAllB
                                    | MergeHit::ResetAll
                            );
                    if kind == ButtonKind::Plain {
                        dialog_kit::put(
                            buf,
                            rect.x,
                            rect.y,
                            &format!(" {label} "),
                            rect.width,
                            plain_button_style(lit),
                        );
                    } else {
                        let state = if disabled {
                            ButtonState::Disabled
                        } else {
                            match kind {
                                ButtonKind::Primary => ButtonState::Primary {
                                    hovered: lit,
                                    focused: false,
                                },
                                ButtonKind::Toggled => ButtonState::Normal {
                                    hovered: lit,
                                    focused: true,
                                },
                                _ => ButtonState::Normal {
                                    hovered: lit,
                                    focused: false,
                                },
                            }
                        };
                        dialog_kit::button(buf, rect.x, rect.y, &label, state);
                        if kind == ButtonKind::Ai && !disabled {
                            for x in rect.x..rect.right() {
                                buf[(x, rect.y)].set_fg(th().status.ai);
                            }
                        }
                    }
                    hits.push((rect, hit));
                }
            }
        }
    }

    fn paint_tabs(
        &self,
        buf: &mut Buffer,
        geo: &Geometry,
        hovered: Option<MergeHit>,
        hits: &mut Vec<(Rect, MergeHit)>,
    ) {
        let l = &self.model.merge().labels;
        let tabs = [
            (
                ConflictSide::Ours,
                format!("A {}", super::labels::tab(&l.current, ConflictSide::Ours)),
            ),
            (
                ConflictSide::Theirs,
                format!(
                    "B {}",
                    super::labels::tab(&l.incoming, ConflictSide::Theirs)
                ),
            ),
            (ConflictSide::Base, tr::BASE.get().to_string()),
        ];
        let row = geo.top_title;
        let per = (row.width.saturating_sub(2) / 3).saturating_sub(3) as usize;
        let mut x = row.x + 1;
        for (side, label) in tabs {
            let label = if side == ConflictSide::Base {
                label
            } else {
                fit(&label, per.max(6))
            };
            let w = dialog_kit::button_width(&label);
            if x + w > row.right() {
                break;
            }
            let lit = hovered == Some(MergeHit::Tab(side));
            let state = if self.tab == side {
                ButtonState::Primary {
                    hovered: lit,
                    focused: false,
                }
            } else {
                ButtonState::Normal {
                    hovered: lit,
                    focused: false,
                }
            };
            let rect = dialog_kit::button(buf, x, row.y, &label, state);
            hits.push((rect, MergeHit::Tab(side)));
            x += w + 1;
        }
    }

    fn paint_output_title(&self, buf: &mut Buffer, geo: &Geometry, lit: bool) {
        let m = &self.model;
        let mut t = format!(" {}", tr::output_title(self.path().to_string_lossy()));
        t.push_str(match m.eol() {
            gitpoulpi_core::domain::Eol::Lf => " \u{b7} LF",
            gitpoulpi_core::domain::Eol::Crlf => " \u{b7} CRLF",
        });
        t.push_str(&format!(
            " \u{b7} {}",
            m.merge().encoding.as_deref().unwrap_or("UTF-8")
        ));
        if m.merge().bom {
            t.push_str(" \u{b7} BOM");
        }
        let (unpicked, _, edited) = m.tally();
        if edited > 0 {
            t.push_str(&format!(" \u{b7} {}", mv::tally_edited(edited as u64)));
        }
        if unpicked > 0 {
            t.push_str(&format!(" \u{b7} {}", mv::tally_left(unpicked as u64)));
        }
        if m.is_dirty() {
            t.push_str(&format!(" \u{25cf} {}", mv::TALLY_MODIFIED.get()));
        }
        t.push(' ');
        let w = geo.out_title.width.saturating_sub(2);
        let style = Style::default()
            .fg(if lit || self.focus == MergePane::Output {
                th().ui.accent
            } else {
                th().ui.muted
            })
            .add_modifier(Modifier::BOLD);
        dialog_kit::put(
            buf,
            geo.out_title.x + 1,
            geo.out_title.y,
            &fit(&t, w as usize),
            w,
            style,
        );
    }

    fn paint_hints(&self, buf: &mut Buffer, geo: &Geometry) {
        let h = geo.hint;
        dialog_kit::fill(buf, h, Style::default().bg(th().ui.panel));
        let saving = self
            .ui
            .saving
            .is_some()
            .then(|| tr::msg_saving(self.path().to_string_lossy()));
        let find = self.find_line();
        if let Some(note) = saving.as_ref().or(find.as_ref()).or(self.note.as_ref()) {
            dialog_kit::put(
                buf,
                h.x + 1,
                h.y,
                note,
                h.width.saturating_sub(1),
                Style::default().fg(th().merge.edit).bg(th().ui.panel),
            );
        } else {
            let w = h.width.saturating_sub(1);
            dialog_kit::put(
                buf,
                h.x + 1,
                h.y,
                &fit_hints(w as usize, self.focus == MergePane::Output),
                w,
                Style::default().fg(th().ui.muted).bg(th().ui.panel),
            );
        }
    }

    // -- highlighting ---------------------------------------------------------

    /// Highlights what is about to be drawn, within one shared budget.
    ///
    /// While keys or wheel steps stream in ([`MergeView::input_busy`]) the
    /// frames parse nothing but what the person is typing on (the Output
    /// caret's line, after an edit): the rest is drawn with the colors it already has
    /// and filled in when the stream stops. A far jump is never parsed
    /// through (see `Highlight::advance_window`).
    fn advance_highlights(&mut self, geo: &Geometry) {
        self.pending = false;
        if !crate::editor_prefs::syntax_highlighting() {
            return;
        }
        let busy = self.input_busy();
        let started = Instant::now();
        let left = || HIGHLIGHT_BUDGET.saturating_sub(started.elapsed());
        // The Output, incremental: an edit invalidates from its line.
        let lines_now = self.model.output().line_count();
        let mut edited = false;
        if let Some(line) = self.model.output_for_caret().take_changed_from() {
            edited = true;
            self.hl_out.invalidate_from(line);
            let delta = lines_now as isize - self.out_lines_seen as isize;
            self.hl_out.shift_stale(line, delta);
        }
        self.out_lines_seen = lines_now;
        let from = self.out_scroll;
        let upto = self.out_scroll + geo.out_content.height as usize;
        {
            let doc = self.model.output();
            let count = doc.line_count();
            let want = if busy {
                // Just the line being typed on, and the lines above it;
                // moving the caret needs nothing until the keys stop.
                if self.focus == MergePane::Output && edited {
                    (doc.cursor_line_col().0 + 1).clamp(from, upto)
                } else {
                    0
                }
            } else {
                upto
            };
            // At most half the budget: the A / B panes must not starve
            // behind a long Output (or the grammar's lazy compile, paid by
            // whoever parses first).
            let ok = self.hl_out.advance_window(
                |i| doc.line(i),
                count,
                from,
                want,
                left().min(HIGHLIGHT_BUDGET / 2),
            );
            self.pending |= !ok || busy;
        }
        if busy {
            self.pending = true;
            return;
        }
        // The static sides, as far down as the panes show.
        let h = geo.top_content.height as usize;
        let rows = &self.top.rows;
        let end = (self.top_scroll + h).min(rows.len());
        let sides = self.model.merge().side_lines();
        for (s, lines) in sides.into_iter().enumerate() {
            let Some(lines) = lines else { continue };
            let (mut lo, mut upto) = (usize::MAX, 0usize);
            for d in &rows[self.top_scroll.min(end)..end] {
                if let DRow::Line { aligned } = d {
                    if let Some(l) = self.aligned.rows[*aligned].lines[s] {
                        lo = lo.min(l);
                        upto = upto.max(l + 1);
                    }
                }
            }
            if lo == usize::MAX {
                continue;
            }
            // Every pane gets a few lines per frame whatever the budget
            // left (`advance` checks the clock every 8 lines), so none
            // stays plain while another is colored.
            let ok = self.hl[s].advance_window(
                |i| lines[i].as_str(),
                lines.len(),
                lo,
                upto,
                left().max(MIN_PANE_BUDGET),
            );
            self.pending |= !ok;
        }
    }

    // -- the A / B / Base panes -----------------------------------------------

    fn paint_pane(
        &self,
        buf: &mut Buffer,
        side: ConflictSide,
        rect: Rect,
        hovered: Option<MergeHit>,
        hits: &mut Vec<(Rect, MergeHit)>,
    ) {
        if rect.width == 0 || rect.height == 0 {
            return;
        }
        let s = slot(side);
        let merge = self.model.merge();
        let Some(lines) = merge.side_lines()[s] else {
            let msg = match side {
                ConflictSide::Base => tr::NO_ANCESTOR.get(),
                _ => tr::NO_TEXT.get(),
            };
            dialog_kit::put(
                buf,
                rect.x + 1,
                rect.y + 1.min(rect.height - 1),
                msg,
                rect.width.saturating_sub(1),
                Style::default().fg(th().ui.muted),
            );
            return;
        };
        let nd = digits(lines.len());
        // A margin, the frame glyph, the pick gutter, the number.
        let prefix = (3 + nd + 1) as u16;
        let degraded = self.aligned.degraded[s];
        let outside = self.model.is_outside();
        let current = self.model.current();
        let n_conflicts = self.model.conflict_count();
        let full_diffs = if self.full_file {
            self.changes.as_ref().and_then(|c| c[s].as_ref())
        } else {
            None
        };
        for r in 0..rect.height {
            let di = self.top_scroll + r as usize;
            let Some(d) = self.top.rows.get(di) else {
                break;
            };
            let y = rect.y + r;
            match *d {
                DRow::Header { conflict } => {
                    self.paint_chunk_header(
                        buf,
                        side,
                        rect,
                        y,
                        conflict,
                        conflict == current,
                        hovered,
                        hits,
                    );
                }
                DRow::Footer { conflict } => {
                    let fs = Style::default().fg(if conflict == current {
                        th().ui.accent
                    } else {
                        th().ui.muted
                    });
                    put_symbol(buf, rect.x + 1, y, "\u{2514}", fs);
                    rule(buf, y, rect.x + 2, rect.right().saturating_sub(1), fs);
                }
                DRow::Line { aligned } => {
                    let row = &self.aligned.rows[aligned];
                    let conflict = row.conflict;
                    let mark = row.marks[s];
                    let block_k = self.top.block_idx[aligned][s];
                    let in_current = conflict == Some(current);
                    let frame_style = Style::default().fg(if in_current {
                        th().ui.accent
                    } else {
                        th().ui.muted
                    });
                    let other = if side == ConflictSide::Ours {
                        Some(ConflictSide::Theirs)
                    } else if side == ConflictSide::Theirs {
                        Some(ConflictSide::Ours)
                    } else {
                        None
                    };
                    // Is this a line the other side brought in (or removed)?
                    let from_other = other.is_some()
                        && conflict.is_none()
                        && match mark {
                            RowMark::Changed | RowMark::Extra => true,
                            RowMark::Absent => {
                                let o = slot(other.expect("checked"));
                                row.marks[o] != RowMark::Absent
                            }
                            RowMark::Same => false,
                        };
                    let pickable = match (conflict, block_k) {
                        (Some(c), Some(k)) if !outside && c < n_conflicts => Some((c, k)),
                        _ => None,
                    };
                    let picked = pickable.is_some_and(|(c, k)| self.model.line_picked(c, side, k));
                    let hit = pickable.map(|(c, k)| MergeHit::Line {
                        conflict: c,
                        side,
                        line: k,
                    });
                    let cursor_here = self.focus == MergePane::of_side(side)
                        && in_current
                        && block_k == Some(self.ui.cursor);
                    // Full file: the side's line against the merge base, in
                    // the Split diff's colors (`diff_view::line_marker`).
                    let (diff_mark, removed_here) = match (full_diffs, number_of(row, s, degraded))
                    {
                        (Some(d), Some(i)) => (
                            match d.lines[i] {
                                LineChange::Same => None,
                                LineChange::Changed | LineChange::Added => {
                                    Some(line_marker(&LineOrigin::Add))
                                }
                            },
                            d.removed_before[i],
                        ),
                        _ => (None, false),
                    };
                    let bg = if hit.is_some() && hovered == hit {
                        Some(th().ui.hover)
                    } else if cursor_here {
                        Some(th().ui.selection)
                    } else if picked {
                        Some(side_pick_bg(side))
                    } else if let Some((_, st)) = diff_mark {
                        st.bg
                    } else if from_other {
                        other.map(|o| {
                            if o == ConflictSide::Ours {
                                th().merge.ours_tint
                            } else {
                                th().merge.theirs_tint
                            }
                        })
                    } else {
                        None
                    };
                    if let Some(bg) = bg {
                        paint_row_bg(buf, rect, y, bg);
                    }
                    let mut spans: Vec<Span<'static>> = Vec::new();
                    let with_bg = |st: Style| bg.map_or(st, |b| st.bg(b));
                    spans.push(match (diff_mark, removed_here) {
                        (Some((m, st)), _) => Span::styled(m, with_bg(st)),
                        (None, true) => {
                            let (m, st) = line_marker(&LineOrigin::Delete);
                            Span::styled(m, with_bg(st))
                        }
                        _ => Span::styled(" ", with_bg(Style::default())),
                    });
                    spans.push(Span::styled(
                        if conflict.is_some() { "\u{2502}" } else { " " },
                        with_bg(frame_style),
                    ));
                    spans.push(match (pickable, picked) {
                        (Some(_), true) => Span::styled(
                            "\u{2713}",
                            with_bg(
                                Style::default()
                                    .fg(side_color(side))
                                    .add_modifier(Modifier::BOLD),
                            ),
                        ),
                        (Some(_), false) => {
                            Span::styled("+", with_bg(Style::default().fg(th().ui.muted)))
                        }
                        _ => Span::styled(" ", with_bg(Style::default())),
                    });
                    // The text: the file's line, or (a pane that could not be
                    // pinned) the conflict block's line.
                    let (text, number): (&str, Option<usize>) = match (degraded, conflict, block_k)
                    {
                        (true, Some(c), Some(k)) => (
                            self.model.block(c, side).get(k).map_or("", |s| s.as_str()),
                            None,
                        ),
                        _ => match row.lines[s] {
                            Some(i) => (lines[i].as_str(), Some(i)),
                            None => ("", None),
                        },
                    };
                    spans.push(Span::styled(
                        match number {
                            Some(i) => format!("{:>nd$} ", i + 1),
                            None => " ".repeat(nd + 1),
                        },
                        with_bg(Style::default().fg(th().ui.muted)),
                    ));
                    let marker = from_other.then(|| {
                        format!(
                            "\u{2591} {}",
                            side_letter(other.expect("from_other has one"))
                        )
                    });
                    let text_w = (rect.width.saturating_sub(prefix) as usize)
                        .saturating_sub(if marker.is_some() { 4 } else { 0 });
                    let runs = if degraded {
                        None
                    } else {
                        number.and_then(|i| self.hl[s].line(i))
                    };
                    let (mut body, used) = text_spans(text, runs, self.h_scroll, text_w, bg);
                    if let Some((_, st)) = diff_mark {
                        for span in &mut body {
                            span.style.fg = st.fg;
                        }
                    }
                    spans.extend(body);
                    if let Some(m) = marker {
                        let pad = rect.width as usize - prefix as usize - used - 3;
                        spans.push(Span::styled(" ".repeat(pad), with_bg(Style::default())));
                        spans.push(Span::styled(
                            m,
                            with_bg(Style::default().fg(side_color(other.expect("set")))),
                        ));
                    }
                    set_line(buf, rect.x, y, spans, rect.width);
                    if let Some(hit) = hit {
                        hits.push((Rect::new(rect.x, y, rect.width, 1), hit));
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn paint_chunk_header(
        &self,
        buf: &mut Buffer,
        side: ConflictSide,
        rect: Rect,
        y: u16,
        c: usize,
        current: bool,
        hovered: Option<MergeHit>,
        hits: &mut Vec<(Rect, MergeHit)>,
    ) {
        let fs = Style::default().fg(if current {
            th().ui.accent
        } else {
            th().ui.muted
        });
        put_symbol(buf, rect.x + 1, y, "\u{250c}", fs);
        rule(buf, y, rect.x + 2, rect.right().saturating_sub(1), fs);
        let outside = self.model.is_outside() || c >= self.model.conflict_count();
        let edited = self.model.is_edited(c);
        let mut x = rect.x + 2;
        let end = rect.right().saturating_sub(1);
        let label = tr::conflict_n(c + 1);
        if outside {
            dialog_kit::put(buf, x, y, &format!(" {label} "), end.saturating_sub(x), fs);
        } else {
            let mark = match self.model.chunk_pick(c, side) {
                ChunkPick::None => "[ ]",
                ChunkPick::Some => "[-]",
                ChunkPick::All => "[x]",
            };
            let hit = MergeHit::Chunk { conflict: c, side };
            let style = if hovered == Some(hit) {
                plain_button_style(true)
            } else if edited {
                Style::default()
                    .fg(th().ui.muted)
                    .add_modifier(Modifier::DIM)
            } else {
                Style::default()
                    .fg(if current {
                        side_color(side)
                    } else {
                        th().ui.muted
                    })
                    .add_modifier(Modifier::BOLD)
            };
            let text = format!("{mark} {label}");
            let w = dialog_kit::put(buf, x, y, &text, end.saturating_sub(x), style).width;
            if w > 0 {
                hits.push((Rect::new(x, y, w, 1), hit));
            }
            x += w;
            if let Some(n) = self.model.order_badge(c, side) {
                x += dialog_kit::put(buf, x, y, " \u{b7} ", end.saturating_sub(x), fs).width;
                let badge = MergeHit::Badge { conflict: c };
                let st = if hovered == Some(badge) {
                    plain_button_style(true)
                } else {
                    Style::default()
                        .fg(side_color(side))
                        .add_modifier(Modifier::BOLD)
                };
                let w = dialog_kit::put(buf, x, y, ordinal(n), end.saturating_sub(x), st).width;
                if w > 0 {
                    hits.push((Rect::new(x, y, w, 1), badge));
                }
                x += w;
            }
            if edited {
                x += dialog_kit::put(
                    buf,
                    x,
                    y,
                    &format!(" \u{270e} {}", mv::CHUNK_EDITED.get()),
                    end.saturating_sub(x),
                    Style::default().fg(th().merge.edit),
                )
                .width;
            }
            dialog_kit::put(buf, x, y, " ", end.saturating_sub(x), fs);
        }
    }

    // -- the Output -----------------------------------------------------------

    fn paint_output(
        &self,
        buf: &mut Buffer,
        rect: Rect,
        out: &layout::OutLayout,
        hovered: Option<MergeHit>,
        hits: &mut Vec<(Rect, MergeHit)>,
    ) {
        if rect.width == 0 || rect.height == 0 {
            return;
        }
        let doc = self.model.output();
        let nd = digits(out.lines).max(2);
        let gutter = (nd + 1 + 1 + 1) as u16;
        let text_w = rect.width.saturating_sub(gutter) as usize;
        let focused = self.focus == MergePane::Output;
        let (caret_line, caret_col) = doc.cursor_line_col();
        let current_region = self.model.region_lines(self.model.current());
        for r in 0..rect.height {
            let row = self.out_scroll + r as usize;
            let y = rect.y + r;
            match out.row_at(row) {
                None => break,
                Some(ORow::Line(i)) => {
                    let hit = MergeHit::OutputLine(i);
                    let in_region = current_region.is_some_and(|(a, b)| a <= i && i < b);
                    let bg = if hovered == Some(hit) {
                        Some(th().ui.hover)
                    } else if in_region {
                        Some(th().merge.current_region)
                    } else {
                        None
                    };
                    if let Some(bg) = bg {
                        paint_row_bg(buf, rect, y, bg);
                    }
                    let with_bg = |st: Style| bg.map_or(st, |b| st.bg(b));
                    let current_line = focused && i == caret_line;
                    let (letter, color) = match self.model.origin_of_line(i) {
                        Origin::Common => (" ", th().ui.muted),
                        Origin::Edited => ("\u{270e}", th().merge.edit),
                        Origin::Side(side) => (side_letter(side), side_color(side)),
                    };
                    let mut spans = vec![
                        Span::styled(
                            format!("{:>nd$} ", i + 1),
                            with_bg(if current_line {
                                Style::default()
                                    .fg(th().ui.text)
                                    .add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(th().ui.muted)
                            }),
                        ),
                        Span::styled(
                            letter,
                            with_bg(Style::default().fg(color).add_modifier(Modifier::BOLD)),
                        ),
                        Span::styled(" ", with_bg(Style::default())),
                    ];
                    let runs = self.hl_out.line(i);
                    let (body, _) = text_spans(doc.line(i), runs, 0, text_w, bg);
                    spans.extend(body);
                    set_line(buf, rect.x, y, spans, rect.width);
                    if focused {
                        paint_selection(buf, doc, i, rect.x + gutter, y, text_w);
                    }
                    if current_line && caret_col < text_w {
                        let cx = rect.x + gutter + caret_col as u16;
                        if buf.area.contains((cx, y).into()) {
                            let cell = &mut buf[(cx, y)];
                            cell.set_style(
                                cell.style()
                                    .add_modifier(crate::text_cursor::caret_modifier()),
                            );
                        }
                    }
                    hits.push((Rect::new(rect.x, y, rect.width, 1), hit));
                }
                Some(ORow::Placeholder(c)) => {
                    let hit = MergeHit::Placeholder { conflict: c };
                    if hovered == Some(hit) {
                        paint_row_bg(buf, rect, y, th().ui.hover);
                    }
                    dialog_kit::put(
                        buf,
                        rect.x + gutter,
                        y,
                        &placeholder_text(c),
                        rect.width.saturating_sub(gutter),
                        Style::default()
                            .fg(th().ui.muted)
                            .add_modifier(Modifier::ITALIC)
                            .bg(if hovered == Some(hit) {
                                th().ui.hover
                            } else {
                                Color::Reset
                            }),
                    );
                    hits.push((Rect::new(rect.x, y, rect.width, 1), hit));
                }
                Some(ORow::NoEol) => {
                    dialog_kit::put(
                        buf,
                        rect.x + gutter,
                        y,
                        &format!("\u{2039} {} \u{203a}", tr::NO_FINAL_NEWLINE.get()),
                        rect.width.saturating_sub(gutter),
                        Style::default()
                            .fg(th().ui.muted)
                            .add_modifier(Modifier::ITALIC),
                    );
                }
            }
        }
    }

    // -- scrollbars -----------------------------------------------------------

    fn paint_scrollbars(
        &self,
        buf: &mut Buffer,
        geo: &Geometry,
        out: &layout::OutLayout,
        hovered: Option<MergeHit>,
        hits: &mut Vec<(Rect, MergeHit)>,
    ) {
        let outer = geo.outer;
        let top_body = Rect::new(
            outer.x,
            geo.top_content.y.saturating_sub(1),
            outer.width,
            geo.top_content.height + 2,
        );
        let out_body = Rect::new(
            outer.x,
            geo.out_title.y,
            outer.width,
            geo.out_content.height + 2,
        );
        let mut bars = [None; 2];
        for (slot, (body, len, offset, which)) in [
            (
                top_body,
                self.top.rows.len(),
                self.top_scroll,
                MergeScroll::Top,
            ),
            (out_body, out.total(), self.out_scroll, MergeScroll::Output),
        ]
        .into_iter()
        .enumerate()
        {
            if let Some(t) = scrollbar::Track::vertical(body, len, offset) {
                bars[slot] = Some(t);
                let hit = MergeHit::Thumb(which);
                scrollbar::render_buf(buf, t, true, hovered == Some(hit));
                // The conflict marks go in front of the thumb's own rect:
                // a click on one jumps to its conflict.
                self.paint_conflict_marks(buf, t, which, len, out, hovered, hits);
                hits.push((
                    Rect::new(t.rect.x, t.rect.y + t.thumb_start, 1, t.thumb_len.max(1)),
                    hit,
                ));
                // The cells no mark and not the thumb own (arrows, bare
                // track): runs, so no two hit rects overlap.
                let mut run: Option<u16> = None;
                for y in t.rect.y..=t.rect.bottom() {
                    let free = y < t.rect.bottom()
                        && !hits.iter().any(|(r, _)| r.contains((t.rect.x, y).into()));
                    match (free, run) {
                        (true, None) => run = Some(y),
                        (false, Some(from)) => {
                            hits.push((
                                Rect::new(t.rect.x, from, 1, y - from),
                                MergeHit::Bar(which),
                            ));
                            run = None;
                        }
                        _ => {}
                    }
                }
            }
        }
        self.ui.bars.set(bars);
    }
}

impl MergeView {
    /// The overview ruler of a scrollbar: one mark per conflict, open or
    /// handled, the current one lit (`scroll_marks`, as the Diff view's).
    /// Several conflicts in one cell show the strongest, the current one
    /// first. The cost is one pass over the conflicts, never the file.
    #[allow(clippy::too_many_arguments)]
    fn paint_conflict_marks(
        &self,
        buf: &mut Buffer,
        track: scrollbar::Track,
        which: MergeScroll,
        len: usize,
        out: &layout::OutLayout,
        hovered: Option<MergeHit>,
        hits: &mut Vec<(Rect, MergeHit)>,
    ) {
        let n = self.model.conflict_count();
        let inner = track.inner_len() as usize;
        if n == 0 || len == 0 || inner == 0 || self.card.is_some() {
            return;
        }
        let current = self.model.current();
        let mut cells: Vec<Option<(Mark, usize)>> = vec![None; inner];
        for c in 0..n {
            let row = match which {
                MergeScroll::Top => self.top.header_at.get(c).copied(),
                MergeScroll::Output => out.row_of_placeholder(c).or_else(|| {
                    self.model
                        .region_lines(c)
                        .map(|(first, _)| out.row_of_line(first))
                }),
            };
            let Some(row) = row.filter(|r| *r < len) else {
                continue;
            };
            let mark = if c == current {
                Mark::ConflictCurrent
            } else if self.model.status(c) == Some(ConflictStatus::Unpicked) {
                Mark::ConflictOpen
            } else {
                Mark::ConflictDone
            };
            let cell = &mut cells[(row * inner / len).min(inner - 1)];
            if cell.is_none_or(|(old, _)| mark > old) {
                *cell = Some((mark, c));
            }
        }
        let marks: Vec<Option<Mark>> = cells.iter().map(|c| c.map(|(m, _)| m)).collect();
        scroll_marks::paint_buf(buf, track, &marks);
        // A mark under the thumb is painted but not clickable: the thumb
        // keeps its drag (and the conflict is on screen anyway).
        let thumb = track.rect.y + track.thumb_start
            ..track.rect.y + track.thumb_start + track.thumb_len.max(1);
        for (i, cell) in cells.iter().enumerate() {
            let Some((_, c)) = cell else { continue };
            let rect = Rect::new(track.rect.x, track.rect.y + track.arrow + i as u16, 1, 1);
            if thumb.contains(&rect.y) {
                continue;
            }
            let hit = MergeHit::ConflictMark(*c);
            if hovered == Some(hit) {
                buf[(rect.x, rect.y)].set_bg(th().ui.hover);
            }
            hits.push((rect, hit));
        }
    }
}

/// Tints the selected cells of Output line `i` (the editor's selection
/// colors); a selection running past the line's end takes one more cell,
/// the line break.
pub(super) fn paint_selection(
    buf: &mut Buffer,
    doc: &crate::editor_buffer::Doc,
    i: usize,
    x: u16,
    y: u16,
    text_w: usize,
) {
    let Some((a, b)) = doc.selection() else {
        return;
    };
    let ls = doc.line_start(i);
    let text = doc.line(i);
    let le = ls + text.len();
    if b <= ls || a > le {
        return;
    }
    let s = a.max(ls) - ls;
    let e = b.min(le) - ls;
    let from = crate::editor_buffer::line_width(&text[..s]);
    let mut to = crate::editor_buffer::line_width(&text[..e]);
    if b > le {
        to += 1;
    }
    for c in from..to.min(text_w) {
        let cx = x + c as u16;
        if buf.area.contains((cx, y).into()) {
            let cell = &mut buf[(cx, y)];
            cell.set_style(
                Style::default()
                    .bg(th().ui.text_selection_bg)
                    .fg(th().ui.text_selection_fg),
            );
        }
    }
}
