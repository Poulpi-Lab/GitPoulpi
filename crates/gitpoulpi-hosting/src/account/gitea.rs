//! Gitea / Forgejo (incl. Codeberg) account-wide lists, through
//! `/repos/issues/search` (`type=pulls|issues`, `review_requested=true`,
//! `assigned=true`, `owner=<namespace>`). A draft is the `draft` flag of the
//! request or a `WIP:` title.

use serde::Deserialize;

use super::{
    clone_url, get_rows, AccountIssue, AccountItems, AccountPr, AccountQuery, Collector, Paging,
};
use crate::client::encode_component;
use crate::gitea::GiteaClient;
use crate::model::{IssueState, IssueSummary, PullRequestState, PullRequestSummary};
use crate::Result;

/// Gitea clamps a page to 50 rows by default.
const PAGE_SIZE: usize = 50;
const MAX_PAGES: usize = 6;
/// Namespaces searched for "every open request".
const MAX_NAMESPACES: usize = 10;

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
struct Repository {
    full_name: String,
}

#[derive(Deserialize)]
struct PullInfo {
    #[serde(default)]
    draft: bool,
}

#[derive(Deserialize)]
struct Row {
    number: u64,
    title: String,
    html_url: String,
    #[serde(default)]
    updated_at: String,
    user: Option<Login>,
    assignees: Option<Vec<Login>>,
    labels: Option<Vec<Label>>,
    milestone: Option<Milestone>,
    repository: Option<Repository>,
    pull_request: Option<PullInfo>,
}

#[derive(Deserialize)]
struct Org {
    username: Option<String>,
    name: Option<String>,
}

fn is_wip(title: &str) -> bool {
    let t = title.trim_start().to_ascii_lowercase();
    t.starts_with("wip:") || t.starts_with("[wip]")
}

impl Row {
    fn into_pr(self, client: &GiteaClient, review: bool, me: &str) -> Option<AccountPr> {
        let project = self.repository?.full_name;
        let draft = self.pull_request.is_some_and(|p| p.draft) || is_wip(&self.title);
        Some(AccountPr {
            clone_url: clone_url(&client.integration, &project),
            project,
            pr: PullRequestSummary {
                number: self.number,
                iid_label: format!("#{}", self.number),
                title: self.title,
                state: if draft {
                    PullRequestState::Draft
                } else {
                    PullRequestState::Open
                },
                author: self.user.map(|u| u.login).unwrap_or_default(),
                web_url: self.html_url,
                updated_at: self.updated_at,
                review_requested: review,
                assignees: self
                    .assignees
                    .unwrap_or_default()
                    .into_iter()
                    .map(|u| u.login)
                    .collect(),
                reviewers: if review {
                    vec![me.to_string()]
                } else {
                    Vec::new()
                },
                ..PullRequestSummary::default()
            },
        })
    }

    fn into_issue(self, client: &GiteaClient) -> Option<AccountIssue> {
        let project = self.repository?.full_name;
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
                assignees: self
                    .assignees
                    .unwrap_or_default()
                    .into_iter()
                    .map(|u| u.login)
                    .collect(),
                labels: self
                    .labels
                    .unwrap_or_default()
                    .into_iter()
                    .map(|l| l.name)
                    .collect(),
                web_url: self.html_url,
                updated_at: self.updated_at,
                milestone: self.milestone.map(|m| m.title).filter(|t| !t.is_empty()),
                ..IssueSummary::default()
            },
        })
    }
}

async fn search(
    client: &GiteaClient,
    filter: &str,
    max_rows: usize,
    what: &str,
) -> Result<Vec<Row>> {
    let url = client.api(&format!("/repos/issues/search?state=open&{filter}"));
    let paging = Paging {
        size_param: "limit",
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
        &|r| r.json(),
    )
    .await
}

async fn organizations(client: &GiteaClient) -> Result<Vec<String>> {
    let paging = Paging {
        size_param: "limit",
        page_size: PAGE_SIZE,
        max_pages: 1,
        max_rows: PAGE_SIZE,
    };
    let rows: Vec<Org> = get_rows(
        &*client.transport,
        &|u| client.get(u),
        &client.api("/user/orgs"),
        &paging,
        "listing organizations",
        &|r| r.json(),
    )
    .await?;
    Ok(rows
        .into_iter()
        .filter_map(|o| o.username.or(o.name).filter(|n| !n.is_empty()))
        .collect())
}

pub(crate) async fn fetch(client: &GiteaClient, query: &AccountQuery) -> Result<AccountItems> {
    let me = client.login().await?;
    let mut out = Collector::default();
    let max_rows = query.row_limit() + 1;

    if query.review_requests {
        match search(
            client,
            "type=pulls&review_requested=true",
            max_rows,
            "searching review requests",
        )
        .await
        {
            Ok(rows) => rows
                .into_iter()
                .filter_map(|r| r.into_pr(client, true, &me))
                .for_each(|p| out.add_pr(p)),
            Err(e) => out.failed("review requests", e)?,
        }
    }

    if query.open_requests {
        let mut spaces: Vec<String> = Vec::new();
        if query.namespaces.is_empty() {
            spaces.push(me.clone());
            match organizations(client).await {
                Ok(orgs) => spaces.extend(orgs),
                Err(e) => out.failed("organizations", e)?,
            }
        } else {
            spaces = query.namespaces.clone();
        }
        if spaces.len() > MAX_NAMESPACES {
            out.note(format!(
                "only the first {MAX_NAMESPACES} of {} namespaces were searched",
                spaces.len()
            ));
            spaces.truncate(MAX_NAMESPACES);
        }
        for ns in spaces {
            let filter = format!("type=pulls&owner={}", encode_component(&ns));
            match search(client, &filter, max_rows, "searching open pull requests").await {
                Ok(rows) => rows
                    .into_iter()
                    .filter_map(|r| r.into_pr(client, false, &me))
                    .for_each(|p| out.add_pr(p)),
                Err(e) => out.failed(&format!("open pull requests of {ns}"), e)?,
            }
        }
    }

    if query.assigned_issues {
        match search(
            client,
            "type=issues&assigned=true",
            max_rows,
            "searching assigned issues",
        )
        .await
        {
            Ok(rows) => rows
                .into_iter()
                .filter_map(|r| r.into_issue(client))
                .for_each(|i| out.add_issue(i)),
            Err(e) => out.failed("assigned issues", e)?,
        }
    }
    Ok(out.finish(query))
}
