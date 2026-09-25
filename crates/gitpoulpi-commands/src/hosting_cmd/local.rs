//! `gitpoulpi hosting issue close|reopen|move|base` -- the local Markdown
//! issue source (slice L1); close, reopen and move also work on Redmine (a
//! status change, slice T2). Other sources answer "not supported".

use serde_json::json;

use crate::repo_cmd::Emitted;

use super::*;

fn only_local(target: &RepoTarget) -> Result<(), String> {
    if target.integration.kind.capabilities().local_folder
        || target.integration.kind.family() == gitpoulpi_config::ApiFamily::Redmine
    {
        Ok(())
    } else {
        Err(format!(
            "{} (`{}`) closes issues on its own site; this command is for local Markdown folders and Redmine",
            target.integration.kind.label(),
            target.integration.id
        ))
    }
}

fn row(target: &RepoTarget, number: u64, status: &str, closed: bool) -> Emitted {
    let mut json = target_json(target);
    json["number"] = json!(number);
    json["status"] = json!(status);
    json["state"] = json!(if closed { "closed" } else { "open" });
    Emitted {
        exit_code: 0,
        json,
        human: format!("issue {number} is now `{status}`"),
    }
}

/// The status an issue has after a write.
async fn status_of(
    client: &dyn gitpoulpi_hosting::HostingClient,
    target: &RepoTarget,
    number: u64,
) -> Result<(String, bool), String> {
    let all = client
        .list_issues(
            &target.project,
            &IssueFilter {
                state: gitpoulpi_hosting::StateFilter::All,
                ..IssueFilter::default()
            },
        )
        .await
        .map_err(|e| e.to_string())?;
    all.into_iter()
        .find(|i| i.number == number)
        .map(|i| {
            (
                i.status_label.unwrap_or_default(),
                i.state == gitpoulpi_hosting::IssueState::Closed,
            )
        })
        .ok_or_else(|| format!("no issue {number} in `{}`", target.project))
}

pub(super) async fn set_state(
    env: &HostingEnv,
    target: &RepoTarget,
    number: u64,
    closed: bool,
) -> Result<Emitted, String> {
    only_local(target)?;
    let client = env.client(&target.integration, &target.integrations)?;
    client
        .set_issue_state(&target.project, number, closed)
        .await
        .map_err(|e| e.to_string())?;
    let (status, closed) = status_of(&*client, target, number).await?;
    Ok(row(target, number, &status, closed))
}

pub(super) async fn move_to(
    env: &HostingEnv,
    target: &RepoTarget,
    number: u64,
    status: &str,
) -> Result<Emitted, String> {
    only_local(target)?;
    let client = env.client(&target.integration, &target.integrations)?;
    let known = client
        .issue_statuses(&target.project)
        .await
        .map_err(|e| e.to_string())?;
    let status = status.trim();
    if status.is_empty() {
        return Err("--status needs a value".into());
    }
    // A known status keeps its spelling; a new one is allowed (the notes are
    // free text) and joins the board.
    let status = known
        .iter()
        .find(|s| s.eq_ignore_ascii_case(status))
        .cloned()
        .unwrap_or_else(|| status.to_string());
    client
        .set_issue_status(&target.project, number, &status)
        .await
        .map_err(|e| e.to_string())?;
    let (status, closed) = status_of(&*client, target, number).await?;
    Ok(row(target, number, &status, closed))
}

#[cfg(feature = "local-issues")]
pub(super) async fn create_base(env: &HostingEnv, target: &RepoTarget) -> Result<Emitted, String> {
    only_local(target)?;
    let client = env.client(&target.integration, &target.integrations)?;
    let statuses = client
        .issue_statuses(&target.project)
        .await
        .map_err(|e| e.to_string())?;
    let path = gitpoulpi_hosting::local_markdown::obsidian::create_base(
        std::path::Path::new(&target.project),
        &statuses,
    )
    .map_err(|e| e.to_string())?;
    let mut json = target_json(target);
    json["base"] = json!(path.to_string_lossy());
    Ok(Emitted {
        exit_code: 0,
        json,
        human: format!("wrote {}", path.display()),
    })
}

#[cfg(not(feature = "local-issues"))]
pub(super) async fn create_base(
    _env: &HostingEnv,
    _target: &RepoTarget,
) -> Result<Emitted, String> {
    Err("this build was made without the local-issues feature".into())
}
