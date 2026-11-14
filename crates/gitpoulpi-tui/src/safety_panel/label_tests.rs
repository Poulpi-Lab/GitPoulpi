//! Every operation kind has its words, in a language that is not English.

use gitpoulpi_core::domain::{OperationKind as K, RestoreItem, RestoreItemKind};
use gitpoulpi_i18n::Lang;

use super::label::{family, not_restorable, restore_item, text, Family};

/// The English description the daemon stores, per recorded kind.
fn sample(kind: K) -> Option<&'static str> {
    Some(match kind {
        K::Commit => "Commit \"fix login\" (abc1234)",
        K::ComposeCommits => "Compose 3 commits",
        K::Checkout => "Checkout main",
        K::Discard => "Discard 3 files",
        K::DiscardHunk => "Discard a hunk of a.txt",
        K::DiscardLine => "Discard lines of a.txt",
        K::StageFile => "Stage a.txt",
        K::StageFolder => "Stage folder src",
        K::StageHunk => "Stage a hunk of a.txt",
        K::StageLine => "Stage lines of a.txt",
        K::StageAll => "Stage all changes",
        K::UnstageFile => "Unstage a.txt",
        K::UnstageFolder => "Unstage folder src",
        K::UnstageHunk => "Unstage a hunk of a.txt",
        K::UnstageLine => "Unstage lines of a.txt",
        K::UnstageAll => "Unstage all changes",
        K::OperationAbort => "Abort the operation",
        K::PatchApply => "Apply patch fix.patch",
        K::DeleteFile => "Delete a.txt",
        K::ConvertFiles => "Convert 2 files",
        K::AddToGitignore => "Ignore target/",
        K::MailmapAdd => "Merge identities into Alex Doe",
        K::GitattributesTreat => "Treat *.png as binary",
        K::ResolveConflict => "Resolve a.txt",
        K::BranchCreate => "Create branch topic (abc1234)",
        K::BranchCreateTracking => "Create branch topic tracking origin/topic",
        K::BranchDelete => "Delete branch topic (abc1234)",
        K::BranchRename => "Rename branch a to b",
        K::BranchSetUpstream => "Set upstream of a to origin/a",
        K::BranchSetRemotes => "Set remotes of a",
        K::SetRepoConfig => "Set core.eol to lf",
        K::SparseSet => "Set sparse checkout",
        K::SparseDisable => "Disable sparse checkout",
        K::TagCreate => "Create tag v1",
        K::TagDelete => "Delete tag v1",
        K::TagAnnotate => "Annotate tag v1",
        K::FastForward => "Fast-forward main to abc1234",
        K::Reword => "Reword abc1234",
        K::AmendCommit => "Amend abc1234",
        K::SplitCommit => "Split abc1234 into 3 commits",
        K::RewriteApply => "Rewrite history: 412 commits, 2 branches, 4 tags",
        K::FetchRefAsBranch => "Fetch refs/pull/4/head as branch pr-4",
        K::Merge => "Merge topic",
        K::Rebase => "Rebase onto main",
        K::RebaseInteractive => "Interactive rebase",
        K::CherryPick => "Cherry-pick 2 commits",
        K::Revert => "Revert abc1234",
        K::Reset => "Reset (hard) to abc1234",
        K::Pull => "Pull",
        K::FlowRun => "Flow finish feature/x",
        K::StashPush => "Stash \"wip\" (abc1234)",
        K::StashPop => "Pop stash \"wip\" (abc1234)",
        K::StashApply => "Apply stash \"wip\" (abc1234)",
        K::StashDrop => "Drop stash \"wip\" (abc1234)",
        K::RemoteAdd => "Add remote origin",
        K::RemoteRemove => "Remove remote origin",
        K::RemoteRename => "Rename remote a to b",
        K::RemoteSetUrl => "Set the URL of remote origin",
        K::RemoteSetPushUrl => "Set the push URL of remote origin",
        K::LfsTrack => "Track *.psd with LFS",
        K::LfsUntrack => "Stop tracking *.psd with LFS",
        K::JournalRestore => "Restore \"Delete branch topic (abc1234)\"",
        K::ReflogRestore => "Restore main to abc1234",
        K::Outside => "commit: fix login - main",
        _ => return None,
    })
}

/// Recorded kinds whose description is a plain word or has nothing to
/// translate beyond what the stored text says: they never reach the journal
/// today. Listed so a new kind cannot slip in without words. LFS fetch / pull /
/// checkout / push and TagPush are here on purpose: they are server or
/// download operations, not `journaled` (see `label.rs`), so no history row,
/// restore wrapper or retired entry can carry them.
const WITHOUT_WORDS: &[K] = &[
    // Journaled, but their catalog words wait for the translators.
    K::ApplyCommit,
    K::RemoteGroupPlan,
    K::StashFromCommit,
    K::Init,
    K::Fetch,
    K::FetchAll,
    K::FetchRefspec,
    K::Push,
    K::PushForReview,
    K::Clone,
    K::SetSshCommand,
    K::SubmoduleInit,
    K::SubmoduleUpdate,
    K::LfsInstall,
    K::LfsFetch,
    K::LfsPull,
    K::LfsCheckout,
    K::LfsPush,
    K::LfsPrune,
    K::LfsLock,
    K::LfsUnlock,
    K::IndexLockRemove,
    K::BranchDeleteRemote,
    K::WorktreeAdd,
    K::TagPush,
    K::TagDeleteRemote,
    K::TagFetch,
    K::AutoStashStartOver,
    K::Undo,
    K::Redo,
    K::RebaseContinue,
    K::RebaseSkip,
    K::RebaseAbort,
    K::ConflictUnresolve,
    K::OperationSkip,
    K::AmContinue,
    K::AmSkip,
    K::AmAbort,
];

#[test]
fn every_operation_kind_has_words_in_french() {
    let _terms = gitpoulpi_i18n::override_git_terms(true);
    for kind in K::ALL {
        let family = family(kind);
        match sample(kind) {
            Some(stored) => {
                let en = {
                    let _l = gitpoulpi_i18n::override_lang(Lang::En);
                    text(kind, stored)
                };
                // The daemon stores "Checkout <name>"; the UI says the verb
                // ("Check out", docs/developer/l10n/en.md).
                let expected = match stored.strip_prefix("Checkout") {
                    Some(rest) => format!("Check out{rest}"),
                    None => stored.to_string(),
                };
                assert_eq!(en, expected, "{kind:?}: English reads back as stored");
                let fr = {
                    let _l = gitpoulpi_i18n::override_lang(Lang::Fr);
                    text(kind, stored)
                };
                assert_ne!(fr, stored, "{kind:?}: no French words for {stored:?}");
                assert_ne!(
                    family,
                    Family::Other,
                    "{kind:?} has words and no icon family"
                );
            }
            None => assert!(
                family == Family::Other || WITHOUT_WORDS.contains(&kind),
                "{kind:?} is recorded in the journal (icon family {family:?}) but has no \
                 words: give it a sample and a template in label.rs"
            ),
        }
    }
}

#[test]
fn a_kind_the_journal_records_always_has_a_sample() {
    for kind in K::ALL {
        if kind.traits().journaled && !WITHOUT_WORDS.contains(&kind) {
            assert!(sample(kind).is_some(), "{kind:?} is journaled");
        }
    }
}

#[test]
fn the_label_keeps_its_names_and_the_short_id() {
    let _terms = gitpoulpi_i18n::override_git_terms(true);
    let _l = gitpoulpi_i18n::override_lang(Lang::Fr);
    assert_eq!(
        text(K::BranchRename, "Rename branch a to b (abc1234)"),
        "Renommer la branche a en b (abc1234)"
    );
    assert_eq!(
        text(K::Reset, "Reset (hard) to abc1234"),
        "Réinitialiser (dure) sur abc1234"
    );
    assert_eq!(
        text(K::StashDrop, "Drop stash \"wip\" (abc1234)"),
        "Supprimer la remise \"wip\" (abc1234)"
    );
    // A file whose name starts like a verb is still just a file.
    assert_eq!(text(K::StageFile, "Stage folder.txt"), "Indexer folder.txt");
}

#[test]
fn an_outside_change_translates_its_tail_and_keeps_git_reason() {
    let _l = gitpoulpi_i18n::override_lang(Lang::Fr);
    let said = text(
        K::Outside,
        "checkout: moving from main to topic - topic and 2 more (while GitPoulpi was closed)",
    );
    assert!(said.contains("passage de main à topic"), "{said}");
    assert!(said.contains("et 2 autres"), "{said}");
    assert!(
        said.ends_with("(pendant que GitPoulpi était fermé)"),
        "{said}"
    );
    // A reason git wrote that has no words of ours stays as git said it.
    assert_eq!(text(K::Outside, "pull: Fast-forward"), "pull: Fast-forward");
}

#[test]
fn restore_details_are_translated_too() {
    let _l = gitpoulpi_i18n::override_lang(Lang::Fr);
    let item = |kind, label: &str| RestoreItem {
        kind,
        label: label.into(),
    };
    assert_eq!(
        restore_item(&item(RestoreItemKind::OldTip, "old tip of main (abc1234)")),
        "ancienne pointe de main (abc1234)"
    );
    assert_eq!(
        restore_item(&item(RestoreItemKind::Branch, "topic (abc1234)")),
        "topic (abc1234)"
    );
    assert_eq!(
        not_restorable("it only added things, nothing was removed"),
        "elle n'a fait qu'ajouter, rien n'a été supprimé"
    );
    assert_eq!(not_restorable("something new"), "something new");
}

#[test]
fn server_operations_never_reach_the_history() {
    for kind in [
        K::LfsFetch,
        K::LfsPull,
        K::LfsCheckout,
        K::LfsPush,
        K::TagPush,
    ] {
        assert!(
            !kind.traits().journaled,
            "{kind:?} is journaled: give it words in label.rs and the catalogs"
        );
    }
}

#[test]
fn a_history_rewrite_row_keeps_its_three_counts() {
    let _terms = gitpoulpi_i18n::override_git_terms(false);
    let _l = gitpoulpi_i18n::override_lang(Lang::Fr);
    assert_eq!(
        text(
            K::RewriteApply,
            "Rewrite history: 412 commits, 2 branches, 4 tags"
        ),
        "Réécrire l'historique : 412 commits, 2 branches, 4 tags"
    );
    assert_eq!(
        text(
            K::RewriteApply,
            "Rewrite history: 1 commit, 1 branch, 0 tags"
        ),
        "Réécrire l'historique : 1 commit, 1 branche, 0 tag"
    );
    // The write-ahead label of an entry that never finished has no counts:
    // it stays as stored.
    assert_eq!(text(K::RewriteApply, "Rewrite history"), "Rewrite history");
    // With the local words the nouns follow the language's own terms.
    let _terms = gitpoulpi_i18n::override_git_terms(true);
    assert_eq!(
        text(
            K::RewriteApply,
            "Rewrite history: 2 commits, 1 branch, 1 tag"
        ),
        "Réécrire l'historique : 2 validations, 1 branche, 1 étiquette"
    );
}
