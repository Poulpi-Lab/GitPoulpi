//! Painting the dialog. The hit rects (`Dlg::hits`) come out of the same walk
//! that paints, inner-first: the buttons, then a row's checkbox or combo,
//! then the row band that covers them. The combo popups are painted last.

use gitpoulpi_i18n::msg::push_group as msg;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::Frame;

use super::input::buttons;
use super::model::{
    run_row_text, state_text, Btn, Part, Phase, PlanView, RunState, Scroll, Select, StateTone,
};
use super::{Core, Dlg, Step, WIN};
use crate::dialog_kit::{self, button, button_width, checkbox, put, ButtonState, CheckState};
use crate::scrollbar::{self, Track};
use crate::theme::th;
use crate::ui_kit::{display_width, truncate_with_ellipsis, wrap_words};

const MAX_WIDTH: u16 = 100;
const NAME_MAX: usize = 18;
const GAP: u16 = 2;

fn tone(t: StateTone) -> Color {
    match t {
        StateTone::Good => th().status.success,
        StateTone::Warn => th().status.warning,
        StateTone::Bad => th().status.danger,
        StateTone::Muted => th().ui.muted,
    }
}

impl Core {
    pub(super) fn render(&mut self, frame: &mut Frame) {
        let run = self.run.clone();
        let Some(dlg) = self.dlg.as_mut() else {
            return;
        };
        let area = frame.area();
        let width = MAX_WIDTH.min(area.width.saturating_sub(4));
        if width < 44 || area.height < 14 {
            return;
        }
        let want = want_height(dlg, run.as_ref());
        let height = want.min(area.height.saturating_sub(2));
        let outer = WIN.place(area, dialog_kit::centered(area, width, height));
        dlg.frame = outer;
        dlg.hits.clear();
        let title = title(dlg, run.as_ref());
        let popup_open = matches!(&dlg.step, Step::Plan(p) if p.rows.iter().any(|r| r.combo.is_open()));
        let buf = frame.buffer_mut();
        dlg.close.framed(buf, outer, &title, &WIN, !popup_open);
        let inner = Rect::new(
            outer.x + 2,
            outer.y + 1,
            outer.width.saturating_sub(4),
            outer.height.saturating_sub(2),
        );
        let list = buttons(dlg, run.as_ref());
        let mut hits: Vec<(Rect, Part)> = Vec::new();
        let mut layers: Vec<(Rect, Part)> = Vec::new();
        let hover = dlg.hover;
        let focus = dlg.btn_focus;
        let branch = dlg.branch.clone();
        match &mut dlg.step {
            Step::Select(s) => paint_select(buf, inner, s, hover, &mut layers),
            Step::Plan(p) => paint_plan(buf, inner, p, &branch, hover, &mut layers),
            Step::Run => paint_run(buf, inner, run.as_ref(), &mut layers),
        }
        // Buttons and the key hint on the last two rows.
        paint_buttons(buf, inner, &list, hover, focus, &mut hits);
        let hint = match &dlg.step {
            Step::Select(_) => msg::HINT_SELECT.get(),
            Step::Plan(_) => msg::HINT_PLAN.get(),
            Step::Run => msg::HINT_RUN.get(),
        };
        let hint_y = inner.bottom().saturating_sub(1);
        put(
            buf,
            inner.x,
            hint_y,
            &truncate_with_ellipsis(hint, inner.width as usize),
            inner.width,
            Style::default().fg(th().ui.muted).bg(th().ui.panel),
        );
        // Inner-first: buttons, then the controls, then the row bands (the
        // bands were pushed last into `layers`).
        hits.extend(layers);
        dlg.hits = hits;
        // The combo popups, over everything.
        if let Step::Plan(p) = &mut dlg.step {
            let bounds = frame.area();
            for row in p.rows.iter_mut() {
                row.combo.render_popup(frame.buffer_mut(), bounds);
            }
        }
    }
}

fn title(dlg: &Dlg, run: Option<&RunState>) -> String {
    match &dlg.step {
        Step::Select(_) => msg::title_select(&dlg.branch),
        Step::Plan(p) if p.ready() => msg::title_plan(p.rows.len() as u64, &dlg.branch),
        Step::Plan(_) => msg::title_run(&dlg.branch),
        Step::Run => msg::title_run(run.map_or(dlg.branch.as_str(), |r| r.branch.as_str())),
    }
}

/// The height the step wants (the frame caps it to the screen).
fn want_height(dlg: &Dlg, run: Option<&RunState>) -> u16 {
    // Frame (2) + buttons + hint + blank.
    let chrome = 2 + 3;
    match &dlg.step {
        Step::Select(s) => {
            // hint (2), blank, rows, blank, remember, notice.
            chrome + 2 + 1 + (s.rows.len() as u16).clamp(3, 10) + 1 + 1 + 1
        }
        Step::Plan(p) => {
            // local, blank, header, rows, blank, tags, then, 2 notes.
            let lines: u16 = p.rows.iter().map(|r| r.lines()).sum::<u16>().clamp(3, 14);
            chrome + 1 + 1 + 1 + lines + 1 + 1 + 1 + 2
        }
        Step::Run => {
            let n = run.map_or(1, |r| r.rows.len()) as u16;
            chrome + n.clamp(3, 12) + 3
        }
    }
}

fn panel() -> Style {
    Style::default().fg(th().ui.text).bg(th().ui.panel)
}

fn muted() -> Style {
    Style::default().fg(th().ui.muted).bg(th().ui.panel)
}

fn paint_buttons(
    buf: &mut Buffer,
    inner: Rect,
    list: &[(Btn, String, bool, bool)],
    hover: Option<Part>,
    focus: Option<usize>,
    hits: &mut Vec<(Rect, Part)>,
) {
    let y = inner.bottom().saturating_sub(3);
    let total: u16 = list
        .iter()
        .map(|(_, l, ..)| button_width(l) + 2)
        .sum::<u16>()
        .saturating_sub(2);
    let mut x = inner.right().saturating_sub(total);
    for (i, (btn, label, primary, enabled)) in list.iter().enumerate() {
        let hovered = hover == Some(Part::Button(*btn));
        let focused = focus == Some(i);
        let state = if !enabled {
            ButtonState::Disabled
        } else if *primary {
            ButtonState::Primary { hovered, focused }
        } else {
            ButtonState::Normal { hovered, focused }
        };
        let rect = button(buf, x, y, label, state);
        hits.push((rect, Part::Button(*btn)));
        x += rect.width + 2;
    }
}

/// How many rows starting at `top` fit into `lines` lines.
fn fit_rows(heights: &[u16], top: usize, lines: u16) -> usize {
    let mut used = 0u16;
    let mut n = 0;
    for h in heights.iter().skip(top) {
        if used + h > lines {
            break;
        }
        used += h;
        n += 1;
    }
    n.max(1).min(heights.len().saturating_sub(top).max(1))
}

/// Keeps `cursor` inside the window and returns the first row to paint and
/// how many rows fit.
fn window(heights: &[u16], cursor: usize, scroll: &Scroll, lines: u16) -> (usize, usize) {
    let mut top = scroll.top.get().min(heights.len().saturating_sub(1));
    if cursor < top {
        top = cursor;
    }
    while top < cursor && cursor >= top + fit_rows(heights, top, lines) {
        top += 1;
    }
    scroll.top.set(top);
    let view = fit_rows(heights, top, lines);
    scroll.viewport.set(view);
    (top, view)
}

fn paint_bar(buf: &mut Buffer, scroll: &Scroll, rect: Rect, total: usize, view: usize, top: usize) {
    scroll.bar.set(None);
    if let Some(track) = Track::column(rect, total, view, top) {
        scrollbar::render_buf(buf, track, true, scroll.drag.held());
        scroll.bar.set(Some(track));
    }
}

// ---------------------------------------------------------------------
// Step 1
// ---------------------------------------------------------------------

fn paint_select(
    buf: &mut Buffer,
    inner: Rect,
    s: &mut Select,
    hover: Option<Part>,
    hits: &mut Vec<(Rect, Part)>,
) {
    let mut y = inner.y;
    for line in wrap_words(msg::SELECT_HINT.get(), inner.width as usize)
        .into_iter()
        .take(2)
    {
        put(buf, inner.x, y, &line, inner.width, muted());
        y += 1;
    }
    y += 1;
    let list_bottom = inner.bottom().saturating_sub(3 + 1 + 2);
    let list = Rect::new(inner.x, y, inner.width, list_bottom.saturating_sub(y));
    if !s.ready {
        let (text, color) = match &s.error {
            Some(e) => (e.clone(), th().status.danger),
            None => (msg::LOADING.get().to_string(), th().ui.muted),
        };
        put(
            buf,
            list.x,
            list.y,
            &truncate_with_ellipsis(&text, list.width as usize),
            list.width,
            Style::default().fg(color).bg(th().ui.panel),
        );
    } else {
        let n = s.rows.len();
        let heights = vec![1u16; n];
        let (top, view) = window(&heights, s.cursor, &s.scroll, list.height);
        let bar_w = u16::from(n > view);
        let width = list.width - bar_w;
        let name_w = s
            .rows
            .iter()
            .map(|r| display_width(&r.remote))
            .max()
            .unwrap_or(4)
            .min(NAME_MAX) as u16;
        let mut checks: Vec<(Rect, Part)> = Vec::new();
        let mut bands: Vec<(Rect, Part)> = Vec::new();
        for i in top..(top + view).min(n) {
            let row = &s.rows[i];
            let yy = list.y + (i - top) as u16;
            let idx = i.min(255) as u8;
            let band = Rect::new(list.x, yy, width, 1);
            let selected = i == s.cursor;
            let bg = dialog_kit::row_bg(selected, hover == Some(Part::Row(idx)));
            dialog_kit::row_band(buf, band, bg);
            if selected {
                put(
                    buf,
                    list.x,
                    yy,
                    "\u{258C}",
                    1,
                    Style::default().fg(th().ui.accent).bg(bg),
                );
            }
            let rect = checkbox(
                buf,
                list.x + 2,
                yy,
                name_w + 4,
                &truncate_with_ellipsis(&row.remote, name_w as usize),
                row.checked,
                CheckState::Enabled {
                    hovered: hover == Some(Part::Check(idx)),
                    focused: false,
                },
            );
            checks.push((rect, Part::Check(idx)));
            let dest_x = list.x + 2 + name_w + 4 + GAP;
            if dest_x < list.x + width {
                put(
                    buf,
                    dest_x,
                    yy,
                    &truncate_with_ellipsis(&row.dest, (list.x + width - dest_x) as usize),
                    list.x + width - dest_x,
                    Style::default().fg(th().ui.muted).bg(bg),
                );
            }
            bands.push((band, Part::Row(idx)));
        }
        hits.extend(checks);
        hits.extend(bands);
        if bar_w > 0 {
            paint_bar(
                buf,
                &s.scroll,
                Rect::new(list.right() - 1, list.y, 1, list.height),
                n,
                view,
                top,
            );
        }
    }
    // Remember, and the "tick one" note.
    let ry = inner.bottom().saturating_sub(3 + 1 + 1);
    let rect = checkbox(
        buf,
        inner.x,
        ry,
        inner.width,
        msg::REMEMBER.get(),
        s.remember,
        CheckState::Enabled {
            hovered: hover == Some(Part::Remember),
            focused: false,
        },
    );
    hits.insert(0, (rect, Part::Remember));
    if s.ready && !s.rows.iter().any(|r| r.checked) {
        put(
            buf,
            inner.x,
            ry + 1,
            msg::SELECT_NONE.get(),
            inner.width,
            Style::default().fg(th().status.warning).bg(th().ui.panel),
        );
    }
}

// ---------------------------------------------------------------------
// Step 2
// ---------------------------------------------------------------------

fn paint_plan(
    buf: &mut Buffer,
    inner: Rect,
    p: &mut PlanView,
    branch: &str,
    hover: Option<Part>,
    hits: &mut Vec<(Rect, Part)>,
) {
    let mut y = inner.y;
    if !p.ready() {
        let (text, color) = match &p.error {
            Some(e) => (e.clone(), th().status.danger),
            None => (msg::LOADING.get().to_string(), th().ui.muted),
        };
        put(
            buf,
            inner.x,
            y + 1,
            &truncate_with_ellipsis(&text, inner.width as usize),
            inner.width,
            Style::default().fg(color).bg(th().ui.panel),
        );
        return;
    }
    if let Some(local) = p.local {
        let sha: String = local.to_string().chars().take(7).collect();
        put(
            buf,
            inner.x,
            y,
            &truncate_with_ellipsis(&msg::local_at(branch, sha), inner.width as usize),
            inner.width,
            muted(),
        );
    }
    y += 2;
    // Columns.
    let n = p.rows.len();
    let rw = p
        .rows
        .iter()
        .map(|r| display_width(&r.t.remote))
        .chain([display_width(msg::COL_REMOTE.get())])
        .max()
        .unwrap_or(6)
        .min(NAME_MAX) as u16;
    let states: Vec<String> = p.rows.iter().map(|r| state_text(&r.t).0).collect();
    let sw = states
        .iter()
        .map(|s| display_width(s))
        .chain([display_width(msg::COL_STATE.get())])
        .max()
        .unwrap_or(8)
        .min(24) as u16;
    let x_remote = inner.x + 2;
    let x_state = x_remote + rw + GAP;
    let x_action = x_state + sw + GAP;
    let action_w = (inner.right().saturating_sub(x_action + 1)).min(40);
    put(buf, x_remote, y, msg::COL_REMOTE.get(), rw, muted());
    put(buf, x_state, y, msg::COL_STATE.get(), sw, muted());
    put(buf, x_action, y, msg::COL_ACTION.get(), action_w, muted());
    y += 1;
    // The rows window: the bottom block (blank, tags, then, notes) is fixed.
    let tail = 1 + 1 + 1 + 2;
    let list_bottom = inner.bottom().saturating_sub(3 + tail);
    let lines = list_bottom.saturating_sub(y).max(1);
    let heights: Vec<u16> = p.rows.iter().map(|r| r.lines()).collect();
    let (top, view) = window(&heights, p.cursor, &p.scroll, lines);
    let bar_w = u16::from(n > view);
    let width = inner.width - bar_w;
    let mut controls: Vec<(Rect, Part)> = Vec::new();
    let mut bands: Vec<(Rect, Part)> = Vec::new();
    let mut yy = y;
    for i in top..(top + view).min(n) {
        let idx = i.min(255) as u8;
        let selected = i == p.cursor;
        let row = &mut p.rows[i];
        let h = row.lines();
        let band = Rect::new(inner.x, yy, width, h);
        let bg = dialog_kit::row_bg(selected, hover == Some(Part::Row(idx)));
        dialog_kit::row_band(buf, band, bg);
        if selected {
            for dy in 0..h {
                put(
                    buf,
                    inner.x,
                    yy + dy,
                    "\u{258C}",
                    1,
                    Style::default().fg(th().ui.accent).bg(bg),
                );
            }
        }
        put(
            buf,
            x_remote,
            yy,
            &truncate_with_ellipsis(&row.t.remote, rw as usize),
            rw,
            Style::default().fg(th().ui.text).bg(bg),
        );
        let (state, t) = state_text(&row.t);
        put(
            buf,
            x_state,
            yy,
            &truncate_with_ellipsis(&state, sw as usize),
            sw,
            Style::default().fg(tone(t)).bg(bg),
        );
        if action_w >= 8 && x_action + action_w <= inner.x + width {
            let field = Rect::new(x_action, yy, action_w, 1);
            row.combo
                .render_field(buf, field, selected, hover == Some(Part::Combo(idx)));
            controls.push((field, Part::Combo(idx)));
        }
        if let Some(line) = row.conflict_line() {
            put(
                buf,
                x_remote + 1,
                yy + 1,
                &truncate_with_ellipsis(&line, (inner.x + width).saturating_sub(x_remote + 1) as usize),
                (inner.x + width).saturating_sub(x_remote + 1),
                Style::default().fg(th().status.warning).bg(bg),
            );
        } else if let Some(other) = &row.t.same_url_as {
            let _ = other;
        }
        bands.push((band, Part::Row(idx)));
        yy += h;
    }
    hits.extend(controls);
    hits.extend(bands);
    if bar_w > 0 {
        paint_bar(
            buf,
            &p.scroll,
            Rect::new(inner.right() - 1, y, 1, lines),
            n,
            view,
            top,
        );
    }
    // Tags, the summary and the warnings.
    let mut ty = list_bottom + 1;
    let rect = checkbox(
        buf,
        inner.x,
        ty,
        inner.width,
        msg::TAGS.get(),
        p.tags,
        CheckState::Enabled {
            hovered: hover == Some(Part::Tags),
            focused: false,
        },
    );
    hits.insert(0, (rect, Part::Tags));
    ty += 1;
    put(
        buf,
        inner.x,
        ty,
        &truncate_with_ellipsis(&p.then_line(), inner.width as usize),
        inner.width,
        panel(),
    );
    ty += 1;
    let warn = Style::default().fg(th().status.warning).bg(th().ui.panel);
    for (remote, count) in p.force_rows().into_iter().take(1) {
        put(
            buf,
            inner.x,
            ty,
            &truncate_with_ellipsis(&msg::force_note(u64::from(count), remote), inner.width as usize),
            inner.width,
            warn.add_modifier(Modifier::BOLD),
        );
        ty += 1;
    }
    let rebase = p.rebase_warning();
    if !rebase.is_empty() && ty < inner.bottom().saturating_sub(3) {
        put(
            buf,
            inner.x,
            ty,
            &truncate_with_ellipsis(&msg::rebase_warn(rebase.join(", ")), inner.width as usize),
            inner.width,
            warn,
        );
    }
}

// ---------------------------------------------------------------------
// Step 3
// ---------------------------------------------------------------------

fn paint_run(
    buf: &mut Buffer,
    inner: Rect,
    run: Option<&RunState>,
    hits: &mut Vec<(Rect, Part)>,
) {
    let Some(run) = run else {
        return;
    };
    let rw = run
        .rows
        .iter()
        .map(|r| display_width(&r.remote))
        .max()
        .unwrap_or(6)
        .min(NAME_MAX) as u16;
    let mut y = inner.y;
    for row in run.rows.iter().take(inner.height.saturating_sub(6) as usize) {
        let (glyph, text, t) = run_row_text(row);
        let c = tone(t);
        put(buf, inner.x + 1, y, glyph, 1, Style::default().fg(c).bg(th().ui.panel));
        put(
            buf,
            inner.x + 3,
            y,
            &truncate_with_ellipsis(&row.remote, rw as usize),
            rw,
            panel(),
        );
        let tx = inner.x + 3 + rw + GAP;
        if tx < inner.right() {
            put(
                buf,
                tx,
                y,
                &truncate_with_ellipsis(&text, (inner.right() - tx) as usize),
                inner.right() - tx,
                Style::default().fg(c).bg(th().ui.panel),
            );
        }
        y += 1;
    }
    let (line, color) = match &run.phase {
        Phase::Going => (msg::RUN_GOING.get().to_string(), th().ui.muted),
        Phase::Failed(why) => (why.clone(), th().status.danger),
        Phase::Ended(status) => {
            use gitpoulpi_core::domain::GroupRunStatus as S;
            match status {
                S::Done => (msg::RUN_DONE.get().to_string(), th().status.success),
                _ => (
                    if run.message.is_empty() {
                        msg::RUN_PROBLEMS.get().to_string()
                    } else {
                        run.message.clone()
                    },
                    th().status.warning,
                ),
            }
        }
    };
    let ly = inner.bottom().saturating_sub(5);
    put(
        buf,
        inner.x,
        ly,
        &truncate_with_ellipsis(&line, inner.width as usize),
        inner.width,
        Style::default().fg(color).bg(th().ui.panel),
    );
    let _ = hits;
}
