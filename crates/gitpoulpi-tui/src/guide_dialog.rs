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
//! * **Truncation** (`Truncation`, built and hosted by `ai_limit_dialog`):
//!   the same warning for the inputs the AI would have silently cut: [Send
//!   all], [Send truncated], [Open setting] and [Cancel].

use std::cell::RefCell;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use gitpoulpi_i18n::msg;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use gitpoulpi_core::domain::Oid;

use crate::ai_ui::AiRequest;
use crate::dialog_kit::{Confirm, ConfirmEvent};
use crate::reword_ui::AmendScope;

/// The settings key the "too large" dialog sends the user to.
pub(crate) const CONFLICT_LIMIT_KEY: &str = "ai.limits.conflict_bytes";

/// Files named in the list before "and N more".
const LISTED: usize = 8;

/// What a button does; the dialog's kind says what that means.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Choice {
    /// Resolve conflicts / Send anyway.
    Primary,
    /// Open setting (the "too large" and truncation dialogs).
    Secondary,
    /// Send truncated (the truncation dialog only; `Primary` is Send all).
    #[cfg_attr(not(feature = "ai"), allow(dead_code))]
    Truncated,
    /// Check out the n-th branch of the off-branch edit dialog, then edit.
    Pick(u8),
    /// Edit in the advanced history rewrite (the off-branch edit dialog).
    Rewrite,
    /// Create a branch at the commit (the off-branch edit dialog).
    CreateBranch,
    Cancel,
}

/// The commit an edit was asked for although it is not on the checked-out
/// branch, and what the dialog offers for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OffBranch {
    pub(crate) commit: Oid,
    /// `None`: Edit commit message; `Some`: Amend, with that scope.
    pub(crate) scope: Option<AmendScope>,
    /// The local branches offered (at most [`MAX_BRANCHES`]), in button order.
    pub(crate) branches: Vec<String>,
}

/// Branch buttons the off-branch dialog lists (keys `1` to `8`).
const MAX_BRANCHES: usize = 8;

/// Cells of a branch name kept in its button.
const BRANCH_CELLS: usize = 28;

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
    /// Inputs over their caps, asked by a worker (`ai_limit_dialog`).
    #[cfg_attr(not(feature = "ai"), allow(dead_code))]
    Truncation,
    /// An edit of a commit that is not on the checked-out branch.
    OffBranch(OffBranch),
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

pub(crate) fn key_of(m: gitpoulpi_i18n::M) -> char {
    lower(m.get().chars().next().unwrap_or(' '))
}

/// One character, lowercased: catalog key letters may be non-ASCII (Cyrillic, Greek...).
fn lower(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

/// The Latin letter on the same physical key as a Cyrillic letter (Russian and
/// Ukrainian layouts over QWERTY), so a localized key letter still works when
/// the keyboard is switched to a Latin layout. `None` for other letters.
pub(crate) fn latin_twin(c: char) -> Option<char> {
    const PAIRS: &[(char, char)] = &[
        ('й', 'q'),
        ('ц', 'w'),
        ('у', 'e'),
        ('к', 'r'),
        ('е', 't'),
        ('н', 'y'),
        ('г', 'u'),
        ('ш', 'i'),
        ('щ', 'o'),
        ('з', 'p'),
        ('ф', 'a'),
        ('ы', 's'),
        ('і', 's'),
        ('в', 'd'),
        ('а', 'f'),
        ('п', 'g'),
        ('р', 'h'),
        ('о', 'j'),
        ('л', 'k'),
        ('д', 'l'),
        ('я', 'z'),
        ('ч', 'x'),
        ('с', 'c'),
        ('м', 'v'),
        ('и', 'b'),
        ('т', 'n'),
        ('ь', 'm'),
    ];
    PAIRS.iter().find(|(cyr, _)| *cyr == c).map(|&(_, lat)| lat)
}

/// "Send anyway (S)": the label with its key.
pub(crate) fn labelled(label: &str, key: char) -> String {
    format!("{label} ({})", key.to_uppercase())
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

    /// `commit` is not on the checked-out branch: check out one of the local
    /// `branches` that hold it and edit there, edit it in the advanced history
    /// rewrite (`rewrite`: the build has it), or, with no such branch, create
    /// one at it. The choices are a list of rows: the arrows and `Ctrl+J` /
    /// `Ctrl+K` move through them, each row has a key.
    pub(crate) fn off_branch(
        commit: Oid,
        scope: Option<AmendScope>,
        branches: &[String],
        rewrite: bool,
    ) -> Self {
        use msg::edit_off_branch as m;
        let short = &commit.to_hex()[..crate::log_menu::SHORT_SHA_LEN];
        let shown: Vec<String> = branches.iter().take(MAX_BRANCHES).cloned().collect();
        let mut body = if shown.is_empty() {
            m::body_none(short)
        } else {
            m::body_branches(short)
        };
        if branches.len() > shown.len() {
            body.push_str("\n\n");
            body.push_str(&msg::conflict_guard::more(
                (branches.len() - shown.len()) as u64,
            ));
        }
        let (rewrite_key, create_key, cancel_key) = (
            key_of(m::KEY_REWRITE),
            key_of(m::KEY_CREATE),
            key_of(m::KEY_CANCEL),
        );
        let mut rows: Vec<(String, char, Choice)> = Vec::new();
        for (i, name) in shown.iter().enumerate() {
            let key = char::from_digit(i as u32 + 1, 10).unwrap_or('9');
            let name = crate::ui_kit::fit(name, BRANCH_CELLS);
            rows.push((
                labelled(&m::checkout(name), key),
                key,
                Choice::Pick(i as u8),
            ));
        }
        if shown.is_empty() {
            rows.push((
                labelled(m::CREATE.get(), create_key),
                create_key,
                Choice::CreateBranch,
            ));
        }
        if rewrite {
            rows.push((
                labelled(m::REWRITE.get(), rewrite_key),
                rewrite_key,
                Choice::Rewrite,
            ));
        }
        rows.push((
            labelled(msg::dialog::CANCEL.get(), cancel_key),
            cancel_key,
            Choice::Cancel,
        ));
        // One width for every row, so the list reads as one column of buttons.
        let widest = rows
            .iter()
            .map(|(l, ..)| crate::display_width(l))
            .max()
            .unwrap_or(0);
        let buttons: Vec<(String, Choice)> = rows
            .iter()
            .map(|(l, _, c)| {
                let pad = widest - crate::display_width(l);
                (format!("{l}{}", " ".repeat(pad)), *c)
            })
            .collect();
        let first = rows[0].2;
        let confirm = Confirm::new(m::TITLE.get(), body, buttons, Choice::Cancel)
            .with_primary(first)
            .stacked()
            .keep_breaks()
            .min_width(56);
        Self {
            kind: Kind::OffBranch(OffBranch {
                commit,
                scope,
                branches: shown,
            }),
            confirm: RefCell::new(confirm),
            keys: rows.into_iter().map(|(_, k, c)| (k, c)).collect(),
        }
    }

    /// A dialog of `kind` made of its parts (the truncation dialog's text is
    /// built where the cuts are known).
    #[cfg_attr(not(feature = "ai"), allow(dead_code))]
    pub(crate) fn assemble(
        kind: Kind,
        confirm: Confirm<Choice>,
        keys: Vec<(char, Choice)>,
    ) -> Self {
        Self {
            kind,
            confirm: RefCell::new(confirm),
            keys,
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
                let c = lower(c);
                if let Some((_, choice)) = self
                    .keys
                    .iter()
                    .find(|(k, _)| *k == c || latin_twin(*k) == Some(c))
                {
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
