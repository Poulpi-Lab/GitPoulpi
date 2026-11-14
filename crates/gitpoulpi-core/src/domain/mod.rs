mod amend;
pub mod b64;
mod change_scope;
mod conflict;
pub mod conflict_prevention;
mod content;
mod diff;
mod file_history;
mod image_info;
mod lfs;
mod merge_doc;
mod mermaid;
mod network_auth;
mod operation;
mod ops;
mod ref_filter;
mod reflog;
mod refs;
mod remote_group;
mod remote_group_plan;
mod remote_group_run;
mod repo;
mod repo_auth;
mod restore;
pub mod rewrite;
mod rules;
mod search;
mod split;
mod stash;
mod stuck;
mod submodule;
mod ui_control;
mod undo;
mod worktree;

pub use amend::{parse_git_date, CommitAmend, GitDate, ResolvedAmend};
pub use change_scope::ChangeScope;
pub use conflict::{
    encode_text, ConflictEntry, ConflictKind, ConflictRefusal, ConflictResolution, ConflictSet,
    ConflictSide, EncodeError, OperationState, ResolveOptions, StageEntry, MODE_GITLINK,
    MODE_SYMLINK,
};
pub use content::{
    commit_summary, decode_commit_text, decode_text, decode_text_lossy, fallback_encoding_name,
    with_fallback_encoding, Blame, BlameContent, BlameLine, Blob, BlobBody, BlobBytes, BlobContent,
    BlobSource, MAX_TEXT_BLOB_BYTES,
};
pub use diff::{
    CommitStat, DiffFileList, DiffLine, DiffTarget, DiscardTarget, FileDiff, FileStatus, Hunk,
    HunkId, LineOrigin, LineSelector, StageTarget, StatusEntry,
};
pub use file_history::{FileChangeKind, FileHistoryEntry, FileHistoryPage};
pub use image_info::{sniff as sniff_image, ImageFormat, ImageInfo};
pub use lfs::{
    attributes_mention_lfs, describe_pointer, human_size, lfs_change, parse_lfs_patterns,
    LfsBinary, LfsChange, LfsCloneMode, LfsDirection, LfsFetchOptions, LfsFile, LfsInstallScope,
    LfsLock, LfsPattern, LfsPointer, LfsProgress, LfsProgressSink, LfsPushOptions, LfsRepoStatus,
    LfsTrackOptions, LfsUnavailableReason, LFS_INSTALL_URL, LFS_MIN_VERSION, LFS_POINTER_MAX_BYTES,
};
pub use merge_doc::{
    aligned_rows, classify_sides, compose_resolution, has_conflict_markers, is_whitespace_only,
    marker_size_for, parse_merge_file_output, render_with_markers, side_changes, side_rows,
    whitespace_choice, whitespace_resolution, AlignedRow, AlignedRows, ConflictOperation,
    ConflictPicks, Eol, LineChange, MarkerStyle, MergeDocument, MergeJob, MergeRegion, Pick,
    RawStage, RowMark, SideBlob, SideChanges, SideLabels, SideRow, WorktreeFileState,
    MIN_MARKER_SIZE,
};
pub use mermaid::{
    attach_refs, commits_to_tip, mermaid_git_graph, MermaidDetail, MermaidOptions,
    MermaidOrientation,
};
pub use network_auth::{
    strip_userinfo, transport_of, NetworkAuth, RemoteAccessOptions, RemoteAccessReport,
    RemoteTransport,
};
pub use operation::{OpTraits, Operation, OperationKind};
pub use ops::{
    autosquash_order, normalize_reword_message, normalize_sparse_dirs, parse_fetch_refspec,
    validate_plan, ApplyRefusal, CheckoutTarget, CherryPickOptions, CherryPickOutcome,
    CloneOptions, ClonePhase, CloneProgress, CloneProgressSink, CommitAuthor, CommitOptions,
    FastForwardOutcome, FetchOptions, FetchOutcome, InitOptions, InteractiveRebase, LogOptions,
    MergeFastForward, MergeOptions, MergeOutcome, PatchApplied, PatchCreated, PatchSource,
    PlanCommit, PullMode, PullOptions, PullOutcome, PushLease, PushOptions, PushOutcome,
    PushRejectReason, PushRejection, RebaseAction, RebaseOptions, RebaseOutcome, RebasePlanDraft,
    RebaseRefusal, RebaseState, RebaseStep, ResetMode, RevertOutcome, RewordOutcome, RewordRefusal,
    SparseState, TagSelection, WorktreeAddOptions, WorktreeChange, WorktreeCommit,
    WorktreeRemoveOptions, WorktreeRemoved, WorktreeStart, FETCH_REFSPEC_NAMESPACE,
};
pub use ref_filter::{visible_tips, RefFilter};
pub use reflog::{
    reflog_refname, LostCommit, LostCommitOptions, LostCommitsPage, LostSource, ReflogEntry,
    ReflogPage, ReflogPosition, ZERO_OID,
};
pub use refs::{leftover_remote_refs, split_remote_ref};
pub use refs::{
    parse_remote_tag_mirror_ref, remote_tag_mirror_ref, Ref, RefKind, RefVisibility, TagSync,
    TagWhere, REMOTE_TAG_MIRROR_PREFIX,
};
pub use remote_group::{
    drop_remote_in, entries_with_remotes, is_remote_group_key, parse_branch_remotes,
    rename_remote_in, value_remote, BranchRemotes, RemoteGroupStatus, RemoteRelation,
    RemoteTargetState, TargetRef, TargetRole, PULL_FROM_VARIABLE, PUSH_TO_VARIABLE,
};
pub use remote_group_plan::{
    classify, normalize_url, suggest, GroupAction, GroupMode, PlannedTarget, RelationFacts,
    RemoteClass, RemoteGroupPlan, RemoteGroupPlanRequest, SuggestContext, Suggestion,
};
pub use remote_group_run::{
    GroupResume, GroupRunStatus, Integration, PendingGroup, RemoteGroupOutcome,
    RemoteGroupRunRequest, RunTarget, StateTarget, StoppedAt, StoppedOp, TargetResult,
    TargetStatus, REMOTE_GROUP_STATE_VERSION,
};
pub use repo::{
    CommitGraph, CommitNode, DiscoveredRepo, Oid, Remote, RepoId, RepoRef, WorkingTree,
};
pub use repo_auth::{
    command_sha256, ActiveIdentity, AuthOverride, HttpsAuth, HttpsIdentity, IdentityVia,
    RemoteIdentity, RepoAuthProfile, RepoAuthSetArgs, RepoAuthTestArgs, RepoAuthView, SshAuth,
    SshIdentity, COMMAND_NEEDS_PERSON,
};
pub use restore::{RestoreItem, RestoreItemKind, RestoreOffer, RestoreOutcome};
pub use rewrite::RewriteRefusal;
pub use rules::{
    attribute_field, attributes_set, candidate_files, rule_dir, rule_pattern, AttributesWrite,
    RuleKind, RuleScope, Treat, INFO_ATTRIBUTES, INFO_EXCLUDE,
};
pub use search::{
    parse_query, split_message, CommitMessage, CommitMessages, CommitMessagesQuery, ParsedQuery,
    SearchHit, SearchHits, SearchKind, SearchRequest, SearchTarget,
};
pub use split::{
    unassigned, validate_split, SplitCommit, SplitFile, SplitOutcome, SplitPart, SplitPiece,
    SplitRefusal, SplitSource, WholeReason,
};
pub use stash::{Stash, StashId};
pub use stuck::{AmOutcome, IndexLock, STALE_INDEX_LOCK_SECS};
pub use submodule::{Submodule, SubmoduleState, SubmoduleUpdateOptions};
pub use ui_control::UiControl;
pub use undo::{
    short, short_opt, Direction, EntrySource, EntryState, EntrySummary, FileHash, HeadState,
    JournalListing, JournalState, ListedEntry, PathHash, RefScope, RefSnapshot, RefUpdate,
    RemoteRefMoved, Snapshot, UndoOutcome, UndoRefusal, UNDO_DISABLED_MESSAGE, UNDO_PIN_PREFIX,
};
pub use worktree::Worktree;
