//! `gitpoulpi hosting board [show]` and `hosting board move` -- the Kanban
//! board of the linked issue source, with the same column logic as the TUI
//! (`gitpoulpi_hosting::board`, pure).

use gitpoulpi_hosting::board::{self, BoardColumn, BoardSource, BoardSpec, ColumnRole};
use gitpoulpi_hosting::{IssueFilter, IssueState, IssueSummary, StateFilter};
use serde_json::{json, Value};

use crate::repo_cmd::Emitted;

use super::*;

/// A host board lists the open issues and this many closed ones (the TUI's
/// `CLOSED_LIMIT`).
const CLOSED_LIMIT: usize = 60;

struct Board {
    client: Box<dyn gitpoulpi_hosting::HostingClient>,
    spec: BoardSpec,
    columns: Vec<BoardColumn>,
    issues: Vec<IssueSummary>,
}

async fn load(env: &HostingEnv, target: &RepoTarget, labels: Vec<String>) -> Result<Board, String> {
    let client = env.client(&target.integration, &target.integrations)?;
    let local = target.integration.kind.capabilities().local_folder;
    let mut filter = IssueFilter {
        query: target.filter.clone(),
        state: if local {
            StateFilter::All
        } else {
            StateFilter::Open
        },
        ..IssueFilter::default()
    };
    let mut issues = client
        .list_issues(&target.project, &filter)
        .await
        .map_err(|e| e.to_string())?;
    if !local {
        filter.state = StateFilter::Closed;
        if let Ok(mut closed) = client.list_issues(&target.project, &filter).await {
            closed.truncate(CLOSED_LIMIT);
            // A host that cannot tell the states apart answers both lists alike.
            closed.retain(|c| !issues.iter().any(|i| i.number == c.number && i.id == c.id));
            issues.extend(closed);
        }
    }
    let declared = client
        .issue_statuses(&target.project)
        .await
        .unwrap_or_default();
    let spec = BoardSpec {
        source: client.board_source(),
        declared,
        labels,
    };
    let columns = board::columns(&spec, &issues);
    Ok(Board {
        client,
        spec,
        columns,
        issues,
    })
}

fn source_name(source: BoardSource) -> &'static str {
    match source {
        BoardSource::OpenClosed => "open_closed",
        BoardSource::LocalStatus => "status",
        BoardSource::ProviderStatus => "provider_status",
        BoardSource::Lists => "lists",
        BoardSource::Labels => "labels",
    }
}

fn role_name(role: ColumnRole) -> &'static str {
    match role {
        ColumnRole::Named => "named",
        ColumnRole::Open => "open",
        ColumnRole::Closed => "closed",
    }
}

fn reference_of(issue: &IssueSummary) -> &str {
    if issue.reference.is_empty() {
        &issue.iid_label
    } else {
        &issue.reference
    }
}

fn card(issue: &IssueSummary) -> Value {
    json!({
        "number": issue.number,
        "reference": reference_of(issue),
        "title": issue.title,
        "state": if issue.state == IssueState::Closed { "closed" } else { "open" },
        "status": issue.status_label,
        "kind": issue.kind_label,
        "assignees": issue.assignees,
        "labels": issue.labels,
        "web_url": issue.web_url,
    })
}

pub(super) async fn show(
    env: &HostingEnv,
    target: &RepoTarget,
    labels: Vec<String>,
) -> Result<Emitted, String> {
    let b = load(env, target, labels).await?;
    let mut cards: Vec<Vec<&IssueSummary>> = vec![Vec::new(); b.columns.len()];
    for issue in &b.issues {
        cards[board::column_index(&b.spec, &b.columns, issue)].push(issue);
    }
    let columns: Vec<Value> = b
        .columns
        .iter()
        .zip(&cards)
        .map(|(col, cards)| {
            json!({
                "name": col.name,
                "role": role_name(col.role),
                "cards": cards.iter().map(|i| card(i)).collect::<Vec<_>>(),
            })
        })
        .collect();
    let human = b
        .columns
        .iter()
        .zip(&cards)
        .map(|(col, cards)| {
            let rows = cards
                .iter()
                .map(|i| format!("  {:<8} {}", reference_of(i), i.title))
                .collect::<Vec<_>>();
            format!("{} ({})\n{}", col.name, cards.len(), rows.join("\n"))
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    let mut json = target_json(target);
    json["source"] = json!(source_name(b.spec.source));
    json["movable"] = json!(b.spec.source.movable());
    json["columns"] = Value::Array(columns);
    Ok(Emitted {
        exit_code: 0,
        json,
        human,
    })
}

/// The card `reference` names: its reference (`ABC-12`, `AB#12`, `#12`) or
/// its number.
fn find_card<'a>(issues: &'a [IssueSummary], reference: &str) -> Result<&'a IssueSummary, String> {
    let want = reference.trim();
    let bare = want.trim_start_matches(['#', '!']);
    issues
        .iter()
        .find(|i| {
            reference_of(i).eq_ignore_ascii_case(want)
                || i.iid_label.eq_ignore_ascii_case(want)
                || i.reference.trim_start_matches(['#', '!']) == bare
                || i.number.to_string() == bare
        })
        .ok_or_else(|| format!("no card `{want}` on the board"))
}

pub(super) async fn move_card(
    env: &HostingEnv,
    target: &RepoTarget,
    labels: Vec<String>,
    reference: &str,
    column: &str,
) -> Result<Emitted, String> {
    let b = load(env, target, labels).await?;
    let issue = find_card(&b.issues, reference)?;
    let want = column.trim();
    let to = b
        .columns
        .iter()
        .find(|c| c.name.eq_ignore_ascii_case(want))
        .ok_or_else(|| {
            format!(
                "no column `{want}` (columns: {})",
                b.columns
                    .iter()
                    .map(|c| c.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;
    let from = &b.columns[board::column_index(&b.spec, &b.columns, issue)].name;
    b.client
        .move_issue(&target.project, issue, &b.spec, to)
        .await
        .map_err(|e| e.to_string())?;
    let mut json = target_json(target);
    json["reference"] = json!(reference_of(issue));
    json["number"] = json!(issue.number);
    json["from"] = json!(from);
    json["column"] = json!(to.name);
    Ok(Emitted {
        exit_code: 0,
        json,
        human: format!("{} moved from {from} to {}", reference_of(issue), to.name),
    })
}
