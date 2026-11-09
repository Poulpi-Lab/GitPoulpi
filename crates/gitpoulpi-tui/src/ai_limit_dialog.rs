//! "Over the AI limit": the question asked when an input of an AI request is
//! over one of the caps of `[ai.limits]` and would have been silently cut.
//!
//! A worker builds its request, finds the inputs over their caps and asks
//! through the job's [`Gate`] (`gitpoulpi_ai::LimitGate`); this dialog lists
//! each of them (which part, its size, the limit, the setting) and offers:
//!
//! * **Send all** (`A`): this request only, nothing is cut. It still goes
//!   through the privacy pipeline (secrets replaced, sensitive files left
//!   out) and the "What is sent" pane, which comes next.
//! * **Send truncated** (`T`): what is over a limit is cut, like it always
//!   was; each line of the dialog says what that cut is.
//! * **Open setting** (`O`): cancels the request and opens Settings > AI on
//!   the row of the first limit listed.
//! * **Cancel** (`C`, `Esc`, the cross, a click outside): sends nothing.
//!
//! Focus starts on Open setting: sending is never one Enter away.
//!
//! The box is the guiding dialog of `guide_dialog` (a `Confirm` with a letter
//! per button). State is process-wide like "What is sent": the window
//! restarts on every tab switch and the question comes from a worker thread
//! that waits for the answer. A request canceled elsewhere (the jobs list)
//! closes its question by itself.
//!
//! Runs nobody starts, and the CLI and MCP, never get here: without a gate the
//! caps truncate and the answer reports it (`gitpoulpi_ai::limits`).

use std::sync::{Arc, Mutex};

use crossterm::event::{KeyEvent, KeyEventKind, MouseEvent};
use gitpoulpi_ai::{Cut, LimitAnswer, LimitGate, Part};
use gitpoulpi_i18n::msg;
use ratatui::layout::Rect;
use ratatui::Frame;

use crate::dialog_kit::Confirm;
use crate::guide_dialog::{key_of, labelled, size_text, Choice, GuideDialog, Kind, Outcome};

/// Cuts listed before "and N more".
const LISTED: usize = 6;

struct Open {
    dialog: GuideDialog,
    /// The row "Open setting" lands on.
    setting: &'static str,
    reply: Option<tokio::sync::oneshot::Sender<LimitAnswer>>,
}

static POPUP: Mutex<Option<Open>> = Mutex::new(None);
/// Questions that arrived while one was open.
static WAITING: Mutex<Vec<Open>> = Mutex::new(Vec::new());
/// Set by "Open setting", taken by the window to open Settings.
static OPEN_SETTING: Mutex<Option<&'static str>> = Mutex::new(None);

/// The worker's side: asks the dialog and waits for the answer.
pub(crate) struct Gate;

impl Gate {
    pub(crate) fn shared() -> Arc<dyn LimitGate> {
        Arc::new(Self)
    }
}

#[async_trait::async_trait]
impl LimitGate for Gate {
    async fn ask(&self, cuts: &[Cut]) -> LimitAnswer {
        let (tx, rx) = tokio::sync::oneshot::channel();
        ask(Open {
            dialog: dialog(cuts),
            setting: cuts
                .first()
                .map(|c| c.setting)
                .unwrap_or("ai.limits.diff_bytes"),
            reply: Some(tx),
        });
        // A dropped window or a canceled request drops the sender.
        rx.await.unwrap_or(LimitAnswer::Cancel)
    }
}

fn ask(open: Open) {
    let mut slot = POPUP.lock().unwrap_or_else(|e| e.into_inner());
    if slot.is_none() {
        *slot = Some(open);
    } else {
        WAITING.lock().unwrap_or_else(|e| e.into_inner()).push(open);
    }
    drop(slot);
    crate::redraw::poke();
}

/// Whether the dialog is up. A question whose request was canceled meanwhile
/// is dropped here, so the dialog never outlives what it asks about.
pub(crate) fn is_open() -> bool {
    let Ok(mut slot) = POPUP.lock() else {
        return false;
    };
    while slot
        .as_ref()
        .is_some_and(|o| !o.reply.as_ref().is_some_and(|r| !r.is_closed()))
    {
        finish(&mut slot);
    }
    slot.is_some()
}

/// Closes the open question and brings up the next live one.
fn finish(slot: &mut Option<Open>) {
    let mut waiting = WAITING.lock().unwrap_or_else(|e| e.into_inner());
    waiting.retain(|o| o.reply.as_ref().is_some_and(|r| !r.is_closed()));
    *slot = if waiting.is_empty() {
        None
    } else {
        Some(waiting.remove(0))
    };
}

fn resolve(slot: &mut Option<Open>, answer: LimitAnswer) {
    if let Some(open) = slot.as_mut() {
        if let Some(reply) = open.reply.take() {
            let _ = reply.send(answer);
        }
    }
    finish(slot);
}

fn apply(slot: &mut Option<Open>, outcome: Outcome) {
    let Outcome::Chose(choice) = outcome else {
        return;
    };
    match choice {
        Choice::Primary => resolve(slot, LimitAnswer::SendAll),
        Choice::Truncated => resolve(slot, LimitAnswer::SendTruncated),
        Choice::Secondary => {
            if let Some(open) = slot.as_ref() {
                *OPEN_SETTING.lock().unwrap_or_else(|e| e.into_inner()) = Some(open.setting);
            }
            resolve(slot, LimitAnswer::Cancel);
        }
        Choice::Cancel => resolve(slot, LimitAnswer::Cancel),
    }
}

/// A key while the dialog is open (it swallows them all).
pub(crate) fn on_key(key: KeyEvent) {
    if key.kind != KeyEventKind::Press {
        return;
    }
    let Ok(mut slot) = POPUP.lock() else {
        return;
    };
    let Some(open) = slot.as_mut() else {
        return;
    };
    let outcome = open.dialog.on_key(key);
    apply(&mut slot, outcome);
}

/// A mouse event while the dialog is open (it swallows them all).
pub(crate) fn on_mouse(mouse: MouseEvent) {
    let Ok(mut slot) = POPUP.lock() else {
        return;
    };
    let Some(open) = slot.as_mut() else {
        return;
    };
    let outcome = open.dialog.on_mouse(mouse);
    apply(&mut slot, outcome);
}

/// The setting "Open setting" chose, once: the window opens Settings on it.
pub(crate) fn take_open_setting() -> Option<&'static str> {
    OPEN_SETTING
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .take()
}

/// Paints the open question over everything.
pub(crate) fn render(frame: &mut Frame) {
    let Ok(slot) = POPUP.lock() else {
        return;
    };
    if let Some(open) = slot.as_ref() {
        let area: Rect = frame.area();
        open.dialog.render(frame.buffer_mut(), area);
    }
}

/// One line per cut: what it is, its size, the limit, the setting and what
/// "Send truncated" does to it.
fn line_of(cut: &Cut) -> String {
    use msg::ai_limits as m;
    let (size, limit) = (size_text(cut.bytes), size_text(cut.limit));
    let name = cut.detail.as_str();
    let key = cut.setting;
    match cut.part {
        Part::Diff => m::part_diff(size, limit, key),
        Part::SplitHunks => m::part_hunks(size, limit, key),
        Part::ReviewDiff => m::part_review_diff(name, size, limit, key),
        Part::ReviewFile => m::part_review_file(name, size, limit, key),
        Part::ReviewText => m::part_review_text(size, limit, key),
        Part::ReviewThreads => m::part_review_threads(size, limit, key),
        Part::ContextFile => m::part_context_file(name, size, limit, key),
        Part::ContextTotal => m::part_context_total(name, size, limit, key),
    }
}

/// The dialog for `cuts` (at least one).
pub(crate) fn dialog(cuts: &[Cut]) -> GuideDialog {
    use msg::ai_limits as m;
    let mut body = String::from(m::TRUNCATE_INTRO.get());
    body.push('\n');
    for cut in cuts.iter().take(LISTED) {
        body.push_str("\n- ");
        body.push_str(&line_of(cut));
    }
    if cuts.len() > LISTED {
        body.push_str("\n- ");
        body.push_str(&msg::conflict_guard::more((cuts.len() - LISTED) as u64));
    }
    body.push_str("\n\n");
    body.push_str(m::TRUNCATE_OUTRO.get());
    let (all, truncated, open, cancel) = (
        key_of(m::KEY_SEND_ALL),
        key_of(m::KEY_SEND_TRUNCATED),
        key_of(m::KEY_OPEN),
        key_of(m::KEY_CANCEL),
    );
    let buttons = vec![
        (labelled(m::SEND_ALL.get(), all), Choice::Primary),
        (
            labelled(m::SEND_TRUNCATED.get(), truncated),
            Choice::Truncated,
        ),
        (labelled(m::OPEN_SETTING.get(), open), Choice::Secondary),
        (labelled(msg::dialog::CANCEL.get(), cancel), Choice::Cancel),
    ];
    let confirm = Confirm::new(m::TRUNCATE_TITLE.get(), body, buttons, Choice::Cancel)
        // Focus lands on "Open setting": sending data out is never one
        // Enter away.
        .with_primary(Choice::Secondary)
        .keep_breaks();
    GuideDialog::assemble(
        Kind::Truncation,
        confirm,
        vec![
            (all, Choice::Primary),
            (truncated, Choice::Truncated),
            (open, Choice::Secondary),
            (cancel, Choice::Cancel),
        ],
    )
}

#[cfg(test)]
mod tests;
