//! History rewrite: the pure model (see the history-rewrite design).
//!
//! A rewrite is a [`Recipe`]: a scope, an ordered list of rules and run
//! options. Everything here is pure (no I/O, no clock): the engine in the
//! backends, the daemon, the CLI/MCP and the TUI workbench all run the same
//! validation and the same date, identity, message and path functions, so
//! what the editor accepts is what the engine runs and a preview matches the
//! apply that follows.
//!
//! Maintainer decisions that shape the model (2026-10-21): the original committer
//! is kept unless the run says "me, now" ([`CommitterPolicy`]); signatures
//! have no default, the request carries the user's explicit
//! [`SignaturePolicy`] whenever signed commits are affected; extracting a
//! subdirectory asks for its [`ExtractMode`] each time; protected branches
//! that are already pushed are rewritten only when listed in
//! [`ScopeOptions::include_protected`].

pub mod analysis;
mod blob;
pub mod dates;
pub mod encoding;
pub mod hashes;
pub mod host_protection;
pub mod identity;
pub mod lfs;
pub mod mailmap;
pub mod message;
pub mod paths;
mod plan;
mod purge;
mod push;
mod recipe;
pub mod script;
pub mod secrets;
pub mod textconv;
mod tools;
pub mod trailers;

pub use analysis::{
    glob_for, human_size, BlobRow, BlobState, DeletedRow, FolderRow, SizeAnalysis, DEFAULT_TOP,
    MAX_TOP,
};
pub use blob::{
    blob_diff, BlobChange, BlobChangeKind, BlobHunk, BlobLine, BlobLineKind, REDACTED_LINE,
    REMOVED_TEXT,
};
pub use dates::{
    in_window, set_offset, shift, window, window_problem, Direction, GitDate, KeepClock, NewTimes,
    WeekWindow, Weekday, WhichDate, WindowInput, WindowOffset, WorkWindowRule, MAX_OFFSET_MINUTES,
};
pub use host_protection::{HostProtection, HowStep, Howto, HowtoWay, Loss, Way};
pub use identity::{
    anonymise_problem, AnonymiseMap, AnonymiseRule, CompiledMatch, Ident, IdentityMatch,
    IdentityRule, IdentitySet, Who,
};
pub use mailmap::{IdentityCount, IdentityList, Mailmap, MailmapAdded};
pub use message::{CompiledPattern, TextPattern, TextReplace};
pub use paths::{validate_glob, ExtractMode, GlobSet, PathMove};
pub use plan::{
    CommitChange, Counts, FieldChange, PathChange, PlanDigest, PushState, RefChange, RewriteField,
    RewriteOutcome, RewritePhase, RewritePlan, RewriteProgress, RewriteProgressSink,
    RewriteWarning, RuleCount, RunId,
};
pub use purge::{
    confirmation_text, host_steps, HostChecklist, HostKind, HostStep, PurgeBlocker,
    PurgeBlockerKind, PurgeCheck, PurgeEntry, PurgeEntryInfo, PurgeOutcome, PurgeProgressSink,
    PurgeRequest, PurgeStatus, PurgeStep,
};
pub use push::{
    note_refs, note_text, NoteRef, NoteRefKind, PushRefResult, PushRefStatus, PushRewritten,
    PushRewrittenOutcome, PushedRef, RewriteMaps, RewriteNote, RewriteRecord,
    DEFAULT_PROTECTED_BRANCHES, PUSHED_FILE,
};
pub use recipe::{
    validate_recipe, CommitterPolicy, ContentFind, ContentReplace, LfsExportRule, LfsImportRule,
    PruneEmpty, Recipe, RecipeIssue, RewriteRefusal, Rule, RuleKind, RunOptions, Scope,
    ScopeOptions, ScriptLanguage, ScriptRef, ScriptSource, SecretText, SignaturePolicy,
    TrailerAction, TrailerIfExists, TrailerRule, RECIPE_VERSION,
};
pub use recipe::{EolRule, ReencodeRule, StripExecRule};
pub use script::{script_hash, ScriptPolicy, ScriptTrust};
pub use secrets::{SecretFinding, SecretMatch, SecretScan, SecretValue};
pub use textconv::{LineEnding, PathFilter, TextEncoding};
pub use tools::{
    OsFamily, RewriteFeature, RewriteTools, ScriptSwitches, SigningCheck, SigningFormat, Tool,
    ToolCheck, ToolRequirement, ToolStatus, SHELLS,
};
