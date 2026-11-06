//! Host-driven flows on GitLab (gitlab.com and self-managed): create the
//! branch of an issue, and tell what the host (or a bot) made for it.
//!
//! - branch: `POST /projects/:id/repository/branches` (`branch`, `ref`);
//! - merge requests of an issue: `GET issues/:iid/related_merge_requests`
//!   (mentions and closes) and `issues/:iid/closed_by` (what will close it);
//! - branches: `GET repository/branches?search=` (substring).

use crate::client::{encode_component, fetch_pages};
use crate::gitlab::{encode_gitlab_project_path, GitLabClient, GitLabMrRow};
use crate::issue_flow::{CreatedBranch, IssueLinks, NewIssueBranch};
use crate::Result;

#[derive(serde::Deserialize)]
struct BranchRow {
    name: String,
    #[serde(default)]
    web_url: Option<String>,
}

impl GitLabClient {
    pub(crate) async fn flow_create_branch(&self, req: &NewIssueBranch) -> Result<CreatedBranch> {
        let url = format!(
            "{}/projects/{}/repository/branches",
            self.integration.api_base(),
            encode_gitlab_project_path(&req.project)
        );
        let body = serde_json::json!({ "branch": req.name, "ref": req.from });
        let response = self.transport.send(self.post(url, &body)).await?;
        response.error_for_status("creating a GitLab branch")?;
        let row: BranchRow = response.json()?;
        Ok(CreatedBranch {
            name: row.name,
            web_url: row.web_url,
        })
    }

    pub(crate) async fn flow_issue_links(&self, project: &str, number: u64) -> Result<IssueLinks> {
        let base = format!(
            "{}/projects/{}/issues/{number}",
            self.integration.api_base(),
            encode_gitlab_project_path(project)
        );
        let related: Vec<GitLabMrRow> = fetch_pages(
            &*self.transport,
            |u| self.get(u),
            &format!("{base}/related_merge_requests?state=all"),
            "listing the merge requests of a GitLab issue",
        )
        .await?;
        let mut requests: Vec<_> = related.into_iter().map(|r| r.into_summary(false)).collect();
        // What will close the issue when merged: the same requests in the
        // usual case; a failure here (an older version) is not an error.
        if let Ok(closing) = fetch_pages::<GitLabMrRow>(
            &*self.transport,
            |u| self.get(u),
            &format!("{base}/closed_by?x=1"),
            "listing the merge requests closing a GitLab issue",
        )
        .await
        {
            for row in closing {
                let summary = row.into_summary(false);
                if !requests.iter().any(|r| r.number == summary.number) {
                    requests.push(summary);
                }
            }
        }
        Ok(IssueLinks {
            branches: Vec::new(),
            requests,
        })
    }

    pub(crate) async fn flow_branch_names(
        &self,
        project: &str,
        search: &str,
    ) -> Result<Vec<String>> {
        let url = format!(
            "{}/projects/{}/repository/branches?search={}",
            self.integration.api_base(),
            encode_gitlab_project_path(project),
            encode_component(search)
        );
        let rows: Vec<BranchRow> = fetch_pages(
            &*self.transport,
            |u| self.get(u),
            &url,
            "searching the branches of a GitLab project",
        )
        .await?;
        Ok(rows.into_iter().map(|r| r.name).collect())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use gitpoulpi_config::IntegrationKind;

    use crate::client::{client_for, HostingClient};
    use crate::http::{Method, RecordedTransport};
    use crate::integration::Integration;
    use crate::model::PullRequestState;

    use super::*;

    fn client(t: Arc<RecordedTransport>) -> Box<dyn HostingClient> {
        client_for(
            Integration::new(
                "gl",
                IntegrationKind::GitlabSelfManaged,
                "https://gitlab.example.com",
            ),
            "tok".into(),
            t,
            &crate::trust::TrustGate::for_tests(),
        )
        .unwrap()
    }

    fn block_on<F: std::future::Future>(f: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(f)
    }

    fn mr(iid: u64, state: &str, source: &str) -> String {
        format!(
            r#"{{"iid":{iid},"title":"Draft: t{iid}","state":"{state}","source_branch":"{source}","target_branch":"main",
            "source_project_id":1,"target_project_id":1,"author":{{"username":"bot"}},
            "web_url":"https://gitlab.example.com/g/p/-/merge_requests/{iid}","updated_at":"2026-01-01T00:00:00Z"}}"#
        )
    }

    #[test]
    fn a_branch_is_created_with_the_branches_api() {
        let t = Arc::new(RecordedTransport::new().on(
            "/repository/branches",
            201,
            r#"{"name":"feature/7-x","web_url":"https://gitlab.example.com/g/p/-/tree/feature/7-x"}"#,
        ));
        let created = block_on(client(t.clone()).create_issue_branch(&NewIssueBranch {
            project: "g/p".into(),
            issue_number: 7,
            name: "feature/7-x".into(),
            from: "develop".into(),
        }))
        .unwrap();
        assert_eq!(created.name, "feature/7-x");
        let request = &t.requests()[0];
        assert_eq!(request.method, Method::Post);
        assert_eq!(
            request.url,
            "https://gitlab.example.com/api/v4/projects/g%2Fp/repository/branches"
        );
        let body: serde_json::Value =
            serde_json::from_str(request.body.as_deref().unwrap()).unwrap();
        assert_eq!(body["branch"], "feature/7-x");
        assert_eq!(body["ref"], "develop");
    }

    #[test]
    fn the_requests_of_an_issue_are_the_related_and_the_closing_ones_without_duplicates() {
        let t = Arc::new(
            RecordedTransport::new()
                .on(
                    "/issues/7/related_merge_requests",
                    200,
                    &format!("[{}]", mr(3, "opened", "feature/7-x")),
                )
                .on(
                    "/issues/7/closed_by",
                    200,
                    &format!(
                        "[{},{}]",
                        mr(3, "opened", "feature/7-x"),
                        mr(4, "merged", "other")
                    ),
                ),
        );
        let links = block_on(client(t.clone()).issue_links("g/p", 7)).unwrap();
        let numbers: Vec<u64> = links.requests.iter().map(|r| r.number).collect();
        assert_eq!(numbers, [3, 4]);
        assert_eq!(links.requests[0].source_branch, "feature/7-x");
        assert_eq!(links.requests[0].state, PullRequestState::Open);
        assert_eq!(links.requests[1].state, PullRequestState::Merged);
        assert!(t.requests()[0]
            .url
            .contains("related_merge_requests?state=all"));
    }

    #[test]
    fn closed_by_failing_on_an_old_version_is_not_an_error() {
        let t = Arc::new(
            RecordedTransport::new()
                .on("/related_merge_requests", 200, "[]")
                .on("/closed_by", 404, "{}"),
        );
        let links = block_on(client(t).issue_links("g/p", 7)).unwrap();
        assert!(links.requests.is_empty());
    }

    #[test]
    fn branches_are_searched_by_substring() {
        let t = Arc::new(RecordedTransport::new().on(
            "/repository/branches",
            200,
            r#"[{"name":"feature/7-x"},{"name":"feature/70-y"}]"#,
        ));
        let names = block_on(client(t.clone()).branch_names("g/p", "feature/7")).unwrap();
        assert_eq!(names, ["feature/7-x", "feature/70-y"]);
        assert!(t.requests()[0].url.contains("search=feature%2F7"));
    }
}
