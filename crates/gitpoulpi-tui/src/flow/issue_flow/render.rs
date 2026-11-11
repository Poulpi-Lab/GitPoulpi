//! Drawing of the host-driven flow dialog: every rect that is clickable is
//! registered by the walk that paints it.

use super::*;

impl IssueFlowDialog {
    // -- drawing ---------------------------------------------------------------

    fn title_text(&self) -> String {
        match &self.kind {
            IssueFlowKind::New(_) => {
                let label = self
                    .seed
                    .view
                    .topics()
                    .get(self.topic)
                    .map(|t| t.display_label().to_string())
                    .unwrap_or_default();
                msg::issue_title_new(&label)
            }
            IssueFlowKind::PickUp => msg::ISSUE_TITLE_PICK_UP.get().to_string(),
            IssueFlowKind::Appeared { number, .. } => msg::issue_title_appeared(number),
        }
    }

    fn primary_label(&self) -> Option<&'static str> {
        match &self.phase {
            Phase::Form => Some(match self.kind {
                IssueFlowKind::New(_) => msg::ISSUE_BTN_CREATE.get(),
                _ => msg::ISSUE_BTN_LOOK_UP.get(),
            }),
            Phase::Found { found, .. } => found.best_branch().map(|_| msg::ISSUE_BTN_PICK_UP.get()),
            _ => None,
        }
    }

    fn secondary_label(&self) -> Option<&'static str> {
        match &self.phase {
            Phase::Form => Some(gitpoulpi_i18n::msg::dialog::CANCEL.get()),
            Phase::Working(..) | Phase::Picking(_) => None,
            Phase::Waiting { .. } => Some(msg::ISSUE_BTN_STOP.get()),
            Phase::Found { .. } => Some(match self.kind {
                IssueFlowKind::Appeared { .. } => msg::ISSUE_BTN_LATER.get(),
                _ => gitpoulpi_i18n::msg::dialog::CANCEL.get(),
            }),
            Phase::Done(_) | Phase::Failed(_) => Some(msg::BTN_CLOSE.get()),
        }
    }

    pub(crate) fn render(&mut self, rect: Rect, buf: &mut Buffer) {
        self.field_parts();
        self.edit.begin_frame();
        let title = self.title_text();
        self.close.framed(buf, rect, &title, &WIN, true);
        let mut hits: Vec<(Rect, Focus)> = Vec::new();
        let inner = Rect::new(
            rect.x + 2,
            rect.y + 1,
            rect.width.saturating_sub(4),
            rect.height.saturating_sub(2),
        );
        if inner.width < 30 || inner.height < 7 {
            self.hits = hits;
            return;
        }
        let panel_bg = th().ui.panel;
        let muted = Style::default().fg(th().ui.muted).bg(panel_bg);
        let normal = Style::default().fg(th().ui.text).bg(panel_bg);
        let buttons_y = inner.bottom().saturating_sub(1);
        let message_y = buttons_y.saturating_sub(1);
        let mut popups = false;
        let label_w: u16 = 18;
        let field_x = inner.x + label_w;
        let field_w = inner.width.saturating_sub(label_w);
        let hover = self.hover;
        let focus = self.focus;
        let mut y = inner.y;
        match &self.phase {
            Phase::Form => match self.kind {
                IssueFlowKind::New(_) => {
                    dialog_kit::put(buf, inner.x, y, msg::FIELD_TYPE.get(), label_w, muted);
                    let r = Rect::new(field_x, y, field_w, 1);
                    self.type_combo.render_field(
                        buf,
                        r,
                        focus == Focus::TypeCombo,
                        hover == Some(Focus::TypeCombo),
                    );
                    hits.push((r, Focus::TypeCombo));
                    popups = true;
                    y += 1;
                    dialog_kit::put(
                        buf,
                        inner.x,
                        y,
                        msg::ISSUE_FIELD_TITLE.get(),
                        label_w,
                        muted,
                    );
                    let r = Rect::new(field_x, y, field_w, 1);
                    dialog_kit::text_input(
                        buf,
                        r,
                        &self.title,
                        "",
                        focus == Focus::Title,
                        hover == Some(Focus::Title),
                        (&self.edit, Focus::Title),
                    );
                    hits.push((r, Focus::Title));
                    y += 1;
                    let flow = flow_of(&self.seed);
                    dialog_kit::put(
                        buf,
                        inner.x,
                        y,
                        msg::ISSUE_FIELD_DESCRIPTION.get(),
                        label_w,
                        muted,
                    );
                    let text = match flow.description_template.as_deref() {
                        Some(file) => msg::issue_description_file(file),
                        None => msg::ISSUE_DESCRIPTION_NONE.get().to_string(),
                    };
                    dialog_kit::put(buf, field_x, y, &text, field_w, normal);
                    y += 1;
                    dialog_kit::put(buf, inner.x, y, msg::ISSUE_FIELD_MODE.get(), label_w, muted);
                    let mode = match flow.mode {
                        IssueFlowMode::Local => msg::ISSUE_MODE_LOCAL.get(),
                        IssueFlowMode::HostCreatesBranch => msg::ISSUE_MODE_HOST.get(),
                        IssueFlowMode::Bot => msg::ISSUE_MODE_BOT.get(),
                    };
                    dialog_kit::put(buf, field_x, y, mode, field_w, normal);
                    y += 2;
                    if flow.mode != IssueFlowMode::Local && y < message_y {
                        let r = dialog_kit::checkbox(
                            buf,
                            field_x,
                            y,
                            field_w,
                            msg::ISSUE_PICK_AFTER.get(),
                            self.pick_after,
                            CheckState::Enabled {
                                hovered: hover == Some(Focus::PickAfter),
                                focused: focus == Focus::PickAfter,
                            },
                        );
                        hits.push((r, Focus::PickAfter));
                    }
                }
                _ => {
                    dialog_kit::put(
                        buf,
                        inner.x,
                        y,
                        msg::ISSUE_FIELD_REFERENCE.get(),
                        label_w,
                        muted,
                    );
                    let r = Rect::new(field_x, y, field_w, 1);
                    dialog_kit::text_input(
                        buf,
                        r,
                        &self.reference,
                        msg::ISSUE_HINT_REFERENCE.get(),
                        focus == Focus::Reference,
                        hover == Some(Focus::Reference),
                        (&self.edit, Focus::Reference),
                    );
                    hits.push((r, Focus::Reference));
                }
            },
            Phase::Working(work, since) => {
                let text = match work {
                    Work::Creating => msg::ISSUE_CREATING.get(),
                    Work::Making => msg::ISSUE_MAKING.get(),
                    Work::Looking => msg::ISSUE_LOOKING.get(),
                };
                if since.elapsed() >= SPIN_AFTER {
                    let spin = crate::activity::spinner_frame(since.elapsed());
                    dialog_kit::put(
                        buf,
                        inner.x,
                        y,
                        &format!("{spin} {text}"),
                        inner.width,
                        muted,
                    );
                }
            }
            Phase::Picking(since) => {
                if since.elapsed() >= SPIN_AFTER {
                    let spin = crate::activity::spinner_frame(since.elapsed());
                    dialog_kit::put(
                        buf,
                        inner.x,
                        y,
                        &format!("{spin} {}", msg::ISSUE_PICKING.get()),
                        inner.width,
                        muted,
                    );
                }
            }
            Phase::Waiting {
                since,
                number,
                wait,
                max,
            } => {
                let spin = crate::activity::spinner_frame(since.elapsed());
                let line = match wait {
                    WaitFor::Branch => msg::issue_waiting_branch(number),
                    WaitFor::MergeRequest => msg::issue_waiting_request(number),
                    WaitFor::BranchOrMergeRequest => msg::issue_waiting_either(number),
                };
                dialog_kit::put(
                    buf,
                    inner.x,
                    y,
                    &format!("{spin} {line}"),
                    inner.width,
                    normal,
                );
                y += 2;
                dialog_kit::put(
                    buf,
                    inner.x,
                    y,
                    &msg::issue_elapsed(since.elapsed().as_secs(), max.as_secs()),
                    inner.width,
                    muted,
                );
                y += 2;
                dialog_kit::put(
                    buf,
                    inner.x,
                    y,
                    msg::ISSUE_WAIT_HINT.get(),
                    inner.width,
                    muted,
                );
            }
            Phase::Found { found, .. } => {
                if found.branches.is_empty() && found.requests.is_empty() {
                    dialog_kit::put(
                        buf,
                        inner.x,
                        y,
                        msg::ISSUE_FOUND_NONE.get(),
                        inner.width,
                        muted,
                    );
                }
                for b in &found.branches {
                    if y >= message_y {
                        break;
                    }
                    dialog_kit::put(
                        buf,
                        inner.x,
                        y,
                        &msg::issue_found_branch(b),
                        inner.width,
                        normal,
                    );
                    y += 1;
                }
                for r in &found.requests {
                    if y >= message_y {
                        break;
                    }
                    let line =
                        msg::issue_found_request(r.number, &r.source_branch, &r.target_branch);
                    dialog_kit::put(buf, inner.x, y, &line, inner.width, normal);
                    y += 1;
                }
            }
            Phase::Done(text) => {
                dialog_kit::put(
                    buf,
                    inner.x,
                    y,
                    text,
                    inner.width,
                    Style::default().fg(th().status.success).bg(panel_bg),
                );
            }
            Phase::Failed(text) => {
                dialog_kit::put(
                    buf,
                    inner.x,
                    y,
                    text,
                    inner.width,
                    Style::default().fg(th().status.danger).bg(panel_bg),
                );
            }
        }
        if let Some(text) = &self.message {
            dialog_kit::put(
                buf,
                inner.x,
                message_y,
                text,
                inner.width,
                Style::default().fg(th().status.warning).bg(panel_bg),
            );
        }
        let mut right = inner.right();
        let mut place = |label: &str, focus_of: Focus, primary: bool, hits: &mut Vec<_>| {
            let w = dialog_kit::button_width(label);
            right = right.saturating_sub(w);
            let focused = self.focus == focus_of;
            let hovered = self.hover == Some(focus_of);
            let state = if primary {
                ButtonState::Primary { hovered, focused }
            } else {
                ButtonState::Normal { hovered, focused }
            };
            let r = dialog_kit::button(buf, right, buttons_y, label, state);
            hits.push((r, focus_of));
            right = right.saturating_sub(1);
        };
        if let Some(label) = self.secondary_label() {
            place(label, Focus::Secondary, false, &mut hits);
        }
        if let Some(label) = self.primary_label() {
            place(label, Focus::Primary, true, &mut hits);
        }
        self.hits = hits;
        if popups {
            self.type_combo.render_popup(buf, rect);
        }
    }
}

/// The rectangle the dialog draws in.
pub(crate) fn issue_flow_rect(frame: Rect) -> Rect {
    WIN.place(
        frame,
        dialog_kit::centered(frame, frame.width.saturating_sub(4).min(90), 14),
    )
}

const WIN: dialog_kit::Win = dialog_kit::Win::movable("issue_flow");
