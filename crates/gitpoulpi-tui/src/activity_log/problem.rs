//! Failures that are not a git action: a clone, a hosting call, an external
//! program, an AI job, a save. They get no card: the status line words them
//! (red, `status_error`) and the Activity Log keeps the entry (title, reason,
//! next step, details, Copy, the remedy button).
//!
//! The call sites hand over the message they used to put on the status line;
//! the wording (title, reason, step) comes from the [`Source`].

use std::collections::VecDeque;
use std::sync::Mutex;

use gitpoulpi_core::FailureKind;
use gitpoulpi_i18n::msg::failure_card as f;

use super::feed::{self, Entry, Severity};
use super::friendly;

/// What kind of operation failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Source {
    Clone,
    /// `git init` / a new repository.
    Create,
    /// A GitHub / GitLab call: lists, pull requests, issues, token checks.
    Hosting,
    /// An external program: editor, difftool, mergetool, terminal, browser.
    Tool,
    Ai,
    /// A file that could not be written.
    Save,
}

impl Source {
    fn title(self) -> String {
        match self {
            Source::Clone => f::TITLE_CLONE,
            Source::Create => f::TITLE_CREATE,
            Source::Hosting => f::TITLE_HOSTING,
            Source::Tool => f::TITLE_TOOL,
            Source::Ai => f::TITLE_AI,
            Source::Save => f::TITLE_SAVE,
        }
        .get()
        .to_string()
    }

    /// What the operation is called in the entry's context and the report
    /// (English, like the rest of the report).
    fn label(self) -> &'static str {
        match self {
            Source::Clone => "Clone",              // i18n-english
            Source::Create => "Create repository", // i18n-english
            Source::Hosting => "Hosting request",  // i18n-english
            Source::Tool => "External program",    // i18n-english
            Source::Ai => "AI request",
            Source::Save => "Save file", // i18n-english
        }
    }

    fn wording(self) -> (String, String) {
        let (summary, hint) = match self {
            Source::Clone => (f::SUMMARY_CLONE, f::HINT_CLONE),
            Source::Create => (f::SUMMARY_CREATE, f::HINT_CREATE),
            Source::Hosting => (f::SUMMARY_HOSTING, f::HINT_HOSTING),
            Source::Tool => (f::SUMMARY_TOOL, f::HINT_TOOL),
            Source::Ai => (f::SUMMARY_AI, f::HINT_AI),
            Source::Save => (f::SUMMARY_SAVE, f::HINT_SAVE),
        };
        (summary.get().to_string(), hint.get().to_string())
    }
}

/// How many remembered readings are kept.
#[cfg_attr(not(feature = "hosting"), allow(dead_code))]
const KINDS_KEPT: usize = 32;

/// Messages already worded for the user (so not readable by their English)
/// with the reading of the typed error they came from.
static KINDS: Mutex<VecDeque<(String, FailureKind)>> = Mutex::new(VecDeque::new());

/// Remembers that `text` came from a failure of `kind`: a message that went
/// through a string channel is still read right when it is reported.
#[cfg_attr(not(feature = "hosting"), allow(dead_code))]
pub(crate) fn remember_kind(text: &str, kind: FailureKind) {
    let mut kinds = KINDS.lock().unwrap_or_else(|e| e.into_inner());
    kinds.retain(|(t, _)| t != text);
    if kinds.len() >= KINDS_KEPT {
        kinds.pop_front();
    }
    kinds.push_back((text.to_string(), kind));
}

fn remembered(text: &str) -> Option<FailureKind> {
    let kinds = KINDS.lock().unwrap_or_else(|e| e.into_inner());
    kinds.iter().find(|(t, _)| t == text).map(|(_, k)| *k)
}

/// The Activity entry and the status line's error for a failure of `source`.
/// `message` is what the status line used to say; it is redacted here and
/// stays the entry's details.
pub(crate) fn report(source: Source, message: &str) {
    push_report(source, message, None, None);
}

/// [`report`] that says which call failed: `label` ("Hosting request: list
/// merge requests") and its main `args` ("host=github.com"), shown with the
/// entry, in the application log line and in the error report.
pub(crate) fn report_as(source: Source, label: &str, args: &str, message: &str) {
    let context = gitpoulpi_core::OpContext::named(label, args);
    push_report(source, message, None, Some(context));
}

/// [`report`] for a failure that has a place on disk but no open tab (a
/// clone's destination): the Global Activity Log lists it under that
/// repository. Returns the entry's id.
pub(crate) fn report_in(source: Source, message: &str, repo: &std::path::Path) -> u64 {
    push_report(source, message, Some(repo), None)
}

fn push_report(
    source: Source,
    message: &str,
    repo: Option<&std::path::Path>,
    context: Option<gitpoulpi_core::OpContext>,
) -> u64 {
    let kind = remembered(message).unwrap_or_else(|| match source {
        Source::Clone | Source::Hosting => FailureKind::classify(message),
        _ => FailureKind::Other,
    });
    let message = gitpoulpi_core::redact(message);
    let mut entry = Entry::new(Severity::Error, source.title(), message);
    entry.kind = kind;
    let (summary, hint) =
        friendly::wording(kind, &entry.message).unwrap_or_else(|| source.wording());
    let line = format!("{}: {summary}", entry.title);
    entry.summary = Some(summary);
    entry.hint = Some(hint);
    entry.repo = repo.map(std::path::Path::to_path_buf);
    let repo_text = repo.map(super::repo_text).unwrap_or_default();
    let id = feed::reserve_id();
    entry.id = id;
    let context = super::complete_context(
        context.unwrap_or_else(|| gitpoulpi_core::OpContext::named(source.label(), "")),
        repo.unwrap_or(std::path::Path::new(crate::tabs::NO_REPOSITORY)),
        id,
    );
    tracing::warn!(target: "tui", repo = repo_text.as_str(), "{}", context.failure_line(&entry.message));
    entry.context = Some(context);
    super::report::warm_up();
    let id = feed::push(entry);
    super::status_error::arm(id, line, None);
    id
}

/// [`report`] for a call site that must hand the status line a string: the
/// status error says it (the window takes it on its next tick), so the line
/// is cleared rather than repeating it.
pub(crate) fn report_and_clear(source: Source, message: &str) -> String {
    report(source, message);
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lock() -> std::sync::MutexGuard<'static, ()> {
        crate::activity_log::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    fn newest() -> Entry {
        feed::snapshot().into_iter().next().expect("an entry")
    }

    #[test]
    fn a_failure_is_an_error_entry_with_its_sources_wording() {
        let _lock = lock();
        report(Source::Tool, "could not start `nope`");
        let entry = newest();
        assert_eq!(entry.severity, Severity::Error);
        assert_eq!(entry.title, f::TITLE_TOOL.get());
        assert_eq!(entry.summary.as_deref(), Some(f::SUMMARY_TOOL.get()));
        assert!(entry.message.contains("nope"));
    }

    #[test]
    fn a_failure_arms_the_status_line_with_title_and_reason() {
        let _lock = lock();
        report(Source::Tool, "could not start `nope`");
        let line = super::super::status_error::take_pending().expect("an error line");
        assert_eq!(
            line,
            format!("error: {}: {}", f::TITLE_TOOL.get(), f::SUMMARY_TOOL.get())
        );
        assert!(super::super::status_error::has_error());
    }

    #[test]
    fn a_remembered_reading_wins_over_the_text() {
        let _lock = lock();
        remember_kind("the host said no", FailureKind::AuthFailed);
        report(Source::Hosting, "the host said no");
        assert_eq!(newest().kind, FailureKind::AuthFailed);
    }

    #[test]
    fn a_failure_names_the_call_that_failed_and_its_arguments() {
        let _lock = lock();
        report_as(
            Source::Hosting,
            "Hosting request: list merge requests",
            "host=gitlab.example.test",
            "the host said no",
        );
        let context = newest().context.expect("a context");
        assert_eq!(context.label, "Hosting request: list merge requests");
        assert_eq!(context.args, "host=gitlab.example.test");
        assert_eq!(context.frontend.as_deref(), Some("TUI"));
        assert!(context.request > 0);
    }

    #[test]
    fn a_failure_with_no_call_named_is_known_by_its_source() {
        let _lock = lock();
        report(Source::Tool, "could not start `nope`");
        let context = newest().context.expect("a context");
        assert_eq!(context.label, "External program");
    }

    #[test]
    fn secrets_never_reach_the_entry() {
        let _lock = lock();
        report(
            Source::Clone,
            "unable to access 'http://me:topsecret@127.0.0.1:1/r.git'",
        );
        let entry = newest();
        assert!(!entry.message.contains("topsecret"), "{}", entry.message);
    }
}
