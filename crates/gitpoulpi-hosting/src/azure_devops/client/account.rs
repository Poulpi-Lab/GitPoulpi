//! Azure DevOps account-wide lists: active pull requests of the whole
//! organization (`{org}/_apis/git/pullrequests`, `searchCriteria.reviewerId`
//! for the ones that wait for me) and the work items assigned to me (WIQL
//! without a project, `[System.AssignedTo] = @Me`). The org-level routes are
//! documented for a project; when the host refuses them (400, 404, 405) the
//! round fans out over the first [`MAX_PROJECTS`] projects instead.

use serde_json::{json, Value};

use super::{wiql_state_clause, AzureDevopsClient, WORK_ITEM_BATCH};
use crate::account::{AccountIssue, AccountItems, AccountPr, AccountQuery, Collector};
use crate::azure_devops::map::{self, List, Pull};
use crate::client::encode_component as enc;
use crate::model::StateFilter;
use crate::{Error, Result};

/// Projects asked one by one when the org-level route is refused.
const MAX_PROJECTS: usize = 20;
/// Pull requests per page.
const PAGE: usize = 100;
/// Pages per query.
const MAX_PAGES: usize = 6;

/// Whether `error` means "this route does not exist here" rather than a
/// problem with the token.
fn route_missing(error: &Error) -> bool {
    matches!(
        error,
        Error::Api {
            status: 400 | 404 | 405,
            ..
        }
    )
}

impl AzureDevopsClient {
    /// Active pull requests under `criteria`, in the whole organization or in
    /// one project, at most about `max_rows`.
    async fn account_pulls(
        &self,
        project: Option<&str>,
        criteria: &str,
        max_rows: usize,
    ) -> Result<Vec<Pull>> {
        let mut out: Vec<Pull> = Vec::new();
        for page in 0..MAX_PAGES {
            let query = format!("{criteria}&$top={PAGE}&$skip={}", page * PAGE);
            let url = self.url(project, "git/pullrequests", &query)?;
            let response = self
                .send(self.get(url), "listing Azure DevOps pull requests")
                .await?;
            let list: List<Pull> = response.json()?;
            let count = list.value.len();
            out.extend(list.value);
            if count < PAGE || out.len() >= max_rows {
                break;
            }
        }
        Ok(out)
    }

    async fn account_projects(&self) -> Result<Vec<String>> {
        let url = self.url(None, "projects", &format!("$top={}", MAX_PROJECTS + 1))?;
        let response = self
            .send(self.get(url), "listing Azure DevOps projects")
            .await?;
        let list: List<map::Project> = response.json()?;
        Ok(list.value.into_iter().map(|p| p.name).collect())
    }

    /// Pull requests of the organization; per project when the org-level
    /// route is missing.
    async fn org_pulls(
        &self,
        criteria: &str,
        max_rows: usize,
        spaces: &[String],
        out: &mut Collector,
    ) -> Result<Vec<Pull>> {
        match self.account_pulls(None, criteria, max_rows).await {
            Err(e) if route_missing(&e) => {
                let mut projects = if spaces.is_empty() {
                    self.account_projects().await?
                } else {
                    spaces.to_vec()
                };
                if projects.len() > MAX_PROJECTS {
                    out.note(format!(
                        "only the first {MAX_PROJECTS} of {} projects were searched",
                        projects.len()
                    ));
                    projects.truncate(MAX_PROJECTS);
                }
                let mut rows = Vec::new();
                for project in projects {
                    match self.account_pulls(Some(&project), criteria, max_rows).await {
                        Ok(more) => rows.extend(more),
                        Err(e) => out.failed(&format!("pull requests of {project}"), e)?,
                    }
                }
                Ok(rows)
            }
            other => other,
        }
    }

    fn account_pr(&self, pull: Pull, me: Option<&str>) -> Result<AccountPr> {
        let project = format!("{}/{}", pull.repository.project.name, pull.repository.name);
        let root = self.root()?;
        let clone_url = pull
            .repository
            .remote_url
            .as_deref()
            .map(map::strip_userinfo)
            .unwrap_or_else(|| {
                format!(
                    "{root}/{}/_git/{}",
                    enc(&pull.repository.project.name),
                    enc(&pull.repository.name)
                )
            });
        Ok(AccountPr {
            project,
            clone_url,
            pr: map::map_pull(&root, me, pull),
        })
    }

    /// WIQL ids of the work items assigned to me, newest change first.
    async fn assigned_ids(&self, project: Option<&str>, top: usize) -> Result<Vec<u64>> {
        let mut clauses = vec!["[System.AssignedTo] = @Me".to_string()];
        clauses.extend(wiql_state_clause(StateFilter::Open));
        let wiql = format!(
            "SELECT [System.Id] FROM WorkItems WHERE {} ORDER BY [System.ChangedDate] DESC",
            clauses.join(" AND ")
        );
        let url = self.url(project, "wit/wiql", &format!("$top={top}"))?;
        let response = self
            .send(
                self.post(url, &json!({ "query": wiql }), None),
                "querying Azure DevOps work items",
            )
            .await?;
        let found: Value = response.json()?;
        Ok(found["workItems"]
            .as_array()
            .map(|a| a.iter().filter_map(|w| w["id"].as_u64()).collect())
            .unwrap_or_default())
    }

    async fn work_items(&self, ids: &[u64]) -> Result<Vec<AccountIssue>> {
        let root = self.root()?;
        let mut out = Vec::new();
        for chunk in ids.chunks(WORK_ITEM_BATCH) {
            let list = chunk
                .iter()
                .map(u64::to_string)
                .collect::<Vec<_>>()
                .join(",");
            let query = format!(
                "ids={list}&fields=System.Title,System.State,System.WorkItemType,\
                 System.AssignedTo,System.CreatedBy,System.ChangedDate,System.Tags,\
                 System.TeamProject"
            );
            let url = self.url(None, "wit/workitems", &query)?;
            let response = self
                .send(self.get(url), "reading Azure DevOps work items")
                .await?;
            let items: List<Value> = response.json()?;
            for item in &items.value {
                let project = item["fields"]["System.TeamProject"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string();
                if let Some(issue) = map::map_work_item(&root, &project, item) {
                    out.push(AccountIssue {
                        project,
                        clone_url: String::new(),
                        issue,
                    });
                }
            }
        }
        Ok(out)
    }

    async fn account_issues(&self, top: usize, out: &mut Collector) -> Result<Vec<AccountIssue>> {
        let ids = match self.assigned_ids(None, top).await {
            Err(e) if route_missing(&e) => {
                let mut ids = Vec::new();
                let mut projects = self.account_projects().await?;
                projects.truncate(MAX_PROJECTS);
                for project in projects {
                    match self.assigned_ids(Some(&project), top).await {
                        Ok(more) => ids.extend(more),
                        Err(e) => out.failed(&format!("work items of {project}"), e)?,
                    }
                }
                ids
            }
            other => other?,
        };
        self.work_items(&ids).await
    }
}

pub(crate) async fn fetch(
    client: &AzureDevopsClient,
    query: &AccountQuery,
) -> Result<AccountItems> {
    let mut out = Collector::default();
    let max_rows = query.row_limit() + 1;

    if query.review_requests || query.open_requests {
        let me = client.connection_user().await?.map(|u| u.id);
        if query.review_requests {
            match me.as_deref() {
                None => out.note("review requests: the server did not report the token's user"),
                Some(me) => {
                    let criteria = format!(
                        "searchCriteria.status=active&searchCriteria.reviewerId={}",
                        enc(me)
                    );
                    match client.org_pulls(&criteria, max_rows, &[], &mut out).await {
                        Ok(rows) => {
                            for pull in rows {
                                let row = client.account_pr(pull, Some(me))?;
                                if row.pr.review_requested {
                                    out.add_pr(row);
                                }
                            }
                        }
                        Err(e) => out.failed("review requests", e)?,
                    }
                }
            }
        }
        if query.open_requests {
            let criteria = "searchCriteria.status=active";
            match client
                .org_pulls(criteria, max_rows, query.search_namespaces(), &mut out)
                .await
            {
                Ok(rows) => {
                    for pull in rows {
                        // The namespaces filter runs on every row in `finish`.
                        out.add_pr(client.account_pr(pull, me.as_deref())?);
                    }
                }
                Err(e) => out.failed("open pull requests", e)?,
            }
        }
    }

    if query.assigned_issues {
        match client.account_issues(max_rows, &mut out).await {
            Ok(rows) => rows.into_iter().for_each(|i| out.add_issue(i)),
            Err(e) => out.failed("assigned work items", e)?,
        }
    }
    Ok(out.finish(query))
}
