use super::*;
use crate::keymap::chord::Key;
use crate::th;
use gitpoulpi_core::domain::ClonePhase;

/// The progress bar's phase word (core keeps the English label for the CLI).
fn phase_label(phase: ClonePhase) -> gitpoulpi_i18n::M {
    match phase {
        ClonePhase::Connecting => msg::clone::PHASE_CONNECTING,
        ClonePhase::Counting => msg::clone::PHASE_COUNTING,
        ClonePhase::Compressing => msg::clone::PHASE_COMPRESSING,
        ClonePhase::Receiving => msg::clone::PHASE_RECEIVING,
        ClonePhase::Resolving => msg::clone::PHASE_RESOLVING,
        ClonePhase::CheckingOut => msg::clone::PHASE_CHECKING_OUT,
        ClonePhase::LfsObjects => msg::clone::PHASE_LFS,
        ClonePhase::Done => msg::clone::PHASE_DONE,
    }
}

/// The private-repository mark: emoji presentation, two cells wide.
const LOCK: &str = "\u{1F512}";
const LOCK_W: u16 = 2;

impl CloneDialog {
    pub(crate) fn render(&mut self, rect: Rect, buf: &mut Buffer) {
        self.edit.sync(self.field_focus());
        self.edit.begin_frame();
        self.list_rows = (rect.height.saturating_sub(ROWS_AROUND_LIST) as usize).max(MIN_LIST_ROWS);
        self.close
            .framed(buf, rect, msg::clone::TITLE.get(), &super::WIN, true);
        let mut hits: Vec<(Rect, CloneHit)> = Vec::new();
        let inner = Rect::new(
            rect.x + 1,
            rect.y + 1,
            rect.width.saturating_sub(2),
            rect.height.saturating_sub(2),
        );
        if inner.width < 40 || inner.height < 12 {
            self.hits = hits;
            return;
        }
        let bottom = inner.bottom();
        // No provider built: the URL form alone, no sources column.
        let url_only = !crate::build_features::HOSTING;
        let left_w = if url_only {
            0
        } else {
            (inner.width / 4).clamp(18, 26)
        };
        let x0 = inner.x + 1;
        let body_bottom = bottom.saturating_sub(1);

        if !url_only {
            dialog_kit::put(
                buf,
                x0,
                inner.y,
                msg::clone::CLONE_FROM.get(),
                left_w,
                Style::default()
                    .fg(th().ui.muted)
                    .bg(th().ui.panel)
                    .add_modifier(Modifier::BOLD),
            );
            for i in 0..self.sources_len() {
                let y = inner.y + 1 + i as u16;
                if y >= body_bottom {
                    break;
                }
                let band = Rect::new(x0, y, left_w, 1);
                let selected = i == self.source;
                let hovered = self.hover == Some(CloneHit::Source(i));
                let bg = if selected && self.focus == CloneFocus::Sources {
                    th().ui.selection
                } else if selected {
                    crate::lighten(th().ui.hover, 10)
                } else if hovered {
                    th().ui.hover
                } else {
                    th().ui.panel
                };
                dialog_kit::fill(buf, band, Style::default().bg(bg));
                dialog_kit::put(
                    buf,
                    x0,
                    y,
                    &self.source_label(i),
                    left_w,
                    Style::default().fg(th().ui.text).bg(bg),
                );
                hits.push((band, CloneHit::Source(i)));
            }
            if self.hosted.is_empty() {
                // The note is a link to the Integrations settings: a Tab
                // stop, Enter or a click opens them.
                let note_y = inner.y + 3;
                let focused = self.focus == CloneFocus::Connect;
                let hovered = self.hover == Some(CloneHit::Part(CloneFocus::Connect));
                let style = if focused || hovered {
                    Style::default()
                        .fg(th().ui.accent)
                        .bg(if focused {
                            th().ui.selection
                        } else {
                            th().ui.hover
                        })
                        .add_modifier(Modifier::UNDERLINED)
                } else {
                    Style::default().fg(th().ui.disabled).bg(th().ui.panel)
                };
                let mut rows = 0u16;
                for (k, line) in
                    crate::ui_kit::wrap_words(msg::clone::CONNECT.get(), left_w as usize)
                        .iter()
                        .enumerate()
                {
                    let y = note_y + k as u16;
                    if y >= body_bottom {
                        break;
                    }
                    dialog_kit::fill(buf, Rect::new(x0, y, left_w, 1), style);
                    dialog_kit::put(buf, x0, y, line, left_w, style);
                    rows += 1;
                }
                if rows > 0 {
                    hits.push((
                        Rect::new(x0, note_y, left_w, rows),
                        CloneHit::Part(CloneFocus::Connect),
                    ));
                }
            }
            for y in inner.y..body_bottom {
                dialog_kit::put(
                    buf,
                    x0 + left_w + 1,
                    y,
                    "\u{2502}",
                    1,
                    Style::default().fg(th().ui.hover).bg(th().ui.panel),
                );
            }
        }

        let fx = if url_only { x0 } else { x0 + left_w + 3 };
        let fw = inner.right().saturating_sub(fx + 1);
        let mut y = inner.y;
        dialog_kit::put(
            buf,
            fx,
            y,
            &self.source_label(self.source),
            fw,
            Style::default()
                .fg(th().ui.text)
                .bg(th().ui.panel)
                .add_modifier(Modifier::BOLD),
        );
        y += 2;
        let label = |buf: &mut Buffer, y: u16, text: &str| {
            dialog_kit::put(
                buf,
                fx,
                y,
                text,
                fw,
                Style::default().fg(th().ui.muted).bg(th().ui.panel),
            );
        };
        label(buf, y, msg::clone::WHERE_TO.get());
        y += 1;
        let browse_w = dialog_kit::button_width(msg::dialog::BROWSE.get());
        let where_rect = Rect::new(fx, y, fw.saturating_sub(browse_w + 1), 1);
        dialog_kit::text_input(
            buf,
            where_rect,
            &self.where_to,
            msg::clone::WHERE_HINT.get(),
            self.focus == CloneFocus::Where,
            self.hover == Some(CloneHit::Part(CloneFocus::Where)),
            (&self.edit, CloneFocus::Where),
        );
        let browse = dialog_kit::button(
            buf,
            where_rect.right() + 1,
            y,
            msg::dialog::BROWSE.get(),
            if self.busy() {
                ButtonState::Disabled
            } else {
                ButtonState::Normal {
                    hovered: self.hover == Some(CloneHit::Part(CloneFocus::Browse)),
                    focused: self.focus == CloneFocus::Browse,
                }
            },
        );
        hits.insert(0, (browse, CloneHit::Part(CloneFocus::Browse)));
        hits.push((where_rect, CloneHit::Part(CloneFocus::Where)));
        y += 2;
        if self.hosted_mode() {
            y = self.render_repos(buf, fx, fw, y, &mut hits);
        } else {
            label(buf, y, msg::clone::URL.get());
            y += 1;
            let url_rect = Rect::new(fx, y, fw, 1);
            dialog_kit::text_input(
                buf,
                url_rect,
                &self.url,
                msg::clone::URL_HINT.get(),
                self.focus == CloneFocus::Url,
                self.hover == Some(CloneHit::Part(CloneFocus::Url)),
                (&self.edit, CloneFocus::Url),
            );
            hits.push((url_rect, CloneHit::Part(CloneFocus::Url)));
            y += 1;
        }
        y += 2;
        label(buf, y, msg::clone::FOLDER_NAME.get());
        y += 1;
        let name_rect = Rect::new(fx, y, fw, 1);
        dialog_kit::text_input(
            buf,
            name_rect,
            &self.name,
            msg::clone::FOLDER_NAME_HINT.get(),
            self.focus == CloneFocus::Name,
            self.hover == Some(CloneHit::Part(CloneFocus::Name)),
            (&self.edit, CloneFocus::Name),
        );
        hits.push((name_rect, CloneHit::Part(CloneFocus::Name)));
        y += 1;
        if let Some(dest) = self.dest() {
            let lead = msg::clone::FULL_PATH.get();
            let lead_w = dialog_kit::put(
                buf,
                fx,
                y,
                lead,
                fw,
                Style::default().fg(th().ui.muted).bg(th().ui.panel),
            )
            .width;
            let room = fw.saturating_sub(lead_w);
            dialog_kit::put(
                buf,
                fx + lead_w,
                y,
                &crate::tail_fit(&dest.display().to_string(), room as usize),
                room,
                Style::default().fg(th().ui.text).bg(th().ui.panel),
            );
        }
        y += 1;
        if let Some((text, warn)) = self.dest_warning() {
            dialog_kit::put(
                buf,
                fx,
                y,
                &text,
                fw,
                Style::default()
                    .fg(if warn {
                        th().status.warning
                    } else {
                        th().ui.muted
                    })
                    .bg(th().ui.panel),
            );
        }
        y += 2;
        let shallow = dialog_kit::checkbox(
            buf,
            fx,
            y,
            fw,
            msg::clone::SHALLOW.get(),
            self.shallow,
            if self.busy() {
                CheckState::Disabled
            } else {
                CheckState::Enabled {
                    hovered: self.hover == Some(CloneHit::Part(CloneFocus::Shallow)),
                    focused: self.focus == CloneFocus::Shallow,
                }
            },
        );
        hits.push((shallow, CloneHit::Part(CloneFocus::Shallow)));
        y += 1;
        let sparse = dialog_kit::checkbox(
            buf,
            fx,
            y,
            fw,
            msg::sparse::CLONE_CHECK.get(),
            self.sparse,
            if self.busy() {
                CheckState::Disabled
            } else {
                CheckState::Enabled {
                    hovered: self.hover == Some(CloneHit::Part(CloneFocus::Sparse)),
                    focused: self.focus == CloneFocus::Sparse,
                }
            },
        );
        hits.push((sparse, CloneHit::Part(CloneFocus::Sparse)));
        if self.sparse {
            y += 1;
            let dirs_rect = Rect::new(fx + 4, y, fw.saturating_sub(4), 1);
            dialog_kit::text_input(
                buf,
                dirs_rect,
                &self.sparse_dirs,
                msg::sparse::CLONE_HINT.get(),
                self.focus == CloneFocus::SparseDirs,
                self.hover == Some(CloneHit::Part(CloneFocus::SparseDirs)),
                (&self.edit, CloneFocus::SparseDirs),
            );
            hits.push((dirs_rect, CloneHit::Part(CloneFocus::SparseDirs)));
        }

        // Progress, or why the button is off, then the last message.
        let status_y = body_bottom.saturating_sub(2).max(y + 1);
        let plan = self.plan();
        if let Some(progress) = &self.progress {
            let bar_w = (fw as usize).saturating_sub(34).clamp(8, 40);
            let filled = bar_w * progress.percent as usize / 100;
            let counts = match (progress.current, progress.total) {
                (Some(c), Some(t)) => format!(" ({c}/{t})"),
                _ => String::new(),
            };
            let text = format!(
                "[{}{}] {:>3}%  {}{counts}",
                "#".repeat(filled),
                "-".repeat(bar_w - filled),
                progress.percent,
                phase_label(progress.phase).get()
            );
            dialog_kit::put(
                buf,
                fx,
                status_y,
                &text,
                fw,
                Style::default().fg(th().ui.accent).bg(th().ui.panel),
            );
        } else if let Err(reason) = &plan {
            dialog_kit::put(
                buf,
                fx,
                status_y,
                reason,
                fw,
                Style::default().fg(th().status.warning).bg(th().ui.panel),
            );
        }
        if let Some(message) = &self.message {
            dialog_kit::put(
                buf,
                fx,
                status_y + 1,
                message,
                fw,
                Style::default().fg(th().status.danger).bg(th().ui.panel),
            );
        }

        let footer = bottom.saturating_sub(1);
        let clone_label = if self.busy() {
            msg::clone::CLONING.get()
        } else {
            msg::clone::GO.get()
        };
        let clone_w = dialog_kit::button_width(clone_label);
        let cancel_w = dialog_kit::button_width(msg::dialog::CANCEL.get());
        let clone_x = inner.right().saturating_sub(clone_w + 1);
        let cancel_x = clone_x.saturating_sub(cancel_w + 1);
        let cancel = dialog_kit::button(
            buf,
            cancel_x,
            footer,
            msg::dialog::CANCEL.get(),
            ButtonState::Normal {
                hovered: self.hover == Some(CloneHit::Part(CloneFocus::Cancel)),
                focused: self.focus == CloneFocus::Cancel,
            },
        );
        let clone = dialog_kit::button(
            buf,
            clone_x,
            footer,
            clone_label,
            if plan.is_err() || self.busy() {
                ButtonState::Disabled
            } else {
                ButtonState::Primary {
                    hovered: self.hover == Some(CloneHit::Part(CloneFocus::Clone)),
                    focused: self.focus == CloneFocus::Clone,
                }
            },
        );
        hits.insert(0, (cancel, CloneHit::Part(CloneFocus::Cancel)));
        hits.insert(0, (clone, CloneHit::Part(CloneFocus::Clone)));
        let browse = dialog_kit::key_label(crate::keymap::Action::DialogBrowse);
        let tab = dialog_kit::key_name(Key::Tab);
        let enter = dialog_kit::key_name(Key::Enter);
        let clear = dialog_kit::ctrl_key('u');
        let sources = dialog_kit::key_label(crate::keymap::Action::DialogSources);
        let mut pairs = vec![
            (tab.as_str(), msg::dialog::HINT_NEXT.get()),
            (enter.as_str(), msg::clone::HINT_CLONE.get()),
            (browse.as_str(), msg::dialog::HINT_BROWSE.get()),
        ];
        if !url_only {
            pairs.push((sources.as_str(), msg::clone::HINT_SOURCES.get()));
        }
        pairs.push((clear.as_str(), msg::dialog::HINT_CLEAR_FIELD.get()));
        dialog_kit::hints(buf, x0, footer, cancel_x.saturating_sub(x0 + 1), &pairs);
        self.hits = if self.confirm.is_some() {
            self.render_confirm(rect, buf)
        } else {
            hits
        };
        crate::pointer_map::from_hits(&self.hits, |h| match h {
            CloneHit::Part(
                CloneFocus::Where
                | CloneFocus::Url
                | CloneFocus::Name
                | CloneFocus::Search
                | CloneFocus::SparseDirs,
            ) => Some(crate::pointer_map::Kind::Field),
            // The source and repository rows light on hover and keep the arrow.
            CloneHit::Source(_) | CloneHit::Repo(_) => Some(crate::pointer_map::Kind::Plain),
            _ => Some(crate::pointer_map::Kind::Hand),
        });
    }

    /// The destination-exists popup, over the form: returns its hit
    /// rects, which are the only ones live while it is open.
    fn render_confirm(&mut self, rect: Rect, buf: &mut Buffer) -> Vec<(Rect, CloneHit)> {
        let Some(confirm) = self.confirm.clone() else {
            return Vec::new();
        };
        let (text, go_label) = match &confirm.kind {
            dest::DestKind::SameRepo => (
                msg::clone::CONFIRM_SAME.get().to_string(),
                msg::clone::BTN_OPEN.get().to_string(),
            ),
            dest::DestKind::NotEmpty { suggestion } => (
                msg::clone::CONFIRM_NOT_EMPTY.get().to_string(),
                msg::clone::btn_use_name(suggestion),
            ),
            _ => (
                msg::clone::CONFIRM_EMPTY.get().to_string(),
                msg::clone::BTN_CLONE_INTO.get().to_string(),
            ),
        };
        let back_label = msg::clone::BTN_BACK.get();
        let w = rect.width.saturating_sub(4).clamp(20, 64);
        let inner_w = w.saturating_sub(4) as usize;
        let lines = crate::ui_kit::wrap_words(&text, inner_w);
        let h = lines.len() as u16 + 6;
        let pop = dialog_kit::centered(rect, w, h);
        self.confirm_rect = pop;
        self.confirm_close
            .panel(buf, pop, msg::clone::CONFIRM_TITLE.get());
        let (x, mut y) = (pop.x + 2, pop.y + 1);
        let muted = Style::default().fg(th().ui.muted).bg(th().ui.panel);
        if let Some(dest) = self.dest() {
            dialog_kit::put(
                buf,
                x,
                y,
                &crate::tail_fit(&dest.display().to_string(), inner_w),
                inner_w as u16,
                muted,
            );
        }
        y += 1;
        for line in &lines {
            dialog_kit::put(
                buf,
                x,
                y,
                line,
                inner_w as u16,
                Style::default().fg(th().ui.text).bg(th().ui.panel),
            );
            y += 1;
        }
        y += 1;
        let go_w = dialog_kit::button_width(&go_label);
        let back_w = dialog_kit::button_width(back_label);
        let go_x = pop.right().saturating_sub(2 + go_w + 1 + back_w);
        let go = dialog_kit::button(
            buf,
            go_x,
            y,
            &go_label,
            ButtonState::Primary {
                hovered: self.hover == Some(CloneHit::Confirm(ConfirmButton::Go)),
                focused: confirm.focus == ConfirmButton::Go,
            },
        );
        let back = dialog_kit::button(
            buf,
            go_x + go_w + 1,
            y,
            back_label,
            ButtonState::Normal {
                hovered: self.hover == Some(CloneHit::Confirm(ConfirmButton::Back)),
                focused: confirm.focus == ConfirmButton::Back,
            },
        );
        let tab = dialog_kit::key_name(Key::Tab);
        dialog_kit::hints(
            buf,
            x,
            y + 1,
            inner_w as u16,
            &[(tab.as_str(), msg::dialog::HINT_NEXT.get())],
        );
        vec![
            (go, CloneHit::Confirm(ConfirmButton::Go)),
            (back, CloneHit::Confirm(ConfirmButton::Back)),
        ]
    }

    /// The hosted source's search box and repository list; returns the
    /// next free row.
    fn render_repos(
        &mut self,
        buf: &mut Buffer,
        fx: u16,
        fw: u16,
        mut y: u16,
        hits: &mut Vec<(Rect, CloneHit)>,
    ) -> u16 {
        dialog_kit::put(
            buf,
            fx,
            y,
            msg::clone::REPOSITORY.get(),
            fw,
            Style::default().fg(th().ui.muted).bg(th().ui.panel),
        );
        // A saved list still showing: the state of its refresh, right-aligned
        // on the label row (no hit target).
        let note = self.hosted_source().and_then(|s| {
            if self.refreshing.contains(&s.profile_id) {
                Some(msg::clone::REFRESHING.get())
            } else if self.refresh_failed.contains_key(&s.profile_id) {
                Some(msg::clone::REFRESH_FAILED.get())
            } else {
                None
            }
        });
        if let Some(note) = note {
            let w = crate::ui_kit::display_width(note) as u16;
            if w + 14 < fw {
                dialog_kit::put(
                    buf,
                    fx + fw - w,
                    y,
                    note,
                    w,
                    Style::default().fg(th().ui.muted).bg(th().ui.panel),
                );
            }
        }
        y += 1;
        let search_rect = Rect::new(fx, y, fw, 1);
        dialog_kit::text_input(
            buf,
            search_rect,
            &self.search,
            msg::clone::SEARCH_HINT.get(),
            self.focus == CloneFocus::Search,
            self.hover == Some(CloneHit::Part(CloneFocus::Search)),
            (&self.edit, CloneFocus::Search),
        );
        hits.push((search_rect, CloneHit::Part(CloneFocus::Search)));
        y += 1;

        let source = self.hosted_source().map(|s| s.profile_id.clone());
        let state = source
            .as_ref()
            .and_then(|id| self.projects.get(id))
            .cloned();
        let empty = self.grouped().is_empty();
        self.list_bar = None;
        self.list_len = 0;
        let list_focus = matches!(self.focus, CloneFocus::Search | CloneFocus::Repos);
        let mut painted = Vec::new();
        let mut private_note = false;
        let mut view = None;
        match &state {
            Some(ProjectsState::Ready(all)) if empty => {
                let text = if all.is_empty() {
                    msg::clone::NO_REPOS.get()
                } else {
                    msg::clone::NO_MATCH.get()
                };
                dialog_kit::put(
                    buf,
                    fx,
                    y,
                    text,
                    fw,
                    Style::default().fg(th().ui.muted).bg(th().ui.panel),
                );
            }
            Some(ProjectsState::Ready(_)) => {
                let groups = self.grouped();
                let list_rows = groups::rows(&groups);
                let flat: Vec<&HostedProject> = groups
                    .iter()
                    .flat_map(|g| g.repos.iter().copied())
                    .collect();
                // The offset may be stale after a search or a new load.
                let offset = self
                    .repo_offset
                    .min(list_rows.len().saturating_sub(self.list_rows));
                // The bar lives in the rightmost column of the list; the
                // rows keep off it.
                let body = Rect::new(fx, y.saturating_sub(1), fw, self.list_rows as u16 + 2);
                let bar = crate::scrollbar::Track::vertical(body, list_rows.len(), offset);
                if let Some(track) = bar {
                    crate::scrollbar::render_buf(buf, track, true, false);
                }
                view = Some((list_rows.len(), offset, bar));
                let cw = fw.saturating_sub(1);
                for (slot, row) in list_rows
                    .iter()
                    .enumerate()
                    .skip(offset)
                    .take(self.list_rows)
                    .map(|(i, r)| (i - offset, r))
                {
                    let row_y = y + slot as u16;
                    let index = match row {
                        groups::ListRow::Header { owner, count } => {
                            let band = Rect::new(fx, row_y, cw, 1);
                            let bg = th().ui.panel;
                            dialog_kit::fill(buf, band, Style::default().bg(bg));
                            let owner = if owner.is_empty() {
                                "-"
                            } else {
                                owner.as_str()
                            };
                            dialog_kit::put(
                                buf,
                                fx,
                                row_y,
                                &format!("{owner} ({count})"),
                                cw,
                                Style::default()
                                    .fg(th().ui.muted)
                                    .bg(bg)
                                    .add_modifier(Modifier::BOLD),
                            );
                            continue;
                        }
                        groups::ListRow::Repo(index) => *index,
                    };
                    let project = flat[index];
                    let band = Rect::new(fx, row_y, cw, 1);
                    let cursor = index == self.repo_cursor && list_focus;
                    let hovered = self.hover == Some(CloneHit::Repo(index));
                    let chosen = self.chosen.as_deref() == Some(project.full_name.as_str());
                    let bg = if cursor {
                        th().ui.selection
                    } else if hovered {
                        th().ui.hover
                    } else {
                        th().ui.panel
                    };
                    dialog_kit::fill(buf, band, Style::default().bg(bg));
                    // The lock is two cells wide (emoji presentation) and
                    // sits at the right end, the text stops before it.
                    let lock_w = if project.private { LOCK_W + 1 } else { 0 };
                    let text_w = cw.saturating_sub(lock_w);
                    let mark = if chosen { "\u{2713} " } else { "  " };
                    let lead = format!("{mark}{}", project.full_name);
                    let lead_w = dialog_kit::put(
                        buf,
                        fx,
                        row_y,
                        &lead,
                        text_w,
                        Style::default()
                            .fg(if chosen { th().ui.accent } else { th().ui.text })
                            .bg(bg)
                            .add_modifier(if cursor || chosen {
                                Modifier::BOLD
                            } else {
                                Modifier::empty()
                            }),
                    )
                    .width;
                    if let Some(desc) = project.description.as_deref().filter(|d| !d.is_empty()) {
                        let room = text_w.saturating_sub(lead_w + 2);
                        if room > 6 {
                            dialog_kit::put(
                                buf,
                                fx + lead_w + 2,
                                row_y,
                                desc,
                                room,
                                Style::default().fg(th().ui.muted).bg(bg),
                            );
                        }
                    }
                    if project.private {
                        dialog_kit::put(
                            buf,
                            fx + cw - LOCK_W,
                            row_y,
                            LOCK,
                            LOCK_W,
                            Style::default().fg(th().ui.text).bg(bg),
                        );
                    }
                    painted.push((band, CloneHit::Repo(index)));
                    if (hovered || cursor && self.hover.is_none()) && project.private {
                        private_note = true;
                    }
                }
            }
            Some(ProjectsState::Failed(why)) => {
                dialog_kit::put(
                    buf,
                    fx,
                    y,
                    &msg::clone::list_failed(why),
                    fw,
                    Style::default().fg(th().status.danger).bg(th().ui.panel),
                );
            }
            _ => {
                dialog_kit::put(
                    buf,
                    fx,
                    y,
                    msg::clone::LOADING.get(),
                    fw,
                    Style::default().fg(th().ui.muted).bg(th().ui.panel),
                );
            }
        }
        if let Some((len, offset, bar)) = view {
            self.list_len = len;
            self.repo_offset = offset;
            self.list_bar = bar;
        }
        if private_note {
            // The tooltip of the lock, right-aligned on the label row.
            let note = msg::clone::PRIVATE_REPO.get();
            let w = crate::ui_kit::display_width(note) as u16;
            if w + 14 < fw {
                dialog_kit::put(
                    buf,
                    fx + fw - w,
                    y.saturating_sub(2),
                    note,
                    w,
                    Style::default().fg(th().ui.muted).bg(th().ui.panel),
                );
            }
        }
        // Rows sit inside the panel: registered inner-first.
        for entry in painted.into_iter().rev() {
            hits.insert(0, entry);
        }
        y + self.list_rows as u16
    }
}
