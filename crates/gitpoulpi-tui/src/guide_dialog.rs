//! Dialogs that guide instead of failing: an expected state (conflicts are
//! not resolved yet, a conflict is over the AI input limit) is explained with
//! what to do next, never reported as an error.
//!
//! Both are a [`Confirm`] box (close cross, Esc and an outside click cancel,
//! Left / Right / Tab / Enter, clicks) with one letter per button on top, so
//! every button has a key:
//!
//! * **Conflicts first** (`ConflictsFirst`): the files that still have
//!   conflicts, [Resolve conflicts] (opens the merge editor) and [Cancel].
//!   Shown by the AI commit composer and by every way of committing.
//! * **Too large** (`TooLarge`): an AI request whose input is over
//!   `ai.limits.*`: the size and the limit, [Send anyway] (this request only;
//!   the request still goes through the privacy pipeline), [Open setting]
//!   (Settings on the limit's row) and [Cancel].

use std::cell::RefCell;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use gitpoulpi_i18n::msg;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::ai_ui::AiRequest;
use crate::dialog_kit::{Confirm, ConfirmEvent};

/// The settings key the "too large" dialog sends the user to.
pub(crate) const CONFLICT_LIMIT_KEY: &str = "ai.limits.conflict_bytes";

/// Files named in the list before "and N more".
const LISTED: usize = 8;

/// What a button does; the dialog's kind says what that means.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Choice {
    /// Resolve conflicts / Send anyway.
    Primary,
    /// Open setting (the "too large" dialog only).
    Secondary,
    Cancel,
}

/// What the conflicts stand in the way of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Blocked {
    /// The AI commit composer.
    Compose,
    /// Any commit.
    Commit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Kind {
    ConflictsFirst(Blocked),
    /// An AI request held back; "Send anyway" starts it with its
    /// `oversize_ok` flag set.
    TooLarge(Box<AiRequest>),
}

#[derive(Debug, Clone)]
pub(crate) struct GuideDialog {
    kind: Kind,
    /// Painting records the hit rects the mouse reads back (the draw
    /// walk only has `&InputMode`).
    confirm: RefCell<Confirm<Choice>>,
    /// One letter per button.
    keys: Vec<(char, Choice)>,
}

/// What the window does after a key or click.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Outcome {
    Stay,
    Chose(Choice),
}

fn key_of(m: gitpoulpi_i18n::M) -> char {
    m.get().chars().next().unwrap_or(' ').to_ascii_lowercase()
}

/// "Send anyway (S)": the label with its key.
fn labelled(label: &str, key: char) -> String {
    format!("{label} ({})", key.to_ascii_uppercase())
}

// i18n-scan: off
/// "48.1 KB (49206 bytes)": what a size means to a person, and the exact count.
pub(crate) fn size_text(bytes: usize) -> String {
    if bytes >= 1_000_000 {
        format!("{:.1} MB ({bytes} bytes)", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.1} KB ({bytes} bytes)", bytes as f64 / 1_000.0)
    } else {
        format!("{bytes} bytes")
    }
}
// i18n-scan: on

impl GuideDialog {
    /// `files` still have conflicts and `blocked` cannot go on.
    pub(crate) fn conflicts_first(blocked: Blocked, files: &[String]) -> Self {
        use msg::conflict_guard as m;
        let mut body = m::count(files.len() as u64);
        body.push('\n');
        for file in files.iter().take(LISTED) {
            body.push_str("\n  ");
            body.push_str(file);
        }
        if files.len() > LISTED {
            body.push_str("\n  ");
            body.push_str(&m::more((files.len() - LISTED) as u64));
        }
        body.push_str("\n\n");
        body.push_str(match blocked {
            Blocked::Compose => m::COMPOSE_HINT.get(),
            Blocked::Commit => m::COMMIT_HINT.get(),
        });
        let (resolve, cancel) = (key_of(m::KEY_RESOLVE), key_of(m::KEY_CANCEL));
        let buttons = vec![
            (labelled(m::RESOLVE.get(), resolve), Choice::Primary),
            (labelled(msg::dialog::CANCEL.get(), cancel), Choice::Cancel),
        ];
        let confirm = Confirm::new(m::TITLE.get(), body, buttons, Choice::Cancel)
            .primary(Choice::Primary)
            .keep_breaks();
        Self {
            kind: Kind::ConflictsFirst(blocked),
            confirm: RefCell::new(confirm),
            keys: vec![(resolve, Choice::Primary), (cancel, Choice::Cancel)],
        }
    }

    /// `request` (a conflict of `path`) is `bytes` and the limit is `limit`.
    pub(crate) fn too_large(request: AiRequest, path: &str, bytes: usize, limit: usize) -> Self {
        use msg::ai_limits as m;
        let body = m::too_large_body(path, size_text(bytes), size_text(limit), CONFLICT_LIMIT_KEY);
        let (send, open, cancel) = (
            key_of(m::KEY_SEND),
            key_of(m::KEY_OPEN),
            key_of(m::KEY_CANCEL),
        );
        let buttons = vec![
            (labelled(m::SEND_ANYWAY.get(), send), Choice::Primary),
            (labelled(m::OPEN_SETTING.get(), open), Choice::Secondary),
            (labelled(msg::dialog::CANCEL.get(), cancel), Choice::Cancel),
        ];
        let confirm = Confirm::new(m::TOO_LARGE_TITLE.get(), body, buttons, Choice::Cancel)
            // Focus lands on "Open setting": sending data out is never one
            // Enter away.
            .with_primary(Choice::Secondary);
        Self {
            kind: Kind::TooLarge(Box::new(request)),
            confirm: RefCell::new(confirm),
            keys: vec![
                (send, Choice::Primary),
                (open, Choice::Secondary),
                (cancel, Choice::Cancel),
            ],
        }
    }

    pub(crate) fn kind(&self) -> &Kind {
        &self.kind
    }

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> Outcome {
        if let KeyCode::Char(c) = key.code {
            if !key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
            {
                let c = c.to_ascii_lowercase();
                if let Some((_, choice)) = self.keys.iter().find(|(k, _)| *k == c) {
                    return Outcome::Chose(*choice);
                }
            }
        }
        Self::outcome(self.confirm.get_mut().on_key(key))
    }

    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) -> Outcome {
        Self::outcome(self.confirm.get_mut().on_mouse(mouse))
    }

    fn outcome(event: ConfirmEvent<Choice>) -> Outcome {
        match event {
            ConfirmEvent::Stay => Outcome::Stay,
            ConfirmEvent::Chose(choice) => Outcome::Chose(choice),
        }
    }

    /// Paints the box centered in `bounds`.
    pub(crate) fn render(&self, buf: &mut Buffer, bounds: Rect) {
        self.confirm.borrow_mut().render(buf, bounds);
    }
}

#[cfg(test)]
mod tests;
