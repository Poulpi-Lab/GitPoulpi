//! `gitpoulpi flow issue new|wait` and `flow pick-up`: host-driven flows.
//!
//! The daemon has no hosting credentials, so the host half runs here (the
//! shared engine is `gitpoulpi_hosting::flow_engine`) and the git half goes
//! through the daemon as the same `FlowPlan`/`FlowRun` requests every other
//! flow action uses (`action: pick_up`), so the plan, the confirmation rules,
//! the journal entry and Undo are the ones of `flow start`.

use std::time::{Duration, Instant};

use gitpoulpi_core::domain::{FetchOptions, FetchOutcome, OperationKind, Ref, RefKind};
use gitpoulpi_core::workflow::{
    issue_draft, FlowAction, FlowRequest, IssueFlow, IssueFlowMode, WaitFor, Workflow,
    WorkflowReport,
};
use gitpoulpi_hosting::flow_engine::{self, Found};
use gitpoulpi_hosting::HostingClient;
use serde_json::{json, Value};

use crate::cli::FlowRunArgs;
use crate::client::Session;
use crate::commands::flow_run::run_action;
use crate::repo_cmd::Emitted;

use super::*;

/// Exit code of a wait that ended without what it waited for.
pub const EXIT_TIMED_OUT: i32 = 4;

/// The arguments of `flow issue new`.
pub struct NewArgs {
    pub topic: String,
    pub title: String,
    pub body: Option<String>,
    pub body_file: Option<std::path::PathBuf>,
    pub labels: Vec<String>,
    pub assignees: Vec<String>,
    pub milestone: Option<String>,
    pub wait: bool,
    pub pick_up: bool,
    pub timeout: Option<u64>,
    pub integration: Option<String>,
    pub run: FlowRunArgs,
}

/// The arguments of `flow issue wait`.
pub struct WaitArgs {
    pub reference: String,
    pub topic: Option<String>,
    pub wait_for: Option<String>,
    pub timeout: Option<u64>,
    pub once: bool,
    pub pick_up: bool,
    pub integration: Option<String>,
    pub run: FlowRunArgs,
}

/// The arguments of `flow pick-up`.
pub struct PickUpArgs {
    pub target: String,
    pub remote: Option<String>,
    pub parent: Option<String>,
    pub no_fetch: bool,
    pub integration: Option<String>,
    pub run: FlowRunArgs,
}

/// What every step needs: the workflow, the hosts, the clients.
struct Ctx {
    workflow: Workflow,
    flow: IssueFlow,
    /// Where issues live and where branches and requests live (often the
    /// same project).
    issue: RepoTarget,
    code: RepoTarget,
    issue_client: Box<dyn HostingClient>,
    code_client: Box<dyn HostingClient>,
    remote_branches: Vec<String>,
}

fn type_ids(workflow: &Workflow) -> Vec<String> {
    workflow.topics.iter().map(|t| t.id.clone()).collect()
}

async fn load_workflow(session: &mut Session) -> Result<Workflow, String> {
    let report: WorkflowReport = session.call(OperationKind::Workflow, json!({})).await?;
    Ok(report.workflow)
}

async fn remote_branch_names(session: &mut Session, remote: &str) -> Result<Vec<String>, String> {
    let refs: Vec<Ref> = session.call(OperationKind::Refs, json!({})).await?;
    Ok(refs
        .into_iter()
        .filter_map(|r| match r.kind {
            RefKind::RemoteBranch { remote: rm, name } if rm == remote && name != "HEAD" => {
                Some(name)
            }
            _ => None,
        })
        .collect())
}

async fn context(
    env: &HostingEnv,
    session: &mut Session,
    id: Option<&str>,
    need_issues: bool,
) -> Result<Ctx, String> {
    let workflow = load_workflow(session).await?;
    let flow = workflow.issue_flow.clone().unwrap_or_default();
    let issue = match repo_target(env, session, id, Which::Issues).await {
        Ok(t) => t,
        Err(e) if need_issues => return Err(e),
        Err(_) => repo_target(env, session, id, Which::Pulls).await?,
    };
    let code = match repo_target(env, session, id, Which::Pulls).await {
        Ok(t) => t,
        Err(_) => RepoTarget {
            integration: issue.integration.clone(),
            integrations: issue.integrations.clone(),
            project: issue.project.clone(),
            remote: issue.remote.clone(),
            ambiguous: issue.ambiguous,
            linked: issue.linked,
            filter: None,
        },
    };
    let issue_client = env.client(&issue.integration, &issue.integrations)?;
    let code_client = env.client(&code.integration, &code.integrations)?;
    let remote = default_remote_name(session).await.unwrap_or_default();
    let remote_branches = if remote.is_empty() {
        Vec::new()
    } else {
        remote_branch_names(session, &remote).await?
    };
    Ok(Ctx {
        workflow,
        flow,
        issue,
        code,
        issue_client,
        code_client,
        remote_branches,
    })
}

fn issue_number(reference: &str) -> Result<u64, String> {
    reference
        .trim()
        .trim_start_matches('#')
        .parse::<u64>()
        .map_err(|_| format!("`{reference}` is not an issue number (use `#12` or `12`)"))
}

fn env_user() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_default()
}

/// The first existing remote branch among the topic's parent base names.
fn parent_on_remote(ctx: &Ctx, topic: &str) -> Option<String> {
    let compiled = ctx.workflow.compile();
    let parent = ctx
        .workflow
        .topics
        .iter()
        .find(|t| t.id == topic)?
        .parent
        .clone()?;
    compiled
        .concrete_base(&parent, &|n| ctx.remote_branches.iter().any(|b| b == n))
        .or_else(|| ctx.remote_branches.contains(&parent).then_some(parent))
}

fn watch(session: &Session, number: u64, topic: &str, title: &str) {
    let Some(root) = session.repo_root() else {
        return;
    };
    let path = crate::commands::repo_state_path(root);
    if let Ok(mut state) = gitpoulpi_session::RepoState::load(&path) {
        state.watch_issue(gitpoulpi_session::WatchedIssue {
            number,
            type_id: topic.to_string(),
            title: title.to_string(),
            created: gitpoulpi_core::time::unix_now(),
        });
        let _ = state.save(&path);
    }
}

fn unwatch(session: &Session, number: u64) {
    let Some(root) = session.repo_root() else {
        return;
    };
    let path = crate::commands::repo_state_path(root);
    if let Ok(mut state) = gitpoulpi_session::RepoState::load(&path) {
        if state.unwatch_issue(number) {
            let _ = state.save(&path);
        }
    }
}

fn found_json(found: &Found) -> Value {
    json!({
        "branches": found.branches,
        "requests": found.requests.iter().map(|r| json!({
            "number": r.number,
            "title": r.title,
            "state": r.state,
            "source_branch": r.source_branch,
            "target_branch": r.target_branch,
            "web_url": r.web_url,
        })).collect::<Vec<_>>(),
        "notes": found.notes,
    })
}

fn found_human(found: &Found) -> String {
    let mut out = Vec::new();
    for b in &found.branches {
        out.push(format!("branch {b}"));
    }
    for r in &found.requests {
        out.push(format!(
            "request {} {} -> {} {}",
            r.iid_label, r.source_branch, r.target_branch, r.web_url
        ));
    }
    if out.is_empty() {
        out.push("nothing yet".to_string());
    }
    out.join("\n")
}

/// Polls for what the host made until `wait` is satisfied or `timeout` is
/// over. Returns what was found and whether it ran out of time. Ctrl+C
/// cancels (an error). Progress goes to a terminal's stderr only.
async fn wait_loop(
    ctx: &Ctx,
    types: &[String],
    number: u64,
    wait: WaitFor,
    timeout: Duration,
    once: bool,
) -> Result<(Found, bool), String> {
    let started = Instant::now();
    let tty = {
        use std::io::IsTerminal;
        std::io::stderr().is_terminal()
    };
    let mut attempt = 0u32;
    let mut failures = 0u32;
    let mut last: Option<Found> = None;
    loop {
        match flow_engine::find_for_issue(
            &*ctx.code_client,
            &ctx.code.project,
            &ctx.flow,
            types,
            number,
        )
        .await
        {
            Ok(found) => {
                failures = 0;
                if found.satisfies(wait) {
                    return Ok((found, false));
                }
                last = Some(found);
            }
            // A hiccup after a first answer is retried; a host that never
            // answered is an error now.
            Err(e) if last.is_none() || failures >= 3 => return Err(e.to_string()),
            Err(_) => failures += 1,
        }
        let pause = flow_engine::backoff(attempt);
        if once || started.elapsed() + pause > timeout {
            return Ok((last.unwrap_or_default(), true));
        }
        if tty {
            eprint!(
                "\rwaiting for the host to make {} for #{number}... {}s ",
                wait.word(),
                started.elapsed().as_secs()
            );
        }
        tokio::select! {
            _ = tokio::time::sleep(pause) => {}
            _ = tokio::signal::ctrl_c() => {
                if tty { eprintln!(); }
                return Err("cancelled".to_string());
            }
        }
        attempt += 1;
    }
}

/// Fetches, then runs the `pick_up` flow action (plan, confirmation rules and
/// output of every flow action).
async fn pick_up_branch(
    session: &mut Session,
    branch: &str,
    parent: Option<String>,
    remote: Option<String>,
    fetch: bool,
    run: &FlowRunArgs,
) -> Result<Emitted, String> {
    let remote_name = match remote {
        Some(r) => r,
        None => default_remote_name(session).await?,
    };
    // A dry run fetches too: it only moves remote-tracking refs, and the plan
    // needs to see the branch the host made.
    if fetch {
        session
            .call::<FetchOutcome>(
                OperationKind::Fetch,
                json!({ "remote": remote_name, "opts": FetchOptions::default() }),
            )
            .await?;
    }
    let req = FlowRequest {
        action: FlowAction::PickUp,
        branch: Some(branch.to_string()),
        parent,
        remote: Some(remote_name),
        ..FlowRequest::default()
    };
    run_action(session, req, run, None, None).await
}

/// `flow issue new`.
pub async fn issue_new(session: &mut Session, args: NewArgs) -> Result<Emitted, String> {
    let env = HostingEnv::ambient()?;
    issue_new_with(&env, session, args).await
}

pub async fn issue_new_with(
    env: &HostingEnv,
    session: &mut Session,
    a: NewArgs,
) -> Result<Emitted, String> {
    if a.title.trim().is_empty() {
        return Err("an issue needs a title (--title)".to_string());
    }
    let ctx = context(env, session, a.integration.as_deref(), true).await?;
    let known = type_ids(&ctx.workflow);
    let topic = ctx
        .workflow
        .topics
        .iter()
        .find(|t| t.id == a.topic)
        .ok_or_else(|| {
            format!(
                "unknown branch type `{}` (known: {})",
                a.topic,
                if known.is_empty() {
                    "none: this repository has no workflow".to_string()
                } else {
                    known.join(", ")
                }
            )
        })?
        .clone();

    let template_text = match (&a.body, &a.body_file) {
        (Some(body), _) => Some(body.clone()),
        (None, Some(path)) => Some(
            std::fs::read_to_string(path)
                .map_err(|e| format!("could not read {}: {e}", path.display()))?,
        ),
        (None, None) => match ctx.flow.description_template.as_deref() {
            Some(rel) => {
                let root = session.repo_root().ok_or("not inside a git repository")?;
                let path = root.join(rel);
                Some(std::fs::read_to_string(&path).map_err(|e| {
                    format!("issue_flow.description_template {}: {e}", path.display())
                })?)
            }
            None => None,
        },
    };
    let mut draft = issue_draft(
        &ctx.flow,
        &topic.id,
        &a.title,
        template_text.as_deref(),
        &env_user(),
        &gitpoulpi_core::time::today(),
    );
    for label in words(a.labels.clone()) {
        if !draft.labels.contains(&label) {
            draft.labels.push(label);
        }
    }
    for who in people(a.assignees.clone()) {
        if !draft.assignees.contains(&who) {
            draft.assignees.push(who);
        }
    }

    let mode = ctx.flow.mode;
    let mode_word = match mode {
        IssueFlowMode::Local => "local",
        IssueFlowMode::HostCreatesBranch => "host-creates-branch",
        IssueFlowMode::Bot => "bot",
    };
    let wait = ctx.flow.wait_kind();
    let from = parent_on_remote(&ctx, &topic.id);
    let mut steps = vec![format!(
        "Create the issue \"{}\" on {} ({})",
        draft.title, ctx.issue.project, ctx.issue.integration.id
    )];
    match mode {
        IssueFlowMode::Local => {
            if a.pick_up {
                steps.push(format!("Start a {} branch from the issue", topic.id));
            }
        }
        IssueFlowMode::HostCreatesBranch => {
            steps.push(format!(
                "Create the branch on the host from {}",
                from.as_deref().unwrap_or("(no remote parent found)")
            ));
            if ctx.flow.wants_merge_request() {
                steps.push("Open a draft merge request that closes the issue".to_string());
            }
            if a.pick_up {
                steps.push("Fetch and pick the branch up".to_string());
            }
        }
        IssueFlowMode::Bot => {
            if a.wait || a.pick_up {
                steps.push(format!("Wait for the bot's {}", wait.word()));
            }
            if a.pick_up {
                steps.push("Pick up what the bot made".to_string());
            }
        }
    }
    let plan = json!({
        "mode": mode_word,
        "project": ctx.issue.project,
        "integration": ctx.issue.integration.id,
        "issue": draft,
        "branch_type": topic.id,
        "steps": steps,
    });
    if a.run.dry_run {
        return Ok(Emitted {
            json: json!({ "plan": plan }),
            human: steps
                .iter()
                .enumerate()
                .map(|(i, s)| format!("{}. {s}", i + 1))
                .collect::<Vec<_>>()
                .join("\n"),
            exit_code: 0,
        });
    }
    if mode == IssueFlowMode::HostCreatesBranch && from.is_none() {
        return Err(format!(
            "host-creates-branch needs the parent of `{}` on the remote (fetch, or fix the \
             type's parent)",
            topic.id
        ));
    }

    let milestone = match &a.milestone {
        Some(wanted) => {
            Some(resolve_milestone(&*ctx.issue_client, &ctx.issue.project, wanted).await?)
        }
        None => None,
    };
    let created =
        flow_engine::create_issue(&*ctx.issue_client, &ctx.issue.project, &draft, milestone)
            .await
            .map_err(|e| e.to_string())?;
    let mut human = format!("created issue {}: {}", created.reference, created.web_url);
    let mut out = json!({
        "issue": { "number": created.number, "reference": created.reference, "web_url": created.web_url },
        "mode": mode_word,
        "branch_type": topic.id,
    });
    watch(session, created.number, &topic.id, &draft.title);

    let timeout = a
        .timeout
        .map(Duration::from_secs)
        .unwrap_or_else(|| ctx.flow.timeout());
    let mut exit_code = 0;
    match mode {
        IssueFlowMode::Local => {
            if a.pick_up {
                let mut fields = gitpoulpi_core::workflow::Fields::new();
                fields.insert("issue".into(), created.number.to_string());
                fields.insert("slug".into(), a.title.clone());
                let req = FlowRequest {
                    action: FlowAction::Start,
                    topic: Some(topic.id.clone()),
                    fields,
                    ..FlowRequest::default()
                };
                let started = run_action(session, req, &a.run, None, None).await?;
                human.push_str(&format!("\n{}", started.human));
                out["started"] = started.json;
                exit_code = started.exit_code;
                unwatch(session, created.number);
            }
        }
        IssueFlowMode::HostCreatesBranch => {
            let from = from.unwrap_or_default();
            let name =
                flow_engine::issue_branch_name(&ctx.flow, &topic.id, created.number, &a.title)?;
            let made = flow_engine::host_creates_branch(
                &*ctx.code_client,
                &ctx.code.project,
                created.number,
                &name,
                &from,
                &from,
                &draft.title,
                ctx.flow.wants_merge_request(),
            )
            .await
            .map_err(|e| format!("the issue {} exists but: {e}", created.reference))?;
            human.push_str(&format!(
                "\ncreated branch {} on the host",
                made.branch.name
            ));
            out["branch"] = json!(made.branch.name);
            if let Some(mr) = &made.request {
                human.push_str(&format!(
                    "\nopened merge request {}: {}",
                    mr.number, mr.web_url
                ));
                out["merge_request"] = json!({
                    "number": mr.number,
                    "web_url": mr.web_url,
                    "target_branch": from,
                });
            }
            if let Some(w) = &made.warning {
                human.push_str(&format!("\nwarning: {w}"));
                out["warning"] = json!(w);
            }
            if a.pick_up {
                let picked = pick_up_branch(
                    session,
                    &made.branch.name,
                    Some(from.clone()),
                    None,
                    true,
                    &a.run,
                )
                .await?;
                human.push_str(&format!("\n{}", picked.human));
                out["picked_up"] = picked.json;
                exit_code = picked.exit_code;
                if exit_code == 0 {
                    unwatch(session, created.number);
                }
            }
        }
        IssueFlowMode::Bot => {
            if a.wait || a.pick_up {
                let (found, timed_out) =
                    wait_loop(&ctx, &known, created.number, wait, timeout, false).await?;
                out["found"] = found_json(&found);
                out["timed_out"] = json!(timed_out);
                human.push_str(&format!("\n{}", found_human(&found)));
                if timed_out {
                    human.push_str(&format!(
                        "\nthe bot has not made it yet; `flow issue wait {}` keeps waiting",
                        created.reference
                    ));
                    exit_code = EXIT_TIMED_OUT;
                } else if a.pick_up {
                    if let Some(branch) = found.best_branch().map(str::to_string) {
                        let parent = found.request_of(&branch).map(|r| r.target_branch.clone());
                        let picked =
                            pick_up_branch(session, &branch, parent, None, true, &a.run).await?;
                        human.push_str(&format!("\n{}", picked.human));
                        out["picked_up"] = picked.json;
                        exit_code = picked.exit_code;
                        if exit_code == 0 {
                            unwatch(session, created.number);
                        }
                    }
                }
            } else {
                human.push_str(&format!(
                    "\nthe bot makes the branch: `flow issue wait {}` waits for it",
                    created.reference
                ));
            }
        }
    }
    Ok(Emitted {
        json: out,
        human,
        exit_code,
    })
}

/// `flow issue wait`.
pub async fn issue_wait(session: &mut Session, args: WaitArgs) -> Result<Emitted, String> {
    let env = HostingEnv::ambient()?;
    issue_wait_with(&env, session, args).await
}

pub async fn issue_wait_with(
    env: &HostingEnv,
    session: &mut Session,
    a: WaitArgs,
) -> Result<Emitted, String> {
    let number = issue_number(&a.reference)?;
    let ctx = context(env, session, a.integration.as_deref(), false).await?;
    let types = match &a.topic {
        Some(t) => vec![t.clone()],
        None => type_ids(&ctx.workflow),
    };
    let wait = a
        .wait_for
        .as_deref()
        .map(WaitFor::parse)
        .unwrap_or_else(|| ctx.flow.wait_kind());
    let timeout = a
        .timeout
        .map(Duration::from_secs)
        .unwrap_or_else(|| ctx.flow.timeout());
    let (found, timed_out) = wait_loop(&ctx, &types, number, wait, timeout, a.once).await?;
    let mut out = json!({
        "issue": number,
        "waited_for": wait.word(),
        "found": found_json(&found),
        "timed_out": timed_out,
    });
    let mut human = found_human(&found);
    let mut exit_code = if timed_out { EXIT_TIMED_OUT } else { 0 };
    if !timed_out && a.pick_up {
        if let Some(branch) = found.best_branch().map(str::to_string) {
            let parent = found.request_of(&branch).map(|r| r.target_branch.clone());
            let picked = pick_up_branch(session, &branch, parent, None, true, &a.run).await?;
            human.push_str(&format!("\n{}", picked.human));
            out["picked_up"] = picked.json;
            exit_code = picked.exit_code;
            if exit_code == 0 {
                unwatch(session, number);
            }
        }
    }
    Ok(Emitted {
        json: out,
        human,
        exit_code,
    })
}

/// `flow pick-up`.
pub async fn pick_up(session: &mut Session, args: PickUpArgs) -> Result<Emitted, String> {
    let env = HostingEnv::ambient()?;
    pick_up_with(&env, session, args).await
}

pub async fn pick_up_with(
    env: &HostingEnv,
    session: &mut Session,
    a: PickUpArgs,
) -> Result<Emitted, String> {
    let target = a.target.trim();
    let mut parent = a.parent.clone();
    let mut issue: Option<u64> = None;
    let mut request: Option<Value> = None;
    let branch: String;
    if let Some(rest) = target.strip_prefix('#') {
        let number = issue_number(rest)?;
        let ctx = context(env, session, a.integration.as_deref(), false).await?;
        let found = flow_engine::find_for_issue(
            &*ctx.code_client,
            &ctx.code.project,
            &ctx.flow,
            &type_ids(&ctx.workflow),
            number,
        )
        .await
        .map_err(|e| e.to_string())?;
        branch = found
            .best_branch()
            .ok_or_else(|| {
                format!("nothing is known for #{number} yet (`flow issue wait #{number}` waits)")
            })?
            .to_string();
        if let Some(r) = found.request_of(&branch) {
            parent = parent.or_else(|| Some(r.target_branch.clone()));
            request = Some(
                json!({ "number": r.number, "web_url": r.web_url, "target_branch": r.target_branch }),
            );
        }
        issue = Some(number);
    } else if let Some(rest) = target
        .strip_prefix('!')
        .or_else(|| target.strip_prefix("mr:"))
        .or_else(|| target.strip_prefix("pr:"))
    {
        let number: u64 = rest
            .trim()
            .parse()
            .map_err(|_| format!("`{target}` is not a merge request number (use `!5`)"))?;
        let ctx = context(env, session, a.integration.as_deref(), false).await?;
        let detail = ctx
            .code_client
            .pull_request_detail(&ctx.code.project, number)
            .await
            .map_err(|e| e.to_string())?;
        branch = detail.summary.source_branch.clone();
        parent = parent.or_else(|| Some(detail.summary.target_branch.clone()));
        request = Some(json!({
            "number": number,
            "web_url": detail.summary.web_url,
            "target_branch": detail.summary.target_branch,
        }));
    } else {
        branch = target.to_string();
        // The request of the branch gives the parent; no host is not an error.
        if parent.is_none() {
            if let Ok(ctx) = context(env, session, a.integration.as_deref(), false).await {
                if let Ok(Some(r)) = ctx
                    .code_client
                    .pull_request_for_branch(&ctx.code.project, &branch)
                    .await
                {
                    parent = Some(r.target_branch.clone());
                    request = Some(
                        json!({ "number": r.number, "web_url": r.web_url, "target_branch": r.target_branch }),
                    );
                }
            }
        }
    }
    let mut picked = pick_up_branch(
        session,
        &branch,
        parent,
        a.remote.clone(),
        !a.no_fetch,
        &a.run,
    )
    .await?;
    if picked.exit_code == 0 && !a.run.dry_run {
        if let Some(n) = issue {
            unwatch(session, n);
        }
    }
    if let Some(r) = request {
        picked.json["merge_request"] = r;
    }
    Ok(picked)
}
