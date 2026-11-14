//! Running a grouped push or pull from the CLI: `push --all-targets|--to`,
//! `pull --all-sources`, `remote-group plan|continue|abort|retry`.
//!
//! One path for all of them: plan (the daemon asks the remotes), settle
//! every row (its default when that is safe to run unattended, else a
//! `--resolve` decision), then `RemoteGroupRun`. Nothing ever resumes by
//! itself: a stop on conflicts is resumed with `remote-group continue`.
//!
//! Exit codes: 0 done, 2 refused (a saved run waits, detached HEAD, a
//! branch that is not checked out), 3 a decision or a confirmation is
//! needed (nothing changed), 4 some remote failed, was rejected, is
//! unknown or was not attempted, or a merge/rebase stopped on conflicts.

use gitpoulpi_config::PullButtonMode;
use gitpoulpi_core::domain::{
    GroupAction, GroupMode, GroupResume, GroupRunStatus, OperationKind, PlannedTarget,
    RemoteGroupOutcome, RemoteGroupPlan, RemoteGroupPlanRequest, RemoteGroupRunRequest,
    RemoteRelation, RunTarget, TargetRef, TargetResult, TargetStatus,
};
use serde_json::json;

use crate::cli::{PullModeArg, RemoteGroupAction};
use crate::client::Session;

use super::upstream::{branch_or_current, detached, refused, status};
use super::*;

/// The arguments of `push --all-targets|--to` and `pull --all-sources`.
pub(super) struct Group {
    pub mode: GroupMode,
    pub branch: Option<String>,
    pub to: Vec<String>,
    pub resolve: Vec<String>,
    pub no_preflight: bool,
    pub tags: bool,
    pub yes: bool,
    pub autostash: bool,
    /// Pull only: the `--mode` flag.
    pub pull_mode: Option<PullModeArg>,
}

/// `git.pull_mode` is rebase in the ambient config (merge when it cannot be
/// read): what a diverged target defaults to and how a pull integrates.
fn config_prefers_rebase() -> bool {
    gitpoulpi_config::Settings::resolve_ambient()
        .map(|s| s.config().git.pull_mode == PullButtonMode::Rebase)
        .unwrap_or(false)
}

fn action_word(a: GroupAction) -> &'static str {
    match a {
        GroupAction::Push => "push",
        GroupAction::FastForwardFrom => "ff",
        GroupAction::MergeFrom => "merge",
        GroupAction::RebaseOnto => "rebase",
        GroupAction::ForceWithLease => "force",
        GroupAction::Skip => "skip",
    }
}

fn parse_action(word: &str) -> Option<GroupAction> {
    Some(match word.trim() {
        "push" => GroupAction::Push,
        "ff" => GroupAction::FastForwardFrom,
        "merge" => GroupAction::MergeFrom,
        "rebase" => GroupAction::RebaseOnto,
        "force" => GroupAction::ForceWithLease,
        "skip" => GroupAction::Skip,
        _ => return None,
    })
}

/// `REMOTE=action`, the remote as `remote` or `remote:branch`.
fn parse_resolves(items: &[String]) -> Result<Vec<(String, GroupAction)>, Box<Emitted>> {
    items
        .iter()
        .map(|item| {
            let bad = || {
                Box::new(refused(
                    "bad_resolve",
                    format!(
                        "{item:?}: expected REMOTE=push|ff|merge|rebase|force|skip \
                         (REMOTE may be remote:branch)"
                    ),
                ))
            };
            let (who, what) = item.rsplit_once('=').ok_or_else(bad)?;
            let action = parse_action(what).ok_or_else(bad)?;
            if who.trim().is_empty() {
                return Err(bad());
            }
            Ok((who.trim().to_string(), action))
        })
        .collect()
}

fn matches_row(who: &str, row: &PlannedTarget) -> bool {
    match who.split_once(':') {
        Some((remote, branch)) => row.remote == remote && row.branch == branch,
        None => row.remote == who,
    }
}

fn describe_relation(row: &PlannedTarget) -> String {
    match &row.relation {
        RemoteRelation::UpToDate => "up to date".into(),
        RemoteRelation::LocalAhead(n) => format!("ahead {n}"),
        RemoteRelation::RemoteAhead(n) => format!("behind {n}"),
        RemoteRelation::Diverged { ahead, behind } => format!("diverged (+{ahead} -{behind})"),
        RemoteRelation::Absent => "not on the remote".into(),
        RemoteRelation::Unrelated => "unrelated history".into(),
    }
}

fn plan_human(plan: &RemoteGroupPlan) -> String {
    let verb = match plan.mode {
        GroupMode::Push => "push",
        GroupMode::Pull => "pull",
    };
    let mut human = format!("plan to {verb} {}:", plan.branch);
    for row in &plan.targets {
        let allowed: Vec<&str> = row.allowed.iter().map(|a| action_word(*a)).collect();
        human.push_str(&format!(
            "\n  {}:{}  {}  default {}  (allowed: {})",
            row.remote,
            row.branch,
            describe_relation(row),
            action_word(row.suggested),
            allowed.join(", ")
        ));
        if let Some(e) = &row.error {
            human.push_str(&format!("  [{e}]"));
        }
        if let Some(twin) = &row.same_url_as {
            human.push_str(&format!("  [same repository as {twin}]"));
        }
        if !row.conflicts.is_empty() {
            human.push_str(&format!(
                "  [would conflict in {}]",
                row.conflicts.join(", ")
            ));
        }
    }
    human
}

/// Whether a row's default may run without anyone looking at it.
fn default_is_unattended(row: &PlannedTarget, plan: &RemoteGroupPlan, ff_only: bool) -> bool {
    if row.error.is_some() || row.missing {
        return false;
    }
    match plan.mode {
        // Only plain pushes (and a twin of an earlier row, skipped): never a
        // merge, a rebase, a pull of someone else's commits or a force.
        GroupMode::Push => {
            row.suggested == GroupAction::Push
                || (row.suggested == GroupAction::Skip && row.same_url_as.is_some())
        }
        GroupMode::Pull => match row.suggested {
            GroupAction::MergeFrom | GroupAction::RebaseOnto => !ff_only,
            GroupAction::FastForwardFrom | GroupAction::Skip => true,
            _ => false,
        },
    }
}

fn status_word(s: &TargetStatus) -> String {
    match s {
        TargetStatus::NotAttempted => "not attempted".into(),
        TargetStatus::Integrating => "integrating".into(),
        TargetStatus::Pushing => "pushing".into(),
        TargetStatus::Integrated => "integrated".into(),
        TargetStatus::Pushed { created: true } => "pushed (new branch)".into(),
        TargetStatus::Pushed { created: false } => "pushed".into(),
        TargetStatus::UpToDate => "up to date".into(),
        TargetStatus::Skipped => "skipped".into(),
        TargetStatus::Rejected { reason } => format!("rejected ({reason:?})"),
        TargetStatus::Failed { kind } => format!("failed ({kind:?})"),
        TargetStatus::RemoteMissing => "remote missing".into(),
        TargetStatus::Unknown => "unknown (canceled mid-push)".into(),
    }
}

fn result_line(t: &TargetResult) -> String {
    let mut line = format!("  {}:{}  {}", t.remote, t.branch, status_word(&t.status));
    if let Some(old) = t.replaced {
        line.push_str(&format!("  (replaced {})", short(old)));
    }
    if let Some(d) = &t.detail {
        line.push_str(&format!("  {d}"));
    }
    line
}

fn outcome_emitted(outcome: RemoteGroupOutcome) -> Result<Emitted, String> {
    let mut human = outcome.message.clone();
    for t in &outcome.targets {
        human.push('\n');
        human.push_str(&result_line(t));
    }
    if let Some(s) = &outcome.stopped {
        human.push_str(&format!(
            "\nstopped on conflicts merging {}:{}: resolve, conclude it, then `gitpoulpi remote-group continue` (or `remote-group abort`)",
            s.remote, s.branch
        ));
    }
    if outcome.status == GroupRunStatus::NeedsResolution {
        let plan = outcome.plan.clone();
        let human = plan.as_ref().map(plan_human).unwrap_or(human);
        let mut out = emitted(
            json!({ "needs_resolution": { "run_id": outcome.run_id, "plan": plan } }),
            human,
        );
        out.exit_code = 3;
        return Ok(out);
    }
    let exit_code = match outcome.status {
        GroupRunStatus::Done | GroupRunStatus::Aborted => 0,
        _ => 4,
    };
    let mut out = emitted(to_value(&outcome)?, human);
    out.exit_code = exit_code;
    Ok(out)
}

/// Sends a `RemoteGroupRun`; a refusal of the daemon (a saved run waits, a
/// branch not checked out, a merge still in progress) is exit 2.
async fn send_run(session: &mut Session, args: serde_json::Value) -> Result<Emitted, String> {
    match session
        .call_typed::<RemoteGroupOutcome>(OperationKind::RemoteGroupRun, args)
        .await?
    {
        Ok(outcome) => outcome_emitted(outcome),
        Err(e) => {
            if let Some(gitpoulpi_core::TypedError::RemoteGroupPending { branch, run_id }) = &e.kind
            {
                let mut out = refused("remote_group_pending", e.message.clone());
                out.json["refused"]["branch"] = json!(branch);
                out.json["refused"]["run_id"] = json!(run_id);
                return Ok(out);
            }
            let reason = if e.message.contains("must be checked out") {
                "checked_out_elsewhere"
            } else {
                "refused"
            };
            Ok(refused(reason, e.message))
        }
    }
}

async fn plan_call(
    session: &mut Session,
    req: &RemoteGroupPlanRequest,
) -> Result<RemoteGroupPlan, String> {
    session
        .call(OperationKind::RemoteGroupPlan, to_value(req)?)
        .await
}

fn explicit_targets(to: &[String], branch: &str) -> Result<Option<Vec<TargetRef>>, Box<Emitted>> {
    if to.is_empty() {
        return Ok(None);
    }
    to.iter()
        .map(|t| {
            TargetRef::parse(t, branch)
                .ok_or_else(|| Box::new(refused("bad_target", format!("{t:?} names no remote"))))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

/// `push --all-targets|--to` and `pull --all-sources`.
pub(super) async fn run_group(session: &mut Session, g: Group) -> Result<Emitted, String> {
    let Some(branch) = branch_or_current(session, g.branch.clone()).await? else {
        return Ok(detached());
    };
    // A saved run waits: nothing new starts until it is continued or aborted
    // (answered before planning, which would ask the remotes for nothing).
    if let Some(pending) = status(session, &branch).await?.pending {
        let mut out = refused(
            "remote_group_pending",
            format!(
                "a grouped push or pull of {} is waiting (run {}): `remote-group continue` or `remote-group abort` it first",
                pending.branch, pending.run_id
            ),
        );
        out.json["refused"]["branch"] = json!(pending.branch);
        out.json["refused"]["run_id"] = json!(pending.run_id);
        return Ok(out);
    }
    let resolves = match parse_resolves(&g.resolve) {
        Ok(r) => r,
        Err(e) => return Ok(*e),
    };
    let targets = match explicit_targets(&g.to, &branch) {
        Ok(t) => t,
        Err(e) => return Ok(*e),
    };
    let (prefer_rebase, ff_only) = match (g.mode, g.pull_mode) {
        (GroupMode::Pull, Some(PullModeArg::Rebase)) => (true, false),
        (GroupMode::Pull, Some(PullModeArg::FfOnly)) => (false, true),
        (GroupMode::Pull, _) => (false, false),
        (GroupMode::Push, _) => (config_prefers_rebase(), false),
    };
    let plan = plan_call(
        session,
        &RemoteGroupPlanRequest {
            branch: branch.clone(),
            mode: g.mode,
            targets,
            preflight: !g.no_preflight,
            prefer_rebase,
            refused: Vec::new(),
        },
    )
    .await?;
    if plan.targets.is_empty() {
        return Ok(refused(
            "no_targets",
            format!(
                "{branch} has no remote to {}",
                match g.mode {
                    GroupMode::Push => "push to: set an upstream or `branch upstream add` one",
                    GroupMode::Pull => "pull from: set an upstream",
                }
            ),
        ));
    }
    // Every `--resolve` must name a row and an action that row allows.
    for (who, action) in &resolves {
        match plan.targets.iter().find(|r| matches_row(who, r)) {
            None => {
                return Ok(refused(
                    "unknown_target",
                    format!("--resolve {who}: {who} is not one of the plan's remotes"),
                ))
            }
            Some(row) if !row.allowed.contains(action) => {
                let allowed: Vec<&str> = row.allowed.iter().map(|a| action_word(*a)).collect();
                return Ok(refused(
                    "action_not_allowed",
                    format!(
                        "--resolve {who}={}: allowed for {who} are {}",
                        action_word(*action),
                        allowed.join(", ")
                    ),
                ));
            }
            Some(_) => {}
        }
    }
    let mut decisions: Vec<RunTarget> = Vec::new();
    let mut open = false;
    for row in &plan.targets {
        let explicit = resolves
            .iter()
            .rev()
            .find(|(who, _)| matches_row(who, row))
            .map(|(_, a)| *a);
        if explicit.is_none() && !default_is_unattended(row, &plan, ff_only) {
            open = true;
        }
        decisions.push(RunTarget::from_planned(
            row,
            explicit.unwrap_or(row.suggested),
        ));
    }
    if open {
        let mut out = emitted(
            json!({ "needs_resolution": { "plan": plan } }),
            format!(
                "{}\nnothing changed: decide the remotes that are not a plain push with \
                 --resolve REMOTE=push|ff|merge|rebase|force|skip",
                plan_human(&plan)
            ),
        );
        out.exit_code = 3;
        return Ok(out);
    }
    let forced: Vec<&str> = decisions
        .iter()
        .filter(|d| d.action == GroupAction::ForceWithLease)
        .map(|d| d.remote.as_str())
        .collect();
    if !forced.is_empty() && !g.yes {
        let prompt = format!(
            "force push (with lease) to {} replaces what is there; run it again with --yes to go ahead",
            forced.join(", ")
        );
        let mut out = emitted(
            json!({ "needs_confirmation": { "prompt": prompt, "forced": forced, "plan": plan } }),
            prompt.clone(),
        );
        out.exit_code = 3;
        return Ok(out);
    }
    let req = RemoteGroupRunRequest {
        branch,
        mode: g.mode,
        targets: decisions,
        tags: g.tags,
        prefer_rebase,
        ..RemoteGroupRunRequest::default()
    };
    send_run(session, with_autostash(to_value(&req)?, g.autostash)).await
}

/// `remote-group plan|continue|abort|retry` (`status` lives in `upstream`).
pub(super) async fn remote_group(
    session: &mut Session,
    action: RemoteGroupAction,
) -> Result<Emitted, String> {
    match action {
        RemoteGroupAction::Status { .. } => unreachable!("handled by upstream::remote_group"),
        RemoteGroupAction::Plan {
            branch,
            to,
            pull,
            no_preflight,
        } => {
            let Some(branch) = branch_or_current(session, branch).await? else {
                return Ok(detached());
            };
            let targets = match explicit_targets(&to, &branch) {
                Ok(t) => t,
                Err(e) => return Ok(*e),
            };
            let mode = if pull {
                GroupMode::Pull
            } else {
                GroupMode::Push
            };
            let plan = plan_call(
                session,
                &RemoteGroupPlanRequest {
                    branch,
                    mode,
                    targets,
                    preflight: !no_preflight,
                    prefer_rebase: config_prefers_rebase(),
                    refused: Vec::new(),
                },
            )
            .await?;
            Ok(emitted(to_value(&plan)?, plan_human(&plan)))
        }
        RemoteGroupAction::Continue {
            run,
            resolve,
            yes,
            branch,
        } => {
            let mut req = RemoteGroupRunRequest {
                resume: Some(GroupResume::Continue),
                expect_run_id: run,
                prefer_rebase: config_prefers_rebase(),
                ..RemoteGroupRunRequest::default()
            };
            if !resolve.is_empty() {
                let resolves = match parse_resolves(&resolve) {
                    Ok(r) => r,
                    Err(e) => return Ok(*e),
                };
                let Some(branch) = branch_or_current(session, branch).await? else {
                    return Ok(detached());
                };
                let Some(pending) = status(session, &branch).await?.pending else {
                    return Ok(refused(
                        "nothing_pending",
                        "no grouped push or pull is waiting to be continued",
                    ));
                };
                // The lease and the relation come from a fresh plan of the
                // saved run's own targets.
                let refs: Vec<TargetRef> = pending
                    .targets
                    .iter()
                    .map(|t| TargetRef {
                        remote: t.remote.clone(),
                        branch: t.branch.clone(),
                    })
                    .collect();
                let plan = plan_call(
                    session,
                    &RemoteGroupPlanRequest {
                        branch: pending.branch.clone(),
                        mode: pending.mode,
                        targets: Some(refs),
                        preflight: true,
                        prefer_rebase: req.prefer_rebase,
                        refused: Vec::new(),
                    },
                )
                .await?;
                let mut forced = Vec::new();
                for (who, action) in &resolves {
                    let Some(row) = plan.targets.iter().find(|r| matches_row(who, r)) else {
                        return Ok(refused(
                            "unknown_target",
                            format!("--resolve {who}: not a target of the saved run"),
                        ));
                    };
                    if !row.allowed.contains(action) {
                        return Ok(refused(
                            "action_not_allowed",
                            format!("--resolve {who}={}: not allowed now", action_word(*action)),
                        ));
                    }
                    if *action == GroupAction::ForceWithLease {
                        forced.push(row.remote.clone());
                    }
                    req.targets.push(RunTarget::from_planned(row, *action));
                }
                if !forced.is_empty() && !yes {
                    let prompt = format!(
                        "force push (with lease) to {} replaces what is there; run it again with --yes to go ahead",
                        forced.join(", ")
                    );
                    let mut out = emitted(
                        json!({ "needs_confirmation": { "prompt": prompt, "forced": forced } }),
                        prompt.clone(),
                    );
                    out.exit_code = 3;
                    return Ok(out);
                }
            }
            send_run(session, to_value(&req)?).await
        }
        RemoteGroupAction::Abort { run } => {
            let req = RemoteGroupRunRequest {
                resume: Some(GroupResume::Abort),
                expect_run_id: run,
                ..RemoteGroupRunRequest::default()
            };
            send_run(session, to_value(&req)?).await
        }
        RemoteGroupAction::Retry { run } => {
            let req = RemoteGroupRunRequest {
                retry: true,
                expect_run_id: run,
                prefer_rebase: config_prefers_rebase(),
                ..RemoteGroupRunRequest::default()
            };
            send_run(session, to_value(&req)?).await
        }
    }
}
