//! The history rewrite requests and answers as frontends see them
//! (the history-rewrite design, "Preview first").
//!
//! A request is the usual `Request { command, args }`; each type below is
//! the `args` (or the answer) of one `OperationKind`:
//!
//! | `OperationKind` | `args` | answer |
//! |---|---|---|
//! | `RewritePreview` | [`PreviewArgs`] | `RewritePlan` without rows (a job) |
//! | `RewriteRows` | [`RowsArgs`] | [`RowsPage`] |
//! | `RewriteApply` | [`ApplyArgs`] | `RewriteOutcome` (a job, one journal entry) |
//! | `RewritePush` | `PushRewritten` (core) | `PushRewrittenOutcome` (a network job, not journaled) |
//! | `RewritePurgeCheck` | [`PurgeArgs`] | `PurgeCheck` (a read: what a purge deletes and loses, what blocks it, the host checklist) |
//! | `RewritePurge` | [`PurgeArgs`] | `PurgeOutcome` (a job that cannot be stopped once it runs; marks the entries `Purged`) |
//! | `RewriteNote` | [`NoteArgs`] | `RewriteNote` |
//! | `RewriteMaps` | [`MapsArgs`] | `RewriteMaps` |
//! | `RewriteBlobDiff` | [`BlobDiffArgs`] | `BlobChange` |
//! | `RewriteScriptCheck` | [`ScriptCheckArgs`] | `ScriptCheck` |
//! | `RewriteScriptTest` | [`ScriptTestArgs`] | `Vec<ScriptTestRow>` |
//! | `RewriteTrustScript` | [`TrustArgs`] | `null` (a human's act, see [`TrustArgs`]) |
//! | `RewriteDiscard` | [`DiscardArgs`] | `null` |
//! | `RewriteScanSecrets` | [`ScanSecretsArgs`] | `SecretScan` (a job; findings, never values) |
//! | `RewriteAnalyze` | [`AnalyzeArgs`] | `SizeAnalysis` (a job; capped lists and totals) |
//! | `RewriteTools` | [`ToolsArgs`] | `RewriteTools` |
//!
//! While a preview runs, [`crate::Event::RewriteProgress`] reports its phase
//! and the job's own `progress` follows it.

use gitpoulpi_core::domain::rewrite::{CommitChange, PlanDigest, Recipe, RunId, ScriptRef};
use gitpoulpi_core::domain::Oid;
use serde::{Deserialize, Serialize};

/// Which rows of the live preview a page lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RowFilter {
    /// Commits whose id changes (dropped ones included).
    #[default]
    Changed,
    /// Commits the run removes.
    Dropped,
    /// Commits already on a remote.
    Pushed,
    All,
}

impl RowFilter {
    pub fn matches(self, row: &CommitChange) -> bool {
        match self {
            Self::Changed => row.new != Some(row.old),
            Self::Dropped => row.new.is_none(),
            Self::Pushed => row.pushed,
            Self::All => true,
        }
    }
}

/// The biggest page the daemon answers.
pub const MAX_ROWS_PAGE: usize = 200;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreviewArgs {
    pub recipe: Recipe,
    /// Content hashes (`script_hash`) of external scripts a human typed or
    /// ran from the command line on this very request: trusted for it, not
    /// stored. The CLI and the TUI fill it; the MCP server never does, so an
    /// agent's script needs a hash the user trusted. See [`TrustArgs`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub vouched: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RowsArgs {
    pub run: RunId,
    #[serde(default)]
    pub filter: RowFilter,
    #[serde(default)]
    pub offset: usize,
    /// Capped at [`MAX_ROWS_PAGE`].
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    MAX_ROWS_PAGE
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RowsPage {
    pub run: RunId,
    pub filter: RowFilter,
    pub offset: usize,
    /// Rows matching the filter in the whole plan.
    pub total: usize,
    pub rows: Vec<CommitChange>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApplyArgs {
    pub run: RunId,
    pub digest: PlanDigest,
    /// The plan moves refs that are on a remote: without this the daemon
    /// answers `NeedsConfirmation` and changes nothing.
    #[serde(default)]
    pub confirmed: bool,
    /// The folder of the new repository when the recipe extracts a
    /// subdirectory into a clone or as a submodule (`rewrite_extract`
    /// instead of an apply here); absent for every other plan.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<std::path::PathBuf>,
}

/// Which history rewrite journal entries a purge is about. Without `entries`
/// and `all`, the newest rewrite that is not purged yet.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PurgeArgs {
    /// Journal entry ids (`gitpoulpi undo --list`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entries: Vec<u64>,
    /// Every rewrite entry of the undo history.
    #[serde(default)]
    pub all: bool,
}

/// The collaborator note of an applied run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteArgs {
    pub run: RunId,
}

/// The `commit-map` and `ref-map` of an applied run, optionally copied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapsArgs {
    pub run: RunId,
    /// Copy the two files into this directory (created when missing).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub out: Option<std::path::PathBuf>,
}

/// What a content rule did to one file of one commit of a previewed run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlobDiffArgs {
    pub run: RunId,
    /// The commit as it is before the rewrite (the row's `old`).
    pub commit: Oid,
    pub path: String,
}

/// Compile a script and read its header.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptCheckArgs {
    pub script: ScriptRef,
    /// As [`PreviewArgs::vouched`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub vouched: Vec<String>,
}

/// Run a script on some commits without writing anything.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptTestArgs {
    pub script: ScriptRef,
    pub commits: Vec<Oid>,
    /// As [`PreviewArgs::vouched`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub vouched: Vec<String>,
}

/// Trust an external script, by the content hash of its text, in this
/// repository's state. **Only a human grants trust**: the TUI's question and
/// `gitpoulpi rewrite script trust` send this; the MCP server has no tool for
/// it and refuses the CLI's `--trust` flags, so an agent can run a script only
/// after the user trusted that exact text. (The socket itself is not a
/// boundary: whoever can write to it can run `git` anyway.)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustArgs {
    pub hash: String,
    /// What the script is called (a path), shown when listing trust.
    #[serde(default)]
    pub label: String,
}

/// Scan the history a scope walks for secrets.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ScanSecretsArgs {
    #[serde(default)]
    pub scope: gitpoulpi_core::domain::rewrite::Scope,
}

/// The size analysis of the history a scope walks.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AnalyzeArgs {
    #[serde(default)]
    pub scope: gitpoulpi_core::domain::rewrite::Scope,
    /// Rows kept in each list; 0 means the default.
    #[serde(default)]
    pub top: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscardArgs {
    pub run: RunId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ToolsArgs {
    /// Probe again instead of answering from the one-minute cache.
    #[serde(default)]
    pub fresh: bool,
}
