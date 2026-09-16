//! The recipe: a scope, an ordered list of rules and run options. The same
//! value is TOML on disk (shared in the repository or only for the user),
//! JSON on the wire, and what the TUI workbench edits.
//!
//! [`validate_recipe`] is the one pure check the editor, the daemon and the
//! CLI all run, so a recipe the editor accepts is a recipe the engine accepts.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use super::dates::{window_problem, KeepClock, WhichDate, WorkWindowRule, MAX_OFFSET_MINUTES};
use super::identity::{
    anonymise_problem, set_problem, AnonymiseRule, CompiledMatch, IdentityRule, Who,
};
use super::message::{CompiledPattern, TextPattern, TextReplace};
use super::paths::{validate_glob, ExtractMode, PathMove};
use super::script::ScriptPolicy;
use super::textconv::{LineEnding, TextEncoding};
pub use super::trailers::TrailerIfExists;
use crate::domain::Oid;

/// The recipe format version this build reads and writes.
pub const RECIPE_VERSION: u32 = 1;

/// Why a rewrite was refused before anything changed. Fieldless and `Copy`
/// so it crosses the IPC wire in [`crate::TypedError`]; the details (which
/// rule, which path) travel in the error's text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RewriteRefusal {
    /// A merge, rebase, cherry-pick, revert or bisect is in progress.
    OperationInProgress,
    /// An auto-stash is waiting to be restored.
    AutoStashPending,
    /// A shallow clone has incomplete history.
    Shallow,
    /// A partial clone, for rules that read file content.
    PartialClone,
    /// A SHA-256 repository (object ids here are 20 bytes).
    UnsupportedObjectFormat,
    /// The scope names no commit or ref.
    EmptyScope,
    /// No enabled rule.
    NoRule,
    /// A rule failed validation.
    InvalidRule,
    /// A path the rewrite would change has uncommitted edits.
    DirtyPathsTouched,
    /// A branch is checked out in another worktree and the run changes
    /// content.
    CheckedOutElsewhere,
    /// A pushed protected branch is in the scope without being listed in
    /// `include_protected`.
    ProtectedRef,
    /// The repository changed since the preview.
    Stale,
    /// A tool the recipe needs is missing.
    ToolMissing,
    /// A script failed.
    ScriptFailed,
    /// The undo data was purged.
    Purged,
    /// A script's language is switched off in the settings
    /// (`rewrite.external_scripts`).
    ScriptsDisabled,
    /// An external script from the repository (or a shared recipe) has not
    /// been trusted by the user; the trust is keyed by its content hash.
    ScriptNotTrusted,
    /// Signed commits are affected and the request carries no
    /// `options.signatures` choice. There is no silent default.
    SignatureChoiceNeeded,
    /// An "extract subdirectory" rule carries no mode. Asked each time.
    ExtractModeNeeded,
    /// Re-signing was chosen and the signing smoke test failed (key
    /// missing, agent locked, program refused); the detail says why.
    SigningFailed,
    /// An LFS export needs objects that are not in the local LFS store; the
    /// detail lists them (one `<oid> <path>` per line) so they can be fetched.
    LfsObjectsMissing,
    /// A path rule would put a file where one already is (in some commit);
    /// the detail lists the commits and paths. Never overwritten silently.
    PathCollision,
}

impl RewriteRefusal {
    /// A stable snake_case code for the CLI and MCP.
    pub fn code(self) -> &'static str {
        match self {
            Self::OperationInProgress => "operation_in_progress",
            Self::AutoStashPending => "auto_stash_pending",
            Self::Shallow => "shallow",
            Self::PartialClone => "partial_clone",
            Self::UnsupportedObjectFormat => "unsupported_object_format",
            Self::EmptyScope => "empty_scope",
            Self::NoRule => "no_rule",
            Self::InvalidRule => "invalid_rule",
            Self::DirtyPathsTouched => "dirty_paths_touched",
            Self::CheckedOutElsewhere => "checked_out_elsewhere",
            Self::ProtectedRef => "protected_ref",
            Self::Stale => "stale",
            Self::ToolMissing => "tool_missing",
            Self::ScriptFailed => "script_failed",
            Self::Purged => "purged",
            Self::ScriptsDisabled => "scripts_disabled",
            Self::ScriptNotTrusted => "script_not_trusted",
            Self::SignatureChoiceNeeded => "signature_choice_needed",
            Self::ExtractModeNeeded => "extract_mode_needed",
            Self::SigningFailed => "signing_failed",
            Self::LfsObjectsMissing => "lfs_objects_missing",
            Self::PathCollision => "path_collision",
        }
    }
}

/// Which refs move and which commits may change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "scope", rename_all = "snake_case")]
pub enum Scope {
    /// HEAD's branch, optionally limited to the commits after `since`.
    CurrentBranch {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        since: Option<Oid>,
    },
    /// Only these commits change (a Log multi-selection); their descendants
    /// are replayed with new parents.
    Selection { commits: Vec<Oid> },
    /// The commits in `base..tip`.
    Range { base: Oid, tip: Oid },
    /// Explicit local branches and tags.
    Refs { refs: Vec<String> },
    /// Every local branch and tag.
    AllLocal,
}

impl Default for Scope {
    fn default() -> Self {
        Scope::CurrentBranch { since: None }
    }
}

/// Options on the scope that are not part of its kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeOptions {
    /// Move tags that point at rewritten commits (default on).
    #[serde(default = "yes")]
    pub include_tags: bool,
    /// Protected branches that are already pushed are excluded unless listed
    /// here, one by one ("check all" fills the list from the preview).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub include_protected: Vec<String>,
    /// More refs to move (the "add to scope" of the preview).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_refs: Vec<String>,
    /// Branches the caller already knows are protected (read from a state
    /// directory the backend cannot guess); added to the bases the repository's
    /// `workflow.toml` marks `protected`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub protected_branches: Vec<String>,
    /// What the host protects (S23), read client side when the repository is
    /// linked: its patterns join the protected names of the preview, and the
    /// rest explains how to get a blocked rewrite through.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_protection: Option<super::HostProtection>,
}

fn yes() -> bool {
    true
}

impl Default for ScopeOptions {
    fn default() -> Self {
        Self {
            include_tags: true,
            include_protected: Vec::new(),
            extra_refs: Vec::new(),
            protected_branches: Vec::new(),
            host_protection: None,
        }
    }
}

/// The committer line of rewritten commits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommitterPolicy {
    /// Keep the original committer and date (a bulk rewrite must not stamp
    /// today on thousands of commits). Only a rule changes it.
    #[default]
    Keep,
    /// "Me, now": the configured identity and the current time, as reword.
    MeNow,
}

/// What happens to the signatures of rewritten commits. Asked each time
/// signed commits are affected: [`RunOptions::signatures`] is an `Option`
/// with no default and the engine refuses with
/// [`RewriteRefusal::SignatureChoiceNeeded`] while it is `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignaturePolicy {
    /// Drop the signatures of rewritten commits (the count is shown first).
    Strip,
    /// Re-sign only the rewritten commits that were signed.
    ResignSigned,
    /// Re-sign every rewritten commit.
    ResignAll,
}

/// Commits that become empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PruneEmpty {
    /// Drop a commit that a content rule emptied (never one that was empty
    /// before).
    #[default]
    Auto,
    Never,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunOptions {
    #[serde(default)]
    pub committer: CommitterPolicy,
    /// `None` until the user chose; see [`SignaturePolicy`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signatures: Option<SignaturePolicy>,
    #[serde(default)]
    pub prune_empty: PruneEmpty,
    /// Rewrite abbreviated commit ids (7+ hex characters) in messages to
    /// the new ids. Off by default.
    #[serde(default)]
    pub update_message_hashes: bool,
    /// Also keep visible `refs/backup/...` refs. Off by default: the undo
    /// pins are the backup.
    #[serde(default)]
    pub keep_backup_refs: bool,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            committer: CommitterPolicy::Keep,
            signatures: None,
            prune_empty: PruneEmpty::Auto,
            update_message_hashes: false,
            keep_backup_refs: false,
        }
    }
}

/// A secret literal typed by the user. Its `Debug` is redacted and a recipe
/// that is saved ([`Recipe::saved`]) never contains it.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SecretText(pub String);

impl SecretText {
    /// The scanner finding this names, when the rule scans: with
    /// `scan: true`, a literal shaped like a finding id (`f-` and twelve hex
    /// digits) selects that finding, and the daemon's backend resolves it to
    /// the value. The TUI never holds the value, only the id.
    pub fn finding_id(&self) -> Option<&str> {
        super::secrets::is_finding_id(&self.0).then_some(self.0.as_str())
    }
}

impl fmt::Debug for SecretText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretText(<redacted>)")
    }
}

/// What a content rule looks for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "find", rename_all = "snake_case")]
pub enum ContentFind {
    Pattern {
        #[serde(flatten)]
        pattern: TextPattern,
    },
    /// Secret literals (replaced by `***REMOVED***` and never shown), and
    /// optionally what the built-in scanner finds. The literals are not part
    /// of a saved recipe.
    ///
    /// With `scan: true` the backend scans the recipe's scope when the
    /// preview runs. A literal shaped like a finding id
    /// ([`SecretText::finding_id`]) picks that finding; with no id at all,
    /// every finding is removed. Ids are not secrets: a saved recipe keeps
    /// them, and an id the scan no longer finds is an `InvalidRule`.
    Secrets {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        literals: Vec<SecretText>,
        #[serde(default)]
        scan: bool,
    },
}

/// Replace text inside the files whose path matches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentReplace {
    /// Path globs; empty means every file.
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(flatten)]
    pub find: ContentFind,
    #[serde(default)]
    pub with: String,
    /// Blobs larger than this are skipped.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_blob_bytes: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrailerAction {
    /// Add `key: value` (what to do when the key is there: `if_exists`).
    Add,
    /// Remove every `key:` trailer (only those whose value matches `find`,
    /// when it is set).
    Remove,
    /// Replace text matching `find` with `with` in the values of the `key`
    /// trailers (an old email by a new one in `Signed-off-by`).
    Map,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrailerRule {
    pub action: TrailerAction,
    /// `Signed-off-by`, `Co-authored-by`... Matched without regard to case.
    pub key: String,
    /// `Add`: the value.
    #[serde(default)]
    pub value: String,
    /// `Add`: when the key is already there.
    #[serde(default)]
    pub if_exists: TrailerIfExists,
    /// `Remove`: only trailers whose value matches (all when `None`).
    /// `Map`: what to replace.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub find: Option<TextPattern>,
    /// `Map`: the replacement (`$1` in regex mode).
    #[serde(default)]
    pub with: String,
}

impl TrailerRule {
    pub fn add(key: &str, value: &str) -> Self {
        Self {
            action: TrailerAction::Add,
            key: key.to_string(),
            value: value.to_string(),
            if_exists: TrailerIfExists::Add,
            find: None,
            with: String::new(),
        }
    }

    pub fn remove(key: &str) -> Self {
        Self {
            action: TrailerAction::Remove,
            ..Self::add(key, "")
        }
    }

    /// Literal `old` -> `new` in the values of `key`.
    pub fn map(key: &str, old: &str, new: &str) -> Self {
        Self {
            action: TrailerAction::Map,
            find: Some(TextPattern::literal(old)),
            with: new.to_string(),
            ..Self::add(key, "")
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScriptLanguage {
    /// The embedded, sandboxed interpreter (Lua). Always available.
    Lua,
    /// A shell script (bash or sh; pwsh or powershell for a PowerShell
    /// script), run as an external process speaking JSON lines. Allowed by
    /// default when a shell is found.
    Shell,
    /// A Python script, external process. Off by default in settings.
    Python,
    /// A Lua script run by an installed `lua` (not the embedded one), external
    /// process. Off by default in settings.
    LuaExternal,
}

impl ScriptLanguage {
    /// Every language, in the order the rule form offers them.
    pub const ALL: [ScriptLanguage; 4] = [
        ScriptLanguage::Lua,
        ScriptLanguage::Shell,
        ScriptLanguage::Python,
        ScriptLanguage::LuaExternal,
    ];

    /// Runs as a process of its own, unsandboxed, with the user's rights.
    pub fn is_external(self) -> bool {
        !matches!(self, ScriptLanguage::Lua)
    }

    /// The stable name used by the CLI (`--script-lang`) and in messages.
    pub fn name(self) -> &'static str {
        match self {
            Self::Lua => "lua",
            Self::Shell => "shell",
            Self::Python => "python",
            Self::LuaExternal => "lua-external",
        }
    }

    /// Parses [`ScriptLanguage::name`].
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|l| l.name() == name)
    }

    /// The value of `rewrite.external_scripts` that switches this language
    /// on (`None`: the embedded Lua, always available).
    pub fn setting_name(self) -> Option<&'static str> {
        match self {
            Self::Lua => None,
            Self::Shell => Some("shell"),
            Self::Python => Some("python"),
            Self::LuaExternal => Some("lua"),
        }
    }

    /// The language a script file's extension suggests.
    pub fn from_extension(path: &str) -> Option<Self> {
        let ext = path.rsplit_once('.')?.1.to_ascii_lowercase();
        match ext.as_str() {
            "lua" => Some(Self::Lua),
            "sh" | "bash" | "ps1" => Some(Self::Shell),
            "py" => Some(Self::Python),
            _ => None,
        }
    }
}

/// Where a script's text comes from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "from", rename_all = "snake_case")]
pub enum ScriptSource {
    Inline {
        text: String,
    },
    /// A path relative to the repository.
    File {
        path: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptRef {
    pub language: ScriptLanguage,
    pub source: ScriptSource,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub params: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LfsImportRule {
    /// Path globs (`.gitattributes` style) to move into LFS.
    pub patterns: Vec<String>,
    /// Only files of at least this many bytes move (absent: every match).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_size: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LfsExportRule {
    /// Path globs whose LFS pointers become the real files again.
    pub patterns: Vec<String>,
}

/// S22: every text file that matches gets one line ending.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EolRule {
    pub target: LineEnding,
    /// Path globs; empty means every file. Binary files are always skipped.
    #[serde(default)]
    pub paths: Vec<String>,
    /// Path globs the rule leaves alone.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
    /// Follow the root `.gitattributes` of each commit: `-text` / `binary`
    /// files are skipped and `eol=lf|crlf` files get that ending. Off by
    /// default: the rule's target and path filters decide alone.
    #[serde(default, skip_serializing_if = "is_false")]
    pub follow_attributes: bool,
    /// Blobs larger than this are skipped.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_blob_bytes: Option<u64>,
}

/// S22: the exec bit (mode 100755) leaves the files that match.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StripExecRule {
    /// Path globs (at least one: stripping every exec bit is never meant).
    pub paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
}

fn is_false(b: &bool) -> bool {
    !*b
}

fn default_fallback() -> TextEncoding {
    TextEncoding::Windows1252
}

fn is_default_fallback(e: &TextEncoding) -> bool {
    *e == TextEncoding::Windows1252
}

/// S24bis: the text encoding of the files that match becomes `target`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReencodeRule {
    pub target: TextEncoding,
    /// The encoding the files are in. Absent: detected per file (byte order
    /// mark, then valid UTF-8, then `fallback`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<TextEncoding>,
    /// What a file that is neither marked nor valid UTF-8 is read as.
    #[serde(
        default = "default_fallback",
        skip_serializing_if = "is_default_fallback"
    )]
    pub fallback: TextEncoding,
    /// Path globs; empty means every file. Binary files are always skipped.
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
    /// Blobs larger than this are skipped.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_blob_bytes: Option<u64>,
}

/// One step of a recipe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RuleKind {
    Identity(IdentityRule),
    /// Apply the repository's `.mailmap`.
    Mailmap {
        #[serde(default)]
        apply_to: Who,
        /// The `.mailmap` to apply: a path relative to the worktree, or an
        /// absolute one. Absent: the worktree's own `.mailmap`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        file: Option<String>,
    },
    Anonymise(AnonymiseRule),
    DateShift {
        by_seconds: i64,
        #[serde(default)]
        apply_to: WhichDate,
    },
    DateTimezone {
        offset_minutes: i16,
        keep: KeepClock,
        #[serde(default)]
        apply_to: WhichDate,
    },
    DateWindow(WorkWindowRule),
    /// Committer date (and identity) become the author's.
    CommitterFromAuthor,
    MessageReplace(TextReplace),
    MessageDeleteLines {
        pattern: TextPattern,
    },
    Trailer(TrailerRule),
    ContentReplace(ContentReplace),
    RemovePaths {
        globs: Vec<String>,
    },
    RemoveBlobsOver {
        bytes: u64,
    },
    MovePaths(PathMove),
    LfsImport(LfsImportRule),
    LfsExport(LfsExportRule),
    /// S22: one line ending for the matching text files.
    LineEndings(EolRule),
    /// S22: the exec bit leaves the matching files.
    StripExec(StripExecRule),
    /// S24bis: one text encoding for the matching text files.
    Reencode(ReencodeRule),
    Script(ScriptRef),
    /// Strip the signature of every commit in scope, even where nothing
    /// else changes.
    RemoveSignatures,
    /// Re-encode commit messages to UTF-8 (S24): messages with an
    /// `encoding` header are decoded and the header goes; UTF-8 read once
    /// as Latin-1 / Windows-1252 is repaired when the round trip is exact.
    /// What cannot be decided is left alone and counted.
    MessageEncoding {
        /// Also repair the author and committer names the same way (the
        /// names of a commit whose header goes are always decoded: they
        /// were in the same encoding).
        #[serde(default)]
        names: bool,
    },
}

impl RuleKind {
    /// A short stable name for labels and CLI output.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Identity(_) => "identity",
            Self::Mailmap { .. } => "mailmap",
            Self::Anonymise(_) => "anonymise",
            Self::DateShift { .. } => "date_shift",
            Self::DateTimezone { .. } => "date_timezone",
            Self::DateWindow(_) => "date_window",
            Self::CommitterFromAuthor => "committer_from_author",
            Self::MessageReplace(_) => "message_replace",
            Self::MessageDeleteLines { .. } => "message_delete_lines",
            Self::Trailer(_) => "trailer",
            Self::ContentReplace(_) => "content_replace",
            Self::RemovePaths { .. } => "remove_paths",
            Self::RemoveBlobsOver { .. } => "remove_blobs_over",
            Self::MovePaths(_) => "move_paths",
            Self::LfsImport(_) => "lfs_import",
            Self::LfsExport(_) => "lfs_export",
            Self::LineEndings(_) => "line_endings",
            Self::StripExec(_) => "strip_exec",
            Self::Reencode(_) => "reencode",
            Self::Script(_) => "script",
            Self::RemoveSignatures => "remove_signatures",
            Self::MessageEncoding { .. } => "message_encoding",
        }
    }

    /// `true` for rules that read or change file content (they need the
    /// objects, refuse partial clones and touch the working tree).
    pub fn changes_content(&self) -> bool {
        matches!(
            self,
            Self::ContentReplace(_)
                | Self::RemovePaths { .. }
                | Self::RemoveBlobsOver { .. }
                | Self::MovePaths(_)
                | Self::LfsImport(_)
                | Self::LfsExport(_)
                | Self::LineEndings(_)
                | Self::StripExec(_)
                | Self::Reencode(_)
                | Self::Script(_)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rule {
    #[serde(default = "yes")]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub kind: RuleKind,
}

impl Rule {
    pub fn new(kind: RuleKind) -> Self {
        Self {
            enabled: true,
            label: None,
            kind,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recipe {
    #[serde(default = "version")]
    pub version: u32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub scope: Scope,
    #[serde(default)]
    pub scope_options: ScopeOptions,
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub options: RunOptions,
    /// What external scripts may do on this run. **Filled by the daemon from
    /// the settings and the trust store; whatever a client sends here is
    /// replaced.** Never saved.
    #[serde(default, skip_serializing_if = "ScriptPolicy::is_default")]
    pub script_policy: ScriptPolicy,
}

fn version() -> u32 {
    RECIPE_VERSION
}

impl Recipe {
    pub fn new(name: &str, scope: Scope, rules: Vec<Rule>) -> Self {
        Self {
            version: RECIPE_VERSION,
            name: name.to_string(),
            scope,
            scope_options: ScopeOptions::default(),
            rules,
            options: RunOptions::default(),
            script_policy: ScriptPolicy::default(),
        }
    }

    /// The rules that run, in order, with their index in `rules`.
    pub fn enabled_rules(&self) -> impl Iterator<Item = (usize, &Rule)> {
        self.rules.iter().enumerate().filter(|(_, r)| r.enabled)
    }

    /// The enabled "extract a subdirectory" rule: its directory and mode.
    pub fn extraction(&self) -> Option<(&str, Option<ExtractMode>)> {
        self.enabled_rules().find_map(|(_, r)| match &r.kind {
            RuleKind::MovePaths(PathMove::ExtractSubdir { dir, mode }) => {
                Some((dir.as_str(), *mode))
            }
            _ => None,
        })
    }

    /// The copy that may be written to disk: secret literals removed. A
    /// secret rule loaded back has to be given its values again.
    pub fn saved(&self) -> Recipe {
        let mut copy = self.clone();
        copy.script_policy = ScriptPolicy::default();
        for rule in &mut copy.rules {
            if let RuleKind::ContentReplace(c) = &mut rule.kind {
                if let ContentFind::Secrets { literals, scan } = &mut c.find {
                    // Finding ids name no value: they stay.
                    literals.retain(|l| *scan && l.finding_id().is_some());
                }
            }
        }
        copy
    }

    /// `true` when a secret literal is part of the recipe.
    pub fn holds_secrets(&self) -> bool {
        self.rules.iter().any(|r| {
            matches!(
                &r.kind,
                RuleKind::ContentReplace(ContentReplace {
                    find: ContentFind::Secrets { literals, scan },
                    ..
                }) if literals.iter().any(|l| !(*scan && l.finding_id().is_some()))
            )
        })
    }
}

/// What [`validate_recipe`] found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeIssue {
    /// The index in `Recipe::rules`, `None` for the recipe as a whole.
    pub rule: Option<usize>,
    pub refusal: RewriteRefusal,
    pub detail: String,
}

impl fmt::Display for RecipeIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.rule {
            Some(i) => write!(f, "rule {}: {}", i + 1, self.detail),
            None => f.write_str(&self.detail),
        }
    }
}

fn issue(rule: Option<usize>, refusal: RewriteRefusal, detail: impl Into<String>) -> RecipeIssue {
    RecipeIssue {
        rule,
        refusal,
        detail: detail.into(),
    }
}

/// Checks a recipe without a repository: the first problem, or `Ok`. Only
/// enabled rules are checked, so a half-edited disabled rule does not block
/// a run. What needs the repository (signed commits, dirty paths, protected
/// refs, shallow clones) is the engine's to refuse.
pub fn validate_recipe(recipe: &Recipe) -> Result<(), RecipeIssue> {
    use RewriteRefusal as R;
    if recipe.version != RECIPE_VERSION {
        return Err(issue(
            None,
            R::InvalidRule,
            format!("recipe version {} is not supported", recipe.version),
        ));
    }
    match &recipe.scope {
        Scope::Selection { commits } if commits.is_empty() => {
            return Err(issue(None, R::EmptyScope, "no commit is selected"));
        }
        Scope::Refs { refs } if refs.iter().all(|r| r.trim().is_empty()) => {
            return Err(issue(None, R::EmptyScope, "no branch or tag is named"));
        }
        Scope::Range { base, tip } if base == tip => {
            return Err(issue(None, R::EmptyScope, "the range is empty"));
        }
        _ => {}
    }
    if recipe.enabled_rules().next().is_none() {
        return Err(issue(None, R::NoRule, "no rule is enabled"));
    }
    for (i, rule) in recipe.enabled_rules() {
        if let Some(detail) = rule_problem(&rule.kind) {
            let refusal = match &rule.kind {
                RuleKind::MovePaths(PathMove::ExtractSubdir { mode: None, .. }) => {
                    R::ExtractModeNeeded
                }
                _ => R::InvalidRule,
            };
            return Err(issue(Some(i), refusal, detail));
        }
    }
    // One extraction per recipe, and when it leaves this repository alone
    // (clone, submodule) it comes before the other path rules, so the
    // directory it names is the one in this repository's own tree.
    let mut moves = recipe.enabled_rules().filter_map(|(i, r)| match &r.kind {
        RuleKind::MovePaths(m) => Some((i, m)),
        _ => None,
    });
    let mut seen_move = false;
    let mut extracted = false;
    for (i, m) in &mut moves {
        if let PathMove::ExtractSubdir { mode, .. } = m {
            if extracted {
                return Err(issue(Some(i), R::InvalidRule, "only one rule can extract"));
            }
            extracted = true;
            if seen_move && matches!(mode, Some(ExtractMode::Clone | ExtractMode::Submodule)) {
                return Err(issue(
                    Some(i),
                    R::InvalidRule,
                    "extracting into a clone or a submodule comes before the other path rules",
                ));
            }
        }
        seen_move = true;
    }
    Ok(())
}

fn globs_problem(globs: &[String]) -> Option<String> {
    globs.iter().find_map(|g| validate_glob(g).err())
}

fn filter_problem(paths: &[String], exclude: &[String]) -> Option<String> {
    globs_problem(paths).or_else(|| globs_problem(exclude))
}

fn pattern_problem(p: &TextPattern) -> Option<String> {
    CompiledPattern::new(p).err()
}

fn trailer_problem(t: &TrailerRule) -> Option<String> {
    let ok_key = !t.key.is_empty() && t.key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    if !ok_key {
        return Some("a trailer key is letters, digits and `-`".to_string());
    }
    match t.action {
        TrailerAction::Add => (t.value.trim().is_empty() || t.value.contains(['\n', '\r']))
            .then(|| "a trailer to add needs a one-line value".to_string()),
        TrailerAction::Remove => t.find.as_ref().and_then(pattern_problem),
        TrailerAction::Map => match &t.find {
            None => Some("a trailer to map needs the text to replace".to_string()),
            Some(p) => pattern_problem(p).or_else(|| {
                t.with
                    .contains(['\n', '\r'])
                    .then(|| "a trailer value is one line".to_string())
            }),
        },
    }
}

fn rule_problem(kind: &RuleKind) -> Option<String> {
    match kind {
        RuleKind::Identity(r) => CompiledMatch::new(&r.when)
            .err()
            .or_else(|| set_problem(&r.set).map(str::to_string)),
        RuleKind::Mailmap { .. }
        | RuleKind::CommitterFromAuthor
        | RuleKind::RemoveSignatures
        | RuleKind::MessageEncoding { .. } => None,
        RuleKind::Anonymise(r) => anonymise_problem(r),
        RuleKind::DateShift { by_seconds, .. } => (*by_seconds == 0)
            .then(|| "a shift of zero changes nothing".to_string())
            .or_else(|| {
                (by_seconds.unsigned_abs() > 200 * 365 * 86_400)
                    .then(|| "the shift is larger than 200 years".to_string())
            }),
        RuleKind::DateTimezone { offset_minutes, .. } => (offset_minutes.abs()
            > MAX_OFFSET_MINUTES)
            .then(|| "the offset is out of range".to_string()),
        RuleKind::DateWindow(w) => window_problem(w),
        RuleKind::MessageReplace(r) => pattern_problem(&r.find),
        RuleKind::MessageDeleteLines { pattern } => pattern_problem(pattern),
        RuleKind::Trailer(t) => trailer_problem(t),
        RuleKind::ContentReplace(c) => globs_problem(&c.paths).or_else(|| match &c.find {
            ContentFind::Pattern { pattern } => pattern_problem(pattern),
            ContentFind::Secrets { literals, scan } => (literals.is_empty() && !*scan)
                .then(|| "give the secret values or turn the scanner on".to_string())
                .or_else(|| {
                    literals
                        .iter()
                        .any(|l| l.0.is_empty())
                        .then(|| "a secret value is empty".to_string())
                }),
        }),
        RuleKind::RemovePaths { globs } => (globs.is_empty())
            .then(|| "name at least one path".to_string())
            .or_else(|| globs_problem(globs)),
        RuleKind::RemoveBlobsOver { bytes } => {
            (*bytes == 0).then(|| "the size limit must be above zero".to_string())
        }
        RuleKind::MovePaths(m) => m.problem(),
        RuleKind::LfsImport(l) => (l.patterns.is_empty())
            .then(|| "name at least one pattern".to_string())
            .or_else(|| globs_problem(&l.patterns)),
        RuleKind::LfsExport(l) => (l.patterns.is_empty())
            .then(|| "name at least one pattern".to_string())
            .or_else(|| globs_problem(&l.patterns)),
        RuleKind::LineEndings(e) => filter_problem(&e.paths, &e.exclude),
        RuleKind::StripExec(e) => (e.paths.is_empty())
            .then(|| "name at least one path".to_string())
            .or_else(|| filter_problem(&e.paths, &e.exclude)),
        RuleKind::Reencode(e) => filter_problem(&e.paths, &e.exclude).or_else(|| {
            (e.from == Some(e.target))
                .then(|| "the source and the target encoding are the same".to_string())
        }),
        RuleKind::Script(s) => match &s.source {
            ScriptSource::Inline { text } if text.trim().is_empty() => {
                Some("the script is empty".to_string())
            }
            ScriptSource::File { path } if path.trim().is_empty() => {
                Some("the script path is empty".to_string())
            }
            _ => None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::rewrite::dates::{WeekWindow, Weekday};
    use crate::domain::rewrite::identity::{IdentityMatch, IdentitySet};
    use crate::domain::rewrite::paths::ExtractMode;

    fn oid(byte: u8) -> Oid {
        Oid([byte; 20])
    }

    fn identity_rule() -> Rule {
        Rule::new(RuleKind::Identity(IdentityRule {
            when: IdentityMatch::email("old@example.com"),
            set: IdentitySet {
                name: Some("Alice Example".into()),
                email: Some("alice@example.com".into()),
            },
            apply_to: Who::Both,
            include_taggers: true,
            map_trailers: false,
        }))
    }

    fn recipe(rules: Vec<Rule>) -> Recipe {
        Recipe::new("test", Scope::default(), rules)
    }

    fn refused(r: &Recipe) -> (Option<usize>, RewriteRefusal) {
        let e = validate_recipe(r).unwrap_err();
        (e.rule, e.refusal)
    }

    #[test]
    fn a_sound_recipe_validates() {
        assert!(validate_recipe(&recipe(vec![identity_rule()])).is_ok());
    }

    #[test]
    fn scope_refusals() {
        let rules = vec![identity_rule()];
        let mut r = recipe(rules.clone());
        r.scope = Scope::Selection { commits: vec![] };
        assert_eq!(refused(&r), (None, RewriteRefusal::EmptyScope));
        r.scope = Scope::Refs {
            refs: vec!["".into()],
        };
        assert_eq!(refused(&r), (None, RewriteRefusal::EmptyScope));
        r.scope = Scope::Range {
            base: oid(1),
            tip: oid(1),
        };
        assert_eq!(refused(&r), (None, RewriteRefusal::EmptyScope));
        r.scope = Scope::Range {
            base: oid(1),
            tip: oid(2),
        };
        assert!(validate_recipe(&r).is_ok());
        r.scope = Scope::AllLocal;
        assert!(validate_recipe(&r).is_ok());
    }

    #[test]
    fn no_rule_and_only_disabled_rules_are_refused() {
        assert_eq!(refused(&recipe(vec![])), (None, RewriteRefusal::NoRule));
        let mut off = identity_rule();
        off.enabled = false;
        assert_eq!(refused(&recipe(vec![off])), (None, RewriteRefusal::NoRule));
    }

    #[test]
    fn a_disabled_broken_rule_does_not_block_and_an_enabled_one_names_its_index() {
        let mut broken = Rule::new(RuleKind::RemovePaths { globs: vec![] });
        broken.enabled = false;
        assert!(validate_recipe(&recipe(vec![identity_rule(), broken.clone()])).is_ok());
        broken.enabled = true;
        assert_eq!(
            refused(&recipe(vec![identity_rule(), broken])),
            (Some(1), RewriteRefusal::InvalidRule)
        );
    }

    #[test]
    fn a_wrong_version_is_refused() {
        let mut r = recipe(vec![identity_rule()]);
        r.version = 99;
        assert_eq!(refused(&r), (None, RewriteRefusal::InvalidRule));
    }

    #[test]
    fn every_rule_kind_has_a_refusal() {
        let bad: Vec<RuleKind> = vec![
            // identity: nothing to set, a bad email, a regex that does not compile
            RuleKind::Identity(IdentityRule {
                when: IdentityMatch::everyone(),
                set: IdentitySet::default(),
                apply_to: Who::Both,
                include_taggers: false,
                map_trailers: false,
            }),
            RuleKind::Identity(IdentityRule {
                when: IdentityMatch::everyone(),
                set: IdentitySet {
                    name: None,
                    email: Some("a<b@example.com".into()),
                },
                apply_to: Who::Both,
                include_taggers: false,
                map_trailers: false,
            }),
            RuleKind::Identity(IdentityRule {
                when: IdentityMatch {
                    name: Some("(".into()),
                    regex: true,
                    ..IdentityMatch::default()
                },
                set: IdentitySet {
                    name: Some("X".into()),
                    email: None,
                },
                apply_to: Who::Both,
                include_taggers: false,
                map_trailers: false,
            }),
            RuleKind::Anonymise(AnonymiseRule {
                email_pattern: "fixed@anonymous.invalid".into(),
                ..AnonymiseRule::default()
            }),
            RuleKind::DateShift {
                by_seconds: 0,
                apply_to: WhichDate::Both,
            },
            RuleKind::DateShift {
                by_seconds: i64::MAX,
                apply_to: WhichDate::Both,
            },
            RuleKind::DateTimezone {
                offset_minutes: 2000,
                keep: KeepClock::Instant,
                apply_to: WhichDate::Both,
            },
            RuleKind::DateWindow(WorkWindowRule {
                windows: vec![WeekWindow::new(&[Weekday::Mon], 600, 600)],
                ..WorkWindowRule::evenings_and_weekends()
            }),
            RuleKind::MessageReplace(TextReplace {
                find: TextPattern::regex("("),
                with: String::new(),
            }),
            RuleKind::MessageReplace(TextReplace {
                find: TextPattern::literal(""),
                with: String::new(),
            }),
            RuleKind::MessageDeleteLines {
                pattern: TextPattern::regex("["),
            },
            RuleKind::Trailer(TrailerRule::add("Co authored", "x")),
            RuleKind::Trailer(TrailerRule::add("Signed-off-by", "")),
            RuleKind::Trailer(TrailerRule::add("Signed-off-by", "two\nlines")),
            // A map without the text to replace, or with a pattern that
            // does not compile, or a multi-line replacement.
            RuleKind::Trailer(TrailerRule {
                find: None,
                ..TrailerRule::map("Signed-off-by", "a", "b")
            }),
            RuleKind::Trailer(TrailerRule {
                find: Some(TextPattern::regex("(")),
                ..TrailerRule::map("Signed-off-by", "a", "b")
            }),
            RuleKind::Trailer(TrailerRule::map("Signed-off-by", "a", "b\nc")),
            RuleKind::Trailer(TrailerRule {
                find: Some(TextPattern::regex("[")),
                ..TrailerRule::remove("Signed-off-by")
            }),
            RuleKind::ContentReplace(ContentReplace {
                paths: vec!["../x".into()],
                find: ContentFind::Pattern {
                    pattern: TextPattern::literal("a"),
                },
                with: String::new(),
                max_blob_bytes: None,
            }),
            RuleKind::ContentReplace(ContentReplace {
                paths: vec![],
                find: ContentFind::Secrets {
                    literals: vec![],
                    scan: false,
                },
                with: String::new(),
                max_blob_bytes: None,
            }),
            RuleKind::RemovePaths { globs: vec![] },
            RuleKind::RemovePaths {
                globs: vec!["[".into()],
            },
            RuleKind::RemoveBlobsOver { bytes: 0 },
            RuleKind::MovePaths(PathMove::IntoSubdir { dir: "".into() }),
            RuleKind::LfsImport(LfsImportRule {
                patterns: vec![],
                min_size: None,
            }),
            RuleKind::LfsExport(LfsExportRule { patterns: vec![] }),
            RuleKind::Script(ScriptRef {
                language: ScriptLanguage::Lua,
                source: ScriptSource::Inline { text: " ".into() },
                params: BTreeMap::new(),
            }),
            RuleKind::Script(ScriptRef {
                language: ScriptLanguage::Shell,
                source: ScriptSource::File { path: "".into() },
                params: BTreeMap::new(),
            }),
        ];
        for (n, kind) in bad.into_iter().enumerate() {
            let name = kind.name();
            let r = recipe(vec![Rule::new(kind)]);
            assert_eq!(
                refused(&r),
                (Some(0), RewriteRefusal::InvalidRule),
                "case {n} ({name})"
            );
        }
    }

    #[test]
    fn extracting_a_subdirectory_asks_for_a_mode() {
        let ask = |mode| {
            recipe(vec![Rule::new(RuleKind::MovePaths(
                PathMove::ExtractSubdir {
                    dir: "app".into(),
                    mode,
                },
            ))])
        };
        assert_eq!(
            refused(&ask(None)),
            (Some(0), RewriteRefusal::ExtractModeNeeded)
        );
        for mode in [
            ExtractMode::Clone,
            ExtractMode::InPlace,
            ExtractMode::Submodule,
        ] {
            assert!(validate_recipe(&ask(Some(mode))).is_ok());
        }
    }

    #[test]
    fn defaults_keep_the_committer_and_ask_about_signatures() {
        let o = RunOptions::default();
        assert_eq!(o.committer, CommitterPolicy::Keep);
        assert_eq!(o.signatures, None);
        assert!(!o.keep_backup_refs);
        assert!(ScopeOptions::default().include_protected.is_empty());
    }

    fn everything() -> Recipe {
        let mut r = Recipe::new(
            "everything",
            Scope::Range {
                base: oid(1),
                tip: oid(2),
            },
            vec![
                identity_rule(),
                Rule {
                    enabled: false,
                    label: Some("anon".into()),
                    kind: RuleKind::Anonymise(AnonymiseRule::default()),
                },
                Rule::new(RuleKind::DateShift {
                    by_seconds: -3600,
                    apply_to: WhichDate::AuthorDate,
                }),
                Rule::new(RuleKind::DateTimezone {
                    offset_minutes: -300,
                    keep: KeepClock::WallClock,
                    apply_to: WhichDate::Both,
                }),
                Rule::new(RuleKind::DateWindow(WorkWindowRule::evenings_and_weekends())),
                Rule::new(RuleKind::CommitterFromAuthor),
                Rule::new(RuleKind::Mailmap {
                    apply_to: Who::Author,
                    file: None,
                }),
                Rule::new(RuleKind::MessageReplace(TextReplace {
                    find: TextPattern::regex("PROJ-(\\d+)"),
                    with: "#$1".into(),
                })),
                Rule::new(RuleKind::MessageDeleteLines {
                    pattern: TextPattern::literal("WIP"),
                }),
                Rule::new(RuleKind::Trailer(TrailerRule::add(
                    "Signed-off-by",
                    "Alice Example <alice@example.com>",
                ))),
                Rule::new(RuleKind::Trailer(TrailerRule::remove("Change-Id"))),
                Rule::new(RuleKind::Trailer(TrailerRule::map(
                    "Co-authored-by",
                    "@old.org",
                    "@new.org",
                ))),
                Rule::new(RuleKind::ContentReplace(ContentReplace {
                    paths: vec!["*.env".into()],
                    find: ContentFind::Pattern {
                        pattern: TextPattern::literal("token"),
                    },
                    with: "x".into(),
                    max_blob_bytes: Some(1 << 20),
                })),
                Rule::new(RuleKind::RemovePaths {
                    globs: vec!["secrets/".into()],
                }),
                Rule::new(RuleKind::RemoveBlobsOver { bytes: 10_000_000 }),
                Rule::new(RuleKind::MovePaths(PathMove::ExtractSubdir {
                    dir: "app".into(),
                    mode: Some(ExtractMode::Submodule),
                })),
                Rule::new(RuleKind::LfsImport(LfsImportRule {
                    patterns: vec!["*.psd".into()],
                    min_size: Some(1 << 20),
                })),
                Rule::new(RuleKind::LfsExport(LfsExportRule {
                    patterns: vec!["*.zip".into()],
                })),
                Rule::new(RuleKind::Script(ScriptRef {
                    language: ScriptLanguage::Lua,
                    source: ScriptSource::Inline {
                        text: "return commit".into(),
                    },
                    params: BTreeMap::from([("who".to_string(), "Alice".to_string())]),
                })),
            ],
        );
        r.scope_options.include_protected = vec!["main".into()];
        r.options.committer = CommitterPolicy::MeNow;
        r.options.signatures = Some(SignaturePolicy::ResignSigned);
        r
    }

    #[test]
    fn a_recipe_round_trips_through_toml() {
        let r = everything();
        let text = toml::to_string(&r).unwrap();
        let back: Recipe = toml::from_str(&text).unwrap();
        assert_eq!(back, r, "{text}");
    }

    #[test]
    fn a_recipe_round_trips_through_json() {
        let r = everything();
        let text = serde_json::to_string(&r).unwrap();
        let back: Recipe = serde_json::from_str(&text).unwrap();
        assert_eq!(back, r);
    }

    #[test]
    fn a_minimal_file_fills_in_defaults() {
        let r: Recipe = toml::from_str(
            r#"
name = "tidy"

[[rules]]
kind = { type = "message_delete_lines", pattern = { pattern = "WIP" } }
"#,
        )
        .unwrap();
        assert_eq!(r.version, RECIPE_VERSION);
        assert_eq!(r.scope, Scope::default());
        assert!(r.rules[0].enabled);
        assert_eq!(r.options.signatures, None);
        assert!(validate_recipe(&r).is_ok());
    }

    #[test]
    fn an_undecided_signature_choice_is_not_written() {
        let r = recipe(vec![identity_rule()]);
        let text = toml::to_string(&r).unwrap();
        assert!(!text.contains("signatures"), "{text}");
    }

    fn secret_recipe() -> Recipe {
        recipe(vec![Rule::new(RuleKind::ContentReplace(ContentReplace {
            paths: vec![],
            find: ContentFind::Secrets {
                literals: vec![SecretText("hunter2-not-a-real-secret".into())],
                scan: true,
            },
            with: "***REMOVED***".into(),
            max_blob_bytes: None,
        }))])
    }

    #[test]
    fn saved_recipes_carry_no_secret_literal() {
        let r = secret_recipe();
        assert!(r.holds_secrets());
        let saved = r.saved();
        assert!(!saved.holds_secrets());
        let text = toml::to_string(&saved).unwrap();
        assert!(!text.contains("hunter2"), "{text}");
        assert!(!serde_json::to_string(&saved).unwrap().contains("hunter2"));
        // Loaded back, the rule keeps its scanner but needs no literal to be valid.
        let back: Recipe = toml::from_str(&text).unwrap();
        assert!(validate_recipe(&back).is_ok());
    }

    #[test]
    fn finding_ids_are_kept_when_saving_but_values_next_to_them_are_not() {
        let mut r = secret_recipe();
        if let RuleKind::ContentReplace(c) = &mut r.rules[0].kind {
            if let ContentFind::Secrets { literals, .. } = &mut c.find {
                literals.push(SecretText("f-0123456789ab".into()));
            }
        }
        // A value and an id: the recipe holds a secret until saved.
        assert!(r.holds_secrets());
        let saved = r.saved();
        assert!(!saved.holds_secrets());
        let text = serde_json::to_string(&saved).unwrap();
        assert!(
            text.contains("f-0123456789ab") && !text.contains("hunter2"),
            "{text}"
        );
        // Only ids: nothing secret to hold.
        assert!(!saved.holds_secrets());
        // Without the scanner the id-shaped text is a literal like any other.
        let mut plain = saved.clone();
        if let RuleKind::ContentReplace(c) = &mut plain.rules[0].kind {
            c.find = ContentFind::Secrets {
                literals: vec![SecretText("f-0123456789ab".into())],
                scan: false,
            };
        }
        assert!(plain.holds_secrets());
        assert!(!plain.saved().holds_secrets());
    }

    #[test]
    fn a_secret_never_shows_up_in_debug_output() {
        let r = secret_recipe();
        assert!(!format!("{r:?}").contains("hunter2"));
    }

    #[test]
    fn refusal_codes_are_stable_and_distinct() {
        use std::collections::HashSet;
        let all = [
            RewriteRefusal::OperationInProgress,
            RewriteRefusal::AutoStashPending,
            RewriteRefusal::Shallow,
            RewriteRefusal::PartialClone,
            RewriteRefusal::UnsupportedObjectFormat,
            RewriteRefusal::EmptyScope,
            RewriteRefusal::NoRule,
            RewriteRefusal::InvalidRule,
            RewriteRefusal::DirtyPathsTouched,
            RewriteRefusal::CheckedOutElsewhere,
            RewriteRefusal::ProtectedRef,
            RewriteRefusal::Stale,
            RewriteRefusal::ToolMissing,
            RewriteRefusal::ScriptFailed,
            RewriteRefusal::Purged,
            RewriteRefusal::SignatureChoiceNeeded,
            RewriteRefusal::ExtractModeNeeded,
        ];
        let codes: HashSet<_> = all.iter().map(|r| r.code()).collect();
        assert_eq!(codes.len(), all.len());
        for r in all {
            assert_eq!(
                serde_json::to_string(&r).unwrap(),
                format!("\"{}\"", r.code())
            );
        }
    }
}
