use super::*;
use crate::display_width;
use crate::keymap::chord::Key;
use crate::th;

impl PreferencesState {
    pub(in crate::preferences) fn render_keys(&mut self, frame: &mut Frame, area: Rect) {
        let screen = frame.area();
        self.keys.relabel();
        let focused = self.focus == PrefFocus::Fields;
        let hover = self.mouse_pos;
        let page = &mut self.keys;
        let buf = frame.buffer_mut();
        dialog_kit::fill(buf, area, Style::default());
        page.rows.clear();
        page.resets.clear();
        page.marks.clear();
        page.popup_hits.clear();
        if area.height < 8 || area.width < 30 {
            dialog_kit::put(
                buf,
                area.x,
                area.y,
                msg::TOO_SMALL.get(),
                area.width,
                Style::default().fg(th().ui.muted),
            );
            return;
        }
        let hovered = |r: Rect| hover.is_some_and(|(c, row)| r.contains((c, row).into()));
        let w = area.width;
        let mut y = area.y;
        let search_w = 30.min(w.saturating_sub(8));
        let search_label = msg::SEARCH.get();
        let search_label_w = display_width(search_label) as u16;
        dialog_kit::put(
            buf,
            area.x,
            y,
            search_label,
            search_label_w,
            Style::default().fg(th().ui.muted),
        );
        let search_rect = Rect::new(area.x + search_label_w + 1, y, search_w, 1);
        page.search_rect = search_rect;
        let in_search = focused && page.zone == Zone::Search && page.popup.is_none();
        let bg = crate::input_bg(in_search, hovered(search_rect));
        dialog_kit::fill(buf, search_rect, Style::default().bg(bg));
        if page.query.is_empty() && !in_search {
            dialog_kit::put(
                buf,
                search_rect.x + 1,
                y,
                msg::SEARCH_PLACEHOLDER.get(),
                search_rect.width.saturating_sub(1),
                Style::default().fg(th().ui.muted).bg(bg),
            );
        } else {
            let text = format!("{}{}", page.query, if in_search { "\u{2588}" } else { "" });
            dialog_kit::put(
                buf,
                search_rect.x + 1,
                y,
                &crate::tail_fit(&text, search_rect.width.saturating_sub(1) as usize),
                search_rect.width.saturating_sub(1),
                Style::default().fg(th().ui.text).bg(bg),
            );
        }
        y += 1;
        // Context, show, reset all.
        let mut x = area.x;
        let context_label = msg::CONTEXT.get();
        let context_label_w = display_width(context_label) as u16;
        dialog_kit::put(
            buf,
            x,
            y,
            context_label,
            context_label_w,
            Style::default().fg(th().ui.muted),
        );
        x += context_label_w + 1;
        // The context dropdown takes what the other controls leave, 14 to 30
        // cells (their labels are as wide as the language makes them).
        let reset_all_label = msg::RESET_ALL_BUTTON.get();
        let others = context_label_w
            + 1
            + 2
            + display_width(msg::SHOW.get()) as u16
            + 1
            + 14
            + dialog_kit::button_width(reset_all_label);
        let ctx_w = w.saturating_sub(others).clamp(14, 30);
        let ctx_rect = Rect::new(x, y, ctx_w, 1);
        page.context.render_field(
            buf,
            ctx_rect,
            focused && page.zone == Zone::Context,
            hovered(ctx_rect),
        );
        x += ctx_w + 2;
        let show_label = msg::SHOW.get();
        let show_label_w = display_width(show_label) as u16;
        dialog_kit::put(
            buf,
            x,
            y,
            show_label,
            show_label_w,
            Style::default().fg(th().ui.muted),
        );
        x += show_label_w + 1;
        let show_rect = Rect::new(x, y, 12, 1);
        page.show.render_field(
            buf,
            show_rect,
            focused && page.zone == Zone::Show,
            hovered(show_rect),
        );
        x += 14;
        let reset_state = ButtonState::Normal {
            hovered: hover.is_some_and(|(c, r)| {
                Rect::new(x, y, dialog_kit::button_width(reset_all_label), 1)
                    .contains((c, r).into())
            }),
            focused: focused && page.zone == Zone::ResetAll,
        };
        page.reset_all_rect = dialog_kit::button(buf, x, y, reset_all_label, reset_state);
        y += 1;
        {
            // Key names come from the chord printer; the plain letters are
            // this page's own fixed keys.
            let enter = dialog_kit::key_name(Key::Enter);
            let del = dialog_kit::key_name(Key::Delete);
            let tab = dialog_kit::key_name(Key::Tab);
            let pairs = [
                (enter.as_str(), msg::HINT_REBIND.get()),
                ("a", msg::HINT_ADD.get()),
                (del.as_str(), msg::HINT_REMOVE.get()),
                ("u", msg::HINT_UNBIND.get()),
                ("r", msg::HINT_RESET.get()),
                ("R", msg::HINT_RESET_ALL.get()),
                ("/", msg::HINT_SEARCH.get()),
                ("c", msg::HINT_CONFLICTS.get()),
                (tab.as_str(), msg::HINT_CONTROLS.get()),
            ];
            let line = pairs
                .iter()
                .map(|(key, what)| format!("{key} {what}"))
                .collect::<Vec<_>>()
                .join("  ");
            dialog_kit::put(buf, area.x, y, &line, w, Style::default().fg(th().ui.muted));
        }
        y += 1;
        if let Some(action) = page.conflicts_of {
            let label = ACTIONS
                .iter()
                .find(|d| d.action == action)
                .map_or_else(Default::default, |d| d.display_label());
            dialog_kit::put(
                buf,
                area.x,
                y,
                &msg::conflicts_of(&label),
                w,
                Style::default().fg(th().status.danger),
            );
        } else if let Some(notice) = page.notice.as_deref() {
            dialog_kit::put(
                buf,
                area.x,
                y,
                notice,
                w,
                Style::default().fg(th().ui.accent),
            );
        } else if let Some(first) = page.issues.first() {
            let text = msg::config_issues(page.issues.len() as u64, &first.id, &first.message);
            dialog_kit::put(
                buf,
                area.x,
                y,
                &text,
                w,
                Style::default().fg(th().status.warning),
            );
        }
        y += 1;
        let list_top = y;
        let detail_rows = 2u16;
        let list_h = area.bottom().saturating_sub(list_top + detail_rows).max(1);
        page.list_height = list_h as usize;
        let lines = page.lines();
        page.scroll = page
            .scroll
            .min(lines.len().saturating_sub(page.list_height));
        page.bar = crate::scrollbar::Track::vertical(
            Rect::new(area.x, list_top - 1, area.width, list_h + 2),
            lines.len(),
            page.scroll,
        );
        let label_w = (w as usize * 2 / 5).clamp(20, 42);
        let reset_label = msg::RESET_BUTTON.get();
        let reset_w = dialog_kit::button_width(reset_label);
        let list_focus = focused && page.zone == Zone::List;
        if lines.is_empty() {
            dialog_kit::put(
                buf,
                area.x + 2,
                list_top,
                msg::NO_MATCH.get(),
                w,
                Style::default().fg(th().ui.muted),
            );
        }
        for (offset, line) in lines
            .iter()
            .skip(page.scroll)
            .take(page.list_height)
            .enumerate()
        {
            let row_y = list_top + offset as u16;
            match *line {
                ListLine::Heading(ctx) => {
                    dialog_kit::put(
                        buf,
                        area.x,
                        row_y,
                        ctx_title(ctx),
                        w,
                        Style::default()
                            .fg(th().ui.accent)
                            .add_modifier(Modifier::BOLD),
                    );
                }
                ListLine::Action(index) => {
                    let def = &ACTIONS[index];
                    let row_rect = Rect::new(area.x, row_y, w, 1);
                    let selected = index == page.selected;
                    let base = if selected && list_focus {
                        Style::default().fg(th().ui.text).bg(th().ui.selection)
                    } else if selected || hovered(row_rect) {
                        Style::default().fg(th().ui.text).bg(th().ui.hover)
                    } else {
                        Style::default().fg(th().ui.text)
                    };
                    dialog_kit::fill(buf, row_rect, base);
                    dialog_kit::put(
                        buf,
                        area.x + 1,
                        row_y,
                        &def.display_label(),
                        label_w as u16,
                        base,
                    );
                    let modified = page.km.is_modified(def.action);
                    let chips_x = area.x + 1 + label_w as u16 + 1;
                    let right_edge = area.right().saturating_sub(reset_w + 3);
                    let chips: Vec<String> = page
                        .km
                        .labels(def.action)
                        .iter()
                        .map(|c| format!("[{c}]"))
                        .collect();
                    let chips_text = if chips.is_empty() {
                        msg::UNBOUND.get().to_string()
                    } else {
                        chips.join(" ")
                    };
                    // A key that clashes with another action's is red; a
                    // warning about the key alone (a terminal that cannot
                    // send it) orange.
                    let chip_style = if chips.is_empty() {
                        base.fg(th().ui.muted)
                    } else if !page.clashing_with(def.action).is_empty()
                        || page.problem_level(def.action) == Some(Level::Error)
                    {
                        base.fg(th().status.danger)
                    } else if page.problem_level(def.action).is_some() {
                        base.fg(th().status.warning)
                    } else {
                        base.fg(th().ui.accent)
                    };
                    let room = right_edge.saturating_sub(chips_x + 2);
                    let used = dialog_kit::put(buf, chips_x, row_y, &chips_text, room, chip_style);
                    if modified {
                        dialog_kit::put(
                            buf,
                            chips_x + used.width + 1,
                            row_y,
                            "\u{2022}",
                            1,
                            base.fg(th().status.warning),
                        );
                    }
                    if page.problem_level(def.action).is_some() {
                        page.marks.push((Rect::new(right_edge, row_y, 1, 1), index));
                    }
                    let clash = !page.clashing_with(def.action).is_empty();
                    match page.problem_level(def.action) {
                        Some(Level::Error) => {
                            dialog_kit::put(
                                buf,
                                right_edge,
                                row_y,
                                "!",
                                1,
                                base.fg(th().status.danger),
                            );
                        }
                        // Red like the key when it clashes with another action.
                        Some(_) => {
                            dialog_kit::put(
                                buf,
                                right_edge,
                                row_y,
                                "!",
                                1,
                                base.fg(if clash {
                                    th().status.danger
                                } else {
                                    th().status.warning
                                }),
                            );
                        }
                        None => {}
                    }
                    let mut row_hit = row_rect;
                    if modified {
                        let rx = area.right().saturating_sub(reset_w + 1);
                        let rr = Rect::new(rx, row_y, reset_w, 1);
                        let state = ButtonState::Normal {
                            hovered: hovered(rr),
                            focused: false,
                        };
                        dialog_kit::button(buf, rx, row_y, reset_label, state);
                        page.resets.push((rr, index));
                        row_hit.width = rx.saturating_sub(area.x);
                    }
                    page.rows.push((row_hit, index));
                }
            }
        }
        if let Some(track) = page.bar {
            let lit = hover.is_some_and(|(c, r)| track.rect.contains((c, r).into()));
            crate::scrollbar::render_buf(buf, track, true, lit);
        }
        // The selected row's details.
        let detail_y = area.bottom().saturating_sub(detail_rows);
        if let Some(def) = ACTIONS
            .get(page.selected)
            .filter(|_| page.visible().contains(&page.selected))
        {
            let default_text = def.defaults.join(", ");
            let mut first = msg::detail(
                def.config_id(),
                if default_text.is_empty() {
                    msg::NONE.get()
                } else {
                    &default_text
                },
            );
            if let Some(hint) = def.hint {
                first.push_str(&format!("   ({hint})"));
            }
            dialog_kit::put(
                buf,
                area.x,
                detail_y,
                &first,
                w,
                Style::default().fg(th().ui.muted),
            );
            let notes: Vec<&Conflict> = page
                .conflicts
                .iter()
                .filter(|c| c.action == def.action || c.other == Some(def.action))
                .collect();
            if let Some(worst) = notes.first() {
                let style = match worst.level {
                    Level::Error => Style::default().fg(th().status.danger),
                    Level::Warning => Style::default().fg(th().status.warning),
                    Level::Info => Style::default().fg(th().ui.muted),
                };
                dialog_kit::put(buf, area.x, detail_y + 1, &worst.message, w, style);
            }
        }
        // Combos' popups, then the page's own popups.
        page.context.render_popup(buf, screen);
        page.show.render_popup(buf, screen);
        render_popup(page, buf, screen, hover);
    }
}

/// Whether the pointer (if known) is inside `rect`.
pub(super) fn pointer_in(pointer: Option<(u16, u16)>, rect: Rect) -> bool {
    pointer.is_some_and(|(c, r)| rect.contains((c, r).into()))
}

pub(super) fn render_popup(
    page: &mut KeybindingsPage,
    buf: &mut Buffer,
    screen: Rect,
    pointer: Option<(u16, u16)>,
) {
    let Some(popup) = page.popup.clone() else {
        return;
    };
    if let Some((c, r)) = pointer {
        page.popup_close.set_hover(c, r);
    }
    match popup {
        Popup::ConfirmReset { focus } => {
            let rect = WIN_RESET.place(screen, dialog_kit::centered(screen, 62, 8));
            page.popup_close
                .framed(buf, rect, msg::CONFIRM_TITLE.get(), &WIN_RESET, true);
            let inner = Rect::new(rect.x + 2, rect.y + 2, rect.width.saturating_sub(4), 3);
            let text = [msg::CONFIRM_LINE_1.get(), msg::CONFIRM_LINE_2.get()];
            for (i, line) in text.iter().enumerate() {
                dialog_kit::put(
                    buf,
                    inner.x,
                    inner.y + i as u16,
                    line,
                    inner.width,
                    Style::default().fg(th().ui.text).bg(th().ui.panel),
                );
            }
            let by = rect.bottom().saturating_sub(2);
            let mut hits = Vec::new();
            let mut x = rect.x + 2;
            for (i, btn) in [Btn::ConfirmReset, Btn::Cancel].into_iter().enumerate() {
                let r = dialog_kit::button(
                    buf,
                    x,
                    by,
                    btn.label(),
                    ButtonState::Normal {
                        hovered: pointer_in(
                            pointer,
                            Rect::new(x, by, dialog_kit::button_width(btn.label()), 1),
                        ),
                        focused: i == focus,
                    },
                );
                hits.push((r, Hit::Button(btn)));
                x += r.width + 2;
            }
            hits.push((rect, Hit::Popup));
            page.popup_hits = hits;
        }
        Popup::Capture(cap) => {
            let def = cap.def();
            let press = {
                let esc = dialog_kit::key_name(Key::Esc);
                if cap.add {
                    msg::press_extra(def.display_label(), esc)
                } else {
                    msg::press_new(def.display_label(), esc)
                }
            };
            // Wide enough for the sentence in this language (72 in English).
            let want = (display_width(&press) + 4).min(screen.width as usize) as u16;
            let rect = WIN_CAPTURE.place(screen, dialog_kit::centered(screen, want.max(72), 15));
            page.popup_close
                .framed(buf, rect, msg::CAPTURE_TITLE.get(), &WIN_CAPTURE, true);
            let style = Style::default().fg(th().ui.text).bg(th().ui.panel);
            let dim = Style::default().fg(th().ui.muted).bg(th().ui.panel);
            let x = rect.x + 2;
            let w = rect.width.saturating_sub(4);
            let mut y = rect.y + 1;
            dialog_kit::put(buf, x, y, &press, w, style);
            y += 1;
            let ctx_line = match def.ctx {
                Ctx::GitPrefix | Ctx::NavPrefix => {
                    let prefix = if def.ctx == Ctx::GitPrefix {
                        Bind::GlobalGitPrefix
                    } else {
                        Bind::GlobalNavPrefix
                    };
                    msg::ctx_after_prefix(def.ctx.title().get(), page.km.label(prefix))
                }
                _ => def.ctx.title().get().to_string(),
            };
            dialog_kit::put(buf, x, y, &ctx_line, w, dim);
            y += 2;
            if let Some(text) = cap.typing.as_ref() {
                dialog_kit::put(
                    buf,
                    x,
                    y,
                    &{
                        use crate::keymap::chord::{KeyChord, Mods};
                        let ctrl_shift = Mods {
                            ctrl: true,
                            shift: true,
                            ..Mods::NONE
                        };
                        let alt = Mods {
                            alt: true,
                            ..Mods::NONE
                        };
                        msg::type_key(format!(
                            "{}, {}, ...",
                            KeyChord::new(ctrl_shift, Key::Char('n')),
                            KeyChord::new(alt, Key::F(2))
                        ))
                    },
                    w,
                    style,
                );
                y += 1;
                let field = Rect::new(x, y, w.min(40), 1);
                dialog_kit::fill(
                    buf,
                    field,
                    Style::default().bg(crate::input_bg(true, false)),
                );
                dialog_kit::put(
                    buf,
                    field.x + 1,
                    y,
                    &format!("{text}\u{2588}"),
                    field.width.saturating_sub(1),
                    Style::default()
                        .fg(th().ui.text)
                        .bg(crate::input_bg(true, false)),
                );
                y += 1;
                if let Some(e) = &cap.error {
                    dialog_kit::put(
                        buf,
                        x,
                        y,
                        e,
                        w,
                        Style::default().fg(th().status.danger).bg(th().ui.panel),
                    );
                }
                y += 1;
            } else {
                let shown = match cap.chord {
                    Some(chord) => msg::key_value(chord),
                    None => msg::KEY_WAITING.get().to_string(),
                };
                dialog_kit::put(
                    buf,
                    x,
                    y,
                    &shown,
                    w,
                    Style::default()
                        .fg(th().ui.accent)
                        .bg(th().ui.panel)
                        .add_modifier(Modifier::BOLD),
                );
                y += 2;
            }
            if let Some(refusal) = &cap.analysis.refusal {
                dialog_kit::put(
                    buf,
                    x,
                    y,
                    &msg::refused(refusal),
                    w,
                    Style::default().fg(th().status.danger).bg(th().ui.panel),
                );
                y += 1;
            }
            for (level, message) in cap.analysis.notes.iter().take(4) {
                let s = match level {
                    Level::Error => Style::default().fg(th().status.danger).bg(th().ui.panel),
                    Level::Warning => Style::default().fg(th().status.warning).bg(th().ui.panel),
                    Level::Info => dim,
                };
                let mark = if *level == Level::Info { "i" } else { "!" };
                dialog_kit::put(buf, x, y, &format!("{mark} {message}"), w, s);
                y += 1;
            }
            if let Some(other) = cap.analysis.replace {
                if let Some(d) = ACTIONS.iter().find(|d| d.action == other) {
                    dialog_kit::put(buf, x, y, &msg::replace_unbinds(d.display_label()), w, dim);
                }
            }
            let by = rect.bottom().saturating_sub(2);
            let mut hits = Vec::new();
            let mut bx = x;
            for (i, btn) in cap.buttons().into_iter().enumerate() {
                let over = pointer_in(
                    pointer,
                    Rect::new(bx, by, dialog_kit::button_width(btn.label()), 1),
                );
                let state = if btn == Btn::Save || btn == Btn::Replace {
                    ButtonState::Primary {
                        hovered: over,
                        focused: i == cap.focus,
                    }
                } else {
                    ButtonState::Normal {
                        hovered: over,
                        focused: i == cap.focus,
                    }
                };
                let r = dialog_kit::button(buf, bx, by, btn.label(), state);
                hits.push((r, Hit::Button(btn)));
                bx += r.width + 2;
            }
            let _ = th().ui.disabled;
            hits.push((rect, Hit::Popup));
            page.popup_hits = hits;
        }
    }
}

/// The two popups move like any dialog (their size follows their content).
const WIN_RESET: dialog_kit::Win = dialog_kit::Win::movable("keys_reset");
const WIN_CAPTURE: dialog_kit::Win = dialog_kit::Win::movable("keys_capture");
