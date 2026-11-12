//! A paste that arrives as keystrokes (Windows).
//!
//! Bracketed paste (`term_modes`) is requested on Unix only: the Windows
//! console input path has no paste brackets, so a terminal that does not ask
//! the user first (WezTerm, Rio, Alacritty, Tabby, Warp, mintty; Windows
//! Terminal shows its own multi-line warning) delivers the text as key
//! events. Every newline is then an `Enter` that submits the commit summary,
//! and every letter outside a text field is a shortcut.
//!
//! What still tells a paste from typing is that the keys are all **already
//! queued** when the input drain reads them: a person cannot put three keys,
//! one of them `Enter`, into the queue between two polls. [`coalesce`] takes
//! such a run of already-queued press events and, when it looks like pasted
//! text, returns it as one [`Event::Paste`] (what a bracketed paste would
//! have produced). Anything else is handed back untouched, in order.
//!
//! The decision is pure and tested everywhere; only the drain's use of it is
//! Windows only (`event_loop/window/drain.rs`).

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// A run this long is a paste even without a line break (a long single line
/// into a field; the letters of one fired shortcuts otherwise).
const LONG_RUN: usize = 8;

/// The text one key event adds to a paste; `None` for a key that cannot be
/// part of pasted text (a chord, an arrow, `Esc`...).
fn pasted_char(key: &KeyEvent) -> Option<char> {
    if key
        .modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
    {
        return None;
    }
    match key.code {
        KeyCode::Char(c) => Some(c),
        KeyCode::Enter => Some('\n'),
        KeyCode::Tab => Some('\t'),
        _ => None,
    }
}

/// Whether `event` can start or continue a pasted run: a key press that
/// types text. (Windows also reports every key's release, which carries no
/// information and does not break a run: see [`coalesce`].)
pub(crate) fn is_text_press(event: &Event) -> bool {
    matches!(event, Event::Key(k) if k.kind == KeyEventKind::Press && pasted_char(k).is_some())
}

/// Whether `event` is a key release (ignored by every handler, never part of
/// the text).
pub(crate) fn is_release(event: &Event) -> bool {
    matches!(event, Event::Key(k) if k.kind == KeyEventKind::Release)
}

/// The paste a run of already-queued events stands for, if it is one.
///
/// A paste is: at least three text presses with a line break strictly inside
/// them (`line one`, `Enter`, `line two`), or at least [`LONG_RUN`] text
/// presses of more than one distinct character. A run that is one repeated
/// character is a held key (auto-repeat queued up during a slow frame), and
/// `abc` + `Enter` is someone typing and submitting: neither is a paste.
pub(crate) fn coalesce(run: &[Event]) -> Option<String> {
    let mut text = String::new();
    let mut presses = 0usize;
    for event in run {
        match event {
            Event::Key(k) if k.kind == KeyEventKind::Press => {
                text.push(pasted_char(k)?);
                presses += 1;
            }
            e if is_release(e) => {}
            _ => return None,
        }
    }
    let breaks_inside = text
        .trim_end_matches('\n')
        .trim_start_matches('\n')
        .contains('\n');
    let mut chars = text.chars();
    let first = chars.next()?;
    let distinct = chars.any(|c| c != first);
    let paste = distinct && ((presses >= 3 && breaks_inside) || presses >= LONG_RUN);
    paste.then_some(text)
}

/// The most events one run takes before it is cut (a longer paste arrives as
/// two).
const MAX_RUN: usize = 4096;

/// Reads the rest of a run that starts with `first` from `next` (which
/// returns an event only when one is already queued) and decides what the
/// drain handles now: `(event, after)`, where `after` are the events to
/// handle next, in order. A paste comes back as one [`Event::Paste`]; typing
/// comes back as its first event with the others queued behind it.
pub(crate) fn take(first: Event, next: &mut dyn FnMut() -> Option<Event>) -> (Event, Vec<Event>) {
    if !is_text_press(&first) {
        return (first, Vec::new());
    }
    let mut run = vec![first];
    let mut after = Vec::new();
    while run.len() < MAX_RUN {
        match next() {
            Some(e) if is_text_press(&e) || is_release(&e) => run.push(e),
            Some(e) => {
                after.push(e);
                break;
            }
            None => break,
        }
    }
    if let Some(text) = coalesce(&run) {
        return (Event::Paste(text), after);
    }
    let mut run = run.into_iter();
    let head = run.next().expect("the run holds its first event");
    let mut rest: Vec<Event> = run.collect();
    rest.extend(after);
    (head, rest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    fn release(code: KeyCode) -> Event {
        let mut k = KeyEvent::new(code, KeyModifiers::NONE);
        k.kind = KeyEventKind::Release;
        Event::Key(k)
    }

    /// What Windows reports for typed text: a press and a release per key.
    fn keys(text: &str) -> Vec<Event> {
        text.chars()
            .flat_map(|c| {
                let code = if c == '\n' {
                    KeyCode::Enter
                } else {
                    KeyCode::Char(c)
                };
                [press(code), release(code)]
            })
            .collect()
    }

    #[test]
    fn two_queued_lines_are_one_paste() {
        assert_eq!(
            coalesce(&keys("line one\nline two")).as_deref(),
            Some("line one\nline two")
        );
        // A trailing newline stays in the text.
        assert_eq!(coalesce(&keys("one\ntwo\n")).as_deref(), Some("one\ntwo\n"));
    }

    #[test]
    fn a_long_single_line_is_a_paste() {
        assert_eq!(
            coalesce(&keys("git commit -m")).as_deref(),
            Some("git commit -m")
        );
    }

    #[test]
    fn typing_and_submitting_is_not_a_paste() {
        // `abc` then Enter, all queued during one slow frame.
        assert_eq!(coalesce(&keys("abc\n")), None);
        assert_eq!(coalesce(&keys("ab")), None);
        assert_eq!(coalesce(&keys("\n")), None);
    }

    #[test]
    fn a_held_key_is_not_a_paste() {
        assert_eq!(coalesce(&keys("jjjjjjjjjj")), None);
        assert_eq!(coalesce(&keys("\n\n\n\n\n\n\n\n")), None);
    }

    #[test]
    fn a_chord_or_a_non_text_event_breaks_the_run() {
        let mut run = keys("one\ntwo");
        run.push(Event::Key(KeyEvent::new(
            KeyCode::Char('c'),
            KeyModifiers::CONTROL,
        )));
        assert_eq!(coalesce(&run), None);
        let mut run = keys("one\ntwo");
        run.push(press(KeyCode::Up));
        assert_eq!(coalesce(&run), None);
        let mut run = keys("one\ntwo");
        run.push(Event::FocusLost);
        assert_eq!(coalesce(&run), None);
    }

    #[test]
    fn shifted_letters_and_tabs_are_text() {
        let shifted = Event::Key(KeyEvent::new(KeyCode::Char('A'), KeyModifiers::SHIFT));
        let run = vec![
            shifted,
            press(KeyCode::Tab),
            press(KeyCode::Enter),
            press(KeyCode::Char('b')),
        ];
        assert_eq!(coalesce(&run).as_deref(), Some("A\t\nb"));
    }

    /// A source that hands out `events` one by one, then reports "nothing queued".
    fn queued(events: Vec<Event>) -> impl FnMut() -> Option<Event> {
        let mut it = events.into_iter();
        move || it.next()
    }

    #[test]
    fn take_turns_a_queued_paste_into_one_event_and_keeps_what_follows() {
        let mut all = keys("one\ntwo");
        let first = all.remove(0);
        all.push(press(KeyCode::Esc));
        all.push(press(KeyCode::Char('q')));
        let (now, after) = take(first, &mut queued(all));
        assert_eq!(now, Event::Paste("one\ntwo".into()));
        // Reading stops at the first event that is not text; the rest stays queued.
        assert_eq!(after, vec![press(KeyCode::Esc)]);
    }

    #[test]
    fn take_hands_typing_back_in_order() {
        let mut all = keys("ab\n");
        let first = all.remove(0);
        let (now, after) = take(first, &mut queued(all));
        assert_eq!(now, press(KeyCode::Char('a')));
        let mut expect = keys("ab\n");
        expect.remove(0);
        assert_eq!(after, expect);
        // A key that is not text starts nothing and reads nothing.
        let (now, after) = take(press(KeyCode::Esc), &mut queued(keys("one\ntwo")));
        assert_eq!(now, press(KeyCode::Esc));
        assert!(after.is_empty());
    }

    #[test]
    fn only_text_presses_start_a_run() {
        assert!(is_text_press(&press(KeyCode::Char('a'))));
        assert!(is_text_press(&press(KeyCode::Enter)));
        assert!(!is_text_press(&release(KeyCode::Char('a'))));
        assert!(!is_text_press(&press(KeyCode::Esc)));
        assert!(!is_text_press(&Event::Key(KeyEvent::new(
            KeyCode::Char('p'),
            KeyModifiers::CONTROL
        ))));
    }
}
