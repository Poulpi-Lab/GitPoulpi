use super::*;
use crate::keymap::chord::Key;
use crate::th;

/// The filter field's widest and narrowest drawn width, in cells.
const FILTER_W: u16 = 26;
const FILTER_MIN_W: u16 = 12;

impl FolderPicker {
    // -- drawing ------------------------------------------------------------------

    /// Draws the whole dialog into `rect` (border included) and records
    /// the hit rects.
    pub(crate) fn render(&mut self, rect: Rect, buf: &mut Buffer) {
        let title = self.title.clone();
        self.close.framed(buf, rect, &title, &WIN, true);
        let mut hits: Vec<(Rect, PickerHit)> = Vec::new();
        let inner = Rect::new(
            rect.x + 1,
            rect.y + 1,
            rect.width.saturating_sub(2),
            rect.height.saturating_sub(2),
        );
        if inner.width < 20 || inner.height < 6 {
            self.left_bar.hide();
            self.content_bar.hide();
            self.drop_bar.hide();
            self.hits = hits;
            return;
        }
        let x0 = inner.x + 1;
        let w = inner.width.saturating_sub(2);
        let bottom = inner.y + inner.height;

        // Row 0: the up button and the path field.
        let up = dialog_kit::button(
            buf,
            x0,
            inner.y,
            "\u{2191}",
            ButtonState::Normal {
                hovered: self.hover == Some(PickerHit::UpButton),
                focused: false,
            },
        );
        let field = Rect::new(up.right() + 1, inner.y, w.saturating_sub(up.width + 1), 1);
        self.sync_edit();
        self.edit.begin_frame();
        dialog_kit::text_input(
            buf,
            field,
            &self.path_draft,
            msg::folder_picker::PATH_HINT.get(),
            self.zone == Zone::Path,
            self.hover == Some(PickerHit::PathField),
            (&self.edit, PickerField::Path),
        );

        // Row 1: "New folder" and "Show hidden folders"; the inline name
        // field takes the row over while it is open.
        let toolbar_y = inner.y + 1;
        if let Some(draft) = self.new_folder_draft.clone() {
            let field = Rect::new(x0, toolbar_y, w, 1);
            dialog_kit::text_input(
                buf,
                field,
                &draft,
                msg::folder_picker::NEW_FOLDER_HINT.get(),
                true,
                self.hover == Some(PickerHit::NewFolderField),
                (&self.edit, PickerField::NewFolder),
            );
            hits.push((field, PickerHit::NewFolderField));
        } else {
            let new_folder = dialog_kit::button(
                buf,
                x0,
                toolbar_y,
                msg::folder_picker::NEW_FOLDER_BUTTON.get(),
                ButtonState::Normal {
                    hovered: self.hover == Some(PickerHit::NewFolder),
                    focused: false,
                },
            );
            hits.push((new_folder, PickerHit::NewFolder));
            let checkbox_x = new_folder.right() + 2;
            // The filter field takes what the checkbox leaves of the row,
            // up to a comfortable width; too little room and it is neither
            // drawn nor registered.
            let checkbox_w = display_width(msg::folder_picker::SHOW_HIDDEN.get()) as u16 + 4;
            let siblings_w = display_width(msg::folder_picker::SHOW_SIBLINGS.get()) as u16 + 4;
            let siblings_x = checkbox_x + checkbox_w + 2;
            let spare = (x0 + w).saturating_sub(siblings_x + siblings_w + 2);
            let filter_w = spare.min(FILTER_W);
            let right = if filter_w >= FILTER_MIN_W {
                (x0 + w).saturating_sub(filter_w + 1)
            } else {
                x0 + w
            };
            let checkbox_room = right.saturating_sub(checkbox_x).min(checkbox_w);
            if filter_w >= FILTER_MIN_W {
                let field = Rect::new(x0 + w - filter_w, toolbar_y, filter_w, 1);
                self.sync_edit();
                dialog_kit::text_input(
                    buf,
                    field,
                    &self.filter,
                    msg::folder_picker::FILTER_HINT.get(),
                    self.zone == Zone::Filter,
                    self.hover == Some(PickerHit::FilterField),
                    (&self.edit, PickerField::Filter),
                );
                hits.push((field, PickerHit::FilterField));
            }
            if checkbox_x < right {
                let checkbox = dialog_kit::checkbox(
                    buf,
                    checkbox_x,
                    toolbar_y,
                    checkbox_room,
                    msg::folder_picker::SHOW_HIDDEN.get(),
                    self.show_hidden,
                    CheckState::Enabled {
                        hovered: self.hover == Some(PickerHit::ShowHidden),
                        focused: false,
                    },
                );
                hits.push((checkbox, PickerHit::ShowHidden));
            }
            if siblings_x < right {
                let checkbox = dialog_kit::checkbox(
                    buf,
                    siblings_x,
                    toolbar_y,
                    right.saturating_sub(siblings_x),
                    msg::folder_picker::SHOW_SIBLINGS.get(),
                    self.show_siblings,
                    CheckState::Enabled {
                        hovered: self.hover == Some(PickerHit::ShowSiblings),
                        focused: false,
                    },
                );
                hits.push((checkbox, PickerHit::ShowSiblings));
            }
        }

        // The panes, below the toolbar row and above the two footer rows.
        let panes_top = inner.y + 2;
        let panes_h = bottom.saturating_sub(2).saturating_sub(panes_top);
        self.panes_w = w;
        let left_w = self
            .want_left_w()
            .map(|want| self.clamp_left_w(want))
            .unwrap_or_else(|| self.default_left_w());
        let left = Rect::new(x0, panes_top, left_w, panes_h);
        let content = Rect::new(
            x0 + left_w + 2,
            panes_top,
            w.saturating_sub(left_w + 2),
            panes_h,
        );
        let splitter_x = x0 + left_w;
        let splitter_hot = self.hover == Some(PickerHit::Splitter) || self.dragging_splitter;
        for y in panes_top..panes_top + panes_h {
            dialog_kit::put(
                buf,
                splitter_x,
                y,
                "\u{2502}",
                1,
                Style::default()
                    .fg(if splitter_hot {
                        th().ui.text
                    } else {
                        th().ui.hover
                    })
                    .bg(th().ui.panel),
            );
        }
        // Lit like every other resize line (the shared `column_resize` paint).
        let grab = Rect::new(splitter_x, panes_top, 1, panes_h);
        if splitter_hot {
            crate::column_resize::paint_grab(buf, gitpoulpi_config::ColumnResize::Line, grab);
        }
        hits.push((grab, PickerHit::Splitter));
        // The cell column beside each list: a click there focuses the list
        // (the `Ctrl+N` badge of the list presses it).
        hits.push((
            Rect::new(x0.saturating_sub(1), panes_top, 1, panes_h),
            PickerHit::LeftList,
        ));
        hits.push((
            Rect::new(content.x.saturating_sub(1), panes_top, 1, panes_h),
            PickerHit::ContentList,
        ));
        self.left_area = left;
        self.content_area = content;
        let mut folds = Vec::new();
        self.render_left(left, buf, &mut hits, &mut folds);
        self.render_content(content, buf, &mut hits);

        // Status row: what choosing picks, or the last message.
        let status_y = bottom.saturating_sub(2);
        if let Some(message) = &self.message {
            dialog_kit::put(
                buf,
                x0,
                status_y,
                message,
                w,
                Style::default().fg(th().status.warning).bg(th().ui.panel),
            );
        } else {
            let chosen = self.chosen();
            let repo_tag_indented = format!("  {}", msg::folder_picker::REPO_TAG.get());
            let tag = if self.chosen_is_repo() {
                repo_tag_indented.as_str()
            } else {
                ""
            };
            let lead = msg::folder_picker::SELECTED.get();
            let room = w.saturating_sub((display_width(lead) + display_width(tag)) as u16);
            let mut x = x0;
            x += dialog_kit::put(
                buf,
                x,
                status_y,
                lead,
                w,
                Style::default().fg(th().ui.muted).bg(th().ui.panel),
            )
            .width;
            x += dialog_kit::put(
                buf,
                x,
                status_y,
                &crate::tail_fit(&chosen.display().to_string(), room as usize),
                room,
                Style::default().fg(th().ui.text).bg(th().ui.panel),
            )
            .width;
            dialog_kit::put(
                buf,
                x,
                status_y,
                tag,
                display_width(tag) as u16,
                Style::default().fg(th().refs.local).bg(th().ui.panel),
            );
        }

        // Footer: hints on the left, the two buttons on the right.
        let footer_y = bottom.saturating_sub(1);
        // A narrow picker clips the labels, never drops a button.
        use dialog_kit::button_row::Btn;
        let [cancel, choose] = dialog_kit::button_row::paint(
            buf,
            x0 + w,
            footer_y,
            w,
            0,
            &[
                Btn {
                    label: msg::dialog::CANCEL.get(),
                    state: ButtonState::Normal {
                        hovered: self.hover == Some(PickerHit::Cancel),
                        focused: false,
                    },
                },
                Btn {
                    label: &self.confirm,
                    state: ButtonState::Primary {
                        hovered: self.hover == Some(PickerHit::Choose),
                        focused: false,
                    },
                },
            ],
        )[..] else {
            unreachable!("two buttons in, two rects out")
        };
        let cancel_x = cancel.x;
        let tab = dialog_kit::key_name(Key::Tab);
        let enter = dialog_kit::key_name(Key::Enter);
        let up_keys = format!(
            "{}/{}",
            dialog_kit::key_name(Key::Backspace),
            dialog_kit::alt_key(Key::Up)
        );
        let resize_keys = dialog_kit::alt_arrows();
        let choose_key = dialog_kit::key_label(Bind::FolderPickerChoose);
        let new_key = dialog_kit::key_label(Bind::FolderPickerNewFolder);
        let hidden_key = dialog_kit::key_label(Bind::FolderPickerHidden);
        let siblings_key = dialog_kit::key_label(Bind::FolderPickerSiblings);
        let filter_key = dialog_kit::key_label(Bind::FolderPickerFilter);
        dialog_kit::hints(
            buf,
            x0,
            footer_y,
            cancel_x.saturating_sub(x0 + 1),
            &[
                (tab.as_str(), msg::folder_picker::HINT_ZONES.get()),
                (enter.as_str(), msg::dialog::HINT_OPEN.get()),
                (filter_key.as_str(), msg::folder_picker::HINT_FILTER.get()),
                (up_keys.as_str(), msg::folder_picker::HINT_UP.get()),
                (choose_key.as_str(), msg::folder_picker::HINT_SELECT.get()),
                (new_key.as_str(), msg::folder_picker::HINT_NEW_FOLDER.get()),
                (hidden_key.as_str(), msg::folder_picker::HINT_HIDDEN.get()),
                (
                    siblings_key.as_str(),
                    msg::folder_picker::HINT_SIBLINGS.get(),
                ),
                (resize_keys.as_str(), msg::folder_picker::HINT_RESIZE.get()),
            ],
        );
        hits.push((cancel, PickerHit::Cancel));
        hits.push((choose, PickerHit::Choose));
        hits.push((up, PickerHit::UpButton));
        hits.push((field, PickerHit::PathField));

        // The completion dropdown, over the panes, registered first.
        let found = self.completions();
        let mut dropdown: Vec<(Rect, PickerHit)> = Vec::new();
        self.drop_area = Rect::default();
        let repo_tag_trailing = format!("{} ", msg::folder_picker::REPO_TAG.get());
        if self.zone == Zone::Path && !found.is_empty() {
            let shown = found.len().min(DROPDOWN_ROWS);
            // Follows the selection when it (or the list) changed; a wheel
            // or a bar drag keeps the window where it was put.
            let key = (self.completion, found.len());
            if key != self.drop_key {
                self.drop_key = key;
                self.drop_top = match self.completion {
                    Some(c) if c < self.drop_top => c,
                    Some(c) if c >= self.drop_top + shown => (c + 1).saturating_sub(shown),
                    Some(_) => self.drop_top,
                    None => 0,
                };
            }
            let start = self.drop_top.min(found.len() - shown);
            self.drop_top = start;
            let rows_h = shown.min(bottom.saturating_sub(inner.y + 1) as usize) as u16;
            let drop_area = Rect::new(field.x, inner.y + 1, field.width, rows_h);
            self.drop_area = drop_area;
            let drop_rows = self.drop_bar.paint(buf, drop_area, found.len(), start);
            for (offset, entry) in found.iter().skip(start).take(shown).enumerate() {
                let index = start + offset;
                let y = inner.y + 1 + offset as u16;
                if y >= bottom {
                    break;
                }
                let row = Rect::new(field.x, y, drop_rows.width, 1);
                let selected = self.completion == Some(index);
                let hovered = self.hover == Some(PickerHit::Completion(index));
                let bg = if selected {
                    th().ui.selection
                } else if hovered {
                    crate::lighten(th().ui.hover, 14)
                } else {
                    th().ui.hover
                };
                dialog_kit::fill(buf, row, Style::default().bg(bg));
                let tag = if entry.is_repo {
                    repo_tag_trailing.as_str()
                } else {
                    ""
                };
                let room = row.width.saturating_sub(display_width(tag) as u16 + 1);
                dialog_kit::put(
                    buf,
                    row.x + 1,
                    y,
                    &entry.name,
                    room,
                    Style::default().fg(th().ui.text).bg(bg),
                );
                dialog_kit::put(
                    buf,
                    row.right().saturating_sub(display_width(tag) as u16),
                    y,
                    tag,
                    display_width(tag) as u16,
                    Style::default().fg(th().refs.local).bg(bg),
                );
                dropdown.push((row, PickerHit::Completion(index)));
            }
        } else {
            self.drop_bar.hide();
        }
        dropdown.extend(hits);
        crate::pointer_map::from_hits(&dropdown, |h| match h {
            PickerHit::PathField | PickerHit::NewFolderField | PickerHit::FilterField => {
                Some(crate::pointer_map::Kind::Field)
            }
            PickerHit::Splitter => Some(crate::pointer_map::Kind::ColResize),
            PickerHit::LeftList | PickerHit::ContentList => None,
            // The places, folders and completions rows light and keep the arrow.
            PickerHit::Completion(_) | PickerHit::Left(_) | PickerHit::Content(_) => {
                Some(crate::pointer_map::Kind::Plain)
            }
            _ => Some(crate::pointer_map::Kind::Hand),
        });
        // Fold marks are controls inside their rows: registered after the
        // rows, whose `Plain` would hide them.
        for fold in folds {
            crate::pointer_map::hand(fold);
        }
        self.hits = dropdown;
        self.follow = false;
    }

    fn render_left(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        hits: &mut Vec<(Rect, PickerHit)>,
        folds: &mut Vec<Rect>,
    ) {
        let rows = self.left_rows();
        let height = area.height as usize;
        if self.follow {
            if let Some(sel) = self.left_sel {
                if sel < self.left_scroll {
                    self.left_scroll = sel;
                } else if sel >= self.left_scroll + height {
                    self.left_scroll = sel + 1 - height;
                }
            }
        }
        self.left_scroll = self.left_scroll.min(rows.len().saturating_sub(height));
        let area = self.left_bar.paint(buf, area, rows.len(), self.left_scroll);
        for (offset, row) in rows.iter().skip(self.left_scroll).take(height).enumerate() {
            let index = self.left_scroll + offset;
            let y = area.y + offset as u16;
            let band = Rect::new(area.x, y, area.width, 1);
            let selected = self.left_sel == Some(index) && self.zone == Zone::Left;
            let hovered = self.hover == Some(PickerHit::Left(index));
            let bg = if selected {
                th().ui.selection
            } else if hovered {
                th().ui.hover
            } else {
                th().ui.panel
            };
            match row {
                LeftRow::Heading(text) => {
                    dialog_kit::put(
                        buf,
                        area.x,
                        y,
                        text.get(),
                        area.width,
                        Style::default()
                            .fg(th().ui.muted)
                            .bg(th().ui.panel)
                            .add_modifier(Modifier::BOLD),
                    );
                    continue;
                }
                LeftRow::Place(i) => {
                    let place = &self.places[*i];
                    dialog_kit::fill(buf, band, Style::default().bg(bg));
                    let fg = if place.kind == PlaceKind::Drive {
                        th().ui.muted
                    } else {
                        th().ui.text
                    };
                    dialog_kit::put(
                        buf,
                        area.x + 1,
                        y,
                        &place.label,
                        area.width.saturating_sub(1),
                        Style::default().fg(fg).bg(bg),
                    );
                }
                LeftRow::Tree {
                    name,
                    depth,
                    current,
                    is_repo,
                    is_hidden,
                    open,
                    ..
                } => {
                    dialog_kit::fill(buf, band, Style::default().bg(bg));
                    let indent = (*depth as u16).min(area.width / 2);
                    let mut style = Style::default().fg(if *is_hidden {
                        th().ui.disabled
                    } else if *is_repo {
                        th().refs.local
                    } else {
                        th().ui.text
                    });
                    if *current {
                        style = style.add_modifier(Modifier::BOLD);
                    }
                    // Siblings shown: a real hierarchy, every folder carries the
                    // shared fold mark (open on the path). Else only the current one.
                    let marker = if self.show_siblings {
                        Some(crate::fold_mark::glyph_open(*open))
                    } else {
                        current.then_some(crate::fold_mark::EXPANDED)
                    };
                    dialog_kit::put(
                        buf,
                        area.x + indent,
                        y,
                        &format!("{} {name}", marker.unwrap_or(" ")),
                        area.width.saturating_sub(indent),
                        style.bg(bg),
                    );
                    if self.show_siblings && indent < area.width {
                        folds.push(Rect::new(area.x + indent, y, 1, 1));
                    }
                }
            }
            hits.push((band, PickerHit::Left(index)));
        }
    }

    fn render_content(&mut self, area: Rect, buf: &mut Buffer, hits: &mut Vec<(Rect, PickerHit)>) {
        if area.height == 0 || area.width < 4 {
            return;
        }
        // Header.
        let note = match &self.listing {
            None => String::new(),
            Some(l) if l.truncated => {
                msg::folder_picker::first_entries(l.dirs.len() + l.files.len())
            }
            Some(_) if !self.filter.is_empty() => {
                let (shown, all) = self.entry_counts();
                msg::folder_picker::summary_filtered(msg::folder_picker::matches(shown as u64), all)
            }
            Some(l) => msg::folder_picker::summary(
                msg::folder_picker::folders(l.dirs.len() as u64),
                msg::folder_picker::files(l.files.len() as u64),
            ),
        };
        dialog_kit::put(
            buf,
            area.x,
            area.y,
            msg::folder_picker::NAME.get(),
            area.width,
            Style::default()
                .fg(th().ui.muted)
                .bg(th().ui.panel)
                .add_modifier(Modifier::BOLD),
        );
        dialog_kit::put(
            buf,
            area.right().saturating_sub(display_width(&note) as u16),
            area.y,
            &note,
            display_width(&note) as u16,
            Style::default().fg(th().ui.muted).bg(th().ui.panel),
        );
        let list = Rect::new(
            area.x,
            area.y + 1,
            area.width,
            area.height.saturating_sub(1),
        );
        let rows = self.content_rows();

        let mut y = list.y;
        let height = list.height as usize;
        if self.follow {
            if let Some(sel) = self.content_sel {
                if sel < self.content_scroll {
                    self.content_scroll = sel;
                } else if sel >= self.content_scroll + height {
                    self.content_scroll = (sel + 1).saturating_sub(height);
                }
            }
        }
        self.content_scroll = self.content_scroll.min(rows.len().saturating_sub(height));
        let list = self
            .content_bar
            .paint(buf, list, rows.len(), self.content_scroll);
        for (offset, row) in rows
            .iter()
            .skip(self.content_scroll)
            .take(height)
            .enumerate()
        {
            let index = self.content_scroll + offset;
            y = list.y + offset as u16;
            let band = Rect::new(list.x, y, list.width, 1);
            let selected = self.content_sel == Some(index);
            let hovered = self.hover == Some(PickerHit::Content(index));
            let bg = match (selected, self.zone == Zone::Content, hovered) {
                (true, true, _) => th().ui.selection,
                (true, false, _) => crate::lighten(th().ui.hover, 10),
                (false, _, true) => th().ui.hover,
                _ => th().ui.panel,
            };
            let (text, tag, fg) = match row {
                ContentRow::Parent(_) => (
                    "..".to_string(),
                    msg::folder_picker::UP_ONE_LEVEL.get(),
                    th().ui.text,
                ),
                ContentRow::Dir(i) => {
                    let entry = &self.listing.as_ref().expect("rows come from it").dirs[*i];
                    (
                        format!("{}{}", entry.name, std::path::MAIN_SEPARATOR),
                        if entry.is_repo {
                            msg::folder_picker::REPO_TAG.get()
                        } else {
                            ""
                        },
                        if entry.is_hidden {
                            th().ui.disabled
                        } else {
                            th().ui.text
                        },
                    )
                }
                ContentRow::File(i) => (
                    self.listing.as_ref().expect("rows come from it").files[*i]
                        .name
                        .clone(),
                    "",
                    if self.mode == PickerMode::File {
                        th().ui.text
                    } else {
                        th().ui.disabled
                    },
                ),
            };
            if self.mode == PickerMode::File || !matches!(row, ContentRow::File(_)) {
                dialog_kit::fill(buf, band, Style::default().bg(bg));
                hits.push((band, PickerHit::Content(index)));
            }
            let tag_w = display_width(tag) as u16;
            dialog_kit::put(
                buf,
                list.x,
                y,
                &text,
                list.width.saturating_sub(tag_w + 1),
                Style::default().fg(fg).bg(bg),
            );
            dialog_kit::put(
                buf,
                list.right().saturating_sub(tag_w),
                y,
                tag,
                tag_w,
                Style::default().fg(th().refs.local).bg(bg),
            );
            y += 1;
        }
        let note_y = if rows.is_empty() { list.y } else { y };
        let note = match &self.listing {
            None => Some((msg::folder_picker::LOADING.get().to_string(), th().ui.muted)),
            Some(l) => match &l.error {
                Some(error) => Some((msg::folder_picker::cannot_open(error), th().status.danger)),
                None if l.dirs.is_empty() && l.files.is_empty() => {
                    Some((msg::folder_picker::EMPTY.get().to_string(), th().ui.muted))
                }
                None if !self.filter.is_empty() && self.entry_counts().0 == 0 => Some((
                    msg::folder_picker::NO_MATCH.get().to_string(),
                    th().ui.muted,
                )),
                None => None,
            },
        };
        if let Some((text, color)) = note {
            if note_y < list.bottom() {
                dialog_kit::put(
                    buf,
                    list.x + 2,
                    note_y,
                    &text,
                    list.width.saturating_sub(2),
                    Style::default().fg(color).bg(th().ui.panel),
                );
            }
        }
    }

    /// Every hit rect of the last draw (tests).
    #[cfg(test)]
    pub(crate) fn hits(&self) -> &[(Rect, PickerHit)] {
        &self.hits
    }
}
