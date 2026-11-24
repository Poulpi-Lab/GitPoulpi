//! `gitpoulpi sequence list|show|validate|run|add|remove`: named sequences of
//! palette commands (see `gitpoulpi_config::sequences`), headless.
//!
//! `list`, `show`, `validate`, `add` and `remove` answer from files on this
//! machine and start no daemon (`is_local`). `run` goes through the daemon
//! one request per step, and each step is the *existing* CLI command run
//! through `execute` (so a step can never behave differently from the command
//! of the same name): [`MAPPING`] turns a vocabulary id and its options into
//! that command's argv, and a test checks that every vocabulary id marked
//! `cli` has a row.
//!
//! Everything that can refuse does so before the first step runs (unknown or
//! terminal-only steps, a forced pause, a pause nobody answers), so a refused
//! run changes nothing.

use std::path::{Path, PathBuf};

use clap::Parser;
use gitpoulpi_config::installation;
use gitpoulpi_config::sequences::{
    self, expand_template, step_kind, PauseMode, Scope, SequenceDef, SequenceProblem, StepKind,
    StepSpec, TemplateVars,
};
use gitpoulpi_config::{Config, ConfigPaths};
use gitpoulpi_core::domain::{OperationKind, Ref, RefKind};
use serde_json::{json, Value};

use super::emitted;
use crate::cli::{Cli, SequenceAction, SequenceScopeArg};
use crate::client::{discover_repo, Session};
use crate::repo_cmd::Emitted;

type Res = Result<Emitted, String>;

/// What the dry run shows for the AI message that does not exist yet.
const AI_PLACEHOLDER: &str = "<message written by the AI when the run reaches the commit>";

// ------------------------------------------------------------------ mapping

/// What one resolved step does.
enum Action {
    /// Run this `gitpoulpi` command (argv after the program name).
    Argv(Vec<String>),
    /// `ai commit-message`; its answer is the message of the next commit.
    AiMessage,
    /// Nothing runs now; the next `stash push` is told to name the stash with
    /// the AI (the CLI has no message-only stash command).
    DeferToStash,
}

/// What a step may use while it resolves.
struct Env {
    repo_root: PathBuf,
    branch: String,
    branches: Vec<String>,
    date: String,
    time: String,
    ai_message: Option<String>,
    stash_ai: bool,
}

impl Env {
    fn vars(&self) -> TemplateVars<'_> {
        TemplateVars {
            branch: &self.branch,
            date: &self.date,
            time: &self.time,
        }
    }

    /// A template with `${n}` taken against `taken`.
    fn expand(&self, template: &str, taken: &dyn Fn(&str) -> bool) -> String {
        expand_template(template, &self.vars(), taken)
    }

    fn branch_taken(&self, name: &str) -> bool {
        self.branches.iter().any(|b| b == name)
    }

    fn worktree_taken(&self, name: &str) -> bool {
        self.branch_taken(name) || worktree_path(&self.repo_root, name).exists()
    }
}

/// `<repo parent>/<repo name>-<label>`, the folder the terminal UI proposes
/// for a worktree (label made folder-safe).
fn worktree_path(repo_root: &Path, label: &str) -> PathBuf {
    let safe: String = label
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || "-_.".contains(c) {
                c
            } else {
                '-'
            }
        })
        .collect();
    let name = repo_root
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "worktree".into());
    repo_root
        .parent()
        .unwrap_or(repo_root)
        .join(format!("{name}-{safe}"))
}

fn text_option<'a>(step: &'a StepSpec, key: &str) -> Option<&'a str> {
    step.options
        .get(key)
        .and_then(toml::Value::as_str)
        .filter(|s| !s.trim().is_empty())
}

fn argv(parts: &[&str]) -> Result<Action, String> {
    Ok(Action::Argv(parts.iter().map(|s| s.to_string()).collect()))
}

type Build = fn(&StepSpec, &mut Env) -> Result<Action, String>;

fn commit(step: &StepSpec, env: &mut Env) -> Result<Action, String> {
    let message = match text_option(step, "message") {
        Some(template) => env.expand(template, &|_| false),
        None => env
            .ai_message
            .clone()
            .ok_or("the commit has no message: give it a `message` option or put ai_generate_commit_message before it")?,
    };
    // `--message=` keeps a message that starts with a dash a value.
    Ok(Action::Argv(vec![
        "commit".into(),
        format!("--message={message}"),
    ]))
}

fn stash_push(_: &StepSpec, env: &mut Env) -> Result<Action, String> {
    let mut args = vec!["stash".to_string(), "push".to_string()];
    if std::mem::take(&mut env.stash_ai) {
        args.push("--ai".into());
    }
    Ok(Action::Argv(args))
}

fn named(
    step: &StepSpec,
    env: &Env,
    what: &str,
    taken: &dyn Fn(&str) -> bool,
) -> Result<String, String> {
    let template =
        text_option(step, "name").ok_or_else(|| format!("{what} needs a `name` option"))?;
    Ok(env.expand(template, taken))
}

fn new_branch(step: &StepSpec, env: &mut Env) -> Result<Action, String> {
    let name = named(step, env, "a new branch", &|n| env.branch_taken(n))?;
    Ok(Action::Argv(vec!["branch".into(), "create".into(), name]))
}

fn new_tag(step: &StepSpec, env: &mut Env) -> Result<Action, String> {
    let name = named(step, env, "a new tag", &|_| false)?;
    Ok(Action::Argv(vec!["tag".into(), "create".into(), name]))
}

fn worktree_add(step: &StepSpec, env: &mut Env, start: &str) -> Result<Action, String> {
    let template = text_option(step, "name").unwrap_or("worktree-${n}");
    let name = env.expand(template, &|n| env.worktree_taken(n));
    let path = worktree_path(&env.repo_root, &name);
    Ok(Action::Argv(vec![
        "worktree".into(),
        "add".into(),
        path.to_string_lossy().into_owned(),
        start.to_string(),
        "-b".into(),
        name,
    ]))
}

fn worktree_from_current(step: &StepSpec, env: &mut Env) -> Result<Action, String> {
    worktree_add(step, env, "HEAD")
}

fn worktree_choose(step: &StepSpec, env: &mut Env) -> Result<Action, String> {
    let branch = text_option(step, "branch")
        .ok_or("choosing a branch needs a `branch` option")?
        .to_string();
    worktree_add(step, env, &branch)
}

fn force_push(_: &StepSpec, _: &mut Env) -> Result<Action, String> {
    Err("a force push never runs headless: do it in the terminal UI".into())
}

/// Vocabulary id -> the command it runs. One row per `cli: true` id of
/// `STEP_VOCABULARY` (checked by `every_cli_step_has_a_mapping`).
const MAPPING: &[(&str, Build)] = &[
    ("stage_all", |_, _| argv(&["stage", "--all"])),
    ("commit", commit),
    ("commit_now", commit),
    ("stash_push", stash_push),
    ("new_branch", new_branch),
    ("new_tag", new_tag),
    ("fetch", |_, _| argv(&["fetch"])),
    ("pull", |_, _| argv(&["pull"])),
    ("pull_with_mode_merge", |_, _| {
        argv(&["pull", "--mode", "merge"])
    }),
    ("pull_with_mode_ff_only", |_, _| {
        argv(&["pull", "--mode", "ff-only"])
    }),
    ("pull_with_mode_rebase", |_, _| {
        argv(&["pull", "--mode", "rebase"])
    }),
    ("push", |_, _| argv(&["push"])),
    ("push_to_all_targets", |_, _| {
        argv(&["push", "--all-targets"])
    }),
    ("pull_all_branches", |_, _| {
        argv(&["pull", "--all-branches"])
    }),
    ("push_all_unpushed_tags", |_, _| argv(&["tag", "push"])),
    ("ai_generate_commit_message", |_, _| Ok(Action::AiMessage)),
    ("ai_generate_stash_message", |_, _| Ok(Action::DeferToStash)),
    ("create_worktree_from_current_branch", worktree_from_current),
    ("create_worktree_choose_branch", worktree_choose),
    // Mapped so the table is complete, refused before and while running.
    ("force_push_with_lease", force_push),
];

fn mapping(id: &str) -> Option<Build> {
    MAPPING.iter().find(|(k, _)| *k == id).map(|(_, b)| *b)
}

// ---------------------------------------------------------------- preflight

fn problem(step: Option<usize>, what: &str, reason: impl Into<String>) -> SequenceProblem {
    SequenceProblem {
        step,
        what: what.to_string(),
        reason: reason.into(),
    }
}

/// The cargo feature a step needs that this build lacks.
fn missing_feature(id: &str) -> Option<&'static str> {
    let needs = if id.starts_with("ai_") {
        ("ai", cfg!(feature = "ai"))
    } else if id.starts_with("create_worktree_") {
        ("worktree", cfg!(feature = "worktree"))
    } else {
        return None;
    };
    (!needs.1).then_some(needs.0)
}

/// Why this run cannot happen headless, whatever `--yes` says. Empty when the
/// sequence is runnable (it may still pause, see [`pause_reason`]).
fn headless_problems(def: &SequenceDef) -> Vec<SequenceProblem> {
    let mut out = Vec::new();
    for (i, step) in def.steps.iter().enumerate() {
        let Some(kind) = step_kind(&step.run) else {
            continue;
        };
        if kind.pause_forced {
            out.push(problem(
                Some(i),
                "forced_pause",
                format!(
                    "{} always asks first and is never accepted headless: run it in the terminal UI",
                    step.run
                ),
            ));
            continue;
        }
        if !kind.cli {
            out.push(problem(
                Some(i),
                "not_headless",
                format!(
                    "{} only works in the terminal UI (it opens a terminal, a tab or an application)",
                    step.run
                ),
            ));
            continue;
        }
        if let Some(feature) = missing_feature(&step.run) {
            out.push(problem(
                Some(i),
                "not_built",
                format!(
                    "{} needs the `{feature}` feature, which this build lacks",
                    step.run
                ),
            ));
            continue;
        }
        let earlier = &def.steps[..i];
        let later = &def.steps[i + 1..];
        match step.run.as_str() {
            "commit" | "commit_now"
                if text_option(step, "message").is_none()
                    && !earlier
                        .iter()
                        .any(|s| s.run == "ai_generate_commit_message") =>
            {
                out.push(problem(
                    Some(i),
                    "no_message",
                    "the commit has no message headless: give it a `message` option or put ai_generate_commit_message before it",
                ));
            }
            "new_branch" | "new_tag" if text_option(step, "name").is_none() => {
                out.push(problem(
                    Some(i),
                    "missing_option",
                    "it needs a `name` option",
                ));
            }
            "create_worktree_choose_branch" if text_option(step, "branch").is_none() => {
                out.push(problem(
                    Some(i),
                    "missing_option",
                    "it needs a `branch` option",
                ));
            }
            "ai_generate_stash_message" if !later.iter().any(|s| s.run == "stash_push") => {
                out.push(problem(
                    Some(i),
                    "no_stash",
                    "the AI stash message is used by a later stash_push step, and there is none",
                ));
            }
            _ => {}
        }
    }
    out
}

/// Whether the step stops for a go-ahead, and why. `has_message`: a commit
/// whose message is known (an option, or the AI step before it) has nothing
/// to ask.
fn pause_reason(kind: &StepKind, step: &StepSpec, has_message: bool) -> Option<String> {
    match step.pause {
        PauseMode::Never => None,
        PauseMode::Always => Some("its pause is \"always\"".to_string()),
        PauseMode::Default => {
            if !kind.pausable {
                return None;
            }
            if matches!(kind.id, "commit" | "commit_now") && has_message {
                return None;
            }
            Some(format!(
                "{} pauses by default ({})",
                kind.id, kind.meaning_en
            ))
        }
    }
}

struct Confirmation {
    step: usize,
    run: String,
    reason: String,
}

fn confirmations(def: &SequenceDef) -> Vec<Confirmation> {
    let mut out = Vec::new();
    for (i, step) in def.steps.iter().enumerate() {
        let Some(kind) = step_kind(&step.run) else {
            continue;
        };
        let has_message = text_option(step, "message").is_some()
            || def.steps[..i]
                .iter()
                .any(|s| s.run == "ai_generate_commit_message");
        if let Some(reason) = pause_reason(kind, step, has_message) {
            out.push(Confirmation {
                step: i,
                run: step.run.clone(),
                reason,
            });
        }
    }
    out
}

fn problem_json(def_steps: &[StepSpec], p: &SequenceProblem) -> Value {
    json!({
        "step": p.step,
        "run": p.step.and_then(|i| def_steps.get(i)).map(|s| s.run.as_str()),
        "what": p.what,
        "reason": p.reason,
    })
}

fn refusal(def: &SequenceDef, problems: &[SequenceProblem]) -> Emitted {
    let lines: Vec<String> = problems
        .iter()
        .map(|p| match p.step {
            Some(i) => format!("step {}: {}", i + 1, p.reason),
            None => p.reason.clone(),
        })
        .collect();
    Emitted {
        json: json!({
            "sequence": def.id,
            "applied": false,
            "problems": problems.iter().map(|p| problem_json(&def.steps, p)).collect::<Vec<_>>(),
        }),
        human: format!("{} cannot run:\n{}", def.id, lines.join("\n")),
        exit_code: 2,
    }
}

// ---------------------------------------------------------------- JSON views

fn step_json(step: &StepSpec) -> Value {
    json!({
        "run": step.run,
        "pause": step.pause.as_str(),
        "options": step.options,
    })
}

/// One sequence as the commands print it.
fn entry(def: &SequenceDef, trusted: bool, extra: &[SequenceProblem]) -> Value {
    let mut problems = sequences::validate(def);
    problems.extend(extra.iter().cloned());
    let headless = headless_problems(def);
    let runnable = trusted && problems.is_empty() && headless.is_empty();
    json!({
        "id": def.id,
        "name": def.name,
        "description": def.description,
        "scope": def.scope.as_str(),
        "owner": def.owner,
        "trusted": trusted,
        "steps": def.steps.iter().map(step_json).collect::<Vec<_>>(),
        "problems": problems,
        "headless_problems": headless,
        "runnable_headless": runnable,
    })
}

/// [`entry`] plus, per step, what it is and whether it needs the terminal UI.
fn detailed(def: &SequenceDef, trusted: bool, extra: &[SequenceProblem]) -> Value {
    let mut value = entry(def, trusted, extra);
    let asks = confirmations(def);
    let details: Vec<Value> = def
        .steps
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let kind = step_kind(&s.run);
            json!({
                "index": i,
                "run": s.run,
                "meaning": kind.map(|k| k.meaning_en),
                "headless": kind.is_some_and(|k| k.cli && !k.pause_forced && missing_feature(&s.run).is_none()),
                "pauses": asks.iter().find(|c| c.step == i).map(|c| c.reason.as_str()),
            })
        })
        .collect();
    value["details"] = Value::Array(details);
    value
}

// ----------------------------------------------------------------- loading

fn config_file() -> Result<PathBuf, String> {
    ConfigPaths::discover()
        .map(|p| p.config_file())
        .ok_or_else(|| "could not determine this OS's config directory".to_string())
}

fn load_config(file: &Path) -> Result<Config, String> {
    Config::load(file).map_err(|e| format!("could not read {}: {e}", file.display()))
}

fn repo_root(repo: Option<&str>) -> Option<PathBuf> {
    discover_repo(repo).ok().map(|r| r.root)
}

/// What this installation sees in `config.toml`.
fn visible(config: &Config, install_id: &str) -> Vec<SequenceDef> {
    sequences::visible_sequences(config, install_id, None)
        .into_iter()
        .map(|(d, _)| d)
        .collect()
}

/// Repository sequences, read without trust (never run).
fn peeked(repo: Option<&str>) -> (Vec<SequenceDef>, Option<String>) {
    let Some(root) = repo_root(repo) else {
        return (Vec::new(), None);
    };
    match sequences::peek_repo_sequences(&root) {
        Ok(defs) => (defs, None),
        Err(e) => (Vec::new(), Some(e.to_string())),
    }
}

/// Duplicate-id problems of `defs`, per index.
fn duplicates(defs: &[SequenceDef]) -> Vec<Vec<SequenceProblem>> {
    let mut out = vec![Vec::new(); defs.len()];
    for (i, p) in sequences::validate_all(defs) {
        if p.what == "duplicate_id" {
            out[i].push(p);
        }
    }
    out
}

// ------------------------------------------------------------------- local

pub(crate) fn run_local(action: SequenceAction, repo: Option<&str>) -> Res {
    match action {
        SequenceAction::List => list(repo),
        SequenceAction::Show { id } => show(&id, repo),
        SequenceAction::Validate { id, file } => validate(id.as_deref(), file.as_deref(), repo),
        SequenceAction::Add {
            file,
            replace,
            scope,
        } => add(&file, replace, scope, repo),
        SequenceAction::Remove { id, scope } => remove(&id, scope, repo),
        SequenceAction::Run { .. } => unreachable!("`sequence run` goes through the daemon"),
    }
}

/// Whether the action needs the session's daemon (only `run`).
pub(crate) fn needs_daemon(action: &SequenceAction) -> bool {
    matches!(action, SequenceAction::Run { .. })
}

fn all_entries(repo: Option<&str>) -> Result<(Vec<Value>, Option<String>), String> {
    let config = load_config(&config_file()?)?;
    let defs = visible(&config, &installation::current().identity.id);
    let dups = duplicates(&defs);
    let mut out: Vec<Value> = defs
        .iter()
        .zip(&dups)
        .map(|(d, extra)| entry(d, true, extra))
        .collect();
    let (repo_defs, error) = peeked(repo);
    out.extend(repo_defs.iter().map(|d| entry(d, false, &[])));
    Ok((out, error))
}

fn list(repo: Option<&str>) -> Res {
    let (entries, error) = all_entries(repo)?;
    let human = if entries.is_empty() {
        "no sequences".to_string()
    } else {
        entries
            .iter()
            .map(|e| {
                format!(
                    "{:<20} {:<12} {} step(s){}",
                    e["id"].as_str().unwrap_or(""),
                    e["scope"].as_str().unwrap_or(""),
                    e["steps"].as_array().map_or(0, Vec::len),
                    if e["trusted"] == json!(false) {
                        "  (repository file, not trusted)"
                    } else {
                        ""
                    }
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let mut value = json!({ "sequences": entries });
    if let Some(error) = error {
        value["repo_error"] = json!(error);
    }
    Ok(emitted(value, human))
}

fn find(id: &str, repo: Option<&str>) -> Result<(SequenceDef, bool, Vec<SequenceProblem>), String> {
    let config = load_config(&config_file()?)?;
    let defs = visible(&config, &installation::current().identity.id);
    let dups = duplicates(&defs);
    if let Some(i) = defs.iter().position(|d| d.id == id) {
        return Ok((defs[i].clone(), true, dups[i].clone()));
    }
    let (repo_defs, _) = peeked(repo);
    if let Some(d) = repo_defs.into_iter().find(|d| d.id == id) {
        return Ok((d, false, Vec::new()));
    }
    Err(format!(
        "no sequence with the id \"{id}\" (`gitpoulpi sequence list` shows them)"
    ))
}

fn show(id: &str, repo: Option<&str>) -> Res {
    let (def, trusted, extra) = find(id, repo)?;
    let human = format!(
        "{} ({}){}\n{}",
        def.name,
        def.scope.as_str(),
        if trusted { "" } else { "  not trusted" },
        def.steps
            .iter()
            .enumerate()
            .map(|(i, s)| format!("{:>3}. {}", i + 1, s.run))
            .collect::<Vec<_>>()
            .join("\n")
    );
    Ok(emitted(detailed(&def, trusted, &extra), human))
}

/// A TOML file's sequences: `[[sequences]]`, or one sequence at the top level.
fn parse_file(path: &str) -> Result<Vec<SequenceDef>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("could not read {path}: {e}"))?;
    let table: toml::Table = toml::from_str(&text).map_err(|e| format!("{path}: {e}"))?;
    if let Some(list) = table.get("sequences") {
        return list
            .clone()
            .try_into::<Vec<SequenceDef>>()
            .map_err(|e| format!("{path}: {e}"));
    }
    toml::Value::Table(table)
        .try_into::<SequenceDef>()
        .map(|d| vec![d])
        .map_err(|e| format!("{path}: not a sequence file ({e})"))
}

fn validate(id: Option<&str>, file: Option<&str>, repo: Option<&str>) -> Res {
    let entries: Vec<Value> = match (id, file) {
        (Some(id), _) => {
            let (def, trusted, extra) = find(id, repo)?;
            vec![entry(&def, trusted, &extra)]
        }
        (None, Some(path)) => {
            let defs = parse_file(path)?;
            let dups = duplicates(&defs);
            defs.iter()
                .zip(&dups)
                .map(|(d, extra)| entry(d, true, extra))
                .collect()
        }
        (None, None) => all_entries(repo)?.0,
    };
    let valid = entries
        .iter()
        .all(|e| e["problems"].as_array().is_some_and(Vec::is_empty));
    let human = if valid {
        format!("{} sequence(s) valid", entries.len())
    } else {
        entries
            .iter()
            .flat_map(|e| {
                e["problems"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(move |p| {
                        format!(
                            "{}: {}",
                            e["id"].as_str().unwrap_or(""),
                            p["reason"].as_str().unwrap_or("")
                        )
                    })
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    Ok(Emitted {
        json: json!({ "valid": valid, "sequences": entries }),
        human,
        exit_code: if valid { 0 } else { 2 },
    })
}

fn scope_of(arg: SequenceScopeArg) -> Scope {
    match arg {
        SequenceScopeArg::Installation => Scope::Installation,
        SequenceScopeArg::Machine => Scope::Machine,
        SequenceScopeArg::Repo => Scope::Repo,
    }
}

/// The ids already stored in the place `scope` names (for this installation).
fn stored_ids(config: &Config, install_id: &str, scope: Scope, root: Option<&Path>) -> Vec<String> {
    match scope {
        Scope::Repo => root
            .and_then(|r| sequences::peek_repo_sequences(r).ok())
            .unwrap_or_default()
            .into_iter()
            .map(|d| d.id)
            .collect(),
        Scope::Machine => config
            .sequences
            .iter()
            .filter(|d| d.scope == Scope::Machine)
            .map(|d| d.id.clone())
            .collect(),
        Scope::Installation => config
            .sequences
            .iter()
            .filter(|d| d.scope == Scope::Installation && d.owner.as_deref() == Some(install_id))
            .map(|d| d.id.clone())
            .collect(),
    }
}

fn add(file: &str, replace: bool, scope: Option<SequenceScopeArg>, repo: Option<&str>) -> Res {
    let config_path = config_file()?;
    let config = load_config(&config_path)?;
    let install_id = installation::current().identity.id.clone();
    let root = repo_root(repo);
    let mut defs = parse_file(file)?;
    if defs.is_empty() {
        return Err(format!("{file} holds no sequence"));
    }
    for d in &mut defs {
        if let Some(s) = scope {
            d.scope = scope_of(s);
        }
        // The writer fills the owner; one a file carries is not trusted.
        d.owner = None;
    }
    let visible_now = visible(&config, &install_id);
    let mut problems: Vec<Value> = Vec::new();
    for (i, d) in defs.iter().enumerate() {
        let mut found: Vec<SequenceProblem> = sequences::validate(d);
        if defs[..i].iter().any(|o| o.id == d.id) {
            found.push(problem(None, "duplicate_id", "the file has this id twice"));
        }
        let here = stored_ids(&config, &install_id, d.scope, root.as_deref());
        if here.contains(&d.id) && !replace {
            found.push(problem(
                None,
                "exists",
                format!(
                    "a {} sequence \"{}\" already exists: pass --replace to overwrite it",
                    d.scope.as_str(),
                    d.id
                ),
            ));
        } else if !here.contains(&d.id) && visible_now.iter().any(|v| v.id == d.id) {
            found.push(problem(
                None,
                "duplicate_id",
                format!("another visible sequence already uses the id \"{}\"", d.id),
            ));
        }
        if d.scope == Scope::Repo && root.is_none() {
            found.push(problem(
                None,
                "no_repository",
                "a repo-scope sequence needs a repository (run inside one or pass --repo)",
            ));
        }
        problems.extend(found.iter().map(|p| {
            let mut v = problem_json(&d.steps, p);
            v["id"] = json!(d.id);
            v
        }));
    }
    if !problems.is_empty() {
        let human = problems
            .iter()
            .map(|p| {
                format!(
                    "{}: {}",
                    p["id"].as_str().unwrap_or(""),
                    p["reason"].as_str().unwrap_or("")
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        return Ok(Emitted {
            json: json!({ "applied": false, "problems": problems }),
            human,
            exit_code: 2,
        });
    }
    for d in &defs {
        sequences::save_sequence(d, &config_path, &install_id, root.as_deref())
            .map_err(|e| format!("could not save \"{}\": {e}", d.id))?;
    }
    let saved: Vec<Value> = defs
        .iter()
        .map(|d| json!({ "id": d.id, "scope": d.scope.as_str() }))
        .collect();
    Ok(emitted(
        json!({ "applied": true, "saved": saved }),
        format!("saved {} sequence(s)", defs.len()),
    ))
}

fn remove(id: &str, scope: Option<SequenceScopeArg>, repo: Option<&str>) -> Res {
    let config_path = config_file()?;
    let config = load_config(&config_path)?;
    let install_id = installation::current().identity.id.clone();
    let root = repo_root(repo);
    let scope = match scope {
        Some(s) => scope_of(s),
        None => [Scope::Installation, Scope::Machine]
            .into_iter()
            .find(|s| {
                stored_ids(&config, &install_id, *s, None)
                    .iter()
                    .any(|i| i == id)
            })
            .unwrap_or(Scope::Installation),
    };
    if scope != Scope::Repo && config.sequences.iter().filter(|d| d.id == id).count() > 1 {
        return Err(format!(
            "several sequences in config.toml share the id \"{id}\": edit the file by hand"
        ));
    }
    let mine = stored_ids(&config, &install_id, scope, root.as_deref())
        .iter()
        .any(|i| i == id);
    let removed = mine
        && sequences::remove_sequence(scope, id, &config_path, root.as_deref())
            .map_err(|e| e.to_string())?;
    Ok(Emitted {
        json: json!({ "id": id, "scope": scope.as_str(), "removed": removed }),
        human: if removed {
            format!("removed {id}")
        } else {
            format!("no {} sequence \"{id}\"", scope.as_str())
        },
        exit_code: if removed { 0 } else { 2 },
    })
}

// --------------------------------------------------------------------- run

/// Runs one resolved command through the shared `execute`.
async fn run_command(
    ipc: &mut Session,
    session: &str,
    repo: Option<&str>,
    args: &[String],
) -> Result<Emitted, String> {
    let mut full = vec!["gitpoulpi".to_string(), "--json".to_string()];
    if let Some(repo) = repo {
        full.push("--repo".into());
        full.push(repo.to_string());
    }
    full.extend(args.iter().cloned());
    let cli = Cli::try_parse_from(full).map_err(|e| e.to_string())?;
    Box::pin(super::execute(ipc, cli, session)).await
}

fn clock() -> (String, String) {
    let now = gitpoulpi_core::time::unix_now();
    let secs = now.rem_euclid(86_400);
    (
        gitpoulpi_core::time::civil_date(now),
        format!("{:02}-{:02}", secs / 3600, secs % 3600 / 60),
    )
}

async fn load_env(ipc: &mut Session, repo: Option<&str>) -> Result<Env, String> {
    let root = discover_repo(repo)?.root;
    let refs: Vec<Ref> = ipc.call(OperationKind::Refs, json!({})).await?;
    let mut branch = "HEAD".to_string();
    let mut branches = Vec::new();
    for r in &refs {
        if let RefKind::LocalBranch { name } = &r.kind {
            if r.is_head {
                branch = name.clone();
            }
            branches.push(name.clone());
        }
    }
    let (date, time) = clock();
    Ok(Env {
        repo_root: root,
        branch,
        branches,
        date,
        time,
        ai_message: None,
        stash_ai: false,
    })
}

/// The message of `ai commit-message`'s answer.
fn ai_message_of(json: &Value) -> Option<String> {
    let summary = json["summary"].as_str()?.trim();
    let description = json["description"].as_str().unwrap_or("").trim();
    Some(if description.is_empty() {
        summary.to_string()
    } else {
        format!("{summary}\n\n{description}")
    })
}

/// `gitpoulpi sequence run`: the one action that needs the daemon.
pub(crate) async fn run_via_daemon(
    ipc: &mut Session,
    session: &str,
    repo: Option<&str>,
    action: SequenceAction,
) -> Res {
    let SequenceAction::Run { id, yes, dry_run } = action else {
        return run_local(action, repo);
    };
    let (def, trusted, extra) = find(&id, repo)?;
    if !trusted {
        return Ok(refusal(
            &def,
            &[problem(
                None,
                "untrusted",
                "this sequence comes from the repository's .gitpoulpi/sequences.toml, which is not trusted: a repository file is somebody else's commands and never runs headless",
            )],
        ));
    }
    let mut problems = sequences::validate(&def);
    problems.extend(extra);
    problems.extend(headless_problems(&def));
    if !problems.is_empty() {
        return Ok(refusal(&def, &problems));
    }
    let asks = confirmations(&def);
    if !dry_run && !yes && !asks.is_empty() {
        let lines: Vec<String> = asks
            .iter()
            .map(|c| format!("step {}: {}", c.step + 1, c.reason))
            .collect();
        return Ok(Emitted {
            json: json!({
                "sequence": def.id,
                "applied": false,
                "needs_confirmation": asks
                    .iter()
                    .map(|c| json!({ "step": c.step, "run": c.run, "reason": c.reason }))
                    .collect::<Vec<_>>(),
                "hint": "run again with --yes to accept these steps as proposed",
            }),
            human: format!(
                "{} needs confirmation, nothing changed (pass --yes to accept as proposed):\n{}",
                def.id,
                lines.join("\n")
            ),
            exit_code: 3,
        });
    }

    let mut env = load_env(ipc, repo).await?;
    if dry_run {
        return dry_run_plan(&def, &mut env, &asks);
    }

    let mut steps: Vec<Value> = Vec::new();
    let mut stopped_at: Option<usize> = None;
    for (i, step) in def.steps.iter().enumerate() {
        if stopped_at.is_some() {
            steps.push(json!({ "index": i, "run": step.run, "status": "skipped" }));
            continue;
        }
        let build = mapping(&step.run).ok_or_else(|| format!("no mapping for {}", step.run))?;
        let outcome: Result<Value, Value> = match build(step, &mut env) {
            Err(e) => Err(json!(e)),
            Ok(Action::DeferToStash) => {
                env.stash_ai = true;
                Ok(json!({ "deferred": "the next stash_push names the stash with the AI" }))
            }
            Ok(Action::AiMessage) => {
                match run_command(ipc, session, repo, &["ai".into(), "commit-message".into()]).await
                {
                    Err(e) => Err(json!(e)),
                    Ok(em) if em.exit_code != 0 => Err(json!(em.human)),
                    Ok(em) => match ai_message_of(&em.json) {
                        Some(message) => {
                            env.ai_message = Some(message);
                            Ok(em.json)
                        }
                        None => Err(json!("the AI answered without a message")),
                    },
                }
            }
            Ok(Action::Argv(args)) => match run_command(ipc, session, repo, &args).await {
                Err(e) => Err(json!(e)),
                Ok(em) if em.exit_code != 0 => Err(json!(em.human)),
                Ok(em) => Ok(em.json),
            },
        };
        match outcome {
            Ok(result) => {
                steps.push(
                    json!({ "index": i, "run": step.run, "status": "done", "result": result }),
                );
            }
            Err(error) => {
                steps.push(
                    json!({ "index": i, "run": step.run, "status": "failed", "error": error }),
                );
                stopped_at = Some(i);
            }
        }
    }
    let human = steps
        .iter()
        .map(|s| {
            format!(
                "[{}] {}. {}{}",
                s["status"].as_str().unwrap_or(""),
                s["index"].as_u64().unwrap_or(0) + 1,
                s["run"].as_str().unwrap_or(""),
                s["error"]
                    .as_str()
                    .map(|e| format!(": {e}"))
                    .unwrap_or_default()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    Ok(Emitted {
        json: json!({
            "sequence": def.id,
            "steps": steps,
            "stopped_at": stopped_at,
            "applied": true,
        }),
        human,
        exit_code: if stopped_at.is_some() { 2 } else { 0 },
    })
}

fn dry_run_plan(def: &SequenceDef, env: &mut Env, asks: &[Confirmation]) -> Res {
    let mut steps: Vec<Value> = Vec::new();
    let mut problems: Vec<SequenceProblem> = Vec::new();
    for (i, step) in def.steps.iter().enumerate() {
        let build = mapping(&step.run).ok_or_else(|| format!("no mapping for {}", step.run))?;
        let (command, note): (Option<Vec<String>>, Option<&str>) = match build(step, env) {
            Err(e) => {
                problems.push(problem(Some(i), "unresolvable", e));
                (None, None)
            }
            Ok(Action::DeferToStash) => {
                env.stash_ai = true;
                (
                    None,
                    Some("the next stash_push names the stash with the AI"),
                )
            }
            Ok(Action::AiMessage) => {
                env.ai_message = Some(AI_PLACEHOLDER.to_string());
                (
                    Some(vec!["ai".into(), "commit-message".into()]),
                    Some("the message is the next commit's"),
                )
            }
            Ok(Action::Argv(args)) => (Some(args), None),
        };
        steps.push(json!({
            "index": i,
            "run": step.run,
            "command": command.map(|c| format!("gitpoulpi {}", c.join(" "))),
            "note": note,
            "needs_confirmation": asks.iter().any(|c| c.step == i),
        }));
    }
    if !problems.is_empty() {
        return Ok(refusal(def, &problems));
    }
    let human = steps
        .iter()
        .map(|s| {
            format!(
                "{:>3}. {}",
                s["index"].as_u64().unwrap_or(0) + 1,
                s["command"]
                    .as_str()
                    .or_else(|| s["note"].as_str())
                    .unwrap_or("")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    Ok(emitted(
        json!({
            "sequence": def.id,
            "dry_run": true,
            "applied": false,
            "steps": steps,
            "needs_yes": !asks.is_empty(),
        }),
        human,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_config::sequences::STEP_VOCABULARY;

    #[test]
    fn every_cli_step_has_a_mapping() {
        for kind in STEP_VOCABULARY.iter().filter(|k| k.cli) {
            assert!(mapping(kind.id).is_some(), "no mapping for {}", kind.id);
        }
    }

    #[test]
    fn no_mapping_names_a_step_the_vocabulary_lacks() {
        for (id, _) in MAPPING {
            let kind = step_kind(id).unwrap_or_else(|| panic!("{id} is not a step"));
            assert!(kind.cli, "{id} is mapped but not marked cli");
        }
    }
}
