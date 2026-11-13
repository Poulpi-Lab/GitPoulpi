//! "Open this link?": the question asked before a web address that was typed
//! in a commit message opens in the browser.
//!
//! The text of a commit message is untrusted (a bot or a stranger wrote it),
//! so a click on an address of the Commit Details pane, or the palette's
//! `Commit Details: Open Link in Message...`, never launches anything by
//! itself: this dialog shows the whole address (broken over lines, never cut)
//! and offers **Open** (`O`, Enter on the focused button), **Next** (`N`, when
//! the message has several addresses; it cycles) and **Cancel** (`C`, `Esc`,
//! the cross, a click outside). Only `http` and `https` addresses get here.
//!
//! State is process-wide like the other question dialogs: the window restarts
//! on every tab switch. The addresses of the message in view are published by
//! the draw walk (`set_message_urls`) for the keyboard twin.

use std::sync::Mutex;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent};
use gitpoulpi_i18n::msg::link_confirm as m;
use ratatui::layout::Rect;
use ratatui::Frame;

use crate::dialog_kit::{Confirm, ConfirmEvent};
use crate::guide_dialog::{key_of, labelled, latin_twin};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Choice {
    Open,
    Next,
    Cancel,
}

/// Cells of the address per dialog line.
const CHUNK: usize = 56;

struct Open {
    urls: Vec<String>,
    at: usize,
    confirm: Confirm<Choice>,
    keys: Vec<(char, Choice)>,
}

static POPUP: Mutex<Option<Open>> = Mutex::new(None);
/// The addresses of the message in view, set by every paint.
static MESSAGE_URLS: Mutex<Vec<String>> = Mutex::new(Vec::new());
/// A status line the window shows once (what was opened, or why not).
static RESULT: Mutex<Option<String>> = Mutex::new(None);

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// The paint walk's answer to "which addresses does the message in view
/// hold".
pub(crate) fn set_message_urls(urls: Vec<String>) {
    *lock(&MESSAGE_URLS) = urls;
}

/// The palette twin: opens the question on the message's first address.
/// `false` when the message in view has none.
pub(crate) fn ask_message_links() -> bool {
    let urls = lock(&MESSAGE_URLS).clone();
    if urls.is_empty() {
        return false;
    }
    ask(urls);
    true
}

/// Asks about `urls` (the first is shown; `Next` cycles). Anything that is
/// not a web address is dropped, and nothing is asked when none is left.
pub(crate) fn ask(urls: Vec<String>) {
    let urls: Vec<String> = urls
        .into_iter()
        .filter(|u| crate::message_links::is_web_url(u))
        .collect();
    if urls.is_empty() {
        return;
    }
    *lock(&POPUP) = Some(build(urls, 0));
    crate::redraw::poke();
}

pub(crate) fn is_open() -> bool {
    lock(&POPUP).is_some()
}

fn build(urls: Vec<String>, at: usize) -> Open {
    let url = &urls[at];
    let mut body = String::from(m::BODY.get());
    body.push_str("\n\n");
    for chunk in chunks(url) {
        body.push_str(&chunk);
        body.push('\n');
    }
    if urls.len() > 1 {
        body.push('\n');
        body.push_str(&m::position((at + 1) as u64, urls.len() as u64));
    }
    let (open, next, cancel) = (
        key_of(m::KEY_OPEN),
        key_of(m::KEY_NEXT),
        key_of(m::KEY_CANCEL),
    );
    let mut buttons = vec![(labelled(m::OPEN.get(), open), Choice::Open)];
    let mut keys = vec![(open, Choice::Open)];
    if urls.len() > 1 {
        buttons.push((labelled(m::NEXT.get(), next), Choice::Next));
        keys.push((next, Choice::Next));
    }
    buttons.push((
        labelled(gitpoulpi_i18n::msg::dialog::CANCEL.get(), cancel),
        Choice::Cancel,
    ));
    keys.push((cancel, Choice::Cancel));
    let confirm = Confirm::new(m::TITLE.get(), body, buttons, Choice::Cancel)
        .with_primary(Choice::Open)
        .keep_breaks()
        .min_width(CHUNK as u16 + 4);
    Open {
        urls,
        at,
        confirm,
        keys,
    }
}

/// The address cut into lines of at most [`CHUNK`] cells, never shortened.
fn chunks(url: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut line = String::new();
    let mut width = 0;
    for ch in url.chars() {
        let w = crate::display_width(&ch.to_string());
        if width + w > CHUNK {
            out.push(std::mem::take(&mut line));
            width = 0;
        }
        line.push(ch);
        width += w;
    }
    if !line.is_empty() {
        out.push(line);
    }
    out
}

fn settle(slot: &mut Option<Open>, event: ConfirmEvent<Choice>) {
    let ConfirmEvent::Chose(choice) = event else {
        return;
    };
    let Some(open) = slot.take() else { return };
    match choice {
        Choice::Cancel => {}
        Choice::Next => {
            let at = (open.at + 1) % open.urls.len();
            *slot = Some(build(open.urls, at));
        }
        Choice::Open => {
            let url = &open.urls[open.at];
            *lock(&RESULT) = Some(match crate::repo_menu::open_url(url) {
                Ok(()) => gitpoulpi_i18n::msg::input::opened_in_browser(url),
                Err(err) => crate::activity_log::report_problem_clear(
                    crate::activity_log::ProblemSource::Tool,
                    &err,
                ),
            });
        }
    }
}

/// A key while the dialog is open (it swallows them all).
pub(crate) fn on_key(key: KeyEvent) {
    if key.kind != KeyEventKind::Press {
        return;
    }
    let mut slot = lock(&POPUP);
    let Some(open) = slot.as_mut() else { return };
    if let KeyCode::Char(c) = key.code {
        if !key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
        {
            let c = c.to_lowercase().next().unwrap_or(c);
            if let Some((_, choice)) = open
                .keys
                .iter()
                .find(|(k, _)| *k == c || latin_twin(*k) == Some(c))
            {
                let event = ConfirmEvent::Chose(*choice);
                settle(&mut slot, event);
                return;
            }
        }
    }
    let event = open.confirm.on_key(key);
    settle(&mut slot, event);
}

/// A mouse event while the dialog is open (it swallows them all).
pub(crate) fn on_mouse(mouse: MouseEvent) {
    let mut slot = lock(&POPUP);
    let Some(open) = slot.as_mut() else { return };
    let event = open.confirm.on_mouse(mouse);
    settle(&mut slot, event);
}

/// What the last answer did (opened, or the browser's refusal), once.
pub(crate) fn take_result() -> Option<String> {
    lock(&RESULT).take()
}

/// Paints the open question over everything.
pub(crate) fn render(frame: &mut Frame) {
    let mut slot = lock(&POPUP);
    if let Some(open) = slot.as_mut() {
        let area: Rect = frame.area();
        open.confirm.render(frame.buffer_mut(), area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_whole_address_is_kept_across_the_lines() {
        let url = format!("https://a.test/{}", "x".repeat(130));
        let lines = chunks(&url);
        assert!(lines.len() > 2);
        assert!(lines.iter().all(|l| crate::display_width(l) <= CHUNK));
        assert_eq!(lines.concat(), url);
    }

    #[test]
    fn only_web_addresses_are_ever_asked_about() {
        ask(vec![
            "javascript:alert(1)".into(),
            "file:///etc/passwd".into(),
        ]);
        assert!(!is_open());
    }
}
