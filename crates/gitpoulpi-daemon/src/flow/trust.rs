//! Trust for the workflow hooks a repository commits: shell commands from
//! `.gitpoulpi/workflow.toml` run on the user's machine, so they run only
//! while the hash of that file's hooks is in the repository state's
//! `trusted_hooks` (a changed file asks again). Hooks of the user's own
//! override (`RepoState::workflow.hooks`) never ask: the user wrote them.
//!
//! The planner reads the file once (`facts::gather`), so the hash checked
//! here and the commands that run come from the same read: there is no gap
//! between check and use. A resumed flow runs commands saved before, so each
//! one must appear in a trusted record (or be the user's own now).

use gitpoulpi_core::domain::RepoRef;
use gitpoulpi_core::workflow::{
    hooks_hash, FlowFacts, FlowPlan, FlowStep, HookInfo, Hooks, HooksReport, HooksTrust,
    TrustHooksArgs,
};
use gitpoulpi_core::Error;
use gitpoulpi_session::{HookLine, LoadedWorkflow, RepoState};

use crate::dispatch::Failure;

fn load_state(repo: &RepoRef) -> RepoState {
    RepoState::load(&crate::autostash::state_path(repo)).unwrap_or_default()
}

fn info(h: &HookLine) -> HookInfo {
    HookInfo {
        phase: h.phase.clone(),
        command: h.command.clone(),
        personal: false,
    }
}

fn infos(hooks: &Hooks) -> Vec<HookInfo> {
    hooks
        .lines()
        .into_iter()
        .map(|(phase, command)| HookInfo {
            phase: phase.to_string(),
            command: command.to_string(),
            personal: false,
        })
        .collect()
}

/// Whether the hooks the plan lists may run: personal, trusted as they are
/// now, or to ask about.
pub(super) fn classify(repo: &RepoRef, facts: &FlowFacts, plan: &FlowPlan) -> HooksTrust {
    if plan.hooks.is_empty() {
        return HooksTrust::Trusted;
    }
    if facts.hooks_personal {
        return HooksTrust::Personal;
    }
    let Some(hash) = facts.shared_hooks_hash.clone() else {
        return HooksTrust::Trusted;
    };
    let state = load_state(repo);
    if state.trusts_hooks(&hash) {
        return HooksTrust::Trusted;
    }
    let previous = state
        .trusted_hooks
        .iter()
        .max_by_key(|t| t.trusted_at)
        .map(|t| t.hooks.iter().map(info).collect());
    HooksTrust::Untrusted { hash, previous }
}

fn not_trusted(hash: String, hooks: Vec<HookInfo>) -> Failure {
    let e = Error::HooksNotTrusted { hash, hooks };
    (e.to_string(), e.typed(), None)
}

/// The refusal of a fresh run: the plan runs hooks and the committed ones
/// are not trusted. Nothing has run.
pub(super) fn check_plan(plan: &FlowPlan) -> Result<(), Failure> {
    let runs_shared_hooks = plan.steps.iter().any(|s| {
        matches!(
            s,
            FlowStep::Hook {
                personal: false,
                ..
            }
        )
    });
    match &plan.hooks_trust {
        HooksTrust::Untrusted { hash, .. } if runs_shared_hooks => {
            Err(not_trusted(hash.clone(), plan.hooks.clone()))
        }
        _ => Ok(()),
    }
}

/// The refusal of a resume: every hook still to run must be trusted (its
/// command in a trusted record) or be one of the user's own now. A saved
/// state is a file in the git directory, so a command put there is checked
/// like a committed one.
pub(super) fn check_resume(repo: &RepoRef, steps: &[FlowStep]) -> Result<(), Failure> {
    let hooks: Vec<&FlowStep> = steps
        .iter()
        .filter(|s| matches!(s, FlowStep::Hook { .. }))
        .collect();
    if hooks.is_empty() {
        return Ok(());
    }
    let state = load_state(repo);
    let loaded = LoadedWorkflow::load(&repo.root, &crate::workflow::state_dir()).ok();
    let own = loaded
        .as_ref()
        .and_then(|l| l.user.as_ref())
        .and_then(|u| u.hooks.as_ref());
    let mut refused = Vec::new();
    for step in hooks {
        let FlowStep::Hook {
            phase,
            command,
            personal,
        } = step
        else {
            continue;
        };
        let trusted = state.trusts_hook_command(phase, command);
        let own_now = *personal
            && own.is_some_and(|h| h.lines().iter().any(|(p, c)| p == phase && c == command));
        if !trusted && !own_now {
            refused.push(HookInfo {
                phase: phase.clone(),
                command: command.clone(),
                personal: false,
            });
        }
    }
    if refused.is_empty() {
        return Ok(());
    }
    let hash = loaded
        .and_then(|l| l.shared)
        .map(|s| hooks_hash(&s.workflow.hooks))
        .unwrap_or_default();
    Err(not_trusted(hash, refused))
}

/// `FlowTrustHooks`. Without a hash it only reports. With one it re-reads
/// the committed file and trusts the hooks only when their hash is exactly
/// the one given, so what the user looked at is what gets trusted.
pub(crate) fn trust_hooks(repo: &RepoRef, args: &serde_json::Value) -> Result<HooksReport, Error> {
    let parsed: TrustHooksArgs = serde_json::from_value(args.clone())
        .map_err(|e| Error::Backend(format!("invalid arguments: {e}")))?;
    let loaded =
        LoadedWorkflow::load(&repo.root, &crate::workflow::state_dir()).map_err(Error::Backend)?;
    let hooks = loaded.shared.map(|s| s.workflow.hooks).unwrap_or_default();
    let hash = if hooks.is_empty() {
        String::new()
    } else {
        hooks_hash(&hooks)
    };
    let path = crate::autostash::state_path(repo);
    let mut state = RepoState::load(&path).unwrap_or_default();
    if let Some(wanted) = parsed.hash {
        if hash.is_empty() || !wanted.eq_ignore_ascii_case(&hash) {
            return Err(Error::Backend(
                "the workflow hooks on disk are not the ones that were shown (their hash \
                 differs): review them again"
                    .into(),
            ));
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs() as i64);
        let lines = hooks
            .lines()
            .into_iter()
            .map(|(phase, command)| HookLine {
                phase: phase.to_string(),
                command: command.to_string(),
            })
            .collect();
        state.trust_hooks(&hash, lines, now);
        state
            .save(&path)
            .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?;
        tracing::info!(target: "daemon", "trusted the workflow hooks ({hash})");
    }
    Ok(HooksReport {
        trusted: !hash.is_empty() && state.trusts_hooks(&hash),
        hooks: infos(&hooks),
        hash,
    })
}
