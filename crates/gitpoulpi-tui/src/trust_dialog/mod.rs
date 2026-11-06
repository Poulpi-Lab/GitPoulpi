//! The "Integration security check" dialog.
//!
//! A saved token is only sent where it was trusted: `gitpoulpi_hosting::trust`
//! refuses a client whose address, account or certificate changed and queues
//! the refusal (`trust::report`). This modal is what asks the person. It is fed
//! from that process-wide queue ([`feed`], once per frame), shows one check at a
//! time (deduplicated per subject and kind: ten refused requests are one
//! question) and numbers its answers:
//!
//! 1. **Disconnect**: deletes the saved token and everything cached from it.
//! 2. **Trust** the new address / account / certificate.
//! 3. **Cancel**: nothing changes; the check is remembered (it does not pop up
//!    again until the palette's *Integrations: Review security check* opens it).
//!
//! No answer is highlighted until an arrow key or a digit: a stray Enter must
//! never accept a change. The body scrolls (wheel, PageUp/PageDown, a draggable
//! bar) because a certificate table can be taller than the screen. Modal like
//! `tag_delete_dialog`: it owns the keyboard and the mouse while open.
//!
//! The state machine ([`Dialog`]) is plain data, tested without the process-wide
//! slot; [`apply`] performs the answer (files and a background token delete).

use std::sync::Mutex;

use crossterm::event::MouseEventKind;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent};
use gitpoulpi_hosting::TrustProblem;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::Frame;

use crate::dialog_kit::CloseHit;
use crate::scrollbar::{BarDrag, Track};

mod apply;
mod body;
mod render;
#[cfg(test)]
mod tests;

pub(crate) use apply::{apply, take_changed};

/// What an answer does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Choice {
    Disconnect,
    Trust,
    Cancel,
}

/// An answer that changes something (a Cancel never leaves the dialog).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Outcome {
    pub(crate) problem: TrustProblem,
    pub(crate) choice: Choice,
}

/// What the last paint put where.
#[derive(Debug, Default, Clone)]
struct Painted {
    window: Rect,
    answers: Vec<Rect>,
    /// The visible body rows.
    body: Rect,
    track: Option<Track>,
}

/// The dialog's whole state.
#[derive(Debug, Default)]
pub(crate) struct Dialog {
    /// The checks to answer; the first is on screen.
    queue: Vec<TrustProblem>,
    /// Checks cancelled in this process: kept for the palette's review.
    snoozed: Vec<TrustProblem>,
    /// The highlighted answer; `None` until an arrow key or a digit.
    cursor: Option<usize>,
    scroll: usize,
    bar: BarDrag,
    painted: Painted,
    pointer: Option<(u16, u16)>,
    close: CloseHit,
}

fn same_question(a: &TrustProblem, b: &TrustProblem) -> bool {
    a.subject() == b.subject() && a.change() == b.change()
}

impl Dialog {
    /// Queues `problems`: one entry per subject and kind (the newest wins), and
    /// a check that was cancelled and is unchanged stays cancelled.
    pub(crate) fn add(&mut self, problems: Vec<TrustProblem>) {
        for problem in problems {
            if let Some(at) = self.snoozed.iter().position(|p| same_question(p, &problem)) {
                if self.snoozed[at].new_text() == problem.new_text() {
                    continue;
                }
                self.snoozed.remove(at);
            }
            match self.queue.iter().position(|p| same_question(p, &problem)) {
                Some(at) => {
                    if at == 0 && self.queue[0] != problem {
                        self.cursor = None;
                        self.scroll = 0;
                    }
                    self.queue[at] = problem;
                }
                None => self.queue.push(problem),
            }
        }
    }

    pub(crate) fn is_open(&self) -> bool {
        !self.queue.is_empty()
    }

    /// The check on screen.
    pub(crate) fn current(&self) -> Option<&TrustProblem> {
        self.queue.first()
    }

    /// Brings the cancelled checks back (the palette). `false` when there is
    /// nothing to review.
    pub(crate) fn open_review(&mut self) -> bool {
        let mut back = std::mem::take(&mut self.snoozed);
        back.append(&mut self.queue);
        self.queue = back;
        self.reset_view();
        self.is_open()
    }

    fn reset_view(&mut self) {
        self.cursor = None;
        self.scroll = 0;
        self.bar = BarDrag::default();
    }

    /// The answers of the current check, in order.
    pub(crate) fn choices(&self) -> Vec<Choice> {
        self.current().map(body::choices).unwrap_or_default()
    }

    /// Answers with the `index`-th choice.
    fn pick(&mut self, index: usize) -> Option<Outcome> {
        let choice = *self.choices().get(index)?;
        let problem = self.queue.remove(0);
        self.reset_view();
        match choice {
            Choice::Cancel => {
                self.snoozed.push(problem);
                None
            }
            choice => Some(Outcome { problem, choice }),
        }
    }

    fn cancel(&mut self) {
        if let Some(index) = self.choices().iter().position(|c| *c == Choice::Cancel) {
            self.pick(index);
        } else if !self.queue.is_empty() {
            let problem = self.queue.remove(0);
            self.snoozed.push(problem);
            self.reset_view();
        }
    }

    fn rows(&self) -> usize {
        self.painted.body.height.max(1) as usize
    }

    fn total_lines(&self) -> usize {
        let width = self.painted.body.width as usize;
        self.current()
            .map_or(0, |p| body::lines(p, width.max(1)).len())
    }

    fn scroll_by(&mut self, delta: isize) {
        let max = self.total_lines().saturating_sub(self.rows());
        self.scroll = self.scroll.saturating_add_signed(delta).min(max);
    }

    /// A key while the dialog is open (it swallows them all).
    pub(crate) fn on_key(&mut self, key: KeyEvent) -> Option<Outcome> {
        if key.kind != KeyEventKind::Press || !self.is_open() {
            return None;
        }
        let last = self.choices().len().saturating_sub(1);
        match key.code {
            KeyCode::Esc => {
                self.cancel();
                None
            }
            KeyCode::Up | KeyCode::Char('k') if key.modifiers == KeyModifiers::NONE => {
                self.cursor = Some(self.cursor.map_or(last, |c| c.saturating_sub(1)));
                None
            }
            KeyCode::Down | KeyCode::Char('j') if key.modifiers == KeyModifiers::NONE => {
                self.cursor = Some(self.cursor.map_or(0, |c| (c + 1).min(last)));
                None
            }
            KeyCode::Char(c @ '1'..='9') if key.modifiers == KeyModifiers::NONE => {
                self.pick(c as usize - '1' as usize)
            }
            KeyCode::Enter => {
                let at = self.cursor?;
                self.pick(at)
            }
            KeyCode::PageDown => {
                self.scroll_by(self.rows() as isize);
                None
            }
            KeyCode::PageUp => {
                self.scroll_by(-(self.rows() as isize));
                None
            }
            KeyCode::Home => {
                self.scroll = 0;
                None
            }
            KeyCode::End => {
                self.scroll_by(isize::MAX / 2);
                None
            }
            _ => None,
        }
    }

    /// A mouse event while the dialog is open (it swallows them all).
    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) -> Option<Outcome> {
        if !self.is_open() {
            return None;
        }
        let at = (mouse.column, mouse.row);
        self.pointer = Some(at);
        if let Some(top) =
            self.bar
                .on_mouse(self.painted.track, mouse, self.total_lines(), self.scroll)
        {
            let max = self.total_lines().saturating_sub(self.rows());
            self.scroll = top.min(max);
            return None;
        }
        let over = self
            .painted
            .answers
            .iter()
            .position(|r| r.contains(at.into()));
        match mouse.kind {
            MouseEventKind::Moved | MouseEventKind::Drag(_) => {
                self.close.set_hover(at.0, at.1);
                if let Some(i) = over {
                    self.cursor = Some(i);
                }
                None
            }
            MouseEventKind::ScrollUp | MouseEventKind::ScrollDown
                if self.painted.window.contains(at.into()) =>
            {
                self.scroll_by(if mouse.kind == MouseEventKind::ScrollUp {
                    -3
                } else {
                    3
                });
                None
            }
            MouseEventKind::Down(MouseButton::Left) => {
                if self.close.hit(at.0, at.1) {
                    self.cancel();
                    return None;
                }
                // A click away from the dialog is swallowed: a security
                // question is never dismissed by accident.
                self.pick(over?)
            }
            _ => None,
        }
    }

    /// Paints the dialog over `area`, last.
    pub(crate) fn render(&mut self, buf: &mut Buffer, area: Rect) {
        render::paint(self, buf, area);
    }
}

static DIALOG: Mutex<Option<Dialog>> = Mutex::new(None);

fn with<R>(f: impl FnOnce(&mut Dialog) -> R) -> R {
    let mut slot = DIALOG.lock().unwrap_or_else(|p| p.into_inner());
    f(slot.get_or_insert_with(Dialog::default))
}

/// Whether the dialog owns the keyboard and the mouse.
pub(crate) fn is_open() -> bool {
    with(|d| d.is_open())
}

/// Moves what the hosting tasks queued into the dialog. Called once per frame:
/// a refusal raised on a worker thread shows up at the next paint.
pub(crate) fn feed() {
    let pending = gitpoulpi_hosting::trust::take_pending();
    if pending.is_empty() {
        return;
    }
    with(|d| d.add(pending));
}

pub(crate) fn on_key(key: KeyEvent) -> Option<Outcome> {
    with(|d| d.on_key(key))
}

pub(crate) fn on_mouse(mouse: MouseEvent) -> Option<Outcome> {
    with(|d| d.on_mouse(mouse))
}

pub(crate) fn render(frame: &mut Frame) {
    feed();
    let area = frame.area();
    with(|d| {
        if d.is_open() {
            d.render(frame.buffer_mut(), area);
        }
    });
}

/// The palette's *Integrations: Review security check*: `false` when nothing
/// waits (the caller says so).
pub(crate) fn open_review() -> bool {
    feed();
    with(|d| d.open_review())
}
