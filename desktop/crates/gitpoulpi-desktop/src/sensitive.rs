//! The sensitive-files question: a stage the daemon's staging guard refused because some files look
//! like secrets. It lists each file with its reason (a path pattern, or the kind of secret found with a
//! masked preview, never a value) and offers "Stage anyway" or "Cancel, stage nothing".
//!
//! Nothing is highlighted until an arrow key or a digit is pressed, so a stray Enter answers nothing;
//! Escape, the close control and a click outside cancel. Cancelling stages nothing, not even the
//! ordinary files of the same stage. Looking at a file first (the terminal UI's "Show in file") is not
//! offered yet.

use gitpoulpi_core::privacy::{Reason, Sensitive};
use gitpoulpi_i18n::msg::stage_guard as msg;
use gitpoulpi_link::{report_ticket, Action, StepOutcome};
use gpui_kit::{App, Window};

use crate::app::workspace_of;
use crate::daemon::host::SensitivePrompt;
use crate::dialogs::{self, Choice, Spec};

/// One line of the list: the path (with the line of a secret) and why it was flagged.
pub fn line_of(file: &Sensitive) -> String {
    let place = match file.line {
        Some(n) => format!("{}:{n}", file.path),
        None => file.path.clone(),
    };
    let why = match &file.reason {
        Reason::Pattern(p) => msg::reason_pattern(p),
        Reason::Secret { rule, preview } => msg::reason_secret(rule, preview),
    };
    format!("{place}  -  {why}")
}

/// The dialog's texts.
pub fn spec_of(files: &[Sensitive]) -> Spec {
    Spec {
        title: msg::TITLE.get().to_string(),
        lines: std::iter::once(msg::QUESTION.get().to_string())
            .chain(files.iter().map(line_of))
            .collect(),
        choices: vec![
            Choice::new(msg::ANSWER_STAGE.get()),
            Choice::new(msg::ANSWER_CANCEL.get()),
        ],
        hint: msg::HINT.get().to_string(),
        highlight_first: false,
        yes_no_keys: false,
    }
}

/// Asks the person. "Stage anyway" sends the action again with sensitive files allowed, under the same
/// ticket; every other way out ends the ticket as canceled.
pub fn ask(prompt: SensitivePrompt, window: &mut Window, cx: &mut App) {
    let SensitivePrompt {
        action,
        files,
        ticket,
        ..
    } = prompt;
    let spec = spec_of(&files);
    dialogs::open(
        window,
        cx,
        spec,
        move |ix, cx| {
            if ix == 0 {
                if let Some(ws) = workspace_of(cx) {
                    let again = Action::AllowSensitive(Box::new(action));
                    ws.read(cx)
                        .backend()
                        .with_link(|link| link.submit_as(ticket, again));
                    return;
                }
            }
            report_ticket(ticket, StepOutcome::Canceled);
        },
        move || report_ticket(ticket, StepOutcome::Canceled),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_names_the_file_its_line_and_the_reason() {
        let pattern = Sensitive {
            path: ".env".into(),
            reason: Reason::Pattern(".env".into()),
            line: None,
        };
        assert!(line_of(&pattern).starts_with(".env  -  "));
        let secret = Sensitive {
            path: "app.conf".into(),
            reason: Reason::Secret {
                rule: "github-token".into(),
                preview: "ghp_****".into(),
            },
            line: Some(12),
        };
        let line = line_of(&secret);
        assert!(line.starts_with("app.conf:12  -  "), "{line}");
        assert!(line.contains("github-token") && line.contains("ghp_****"));
        assert!(!line_of(&pattern).contains("\u{2014}"));
    }

    #[test]
    fn the_answers_are_numbered_stage_then_cancel() {
        let spec = spec_of(&[]);
        assert_eq!(spec.choices.len(), 2);
        assert!(!spec.highlight_first, "a stray Enter answers nothing");
    }
}
