//! A failure in the user's words: a title for the operation, a one-line
//! reason and a suggested next step, and the buttons that can do the step.
//!
//! The reading of git's message (`FailureKind`) lives in `gitpoulpi-core`;
//! only the wording and the choice of buttons are here.

use gitpoulpi_core::{Error as CoreError, FailureKind};
use gitpoulpi_i18n::msg::git_error as g;

use crate::daemon_link::Action;

/// What an Activity Log entry can offer to do about a failure.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Remedy {
    /// Pull the branch, then push again by hand.
    PullFirst,
    /// Open Settings on the Integrations page.
    OpenIntegrations,
    /// Run the failed action again with the auto-stash.
    StashAndRetry(Action),
    /// Run the failed action again.
    Retry(Action),
}

impl Remedy {
    pub(crate) fn label(&self) -> String {
        match self {
            Remedy::PullFirst => g::ACTION_PULL_FIRST.get().to_string(),
            Remedy::OpenIntegrations => g::ACTION_INTEGRATIONS.get().to_string(),
            Remedy::StashAndRetry(_) => g::ACTION_STASH_RETRY.get().to_string(),
            Remedy::Retry(_) => g::ACTION_RETRY.get().to_string(),
        }
    }
}

/// "Push failed" for the action that failed.
pub(crate) fn title_for(action: &Action) -> String {
    let action = match action {
        Action::AutoStashed(inner) => inner,
        other => other,
    };
    match action {
        Action::Push { .. }
        | Action::PushForReview { .. }
        | Action::PullThenPush { .. }
        | Action::FetchThenForcePush { .. } => g::TITLE_PUSH,
        Action::Pull(_) => g::TITLE_PULL,
        Action::Fetch(_) | Action::FetchAll => g::TITLE_FETCH,
        Action::CheckoutBranch(_) | Action::CheckoutCommit(_) | Action::CheckoutThen { .. } => {
            g::TITLE_CHECKOUT
        }
        Action::Merge(_) => g::TITLE_MERGE,
        Action::Rebase(_) | Action::RebaseInteractive(_) => g::TITLE_REBASE,
        Action::Commit { .. } | Action::ComposeCommits { .. } => g::TITLE_COMMIT,
        Action::StashPush
        | Action::StashPushWith { .. }
        | Action::StashFile(_)
        | Action::StashPop(_)
        | Action::StashApply(_) => g::TITLE_STASH,
        _ => g::TITLE_OTHER,
    }
    .get()
    .to_string()
}

/// The short "done" title of a push or a pull the user asked for (not the
/// timed fetches): the Activity Log's entry title.
pub(crate) fn done_title(action: &Action) -> Option<String> {
    let action = match action {
        Action::AutoStashed(inner) => inner,
        other => other,
    };
    match action {
        Action::Push { .. } | Action::PullThenPush { .. } | Action::FetchThenForcePush { .. } => {
            Some(g::DONE_PUSH.get().to_string())
        }
        Action::PushForReview { .. } => Some(g::DONE_PUSH_REVIEW.get().to_string()),
        Action::Pull(_) => Some(g::DONE_PULL.get().to_string()),
        _ => None,
    }
}

/// How the failure reads: from the detail git left, else from the typed
/// error itself.
pub(crate) fn kind_of(error: &CoreError) -> FailureKind {
    match error {
        CoreError::NotFastForward => FailureKind::NonFastForward,
        CoreError::DirtyWorkingTree { .. } => FailureKind::WouldOverwrite,
        CoreError::Conflict { .. } => FailureKind::MergeConflict,
        other => match other.detail() {
            Some(detail) => detail.kind(),
            // A plain message: git's words may still be in it.
            None => FailureKind::classify(&other.to_string()),
        },
    }
}

/// The one-line reason and the suggested step for `kind`; `None` when the
/// failure was not recognized (the entry then shows the raw message).
pub(crate) fn wording(kind: FailureKind) -> Option<(String, String)> {
    let (summary, hint) = match kind {
        FailureKind::NonFastForward => (g::SUMMARY_NON_FF, g::HINT_NON_FF),
        FailureKind::AuthFailed => (g::SUMMARY_AUTH, g::HINT_AUTH),
        FailureKind::MergeConflict => (g::SUMMARY_CONFLICT, g::HINT_CONFLICT),
        FailureKind::WouldOverwrite => (g::SUMMARY_OVERWRITE, g::HINT_OVERWRITE),
        FailureKind::NetworkUnreachable => (g::SUMMARY_NETWORK, g::HINT_NETWORK),
        FailureKind::LockExists => (g::SUMMARY_LOCK, g::HINT_LOCK),
        FailureKind::Other => return None,
    };
    Some((summary.get().to_string(), hint.get().to_string()))
}

/// Whether the daemon's auto-stash applies to `action` (it only reads the
/// flag for these).
fn stashable(action: &Action) -> bool {
    matches!(
        action,
        Action::Pull(_)
            | Action::CheckoutBranch(_)
            | Action::CheckoutCommit(_)
            | Action::CheckoutThen { .. }
            | Action::Merge(_)
            | Action::Rebase(_)
    )
}

/// The button, if any, that does something about the failure.
pub(crate) fn remedy(kind: FailureKind, action: Option<&Action>) -> Option<Remedy> {
    let pushing = action.is_some_and(|a| {
        matches!(
            a,
            Action::Push { .. } | Action::PullThenPush { .. } | Action::FetchThenForcePush { .. }
        )
    });
    match kind {
        FailureKind::NonFastForward if pushing => Some(Remedy::PullFirst),
        FailureKind::AuthFailed => Some(Remedy::OpenIntegrations),
        FailureKind::WouldOverwrite => action
            .filter(|a| stashable(a))
            .map(|a| Remedy::StashAndRetry(a.clone())),
        FailureKind::NetworkUnreachable => action.map(|a| Remedy::Retry(a.clone())),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::PullOptions;

    #[test]
    fn every_recognized_failure_has_a_reason_and_a_step_and_other_has_none() {
        for kind in [
            FailureKind::NonFastForward,
            FailureKind::AuthFailed,
            FailureKind::MergeConflict,
            FailureKind::WouldOverwrite,
            FailureKind::NetworkUnreachable,
            FailureKind::LockExists,
        ] {
            let (summary, hint) = wording(kind).unwrap_or_else(|| panic!("{kind:?}"));
            assert!(!summary.is_empty() && !hint.is_empty());
        }
        assert!(wording(FailureKind::Other).is_none());
    }

    #[test]
    fn the_button_follows_the_failure_and_the_action() {
        let pull = Action::Pull(PullOptions::default());
        let push = Action::Push {
            remote: "origin".into(),
            opts: Default::default(),
        };
        assert_eq!(
            remedy(FailureKind::NonFastForward, Some(&push)),
            Some(Remedy::PullFirst)
        );
        assert_eq!(
            remedy(FailureKind::NonFastForward, Some(&pull)),
            None,
            "pull first makes no sense for a pull"
        );
        assert_eq!(
            remedy(FailureKind::AuthFailed, Some(&push)),
            Some(Remedy::OpenIntegrations)
        );
        assert_eq!(
            remedy(FailureKind::WouldOverwrite, Some(&pull)),
            Some(Remedy::StashAndRetry(pull.clone()))
        );
        assert_eq!(
            remedy(FailureKind::WouldOverwrite, Some(&push)),
            None,
            "the daemon only auto-stashes checkout, merge, rebase and pull"
        );
        assert_eq!(
            remedy(FailureKind::NetworkUnreachable, Some(&push)),
            Some(Remedy::Retry(push))
        );
        assert_eq!(remedy(FailureKind::LockExists, Some(&pull)), None);
    }

    #[test]
    fn typed_errors_read_without_a_detail() {
        assert_eq!(
            kind_of(&CoreError::NotFastForward),
            FailureKind::NonFastForward
        );
        assert_eq!(
            kind_of(&CoreError::Backend("boom".into())),
            FailureKind::Other
        );
        assert_eq!(
            kind_of(&CoreError::Backend(
                "git pull failed: Could not resolve host: x".into()
            )),
            FailureKind::NetworkUnreachable
        );
    }

    #[test]
    fn titles_name_the_operation_even_inside_an_auto_stash() {
        use gitpoulpi_i18n::msg::git_error as g;
        let pull = Action::Pull(PullOptions::default());
        assert_eq!(title_for(&pull), g::TITLE_PULL.get());
        assert_eq!(
            title_for(&Action::AutoStashed(Box::new(pull))),
            g::TITLE_PULL.get()
        );
        assert_eq!(title_for(&Action::StageAll), g::TITLE_OTHER.get());
    }
}
