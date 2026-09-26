//! `gitpoulpi hosting` -- assignees, reviewers, labels and milestones
//! (slice M): `members`, `labels`, `milestones`, `pr-edit`, `issue edit`,
//! and the flags `pr` / `issue create` take. GitHub and GitLab only: every
//! other provider answers `Unsupported`.

use gitpoulpi_hosting::{HostedMilestone, HostingClient, IssueEdit, ItemMeta, MetaKind, PrEdit};
use serde_json::json;

use crate::cli::IssueMetaArgs;
use crate::repo_cmd::Emitted;

use super::*;

/// What `pr-edit` / `issue edit` were asked, trimmed and without the `@`
/// people type before a login.
pub(super) struct EditAsk {
    pub add_assignee: Vec<String>,
    pub remove_assignee: Vec<String>,
    pub add_reviewer: Vec<String>,
    pub remove_reviewer: Vec<String>,
    pub add_label: Vec<String>,
    pub remove_label: Vec<String>,
    pub milestone: Option<String>,
    pub no_milestone: bool,
}

impl EditAsk {
    pub(super) fn new(meta: IssueMetaArgs, add_reviewer: Vec<String>, remove: Vec<String>) -> Self {
        Self {
            add_assignee: people(meta.add_assignee),
            remove_assignee: people(meta.remove_assignee),
            add_reviewer: people(add_reviewer),
            remove_reviewer: people(remove),
            add_label: words(meta.add_label),
            remove_label: words(meta.remove_label),
            milestone: meta.milestone.map(|m| m.trim().to_string()),
            no_milestone: meta.no_milestone,
        }
    }

    fn is_empty(&self) -> bool {
        self.add_assignee.is_empty()
            && self.remove_assignee.is_empty()
            && self.add_reviewer.is_empty()
            && self.remove_reviewer.is_empty()
            && self.add_label.is_empty()
            && self.remove_label.is_empty()
            && self.milestone.is_none()
            && !self.no_milestone
    }
}

/// What a create call was asked to carry besides its text.
#[derive(Default)]
pub(super) struct NewMeta {
    pub assignees: Vec<String>,
    pub reviewers: Vec<String>,
    pub labels: Vec<String>,
    /// A milestone title (or id), resolved against the project.
    pub milestone: Option<String>,
}

/// Which list's project a metadata read looks at.
pub(super) fn source_which(source: LinkKind) -> Which {
    match source {
        LinkKind::Pulls => Which::Pulls,
        LinkKind::Issues => Which::Issues,
    }
}

/// Logins: trimmed, without a leading `@`, no empty entries.
pub(super) fn people(list: Vec<String>) -> Vec<String> {
    list.into_iter()
        .map(|p| p.trim().trim_start_matches('@').to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

/// Names (labels): trimmed, no empty entries.
pub(super) fn words(list: Vec<String>) -> Vec<String> {
    list.into_iter()
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

/// `current` plus `add`, minus `remove` (case-insensitive, order kept).
fn changed(current: &[String], add: &[String], remove: &[String]) -> Vec<String> {
    let mut out: Vec<String> = current
        .iter()
        .filter(|c| !remove.iter().any(|r| r.eq_ignore_ascii_case(c)))
        .cloned()
        .collect();
    for a in add {
        if !out.iter().any(|o| o.eq_ignore_ascii_case(a)) {
            out.push(a.clone());
        }
    }
    out
}

/// A milestone given by title (ignoring case), else by id. A name the
/// project does not have lists the ones it has, so nothing is sent with a
/// guess.
pub(super) async fn resolve_milestone(
    client: &dyn HostingClient,
    project: &str,
    wanted: &str,
) -> Result<u64, String> {
    let all = client
        .project_milestones(project)
        .await
        .map_err(|e| e.to_string())?;
    let wanted = wanted.trim();
    all.iter()
        .find(|m| m.title.eq_ignore_ascii_case(wanted))
        .or_else(|| all.iter().find(|m| m.id.to_string() == wanted))
        .map(|m| m.id)
        .ok_or_else(|| {
            format!(
                "no open milestone `{wanted}` on this project{}",
                if all.is_empty() {
                    String::new()
                } else {
                    format!(
                        "; choose one of: {}",
                        all.iter()
                            .map(|m| m.title.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            )
        })
}

fn milestone_text(m: &Option<HostedMilestone>) -> String {
    m.as_ref().map(|m| m.title.clone()).unwrap_or_default()
}

fn meta_json(target: &RepoTarget, number: u64, meta: &ItemMeta) -> serde_json::Value {
    let mut out = target_json(target);
    out["number"] = json!(number);
    out["assignees"] = json!(meta.assignees);
    out["reviewers"] = json!(meta.reviewers);
    out["labels"] = json!(meta.labels);
    out["milestone"] = json!(meta.milestone);
    out
}

fn meta_human(number: u64, kind: MetaKind, meta: &ItemMeta) -> String {
    let mut lines = vec![format!(
        "{} {number}",
        match kind {
            MetaKind::Issue => "issue",
            MetaKind::PullRequest => "pull/merge request",
        }
    )];
    lines.push(format!("assignees: {}", meta.assignees.join(", ")));
    if kind == MetaKind::PullRequest {
        lines.push(format!("reviewers: {}", meta.reviewers.join(", ")));
    }
    lines.push(format!("labels: {}", meta.labels.join(", ")));
    lines.push(format!("milestone: {}", milestone_text(&meta.milestone)));
    lines.join("\n")
}

/// `hosting pr-edit` and `hosting issue edit`.
pub(super) async fn edit(
    env: &HostingEnv,
    target: &RepoTarget,
    number: u64,
    kind: MetaKind,
    ask: EditAsk,
) -> Result<Emitted, String> {
    if ask.is_empty() {
        return Err(
            "nothing to change: pass --add-assignee, --remove-assignee, \
             --add-label, --remove-label, --milestone or --no-milestone (and, for a pull \
             request, --add-reviewer / --remove-reviewer)"
                .to_string(),
        );
    }
    if kind == MetaKind::Issue && (!ask.add_reviewer.is_empty() || !ask.remove_reviewer.is_empty())
    {
        return Err("an issue has no reviewers".to_string());
    }
    let client = env.client(&target.integration, &target.integrations)?;
    let before = client
        .item_meta(&target.project, number, kind)
        .await
        .map_err(|e| e.to_string())?;
    let assignees = (!ask.add_assignee.is_empty() || !ask.remove_assignee.is_empty())
        .then(|| changed(&before.assignees, &ask.add_assignee, &ask.remove_assignee));
    let reviewers = (!ask.add_reviewer.is_empty() || !ask.remove_reviewer.is_empty())
        .then(|| changed(&before.reviewers, &ask.add_reviewer, &ask.remove_reviewer));
    let labels = (!ask.add_label.is_empty() || !ask.remove_label.is_empty())
        .then(|| changed(&before.labels, &ask.add_label, &ask.remove_label));
    let milestone = if ask.no_milestone {
        Some(None)
    } else if let Some(title) = &ask.milestone {
        Some(Some(
            resolve_milestone(&*client, &target.project, title).await?,
        ))
    } else {
        None
    };
    match kind {
        MetaKind::Issue => {
            client
                .update_issue(
                    &target.project,
                    number,
                    &IssueEdit {
                        assignees,
                        labels,
                        milestone,
                    },
                )
                .await
        }
        MetaKind::PullRequest => {
            client
                .update_pull_request(
                    &target.project,
                    number,
                    &PrEdit {
                        assignees,
                        reviewers,
                        labels,
                        milestone,
                    },
                )
                .await
        }
    }
    .map_err(|e| e.to_string())?;
    let after = client
        .item_meta(&target.project, number, kind)
        .await
        .map_err(|e| e.to_string())?;
    Ok(Emitted {
        exit_code: 0,
        json: meta_json(target, number, &after),
        human: meta_human(number, kind, &after),
    })
}

fn matches(text: &str, query: &Option<String>) -> bool {
    query
        .as_deref()
        .map(str::trim)
        .filter(|q| !q.is_empty())
        .map_or(true, |q| text.to_lowercase().contains(&q.to_lowercase()))
}

/// `hosting members`.
pub(super) async fn members(
    env: &HostingEnv,
    target: &RepoTarget,
    query: Option<String>,
) -> Result<Emitted, String> {
    let list = env
        .client(&target.integration, &target.integrations)?
        .project_members(&target.project, query.as_deref().unwrap_or(""))
        .await
        .map_err(|e| e.to_string())?;
    let human = list
        .iter()
        .map(|u| {
            if u.name.is_empty() {
                u.login.clone()
            } else {
                format!("{:<24} {}", u.login, u.name)
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    Ok(Emitted {
        exit_code: 0,
        json: super::prs::merge(
            target_json(target),
            "members",
            serde_json::to_value(&list).map_err(|e| e.to_string())?,
        ),
        human,
    })
}

/// `hosting labels`.
pub(super) async fn labels(
    env: &HostingEnv,
    target: &RepoTarget,
    query: Option<String>,
) -> Result<Emitted, String> {
    let list: Vec<_> = env
        .client(&target.integration, &target.integrations)?
        .project_labels(&target.project)
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .filter(|l| matches(&l.name, &query))
        .collect();
    let human = list
        .iter()
        .map(|l| {
            if l.description.is_empty() {
                l.name.clone()
            } else {
                format!("{:<24} {}", l.name, l.description)
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    Ok(Emitted {
        exit_code: 0,
        json: super::prs::merge(
            target_json(target),
            "labels",
            serde_json::to_value(&list).map_err(|e| e.to_string())?,
        ),
        human,
    })
}

/// `hosting milestones`.
pub(super) async fn milestones(
    env: &HostingEnv,
    target: &RepoTarget,
    query: Option<String>,
) -> Result<Emitted, String> {
    let list: Vec<_> = env
        .client(&target.integration, &target.integrations)?
        .project_milestones(&target.project)
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .filter(|m| matches(&m.title, &query))
        .collect();
    let human = list
        .iter()
        .map(|m| match &m.due_on {
            Some(due) => format!("{:<24} due {due}", m.title),
            None => m.title.clone(),
        })
        .collect::<Vec<_>>()
        .join("\n");
    Ok(Emitted {
        exit_code: 0,
        json: super::prs::merge(
            target_json(target),
            "milestones",
            serde_json::to_value(&list).map_err(|e| e.to_string())?,
        ),
        human,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adding_and_removing_keeps_order_and_ignores_case() {
        let cur = vec!["a".to_string(), "B".to_string()];
        let out = changed(
            &cur,
            &["c".to_string(), "b".to_string()],
            &["A".to_string()],
        );
        assert_eq!(out, vec!["B", "c"]);
    }

    #[test]
    fn people_lose_their_at_sign() {
        assert_eq!(people(vec![" @ann ".into(), "".into()]), vec!["ann"]);
    }
}
