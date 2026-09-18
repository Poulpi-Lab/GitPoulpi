//! Redraw on demand: the loop draws when something may have changed, and
//! otherwise blocks on terminal input until the next registered deadline.
//!
//! A frame is due when
//!
//! * an input event was handled last iteration ([`RedrawState::input_handled`],
//!   set by the drain for every event, handled or not);
//! * something animates ([`WindowState::animating`]): then the loop runs at
//!   the frame rate;
//! * a worker handed the UI thread something ([`crate::redraw::poke`]; every
//!   channel the loop drains is fed through `NotifySend::send_notify`);
//! * a deadline passed ([`WindowState::next_deadline`]).
//!
//! **Adding a time-driven visual means adding its deadline to
//! `next_deadline` (or to `animating` when it moves every frame).** Anything
//! that changes state on the UI thread from a worker must `poke`.
//!
//! An iteration entered by a wake runs one more frame after it: the outcomes
//! applied at its end are drawn by that one, never left for the next event.

use std::time::{Duration, Instant};

use crate::event_loop::*;
use init::WindowState;

/// One frame at the animation rate (spinner pacing).
pub(in crate::event_loop) const FRAME: Duration = Duration::from_millis(33);

/// How often an idle loop looks at the poke counter. crossterm cannot wake
/// its own `poll` from another thread, so this bounds the latency of a
/// worker's answer; one atomic load per slice costs nothing.
const WAKE_SLICE: Duration = Duration::from_millis(20);

/// While a status message outlives its minimum age, how often the loop looks
/// for the refresh that ends it.
const MESSAGE_RECHECK: Duration = Duration::from_millis(500);

/// An overdue deadline that no frame resolves (a bug, not a feature) wakes
/// the loop at most this often instead of spinning it.
const OVERDUE_GAP: Duration = Duration::from_millis(100);

pub(in crate::event_loop) struct RedrawState {
    /// The next iteration draws without waiting.
    needs_redraw: bool,
    /// The poke counter when the running iteration began.
    seen_epoch: u64,
    /// The running iteration is a news iteration (see `begin_iteration`).
    woke: bool,
    /// The wake was an input event: the drain takes it before any frame is
    /// drawn, so the key is handled now and its result is what the next
    /// frame shows (not one stale frame in between).
    input_wake: bool,
    /// Set by the input drain: a terminal event was read this iteration.
    pub(in crate::event_loop) input_handled: bool,
    last_timer_wake: Instant,
}

impl Default for RedrawState {
    fn default() -> Self {
        Self {
            needs_redraw: true,
            seen_epoch: crate::redraw::epoch(),
            woke: false,
            input_wake: false,
            input_handled: false,
            last_timer_wake: Instant::now(),
        }
    }
}

impl WindowState {
    /// Something on screen changes by itself every frame (a spinner, a
    /// progress counter): the loop keeps the frame rate while this holds.
    pub(in crate::event_loop) fn animating(&self) -> bool {
        self.still_loading
            || self.link.in_flight_spinning()
            || self.hosting.animating()
            || self
                .management
                .as_ref()
                .is_some_and(|m| m.radar.animating())
            || self.repo_dialogs.animating()
            || self.ai_ui.running()
            || crate::about::animating()
            || crate::image_view::pane::animating()
            || crate::image_view::split::animating()
            || crate::open_with::animating()
            || self.merge.as_ref().is_some_and(|m| m.highlight_pending())
            || self.editor.as_ref().is_some_and(|e| e.highlight_pending())
            || self.reveal.pending()
            || matches!(&self.input_mode, InputMode::WorkspaceSearch(s) if s.animating())
            || crate::blame_view::waiting()
            || self
                .file_insight
                .as_ref()
                .is_some_and(|f| f.blame_highlight_pending())
            || self.merge_flow.loading()
            || self.preferences.as_ref().is_some_and(|p| p.animating())
    }

    /// The earliest moment a time-driven change becomes visible, when
    /// nothing animates: the in-flight text appearing, the tooltip dwell
    /// ending, a toast expiring, the auto-fetch and hosting timers.
    fn next_deadline(&self, now: Instant) -> Option<Instant> {
        // A status message is ended by the first refresh that lands after
        // `MESSAGE_MIN_LIFETIME`, and a refresh that read nothing new is
        // delivered without a wake: look in on the loop until it is gone.
        let message = self.last_message.is_some().then(|| {
            let due = self.message_set_at + super::apply::MESSAGE_MIN_LIFETIME;
            if due > now {
                due
            } else {
                now + MESSAGE_RECHECK
            }
        });
        [
            message,
            self.link.in_flight_text_due_at(),
            self.tooltip.next_deadline(now),
            self.ui_viewer.next_deadline(),
            self.next_fetch_deadline(),
            self.hosting.next_due(),
            self.arrow_repeat.next_deadline(),
            self.reveal.next_deadline(),
            match &self.input_mode {
                InputMode::WorkspaceSearch(s) => s.next_deadline(),
                _ => None,
            },
            match &self.input_mode {
                InputMode::Rewrite(v) => v.next_deadline(),
                _ => None,
            },
            self.history.as_ref().and_then(|h| h.next_deadline(now)),
            self.file_insight
                .as_ref()
                .and_then(|f| f.blame_hover_deadline(now)),
            crate::launcher_sync::next_deadline(),
            crate::diff_recode::next_deadline(),
            crate::workspace_ui::watch::next_deadline(),
            self.management
                .as_ref()
                .and_then(|m| m.radar.next_deadline()),
            crate::jobs_ui::next_deadline(),
            self.repo_dialogs.next_deadline(),
        ]
        .into_iter()
        .flatten()
        .min()
    }

    /// Blocks until a frame is due: returns at once when one is already
    /// pending, else waits for input, a poke or a deadline.
    pub(in crate::event_loop) fn wait_for_work(&mut self) -> io::Result<()> {
        if std::mem::take(&mut self.redraw.needs_redraw) {
            self.begin_iteration(false);
            return Ok(());
        }
        // Read once: a deadline that has passed no longer reports itself (the
        // in-flight text is "due" only until it shows), and nothing but a
        // poke or an input event can move one.
        let mut input_wake = false;
        let due = self
            .next_deadline(Instant::now())
            .map(|due| due.max(self.redraw.last_timer_wake + OVERDUE_GAP));
        loop {
            if crate::redraw::epoch() != self.redraw.seen_epoch {
                break;
            }
            let now = Instant::now();
            let mut slice = WAKE_SLICE;
            if let Some(due) = due {
                if due <= now {
                    self.redraw.last_timer_wake = now;
                    break;
                }
                slice = slice.min(due - now);
            }
            match event::poll(slice) {
                Ok(true) => {
                    input_wake = true;
                    break;
                }
                Ok(false) => {}
                Err(err) => {
                    exit_log::tolerate_input_error(err, &mut self.input_errors)
                        .map_err(exit_log::io_context("waiting for terminal input"))?;
                    break;
                }
            }
        }
        self.begin_iteration(true);
        self.redraw.input_wake = input_wake;
        Ok(())
    }

    /// An iteration is about to run. It is a *news* iteration when it was
    /// woken, or when anything was poked since the last one began: such an
    /// iteration may consume worker results after its own draw (the
    /// outcomes `apply_outcomes` takes), so it is always followed by a
    /// frame that shows them.
    fn begin_iteration(&mut self, waited: bool) {
        let epoch = crate::redraw::epoch();
        self.redraw.input_wake = false;
        self.redraw.woke = waited || epoch != self.redraw.seen_epoch;
        self.redraw.seen_epoch = epoch;
    }

    /// Whether this iteration draws before it drains input.
    pub(in crate::event_loop) fn draws_before_input(&self) -> bool {
        !self.redraw.input_wake
    }

    /// Decides whether the next iteration draws without waiting.
    pub(in crate::event_loop) fn end_iteration(&mut self) {
        let animating = self.animating();
        let redraw = &mut self.redraw;
        redraw.needs_redraw = std::mem::take(&mut redraw.input_handled)
            || redraw.woke
            || crate::redraw::epoch() != redraw.seen_epoch
            || animating;
    }
}
