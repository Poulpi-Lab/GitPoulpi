//! The Settings "AI" page: the general switch, connection profiles, the
//! global instructions, and per-action profiles and instructions; the
//! repository page overrides them per repository.
//!
//! A child module of `preferences` so it can use that module's private
//! `Field`/`Control`/`Target` machinery: the page is a list of ordinary
//! rows, and what is special about it -- a key that goes to
//! `gitpoulpi-secrets` instead of `config.toml`, a *Test connection*
//! that calls a model, a multi-line system-prompt editor -- lives here.
//!
//! Nothing here blocks the input loop: storing a key (a credential-helper
//! subprocess) and testing a connection (a network round trip) run on
//! their own threads and report on a channel `poll_ai` drains.

mod hosting_reply_rows;
mod limits_rows;
mod open_agent;
mod privacy_rows;
mod review_browse;
mod review_rows;

pub(in crate::preferences) use privacy_rows::PrivacyRow;
pub(in crate::preferences) use review_rows::ReviewRow;

use crate::th;
use std::collections::HashMap;
use std::sync::mpsc;

use gitpoulpi_config::{AiAction, AiProfile, AiProviderKind, RepoInstructionsMode};

use super::editor_close::Act;
use super::*;
use crate::text_edit::KeyEdit;
use gitpoulpi_i18n::msg::ai_page as msg;

/// Which piece of the AI page a row edits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AiTarget {
    DefaultProfile,
    ProfileName,
    ProfileKind,
    ProfileModel,
    ProfileBaseUrl,
    ProfileKey,
    ProfileCommand,
    ProfileArgs,
    /// The kind of agent (what the suggestions follow), apart from the
    /// command.
    ProfileHarness,
    /// The value of the n-th environment variable of the profile (in key
    /// order).
    ProfileEnv(usize),
    /// A combo: pick a suggested variable name or type `NAME` /
    /// `NAME=value`.
    ProfileEnvAdd,
    /// The HTTP provider preset combo.
    ProfilePreset,
    /// The local agent's option toggles (a combo whose pick adds or
    /// removes the option's arguments).
    ProfileOptions,
    /// The local agent's model, written as the harness's flag in the
    /// arguments.
    ProfileAgentModel,
    /// Headless or the virtual terminal.
    ProfileRunMode,
    /// The agent's working folder.
    ProfileWorkdir,
    /// Seconds before the agent is given up on.
    ProfileTimeout,
    /// An action's profile combo (`ai.actions.<action>.profile`).
    ActionProfile(AiScope, AiAction),
    /// A section's instructions (`ai.global_instructions` or
    /// `ai.actions.<action>.instructions`, or the repository's override),
    /// edited in the multi-line overlay.
    Instructions(AiScope, AiSection),
    /// The repository instructions' combo: add to, or replace, the global
    /// instructions.
    RepoMode,
    /// One of the four code review context rows (`[ai.review]`).
    Review(AiScope, ReviewRow),
    /// A privacy row (`[ai.privacy]`, `[ai.sent_preview]`).
    Privacy(PrivacyRow),
}

/// Where an AI row is saved: `config.toml`, or this repository's
/// `RepoPreferences::ai_overrides` (`.gitpoulpi/repo-state.toml`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AiScope {
    Global,
    Repo,
}

/// A block of the page that has instructions: the global one, or an action's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AiSection {
    Global,
    Action(AiAction),
    /// The base prompt of the writing help on a host
    /// (`ai.hosting_reply.base_prompt`): global only, no repository twin.
    HostingBase,
}

impl AiSection {
    fn title(self) -> &'static str {
        match self {
            Self::Global => msg::TITLE_GLOBAL,
            Self::Action(AiAction::CommitMessage) => msg::TITLE_COMMIT_MESSAGE,
            Self::Action(AiAction::ExplainChanges) => msg::TITLE_EXPLAIN,
            Self::Action(AiAction::StashMessage) => msg::TITLE_STASH,
            Self::Action(AiAction::PullRequestDescription) => msg::TITLE_PULL_REQUEST,
            Self::Action(AiAction::CommitComposer) => msg::TITLE_COMPOSER,
            Self::Action(AiAction::ConflictResolve) => msg::TITLE_CONFLICT_RESOLVE,
            Self::Action(AiAction::CodeReview) => gitpoulpi_i18n::msg::review_ai::TITLE_CODE_REVIEW,
            Self::Action(AiAction::SplitCommit) => msg::TITLE_SPLIT_COMMIT,
            Self::Action(AiAction::RewriteScript) => gitpoulpi_i18n::msg::rewrite_ai::TITLE,
            Self::Action(AiAction::RewriteRules) => gitpoulpi_i18n::msg::rewrite_rules_ai::TITLE,
            Self::Action(AiAction::WorktreeName) => msg::TITLE_WORKTREE_NAME,
            Self::Action(AiAction::HostingReply) => msg::TITLE_HOSTING_REPLY,
            Self::Action(AiAction::IssueBody) => msg::TITLE_ISSUE_BODY,
            Self::Action(AiAction::SequenceDraft) => gitpoulpi_i18n::msg::sequence_ai::TITLE,
            Self::HostingBase => msg::TITLE_HOSTING_BASE,
        }
        .get()
    }

    /// "The profile GitPoulpi will use to ...": what the action does, as a
    /// whole sentence (`repo`: the repository page's wording).
    fn profile_help(action: AiAction, repo: bool) -> &'static str {
        match (action, repo) {
            (AiAction::CommitMessage, false) => msg::PROFILE_HELP_COMMIT_MESSAGE,
            (AiAction::ExplainChanges, false) => msg::PROFILE_HELP_EXPLAIN,
            (AiAction::StashMessage, false) => msg::PROFILE_HELP_STASH,
            (AiAction::PullRequestDescription, false) => msg::PROFILE_HELP_PULL_REQUEST,
            (AiAction::CommitComposer, false) => msg::PROFILE_HELP_COMPOSER,
            (AiAction::SplitCommit, false) => msg::PROFILE_HELP_SPLIT_COMMIT,
            (AiAction::RewriteScript, false) => gitpoulpi_i18n::msg::rewrite_ai::PROFILE_HELP,
            (AiAction::RewriteRules, false) => gitpoulpi_i18n::msg::rewrite_rules_ai::PROFILE_HELP,
            (AiAction::WorktreeName, false) => msg::PROFILE_HELP_WORKTREE_NAME,
            (AiAction::WorktreeName, true) => msg::REPO_PROFILE_HELP_WORKTREE_NAME,
            (AiAction::HostingReply, false) => msg::PROFILE_HELP_HOSTING_REPLY,
            (AiAction::HostingReply, true) => msg::REPO_PROFILE_HELP_HOSTING_REPLY,
            (AiAction::IssueBody, false) => msg::PROFILE_HELP_ISSUE_BODY,
            (AiAction::IssueBody, true) => msg::REPO_PROFILE_HELP_ISSUE_BODY,
            (AiAction::SequenceDraft, false) => gitpoulpi_i18n::msg::sequence_ai::PROFILE_HELP,
            (AiAction::SequenceDraft, true) => gitpoulpi_i18n::msg::sequence_ai::REPO_PROFILE_HELP,
            (AiAction::ConflictResolve, false) => msg::PROFILE_HELP_CONFLICT_RESOLVE,
            (AiAction::CodeReview, false) => {
                gitpoulpi_i18n::msg::review_ai::PROFILE_HELP_CODE_REVIEW
            }
            (AiAction::CommitMessage, true) => msg::REPO_PROFILE_HELP_COMMIT_MESSAGE,
            (AiAction::ExplainChanges, true) => msg::REPO_PROFILE_HELP_EXPLAIN,
            (AiAction::StashMessage, true) => msg::REPO_PROFILE_HELP_STASH,
            (AiAction::PullRequestDescription, true) => msg::REPO_PROFILE_HELP_PULL_REQUEST,
            (AiAction::CommitComposer, true) => msg::REPO_PROFILE_HELP_COMPOSER,
            (AiAction::SplitCommit, true) => msg::REPO_PROFILE_HELP_SPLIT_COMMIT,
            (AiAction::RewriteScript, true) => gitpoulpi_i18n::msg::rewrite_ai::REPO_PROFILE_HELP,
            (AiAction::RewriteRules, true) => {
                gitpoulpi_i18n::msg::rewrite_rules_ai::REPO_PROFILE_HELP
            }
            (AiAction::ConflictResolve, true) => msg::REPO_PROFILE_HELP_CONFLICT_RESOLVE,
            (AiAction::CodeReview, true) => {
                gitpoulpi_i18n::msg::review_ai::REPO_PROFILE_HELP_CODE_REVIEW
            }
        }
        .get()
    }

    /// The `config.toml` key holding this section's instructions.
    fn key(self) -> String {
        match self {
            Self::Global => "ai.global_instructions".to_string(),
            Self::Action(a) => format!("ai.actions.{}.instructions", a.key()),
            Self::HostingBase => "ai.hosting_reply.base_prompt".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AiButton {
    Test,
    /// Open the agent once, visibly, in its working folder, so the user
    /// answers its first-run questions (folder trust, login...).
    OpenAgent,
    /// Type a new value for the n-th environment variable of the profile.
    EnvEdit(usize),
    /// Drop the n-th environment variable (its `[x]` button).
    EnvRemove(usize),
    /// Expand or collapse the archive of old presets.
    ToggleLegacy,
    /// Delete `ai.legacy_presets`.
    DeleteLegacy,
    /// Drop this repository's override for an action.
    ResetRepo(AiAction),
    /// Drop this repository's own code review context values.
    ResetReview,
    /// "Browse..." next to the extra file field of the code review rows.
    BrowseExtra(AiScope),
    /// "Show all again": every request is shown before it is sent.
    ShowAllPreviews,
}

/// What a background thread reports.
enum AiEvent {
    KeyStored {
        token_ref: String,
        result: Result<(), String>,
    },
    Tested {
        profile_id: String,
        result: Result<(), String>,
    },
}

/// The outcome of the last *Test connection*.
#[derive(Debug, Clone, PartialEq, Eq)]
enum TestState {
    Running,
    Ok,
    Failed(String),
}

/// The multi-line instructions editor, an overlay over the page.
struct PromptEdit {
    scope: AiScope,
    section: AiSection,
    text: String,
    /// The shared editing core: caret, selection, mouse mapping.
    field: crate::text_edit::FieldEditor<()>,
    /// Rows of the text area at the last paint (PageUp / PageDown).
    page_rows: usize,
    /// The Variables dropdown and the `${` autocomplete.
    vars: prompt_vars::VarUi,
    close: super::editor_close::EditorClose,
    /// Whether the Quick insert row (above the text) has the keyboard.
    chips_focused: bool,
    /// The focused chip of that row.
    chip: usize,
    /// A one-line note for the hint row (a chip already inserted); cleared
    /// by the next key.
    note: Option<String>,
}

/// The AI page's own state (selection, presence of keys, background work).
#[derive(Default)]
pub(super) struct AiPage {
    profile: usize,
    /// `(variable, field index)` of the environment value being typed: that
    /// row is drawn as a text field for the length of the edit.
    env_edit: Option<(usize, usize)>,
    /// The focused chip of the selected chips row.
    pub(super) chip: usize,
    /// Whether the old-presets archive is expanded.
    legacy_open: bool,
    /// Whether a key is stored, by `token_ref`; `None` is "checking".
    key_present: HashMap<String, Option<bool>>,
    tests: HashMap<String, TestState>,
    rx: Option<mpsc::Receiver<AiEvent>>,
    tx: Option<mpsc::Sender<AiEvent>>,
    presence_rx: Option<mpsc::Receiver<(String, bool)>>,
    prompt: Option<PromptEdit>,
    /// The editor's Save / Cancel buttons, recorded by the paint walk.
    prompt_hits: Vec<(Rect, PromptButton)>,
    /// The editor's Quick insert chips, recorded by the paint walk.
    chip_hits: Vec<(Rect, usize)>,
    /// The file picker of "Browse..." next to the extra file field, with the
    /// page it fills.
    browse: Option<(AiScope, crate::file_browse::FileBrowse)>,
}

/// A button of the custom-instructions editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PromptButton {
    Save,
    Cancel,
    /// Step the text's edit history back (`Ctrl+Z`).
    Undo,
    /// Step it forward again (`Ctrl+Y`).
    Redo,
}

impl AiPage {
    pub(super) fn selected_index(&self) -> usize {
        self.profile
    }

    pub(super) fn select_index(&mut self, index: usize) {
        self.profile = index;
    }

    pub(super) fn prompt_open(&self) -> bool {
        self.prompt.is_some()
    }

    /// The focused chip of a row of `len` chips.
    pub(super) fn chip_focus(&self, len: usize) -> usize {
        self.chip.min(len.saturating_sub(1))
    }

    fn sender(&mut self) -> mpsc::Sender<AiEvent> {
        if self.tx.is_none() {
            let (tx, rx) = mpsc::channel();
            self.tx = Some(tx);
            self.rx = Some(rx);
        }
        self.tx.clone().expect("just created")
    }
}

/// Splits `text` into arguments on whitespace, honouring double quotes.
/// What a secret environment value shows as.
const ENV_MASK: &str = "\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}";

fn arg_purpose_text(purpose: gitpoulpi_ai::presets::ArgPurpose) -> &'static str {
    use gitpoulpi_ai::presets::ArgPurpose as P;
    match purpose {
        P::NoTools => msg::ARG_NO_TOOLS,
        P::NoSession => msg::ARG_NO_SESSION,
        P::PlainText => msg::ARG_PLAIN_TEXT,
        P::OneTurn => msg::ARG_ONE_TURN,
        P::LowEffort => msg::ARG_LOW_EFFORT,
        P::Quiet => msg::ARG_QUIET,
        P::ReadOnly => msg::ARG_READ_ONLY,
        P::NoPrompts => msg::ARG_NO_PROMPTS,
        P::NoGitRepo => msg::ARG_NO_GIT_REPO,
        P::NoStream => msg::ARG_NO_STREAM,
        P::InlineScreen => msg::ARG_INLINE_SCREEN,
        P::NoEdits => msg::ARG_NO_EDITS,
        P::NoProjectContext => msg::ARG_NO_PROJECT_CONTEXT,
        P::NoUpdate => msg::ARG_NO_UPDATE,
        P::NoTelemetry => msg::ARG_NO_TELEMETRY,
        P::Offline => msg::ARG_OFFLINE,
    }
    .get()
}

fn env_purpose_text(purpose: gitpoulpi_ai::presets::EnvPurpose) -> &'static str {
    use gitpoulpi_ai::presets::EnvPurpose as P;
    match purpose {
        P::ApiKey => msg::ENV_API_KEY,
        P::ConfigDir => msg::ENV_CONFIG_DIR,
        P::DefaultModel => msg::ENV_DEFAULT_MODEL,
        P::Effort => msg::ENV_EFFORT,
        P::ThinkingBudget => msg::ENV_THINKING_BUDGET,
        P::MaxOutput => msg::ENV_MAX_OUTPUT,
        P::NoTelemetry => msg::ENV_NO_TELEMETRY,
        P::NoTraffic => msg::ENV_NO_TRAFFIC,
        P::NoUpdate => msg::ENV_NO_UPDATE,
        P::BaseUrl => msg::ENV_BASE_URL,
        P::Host => msg::ENV_HOST,
        P::Proxy => msg::ENV_PROXY,
        P::NoColor => msg::ENV_NO_COLOR,
        P::Offline => msg::ENV_OFFLINE,
        P::Permissions => msg::ENV_PERMISSIONS,
        P::Timeout => msg::ENV_TIMEOUT,
    }
    .get()
}

/// A new agent type: each mode starts from the type's own defaults; a type
/// with no terminal mode goes back to headless.
fn reset_args_for(profile: &mut AiProfile, h: &'static gitpoulpi_ai::presets::Harness) {
    use gitpoulpi_config::AgentRunMode;
    profile.args = h.default_args();
    profile.terminal_args = h.default_args_for(AgentRunMode::Terminal);
    if h.terminal().is_none() {
        profile.run_mode = AgentRunMode::Headless;
    }
}

fn split_args(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut has = false;
    for c in text.chars() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
                has = true;
            }
            c if c.is_whitespace() && !in_quotes => {
                if has {
                    out.push(std::mem::take(&mut current));
                    has = false;
                }
            }
            c => {
                current.push(c);
                has = true;
            }
        }
    }
    if has {
        out.push(current);
    }
    out
}

/// The inverse of [`split_args`].
fn join_args(args: &[String]) -> String {
    args.iter()
        .map(|a| {
            if a.is_empty() || a.chars().any(char::is_whitespace) {
                format!("\"{a}\"")
            } else {
                a.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// A provider's name on the page (the config crate's `label()` is English).
fn provider_label(kind: AiProviderKind) -> &'static str {
    match kind {
        AiProviderKind::OpenAi => msg::PROVIDER_OPENAI,
        AiProviderKind::Anthropic => msg::PROVIDER_ANTHROPIC,
        AiProviderKind::OpenRouter => msg::PROVIDER_OPENROUTER,
        AiProviderKind::OpenAiCompatible => msg::PROVIDER_COMPATIBLE,
        AiProviderKind::LocalAgent => msg::PROVIDER_LOCAL_AGENT,
    }
    .get()
}

/// An action's name for the archive of old presets.
fn action_label(action: AiAction) -> &'static str {
    match action {
        AiAction::CommitMessage => msg::ACTION_COMMIT_MESSAGE,
        AiAction::ExplainChanges => msg::ACTION_EXPLAIN,
        AiAction::StashMessage => msg::ACTION_STASH,
        AiAction::PullRequestDescription => msg::ACTION_PULL_REQUEST,
        AiAction::CommitComposer => msg::ACTION_COMPOSER,
        AiAction::SplitCommit => msg::ACTION_SPLIT_COMMIT,
        AiAction::RewriteScript => gitpoulpi_i18n::msg::rewrite_ai::ACTION,
        AiAction::RewriteRules => gitpoulpi_i18n::msg::rewrite_rules_ai::ACTION,
        AiAction::WorktreeName => msg::ACTION_WORKTREE_NAME,
        AiAction::HostingReply => msg::ACTION_HOSTING_REPLY,
        AiAction::IssueBody => msg::ACTION_ISSUE_BODY,
        AiAction::SequenceDraft => gitpoulpi_i18n::msg::sequence_ai::ACTION,
        AiAction::ConflictResolve => msg::ACTION_CONFLICT_RESOLVE,
        AiAction::CodeReview => gitpoulpi_i18n::msg::review_ai::ACTION_CODE_REVIEW,
    }
    .get()
}

/// How a repository's instructions combine with the global ones.
fn mode_label(mode: RepoInstructionsMode) -> &'static str {
    match mode {
        RepoInstructionsMode::Add => msg::MODE_ADD,
        RepoInstructionsMode::Replace => msg::MODE_REPLACE,
    }
    .get()
}

/// `Name (Provider)`, how a profile is listed in the dropdowns.
fn profile_option(p: &AiProfile) -> String {
    format!("{} ({})", p.display_name(), provider_label(p.kind))
}

pub(super) fn first_line_preview(text: &str) -> String {
    let mut lines = text.lines();
    let first = lines.next().unwrap_or("").trim().to_string();
    let more = lines.count();
    let first = if crate::ui_kit::display_width(&first) > 48 {
        format!("{}...", crate::ui_kit::take_width(&first, 48))
    } else {
        first
    };
    if more > 0 {
        format!("{first}  {}", msg::preview_more(more as u64))
    } else if first.is_empty() {
        msg::PREVIEW_EMPTY.get().to_string()
    } else {
        first
    }
}

impl PreferencesState {
    fn write_ai_list<T: serde::Serialize>(&mut self, key: &str, items: &[T]) {
        let Some(path) = self.config_file() else {
            self.message = Some(msg::NO_CONFIG_DIR.get().to_string());
            return;
        };
        match set_list_in_file(&path, key, items) {
            Ok(()) => self.message = Some(msg::list_saved(key)),
            Err(err) => self.message = Some(msg::write_failed(key, err)),
        }
    }

    fn write_ai_profiles(&mut self) {
        let items = self.config.ai.profiles.clone();
        self.write_ai_list("ai.profiles", &items);
    }

    fn selected_ai_profile(&self) -> Option<usize> {
        let len = self.config.ai.profiles.len();
        (len > 0).then(|| self.ai.profile.min(len - 1))
    }

    /// Starts the background probe of which profiles have a stored key.
    /// Called when the page is first looked at.
    pub(super) fn ai_probe_keys(&mut self) {
        if !self.config.ai.enabled {
            return;
        }
        let refs: Vec<String> = self
            .config
            .ai
            .profiles
            .iter()
            .filter(|p| !p.token_ref.is_empty())
            .map(|p| p.token_ref.clone())
            .collect();
        for r in &refs {
            self.ai.key_present.entry(r.clone()).or_insert(None);
        }
        self.ai.presence_rx = spawn_token_presence_probe(std::sync::Arc::clone(&self.tokens), refs);
    }

    pub(super) fn ai_commit_choice(&mut self, target: AiTarget, next: usize) {
        match target {
            AiTarget::DefaultProfile => {
                let id = self
                    .config
                    .ai
                    .profiles
                    .get(next)
                    .map(|p| p.id.clone())
                    .unwrap_or_default();
                self.write_global("ai.default_profile", &id);
            }
            AiTarget::ActionProfile(scope, action) => {
                let id = next
                    .checked_sub(1)
                    .and_then(|i| self.config.ai.profiles.get(i))
                    .map(|p| p.id.clone())
                    .unwrap_or_default();
                match scope {
                    AiScope::Global => {
                        self.write_global(&format!("ai.actions.{}.profile", action.key()), &id);
                    }
                    AiScope::Repo => self.edit_repo_override(action, |o| o.profile = id),
                }
            }
            AiTarget::Review(scope, row) => self.review_commit_choice(scope, row, next),
            AiTarget::RepoMode => {
                if let Some(mode) = RepoInstructionsMode::ALL.get(next).copied() {
                    self.repo_state.preferences.ai_instructions.mode = mode;
                    self.write_repo_state();
                }
            }
            AiTarget::ProfileKind => {
                let Some(i) = self.selected_ai_profile() else {
                    return;
                };
                let Some(kind) = AiProviderKind::ALL.get(next).copied() else {
                    return;
                };
                let profile = &mut self.config.ai.profiles[i];
                let old = profile.kind;
                profile.kind = kind;
                if profile.model.trim().is_empty() || profile.model == old.default_model() {
                    profile.model = kind.default_model().to_string();
                }
                profile.token_ref = if kind.needs_key() {
                    AiProfile::token_ref_for(&profile.id)
                } else {
                    String::new()
                };
                if kind == AiProviderKind::LocalAgent && profile.command.is_empty() {
                    profile.command = "claude".into();
                    profile.harness = "claude".into();
                    profile.args = vec!["-p".into()];
                }
                self.write_ai_profiles();
                self.ai_probe_keys();
            }
            _ => {}
        }
    }

    /// A `Combo` row of the profile editor: `value` is the option's value
    /// or typed text.
    pub(super) fn ai_commit_combo(&mut self, target: AiTarget, value: &str) {
        use gitpoulpi_ai::presets;
        let Some(i) = self.selected_ai_profile() else {
            return;
        };
        match target {
            AiTarget::ProfileHarness => {
                let profile = &mut self.config.ai.profiles[i];
                let old = presets::harness_of(profile).map(|h| h.command);
                if value == presets::OTHER_HARNESS {
                    profile.harness = presets::OTHER_HARNESS.into();
                } else if let Some(h) = presets::harness_named(value) {
                    profile.harness = h.command.into();
                    // Picking a type writes its command (owner); the user
                    // edits it afterwards for an alias or a wrapper.
                    profile.command = h.command.into();
                    if old != Some(h.command) {
                        reset_args_for(profile, h);
                    }
                } else {
                    return;
                }
                self.write_ai_profiles();
            }
            AiTarget::ProfileRunMode => {
                use gitpoulpi_config::AgentRunMode;
                let profile = &mut self.config.ai.profiles[i];
                let mode = match value {
                    "terminal" => AgentRunMode::Terminal,
                    "headless" => AgentRunMode::Headless,
                    _ => return,
                };
                let harness = presets::harness_of(profile);
                if mode == AgentRunMode::Terminal && harness.and_then(|h| h.terminal()).is_none() {
                    return;
                }
                profile.run_mode = mode;
                if mode == AgentRunMode::Terminal && profile.terminal_args.is_empty() {
                    if let Some(h) = harness {
                        profile.terminal_args = h.default_args_for(mode);
                    }
                }
                self.write_ai_profiles();
            }
            AiTarget::ProfileWorkdir => {
                use gitpoulpi_config::AgentWorkdir;
                let profile = &mut self.config.ai.profiles[i];
                profile.workdir = match value {
                    "dedicated" => AgentWorkdir::Dedicated,
                    "repository" => AgentWorkdir::Repository,
                    "default" => AgentWorkdir::Default,
                    _ => return,
                };
                self.write_ai_profiles();
            }
            AiTarget::ProfileEnvAdd => {
                let typed = value.trim();
                let (name, given) = match typed.split_once('=') {
                    Some((n, v)) => (n.trim(), Some(v.trim())),
                    None => (typed, None),
                };
                if name.is_empty() || name.contains(char::is_whitespace) {
                    return;
                }
                let profile = &mut self.config.ai.profiles[i];
                if profile.env.contains_key(name) {
                    return;
                }
                let suggested = presets::harness_of(profile)
                    .and_then(|h| h.env.iter().find(|e| e.name == name))
                    .map_or("", |e| e.value);
                profile
                    .env
                    .insert(name.to_string(), given.unwrap_or(suggested).to_string());
                self.write_ai_profiles();
            }
            AiTarget::ProfileAgentModel => {
                let profile = &mut self.config.ai.profiles[i];
                if let Some(h) = presets::harness_of(profile) {
                    let model = Some(value.trim()).filter(|m| !m.is_empty());
                    let next = h.with_model(profile.active_args(), model);
                    *profile.active_args_mut() = next;
                    self.write_ai_profiles();
                }
            }
            AiTarget::ProfileOptions => {
                let profile = &mut self.config.ai.profiles[i];
                let option = presets::harness_of(profile)
                    .zip(value.parse::<usize>().ok())
                    .and_then(|(h, n)| h.options_for(profile.run_mode).get(n));
                if let Some(option) = option {
                    let next = presets::Harness::toggle_option(option, profile.active_args());
                    *profile.active_args_mut() = next;
                    self.write_ai_profiles();
                }
            }
            AiTarget::ProfilePreset => {
                if let Some(preset) = presets::provider_named(value) {
                    preset.apply(&mut self.config.ai.profiles[i]);
                    self.write_ai_profiles();
                    self.ai_probe_keys();
                }
            }
            AiTarget::ProfileModel => self.ai_commit_text(target, value),
            _ => {}
        }
    }

    pub(super) fn ai_commit_bool(&mut self, target: AiTarget, value: bool) {
        match target {
            AiTarget::Review(scope, row) => self.review_commit_bool(scope, row, value),
            AiTarget::Privacy(row) => self.privacy_commit_bool(row, value),
            _ => {}
        }
    }

    pub(super) fn ai_commit_text(&mut self, target: AiTarget, draft: &str) {
        use gitpoulpi_ai::presets;
        let value = draft.trim().to_string();
        match target {
            AiTarget::Review(scope, _) => self.review_commit_text(scope, draft),
            AiTarget::ProfileName
            | AiTarget::ProfileModel
            | AiTarget::ProfileBaseUrl
            | AiTarget::ProfileCommand
            | AiTarget::ProfileArgs
            | AiTarget::ProfileTimeout => {
                let Some(i) = self.selected_ai_profile() else {
                    return;
                };
                let profile = &mut self.config.ai.profiles[i];
                match target {
                    AiTarget::ProfileName => profile.name = value,
                    AiTarget::ProfileModel => profile.model = value,
                    AiTarget::ProfileBaseUrl => profile.base_url = value,
                    AiTarget::ProfileCommand => {
                        // The type is kept when it is set; otherwise a
                        // command naming a known agent gives it.
                        let old = presets::harness_of(profile).map(|h| h.command);
                        profile.command = value;
                        if profile.harness.trim().is_empty() {
                            match (presets::harness_for(&profile.command), old) {
                                (Some(h), _) => {
                                    profile.harness = h.command.into();
                                    if old != Some(h.command) {
                                        reset_args_for(profile, h);
                                    }
                                }
                                // An alias: keep the type the old command
                                // implied (a profile from before the
                                // `harness` key).
                                (None, Some(old)) => profile.harness = old.into(),
                                (None, None) => {}
                            }
                        }
                    }
                    AiTarget::ProfileTimeout => {
                        // Seconds; empty is the mode's default.
                        match value.parse::<u64>() {
                            Ok(n) => profile.timeout_secs = n,
                            Err(_) if value.is_empty() => profile.timeout_secs = 0,
                            Err(_) => {
                                self.message = Some(msg::TIMEOUT_NOT_A_NUMBER.get().to_string());
                                return;
                            }
                        }
                    }
                    _ => *profile.active_args_mut() = split_args(draft),
                }
                self.write_ai_profiles();
            }
            AiTarget::ProfileEnv(n) => {
                let value = draft.to_string();
                let Some(i) = self.selected_ai_profile() else {
                    return;
                };
                let profile = &mut self.config.ai.profiles[i];
                let Some(name) = profile.env.keys().nth(n).cloned() else {
                    return;
                };
                // A hidden (secret) value is typed from scratch: empty
                // leaves it alone.
                let secret = presets::env_value_is_secret(&name, &profile.env[&name]);
                if secret && value.is_empty() {
                    return;
                }
                profile.env.insert(name, value);
                self.write_ai_profiles();
            }
            AiTarget::ProfileKey => {
                let Some(i) = self.selected_ai_profile() else {
                    return;
                };
                if value.is_empty() {
                    self.message = Some(msg::EMPTY_KEY.get().to_string());
                    return;
                }
                let profile = &mut self.config.ai.profiles[i];
                if profile.token_ref.is_empty() {
                    profile.token_ref = AiProfile::token_ref_for(&profile.id);
                    self.write_ai_profiles();
                }
                let token_ref = self.config.ai.profiles[i].token_ref.clone();
                self.ai.key_present.insert(token_ref.clone(), None);
                let tokens = std::sync::Arc::clone(&self.tokens);
                let tx = self.ai.sender();
                self.message = Some(msg::STORING_KEY.get().to_string());
                std::thread::spawn(move || {
                    let result = tokens.store(&token_ref, &value).map_err(|e| e.to_string());
                    let _ = tx.send_notify(AiEvent::KeyStored { token_ref, result });
                });
            }
            _ => {}
        }
    }

    /// Enter on a row of this page that is not a plain edit. `true` when
    /// handled here.
    pub(super) fn ai_activate(&mut self, target: AiTarget) -> bool {
        match target {
            // A new key is typed from scratch: the row shows "(stored)",
            // which must not become the draft.
            AiTarget::ProfileKey => {
                self.edit = Some((self.field, String::new(), false));
                true
            }
            // A hidden value is typed from scratch, never shown.
            AiTarget::ProfileEnv(n) => {
                use gitpoulpi_ai::presets;
                let hidden = self
                    .selected_ai_profile()
                    .and_then(|i| self.config.ai.profiles[i].env.iter().nth(n))
                    .is_some_and(|(k, v)| presets::env_value_is_secret(k, v));
                if hidden {
                    self.edit = Some((self.field, String::new(), false));
                    true
                } else {
                    false
                }
            }
            AiTarget::Instructions(scope, section) => {
                let text = self.instructions_of(scope, section);
                self.ai.prompt = Some(PromptEdit {
                    scope,
                    section,
                    field: {
                        let mut field = crate::text_edit::FieldEditor::default();
                        field.caret_mut(());
                        field
                    },
                    page_rows: 5,
                    close: super::editor_close::EditorClose::new(&text),
                    text,
                    vars: prompt_vars::VarUi::new(),
                    chips_focused: false,
                    chip: 0,
                    note: None,
                });
                true
            }
            _ => false,
        }
    }

    /// The palette's "AI: Edit Profile": Settings > AI on the "Edit
    /// profile" row, its list open.
    pub(crate) fn open_ai_profile_list(&mut self) {
        self.section = PrefSection::ALL
            .iter()
            .position(|s| *s == PrefSection::Ai)
            .unwrap_or(0);
        self.remember_sub_tab(PrefSection::Ai, SubTab::List);
        self.focus = PrefFocus::List;
        self.strip = false;
        self.sync_list();
    }

    /// The palette's "Insert Prompt Variable": Settings > AI, the global
    /// instruction editor open, the Variables dropdown showing.
    pub(crate) fn open_prompt_variables(&mut self) {
        self.section = PrefSection::ALL
            .iter()
            .position(|s| *s == PrefSection::Ai)
            .unwrap_or(0);
        self.remember_sub_tab(PrefSection::Ai, SubTab::General);
        self.focus = PrefFocus::Fields;
        let target = AiTarget::Instructions(AiScope::Global, AiSection::Global);
        if let Some(i) = self
            .fields()
            .iter()
            .position(|f| f.target == Target::Ai(target))
        {
            self.field = i;
        }
        self.ai_activate(target);
        if let Some(edit) = self.ai.prompt.as_mut() {
            edit.vars.combo.open();
        }
    }

    fn instructions_of(&self, scope: AiScope, section: AiSection) -> String {
        match (scope, section) {
            (AiScope::Repo, AiSection::Global) => {
                self.repo_state.preferences.ai_instructions.text.clone()
            }
            (AiScope::Repo, AiSection::Action(a)) => self.repo_override(a).instructions,
            (_, AiSection::Global) => self.config.ai.global_instructions.clone(),
            (_, AiSection::Action(a)) => self.config.ai.actions.get(a).instructions.clone(),
            (_, AiSection::HostingBase) => self.config.ai.hosting_reply.base_prompt.clone(),
        }
    }

    /// This repository's override for `action` (default: nothing set).
    /// Reads through the legacy `ai_presets` choice, like the resolver.
    fn repo_override(&self, action: AiAction) -> gitpoulpi_config::AiActionSettings {
        self.repo_state
            .preferences
            .ai_override(action.key(), &self.config.ai)
            .unwrap_or_default()
    }

    /// Changes this repository's override for `action` and saves
    /// `repo-state.toml`; an override left empty is removed, so "inherit"
    /// is the absence of a table.
    fn edit_repo_override(
        &mut self,
        action: AiAction,
        change: impl FnOnce(&mut gitpoulpi_config::AiActionSettings),
    ) {
        let mut o = self.repo_override(action);
        change(&mut o);
        let prefs = &mut self.repo_state.preferences;
        prefs.ai_presets.remove(action.key());
        if o.is_empty() {
            prefs.ai_overrides.remove(action.key());
        } else {
            prefs.ai_overrides.insert(action.key().to_string(), o);
        }
        self.write_repo_state();
    }

    /// Saves a section's instructions to the scope's own store: the one
    /// save path of the chips and the overlay editor.
    fn write_instructions(&mut self, scope: AiScope, section: AiSection, text: String) {
        match (scope, section) {
            (AiScope::Repo, AiSection::Global) => {
                self.repo_state.preferences.ai_instructions.text = text;
                self.write_repo_state();
            }
            (AiScope::Repo, AiSection::Action(a)) => {
                self.edit_repo_override(a, |o| o.instructions = text)
            }
            _ => {
                self.write_global(&section.key(), &text);
            }
        }
    }

    /// The profile list of the page (`list_page::list_spec`).
    pub(super) fn ai_list_spec(&self) -> list_editor::ListSpec {
        use list_editor::{ListItem, Mark};
        let items = self
            .config
            .ai
            .profiles
            .iter()
            .map(|p| ListItem {
                label: p.display_name().to_string(),
                sub: Some(provider_label(p.kind).to_string()),
                mark: if p.id == self.config.ai.default_profile {
                    Mark::Active
                } else {
                    Mark::None
                },
                pristine: p.name == msg::NEW_PROFILE.english()
                    && p.model.is_empty()
                    && p.env.is_empty()
                    && p.token_ref.is_empty(),
                ..ListItem::default()
            })
            .collect();
        list_editor::ListSpec {
            title: msg::PROFILES_HEADING.get().to_string(),
            items,
            duplicate: true,
            reorder: true,
            min_len: 1,
            ..list_editor::ListSpec::default()
        }
    }

    /// Applies a list operation to `ai.profiles`.
    pub(super) fn ai_profile_op(&mut self, op: list_editor::ListOp) {
        use list_editor::ListOp;
        let len = self.config.ai.profiles.len();
        match op {
            ListOp::Add(_) => {
                let id = self.config.ai.new_profile_id("profile");
                self.config.ai.profiles.push(AiProfile::new(
                    id,
                    // Stored in config.toml: always the English name.
                    msg::NEW_PROFILE.english(),
                    AiProviderKind::OpenAi,
                ));
                self.ai.profile = len;
            }
            ListOp::Duplicate(i) if i < len => {
                let mut copy = self.config.ai.profiles[i].clone();
                copy.id = self.config.ai.new_profile_id(&copy.id);
                copy.name =
                    gitpoulpi_i18n::msg::settings_profiles::duplicate_name(copy.display_name());
                // The key belongs to the original: the copy has none until
                // one is typed.
                copy.token_ref = String::new();
                self.config.ai.profiles.insert(i + 1, copy);
                self.ai.profile = i + 1;
            }
            ListOp::Remove(i) if i < len && len > 1 => {
                let removed = self.config.ai.profiles[i].clone();
                let had_default = self.config.ai.default_profile == removed.id;
                let named_it: Vec<AiAction> = AiAction::ALL
                    .into_iter()
                    .filter(|a| self.config.ai.actions.get(*a).profile == removed.id)
                    .collect();
                self.config.ai.remove_profile(&removed.id);
                self.ai.profile = i.saturating_sub(1);
                self.write_ai_profiles();
                if had_default {
                    self.write_global("ai.default_profile", "");
                }
                for action in named_it {
                    self.write_global(&format!("ai.actions.{}.profile", action.key()), "");
                }
                if !removed.token_ref.is_empty() {
                    let tokens = std::sync::Arc::clone(&self.tokens);
                    std::thread::spawn(move || {
                        let _ = tokens.delete(&removed.token_ref);
                    });
                }
                self.field = self.first_selectable();
                return;
            }
            ListOp::Move { from, to } if from < len && to < len => {
                let item = self.config.ai.profiles.remove(from);
                self.config.ai.profiles.insert(to, item);
                self.ai.profile = to;
            }
            _ => return,
        }
        self.write_ai_profiles();
        self.ai_probe_keys();
    }

    pub(super) fn ai_run_button(&mut self, button: AiButton) {
        match button {
            AiButton::EnvEdit(n) => {
                let Some(i) = self.selected_ai_profile() else {
                    return;
                };
                let Some((name, value)) = self.config.ai.profiles[i].env.iter().nth(n) else {
                    return;
                };
                // A hidden value is typed from scratch, never shown.
                let secret = gitpoulpi_ai::presets::env_value_is_secret(name, value);
                let draft = if secret { String::new() } else { value.clone() };
                self.ai.env_edit = Some((n, self.field));
                self.edit = Some((self.field, draft, false));
            }
            AiButton::EnvRemove(n) => {
                let Some(i) = self.selected_ai_profile() else {
                    return;
                };
                let profile = &mut self.config.ai.profiles[i];
                if let Some(name) = profile.env.keys().nth(n).cloned() {
                    profile.env.remove(&name);
                    self.write_ai_profiles();
                }
            }
            AiButton::ResetReview => self.review_reset_repo(),
            AiButton::ShowAllPreviews => self.privacy_show_all(),
            AiButton::BrowseExtra(scope) => self.review_open_browse(scope),
            AiButton::ResetRepo(action) => {
                self.edit_repo_override(action, |o| *o = Default::default());
                self.message = Some(msg::back_to_global(AiSection::Action(action).title()));
            }
            AiButton::ToggleLegacy => self.ai.legacy_open = !self.ai.legacy_open,
            AiButton::DeleteLegacy => {
                self.config.ai.legacy_presets.clear();
                self.write_ai_list(
                    "ai.legacy_presets",
                    &Vec::<gitpoulpi_config::AiPreset>::new(),
                );
                self.message = Some(msg::PRESETS_DELETED.get().to_string());
                self.field = self.first_selectable();
            }
            AiButton::OpenAgent => self.ai_open_agent(),
            AiButton::Test => {
                let Some(i) = self.selected_ai_profile() else {
                    return;
                };
                let profile = self.config.ai.profiles[i].clone();
                let repo_root = self.repo_root.clone();
                self.ai.tests.insert(profile.id.clone(), TestState::Running);
                let tokens = std::sync::Arc::clone(&self.tokens);
                let tx = self.ai.sender();
                std::thread::spawn(move || {
                    let key = if profile.kind.needs_key() && !profile.token_ref.is_empty() {
                        tokens.load(&profile.token_ref).ok().flatten()
                    } else {
                        None
                    };
                    let result = match tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                    {
                        Ok(runtime) => runtime.block_on(async {
                            let client = gitpoulpi_ai::client_for_network_in(
                                &profile,
                                key,
                                Some(&repo_root),
                                &gitpoulpi_ai::Privacy::default(),
                            )?;
                            gitpoulpi_ai::test_connection(&*client).await
                        }),
                        Err(e) => Err(gitpoulpi_ai::Error::Invalid(e.to_string())),
                    }
                    .map_err(|e| e.to_string());
                    let _ = tx.send_notify(AiEvent::Tested {
                        profile_id: profile.id,
                        result,
                    });
                });
            }
        }
    }

    /// Applies what the key-presence probe, a key store and a connection
    /// test have answered since the last frame.
    pub(super) fn poll_ai(&mut self) {
        if let Some(rx) = &self.ai.presence_rx {
            let answers: Vec<(String, bool)> = std::iter::from_fn(|| rx.try_recv().ok()).collect();
            for (token_ref, present) in answers {
                self.ai.key_present.insert(token_ref, Some(present));
            }
        }
        let events: Vec<AiEvent> = match &self.ai.rx {
            Some(rx) => std::iter::from_fn(|| rx.try_recv().ok()).collect(),
            None => Vec::new(),
        };
        for event in events {
            match event {
                AiEvent::KeyStored { token_ref, result } => match result {
                    Ok(()) => {
                        self.ai.key_present.insert(token_ref, Some(true));
                        self.message = Some(msg::KEY_SAVED.get().to_string());
                    }
                    Err(why) => {
                        self.ai.key_present.insert(token_ref, Some(false));
                        self.message = Some(msg::key_not_saved(why));
                    }
                },
                AiEvent::Tested { profile_id, result } => {
                    let state = match result {
                        Ok(()) => TestState::Ok,
                        Err(why) => TestState::Failed(why),
                    };
                    self.ai.tests.insert(profile_id, state);
                }
            }
        }
    }

    pub(super) fn ai_prompt_key(&mut self, key: KeyEvent) -> bool {
        let Some(edit) = self.ai.prompt.as_mut() else {
            return true;
        };
        if edit.close.asking() {
            if edit.close.key(key, &edit.text) == Act::Close {
                self.ai_prompt_cancel();
            }
            return true;
        }
        edit.note = None;
        if edit.vars_key(&key) {
            return true;
        }
        if edit.chips_key(&key) {
            return true;
        }
        match edit.close.key(key, &edit.text) {
            Act::Close => {
                self.ai_prompt_cancel();
                return true;
            }
            Act::Spent => return true,
            Act::Pass => {}
        }
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Char('s' | 'S' | '\u{13}') if ctrl || key.code == KeyCode::Char('\u{13}') => {
                self.ai_prompt_save()
            }
            // Tab (or Shift+Tab) moves to the Quick insert row above.
            KeyCode::Tab | KeyCode::BackTab if !ctrl => {
                edit.chips_focused = !edit.section.chips().is_empty();
            }
            // Tab (or Shift+Tab) moves to the Quick insert row above.
            KeyCode::Tab | KeyCode::BackTab if !ctrl => {
                edit.chips_focused = !edit.section.chips().is_empty();
            }
            KeyCode::Enter => {
                edit.field.caret_mut(()).insert(&mut edit.text, "\n");
                edit.after_edit();
            }
            // A page of rows up / down, selecting with Shift: the arrow
            // key, repeated.
            KeyCode::PageUp | KeyCode::PageDown => {
                let arrow = if key.code == KeyCode::PageUp {
                    KeyCode::Up
                } else {
                    KeyCode::Down
                };
                let step = KeyEvent::new(arrow, key.modifiers);
                for _ in 0..edit.page_rows.saturating_sub(1).max(1) {
                    edit.field.caret_mut(()).on_key(&mut edit.text, &step, true);
                }
                edit.after_edit();
            }
            _ => {
                let done = edit.field.caret_mut(()).on_key(&mut edit.text, &key, true);
                if let KeyEdit::Copy(s) | KeyEdit::Cut(s) = done {
                    edit.note = Some(crate::clipboard::copy_shared(
                        gitpoulpi_i18n::msg::input::WHAT_SELECTION.get(),
                        &s,
                    ));
                }
                edit.after_edit();
            }
        }
        true
    }

    /// Esc / the Cancel button: close the editor, discarding the text.
    fn ai_prompt_cancel(&mut self) {
        self.ai.prompt = None;
        self.ai.prompt_hits.clear();
        self.ai.chip_hits.clear();
    }

    /// Ctrl+S / the Save button: write the text and close the editor.
    fn ai_prompt_save(&mut self) {
        self.ai.prompt_hits.clear();
        self.ai.chip_hits.clear();
        self.ai.chip_hits.clear();
        if let Some(edit) = self.ai.prompt.take() {
            self.write_instructions(edit.scope, edit.section, edit.text);
        }
    }

    /// The editor is modal: a left press on Save / Cancel acts, a press
    /// anywhere else (inside or outside the popup) does nothing - it is
    /// never passed to the page behind, and never discards typed text.
    pub(super) fn ai_prompt_mouse(
        &mut self,
        kind: MouseEventKind,
        col: u16,
        row: u16,
        shift: bool,
    ) {
        let event = crossterm::event::MouseEvent {
            kind,
            column: col,
            row,
            modifiers: KeyModifiers::NONE,
        };
        // The confirmation is topmost, then the dropdown and the
        // autocomplete popup above the editor.
        if let Some(edit) = self.ai.prompt.as_mut() {
            if edit.close.asking() {
                if edit.close.mouse(event, &edit.text) == Act::Close {
                    self.ai_prompt_cancel();
                }
                return;
            }
        }
        if self.ai.vars_mouse(kind, col, row) {
            return;
        }
        let chip_hit = if kind == MouseEventKind::Down(MouseButton::Left) {
            crate::dialog_kit::hit_at(&self.ai.chip_hits, col, row)
        } else {
            None
        };
        let Some(edit) = self.ai.prompt.as_mut() else {
            return;
        };
        if let Some(chip) = chip_hit {
            edit.note = None;
            edit.insert_chip(chip);
            return;
        }
        // The text: a press places the caret (Shift extends), a drag
        // selects, the wheel scrolls.
        if !edit.close.asking()
            && edit
                .field
                .area_mouse((), &edit.text, kind, (col, row), shift)
        {
            edit.after_edit();
            return;
        }
        let button = if kind == MouseEventKind::Down(MouseButton::Left) {
            crate::dialog_kit::hit_at(&self.ai.prompt_hits, col, row)
        } else {
            None
        };
        if button.is_none() {
            match edit.close.mouse(event, &edit.text) {
                Act::Close => return self.ai_prompt_cancel(),
                Act::Spent | Act::Pass => return,
            }
        }
        match button {
            Some(PromptButton::Save) => self.ai_prompt_save(),
            Some(PromptButton::Undo) => {
                edit.undo();
            }
            Some(PromptButton::Redo) => {
                edit.redo();
            }
            Some(PromptButton::Cancel) if edit.close.cancel_button(&edit.text) == Act::Close => {
                self.ai_prompt_cancel()
            }
            _ => {}
        }
    }

    pub(super) fn ai_prompt_paste(&mut self, text: &str) {
        if let Some(edit) = self.ai.prompt.as_mut() {
            edit.field.caret_mut(()).insert(&mut edit.text, text);
            edit.after_edit();
        }
    }

    /// Draws the editor over the page, when it is open.
    pub(super) fn render_ai_prompt(&mut self, frame: &mut Frame, area: Rect) {
        self.ai.prompt_hits.clear();
        self.ai.chip_hits.clear();
        let mouse = self.mouse_pos;
        let Some(edit) = self.ai.prompt.as_mut() else {
            return;
        };
        edit.vars.ac_hits.clear();
        let hovered = |r: Rect| mouse.is_some_and(|(c, row)| r.contains((c, row).into()));
        let width = (area.width * 4 / 5).max(30).min(area.width);
        let height = (area.height * 3 / 5).max(8).min(area.height);
        let rect = WIN.place(
            area,
            Rect::new(
                area.x + (area.width - width) / 2,
                area.y + (area.height - height) / 2,
                width,
                height,
            ),
        );
        let title = msg::editor_title(
            if edit.scope == AiScope::Repo && edit.section == AiSection::Global {
                msg::REPO_INSTRUCTIONS_HEADING.get()
            } else {
                edit.section.title()
            },
        );
        edit.close
            .framed(frame.buffer_mut(), rect, &title, &WIN, true);
        super::editor_close::footer(
            frame.buffer_mut(),
            rect,
            &msg::editor_footer(
                crate::dialog_kit::ctrl_key('s'),
                crate::dialog_kit::key_name(crate::keymap::chord::Key::Enter),
                crate::dialog_kit::key_name(crate::keymap::chord::Key::Tab),
            ),
        );
        let inner = rect.inner(ratatui::layout::Margin::new(1, 1));
        // The last inner row is the button row (Variables, Save, Cancel),
        // the one above it the tooltip / warning line; the text gets the
        // rest.
        let (inner, hint_y, footer_y) = if inner.height >= 5 {
            (
                Rect::new(inner.x, inner.y, inner.width, inner.height - 2),
                Some(inner.bottom() - 2),
                Some(inner.bottom() - 1),
            )
        } else if inner.height >= 3 {
            (
                Rect::new(inner.x, inner.y, inner.width, inner.height - 1),
                None,
                Some(inner.bottom() - 1),
            )
        } else {
            (inner, None, None)
        };
        // The Quick insert row on top, the text below it.
        let (chip_rows, chip_hits) = edit.draw_chips(frame.buffer_mut(), inner, hovered);
        self.ai.chip_hits = chip_hits;
        let inner = Rect::new(
            inner.x,
            inner.y + chip_rows,
            inner.width,
            inner.height - chip_rows,
        );
        let Some(edit) = self.ai.prompt.as_mut() else {
            return;
        };
        let look = crate::text_field::EditLook::new(
            Style::default()
                .fg(th().status.warning)
                .bg(crate::input_focus_bg()),
            "\u{2588}",
        );
        let text_width = inner.width.saturating_sub(1) as usize;
        let lines = crate::text_field::edit_lines(
            &edit.text,
            edit.field.caret(()),
            text_width,
            inner.height as usize,
            true,
            look,
        );
        frame.render_widget(Paragraph::new(lines), inner);
        // The field's click rect: the same width `edit_lines` wrapped to.
        edit.page_rows = inner.height as usize;
        edit.field.begin_frame();
        edit.field.drawn(
            (),
            Rect::new(
                inner.x,
                inner.y,
                inner.width.saturating_sub(1),
                inner.height,
            ),
            true,
        );

        // The `${` autocomplete, hanging under the caret.
        let completion = edit.completion();
        if let Some((_, matches)) = &completion {
            let (crow, ccol) = crate::text_field::caret_cell(
                &edit.text,
                &edit.field.caret(()).cloned().unwrap_or_default(),
                text_width,
                inner.height as usize,
            );
            let rows = matches.len().min(6) as u16;
            let entries: Vec<String> = matches
                .iter()
                .map(|n| format!("${{{n}}}  {}", prompt_vars::description(n)))
                .collect();
            let want = entries
                .iter()
                .map(|e| crate::display_width(e))
                .max()
                .unwrap_or(10) as u16
                + 4;
            let pw = want.min(inner.width).max(12);
            let px = (inner.x + ccol as u16).min(inner.right().saturating_sub(pw));
            let below = inner.y + crow as u16 + 1;
            let py = if below + rows + 2 <= inner.bottom() || crow < rows as usize + 2 {
                below
            } else {
                (inner.y + crow as u16).saturating_sub(rows + 2)
            };
            let popup = Rect::new(px, py, pw, rows + 2).intersection(frame.area());
            if popup.height >= 3 {
                frame.render_widget(Clear, popup);
                let blk = Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(th().ui.accent))
                    .style(Style::default().bg(th().ui.panel));
                let body = blk.inner(popup);
                frame.render_widget(blk, popup);
                let cursor = edit.vars.ac_cursor.min(matches.len() - 1);
                let buf = frame.buffer_mut();
                for (i, entry) in entries.iter().enumerate().take(body.height as usize) {
                    let band = Rect::new(body.x, body.y + i as u16, body.width, 1);
                    let bg = if i == cursor {
                        th().ui.selection
                    } else if hovered(band) {
                        th().ui.hover
                    } else {
                        th().ui.panel
                    };
                    crate::dialog_kit::fill(buf, band, Style::default().bg(bg));
                    crate::dialog_kit::put(
                        buf,
                        band.x,
                        band.y,
                        entry,
                        band.width,
                        Style::default().fg(th().ui.text).bg(bg),
                    );
                    edit.vars.ac_hits.push((band, i));
                }
            }
        }

        if let Some(y) = footer_y {
            use crate::dialog_kit::{self, ButtonState};
            let cancel_label = msg::BTN_CANCEL.get();
            let save_label = msg::BTN_SAVE.get();
            let cancel_w = dialog_kit::button_width(cancel_label);
            let save_w = dialog_kit::button_width(save_label);
            let right = inner.right();
            let cancel_x = right.saturating_sub(cancel_w);
            let save_x = cancel_x.saturating_sub(save_w + 1);
            // Undo and Redo sit left of Save: the buttons for Ctrl+Z / Ctrl+Y.
            let undo_label = gitpoulpi_i18n::msg::toolbar::UNDO.get();
            let redo_label = gitpoulpi_i18n::msg::toolbar::REDO.get();
            let undo_w = dialog_kit::button_width(undo_label);
            let redo_w = dialog_kit::button_width(redo_label);
            let redo_x = save_x.saturating_sub(redo_w + 1);
            let undo_x = redo_x.saturating_sub(undo_w + 1);
            let (can_undo, can_redo) = edit.field.caret(()).map_or((false, false), |c| {
                (c.can_undo(&edit.text), c.can_redo(&edit.text))
            });
            let buf = frame.buffer_mut();
            let at = |x: u16, w: u16| Rect::new(x, y, w, 1);
            let state = |r: Rect, primary: bool| {
                let h = hovered(r);
                if primary {
                    ButtonState::Primary {
                        hovered: h,
                        focused: false,
                    }
                } else {
                    ButtonState::Normal {
                        hovered: h,
                        focused: false,
                    }
                }
            };
            let save =
                dialog_kit::button(buf, save_x, y, save_label, state(at(save_x, save_w), true));
            let cancel = dialog_kit::button(
                buf,
                cancel_x,
                y,
                cancel_label,
                state(at(cancel_x, cancel_w), false),
            );
            let history_state = |r: Rect, on: bool| {
                if on {
                    ButtonState::Normal {
                        hovered: hovered(r),
                        focused: false,
                    }
                } else {
                    ButtonState::Disabled
                }
            };
            let undo = dialog_kit::button(
                buf,
                undo_x,
                y,
                undo_label,
                history_state(at(undo_x, undo_w), can_undo),
            );
            let redo = dialog_kit::button(
                buf,
                redo_x,
                y,
                redo_label,
                history_state(at(redo_x, redo_w), can_redo),
            );
            // The Variables dropdown, left of the buttons.
            let vars_w = (crate::display_width(msg::VARIABLES_BUTTON.get()) as u16 + 4)
                .min(undo_x.saturating_sub(inner.x + 1));
            let field = Rect::new(inner.x, y, vars_w, 1);
            if hint_y.is_some() && vars_w >= 6 {
                let h = hovered(field);
                edit.vars.combo.render_field(buf, field, false, h);
            } else {
                edit.vars.combo.close();
                edit.vars.combo.set_anchor(Rect::default());
            }
            // The tooltip / warning line.
            if let Some(hy) = hint_y {
                let unknown = edit.unknown();
                let describing = edit
                    .vars
                    .combo
                    .hovered_value()
                    .or_else(|| edit.vars.combo.highlighted_value())
                    .map(str::to_string)
                    .or_else(|| {
                        completion
                            .as_ref()
                            .map(|(_, m)| m[edit.vars.ac_cursor.min(m.len() - 1)].to_string())
                    });
                let (text, style) = if let Some(name) = describing {
                    (
                        format!("${{{name}}}: {}", prompt_vars::hint(&name)),
                        Style::default().fg(th().ui.muted),
                    )
                } else if let Some(note) = edit.note.clone() {
                    (note, Style::default().fg(th().status.warning))
                } else if !unknown.is_empty() {
                    let names = unknown
                        .iter()
                        .map(|n| format!("${{{n}}}"))
                        .collect::<Vec<_>>()
                        .join(", ");
                    (
                        msg::unknown_variables(names),
                        Style::default().fg(th().status.warning),
                    )
                } else if hovered(field) || edit.vars.combo.is_open() {
                    (
                        msg::variables_tooltip(dialog_kit::ctrl_key('t')),
                        Style::default().fg(th().ui.muted),
                    )
                } else {
                    (String::new(), Style::default())
                };
                dialog_kit::put(buf, inner.x, hy, &text, inner.width, style);
            }
            self.ai.prompt_hits = vec![
                (save, PromptButton::Save),
                (cancel, PromptButton::Cancel),
                (undo, PromptButton::Undo),
                (redo, PromptButton::Redo),
            ];
        }
        if let Some(edit) = self.ai.prompt.as_mut() {
            edit.vars.combo.render_popup(frame.buffer_mut(), area);
            edit.close.render(frame.buffer_mut());
        }
    }
}

mod fields;
mod prompt_chips;
mod prompt_vars;

/// The editor resizes: its text area takes the whole rect, so it grows and
/// shrinks with it.
const WIN: crate::dialog_kit::Win = crate::dialog_kit::Win::resizable("settings_ai_editor", 40, 10);

#[cfg(test)]
mod tests;
