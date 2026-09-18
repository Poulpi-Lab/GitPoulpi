//! The status line's error. A failed operation (a push, a fetch, a checkout,
//! a clone, a hosting call, an external program, an AI request, a save...)
//! has no framed card: the status line words it in the error style, and a
//! click on it, or the palette's `Activity Log: Open the Last Error`, opens
//! the Activity Log on the entry that holds the details and the remedy
//! buttons.
//!
//! The worker thread (or the code that reported the problem) arms the
//! entry's id with the line to show; the window takes it on its next tick
//! (`take_pending`), whichever tab or view is in front, and the footer paints
//! it. The link's error channel carries the same failure as a raw line:
//! `claim` recognizes it by that text and keeps the short wording, in
//! whichever order the two arrive. Where the footer is not visible (the Home
//! tab, the Radar, the Workspaces, Settings) the draw's last pass paints the
//! line itself on the bottom row (`paint_fallback`).

use std::sync::Mutex;

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::Frame;

use super::feed;

#[derive(Clone)]
struct Armed {
    id: u64,
    /// "Pull failed: The remote could not be reached." (without "error: ").
    reason: String,
    /// The raw line the link's error channel carries for the same failure.
    raw: Option<String>,
}

struct State {
    pending: Vec<Armed>,
    /// Taken by the window already; the channel's raw line may still come.
    recent: Option<Armed>,
    shown: Option<(u64, String)>,
    rect: Rect,
    /// The bottom row of a surface that covers the footer (Home, Settings).
    covered: Option<Rect>,
}

static STATE: Mutex<State> = Mutex::new(State {
    pending: Vec::new(),
    recent: None,
    shown: None,
    rect: Rect::new(0, 0, 0, 0),
    covered: None,
});

fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    f(&mut STATE.lock().unwrap_or_else(|e| e.into_inner()))
}

/// A failure was recorded as entry `id`; `reason` is the one-line wording
/// for the status line ("Pull failed: The remote could not be reached."),
/// the rest stays in the entry. `raw` is the line the link's error channel
/// carries for it, when it has one.
pub(super) fn arm(id: u64, reason: String, raw: Option<String>) {
    with(|s| {
        s.pending.push(Armed { id, reason, raw });
        let extra = s.pending.len().saturating_sub(16);
        s.pending.drain(..extra);
    });
    crate::redraw::poke();
}

fn show(s: &mut State, armed: &Armed) -> String {
    let short = gitpoulpi_i18n::msg::window::error_line(&armed.reason);
    s.shown = Some((armed.id, short.clone()));
    short
}

/// The window's tick: the line of the newest failure nobody showed yet.
pub(crate) fn take_pending() -> Option<String> {
    with(|s| {
        let newest = s.pending.last().cloned()?;
        s.pending.clear();
        let line = show(s, &newest);
        s.recent = Some(newest);
        Some(line)
    })
}

/// The window is about to put the error `line` (from the link's error
/// channel) in the status line. A failure that was reported has its short
/// wording (and the line becomes clickable); any other error keeps `line`.
pub(crate) fn claim(line: String) -> String {
    with(|s| {
        let is_it = |a: &Armed| a.raw.as_deref() == Some(line.as_str());
        if let Some(i) = s.pending.iter().position(is_it) {
            let armed = s.pending.remove(i);
            let short = show(s, &armed);
            s.recent = Some(armed);
            return short;
        }
        if let Some(armed) = s.recent.clone().filter(is_it) {
            return show(s, &armed);
        }
        s.shown = None;
        line
    })
}

/// A new window starts with the failure still unread, if there is one (a
/// tab switch builds a new window: the error must not vanish with it).
pub(crate) fn carry() -> Option<String> {
    with(|s| {
        let (id, text) = s.shown.clone()?;
        feed::is_unread(id).then_some(text)
    })
}

/// Whether `message` is the status line of a failure.
pub(crate) fn is_showing(message: &str) -> bool {
    with(|s| s.shown.as_ref().is_some_and(|(_, t)| t == message))
}

/// A failure's line was recorded (the palette entries' availability).
pub(crate) fn has_error() -> bool {
    with(|s| s.shown.is_some())
}

/// The entry of the failure the status line shows.
pub(crate) fn shown_entry() -> Option<feed::Entry> {
    with(|s| s.shown.as_ref().map(|(id, _)| *id)).and_then(feed::get)
}

/// The draw starts a frame: no rect, no covering surface yet.
pub(crate) fn begin_frame() {
    with(|s| {
        s.rect = Rect::default();
        s.covered = None;
    });
}

/// The footer paints the message at `rect` this frame.
pub(crate) fn note_rect(rect: Rect) {
    with(|s| s.rect = rect);
}

/// A surface covers the footer this frame (Home, Settings): `bar` is its
/// bottom row, where the last pass paints the error.
pub(crate) fn note_covered(bar: Rect) {
    with(|s| {
        s.covered = Some(bar);
        s.rect = Rect::default();
    });
}

pub(crate) fn hit(col: u16, row: u16) -> bool {
    with(|s| s.rect.width > 0 && s.rect.contains((col, row).into()))
}

/// Opens the Activity Log on the entry of the failure.
pub(crate) fn open_entry() {
    if let Some(id) = with(|s| s.shown.as_ref().map(|(id, _)| *id)) {
        super::panel::open_global_on(id);
    }
}

/// The last pass of the draw: where a surface covers the footer, the unread
/// error is painted on that surface's bottom row (left of the Activity
/// button and the jobs indicator), clickable like the footer's.
pub(crate) fn paint_fallback(frame: &mut Frame) {
    let Some((bar, id, line)) = with(|s| {
        let bar = s.covered?;
        let (id, line) = s.shown.clone()?;
        Some((bar, id, line))
    }) else {
        return;
    };
    if !feed::is_unread(id) || bar.width < 20 {
        return;
    }
    let reserve = super::status_button_width() + crate::jobs_ui::width_for() + 2;
    let room = bar.width.saturating_sub(reserve + 2);
    if room < 12 {
        return;
    }
    let buf = frame.buffer_mut();
    let bg = buf[(bar.x + 1, bar.y)].bg;
    let style = Style::default()
        .fg(crate::th().status.danger)
        .bg(bg)
        .add_modifier(Modifier::BOLD);
    let text = format!("\u{2022} {line} ");
    let painted = crate::dialog_kit::put(buf, bar.x + 1, bar.y, &text, room, style);
    with(|s| s.rect = painted);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reset() {
        with(|s| {
            s.pending.clear();
            s.recent = None;
            s.shown = None;
        });
    }

    #[test]
    fn the_channel_line_and_the_tick_agree_whichever_comes_first() {
        let _lock = super::super::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        reset();
        arm(1, "Fetch failed: no".into(), Some("raw one".into()));
        // The channel first.
        let line = claim("raw one".into());
        assert!(line.contains("Fetch failed: no"), "{line}");
        assert!(take_pending().is_none(), "already claimed");
        // The tick first, then the channel.
        arm(2, "Pull failed: no".into(), Some("raw two".into()));
        let ticked = take_pending().expect("pending");
        assert_eq!(claim("raw two".into()), ticked);
        // A line nobody armed stays as it is and is not clickable.
        assert_eq!(claim("something else".into()), "something else");
        assert!(!has_error());
        reset();
    }
}
