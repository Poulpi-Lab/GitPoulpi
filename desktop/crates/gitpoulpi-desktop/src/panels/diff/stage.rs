//! Hunk and line staging from the diff pane: the cursor row, the line selection (`v`), and the requests
//! the pane makes of the window (stage, unstage, discard a hunk or the selected lines).
//!
//! The pane knows no backend: a request is an event ([`DiffPaneEvent::Part`]) the workspace carries out,
//! asking first when it discards. Which rows are changed lines and which hunk each belongs to is the
//! shared diff model's (`DiffRow`, `hunk_at`, `selected_lines`), so the numbering is the one the daemon
//! resolves hunk ids and line indices against.

use gitpoulpi_app::diff::{hunk_at, selected_lines};
use gitpoulpi_core::domain::HunkId;
use gitpoulpi_i18n::msg::diff_actions as msg;
use gpui_kit::{px, Context, ListOffset, Window};

use super::{DiffPane, DiffPaneEvent};
use crate::daemon::staging::PartOp;

/// Where the open diff comes from, which decides what its hunks can do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiffSource {
    /// A commit's diff: read-only.
    Commit,
    /// The working tree against the index: hunks and lines can be staged or discarded.
    Unstaged,
    /// The index against HEAD: hunks and lines can be unstaged.
    Staged,
}

impl DiffSource {
    /// A working-tree half: hunk ids in it mean something to the daemon.
    pub fn is_working(self) -> bool {
        !matches!(self, DiffSource::Commit)
    }
}

/// What the pane asks the window to do with one hunk, or some lines of it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartRequest {
    pub path: std::path::PathBuf,
    pub hunk: HunkId,
    /// `None`: the whole hunk. `Some`: these lines of it (indices into the hunk's own lines).
    pub lines: Option<Vec<u32>>,
    pub op: PartOp,
    /// The selection reached into another hunk; only the first one is acted on.
    pub spilled: bool,
}

impl DiffPane {
    pub fn source(&self) -> DiffSource {
        self.source
    }

    /// The row the keyboard is on.
    pub fn cursor_row(&self) -> usize {
        self.cursor
    }

    /// The rows of the line selection (`v` then move), ends included, when one is open.
    pub fn selection(&self) -> Option<(usize, usize)> {
        self.anchor
            .map(|a| (a.min(self.cursor), a.max(self.cursor)))
    }

    /// The rows that stand for a changed line (what a line selection can cover), in order.
    pub fn changed_rows(&self) -> Vec<usize> {
        self.rows
            .meta
            .iter()
            .enumerate()
            .filter(|(_, m)| m.lines.iter().any(Option::is_some))
            .map(|(ix, _)| ix)
            .collect()
    }

    /// The hunk headers' rows.
    pub fn hunk_rows(&self) -> Vec<usize> {
        self.rows.hunk_starts.clone()
    }

    /// The path shown.
    pub fn current_path(&self) -> Option<std::path::PathBuf> {
        self.files.get(self.selected).map(|f| f.path.clone())
    }

    /// Puts the cursor on `row` (clamped). With `extend` the selection grows from where it was (or from
    /// the old cursor row when none was open).
    pub fn set_cursor(&mut self, row: usize, extend: bool, cx: &mut Context<Self>) {
        let last = self.rows.lines.len().saturating_sub(1);
        if extend && self.anchor.is_none() {
            self.anchor = Some(self.cursor);
        }
        self.cursor = row.min(last);
        self.list.scroll_to_reveal_item(self.cursor);
        cx.notify();
    }

    /// Up / Down (or `k` / `j`).
    pub fn move_cursor(&mut self, delta: isize, extend: bool, cx: &mut Context<Self>) {
        self.set_cursor(self.cursor.saturating_add_signed(delta), extend, cx);
    }

    /// `v`: starts a line selection at the cursor, or drops the one that is open.
    pub fn toggle_selection(&mut self, cx: &mut Context<Self>) {
        self.anchor = match self.anchor {
            Some(_) => {
                cx.emit(DiffPaneEvent::Say(msg::SELECTION_CLEARED.get().to_string()));
                None
            }
            None => Some(self.cursor),
        };
        cx.notify();
    }

    /// The first hunk header of the file, where a fresh file puts the cursor.
    pub(super) fn home_row(&self) -> usize {
        self.rows.hunk_starts.first().copied().unwrap_or(0)
    }

    /// The whole hunk under the cursor.
    pub fn request_hunk(&mut self, op: PartOp, cx: &mut Context<Self>) {
        let meta = &self.rows.meta;
        let Some(hunk) = hunk_at(meta, self.cursor) else {
            return self.say(msg::NO_HUNK.get(), cx);
        };
        self.send(hunk, None, op, false, cx);
    }

    /// The selected lines (or the one under the cursor when no selection is open).
    pub fn request_lines(&mut self, op: PartOp, cx: &mut Context<Self>) {
        let (from, to) = self.selection().unwrap_or((self.cursor, self.cursor));
        let Some((hunk, lines, spilled)) = selected_lines(&self.rows.meta, from, to) else {
            return self.say(msg::NO_HUNK.get(), cx);
        };
        if lines.is_empty() {
            return self.say(msg::SELECTION_NO_LINES.get(), cx);
        }
        self.anchor = None;
        self.send(hunk, Some(lines), op, spilled, cx);
    }

    /// `Space` (Stage or Unstage the hunk): which one the half the diff was opened from decides.
    pub fn request_toggle_hunk(&mut self, cx: &mut Context<Self>) {
        match self.source {
            DiffSource::Unstaged => self.request_hunk(PartOp::Stage, cx),
            DiffSource::Staged => self.request_hunk(PartOp::Unstage, cx),
            DiffSource::Commit => self.say_not_working(false, cx),
        }
    }

    /// `Enter` (Stage or Unstage the selected lines).
    pub fn request_toggle_lines(&mut self, cx: &mut Context<Self>) {
        match self.source {
            DiffSource::Unstaged => self.request_lines(PartOp::Stage, cx),
            DiffSource::Staged => self.request_lines(PartOp::Unstage, cx),
            DiffSource::Commit => self.say_not_working(false, cx),
        }
    }

    fn send(
        &mut self,
        hunk: HunkId,
        lines: Option<Vec<u32>>,
        op: PartOp,
        spilled: bool,
        cx: &mut Context<Self>,
    ) {
        let discard = op == PartOp::Discard;
        let allowed = match (self.source, op) {
            (DiffSource::Commit, _) => return self.say_not_working(discard, cx),
            (DiffSource::Unstaged, PartOp::Unstage) | (DiffSource::Staged, PartOp::Stage) => {
                return self.say_wrong_half(cx)
            }
            (DiffSource::Staged, PartOp::Discard) => {
                return self.say(msg::DISCARD_ON_STAGED.get(), cx)
            }
            _ => true,
        };
        let Some(path) = self.current_path().filter(|_| allowed) else {
            return;
        };
        cx.emit(DiffPaneEvent::Part(PartRequest {
            path,
            hunk,
            lines,
            op,
            spilled,
        }));
    }

    fn say(&mut self, text: &str, cx: &mut Context<Self>) {
        cx.emit(DiffPaneEvent::Say(text.to_string()));
    }

    fn say_not_working(&mut self, discard: bool, cx: &mut Context<Self>) {
        let text = if discard {
            msg::NOT_WORKING_DIFF_DISCARD.get()
        } else {
            msg::NOT_WORKING_DIFF.get()
        };
        self.say(text, cx);
    }

    fn say_wrong_half(&mut self, cx: &mut Context<Self>) {
        let text = match self.source {
            DiffSource::Unstaged => {
                gitpoulpi_i18n::msg::palette::WHY_THIS_DIFF_IS_THE_UNSTAGED_HALF_USE_THE_STAGE.get()
            }
            _ => {
                gitpoulpi_i18n::msg::palette::WHY_THIS_DIFF_IS_THE_STAGED_HALF_USE_THE_UNSTAGE.get()
            }
        };
        self.say(text, cx);
    }

    /// Replaces the files after the working tree changed (a hunk was staged): the same path stays open
    /// when it still has changes, else its neighbour; the cursor keeps its row where it can.
    pub fn refresh_files(
        &mut self,
        files: std::sync::Arc<Vec<gitpoulpi_core::domain::FileDiff>>,
        cx: &mut Context<Self>,
    ) {
        let path = self.current_path();
        let at = path
            .as_ref()
            .and_then(|p| files.iter().position(|f| &f.path == p))
            .unwrap_or(self.selected);
        if files.is_empty() {
            cx.emit(DiffPaneEvent::Close);
            return;
        }
        let keep = (path.is_some() && files.get(at).map(|f| &f.path) == path.as_ref())
            .then_some(self.cursor);
        self.files = files;
        self.selected = at.min(self.files.len() - 1);
        self.anchor = None;
        self.keep_cursor = keep;
        self.rebuild(cx);
    }

    /// Scrolls so the cursor row is at the top (tests and the "go to hunk" jumps).
    pub fn reveal_cursor(&mut self) {
        self.list.scroll_to(ListOffset {
            item_ix: self.cursor,
            offset_in_item: px(0.0),
        });
    }

    /// Focus helper for the commands: the pane takes the keyboard.
    pub fn take_focus(&self, window: &mut Window, cx: &mut gpui_kit::App) {
        self.focus.focus(window, cx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_working_halves_stage() {
        assert!(!DiffSource::Commit.is_working());
        assert!(DiffSource::Unstaged.is_working());
        assert!(DiffSource::Staged.is_working());
    }
}
