//! `gitpoulpi rebase` handlers.

use gitpoulpi_core::domain::{
    short, InteractiveRebase, OperationKind, RebaseOptions, RebaseOutcome, RebaseState, RebaseStep,
};
use serde_json::json;

use crate::client::Session;

use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) async fn rebase_command(
    session: &mut Session,
    onto: Option<String>,
    interactive: bool,
    plan: Option<String>,
    root: bool,
    onto_commit: Option<String>,
    autosquash: bool,
    continue_: bool,
    skip: bool,
    abort: bool,
    status: bool,
    autostash: bool,
) -> Result<Emitted, String> {
    if status {
        let state: Option<RebaseState> =
            session.call(OperationKind::RebaseState, json!({})).await?;
        let human = match &state {
            Some(s) => format!(
                "rebase in progress: step {}/{}{}",
                s.step,
                s.total,
                if s.conflicted { " (conflicted)" } else { "" }
            ),
            None => "no rebase in progress".to_string(),
        };
        return Ok(emitted(to_value(&state)?, human));
    }
    if continue_ {
        let outcome: RebaseOutcome = session
            .call(OperationKind::RebaseContinue, json!({}))
            .await?;
        let human = rebase_outcome_human(&outcome);
        return Ok(emitted(to_value(&outcome)?, human));
    }
    if skip {
        let outcome: RebaseOutcome = session.call(OperationKind::RebaseSkip, json!({})).await?;
        let human = rebase_outcome_human(&outcome);
        return Ok(emitted(to_value(&outcome)?, human));
    }
    if abort {
        session
            .call::<()>(OperationKind::RebaseAbort, json!({}))
            .await?;
        return Ok(emitted(
            json!({ "aborted": true }),
            "rebase aborted".to_string(),
        ));
    }

    if root && onto.is_some() {
        return Err("rebase: give either <onto> or --root, not both".into());
    }

    if let Some(plan) = plan {
        let steps = read_plan_steps(&plan)?;
        let upstream = match &onto {
            Some(o) if !root => Some(resolve(session, o).await?),
            _ => None,
        };
        let onto_commit = match onto_commit {
            Some(o) => Some(resolve(session, &o).await?),
            None => None,
        };
        let outcome: RebaseOutcome = session
            .call(
                OperationKind::RebaseInteractive,
                json!({
                    "request": InteractiveRebase {
                        upstream,
                        onto: onto_commit,
                        steps,
                        autosquash,
                        cherry_pick: false,
                    },
                }),
            )
            .await?;
        let human = rebase_outcome_human(&outcome);
        return Ok(emitted(to_value(&outcome)?, human));
    }

    if interactive && !autosquash {
        return Err(
            "interactive rebase needs --plan (there is no interactive editor here) - \
             build one with `rebase-plan`, edit the actions, then pass it back"
                .into(),
        );
    }

    let onto_name =
        onto.ok_or("rebase: needs <onto>, --root, or one of --continue/--skip/--abort/--status")?;
    let onto_ref = find_ref(session, &onto_name).await?;
    let outcome: RebaseOutcome = session
        .call(
            OperationKind::Rebase,
            with_autostash(
                json!({ "onto": onto_ref, "opts": RebaseOptions { autosquash } }),
                autostash,
            ),
        )
        .await?;
    let human = rebase_outcome_human(&outcome);
    Ok(emitted(to_value(&outcome)?, human))
}

/// Reads a `--plan` argument's `Vec<RebaseStep>` JSON: `-` means stdin,
/// anything else is a file path - the same convention `patch apply`
/// and every other "read this JSON from somewhere" flag in this CLI use.
pub(super) fn read_plan_steps(plan: &str) -> Result<Vec<RebaseStep>, String> {
    let raw = if plan == "-" {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| format!("failed to read the plan from stdin: {e}"))?;
        buf
    } else {
        std::fs::read_to_string(plan).map_err(|e| format!("failed to read {plan}: {e}"))?
    };
    serde_json::from_str(&raw).map_err(|e| format!("plan is not a valid `Vec<RebaseStep>`: {e}"))
}

pub(super) fn rebase_outcome_human(outcome: &RebaseOutcome) -> String {
    match outcome {
        RebaseOutcome::Completed => "rebased".to_string(),
        RebaseOutcome::Conflicted { paused_at, .. } => {
            format!("paused on a conflict at {}", short(*paused_at))
        }
        RebaseOutcome::Stopped { at, .. } => {
            format!("stopped at {} (edit)", short(*at))
        }
    }
}
