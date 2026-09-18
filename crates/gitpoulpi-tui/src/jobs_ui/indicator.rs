//! The jobs indicator at the right of the status bar: hidden with no job,
//! otherwise a spinner, what runs and how far, and a `[x]` that cancels.
//! One job: `⠋ Push: tool 42% [x]`; several: `⠋ Jobs: 3 42% [x]` (the
//! oldest one's progress, and the `[x]` cancels the oldest too). A job that
//! just ended shows `✓` or `✗` for five seconds.
//!
//! The text and the rects come out of one walk (`layout`), so a click lands
//! on what was painted.

use std::time::Instant;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use gitpoulpi_i18n::msg::jobs as m;
use gitpoulpi_ipc::{JobInfo, JobState};

use super::{active, flash, with, wording};
use crate::{display_width, th};

/// The cancel button's glyphs.
const CANCEL: &str = "[x]"; // i18n-english
/// The longest the label part may be, so the breadcrumb keeps its room.
const MAX_LABEL: usize = 34;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum IndicatorHit {
    Label,
    Cancel,
}

/// What the indicator says and how it is drawn, before it has a place.
struct Parts {
    text: String,
    style: Style,
    /// The job `[x]` would cancel; `None` draws no button (a job that
    /// ended).
    target: Option<JobInfo>,
}

fn progress_text(job: &JobInfo) -> String {
    match (job.state, job.progress) {
        (JobState::Queued, _) => format!(" {}", wording::state(JobState::Queued)),
        (JobState::Cancelling, _) => format!(" {}", wording::state(JobState::Cancelling)),
        (_, Some(p)) => format!(" {p}%"),
        (_, None) => job
            .phase
            .as_deref()
            .map(|p| format!(" {p}"))
            .unwrap_or_default(),
    }
}

/// Spinner time base: since the first frame that asked.
fn since() -> std::time::Duration {
    static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    START.get_or_init(Instant::now).elapsed()
}

/// How long a job has run: `12s`, `1m 05s`, `1h 02m`. Empty under a second.
pub(super) fn elapsed_text(run: std::time::Duration) -> String {
    let s = run.as_secs();
    match s {
        0 => String::new(),
        1..=59 => format!("{s}s"),
        60..=3599 => format!("{}m {:02}s", s / 60, s % 60),
        _ => format!("{}h {:02}m", s / 3600, s % 3600 / 60),
    }
}

fn parts(now: Instant) -> Option<Parts> {
    let jobs = active();
    let accent = Style::default()
        .fg(th().ui.accent)
        .add_modifier(Modifier::BOLD);
    if let Some(first) = jobs.first() {
        let spinner = crate::activity::spinner_frame(since());
        let name = if jobs.len() == 1 {
            wording::label(first)
        } else {
            m::jobs_count(jobs.len())
        };
        let base = format!("{spinner} {name}{}", progress_text(first));
        // The time spent keeps its place when a long label is cut.
        let ran = super::first_seen(first.id)
            .map(|at| elapsed_text(now.saturating_duration_since(at)))
            .unwrap_or_default();
        let text = if ran.is_empty() {
            base
        } else {
            let room = MAX_LABEL.saturating_sub(display_width(&ran) + 1);
            format!("{} {ran}", crate::fit(&base, room))
        };
        // `[x]` goes to the oldest job that can be stopped, else the oldest.
        let target = jobs
            .iter()
            .find(|j| j.is_cancellable())
            .unwrap_or(first)
            .clone();
        return Some(Parts {
            text,
            style: accent,
            target: Some(target),
        });
    }
    let ended = flash(now)?;
    let (glyph, color) = match ended.state {
        JobState::Done => ("\u{2713}", th().status.success),
        JobState::Canceled => ("\u{2717}", th().status.warning),
        _ => ("\u{2717}", th().status.danger),
    };
    let tail = match ended.state {
        JobState::Done => String::new(),
        state => format!(" ({})", wording::state(state)),
    };
    // The label gives way, never the verdict: "(Canceled)" is the news.
    let room = MAX_LABEL.saturating_sub(display_width(&tail) + display_width(glyph) + 1);
    Some(Parts {
        text: format!(
            "{glyph} {}{tail}",
            crate::fit(&wording::label(&ended), room)
        ),
        style: Style::default().fg(color).add_modifier(Modifier::BOLD),
        target: None,
    })
}

/// Cells the indicator takes (spaces around included), 0 when hidden. The
/// footer reserves them before it lays the breadcrumb out.
pub(crate) fn width_for() -> u16 {
    let Some(p) = parts(Instant::now()) else {
        return 0;
    };
    let label = display_width(&p.text).min(MAX_LABEL);
    let button = if p.target.is_some() {
        display_width(CANCEL) + 1
    } else {
        0
    };
    (label + button + 2) as u16
}

/// Forgets the rects (the frame shows a prompt instead of the idle line).
pub(crate) fn clear() {
    with(|c| {
        c.ind_label = Rect::default();
        c.ind_cancel = Rect::default();
        c.ind_target = None;
    });
}

/// Paints the indicator so it ends at column `right` of `row` and starts
/// no further left than `min_x` (what is left of it keeps its room), and
/// records its rects. Does nothing (and records nothing) when there is no job
/// or no room.
pub(crate) fn paint(buf: &mut Buffer, row: Rect, right: u16, min_x: u16) {
    clear();
    let pointer = with(|c| c.pointer);
    let Some(p) = parts(Instant::now()) else {
        return;
    };
    let label_w = display_width(&p.text).min(MAX_LABEL);
    let button_w = if p.target.is_some() {
        display_width(CANCEL)
    } else {
        0
    };
    let total = label_w + if button_w > 0 { button_w + 1 } else { 0 } + 2;
    let total = total as u16;
    if right < min_x + total || row.width < total {
        return;
    }
    let x0 = right - total;
    let over = |r: Rect| pointer.is_some_and(|(x, y)| r.contains((x, y).into()));

    let label_rect = Rect::new(x0, row.y, label_w as u16 + 2, 1);
    let mut style = p.style;
    if over(label_rect) {
        style = style.bg(th().ui.button_hover);
    }
    crate::dialog_kit::put(
        buf,
        label_rect.x,
        row.y,
        &format!(" {} ", crate::fit(&p.text, label_w)),
        label_rect.width,
        style,
    );
    let mut cancel_rect = Rect::default();
    if let Some(target) = &p.target {
        cancel_rect = Rect::new(label_rect.right(), row.y, button_w as u16, 1);
        let style = if target.is_cancellable() {
            crate::theme::plain_button_style(over(cancel_rect)).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(th().ui.disabled)
        };
        crate::dialog_kit::put(buf, cancel_rect.x, row.y, CANCEL, cancel_rect.width, style);
    }
    let target = p.target.as_ref().map(|t| t.id);
    with(|c| {
        c.ind_label = label_rect;
        c.ind_cancel = cancel_rect;
        c.ind_target = target;
    });
}

/// What a press at (`col`, `row`) lands on.
pub(super) fn hit(col: u16, row: u16) -> Option<IndicatorHit> {
    with(|c| {
        let at = (col, row).into();
        if c.ind_cancel.width > 0 && c.ind_cancel.contains(at) {
            Some(IndicatorHit::Cancel)
        } else if c.ind_label.width > 0 && c.ind_label.contains(at) {
            Some(IndicatorHit::Label)
        } else {
            None
        }
    })
}
