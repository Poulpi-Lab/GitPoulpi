//! User-defined named action sequences: the stored shape, the step
//! vocabulary, validation, the name templates and where a sequence lives.
//!
//! Pure data: nothing here runs a step or draws anything. The runner, the
//! keymap and Settings are adapters around it. A sequence is a list of
//! palette command ids (`StepSpec::run`, the ids pinned by
//! `command_ids_are_pinned` in `gitpoulpi-tui`) with a per-step pause mode
//! and a few options.
//!
//! ```toml
//! [[sequences]]
//! id = "backup"
//! name = "Backup"
//! scope = "installation"        # or "machine"; "repo" lives in the repository
//! owner = "3f9a0c1d2b4e5f60"    # the installation id, set with scope = installation
//!
//! [[sequences.steps]]
//! run = "stage_all"
//!
//! [[sequences.steps]]
//! run = "ai_generate_commit_message"
//! pause = "always"
//! ```
//!
//! **Unknown data is kept.** A key this build does not know (on the
//! sequence or on a step) and a step whose `run` this build does not know
//! survive a load and a save; `validate` flags the step, it never drops it.
//!
//! **Scope.** `installation` (the default) is seen only by the installation
//! whose id is `owner`; `machine` by every installation sharing the
//! `config.toml`; `repo` lives in `<repo>/.gitpoulpi/sequences.toml`, is seen
//! only in that repository and only once it is trusted (a committed file is
//! someone else's commands: see [`repo_sequences_hash`]).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{secure_fs, Config, Error, Result};

/// Most steps one sequence may have.
pub const MAX_STEPS: usize = 32;
/// Longest slug.
pub const MAX_ID_LEN: usize = 48;
/// The `[keybindings.<context>]` that holds a sequence's key by its id.
pub const KEYBINDINGS_CONTEXT: &str = "sequence";
/// Slugs a sequence cannot take: the runner's own key actions.
pub const RESERVED_IDS: [&str; 2] = ["cancel", "continue"];
/// A repository's sequences, relative to its root.
pub const REPO_FILE: &str = ".gitpoulpi/sequences.toml";

crate::schema::config_enum! {
    /// Who sees a sequence.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum Scope {
        /// Only the installation whose id is the sequence's `owner`.
        #[default]
        Installation => "installation",
        /// Every installation sharing this `config.toml`.
        Machine => "machine",
        /// Only the repository holding `.gitpoulpi/sequences.toml`.
        Repo => "repo",
    }
}

crate::schema::config_enum! {
    /// Whether a step stops for the user.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum PauseMode {
        /// What the step's kind does by default: pauses on its usual dialog
        /// when it needs input.
        #[default]
        Default => "default",
        /// Stop before the step and ask to continue.
        Always => "always",
        /// Never stop: the step's options must give it everything it needs.
        Never => "never",
    }
}

impl PauseMode {
    fn is_default(&self) -> bool {
        *self == PauseMode::Default
    }
}

/// Where a visible sequence comes from (the same three places as [`Scope`]).
pub type Origin = Scope;

/// One step: a palette command and how to run it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StepSpec {
    /// A palette command id. Not checked on load: an id this build does not
    /// know is kept and flagged by [`validate`].
    pub run: String,
    #[serde(default, skip_serializing_if = "PauseMode::is_default")]
    pub pause: PauseMode,
    /// The step's options, by [`OptionSpec::key`].
    #[serde(default, skip_serializing_if = "toml::Table::is_empty")]
    pub options: toml::Table,
    /// Keys a newer build wrote, kept.
    #[serde(flatten)]
    pub extra: toml::Table,
}

impl StepSpec {
    pub fn new(run: &str) -> Self {
        StepSpec {
            run: run.to_string(),
            ..StepSpec::default()
        }
    }

    pub fn with_pause(mut self, pause: PauseMode) -> Self {
        self.pause = pause;
        self
    }

    pub fn with_option(mut self, key: &str, value: impl Into<toml::Value>) -> Self {
        self.options.insert(key.to_string(), value.into());
        self
    }
}

/// One named sequence.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SequenceDef {
    /// Stable slug: the key in `[keybindings.sequence]` and the CLI name.
    pub id: String,
    /// The user's own text; never translated.
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub scope: Scope,
    /// The installation id, set with `scope = "installation"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default)]
    pub steps: Vec<StepSpec>,
    /// Keys a newer build wrote, kept.
    #[serde(flatten)]
    pub extra: toml::Table,
}

// ------------------------------------------------------------ vocabulary

/// How a step behaves, for the runner and for guidance in Settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepClass {
    /// Returns at once, changes nothing in the repository.
    Instant,
    /// Changes the repository, asks nothing by default.
    Mutation,
    /// Talks to a remote.
    Network,
    /// Calls the configured AI.
    Ai,
    /// Opens a dialog to get its input.
    Dialog,
}

/// The kind of value an option takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionKind {
    Text,
    /// Text that goes through [`expand_template`].
    Template,
    Bool,
    Choice(&'static [&'static str]),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptionSpec {
    pub key: &'static str,
    pub kind: OptionKind,
    /// The step cannot be `pause = "never"` without it: it is the answer the
    /// dialog would have asked for.
    pub required_when_never: bool,
}

/// One allowed step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepKind {
    /// The palette command id.
    pub id: &'static str,
    pub class: StepClass,
    /// `pause` means something: the step can stop for input.
    pub pausable: bool,
    /// `pause = "never"` is refused (the step always asks first).
    pub pause_forced: bool,
    pub options: &'static [OptionSpec],
    /// One English line, for the AI vocabulary and Settings.
    pub meaning_en: &'static str,
    /// Runs headless (`gitpoulpi sequence run`).
    pub cli: bool,
    /// The cargo feature the command needs, `None` when always built.
    pub requires: Option<&'static str>,
    /// The command is added by a later slice: not yet a palette id, but
    /// known here so presets that use it validate clean.
    pub pending: bool,
}

const fn opt(key: &'static str, kind: OptionKind, required_when_never: bool) -> OptionSpec {
    OptionSpec {
        key,
        kind,
        required_when_never,
    }
}

const NAME: OptionSpec = opt("name", OptionKind::Template, true);
const OPEN_TAB: OptionSpec = opt("open_tab", OptionKind::Bool, false);

const fn kind(
    id: &'static str,
    class: StepClass,
    pausable: bool,
    options: &'static [OptionSpec],
    meaning_en: &'static str,
    cli: bool,
) -> StepKind {
    StepKind {
        id,
        class,
        pausable,
        pause_forced: false,
        options,
        meaning_en,
        cli,
        requires: None,
        pending: false,
    }
}

/// Every command a sequence may run. Ids are the snake-cased `Debug` names
/// of `PaletteCommand` (`stash_pop_latest` style); the TUI checks this list
/// against its catalogue.
pub const STEP_VOCABULARY: &[StepKind] = &[
    kind(
        "stage_all",
        StepClass::Mutation,
        false,
        &[],
        "Stage every change in the working tree",
        true,
    ),
    kind(
        "commit",
        StepClass::Mutation,
        true,
        &[opt("message", OptionKind::Template, false)],
        "Commit the staged changes with the message in the composer",
        true,
    ),
    kind(
        "commit_now",
        StepClass::Mutation,
        true,
        &[opt("message", OptionKind::Template, false)],
        "Commit the staged changes now",
        true,
    ),
    kind(
        "stash_push",
        StepClass::Mutation,
        false,
        &[],
        "Stash the working tree changes",
        true,
    ),
    kind(
        "new_branch",
        StepClass::Dialog,
        true,
        &[NAME],
        "Create a branch at the current commit",
        true,
    ),
    kind(
        "new_tag",
        StepClass::Dialog,
        true,
        &[NAME],
        "Create a tag at the current commit",
        true,
    ),
    kind(
        "fetch",
        StepClass::Network,
        false,
        &[],
        "Fetch from the remotes",
        true,
    ),
    kind(
        "pull",
        StepClass::Network,
        false,
        &[],
        "Pull the current branch with the configured mode",
        true,
    ),
    kind(
        "pull_with_mode_merge",
        StepClass::Network,
        false,
        &[],
        "Pull the current branch, merging when it cannot fast-forward",
        true,
    ),
    kind(
        "pull_with_mode_ff_only",
        StepClass::Network,
        false,
        &[],
        "Pull the current branch, fast-forward only",
        true,
    ),
    kind(
        "pull_with_mode_rebase",
        StepClass::Network,
        false,
        &[],
        "Pull the current branch with rebase",
        true,
    ),
    kind(
        "push",
        StepClass::Network,
        // With `git.push_behavior = ask` the "Push to..." chooser pauses the
        // run; `never` pushes to the `remote` option instead.
        true,
        &[opt("remote", OptionKind::Text, true)],
        "Push the current branch",
        true,
    ),
    kind(
        "push_to_all_targets",
        StepClass::Network,
        false,
        &[],
        "Push the current branch to every push target",
        true,
    ),
    kind(
        "pull_all_branches",
        StepClass::Network,
        false,
        &[],
        "Fast-forward every local branch from its upstream",
        true,
    ),
    kind(
        "push_all_unpushed_tags",
        StepClass::Network,
        false,
        &[],
        "Push every tag no remote has yet",
        true,
    ),
    kind(
        "ai_generate_commit_message",
        StepClass::Ai,
        true,
        &[],
        "Write the commit message with the AI",
        true,
    ),
    kind(
        "ai_compose_commits",
        StepClass::Ai,
        true,
        &[],
        "Let the AI group the changes into several commits",
        false,
    ),
    kind(
        "ai_generate_stash_message",
        StepClass::Ai,
        true,
        &[],
        "Write the stash message with the AI",
        true,
    ),
    kind(
        "create_worktree_from_current_branch",
        StepClass::Dialog,
        true,
        &[NAME, OPEN_TAB],
        "Create a worktree on a new branch from the current one",
        true,
    ),
    kind(
        "create_worktree_choose_branch",
        StepClass::Dialog,
        true,
        &[NAME, opt("branch", OptionKind::Text, true), OPEN_TAB],
        "Create a worktree from a branch you pick",
        true,
    ),
    kind(
        "tab_dir_terminal",
        StepClass::Instant,
        false,
        &[],
        "Open a terminal in the tab's folder",
        false,
    ),
    kind(
        "tab_open_in_default",
        StepClass::Instant,
        false,
        &[],
        "Open the tab's folder in the default application",
        false,
    ),
    kind(
        "tab_open_in",
        StepClass::Dialog,
        true,
        &[opt("label", OptionKind::Text, true)],
        "Open the tab's folder in the Open in entry with this label",
        false,
    ),
    kind(
        "tab_open_agent_here",
        StepClass::Instant,
        true,
        &[opt("profile", OptionKind::Text, true)],
        "Open the AI agent in a terminal in the tab's folder",
        false,
    ),
    kind(
        "open_worktree_in_tab",
        StepClass::Instant,
        false,
        &[],
        "Open the selected worktree in a tab",
        false,
    ),
    StepKind {
        pause_forced: true,
        ..kind(
            "force_push_with_lease",
            StepClass::Network,
            true,
            &[],
            "Force-push the current branch, only if the remote is where you last saw it",
            true,
        )
    },
];

/// The vocabulary entry of a step id.
pub fn step_kind(id: &str) -> Option<&'static StepKind> {
    STEP_VOCABULARY.iter().find(|k| k.id == id)
}

/// Commands that are never steps, with the reason, by exact id. Ids this
/// table does not list fall to the patterns of [`excluded_reason`].
pub const EXCLUDED_REASON: &[(&str, &str)] = &[
    (
        "discard_all_changes",
        "it throws work away and cannot be undone by a repeat",
    ),
    ("stash_drop", "it deletes a stash"),
    ("auto_stash_drop", "it deletes a stash"),
    ("worktree_remove_selected", "it removes a worktree"),
    ("delete_selected_ref", "it deletes a branch or tag"),
    (
        "delete_selected_remote_branch",
        "it deletes a branch on a remote",
    ),
    (
        "delete_selected_branch_and_remote",
        "it deletes a branch on a remote",
    ),
    (
        "delete_selected_tag_from_remote",
        "it deletes a tag on a remote",
    ),
    ("force_push_selected_tag", "it overwrites a tag on a remote"),
    ("take_remote_selected_tag", "it overwrites a tag"),
    ("remove_tab", "it closes something the user chose"),
    ("close_all_tabs", "it closes every tab"),
    (
        "index_lock_remove",
        "it deletes a lock file git is guarding",
    ),
];

/// Why `id` can never be a step: destructive, rewriting history, closing the
/// program or Settings. `None` for an id that is simply not in the
/// vocabulary (unknown) or is allowed.
pub fn excluded_reason(id: &str) -> Option<&'static str> {
    if step_kind(id).is_some() {
        return None;
    }
    if let Some((_, why)) = EXCLUDED_REASON.iter().find(|(k, _)| *k == id) {
        return Some(why);
    }
    const PATTERNS: &[(&str, &str)] = &[
        ("reset_to_", "a reset moves or discards work"),
        ("quit", "it closes the program under the running sequence"),
        ("discard", "it throws work away"),
        ("delete", "it deletes something"),
        ("remove", "it removes something"),
        ("purge", "it purges history data"),
        ("rewrite", "it rewrites history"),
        ("drop", "it drops work"),
        (
            "settings",
            "Settings rows change the configuration, not the repository",
        ),
        (
            "preferences",
            "Settings rows change the configuration, not the repository",
        ),
        (NESTED_PREFIX, NESTED_REASON),
    ];
    PATTERNS
        .iter()
        .find(|(p, _)| id.starts_with(p) || id.contains(&format!("_{p}")))
        .map(|(_, why)| *why)
}

const NESTED_PREFIX: &str = "sequence";
const NESTED_REASON: &str = "a sequence cannot run another sequence";

// ------------------------------------------------------------ validation

/// Why a sequence or one of its steps is not acceptable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SequenceProblem {
    /// The step index; `None` for the sequence as a whole.
    pub step: Option<usize>,
    /// A short code: `empty_name`, `bad_id`, `too_many_steps`, `no_steps`,
    /// `unknown_run`, `excluded_run`, `nested_sequence`, `unknown_option`,
    /// `bad_option_value`, `never_forced`, `never_needs_option`,
    /// `duplicate_id`.
    pub what: String,
    /// One English sentence.
    pub reason: String,
}

fn problem(step: Option<usize>, what: &str, reason: impl Into<String>) -> SequenceProblem {
    SequenceProblem {
        step,
        what: what.to_string(),
        reason: reason.into(),
    }
}

/// Whether `id` is a usable slug: `a-z 0-9 _ -`, not starting with `_` or
/// `-`, at most [`MAX_ID_LEN`] long (it is a TOML bare key and a CLI name).
pub fn is_valid_id(id: &str) -> bool {
    // `cancel` and `continue` are the runner's own key actions
    // (`sequence.cancel`, `sequence.continue`).
    !RESERVED_IDS.contains(&id)
        && !id.is_empty()
        && id.len() <= MAX_ID_LEN
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
        && !id.starts_with(['_', '-'])
}

fn option_value_ok(kind: OptionKind, value: &toml::Value) -> bool {
    match kind {
        OptionKind::Text | OptionKind::Template => value.is_str(),
        OptionKind::Bool => value.is_bool(),
        OptionKind::Choice(choices) => value.as_str().is_some_and(|s| choices.contains(&s)),
    }
}

/// Every problem of one sequence; empty when it is acceptable. A step this
/// build does not know is reported, not removed.
pub fn validate(def: &SequenceDef) -> Vec<SequenceProblem> {
    let mut out = Vec::new();
    if def.name.trim().is_empty() {
        out.push(problem(None, "empty_name", "the sequence has no name"));
    }
    if !is_valid_id(&def.id) {
        out.push(problem(
            None,
            "bad_id",
            format!(
                "\"{}\" is not a usable id: use a-z, 0-9, _ and -, up to {MAX_ID_LEN} characters, starting with a letter or digit",
                def.id
            ),
        ));
    }
    if def.steps.is_empty() {
        out.push(problem(None, "no_steps", "the sequence has no steps"));
    }
    if def.steps.len() > MAX_STEPS {
        out.push(problem(
            None,
            "too_many_steps",
            format!("{} steps; at most {MAX_STEPS}", def.steps.len()),
        ));
    }
    for (i, step) in def.steps.iter().enumerate() {
        let at = Some(i);
        let Some(vocab) = step_kind(&step.run) else {
            match excluded_reason(&step.run) {
                Some(why) => {
                    let what = if why == NESTED_REASON {
                        "nested_sequence"
                    } else {
                        "excluded_run"
                    };
                    out.push(problem(
                        at,
                        what,
                        format!("\"{}\" is not allowed: {why}", step.run),
                    ));
                }
                None => out.push(problem(
                    at,
                    "unknown_run",
                    format!(
                        "\"{}\" is not a command this version can run in a sequence",
                        step.run
                    ),
                )),
            }
            continue;
        };
        for (key, value) in &step.options {
            match vocab.options.iter().find(|o| o.key == key) {
                None => out.push(problem(
                    at,
                    "unknown_option",
                    format!("\"{}\" has no option \"{key}\"", step.run),
                )),
                Some(spec) if !option_value_ok(spec.kind, value) => out.push(problem(
                    at,
                    "bad_option_value",
                    format!(
                        "option \"{key}\" of \"{}\" has the wrong kind of value",
                        step.run
                    ),
                )),
                Some(_) => {}
            }
        }
        if step.pause == PauseMode::Never {
            if vocab.pause_forced {
                out.push(problem(
                    at,
                    "never_forced",
                    format!(
                        "\"{}\" always asks before it runs; \"never\" is not allowed",
                        step.run
                    ),
                ));
            }
            for spec in vocab.options.iter().filter(|o| o.required_when_never) {
                let given = step.options.get(spec.key).is_some_and(|v| match v {
                    toml::Value::String(s) => !s.trim().is_empty(),
                    _ => true,
                });
                if !given {
                    out.push(problem(
                        at,
                        "never_needs_option",
                        format!(
                            "\"{}\" never stops, so it needs the option \"{}\"",
                            step.run, spec.key
                        ),
                    ));
                }
            }
        }
    }
    out
}

/// [`validate`] for each sequence, plus a `duplicate_id` problem for every
/// id the set holds twice (the problem's step is `None`). Returns
/// `(index in defs, problem)`.
pub fn validate_all(defs: &[SequenceDef]) -> Vec<(usize, SequenceProblem)> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for (i, def) in defs.iter().enumerate() {
        for p in validate(def) {
            out.push((i, p));
        }
        if !seen.insert(def.id.as_str()) {
            out.push((
                i,
                problem(
                    None,
                    "duplicate_id",
                    format!(
                        "another visible sequence already uses the id \"{}\"",
                        def.id
                    ),
                ),
            ));
        }
    }
    out
}

// -------------------------------------------------------------- templates

/// What a template may refer to.
#[derive(Debug, Clone, Copy)]
pub struct TemplateVars<'a> {
    /// The current branch, `/` kept as it is.
    pub branch: &'a str,
    /// `yyyy-mm-dd`.
    pub date: &'a str,
    /// `hh-mm`.
    pub time: &'a str,
}

/// Expands `${branch}`, `${date}`, `${time}` and `${n}`. `${n}` is the first
/// number from 1 up for which `exists(expanded)` is false (so
/// `wip-${n}` gives `wip-1`, or `wip-2` when `wip-1` is taken). Unknown
/// `${...}` stay as written. Without `${n}` the text is expanded once.
pub fn expand_template(
    template: &str,
    vars: &TemplateVars,
    exists: &dyn Fn(&str) -> bool,
) -> String {
    let base = template
        .replace("${branch}", vars.branch)
        .replace("${date}", vars.date)
        .replace("${time}", vars.time);
    if !base.contains("${n}") {
        return base;
    }
    for n in 1..=9999u32 {
        let candidate = base.replace("${n}", &n.to_string());
        if !exists(&candidate) {
            return candidate;
        }
    }
    base.replace("${n}", "10000")
}

// ---------------------------------------------------------------- presets

/// The sequences offered (never active by themselves). English names: the
/// TUI localises them when it inserts one. Scope is the default
/// (`installation`) without an owner; the inserter sets it.
pub fn presets() -> Vec<SequenceDef> {
    vec![
        SequenceDef {
            id: "backup".into(),
            name: "Backup".into(),
            description: Some(
                "Stage everything, write the message with the AI, commit and push".into(),
            ),
            steps: vec![
                StepSpec::new("stage_all"),
                StepSpec::new("ai_generate_commit_message"),
                StepSpec::new("commit"),
                StepSpec::new("push"),
            ],
            ..SequenceDef::default()
        },
        SequenceDef {
            id: "compose_and_push".into(),
            name: "Compose and push".into(),
            description: Some("Let the AI split the changes into commits, then push".into()),
            steps: vec![StepSpec::new("ai_compose_commits"), StepSpec::new("push")],
            ..SequenceDef::default()
        },
        SequenceDef {
            id: "worktree_agent".into(),
            name: "Worktree and agent".into(),
            description: Some(
                "Create a worktree for the current branch and open the AI agent in it".into(),
            ),
            steps: vec![
                StepSpec::new("create_worktree_from_current_branch")
                    .with_option("name", "wip-${date}-${n}")
                    .with_option("open_tab", true),
                StepSpec::new("tab_open_agent_here"),
            ],
            ..SequenceDef::default()
        },
    ]
}

// ------------------------------------------------------------- visibility

/// The sequences `install_id` sees: its own (`scope = installation` with
/// that owner), every `machine` one, and, when `repo` is given with
/// `trusted == true`, the repository's file. An untrusted repository file is
/// not even read. A `repo` entry in `config.toml` is ignored (that scope
/// lives in the repository). Order: installation, machine, repo.
pub fn visible_sequences(
    config: &Config,
    install_id: &str,
    repo: Option<(&Path, bool)>,
) -> Vec<(SequenceDef, Origin)> {
    let mut out: Vec<(SequenceDef, Origin)> = Vec::new();
    for scope in [Scope::Installation, Scope::Machine] {
        for def in &config.sequences {
            let mine = scope != Scope::Installation || def.owner.as_deref() == Some(install_id);
            if def.scope == scope && mine {
                out.push((def.clone(), scope));
            }
        }
    }
    if let Some((root, true)) = repo {
        if let Ok(defs) = load_repo_sequences(root, true) {
            out.extend(defs.into_iter().map(|d| (d, Scope::Repo)));
        }
    }
    out
}

// ----------------------------------------------------------- repo file

/// `<repo>/.gitpoulpi/sequences.toml`.
pub fn repo_file(repo: &Path) -> PathBuf {
    repo.join(REPO_FILE)
}

/// Reads the repository's sequences. With `trusted == false` nothing is read
/// and the answer is empty: a committed file is someone else's commands.
/// Entries come back with `scope = repo` and no owner whatever the file says.
pub fn load_repo_sequences(repo: &Path, trusted: bool) -> Result<Vec<SequenceDef>> {
    if !trusted {
        return Ok(Vec::new());
    }
    peek_repo_sequences(repo)
}

/// Reads the repository's sequences without the trust check, for the prompt
/// that asks whether to trust them. Never run what this returns.
pub fn peek_repo_sequences(repo: &Path) -> Result<Vec<SequenceDef>> {
    let path = repo_file(repo);
    if !path.exists() {
        return Ok(Vec::new());
    }
    parse_repo_sequences(&std::fs::read_to_string(path)?)
}

/// Parses the text of a repository sequences file.
pub fn parse_repo_sequences(text: &str) -> Result<Vec<SequenceDef>> {
    let doc: toml::Table = toml::from_str(text)?;
    let mut defs: Vec<SequenceDef> = match doc.get("sequences") {
        Some(v) => v
            .clone()
            .try_into()
            .map_err(|err: toml::de::Error| Error::BadValue {
                key: "sequences".into(),
                value: "<list>".into(),
                expected: err.to_string(),
            })?,
        None => Vec::new(),
    };
    for d in &mut defs {
        d.scope = Scope::Repo;
        d.owner = None;
    }
    Ok(defs)
}

/// A digest of what would run, for the trust record: SHA-256 (hex) over each
/// sequence's id, then each step's `run`, `pause` and options in order. Any
/// change of a command, a pause or an option asks again; the name and the
/// description do not matter.
pub fn repo_sequences_hash(defs: &[SequenceDef]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    // The order of the sequences in the file does not matter.
    let mut sorted: Vec<&SequenceDef> = defs.iter().collect();
    sorted.sort_by(|a, b| a.id.cmp(&b.id));
    for d in sorted {
        h.update(d.id.as_bytes());
        h.update([0]);
        for s in &d.steps {
            h.update(s.run.as_bytes());
            h.update([1]);
            h.update(s.pause.as_str().as_bytes());
            h.update([1]);
            h.update(toml::to_string(&s.options).unwrap_or_default().as_bytes());
            h.update([2]);
        }
        h.update([3]);
    }
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

// --------------------------------------------------------------- writers

fn read_doc(path: &Path) -> Result<toml::Table> {
    if !path.exists() {
        return Ok(toml::Table::new());
    }
    let raw = std::fs::read_to_string(path)?;
    if raw.trim().is_empty() {
        return Ok(toml::Table::new());
    }
    Ok(toml::from_str(&raw)?)
}

fn id_of(v: &toml::Value) -> Option<&str> {
    v.get("id").and_then(toml::Value::as_str)
}

fn upsert(doc: &mut toml::Table, def: &SequenceDef) -> Result<()> {
    let value = toml::Value::try_from(def)?;
    let list = doc
        .entry("sequences")
        .or_insert_with(|| toml::Value::Array(Vec::new()));
    let toml::Value::Array(items) = list else {
        return Err(Error::BadValue {
            key: "sequences".into(),
            value: "<not a list>".into(),
            expected: "an array of tables".into(),
        });
    };
    match items.iter().position(|v| id_of(v) == Some(def.id.as_str())) {
        Some(i) => items[i] = value,
        None => items.push(value),
    }
    Ok(())
}

fn remove(doc: &mut toml::Table, id: &str) -> bool {
    let Some(toml::Value::Array(items)) = doc.get_mut("sequences") else {
        return false;
    };
    let before = items.len();
    items.retain(|v| id_of(v) != Some(id));
    let removed = items.len() != before;
    if items.is_empty() {
        doc.remove("sequences");
    }
    removed
}

fn write_config_doc(path: &Path, doc: toml::Table) -> Result<()> {
    // The whole document must still load, like `set_in_file`.
    let _: Config =
        toml::Value::Table(doc.clone())
            .try_into()
            .map_err(|err: toml::de::Error| Error::BadValue {
                key: "sequences".into(),
                value: "<list>".into(),
                expected: err.to_string(),
            })?;
    if let Some(parent) = path.parent() {
        secure_fs::create_private_dir_all(parent)?;
    }
    Ok(secure_fs::write_private(
        path,
        toml::to_string_pretty(&doc)?.as_bytes(),
    )?)
}

/// Saves `def` where its scope says, replacing the sequence with the same id
/// there and keeping every other key of the file: `installation` and
/// `machine` in `[[sequences]]` of `config_path` (an installation one gets
/// `install_id` as owner when it has none), `repo` in the repository's file
/// (`repo` must then be given). The caller decides trust before writing.
pub fn save_sequence(
    def: &SequenceDef,
    config_path: &Path,
    install_id: &str,
    repo: Option<&Path>,
) -> Result<()> {
    match def.scope {
        Scope::Repo => {
            let root = repo.ok_or_else(|| Error::BadValue {
                key: format!("sequences.{}", def.id),
                value: "repo".into(),
                expected: "a repository for a repo-scope sequence".into(),
            })?;
            let path = repo_file(root);
            let mut doc = read_doc(&path)?;
            let mut stored = def.clone();
            stored.owner = None;
            upsert(&mut doc, &stored)?;
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, toml::to_string_pretty(&doc)?)?;
            Ok(())
        }
        scope => {
            let mut stored = def.clone();
            if scope == Scope::Installation && stored.owner.is_none() {
                stored.owner = Some(install_id.to_string());
            }
            let mut doc = read_doc(config_path)?;
            upsert(&mut doc, &stored)?;
            write_config_doc(config_path, doc)
        }
    }
}

/// Removes the sequence `id` from the place its `scope` names; `false` when
/// it was not there.
pub fn remove_sequence(
    scope: Scope,
    id: &str,
    config_path: &Path,
    repo: Option<&Path>,
) -> Result<bool> {
    match scope {
        Scope::Repo => {
            let Some(root) = repo else { return Ok(false) };
            let path = repo_file(root);
            if !path.exists() {
                return Ok(false);
            }
            let mut doc = read_doc(&path)?;
            let removed = remove(&mut doc, id);
            if removed {
                std::fs::write(path, toml::to_string_pretty(&doc)?)?;
            }
            Ok(removed)
        }
        _ => {
            let mut doc = read_doc(config_path)?;
            let removed = remove(&mut doc, id);
            if removed {
                write_config_doc(config_path, doc)?;
            }
            Ok(removed)
        }
    }
}

/// Puts the sequences named in `order` into that relative order, each in a
/// slot one of them already held; every other entry of the file keeps its
/// place (another installation's sequences are not ours to move). The
/// storage is the one `scope` names, like [`remove_sequence`]. `false` when
/// nothing moved.
pub fn reorder_sequences(
    scope: Scope,
    order: &[String],
    config_path: &Path,
    repo: Option<&Path>,
) -> Result<bool> {
    fn apply(doc: &mut toml::Table, order: &[String]) -> bool {
        let Some(toml::Value::Array(items)) = doc.get_mut("sequences") else {
            return false;
        };
        let slots: Vec<usize> = items
            .iter()
            .enumerate()
            .filter(|(_, v)| id_of(v).is_some_and(|id| order.iter().any(|o| o == id)))
            .map(|(i, _)| i)
            .collect();
        let wanted: Vec<toml::Value> = order
            .iter()
            .filter_map(|id| items.iter().find(|v| id_of(v) == Some(id.as_str())))
            .cloned()
            .collect();
        if wanted.len() != slots.len() {
            return false;
        }
        let mut moved = false;
        for (slot, value) in slots.into_iter().zip(wanted) {
            if items[slot] != value {
                items[slot] = value;
                moved = true;
            }
        }
        moved
    }
    match scope {
        Scope::Repo => {
            let Some(root) = repo else { return Ok(false) };
            let path = repo_file(root);
            if !path.exists() {
                return Ok(false);
            }
            let mut doc = read_doc(&path)?;
            let moved = apply(&mut doc, order);
            if moved {
                std::fs::write(path, toml::to_string_pretty(&doc)?)?;
            }
            Ok(moved)
        }
        _ => {
            let mut doc = read_doc(config_path)?;
            let moved = apply(&mut doc, order);
            if moved {
                write_config_doc(config_path, doc)?;
            }
            Ok(moved)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reordering_moves_only_the_named_sequences_into_their_own_slots() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        for id in ["a", "foreign", "b", "c"] {
            let mut d = def(id, vec![StepSpec::new("push")]);
            d.owner = Some(if id == "foreign" { "other" } else { "me" }.into());
            save_sequence(&d, &path, "me", None).unwrap();
        }
        let order: Vec<String> = ["c", "a", "b"].iter().map(|s| s.to_string()).collect();
        assert!(reorder_sequences(Scope::Installation, &order, &path, None).unwrap());
        let (config, _) = Config::from_toml_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let ids: Vec<&str> = config.sequences.iter().map(|d| d.id.as_str()).collect();
        assert_eq!(ids, ["c", "foreign", "a", "b"]);
        assert!(!reorder_sequences(Scope::Installation, &order, &path, None).unwrap());
    }

    fn def(id: &str, steps: Vec<StepSpec>) -> SequenceDef {
        SequenceDef {
            id: id.into(),
            name: "Name".into(),
            steps,
            ..SequenceDef::default()
        }
    }

    fn whats(d: &SequenceDef) -> Vec<String> {
        validate(d).into_iter().map(|p| p.what).collect()
    }

    #[test]
    fn a_sequence_round_trips_with_unknown_fields_and_an_unknown_step() {
        let text = r#"
[[sequences]]
id = "x"
name = "X"
future_flag = 7

[[sequences.steps]]
run = "stage_all"
colour = "red"

[[sequences.steps]]
run = "from_the_future"
pause = "always"
[sequences.steps.options]
whatever = [1, 2]
"#;
        let (config, _) = Config::from_toml_str(text).unwrap();
        let d = &config.sequences[0];
        assert_eq!(d.extra["future_flag"].as_integer(), Some(7));
        assert_eq!(d.steps[0].extra["colour"].as_str(), Some("red"));
        assert_eq!(d.steps[1].run, "from_the_future");
        assert_eq!(d.steps[1].pause, PauseMode::Always);
        let again = toml::to_string(&config).unwrap();
        let (back, _) = Config::from_toml_str(&again).unwrap();
        assert_eq!(back.sequences, config.sequences);
        assert!(whats(d).contains(&"unknown_run".to_string()));
        assert_eq!(back.sequences[0].steps.len(), 2, "the unknown step is kept");
    }

    #[test]
    fn defaults_are_not_written() {
        let text = toml::to_string(&def("a", vec![StepSpec::new("push")])).unwrap();
        assert!(
            !text.contains("pause") && !text.contains("options"),
            "{text}"
        );
        assert!(text.contains("scope = \"installation\""), "{text}");
    }

    #[test]
    fn validate_rules() {
        assert!(whats(&def("ok", vec![StepSpec::new("push")])).is_empty());
        let mut d = def("ok", vec![StepSpec::new("push")]);
        d.name = "  ".into();
        assert_eq!(whats(&d), ["empty_name"]);
        for bad in ["", "Has Space", "-x", "UP", &"a".repeat(MAX_ID_LEN + 1)] {
            assert_eq!(
                whats(&def(bad, vec![StepSpec::new("push")])),
                ["bad_id"],
                "{bad}"
            );
        }
        let many = def("m", vec![StepSpec::new("fetch"); MAX_STEPS + 1]);
        assert_eq!(whats(&many), ["too_many_steps"]);
        assert!(whats(&def("m", vec![StepSpec::new("fetch"); MAX_STEPS])).is_empty());
        assert_eq!(whats(&def("e", vec![])), ["no_steps"]);

        let d = def(
            "u",
            vec![StepSpec::new("push"), StepSpec::new("no_such_thing")],
        );
        let p = validate(&d);
        assert_eq!((p[0].step, p[0].what.as_str()), (Some(1), "unknown_run"));

        for id in [
            "discard_all_changes",
            "stash_drop",
            "quit",
            "worktree_remove_selected",
            "reset_to_selected_commit_hard",
        ] {
            let p = validate(&def("x", vec![StepSpec::new(id)]));
            assert_eq!(p[0].what, "excluded_run", "{id}");
            assert!(excluded_reason(id).is_some(), "{id}");
        }
        assert_eq!(
            whats(&def("n", vec![StepSpec::new("sequence_run")])),
            ["nested_sequence"]
        );

        let bad_opt = StepSpec::new("push").with_option("nope", "x");
        assert_eq!(whats(&def("o", vec![bad_opt])), ["unknown_option"]);
        let bad_val =
            StepSpec::new("create_worktree_from_current_branch").with_option("open_tab", "yes");
        assert_eq!(whats(&def("o", vec![bad_val])), ["bad_option_value"]);

        let forced = StepSpec::new("force_push_with_lease").with_pause(PauseMode::Never);
        assert_eq!(whats(&def("f", vec![forced])), ["never_forced"]);
        let always = StepSpec::new("force_push_with_lease").with_pause(PauseMode::Always);
        assert!(whats(&def("f", vec![always])).is_empty());

        let never = StepSpec::new("new_branch").with_pause(PauseMode::Never);
        assert_eq!(
            whats(&def("b", vec![never.clone()])),
            ["never_needs_option"]
        );
        let given = never.with_option("name", "wip-${n}");
        assert!(whats(&def("b", vec![given])).is_empty());
        // The AI message step may be `never` on purpose.
        let ai = StepSpec::new("ai_generate_commit_message").with_pause(PauseMode::Never);
        assert!(whats(&def("a", vec![ai])).is_empty());
    }

    #[test]
    fn duplicate_ids_in_a_set_are_reported() {
        let a = def("same", vec![StepSpec::new("push")]);
        let all = validate_all(&[a.clone(), def("other", vec![StepSpec::new("push")]), a]);
        assert_eq!(all.len(), 1);
        assert_eq!((all[0].0, all[0].1.what.as_str()), (2, "duplicate_id"));
    }

    #[test]
    fn the_vocabulary_is_consistent() {
        let mut seen = BTreeSet::new();
        for k in STEP_VOCABULARY {
            assert!(seen.insert(k.id), "{}", k.id);
            assert!(is_valid_id(k.id), "{}", k.id);
            assert!(!k.meaning_en.is_empty() && !k.meaning_en.contains('\u{2014}'));
            if k.pause_forced {
                assert!(k.pausable, "{}", k.id);
            }
            assert!(excluded_reason(k.id).is_none(), "{}", k.id);
        }
    }

    #[test]
    fn templates_expand() {
        let vars = TemplateVars {
            branch: "feat/x",
            date: "2026-11-24",
            time: "09-05",
        };
        let none = |_: &str| false;
        assert_eq!(
            expand_template("${branch}-${date}-${time}", &vars, &none),
            "feat/x-2026-11-24-09-05"
        );
        assert_eq!(expand_template("wip-${n}", &vars, &none), "wip-1");
        let taken = |s: &str| s == "wip-2026-11-24-1" || s == "wip-2026-11-24-2";
        assert_eq!(
            expand_template("wip-${date}-${n}", &vars, &taken),
            "wip-2026-11-24-3"
        );
        assert_eq!(
            expand_template("a-${unknown}", &vars, &none),
            "a-${unknown}"
        );
        assert_eq!(expand_template("plain", &vars, &|_| true), "plain");
    }

    #[test]
    fn presets_validate_clean_and_pending_steps_are_not_errors() {
        let ids: Vec<_> = presets().iter().map(|d| d.id.clone()).collect();
        assert_eq!(ids, ["backup", "compose_and_push", "worktree_agent"]);
        for d in presets() {
            assert!(validate(&d).is_empty(), "{}: {:?}", d.id, validate(&d));
        }
        assert!(validate_all(&presets()).is_empty());
        // S3 added both palette commands: no step is pending any more (the
        // flag stays for a later slice); a truly unknown id is flagged.
        assert!(!step_kind("tab_open_agent_here").unwrap().pending);
        assert!(!step_kind("tab_open_in").unwrap().pending);
        assert!(!step_kind("push").unwrap().pending);
        let mut d = presets().remove(0);
        d.steps.push(StepSpec::new("tab_open_agent_hear"));
        assert_eq!(whats(&d), ["unknown_run"]);
    }

    fn owned(id: &str, owner: &str, scope: Scope) -> SequenceDef {
        SequenceDef {
            scope,
            owner: (scope == Scope::Installation).then(|| owner.to_string()),
            ..def(id, vec![StepSpec::new("push")])
        }
    }

    #[test]
    fn visibility_follows_scope_and_trust() {
        let repo = tempfile::tempdir().unwrap();
        save_sequence(
            &SequenceDef {
                scope: Scope::Repo,
                ..def("team", vec![StepSpec::new("fetch")])
            },
            &repo.path().join("unused.toml"),
            "me",
            Some(repo.path()),
        )
        .unwrap();
        let config = Config {
            sequences: vec![
                owned("mine", "me", Scope::Installation),
                owned("theirs", "other", Scope::Installation),
                owned("shared", "", Scope::Machine),
                owned("misplaced", "", Scope::Repo),
            ],
            ..Config::default()
        };
        let ids = |v: Vec<(SequenceDef, Origin)>| {
            v.into_iter().map(|(d, o)| (d.id, o)).collect::<Vec<_>>()
        };
        assert_eq!(
            ids(visible_sequences(&config, "me", None)),
            [
                ("mine".into(), Scope::Installation),
                ("shared".into(), Scope::Machine)
            ]
        );
        assert_eq!(
            ids(visible_sequences(&config, "other", None)),
            [
                ("theirs".into(), Scope::Installation),
                ("shared".into(), Scope::Machine)
            ]
        );
        assert_eq!(
            visible_sequences(&config, "me", Some((repo.path(), false))).len(),
            2
        );
        let trusted = ids(visible_sequences(&config, "me", Some((repo.path(), true))));
        assert_eq!(trusted.last().unwrap(), &("team".to_string(), Scope::Repo));
        assert_eq!(trusted.len(), 3);
    }

    #[test]
    fn an_untrusted_repo_file_is_not_read_and_the_hash_tracks_commands() {
        let repo = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(repo.path().join(".gitpoulpi")).unwrap();
        std::fs::write(repo_file(repo.path()), "not toml at all {{{").unwrap();
        assert!(load_repo_sequences(repo.path(), false).unwrap().is_empty());
        assert!(load_repo_sequences(repo.path(), true).is_err());
        let a = vec![def("s", vec![StepSpec::new("push")])];
        let mut renamed = a.clone();
        renamed[0].name = "Other".into();
        assert_eq!(repo_sequences_hash(&a), repo_sequences_hash(&renamed));
        let b = vec![def(
            "s",
            vec![StepSpec::new("push"), StepSpec::new("fetch")],
        )];
        assert_ne!(repo_sequences_hash(&a), repo_sequences_hash(&b));
    }

    #[test]
    fn writers_put_a_sequence_in_the_right_place_and_keep_the_rest() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = dir.path().join("config.toml");
        std::fs::write(
            &cfg,
            "[ui]\ntheme = \"light\"\n\n[[sequences]]\nid = \"old\"\nname = \"Old\"\nscope = \"machine\"\nkept = true\n",
        )
        .unwrap();
        let mut d = def("new", vec![StepSpec::new("push")]);
        save_sequence(&d, &cfg, "me", None).unwrap();
        let (config, _) = Config::from_toml_str(&std::fs::read_to_string(&cfg).unwrap()).unwrap();
        assert_eq!(config.ui.theme, "light");
        assert_eq!(config.sequences.len(), 2);
        assert_eq!(config.sequences[0].extra["kept"].as_bool(), Some(true));
        assert_eq!(config.sequences[1].owner.as_deref(), Some("me"));
        // Replace by id.
        d.name = "Renamed".into();
        save_sequence(&d, &cfg, "me", None).unwrap();
        let (config, _) = Config::from_toml_str(&std::fs::read_to_string(&cfg).unwrap()).unwrap();
        assert_eq!(config.sequences.len(), 2);
        assert_eq!(config.sequences[1].name, "Renamed");
        // A repo sequence goes to the repository, not the config.
        let repo = dir.path().join("repo");
        let mut r = def("team", vec![StepSpec::new("fetch")]);
        r.scope = Scope::Repo;
        assert!(save_sequence(&r, &cfg, "me", None).is_err());
        save_sequence(&r, &cfg, "me", Some(&repo)).unwrap();
        assert!(repo_file(&repo).exists());
        assert_eq!(Config::load(&cfg).unwrap().sequences.len(), 2);
        let loaded = load_repo_sequences(&repo, true).unwrap();
        assert_eq!(
            (loaded[0].id.as_str(), loaded[0].scope),
            ("team", Scope::Repo)
        );
        // Remove.
        assert!(remove_sequence(Scope::Machine, "old", &cfg, None).unwrap());
        assert!(!remove_sequence(Scope::Machine, "old", &cfg, None).unwrap());
        assert!(remove_sequence(Scope::Repo, "team", &cfg, Some(&repo)).unwrap());
        assert!(load_repo_sequences(&repo, true).unwrap().is_empty());
    }

    #[test]
    fn the_sequence_keybindings_context_loads() {
        let text = "[keybindings.sequence]\nbackup = \"Ctrl+G b\"\n";
        let (config, _) = Config::from_toml_str(text).unwrap();
        assert_eq!(
            config.keybindings.get(KEYBINDINGS_CONTEXT, "backup"),
            Some(&["Ctrl+G b".to_string()][..])
        );
    }
}
