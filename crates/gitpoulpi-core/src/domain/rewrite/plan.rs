//! What a preview returns and an apply consumes: the plan (counts, ref
//! moves, warnings, per-commit rows), the outcome, and the progress sink.

use std::ops::ControlFlow;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::domain::Oid;

/// Names one preview's temporary objects under the repository's git dir.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RunId(pub String);

/// Fingerprint of the repository state a plan was computed from (every ref
/// the run moves with its old id, plus the recipe). Apply refuses a plan
/// whose digest no longer matches.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PlanDigest(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Counts {
    pub commits_walked: u64,
    pub commits_changed: u64,
    pub commits_dropped: u64,
    pub refs_moved: u32,
    pub tags_moved: u32,
    /// Signed commits among the changed ones: the number a signature choice
    /// is about.
    pub signed_changed: u64,
    /// Commits the preview signed (re-sign choices): their ids already hold
    /// the new signature.
    #[serde(default)]
    pub resigned: u64,
    /// What the signing took, in milliseconds (0 when nothing was signed).
    #[serde(default)]
    pub signing_millis: u64,
    /// Content rules: distinct new blobs written.
    #[serde(default)]
    pub blobs_changed: u64,
    /// Content rules: distinct matching blobs left alone because they are
    /// binary (a NUL in the first 8 KiB) or not text a pattern can read.
    #[serde(default)]
    pub blobs_skipped_binary: u64,
    /// Content rules: distinct matching blobs left alone because they are
    /// over the rule's size limit.
    #[serde(default)]
    pub blobs_skipped_large: u64,
    /// Encoding rules: distinct matching text blobs left alone because the
    /// source encoding cannot read them or the target cannot hold them.
    #[serde(default)]
    pub blobs_skipped_undecodable: u64,
    /// Applying moves the checked-out commit to another tree: Apply updates
    /// the index and the working tree, and undo puts the files back.
    #[serde(default)]
    pub head_tree_changed: bool,
    /// `update_message_hashes`: abbreviated ids replaced by the new ids.
    #[serde(default)]
    pub message_hashes_updated: u64,
    /// `update_message_hashes`: candidates left alone (ambiguous, unknown,
    /// or pointing at a commit re-created later).
    #[serde(default)]
    pub message_hashes_left: u64,
    /// LFS import: distinct blobs that became pointers.
    #[serde(default)]
    pub lfs_blobs: u64,
    /// LFS import: bytes of file content that move into the LFS store.
    #[serde(default)]
    pub lfs_bytes: u64,
    /// LFS import: distinct objects written to the local LFS store at Apply.
    #[serde(default)]
    pub lfs_objects: u64,
    /// LFS export: distinct pointers that became the real files again.
    #[serde(default)]
    pub lfs_restored: u64,
    /// The run is an LFS import or export: Apply offers "Push LFS objects"
    /// afterwards and the collaborator note mentions LFS.
    #[serde(default)]
    pub lfs_run: bool,
    /// The message-encoding rule: commits it left alone because the
    /// encoding could not be decided.
    #[serde(default)]
    pub encoding_left: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleCount {
    /// Index in `Recipe::rules`.
    pub rule: usize,
    pub commits: u64,
}

/// Whether the old value of a ref is on a remote.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum PushState {
    LocalOnly,
    OnRemote {
        remote: String,
        /// The lease a later force push uses: what the remote had when the
        /// preview ran, not whatever the tracking ref says at push time.
        remote_oid: Oid,
        /// Rewritten commits that exist on the remote.
        rewritten_on_remote: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefChange {
    pub name: String,
    pub old: Oid,
    /// `None`: the ref is deleted (an extraction that leaves it empty).
    pub new: Option<Oid>,
    pub pushed: PushState,
    pub protected: bool,
    /// Whether the user ticked it (protected refs start unticked).
    pub included: bool,
    pub checked_out_in: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RewriteField {
    AuthorName,
    AuthorEmail,
    AuthorDate,
    CommitterName,
    CommitterEmail,
    CommitterDate,
    Message,
    Tree,
    Signature,
    /// The `encoding` header (the message-encoding rule drops it).
    Encoding,
}

/// One changed field; values of secret rules are redacted daemon side.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldChange {
    pub field: RewriteField,
    pub old: String,
    pub new: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "change", rename_all = "snake_case")]
pub enum PathChange {
    Removed {
        path: String,
    },
    Renamed {
        from: String,
        to: String,
    },
    Modified {
        path: String,
    },
    /// A file became an LFS pointer (LFS import).
    LfsImported {
        path: String,
    },
    /// An LFS pointer became the real file again (LFS export).
    LfsExported {
        path: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitChange {
    pub old: Oid,
    /// `None`: the commit is dropped.
    pub new: Option<Oid>,
    pub fields: Vec<FieldChange>,
    pub paths: Vec<PathChange>,
    pub pushed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "warning", rename_all = "snake_case")]
pub enum RewriteWarning {
    /// Rewritten commits lose their signature (or are re-signed, per the
    /// chosen policy).
    SignaturesAffected {
        count: u64,
    },
    PushedRefs {
        count: u32,
    },
    SignedTagsDropped {
        count: u32,
    },
    /// Pushed protected refs left out of the run (list them to include).
    ProtectedExcluded {
        refs: Vec<String>,
    },
    /// Local branches that contain rewritten commits but are not in scope.
    OtherBranchesKeepOldHistory {
        refs: Vec<String>,
    },
    /// Stashes based on rewritten commits keep the old history alive.
    StashesKeepOldHistory {
        count: u32,
    },
}

/// A preview's result. The rows are paged over IPC, so a plan that crosses
/// the wire carries none (`rows` is what the first page holds).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RewritePlan {
    pub run: RunId,
    pub digest: PlanDigest,
    pub counts: Counts,
    pub refs: Vec<RefChange>,
    pub warnings: Vec<RewriteWarning>,
    pub per_rule: Vec<RuleCount>,
    #[serde(default)]
    pub rows: Vec<CommitChange>,
}

/// What an apply did.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RewriteOutcome {
    pub run: RunId,
    /// The refs that moved, old and new ids.
    pub moved: Vec<RefChange>,
    pub commits_changed: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RewritePhase {
    Walking,
    Rewriting,
    Indexing,
    Updating,
    /// Signing the re-created commits; `done` counts the signed ones.
    Signing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RewriteProgress {
    pub phase: RewritePhase,
    pub done: u64,
    pub total: u64,
}

/// Progress callback; returning `Break` cancels the run (nothing is left
/// behind but unreachable objects).
pub type RewriteProgressSink<'a> = &'a mut dyn FnMut(RewriteProgress) -> ControlFlow<()>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plan_round_trips_through_json() {
        let plan = RewritePlan {
            run: RunId("r1".into()),
            digest: PlanDigest("abc".into()),
            counts: Counts {
                commits_walked: 10,
                commits_changed: 3,
                ..Counts::default()
            },
            refs: vec![RefChange {
                name: "refs/heads/main".into(),
                old: Oid([1; 20]),
                new: Some(Oid([2; 20])),
                pushed: PushState::OnRemote {
                    remote: "origin".into(),
                    remote_oid: Oid([1; 20]),
                    rewritten_on_remote: 3,
                },
                protected: true,
                included: false,
                checked_out_in: None,
            }],
            warnings: vec![RewriteWarning::SignaturesAffected { count: 2 }],
            per_rule: vec![RuleCount {
                rule: 0,
                commits: 3,
            }],
            rows: vec![CommitChange {
                old: Oid([1; 20]),
                new: None,
                fields: vec![FieldChange {
                    field: RewriteField::Message,
                    old: "a".into(),
                    new: "b".into(),
                }],
                paths: vec![PathChange::Removed { path: "x".into() }],
                pushed: true,
            }],
        };
        let text = serde_json::to_string(&plan).unwrap();
        assert_eq!(serde_json::from_str::<RewritePlan>(&text).unwrap(), plan);
    }
}
