//! GitHub (and Enterprise Server) account-wide lists, through the search API:
//! `is:pr review-requested:@me`, `is:pr user:<me> org:<a> ...`,
//! `is:issue assignee:@me`. Search is rate limited much harder than the REST
//! API (30 calls a minute), so a round asks for the lists, not for each
//! repository: one query for review requests, one for issues, at most
//! [`MAX_OPEN_QUERIES`] for the open requests, each at most [`MAX_PAGES`] pages.

use serde::Deserialize;

use super::{
    clone_url, get_rows, AccountIssue, AccountItems, AccountPr, AccountQuery, Collector, Paging,
};
use crate::client::encode_component;
use crate::github::GitHubClient;
use crate::model::{IssueState, IssueSummary, PullRequestState, PullRequestSummary};
use crate::Result;

/// Pages of 100 per query (the search API never returns past 1000 anyway).
const MAX_PAGES: usize = 3;
/// Search queries spent on "every open request" in one round.
const MAX_OPEN_QUERIES: usize = 3;
/// Namespaces per query: search allows five boolean operators in a query.
const NAMESPACES_PER_QUERY: usize = 4;
const PAGE_SIZE: usize = 100;

#[derive(Deserialize)]
struct Search {
    #[serde(default)]
    items: Vec<Row>,
}

#[derive(Deserialize)]
struct Login {
    login: String,
}

#[derive(Deserialize)]
struct Label {
    name: String,
}

#[derive(Deserialize)]
struct Milestone {
    #[serde(default)]
    title: String,
}

#[derive(Deserialize)]
struct Row {
    number: u64,
    /// The GraphQL id, which `nodes(ids:)` reads the branches by.
    #[serde(default)]
    node_id: String,
    title: String,
    #[serde(default)]
    draft: bool,
    html_url: String,
    #[serde(default)]
    updated_at: String,
    repository_url: String,
    user: Option<Login>,
    #[serde(default)]
    assignees: Vec<Login>,
    #[serde(default)]
    labels: Vec<Label>,
    milestone: Option<Milestone>,
}

/// `owner/name` out of `https://api.github.com/repos/owner/name`.
fn project_of(repository_url: &str) -> Option<String> {
    let (_, project) = repository_url.split_once("/repos/")?;
    let project = project.trim_matches('/');
    (!project.is_empty()).then(|| project.to_string())
}

impl Row {
    fn into_pr(self, client: &GitHubClient, me: &str, review: bool) -> Option<AccountPr> {
        let project = project_of(&self.repository_url)?;
        let author = self.user.map(|u| u.login).unwrap_or_default();
        Some(AccountPr {
            clone_url: clone_url(&client.integration, &project),
            project,
            pr: PullRequestSummary {
                number: self.number,
                iid_label: format!("#{}", self.number),
                title: self.title,
                state: if self.draft {
                    PullRequestState::Draft
                } else {
                    PullRequestState::Open
                },
                author,
                web_url: self.html_url,
                updated_at: self.updated_at,
                review_requested: review,
                assignees: self.assignees.into_iter().map(|u| u.login).collect(),
                reviewers: if review {
                    vec![me.to_string()]
                } else {
                    Vec::new()
                },
                ..PullRequestSummary::default()
            },
        })
    }

    fn into_issue(self, client: &GitHubClient) -> Option<AccountIssue> {
        let project = project_of(&self.repository_url)?;
        Some(AccountIssue {
            clone_url: clone_url(&client.integration, &project),
            project,
            issue: IssueSummary {
                number: self.number,
                id: self.number.to_string(),
                reference: format!("#{}", self.number),
                iid_label: format!("#{}", self.number),
                title: self.title,
                state: IssueState::Open,
                author: self.user.map(|u| u.login).unwrap_or_default(),
                assignees: self.assignees.into_iter().map(|u| u.login).collect(),
                labels: self.labels.into_iter().map(|l| l.name).collect(),
                web_url: self.html_url,
                updated_at: self.updated_at,
                milestone: self.milestone.map(|m| m.title).filter(|t| !t.is_empty()),
                ..IssueSummary::default()
            },
        })
    }
}

/// Runs one search `query` (raw, not yet encoded) and returns its rows.
async fn search(
    client: &GitHubClient,
    query: &str,
    max_rows: usize,
    what: &str,
) -> Result<Vec<Row>> {
    let url = format!(
        "{}/search/issues?sort=updated&order=desc&q={}",
        client.integration.api_base(),
        encode_component(query)
    );
    let paging = Paging {
        size_param: "per_page",
        page_size: PAGE_SIZE,
        max_pages: MAX_PAGES,
        max_rows,
    };
    get_rows(
        &*client.transport,
        &|u| client.get(u),
        &url,
        &paging,
        what,
        &|r| Ok(r.json::<Search>()?.items),
    )
    .await
}

/// The organizations the token's user belongs to.
async fn organizations(client: &GitHubClient) -> Result<Vec<String>> {
    let url = format!("{}/user/orgs", client.integration.api_base());
    let paging = Paging {
        size_param: "per_page",
        page_size: PAGE_SIZE,
        max_pages: 1,
        max_rows: PAGE_SIZE,
    };
    let rows: Vec<Login> = get_rows(
        &*client.transport,
        &|u| client.get(u),
        &url,
        &paging,
        "listing the GitHub organizations",
        &|r| r.json(),
    )
    .await?;
    Ok(rows.into_iter().map(|o| o.login).collect())
}

/// The search qualifiers that scope a global query (review requests,
/// assigned issues) to the plain namespaces, server side: an owner
/// (`user:`, which covers organizations too) or one repository (`repo:`).
/// Repeated qualifiers of a kind are alternatives. Empty in regex mode and
/// when no namespace is set; `finish` filters the rows either way.
fn scope_qualifiers(query: &AccountQuery) -> String {
    query
        .search_namespaces()
        .iter()
        .map(|ns| ns.trim().trim_matches('/'))
        .filter(|ns| !ns.is_empty() && !ns.contains(char::is_whitespace))
        .map(|ns| {
            if ns.contains('/') {
                format!(" repo:{ns}")
            } else {
                format!(" user:{ns}")
            }
        })
        .collect()
}

const NODES_QUERY: &str = "query($ids:[ID!]!){nodes(ids:$ids){... on PullRequest{number \
     repository{nameWithOwner} headRefName baseRefName headRepository{nameWithOwner}}}}";
/// GraphQL calls spent on branches in one round (100 requests each).
const MAX_BRANCH_CALLS: usize = 5;

/// Search rows carry no head or base ref: reads them in batches, by node id.
async fn fill_branches(
    client: &GitHubClient,
    items: &mut AccountItems,
    ids: &std::collections::HashMap<(String, u64), String>,
) {
    use super::branches::{apply, fill_known, key, remember, Known};
    let api = client.integration.api_base();
    let todo: Vec<String> = fill_known(&api, items)
        .into_iter()
        .filter_map(|i| {
            let r = &items.prs[i];
            ids.get(&(r.project.to_lowercase(), r.pr.number)).cloned()
        })
        .collect();
    for chunk in todo.chunks(PAGE_SIZE).take(MAX_BRANCH_CALLS) {
        let Ok(data) = client
            .graphql(
                "reading the branches of GitHub pull requests",
                NODES_QUERY,
                serde_json::json!({ "ids": chunk }),
            )
            .await
        else {
            return;
        };
        for node in data["nodes"].as_array().into_iter().flatten() {
            let (Some(project), Some(number)) = (
                node["repository"]["nameWithOwner"].as_str(),
                node["number"].as_u64(),
            ) else {
                continue;
            };
            let text = |v: &serde_json::Value| v.as_str().unwrap_or_default().to_string();
            let fork = node["headRepository"]["nameWithOwner"]
                .as_str()
                .filter(|h| !h.eq_ignore_ascii_case(project))
                .map(str::to_string);
            let known = Known {
                source: text(&node["headRefName"]),
                target: text(&node["baseRefName"]),
                fork,
            };
            if let Some(row) = items
                .prs
                .iter_mut()
                .find(|r| r.project.eq_ignore_ascii_case(project) && r.pr.number == number)
            {
                remember(
                    key(&api, &row.project, number, &row.pr.updated_at),
                    known.clone(),
                );
                apply(row, &known);
            }
        }
    }
}

pub(crate) async fn fetch(client: &GitHubClient, query: &AccountQuery) -> Result<AccountItems> {
    let me = client.login().await?;
    let mut out = Collector::default();
    let mut ids = std::collections::HashMap::new();
    let mut note_ids = |rows: &[Row]| {
        for r in rows.iter().filter(|r| !r.node_id.is_empty()) {
            if let Some(p) = project_of(&r.repository_url) {
                ids.insert((p.to_lowercase(), r.number), r.node_id.clone());
            }
        }
    };
    let max_rows = query.row_limit() + 1;
    let drafts = if query.include_drafts {
        ""
    } else {
        " draft:false"
    };

    if query.review_requests {
        let q = format!(
            "is:pr is:open archived:false{drafts} review-requested:@me{}",
            scope_qualifiers(query)
        );
        match search(client, &q, max_rows, "searching review requests on GitHub").await {
            Ok(rows) => {
                note_ids(&rows);
                rows.into_iter()
                    .filter_map(|r| r.into_pr(client, &me, true))
                    .for_each(|p| out.add_pr(p))
            }
            Err(e) => out.failed("review requests", e)?,
        }
    }

    if query.open_requests {
        let mut spaces: Vec<String> = Vec::new();
        if query.search_namespaces().is_empty() {
            spaces.push(format!("user:{me}"));
            match organizations(client).await {
                Ok(orgs) => spaces.extend(orgs.into_iter().map(|o| format!("org:{o}"))),
                Err(e) => out.failed("organizations", e)?,
            }
        } else {
            for ns in query.search_namespaces() {
                let kind = if ns.contains('/') {
                    "repo"
                } else if ns.eq_ignore_ascii_case(&me) {
                    "user"
                } else {
                    "org"
                };
                spaces.push(format!("{kind}:{ns}"));
            }
        }
        let room = MAX_OPEN_QUERIES * NAMESPACES_PER_QUERY;
        if spaces.len() > room {
            out.note(format!(
                "only the first {room} of {} namespaces were searched",
                spaces.len()
            ));
            spaces.truncate(room);
        }
        for chunk in spaces.chunks(NAMESPACES_PER_QUERY) {
            let q = format!("is:pr is:open archived:false{drafts} {}", chunk.join(" "));
            match search(
                client,
                &q,
                max_rows,
                "searching open pull requests on GitHub",
            )
            .await
            {
                Ok(rows) => {
                    note_ids(&rows);
                    rows.into_iter()
                        .filter_map(|r| r.into_pr(client, &me, false))
                        .for_each(|p| out.add_pr(p))
                }
                Err(e) => out.failed("open pull requests", e)?,
            }
        }
    }

    if query.assigned_issues {
        let q = format!(
            "is:issue is:open archived:false assignee:@me{}",
            scope_qualifiers(query)
        );
        match search(client, &q, max_rows, "searching assigned issues on GitHub").await {
            Ok(rows) => rows
                .into_iter()
                .filter_map(|r| r.into_issue(client))
                .for_each(|i| out.add_issue(i)),
            Err(e) => out.failed("assigned issues", e)?,
        }
    }
    let mut items = out.finish(query);
    fill_branches(client, &mut items, &ids).await;
    Ok(items)
}
