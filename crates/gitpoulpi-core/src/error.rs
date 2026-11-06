use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::domain::{
    ConflictRefusal, LfsUnavailableReason, PushRejection, RebaseRefusal, RewordRefusal,
    RewriteRefusal, SplitRefusal, UndoRefusal,
};
use crate::failure::ErrorDetail;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("operation not supported by this backend: {operation}")]
    Unsupported { operation: &'static str },

    #[error("repository not found at {0}")]
    RepoNotFound(PathBuf),

    #[error("working tree is not clean, operation requires a clean tree: {operation}")]
    DirtyWorkingTree { operation: &'static str },

    #[error("conflict encountered during {operation}")]
    Conflict { operation: &'static str },

    /// The remote refused a push in a way the user can act on: it has
    /// commits we do not (`NonFastForward`), or a lease found it moved
    /// (`StaleLease`). A variant rather than a `Backend` string because
    /// the TUI answers it with a dialog (pull first / force with lease /
    /// cancel), and a dialog keyed on git's English stderr would break
    /// on the first localised git.
    #[error("{} rejected the push of {}: {}", .0.remote, .0.branch, reject_reason(.0))]
    PushRejected(PushRejection),

    /// A fast-forward-only pull found the branch and its upstream
    /// diverged, or `GitBackend::fast_forward` found the branch and its
    /// target diverged. Nothing was changed.
    #[error("cannot fast-forward: the two have diverged")]
    NotFastForward,

    /// `reword` refused before changing anything (see
    /// [`RewordRefusal`]). Typed because the TUI words each case
    /// differently and the CLI reports it as a stable code.
    #[error("cannot change that commit: {}", reword_reason(*.0))]
    RewordRefused(RewordRefusal),
    /// `split_commit` refused before changing anything (see
    /// [`SplitRefusal`]).
    #[error("cannot split that commit: {}", split_reason(*.0))]
    SplitRefused(SplitRefusal),
    /// A history rewrite refused before changing anything (see
    /// [`RewriteRefusal`]); the string says which rule, path or ref.
    #[error("cannot rewrite history: {}{}", rewrite_reason(*.0), rewrite_detail(.1))]
    RewriteRefused(RewriteRefusal, String),

    /// An interactive rebase (or `continue`/`skip`/`abort`) refused
    /// before changing anything (see [`RebaseRefusal`]).
    #[error("cannot rebase: {}", rebase_reason(*.0))]
    RebaseRefused(RebaseRefusal),

    /// A conflict call (`resolve_conflict`, `unresolve_conflict`,
    /// `operation_abort`) refused before changing anything (see
    /// [`ConflictRefusal`]).
    #[error("cannot resolve: {}", .0.message())]
    ConflictRefused(ConflictRefusal),

    /// An undo or redo (or a compare-and-swap ref update) refused before
    /// changing anything (see [`UndoRefusal`]).
    #[error("cannot undo/redo: {}", .0.message())]
    UndoRefused(UndoRefusal),

    /// An undo or redo that is risky (it would overwrite working-tree
    /// files, or the undone commit is already on its upstream) and was
    /// not sent with `confirmed`. Nothing was changed; the prompt names
    /// what is at stake.
    #[error("confirmation needed: {0}")]
    NeedsConfirmation(String),

    /// An LFS operation needs the `git-lfs` program and it is missing (or
    /// too old). Typed so every frontend can show the install link
    /// instead of git's raw "'lfs' is not a git command".
    #[error("{}", .reason.message())]
    LfsUnavailable { reason: LfsUnavailableReason },

    /// The LFS remote does not implement the locking API.
    #[error("the LFS server behind {remote} does not support file locking")]
    LfsLockingUnsupported { remote: String },

    /// `operation` was blocked by local changes, `general.auto_stash` is
    /// `ask`, and nobody was there to answer. Nothing was changed. A UI
    /// answers with its dialog and resends the request with
    /// `autostash: "always"`; a CLI/MCP caller reads the message and
    /// passes `--autostash`. Typed so the TUI never matches on git's
    /// English stderr.
    #[error(
        "{operation} is blocked by local changes; stash them automatically \
         (--autostash, or `general.auto_stash = \"always\"`) or commit or discard them first"
    )]
    NeedsAutoStash { operation: String },

    /// The repository has a pending auto-stash (`stash` is its index in
    /// the stash list) and `operation` would bury or tangle it. Nothing
    /// was changed. `hint` says how to resolve it. Typed so each frontend
    /// can point at its own resolving actions.
    #[error("{operation} is blocked: an auto-stash (stash@{{{}}}) is still waiting; {hint}", .stash.0)]
    PendingAutoStash {
        operation: String,
        stash: crate::domain::StashId,
        hint: String,
    },

    /// The operation belongs to a feature this binary was built without
    /// (a custom build, see `docs/user/custom-builds.md`). `feature` is the
    /// cargo feature name (`lfs`, `worktree`, ...). Typed so each frontend
    /// words it in its own language and the CLI reports a stable code.
    #[error("{feature} is not built into this binary")]
    NotBuilt { feature: &'static str },

    /// A background job was canceled (`JobCancel`, or a client that asked for
    /// cancel-on-disconnect went away). The repository is as it was before the
    /// operation, or at its last safe point (see the cancel policy in
    /// the background-ops design`). Every caller waiting on the
    /// job, the one that started it and the ones that joined it, gets this.
    #[error("canceled (job {job})")]
    Canceled { job: u64 },

    /// `JobCancel` on a job that cannot be stopped safely right now (it is
    /// past its point of no return, or it has already ended). Nothing was
    /// changed.
    #[error("cannot be canceled: {}", .reason.message())]
    NotCancellable { reason: NotCancellableReason },

    /// A second request for a job that must not run twice (a clone into a
    /// destination another clone is filling). `existing` is the job already
    /// running.
    #[error("already running as job {existing}")]
    JobConflict { existing: u64 },

    /// The committed workflow hooks were asked to run but this user has not
    /// trusted them as they are now. Nothing ran. `hash` is what
    /// `FlowTrustHooks` takes, `hooks` the exact commands.
    #[error(
        "the repository's workflow hooks are not trusted ({} command(s), hash {hash}); review \
         them, then `gitpoulpi flow trust-hooks --hash {hash}`",
        .hooks.len()
    )]
    HooksNotTrusted {
        hash: String,
        hooks: Vec<crate::workflow::HookInfo>,
    },

    /// A stage request would add files that look sensitive (a private key, a
    /// `.env`, a recognised token) and the caller did not allow it. Nothing
    /// was staged. `files` lists each file with its reason, never a value.
    #[error(
        "sensitive files were not staged: {}; ask the user, then stage again allowing \
         sensitive files",
        crate::privacy::summary(.files)
    )]
    SensitiveStage {
        files: Vec<crate::privacy::Sensitive>,
    },

    #[error("backend error: {0}")]
    Backend(String),

    /// A backend failure with its facts attached (the command that ran, its
    /// exit code and stderr, or libgit2's class and code), so a frontend can
    /// say what went wrong instead of "the backend failed". `Display` is
    /// the same text a plain `Backend` error would have.
    #[error("backend error: {message}")]
    Git {
        message: String,
        detail: Box<ErrorDetail>,
    },

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Why `JobCancel` refused ([`Error::NotCancellable`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotCancellableReason {
    /// A pull past its fetch: the merge, rebase or fast-forward is writing
    /// the index and the working tree.
    Integrating,
    /// Files are being rewritten in the working tree (LFS checkout).
    RewritingFiles,
    /// A multi-step workflow: it can only stop between steps.
    InSteps,
    /// A client-side job past its send: the host may already have acted
    /// (a pull request or issue being created).
    Sent,
    /// Old history is being deleted (`rewrite purge`): the pins, the
    /// reflogs and the objects go, and stopping half way would leave an
    /// undo entry without its objects.
    Purging,
    /// The job has already ended.
    Finished,
    /// No such job (never existed, or already dropped from the list).
    UnknownJob,
}

impl NotCancellableReason {
    pub fn message(self) -> &'static str {
        match self {
            Self::Integrating => "merging - cannot stop safely",
            Self::RewritingFiles => "rewriting files - cannot stop safely",
            Self::InSteps => "running a multi-step workflow - cannot stop between its steps yet",
            Self::Sent => "sent to the host, waiting for its answer",
            Self::Purging => "deleting the old history - cannot stop safely",
            Self::Finished => "it has already ended",
            Self::UnknownJob => "no such job",
        }
    }
}

/// The cargo features `NotBuilt` can name. A name off the wire that is not
/// in the list (a newer peer) becomes `"unknown"` rather than leaking.
pub const BUILD_FEATURES: &[&str] = &[
    "ai",
    "hosting",
    "github",
    "gitlab",
    "mcp",
    "lfs",
    "worktree",
    "submodule",
    "avatar-fetch",
    "history-rewrite",
];

fn known_feature(name: &str) -> &'static str {
    BUILD_FEATURES
        .iter()
        .find(|f| **f == name)
        .copied()
        .unwrap_or("unknown")
}

fn reject_reason(rejection: &PushRejection) -> &'static str {
    match rejection.reason {
        crate::domain::PushRejectReason::NonFastForward => {
            "the remote has commits this branch does not (pull first, or force push)"
        }
        crate::domain::PushRejectReason::StaleLease => {
            "the remote branch moved since the last fetch, so the force push was refused"
        }
    }
}

fn reword_reason(refusal: RewordRefusal) -> &'static str {
    match refusal {
        RewordRefusal::NotOnCurrentBranch => {
            "the commit is not on the current branch (it is not HEAD or an ancestor of HEAD)"
        }
        RewordRefusal::OperationInProgress => {
            "a merge, rebase, cherry-pick, revert or bisect is in progress"
        }
        RewordRefusal::EmptyMessage => "the new message is empty",
        RewordRefusal::InvalidIdentity => {
            "a name or email is empty or has a character that cannot be stored (< > or a line break)"
        }
        RewordRefusal::InvalidDate => "a date is not in the raw format or ISO 8601",
    }
}

fn rewrite_detail(detail: &str) -> String {
    if detail.is_empty() {
        String::new()
    } else {
        format!(" ({detail})")
    }
}

fn rewrite_reason(refusal: RewriteRefusal) -> &'static str {
    use RewriteRefusal as R;
    match refusal {
        R::OperationInProgress => "a merge, rebase, cherry-pick, revert or bisect is in progress",
        R::AutoStashPending => "an auto-stash is waiting to be restored",
        R::Shallow => "this is a shallow clone, its history is incomplete",
        R::PartialClone => "this is a partial clone and the rules read file content",
        R::UnsupportedObjectFormat => "SHA-256 repositories are not supported",
        R::EmptyScope => "the scope contains no commit or ref",
        R::NoRule => "no rule is enabled",
        R::InvalidRule => "a rule is not valid",
        R::DirtyPathsTouched => "files the rewrite would change have uncommitted edits",
        R::CheckedOutElsewhere => "a branch is checked out in another worktree",
        R::ProtectedRef => "a pushed protected branch is in scope and was not included",
        R::Stale => "the repository changed since the preview, preview again",
        R::ToolMissing => "a tool the rules need is not installed",
        R::ScriptFailed => "a script failed",
        R::Purged => "the old history was purged, there is nothing to restore",
        R::ScriptsDisabled => {
            "this kind of script is switched off in the settings (rewrite.external_scripts)"
        }
        R::ScriptNotTrusted => "the script is not trusted yet",
        R::SignatureChoiceNeeded => {
            "signed commits are affected: choose to strip, re-sign signed or re-sign all"
        }
        R::ExtractModeNeeded => "choose how to extract: clone, in place or submodule",
        R::SigningFailed => {
            "signing does not work here (key, agent or program), check the signing setup"
        }
        R::LfsObjectsMissing => {
            "LFS objects the export needs are not in the local LFS store: fetch them first"
        }
        R::PathCollision => "a path rule would put a file where one already exists",
    }
}

fn split_reason(refusal: SplitRefusal) -> &'static str {
    match refusal {
        SplitRefusal::NotOnCurrentBranch => {
            "the commit is not on the current branch (it is not HEAD or an ancestor of HEAD)"
        }
        SplitRefusal::MergeCommit => "a merge commit cannot be split",
        SplitRefusal::EmptyCommit => "the commit changes nothing",
        SplitRefusal::OperationInProgress => {
            "a merge, rebase, cherry-pick, revert or bisect is in progress"
        }
        SplitRefusal::TooFewParts => "a split needs at least two commits",
        SplitRefusal::EmptyPart => "a new commit has no changes assigned to it",
        SplitRefusal::EmptyMessage => "a new commit has an empty message",
        SplitRefusal::Unassigned => "some changes are not assigned to any commit",
        SplitRefusal::AssignedTwice => "some changes are assigned to more than one commit",
        SplitRefusal::UnknownPiece => "a piece does not exist in the commit or cannot be selected",
        SplitRefusal::WholeFileOnly => "that file can only be assigned as a whole",
        SplitRefusal::SigningFailed => "signing the new commits failed",
    }
}

fn rebase_reason(refusal: RebaseRefusal) -> &'static str {
    match refusal {
        RebaseRefusal::OperationInProgress => {
            "a merge, rebase, cherry-pick, revert or bisect is already in progress"
        }
        RebaseRefusal::UncommittedChanges => "the working tree has uncommitted changes",
        RebaseRefusal::NotOnCurrentBranch => {
            "the target is not on the current branch, or HEAD is detached"
        }
        RebaseRefusal::MergeInRange => "the range contains a merge commit",
        RebaseRefusal::PlanMismatch => {
            "the plan is not exactly a reordering of the commits being rebased"
        }
        RebaseRefusal::SquashWithoutPredecessor => {
            "the oldest step cannot be squash or fixup: it has nothing to combine into"
        }
        RebaseRefusal::EmptyRewordMessage => "a reword step has no message",
        RebaseRefusal::NoRebaseInProgress => "no rebase is in progress",
    }
}

/// The part of an [`Error`] a client must be able to *match on*, in a
/// shape that crosses the IPC boundary.
///
/// `Error` itself is not serialisable (it wraps an `io::Error`) and most
/// of it is only ever shown to a human, so the wire carries the message
/// plus, for the handful of errors a frontend reacts to with more than a
/// status line, this. A client rebuilds the typed `Error` with
/// [`Error::from_typed`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypedError {
    PushRejected(PushRejection),
    NotFastForward,
    RewordRefused(RewordRefusal),
    SplitRefused(SplitRefusal),
    RewriteRefused(RewriteRefusal, String),
    RebaseRefused(RebaseRefusal),
    ConflictRefused(ConflictRefusal),
    UndoRefused(UndoRefusal),
    NeedsConfirmation(String),
    LfsUnavailable(LfsUnavailableReason),
    LfsLockingUnsupported {
        remote: String,
    },
    NeedsAutoStash {
        operation: String,
    },
    PendingAutoStash {
        operation: String,
        stash: crate::domain::StashId,
        hint: String,
    },
    /// The feature (cargo feature name) this binary was built without.
    NotBuilt {
        feature: String,
    },
    Canceled {
        job: u64,
    },
    NotCancellable {
        reason: NotCancellableReason,
    },
    JobConflict {
        existing: u64,
    },
    HooksNotTrusted {
        hash: String,
        hooks: Vec<crate::workflow::HookInfo>,
    },
    SensitiveStage {
        files: Vec<crate::privacy::Sensitive>,
    },
}

impl Error {
    /// The typed part of this error, if it has one.
    pub fn typed(&self) -> Option<TypedError> {
        match self {
            Error::PushRejected(r) => Some(TypedError::PushRejected(r.clone())),
            Error::NotFastForward => Some(TypedError::NotFastForward),
            Error::RewordRefused(r) => Some(TypedError::RewordRefused(*r)),
            Error::SplitRefused(r) => Some(TypedError::SplitRefused(*r)),
            Error::RewriteRefused(r, d) => Some(TypedError::RewriteRefused(*r, d.clone())),
            Error::RebaseRefused(r) => Some(TypedError::RebaseRefused(*r)),
            Error::ConflictRefused(r) => Some(TypedError::ConflictRefused(r.clone())),
            Error::UndoRefused(r) => Some(TypedError::UndoRefused(r.clone())),
            Error::NeedsConfirmation(p) => Some(TypedError::NeedsConfirmation(p.clone())),
            Error::LfsUnavailable { reason } => Some(TypedError::LfsUnavailable(reason.clone())),
            Error::LfsLockingUnsupported { remote } => Some(TypedError::LfsLockingUnsupported {
                remote: remote.clone(),
            }),
            Error::NeedsAutoStash { operation } => Some(TypedError::NeedsAutoStash {
                operation: operation.clone(),
            }),
            Error::PendingAutoStash {
                operation,
                stash,
                hint,
            } => Some(TypedError::PendingAutoStash {
                operation: operation.clone(),
                stash: *stash,
                hint: hint.clone(),
            }),
            Error::NotBuilt { feature } => Some(TypedError::NotBuilt {
                feature: (*feature).to_string(),
            }),
            Error::Canceled { job } => Some(TypedError::Canceled { job: *job }),
            Error::NotCancellable { reason } => {
                Some(TypedError::NotCancellable { reason: *reason })
            }
            Error::JobConflict { existing } => Some(TypedError::JobConflict {
                existing: *existing,
            }),
            Error::HooksNotTrusted { hash, hooks } => Some(TypedError::HooksNotTrusted {
                hash: hash.clone(),
                hooks: hooks.clone(),
            }),
            Error::SensitiveStage { files } => Some(TypedError::SensitiveStage {
                files: files.clone(),
            }),
            _ => None,
        }
    }

    /// The facts of the failure, when the backend recorded them.
    pub fn detail(&self) -> Option<&ErrorDetail> {
        match self {
            Error::Git { detail, .. } => Some(detail),
            Error::PushRejected(r) => r.detail.as_deref(),
            _ => None,
        }
    }

    /// A backend failure carrying `detail`.
    pub fn git(message: impl Into<String>, detail: ErrorDetail) -> Self {
        Error::Git {
            message: message.into(),
            detail: Box::new(detail),
        }
    }

    /// [`Error::from_wire_message`] with the detail that travelled beside
    /// the message, when there was one.
    pub fn from_wire(message: String, detail: Option<ErrorDetail>) -> Self {
        match (Self::from_wire_message(message), detail) {
            (Error::Backend(message), Some(detail)) => Error::git(message, detail),
            (other, _) => other,
        }
    }

    /// An error that arrived over the wire as plain text: the daemon sent
    /// `Error`'s own `Display`, which for a backend failure already starts
    /// with `backend error: `. Wrapping that text in [`Error::Backend`]
    /// again would print the prefix twice, so it is taken off first.
    pub fn from_wire_message(message: String) -> Self {
        match message.strip_prefix("backend error: ") {
            Some(rest) => Error::Backend(rest.to_string()),
            None => Error::Backend(message),
        }
    }

    /// The inverse of [`Error::typed`].
    pub fn from_typed(typed: TypedError) -> Self {
        match typed {
            TypedError::PushRejected(r) => Error::PushRejected(r),
            TypedError::NotFastForward => Error::NotFastForward,
            TypedError::RewordRefused(r) => Error::RewordRefused(r),
            TypedError::SplitRefused(r) => Error::SplitRefused(r),
            TypedError::RewriteRefused(r, d) => Error::RewriteRefused(r, d),
            TypedError::RebaseRefused(r) => Error::RebaseRefused(r),
            TypedError::ConflictRefused(r) => Error::ConflictRefused(r),
            TypedError::UndoRefused(r) => Error::UndoRefused(r),
            TypedError::NeedsConfirmation(p) => Error::NeedsConfirmation(p),
            TypedError::LfsUnavailable(reason) => Error::LfsUnavailable { reason },
            TypedError::LfsLockingUnsupported { remote } => Error::LfsLockingUnsupported { remote },
            TypedError::NeedsAutoStash { operation } => Error::NeedsAutoStash { operation },
            TypedError::PendingAutoStash {
                operation,
                stash,
                hint,
            } => Error::PendingAutoStash {
                operation,
                stash,
                hint,
            },
            TypedError::NotBuilt { feature } => Error::NotBuilt {
                feature: known_feature(&feature),
            },
            TypedError::Canceled { job } => Error::Canceled { job },
            TypedError::NotCancellable { reason } => Error::NotCancellable { reason },
            TypedError::JobConflict { existing } => Error::JobConflict { existing },
            TypedError::HooksNotTrusted { hash, hooks } => Error::HooksNotTrusted { hash, hooks },
            TypedError::SensitiveStage { files } => Error::SensitiveStage { files },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::PushRejectReason;

    /// The daemon sends `Error`'s Display text; wrapping it again must not
    /// print "backend error: backend error: ...".
    #[test]
    fn a_backend_error_crossing_the_wire_keeps_one_prefix() {
        let sent = Error::Backend("x is not in commit abc".into()).to_string();
        assert_eq!(sent, "backend error: x is not in commit abc");
        let received = Error::from_wire_message(sent.clone());
        assert_eq!(received.to_string(), sent);
        // Text that never had the prefix is still wrapped once.
        assert_eq!(
            Error::from_wire_message("boom".into()).to_string(),
            "backend error: boom"
        );
    }

    /// A typed error survives the trip through JSON and back into an
    /// `Error` - the round trip every frontend depends on to tell "the
    /// remote diverged" from any other failure.
    #[test]
    fn a_typed_error_round_trips_through_json() {
        let original = Error::PushRejected(PushRejection {
            remote: "origin".into(),
            branch: "main".into(),
            reason: PushRejectReason::NonFastForward,
            detail: None,
        });
        let wire = serde_json::to_string(&original.typed().unwrap()).unwrap();
        let back = Error::from_typed(serde_json::from_str(&wire).unwrap());
        assert!(matches!(
            back,
            Error::PushRejected(PushRejection { ref remote, reason: PushRejectReason::NonFastForward, .. })
                if remote == "origin"
        ));
        assert!(Error::Backend("x".into()).typed().is_none());
    }

    #[test]
    fn split_refusals_round_trip_through_json() {
        let original = Error::SplitRefused(SplitRefusal::Unassigned);
        let wire = serde_json::to_string(&original.typed().unwrap()).unwrap();
        let back = Error::from_typed(serde_json::from_str(&wire).unwrap());
        assert!(matches!(
            back,
            Error::SplitRefused(SplitRefusal::Unassigned)
        ));
        assert!(back.to_string().contains("not assigned"));
    }

    #[test]
    fn rewrite_refusals_round_trip_through_json_with_their_detail() {
        let original = Error::RewriteRefused(
            RewriteRefusal::InvalidRule,
            "rule 2: the pattern is empty".into(),
        );
        let wire = serde_json::to_string(&original.typed().unwrap()).unwrap();
        let back = Error::from_typed(serde_json::from_str(&wire).unwrap());
        match &back {
            Error::RewriteRefused(RewriteRefusal::InvalidRule, detail) => {
                assert_eq!(detail, "rule 2: the pattern is empty");
            }
            other => panic!("{other:?}"),
        }
        assert!(back.to_string().contains("a rule is not valid"));
        assert!(back.to_string().contains("rule 2"));
        // No detail: no empty parentheses.
        let bare = Error::RewriteRefused(RewriteRefusal::NoRule, String::new());
        assert_eq!(
            bare.to_string(),
            "cannot rewrite history: no rule is enabled"
        );
    }

    #[test]
    fn lfs_errors_round_trip_and_name_the_install_page() {
        let original = Error::LfsUnavailable {
            reason: LfsUnavailableReason::Missing,
        };
        assert!(original.to_string().contains("git-lfs.com"));
        let wire = serde_json::to_string(&original.typed().unwrap()).unwrap();
        let back = Error::from_typed(serde_json::from_str(&wire).unwrap());
        assert!(matches!(
            back,
            Error::LfsUnavailable {
                reason: LfsUnavailableReason::Missing
            }
        ));
        let locking = Error::LfsLockingUnsupported {
            remote: "origin".into(),
        };
        let wire = serde_json::to_string(&locking.typed().unwrap()).unwrap();
        assert!(matches!(
            Error::from_typed(serde_json::from_str(&wire).unwrap()),
            Error::LfsLockingUnsupported { ref remote } if remote == "origin"
        ));
    }

    #[test]
    fn job_errors_round_trip_with_stable_kinds() {
        let canceled = Error::Canceled { job: 7 };
        let wire = serde_json::to_string(&canceled.typed().unwrap()).unwrap();
        assert_eq!(wire, r#"{"Canceled":{"job":7}}"#);
        assert!(matches!(
            Error::from_typed(serde_json::from_str(&wire).unwrap()),
            Error::Canceled { job: 7 }
        ));
        let refused = Error::NotCancellable {
            reason: NotCancellableReason::Integrating,
        };
        assert!(refused.to_string().contains("merging"));
        assert!(matches!(
            Error::from_typed(refused.typed().unwrap()),
            Error::NotCancellable {
                reason: NotCancellableReason::Integrating
            }
        ));
        assert!(matches!(
            Error::from_typed(Error::JobConflict { existing: 3 }.typed().unwrap()),
            Error::JobConflict { existing: 3 }
        ));
    }

    #[test]
    fn hooks_not_trusted_round_trips_with_its_commands() {
        let original = Error::HooksNotTrusted {
            hash: "ab".repeat(32),
            hooks: vec![crate::workflow::HookInfo {
                phase: "after_start".into(),
                command: "make".into(),
                personal: false,
            }],
        };
        assert!(original.to_string().contains("not trusted"));
        let wire = serde_json::to_string(&original.typed().unwrap()).unwrap();
        match Error::from_typed(serde_json::from_str(&wire).unwrap()) {
            Error::HooksNotTrusted { hash, hooks } => {
                assert_eq!(hash.len(), 64);
                assert_eq!(hooks[0].command, "make");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn sensitive_stage_round_trips_with_reasons_and_no_value() {
        use crate::privacy::{Reason, Sensitive};
        let original = Error::SensitiveStage {
            files: vec![
                Sensitive {
                    path: ".env".into(),
                    reason: Reason::Pattern(".env".into()),
                },
                Sensitive {
                    path: "a.txt".into(),
                    reason: Reason::Secret {
                        rule: "github-token".into(),
                        preview: "ghp_****".into(),
                    },
                },
            ],
        };
        assert!(original.to_string().contains(".env, a.txt"));
        let wire = serde_json::to_string(&original.typed().unwrap()).unwrap();
        match Error::from_typed(serde_json::from_str(&wire).unwrap()) {
            Error::SensitiveStage { files } => {
                assert_eq!(files.len(), 2);
                assert_eq!(files[1].path, "a.txt");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn not_built_round_trips_and_serialises_with_a_stable_kind() {
        let original = Error::NotBuilt { feature: "lfs" };
        assert_eq!(original.to_string(), "lfs is not built into this binary");
        let typed = original.typed().unwrap();
        let wire = serde_json::to_string(&typed).unwrap();
        assert_eq!(wire, r#"{"NotBuilt":{"feature":"lfs"}}"#);
        assert!(matches!(
            Error::from_typed(serde_json::from_str(&wire).unwrap()),
            Error::NotBuilt { feature: "lfs" }
        ));
        // A feature this build does not know is not leaked as a &'static str.
        assert!(matches!(
            Error::from_typed(TypedError::NotBuilt {
                feature: "zzz".into()
            }),
            Error::NotBuilt { feature: "unknown" }
        ));
    }
}
