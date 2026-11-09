//! The daemon's typed errors and refusals, worded for the TUI.
//!
//! The daemon, the CLI and MCP keep the English `Display` text of
//! [`gitpoulpi_core::Error`] (agents and scripts read it). The TUI translates
//! only at its display edge: [`error_text`] maps the typed part of an error
//! (`Error::typed`, the same `TypedError` that crosses IPC) to the catalog's
//! `[typed_error]` section, and shows everything else (`Backend`, `Io`, git's
//! own stderr) as it is. In English the sentences are the `Display` text word
//! for word (a test pins that).

use gitpoulpi_core::domain::{
    short, ConflictRefusal, HeadState, LfsUnavailableReason, PushRejectReason, RebaseRefusal,
    RewordRefusal, SplitRefusal, UndoRefusal, LFS_INSTALL_URL, LFS_MIN_VERSION,
};
use gitpoulpi_core::{Error as CoreError, TypedError};
use gitpoulpi_i18n::msg;

/// What the TUI shows for `error`: translated when it is typed, as the
/// daemon worded it otherwise.
pub(crate) fn error_text(error: &CoreError) -> String {
    match error.typed() {
        Some(typed) => typed_text(&typed),
        None => error.to_string(),
    }
}

/// The message for one [`TypedError`].
pub(crate) fn typed_text(typed: &TypedError) -> String {
    match typed {
        TypedError::PushRejected(r) => msg::typed_error::push_rejected(
            &r.remote,
            &r.branch,
            match r.reason {
                PushRejectReason::NonFastForward => msg::typed_error::PUSH_NON_FAST_FORWARD,
                PushRejectReason::StaleLease => msg::typed_error::PUSH_STALE_LEASE,
            },
        ),
        TypedError::NotFastForward => msg::typed_error::NOT_FAST_FORWARD.get().to_string(),
        TypedError::RewordRefused(r) => msg::typed_error::reword_refused(match r {
            RewordRefusal::NotOnCurrentBranch => msg::typed_error::REWORD_NOT_ON_BRANCH,
            RewordRefusal::OperationInProgress => msg::typed_error::REWORD_IN_PROGRESS,
            RewordRefusal::EmptyMessage => msg::typed_error::REWORD_EMPTY_MESSAGE,
            RewordRefusal::InvalidIdentity => msg::typed_error::REWORD_INVALID_IDENTITY,
            RewordRefusal::InvalidDate => msg::typed_error::REWORD_INVALID_DATE,
        }),
        TypedError::SplitRefused(r) => msg::typed_error::split_refused(split_refusal(*r)),
        TypedError::RewriteRefused(refusal, detail) => {
            msg::typed_error::rewrite_refused(rewrite_refusal(*refusal), rewrite_detail(detail))
        }
        TypedError::RebaseRefused(r) => msg::typed_error::rebase_refused(match r {
            RebaseRefusal::OperationInProgress => msg::typed_error::REBASE_IN_PROGRESS,
            RebaseRefusal::UncommittedChanges => msg::typed_error::REBASE_UNCOMMITTED,
            RebaseRefusal::NotOnCurrentBranch => msg::typed_error::REBASE_NOT_ON_BRANCH,
            RebaseRefusal::MergeInRange => msg::typed_error::REBASE_MERGE_IN_RANGE,
            RebaseRefusal::PlanMismatch => msg::typed_error::REBASE_PLAN_MISMATCH,
            RebaseRefusal::SquashWithoutPredecessor => msg::typed_error::REBASE_SQUASH_FIRST,
            RebaseRefusal::EmptyRewordMessage => msg::typed_error::REBASE_EMPTY_REWORD,
            RebaseRefusal::NoRebaseInProgress => msg::typed_error::REBASE_NONE,
        }),
        TypedError::ConflictRefused(r) => msg::typed_error::conflict_refused(conflict_refusal(r)),
        TypedError::UndoRefused(r) => msg::typed_error::undo_refused(undo_refusal(r)),
        TypedError::NeedsConfirmation(prompt) => msg::typed_error::needs_confirmation(prompt),
        TypedError::LfsUnavailable(reason) => lfs_unavailable(reason),
        TypedError::LfsLockingUnsupported { remote } => {
            msg::typed_error::lfs_locking_unsupported(remote)
        }
        TypedError::NeedsAutoStash { operation } => msg::typed_error::needs_autostash(operation),
        TypedError::PendingAutoStash {
            operation, stash, ..
        } => pending_autostash(operation, stash.0),
        TypedError::NotBuilt { feature } => msg::typed_error::not_built(feature),
        // No catalog sentence until the TUI surface of the per-repository
        // authentication lands: the English text of the error itself.
        TypedError::RepoAuthUnavailable { .. }
        | TypedError::GrantRefused { .. }
        | TypedError::TokenCommandFailed { .. }
        | TypedError::RepoAuthCommandChanged { .. } => {
            CoreError::from_typed(typed.clone()).to_string()
        }
        // The action that was running when its job was canceled (the label
        // comes from the jobs cell; a job the window never heard of is `#id`).
        TypedError::Canceled { job } => msg::jobs::canceled(crate::jobs_ui::label_of_id(*job)),
        TypedError::NotCancellable { reason } => {
            msg::jobs::not_cancellable(crate::jobs_ui::reason(*reason))
        }
        TypedError::JobConflict { existing } => msg::jobs::job_conflict(existing),
        TypedError::HooksNotTrusted { hooks, .. } => {
            gitpoulpi_i18n::msg::flow_hook_trust::not_trusted(hooks.len() as u64)
        }
        TypedError::SensitiveStage { files } => gitpoulpi_i18n::msg::stage_guard::sensitive_stage(
            gitpoulpi_core::privacy::summary(files),
        ),
    }
}

/// Why a history rewrite was refused, in the UI language. The workbench shows
/// the same words beside the rule or option at fault.
pub(crate) fn rewrite_refusal(
    refusal: gitpoulpi_core::domain::rewrite::RewriteRefusal,
) -> gitpoulpi_i18n::M {
    use gitpoulpi_core::domain::rewrite::RewriteRefusal as R;
    use msg::rewrite_refusal as r;
    match refusal {
        R::OperationInProgress => r::OPERATION_IN_PROGRESS,
        R::AutoStashPending => r::AUTO_STASH_PENDING,
        R::Shallow => r::SHALLOW,
        R::PartialClone => r::PARTIAL_CLONE,
        R::UnsupportedObjectFormat => r::UNSUPPORTED_OBJECT_FORMAT,
        R::EmptyScope => r::EMPTY_SCOPE,
        R::NoRule => r::NO_RULE,
        R::InvalidRule => r::INVALID_RULE,
        R::DirtyPathsTouched => r::DIRTY_PATHS_TOUCHED,
        R::CheckedOutElsewhere => r::CHECKED_OUT_ELSEWHERE,
        R::ProtectedRef => r::PROTECTED_REF,
        R::Stale => r::STALE,
        R::ToolMissing => r::TOOL_MISSING,
        R::ScriptFailed => r::SCRIPT_FAILED,
        R::Purged => r::PURGED,
        R::SignatureChoiceNeeded => r::SIGNATURE_CHOICE_NEEDED,
        R::ExtractModeNeeded => r::EXTRACT_MODE_NEEDED,
        R::ScriptsDisabled => r::SCRIPTS_DISABLED,
        R::ScriptNotTrusted => r::SCRIPT_NOT_TRUSTED,
        R::SigningFailed => r::SIGNING_FAILED,
        R::LfsObjectsMissing => r::LFS_OBJECTS_MISSING,
        R::PathCollision => gitpoulpi_i18n::msg::rewrite_paths::COLLISION_REASON,
    }
}

/// The detail the daemon adds (which rule, which path, which ref) in
/// parentheses after the reason, as the English `Display` does.
fn rewrite_detail(detail: &str) -> String {
    if detail.is_empty() {
        String::new()
    } else if let (Some(commit), reason) =
        gitpoulpi_core::domain::rewrite::script::parse_script_failure(detail)
    {
        // A script that failed names its commit by the full id: seven hex
        // digits are what the Log shows.
        let hex = commit.to_hex();
        format!(" ({}: {reason})", &hex[..hex.len().min(7)])
    } else {
        format!(" ({detail})")
    }
}

/// Why a split was refused, in the UI language. The split editor shows the
/// same words for what it checks itself (`validate_split`).
pub(crate) fn split_refusal(refusal: SplitRefusal) -> gitpoulpi_i18n::M {
    use msg::split_ui as s;
    match refusal {
        // The two reasons reword shares: the same words.
        SplitRefusal::NotOnCurrentBranch => msg::typed_error::REWORD_NOT_ON_BRANCH,
        SplitRefusal::OperationInProgress => msg::typed_error::REWORD_IN_PROGRESS,
        SplitRefusal::MergeCommit => s::REFUSE_MERGE_COMMIT,
        SplitRefusal::EmptyCommit => s::REFUSE_EMPTY_COMMIT,
        SplitRefusal::TooFewParts => s::REFUSE_TOO_FEW_PARTS,
        SplitRefusal::EmptyPart => s::REFUSE_EMPTY_PART,
        SplitRefusal::EmptyMessage => s::REFUSE_EMPTY_MESSAGE,
        SplitRefusal::Unassigned => s::REFUSE_UNASSIGNED,
        SplitRefusal::AssignedTwice => s::REFUSE_ASSIGNED_TWICE,
        SplitRefusal::UnknownPiece => s::REFUSE_UNKNOWN_PIECE,
        SplitRefusal::WholeFileOnly => s::REFUSE_WHOLE_ONLY,
        SplitRefusal::SigningFailed => s::REFUSE_SIGNING_FAILED,
    }
}

/// A pending auto-stash blocks `operation`: points at the banner's buttons,
/// not at CLI commands (so the daemon's own `hint` is not shown).
pub(crate) fn pending_autostash(operation: &str, stash: u32) -> String {
    msg::typed_error::pending_autostash(
        operation,
        stash,
        msg::autostash::POP_NOW,
        msg::autostash::SHOW_STASH,
        msg::autostash::DROP,
    )
}

/// The git-lfs message with the install link. The link stays in every
/// language: the window offers to open it when an error contains it.
pub(crate) fn lfs_unavailable(reason: &LfsUnavailableReason) -> String {
    let lead = match reason {
        LfsUnavailableReason::Missing => msg::typed_error::LFS_MISSING_LEAD.get().to_string(),
        LfsUnavailableReason::TooOld(version) => {
            msg::typed_error::lfs_too_old_lead(version, LFS_MIN_VERSION.0, LFS_MIN_VERSION.1)
        }
    };
    msg::typed_error::lfs_unavailable(lead, LFS_INSTALL_URL)
}

fn conflict_refusal(refusal: &ConflictRefusal) -> String {
    match refusal {
        ConflictRefusal::Stale => msg::typed_error::CONFLICT_STALE.get().to_string(),
        ConflictRefusal::NotConflicted => {
            msg::typed_error::CONFLICT_NOT_CONFLICTED.get().to_string()
        }
        ConflictRefusal::AlreadyConflicted => msg::typed_error::CONFLICT_ALREADY.get().to_string(),
        ConflictRefusal::NoOperation => msg::typed_error::CONFLICT_NO_OPERATION.get().to_string(),
        ConflictRefusal::Unencodable { line } => msg::typed_error::conflict_unencodable(line),
        ConflictRefusal::LocalChanges { paths } => {
            msg::typed_error::conflict_local_changes(paths.join(", "))
        }
    }
}

/// Why an undo or redo did not happen, for the tooltip, the palette and the
/// status line (`UndoRefusal::message`, translated).
pub(crate) fn undo_refusal(refusal: &UndoRefusal) -> String {
    let nothing = |oid: Option<gitpoulpi_core::domain::Oid>| {
        oid.map_or_else(|| msg::typed_error::REF_NOTHING.get().to_string(), short)
    };
    match refusal {
        UndoRefusal::NothingToUndo => msg::undo_ui::NOTHING_TO_UNDO.get().to_string(),
        UndoRefusal::NothingToRedo => msg::undo_ui::NOTHING_TO_REDO.get().to_string(),
        UndoRefusal::Disabled => msg::undo_ui::DISABLED.get().to_string(),
        UndoRefusal::OperationEnded => msg::typed_error::UNDO_OPERATION_ENDED.get().to_string(),
        UndoRefusal::InProgress { op } => msg::typed_error::undo_in_progress(op),
        UndoRefusal::RefMoved {
            name,
            expected,
            actual,
            ..
        } => msg::typed_error::undo_ref_moved(
            name.strip_prefix("refs/heads/")
                .or_else(|| name.strip_prefix("refs/tags/"))
                .unwrap_or(name),
            nothing(*actual),
            nothing(*expected),
        ),
        UndoRefusal::HeadMoved { expected, actual } => {
            msg::typed_error::undo_head_moved(head_state(actual), head_state(expected))
        }
        UndoRefusal::ConfigChanged { section } => msg::typed_error::undo_config_changed(section),
        UndoRefusal::Interrupted { entry } => msg::typed_error::undo_interrupted(entry),
        UndoRefusal::Purged { entry } => msg::typed_error::undo_purged(entry),
        UndoRefusal::RemoteRefMoved(moved) => msg::typed_error::undo_remote_ref_moved(
            moved
                .name
                .strip_prefix("refs/heads/")
                .or_else(|| moved.name.strip_prefix("refs/tags/"))
                .unwrap_or(&moved.name),
            &moved.remote,
            nothing(moved.actual),
            nothing(moved.expected),
        ),
        UndoRefusal::RemoteObjectMissing { oid } => {
            msg::typed_error::undo_remote_object_missing(short(*oid))
        }
        UndoRefusal::WouldOverwrite { detail } => {
            msg::typed_error::undo_would_overwrite(detail.trim())
        }
        UndoRefusal::PathChanged { path } => msg::typed_error::undo_path_changed(path),
        UndoRefusal::IndexChanged => msg::typed_error::UNDO_INDEX_CHANGED.get().to_string(),
        UndoRefusal::StashChanged { commit } => {
            msg::typed_error::undo_stash_changed(short(*commit))
        }
        UndoRefusal::StaleEntry {
            expected_id,
            top_id,
        } => msg::typed_error::undo_stale_entry(expected_id, top_id),
        UndoRefusal::NotRestorable { reason } => msg::typed_error::undo_not_restorable(reason),
        UndoRefusal::NameTaken { name, suggested } => {
            msg::typed_error::undo_name_taken(name, suggested)
        }
        UndoRefusal::AlreadyRestored => msg::typed_error::UNDO_ALREADY_RESTORED.get().to_string(),
        UndoRefusal::NoSuchEntry { id } => msg::typed_error::undo_no_such_entry(id),
    }
}

fn head_state(state: &HeadState) -> String {
    match state {
        HeadState::Branch(name) => name.clone(),
        HeadState::Detached(oid) => msg::typed_error::head_detached_at(short(*oid)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::{Oid, PushRejection, StashId};
    use gitpoulpi_i18n::{override_git_terms, with_lang, Lang};

    fn oid(c: char) -> Oid {
        Oid::from_hex(&c.to_string().repeat(40)).unwrap()
    }

    /// One of every typed error, every refusal variant included.
    fn every_typed() -> Vec<TypedError> {
        let undo = [
            UndoRefusal::NothingToUndo,
            UndoRefusal::NothingToRedo,
            UndoRefusal::InProgress { op: "merge".into() },
            UndoRefusal::RefMoved {
                name: "refs/heads/main".into(),
                expected: Some(oid('a')),
                actual: None,
                restore: None,
            },
            UndoRefusal::NotRestorable {
                reason: "it only moved HEAD".into(),
            },
            UndoRefusal::NameTaken {
                name: "feature".into(),
                suggested: "feature-restored".into(),
            },
            UndoRefusal::AlreadyRestored,
            UndoRefusal::NoSuchEntry { id: 9 },
            UndoRefusal::HeadMoved {
                expected: HeadState::Branch("main".into()),
                actual: HeadState::Detached(oid('b')),
            },
            UndoRefusal::ConfigChanged {
                section: "remote.origin".into(),
            },
            UndoRefusal::Interrupted { entry: 7 },
            UndoRefusal::WouldOverwrite {
                detail: " a.txt ".into(),
            },
            UndoRefusal::PathChanged {
                path: "a.txt".into(),
            },
            UndoRefusal::IndexChanged,
            UndoRefusal::StashChanged { commit: oid('c') },
            UndoRefusal::RemoteRefMoved(Box::new(gitpoulpi_core::domain::RemoteRefMoved {
                remote: "origin".into(),
                name: "refs/heads/topic".into(),
                expected: None,
                actual: Some(oid('d')),
            })),
            UndoRefusal::RemoteObjectMissing { oid: oid('e') },
            UndoRefusal::StaleEntry {
                expected_id: 3,
                top_id: 5,
            },
            UndoRefusal::Disabled,
            UndoRefusal::OperationEnded,
        ];
        let conflict = [
            ConflictRefusal::Stale,
            ConflictRefusal::NotConflicted,
            ConflictRefusal::AlreadyConflicted,
            ConflictRefusal::NoOperation,
            ConflictRefusal::Unencodable { line: 4 },
            ConflictRefusal::LocalChanges {
                paths: vec!["a.txt".into(), "dir/b.txt".into()],
            },
        ];
        let reword = [
            RewordRefusal::NotOnCurrentBranch,
            RewordRefusal::OperationInProgress,
            RewordRefusal::EmptyMessage,
            RewordRefusal::InvalidIdentity,
            RewordRefusal::InvalidDate,
        ];
        let split = [
            SplitRefusal::NotOnCurrentBranch,
            SplitRefusal::MergeCommit,
            SplitRefusal::EmptyCommit,
            SplitRefusal::OperationInProgress,
            SplitRefusal::TooFewParts,
            SplitRefusal::EmptyPart,
            SplitRefusal::EmptyMessage,
            SplitRefusal::Unassigned,
            SplitRefusal::AssignedTwice,
            SplitRefusal::UnknownPiece,
            SplitRefusal::WholeFileOnly,
            SplitRefusal::SigningFailed,
        ];
        let rebase = [
            RebaseRefusal::OperationInProgress,
            RebaseRefusal::UncommittedChanges,
            RebaseRefusal::NotOnCurrentBranch,
            RebaseRefusal::MergeInRange,
            RebaseRefusal::PlanMismatch,
            RebaseRefusal::SquashWithoutPredecessor,
            RebaseRefusal::EmptyRewordMessage,
            RebaseRefusal::NoRebaseInProgress,
        ];
        let mut all = vec![
            TypedError::NotFastForward,
            TypedError::NeedsConfirmation("it is risky".into()),
            TypedError::LfsUnavailable(LfsUnavailableReason::Missing),
            TypedError::LfsUnavailable(LfsUnavailableReason::TooOld("2.9".into())),
            TypedError::LfsLockingUnsupported {
                remote: "origin".into(),
            },
            TypedError::NeedsAutoStash {
                operation: "checkout".into(),
            },
            TypedError::PendingAutoStash {
                operation: "merge".into(),
                stash: StashId(2),
                hint: "pop it".into(),
            },
            TypedError::HooksNotTrusted {
                hash: "0".repeat(64),
                hooks: vec![gitpoulpi_core::workflow::HookInfo {
                    phase: "after_start".into(),
                    command: "make".into(),
                    personal: false,
                }],
            },
            TypedError::SensitiveStage {
                files: vec![
                    gitpoulpi_core::privacy::Sensitive {
                        path: ".env".into(),
                        reason: gitpoulpi_core::privacy::Reason::Pattern(".env".into()),
                    },
                    gitpoulpi_core::privacy::Sensitive {
                        path: "a.txt".into(),
                        reason: gitpoulpi_core::privacy::Reason::Secret {
                            rule: "github-token".into(),
                            preview: "ghp_****".into(),
                        },
                    },
                ],
            },
        ];
        for reason in [
            PushRejectReason::NonFastForward,
            PushRejectReason::StaleLease,
        ] {
            all.push(TypedError::PushRejected(PushRejection {
                remote: "origin".into(),
                branch: "main".into(),
                reason,
                detail: None,
            }));
        }
        all.extend(undo.into_iter().map(TypedError::UndoRefused));
        all.extend(conflict.into_iter().map(TypedError::ConflictRefused));
        all.extend(reword.into_iter().map(TypedError::RewordRefused));
        all.extend(split.into_iter().map(TypedError::SplitRefused));
        all.extend(rebase.into_iter().map(TypedError::RebaseRefused));
        use gitpoulpi_core::domain::rewrite::RewriteRefusal as Rw;
        let rewrite = [
            Rw::OperationInProgress,
            Rw::AutoStashPending,
            Rw::Shallow,
            Rw::PartialClone,
            Rw::UnsupportedObjectFormat,
            Rw::EmptyScope,
            Rw::NoRule,
            Rw::InvalidRule,
            Rw::DirtyPathsTouched,
            Rw::CheckedOutElsewhere,
            Rw::ProtectedRef,
            Rw::Stale,
            Rw::ToolMissing,
            Rw::ScriptFailed,
            Rw::Purged,
            Rw::SignatureChoiceNeeded,
            Rw::ExtractModeNeeded,
            Rw::ScriptsDisabled,
            Rw::ScriptNotTrusted,
            Rw::PathCollision,
        ];
        all.extend(
            rewrite
                .into_iter()
                .map(|r| TypedError::RewriteRefused(r, String::new())),
        );
        all.push(TypedError::RewriteRefused(
            Rw::InvalidRule,
            "rule 2: the shift is zero".into(),
        ));
        all
    }

    /// In English the TUI says what the daemon, the CLI and MCP say, except
    /// where the TUI deliberately points at its own buttons (the auto-stash
    /// banner) instead of at CLI commands.
    #[test]
    fn english_is_the_daemons_display_text() {
        for typed in every_typed() {
            let shown = typed_text(&typed);
            if matches!(typed, TypedError::PendingAutoStash { .. }) {
                assert_eq!(
                    shown,
                    "merge is blocked: auto-stash stash@{2} is still waiting - \
                     use the banner's Pop now, Show stash or Drop first"
                );
                continue;
            }
            // The TUI points at its own question, not at the CLI command.
            if matches!(typed, TypedError::HooksNotTrusted { .. }) {
                assert_eq!(
                    shown,
                    "The workflow hooks are not trusted: 1 command was not run. Review it and \
                     trust it first."
                );
                continue;
            }
            assert_eq!(
                shown,
                CoreError::from_typed(typed.clone()).to_string(),
                "{typed:?}"
            );
        }
    }

    /// Every typed error has a French sentence, in both vocabularies, and no
    /// placeholder is left unfilled.
    #[test]
    fn every_typed_error_has_a_french_translation() {
        for typed in every_typed() {
            let english = typed_text(&typed);
            for local in [false, true] {
                let _vocabulary = override_git_terms(local);
                let french = with_lang(Lang::Fr, || typed_text(&typed));
                assert_ne!(french, english, "{typed:?} is not translated");
                assert!(!french.replace("stash@{", "").contains('{'), "{french}");
            }
        }
    }

    #[test]
    fn a_few_french_sentences() {
        with_lang(Lang::Fr, || {
            assert_eq!(
                typed_text(&TypedError::NotFastForward),
                "avance rapide impossible\u{a0}: les deux ont divergé"
            );
            assert_eq!(
                typed_text(&TypedError::UndoRefused(UndoRefusal::RefMoved {
                    name: "refs/heads/main".into(),
                    expected: Some(oid('a')),
                    actual: None,
                    restore: None,
                })),
                "impossible d'annuler ou de rétablir\u{a0}: main est à rien, et non à aaaaaaa \
                 où l'opération l'avait laissé\u{a0}; modifié en dehors de GitPoulpi"
            );
            let _vocabulary = override_git_terms(true);
            assert_eq!(
                typed_text(&TypedError::RebaseRefused(
                    RebaseRefusal::NoRebaseInProgress
                )),
                "impossible de rebaser\u{a0}: aucun rebasage n'est en cours"
            );
            // The git-lfs link stays: the window offers to open it.
            assert!(
                typed_text(&TypedError::LfsUnavailable(LfsUnavailableReason::Missing))
                    .contains(LFS_INSTALL_URL)
            );
        });
    }

    /// Text that is not typed (git's stderr, I/O) is shown as it came.
    #[test]
    fn untyped_errors_are_shown_as_they_are() {
        let backend = CoreError::Backend("fatal: not a git repository".into());
        with_lang(Lang::Fr, || {
            assert_eq!(error_text(&backend), backend.to_string());
        });
    }
}
