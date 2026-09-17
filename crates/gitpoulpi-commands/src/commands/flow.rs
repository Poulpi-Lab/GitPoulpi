//! `gitpoulpi flow` handlers: the read-only side of branching workflows.

use gitpoulpi_core::domain::OperationKind;
use gitpoulpi_core::workflow::{
    BranchKind, BranchReport, FlowAction as Act, FlowRequest, SyncMethod, Workflow, WorkflowReport,
    WorkflowTemplate,
};
use serde_json::json;

use crate::cli::{FlowAction, FlowConfigAction};
use crate::client::Session;

use super::flow_run::{finish_mode, parse_fields, run_action, MrArgs};
use super::*;

/// `flow templates`: answers from the built-in presets, no repository.
pub(super) fn templates() -> Result<Emitted, String> {
    let mut items = Vec::new();
    let mut lines = Vec::new();
    for t in WorkflowTemplate::ALL {
        let w = t.workflow();
        lines.push(format!("{:<12} {}", t.id(), t.description()));
        for b in &w.bases {
            let names: Vec<_> = b.all_names().collect();
            let parent = b
                .parent
                .as_deref()
                .map(|p| format!(" (from {p})"))
                .unwrap_or_default();
            lines.push(format!("    base   {}{parent}", names.join(" | ")));
        }
        for tp in &w.topics {
            let parent = tp
                .parent
                .as_deref()
                .map(|p| format!(" (from {p})"))
                .unwrap_or_default();
            lines.push(format!("    topic  {:<9} {}{parent}", tp.id, tp.pattern));
        }
        items.push(json!({
            "id": t.id(),
            "label": t.label(),
            "description": t.description(),
            "workflow": w,
        }));
    }
    Ok(emitted(json!({ "templates": items }), lines.join("\n")))
}

async fn report(session: &mut Session) -> Result<WorkflowReport, String> {
    session.call(OperationKind::Workflow, json!({})).await
}

fn toml_of(workflow: &Workflow) -> Result<String, String> {
    gitpoulpi_session::render_toml(workflow, None).map_err(|e| e.to_string())
}

fn state_dir() -> String {
    gitpoulpi_config::Settings::resolve_ambient()
        .map(|s| s.config().session.per_repo_state_dir.clone())
        .unwrap_or_else(|_| gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION.to_string())
}

fn workflow_file(session: &Session) -> Result<std::path::PathBuf, String> {
    let root = session
        .repo_root()
        .ok_or("not inside a git repository")?
        .to_path_buf();
    Ok(gitpoulpi_session::workflow_path_for(&root, &state_dir()))
}

/// Prints `workflow` as TOML; with `write`, creates the file when absent.
fn print_or_write(
    session: &Session,
    workflow: &Workflow,
    write: bool,
    extra: serde_json::Value,
) -> Result<Emitted, String> {
    let text = toml_of(workflow)?;
    let path = workflow_file(session)?;
    let mut json = json!({
        "path": path,
        "toml": text,
        "written": false,
        "workflow": workflow,
    });
    if let (Some(o), Some(e)) = (json.as_object_mut(), extra.as_object()) {
        o.extend(e.clone());
    }
    if !write {
        return Ok(emitted(json, text));
    }
    let created =
        gitpoulpi_session::save_shared(&path, workflow, false).map_err(|e| e.to_string())?;
    if !created {
        return Err(format!(
            "{} already exists; not overwritten (edit it, or delete it first)",
            path.display()
        ));
    }
    json["written"] = json!(true);
    Ok(emitted(json, format!("{text}\nwrote {}", path.display())))
}

fn describe(b: &BranchReport) -> String {
    let name = match &b.remote {
        Some(r) => format!("{r}/{}", b.name),
        None => b.name.clone(),
    };
    let kind = match b.kind {
        BranchKind::Base => "base".to_string(),
        BranchKind::Topic => format!("topic {}", b.type_id.as_deref().unwrap_or("?")),
        BranchKind::Unclassified => "-".to_string(),
    };
    let parent = b
        .parent
        .as_ref()
        .map(|p| format!("parent {}", p.name))
        .unwrap_or_default();
    let counts = match (b.ahead, b.behind) {
        (Some(a), Some(bh)) => format!("ahead {a}, behind {bh}"),
        _ => String::new(),
    };
    let head = if b.is_head { "*" } else { " " };
    format!("{head} {name:<32} {kind:<18} {parent:<24} {counts}")
        .trim_end()
        .to_string()
}

pub(super) async fn flow(session: &mut Session, action: FlowAction) -> Result<Emitted, String> {
    match action {
        FlowAction::Templates => templates(),
        FlowAction::Start {
            r#type,
            fields,
            from,
            publish,
            run,
        } => {
            let req = FlowRequest {
                action: Act::Start,
                topic: Some(r#type),
                fields: parse_fields(&fields)?,
                from,
                publish,
                ..FlowRequest::default()
            };
            run_action(session, req, &run, None, None).await
        }
        FlowAction::Finish {
            branch,
            mode,
            title,
            delete,
            keep,
            delete_remote,
            remote,
            allow_behind,
            draft,
            body,
            assignees,
            reviewers,
            labels,
            milestone,
            integration,
            run,
            resume,
        } => {
            let req = FlowRequest {
                action: Act::Finish,
                branch,
                finish_mode: mode.as_deref().and_then(finish_mode),
                title,
                delete: if delete {
                    Some(true)
                } else if keep {
                    Some(false)
                } else {
                    None
                },
                delete_remote,
                remote,
                allow_behind,
                ..FlowRequest::default()
            };
            let mr = MrArgs {
                draft,
                body,
                assignees,
                reviewers,
                labels,
                milestone,
                integration,
            };
            run_action(session, req, &run, Some(&resume), Some(mr)).await
        }
        FlowAction::Sync {
            branch,
            stack,
            rebase,
            merge,
            run,
            resume,
        } => {
            let req = FlowRequest {
                action: Act::Sync,
                branch,
                stack,
                method: if rebase {
                    Some(SyncMethod::Rebase)
                } else if merge {
                    Some(SyncMethod::Merge)
                } else {
                    None
                },
                ..FlowRequest::default()
            };
            run_action(session, req, &run, Some(&resume), None).await
        }
        FlowAction::Promote {
            branch,
            allow_behind,
            run,
            resume,
        } => {
            let req = FlowRequest {
                action: Act::Promote,
                branch: Some(branch),
                allow_behind,
                ..FlowRequest::default()
            };
            run_action(session, req, &run, Some(&resume), None).await
        }
        FlowAction::SetParent {
            branch,
            parent,
            clear,
            run,
        } => {
            let req = FlowRequest {
                action: Act::SetParent,
                branch: Some(branch),
                parent,
                clear,
                ..FlowRequest::default()
            };
            run_action(session, req, &run, None, None).await
        }
        FlowAction::Issue { action } => issue(session, action).await,
        FlowAction::PickUp {
            target,
            remote,
            parent,
            no_fetch,
            integration,
            run,
        } => pick_up(session, target, remote, parent, no_fetch, integration, run).await,
        FlowAction::Status => {
            let r = report(session).await?;
            let mut lines = vec![if r.configured {
                let mut from = Vec::new();
                if r.sources.shared_file {
                    from.push("shared file");
                }
                if r.sources.user_override {
                    from.push("user override");
                }
                format!("workflow: {} ({})", r.template.id(), from.join(" + "))
            } else {
                let hint = r.detections.first().map_or(String::new(), |d| {
                    format!(
                        "; closest template: {} (`flow config init --template {}`)",
                        d.template.id(),
                        d.template.id()
                    )
                });
                format!("no workflow configured{hint}")
            }];
            lines.extend(r.branches.iter().map(describe));
            lines.extend(r.warnings.iter().map(|w| format!("warning: {w}")));
            for i in &r.imports {
                lines.push(format!(
                    "found {} settings: `flow import --from {}` turns them into a workflow",
                    i.source, i.source
                ));
            }
            Ok(emitted(
                serde_json::to_value(&r).map_err(|e| e.to_string())?,
                lines.join("\n"),
            ))
        }
        FlowAction::Detect => {
            let r = report(session).await?;
            let mut lines = Vec::new();
            for d in &r.detections {
                let bases: Vec<String> = d
                    .bases
                    .iter()
                    .map(|b| match &b.found {
                        Some(f) if *f == b.base => f.clone(),
                        Some(f) => format!("{f} (for {})", b.base),
                        None => format!("{} (missing)", b.base),
                    })
                    .collect();
                let topics: Vec<String> = d
                    .topics
                    .iter()
                    .map(|t| format!("{} x{}", t.id, t.count))
                    .collect();
                lines.push(format!(
                    "{:<12} {:>3}  bases: {}{}",
                    d.template.id(),
                    d.score,
                    if bases.is_empty() {
                        "-".to_string()
                    } else {
                        bases.join(", ")
                    },
                    if topics.is_empty() {
                        String::new()
                    } else {
                        format!("; topics: {}", topics.join(", "))
                    }
                ));
            }
            Ok(emitted(
                json!({ "detections": r.detections }),
                lines.join("\n"),
            ))
        }
        FlowAction::Config {
            show_effective,
            action,
        } => match action {
            None => config_show(session, show_effective).await,
            Some(FlowConfigAction::Show { show_effective: e }) => {
                config_show(session, show_effective || e).await
            }
            Some(FlowConfigAction::Init { template, write }) => {
                let t = WorkflowTemplate::from_id(&template)
                    .ok_or_else(|| format!("unknown template `{template}`"))?;
                print_or_write(session, &t.workflow(), write, json!({ "template": t.id() }))
            }
        },
        FlowAction::Import { from, write } => {
            let r = report(session).await?;
            let found = r
                .imports
                .into_iter()
                .find(|i| i.source == from)
                .ok_or_else(|| format!("no {from} settings found in this repository"))?;
            let notes = found.notes.clone();
            let mut out = print_or_write(
                session,
                &found.workflow,
                write,
                json!({ "source": found.source, "notes": notes }),
            )?;
            if !notes.is_empty() {
                out.human = format!(
                    "{}\n{}",
                    out.human,
                    notes
                        .iter()
                        .map(|n| format!("note: {n}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                );
            }
            Ok(out)
        }
    }
}

async fn config_show(session: &mut Session, effective: bool) -> Result<Emitted, String> {
    let path = workflow_file(session)?;
    if effective {
        let r = report(session).await?;
        let text = toml_of(&r.workflow)?;
        return Ok(emitted(
            json!({
                "path": path,
                "effective": true,
                "configured": r.configured,
                "sources": r.sources,
                "workflow": r.workflow,
                "toml": text,
                "warnings": r.warnings,
            }),
            if r.configured {
                text
            } else {
                "no workflow configured".to_string()
            },
        ));
    }
    let loaded =
        gitpoulpi_session::load_shared(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    match loaded {
        Some(shared) => {
            let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            Ok(emitted(
                json!({
                    "path": path,
                    "exists": true,
                    "workflow": shared.workflow,
                    "unknown_keys": shared.unknown_keys,
                    "toml": text,
                }),
                text,
            ))
        }
        None => Ok(emitted(
            json!({ "path": path, "exists": false, "workflow": Workflow::default(), "toml": "" }),
            format!(
                "no workflow file at {} (`flow detect` proposes a template, `flow config init --template <t> --write` creates it)",
                path.display()
            ),
        )),
    }
}

#[cfg(feature = "hosting")]
async fn issue(
    session: &mut Session,
    action: crate::cli::FlowIssueAction,
) -> Result<Emitted, String> {
    use crate::cli::FlowIssueAction as Issue;
    use crate::hosting_cmd::flow_issue as fi;
    match action {
        Issue::New {
            r#type,
            title,
            body,
            body_file,
            labels,
            assignees,
            milestone,
            wait,
            pick_up,
            timeout,
            integration,
            run,
        } => {
            fi::issue_new(
                session,
                fi::NewArgs {
                    topic: r#type,
                    title,
                    body,
                    body_file,
                    labels,
                    assignees,
                    milestone,
                    wait,
                    pick_up,
                    timeout,
                    integration,
                    run,
                },
            )
            .await
        }
        Issue::Wait {
            reference,
            r#type,
            wait_for,
            timeout,
            once,
            pick_up,
            integration,
            run,
        } => {
            fi::issue_wait(
                session,
                fi::WaitArgs {
                    reference,
                    topic: r#type,
                    wait_for,
                    timeout,
                    once,
                    pick_up,
                    integration,
                    run,
                },
            )
            .await
        }
    }
}

#[cfg(feature = "hosting")]
async fn pick_up(
    session: &mut Session,
    target: String,
    remote: Option<String>,
    parent: Option<String>,
    no_fetch: bool,
    integration: Option<String>,
    run: crate::cli::FlowRunArgs,
) -> Result<Emitted, String> {
    crate::hosting_cmd::flow_issue::pick_up(
        session,
        crate::hosting_cmd::flow_issue::PickUpArgs {
            target,
            remote,
            parent,
            no_fetch,
            integration,
            run,
        },
    )
    .await
}

#[cfg(not(feature = "hosting"))]
async fn issue(
    _session: &mut Session,
    _action: crate::cli::FlowIssueAction,
) -> Result<Emitted, String> {
    Err("this build has no hosting integrations, so `flow issue` is not available".to_string())
}

#[cfg(not(feature = "hosting"))]
async fn pick_up(
    session: &mut Session,
    target: String,
    remote: Option<String>,
    parent: Option<String>,
    _no_fetch: bool,
    _integration: Option<String>,
    run: crate::cli::FlowRunArgs,
) -> Result<Emitted, String> {
    // Without a host the target can only be a branch name.
    let req = FlowRequest {
        action: Act::PickUp,
        branch: Some(target),
        parent,
        remote,
        ..FlowRequest::default()
    };
    run_action(session, req, &run, None, None).await
}
