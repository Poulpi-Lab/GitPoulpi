//! Press-and-hold on a scrollbar end arrow: the state machine behind the
//! auto-repeat. Terminals send one `Down`, then `Drag` only when the pointer
//! changes cell, then `Up`: nothing while the button is held still, so the
//! event loop asks this for the next repeat and registers its deadline
//! (`window/idle.rs`).
//!
//! The first scroll is the press itself. After [`DELAY`] a held press repeats
//! every [`RATE`], the way native scrollbars do; leaving the arrow pauses it
//! and coming back resumes it. The event loop replays the original press for
//! each repeat, so every scrollbar built on the shared component (whatever
//! handles its clicks) repeats without a handler of its own.

use std::time::{Duration, Instant};

/// How long a press is held before it starts repeating.
pub(crate) const DELAY: Duration = Duration::from_millis(380);
/// The gap between two repeats: a gentle steady rate.
pub(crate) const RATE: Duration = Duration::from_millis(55);

#[derive(Debug, Default)]
pub(crate) struct ArrowRepeat {
    held: Option<Held>,
}

#[derive(Debug)]
struct Held {
    cell: (u16, u16),
    next: Instant,
    inside: bool,
}

impl ArrowRepeat {
    /// The button went down on an arrow cell.
    pub(crate) fn press(&mut self, cell: (u16, u16), now: Instant) {
        self.held = Some(Held {
            cell,
            next: now + DELAY,
            inside: true,
        });
    }

    /// The pointer moved with the button still down.
    pub(crate) fn pointer(&mut self, cell: (u16, u16), now: Instant) {
        if let Some(h) = &mut self.held {
            let inside = cell == h.cell;
            if inside && !h.inside {
                h.next = now + RATE;
            }
            h.inside = inside;
        }
    }

    /// Release, Esc/any key, focus loss, a resize: the gesture is over.
    pub(crate) fn stop(&mut self) {
        self.held = None;
    }

    #[cfg(test)]
    pub(crate) fn active(&self) -> bool {
        self.held.is_some()
    }

    /// Whether a repeat is due now; when it is, schedules the next one and
    /// returns the cell to press again. A late wake gives one repeat, never a
    /// burst to catch up.
    pub(crate) fn due(&mut self, now: Instant) -> Option<(u16, u16)> {
        let h = self.held.as_mut()?;
        if !h.inside || now < h.next {
            return None;
        }
        h.next = now + RATE;
        Some(h.cell)
    }

    /// When the loop must wake for the next repeat (`None` when idle or
    /// paused outside the arrow).
    pub(crate) fn next_deadline(&self) -> Option<Instant> {
        self.held.as_ref().filter(|h| h.inside).map(|h| h.next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CELL: (u16, u16) = (5, 9);

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    #[test]
    fn nothing_repeats_before_the_delay_then_one_per_rate() {
        let t0 = Instant::now();
        let mut r = ArrowRepeat::default();
        assert_eq!(r.next_deadline(), None);
        r.press(CELL, t0);
        assert_eq!(r.due(t0 + DELAY - ms(1)), None);
        assert_eq!(r.due(t0 + DELAY), Some(CELL));
        assert_eq!(r.due(t0 + DELAY + RATE - ms(1)), None);
        assert_eq!(r.due(t0 + DELAY + RATE), Some(CELL));
        assert_eq!(r.next_deadline(), Some(t0 + DELAY + RATE + RATE));
    }

    #[test]
    fn a_late_wake_gives_one_repeat_not_a_burst() {
        let t0 = Instant::now();
        let mut r = ArrowRepeat::default();
        r.press(CELL, t0);
        assert_eq!(r.due(t0 + ms(2000)), Some(CELL));
        assert_eq!(r.due(t0 + ms(2000)), None);
    }

    #[test]
    fn leaving_the_arrow_pauses_and_coming_back_resumes() {
        let t0 = Instant::now();
        let mut r = ArrowRepeat::default();
        r.press(CELL, t0);
        r.pointer((6, 9), t0 + ms(100));
        assert_eq!(r.next_deadline(), None);
        assert_eq!(r.due(t0 + ms(900)), None);
        assert!(r.active());
        r.pointer(CELL, t0 + ms(1000));
        assert_eq!(r.due(t0 + ms(1000)), None);
        assert_eq!(r.due(t0 + ms(1000) + RATE), Some(CELL));
    }

    #[test]
    fn release_stops_it() {
        let t0 = Instant::now();
        let mut r = ArrowRepeat::default();
        r.press(CELL, t0);
        r.stop();
        assert!(!r.active());
        assert_eq!(r.due(t0 + ms(5000)), None);
        assert_eq!(r.next_deadline(), None);
    }
}
