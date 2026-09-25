//! `gitpoulpi hosting` -- pr create.

use gitpoulpi_core::domain::{OperationKind, Ref, RefKind, Remote};
use gitpoulpi_hosting::{remote_web_base, HostedProject, NewPullRequest};
use serde_json::json;

use crate::client::Session;
use crate::repo_cmd::Emitted;

use super::*;

/// The remote to fetch from when nothing named one: the only remote, else
/// `origin`.
pub(super) async fn default_remote_name(ipc: &mut Session) -> Result<String, String> {
    let remotes: Vec<Remote> = ipc.call(OperationKind::Remotes, json!({})).await?;
    match remotes.as_slice() {
        [only] => Ok(only.name.clone()),
        _ => remotes
            .iter()
            .find(|r| r.name == "origin")
            .map(|r| r.name.clone())
            .ok_or_else(|| "no remote to fetch from (pass --remote)".to_string()),
    }
}

/// The checked-out branch's name.
pub(super) async fn current_branch(ipc: &mut Session) -> Result<String, String> {
    let refs: Vec<Ref> = ipc.call(OperationKind::Refs, json!({})).await?;
    refs.iter()
        .find_map(|r| match (&r.kind, r.is_head) {
            (RefKind::LocalBranch { name }, true) => Some(name.clone()),
            _ => None,
        })
        .ok_or_else(|| "HEAD is detached - pass a branch name".to_string())
}

/// `hosting pr` - opens a pull/merge request from a branch of the
/// current repository.
#[allow(clippy::too_many_arguments)]
pub(super) async fn run_pr(
    env: &HostingEnv,
    ipc: &mut Session,
    source: Option<String>,
    target: Option<String>,
    title: Option<String>,
    body: Option<String>,
    draft: bool,
    id: Option<String>,
    web: bool,
    meta: NewMeta,
) -> Result<Emitted, String> {
    let refs: Vec<Ref> = ipc.call(OperationKind::Refs, json!({})).await?;
    let remotes: Vec<Remote> = ipc.call(OperationKind::Remotes, json!({})).await?;

    let source = match source {
        Some(source) => source,
        None => refs
            .iter()
            .find_map(|r| match (&r.kind, r.is_head) {
                (RefKind::LocalBranch { name }, true) => Some(name.clone()),
                _ => None,
            })
            .ok_or_else(|| {
                "HEAD is detached - pass --source, or check a branch out first".to_string()
            })?,
    };
    let upstream = refs.iter().find_map(|r| match &r.kind {
        RefKind::LocalBranch { name } if *name == source => r.upstream.clone(),
        _ => None,
    });
    let Some(upstream) = upstream else {
        return Err(format!(
            "{source} has no upstream - push it first (`gitpoulpi push --set-upstream`)"
        ));
    };
    let remote_names: Vec<String> = remotes.iter().map(|r| r.name.clone()).collect();
    let (remote_name, source_remote_branch) =
        gitpoulpi_core::domain::split_remote_ref(&upstream, &remote_names)
            .ok_or_else(|| format!("could not tell which remote `{upstream}` belongs to"))?;
    let remote = remotes
        .iter()
        .find(|r| r.name == remote_name)
        .ok_or_else(|| format!("no such remote: {remote_name}"))?;

    let target = target.unwrap_or_else(|| {
        default_pr_target(&refs, &remote_name).unwrap_or_else(|| "main".to_string())
    });

    let title = match title {
        Some(title) => title,
        None => {
            let graph: gitpoulpi_core::domain::CommitGraph = ipc
                .call(
                    OperationKind::Log,
                    json!({ "opts": gitpoulpi_core::domain::LogOptions { limit: Some(1), ..Default::default() } }),
                )
                .await?;
            graph
                .nodes
                .first()
                .map(|n| n.summary.clone())
                .unwrap_or_else(|| source.clone())
        }
    };
    let body = body.unwrap_or_default();

    let (web_base, project) = remote_web_base(&remote.fetch_url).ok_or_else(|| {
        format!(
            "`{}` is not a URL a host can be told from",
            remote.fetch_url
        )
    })?;

    let integrations = env.integrations()?;
    let integration = match &id {
        Some(id) => Some(pick(&integrations, Some(id))?),
        None => gitpoulpi_hosting::match_integration(&remote.fetch_url, &integrations),
    };
    let web_base = gitpoulpi_hosting::web_base_for(integration, web_base);

    if !web {
        if let Some(integration) = integration {
            if let Ok(client) = env.client(integration, &integrations) {
                let milestone = match &meta.milestone {
                    Some(wanted) => Some(resolve_milestone(&*client, &project, wanted).await?),
                    None => None,
                };
                let created = client
                    .create_pull_request(&NewPullRequest {
                        project,
                        source_branch: source_remote_branch.to_string(),
                        target_branch: target.clone(),
                        title,
                        body,
                        draft,
                        source_project: None,
                        assignees: meta.assignees,
                        reviewers: meta.reviewers,
                        labels: meta.labels,
                        milestone,
                    })
                    .await
                    .map_err(|e| e.to_string())?;
                let mut human = format!("opened #{}: {}", created.number, created.web_url);
                let mut out = json!({ "number": created.number, "web_url": created.web_url });
                if let Some(warning) = &created.warning {
                    human.push_str(&format!("\nwarning: {warning}"));
                    out["warning"] = json!(warning);
                }
                return Ok(Emitted {
                    exit_code: 0,
                    json: out,
                    human,
                });
            }
        }
    }

    // `--web`, or no integration matched this remote, or no token for
    // it: the URL is still useful (it opens the host's own compose page,
    // pre-filled), so this is not an error.
    let kind = integration
        .map(|i| i.kind)
        .or_else(|| gitpoulpi_hosting::well_known_kind(&web_base))
        .ok_or_else(|| {
            format!(
                "no configured integration matches {web_base}, and its hostname is not \
                 github.com/gitlab.com - pass --id, or add an [[integrations]] entry"
            )
        })?;
    let url = gitpoulpi_hosting::new_pull_request_form_url(
        kind,
        &web_base,
        &gitpoulpi_hosting::PullRequestForm {
            project: &project,
            source_project: None,
            source_branch: source_remote_branch,
            target_branch: Some(&target),
            title: &title,
            body: &body,
            draft,
        },
    );
    Ok(Emitted {
        exit_code: 0,
        json: json!({ "url": url }),
        human: url,
    })
}

/// Guesses the remote's default branch from what `refs()` already
/// reported: a remote-tracking `HEAD` when the backend surfaces one,
/// else `main`/`master` among that remote's branches, in that order --
/// the same fallback ladder `pull_request.rs` uses in the TUI.
pub(super) fn default_pr_target(refs: &[Ref], remote: &str) -> Option<String> {
    let remote_branch = |name: &str| {
        refs.iter().any(|r| {
            matches!(&r.kind, RefKind::RemoteBranch { remote: rm, name: n } if rm == remote && n == name)
        })
    };
    if remote_branch("HEAD") {
        return Some("HEAD".to_string());
    }
    ["main", "master"]
        .into_iter()
        .find(|n| remote_branch(n))
        .map(str::to_string)
}

/// Turns what the user typed into exactly one project: an exact full-path
/// match wins, then a unique substring, so `hosting clone thing` works
/// when there is one `thing` and says so when there are three (picking the
/// first would clone the wrong repository silently).
pub(super) fn pick_project<'a>(
    projects: &'a [HostedProject],
    typed: &str,
    wanted: &str,
    integration_id: &str,
) -> Result<&'a HostedProject, String> {
    if let Some(exact) = projects
        .iter()
        .find(|p| p.full_name.to_lowercase() == wanted)
    {
        return Ok(exact);
    }
    let matches: Vec<&HostedProject> = projects.iter().filter(|p| p.matches(wanted)).collect();
    match matches.as_slice() {
        [] => Err(format!(
            "no project matching `{typed}` on `{integration_id}`"
        )),
        [only] => Ok(only),
        many => Err(format!(
            "`{typed}` matches {} projects: {}",
            many.len(),
            many.iter()
                .take(10)
                .map(|p| p.full_name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}
