//! A failure in the user's words: a title for the operation, a one-line
//! reason and a suggested next step, and the buttons that can do the step.
//!
//! The reading of git's message (`FailureKind`) lives in `gitpoulpi-core`;
//! only the wording and the choice of buttons are here.

use gitpoulpi_core::domain::ConflictRefusal;
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
    /// Open Settings on this repository's SSH & credentials page.
    OpenRepoAuth,
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
            Remedy::OpenRepoAuth => g::ACTION_REPO_AUTH.get().to_string(),
            Remedy::StashAndRetry(_) => g::ACTION_STASH_RETRY.get().to_string(),
            Remedy::Retry(_) => g::ACTION_RETRY.get().to_string(),
        }
    }
}

/// The failed action and its arguments, for the error report: the variant's
/// debug text, redacted and cut at a length a chat can take.
pub(crate) fn action_text(action: &Action) -> String {
    const MAX: usize = 400;
    let text = gitpoulpi_core::redact(&format!("{action:?}"));
    if text.chars().count() <= MAX {
        return text;
    }
    let cut: String = text.chars().take(MAX).collect();
    format!("{cut}...")
}

/// "Push failed" for the action that failed.
pub(crate) fn title_for(action: &Action) -> String {
    let action = match action {
        Action::AutoStashed(inner) | Action::AllowSensitive(inner) => inner,
        other => other,
    };
    match action {
        Action::Push { .. }
        | Action::PushForReview { .. }
        | Action::PullThenPush { .. }
        | Action::FetchThenForcePush { .. } => g::TITLE_PUSH,
        Action::Pull(_) => g::TITLE_PULL,
        Action::PullAllBranches(_) => g::TITLE_PULL_ALL_BRANCHES,
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
        Action::AutoStashed(inner) | Action::AllowSensitive(inner) => inner,
        other => other,
    };
    match action {
        Action::Push { .. } | Action::PullThenPush { .. } | Action::FetchThenForcePush { .. } => {
            Some(g::DONE_PUSH.get().to_string())
        }
        Action::PushForReview { .. } => Some(g::DONE_PUSH_REVIEW.get().to_string()),
        Action::Pull(_) => Some(g::DONE_PULL.get().to_string()),
        Action::PullAllBranches(_) => Some(g::DONE_PULL_ALL_BRANCHES.get().to_string()),
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
        CoreError::ConflictRefused(ConflictRefusal::LocalChanges { .. }) => {
            FailureKind::LocalChangesBlockAbort
        }
        other => match other.detail() {
            Some(detail) => detail.kind(),
            // A plain message: git's words may still be in it.
            None => FailureKind::classify(&other.to_string()),
        },
    }
}

/// The one-line reason and the suggested step for `kind`; `None` when the
/// failure was not recognized (the entry then shows the raw message).
/// `text` is the message (or stderr) the failure came from: a credential
/// helper failure quotes its offending line.
pub(crate) fn wording(kind: FailureKind, text: &str) -> Option<(String, String)> {
    if kind == FailureKind::CredentialHelperFailed {
        let line = FailureKind::credential_helper_line(text).unwrap_or_default();
        return Some((
            g::summary_credential_helper(line),
            g::HINT_CREDENTIAL_HELPER.get().to_string(),
        ));
    }
    let (summary, hint) = match kind {
        FailureKind::NonFastForward => (g::SUMMARY_NON_FF, g::HINT_NON_FF),
        FailureKind::AuthFailed => (g::SUMMARY_AUTH, g::HINT_AUTH),
        FailureKind::MergeConflict => (g::SUMMARY_CONFLICT, g::HINT_CONFLICT),
        FailureKind::WouldOverwrite => (g::SUMMARY_OVERWRITE, g::HINT_OVERWRITE),
        FailureKind::HostKeyUnverified => (g::SUMMARY_HOST_KEY, g::HINT_HOST_KEY),
        FailureKind::NetworkUnreachable => (g::SUMMARY_NETWORK, g::HINT_NETWORK),
        FailureKind::LockExists => (g::SUMMARY_LOCK, g::HINT_LOCK),
        // The typed refusal has its own (localised) sentence naming the files.
        FailureKind::CredentialHelperFailed
        | FailureKind::LocalChangesBlockAbort
        | FailureKind::Other => return None,
    };
    Some((summary.get().to_string(), hint.get().to_string()))
}

/// Whether the daemon's auto-stash applies to `action` (it only reads the
/// flag for these).
fn stashable(action: &Action) -> bool {
    matches!(
        action,
        Action::Pull(_)
            | Action::PullAllBranches(_)
            | Action::CheckoutBranch(_)
            | Action::CheckoutCommit(_)
            | Action::CheckoutThen { .. }
            | Action::Merge(_)
            | Action::Rebase(_)
    )
}

/// What an entry knows about the repository's sign-in, to choose its buttons.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct AuthContext {
    /// The failure is the repository's own sign-in settings' (a typed
    /// repo-auth error, a helper that could not run, an unknown host key) or
    /// happened inside a repository, where its SSH & credentials page can fix
    /// an authentication failure.
    pub(crate) repo_auth: bool,
    /// An integration account is (or may be) the source of the sign-in: the
    /// Integrations page can fix a refused token.
    pub(crate) integration: bool,
}

/// Whether `error` is one of the typed errors of a repository's sign-in
/// profile (they all lead to the SSH & credentials page).
pub(crate) fn is_repo_auth_error(error: &CoreError) -> bool {
    match error {
        CoreError::RepoAuthUnavailable { .. }
        | CoreError::GrantRefused { .. }
        | CoreError::CredentialHelperUnreachable
        | CoreError::RepoAuthCommandChanged { .. }
        | CoreError::TokenCommandFailed { .. } => true,
        CoreError::NeedsConfirmation(prompt) => {
            prompt == gitpoulpi_core::domain::COMMAND_NEEDS_PERSON
        }
        _ => false,
    }
}

/// The context for a failure of kind `kind` in a repository whose identity
/// is `identity` (`None`: everything default).
pub(crate) fn auth_context(
    kind: FailureKind,
    typed_repo_auth: bool,
    in_repository: bool,
    identity: Option<&gitpoulpi_core::domain::ActiveIdentity>,
) -> AuthContext {
    let repo_auth = typed_repo_auth
        || matches!(
            kind,
            FailureKind::CredentialHelperFailed | FailureKind::HostKeyUnverified
        )
        || (kind == FailureKind::AuthFailed && in_repository);
    AuthContext {
        repo_auth,
        // A default profile signs in through the integrations' tokens, so
        // the old button stays; an explicit key or helper does not.
        integration: !typed_repo_auth && identity.is_none_or(|i| i.integration),
    }
}

/// Every button of a failure, in order: the repository's own sign-in page
/// first, then what `remedy` offers. An authentication failure offers the
/// Integrations page only when an integration account may be its source.
pub(crate) fn remedies(
    kind: FailureKind,
    action: Option<&Action>,
    context: AuthContext,
) -> Vec<Remedy> {
    let mut out = Vec::new();
    if context.repo_auth {
        out.push(Remedy::OpenRepoAuth);
    }
    if kind == FailureKind::AuthFailed {
        if context.integration || !context.repo_auth {
            out.push(Remedy::OpenIntegrations);
        }
    } else if let Some(remedy) = remedy(kind, action) {
        out.push(remedy);
    }
    out
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
        FailureKind::NetworkUnreachable | FailureKind::LocalChangesBlockAbort => {
            action.map(|a| Remedy::Retry(a.clone()))
        }
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
            FailureKind::HostKeyUnverified,
            FailureKind::NetworkUnreachable,
            FailureKind::LockExists,
            FailureKind::CredentialHelperFailed,
        ] {
            let (summary, hint) =
                wording(kind, "sh: x: not found").unwrap_or_else(|| panic!("{kind:?}"));
            assert!(!summary.is_empty() && !hint.is_empty());
        }
        assert!(wording(FailureKind::Other, "").is_none());
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
    fn a_failing_credential_helper_is_named_with_its_line_and_the_command_to_check() {
        let message = "backend error: git fetch --porcelain --all --prune failed: sh: /c/Users/X/.config/my-tool/token.sh: No such file or directory\nremote: Invalid username or token. Password authentication is not supported for Git operations.\nfatal: Authentication failed for 'https://github.com/Poulpi-Lab/GitPoulpi.git/'";
        let error = CoreError::Backend(message.into());
        let kind = kind_of(&error);
        assert_eq!(kind, FailureKind::CredentialHelperFailed);
        let (summary, hint) = wording(kind, message).unwrap();
        assert!(
            summary.contains("sh: /c/Users/X/.config/my-tool/token.sh: No such file or directory"),
            "{summary}"
        );
        assert!(!summary.contains("refused the credentials"), "{summary}");
        assert!(
            hint.contains("git config --show-origin --get-all credential.helper"),
            "{hint}"
        );
        assert_eq!(remedy(kind, None), None);
        assert_eq!(
            remedies(kind, None, auth_context(kind, false, true, None)),
            vec![Remedy::OpenRepoAuth],
            "a helper that could not run leads to the page that picks another source"
        );
    }

    #[test]
    fn the_new_readings_of_the_repository_sign_in_each_lead_to_its_page() {
        use gitpoulpi_core::domain::{ActiveIdentity, COMMAND_NEEDS_PERSON};
        let typed = [
            CoreError::RepoAuthUnavailable {
                reason: "the key file does not exist".into(),
            },
            CoreError::GrantRefused {
                host: "evil.example".into(),
                expected: "github.com".into(),
            },
            CoreError::CredentialHelperUnreachable,
            CoreError::RepoAuthCommandChanged {
                path: "tools/token.sh".into(),
            },
            CoreError::TokenCommandFailed {
                exit: Some(3),
                stderr_tail: "vault is sealed".into(),
            },
            CoreError::NeedsConfirmation(COMMAND_NEEDS_PERSON.into()),
        ];
        for error in typed {
            assert!(is_repo_auth_error(&error), "{error}");
            let text = crate::daemon_link::error_text::error_text(&error);
            assert!(!text.is_empty() && text.is_ascii(), "{text}");
            assert!(
                !text.contains("token command failed (exit") || text.contains("vault is sealed"),
                "{text}"
            );
            assert_eq!(
                remedies(
                    kind_of(&error),
                    None,
                    auth_context(kind_of(&error), true, true, None)
                ),
                vec![Remedy::OpenRepoAuth],
                "{error}"
            );
        }
        // Another confirmation (an undo that overwrites files) is not about sign-in.
        assert!(!is_repo_auth_error(&CoreError::NeedsConfirmation(
            "it would overwrite files".into()
        )));
        // The sentences name what happened, not the English of the error type.
        let refused = crate::daemon_link::error_text::error_text(&CoreError::GrantRefused {
            host: "evil.example".into(),
            expected: "github.com".into(),
        });
        assert_eq!(
            refused,
            "GitPoulpi did not send the account token to evil.example: that is not github.com, the address this account is bound to."
        );
        let silent = crate::daemon_link::error_text::error_text(&CoreError::TokenCommandFailed {
            exit: None,
            stderr_tail: String::new(),
        });
        assert_eq!(silent, "This repository's token command failed.");
        // The host key and a refused login read the same way.
        let key = auth_context(FailureKind::HostKeyUnverified, false, true, None);
        assert_eq!(
            remedies(FailureKind::HostKeyUnverified, None, key),
            vec![Remedy::OpenRepoAuth]
        );
        let (_, host_hint) = wording(FailureKind::HostKeyUnverified, "").unwrap();
        assert!(host_hint.contains("known hosts"), "{host_hint}");
        // A refused login inside a repository: its page, and the
        // Integrations page unless the identity is a key or a command.
        let plain = auth_context(FailureKind::AuthFailed, false, true, None);
        assert_eq!(
            remedies(FailureKind::AuthFailed, None, plain),
            vec![Remedy::OpenRepoAuth, Remedy::OpenIntegrations]
        );
        let key_id = ActiveIdentity {
            ssh: Some(gitpoulpi_core::domain::SshIdentity {
                name: "work".into(),
                of_account: false,
            }),
            ..ActiveIdentity::default()
        };
        let keyed = auth_context(FailureKind::AuthFailed, false, true, Some(&key_id));
        assert_eq!(
            remedies(FailureKind::AuthFailed, None, keyed),
            vec![Remedy::OpenRepoAuth]
        );
        let account = ActiveIdentity {
            integration: true,
            ..key_id
        };
        let with_account = auth_context(FailureKind::AuthFailed, false, true, Some(&account));
        assert_eq!(
            remedies(FailureKind::AuthFailed, None, with_account),
            vec![Remedy::OpenRepoAuth, Remedy::OpenIntegrations]
        );
        // Outside a repository there is no page of its own: as before.
        let outside = auth_context(FailureKind::AuthFailed, false, false, None);
        assert_eq!(
            remedies(FailureKind::AuthFailed, None, outside),
            vec![Remedy::OpenIntegrations]
        );
    }

    #[test]
    fn an_abort_blocked_by_local_changes_reads_as_such_and_offers_a_retry() {
        let error = CoreError::ConflictRefused(ConflictRefusal::LocalChanges {
            paths: vec!["dev-mods/host.ts".into()],
        });
        let kind = kind_of(&error);
        assert_eq!(kind, FailureKind::LocalChangesBlockAbort);
        let abort = Action::OperationAbort;
        assert_eq!(remedy(kind, Some(&abort)), Some(Remedy::Retry(abort)));
        assert!(
            crate::daemon_link::error_text::error_text(&error).contains("dev-mods/host.ts"),
            "the reading names the file"
        );
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
