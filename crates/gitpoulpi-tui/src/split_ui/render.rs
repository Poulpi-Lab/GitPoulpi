//! The split editor's paint walk and its mouse.
//!
//! One walk paints and registers: every rect in the returned hit list is the
//! one a glyph was drawn in (`where-we-are.md`, "Hit targets"). The list is
//! ordered inner-first (a row's `[N v]` field before the row), and the whole
//! editor goes in front of the panes behind it (`overlays.rs`).

use super::*;
use crate::dialog_kit::{
    self, button, button_width, checkbox, CheckState, CloseHit, Dismiss, DismissKind, Guard,
};
use crate::popup_list::ClickPair;
use crate::reword_ui::RewordPart;
use crate::theme::plain_button_style;
use std::cell::{Cell, RefCell};

/// The clickable parts of the editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SplitHit {
    /// The close cross on the frame.
    Close,
    Cancel,
    Reset,
    /// The Split button.
    Apply,
    /// The "Sign commits" checkbox.
    Sign,
    /// A commit of the plan.
    Part(usize),
    PartEdit(usize),
    PartUp(usize),
    PartDown(usize),
    PartDelete(usize),
    /// "+ New commit".
    NewPart,
    /// A file, hunk or line of the changes pane.
    Row(SplitRow),
    /// The `[N v]` field of a row: opens the list of commits.
    Assign(SplitRow),
    /// The list of commits is open and owns the pointer.
    Modal,
    /// A part of the embedded message editor.
    Edit(RewordPart),
}

/// What the last paint learned. Interior mutability: the editor is drawn
/// from a shared reference, like the composer's.
#[derive(Debug, Clone, Default)]
pub(super) struct View {
    /// The list of commits a change can be given to, while open.
    pub(super) combo: RefCell<Option<Combo>>,
    /// The editor's frame as last painted (a click outside is dismissal).
    frame: Cell<Rect>,
    /// First changes row shown, rows in all, rows that fit.
    top: Cell<usize>,
    len: Cell<usize>,
    height: Cell<usize>,
    /// The cursor the window last followed.
    followed: Cell<usize>,
    flash: Cell<bool>,
    clicks: Cell<ClickPair>,
    /// The list's scrollbar as last painted, and the thumb held.
    pub(super) bar: Cell<Option<crate::scrollbar::Track>>,
    bar_drag: Cell<crate::scrollbar::BarDrag>,
}

impl PartialEq for View {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl View {
    pub(super) fn scroll(&self, delta: isize) {
        let max = self.len.get().saturating_sub(self.height.get());
        self.top
            .set((self.top.get() as isize + delta).clamp(0, max as isize) as usize);
    }

    /// The list's scrollbar: a press on it, a drag of the thumb. `true` when
    /// the bar took the event; the window then stays where it was put.
    pub(super) fn bar_mouse(&self, mouse: crossterm::event::MouseEvent) -> bool {
        let mut drag = self.bar_drag.get();
        let top = drag.on_mouse(self.bar.get(), mouse, self.len.get(), self.top.get());
        self.bar_drag.set(drag);
        match top {
            Some(top) => {
                self.top.set(top);
                true
            }
            None => false,
        }
    }

    pub(super) fn flash_cross(&self) {
        self.flash.set(true);
    }
}

/// Answers `dismiss` for the editor: the one rule needs a `Dismiss`, the
/// editor only knows whether its plan was edited.
pub(super) struct Probe {
    dirty: bool,
    close: CloseHit,
}

impl Probe {
    pub(super) fn new(dirty: bool) -> Self {
        Self {
            dirty,
            close: CloseHit::default(),
        }
    }
}

impl Dismiss for Probe {
    fn is_dirty(&self) -> bool {
        self.dirty
    }
    fn guard(&self) -> Guard {
        Guard::IgnoreOutsideWhenDirty
    }
    fn close_hit(&self) -> &CloseHit {
        &self.close
    }
    fn close_hit_mut(&mut self) -> &mut CloseHit {
        &mut self.close
    }
}

/// The colour of commit `k` (the graph's lane palette, so it reads as "a
/// commit" everywhere).
fn part_color(k: usize) -> Color {
    let palette = th().graph.palette;
    palette
        .get(k % palette.len().max(1))
        .copied()
        .unwrap_or(th().ui.accent)
}

/// The one-cell-ish owner mark of a row: the commit's number in its colour.
fn owner_mark(owner: RowOwner) -> (String, Color) {
    match owner {
        RowOwner::Part(k) if k < 9 => ((k + 1).to_string(), part_color(k)),
        RowOwner::Part(k) => (format!("{}", k + 1), part_color(k)),
        RowOwner::Mixed => ("*".into(), th().status.warning),
        RowOwner::Unassigned => ("\u{b7}".into(), th().status.warning),
    }
}

fn whole_reason(reason: gitpoulpi_core::domain::WholeReason) -> &'static str {
    use gitpoulpi_core::domain::WholeReason as W;
    match reason {
        W::Binary => msg::WHOLE_BINARY.get(),
        W::ModeOnly => msg::WHOLE_MODE.get(),
        W::Added => msg::WHOLE_ADDED.get(),
        W::Deleted => msg::WHOLE_DELETED.get(),
        W::Submodule => msg::WHOLE_SUBMODULE.get(),
        W::Lfs => msg::WHOLE_LFS.get(),
    }
}

fn status_letter(status: gitpoulpi_core::domain::FileStatus) -> char {
    use gitpoulpi_core::domain::FileStatus as S;
    match status {
        S::Untracked | S::Added | S::Staged => 'A',
        S::Modified => 'M',
        S::Conflicted => 'U',
        S::Renamed => 'R',
        S::Deleted => 'D',
    }
}

/// The text of a changed line: its first line, tabs spread, no newline.
fn line_text(content: &str) -> String {
    content
        .trim_end_matches(['\n', '\r'])
        .replace('\t', "    ")
        .chars()
        .filter(|c| !c.is_control())
        .collect()
}

impl SplitForm {
    /// Puts `hit` into effect: the mouse's way into the same methods the keys
    /// use.
    pub(crate) fn on_press(&mut self, hit: SplitHit, col: u16, row: u16) -> SplitFormOutcome {
        if self.applying {
            return SplitFormOutcome::Stay;
        }
        self.error = None;
        if let (SplitHit::Edit(part), Some((_, form))) = (hit, self.editing.as_mut()) {
            let outcome = form.on_press(part, col, row);
            return self.finish_editing(outcome);
        }
        match hit {
            SplitHit::Close => return self.request_dismiss(DismissKind::Cross),
            SplitHit::Cancel => return self.request_dismiss(DismissKind::Cancel),
            SplitHit::Reset => self.reset(),
            SplitHit::Apply => return self.try_apply(),
            SplitHit::Sign => self.sign = !self.sign,
            SplitHit::Part(k) => {
                self.select_part(k);
                let mut clicks = self.view.clicks.get();
                let double = clicks.register(k);
                self.view.clicks.set(clicks);
                if double {
                    self.start_edit(k);
                }
            }
            SplitHit::PartEdit(k) => {
                self.select_part(k);
                self.start_edit(k);
            }
            SplitHit::PartUp(k) => self.move_part(k, -1),
            SplitHit::PartDown(k) => self.move_part(k, 1),
            SplitHit::PartDelete(k) => self.remove_part(k),
            SplitHit::NewPart => {
                self.add_part();
                self.pane = SplitPane::Parts;
            }
            SplitHit::Row(r) => self.select_row(r),
            SplitHit::Assign(r) => {
                self.select_row(r);
                self.open_assign_combo();
            }
            SplitHit::Modal | SplitHit::Edit(_) => {}
        }
        SplitFormOutcome::Stay
    }

    /// Whether `(col, row)` is outside the editor as last painted.
    pub(crate) fn is_outside(&self, col: u16, row: u16) -> bool {
        let frame = self.view.frame.get();
        frame.width > 0 && !frame.contains((col, row).into())
    }

    /// Draws the editor over `area` and returns its click targets, from the
    /// walk that painted them.
    pub(crate) fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        hovered: Option<SplitHit>,
    ) -> Vec<(Rect, SplitHit)> {
        // Specific targets first, the bands that contain them after.
        let mut fine: Vec<(Rect, SplitHit)> = Vec::new();
        let mut coarse: Vec<(Rect, SplitHit)> = Vec::new();
        self.view.frame.set(area);
        let id = self.source.commit.to_hex();
        let summary = crate::reword_ui::split_message(&self.source.message).0;
        let title = msg::title(&id[..id.len().min(7)], &summary);
        {
            let buf = frame.buffer_mut();
            dialog_kit::panel(
                buf,
                area,
                &truncate_with_ellipsis(&title, area.width.saturating_sub(10) as usize),
            );
            let block = area.intersection(buf.area);
            if let Some(cross) =
                crate::chrome::close_cross::paint(buf, block, hovered == Some(SplitHit::Close))
            {
                if self.view.flash.take() {
                    crate::chrome::close_cross::flash(buf, block);
                }
                fine.push((cross, SplitHit::Close));
            }
        }
        let inner = Rect::new(
            area.x + 1,
            area.y + 1,
            area.width.saturating_sub(2),
            area.height.saturating_sub(2),
        );
        if inner.width < 40 || inner.height < 9 {
            fine.extend(coarse);
            return fine;
        }
        let body = Rect::new(inner.x, inner.y, inner.width, inner.height - 2);
        let left_w = (inner.width * 2 / 5).clamp(24, 40);
        let left = Rect::new(body.x, body.y, left_w, body.height);
        let right = Rect::new(
            left.right() + 1,
            body.y,
            body.right().saturating_sub(left.right() + 1),
            body.height,
        );
        let dim = Style::default().fg(th().ui.muted);
        {
            let buf = frame.buffer_mut();
            for y in body.y..body.bottom() {
                dialog_kit::put(buf, left.right(), y, "\u{2502}", 1, dim.bg(th().ui.panel));
            }
        }
        self.paint_parts(frame, left, hovered, &mut fine, &mut coarse);
        self.paint_changes(frame, right, hovered, &mut fine, &mut coarse);
        self.paint_footer(frame, inner, hovered, &mut fine);

        if let Some((k, form)) = &self.editing {
            // The message editor is modal: only its own parts take clicks.
            let _ = k;
            return form
                .render(
                    frame,
                    area,
                    match hovered {
                        Some(SplitHit::Edit(p)) => Some(p),
                        _ => None,
                    },
                )
                .into_iter()
                .map(|(r, p)| (r, SplitHit::Edit(p)))
                .collect();
        }
        if let Some(combo) = self.view.combo.borrow_mut().as_mut() {
            combo.render_popup(frame.buffer_mut(), area);
            return vec![(area, SplitHit::Modal)];
        }
        fine.extend(coarse);
        fine
    }

    fn paint_parts(
        &self,
        frame: &mut Frame,
        left: Rect,
        hovered: Option<SplitHit>,
        fine: &mut Vec<(Rect, SplitHit)>,
        coarse: &mut Vec<(Rect, SplitHit)>,
    ) {
        let dim = Style::default().fg(th().ui.muted);
        let focused = self.pane() == SplitPane::Parts;
        let buf = frame.buffer_mut();
        let w = left.width;
        dialog_kit::put(
            buf,
            left.x,
            left.y,
            msg::PARTS_TITLE.get(),
            w,
            Style::default()
                .fg(if focused {
                    th().ui.accent
                } else {
                    th().ui.text
                })
                .add_modifier(Modifier::BOLD),
        );
        let unassigned = self.unassigned_count();
        if unassigned > 0 {
            dialog_kit::put(
                buf,
                left.x,
                left.y + 1,
                &format!("! {}", msg::unassigned_left(unassigned as u64)),
                w,
                Style::default().fg(th().status.warning),
            );
        }
        // Heading, the unassigned line, the New row and the sign rows around
        // the list.
        let list_h = (left.height.saturating_sub(5) as usize).max(1);
        let n = self.part_count();
        let first = (self.selected_part() + 1)
            .saturating_sub(list_h)
            .min(n.saturating_sub(list_h));
        let with_controls = w >= 30;
        for (slot, k) in (first..n).take(list_h).enumerate() {
            let y = left.y + 2 + slot as u16;
            let rect = Rect::new(left.x, y, w, 1);
            let selected = k == self.selected_part();
            let hover = matches!(
                hovered,
                Some(SplitHit::Part(h) | SplitHit::PartEdit(h) | SplitHit::PartUp(h)
                    | SplitHit::PartDown(h) | SplitHit::PartDelete(h)) if h == k
            );
            if selected && focused {
                dialog_kit::fill(buf, rect, Style::default().bg(th().ui.selection));
            } else if hover {
                dialog_kit::fill(buf, rect, Style::default().bg(th().ui.hover));
            }
            let bg = if selected && focused {
                th().ui.selection
            } else if hover {
                th().ui.hover
            } else {
                th().ui.panel
            };
            let mut right_edge = rect.right();
            // The piece count, then (selected row) the controls.
            let count = format!("{} ", self.part_pieces(k));
            let count_w = display_width(&count) as u16;
            right_edge -= count_w;
            dialog_kit::put(buf, right_edge, y, &count, count_w, dim.bg(bg));
            if selected && with_controls {
                let controls = [
                    (" \u{270E} ", SplitHit::PartEdit(k), true),
                    (" \u{25B2} ", SplitHit::PartUp(k), k > 0),
                    (" \u{25BC} ", SplitHit::PartDown(k), k + 1 < n),
                    (" \u{2715} ", SplitHit::PartDelete(k), n > 2),
                ];
                for (label, hit, enabled) in controls.into_iter().rev() {
                    let cw = display_width(label) as u16;
                    if right_edge < rect.x + 8 + cw {
                        break;
                    }
                    right_edge -= cw;
                    let r = Rect::new(right_edge, y, cw, 1);
                    let style = if enabled {
                        plain_button_style(hovered == Some(hit)).bg(if hovered == Some(hit) {
                            th().ui.button_hover
                        } else {
                            bg
                        })
                    } else {
                        Style::default().fg(th().ui.disabled).bg(bg)
                    };
                    dialog_kit::put(buf, r.x, y, label, cw, style);
                    if enabled {
                        fine.push((r, hit));
                    }
                }
            }
            let num = format!("{} ", k + 1);
            let num_w = display_width(&num) as u16;
            dialog_kit::put(
                buf,
                rect.x,
                y,
                &num,
                num_w,
                Style::default()
                    .fg(part_color(k))
                    .bg(bg)
                    .add_modifier(Modifier::BOLD),
            );
            let (summary, _) = crate::reword_ui::split_message(self.part_message(k));
            let (text, style) = if summary.trim().is_empty() {
                (msg::NO_MESSAGE.get().to_string(), dim.bg(bg))
            } else {
                (summary, Style::default().fg(th().ui.text).bg(bg))
            };
            dialog_kit::put(
                buf,
                rect.x + num_w,
                y,
                &text,
                right_edge.saturating_sub(rect.x + num_w + 1),
                style,
            );
            coarse.push((rect, SplitHit::Part(k)));
        }
        let shown = (n - first).min(list_h);
        let new_y = left.y + 2 + shown as u16;
        if new_y < left.bottom().saturating_sub(2) {
            let label = msg::NEW_PART.get();
            let hover = hovered == Some(SplitHit::NewPart);
            let r = dialog_kit::put(
                buf,
                left.x,
                new_y,
                label,
                w,
                Style::default().fg(th().ui.accent).bg(if hover {
                    th().ui.hover
                } else {
                    th().ui.panel
                }),
            );
            fine.push((r, SplitHit::NewPart));
        }
        // The sign checkbox and, unchecked, the note about dropped signatures.
        let sign_y = left.bottom().saturating_sub(2);
        let r = checkbox(
            buf,
            left.x,
            sign_y,
            w,
            msg::SIGN.get(),
            self.sign(),
            CheckState::Enabled {
                hovered: hovered == Some(SplitHit::Sign),
                focused: false,
            },
        );
        fine.push((Rect::new(r.x, r.y, r.width.max(1), 1), SplitHit::Sign));
        if !self.sign() {
            dialog_kit::put(
                buf,
                left.x,
                sign_y + 1,
                msg::SIGN_OFF_NOTE.get(),
                w,
                dim.bg(th().ui.panel),
            );
        }
    }

    fn paint_changes(
        &self,
        frame: &mut Frame,
        right: Rect,
        hovered: Option<SplitHit>,
        fine: &mut Vec<(Rect, SplitHit)>,
        coarse: &mut Vec<(Rect, SplitHit)>,
    ) {
        let dim = Style::default().fg(th().ui.muted);
        let focused = self.pane() == SplitPane::Changes;
        let rows = self.rows();
        let h = right.height.saturating_sub(1) as usize;
        // Follow the cursor when it moved; the wheel moves the window alone.
        let mut top = self.view.top.get();
        if self.view.followed.get() != self.row_cursor {
            self.view.followed.set(self.row_cursor);
            if self.row_cursor < top {
                top = self.row_cursor;
            } else if self.row_cursor >= top + h {
                top = self.row_cursor + 1 - h.max(1);
            }
        }
        top = top.min(rows.len().saturating_sub(h));
        self.view.top.set(top);
        self.view.len.set(rows.len());
        self.view.height.set(h);
        // The bar rides the editor's right border, beside the rows.
        let track = crate::scrollbar::Track::column(
            Rect::new(self.view.frame.get().right() - 1, right.y + 1, 1, h as u16),
            rows.len(),
            h,
            top,
        );
        self.view.bar.set(track);
        if let Some(track) = track {
            crate::scrollbar::render(frame, track, true, false);
        }

        let selected_rows = self.selection();
        let buf = frame.buffer_mut();
        let mut heading = msg::CHANGES_TITLE.get().to_string();
        if self.line_mode() {
            heading.push_str(&format!("  \u{b7} {}", msg::LINE_MODE.get()));
        }
        dialog_kit::put(
            buf,
            right.x,
            right.y,
            &heading,
            right.width,
            Style::default()
                .fg(if focused {
                    th().ui.accent
                } else {
                    th().ui.text
                })
                .add_modifier(Modifier::BOLD),
        );
        let position = format!("{}/{}", self.row_cursor + 1, rows.len());
        let pos_w = display_width(&position) as u16;
        if right.width > pos_w + 24 {
            dialog_kit::put(buf, right.right() - pos_w, right.y, &position, pos_w, dim);
        }
        let mut cursor_field = None;
        for (slot, row) in rows.iter().skip(top).take(h).enumerate() {
            let y = right.y + 1 + slot as u16;
            let rect = Rect::new(right.x, y, right.width, 1);
            let is_cursor = slot + top == self.row_cursor;
            let selected = selected_rows.contains(row) && focused;
            let hover =
                matches!(hovered, Some(SplitHit::Row(r) | SplitHit::Assign(r)) if r == *row);
            let bg = if selected {
                th().ui.selection
            } else if hover {
                th().ui.hover
            } else {
                th().ui.panel
            };
            dialog_kit::fill(buf, rect, Style::default().bg(bg));
            let owner = self.row_owner(*row);
            // The combo field at the right end, then the gutter at the left.
            let (mark, color) = owner_mark(owner);
            let field = format!("[{mark:>2} \u{25BE}]");
            let field_w = display_width(&field) as u16;
            let field_rect = Rect::new(rect.right().saturating_sub(field_w + 1), y, field_w, 1);
            let field_hover = hovered == Some(SplitHit::Assign(*row));
            dialog_kit::put(
                buf,
                field_rect.x,
                y,
                &field,
                field_w,
                Style::default()
                    .fg(if field_hover {
                        th().ui.on_button
                    } else {
                        color
                    })
                    .bg(if field_hover {
                        th().ui.button_hover
                    } else {
                        bg
                    }),
            );
            fine.push((field_rect, SplitHit::Assign(*row)));
            if is_cursor {
                cursor_field = Some(field_rect);
            }
            let gutter = Style::default()
                .fg(color)
                .bg(bg)
                .add_modifier(Modifier::BOLD);
            dialog_kit::put(buf, rect.x, y, &mark, 2, gutter);
            let text_x = rect.x + 2;
            let text_w = field_rect.x.saturating_sub(text_x + 1);
            match *row {
                SplitRow::File(fi) => {
                    let file = &self.source.files[fi];
                    let letter = status_letter(file.status);
                    let path = file.path.display().to_string();
                    let name = format!("{letter} {path}");
                    let name_r = dialog_kit::put(
                        buf,
                        text_x,
                        y,
                        &name,
                        text_w,
                        Style::default()
                            .fg(th().ui.text)
                            .bg(bg)
                            .add_modifier(Modifier::BOLD),
                    );
                    if let Some(reason) = file.whole_only {
                        let used = name_r.width + 2;
                        if text_w > used {
                            dialog_kit::put(
                                buf,
                                text_x + used,
                                y,
                                whole_reason(reason),
                                text_w - used,
                                dim.bg(bg),
                            );
                        }
                    }
                }
                SplitRow::Hunk(fi, hi) => {
                    let hunk = &self.source.files[fi].hunks[hi];
                    let header = format!(
                        "  @@ -{},{} +{},{} @@",
                        hunk.old_start, hunk.old_lines, hunk.new_start, hunk.new_lines
                    );
                    dialog_kit::put(
                        buf,
                        text_x,
                        y,
                        &header,
                        text_w,
                        Style::default().fg(th().ui.accent).bg(bg),
                    );
                }
                SplitRow::Line(fi, hi, li) => {
                    let line = &self.source.files[fi].hunks[hi].lines[li as usize];
                    let (sign, fg) = match line.origin {
                        LineOrigin::Add => ("+", th().diff.add_fg),
                        LineOrigin::Delete => ("-", th().diff.del_fg),
                        LineOrigin::Context => (" ", th().ui.text),
                    };
                    dialog_kit::put(
                        buf,
                        text_x,
                        y,
                        &format!("    {sign} {}", line_text(&line.content)),
                        text_w,
                        Style::default().fg(fg).bg(bg),
                    );
                }
            }
            coarse.push((rect, SplitHit::Row(*row)));
        }
        // The list of commits hangs from the cursor row's field (or the
        // heading when the row is scrolled out).
        if let Some(combo) = self.view.combo.borrow_mut().as_mut() {
            combo.set_anchor(
                cursor_field
                    .unwrap_or_else(|| Rect::new(right.right().saturating_sub(8), right.y, 7, 1)),
            );
        }
    }

    fn paint_footer(
        &self,
        frame: &mut Frame,
        inner: Rect,
        hovered: Option<SplitHit>,
        fine: &mut Vec<(Rect, SplitHit)>,
    ) {
        let valid = self.validation().is_ok() && !self.applying;
        let buf = frame.buffer_mut();
        let by = inner.bottom() - 2;
        let ky = inner.bottom() - 1;
        let reset = gitpoulpi_i18n::msg::compose_ui::RESET.get();
        let cancel = gitpoulpi_i18n::msg::dialog::CANCEL.get();
        let split = msg::split_into(self.part_count() as u64);
        let total = button_width(reset) + 1 + button_width(cancel) + 1 + button_width(&split);
        let mut x = inner.right().saturating_sub(total);
        let r = button(
            buf,
            x,
            by,
            reset,
            dialog_kit::ButtonState::Normal {
                hovered: hovered == Some(SplitHit::Reset),
                focused: false,
            },
        );
        fine.push((r, SplitHit::Reset));
        x += r.width + 1;
        let r = button(
            buf,
            x,
            by,
            cancel,
            dialog_kit::ButtonState::Normal {
                hovered: hovered == Some(SplitHit::Cancel),
                focused: false,
            },
        );
        fine.push((r, SplitHit::Cancel));
        x += r.width + 1;
        let r = button(
            buf,
            x,
            by,
            &split,
            if valid {
                dialog_kit::ButtonState::Primary {
                    hovered: hovered == Some(SplitHit::Apply),
                    focused: false,
                }
            } else {
                dialog_kit::ButtonState::Disabled
            },
        );
        fine.push((r, SplitHit::Apply));
        // The status: in flight, the last refusal, or why Split is disabled.
        let room = total_room(inner.width, total);
        let (text, color) = if self.applying {
            (msg::SPLITTING.get().to_string(), th().ui.accent)
        } else if let Some(error) = &self.error {
            (error.clone(), th().status.danger)
        } else if let Some(why) = self.status_text() {
            (why, th().status.warning)
        } else {
            (String::new(), th().ui.muted)
        };
        dialog_kit::put(
            buf,
            inner.x,
            by,
            &text,
            room,
            Style::default().fg(color).bg(th().ui.panel),
        );
        let keys = msg::footer_keys(crate::keymap::current().shown(Bind::ComposeApply));
        dialog_kit::put(
            buf,
            inner.x,
            ky,
            &keys,
            inner.width,
            Style::default().fg(th().ui.muted).bg(th().ui.panel),
        );
    }
}

/// Columns left for the status text beside the buttons.
fn total_room(width: u16, buttons: u16) -> u16 {
    width.saturating_sub(buttons + 1)
}

/// The editor's own target under the pointer, among its targets only.
pub(crate) fn split_hit_at(
    hits: &[(Rect, crate::chrome::HitTarget)],
    col: u16,
    row: u16,
) -> Option<SplitHit> {
    hits.iter().find_map(|(rect, target)| match target {
        crate::chrome::HitTarget::SplitForm(part) if rect.contains((col, row).into()) => {
            Some(*part)
        }
        _ => None,
    })
}
