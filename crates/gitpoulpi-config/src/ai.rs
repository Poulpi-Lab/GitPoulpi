//! The `[ai]` config: connection profiles, custom instructions and the
//! per-action settings.
//!
//! GitPoulpi never requires an account and AI is **off by default**
//! (`ai.enabled = false`); every AI entry point in the frontends is hidden
//! while it is off. An API key is never stored here: a profile carries a
//! `token_ref`, a key into `gitpoulpi-secrets`, exactly like an
//! `[[integrations]]` entry.
//!
//! ```toml
//! [ai]
//! enabled = true
//! default_profile = "default"
//! global_instructions = "Write in English. Be terse."
//!
//! [[ai.profiles]]
//! id = "default"
//! name = "Default"
//! kind = "open_ai"
//! model = "gpt-5"
//! token_ref = "ai:default"
//!
//! [ai.actions.commit_message]
//! profile = ""                       # empty = the default profile
//! instructions = "Use conventional commits."
//! ```
//!
//! The prompt an action sends is the built-in base prompt (kept in
//! `gitpoulpi-ai`, not user-visible), then `global_instructions`, then the
//! action's `instructions`; empty parts are skipped. A repository can
//! override, per action, the profile and the instructions
//! (`gitpoulpi-session`'s `RepoPreferences::ai_overrides`); an empty field
//! means "use the global setting".
//!
//! # Migration
//!
//! Files written before 2026-10-01 had `[[ai.presets]]` (named system
//! prompts) and `[ai.defaults]` (a preset id per action). On load each
//! action's default preset becomes that action's `{ profile, instructions }`
//! and every old preset is kept verbatim in `[[ai.legacy_presets]]` so
//! nothing is lost (including the ones that were nobody's default, and the
//! ones per-repo choices still refer to: see [`AiConfig::legacy_preset`]).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The things AI can do for the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiAction {
    CommitMessage,
    ExplainChanges,
    StashMessage,
    PullRequestDescription,
    CommitComposer,
    ConflictResolve,
    CodeReview,
    /// Proposes how to split one commit into several (the split editor's ✦).
    SplitCommit,
    /// Writes a history rewrite script (the Script rule's ✦).
    RewriteScript,
    /// Prepares the rules and settings of a history rewrite from a
    /// description (the workbench's "Create Rules with AI"). It only fills
    /// the workbench: it never previews or applies anything.
    RewriteRules,
    /// Names a new worktree and its branch from a description of the task
    /// (the Create worktree dialog's ✦). It only fills two fields.
    WorktreeName,
}

impl AiAction {
    pub const ALL: [AiAction; 11] = [
        Self::CommitMessage,
        Self::ExplainChanges,
        Self::StashMessage,
        Self::PullRequestDescription,
        Self::CommitComposer,
        Self::ConflictResolve,
        Self::CodeReview,
        Self::SplitCommit,
        Self::RewriteScript,
        Self::RewriteRules,
        Self::WorktreeName,
    ];

    /// The snake_case key used in `config.toml` and on the CLI.
    pub fn key(self) -> &'static str {
        match self {
            Self::CommitMessage => "commit_message",
            Self::ExplainChanges => "explain_changes",
            Self::StashMessage => "stash_message",
            Self::PullRequestDescription => "pull_request_description",
            Self::CommitComposer => "commit_composer",
            Self::ConflictResolve => "conflict_resolve",
            Self::CodeReview => "code_review",
            Self::SplitCommit => "split_commit",
            Self::RewriteScript => "rewrite_script",
            Self::RewriteRules => "rewrite_rules",
            Self::WorktreeName => "worktree_name",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|a| a.key() == key)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::CommitMessage => "Commit message",
            Self::ExplainChanges => "Explain changes",
            Self::StashMessage => "Stash message",
            Self::PullRequestDescription => "Pull request description",
            Self::CommitComposer => "Commit composer",
            Self::ConflictResolve => "Conflict resolution",
            Self::CodeReview => "Code review",
            Self::SplitCommit => "Split commit",
            Self::RewriteScript => "History rewrite script",
            Self::RewriteRules => "History rewrite rules",
            Self::WorktreeName => "Worktree name",
        }
    }
}

/// How a profile reaches a model.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiProviderKind {
    #[default]
    OpenAi,
    Anthropic,
    OpenRouter,
    /// Any server speaking OpenAI's chat-completions dialect at `base_url`
    /// (Ollama, LM Studio, vLLM, a corporate gateway, ...).
    OpenAiCompatible,
    /// A local agent CLI (`claude -p`, ...): the prompt goes on stdin, the
    /// answer comes back on stdout. No key, no network from GitPoulpi.
    LocalAgent,
}

impl AiProviderKind {
    pub const ALL: [AiProviderKind; 5] = [
        Self::OpenAi,
        Self::Anthropic,
        Self::OpenRouter,
        Self::OpenAiCompatible,
        Self::LocalAgent,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::OpenAi => "OpenAI",
            Self::Anthropic => "Anthropic",
            Self::OpenRouter => "OpenRouter",
            Self::OpenAiCompatible => "OpenAI-compatible",
            Self::LocalAgent => "Local agent",
        }
    }

    /// Whether the kind needs an API key.
    pub fn needs_key(self) -> bool {
        !matches!(self, Self::LocalAgent)
    }

    /// The base URL used when a profile leaves `base_url` empty.
    pub fn default_base_url(self) -> &'static str {
        match self {
            Self::OpenAi => "https://api.openai.com/v1",
            Self::Anthropic => "https://api.anthropic.com/v1",
            Self::OpenRouter => "https://openrouter.ai/api/v1",
            Self::OpenAiCompatible => "http://localhost:11434/v1",
            Self::LocalAgent => "",
        }
    }

    pub fn default_model(self) -> &'static str {
        match self {
            Self::OpenAi => "gpt-5",
            Self::Anthropic => "claude-sonnet-5",
            Self::OpenRouter => "openai/gpt-5",
            Self::OpenAiCompatible => "llama3",
            Self::LocalAgent => "",
        }
    }
}

/// How a local agent profile runs the agent.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRunMode {
    /// The agent's non-interactive mode (`claude -p`): prompt on stdin,
    /// answer on stdout.
    #[default]
    Headless,
    /// The agent's interactive screen, driven in a virtual terminal: the
    /// prompt is typed in, the answer is read back.
    Terminal,
}

impl AgentRunMode {
    pub const ALL: [AgentRunMode; 2] = [Self::Headless, Self::Terminal];
}

/// Where a local agent runs (its working folder).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentWorkdir {
    /// Headless: wherever GitPoulpi runs (as before this key existed).
    /// Terminal: the dedicated folder.
    #[default]
    Default,
    /// An empty folder GitPoulpi owns, so the agent sees no project
    /// context and asks for trust once, not once per repository.
    Dedicated,
    /// The repository the action is run in: the agent sees the project's
    /// own context, and asks for trust once per repository.
    Repository,
}

impl AgentWorkdir {
    pub const ALL: [AgentWorkdir; 3] = [Self::Default, Self::Dedicated, Self::Repository];
}

/// The seconds a headless agent may take before it is given up on.
pub const DEFAULT_HEADLESS_TIMEOUT_SECS: u64 = 120;
/// The seconds an agent driven in a virtual terminal may take.
pub const DEFAULT_TERMINAL_TIMEOUT_SECS: u64 = 180;

fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    *value == T::default()
}

/// A connection to a model.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AiProfile {
    pub id: String,
    pub name: String,
    pub kind: AiProviderKind,
    /// The model name; empty means the kind's default. Unused by
    /// `local_agent`.
    pub model: String,
    /// API base URL; empty means the kind's default. Unused by
    /// `local_agent`.
    pub base_url: String,
    /// A key into `gitpoulpi-secrets` holding the API key. Never the key.
    pub token_ref: String,
    /// `local_agent` only: the executable (`claude`).
    pub command: String,
    /// `local_agent` only: its arguments (`["-p"]`). The prompt is passed
    /// on stdin.
    pub args: Vec<String>,
    /// `local_agent` only: which known agent CLI this profile is
    /// (`claude`, `codex`, ...), independent of `command`: the command
    /// may be an alias, a wrapper script or an absolute path. Empty means
    /// "infer it from `command`" (profiles written before this key).
    #[serde(skip_serializing_if = "String::is_empty")]
    pub harness: String,
    /// `local_agent` only: environment variables set for that
    /// subprocess, and only for it. A value may use `${env:NAME}` to
    /// borrow one from GitPoulpi's own environment. Never logged.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub env: BTreeMap<String, String>,
    /// `local_agent` only: headless (`args`) or the interactive screen in a
    /// virtual terminal (`terminal_args`).
    #[serde(skip_serializing_if = "is_default")]
    pub run_mode: AgentRunMode,
    /// `local_agent` only: the arguments of the terminal mode, kept apart
    /// from `args` so switching modes loses nothing. Empty means the
    /// harness's terminal defaults.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub terminal_args: Vec<String>,
    /// `local_agent` only: where the agent runs, both modes.
    #[serde(skip_serializing_if = "is_default")]
    pub workdir: AgentWorkdir,
    /// `local_agent` only: seconds before the agent is given up on; `0`
    /// means the mode's default (120 headless, 180 terminal).
    #[serde(skip_serializing_if = "is_default")]
    pub timeout_secs: u64,
}

impl AiProfile {
    pub fn token_ref_for(id: &str) -> String {
        format!("ai:{id}")
    }

    pub fn new(id: impl Into<String>, name: impl Into<String>, kind: AiProviderKind) -> Self {
        let id = id.into();
        let mut profile = Self {
            token_ref: if kind.needs_key() {
                Self::token_ref_for(&id)
            } else {
                String::new()
            },
            id,
            name: name.into(),
            kind,
            ..Self::default()
        };
        profile.model = kind.default_model().to_string();
        if kind == AiProviderKind::LocalAgent {
            profile.command = "claude".into();
            profile.args = vec!["-p".into()];
            profile.harness = "claude".into();
        }
        profile
    }

    /// The arguments of the active run mode.
    pub fn active_args(&self) -> &Vec<String> {
        match self.run_mode {
            AgentRunMode::Headless => &self.args,
            AgentRunMode::Terminal => &self.terminal_args,
        }
    }

    pub fn active_args_mut(&mut self) -> &mut Vec<String> {
        match self.run_mode {
            AgentRunMode::Headless => &mut self.args,
            AgentRunMode::Terminal => &mut self.terminal_args,
        }
    }

    /// The seconds the agent may take in the active mode.
    pub fn effective_timeout_secs(&self) -> u64 {
        if self.timeout_secs > 0 {
            self.timeout_secs
        } else {
            match self.run_mode {
                AgentRunMode::Headless => DEFAULT_HEADLESS_TIMEOUT_SECS,
                AgentRunMode::Terminal => DEFAULT_TERMINAL_TIMEOUT_SECS,
            }
        }
    }

    pub fn display_name(&self) -> &str {
        if self.name.trim().is_empty() {
            &self.id
        } else {
            &self.name
        }
    }

    pub fn effective_model(&self) -> &str {
        if self.model.trim().is_empty() {
            self.kind.default_model()
        } else {
            self.model.trim()
        }
    }

    pub fn effective_base_url(&self) -> &str {
        if self.base_url.trim().is_empty() {
            self.kind.default_base_url()
        } else {
            self.base_url.trim().trim_end_matches('/')
        }
    }
}

/// A system prompt bound to an action and a profile, as the pre-2026-10-01
/// config stored it. Only kept as the archive `[[ai.legacy_presets]]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AiPreset {
    pub id: String,
    pub name: String,
    pub action: AiAction,
    pub profile: String,
    pub system_prompt: String,
}

impl Default for AiPreset {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            action: AiAction::CommitMessage,
            profile: String::new(),
            system_prompt: String::new(),
        }
    }
}

/// How a repository's own instructions combine with the global ones.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepoInstructionsMode {
    /// Appended after the global instructions.
    #[default]
    Add,
    /// Used instead of the global instructions.
    Replace,
}

impl RepoInstructionsMode {
    pub const ALL: [Self; 2] = [Self::Add, Self::Replace];

    pub fn label(self) -> &'static str {
        match self {
            Self::Add => "Add to global",
            Self::Replace => "Replace global",
        }
    }
}

/// Whether the local agent of a review runs in the repository (its
/// working folder), overriding the profile's `workdir` for that run.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunInRepo {
    /// Whatever the AI profile's `workdir` says.
    #[default]
    Profile,
    /// In the repository, whatever the profile says.
    Yes,
    /// Not in the repository: a profile set to `repository` runs in the
    /// dedicated folder for this run.
    No,
}

impl RunInRepo {
    pub const ALL: [RunInRepo; 3] = [Self::Profile, Self::Yes, Self::No];

    pub fn key(self) -> &'static str {
        match self {
            Self::Profile => "profile",
            Self::Yes => "yes",
            Self::No => "no",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|v| v.key() == key.trim())
    }

    /// The working folder mode a run gets, given the profile's own.
    pub fn apply(self, profile: AgentWorkdir) -> AgentWorkdir {
        match (self, profile) {
            (Self::Yes, _) => AgentWorkdir::Repository,
            (Self::No, AgentWorkdir::Repository) => AgentWorkdir::Dedicated,
            (_, other) => other,
        }
    }
}

/// `[ai.privacy]`: what is taken out of a request before it is sent to a
/// model. Both default on; turning one off is a choice the Settings page
/// flags in red.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AiPrivacy {
    /// Replace every recognised secret in the prompt with `[REDACTED:<rule>]`.
    pub redact_secrets: bool,
    /// Leave out the diff of every file matching `[privacy] sensitive_patterns`.
    pub exclude_sensitive_files: bool,
}

impl Default for AiPrivacy {
    fn default() -> Self {
        Self {
            redact_secrets: true,
            exclude_sensitive_files: true,
        }
    }
}

/// Default for `ai.limits.conflict_bytes`: the most the three blocks (base,
/// ours, theirs) of one merge conflict may add up to before a request is
/// held back behind a warning.
pub const DEFAULT_CONFLICT_BYTES: usize = 40_000;
/// Smallest `ai.limits.conflict_bytes` takes (a hand-edited `0` or a tiny
/// number would hold back every conflict).
pub const MIN_CONFLICT_BYTES: usize = 1_000;
/// Largest `ai.limits.conflict_bytes` takes.
pub const MAX_CONFLICT_BYTES: usize = 50_000_000;

/// Default for `ai.limits.diff_bytes`: the diff part of a commit message,
/// explain, stash, pull request, composer or split prompt, and the total
/// diff of a code review.
pub const DEFAULT_DIFF_BYTES: usize = 60_000;
/// Default for `ai.limits.review_file_bytes`: one file's diff in a review.
pub const DEFAULT_REVIEW_FILE_BYTES: usize = 12_000;
/// Default for `ai.limits.review_text_bytes`: a review's description.
pub const DEFAULT_REVIEW_TEXT_BYTES: usize = 4_000;
/// Default for `ai.limits.review_threads_bytes`: the review comments that
/// already exist, listed so the model does not repeat them.
pub const DEFAULT_REVIEW_THREADS_BYTES: usize = 6_000;
/// Default for `ai.limits.review_context_file_bytes`: one project context
/// file (`CLAUDE.md`, `AGENTS.md`, the extra file) of a review.
pub const DEFAULT_REVIEW_CONTEXT_FILE_BYTES: usize = 8 * 1024;
/// Default for `ai.limits.review_context_bytes`: all the context files of a
/// review together.
pub const DEFAULT_REVIEW_CONTEXT_BYTES: usize = 16 * 1024;
/// Smallest `ai.limits.*` (other than the conflict's) takes.
pub const MIN_LIMIT_BYTES: usize = 1_000;
/// Largest `ai.limits.*` (other than the conflict's) takes.
pub const MAX_LIMIT_BYTES: usize = 50_000_000;

/// `[ai.limits]`: how much input a request may carry before GitPoulpi warns
/// instead of sending. The limit is GitPoulpi's own guard (a model's context
/// window differs per model), so it is a setting: over it, the user is asked
/// whether to send anyway (a conflict) or to send everything / send it
/// truncated (every other input).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AiLimits {
    /// Bytes of one merge conflict (base + ours + theirs) a "Suggest a
    /// resolution" request may carry.
    pub conflict_bytes: usize,
    /// Bytes of the diff a commit message, explanation, stash name, pull
    /// request description, commit plan or split plan may carry; also the
    /// total diff of a code review.
    pub diff_bytes: usize,
    /// Bytes of one file's diff in a code review.
    pub review_file_bytes: usize,
    /// Bytes of a review's description.
    pub review_text_bytes: usize,
    /// Bytes of the existing review comments given to a code review.
    pub review_threads_bytes: usize,
    /// Bytes of one project context file given to a code review.
    pub review_context_file_bytes: usize,
    /// Bytes of all the project context files given to a code review.
    pub review_context_bytes: usize,
}

impl Default for AiLimits {
    fn default() -> Self {
        Self {
            conflict_bytes: DEFAULT_CONFLICT_BYTES,
            diff_bytes: DEFAULT_DIFF_BYTES,
            review_file_bytes: DEFAULT_REVIEW_FILE_BYTES,
            review_text_bytes: DEFAULT_REVIEW_TEXT_BYTES,
            review_threads_bytes: DEFAULT_REVIEW_THREADS_BYTES,
            review_context_file_bytes: DEFAULT_REVIEW_CONTEXT_FILE_BYTES,
            review_context_bytes: DEFAULT_REVIEW_CONTEXT_BYTES,
        }
    }
}

fn clamp_limit(value: usize) -> usize {
    value.clamp(MIN_LIMIT_BYTES, MAX_LIMIT_BYTES)
}

impl AiLimits {
    /// The settings keys, in the order Settings shows them.
    pub const KEYS: [&'static str; 7] = [
        "ai.limits.conflict_bytes",
        "ai.limits.diff_bytes",
        "ai.limits.review_file_bytes",
        "ai.limits.review_text_bytes",
        "ai.limits.review_threads_bytes",
        "ai.limits.review_context_file_bytes",
        "ai.limits.review_context_bytes",
    ];

    /// `conflict_bytes` with a hand-edited out-of-range value brought back
    /// into `MIN_CONFLICT_BYTES..=MAX_CONFLICT_BYTES`.
    pub fn conflict_bytes_clamped(&self) -> usize {
        self.conflict_bytes
            .clamp(MIN_CONFLICT_BYTES, MAX_CONFLICT_BYTES)
    }

    /// `diff_bytes`, brought back into `MIN_LIMIT_BYTES..=MAX_LIMIT_BYTES`.
    pub fn diff_bytes_clamped(&self) -> usize {
        clamp_limit(self.diff_bytes)
    }

    /// `review_file_bytes`, clamped like [`Self::diff_bytes_clamped`].
    pub fn review_file_bytes_clamped(&self) -> usize {
        clamp_limit(self.review_file_bytes)
    }

    /// `review_text_bytes`, clamped like [`Self::diff_bytes_clamped`].
    pub fn review_text_bytes_clamped(&self) -> usize {
        clamp_limit(self.review_text_bytes)
    }

    /// `review_threads_bytes`, clamped like [`Self::diff_bytes_clamped`].
    pub fn review_threads_bytes_clamped(&self) -> usize {
        clamp_limit(self.review_threads_bytes)
    }

    /// `review_context_file_bytes`, clamped like [`Self::diff_bytes_clamped`].
    pub fn review_context_file_bytes_clamped(&self) -> usize {
        clamp_limit(self.review_context_file_bytes)
    }

    /// `review_context_bytes`, clamped like [`Self::diff_bytes_clamped`].
    pub fn review_context_bytes_clamped(&self) -> usize {
        clamp_limit(self.review_context_bytes)
    }
}

/// `[ai.sent_preview]`: one switch per action key. `true` (the default)
/// shows what is sent before each request; the dialog's "Don't show again
/// for <action>" turns one off. A key the file does not name is `true`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "BTreeMap<String, bool>", into = "BTreeMap<String, bool>")]
pub struct AiSentPreview(BTreeMap<String, bool>);

impl Default for AiSentPreview {
    fn default() -> Self {
        Self::from(BTreeMap::new())
    }
}

impl From<BTreeMap<String, bool>> for AiSentPreview {
    fn from(mut map: BTreeMap<String, bool>) -> Self {
        for action in AiAction::ALL {
            map.entry(action.key().to_string()).or_insert(true);
        }
        Self(map)
    }
}

impl From<AiSentPreview> for BTreeMap<String, bool> {
    fn from(p: AiSentPreview) -> Self {
        p.0
    }
}

impl AiSentPreview {
    /// Whether the request of `action` is shown first.
    pub fn shows(&self, action: AiAction) -> bool {
        self.0.get(action.key()).copied().unwrap_or(true)
    }

    pub fn set(&mut self, action: AiAction, show: bool) {
        self.0.insert(action.key().to_string(), show);
    }

    /// Back to showing every action's request.
    pub fn show_all(&mut self) {
        *self = Self::default();
    }

    /// Whether every action is shown (the default).
    pub fn all_shown(&self) -> bool {
        AiAction::ALL.into_iter().all(|a| self.shows(a))
    }
}

/// `[ai.review]`: the project context a code review draft is given next to
/// the diff, and where the agent runs. A repository overrides any of these
/// ([`RepoAiReview`]); a single run overrides both.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AiReviewContext {
    pub run_in_repository: RunInRepo,
    /// Give the model the repository's `CLAUDE.md` (read at the branch the
    /// request targets).
    pub include_claude_md: bool,
    /// Same for `AGENTS.md`.
    pub include_agents_md: bool,
    /// One more file: a path (relative to the repository root, or absolute)
    /// or a link. Empty: none.
    pub extra_file: String,
}

impl Default for AiReviewContext {
    fn default() -> Self {
        Self {
            run_in_repository: RunInRepo::Profile,
            include_claude_md: true,
            include_agents_md: true,
            extra_file: String::new(),
        }
    }
}

impl AiReviewContext {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

/// One repository's (or one project's, for a repository that is not cloned)
/// own `[ai.review]` values. Every field is optional: `None` inherits the
/// global one. A run's own choices have the same shape.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RepoAiReview {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_in_repository: Option<RunInRepo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_claude_md: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_agents_md: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_file: Option<String>,
    /// `Some(true)`: no extra file at this level, whatever a lower level
    /// (the global `extra_file`) says. `None` (and `Some(false)`) inherit:
    /// an empty `extra_file` is not the same thing, it also inherits. A file
    /// without the key is read as `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_extra_file: Option<bool>,
}

impl RepoAiReview {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// What this level says about the extra file: `None` inherits, `Some("")`
    /// means none (the "No extra file" switch, or the older explicit empty
    /// value), anything else is the file. The switch wins over a file.
    pub fn extra_choice(&self) -> Option<String> {
        if self.no_extra_file == Some(true) {
            return Some(String::new());
        }
        self.extra_file.clone()
    }
}

/// A repository's instructions for every AI action, and how they combine
/// with the global ones. Empty text changes nothing in either mode.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RepoAiInstructions {
    pub text: String,
    pub mode: RepoInstructionsMode,
}

impl RepoAiInstructions {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

/// A workspace's instructions: for every action (`general`) and per action
/// (`actions`, keyed by [`AiAction::key`]), each with the same add/replace
/// mode as a repository's. They sit between the global and the repository
/// levels and apply to every repository of the workspace. Stored on the
/// `[[workspace]]` entry (`gitpoulpi-session`'s `Workspace`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkspaceAi {
    pub general: RepoAiInstructions,
    pub actions: BTreeMap<String, RepoAiInstructions>,
}

impl WorkspaceAi {
    /// The workspace's instructions for one action (empty when it has none).
    pub fn for_action(&self, action: AiAction) -> RepoAiInstructions {
        self.actions.get(action.key()).cloned().unwrap_or_default()
    }

    pub fn is_empty(&self) -> bool {
        self.general.text.trim().is_empty()
            && self.actions.values().all(|a| a.text.trim().is_empty())
    }
}

/// What one action is told: the profile it runs on and its instructions.
/// Both empty means "nothing special" (globally) or "inherit" (in a
/// repository override).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AiActionSettings {
    /// A profile id; empty means `ai.default_profile`.
    pub profile: String,
    /// Custom instructions for this action, added after the global ones.
    pub instructions: String,
}

impl AiActionSettings {
    pub fn is_empty(&self) -> bool {
        self.profile.is_empty() && self.instructions.is_empty()
    }

    /// What a pre-2026-10-01 preset becomes.
    pub fn from_preset(preset: &AiPreset) -> Self {
        Self {
            profile: preset.profile.clone(),
            instructions: preset.system_prompt.clone(),
        }
    }
}

/// The per-action settings, one entry per [`AiAction`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AiActions {
    pub commit_message: AiActionSettings,
    pub explain_changes: AiActionSettings,
    pub stash_message: AiActionSettings,
    pub pull_request_description: AiActionSettings,
    pub commit_composer: AiActionSettings,
    pub conflict_resolve: AiActionSettings,
    pub code_review: AiActionSettings,
    pub split_commit: AiActionSettings,
    pub rewrite_script: AiActionSettings,
    pub rewrite_rules: AiActionSettings,
    pub worktree_name: AiActionSettings,
}

impl AiActions {
    pub fn get(&self, action: AiAction) -> &AiActionSettings {
        match action {
            AiAction::CommitMessage => &self.commit_message,
            AiAction::ExplainChanges => &self.explain_changes,
            AiAction::StashMessage => &self.stash_message,
            AiAction::PullRequestDescription => &self.pull_request_description,
            AiAction::CommitComposer => &self.commit_composer,
            AiAction::ConflictResolve => &self.conflict_resolve,
            AiAction::CodeReview => &self.code_review,
            AiAction::SplitCommit => &self.split_commit,
            AiAction::RewriteScript => &self.rewrite_script,
            AiAction::RewriteRules => &self.rewrite_rules,
            AiAction::WorktreeName => &self.worktree_name,
        }
    }

    pub fn get_mut(&mut self, action: AiAction) -> &mut AiActionSettings {
        match action {
            AiAction::CommitMessage => &mut self.commit_message,
            AiAction::ExplainChanges => &mut self.explain_changes,
            AiAction::StashMessage => &mut self.stash_message,
            AiAction::PullRequestDescription => &mut self.pull_request_description,
            AiAction::CommitComposer => &mut self.commit_composer,
            AiAction::ConflictResolve => &mut self.conflict_resolve,
            AiAction::CodeReview => &mut self.code_review,
            AiAction::SplitCommit => &mut self.split_commit,
            AiAction::RewriteScript => &mut self.rewrite_script,
            AiAction::RewriteRules => &mut self.rewrite_rules,
            AiAction::WorktreeName => &mut self.worktree_name,
        }
    }
}

/// Why an action could not be resolved to a runnable profile.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AiResolveError {
    #[error("AI is turned off (Settings > AI)")]
    Disabled,
    #[error("no AI profile is configured (Settings > AI > Profiles)")]
    NoProfile,
    #[error("the AI profile {0:?} no longer exists")]
    MissingProfile(String),
}

/// Everything the config says about running one action: the connection
/// and the two user-written parts of the prompt (either may be empty).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiResolved {
    pub profile: AiProfile,
    pub global_instructions: String,
    pub action_instructions: String,
    /// `action_instructions` is the repository's own text (it replaced the
    /// global action instructions), so a workspace level cannot sit between
    /// them: a repository override is the most specific word.
    pub action_from_repo: bool,
}

/// The id of the profile that always exists (its name is "Default").
pub const DEFAULT_PROFILE_ID: &str = "default";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AiConfig {
    /// The general switch. Off: every AI entry point is hidden.
    pub enabled: bool,
    /// The profile actions fall back to when they name none. Empty means
    /// the first profile.
    pub default_profile: String,
    /// Custom instructions applied to every AI action.
    pub global_instructions: String,
    pub profiles: Vec<AiProfile>,
    pub actions: AiActions,
    /// `[ai.review]`: what a code review draft is given besides the diff.
    pub review: AiReviewContext,
    /// `[ai.privacy]`: what is stripped from a request before it leaves.
    pub privacy: AiPrivacy,
    /// `[ai.limits]`: how much input a request may carry before a warning.
    pub limits: AiLimits,
    /// `[ai.sent_preview]`: per action, whether "What is sent" is shown
    /// before the request goes out.
    pub sent_preview: AiSentPreview,
    /// Archive of the pre-2026-10-01 `[[ai.presets]]`, written by the
    /// migration and never read by an action. See the module docs.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub legacy_presets: Vec<AiPreset>,
}

impl Default for AiConfig {
    fn default() -> Self {
        let mut ai = Self {
            enabled: false,
            default_profile: String::new(),
            global_instructions: String::new(),
            profiles: Vec::new(),
            actions: AiActions::default(),
            review: AiReviewContext::default(),
            privacy: AiPrivacy::default(),
            limits: AiLimits::default(),
            sent_preview: AiSentPreview::default(),
            legacy_presets: Vec::new(),
        };
        ai.ensure_default_profile();
        ai
    }
}

impl AiConfig {
    /// The profile named "Default" is created when there is no profile at
    /// all, and becomes `default_profile`; a `default_profile` naming a
    /// profile that does not exist is cleared (the first profile is used).
    pub fn ensure_default_profile(&mut self) {
        if self.profiles.is_empty() {
            self.profiles.push(AiProfile::new(
                DEFAULT_PROFILE_ID,
                "Default",
                AiProviderKind::OpenAi,
            ));
            if self.default_profile.is_empty() {
                self.default_profile = DEFAULT_PROFILE_ID.into();
            }
        }
        if !self.default_profile.is_empty() && self.profile(&self.default_profile).is_none() {
            self.default_profile.clear();
        }
    }

    /// Files written before the `[ai]` section did anything carried
    /// `provider`/`model` and `enabled = true` by default; that `true`
    /// was never a choice, so it must not turn AI on now.
    pub(crate) fn migrate_legacy(&mut self, raw: Option<&toml::Value>) {
        let Some(table) = raw.and_then(|v| v.as_table()) else {
            return;
        };
        let legacy = table.contains_key("provider") || table.contains_key("model");
        if legacy && !table.contains_key("profiles") && !table.contains_key("presets") {
            self.enabled = false;
        }
    }

    /// The preset-era shape to the current one, on the raw document:
    /// `ai.presets` + `ai.defaults` become `ai.actions.<action>` (the
    /// default preset's profile and prompt) and `ai.legacy_presets` (every
    /// preset, verbatim). Idempotent, and never overwrites an `actions`
    /// entry that already exists. Run on every load and by the in-file
    /// editors, so an edit can never resurrect the old keys.
    pub(crate) fn migrate_document(doc: &mut toml::Value) {
        use toml::Value;
        let Some(ai) = doc.get_mut("ai").and_then(Value::as_table_mut) else {
            return;
        };
        let presets = ai.remove("presets");
        let defaults = ai.remove("defaults");
        if presets.is_none() && defaults.is_none() {
            return;
        }
        let presets: Vec<Value> = match presets {
            Some(Value::Array(a)) => a,
            _ => Vec::new(),
        };
        let mut actions = match ai.remove("actions") {
            Some(Value::Table(t)) => t,
            _ => toml::map::Map::new(),
        };
        for action in AiAction::ALL {
            if actions.contains_key(action.key()) {
                continue;
            }
            let Some(id) = defaults
                .as_ref()
                .and_then(|d| d.get(action.key()))
                .and_then(Value::as_str)
            else {
                continue;
            };
            let found = presets.iter().find(|p| {
                p.get("id").and_then(Value::as_str) == Some(id)
                    && p.get("action").and_then(Value::as_str) == Some(action.key())
            });
            if let Some(preset) = found {
                let mut entry = toml::map::Map::new();
                for (from, to) in [("profile", "profile"), ("system_prompt", "instructions")] {
                    if let Some(v) = preset.get(from) {
                        entry.insert(to.into(), v.clone());
                    }
                }
                actions.insert(action.key().into(), Value::Table(entry));
            }
        }
        if !actions.is_empty() {
            ai.insert("actions".into(), Value::Table(actions));
        }
        if !presets.is_empty() {
            let mut archive = match ai.remove("legacy_presets") {
                Some(Value::Array(a)) => a,
                _ => Vec::new(),
            };
            archive.extend(presets);
            ai.insert("legacy_presets".into(), Value::Array(archive));
        }
    }

    pub fn profile(&self, id: &str) -> Option<&AiProfile> {
        self.profiles.iter().find(|p| p.id == id)
    }

    /// An archived pre-migration preset, by id: how a per-repo preset
    /// choice made before the migration is still understood.
    pub fn legacy_preset(&self, id: &str) -> Option<&AiPreset> {
        self.legacy_presets.iter().find(|p| p.id == id)
    }

    /// The profile an action runs on when the action names none.
    fn fallback_profile(&self) -> Result<&AiProfile, AiResolveError> {
        if !self.default_profile.is_empty() {
            if let Some(p) = self.profile(&self.default_profile) {
                return Ok(p);
            }
        }
        self.profiles.first().ok_or(AiResolveError::NoProfile)
    }

    /// Everything needed to run `action` in a repository whose override
    /// (if it has one) is `repo`. An empty override field inherits; an
    /// override naming a profile that is gone is ignored, an action
    /// naming one is an error.
    pub fn resolve(
        &self,
        action: AiAction,
        repo: Option<&AiActionSettings>,
    ) -> Result<AiResolved, AiResolveError> {
        if !self.enabled {
            return Err(AiResolveError::Disabled);
        }
        let settings = self.actions.get(action);
        let repo_profile = repo
            .map(|r| r.profile.as_str())
            .filter(|id| !id.is_empty())
            .and_then(|id| self.profile(id));
        let profile = match repo_profile {
            Some(p) => p,
            None if !settings.profile.is_empty() => self
                .profile(&settings.profile)
                .ok_or_else(|| AiResolveError::MissingProfile(settings.profile.clone()))?,
            None => self.fallback_profile()?,
        };
        let (action_instructions, action_from_repo) = match repo {
            Some(r) if !r.instructions.trim().is_empty() => (r.instructions.clone(), true),
            _ => (settings.instructions.clone(), false),
        };
        Ok(AiResolved {
            profile: profile.clone(),
            global_instructions: self.global_instructions.clone(),
            action_instructions,
            action_from_repo,
        })
    }

    /// A fresh id (`base`, `base-2`, ...) unused by profiles.
    pub fn new_profile_id(&self, base: &str) -> String {
        unique_id(base, |id| self.profiles.iter().any(|p| p.id == id))
    }

    /// Deletes a profile, and clears every reference to it so nothing
    /// is left pointing at a ghost.
    pub fn remove_profile(&mut self, id: &str) {
        self.profiles.retain(|p| p.id != id);
        if self.default_profile == id {
            self.default_profile.clear();
        }
        for action in AiAction::ALL {
            let settings = self.actions.get_mut(action);
            if settings.profile == id {
                settings.profile.clear();
            }
        }
    }
}

fn unique_id(base: &str, taken: impl Fn(&str) -> bool) -> String {
    let slug: String = base
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    let slug = if slug.is_empty() {
        "item".to_string()
    } else {
        slug
    };
    if !taken(&slug) {
        return slug;
    }
    (2..)
        .map(|n| format!("{slug}-{n}"))
        .find(|id| !taken(id))
        .expect("an unbounded range always yields")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;

    fn enabled() -> AiConfig {
        let mut ai = AiConfig {
            enabled: true,
            ..AiConfig::default()
        };
        ai.profiles.push(AiProfile::new(
            "cc",
            "Claude Code",
            AiProviderKind::LocalAgent,
        ));
        ai
    }

    /// A config as GitPoulpi wrote it before the redesign: presets, a
    /// default per action, one preset nobody defaults to.
    const OLD: &str = r#"
[general]
default_branch_name = "dev"

[ai]
enabled = true
default_profile = "gpt"

[[ai.profiles]]
id = "gpt"
name = "ChatGPT"
kind = "open_ai"
token_ref = "ai:gpt"

[[ai.profiles]]
id = "cc"
name = "Claude Code"
kind = "local_agent"
command = "claude"
args = ["-p"]

[[ai.presets]]
id = "conv"
name = "Conventional EN"
action = "commit_message"
profile = "gpt"
system_prompt = "Use conventional commits.\nIn English."

[[ai.presets]]
id = "fr"
name = "Francais"
action = "commit_message"
profile = "cc"
system_prompt = "En francais."

[[ai.presets]]
id = "terse"
name = "Terse"
action = "stash_message"
system_prompt = "Three words."

[ai.defaults]
commit_message = "conv"
stash_message = "builtin:stash_message"
explain_changes = "ghost"
"#;

    #[test]
    fn ai_is_off_by_default_and_resolves_to_disabled() {
        let config = Config::default();
        assert!(!config.ai.enabled);
        assert_eq!(
            config.ai.resolve(AiAction::CommitMessage, None),
            Err(AiResolveError::Disabled)
        );
    }

    #[test]
    fn a_default_profile_is_created_when_there_is_none() {
        let ai = Config::default().ai;
        assert_eq!(ai.profiles.len(), 1);
        assert_eq!(ai.profiles[0].id, DEFAULT_PROFILE_ID);
        assert_eq!(ai.profiles[0].name, "Default");
        assert_eq!(ai.default_profile, DEFAULT_PROFILE_ID);

        // A file with an empty list, or only `enabled`, gets it too.
        for raw in ["[ai]\nenabled = true\n", "[ai]\nprofiles = []\n", ""] {
            let (config, _) = Config::from_toml_str(raw).unwrap();
            assert_eq!(config.ai.profiles.len(), 1, "{raw}");
            assert_eq!(config.ai.default_profile, DEFAULT_PROFILE_ID, "{raw}");
        }

        // Other profiles are left alone, no Default is forced next to them.
        let raw = "[ai]\ndefault_profile = \"cc\"\n\n[[ai.profiles]]\nid = \"cc\"\nname = \"Mine\"\nkind = \"local_agent\"\n";
        let (config, _) = Config::from_toml_str(raw).unwrap();
        assert_eq!(config.ai.profiles.len(), 1);
        assert_eq!(config.ai.profiles[0].id, "cc");
        assert_eq!(config.ai.default_profile, "cc");
    }

    #[test]
    fn an_old_config_with_provider_and_model_loads_with_ai_off() {
        let raw = "[ai]\nprovider = \"anthropic\"\nmodel = \"claude-sonnet-5\"\nenabled = true\n";
        let (config, _) = Config::from_toml_str(raw).unwrap();
        assert!(!config.ai.enabled);
    }

    #[test]
    fn harness_and_env_round_trip_and_old_profiles_still_load() {
        let mut ai = enabled();
        ai.profiles[0].command = "/opt/bin/my-claude-alias".into();
        ai.profiles[0].harness = "claude".into();
        ai.profiles[0].env.insert(
            "CLAUDE_CONFIG_DIR".into(),
            "${env:HOME}/.claude-work".into(),
        );
        let config = Config {
            ai,
            ..Config::default()
        };
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        config.save(&path).unwrap();
        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(raw.contains("harness = \"claude\""), "{raw}");
        assert!(raw.contains("CLAUDE_CONFIG_DIR"), "{raw}");
        assert_eq!(Config::load(&path).unwrap().ai, config.ai);

        // A profile without the new keys loads with them empty, and a
        // profile that has none writes none.
        let old = "[ai]\nenabled = true\n[[ai.profiles]]\nid = \"a\"\nkind = \"local_agent\"\ncommand = \"claude\"\nargs = [\"-p\"]\n";
        let (loaded, _) = Config::from_toml_str(old).unwrap();
        assert!(loaded.ai.profiles[0].harness.is_empty());
        assert!(loaded.ai.profiles[0].env.is_empty());
        config.save(&path).unwrap();
        let plain = Config {
            ai: loaded.ai,
            ..Config::default()
        };
        plain.save(&path).unwrap();
        let raw = std::fs::read_to_string(&path).unwrap();
        // (`.env` also appears in `[privacy] sensitive_patterns`.)
        assert!(
            !raw.contains("harness") && !raw.contains("profiles.env"),
            "{raw}"
        );
    }

    #[test]
    fn sent_preview_defaults_on_and_a_partial_table_keeps_the_other_actions_on() {
        let (c, _) = Config::from_toml_str("[ai.sent_preview]\ncommit_message = false\n").unwrap();
        assert!(!c.ai.sent_preview.shows(AiAction::CommitMessage));
        assert!(c.ai.sent_preview.shows(AiAction::CodeReview));
        assert!(!c.ai.sent_preview.all_shown());
        let mut p = c.ai.sent_preview.clone();
        p.show_all();
        assert!(p.all_shown());
        // `config set` reaches a key the file never named.
        let mut cfg = c;
        cfg.set("ai.sent_preview.code_review", "false").unwrap();
        assert!(!cfg.ai.sent_preview.shows(AiAction::CodeReview));
        assert!(
            AiPrivacy::default().redact_secrets && AiPrivacy::default().exclude_sensitive_files
        );
    }

    #[test]
    fn a_new_config_round_trips_and_never_holds_a_key() {
        let mut ai = enabled();
        ai.global_instructions = "Write in English.\nBe terse.".into();
        ai.actions.commit_message = AiActionSettings {
            profile: "cc".into(),
            instructions: "Use conventional commits.".into(),
        };
        ai.actions.stash_message.instructions = "Three words.".into();
        let config = Config {
            ai,
            ..Config::default()
        };
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        config.save(&path).unwrap();
        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(raw.contains("[[ai.profiles]]"), "{raw}");
        assert!(raw.contains("token_ref = \"ai:default\""), "{raw}");
        assert!(raw.contains("[ai.actions.commit_message]"), "{raw}");
        assert!(!raw.contains("presets"), "{raw}");
        assert!(!raw.contains("defaults"), "{raw}");
        let loaded = Config::load(&path).unwrap();
        assert_eq!(loaded.ai, config.ai);
    }

    #[test]
    fn an_old_config_migrates_defaults_and_keeps_every_preset() {
        let (config, _) = Config::from_toml_str(OLD).unwrap();
        let ai = &config.ai;
        assert!(ai.enabled);
        assert_eq!(config.general.default_branch_name, "dev");
        assert_eq!(ai.profiles.len(), 2);
        // The default preset of an action is the action's settings.
        assert_eq!(
            ai.actions.commit_message,
            AiActionSettings {
                profile: "gpt".into(),
                instructions: "Use conventional commits.\nIn English.".into(),
            }
        );
        // Built-in or dangling defaults carry nothing.
        assert!(ai.actions.stash_message.is_empty());
        assert!(ai.actions.explain_changes.is_empty());
        // No preset is lost: all three survive verbatim in the archive.
        let ids: Vec<&str> = ai.legacy_presets.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(ids, ["conv", "fr", "terse"]);
        assert_eq!(
            ai.legacy_preset("terse").unwrap().system_prompt,
            "Three words."
        );
        assert_eq!(ai.legacy_preset("fr").unwrap().profile, "cc");

        // Saving writes the new shape, and reloading changes nothing.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        config.save(&path).unwrap();
        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(!raw.contains("[ai.defaults]"), "{raw}");
        assert!(!raw.contains("[[ai.presets]]"), "{raw}");
        assert!(raw.contains("[[ai.legacy_presets]]"), "{raw}");
        assert_eq!(Config::load(&path).unwrap().ai, config.ai);
    }

    #[test]
    fn the_in_file_editors_migrate_too_and_never_resurrect_old_keys() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, OLD).unwrap();
        crate::set_in_file(&path, "ai.global_instructions", "Be kind.").unwrap();
        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(!raw.contains("[ai.defaults]"), "{raw}");
        let loaded = Config::load(&path).unwrap();
        assert_eq!(loaded.ai.global_instructions, "Be kind.");
        assert_eq!(
            loaded.ai.actions.commit_message.instructions,
            "Use conventional commits.\nIn English."
        );
        assert_eq!(loaded.ai.legacy_presets.len(), 3);
    }

    #[test]
    fn resolution_layers_repo_over_action_over_default() {
        let mut ai = enabled();
        ai.profiles
            .push(AiProfile::new("gpt", "GPT", AiProviderKind::OpenAi));
        ai.global_instructions = "G".into();
        ai.actions.commit_message = AiActionSettings {
            profile: "cc".into(),
            instructions: "A".into(),
        };
        let pick = |ai: &AiConfig, repo: Option<&AiActionSettings>| {
            let r = ai.resolve(AiAction::CommitMessage, repo).unwrap();
            (r.profile.id, r.action_instructions)
        };

        // Default profile for an action that names none.
        let r = ai.resolve(AiAction::StashMessage, None).unwrap();
        assert_eq!(r.profile.id, DEFAULT_PROFILE_ID);
        assert_eq!(r.global_instructions, "G");
        assert_eq!(r.action_instructions, "");

        // The action's own profile and instructions.
        assert_eq!(pick(&ai, None), ("cc".into(), "A".into()));

        // An empty repo override inherits everything.
        let empty = AiActionSettings::default();
        assert_eq!(pick(&ai, Some(&empty)), pick(&ai, None));

        // A repo override replaces what it sets, field by field.
        let repo = AiActionSettings {
            profile: "gpt".into(),
            instructions: String::new(),
        };
        assert_eq!(pick(&ai, Some(&repo)), ("gpt".into(), "A".into()));
        let repo = AiActionSettings {
            profile: String::new(),
            instructions: "R".into(),
        };
        assert_eq!(pick(&ai, Some(&repo)), ("cc".into(), "R".into()));

        // A stale repo profile is ignored; a stale action profile is an error.
        let repo = AiActionSettings {
            profile: "gone".into(),
            instructions: String::new(),
        };
        assert_eq!(pick(&ai, Some(&repo)), ("cc".into(), "A".into()));
        ai.actions.commit_message.profile = "gone".into();
        assert_eq!(
            ai.resolve(AiAction::CommitMessage, None),
            Err(AiResolveError::MissingProfile("gone".into()))
        );
        ai.profiles.clear();
        assert_eq!(
            ai.resolve(AiAction::StashMessage, None),
            Err(AiResolveError::NoProfile)
        );
    }

    #[test]
    fn removing_a_profile_leaves_no_dangling_reference() {
        let mut ai = enabled();
        ai.default_profile = "cc".into();
        ai.actions.stash_message.profile = "cc".into();
        ai.remove_profile("cc");
        assert!(ai.default_profile.is_empty());
        assert!(ai.actions.stash_message.profile.is_empty());
    }

    #[test]
    fn ids_are_slugged_and_unique() {
        let ai = enabled();
        assert_eq!(ai.new_profile_id("Claude Code"), "claude-code");
        assert_eq!(ai.new_profile_id("cc"), "cc-2");
    }

    #[test]
    fn the_ai_lists_are_written_under_the_ai_table_and_removed_cleanly() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "[general]\ndefault_branch_name = \"dev\"\n").unwrap();
        let profiles = vec![AiProfile::new("gpt", "ChatGPT", AiProviderKind::OpenAi)];
        crate::set_list_in_file(&path, "ai.profiles", &profiles).unwrap();
        crate::set_in_file(&path, "ai.enabled", "true").unwrap();
        let loaded = Config::load(&path).unwrap();
        assert!(loaded.ai.enabled);
        assert_eq!(loaded.ai.profiles, profiles);
        assert_eq!(loaded.general.default_branch_name, "dev");
        crate::set_list_in_file::<AiProfile>(&path, "ai.profiles", &[]).unwrap();
        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(!raw.contains("profiles"), "{raw}");
        assert!(raw.contains("enabled = true"), "{raw}");
    }

    #[test]
    fn the_terminal_run_mode_round_trips_and_old_profiles_stay_headless() {
        let old = AiProfile::new("cc", "Claude", AiProviderKind::LocalAgent);
        let raw = toml::to_string(&old).unwrap();
        for key in ["run_mode", "terminal_args", "workdir", "timeout_secs"] {
            assert!(!raw.contains(key), "defaults are not written: {raw}");
        }
        let loaded: AiProfile = toml::from_str(&raw).unwrap();
        assert_eq!(loaded.run_mode, AgentRunMode::Headless);
        assert_eq!(loaded.effective_timeout_secs(), 120);

        let mut profile = old;
        profile.run_mode = AgentRunMode::Terminal;
        profile.terminal_args = vec!["--tools".into(), String::new()];
        profile.workdir = AgentWorkdir::Repository;
        profile.timeout_secs = 90;
        let raw = toml::to_string(&profile).unwrap();
        assert!(raw.contains("run_mode = \"terminal\""), "{raw}");
        assert!(raw.contains("workdir = \"repository\""), "{raw}");
        let back: AiProfile = toml::from_str(&raw).unwrap();
        assert_eq!(back, profile);
        // Each mode keeps its own arguments.
        assert_eq!(back.active_args(), &back.terminal_args);
        assert_eq!(back.args, vec!["-p".to_string()]);
        let mut term_default = back.clone();
        term_default.timeout_secs = 0;
        assert_eq!(term_default.effective_timeout_secs(), 180);
    }

    #[test]
    fn the_no_extra_file_switch_round_trips_and_old_files_still_load() {
        // A file written before the switch existed.
        let old: RepoAiReview = toml::from_str("extra_file = \"docs/R.md\"\n").unwrap();
        assert_eq!(old.no_extra_file, None);
        assert_eq!(old.extra_choice().as_deref(), Some("docs/R.md"));
        let legacy: RepoAiReview = toml::from_str("extra_file = \"\"\n").unwrap();
        assert_eq!(legacy.extra_choice().as_deref(), Some(""), "still none");
        // Unset is not written; set is.
        assert!(!toml::to_string(&old).unwrap().contains("no_extra_file"));
        let on = RepoAiReview {
            no_extra_file: Some(true),
            ..Default::default()
        };
        let raw = toml::to_string(&on).unwrap();
        assert!(raw.contains("no_extra_file = true"), "{raw}");
        assert_eq!(toml::from_str::<RepoAiReview>(&raw).unwrap(), on);
        assert_eq!(on.extra_choice().as_deref(), Some(""));
        assert!(!on.is_empty());
    }

    #[test]
    fn the_limits_table_defaults_to_todays_cap_and_reads_back() {
        let ai: AiConfig = toml::from_str("enabled = true").unwrap();
        assert_eq!(ai.limits.conflict_bytes, 40_000);
        assert_eq!(ai.limits, AiLimits::default());
        let ai: AiConfig = toml::from_str("[limits]\nconflict_bytes = 90000\n").unwrap();
        assert_eq!(ai.limits.conflict_bytes, 90_000);
        assert_eq!(ai.limits.conflict_bytes_clamped(), 90_000);
        let raw = toml::to_string(&ai).unwrap();
        assert!(raw.contains("conflict_bytes = 90000"), "{raw}");
        // A hand-edited extreme is brought back into range.
        let tiny = AiLimits {
            conflict_bytes: 0,
            ..AiLimits::default()
        };
        assert_eq!(tiny.conflict_bytes_clamped(), MIN_CONFLICT_BYTES);
        let huge = AiLimits {
            conflict_bytes: usize::MAX,
            ..AiLimits::default()
        };
        assert_eq!(huge.conflict_bytes_clamped(), MAX_CONFLICT_BYTES);
    }

    #[test]
    fn every_other_limit_defaults_to_todays_cap_reads_back_and_clamps() {
        let ai: AiConfig = toml::from_str("enabled = true").unwrap();
        let l = ai.limits;
        assert_eq!(
            (
                l.diff_bytes,
                l.review_file_bytes,
                l.review_text_bytes,
                l.review_threads_bytes,
                l.review_context_file_bytes,
                l.review_context_bytes
            ),
            (60_000, 12_000, 4_000, 6_000, 8_192, 16_384)
        );
        let ai: AiConfig = toml::from_str(
            "[limits]\ndiff_bytes = 90000\nreview_file_bytes = 20000\nreview_text_bytes = 5000\n\
             review_threads_bytes = 7000\nreview_context_file_bytes = 9000\n\
             review_context_bytes = 30000\n",
        )
        .unwrap();
        assert_eq!(ai.limits.diff_bytes_clamped(), 90_000);
        assert_eq!(ai.limits.review_file_bytes_clamped(), 20_000);
        assert_eq!(ai.limits.review_text_bytes_clamped(), 5_000);
        assert_eq!(ai.limits.review_threads_bytes_clamped(), 7_000);
        assert_eq!(ai.limits.review_context_file_bytes_clamped(), 9_000);
        assert_eq!(ai.limits.review_context_bytes_clamped(), 30_000);
        assert_eq!(
            ai.limits.conflict_bytes, 40_000,
            "an unnamed key keeps its default"
        );
        let raw = toml::to_string(&ai).unwrap();
        assert!(raw.contains("review_context_bytes = 30000"), "{raw}");
        let zero = AiLimits {
            diff_bytes: 0,
            review_file_bytes: 0,
            review_text_bytes: usize::MAX,
            ..AiLimits::default()
        };
        assert_eq!(zero.diff_bytes_clamped(), MIN_LIMIT_BYTES);
        assert_eq!(zero.review_file_bytes_clamped(), MIN_LIMIT_BYTES);
        assert_eq!(zero.review_text_bytes_clamped(), MAX_LIMIT_BYTES);
    }

    #[test]
    fn the_review_table_defaults_and_reads_back() {
        let ai: AiConfig = toml::from_str("enabled = true").unwrap();
        assert_eq!(ai.review, AiReviewContext::default());
        assert!(ai.review.include_claude_md && ai.review.include_agents_md);
        let ai: AiConfig = toml::from_str(
            "[review]\nrun_in_repository = \"yes\"\ninclude_claude_md = false\nextra_file = \"docs/R.md\"\n",
        )
        .unwrap();
        assert_eq!(ai.review.run_in_repository, RunInRepo::Yes);
        assert!(!ai.review.include_claude_md);
        assert!(ai.review.include_agents_md, "unmentioned keeps its default");
        assert_eq!(ai.review.extra_file, "docs/R.md");
    }

    #[test]
    fn run_in_repo_overrides_the_profile_workdir_for_a_run() {
        use AgentWorkdir as W;
        for profile in W::ALL {
            assert_eq!(RunInRepo::Profile.apply(profile), profile);
            assert_eq!(RunInRepo::Yes.apply(profile), W::Repository);
        }
        // `no` only moves an agent that would have run in the repository.
        assert_eq!(RunInRepo::No.apply(W::Repository), W::Dedicated);
        assert_eq!(RunInRepo::No.apply(W::Default), W::Default);
        assert_eq!(RunInRepo::No.apply(W::Dedicated), W::Dedicated);
        for v in RunInRepo::ALL {
            assert_eq!(RunInRepo::from_key(v.key()), Some(v));
        }
        assert_eq!(RunInRepo::from_key("maybe"), None);
    }
}
