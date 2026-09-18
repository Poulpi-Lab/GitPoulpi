//! The jobs panel: one row per job (what it does, where, how far, in which
//! state) and a Cancel button per row. A job that cannot be stopped keeps the
//! button disabled and says why on the line under it.
//!
//! Modal while open, like the Activity Log: `Esc` or a click outside closes
//! it and nothing behind it hovers. Keys: the shared popup list keys, `c` or
//! `Del` cancels the selected job, `Enter` goes to its repository's tab.
//! Mouse: a click selects, a double click goes to the repository, a click on
//! a row's Cancel cancels it, the wheel and the scrollbar scroll. The hit
//! rects come out of the walk that paints (`render`).

use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::Frame;

use gitpoulpi_i18n::msg::jobs as m;
use gitpoulpi_ipc::{JobInfo, JobState};

use super::{all, cancel, cancel_all, cancellable, with, wording, ClickPair, PanelHit};
use crate::dialog_kit::{self, ButtonState};
use crate::{display_width, th};

/// Two clicks on a row this close make a double click.
const DOUBLE_CLICK: Duration = Duration::from_millis(500);
/// Cells of the progress bar.
const BAR: usize = 8;

pub(crate) fn is_open() -> bool {
    with(|c| c.open)
}

pub(super) fn open() {
    crate::scrollbar::set_pointer(None);
    with(|c| {
        c.open = true;
        c.selected = None;
        c.offset = 0;
        c.follow = true;
        c.grab = None;
        c.last_click = None;
        c.panel_notice = None;
    });
    crate::redraw::poke();
}

pub(crate) fn close() {
    with(|c| {
        c.open = false;
        c.grab = None;
        c.pointer = None;
        c.hits.clear();
    });
    crate::redraw::poke();
}

/// The selected job's id, the first job when none is (or the old one is
/// gone).
fn selection(jobs: &[JobInfo]) -> Option<u64> {
    let held = with(|c| c.selected);
    held.filter(|id| jobs.iter().any(|j| j.id == *id))
        .or_else(|| jobs.first().map(|j| j.id))
}

fn go_to(job: &JobInfo) {
    if let Some(repo) = job.repo.clone() {
        with(|c| c.goto = Some(repo));
    }
    close();
}

// ---- keys ----

pub(crate) fn on_key(key: KeyEvent) {
    let jobs = all();
    let selected = selection(&jobs);
    let index = selected.and_then(|id| jobs.iter().position(|j| j.id == id));
    let page = with(|c| usize::from(c.list.height)).max(1);
    if let Some(next) = crate::popup_list::nav(&key, index.unwrap_or(0), jobs.len(), page) {
        with(|c| {
            c.selected = jobs.get(next).map(|j| j.id);
            c.follow = true;
        });
        return;
    }
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => close(),
        KeyCode::Char('c') | KeyCode::Delete if key.modifiers == KeyModifiers::NONE => {
            if let Some(id) = selected {
                cancel(id);
            }
        }
        KeyCode::Char('C') => cancel_all(),
        KeyCode::Enter => {
            if let Some(job) = index.and_then(|i| jobs.get(i)) {
                go_to(job);
            }
        }
        _ => {}
    }
}

// ---- mouse ----

fn bar_region(c: &super::Cell) -> crate::ui_kit::ListBarRegion {
    crate::ui_kit::ListBarRegion {
        bar: crate::ui_kit::ListBar::JobsPanel,
        area: c.body,
        len: c.lines,
        visible: usize::from(c.list.height).max(1),
    }
}

pub(super) fn on_mouse(mouse: MouseEvent) {
    with(|c| c.pointer = Some((mouse.column, mouse.row)));
    let at = (mouse.column, mouse.row).into();
    match mouse.kind {
        MouseEventKind::ScrollUp => with(|c| c.offset = c.offset.saturating_sub(3)),
        MouseEventKind::ScrollDown => with(|c| c.offset = c.offset.saturating_add(3)),
        MouseEventKind::Down(MouseButton::Left) => press(at, mouse),
        MouseEventKind::Drag(MouseButton::Left) => with(|c| {
            if let Some(grab) = c.grab {
                c.offset = crate::ui_kit::list_bar_drag(bar_region(c), c.offset, grab, mouse.row);
            }
        }),
        MouseEventKind::Up(MouseButton::Left) => with(|c| c.grab = None),
        _ => {}
    }
}

fn press(at: ratatui::layout::Position, mouse: MouseEvent) {
    let on_bar = with(|c| {
        let Some(press) =
            crate::ui_kit::list_bar_press(bar_region(c), c.offset, mouse.column, mouse.row)
        else {
            return false;
        };
        if let Some(offset) = press.jump_to {
            c.offset = offset;
        }
        c.grab = (!press.arrow).then_some(press.grab);
        true
    });
    if on_bar {
        return;
    }
    let (close_hit, inside, hit) = with(|c| {
        (
            c.close.contains(at),
            c.window.contains(at),
            c.hits.iter().find(|(r, _)| r.contains(at)).map(|(_, h)| *h),
        )
    });
    if close_hit || !inside {
        close();
        return;
    }
    match hit {
        Some(PanelHit::Cancel(id)) => {
            with(|c| c.selected = Some(id));
            cancel(id);
        }
        Some(PanelHit::CancelAll) => cancel_all(),
        Some(PanelHit::Row(id)) => {
            let now = Instant::now();
            let double = with(|c| {
                let double = c
                    .last_click
                    .is_some_and(|p| p.id == id && now.duration_since(p.at) <= DOUBLE_CLICK);
                c.selected = Some(id);
                c.last_click = (!double).then_some(ClickPair { id, at: now });
                double
            });
            if double {
                if let Some(job) = super::find(id) {
                    go_to(&job);
                }
            }
        }
        None => {}
    }
}

// ---- painting ----

fn bar(progress: Option<u8>) -> String {
    match progress {
        Some(p) => {
            let filled = (usize::from(p.min(100)) * BAR).div_ceil(100);
            format!(
                "{}{} {p:>3}%",
                "\u{2588}".repeat(filled),
                "\u{2591}".repeat(BAR - filled)
            )
        }
        None => " ".repeat(BAR + 5),
    }
}

/// One painted line of the list.
enum Line {
    Job(usize),
    Reason(usize),
}

fn lines(jobs: &[JobInfo]) -> Vec<Line> {
    let mut out = Vec::new();
    for (i, job) in jobs.iter().enumerate() {
        out.push(Line::Job(i));
        if wording::why(job).is_some() {
            out.push(Line::Reason(i));
        }
    }
    out
}

pub(crate) fn render(frame: &mut Frame) {
    if !is_open() {
        return;
    }
    let area = frame.area();
    if area.width < 40 || area.height < 10 {
        return;
    }
    let jobs = all();
    let selected = selection(&jobs);
    let rows = lines(&jobs);
    let want_w = area.width.saturating_sub(4).min(110);
    let want_h = (rows.len() as u16 + 9).clamp(11, area.height.saturating_sub(2).min(30));
    let rect = dialog_kit::centered(area, want_w, want_h);
    let buf = frame.buffer_mut();
    dialog_kit::panel(buf, rect, m::TITLE.get());

    let pointer = with(|c| c.pointer);
    let over = |r: Rect| pointer.is_some_and(|(x, y)| r.contains((x, y).into()));
    let panel_bg = th().ui.panel;

    let close_spot = Rect::new(rect.right().saturating_sub(5), rect.y, 3, 1);
    let close = crate::chrome::close_cross::paint(buf, rect, over(close_spot)).unwrap_or_default();

    let (x0, inner_w) = (rect.x + 2, rect.width.saturating_sub(4));
    let mut hits: Vec<(Rect, PanelHit)> = Vec::new();

    // Toolbar: Cancel all.
    let any = !cancellable().is_empty();
    let all_label = m::BUTTON_CANCEL_ALL.get();
    let all_w = dialog_kit::button_width(all_label);
    let all_rect = Rect::new(x0, rect.y + 2, all_w, 1);
    let state = if any {
        ButtonState::Normal {
            hovered: over(all_rect),
            focused: false,
        }
    } else {
        ButtonState::Disabled
    };
    let drawn = dialog_kit::button(buf, all_rect.x, all_rect.y, all_label, state);
    if any {
        hits.push((drawn, PanelHit::CancelAll));
    }

    // The list sits like the Activity Log's: a blank arrow row above and
    // below, the notice and hint rows under.
    let list = Rect::new(
        x0,
        rect.y + 5,
        inner_w,
        rect.bottom().saturating_sub(3).saturating_sub(rect.y + 5),
    );
    let body = Rect::new(list.x, list.y - 1, list.width, list.height + 2);
    let row_w = list.width.saturating_sub(1);
    let page = usize::from(list.height);

    // Keep the selected job in view when a key moved it.
    let sel_line = rows
        .iter()
        .position(|l| matches!(l, Line::Job(i) if Some(jobs[*i].id) == selected))
        .unwrap_or(0);
    let offset = with(|c| {
        let max = rows.len().saturating_sub(page);
        let mut o = c.offset.min(max);
        if c.follow {
            if sel_line < o {
                o = sel_line;
            } else if sel_line >= o + page {
                o = sel_line + 1 - page;
            }
            c.follow = false;
        }
        c.offset = o;
        o
    });

    if jobs.is_empty() {
        dialog_kit::put(
            buf,
            list.x,
            list.y,
            m::EMPTY.get(),
            row_w,
            Style::default().fg(th().ui.muted).bg(panel_bg),
        );
    }
    for (n, line) in rows.iter().skip(offset).take(page).enumerate() {
        let y = list.y + n as u16;
        match line {
            Line::Job(i) => {
                paint_job(
                    buf,
                    &mut hits,
                    Rect::new(list.x, y, row_w, 1),
                    &jobs[*i],
                    Some(jobs[*i].id) == selected,
                    &over,
                );
            }
            Line::Reason(i) => {
                let reason = wording::why(&jobs[*i]).unwrap_or_default();
                let fill = if Some(jobs[*i].id) == selected {
                    th().ui.selection
                } else {
                    panel_bg
                };
                dialog_kit::fill(
                    buf,
                    Rect::new(list.x, y, row_w, 1),
                    Style::default().bg(fill),
                );
                dialog_kit::put(
                    buf,
                    list.x + 4,
                    y,
                    reason,
                    row_w.saturating_sub(4),
                    Style::default().fg(th().status.warning).bg(fill),
                );
            }
        }
    }

    with(|c| {
        c.window = rect;
        c.close = close;
        c.list = list;
        c.body = body;
        c.lines = rows.len();
        c.hits = hits;
    });
    if let Some(track) = crate::scrollbar::Track::vertical(body, rows.len(), offset) {
        crate::scrollbar::with_pointer(pointer, || {
            crate::scrollbar::render_buf(buf, track, true, false)
        });
    }
    if let Some(notice) = with(|c| c.panel_notice.clone()) {
        dialog_kit::put(
            buf,
            x0,
            rect.bottom().saturating_sub(3),
            &notice,
            inner_w.saturating_sub(2),
            Style::default().fg(th().ui.accent).bg(panel_bg),
        );
    }
    dialog_kit::put(
        buf,
        x0,
        rect.bottom().saturating_sub(2),
        m::HINT.get(),
        inner_w,
        Style::default().fg(th().ui.muted).bg(panel_bg),
    );
}

/// One job's line: `⠋ Push: tool  ████░░░░  42%  Running   [Cancel]`. The
/// button's rect goes first in `hits`, the row's after it (inner first).
fn paint_job(
    buf: &mut Buffer,
    hits: &mut Vec<(Rect, PanelHit)>,
    row: Rect,
    job: &JobInfo,
    selected: bool,
    over: &dyn Fn(Rect) -> bool,
) {
    let bg = dialog_kit::row_bg(selected, over(row));
    dialog_kit::row_band(buf, row, bg);

    // The Cancel button at the right end: enabled while the job can be
    // stopped, drawn disabled while it cannot, absent once it ended.
    let label = m::BUTTON_CANCEL.get();
    let button_w = dialog_kit::button_width(label);
    let mut text_w = row.width;
    if !job.state.is_finished() && row.width > button_w + 20 {
        let r = Rect::new(row.right() - button_w, row.y, button_w, 1);
        let state = if job.is_cancellable() {
            ButtonState::Normal {
                hovered: over(r),
                focused: false,
            }
        } else {
            ButtonState::Disabled
        };
        let drawn = dialog_kit::button(buf, r.x, r.y, label, state);
        // A press on a disabled button says why on the status line.
        hits.push((drawn, PanelHit::Cancel(job.id)));
        text_w = row.width - button_w - 1;
    }
    let glyph = match job.state {
        JobState::Done => "\u{2713}",
        JobState::Failed | JobState::Canceled => "\u{2717}",
        JobState::Queued => "\u{00B7}",
        JobState::Running | JobState::Cancelling => "\u{25CF}",
    };
    let name_w = (usize::from(text_w) * 4 / 10).max(12);
    let name = crate::fit(&wording::label(job), name_w);
    let pad = name_w.saturating_sub(display_width(&name));
    let line = format!(
        "{glyph} {name}{}  {}  {}",
        " ".repeat(pad),
        bar(job.progress),
        wording::state(job.state)
    );
    dialog_kit::put(
        buf,
        row.x,
        row.y,
        &line,
        text_w,
        Style::default()
            .fg(th().ui.text)
            .bg(bg)
            .add_modifier(if job.state.is_finished() {
                Modifier::empty()
            } else {
                Modifier::BOLD
            }),
    );
    hits.push((row, PanelHit::Row(job.id)));
}
