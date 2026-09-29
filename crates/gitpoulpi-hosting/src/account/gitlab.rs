//! GitLab (gitlab.com and self-managed) account-wide lists.
//!
//! Review requests and assigned issues are single global lists
//! (`/merge_requests?reviewer_username=`, `/issues?scope=assigned_to_me`).
//! "Every open request" has no global endpoint: it walks my top-level groups
//! (at most [`MAX_GROUPS`]) and my owned projects (at most [`MAX_PROJECTS`]),
//! or the namespaces the profile names.

use serde::Deserialize;

use super::{
    clone_url, get_rows, AccountIssue, AccountItems, AccountPr, AccountQuery, Collector, Paging,
};
use crate::client::encode_component;
use crate::gitlab::{GitLabClient, GitLabMrRow};
use crate::model::{IssueState, IssueSummary, PullRequestState};
use crate::Result;

const PAGE_SIZE: usize = 100;
/// Pages of 100 per list.
const MAX_PAGES: usize = 3;
const MAX_GROUPS: usize = 10;
const MAX_PROJECTS: usize = 10;

#[derive(Deserialize)]
struct References {
    #[serde(default)]
    full: String,
}

#[derive(Deserialize)]
struct Named {
    #[serde(default)]
    username: String,
}

#[derive(Deserialize)]
struct Milestone {
    #[serde(default)]
    title: String,
}

#[derive(Deserialize)]
struct IssueRow {
    iid: u64,
    #[serde(default)]
    title: String,
    #[serde(default)]
    web_url: String,
    #[serde(default)]
    updated_at: String,
    author: Option<Named>,
    #[serde(default)]
    assignees: Vec<Named>,
    #[serde(default)]
    labels: Vec<String>,
    milestone: Option<Milestone>,
    #[serde(default)]
    references: Option<References>,
}

#[derive(Deserialize)]
struct Space {
    id: i64,
}

/// `group/proj` out of the reference `group/proj!12` / `group/proj#3`, else
/// out of the web URL (`.../group/proj/-/merge_requests/12`).
fn project_of(full: Option<&References>, web_url: &str, base: &str) -> Option<String> {
    if let Some(r) = full {
        if let Some((project, _)) = r.full.rsplit_once(['!', '#']) {
            if !project.is_empty() {
                return Some(project.to_string());
            }
        }
    }
    let rest = web_url.strip_prefix(base)?.trim_start_matches('/');
    let (project, _) = rest.split_once("/-/")?;
    (!project.is_empty()).then(|| project.to_string())
}

fn paging(max_rows: usize) -> Paging<'static> {
    Paging {
        size_param: "per_page",
        page_size: PAGE_SIZE,
        max_pages: MAX_PAGES,
        max_rows,
    }
}

async fn rows<T: serde::de::DeserializeOwned>(
    client: &GitLabClient,
    url: &str,
    paging: &Paging<'_>,
    what: &str,
) -> Result<Vec<T>> {
    get_rows(
        &*client.transport,
        &|u| client.get(u),
        url,
        paging,
        what,
        &|r| r.json(),
    )
    .await
}

/// Reads a list of merge requests into the collector; `review` marks them as
/// awaiting my review (the review list), else it is read off the reviewers.
async fn merge_requests(
    client: &GitLabClient,
    out: &mut Collector,
    me: &str,
    url: &str,
    max_rows: usize,
    what: &str,
    review: bool,
) -> Result<()> {
    let base = crate::spec::web_base(&client.integration).unwrap_or_default();
    match rows::<serde_json::Value>(client, url, &paging(max_rows), what).await {
        Ok(found) => {
            for value in found {
                // The reference and the web URL name the project; the typed
                // row (which owns `web_url`) cannot share the object.
                let references = value
                    .pointer("/references/full")
                    .and_then(|v| v.as_str())
                    .map(|full| References {
                        full: full.to_string(),
                    });
                let web_url = value.get("web_url").and_then(|v| v.as_str()).unwrap_or("");
                let Some(project) = project_of(references.as_ref(), web_url, &base) else {
                    continue;
                };
                let Ok(row) = serde_json::from_value::<GitLabMrRow>(value.clone()) else {
                    out.note(format!("{what}: a merge request could not be read"));
                    continue;
                };
                let mut pr = row.into_summary(review);
                if !pr.review_requested {
                    pr.review_requested = pr.reviewers.iter().any(|r| r.eq_ignore_ascii_case(me));
                }
                if pr.state == PullRequestState::Open || pr.state == PullRequestState::Draft {
                    out.add_pr(AccountPr {
                        clone_url: clone_url(&client.integration, &project),
                        project,
                        pr,
                    });
                }
            }
            Ok(())
        }
        Err(e) => out.failed(what, e),
    }
}

/// The ids of up to `cap` spaces (groups or projects) listed at `url`, plus a
/// note when more existed.
async fn first_ids(
    client: &GitLabClient,
    out: &mut Collector,
    url: &str,
    cap: usize,
    what: &str,
) -> Result<Vec<i64>> {
    let one_page = Paging {
        size_param: "per_page",
        page_size: cap + 1,
        max_pages: 1,
        max_rows: cap + 1,
    };
    match rows::<Space>(client, url, &one_page, what).await {
        Ok(mut found) => {
            if found.len() > cap {
                found.truncate(cap);
                out.note(format!("{what}: only the first {cap} were searched"));
            }
            Ok(found.into_iter().map(|s| s.id).collect())
        }
        Err(e) => {
            out.failed(what, e)?;
            Ok(Vec::new())
        }
    }
}

pub(crate) async fn fetch(client: &GitLabClient, query: &AccountQuery) -> Result<AccountItems> {
    let me = client.login().await?;
    let api = client.integration.api_base();
    let mut out = Collector::default();
    let max_rows = query.row_limit() + 1;
    let draft = if query.include_drafts {
        ""
    } else {
        "&draft=no"
    };

    if query.review_requests {
        let url = format!(
            "{api}/merge_requests?scope=all&state=opened&order_by=updated_at&sort=desc&reviewer_username={}{draft}",
            encode_component(&me)
        );
        merge_requests(
            client,
            &mut out,
            &me,
            &url,
            max_rows,
            "review requests",
            true,
        )
        .await?;
    }

    if query.open_requests {
        let tail = format!("state=opened&order_by=updated_at&sort=desc&non_archived=true{draft}");
        let mut group_urls: Vec<String> = Vec::new();
        let mut project_urls: Vec<String> = Vec::new();
        if query.namespaces.is_empty() {
            let groups = format!("{api}/groups?min_access_level=10&top_level_only=true");
            for id in first_ids(client, &mut out, &groups, MAX_GROUPS, "groups").await? {
                group_urls.push(format!(
                    "{api}/groups/{id}/merge_requests?include_subgroups=true&{tail}"
                ));
            }
            let owned = format!(
                "{api}/projects?owned=true&simple=true&archived=false&order_by=last_activity_at&sort=desc"
            );
            for id in first_ids(client, &mut out, &owned, MAX_PROJECTS, "owned projects").await? {
                project_urls.push(format!("{api}/projects/{id}/merge_requests?{tail}"));
            }
        } else {
            for ns in &query.namespaces {
                if ns.eq_ignore_ascii_case(&me) {
                    let owned = format!(
                        "{api}/projects?owned=true&simple=true&archived=false&order_by=last_activity_at&sort=desc"
                    );
                    let ids =
                        first_ids(client, &mut out, &owned, MAX_PROJECTS, "owned projects").await?;
                    for id in ids {
                        project_urls.push(format!("{api}/projects/{id}/merge_requests?{tail}"));
                    }
                } else {
                    group_urls.push(format!(
                        "{api}/groups/{}/merge_requests?include_subgroups=true&{tail}",
                        encode_component(ns)
                    ));
                }
            }
        }
        for url in group_urls.iter().chain(project_urls.iter()) {
            merge_requests(
                client,
                &mut out,
                &me,
                url,
                max_rows,
                "open merge requests",
                false,
            )
            .await?;
        }
    }

    if query.assigned_issues {
        let url =
            format!("{api}/issues?scope=assigned_to_me&state=opened&order_by=updated_at&sort=desc");
        let base = crate::spec::web_base(&client.integration).unwrap_or_default();
        match rows::<IssueRow>(client, &url, &paging(max_rows), "assigned issues").await {
            Ok(found) => {
                for r in found {
                    let Some(project) = project_of(r.references.as_ref(), &r.web_url, &base) else {
                        continue;
                    };
                    out.add_issue(AccountIssue {
                        clone_url: clone_url(&client.integration, &project),
                        project,
                        issue: IssueSummary {
                            number: r.iid,
                            id: r.iid.to_string(),
                            reference: format!("#{}", r.iid),
                            iid_label: format!("#{}", r.iid),
                            title: r.title,
                            state: IssueState::Open,
                            author: r.author.map(|a| a.username).unwrap_or_default(),
                            assignees: r.assignees.into_iter().map(|a| a.username).collect(),
                            labels: r.labels,
                            web_url: r.web_url,
                            updated_at: r.updated_at,
                            milestone: r.milestone.map(|m| m.title).filter(|t| !t.is_empty()),
                            ..IssueSummary::default()
                        },
                    });
                }
            }
            Err(e) => out.failed("assigned issues", e)?,
        }
    }
    Ok(out.finish(query))
}
