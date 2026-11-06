//! `gitpoulpi flow start|finish|sync|promote|set-parent`: plan, confirm, run.
//!
//! Every action asks the daemon for its plan first (`FlowPlan`: the steps,
//! blockers, warnings, hooks), shows it, and runs it (`FlowRun`) only when
//! told to: `--yes`, or an answer on a terminal. Anything that is not a
//! terminal (a script, an MCP agent) gets the plan back with exit code 3 and
//! nothing changed. Hooks are shell commands from a file a team shares: they
//! run only with `--run-hooks`, or when a terminal says yes after seeing them.

use std::io::{BufRead, IsTerminal, Write};

use gitpoulpi_core::domain::OperationKind;
use gitpoulpi_core::workflow::{
    Fields, FinishMode, FlowAction as Act, FlowOutcome, FlowPlan, FlowRequest, FlowResume,
    FlowStatus, HooksReport, HooksTrust, StepStatus, TrustHooksArgs,
};
use gitpoulpi_core::TypedError;
use serde_json::{json, Value};

use crate::cli::{FlowResumeArgs, FlowRunArgs};
use crate::client::Session;

use super::*;

/// What a finish forwards to the pull/merge request.
#[derive(Default)]
#[cfg_attr(not(feature = "hosting"), allow(dead_code))]
pub(crate) struct MrArgs {
    pub draft: bool,
    pub body: Option<String>,
    pub assignees: Vec<String>,
    pub reviewers: Vec<String>,
    pub labels: Vec<String>,
    pub milestone: Option<String>,
    pub integration: Option<String>,
}

/// `name=value` words; one bare word is the `slug`.
pub(super) fn parse_fields(words: &[String]) -> Result<Fields, String> {
    let mut fields = Fields::new();
    for word in words {
        match word.split_once('=') {
            Some((name, value)) if !name.trim().is_empty() => {
                fields.insert(name.trim().to_string(), value.to_string());
            }
            Some(_) => return Err(format!("`{word}` has no field name before `=`")),
            None => {
                if fields.contains_key("slug") {
                    return Err(format!(
                        "`{word}`: only one bare word is taken as the slug (use name=value)"
                    ));
                }
                fields.insert("slug".to_string(), word.clone());
            }
        }
    }
    Ok(fields)
}

pub(super) fn finish_mode(word: &str) -> Option<FinishMode> {
    match word {
        "local-merge" => Some(FinishMode::LocalMerge),
        "merge-request" => Some(FinishMode::MergeRequest),
        _ => None,
    }
}

/// The plan as lines for a terminal.
fn render_plan(plan: &FlowPlan) -> String {
    let mut out = vec![plan.title.clone()];
    if plan.steps.is_empty() && plan.can_run() {
        out.push("  nothing to do".to_string());
    }
    for (i, step) in plan.steps.iter().enumerate() {
        out.push(format!("  {}. {}", i + 1, step.describe()));
        for command in step.git() {
            out.push(format!("       $ {command}"));
        }
    }
    for hook in plan.hooks.iter().filter(|_| !plan.hooks_run) {
        out.push(format!("  hook not run ({}): {}", hook.phase, hook.command));
    }
    for line in &plan.irreversible {
        out.push(format!("  not undoable: {line}"));
    }
    for line in &plan.warnings {
        out.push(format!("  warning: {line}"));
    }
    for line in &plan.blockers {
        out.push(format!("  blocked: {line}"));
    }
    out.join("\n")
}

fn render_outcome(outcome: &FlowOutcome) -> String {
    let mut out = Vec::new();
    for report in &outcome.steps {
        let mark = match report.status {
            StepStatus::Done => "done",
            StepStatus::Skipped => "skipped",
            StepStatus::Failed => "FAILED",
            StepStatus::Conflicted => "CONFLICT",
            StepStatus::Pending => "pending",
            StepStatus::NotRun => "not run",
        };
        out.push(format!("[{mark}] {}", report.step.describe()));
        if let Some(text) = report.output.as_deref().filter(|t| !t.trim().is_empty()) {
            // A hook's output is shown whole, indented under its step.
            for line in text.trim_end().lines() {
                out.push(format!("        {line}"));
            }
        }
    }
    out.push(outcome.message.clone());
    out.join("\n")
}

/// Whether the action needs a go-ahead: the ones that merge, rewrite,
/// push or run hooks do. Recording a parent, aborting a stopped flow and a
/// plain local start do not.
fn needs_confirmation(req: &FlowRequest, plan: &FlowPlan) -> bool {
    if plan.steps.is_empty() || req.resume == Some(FlowResume::Abort) {
        return false;
    }
    match req.action {
        // Local and undoable: recording a parent, taking over a branch.
        Act::SetParent | Act::PickUp => false,
        Act::Start => !plan.irreversible.is_empty(),
        _ => true,
    }
}

fn ask(prompt: &str) -> Result<bool, String> {
    eprint!("{prompt}");
    let _ = std::io::stderr().flush();
    let mut line = String::new();
    std::io::stdin()
        .lock()
        .read_line(&mut line)
        .map_err(|e| format!("could not read the answer: {e}"))?;
    Ok(matches!(line.trim().to_lowercase().as_str(), "y" | "yes"))
}

/// The commands of an untrusted plan, for a refusal or a question.
fn untrusted_of(plan: &FlowPlan) -> Option<&str> {
    match &plan.hooks_trust {
        HooksTrust::Untrusted { hash, .. } if !plan.hooks.is_empty() => Some(hash),
        _ => None,
    }
}

/// `--run-hooks` on committed hooks nobody trusted: nothing runs, the
/// commands and the way to trust them come back.
fn hooks_refusal(plan: &FlowPlan, hash: &str) -> Emitted {
    let trust_command = format!("gitpoulpi flow trust-hooks --hash {hash}");
    let hooks: Vec<Value> = plan
        .hooks
        .iter()
        .map(|h| json!({ "phase": h.phase, "command": h.command }))
        .collect();
    let mut human = vec![
        "the repository's workflow hooks are not trusted, nothing was run:".to_string(),
    ];
    for hook in &plan.hooks {
        human.push(format!("  {}: {}", hook.phase, hook.command));
    }
    if let HooksTrust::Untrusted {
        previous: Some(_), ..
    } = &plan.hooks_trust
    {
        human.push("the hooks changed since you trusted them".to_string());
    }
    human.push(format!(
        "read them, then run `{trust_command}` yourself (or repeat without --run-hooks)"
    ));
    Emitted {
        json: json!({ "refused": {
            "reason": "hooks_not_trusted",
            "hash": hash,
            "hooks": hooks,
            "trust_command": trust_command,
        } }),
        human: human.join("\n"),
        exit_code: 2,
    }
}

/// `flow trust-hooks [--hash H]`: shows the committed hooks and whether they
/// are trusted; with the hash, trusts them. Only a person (the CLI's own
/// session) may.
pub(crate) async fn trust_hooks(
    session: &mut Session,
    hash: Option<String>,
) -> Result<Emitted, String> {
    if hash.is_some() && !session.may_trust() {
        return Err(
            "only a person can trust workflow hooks: run `gitpoulpi flow trust-hooks --hash H` \
             yourself"
                .into(),
        );
    }
    let report: HooksReport = session
        .call(
            OperationKind::FlowTrustHooks,
            to_value(&TrustHooksArgs { hash })?,
        )
        .await?;
    let mut human = Vec::new();
    if report.hooks.is_empty() {
        human.push("the repository's workflow file commits no hooks".to_string());
    } else {
        for hook in &report.hooks {
            human.push(format!("{}: {}", hook.phase, hook.command));
        }
        human.push(if report.trusted {
            format!("trusted (hash {})", report.hash)
        } else {
            format!(
                "not trusted: `gitpoulpi flow trust-hooks --hash {}` after reading them",
                report.hash
            )
        });
    }
    Ok(Emitted {
        json: to_value(&report)?,
        human: human.join("\n"),
        exit_code: 0,
    })
}

fn args_json(req: &FlowRequest, run: &FlowRunArgs) -> Result<Value, String> {
    Ok(with_autostash(to_value(req)?, run.autostash))
}

pub(crate) async fn run_action(
    session: &mut Session,
    mut req: FlowRequest,
    run: &FlowRunArgs,
    resume: Option<&FlowResumeArgs>,
    mr: Option<MrArgs>,
) -> Result<Emitted, String> {
    if let Some(r) = resume {
        req.resume = if r.continue_ {
            Some(FlowResume::Continue)
        } else if r.abort {
            Some(FlowResume::Abort)
        } else {
            None
        };
    }
    req.run_hooks = run.run_hooks;
    let mut plan: FlowPlan = session
        .call(OperationKind::FlowPlan, args_json(&req, run)?)
        .await?;

    if run.dry_run {
        let exit_code = if plan.can_run() { 0 } else { 1 };
        return Ok(Emitted {
            json: json!({ "plan": plan }),
            human: render_plan(&plan),
            exit_code,
        });
    }
    if !plan.can_run() {
        return Err(format!("{}: {}", plan.title, plan.blockers.join("; ")));
    }

    // Committed hooks nobody trusted never run on an explicit `--run-hooks`:
    // the refusal names them and the command a person runs to trust them.
    if req.run_hooks {
        if let Some(hash) = untrusted_of(&plan) {
            return Ok(hooks_refusal(&plan, hash));
        }
    }

    let terminal = std::io::stdin().is_terminal()
        && std::io::stdout().is_terminal()
        && std::io::stderr().is_terminal();
    // Hooks never run silently: a terminal sees them and says yes.
    if !plan.hooks.is_empty() && !req.run_hooks && !run.no_hooks && terminal {
        eprintln!("The workflow configures hooks (shell commands, run in the repository):");
        for hook in &plan.hooks {
            eprintln!("  {}: {}", hook.phase, hook.command);
        }
        let untrusted = untrusted_of(&plan).map(str::to_string);
        let question = if untrusted.is_some() {
            "These hooks are not trusted yet. Trust them and run them too? [y/N] "
        } else {
            "Run them too? [y/N] "
        };
        if ask(question)? {
            if let Some(hash) = untrusted {
                if !session.may_trust() {
                    return Err("only a person can trust workflow hooks".into());
                }
                // The person just read the commands: remember the trust (the
                // daemon refuses when the file changed since the plan).
                let _: HooksReport = session
                    .call(
                        OperationKind::FlowTrustHooks,
                        to_value(&TrustHooksArgs { hash: Some(hash) })?,
                    )
                    .await?;
            }
            req.run_hooks = true;
            plan = session
                .call(OperationKind::FlowPlan, args_json(&req, run)?)
                .await?;
        }
    }
    if needs_confirmation(&req, &plan) && !run.yes {
        if terminal {
            eprintln!("{}", render_plan(&plan));
            if !ask("Run these steps? [y/N] ")? {
                return Ok(Emitted {
                    json: json!({ "cancelled": true, "plan": plan }),
                    human: "cancelled, nothing was changed".to_string(),
                    exit_code: 1,
                });
            }
        } else {
            let prompt = format!("{}: run these steps?", plan.title);
            return Ok(Emitted {
                json: json!({ "needs_confirmation": { "prompt": prompt, "plan": plan } }),
                human: format!(
                    "{}\nNothing was changed; repeat with --yes to go ahead (--dry-run only \
                     prints the plan).",
                    render_plan(&plan)
                ),
                exit_code: 3,
            });
        }
    }

    let outcome = match session
        .call_typed::<FlowOutcome>(OperationKind::FlowRun, args_json(&req, run)?)
        .await?
    {
        Ok(outcome) => outcome,
        Err(e) => {
            return Err(match e.kind {
                Some(TypedError::NeedsAutoStash { .. }) => format!(
                    "{} (local changes are in the way: repeat with --autostash to stash them \
                     and put them back)",
                    e.message
                ),
                _ => e.message,
            })
        }
    };
    let mut json = to_value(&outcome)?;
    let mut human = render_outcome(&outcome);
    let mut exit_code = match outcome.status {
        FlowStatus::Done | FlowStatus::NothingToDo | FlowStatus::Aborted => 0,
        FlowStatus::Conflicted => 2,
        FlowStatus::Failed => 1,
    };
    if let (FlowStatus::Done, Some(spec)) = (outcome.status, &outcome.merge_request) {
        match open_merge_request(session, spec, mr.unwrap_or_default()).await {
            Ok(opened) => {
                human.push_str(&format!("\n{}", opened.human));
                json["opened"] = opened.json;
            }
            Err(message) => {
                human.push_str(&format!(
                    "\nthe branch is pushed but the merge request could not be opened: {message}"
                ));
                json["open_error"] = json!(message);
                exit_code = 1;
            }
        }
    }
    Ok(Emitted {
        json,
        human,
        exit_code,
    })
}

#[cfg(feature = "hosting")]
async fn open_merge_request(
    session: &mut Session,
    spec: &gitpoulpi_core::workflow::MergeRequestSpec,
    mr: MrArgs,
) -> Result<Emitted, String> {
    let action = crate::cli::HostingAction::Pr {
        source: Some(spec.source.clone()),
        target: Some(spec.target.clone()),
        title: Some(spec.title.clone()),
        body: mr.body,
        draft: mr.draft,
        id: mr.integration,
        web: false,
        assignees: mr.assignees,
        reviewers: mr.reviewers,
        labels: mr.labels,
        milestone: mr.milestone,
    };
    crate::hosting_cmd::run_via_daemon(session, action).await
}

#[cfg(not(feature = "hosting"))]
async fn open_merge_request(
    _session: &mut Session,
    spec: &gitpoulpi_core::workflow::MergeRequestSpec,
    _mr: MrArgs,
) -> Result<Emitted, String> {
    Ok(emitted(
        json!({ "source": spec.source, "target": spec.target }),
        format!(
            "this build has no hosting integrations: open the request from {} into {} on your host",
            spec.source, spec.target
        ),
    ))
}
