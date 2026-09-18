//! The Blame view: the whole file with line numbers and
//! syntax colors, and an annotation column (avatar, author, age, summary)
//! on the right by default (`ui.blame_side`), grouped per **hunk** - a run
//! of consecutive lines one commit last touched. Only a hunk's first line
//! carries the text; the rest show a per-author color bar. A second
//! one-cell strip is the age heatmap (recent warm, old cold, relative to
//! the file's own oldest and newest line).
//!
//! # Pieces
//!
//! - [`hunks`]: the hunks of a result, computed once when it lands (a
//!   20 000-line file costs one linear pass, never a per-frame walk).
//! - [`layout`]: the columns as a pure function of the pane's width.
//! - [`paint`]: the rows, written straight into the frame buffer, plus the
//!   hit rects (annotation cells, the border handle).
//! - [`card`]: the detail card (on hover after a short dwell, or pinned
//!   with a key) and the right-click menu - one popup painter.
//! - [`ops`]: the actions, one enum shared by keys, mouse and palette.
//!
//! State lives in [`BlameView`], owned by the [`FileInsight`] that shows
//! the result (`FileInsight::blame`); selection and scroll stay the
//! insight's own (`selected`, `offset`, one row per file line), so the
//! wheel, the scrollbar, `j`/`k`, PageUp and Home keep working untouched.
//!
//! Highlighting is the editor's incremental [`Highlight`]: parsed under a
//! time budget each frame, the rest of the screen drawn plain until it
//! catches up (`highlight_pending` keeps the loop drawing).

mod card;
mod hunks;
mod layout;
mod ops;
mod paint;
mod pending;

#[cfg(test)]
mod tests;

use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, MouseEventKind};
use gitpoulpi_core::domain::{Blame, BlameContent, BlameLine, Oid};
use gitpoulpi_i18n::msg::blame_view as msg;
use gitpoulpi_i18n::M;
use ratatui::layout::Rect;

use crate::editor_highlight::Highlight;
use crate::file_view::{FileInsight, FileInsightData};
use crate::keymap::Keymap;

pub(crate) use hunks::Hunk;
pub(crate) use layout::MIN_MAP_BODY;
pub(crate) use ops::{BlameHit, BlameOp};
pub(crate) use paint::{legend, legend_width, paint_minimap, render, BlameEnv};
pub(crate) use pending::{settle, waiting, want_diff, Ready};

/// How long the pointer rests on an annotation before its card opens.
pub(crate) const HOVER_DELAY: Duration = Duration::from_millis(450);

static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// The right-click menu: where it was opened, for which hunk.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Menu {
    pub(crate) x: u16,
    pub(crate) y: u16,
    pub(crate) hunk: usize,
    pub(crate) sel: usize,
}

pub(crate) struct BlameView {
    pub(crate) hunks: Vec<Hunk>,
    /// Hunk index of every line.
    hunk_of: Vec<u32>,
    pub(crate) hl: Highlight,
    /// Highlighting did not reach what is on screen within the frame.
    pub(crate) hl_pending: bool,
    /// The card was opened with its key and stays on the selected line's
    /// hunk until closed.
    pub(crate) pinned: bool,
    /// The hunk the pointer rests on, and since when.
    hover: Option<(usize, Instant)>,
    /// A hunk whose card was dismissed with Esc while the pointer is
    /// still on it: no card until the pointer leaves.
    dismissed: Option<usize>,
    pub(crate) menu: Option<Menu>,
    /// The annotation border is being dragged.
    pub(crate) drag: bool,
    /// "Go to line" being typed.
    pub(crate) goto: Option<String>,
    /// The pane's inner rect as last painted (the drag maths needs it).
    pub(crate) inner: Rect,
    /// The annotation side as last painted.
    pub(crate) left: bool,
    /// Names this result for the minimap's cache.
    id: u64,
    /// The minimap (`ui.minimap`), colored by the age heat.
    pub(crate) minimap: crate::minimap::Minimap,
}

impl BlameView {
    pub(crate) fn new(path: &std::path::Path, lines: &[BlameLine]) -> Self {
        let (hunks, hunk_of) = hunks::build(lines);
        let first = lines.first().map_or("", |l| l.content.as_str());
        Self {
            hunks,
            hunk_of,
            hl: Highlight::for_file(path, first),
            hl_pending: false,
            pinned: false,
            hover: None,
            dismissed: None,
            menu: None,
            drag: false,
            goto: None,
            inner: Rect::default(),
            left: false,
            id: NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            minimap: crate::minimap::Minimap::default(),
        }
    }

    /// The hunk index of `line`.
    pub(crate) fn hunk_at(&self, line: usize) -> Option<usize> {
        self.hunk_of.get(line).map(|h| *h as usize)
    }

    /// Records where the pointer is, for the hover card's dwell.
    pub(crate) fn track_hover(&mut self, over: Option<BlameHit>, now: Instant) {
        match over {
            Some(BlameHit::Annot(line)) => {
                let hunk = self.hunk_at(line);
                match (hunk, self.hover) {
                    (Some(h), Some((cur, _))) if h == cur => {}
                    (Some(h), _) => {
                        self.hover = Some((h, now));
                        if self.dismissed != Some(h) {
                            self.dismissed = None;
                        }
                    }
                    (None, _) => self.hover = None,
                }
            }
            // On the card itself the hovered hunk stays.
            Some(BlameHit::Card | BlameHit::Op(_)) => {}
            _ => {
                self.hover = None;
                self.dismissed = None;
            }
        }
    }

    /// The hunk whose card is on screen: the pinned one (the selected
    /// line's), else the hovered one after the dwell.
    pub(crate) fn card_hunk(&self, selected: usize, now: Instant) -> Option<usize> {
        if self.menu.is_some() {
            return None;
        }
        if self.pinned {
            return self.hunk_at(selected);
        }
        match self.hover {
            Some((h, since)) if now.duration_since(since) >= HOVER_DELAY => {
                (self.dismissed != Some(h)).then_some(h)
            }
            _ => None,
        }
    }

    /// When the hover card will open, if it is waiting for its dwell.
    pub(crate) fn hover_deadline(&self, now: Instant) -> Option<Instant> {
        let (h, since) = self.hover?;
        let due = since + HOVER_DELAY;
        (due > now && self.dismissed != Some(h) && !self.pinned && self.menu.is_none())
            .then_some(due)
    }
}

/// Where an op landed in the keyboard handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KeyOutcome {
    /// Not the Blame view's key: the generic handler takes it.
    Pass,
    Handled,
    /// The caller runs the op (it needs the window's state).
    Run(BlameOp),
}

/// What a mouse event did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MouseOutcome {
    Pass,
    Handled,
    Run(BlameOp),
    /// The border drag ended: persist this width.
    Width(u16),
}

impl FileInsight {
    /// Whether this is a Blame result the new view draws.
    pub(crate) fn blame_lines(&self) -> Option<&[BlameLine]> {
        match &self.data {
            FileInsightData::Blame(Blame {
                content: BlameContent::Lines(lines),
                ..
            }) if self.blame.is_some() => Some(lines),
            _ => None,
        }
    }

    /// The hunk under the selected line.
    pub(crate) fn blame_selected_hunk(&self) -> Option<&Hunk> {
        let view = self.blame.as_ref()?;
        view.hunks.get(view.hunk_at(self.selected)?)
    }

    /// The commit of the selected line's hunk, `None` when uncommitted.
    pub(crate) fn blame_selected_commit(&self) -> Option<Oid> {
        self.blame_selected_hunk()
            .filter(|h| !h.uncommitted)
            .map(|h| h.commit)
    }

    /// The Blame view keeps the loop drawing while syntax colors catch up.
    pub(crate) fn blame_highlight_pending(&self) -> bool {
        self.blame.as_ref().is_some_and(|b| b.hl_pending)
    }

    /// The moment the hover card opens, for the loop's deadline.
    pub(crate) fn blame_hover_deadline(&self, now: Instant) -> Option<Instant> {
        self.blame.as_ref().and_then(|b| b.hover_deadline(now))
    }

    /// Closes the card, the menu and the prompt (after an action ran).
    pub(crate) fn blame_close_popups(&mut self) {
        if let Some(b) = self.blame.as_mut() {
            b.pinned = false;
            b.menu = None;
            b.goto = None;
            b.hover = None;
        }
    }

    /// Moves the selection by hunks. `Err` is the status-line reason.
    pub(crate) fn blame_nav(&mut self, op: BlameOp, viewport: usize) -> Result<(), M> {
        let Some(view) = self.blame.as_ref() else {
            return Ok(());
        };
        let Some(cur) = view.hunk_at(self.selected) else {
            return Ok(());
        };
        let target = match op {
            BlameOp::NextHunk => view.hunks.get(cur + 1).map(|h| h.start),
            BlameOp::PrevHunk => {
                if self.selected > view.hunks[cur].start {
                    Some(view.hunks[cur].start)
                } else {
                    cur.checked_sub(1).map(|p| view.hunks[p].start)
                }
            }
            BlameOp::NextSame => {
                let commit = view.hunks[cur].commit;
                view.hunks[cur + 1..]
                    .iter()
                    .find(|h| h.commit == commit)
                    .map(|h| h.start)
            }
            BlameOp::PrevSame => {
                let commit = view.hunks[cur].commit;
                view.hunks[..cur]
                    .iter()
                    .rev()
                    .find(|h| h.commit == commit)
                    .map(|h| h.start)
            }
            _ => return Ok(()),
        };
        match target {
            Some(line) => {
                self.select(line, viewport);
                Ok(())
            }
            None if matches!(op, BlameOp::NextSame | BlameOp::PrevSame) => Err(msg::NO_SAME),
            None => Err(msg::NO_MORE_HUNKS),
        }
    }

    /// Keys of the Blame view. The text selection's Esc, closing, the row
    /// moves and Enter (the Log jump) stay with the generic handler.
    pub(crate) fn blame_key(&mut self, km: &Keymap, key: &KeyEvent, viewport: usize) -> KeyOutcome {
        let Some(view) = self.blame.as_mut() else {
            return KeyOutcome::Pass;
        };
        // "Go to line": digits, Backspace, Enter, Esc; every other key is
        // swallowed while the prompt is up.
        if let Some(buf) = view.goto.as_mut() {
            match key.code {
                KeyCode::Char(c) if c.is_ascii_digit() && buf.len() < 9 => buf.push(c),
                KeyCode::Backspace => {
                    buf.pop();
                }
                KeyCode::Esc => view.goto = None,
                KeyCode::Enter => {
                    let line = buf.parse::<usize>().ok();
                    view.goto = None;
                    if let Some(n) = line.filter(|n| *n > 0) {
                        let last = self.len().saturating_sub(1);
                        self.select((n - 1).min(last), viewport);
                    }
                }
                _ => {}
            }
            return KeyOutcome::Handled;
        }
        if let Some(menu) = view.menu.as_mut() {
            let last = BlameOp::POPUP.len() - 1;
            match key.code {
                KeyCode::Esc => view.menu = None,
                KeyCode::Up | KeyCode::Char('k') => menu.sel = menu.sel.saturating_sub(1),
                KeyCode::Down | KeyCode::Char('j') => menu.sel = (menu.sel + 1).min(last),
                KeyCode::Enter => {
                    let op = BlameOp::POPUP[menu.sel];
                    view.menu = None;
                    return KeyOutcome::Run(op);
                }
                _ => {}
            }
            return KeyOutcome::Handled;
        }
        if key.code == KeyCode::Esc
            && (view.pinned || view.card_hunk(self.selected, Instant::now()).is_some())
        {
            view.dismissed = view.hover.map(|(h, _)| h);
            view.pinned = false;
            view.hover = None;
            return KeyOutcome::Handled;
        }
        let Some(op) = BlameOp::ALL
            .into_iter()
            .find(|op| *op != BlameOp::ShowInLog && km.is(op.action(), key))
        else {
            return KeyOutcome::Pass;
        };
        match op {
            BlameOp::ToggleCard => {
                view.pinned = !view.pinned;
                view.hover = None;
                KeyOutcome::Handled
            }
            BlameOp::Goto => {
                view.goto = Some(String::new());
                KeyOutcome::Handled
            }
            _ => KeyOutcome::Run(op),
        }
    }

    /// The mouse over the Blame view (`hit` is what the pointer is on).
    pub(crate) fn blame_mouse(
        &mut self,
        kind: MouseEventKind,
        col: u16,
        hit: Option<BlameHit>,
    ) -> MouseOutcome {
        use crossterm::event::MouseButton::Left;
        let Some(view) = self.blame.as_mut() else {
            return MouseOutcome::Pass;
        };
        if view.drag {
            return match kind {
                MouseEventKind::Drag(Left) => {
                    let want = layout::width_for_pointer(view.inner, view.left, col);
                    crate::live_settings::install_blame_width(want);
                    MouseOutcome::Handled
                }
                MouseEventKind::Up(Left) => {
                    view.drag = false;
                    MouseOutcome::Width(crate::live_settings::blame_width())
                }
                _ => MouseOutcome::Handled,
            };
        }
        match (kind, hit) {
            (MouseEventKind::Down(Left), Some(BlameHit::Op(op))) => MouseOutcome::Run(op),
            (MouseEventKind::Down(Left), Some(BlameHit::Card)) => MouseOutcome::Handled,
            (MouseEventKind::Down(Left), Some(BlameHit::Handle)) => {
                view.drag = true;
                MouseOutcome::Handled
            }
            (MouseEventKind::Down(Left), _) if view.menu.is_some() => {
                view.menu = None;
                MouseOutcome::Pass
            }
            _ => MouseOutcome::Pass,
        }
    }

    /// Opens the right-click menu of the hunk of `line` at `(x, y)`.
    /// `false` when there is nothing to offer (uncommitted lines).
    pub(crate) fn blame_open_menu(&mut self, line: usize, x: u16, y: u16, viewport: usize) -> bool {
        let Some(hunk) = self.blame.as_ref().and_then(|b| b.hunk_at(line)) else {
            return false;
        };
        self.select(line, viewport);
        let Some(view) = self.blame.as_mut() else {
            return false;
        };
        if view.hunks[hunk].uncommitted {
            return false;
        }
        view.pinned = false;
        view.hover = None;
        view.menu = Some(Menu { x, y, hunk, sel: 0 });
        true
    }

    /// The text column's cells within the inner width: `(x, width)`.
    pub(crate) fn blame_text_cols(&self, width: u16, lines: usize) -> (u16, u16) {
        let l = layout::layout(
            Rect::new(0, 0, width, 1),
            lines,
            crate::live_settings::blame_left(),
            crate::live_settings::blame_width(),
        );
        (l.text_x, l.text_w)
    }

    /// The request that blames the selected hunk's commit's parent: the
    /// revision and path the line had before the commit (`BlameLine::previous`,
    /// which follows renames). `Err` when the commit added the file.
    pub(crate) fn blame_parent_target(&self) -> Result<crate::daemon_link::FileInsightRequest, M> {
        let hunk = self.blame_selected_hunk().ok_or(msg::NOT_COMMITTED_YET)?;
        if hunk.uncommitted {
            return Err(msg::NOT_COMMITTED_YET);
        }
        let line = self
            .blame_lines()
            .and_then(|l| l.get(hunk.start))
            .ok_or(msg::NOT_COMMITTED_YET)?;
        let (at, path) = line.previous.clone().ok_or(msg::NO_PREVIOUS)?;
        Ok(crate::daemon_link::FileInsightRequest {
            at,
            path,
            worktree: false,
            staged: false,
            ..self.request.clone()
        })
    }
}

/// Seconds since the epoch, for the relative ages.
pub(crate) fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}
