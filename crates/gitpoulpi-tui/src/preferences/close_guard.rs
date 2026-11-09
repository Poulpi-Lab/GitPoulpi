//! The Settings close guard (see the ux-settings-and-close design, X1): closing
//! the screen (Esc, `q`, the cross, a click outside) asks first when
//! something would be lost or left half done:
//!
//! * the Workflow draft was edited and not saved (the save bar's dot),
//! * the commit template was edited and not saved,
//! * a profile has no author email (a warning item of the Profiles list).
//!
//! The question is a [`Confirm`]: [Go to it] [Discard and close] [Cancel]
//! for unsaved work, [Go to it] [Close anyway] [Cancel] for a warning.
//! Open sub-editors need no entry: they own Esc, the cross and the click
//! outside, and ask "Discard changes?" themselves (`editor_close.rs`).

use crossterm::event::{KeyEvent, MouseEvent};
use gitpoulpi_i18n::msg::settings_screen as screen;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::{PrefFocus, PrefSection, PreferencesState};
use crate::dialog_kit::{Confirm, ConfirmEvent};

/// What stops Settings from closing quietly, most costly first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CloseIssue {
    /// The Workflow draft differs from what was loaded.
    Workflow,
    /// The commit template was edited and not saved.
    Template,
    /// The SSH & credentials page was edited and not saved.
    RepoAuth,
    /// The profile at this index has no author email.
    Profile(usize),
}

impl CloseIssue {
    /// Unsaved work (discarding loses it) as opposed to a warning.
    fn loses_work(self) -> bool {
        !matches!(self, CloseIssue::Profile(_))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CloseChoice {
    GoTo,
    Close,
    Cancel,
}

/// The open question and what it is about.
pub(super) struct CloseAsk {
    confirm: Confirm<CloseChoice>,
    issue: CloseIssue,
}

impl PreferencesState {
    /// The first thing that closing now would lose or leave undone.
    pub(super) fn close_guard(&self) -> Option<CloseIssue> {
        // Reading the draft loads the files: a page never visited has no
        // draft and nothing to lose.
        if self.workflow_draft_loaded() && self.wf_dirty() {
            return Some(CloseIssue::Workflow);
        }
        if self.repo_pages.template.dirty {
            return Some(CloseIssue::Template);
        }
        if self.ra_dirty() {
            return Some(CloseIssue::RepoAuth);
        }
        self.config
            .profiles
            .iter()
            .position(|p| p.author_email.trim().is_empty())
            .map(CloseIssue::Profile)
    }

    /// A request to close Settings. Returns whether Settings stays open:
    /// `false` closes it now, `true` means a question was opened.
    pub(super) fn request_close(&mut self) -> bool {
        let Some(issue) = self.close_guard() else {
            return false;
        };
        let (title, body) = match issue {
            CloseIssue::Workflow => (screen::CLOSE_TITLE.get(), screen::CLOSE_WORKFLOW.get()),
            CloseIssue::Template => (screen::CLOSE_TITLE.get(), screen::CLOSE_TEMPLATE.get()),
            CloseIssue::RepoAuth => (
                screen::CLOSE_TITLE.get(),
                gitpoulpi_i18n::msg::repo_auth::CLOSE_BODY.get(),
            ),
            CloseIssue::Profile(_) => (screen::CLOSE_TITLE.get(), screen::CLOSE_PROFILE.get()),
        };
        let leave = if issue.loses_work() {
            screen::CLOSE_DISCARD.get()
        } else {
            screen::CLOSE_ANYWAY.get()
        };
        let confirm = Confirm::new(
            title,
            body,
            vec![
                (screen::CLOSE_GO_TO.get().to_string(), CloseChoice::GoTo),
                (leave.to_string(), CloseChoice::Close),
                (screen::CANCEL.get().to_string(), CloseChoice::Cancel),
            ],
            CloseChoice::Cancel,
        )
        .with_primary(CloseChoice::GoTo);
        self.close_ask = Some(CloseAsk { confirm, issue });
        true
    }

    /// A key while the question is up. Returns whether Settings stays open.
    pub(super) fn close_ask_key(&mut self, key: KeyEvent) -> bool {
        let Some(ask) = self.close_ask.as_mut() else {
            return true;
        };
        let event = ask.confirm.on_key(key);
        self.finish_close_ask(event)
    }

    /// The mouse while the question is up. Returns whether Settings stays open.
    pub(super) fn close_ask_mouse(&mut self, mouse: MouseEvent) -> bool {
        let Some(ask) = self.close_ask.as_mut() else {
            return true;
        };
        let event = ask.confirm.on_mouse(mouse);
        self.finish_close_ask(event)
    }

    fn finish_close_ask(&mut self, event: ConfirmEvent<CloseChoice>) -> bool {
        let ConfirmEvent::Chose(choice) = event else {
            return true;
        };
        let Some(ask) = self.close_ask.take() else {
            return true;
        };
        match choice {
            CloseChoice::Cancel => true,
            CloseChoice::Close => false,
            CloseChoice::GoTo => {
                self.go_to_issue(ask.issue);
                true
            }
        }
    }

    /// Lands on the page (and the profile) the issue is about.
    fn go_to_issue(&mut self, issue: CloseIssue) {
        let (section, focus) = match issue {
            CloseIssue::Workflow => (PrefSection::Workflow, PrefFocus::Fields),
            CloseIssue::Template => (PrefSection::CommitTemplate, PrefFocus::Fields),
            CloseIssue::RepoAuth => (PrefSection::RepoAuth, PrefFocus::Fields),
            CloseIssue::Profile(_) => (PrefSection::Profiles, PrefFocus::List),
        };
        self.section = PrefSection::ALL
            .iter()
            .position(|s| *s == section)
            .unwrap_or(self.section);
        if let CloseIssue::Profile(index) = issue {
            self.profile_index = index;
        }
        self.focus = focus;
        self.strip = false;
        self.scroll = 0;
        self.field = self.first_selectable();
        self.sync_list();
    }

    /// Paints the question when it is up (call after the page).
    pub(super) fn render_close_ask(&mut self, buf: &mut Buffer, area: Rect) {
        if let Some(ask) = self.close_ask.as_mut() {
            ask.confirm.render(buf, area);
        }
    }
}
