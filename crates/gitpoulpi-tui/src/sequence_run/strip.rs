//! The run strip: one row over the Log's top border, like the banners
//! (`operation_banner.rs`), showing which sequence runs, at which step, and
//! what it waits for. `[Continue]` when a person's go-ahead is what is
//! missing, `[Go there]` when another tab is on screen, `[Details]` after a
//! failure, and the close cross (`chrome::close_cross`) which cancels (or
//! dismisses a failed run).
//!
//! The hit rects come from the paint walk, never from a second layout.

use super::machine::{PauseKind, Run, StepState};
use crate::daemon_link::UserWait;
use crate::operation_banner::{paint_strip, StripButton as Button};
use crate::theme::th;
use gitpoulpi_i18n::msg::sequence_run as m;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::Frame;

/// The strip's buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StripButton {
    Continue,
    GoThere,
    Details,
    /// The close cross: cancel the run, or dismiss a failed one.
    Close,
}

/// Columns the close cross keeps at the strip's right end.
const CROSS_ROOM: u16 = 5;

/// The words of the strip at every length they can be drawn at, longest
/// first.
pub(crate) fn texts(run: &Run) -> Vec<String> {
    let (k, n) = run.position();
    let step = run.step_label();
    let name = &run.name;
    let long = match &run.state {
        StepState::Failed { text, .. } => m::strip_failed(name, k, n, &step, text),
        StepState::Paused(PauseKind::BeforeStep) => m::strip_next(name, k, n, &step),
        StepState::Paused(PauseKind::ReviewMessage) => m::strip_review(name, k, n),
        StepState::Paused(PauseKind::EmptyCommit) => m::strip_commit_message(name, k, n),
        StepState::Paused(PauseKind::ComposeReview | PauseKind::Dialog(_)) => {
            m::strip_dialog(name, k, n)
        }
        StepState::WaitingForUser(UserWait::PushRejected) => {
            m::strip_push_rejected(name, k, n)
        }
        StepState::WaitingForUser(_) => m::strip_user(name, k, n),
        StepState::WaitingForWindow => {
            let tab = run
                .ctx
                .target_root
                .file_name()
                .map_or_else(String::new, |s| s.to_string_lossy().into_owned());
            m::strip_waiting_tab(name, k, n, tab)
        }
        _ => m::strip_running(name, k, n, &step),
    };
    let short = match &run.state {
        StepState::Failed { .. } => m::strip_stopped(name, k, n, &step),
        _ => m::strip_running(name, k, n, &step),
    };
    vec![format!("\u{25b6} {long}"), format!("\u{25b6} {short}")]
}

/// The buttons the state calls for (the close cross is separate).
pub(crate) fn buttons(run: &Run) -> Vec<(StripButton, String)> {
    let mut out = Vec::new();
    if run.can_continue() {
        out.push((StripButton::Continue, m::BUTTON_CONTINUE.get().to_string()));
    }
    match run.state {
        StepState::WaitingForWindow => {
            out.push((StripButton::GoThere, m::BUTTON_GO_THERE.get().to_string()));
        }
        StepState::Failed { .. } => {
            out.push((StripButton::Details, m::BUTTON_DETAILS.get().to_string()));
        }
        _ => {}
    }
    out
}

/// Draws the strip on `area`'s first row; returns each button's rect, the
/// close cross included.
pub(crate) fn render(
    frame: &mut Frame,
    area: Rect,
    run: &Run,
    hovered: Option<StripButton>,
) -> Vec<(Rect, StripButton)> {
    if area.height == 0 || area.width <= CROSS_ROOM {
        return Vec::new();
    }
    let row = Rect::new(area.x, area.y, area.width, 1);
    let failed = matches!(run.state, StepState::Failed { .. });
    let list = buttons(run);
    let drawn: Vec<Button<'_>> = list
        .iter()
        .map(|(b, label)| Button {
            label,
            danger: false,
            hovered: hovered == Some(*b),
            disabled: false,
        })
        .collect();
    let body = Rect::new(row.x, row.y, row.width - CROSS_ROOM, 1);
    let mut hits: Vec<(Rect, StripButton)> =
        paint_strip(frame, body, &texts(run), &drawn)
            .into_iter()
            .map(|(rect, i)| (rect, list[i].0))
            .collect();
    // The cross's room: the strip's own background, red after a failure.
    let bg = if failed {
        th().chrome.banner_danger_bg
    } else {
        th().chrome.banner_bg
    };
    let tail = Rect::new(body.right(), row.y, CROSS_ROOM, 1);
    frame.render_widget(
        ratatui::widgets::Block::default().style(Style::default().bg(bg)),
        tail,
    );
    if failed {
        // The words on red; the buttons keep their own fill.
        let buf = frame.buffer_mut();
        for x in body.x..body.right() {
            if hits.iter().any(|(r, _)| x >= r.x && x < r.right()) {
                continue;
            }
            let cell = &mut buf[(x, row.y)];
            cell.set_bg(th().chrome.banner_danger_bg);
        }
    }
    if let Some(rect) = crate::chrome::close_cross::paint(
        frame.buffer_mut(),
        row,
        hovered == Some(StripButton::Close),
    ) {
        hits.push((rect, StripButton::Close));
    }
    hits
}
