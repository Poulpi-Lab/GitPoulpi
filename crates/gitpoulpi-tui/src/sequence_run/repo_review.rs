//! "Sequences of this repository": the read-only list of every sequence and
//! step of the repository's `.gitpoulpi/sequences.toml`, shown before the
//! user trusts them. `[Trust]` records exactly what was listed; `[Close]`
//! (Esc, the cross, a click outside) trusts nothing.
//!
//! The box is the guiding dialog of `guide_dialog` (a `Confirm` with a letter
//! per button), and like the other question dialogs its state is
//! process-wide: the window restarts on every tab switch. A `Confirm` cuts a
//! body that does not fit the terminal, and a person must never trust what
//! they could not read: when the list does not fit, `[Trust]` is not offered
//! (the box says so and points to Settings > Sequences, which lists every
//! step) and shrinking the terminal after opening rebuilds the box the same
//! way.

use std::sync::Mutex;

use crossterm::event::{KeyEvent, KeyEventKind, MouseEvent};
use gitpoulpi_config::sequences::{PauseMode, SequenceDef};
use gitpoulpi_i18n::msg::sequence_run as m;
use ratatui::layout::Rect;
use ratatui::Frame;

use crate::dialog_kit::Confirm;
use crate::guide_dialog::{key_of, labelled, Choice, GuideDialog, Kind, Outcome};

/// The widest the box is drawn.
const MAX_WIDTH: u16 = 100;

struct Open {
    dialog: GuideDialog,
    /// The terminal size the box was laid out for.
    built_for: (u16, u16),
    fits: bool,
}

static POPUP: Mutex<Option<Open>> = Mutex::new(None);
static RESULT: Mutex<Option<String>> = Mutex::new(None);

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// One line per step, a heading per sequence.
pub(crate) fn body(defs: &[SequenceDef]) -> String {
    let mut out = String::from(m::REPO_REVIEW_INTRO.get());
    for def in defs {
        let name = if def.name.trim().is_empty() {
            &def.id
        } else {
            &def.name
        };
        out.push_str("\n\n");
        out.push_str(&m::repo_review_sequence(name, &def.id));
        for (i, step) in def.steps.iter().enumerate() {
            let mut line = format!("\n  {}", m::repo_review_step((i + 1) as u64, &step.run));
            match step.pause {
                PauseMode::Always => {
                    line.push_str("  \u{b7} ");
                    line.push_str(m::REPO_REVIEW_PAUSE_ALWAYS.get());
                }
                PauseMode::Never => {
                    line.push_str("  \u{b7} ");
                    line.push_str(m::REPO_REVIEW_PAUSE_NEVER.get());
                }
                PauseMode::Default => {}
            }
            if !step.options.is_empty() {
                let options: Vec<String> = step
                    .options
                    .iter()
                    .map(|(k, v)| match v {
                        toml::Value::String(s) => format!("{k}={s}"),
                        other => format!("{k}={other}"),
                    })
                    .collect();
                line.push_str("  \u{b7} ");
                line.push_str(&m::repo_review_options(options.join(", ")));
            }
            out.push_str(&line);
        }
    }
    out
}

/// How many rows the box needs for `body` at `inner_width` columns (what
/// `Confirm::render` lays out: wrapped lines, title, buttons).
fn rows_needed(body: &str, inner_width: usize) -> usize {
    let lines: usize = body
        .split('\n')
        .map(|line| {
            let wrapped = crate::ui_kit::wrap_words(line, inner_width);
            wrapped.len().max(1)
        })
        .sum();
    lines + 4
}

fn build(defs: &[SequenceDef], area: (u16, u16)) -> Open {
    let width = area.0.saturating_sub(2).clamp(44, MAX_WIDTH);
    let inner = usize::from(width.saturating_sub(4)).max(1);
    let text = body(defs);
    // Long lines are broken by the box too: count them at their real width.
    let fits = rows_needed(&text, inner) <= usize::from(area.1.saturating_sub(1))
        && area.0 >= 48
        && defs.iter().all(|d| {
            d.steps
                .iter()
                .all(|s| crate::ui_kit::display_width(&s.run) < inner)
        });
    let (trust, close) = (key_of(m::REPO_KEY_TRUST), key_of(m::REPO_KEY_CLOSE));
    let mut buttons = Vec::new();
    let mut keys = Vec::new();
    if fits {
        buttons.push((labelled(m::REPO_REVIEW_TRUST.get(), trust), Choice::Primary));
        keys.push((trust, Choice::Primary));
    }
    buttons.push((labelled(m::REPO_REVIEW_CLOSE.get(), close), Choice::Cancel));
    keys.push((close, Choice::Cancel));
    let shown = if fits {
        text
    } else {
        format!("{text}\n\n{}", m::REPO_REVIEW_TOO_LONG.get())
    };
    let confirm = Confirm::new(m::REPO_REVIEW_TITLE.get(), shown, buttons, Choice::Cancel)
        .with_primary(Choice::Cancel)
        .min_width(width)
        .keep_breaks();
    Open {
        dialog: GuideDialog::assemble(Kind::RepoSequences, confirm, keys),
        built_for: area,
        fits,
    }
}

/// Opens the review for the active repository's file.
pub(crate) fn open() {
    let defs = super::repo::reviewed();
    if defs.is_empty() {
        return;
    }
    let area = crossterm::terminal::size().unwrap_or((100, 30));
    *lock(&POPUP) = Some(build(&defs, area));
    crate::redraw::poke();
}

pub(crate) fn is_open() -> bool {
    lock(&POPUP).is_some()
}

fn apply(slot: &mut Option<Open>, outcome: Outcome) {
    let Outcome::Chose(choice) = outcome else {
        return;
    };
    if choice == Choice::Primary && slot.as_ref().is_some_and(|o| o.fits) {
        let said = match super::repo::trust() {
            Ok(n) => m::say_repo_trusted(n as u64),
            Err(why) => why,
        };
        *lock(&RESULT) = Some(said);
    }
    *slot = None;
    crate::redraw::poke();
}

/// A key while the dialog is open (it swallows them all).
pub(crate) fn on_key(key: KeyEvent) {
    if key.kind != KeyEventKind::Press {
        return;
    }
    let mut slot = lock(&POPUP);
    let Some(open) = slot.as_mut() else {
        return;
    };
    let outcome = open.dialog.on_key(key);
    apply(&mut slot, outcome);
}

/// A mouse event while the dialog is open (it swallows them all).
pub(crate) fn on_mouse(mouse: MouseEvent) {
    let mut slot = lock(&POPUP);
    let Some(open) = slot.as_mut() else {
        return;
    };
    let outcome = open.dialog.on_mouse(mouse);
    apply(&mut slot, outcome);
}

/// The status line text the last answer leaves, once.
pub(crate) fn take_result() -> Option<String> {
    lock(&RESULT).take()
}

/// Paints the dialog over everything; a resized terminal re-fits it first.
pub(crate) fn render(frame: &mut Frame) {
    let area: Rect = frame.area();
    let mut slot = lock(&POPUP);
    let Some(open) = slot.as_mut() else {
        return;
    };
    if open.built_for != (area.width, area.height) {
        let defs = super::repo::reviewed();
        if defs.is_empty() {
            *slot = None;
            return;
        }
        *slot = Some(build(&defs, (area.width, area.height)));
    }
    if let Some(open) = slot.as_ref() {
        open.dialog.render(frame.buffer_mut(), area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_config::sequences::StepSpec;

    fn def(id: &str, steps: usize) -> SequenceDef {
        let mut d = SequenceDef {
            id: id.into(),
            name: format!("Name {id}"),
            ..SequenceDef::default()
        };
        for i in 0..steps {
            d.steps.push(if i == 1 {
                StepSpec::new("commit")
                    .with_pause(PauseMode::Always)
                    .with_option("message", "wip")
            } else {
                StepSpec::new("stage_all")
            });
        }
        d
    }

    #[test]
    fn the_body_lists_every_step_with_its_pause_and_options() {
        let text = body(&[def("ship", 3)]);
        assert!(text.contains("Name ship (ship)"), "{text}");
        assert!(text.contains("1. stage_all"), "{text}");
        assert!(text.contains("2. commit"), "{text}");
        assert!(text.contains("pauses before it runs"), "{text}");
        assert!(text.contains("options: message=wip"), "{text}");
        assert!(text.contains("3. stage_all"), "{text}");
    }

    #[test]
    fn trust_is_offered_only_when_every_step_could_be_read() {
        assert!(build(&[def("a", 3)], (100, 40)).fits);
        let long = build(&[def("a", 80)], (100, 24));
        assert!(!long.fits, "a list the box would cut offers no Trust");
    }
}
