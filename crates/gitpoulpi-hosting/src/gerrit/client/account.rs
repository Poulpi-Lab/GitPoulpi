//! Gerrit account-wide lists, through change queries: `is:open reviewer:self
//! -owner:self -is:ignored` for what waits for my review, `is:open
//! projects:<prefix>` per namespace (or `owner:self OR reviewer:self` when no
//! namespace is named) for the open requests. A work-in-progress change is a
//! draft, so `-is:wip` keeps it out unless drafts are asked for. Gerrit has
//! no issues. The last row of a page says `_more_changes` when the server
//! held more than it returned.

use super::{encode_component, quote, GerritClient, MAX_PAGES, PAGE};
use crate::account::{AccountItems, AccountPr, AccountQuery, Collector};
use crate::gerrit::map::ChangeInfo;
use crate::Result;

/// Namespaces searched for "every open request".
const MAX_NAMESPACES: usize = 10;
const OPTIONS: &str = "&o=DETAILED_ACCOUNTS&o=LABELS&o=CURRENT_REVISION";

impl GerritClient {
    /// Up to `max_rows` changes for `query`; the flag says the server held more.
    async fn account_changes(
        &self,
        query: &str,
        max_rows: usize,
    ) -> Result<(Vec<ChangeInfo>, bool)> {
        let mut out: Vec<ChangeInfo> = Vec::new();
        let mut more = false;
        for _ in 0..MAX_PAGES {
            let n = PAGE.min(max_rows.saturating_sub(out.len()).max(1));
            let page: Vec<ChangeInfo> = self
                .get_json(
                    &format!(
                        "/changes/?q={}{OPTIONS}&n={n}&S={}",
                        encode_component(query),
                        out.len()
                    ),
                    "listing Gerrit changes",
                )
                .await?;
            more = page.last().is_some_and(|c| c._more_changes);
            out.extend(page);
            if !more || out.len() >= max_rows {
                break;
            }
        }
        Ok((out, more))
    }

    fn account_pr(&self, change: &ChangeInfo, review: bool) -> AccountPr {
        let site = self.site();
        let mut pr = change.summary(&site, None);
        pr.review_requested = review;
        AccountPr {
            clone_url: format!("{site}/{}", change.project),
            project: change.project.clone(),
            pr,
        }
    }
}

pub(crate) async fn fetch(client: &GerritClient, query: &AccountQuery) -> Result<AccountItems> {
    let mut out = Collector::default();
    let max_rows = query.row_limit() + 1;
    let wip = if query.include_drafts { "" } else { " -is:wip" };

    if query.review_requests {
        let q = format!("is:open reviewer:self -owner:self -is:ignored{wip}");
        match client.account_changes(&q, max_rows).await {
            Ok((rows, more)) => {
                rows.iter()
                    .for_each(|c| out.add_pr(client.account_pr(c, true)));
                if more {
                    out.cut("more review requests exist than were listed");
                }
            }
            Err(e) => out.failed("review requests", e)?,
        }
    }

    if query.open_requests {
        let mut spaces: Vec<String> = query.search_namespaces().to_vec();
        if spaces.len() > MAX_NAMESPACES {
            out.note(format!(
                "only the first {MAX_NAMESPACES} of {} namespaces were searched",
                spaces.len()
            ));
            spaces.truncate(MAX_NAMESPACES);
        }
        let queries: Vec<(String, String)> = if spaces.is_empty() {
            vec![(
                format!("is:open (owner:self OR reviewer:self){wip}"),
                "open changes".to_string(),
            )]
        } else {
            spaces
                .iter()
                .map(|ns| {
                    (
                        format!("is:open projects:{}{wip}", quote(ns)),
                        format!("open changes of {ns}"),
                    )
                })
                .collect()
        };
        for (q, what) in queries {
            match client.account_changes(&q, max_rows).await {
                Ok((rows, more)) => {
                    rows.iter()
                        .for_each(|c| out.add_pr(client.account_pr(c, false)));
                    if more {
                        out.cut(format!("more {what} exist than were listed"));
                    }
                }
                Err(e) => out.failed(&what, e)?,
            }
        }
    }

    if query.assigned_issues {
        out.note("Gerrit has no issues");
    }
    Ok(out.finish(query))
}
