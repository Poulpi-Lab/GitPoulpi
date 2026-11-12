//! What a history row says, in the UI language. The journal stores one
//! English description per entry (`label`); the row is rebuilt here from the
//! entry's kind and the words recorded in that description, through catalog
//! strings. The stored text stays the fallback for an entry whose shape is not
//! recognised (an older journal, a kind added later), so a row never goes blank.
//!
//! The daemon's wording is fixed per kind (`journal/labels.rs`,
//! `journal_snap.rs`, `journal_stashes_remotes.rs`), so each kind below peels
//! its own verb and keeps the names, paths and short ids as recorded.

use gitpoulpi_core::domain::{
    EntryState, EntrySummary, ListedEntry, OperationKind, RestoreItem, RestoreItemKind,
};
use gitpoulpi_i18n::msg::undo_label as l;

/// The icon family of a kind: one glyph column in the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Family {
    Commit,
    Branch,
    Tag,
    Stash,
    /// Reset, restores and other moves of a ref.
    Move,
    /// Merge, rebase, cherry-pick, revert and the conflict work around them.
    Merge,
    /// Discarded or deleted files.
    Discard,
    /// Files rewritten, patched or ignored.
    Edit,
    Staging,
    /// Configuration, remotes, LFS.
    Config,
    /// Made outside GitPoulpi.
    Outside,
    /// Never recorded in the journal: a read or a daemon control request.
    Other,
}

impl Family {
    /// Width-2 glyphs with default emoji presentation (no VS16), like the
    /// sidebar's (`theme::TAG_ICON`, `STASH_ICON`): see "Emoji presentation"
    /// in docs/developer/where-we-are.md.
    pub(crate) fn glyph(self) -> &'static str {
        match self {
            Family::Commit => "\u{1F4BE}", // floppy disk
            Family::Branch => "\u{1F331}", // seedling
            Family::Tag => crate::theme::TAG_ICON,
            Family::Stash => crate::theme::STASH_ICON,
            Family::Move => "\u{23EA}", // rewind
            Family::Merge => crate::theme::PULL_REQUEST_ICON,
            Family::Discard => "\u{1F525}", // fire
            Family::Edit => "\u{1F4DD}",    // memo
            Family::Staging => "\u{2795}",  // heavy plus
            Family::Config => "\u{1F527}",  // wrench
            Family::Outside => "\u{1F6AA}", // door
            Family::Other => "\u{00B7} ",   // middle dot, padded to the column
        }
    }
}

/// The family of every operation kind. No wildcard: a new kind must be
/// placed here (or in `Other`, on purpose) before the crate compiles.
pub(crate) fn family(kind: OperationKind) -> Family {
    use OperationKind as K;
    match kind {
        K::Commit
        | K::Reword
        | K::AmendCommit
        | K::SplitCommit
        | K::ComposeCommits
        | K::RewriteApply => Family::Commit,
        K::Checkout
        | K::BranchCreate
        | K::BranchCreateTracking
        | K::BranchDelete
        | K::BranchDeleteRemote
        | K::BranchRename
        | K::BranchSetUpstream
        | K::FastForward
        | K::FetchRefAsBranch
        | K::WorktreeAdd
        | K::FlowRun => Family::Branch,
        K::TagCreate
        | K::TagDelete
        | K::TagAnnotate
        | K::TagPush
        | K::TagDeleteRemote
        | K::TagFetch => Family::Tag,
        K::StashPush
        | K::StashFromCommit
        | K::StashPop
        | K::StashApply
        | K::StashDrop
        | K::AutoStashStartOver => Family::Stash,
        K::Reset | K::JournalRestore | K::ReflogRestore | K::Undo | K::Redo => Family::Move,
        K::Merge
        | K::Rebase
        | K::RebaseInteractive
        | K::RebaseContinue
        | K::RebaseSkip
        | K::RebaseAbort
        | K::CherryPick
        | K::Revert
        | K::Pull
        | K::ResolveConflict
        | K::ConflictUnresolve
        | K::OperationAbort
        | K::OperationSkip
        | K::AmContinue
        | K::AmSkip
        | K::AmAbort => Family::Merge,
        K::Discard | K::DiscardHunk | K::DiscardLine | K::DeleteFile => Family::Discard,
        K::PatchApply
        | K::ApplyCommit
        | K::ConvertFiles
        | K::AddToGitignore
        | K::MailmapAdd
        | K::GitattributesTreat => Family::Edit,
        K::StageFile
        | K::StageHunk
        | K::StageLine
        | K::StageFolder
        | K::StageAll
        | K::UnstageFile
        | K::UnstageHunk
        | K::UnstageLine
        | K::UnstageFolder
        | K::UnstageAll => Family::Staging,
        K::Init
        | K::Fetch
        | K::FetchAll
        | K::FetchRefspec
        | K::Push
        | K::PushForReview
        | K::Clone
        | K::RemoteAdd
        | K::RemoteRemove
        | K::RemoteRename
        | K::RemoteSetUrl
        | K::RemoteSetPushUrl
        | K::SetSshCommand
        | K::SetRepoConfig
        | K::SparseSet
        | K::SparseDisable
        | K::SubmoduleInit
        | K::SubmoduleUpdate
        | K::LfsInstall
        | K::LfsTrack
        | K::LfsUntrack
        | K::LfsFetch
        | K::LfsPull
        | K::LfsCheckout
        | K::LfsPush
        | K::LfsPrune
        | K::LfsLock
        | K::LfsUnlock
        | K::IndexLockRemove => Family::Config,
        K::Outside => Family::Outside,
        K::Discover
        | K::Status
        | K::Log
        | K::CountCommits
        | K::Diff
        | K::DiffFiles
        | K::CommitStats
        | K::Search
        | K::Refs
        | K::Stashes
        | K::Remotes
        | K::LeftoverRemotes
        | K::Worktrees
        | K::Submodules
        | K::Resolve
        | K::MergeBase
        | K::PredictConflicts
        | K::ReadBlob
        | K::ReadBlobBytes
        | K::Blame
        | K::FileHistory
        | K::ListTree
        | K::PatchCreate
        | K::BinaryPaths
        | K::MailmapIdentities
        | K::MailmapResolve
        | K::RebasePlan
        | K::SplitSource
        | K::RebaseState
        | K::SshCommand
        | K::RepoConfig
        | K::Workflow
        | K::FlowPlan
        | K::SparseStatus
        | K::Conflicts
        | K::ConflictEntries
        | K::ConflictShow
        | K::OperationState
        | K::IndexLock
        | K::LfsStatus
        | K::LfsFiles
        | K::LfsLocks
        | K::RemoteAccessCheck
        | K::RepoAuthGet
        | K::RepoAuthSet
        | K::RepoAuthClear
        | K::RepoAuthTest
        | K::Hello
        | K::CredentialGrant
        | K::JournalState
        | K::JournalList
        | K::Reflog
        | K::LostCommits
        | K::Shutdown
        | K::SetRefVisibility
        | K::UiAttach
        | K::UiPresence
        | K::UiReport
        | K::UiShow
        | K::UiQuery
        | K::JobList
        | K::RewritePreview
        | K::RewriteDiscard
        | K::RewritePush
        | K::RewriteNote
        | K::RewriteMaps
        | K::RewritePurgeCheck
        | K::RewritePurge
        | K::RewriteRows
        | K::RewriteBlobDiff
        | K::RewriteScriptCheck
        | K::RewriteScriptTest
        | K::RewriteTrustScript
        | K::FlowTrustHooks
        | K::RewriteScanSecrets
        | K::RewriteAnalyze
        | K::RewriteCheckSigning
        | K::RewriteTools
        | K::JobCancel => Family::Other,
    }
}

/// The row text of a listed entry.
pub(crate) fn entry_text(entry: &ListedEntry) -> String {
    summary_text(&entry.summary)
}

/// The words for one journal entry, in the UI language; the stored English
/// description when its shape is not recognised.
pub(crate) fn summary_text(summary: &EntrySummary) -> String {
    let words = text(summary.kind, &summary.label);
    // A purged rewrite stays in the list, marked: its old history is gone.
    if summary.state == EntryState::Purged {
        return format!("{words} ({})", l::PURGED_SUFFIX.get());
    }
    words
}

// The English the daemon stores, matched here and never shown: the words
// the user reads come from the catalog (`undo_label`).
// i18n-scan: off
pub(crate) fn text(kind: OperationKind, stored: &str) -> String {
    if kind == OperationKind::Outside {
        return outside(stored).unwrap_or_else(|| stored.to_string());
    }
    let (body, sha) = split_sha(stored);
    match parse(kind, body) {
        Some(words) => format!("{words}{sha}"),
        None => stored.to_string(),
    }
}

/// `Commit "x" (abc1234)` -> (`Commit "x"`, ` (abc1234)`).
fn split_sha(label: &str) -> (&str, &str) {
    let Some(open) = label.rfind(" (") else {
        return (label, "");
    };
    let inner = &label[open + 2..];
    match inner.strip_suffix(')') {
        Some(hex)
            if (7..=40).contains(&hex.len()) && hex.bytes().all(|b| b.is_ascii_hexdigit()) =>
        {
            (&label[..open], &label[open..])
        }
        _ => (label, ""),
    }
}

/// A path, or `N files` as the daemon writes it for several.
fn subject(s: &str) -> String {
    match s.strip_suffix(" files").and_then(|n| n.parse::<u64>().ok()) {
        Some(n) => l::n_files(n),
        None => s.to_string(),
    }
}

fn count_after(body: &str, prefix: &str) -> Option<u64> {
    body.strip_prefix(prefix)?
        .split_whitespace()
        .next()?
        .parse()
        .ok()
}

fn quoted(s: &str) -> bool {
    s.starts_with('"') && s.ends_with('"') && s.len() >= 2
}

fn parse(kind: OperationKind, body: &str) -> Option<String> {
    use OperationKind as K;
    let rest = |prefix: &str| body.strip_prefix(prefix);
    match kind {
        K::Commit => rest("Commit ")
            .map(l::commit)
            .or_else(|| rest("Amend ").map(l::amend)),
        K::ComposeCommits => count_after(body, "Compose ").map(l::compose),
        K::Checkout => {
            if body == "Checkout" {
                Some(l::CHECKOUT_PLAIN.get().to_string())
            } else if let Some(name) = rest("Checkout ") {
                Some(l::checkout(name))
            } else if let Some(path) = rest("Discard ") {
                Some(l::discard(subject(path)))
            } else {
                rest("Restore ").map(|files| l::restore_files(subject(files)))
            }
        }
        K::Discard => rest("Discard ").map(|files| l::discard(subject(files))),
        K::DiscardHunk => match rest("Discard a hunk of ") {
            Some(path) => Some(l::discard_hunk(path)),
            None => (body == "Discard a hunk").then(|| l::DISCARD_HUNK_PLAIN.get().to_string()),
        },
        K::DiscardLine => match rest("Discard lines of ") {
            Some(path) => Some(l::discard_lines(path)),
            None => (body == "Discard lines").then(|| l::DISCARD_LINES_PLAIN.get().to_string()),
        },
        K::StageFile | K::StageHunk | K::StageLine | K::StageFolder | K::StageAll => {
            rest("Stage ").map(staging_words(true))
        }
        K::UnstageFile | K::UnstageHunk | K::UnstageLine | K::UnstageFolder | K::UnstageAll => {
            rest("Unstage ").map(staging_words(false))
        }
        K::OperationAbort => {
            (body == "Abort the operation").then(|| l::ABORT_OPERATION.get().to_string())
        }
        K::PatchApply => match rest("Apply patch ") {
            Some(file) => Some(l::apply_patch(file)),
            None => (body == "Apply patch").then(|| l::APPLY_PATCH_PLAIN.get().to_string()),
        },
        K::DeleteFile => rest("Delete ").map(|f| l::delete_files(subject(f))),
        K::ConvertFiles => rest("Convert ").map(|f| l::convert_files(subject(f))),
        K::AddToGitignore => rest("Ignore ").map(l::ignore),
        K::MailmapAdd => rest("Merge identities into ").map(l::mailmap_merge),
        K::GitattributesTreat => {
            let (what, how) = rest("Treat ")?.rsplit_once(" as ")?;
            let how = match how {
                "binary" => l::TREAT_BINARY.get().to_string(),
                "text" => l::TREAT_TEXT.get().to_string(),
                other => other.to_string(),
            };
            Some(l::treat(subject(what), how))
        }
        K::ResolveConflict => match rest("Auto-resolve (") {
            Some(tail) => {
                let (why, path) = tail.split_once(") ")?;
                Some(l::auto_resolve(why, path))
            }
            None => rest("Resolve ").map(l::resolve),
        },
        K::BranchCreate => rest("Create branch ").map(l::branch_create),
        K::BranchDelete => rest("Delete branch ").map(l::branch_delete),
        K::BranchCreateTracking => {
            let (name, upstream) = rest("Create branch ")?.split_once(" tracking ")?;
            Some(l::branch_create_tracking(name, upstream))
        }
        K::BranchRename => {
            let (old, new) = rest("Rename branch ")?.split_once(" to ")?;
            Some(l::branch_rename(old, new))
        }
        K::BranchSetUpstream => match rest("Set upstream of ") {
            Some(tail) => {
                let (branch, upstream) = tail.split_once(" to ")?;
                Some(l::upstream_set(branch, upstream))
            }
            None => rest("Unset upstream of ").map(l::upstream_unset),
        },
        K::SetRepoConfig => match rest("Unset ") {
            Some(key) => Some(l::config_unset(key)),
            None => {
                let (key, value) = rest("Set ")?.split_once(" to ")?;
                Some(l::config_set(key, value))
            }
        },
        K::SparseSet => (body == "Set sparse checkout").then(|| l::SPARSE_SET.get().to_string()),
        K::SparseDisable => {
            (body == "Disable sparse checkout").then(|| l::SPARSE_DISABLE.get().to_string())
        }
        K::TagCreate => rest("Create tag ").map(l::tag_create),
        K::TagDelete => rest("Delete tag ").map(l::tag_delete),
        K::TagAnnotate => rest("Annotate tag ").map(l::tag_annotate),
        K::FastForward => {
            let (branch, target) = rest("Fast-forward ")?.rsplit_once(" to ")?;
            Some(l::fast_forward(branch, target))
        }
        K::Reword => rest("Reword ").map(l::reword),
        K::AmendCommit => rest("Amend ").map(l::amend_commit),
        K::SplitCommit => {
            let (id, count) = rest("Split ")?.split_once(" into ")?;
            let n = count.strip_suffix(" commits")?.parse().ok()?;
            Some(l::split_commit(n, id))
        }
        K::RewriteApply => {
            let mut counts = rest("Rewrite history: ")?
                .split(", ")
                .map(|part| part.split_once(' ')?.0.parse::<u64>().ok());
            let (commits, branches, tags) = (counts.next()??, counts.next()??, counts.next()??);
            Some(l::rewrite_apply(
                l::rewrite_commits(commits),
                l::rewrite_branches(branches),
                l::rewrite_tags(tags),
            ))
        }
        K::FetchRefAsBranch => {
            let (remote_ref, branch) = rest("Fetch ")?.split_once(" as branch ")?;
            Some(l::fetch_as_branch(remote_ref, branch))
        }
        K::Merge => match rest("Merge ") {
            Some(name) => Some(l::merge(name)),
            None => (body == "Merge").then(|| l::MERGE_PLAIN.get().to_string()),
        },
        K::Rebase => match rest("Rebase onto ") {
            Some(name) => Some(l::rebase_onto(name)),
            None => (body == "Rebase").then(|| l::REBASE_PLAIN.get().to_string()),
        },
        K::RebaseInteractive => {
            (body == "Interactive rebase").then(|| l::REBASE_INTERACTIVE.get().to_string())
        }
        K::CherryPick => count_after(body, "Cherry-pick ").map(l::cherry_pick),
        K::Revert => match rest("Revert ") {
            Some(commit) => Some(l::revert(commit)),
            None => (body == "Revert").then(|| l::REVERT_PLAIN.get().to_string()),
        },
        K::Reset => {
            let tail = rest("Reset (")?;
            let (mode, target) = tail.split_once(") to ")?;
            let mode = match mode {
                "soft" => l::RESET_SOFT.get().to_string(),
                "mixed" => l::RESET_MIXED.get().to_string(),
                "hard" => l::RESET_HARD.get().to_string(),
                other => other.to_string(),
            };
            Some(l::reset(mode, target))
        }
        K::Pull => (body == "Pull").then(|| l::PULL.get().to_string()),
        K::FlowRun => rest("Flow ").map(l::flow),
        K::StashPush => match body {
            "Stash changes" => Some(l::STASH_PUSH.get().to_string()),
            _ => rest("Stash ")
                .filter(|t| quoted(t))
                .map(l::stash_push_titled),
        },
        K::StashPop => stash_words(body, "Pop stash", l::STASH_POP.get(), &|t| {
            l::stash_pop_titled(t)
        }),
        K::StashApply => stash_words(body, "Apply stash", l::STASH_APPLY.get(), &|t| {
            l::stash_apply_titled(t)
        }),
        K::StashDrop => stash_words(body, "Drop stash", l::STASH_DROP.get(), &|t| {
            l::stash_drop_titled(t)
        }),
        K::RemoteAdd => rest("Add remote ").map(l::remote_add),
        K::RemoteRemove => rest("Remove remote ").map(l::remote_remove),
        K::RemoteRename => {
            let (old, new) = rest("Rename remote ")?.split_once(" to ")?;
            Some(l::remote_rename(old, new))
        }
        K::RemoteSetUrl => rest("Set the URL of remote ").map(l::remote_url),
        K::RemoteSetPushUrl => rest("Set the push URL of remote ").map(l::remote_push_url),
        K::LfsTrack => rest("Track ")?.strip_suffix(" with LFS").map(l::lfs_track),
        K::LfsUntrack => rest("Stop tracking ")?
            .strip_suffix(" with LFS")
            .map(l::lfs_untrack),
        K::JournalRestore => rest("Restore \"")?.strip_suffix('"').map(l::restore_entry),
        K::ReflogRestore => {
            let (name, target) = rest("Restore ")?.rsplit_once(" to ")?;
            Some(l::reflog_restore(name, target))
        }
        // `ApplyCommit` / `StashFromCommit` rows read as the daemon stored
        // them until their catalog words are translated (see `WITHOUT_WORDS`).
        // Recorded by no code path today, or with no words of their own:
        // the stored text stands. The server operations here (LFS fetch,
        // pull, checkout, push, TagPush, ...) cannot reach the journal at all:
        // `capture_before` only records a `journaled` kind, none of them is
        // (pinned by `op_classification`), a `JournalRestore` row carries the
        // source entry's words and an outside move is `K::Outside`. Give one
        // a template here the day it is journaled.
        K::Init
        | K::ApplyCommit
        | K::StashFromCommit
        | K::Fetch
        | K::FetchAll
        | K::FetchRefspec
        | K::Push
        | K::PushForReview
        | K::Clone
        | K::SetSshCommand
        | K::SubmoduleInit
        | K::SubmoduleUpdate
        | K::LfsInstall
        | K::LfsFetch
        | K::LfsPull
        | K::LfsCheckout
        | K::LfsPush
        | K::LfsPrune
        | K::LfsLock
        | K::LfsUnlock
        | K::IndexLockRemove
        | K::BranchDeleteRemote
        | K::WorktreeAdd
        | K::TagPush
        | K::TagDeleteRemote
        | K::TagFetch
        | K::AutoStashStartOver
        | K::Undo
        | K::Redo
        | K::RebaseContinue
        | K::RebaseSkip
        | K::RebaseAbort
        | K::ConflictUnresolve
        | K::OperationSkip
        | K::AmContinue
        | K::AmSkip
        | K::AmAbort
        | K::Outside
        | K::Discover
        | K::Status
        | K::Log
        | K::CountCommits
        | K::Diff
        | K::DiffFiles
        | K::CommitStats
        | K::Search
        | K::Refs
        | K::Stashes
        | K::Remotes
        | K::LeftoverRemotes
        | K::Worktrees
        | K::Submodules
        | K::Resolve
        | K::MergeBase
        | K::PredictConflicts
        | K::ReadBlob
        | K::ReadBlobBytes
        | K::Blame
        | K::FileHistory
        | K::ListTree
        | K::PatchCreate
        | K::BinaryPaths
        | K::MailmapIdentities
        | K::MailmapResolve
        | K::RebasePlan
        | K::SplitSource
        | K::RebaseState
        | K::SshCommand
        | K::RepoConfig
        | K::Workflow
        | K::FlowPlan
        | K::SparseStatus
        | K::Conflicts
        | K::ConflictEntries
        | K::ConflictShow
        | K::OperationState
        | K::IndexLock
        | K::LfsStatus
        | K::LfsFiles
        | K::LfsLocks
        | K::RemoteAccessCheck
        | K::RepoAuthGet
        | K::RepoAuthSet
        | K::RepoAuthClear
        | K::RepoAuthTest
        | K::Hello
        | K::CredentialGrant
        | K::JournalState
        | K::JournalList
        | K::Reflog
        | K::LostCommits
        | K::Shutdown
        | K::SetRefVisibility
        | K::UiAttach
        | K::UiPresence
        | K::UiReport
        | K::UiShow
        | K::UiQuery
        | K::JobList
        | K::RewritePreview
        | K::RewriteDiscard
        | K::RewritePush
        | K::RewriteNote
        | K::RewriteMaps
        | K::RewritePurgeCheck
        | K::RewritePurge
        | K::RewriteRows
        | K::RewriteBlobDiff
        | K::RewriteScriptCheck
        | K::RewriteScriptTest
        | K::RewriteTrustScript
        | K::FlowTrustHooks
        | K::RewriteScanSecrets
        | K::RewriteAnalyze
        | K::RewriteCheckSigning
        | K::RewriteTools
        | K::JobCancel => None,
    }
}

/// `Stage a.txt`, `Stage folder d`, `Stage a hunk of a.txt`, ... with the
/// verb already peeled.
fn staging_words(stage: bool) -> impl Fn(&str) -> String {
    move |rest| {
        if let Some(p) = rest.strip_prefix("folder ") {
            if stage {
                l::stage_folder(p)
            } else {
                l::unstage_folder(p)
            }
        } else if let Some(p) = rest.strip_prefix("a hunk of ") {
            if stage {
                l::stage_hunk(p)
            } else {
                l::unstage_hunk(p)
            }
        } else if let Some(p) = rest.strip_prefix("lines of ") {
            if stage {
                l::stage_lines(p)
            } else {
                l::unstage_lines(p)
            }
        } else if rest == "all changes" {
            if stage {
                l::STAGE_ALL.get().to_string()
            } else {
                l::UNSTAGE_ALL.get().to_string()
            }
        } else if stage {
            l::stage_file(rest)
        } else {
            l::unstage_file(rest)
        }
    }
}

/// `Pop stash` or `Pop stash "title"`.
fn stash_words(
    body: &str,
    prefix: &str,
    plain: &str,
    titled: &dyn Fn(&str) -> String,
) -> Option<String> {
    if body == prefix {
        return Some(plain.to_string());
    }
    body.strip_prefix(prefix)?
        .strip_prefix(' ')
        .filter(|t| quoted(t))
        .map(titled)
}

// -- outside changes ---------------------------------------------------------

/// The label of an `Outside` entry: git's own reflog reason, then the refs it
/// touched, then notes. Only the parts GitPoulpi wrote and the common reasons
/// git wrote are translated; any other reason stays as git recorded it.
fn outside(stored: &str) -> Option<String> {
    let mut rest = stored;
    let mut tail: Vec<String> = Vec::new();
    loop {
        if let Some(head) = rest.strip_suffix(" (while GitPoulpi was closed)") {
            tail.push(format!(" ({})", l::OUTSIDE_CLOSED.get()));
            rest = head;
            continue;
        }
        if let Some(open) = rest.rfind(" (").filter(|_| rest.ends_with(')')) {
            let note = &rest[open + 2..rest.len() - 1];
            let words = note
                .strip_prefix("worktree added ")
                .map(l::outside_worktree_added)
                .or_else(|| {
                    note.strip_prefix("worktree removed ")
                        .map(l::outside_worktree_removed)
                });
            if let Some(words) = words {
                tail.push(format!(" ({words})"));
                rest = &rest[..open];
                continue;
            }
        }
        break;
    }
    let (reason, names) = match rest.rsplit_once(" - ") {
        Some((reason, names)) => (reason, Some(names)),
        None => (rest, None),
    };
    let mut out = outside_reason(reason);
    if let Some(names) = names {
        out.push_str(" - ");
        out.push_str(&more_names(names));
    }
    for part in tail.iter().rev() {
        out.push_str(part);
    }
    (out != stored).then_some(out)
}

/// `a, b, c and 2 more`.
fn more_names(names: &str) -> String {
    if let Some((head, tail)) = names.rsplit_once(" and ") {
        if let Some(n) = tail
            .strip_suffix(" more")
            .and_then(|n| n.parse::<u64>().ok())
        {
            return format!("{head} {}", l::outside_more(n));
        }
    }
    names.to_string()
}

fn outside_reason(reason: &str) -> String {
    if let Some(message) = reason.strip_prefix("commit (amend): ") {
        return l::outside_amend(message);
    }
    if let Some(message) = reason.strip_prefix("commit: ") {
        return l::outside_commit(message);
    }
    if let Some(tail) = reason.strip_prefix("checkout: moving from ") {
        if let Some((from, to)) = tail.split_once(" to ") {
            return l::outside_checkout(from, to);
        }
    }
    if let Some(target) = reason.strip_prefix("reset: moving to ") {
        return l::outside_reset(target);
    }
    reason.to_string()
}

// -- restore details ---------------------------------------------------------

/// One "restore only this" item in the UI language. A branch, file or ref
/// is a name and keeps its recorded text.
pub(crate) fn restore_item(item: &RestoreItem) -> String {
    match item.kind {
        RestoreItemKind::Stash => match item.label.strip_prefix("stash ") {
            Some(what) => l::item_stash(what),
            None => item.label.clone(),
        },
        RestoreItemKind::Tag => match item.label.strip_prefix("tag ") {
            Some(what) => l::item_tag(what),
            None => item.label.clone(),
        },
        _ => match item.label.strip_prefix("old tip of ") {
            Some(what) => l::item_old_tip(what),
            None => item.label.clone(),
        },
    }
}

/// Why "restore only this" has nothing to bring back, in the UI language.
pub(crate) fn not_restorable(why: &str) -> String {
    let words = match why {
        "it only changed the index (staging), nothing was removed" => l::NR_INDEX,
        "the files it changed held nothing that is lost" => l::NR_FILES,
        "it only moved HEAD; the reflog lists the earlier positions" => l::NR_HEAD,
        "it only added things, nothing was removed" => l::NR_ADDED,
        "it removed nothing that can be brought back without guessing" => l::NR_OTHER,
        "the operation never completed, so it is not known what it removed" => l::NR_INCOMPLETE,
        _ => return why.to_string(),
    };
    words.get().to_string()
}
// i18n-scan: on
