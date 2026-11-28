use super::*;
use gitpoulpi_i18n::msg::editor as msg;

impl Editor {
    /// Draws the editor over the central column `area`. Returns its
    /// clickable rects.
    pub(crate) fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        hovered: Option<EditorHit>,
    ) -> Vec<(Rect, EditorHit)> {
        let mut hits = Vec::new();
        if let Some(line) = self.doc.take_changed_from() {
            self.hl.invalidate_from(line);
        }
        if self.longest.0 != self.doc.version() {
            let tab = self.doc.tab_cols();
            let width = (0..self.doc.line_count())
                .map(|i| line_width_in(self.doc.line(i), tab))
                .max()
                .unwrap_or(0);
            self.longest = (self.doc.version(), width);
        }
        let [bar_area, body] =
            Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(area);
        self.body = body;
        self.bar = bar_area;

        // The body: title with the dirty marker, `✕ Esc` on the border.
        let dirty = if self.doc.is_dirty() {
            format!(" \u{25CF} {}", msg::MODIFIED.get())
        } else {
            String::new()
        };
        // Plain-glyph hover, like the gear/app-close buttons and the
        // Diff/File View's `\u{2715} Esc`.
        let close_hovered = hovered == Some(EditorHit::Close);
        let block = Block::default()
            .title(Line::from(vec![
                Span::raw(format!(" {}", msg::title(self.shown()))),
                Span::styled(
                    dirty,
                    Style::default()
                        .fg(th().merge.edit)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
            ]))
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(th().ui.border_focused));
        self.minimap_pad = minimap::padding_for(body, live_settings::minimap());
        self.minimap_rect = (self.minimap_pad > 0)
            .then(|| minimap::rect_in(body))
            .flatten();
        let block = block.padding(ratatui::widgets::Padding::right(self.minimap_pad));
        let inner = block.inner(body);
        frame.render_clipped(block, body);
        if let Some(rect) =
            crate::chrome::close_cross::paint(frame.buffer_mut(), body, close_hovered)
        {
            hits.push((rect, EditorHit::Close));
        }

        let count = self.doc.line_count();
        let numbers_on = crate::editor_prefs::line_numbers();
        let digits = count.to_string().len();
        let gutter = if numbers_on { (digits + 1) as u16 } else { 0 };
        self.text_area = Rect::new(
            inner.x + gutter.min(inner.width),
            inner.y,
            inner.width.saturating_sub(gutter),
            inner.height,
        );
        let height = inner.height as usize;
        let width = self.text_area.width as usize;
        // The editable text: the I-beam (a button or thumb on top answers first).
        if !self.prompt {
            crate::pointer_map::field(self.text_area);
        }

        // Keep the caret on screen after a key; keep the view in range
        // always.
        let (caret_line, caret_col) = self.doc.cursor_line_col();
        if self.follow && height > 0 {
            if caret_line < self.top {
                self.top = caret_line;
            } else if caret_line >= self.top + height {
                self.top = caret_line + 1 - height;
            }
            if caret_col < self.left {
                self.left = caret_col;
            } else if width > 0 && caret_col >= self.left + width {
                self.left = caret_col + 1 - width;
            }
            self.follow = false;
        }
        self.top = self.top.min(count.saturating_sub(height.max(1)));

        // Highlight what is about to be drawn (see `editor_highlight`).
        let doc = &self.doc;
        let caught_up = !crate::editor_prefs::syntax_highlighting()
            || self
                .hl
                .advance(|i| doc.line(i), count, self.top + height, HIGHLIGHT_BUDGET);

        self.hl_pending = !caught_up;

        let caret = self.doc.cursor();
        let selection = self.doc.selection();
        let find_query = self
            .find
            .as_ref()
            .map(|f| f.query.to_ascii_lowercase())
            .filter(|q| !q.is_empty());
        // The selected text, tinted wherever else it occurs -- the same
        // `scroll_marks::match_ranges` source as the ruler's ticks.
        self.occ_needle = self.selected_needle();
        let mut rows: Vec<Line> = Vec::with_capacity(height);
        for r in 0..height {
            let i = self.top + r;
            if i >= count {
                break;
            }
            let current = i == caret_line;
            let number_style = if current {
                Style::default()
                    .fg(th().ui.text)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(th().ui.muted)
            };
            let mut spans = Vec::new();
            if numbers_on {
                spans.push(Span::styled(format!("{:>digits$} ", i + 1), number_style));
            }
            spans.extend(self.row_spans(
                i,
                width,
                caret,
                selection,
                find_query.as_deref(),
                current,
            ));
            rows.push(Line::from(spans));
        }
        frame.render_clipped(Paragraph::new(rows), inner);

        if let Some(rect) = self.minimap_rect {
            let doc = &self.doc;
            self.minimap
                .draw(frame, rect, doc.version(), count, (self.top, height), |i| {
                    (std::borrow::Cow::Borrowed(doc.line(i)), None)
                });
            hits.push((rect, EditorHit::Minimap));
        }
        // Scrollbars on the border.
        if let Some(t) = self.v_track() {
            let lit =
                hovered == Some(EditorHit::VThumb) || matches!(self.drag, Some(Drag::VThumb(_)));
            scrollbar::render(frame, t, true, lit);
            let ticks = self.ruler_ticks();
            let cells = scroll_marks::project(t.inner_len(), count, ticks);
            scroll_marks::paint(frame, t, &cells);
            hits.push((
                Rect::new(t.rect.x, t.rect.y + t.thumb_start, 1, t.thumb_len.max(1)),
                EditorHit::VThumb,
            ));
        }
        if let Some(t) = self.h_track() {
            let lit =
                hovered == Some(EditorHit::HThumb) || matches!(self.drag, Some(Drag::HThumb(_)));
            scrollbar::render(frame, t, false, lit);
            hits.push((
                Rect::new(t.rect.x + t.thumb_start, t.rect.y, t.thumb_len.max(1), 1),
                EditorHit::HThumb,
            ));
        }

        self.render_bar(frame, bar_area, hovered, caught_up, &mut hits);
        if self.prompt {
            self.render_prompt(frame, body, hovered, &mut hits);
        }
        // Over everything else, the menu of a status chip.
        self.render_menu(frame, area);
        hits
    }

    /// One text row: the visible slice of line `i`, highlighted, with the
    /// selection, find hits, caret and current-line band on top.
    fn row_spans(
        &self,
        i: usize,
        width: usize,
        caret: usize,
        selection: Option<(usize, usize)>,
        find: Option<&str>,
        current: bool,
    ) -> Vec<Span<'static>> {
        let start = self.doc.line_start(i);
        let text = self.doc.line(i);
        let occ: Vec<(usize, usize)> = scroll_marks::match_ranges(&self.occ_needle, text)
            .into_iter()
            .map(|(a, b)| (start + a, start + b))
            .collect();
        let row_bg = current.then_some(th().editor.current_line);
        let base = |style: Style| match row_bg {
            Some(bg) if style.bg.is_none() => style.bg(bg),
            _ => style,
        };
        let hits: Vec<(usize, usize)> = find
            .map(|q| {
                let hay = text.to_ascii_lowercase();
                hay.match_indices(q)
                    .map(|(a, m)| (start + a, start + a + m.len()))
                    .collect()
            })
            .unwrap_or_default();
        let runs = if crate::editor_prefs::syntax_highlighting() {
            self.hl.line(i)
        } else {
            None
        };
        let mut run_idx = 0;
        let mut run_end = runs.and_then(|r| r.first()).map_or(usize::MAX, |r| r.len);

        let mut out: Vec<Span<'static>> = Vec::new();
        let mut push = |s: &str, style: Style| match out.last_mut() {
            Some(last) if last.style == style => last.content.to_mut().push_str(s),
            _ => out.push(Span::styled(s.to_string(), style)),
        };
        let right = self.left + width;
        let tab = self.doc.tab_cols();
        let mut col = 0usize;
        for (bi, c) in text.char_indices() {
            if col >= right {
                break;
            }
            let w = char_cells_in(c, col, tab);
            if col + w <= self.left {
                col += w;
                continue;
            }
            if let Some(runs) = runs {
                while bi >= run_end && run_idx + 1 < runs.len() {
                    run_idx += 1;
                    run_end += runs[run_idx].len;
                }
            }
            let mut style = runs
                .and_then(|r| r.get(run_idx))
                .map_or(Style::default().fg(th().editor.text), |r| {
                    crate::editor_highlight::style_of(r.style)
                });
            if style.fg.is_none() {
                style = style.fg(th().editor.text);
            }
            let abs = start + bi;
            if occ.iter().any(|(a, b)| (*a..*b).contains(&abs)) {
                style = style.bg(scroll_marks::occurrence_bg(style.bg.or(row_bg)));
            }
            if hits.iter().any(|(a, b)| (*a..*b).contains(&abs)) {
                style = style.bg(th().editor.find_hit).fg(th().editor.find_hit_fg);
            }
            if selection.is_some_and(|(a, b)| (a..b).contains(&abs)) {
                style = style
                    .bg(th().ui.text_selection_bg)
                    .fg(th().ui.text_selection_fg);
            }
            if abs == caret {
                style = style.add_modifier(crate::text_cursor::caret_modifier());
            }
            let style = base(style);
            let visible = (col + w).min(right) - col.max(self.left);
            if c == '\t' || visible < w {
                push(&" ".repeat(visible), style);
            } else if c.is_control() {
                push("\u{00B7}", style.fg(th().ui.muted));
            } else {
                push(c.encode_utf8(&mut [0; 4]), style);
            }
            col += w;
        }
        // Past the text: the caret at the line's end, or the selected
        // line break, as one cell.
        let end = start + text.len();
        let mut used = col.saturating_sub(self.left).min(width);
        if col >= self.left && col < right {
            let newline_selected = i + 1 < self.doc.line_count()
                && selection.is_some_and(|(a, b)| a <= end && end < b);
            if caret == end {
                push(
                    " ",
                    base(
                        Style::default()
                            .fg(th().editor.text)
                            .add_modifier(crate::text_cursor::caret_modifier()),
                    ),
                );
                used += 1;
            } else if newline_selected {
                push(" ", Style::default().bg(th().ui.text_selection_bg));
                used += 1;
            }
        }
        if let Some(bg) = row_bg {
            push(
                &" ".repeat(width.saturating_sub(used)),
                Style::default().bg(bg),
            );
        }
        out
    }

    fn render_bar(
        &mut self,
        frame: &mut Frame,
        bar: Rect,
        hovered: Option<EditorHit>,
        caught_up: bool,
        hits: &mut Vec<(Rect, EditorHit)>,
    ) {
        let button = |label: &str, hit: EditorHit, enabled: bool| {
            let style = if !enabled {
                Style::default()
                    .fg(th().ui.muted)
                    .add_modifier(Modifier::DIM)
            } else if hit == EditorHit::ToggleMinimap && live_settings::minimap() {
                // Engaged, like the Diff view's toggles.
                Style::default()
                    .bg(th().ui.accent)
                    .fg(th().ui.on_accent)
                    .add_modifier(Modifier::BOLD)
            } else if hovered == Some(hit) {
                Style::default()
                    .bg(th().ui.button_hover)
                    .fg(th().ui.on_button)
            } else {
                Style::default().bg(th().ui.button).fg(th().ui.on_button)
            };
            Span::styled(format!(" {label} "), style)
        };
        let mut spans: Vec<Span> = Vec::new();
        let mut x = bar.x;
        let mut place = |spans: &mut Vec<Span<'static>>,
                         span: Span<'static>,
                         hit: Option<EditorHit>,
                         hits: &mut Vec<(Rect, EditorHit)>| {
            let w = span.width() as u16;
            if x + w > bar.right() {
                return false;
            }
            if let Some(hit) = hit {
                hits.push((Rect::new(x, bar.y, w, 1), hit));
            }
            spans.push(span);
            x += w;
            true
        };
        let gap = || Span::raw(" ");
        if let Some(find) = self.find.as_mut() {
            place(
                &mut spans,
                Span::styled(
                    format!(" {} ", msg::FIND.get()),
                    Style::default()
                        .fg(th().ui.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                None,
                hits,
            );
            let field = crate::text_field::edit_line(
                &find.query,
                Some(&find.caret),
                FIND_FIELD_WIDTH as usize,
                true,
                EditLook::new(
                    Style::default().bg(th().ui.panel).fg(th().ui.text),
                    "\u{2588}",
                ),
            );
            for s in field.spans {
                place(&mut spans, s, None, hits);
            }
            place(&mut spans, gap(), None, hits);
            place(
                &mut spans,
                button("\u{2191}", EditorHit::FindPrev, true),
                Some(EditorHit::FindPrev),
                hits,
            );
            place(&mut spans, gap(), None, hits);
            place(
                &mut spans,
                button("\u{2193}", EditorHit::FindNext, true),
                Some(EditorHit::FindNext),
                hits,
            );
            // Match count, cached per query and document version.
            let query = find.query.to_ascii_lowercase();
            let version = self.doc.version();
            let fresh = find
                .cache
                .as_ref()
                .is_some_and(|(q, v, _)| *q == query && *v == version);
            if !fresh {
                let starts = if query.is_empty() {
                    Vec::new()
                } else {
                    self.doc
                        .text()
                        .to_ascii_lowercase()
                        .match_indices(&query)
                        .map(|(i, _)| i)
                        .collect()
                };
                find.cache = Some((query.clone(), version, starts));
            }
            let starts = &find.cache.as_ref().expect("filled above").2;
            let label = if query.is_empty() {
                format!("  {}", msg::FIND_HELP.get())
            } else if starts.is_empty() {
                format!("  {}", msg::FIND_NONE.get())
            } else {
                let at = self.doc.selection().map(|(a, _)| a);
                match at.and_then(|a| starts.iter().position(|s| *s == a)) {
                    Some(i) => format!("  {}", msg::find_at(i + 1, starts.len())),
                    None => format!("  {}", msg::find_matches(starts.len() as u64)),
                }
            };
            place(
                &mut spans,
                Span::styled(label, Style::default().fg(th().ui.muted)),
                None,
                hits,
            );
        } else {
            for (label, hit) in BAR_BUTTONS {
                let enabled = match hit {
                    EditorHit::Save => self.doc.is_dirty(),
                    EditorHit::Undo => self.doc.can_undo(),
                    EditorHit::Redo => self.doc.can_redo(),
                    _ => true,
                };
                let label = match hit {
                    EditorHit::ToggleMinimap => format!("\u{25A7} {}", label.get()),
                    _ => label.get().to_string(),
                };
                if !place(&mut spans, button(&label, *hit, enabled), Some(*hit), hits) {
                    break;
                }
                place(&mut spans, gap(), None, hits);
            }
            // `[x] Follow selection`, drawn like the app's other checkboxes.
            let mark = if crate::editor_prefs::follow_selection() {
                "[x]"
            } else {
                "[ ]"
            };
            let check_style = if hovered == Some(EditorHit::FollowSelection) {
                Style::default().fg(th().ui.text).bg(th().ui.hover)
            } else {
                Style::default().fg(th().ui.text)
            };
            place(
                &mut spans,
                Span::styled(
                    format!("{mark} {}", msg::FOLLOW_SELECTION.get()),
                    check_style,
                ),
                Some(EditorHit::FollowSelection),
                hits,
            );
        }
        // The status on the right: the chips (encoding, line breaks,
        // indentation) are clickable; what does not fit is dropped, the
        // least useful first.
        let (line, col) = self.doc.cursor_line_col();
        let eol = self.eol_chip_text();
        let indent = self.doc.indent_label();
        let syntax = if crate::editor_prefs::syntax_highlighting() {
            self.hl.name()
        } else {
            msg::PLAIN_TEXT.get()
        };
        let highlighting = if caught_up {
            None
        } else {
            Some(msg::HIGHLIGHTING.get().to_string())
        };
        // (text, chip, kept at width): index = drop priority, last dropped first.
        let mut segments: Vec<(String, Option<EditorHit>)> = vec![
            (highlighting.unwrap_or_default(), None),
            (syntax.to_string(), None),
            (msg::position(line + 1, col + 1), None),
            (indent, Some(EditorHit::IndentChip)),
            (eol, Some(EditorHit::EolChip)),
            (
                self.doc.charset().name().to_string(),
                Some(EditorHit::EncodingChip),
            ),
        ];
        segments.retain(|(t, _)| !t.is_empty());
        let used = x - bar.x;
        let room = (bar.width.saturating_sub(used)) as usize;
        let width = |segments: &[(String, Option<EditorHit>)]| -> usize {
            segments
                .iter()
                .map(|(t, _)| display_width(t) + 2)
                .sum::<usize>()
                + 1
        };
        while width(&segments) >= room && !segments.is_empty() {
            segments.remove(0);
        }
        self.chips = [None; 3];
        if !segments.is_empty() {
            let total = width(&segments);
            spans.push(Span::raw(" ".repeat(room - total)));
            let mut sx = bar.right() - total as u16;
            // Display order: highlighting note, position, then the chips,
            // then the syntax - the syntax stays last as before.
            let order = |t: &(String, Option<EditorHit>)| match t.1 {
                Some(EditorHit::EncodingChip) => 3,
                Some(EditorHit::EolChip) => 4,
                Some(_) => 5,
                None if t.0 == syntax => 6,
                None => 0,
            };
            segments.sort_by_key(|t| order(t));
            for (text, hit) in segments {
                let w = (display_width(&text) + 2) as u16;
                let label = format!(" {text} ");
                match hit {
                    Some(hit) => {
                        let lit = hovered == Some(hit)
                            || self
                                .menu
                                .as_ref()
                                .is_some_and(|m| m.chip_hit() == Some(hit));
                        let style = if lit {
                            Style::default()
                                .bg(th().ui.button_hover)
                                .fg(th().ui.on_button)
                        } else {
                            Style::default().fg(th().ui.text)
                        };
                        let rect = Rect::new(sx, bar.y, w, 1);
                        hits.push((rect, hit));
                        if let Some(slot) = status_menu::CHIPS.iter().position(|(_, h)| *h == hit) {
                            self.chips[slot] = Some(rect);
                        }
                        spans.push(Span::styled(label, style));
                    }
                    None => spans.push(Span::styled(label, Style::default().fg(th().ui.muted))),
                }
                sx += w;
            }
            spans.push(Span::raw(" "));
        }
        frame.render_clipped(Paragraph::new(Line::from(spans)), bar);
    }

    fn render_prompt(
        &self,
        frame: &mut Frame,
        body: Rect,
        hovered: Option<EditorHit>,
        hits: &mut Vec<(Rect, EditorHit)>,
    ) {
        let question = self.prompt_question();
        // The effective keys, capitalized.
        let km = crate::keymap::current();
        let shown = |a| match km.shown(a) {
            k if k.is_empty() => "-".to_string(),
            k => k.to_uppercase(),
        };
        let buttons = [
            (
                msg::SAVE.get(),
                shown(Action::EditorPromptSave),
                EditorHit::PromptSave,
            ),
            (
                msg::DISCARD.get(),
                shown(Action::EditorPromptDiscard),
                EditorHit::PromptDiscard,
            ),
            (
                gitpoulpi_i18n::msg::dialog::CANCEL.get(),
                "Esc".to_string(),
                EditorHit::PromptCancel,
            ),
        ];
        let buttons_w: u16 = buttons
            .iter()
            .map(|(l, k, _)| display_width(&format!(" {l} ({k}) ")) as u16 + 2)
            .sum();
        let w = (display_width(&question) as u16 + 4)
            .max(buttons_w + 2)
            .min(body.width);
        let h = 5u16.min(body.height);
        if w < 10 || h < 5 {
            return;
        }
        let area = Rect::new(
            body.x + (body.width - w) / 2,
            body.y + (body.height - h) / 2,
            w,
            h,
        );
        frame.render_clipped(Clear, area);
        let block = Block::default()
            .title(format!(" {} ", msg::PROMPT_TITLE.get()))
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(th().merge.edit))
            .style(Style::default().bg(th().ui.panel));
        let inner = block.inner(area);
        frame.render_clipped(block, area);
        frame.render_clipped(
            Paragraph::new(Span::styled(question, Style::default().fg(th().ui.text))),
            Rect::new(inner.x + 1, inner.y, inner.width.saturating_sub(1), 1),
        );
        let mut x = inner.x + 1;
        let y = inner.y + 2;
        let mut spans = Vec::new();
        for (label, key, hit) in buttons {
            let text = format!(" {label} ({key}) ");
            let bw = display_width(&text) as u16;
            let style = if hovered == Some(hit) {
                Style::default()
                    .bg(th().ui.button_hover)
                    .fg(th().ui.on_button)
            } else if hit == EditorHit::PromptSave {
                Style::default()
                    .bg(th().ui.accent)
                    .fg(th().ui.on_accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().bg(th().ui.button).fg(th().ui.on_button)
            };
            hits.push((Rect::new(x, y, bw, 1), hit));
            spans.push(Span::styled(text, style));
            spans.push(Span::raw("  "));
            x += bw + 2;
        }
        frame.render_clipped(
            Paragraph::new(Line::from(spans)),
            Rect::new(inner.x + 1, y, inner.width.saturating_sub(1), 1),
        );
    }
}
