//! Painting the Activity Log window and the status bar button. Child of
//! `panel`: it reads the window's private state and writes the rects the
//! mouse handlers use, from the same walk.

use super::*;

// ---- the status bar button ----

fn button_label() -> (String, bool) {
    let unread = feed::unread_errors();
    if unread > 0 {
        (msg::button_unread(unread), true)
    } else {
        (msg::BUTTON.get().to_string(), false)
    }
}

/// Cells the status bar's Activity button takes (with a space either side).
pub(crate) fn status_button_width() -> u16 {
    display_width(&button_label().0) as u16 + 2
}

/// Paints the Activity button at the right end of `footer`, and records its
/// rect (the click target and the hover).
pub(crate) fn paint_status_button(buf: &mut Buffer, footer: Rect) {
    let (label, urgent) = button_label();
    let width = display_width(&label) as u16 + 2;
    if footer.width < width + 40 {
        with(|s| s.status_button = Rect::default());
        return;
    }
    let rect = Rect::new(footer.right() - width, footer.y, width, 1);
    let hovered = with(|s| s.pointer).is_some_and(|(x, y)| rect.contains((x, y).into()));
    let style = if urgent {
        Style::default()
            .fg(th().status.danger)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(th().ui.muted)
    };
    let style = if hovered {
        style.bg(th().ui.button_hover)
    } else {
        style
    };
    dialog_kit::put(buf, rect.x, rect.y, &format!(" {label} "), width, style);
    with(|s| s.status_button = rect);
}

/// Tells the Activity Log whether the window in front is the Home tab, which
/// has no repository: `open` then lands on "All repositories".
pub(crate) fn set_home(home: bool) {
    with(|s| s.home = home);
}

/// Forgets the button (the frame shows a prompt instead of the idle line).
pub(crate) fn clear_status_button() {
    with(|s| s.status_button = Rect::default());
}

/// A click on the status bar's Activity button.
pub(crate) fn status_button_hit(col: u16, row: u16) -> bool {
    with(|s| s.status_button.contains((col, row).into()))
}

/// The pointer, for the status bar button's hover.
pub(crate) fn note_pointer(col: u16, row: u16) {
    with(|s| s.pointer = Some((col, row)));
}

// ---- painting ----

enum Line {
    Header(usize),
    Text(String, Style),
    Buttons(usize),
}

fn severity_color(severity: Severity) -> ratatui::style::Color {
    match severity {
        Severity::Error => th().status.danger,
        Severity::Warning => th().status.warning,
        Severity::Info => th().status.success,
    }
}

fn activity_lines(entries: &[Entry], expanded: &[u64], width: usize) -> Vec<Line> {
    let panel = th().ui.panel;
    let text = Style::default().fg(th().ui.text).bg(panel);
    let muted = Style::default().fg(th().ui.muted).bg(panel);
    let mut out = Vec::new();
    for (i, entry) in entries.iter().enumerate() {
        out.push(Line::Header(i));
        if !expanded.contains(&entry.id) {
            continue;
        }
        let indent = "    ";
        let room = width.saturating_sub(indent.len() + 1);
        if let Some(summary) = &entry.summary {
            for l in crate::ui_kit::wrap_words(summary, room) {
                out.push(Line::Text(format!("{indent}{l}"), text));
            }
        }
        if let Some(hint) = &entry.hint {
            for l in crate::ui_kit::wrap_words(hint, room) {
                out.push(Line::Text(format!("{indent}{l}"), muted));
            }
        }
        let raw: Vec<String> = match &entry.detail {
            Some(d) => d.lines(),
            None => entry.message.lines().map(str::to_string).collect(),
        };
        for l in raw.iter().take(ENTRY_LINES) {
            out.push(Line::Text(
                crate::fit(&format!("{indent}{l}"), width.saturating_sub(1)),
                muted,
            ));
        }
        if raw.len() > ENTRY_LINES {
            out.push(Line::Text(
                format!("{indent}{}", msg::more_lines(raw.len() - ENTRY_LINES)),
                muted,
            ));
        }
        out.push(Line::Buttons(i));
    }
    out
}

/// Paints the window when open.
pub(crate) fn render(frame: &mut Frame) {
    if !is_open() {
        return;
    }
    let area = frame.area();
    if area.width < 40 || area.height < 12 {
        return;
    }
    let want_w = area.width.saturating_sub(4).min(120);
    let want_h = area.height.saturating_sub(2).min(40);
    let rect = dialog_kit::centered(area, want_w, want_h);
    let buf = frame.buffer_mut();
    dialog_kit::panel(buf, rect, msg::TITLE.get());

    let pointer = with(|s| s.pointer);
    // A box's list or the clear confirmation owns the pointer: nothing
    // behind hovers.
    let level_open = with(|s| {
        s.clear_open
            || [&s.level, &s.scope_combo]
                .iter()
                .any(|c| c.as_ref().is_some_and(Combo::is_open))
    });
    let over = |r: Rect| !level_open && pointer.is_some_and(|(x, y)| r.contains((x, y).into()));

    let close_spot = Rect::new(rect.right().saturating_sub(5), rect.y, 3, 1);
    let close = crate::chrome::close_cross::paint(buf, rect, over(close_spot)).unwrap_or_default();

    let mut hits: Vec<(Rect, Hit)> = Vec::new();
    let (x0, inner_w) = (rect.x + 2, rect.width.saturating_sub(4));
    let tab = with(|s| s.tab);

    // Tabs.
    let mut x = x0;
    for (t, label) in [
        (Tab::Git, msg::TAB_GIT.get()),
        (Tab::Application, msg::TAB_APPLICATION.get()),
    ] {
        let text = format!(" {label} ");
        let w = display_width(&text) as u16;
        let r = Rect::new(x, rect.y + 1, w, 1);
        let style = if t == tab {
            Style::default()
                .fg(th().ui.on_accent)
                .bg(th().ui.accent)
                .add_modifier(Modifier::BOLD)
        } else if over(r) {
            crate::theme::plain_button_style(true)
        } else {
            Style::default().fg(th().ui.text).bg(th().ui.panel)
        };
        dialog_kit::put(buf, r.x, r.y, &text, w, style);
        hits.push((r, Hit::Tab(t)));
        x += w + 1;
    }

    // The Scope box, for both tabs.
    paint_scope(buf, x0, rect.y + 2, inner_w, &over);

    // Toolbar.
    let mut y = rect.y + 3;
    let button = |buf: &mut Buffer,
                  hits: &mut Vec<(Rect, Hit)>,
                  x: &mut u16,
                  y: u16,
                  label: &str,
                  hit: Hit| {
        let w = dialog_kit::button_width(label);
        if usize::from(*x + w) > usize::from(x0 + inner_w) {
            return;
        }
        let r = Rect::new(*x, y, w, 1);
        let drawn = dialog_kit::button(
            buf,
            *x,
            y,
            label,
            ButtonState::Normal {
                hovered: over(r),
                focused: false,
            },
        );
        hits.push((drawn, hit));
        *x += w + 1;
    };
    match tab {
        Tab::Git => {
            let mut bx = x0;
            button(
                buf,
                &mut hits,
                &mut bx,
                y,
                msg::COPY_DIAGNOSTICS.get(),
                Hit::CopyDiagnostics,
            );
            y += 2;
        }
        Tab::Application => {
            let label = msg::LEVEL_LABEL.get();
            let muted = Style::default().fg(th().ui.muted).bg(th().ui.panel);
            dialog_kit::put(buf, x0, y, label, inner_w, muted);
            let fx = x0 + display_width(label) as u16 + 1;
            let level_field = Rect::new(fx, y, 16, 1);
            with(|s| s.level_field = level_field);
            let (focus, current) = with(|s| (s.filter_focus, s.view_level));
            let _ = focus;
            with(|s| {
                let combo = s.level.get_or_insert_with(|| level_combo(current));
                combo.select(Some(level_value(current).to_string()));
                combo.render_field(buf, level_field, false, over(level_field));
            });
            let flabel = msg::FILTER_LABEL.get();
            let lx = level_field.right() + 2;
            dialog_kit::put(buf, lx, y, flabel, inner_w, muted);
            let ffx = lx + display_width(flabel) as u16 + 1;
            let fw = (x0 + inner_w).saturating_sub(ffx).min(30);
            let filter_rect = Rect::new(ffx, y, fw, 1);
            let (text, focused) = with(|s| (s.filter.clone(), s.filter_focus));
            paint_filter(buf, filter_rect, &text, focused, over(filter_rect));
            hits.push((filter_rect, Hit::Filter));
            y += 1;
            paint_toggles(buf, &mut hits, x0, y, inner_w, &over);
            y += 1;
            let mut bx = x0;
            button(
                buf,
                &mut hits,
                &mut bx,
                y,
                msg::COPY_ALL.get(),
                Hit::CopyAll,
            );
            button(
                buf,
                &mut hits,
                &mut bx,
                y,
                msg::OPEN_FOLDER.get(),
                Hit::OpenFolder,
            );
            button(
                buf,
                &mut hits,
                &mut bx,
                y,
                msg::COPY_DIAGNOSTICS.get(),
                Hit::CopyDiagnostics,
            );
            button(
                buf,
                &mut hits,
                &mut bx,
                y,
                msg::CLEAR_LOG.get(),
                Hit::ClearLog,
            );
            y += 1;
        }
    }

    // The list sits between the toolbar and the notice and hint rows. The
    // Application tab keeps the row above the notice for its horizontal bar.
    let bar_row = u16::from(tab == Tab::Application);
    let list = Rect::new(
        x0,
        y + 1,
        inner_w,
        (rect.bottom().saturating_sub(3 + bar_row)).saturating_sub(y + 1),
    );
    let body = Rect::new(
        list.x,
        list.y.saturating_sub(1),
        list.width,
        list.height + 2,
    );
    let row_w = usize::from(list.width.saturating_sub(1));
    let page = usize::from(list.height);

    let (lines_len, offset) = match tab {
        Tab::Git => paint_activity(buf, &mut hits, list, row_w, page, &over),
        Tab::Application => paint_app(buf, list, row_w, page),
    };
    with(|s| {
        s.window = rect;
        s.close = close;
        s.list = list;
        s.body = body;
        s.lines = lines_len;
        s.hits = hits;
        s.hbody = body;
    });
    if let Some(track) = crate::scrollbar::Track::vertical(body, lines_len, offset) {
        crate::scrollbar::with_pointer(if level_open { None } else { pointer }, || {
            crate::scrollbar::render_buf(buf, track, true, false)
        });
    }
    // The Application lines are never cut: a bar along the bottom scrolls them.
    if let Some(track) = with(|s| hbar(s)) {
        crate::scrollbar::with_pointer(if level_open { None } else { pointer }, || {
            crate::scrollbar::render_buf(buf, track, false, false)
        });
    }

    // Notice and hint rows. Hovering a grayed level box says why it is gray.
    let notice = with(|s| s.notice.clone()).or_else(|| {
        if tab != Tab::Application || level_open {
            return None;
        }
        let why = why_hovered(pointer);
        if why.is_some() {
            return why;
        }
        Some(msg::HINT_LEVELS.get().to_string())
    });
    if let Some(notice) = notice {
        dialog_kit::put(
            buf,
            x0,
            rect.bottom().saturating_sub(3),
            &notice,
            inner_w,
            Style::default().fg(th().ui.accent).bg(th().ui.panel),
        );
    }
    let hint = match tab {
        Tab::Git => msg::HINT_GIT.get(),
        Tab::Application => msg::HINT_APPLICATION.get(),
    };
    dialog_kit::put(
        buf,
        x0,
        rect.bottom().saturating_sub(2),
        hint,
        inner_w,
        Style::default().fg(th().ui.muted).bg(th().ui.panel),
    );

    // The boxes' lists last, over everything.
    with(|s| {
        for combo in [&mut s.level, &mut s.scope_combo].into_iter().flatten() {
            combo.render_popup(buf, buf.area);
        }
    });
    if with(|s| s.clear_open) {
        let popup = paint_clear_confirm(buf, rect);
        // Popup rects first: `press` takes the first match.
        with(|s| {
            s.hits.splice(0..0, popup);
        });
    }
    if tab == Tab::Application {
        start_refresher();
    }
}

/// The clear confirmation, centred over the window: what goes, that it
/// cannot be undone, and Cancel (the default) / Clear. Returns its hits.
fn paint_clear_confirm(buf: &mut Buffer, window: Rect) -> Vec<(Rect, Hit)> {
    use ratatui::widgets::{Paragraph, Widget, Wrap};
    let w = window.width.saturating_sub(8).clamp(20, 64);
    let rect = dialog_kit::centered(window, w, 9);
    dialog_kit::fill(buf, rect, Style::default().bg(th().ui.panel));
    dialog_kit::panel(buf, rect, msg::CLEAR_TITLE.get());
    let inner_w = rect.width.saturating_sub(4);
    Paragraph::new(msg::CLEAR_QUESTION.get())
        .style(Style::default().fg(th().ui.text).bg(th().ui.panel))
        .wrap(Wrap { trim: true })
        .render(Rect::new(rect.x + 2, rect.y + 2, inner_w, 3), buf);
    let (selected, pointer) = with(|s| (s.clear_sel, s.pointer));
    let y = rect.y + 5;
    let mut x = rect.x + 2;
    let mut hits = Vec::new();
    for (i, (label, hit)) in [
        (msg::CLEAR_CANCEL.get(), Hit::ClearCancel),
        (msg::CLEAR_CONFIRM.get(), Hit::ClearConfirm),
    ]
    .into_iter()
    .enumerate()
    {
        let bw = dialog_kit::button_width(label);
        let r = Rect::new(x, y, bw, 1);
        let hovered = pointer.is_some_and(|(c, row)| r.contains((c, row).into()));
        let drawn = dialog_kit::button(
            buf,
            x,
            y,
            label,
            ButtonState::Normal {
                hovered,
                focused: selected == i,
            },
        );
        hits.push((drawn, hit));
        x += bw + 1;
    }
    dialog_kit::put(
        buf,
        rect.x + 2,
        rect.y + 7,
        msg::CLEAR_HINT.get(),
        inner_w,
        Style::default().fg(th().ui.muted).bg(th().ui.panel),
    );
    hits
}

/// The display toggles row: `Show: [x] Error 2  [x] Warning 0 ... [All] [None]`.
/// Levels above the capture level are grayed (and say why on hover).
fn paint_toggles(
    buf: &mut Buffer,
    hits: &mut Vec<(Rect, Hit)>,
    x0: u16,
    y: u16,
    inner_w: u16,
    over: &dyn Fn(Rect) -> bool,
) {
    let end = x0 + inner_w;
    let muted = Style::default().fg(th().ui.muted).bg(th().ui.panel);
    let label = msg::SHOW_LABEL.get();
    dialog_kit::put(buf, x0, y, label, inner_w, muted);
    let mut x = x0 + display_width(label) as u16 + 1;
    let (view, shown, filter) = with(|s| (s.view_level, s.shown, s.filter.clone()));
    let counts = level_counts(&scoping(), view, &filter);
    for level in Level::ALL {
        let enabled = level <= view;
        let text = if enabled {
            format!("{} {}", level_label(level), counts[level as usize])
        } else {
            level_label(level).to_string()
        };
        let w = display_width(&text) as u16 + 4;
        if x + w > end {
            break;
        }
        let probe = Rect::new(x, y, w, 1);
        let state = if enabled {
            dialog_kit::CheckState::Enabled {
                hovered: over(probe),
                focused: false,
            }
        } else {
            dialog_kit::CheckState::Disabled
        };
        let drawn =
            dialog_kit::checkbox(buf, x, y, w, &text, enabled && shown[level as usize], state);
        hits.push((drawn, Hit::Toggle(level)));
        x += w + 2;
    }
    for (label, hit) in [
        (msg::SHOW_ALL.get(), Hit::ShowAll),
        (msg::SHOW_NONE.get(), Hit::ShowNone),
    ] {
        let w = dialog_kit::button_width(label);
        if x + w > end {
            break;
        }
        let r = Rect::new(x, y, w, 1);
        let drawn = dialog_kit::button(
            buf,
            x,
            y,
            label,
            ButtonState::Normal {
                hovered: over(r),
                focused: false,
            },
        );
        hits.push((drawn, hit));
        x += w + 1;
    }
}

/// Why the grayed level box under the pointer is gray, when it is.
fn why_hovered(pointer: Option<(u16, u16)>) -> Option<String> {
    let (x, y) = pointer?;
    with(|s| {
        s.hits
            .iter()
            .find(|(r, _)| r.contains((x, y).into()))
            .and_then(|(_, h)| match h {
                Hit::Toggle(l) if *l > s.view_level => {
                    Some(msg::why_level_off(level_label(s.view_level)))
                }
                _ => None,
            })
    })
}

/// The Scope box: `Scope: [This repository (name) v]`.
fn paint_scope(buf: &mut Buffer, x0: u16, y: u16, inner_w: u16, over: &dyn Fn(Rect) -> bool) {
    let muted = Style::default().fg(th().ui.muted).bg(th().ui.panel);
    let end = x0 + inner_w;
    let label = msg::SCOPE_LABEL.get();
    dialog_kit::put(buf, x0, y, label, inner_w, muted);
    let fx = x0 + display_width(label) as u16 + 1;
    let width = 44.min(end.saturating_sub(fx));
    if width < 12 {
        // Too narrow: the box is neither drawn nor reachable.
        with(|s| s.scope_field = Rect::default());
        return;
    }
    let field = Rect::new(fx, y, width, 1);
    with(|s| {
        s.scope_field = field;
        // An open list keeps its rows; a closed one follows the scope.
        if !s.scope_combo.as_ref().is_some_and(Combo::is_open) {
            s.scope_combo = None;
        }
    });
    let mut combo = with(|s| s.scope_combo.take()).unwrap_or_else(scope_combo_closed);
    combo.render_field(buf, field, false, over(field));
    with(|s| s.scope_combo = Some(combo));
}

fn paint_filter(buf: &mut Buffer, rect: Rect, text: &str, focused: bool, hovered: bool) {
    let bg = crate::input_bg(focused, hovered);
    dialog_kit::fill(buf, rect, Style::default().bg(bg));
    if text.is_empty() && !focused {
        dialog_kit::put(
            buf,
            rect.x + 1,
            rect.y,
            msg::FILTER_PLACEHOLDER.get(),
            rect.width.saturating_sub(1),
            Style::default().fg(th().ui.muted).bg(bg),
        );
        return;
    }
    let shown = if focused {
        format!("{text}\u{2588}")
    } else {
        text.to_string()
    };
    // Keep the end of a long query in view.
    let room = usize::from(rect.width.saturating_sub(1));
    let shown = if display_width(&shown) > room {
        crate::ui_kit::tail_fit(&shown, room)
    } else {
        shown
    };
    dialog_kit::put(
        buf,
        rect.x + 1,
        rect.y,
        &shown,
        rect.width.saturating_sub(1),
        Style::default().fg(th().ui.text).bg(bg),
    );
}

fn paint_activity(
    buf: &mut Buffer,
    hits: &mut Vec<(Rect, Hit)>,
    list: Rect,
    row_w: usize,
    page: usize,
    over: &dyn Fn(Rect) -> bool,
) -> (usize, usize) {
    let entries = entries();
    let sc = scoping();
    let slot = tab_index(Tab::Git);
    // New errors while it is open are looked at.
    feed::mark_all_read();
    let panel = th().ui.panel;
    if entries.is_empty() {
        let lines = crate::ui_kit::wrap_words(msg::GIT_EMPTY.get(), row_w);
        for (i, l) in lines.iter().take(page).enumerate() {
            dialog_kit::put(
                buf,
                list.x,
                list.y + i as u16,
                l,
                row_w as u16,
                Style::default().fg(th().ui.muted).bg(panel),
            );
        }
        return (0, 0);
    }
    let (expanded, selected) = with(|s| (s.expanded.clone(), s.selected));
    let selected = selected.min(entries.len() - 1);
    let lines = activity_lines(&entries, &expanded, row_w);
    // Keep the selection's header in view when a key moved it.
    let header_at = lines
        .iter()
        .position(|l| matches!(l, Line::Header(i) if *i == selected))
        .unwrap_or(0);
    let offset = with(|s| {
        s.selected = selected;
        let max = lines.len().saturating_sub(page);
        let mut o = s.offset[slot].min(max);
        if s.follow {
            if header_at < o {
                o = header_at;
            } else if header_at >= o + page {
                o = header_at + 1 - page;
            }
            s.follow = false;
        }
        s.offset[slot] = o;
        o
    });
    for (row, line) in lines.iter().skip(offset).take(page).enumerate() {
        let y = list.y + row as u16;
        let width = row_w as u16;
        match line {
            Line::Header(i) => {
                let entry = &entries[*i];
                let is_open = expanded.contains(&entry.id);
                let r = Rect::new(list.x, y, width, 1);
                let bg = if *i == selected {
                    th().ui.selection
                } else if over(r) {
                    th().ui.hover
                } else {
                    panel
                };
                dialog_kit::fill(buf, r, Style::default().bg(bg));
                let marker = crate::fold_mark::glyph_open(is_open);
                let clock = gitpoulpi_log::clock(entry.ts_ms);
                let tail = entry
                    .summary
                    .clone()
                    .unwrap_or_else(|| entry.message.lines().next().unwrap_or("").to_string());
                // "Push - 123 ms - The remote could not be reached."
                let mut rest = String::new();
                if let Some(ms) = entry
                    .duration_ms
                    .filter(|_| crate::activity_log::timing::timings_on())
                {
                    rest.push_str(&format!(" - {}", msg::duration(ms)));
                }
                if !tail.is_empty() && tail != entry.title && entry.list_title.is_none() {
                    rest.push_str(&format!(" - {tail}"));
                }
                // Rows of several repositories say whose each one is; a row of
                // none, under a narrow scope, says it is app-wide.
                if sc.judge(entry.repo.as_deref(), entry.workspace.as_deref()) == Belongs::AppWide {
                    rest.push_str(&format!(" [{}]", msg::APP_WIDE.get()));
                } else if sc.many() {
                    if let Some(name) = entry
                        .repo
                        .as_deref()
                        .and_then(std::path::Path::file_name)
                        .map(|n| n.to_string_lossy().into_owned())
                        .or_else(|| entry.workspace.clone())
                    {
                        rest.push_str(&format!(" [{name}]"));
                    }
                }
                let head = format!("{marker} {clock}  ");
                let x = dialog_kit::put(
                    buf,
                    list.x,
                    y,
                    &head,
                    width,
                    Style::default().fg(th().ui.muted).bg(bg),
                );
                let glyph = dialog_kit::put(
                    buf,
                    x.right(),
                    y,
                    "\u{25CF} ",
                    width.saturating_sub(x.width),
                    Style::default().fg(severity_color(entry.severity)).bg(bg),
                );
                let used = x.width + glyph.width;
                let title = dialog_kit::put(
                    buf,
                    glyph.right(),
                    y,
                    entry.list_title.as_deref().unwrap_or(&entry.title),
                    width.saturating_sub(used),
                    Style::default()
                        .fg(th().ui.text)
                        .bg(bg)
                        .add_modifier(Modifier::BOLD),
                );
                let used = used + title.width;
                dialog_kit::put(
                    buf,
                    title.right(),
                    y,
                    &rest,
                    width.saturating_sub(used),
                    Style::default().fg(th().ui.muted).bg(bg),
                );
                hits.push((r, Hit::Header(*i)));
            }
            Line::Text(text, style) => {
                dialog_kit::put(buf, list.x, y, text, width, *style);
            }
            Line::Buttons(i) => {
                let label = msg::COPY.get();
                let r = Rect::new(list.x + 4, y, dialog_kit::button_width(label), 1);
                let drawn = dialog_kit::button(
                    buf,
                    r.x,
                    r.y,
                    label,
                    ButtonState::Normal {
                        hovered: over(r),
                        focused: false,
                    },
                );
                hits.push((drawn, Hit::CopyEntry(*i)));
                // The remedy next to it, when the failure has one.
                if let Some(remedy) = super::remedy_of(&entries[*i]) {
                    let label = remedy.label();
                    let r = Rect::new(drawn.right() + 1, y, dialog_kit::button_width(&label), 1);
                    let drawn = dialog_kit::button(
                        buf,
                        r.x,
                        r.y,
                        &label,
                        ButtonState::Normal {
                            hovered: over(r),
                            focused: false,
                        },
                    );
                    hits.push((drawn, Hit::Remedy(*i)));
                }
            }
        }
    }
    (lines.len(), offset)
}

/// `text` from display column `skip` on, at most `width` columns wide
/// (whole characters; a wide one cut by the left edge becomes a space).
fn window_cells(text: &str, skip: usize, width: usize) -> String {
    let mut out = String::new();
    let (mut col, mut used) = (0usize, 0usize);
    for c in text.chars() {
        let c = if c.is_control() { ' ' } else { c };
        let w = display_width(c.encode_utf8(&mut [0; 4]));
        if col + w <= skip {
            col += w;
            continue;
        }
        if col < skip {
            out.push(' ');
            used += col + w - skip;
            col += w;
            continue;
        }
        if used + w > width {
            break;
        }
        out.push(c);
        used += w;
        col += w;
    }
    out
}

fn paint_app(buf: &mut Buffer, list: Rect, row_w: usize, page: usize) -> (usize, usize) {
    let (level, shown, filter) = with(|s| (s.view_level, s.shown, s.filter.clone()));
    let sc = scoping();
    let records = visible_records(&sc, level, &shown, &filter);
    let panel = th().ui.panel;
    let mut lines: Vec<(String, Style)> = Vec::new();
    for r in &records {
        let style = Style::default()
            .fg(match r.level {
                Level::Error => th().status.danger,
                Level::Warn => th().status.warning,
                Level::Info => th().ui.text,
                Level::Debug | Level::Trace => th().ui.muted,
            })
            .bg(panel);
        let mut parts = r.message.lines();
        let first = parts.next().unwrap_or("");
        // Whose line it is, when the scope mixes: the repository's name, or
        // app-wide for a line of none under a narrow scope.
        let owner = match r.repo.as_deref() {
            Some(repo) if sc.many() => format!("[{}] ", short_name(std::path::Path::new(repo))),
            None if sc.scope != Scope::All => format!("[{}] ", msg::APP_WIDE.get()),
            _ => String::new(),
        };
        lines.push((
            format!(
                "{} {:<5} {:<6} {}: {owner}{}",
                r.clock(),
                r.level.as_str(),
                r.process,
                r.target,
                first
            ),
            style,
        ));
        for more in parts {
            lines.push((format!("    {more}"), style));
        }
    }
    if lines.is_empty() {
        with(|s| {
            s.longest = 0;
            s.hoffset = 0;
        });
        for (i, l) in crate::ui_kit::wrap_words(msg::APP_EMPTY.get(), row_w)
            .iter()
            .take(page)
            .enumerate()
        {
            dialog_kit::put(
                buf,
                list.x,
                list.y + i as u16,
                l,
                row_w as u16,
                Style::default().fg(th().ui.muted).bg(panel),
            );
        }
        return (0, 0);
    }
    let longest = lines
        .iter()
        .map(|(text, _)| display_width(text))
        .max()
        .unwrap_or(0);
    let (offset, hoffset) = with(|s| {
        let max = lines.len().saturating_sub(page);
        s.offset[1] = s.offset[1].min(max);
        s.longest = longest;
        s.hoffset = s.hoffset.min(longest.saturating_sub(row_w));
        (s.offset[1], s.hoffset)
    });
    for (row, (text, style)) in lines.iter().skip(offset).take(page).enumerate() {
        dialog_kit::put(
            buf,
            list.x,
            list.y + row as u16,
            &window_cells(text, hoffset, row_w),
            row_w as u16,
            *style,
        );
    }
    (lines.len(), offset)
}
