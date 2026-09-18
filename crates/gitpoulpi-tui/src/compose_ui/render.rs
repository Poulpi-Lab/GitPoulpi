use super::*;
use gitpoulpi_i18n::msg::compose_ui as msg;

impl ComposeForm {
    /// The small modal around the dropdown, always open.
    fn render_move(&self, frame: &mut Frame, area: Rect, m: &MoveFile) {
        let w = 64.min(area.width);
        let h = 12.min(area.height);
        if w < 30 || h < 6 {
            return;
        }
        let rect = crate::dialog_kit::centered(area, w, h);
        let buf = frame.buffer_mut();
        crate::dialog_kit::panel(buf, rect, &format!(" {} ", msg::MOVE_TITLE.get()));
        let file = self
            .commits
            .get(m.from.0)
            .and_then(|c| c.files.get(m.from.1))
            .cloned()
            .unwrap_or_default();
        crate::dialog_kit::put(
            buf,
            rect.x + 2,
            rect.y + 1,
            &file,
            rect.width.saturating_sub(4),
            Style::default().fg(th().ui.text),
        );
        let mut combo = m.combo.borrow_mut();
        combo.render_field(
            buf,
            Rect::new(rect.x + 2, rect.y + 3, rect.width.saturating_sub(4), 1),
            true,
            false,
        );
        combo.render_popup(buf, rect);
        crate::dialog_kit::put(
            buf,
            rect.x + 2,
            rect.bottom() - 2,
            msg::MOVE_KEYS.get(),
            rect.width.saturating_sub(4),
            Style::default().fg(th().ui.muted),
        );
    }

    /// The right pane while the review is open: the files the selected
    /// proposal will commit.
    pub(crate) fn render_files_pane(
        &self,
        frame: &mut Frame,
        area: Rect,
        hovered: Option<ComposePart>,
    ) -> Vec<(Rect, ComposePart)> {
        let mut hits = Vec::new();
        let Some((i, c)) = self.selected_commit() else {
            return hits;
        };
        let dim = Style::default().fg(th().ui.muted);
        frame.render_clipped(Clear, area);
        frame.render_clipped(
            Block::default()
                .title(format!(
                    " {} ",
                    msg::pane_title(c.files.len() as u64, i + 1)
                ))
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(th().ui.muted)),
            area,
        );
        let inner = Rect::new(
            area.x + 1,
            area.y + 1,
            area.width.saturating_sub(2),
            area.height.saturating_sub(2),
        );
        if inner.width == 0 || inner.height == 0 {
            return hits;
        }
        let mut lines: Vec<Line> = Vec::new();
        for (j, file) in c.files.iter().enumerate() {
            let selected = self.selected == ComposeRow::File(i, j);
            let hover = hovered == Some(ComposePart::PaneFile(j));
            let grabbed = self.dragging() && self.drag.is_some_and(|d| d.from == (i, j));
            let mut style = Style::default().fg(if c.dropped {
                th().ui.muted
            } else {
                th().ui.text
            });
            if selected {
                style = style.bg(th().ui.selection);
            } else if hover {
                style = style.bg(th().ui.hover);
            }
            if grabbed {
                style = style.add_modifier(Modifier::DIM);
            }
            if (j as u16) < inner.height {
                hits.push((
                    Rect::new(inner.x, inner.y + j as u16, inner.width, 1),
                    ComposePart::PaneFile(j),
                ));
            }
            lines.push(Line::from(Span::styled(
                format!(
                    "{:<w$}",
                    truncate_with_ellipsis(file, inner.width as usize),
                    w = inner.width as usize
                ),
                style,
            )));
        }
        if c.dropped {
            lines.push(Line::default());
            lines.push(Line::from(Span::styled(msg::DROPPED_STAYS.get(), dim)));
        }
        frame.render_clipped(Paragraph::new(lines), inner);
        hits
    }

    pub(crate) fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        hovered: Option<ComposePart>,
    ) -> Vec<(Rect, ComposePart)> {
        let mut hits = Vec::new();
        frame.render_clipped(Clear, area);
        frame.render_clipped(
            Block::default()
                .title(format!(" \u{2726} {} ", msg::TITLE.get()))
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(th().ui.accent))
                .style(Style::default().bg(th().ui.panel)),
            area,
        );
        let inner = Rect::new(
            area.x + 2,
            area.y + 1,
            area.width.saturating_sub(4),
            area.height.saturating_sub(2),
        );
        if inner.width < 40 || inner.height < 8 {
            return hits;
        }
        let dim = Style::default().fg(th().ui.muted);
        let strong = Style::default()
            .fg(th().ui.text)
            .add_modifier(Modifier::BOLD);
        let n_files: usize = self.commits.iter().map(|c| c.files.len()).sum();
        let kept = self.plan().len();
        frame.render_clipped(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    if n_files == 1 {
                        msg::proposes_one_file(self.commits.len() as u64)
                    } else {
                        msg::proposes(self.commits.len() as u64, n_files)
                    },
                    strong,
                ),
                Span::styled(format!("  \u{b7}  {}", msg::will_be_made(kept as u64)), dim),
            ])),
            Rect::new(inner.x, inner.y, inner.width, 1),
        );
        frame.render_clipped(
            Paragraph::new(Span::styled(
                truncate_with_ellipsis(msg::REVIEW_HINT.get(), inner.width as usize),
                dim,
            )),
            Rect::new(inner.x, inner.y + 1, inner.width, 1),
        );

        let rows_top = inner.y + 3;
        let footer_h = 3u16;
        let rows_h = inner.height.saturating_sub(3 + footer_h) as usize;
        let rows = self.rows();
        let cursor = rows.iter().position(|r| *r == self.selected).unwrap_or(0);
        let follow = (cursor + 1).saturating_sub(rows_h);
        let offset = match self.wheel.get() {
            Some((offset, anchor)) if anchor == self.selected => {
                offset.min(rows.len().saturating_sub(rows_h))
            }
            _ => {
                self.wheel.set(None);
                follow
            }
        };
        self.window.set((offset, rows_h, rows.len()));
        for (slot, row) in rows.iter().skip(offset).take(rows_h).enumerate() {
            let y = rows_top + slot as u16;
            let rect = Rect::new(inner.x, y, inner.width, 1);
            let selected = *row == self.selected;
            let hover_row = matches!(hovered, Some(ComposePart::Row(r)) if r == *row);
            if selected {
                frame.render_clipped(
                    Block::default().style(Style::default().bg(th().ui.selection)),
                    rect,
                );
            } else if hover_row {
                frame.render_clipped(
                    Block::default().style(Style::default().bg(th().ui.hover)),
                    rect,
                );
            }
            hits.push((rect, ComposePart::Row(*row)));
            match *row {
                ComposeRow::Commit(i) => {
                    let c = &self.commits[i];
                    let color = if c.dropped {
                        th().ui.muted
                    } else {
                        th().ui.text
                    };
                    let drop_target = self
                        .drag
                        .is_some_and(|d| d.moved && d.over == Some(i) && d.from.0 != i);
                    if drop_target {
                        frame.render_clipped(
                            Block::default().style(
                                Style::default()
                                    .bg(th().ui.accent)
                                    .fg(th().chrome.on_bright),
                            ),
                            rect,
                        );
                    }
                    hits.insert(0, (Rect::new(rect.x, y, 2, 1), ComposePart::Toggle(i)));
                    let mut spans = vec![
                        Span::styled(
                            format!("{} ", crate::fold_mark::glyph_open(c.expanded)),
                            Style::default().fg(th().ui.accent),
                        ),
                        Span::styled(
                            format!("{:>2} ", i + 1),
                            Style::default().fg(th().ui.accent),
                        ),
                    ];
                    // Controls first (right-aligned) so the message can take
                    // what is left.
                    let mut controls_w = 0u16;
                    if selected {
                        let controls = [
                            (" \u{25B2} ".to_string(), ComposePart::CommitUp(i)),
                            (" \u{25BC} ".to_string(), ComposePart::CommitDown(i)),
                            (" \u{270E} ".to_string(), ComposePart::CommitEdit(i)),
                            (
                                if c.dropped {
                                    format!(" {} ", msg::CTL_RESTORE.get())
                                } else {
                                    format!(" \u{2715} {} ", msg::CTL_DROP.get())
                                },
                                ComposePart::CommitDrop(i),
                            ),
                        ];
                        let total: u16 = controls
                            .iter()
                            .map(|(l, _)| display_width(l) as u16 + 1)
                            .sum();
                        let mut x = rect.right().saturating_sub(total);
                        controls_w = total;
                        for (label, part) in controls {
                            let w = display_width(&label) as u16;
                            let r = Rect::new(x, y, w, 1);
                            frame.render_clipped(
                                Paragraph::new(Span::styled(
                                    label,
                                    plain_button_style(hovered == Some(part)),
                                )),
                                r,
                            );
                            hits.insert(0, (r, part));
                            x += w + 1;
                        }
                    }
                    let avail = (rect.width as usize)
                        .saturating_sub(5 + controls_w as usize + 14)
                        .max(4);
                    let (summary, body) = crate::reword_ui::split_message(&c.message);
                    let mut style = Style::default().fg(color).add_modifier(Modifier::BOLD);
                    if c.dropped {
                        style = style.add_modifier(Modifier::CROSSED_OUT);
                    }
                    spans.push(Span::styled(truncate_with_ellipsis(&summary, avail), style));
                    if !body.trim().is_empty() {
                        spans.push(Span::styled(" \u{2261}", dim));
                    }
                    let files = c.files.len();
                    spans.push(Span::styled(
                        format!(
                            "  {}",
                            if c.dropped {
                                msg::row_files_dropped(files as u64)
                            } else {
                                msg::row_files(files as u64)
                            }
                        ),
                        dim,
                    ));
                    frame.render_clipped(Paragraph::new(Line::from(spans)), rect);
                }
                ComposeRow::File(i, j) => {
                    let c = &self.commits[i];
                    let color = if c.dropped {
                        th().ui.muted
                    } else {
                        th().ui.text
                    };
                    let grabbed = self.dragging() && self.drag.is_some_and(|d| d.from == (i, j));
                    let branch = if j + 1 == c.files.len() {
                        "\u{2514}\u{2500}"
                    } else {
                        "\u{251C}\u{2500}"
                    };
                    frame.render_clipped(
                        Paragraph::new(Line::from(vec![
                            Span::styled(format!("     {branch} "), dim),
                            Span::styled(
                                c.files[j].clone(),
                                if grabbed {
                                    Style::default().fg(color).add_modifier(Modifier::DIM)
                                } else {
                                    Style::default().fg(color)
                                },
                            ),
                        ])),
                        rect,
                    );
                    if selected {
                        let controls = [
                            (
                                format!(" \u{2191} {} ", msg::CTL_PREVIOUS.get()),
                                ComposePart::FilePrev(i, j),
                                i > 0,
                            ),
                            (
                                format!(" {} ", msg::CTL_MOVE.get()),
                                ComposePart::FileMove(i, j),
                                true,
                            ),
                            (
                                format!(" \u{2193} {} ", msg::CTL_NEXT.get()),
                                ComposePart::FileNext(i, j),
                                i + 1 < self.commits.len(),
                            ),
                            (
                                format!(" {} ", msg::CTL_OWN.get()),
                                ComposePart::FileNew(i, j),
                                c.files.len() > 1,
                            ),
                        ];
                        let total: u16 = controls
                            .iter()
                            .map(|(l, _, _)| display_width(l) as u16 + 1)
                            .sum();
                        let mut x = rect.right().saturating_sub(total);
                        for (label, part, enabled) in controls {
                            let w = display_width(&label) as u16;
                            let r = Rect::new(x, y, w, 1);
                            let style = if !enabled {
                                Style::default().fg(th().ui.muted)
                            } else {
                                plain_button_style(hovered == Some(part))
                            };
                            frame.render_clipped(Paragraph::new(Span::styled(label, style)), r);
                            if enabled {
                                hits.insert(0, (r, part));
                            }
                            x += w + 1;
                        }
                    }
                }
            }
        }

        let footer_y = inner.bottom() - 1;
        let message = self.error.clone().unwrap_or_else(|| {
            if let Some(d) = self.drag.filter(|d| d.moved) {
                let file = self.commits[d.from.0].files.get(d.from.1).cloned();
                return msg::moving(file.unwrap_or_default());
            }
            msg::footer_keys(crate::keymap::current().shown(Bind::ComposeApply))
        });
        frame.render_clipped(
            Paragraph::new(Span::styled(
                truncate_with_ellipsis(&message, inner.width as usize),
                if self.error.is_some() {
                    Style::default().fg(th().status.danger)
                } else {
                    dim
                },
            )),
            Rect::new(inner.x, footer_y, inner.width, 1),
        );
        let apply_label = format!(" {} ", msg::create(kept as u64));
        let buttons = [
            (
                ComposePart::Reset,
                format!(" {} ", msg::RESET.get()),
                th().chrome.action_reset,
                th().chrome.tint_reset,
            ),
            (
                ComposePart::Cancel,
                format!(" {} ", gitpoulpi_i18n::msg::dialog::CANCEL.get()),
                th().status.danger,
                th().chrome.tint_danger,
            ),
            (
                ComposePart::Apply,
                apply_label,
                th().refs.local,
                th().chrome.tint_success,
            ),
        ];
        let total: u16 = buttons
            .iter()
            .map(|(_, l, _, _)| display_width(l) as u16 + 1)
            .sum::<u16>()
            - 1;
        let mut x = inner.x + inner.width.saturating_sub(total);
        for (part, label, fg, bg) in buttons {
            let w = display_width(&label) as u16;
            let style = if hovered == Some(part) {
                plain_button_style(true)
            } else {
                Style::default().fg(fg).bg(bg)
            };
            let rect = Rect::new(x, footer_y - 1, w, 1);
            frame.render_clipped(Paragraph::new(Span::styled(label, style)), rect);
            hits.insert(0, (rect, part));
            x += w + 1;
        }
        if let Some(m) = &self.moving {
            hits.clear();
            self.render_move(frame, area, m);
        }
        if let Some((_, form)) = &self.editing {
            // The popup is modal: only its own parts take clicks.
            hits = form
                .render(
                    frame,
                    area,
                    match hovered {
                        Some(ComposePart::Editing(part)) => Some(part),
                        _ => None,
                    },
                )
                .into_iter()
                .map(|(rect, part)| (rect, ComposePart::Editing(part)))
                .collect();
        }
        hits
    }
}
