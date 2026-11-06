//! Host-driven flows on Gitea and Forgejo: create the branch of an issue
//! (`POST /repos/{p}/branches`) and search branches by name. Gitea links
//! nothing to an issue itself, so the caller matches branches and pull
//! requests by the workflow's naming pattern.

use crate::issue_flow::{CreatedBranch, NewIssueBranch};
use crate::Result;

use super::GiteaClient;

#[derive(serde::Deserialize)]
struct BranchRow {
    name: String,
}

impl GiteaClient {
    pub(crate) async fn flow_create_branch(&self, req: &NewIssueBranch) -> Result<CreatedBranch> {
        let url = self.repo_url(&req.project, "branches");
        let body = serde_json::json!({
            "new_branch_name": req.name,
            "old_branch_name": req.from,
        });
        let response = self.transport.send(self.post(url, &body)).await?;
        response.error_for_status("creating a branch")?;
        let row: BranchRow = response.json()?;
        Ok(CreatedBranch {
            name: row.name,
            web_url: None,
        })
    }

    pub(crate) async fn flow_branch_names(
        &self,
        project: &str,
        search: &str,
    ) -> Result<Vec<String>> {
        let rows: Vec<BranchRow> = self
            .pages(
                &self.repo_url(project, "branches"),
                "listing the branches of a repository",
            )
            .await?;
        Ok(rows
            .into_iter()
            .map(|r| r.name)
            .filter(|n| n.contains(search))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use gitpoulpi_config::IntegrationKind;

    use crate::client::{client_for, HostingClient};
    use crate::http::{Method, RecordedTransport};
    use crate::integration::Integration;

    use super::*;

    fn client(t: Arc<RecordedTransport>) -> Box<dyn HostingClient> {
        client_for(
            Integration::new(
                "fj",
                IntegrationKind::Forgejo,
                "https://codeberg.example.com",
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

    #[test]
    fn a_branch_is_created_from_the_old_branch() {
        let t =
            Arc::new(RecordedTransport::new().on("/branches", 201, r#"{"name":"feature/7-x"}"#));
        let created = block_on(client(t.clone()).create_issue_branch(&NewIssueBranch {
            project: "o/r".into(),
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
            "https://codeberg.example.com/api/v1/repos/o/r/branches"
        );
        let body: serde_json::Value =
            serde_json::from_str(request.body.as_deref().unwrap()).unwrap();
        assert_eq!(body["new_branch_name"], "feature/7-x");
        assert_eq!(body["old_branch_name"], "develop");
    }

    #[test]
    fn branches_are_filtered_by_substring_and_links_are_empty() {
        let t = Arc::new(RecordedTransport::new().on(
            "/branches",
            200,
            r#"[{"name":"main"},{"name":"feature/7-x"}]"#,
        ));
        let c = client(t);
        assert_eq!(
            block_on(c.branch_names("o/r", "feature/7")).unwrap(),
            ["feature/7-x"]
        );
        let links = block_on(c.issue_links("o/r", 7)).unwrap();
        assert!(links.branches.is_empty() && links.requests.is_empty());
    }
}
