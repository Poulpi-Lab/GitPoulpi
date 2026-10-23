//! "Browse..." next to the Extra file fields of Settings > AI > Code review
//! (global) and Settings > Repo-specific AI > Code review: the folder picker
//! in file mode over the page. A repository page starts at the repository
//! root and stores a file inside it relative to the root; the global page has
//! no root and stores an absolute path.

use crossterm::event::{KeyEvent, MouseEvent, MouseEventKind};
use ratatui::Frame;

use super::*;
use crate::file_browse::{BrowseOutcome, FileBrowse};
use gitpoulpi_i18n::msg::review_options as msg;

impl AiPage {
    /// Whether the file picker is the open dialog.
    pub(in crate::preferences) fn browse_open(&self) -> bool {
        self.browse.is_some()
    }
}

impl PreferencesState {
    /// The Browse button of the Extra file row of `scope`.
    pub(super) fn review_open_browse(&mut self, scope: AiScope) {
        let (root, current) = match scope {
            AiScope::Global => (None, self.config.ai.review.extra_file.clone()),
            AiScope::Repo => (
                Some(self.repo_root.clone()),
                self.repo_state
                    .preferences
                    .ai_review
                    .extra_file
                    .clone()
                    .unwrap_or_default(),
            ),
        };
        self.ai.browse = Some((
            scope,
            FileBrowse::open(
                msg::BROWSE_TITLE.get(),
                msg::BROWSE_CONFIRM.get(),
                root.as_deref(),
                &current,
            ),
        ));
    }

    fn review_browsed(&mut self, outcome: BrowseOutcome) {
        match outcome {
            BrowseOutcome::Stay => {}
            BrowseOutcome::Cancel => self.ai.browse = None,
            BrowseOutcome::Chosen(path) => {
                if let Some((scope, _)) = self.ai.browse.take() {
                    self.review_commit_text(scope, &path);
                }
            }
        }
    }

    pub(in crate::preferences) fn review_browse_key(&mut self, key: KeyEvent) {
        let Some((_, browse)) = self.ai.browse.as_mut() else {
            return;
        };
        let outcome = browse.on_key(key);
        self.review_browsed(outcome);
    }

    pub(in crate::preferences) fn review_browse_mouse(
        &mut self,
        kind: MouseEventKind,
        col: u16,
        row: u16,
    ) {
        let Some((_, browse)) = self.ai.browse.as_mut() else {
            return;
        };
        let outcome = browse.on_mouse(MouseEvent {
            kind,
            column: col,
            row,
            modifiers: crossterm::event::KeyModifiers::NONE,
        });
        self.review_browsed(outcome);
    }

    pub(in crate::preferences) fn review_browse_paste(&mut self, text: &str) {
        if let Some((_, browse)) = self.ai.browse.as_mut() {
            browse.paste(text);
        }
    }

    pub(in crate::preferences) fn render_review_browse(&mut self, frame: &mut Frame, area: Rect) {
        if let Some((_, browse)) = self.ai.browse.as_mut() {
            browse.render(area, frame.buffer_mut());
        }
    }
}
