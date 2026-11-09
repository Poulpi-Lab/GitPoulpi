//! Drawing the Preferences screen: sidebar, body, rows and the undo-off confirmation.

use super::*;
use crate::th;
use gitpoulpi_i18n::msg::settings_screen as screen;

impl PreferencesState {
    // ---------------------------------------------------------------
    // Rendering
    // ---------------------------------------------------------------

    pub(crate) fn render(&mut self, frame: &mut Frame, area: Rect) {
        // The event loop clears the scrollbars' pointer while Settings is
        // open (it owns the mouse), so the bars here light from Settings' own
        // pointer, except under an open dropdown.
        let pointer = if self.choice.is_some() {
            None
        } else {
            self.mouse_pos
        };
        crate::scrollbar::with_pointer(pointer, || self.render_screen(frame, area));
    }

    fn render_screen(&mut self, frame: &mut Frame, area: Rect) {
        self.area = area;
        frame.render_widget(Clear, area);
        // The title line wears the main window's: the octopus (a picture
        // where the terminal draws one), then the app's name, here before the
        // screen's own title. Three columns are left for the octopus.
        let title = format!("   GitPoulpi - {}", screen::TITLE.get());
        let title = crate::ui_kit::fit(&title, usize::from(area.width.saturating_sub(9)));
        let mut block = Block::default()
            .title(Line::from(vec![
                Span::styled(title, Style::default().fg(th().ui.text)),
                Span::raw(" "),
            ]))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(th().ui.accent));
        if let Some(notice) = self.mode_hint.as_deref().or(self.notice.as_deref()) {
            block = block.title_bottom(Line::from(Span::styled(
                format!(" {notice} "),
                Style::default().fg(th().ui.accent),
            )));
        }
        let inner = block.inner(area);
        frame.render_widget(block, area);
        // The shared close cross; its rect is the hit target.
        let hovered = self.mouse_pos.is_some_and(|(col, row)| {
            Rect::new(area.right().saturating_sub(5), area.y, 3, 1).contains((col, row).into())
        });
        self.close_rect = crate::chrome::close_cross::paint(frame.buffer_mut(), area, hovered);
        self.logo_rect = None;
        if area.width >= 16 {
            let hovered = self.mouse_pos.is_some_and(|(col, row)| {
                Rect::new(area.x + 1, area.y, 3, 1).contains((col, row).into())
            });
            let mut hits = Vec::new();
            crate::chrome::render_about_button(frame, area.x + 1, area.y, hovered, &mut hits);
            self.logo_rect = hits.first().map(|(rect, _)| *rect);
            for (rect, _) in &hits {
                crate::pointer_map::hand(*rect);
            }
        }

        let [sidebar, body] =
            Layout::horizontal([Constraint::Length(28), Constraint::Min(20)]).areas(inner);
        self.sidebar_area = sidebar;
        self.body_area = body;

        self.render_sidebar(frame, sidebar);
        self.render_body(frame, body);
        self.render_ai_prompt(frame, inner);
        self.render_review_browse(frame, area);
        self.render_workflow_editor(frame, inner);
        self.render_repo_pages_editor(frame, inner);
        self.render_repo_auth_overlays(frame, inner);
        if let Some((_, combo)) = self.choice.as_mut() {
            combo.render_popup(frame.buffer_mut(), area);
        }
        self.render_search(frame);
        self.render_undo_off_confirm(frame, area);
        self.render_confirm_action(frame, area);
        self.render_close_ask(frame.buffer_mut(), area);
    }

    pub(super) fn render_sidebar(&mut self, frame: &mut Frame, area: Rect) {
        self.sidebar_rows.clear();
        let mut lines: Vec<Line<'static>> = Vec::new();
        // Row 0 is the search box (painted last), row 1 a spacer.
        let mut y = area.y + 2;
        lines.push(Line::from(""));
        lines.push(Line::from(""));
        fn push(lines: &mut Vec<Line<'static>>, line: Line<'static>) {
            lines.push(line);
        }

        let group = |text: &str| {
            Line::from(Span::styled(
                text.to_string(),
                Style::default()
                    .fg(th().ui.muted)
                    .add_modifier(Modifier::BOLD),
            ))
        };

        push(&mut lines, group(screen::CURRENT_PROFILE.get()));
        y += 1;
        let profile_name = self
            .config
            .active_profile()
            .map(|p| p.name.clone())
            .unwrap_or_else(|| screen::DEFAULT_PROFILE.get().to_string());
        push(
            &mut lines,
            Line::from(Span::styled(
                format!("  {profile_name}"),
                Style::default().fg(th().ui.text),
            )),
        );
        y += 1;
        push(&mut lines, Line::from(""));
        y += 1;

        push(&mut lines, group(screen::PREFERENCES.get()));
        y += 1;
        for (index, section) in PrefSection::ALL.iter().enumerate() {
            if !section.is_built() {
                continue;
            }
            if *section == PrefSection::GitAndEncoding {
                push(&mut lines, Line::from(""));
                y += 1;
                push(&mut lines, group(screen::REPO_SPECIFIC.get()));
                y += 1;
                push(
                    &mut lines,
                    Line::from(Span::styled(
                        format!("  {}", screen::repo_line(&self.repo_label)),
                        Style::default().fg(th().ui.text),
                    )),
                );
                y += 1;
            }
            let selected = index == self.section;
            let style = if selected && self.focus == PrefFocus::Sections {
                Style::default().fg(th().ui.text).bg(th().ui.selection)
            } else if selected {
                Style::default().fg(th().ui.text).bg(th().ui.hover)
            } else {
                Style::default().fg(th().ui.muted)
            };
            push(
                &mut lines,
                Line::from(Span::styled(
                    {
                        let title = crate::ui_kit::fit(section.title(), 24);
                        let pad = 24usize.saturating_sub(display_width(&title));
                        format!("  {title}{}", " ".repeat(pad))
                    },
                    style,
                )),
            );
            self.sidebar_rows.push((y, index));
            y += 1;
        }

        // Same reason as `render_body`'s own `Clear` below.
        frame.render_widget(Clear, area);
        frame.render_widget(
            Paragraph::new(lines).block(
                Block::default()
                    .borders(Borders::RIGHT)
                    .border_style(Style::default().fg(th().ui.muted)),
            ),
            area,
        );
        // Hover: a section row under the pointer lights like a menu row.
        if let Some((col, row)) = self.mouse_pos {
            if col >= area.x && col < area.right() {
                if let Some((y, _)) = self
                    .sidebar_rows
                    .iter()
                    .find(|(y, index)| *y == row && *index != self.section)
                {
                    frame.buffer_mut().set_style(
                        Rect::new(area.x, *y, area.width.saturating_sub(1), 1),
                        Style::default().bg(th().ui.hover).fg(th().ui.text),
                    );
                }
            }
        }
        self.render_search_box(frame, area);
    }

    pub(super) fn render_body(&mut self, frame: &mut Frame, area: Rect) {
        self.strip_rect = Rect::new(0, 0, 0, 0);
        let section = self.current_section();
        self.list_area = Rect::default();
        self.tab_rects.clear();
        self.body_head = 2;
        if section.has_list() {
            self.render_list_body(frame, area);
            return;
        }
        if section == PrefSection::Keybindings {
            self.render_keys_body(frame, area);
            return;
        }
        if !section.has_strip() {
            self.render_field_list(frame, area);
            return;
        }
        // A page that draws its own body: its generic rows (title included)
        // first, the page below them.
        let fields = self.fields();
        let inner_width = area.width.saturating_sub(2) as usize;
        self.set_label_col(&fields, inner_width);
        let lines: usize = fields
            .iter()
            .enumerate()
            .map(|(i, f)| self.field_lines(f, i, false, inner_width).len())
            .sum();
        let height = (lines as u16 + 3).min(area.height.saturating_sub(8));
        let strip = Rect::new(area.x, area.y, area.width, height);
        self.strip_rect = strip;
        self.render_field_list(frame, strip);
        let rest = Rect::new(
            area.x + 1,
            area.y + height,
            area.width.saturating_sub(2),
            area.height.saturating_sub(height),
        );
        self.field_rows.retain(|(y, _)| *y < area.y + height);
        let focused = self.focus == PrefFocus::Fields && !self.strip_active();
        self.integrations.render(
            frame,
            rest,
            &self.config,
            &self.token_presence,
            focused,
            self.mouse_pos,
        );
    }

    /// The Keyboard & mouse page: the title and the sub-tab strip, then the
    /// generic rows on the first tab, the keybindings editor on the second.
    fn render_keys_body(&mut self, frame: &mut Frame, area: Rect) {
        let area = self.render_tab_head(frame, area);
        self.body_head = 1;
        if self.sub_tab() == SubTab::General {
            self.render_field_list(frame, area);
            return;
        }
        // The editor paints its own body: nothing of the generic list is live.
        self.field_rows.clear();
        self.chip_rects.clear();
        self.btn_rects.clear();
        self.body_bar = None;
        self.body_lines = 0;
        // The title and the strip stay the tabs', not the editor's.
        self.strip_rect = Rect::new(area.x, area.y.saturating_sub(2), area.width, 2);
        let rest = Rect::new(
            area.x + 1,
            area.y,
            area.width.saturating_sub(2),
            area.height,
        );
        self.render_keys(frame, rest);
    }

    /// The generic list of rows: the page's own rows, or a custom page's strip.
    pub(super) fn render_field_list(&mut self, frame: &mut Frame, area: Rect) {
        self.field_rows.clear();
        self.fields_x = area.x;
        self.caret_edit.sync(self.edit.as_ref().map(|(i, _, _)| *i));
        self.caret_edit.begin_frame();
        self.edit_value_at.set(None);
        self.choice_value_at.set(None);
        let inner_width = area.width.saturating_sub(2) as usize;
        let head = self.body_head;
        self.body_height = area.height.saturating_sub(head) as usize;
        let fields = self.fields();
        self.set_label_col(&fields, inner_width);

        // Build every line first, remembering which field each belongs
        // to, then take the scrolled window. Building the window first
        // is how a help line ends up attributed to the field below it.
        let mut lines: Vec<Line<'static>> = Vec::new();
        let mut owners: Vec<Option<usize>> = Vec::new();
        let mut starts: Vec<usize> = Vec::new();
        self.chip_cells.borrow_mut().clear();
        self.btn_cells.borrow_mut().clear();
        self.ctrl_cells.borrow_mut().clear();
        for (index, field) in fields.iter().enumerate() {
            let selected = index == self.field && self.list_focused();
            starts.push(lines.len());
            for line in self.field_lines(field, index, selected, inner_width) {
                lines.push(line);
                owners.push(field.selectable().then_some(index));
            }
        }

        // Pinned rows (the Workflow page's save bar): split off the end of the
        // page and paint it under the scrolling window, always visible.
        let sticky = if self.current_section() == PrefSection::Workflow {
            Self::wf_sticky_rows().min(fields.len())
        } else {
            0
        };
        let split = if sticky > 0 {
            starts[fields.len() - sticky]
        } else {
            lines.len()
        };
        let mut tail: Vec<Line<'static>> = lines.split_off(split);
        let tail_owners: Vec<Option<usize>> = owners.split_off(split);
        let tail_h = tail.len().min(self.body_height.saturating_sub(1));
        tail.truncate(tail_h);
        self.body_height -= tail_h;

        // Keep the selected field in view -- only when the selection has
        // just moved: run every frame, it pulled a wheel-scrolled page
        // straight back to the selected field, so the rows past the first
        // screen (a long page's last ones) could never be reached.
        let selection = (self.section, self.field, self.focus);
        let moved = self.kept_in_view != Some(selection);
        self.kept_in_view = Some(selection);
        if let Some(first) = owners
            .iter()
            .position(|o| *o == Some(self.field))
            .filter(|_| moved)
        {
            let last = owners
                .iter()
                .rposition(|o| *o == Some(self.field))
                .unwrap_or(first);
            if first < self.scroll {
                self.scroll = first;
            } else if self.body_height > 0 && last >= self.scroll + self.body_height {
                self.scroll = last + 1 - self.body_height;
            }
        }
        // Never past the last full page: no wheel-scrolling to a blank screen.
        self.scroll = self
            .scroll
            .min(lines.len().saturating_sub(self.body_height));
        if self.current_section().has_strip() {
            self.scroll = 0;
        }

        // The title takes the body's first row and the padding the second: the
        // first line of the page is painted on `area.y + 2`.
        let top = area.y + head;
        for (offset, owner) in owners
            .iter()
            .skip(self.scroll)
            .take(self.body_height)
            .enumerate()
        {
            if let Some(index) = owner {
                self.field_rows.push((top + offset as u16, *index));
            }
        }
        // The edited value's text rect, on its field's first visible row
        // (the label/value row) -- where a click or a drag in it lands.
        if let (Some((index, _, _)), Some((dx, dy, w))) =
            (self.edit.as_ref(), self.edit_value_at.get())
        {
            let first = owners.iter().position(|o| o == &Some(*index));
            if let Some(line) =
                first.filter(|l| *l >= self.scroll && *l < self.scroll + self.body_height)
            {
                let y = top + (line - self.scroll) as u16 + dy;
                self.caret_edit
                    .drawn(*index, Rect::new(area.x + 1 + dx, y, w, 1), false);
            }
        }

        if let (Some((index, combo)), Some((dx, dy, w))) =
            (self.choice.as_mut(), self.choice_value_at.get())
        {
            let first = owners.iter().position(|o| o == &Some(*index));
            if let Some(line) =
                first.filter(|l| *l >= self.scroll && *l < self.scroll + self.body_height)
            {
                let y = top + (line - self.scroll) as u16 + dy;
                combo.set_anchor(Rect::new(area.x + 1 + dx, y, w, 1));
            }
        }

        // The chips' hit rects, from the same walk that painted them.
        self.chip_rects.clear();
        for (field, chip, line, dx, w) in self.chip_cells.borrow().iter().copied() {
            let at = starts.get(field).copied().unwrap_or(0) + line;
            if at >= self.scroll && at < self.scroll + self.body_height && at < split {
                let y = top + (at - self.scroll) as u16;
                self.chip_rects
                    .push((Rect::new(area.x + 1 + dx, y, w, 1), field, chip));
            }
        }

        // The buttons' hit rects, the same way.
        self.btn_rects.clear();
        for (field, button, line, dx, w) in self.btn_cells.borrow().iter().copied() {
            let at = starts.get(field).copied().unwrap_or(0) + line;
            if at >= self.scroll && at < self.scroll + self.body_height && at < split {
                let y = top + (at - self.scroll) as u16;
                self.btn_rects
                    .push((Rect::new(area.x + 1 + dx, y, w, 1), field, button));
            } else if at >= split && at - split < tail_h {
                let y = top + self.body_height as u16 + (at - split) as u16;
                self.btn_rects
                    .push((Rect::new(area.x + 1 + dx, y, w, 1), field, button));
            }
        }
        // The pointer: the hand on the controls and chips, the I-beam on the
        // input boxes (a number or a text box), the arrow on the rest of the
        // row (label, help), which only lights.
        for (rect, _, _) in &self.chip_rects {
            crate::pointer_map::hand(*rect);
        }
        for (rect, field, button) in &self.btn_rects {
            let number = button == &quantity::PART_NUMBER
                && matches!(
                    fields.get(*field).map(|f| &f.control),
                    Some(Control::Quantity { .. })
                );
            if number {
                crate::pointer_map::field(*rect);
            } else {
                crate::pointer_map::hand(*rect);
            }
        }
        for (field, input, line, dx, w) in self.ctrl_cells.borrow().iter().copied() {
            let at = starts.get(field).copied().unwrap_or(0) + line;
            if at >= self.scroll && at < self.scroll + self.body_height && at < split {
                let rect = Rect::new(
                    area.x + 1 + dx,
                    top + (at - self.scroll) as u16,
                    w.min(area.right().saturating_sub(area.x + 1 + dx)),
                    1,
                );
                if input {
                    crate::pointer_map::field(rect);
                } else {
                    crate::pointer_map::hand(rect);
                }
            }
        }
        for (offset, owner) in tail_owners.iter().take(tail_h).enumerate() {
            if let Some(index) = owner {
                self.field_rows
                    .push((top + self.body_height as u16 + offset as u16, *index));
            }
        }

        let total = lines.len();
        let visible: Vec<Line<'static>> = lines
            .into_iter()
            .skip(self.scroll)
            .take(self.body_height)
            .collect();

        // A tabbed page painted its title above the strip already.
        let title = if head >= 2 {
            format!(
                " {} ",
                if self.current_section().is_repo_scoped() {
                    screen::page_title_repo(self.current_section().title())
                } else {
                    screen::page_title_global(self.current_section().title())
                }
            )
        } else {
            String::new()
        };
        // `render`'s full-screen `Clear` covers whatever was drawn before
        // Preferences opened; this one, scoped to the body, covers a row
        // shrinking between two Preferences frames: `Paragraph` only
        // re-styles cells it has no text for and never resets their symbol.
        // Pinned by `preferences_render_leaves_no_stale_cells_from_a_previous_frame`.
        frame.render_widget(Clear, area);
        let mut block = Block::default()
            .borders(Borders::NONE)
            .padding(ratatui::widgets::Padding::new(1, 1, head.min(1), 0));
        // An empty title would still take the top row.
        if head >= 2 {
            block = block.title(title);
        }
        frame.render_widget(Paragraph::new(visible).block(block), area);
        if tail_h > 0 {
            frame.render_widget(
                Paragraph::new(tail),
                Rect::new(
                    area.x + 1,
                    top + self.body_height as u16,
                    area.width.saturating_sub(2),
                    tail_h as u16,
                ),
            );
        }
        // The scrollbar in the right padding column, beside the text rows.
        self.body_lines = total;
        self.body_bar = if self.current_section().has_strip() {
            None
        } else {
            crate::scrollbar::Track::vertical(
                Rect::new(area.x, top - 1, area.width, self.body_height as u16 + 2),
                total,
                self.scroll,
            )
        };
        if let Some(track) = self.body_bar {
            let lit = self
                .mouse_pos
                .is_some_and(|(c, r)| track.rect.contains((c, r).into()));
            crate::scrollbar::render(frame, track, true, lit);
        }
        self.paint_body_hover(frame, area);
    }

    /// Hover on the settings body: the field row under the pointer lights
    /// (every line of it, help included) unless it is the selected one, and
    /// a chip under the pointer fills like any plain button.
    fn paint_body_hover(&self, frame: &mut Frame, area: Rect) {
        let Some((col, row)) = self.mouse_pos else {
            return;
        };
        if col < area.x || col >= area.right() {
            return;
        }
        let buf = frame.buffer_mut();
        let selected = self.list_focused();
        if let Some((_, hovered)) = self.field_rows.iter().find(|(y, _)| *y == row) {
            if !(selected && *hovered == self.field) {
                for (y, index) in &self.field_rows {
                    if index == hovered {
                        buf.set_style(
                            Rect::new(area.x + 1, *y, area.width.saturating_sub(2), 1),
                            Style::default().bg(th().ui.hover),
                        );
                    }
                }
            }
        }
        for (rect, _, _) in &self.chip_rects {
            if rect.contains((col, row).into()) {
                buf.set_style(*rect, crate::theme::plain_button_style(true));
            }
        }
    }

    /// A `Chips` row: the chips as `[+ text]` buttons, wrapped to the page
    /// width. Records each chip's cell for the hit rects.
    pub(super) fn chip_lines(
        &self,
        index: usize,
        labels: &[String],
        selected: bool,
        width: usize,
    ) -> Vec<Line<'static>> {
        let mut lines: Vec<Line<'static>> = Vec::new();
        let mut row: Vec<Span<'static>> = vec![Span::raw("  ")];
        let mut used = 2usize;
        let mut line_no = 0usize;
        let focused = self.ai.chip_focus(labels.len());
        for (chip, label) in labels.iter().enumerate() {
            let text = format!("[+ {label}]");
            let w = display_width(&text);
            if used > 2 && used + w > width {
                lines.push(Line::from(std::mem::take(&mut row)));
                row = vec![Span::raw("  ")];
                used = 2;
                line_no += 1;
            }
            let style = if selected && chip == focused {
                Style::default().fg(th().ui.text).bg(th().ui.selection)
            } else {
                Style::default().fg(th().ui.accent)
            };
            self.chip_cells
                .borrow_mut()
                .push((index, chip, line_no, used as u16, w as u16));
            row.push(Span::styled(text, style));
            row.push(Span::raw(" "));
            used += w + 1;
        }
        lines.push(Line::from(row));
        lines
    }

    /// An `Area` row: the label, then the text in a gutter box (at most
    /// five lines), then the help.
    pub(super) fn area_lines(
        &self,
        field: &Field,
        text: &str,
        selected: bool,
        width: usize,
    ) -> Vec<Line<'static>> {
        const MAX: usize = 5;
        let label_style = if selected {
            Style::default().fg(th().ui.text).bg(th().ui.selection)
        } else {
            Style::default().fg(th().ui.text)
        };
        let mut lines = vec![Line::from(vec![
            Span::styled(
                crate::terminology::tr(&field.label).into_owned(),
                label_style,
            ),
            Span::styled(
                if selected {
                    format!("  {}", screen::ENTER_EDIT.get())
                } else {
                    String::new()
                },
                Style::default().fg(th().ui.muted),
            ),
        ])];
        let mut body: Vec<String> = Vec::new();
        for raw in text.lines() {
            body.extend(wrap_words(raw, width.saturating_sub(6)));
        }
        let empty = body.is_empty();
        if empty {
            body.push(match &field.placeholder {
                Some(p) => {
                    let first = p.lines().next().unwrap_or("");
                    if p.lines().count() > 1 {
                        format!("{first} ...")
                    } else {
                        first.to_string()
                    }
                }
                None => screen::EMPTY.get().to_string(),
            });
        }
        let more = body.len().saturating_sub(MAX);
        body.truncate(MAX);
        let (gutter, style) = if selected {
            (th().ui.accent, Style::default().fg(th().ui.text))
        } else {
            (th().ui.muted, Style::default().fg(th().ui.accent))
        };
        let style = if empty {
            Style::default().fg(th().ui.muted)
        } else {
            style
        };
        for line in body {
            lines.push(Line::from(vec![
                Span::styled("  \u{2502} ", Style::default().fg(gutter)),
                Span::styled(line, style),
            ]));
        }
        if more > 0 {
            lines.push(Line::from(Span::styled(
                format!("  \u{2502} {}", screen::more_lines(more as u64)),
                Style::default().fg(th().ui.muted),
            )));
        }
        if let Some(help) = &field.help {
            for line in wrap_words(&crate::terminology::tr(help), width.saturating_sub(2)) {
                lines.push(Line::from(Span::styled(
                    format!("  {line}"),
                    Style::default().fg(th().ui.muted),
                )));
            }
        }
        lines.push(Line::from(""));
        lines
    }

    fn render_undo_off_confirm(&mut self, frame: &mut Frame, area: Rect) {
        if let Some(confirm) = self.confirm_undo_off.as_mut() {
            confirm.render(frame.buffer_mut(), area);
        }
    }
}
