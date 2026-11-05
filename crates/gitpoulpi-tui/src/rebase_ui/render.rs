use super::*;
use gitpoulpi_i18n::msg::rebase_ui as msg;

impl RebaseForm {
    /// Draws the view over all of `area` (the central graph column) and
    /// returns its hit rects, most specific first (pills before the row
    /// they sit on). `frame_area` is only used by the embedded message
    /// editor, which is a real popup. `hovered` is the part under the
    /// pointer: every control lights with `plain_button_style(true)`.
    pub(crate) fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        frame_area: Rect,
        hovered: Option<RebasePart>,
        theme: crate::graph::LaneTheme,
    ) -> Vec<(Rect, RebasePart)> {
        let mut hits = Vec::new();
        frame.render_clipped(Clear, area);
        let title = format!(
            " {} ",
            if self.cherry_pick {
                msg::TITLE_CHERRY_PICK.get()
            } else {
                msg::TITLE_REBASE.get()
            }
        );
        frame.render_clipped(
            Block::default()
                .title(title)
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
        if inner.width < 30 || inner.height < 7 {
            return hits;
        }
        let w = inner.width as usize;

        // Header: what is being done, to what, onto what -- and, at the
        // right, the button that walks away from all of it.
        let n = self.rows.len();
        let cancel_label = format!(
            " {} ",
            if self.cherry_pick {
                msg::CANCEL_CHERRY_PICK.get()
            } else {
                msg::CANCEL_REBASE.get()
            }
        );
        let cancel_label = cancel_label.as_str();
        let cancel_w = display_width(cancel_label) as u16;
        let cancel_rect = Rect::new(
            inner.x + inner.width.saturating_sub(cancel_w),
            inner.y,
            cancel_w.min(inner.width),
            1,
        );
        let dim = Style::default().fg(th().ui.muted);
        let strong = Style::default()
            .fg(th().ui.text)
            .add_modifier(Modifier::BOLD);
        let branch_pill = Style::default()
            .fg(th().ui.on_button)
            .bg(th().chrome.pill_branch);
        let onto_pill = Style::default()
            .fg(th().ui.on_button)
            .bg(th().chrome.pill_onto);
        let mut header: Vec<Span> = Vec::new();
        if self.cherry_pick {
            header.push(Span::styled(
                format!("{} ", msg::header_cherry_picking(n as u64)),
                strong,
            ));
            header.push(Span::styled(self.title.clone(), onto_pill));
            if let Some(sha) = &self.head_sha {
                header.push(Span::raw(" "));
                header.push(Span::styled(sha.clone(), onto_pill));
            }
        } else {
            header.push(Span::styled(
                format!("{} ", msg::HEADER_REBASING.get()),
                strong,
            ));
            header.push(Span::styled(self.title.clone(), branch_pill));
            match &self.onto_label {
                Some(onto) => {
                    header.push(Span::styled(
                        format!(" {} ", msg::HEADER_ONTO.get()),
                        strong,
                    ));
                    header.push(Span::styled(onto.clone(), onto_pill));
                }
                None => header.push(Span::styled(
                    format!(" \u{b7} {}", msg::header_count(n as u64)),
                    dim,
                )),
            }
        }
        frame.render_clipped(
            Paragraph::new(Line::from(header)),
            Rect::new(
                inner.x,
                inner.y,
                inner.width.saturating_sub(cancel_w + 1),
                1,
            ),
        );
        frame.render_clipped(
            Paragraph::new(Span::styled(
                cancel_label,
                if hovered == Some(RebasePart::Cancel) {
                    plain_button_style(true)
                } else {
                    Style::default()
                        .fg(th().status.danger)
                        .bg(th().chrome.tint_danger)
                },
            )),
            cancel_rect,
        );
        hits.push((cancel_rect, RebasePart::Cancel));
        frame.render_clipped(
            Paragraph::new(Span::styled(
                truncate_with_ellipsis(
                    &msg::hint(
                        crate::keymap::current().shown(Bind::RebaseMoveUp),
                        crate::keymap::current().shown(Bind::RebaseMoveDown),
                    ),
                    w,
                ),
                Style::default().fg(th().ui.muted),
            )),
            Rect::new(inner.x, inner.y + 1, inner.width, 1),
        );

        let label_w = Self::action_label_width();
        let rows_top = inner.y + 3;
        let rows_h = (inner.height - 3 - 3) as usize;
        let display = self.display_rows();
        let follow = (self.cursor + 1).saturating_sub(rows_h);
        let offset = match self.scroll.get() {
            Some((offset, anchor)) if anchor == self.cursor => {
                offset.min(display.len().saturating_sub(rows_h))
            }
            _ => {
                self.scroll.set(None);
                follow
            }
        };
        self.window.set((offset, rows_h, display.len()));
        // The bar rides the frame's right border, beside the rows.
        let track = crate::scrollbar::Track::column(
            Rect::new(area.right() - 1, rows_top, 1, rows_h as u16),
            display.len(),
            rows_h,
            offset,
        );
        self.bar.set(track);
        if let Some(track) = track {
            crate::scrollbar::render(frame, track, true, false);
        }
        let mut open_list: Option<(u16, usize)> = None;
        for (slot, (i, row)) in display
            .iter()
            .enumerate()
            .skip(offset)
            .take(rows_h)
            .enumerate()
        {
            let y = rows_top + slot as u16;
            let rect = Rect::new(inner.x, y, inner.width, 1);
            let hovered_row = matches!(
                hovered,
                Some(RebasePart::Row(r) | RebasePart::Action(r, _)) if r == i
            );
            if i == self.cursor {
                frame.render_clipped(
                    Block::default().style(Style::default().bg(th().ui.selection)),
                    rect,
                );
            } else if hovered_row {
                frame.render_clipped(
                    Block::default().style(Style::default().bg(th().ui.hover)),
                    rect,
                );
            }
            let dropped = row.action == RebaseAction::Drop;
            let folded = matches!(row.action, RebaseAction::Squash | RebaseAction::Fixup);
            let color = Self::action_color(row.action);
            // Drag handle and the lane node.
            let handle_style = if hovered == Some(RebasePart::Row(i)) {
                Style::default().fg(th().ui.on_button)
            } else {
                Style::default().fg(th().ui.muted)
            };
            frame.render_clipped(
                Paragraph::new(Span::styled("\u{2261}", handle_style)),
                Rect::new(inner.x, y, 1, 1),
            );
            // The action dropdown button: the row's current action, in
            // its color, with a chevron. Its list is drawn after all rows.
            let button_x = inner.x + 2;
            let label = format!(
                " {}{} \u{25BE} ",
                Self::action_label(row.action),
                " ".repeat(label_w.saturating_sub(display_width(Self::action_label(row.action))))
            );
            let bw = display_width(&label) as u16;
            let part = RebasePart::Dropdown(i);
            let bstyle = if hovered == Some(part) {
                plain_button_style(true)
            } else {
                Style::default()
                    .fg(color)
                    .bg(th().chrome.dropdown_bg)
                    .add_modifier(Modifier::BOLD)
            };
            let brect = Rect::new(button_x, y, bw, 1);
            if button_x + bw <= inner.x + inner.width {
                frame.render_clipped(Paragraph::new(Span::styled(label, bstyle)), brect);
                hits.push((brect, part));
            }
            if self.dropdown == Some(i) {
                open_list = Some((y, i));
            }
            // The node -- for a squash/fixup, the arrow into the row below.
            // A kept commit's dot is drawn in the graph's first lane color
            // (the rows are one branch's worth of commits), in the active
            // `ui.graph_color_theme`.
            let color = if dropped || folded {
                color
            } else {
                crate::graph::lane_palette(theme)[0]
            };
            let node = if dropped {
                "\u{25CB}"
            } else if folded {
                "\u{2193}"
            } else {
                "\u{25CF}"
            };
            let mut x = button_x + bw + 1;
            frame.render_clipped(
                Paragraph::new(Span::styled(node, Style::default().fg(color))),
                Rect::new(x, y, 1, 1),
            );
            let right_edge = inner.x + inner.width;

            // Short hash and message (the new one, for a reword).
            x += 2;
            let sha = row.commit.to_hex()[..SHORT_SHA_LEN].to_string();
            let shown = if row.action == RebaseAction::Reword {
                row.new_message.as_deref().unwrap_or(&row.summary)
            } else {
                &row.summary
            };
            let first_line = shown.lines().next().unwrap_or("");
            let text = format!("{sha} {first_line}");
            // The "split..." button takes the right end of a row that has
            // room for it next to a readable message.
            let split_label = format!(" {} ", msg::SPLIT_BUTTON.get());
            let split_w = display_width(&split_label) as u16;
            let show_split =
                self.can_split() && !dropped && right_edge.saturating_sub(x) >= split_w + 20;
            let text_w = right_edge.saturating_sub(x) - if show_split { split_w + 1 } else { 0 };
            let text_style = if dropped {
                Style::default()
                    .fg(th().ui.muted)
                    .add_modifier(Modifier::CROSSED_OUT)
            } else if folded {
                Style::default().fg(th().ui.muted)
            } else if row.action == RebaseAction::Reword {
                Style::default().fg(th().ui.accent)
            } else {
                Style::default().fg(th().ui.text)
            };
            if text_w > 0 {
                frame.render_clipped(
                    Paragraph::new(Span::styled(
                        truncate_with_ellipsis(&text, text_w as usize),
                        text_style,
                    )),
                    Rect::new(x, y, text_w, 1),
                );
            }
            if show_split {
                let part = RebasePart::Split(i);
                let srect = Rect::new(right_edge - split_w, y, split_w, 1);
                let style = if hovered == Some(part) {
                    plain_button_style(true)
                } else {
                    Style::default()
                        .fg(th().ui.accent)
                        .bg(th().chrome.dropdown_bg)
                };
                frame.render_clipped(Paragraph::new(Span::styled(split_label, style)), srect);
                // Inner first: the row's own rect (pushed below) would
                // otherwise cover the button.
                hits.push((srect, part));
            }
            hits.push((rect, RebasePart::Row(i)));
        }

        // The open dropdown, over the rows below its button (or above it
        // when it would run off the bottom of the view). Its choices are
        // registered first so they win over the rows they cover.
        if let Some((row_y, i)) = open_list {
            let choices = self.choices();
            let list_h = choices.len() as u16 + 2;
            let list_w = label_w as u16 + 6;
            let list_y = if row_y + 1 + list_h <= area.y + area.height.saturating_sub(1) {
                row_y + 1
            } else {
                row_y.saturating_sub(list_h)
            };
            let list_rect = Rect::new(inner.x + 2, list_y, list_w, list_h);
            frame.render_clipped(Clear, list_rect);
            frame.render_clipped(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(th().ui.muted))
                    .style(Style::default().bg(th().ui.panel)),
                list_rect,
            );
            let mut choice_hits = Vec::new();
            for (k, action) in choices.iter().enumerate() {
                let crect = Rect::new(list_rect.x + 1, list_rect.y + 1 + k as u16, list_w - 2, 1);
                let part = RebasePart::Action(i, *action);
                let style = if hovered == Some(part) || k == self.dropdown_cursor {
                    Style::default()
                        .fg(th().chrome.on_bright)
                        .bg(th().chrome.dropdown_hit)
                } else {
                    Style::default().fg(th().ui.text)
                };
                frame.render_clipped(
                    Paragraph::new(Span::styled(
                        format!(
                            " {}{}",
                            Self::action_label(*action),
                            " ".repeat(
                                ((list_w - 3) as usize)
                                    .saturating_sub(display_width(Self::action_label(*action)))
                            )
                        ),
                        style,
                    )),
                    crect,
                );
                choice_hits.push((crect, part));
            }
            hits.splice(0..0, choice_hits);
        }

        // Footer: error, key hints, then autosquash on the left and the
        // buttons on the right.
        let footer_y = inner.y + inner.height - 1;
        // Left: "shortcuts:", then one mnemonic per action (its first
        // letter lit), then the row-move chords. The keys that reach the
        // buttons on the right are spelled on the line above.
        let checkbox = format!(
            "[{}] {}",
            if self.autosquash { "x" } else { " " },
            msg::AUTOSQUASH.get()
        );
        let checkbox_w = display_width(&checkbox) as u16;
        let hint_room = inner.width.saturating_sub(checkbox_w + 2);
        // Row A: the error if there is one, else the keys that reach the
        // list, autosquash, Start, Reset and Cancel.
        let km = crate::keymap::current();
        let shown = |a| match km.shown(a) {
            k if k.is_empty() => "-".to_string(),
            k => k,
        };
        let keys_line = msg::keys_line(
            shown(Bind::RebaseAutosquash),
            shown(Bind::RebaseStart),
            shown(Bind::RebaseReset),
        );
        let (row_a, row_a_style) = match &self.error {
            Some(error) => (error.as_str(), Style::default().fg(th().status.danger)),
            None => (keys_line.as_str(), Style::default().fg(th().ui.muted)),
        };
        frame.render_clipped(
            Paragraph::new(Span::styled(
                truncate_with_ellipsis(row_a, hint_room as usize),
                row_a_style,
            )),
            Rect::new(inner.x, footer_y - 2, hint_room, 1),
        );
        frame.render_clipped(
            Paragraph::new(Span::styled(
                msg::SHORTCUTS.get(),
                Style::default().fg(th().ui.muted),
            )),
            Rect::new(
                inner.x,
                footer_y - 1,
                inner.width.min(display_width(msg::SHORTCUTS.get()) as u16),
                1,
            ),
        );
        let mut spans: Vec<Span> = Vec::new();
        let lit = Style::default()
            .fg(th().ui.text)
            .add_modifier(Modifier::BOLD);
        let mut mnemonics = vec![
            (Bind::RebasePick, RebaseAction::Pick),
            (Bind::RebaseSquash, RebaseAction::Squash),
            (Bind::RebaseReword, RebaseAction::Reword),
            (Bind::RebaseDrop, RebaseAction::Drop),
        ];
        if !self.cherry_pick {
            mnemonics.extend([
                (Bind::RebaseFixup, RebaseAction::Fixup),
                (Bind::RebaseEdit, RebaseAction::Edit),
            ]);
        }
        for (action, what) in mnemonics {
            let name = Self::action_label(what);
            let key = km.shown(action);
            let first = name.chars().next().unwrap_or(' ');
            if key.eq_ignore_ascii_case(first.encode_utf8(&mut [0; 4])) {
                // The default look: the action's first letter, lit.
                spans.push(Span::styled(key.to_uppercase(), lit));
                spans.push(Span::styled(
                    &name[first.len_utf8()..],
                    Style::default().fg(th().ui.muted),
                ));
            } else {
                spans.push(Span::styled(
                    if key.is_empty() { "-".to_string() } else { key },
                    lit,
                ));
                spans.push(Span::styled(
                    format!(" {name}"),
                    Style::default().fg(th().ui.muted),
                ));
            }
            spans.push(Span::raw("  "));
        }
        for (action, what) in [
            (Bind::RebaseMoveUp, msg::MOVE_UP.get()),
            (Bind::RebaseMoveDown, msg::MOVE_DOWN.get()),
        ] {
            spans.push(Span::styled(
                shown(action)
                    .replace("Up", "\u{2191}")
                    .replace("Down", "\u{2193}"),
                lit,
            ));
            spans.push(Span::styled(
                format!(" {what}  "),
                Style::default().fg(th().ui.muted),
            ));
        }
        if self.can_split() {
            spans.push(Span::styled(shown(Bind::RebaseSplit), lit));
            spans.push(Span::styled(
                format!(" {}  ", msg::SPLIT_BUTTON.get()),
                Style::default().fg(th().ui.muted),
            ));
        }
        frame.render_clipped(
            Paragraph::new(Line::from(spans)),
            Rect::new(inner.x, footer_y, inner.width, 1),
        );

        let cb_rect = Rect::new(
            inner.x + inner.width.saturating_sub(checkbox_w),
            footer_y - 2,
            checkbox_w.min(inner.width),
            1,
        );
        frame.render_clipped(
            Paragraph::new(Span::styled(
                &checkbox,
                plain_button_style(hovered == Some(RebasePart::Autosquash)),
            )),
            cb_rect,
        );
        hits.push((cb_rect, RebasePart::Autosquash));

        let start_label = format!(
            " {} ",
            if self.cherry_pick {
                msg::START_CHERRY_PICK.get()
            } else {
                msg::START_REBASE.get()
            }
        );
        let bottom_cancel = cancel_label.to_string();
        let reset_label = format!(" {} ", msg::RESET.get());
        // Reset blue, Cancel red, Start green -- outlines
        // drawn as tinted fills on a one-row button.
        let buttons = [
            (
                RebasePart::Reset,
                reset_label,
                th().chrome.action_reset,
                th().chrome.tint_reset,
            ),
            (
                RebasePart::Cancel,
                bottom_cancel,
                th().status.danger,
                th().chrome.tint_danger,
            ),
            (
                RebasePart::Start,
                start_label,
                th().refs.local,
                th().chrome.tint_success,
            ),
        ];
        let total: u16 = buttons
            .iter()
            .map(|(_, l, _, _)| display_width(l) as u16 + 1)
            .sum::<u16>()
            - 1;
        // Only as many buttons as fit beside the mnemonics.
        let mut x = inner.x + inner.width.saturating_sub(total);
        for (part, label, fg, bg) in buttons {
            let bw = display_width(&label) as u16;
            let style = if hovered == Some(part) {
                plain_button_style(true)
            } else {
                Style::default().fg(fg).bg(bg)
            };
            let rect = Rect::new(x, footer_y - 1, bw, 1);
            frame.render_clipped(Paragraph::new(Span::styled(label, style)), rect);
            hits.push((rect, part));
            x += bw + 1;
        }

        if let Some(editing) = &self.editing {
            let sub_hits = editing.render(
                frame,
                frame_area,
                match hovered {
                    Some(RebasePart::Editing(part)) => Some(part),
                    _ => None,
                },
            );
            hits = sub_hits
                .into_iter()
                .map(|(rect, part)| (rect, RebasePart::Editing(part)))
                .collect();
        }
        hits
    }
}
