//! Bitbucket Data Center account-wide lists, through the dashboard:
//! `GET /rest/api/1.0/dashboard/pull-requests?state=OPEN&role=...`. Requests
//! that wait for my review are `role=REVIEWER&participantStatus=UNAPPROVED`;
//! "every open request" is the ones I wrote (`AUTHOR`) or take part in
//! (`PARTICIPANT`). The dashboard names no namespaces, so a project filter
//! keeps the rows whose project key is listed. There are no issues.

use super::{
    encode_component, BitbucketDcClient, HttpRequest, Page, PullRequest, RepoRef, Result, LIMIT,
};
use crate::account::{AccountItems, AccountPr, AccountQuery, Collector};

/// Pages per role.
const MAX_PAGES: usize = 6;

impl BitbucketDcClient {
    /// The dashboard's open requests for `role` (plus `extra` query text).
    async fn dashboard(
        &self,
        role: &str,
        extra: &str,
        max_rows: usize,
    ) -> Result<Vec<PullRequest>> {
        let first = format!(
            "{}/dashboard/pull-requests?state=OPEN&role={}&order=NEWEST&limit={LIMIT}{extra}",
            self.api(),
            encode_component(role)
        );
        let mut out: Vec<PullRequest> = Vec::new();
        let mut start: Option<u64> = None;
        for _ in 0..MAX_PAGES {
            let url = match start {
                None => first.clone(),
                Some(s) => format!("{first}&start={s}"),
            };
            let response = self
                .transport
                .send(self.authed(HttpRequest::get(url)))
                .await?;
            response.error_for_status("listing Bitbucket pull requests")?;
            let page: Page<PullRequest> = response.json()?;
            out.extend(page.values);
            match (page.is_last_page, page.next_page_start) {
                (false, Some(next)) if out.len() < max_rows => start = Some(next),
                _ => break,
            }
        }
        Ok(out)
    }

    fn account_pr(&self, pull: PullRequest, review: bool) -> AccountPr {
        let key = pull.to_ref.repository.project.key.clone();
        let slug = pull.to_ref.repository.slug.clone();
        let project = format!("{key}/{slug}");
        let clone_url = clone_of(self, &pull.to_ref.repository, &key, &slug);
        let mut summary = pull.into_summary(&project, None);
        summary.review_requested = review;
        AccountPr {
            project,
            clone_url,
            pr: summary,
        }
    }
}

/// The repository's HTTP clone link, else `<server>/scm/<key>/<slug>.git`.
fn clone_of(client: &BitbucketDcClient, repo: &RepoRef, key: &str, slug: &str) -> String {
    let linked = repo
        .links
        .clone
        .iter()
        .find(|l| {
            l.name
                .as_deref()
                .is_some_and(|n| n == "http" || n == "https")
        })
        .and_then(|l| l.href.clone())
        .map(|u| super::without_user(&u));
    linked.unwrap_or_else(|| {
        crate::spec::web_base(&client.integration)
            .map(|base| format!("{base}/scm/{}/{slug}.git", key.to_lowercase()))
            .unwrap_or_default()
    })
}

pub(crate) async fn fetch(
    client: &BitbucketDcClient,
    query: &AccountQuery,
) -> Result<AccountItems> {
    let mut out = Collector::default();
    let max_rows = query.row_limit() + 1;

    if query.review_requests {
        match client
            .dashboard("REVIEWER", "&participantStatus=UNAPPROVED", max_rows)
            .await
        {
            Ok(rows) => rows
                .into_iter()
                .map(|p| client.account_pr(p, true))
                .for_each(|p| out.add_pr(p)),
            Err(e) => out.failed("review requests", e)?,
        }
    }

    if query.open_requests {
        for role in ["AUTHOR", "PARTICIPANT"] {
            match client.dashboard(role, "", max_rows).await {
                Ok(rows) => {
                    for pull in rows {
                        let key = &pull.to_ref.repository.project.key;
                        let wanted = query.namespaces.is_empty()
                            || query.namespaces.iter().any(|n| n.eq_ignore_ascii_case(key));
                        if wanted {
                            out.add_pr(client.account_pr(pull, false));
                        }
                    }
                }
                Err(e) => out.failed(&format!("open pull requests ({role})"), e)?,
            }
        }
    }

    if query.assigned_issues {
        out.note(super::NO_ISSUES);
    }
    Ok(out.finish(query))
}
