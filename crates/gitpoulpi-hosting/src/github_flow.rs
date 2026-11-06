//! Host-driven flows on GitHub (and Enterprise Server): create the linked
//! branch of an issue (GraphQL `createLinkedBranch`), and tell what is linked
//! to an issue (GraphQL `linkedBranches` and the timeline's `connected` and
//! `cross-referenced` events, which is how a pull request that closes an
//! issue shows up).
//!
//! The mutation needs the issue's node id and the base commit's sha: both
//! come from REST first (`issues/:n` has `node_id`, `git/ref/heads/:b` has
//! the sha), so only the mutation and the links query are GraphQL.

use crate::client::encode_component;
use crate::github::GitHubClient;
use crate::issue_flow::{CreatedBranch, IssueLinks, NewIssueBranch};
use crate::model::{PullRequestState, PullRequestSummary};
use crate::{Error, Result};

impl GitHubClient {
    pub(crate) async fn flow_create_branch(&self, req: &NewIssueBranch) -> Result<CreatedBranch> {
        #[derive(serde::Deserialize)]
        struct Issue {
            node_id: String,
        }
        #[derive(serde::Deserialize)]
        struct Ref {
            object: Object,
        }
        #[derive(serde::Deserialize)]
        struct Object {
            sha: String,
        }
        let api = self.integration.api_base();
        let issue = self
            .transport
            .send(self.get(format!(
                "{api}/repos/{}/issues/{}",
                req.project, req.issue_number
            )))
            .await?;
        issue.error_for_status("looking the issue up")?;
        let issue: Issue = issue.json()?;
        let base = self
            .transport
            .send(self.get(format!(
                "{api}/repos/{}/git/ref/heads/{}",
                req.project,
                req.from.split('/').map(encode_component).collect::<Vec<_>>().join("/")
            )))
            .await?;
        base.error_for_status("looking the base branch up")?;
        let base: Ref = base.json()?;
        let value = self
            .graphql(
                "creating a linked branch",
                "mutation($issue: ID!, $oid: GitObjectID!, $name: String!) { \
                 createLinkedBranch(input: {issueId: $issue, oid: $oid, name: $name}) \
                 { linkedBranch { ref { name } } } }",
                serde_json::json!({ "issue": issue.node_id, "oid": base.object.sha, "name": req.name }),
            )
            .await?;
        let name = value
            .pointer("/createLinkedBranch/linkedBranch/ref/name")
            .and_then(|n| n.as_str())
            .unwrap_or(&req.name)
            .to_string();
        Ok(CreatedBranch {
            name,
            web_url: None,
        })
    }

    pub(crate) async fn flow_issue_links(&self, project: &str, number: u64) -> Result<IssueLinks> {
        let (owner, name) = project
            .split_once('/')
            .ok_or_else(|| Error::BadUrl(project.to_string()))?;
        const PR: &str = "number title state isDraft url headRefName baseRefName updatedAt \
                          author { login }";
        let query = format!(
            "query($o: String!, $n: String!, $i: Int!) {{ repository(owner: $o, name: $n) {{ \
             issue(number: $i) {{ linkedBranches(first: 50) {{ nodes {{ ref {{ name }} }} }} \
             timelineItems(first: 100, itemTypes: [CONNECTED_EVENT, CROSS_REFERENCED_EVENT]) \
             {{ nodes {{ __typename \
             ... on ConnectedEvent {{ subject {{ __typename ... on PullRequest {{ {PR} }} }} }} \
             ... on CrossReferencedEvent {{ source {{ __typename ... on PullRequest {{ {PR} }} }} }} \
             }} }} }} }} }}"
        );
        let value = self
            .graphql(
                "reading the links of an issue",
                &query,
                serde_json::json!({ "o": owner, "n": name, "i": number }),
            )
            .await?;
        let issue = value.pointer("/repository/issue");
        let mut links = IssueLinks::default();
        let Some(issue) = issue else {
            return Ok(links);
        };
        if let Some(nodes) = issue
            .pointer("/linkedBranches/nodes")
            .and_then(|n| n.as_array())
        {
            for node in nodes {
                if let Some(n) = node.pointer("/ref/name").and_then(|n| n.as_str()) {
                    links.branches.push(n.to_string());
                }
            }
        }
        if let Some(nodes) = issue
            .pointer("/timelineItems/nodes")
            .and_then(|n| n.as_array())
        {
            for node in nodes {
                let pr = node
                    .get("subject")
                    .or_else(|| node.get("source"))
                    .filter(|p| {
                        p.get("__typename").and_then(|t| t.as_str()) == Some("PullRequest")
                    });
                if let Some(pr) = pr.and_then(pull_summary) {
                    if !links.requests.iter().any(|r| r.number == pr.number) {
                        links.requests.push(pr);
                    }
                }
            }
        }
        Ok(links)
    }

    pub(crate) async fn flow_branch_names(
        &self,
        project: &str,
        prefix: &str,
    ) -> Result<Vec<String>> {
        #[derive(serde::Deserialize)]
        struct Row {
            #[serde(rename = "ref")]
            name: String,
        }
        let url = format!(
            "{}/repos/{project}/git/matching-refs/heads/{}?per_page=100",
            self.integration.api_base(),
            prefix
                .split('/')
                .map(encode_component)
                .collect::<Vec<_>>()
                .join("/")
        );
        let response = self.transport.send(self.get(url)).await?;
        response.error_for_status("searching the branches of a GitHub repository")?;
        let rows: Vec<Row> = response.json()?;
        Ok(rows
            .into_iter()
            .map(|r| {
                r.name
                    .strip_prefix("refs/heads/")
                    .unwrap_or(&r.name)
                    .to_string()
            })
            .collect())
    }
}

fn pull_summary(pr: &serde_json::Value) -> Option<PullRequestSummary> {
    let text = |k: &str| {
        pr.get(k)
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string()
    };
    let number = pr.get("number")?.as_u64()?;
    let state = match text("state").as_str() {
        "MERGED" => PullRequestState::Merged,
        "CLOSED" => PullRequestState::Closed,
        _ if pr.get("isDraft").and_then(|d| d.as_bool()) == Some(true) => PullRequestState::Draft,
        _ => PullRequestState::Open,
    };
    Some(PullRequestSummary {
        number,
        iid_label: format!("#{number}"),
        title: text("title"),
        state,
        source_branch: text("headRefName"),
        target_branch: text("baseRefName"),
        source_project: None,
        author: pr
            .pointer("/author/login")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        web_url: text("url"),
        updated_at: text("updatedAt"),
        review_requested: false,
        assignees: Vec::new(),
        reviewers: Vec::new(),
        checks: None,
        has_conflicts: None,
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use gitpoulpi_config::IntegrationKind;

    use crate::client::{client_for, HostingClient};
    use crate::http::{Method, RecordedTransport};
    use crate::integration::Integration;

    use super::*;

    fn client(t: Arc<RecordedTransport>, url: &str) -> Box<dyn HostingClient> {
        let kind = if url.contains("github.com") {
            IntegrationKind::Github
        } else {
            IntegrationKind::GithubEnterprise
        };
        client_for(
            Integration::new("gh", kind, url),
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
    fn a_linked_branch_is_made_from_the_issue_node_and_the_base_sha() {
        let t = Arc::new(
            RecordedTransport::new()
                .on("/issues/7", 200, r#"{"node_id":"I_kwDO7","number":7}"#)
                .on("/git/ref/heads/develop", 200, r#"{"object":{"sha":"abc123"}}"#)
                .on(
                    "/graphql",
                    200,
                    r#"{"data":{"createLinkedBranch":{"linkedBranch":{"ref":{"name":"feature/7-x"}}}}}"#,
                ),
        );
        let c = client(t.clone(), "https://github.com");
        let created = block_on(c.create_issue_branch(&NewIssueBranch {
            project: "o/r".into(),
            issue_number: 7,
            name: "feature/7-x".into(),
            from: "develop".into(),
        }))
        .unwrap();
        assert_eq!(created.name, "feature/7-x");
        let requests = t.requests();
        assert_eq!(requests[2].method, Method::Post);
        assert_eq!(requests[2].url, "https://api.github.com/graphql");
        let body: serde_json::Value =
            serde_json::from_str(requests[2].body.as_deref().unwrap()).unwrap();
        assert!(body["query"]
            .as_str()
            .unwrap()
            .contains("createLinkedBranch"));
        assert_eq!(body["variables"]["issue"], "I_kwDO7");
        assert_eq!(body["variables"]["oid"], "abc123");
        assert_eq!(body["variables"]["name"], "feature/7-x");
    }

    #[test]
    fn enterprise_server_has_its_own_graphql_root() {
        let t = Arc::new(
            RecordedTransport::new()
                .on("/issues/7", 200, r#"{"node_id":"I"}"#)
                .on("/git/ref/heads/main", 200, r#"{"object":{"sha":"a"}}"#)
                .on(
                    "/api/graphql",
                    200,
                    r#"{"data":{"createLinkedBranch":null}}"#,
                ),
        );
        let c = client(t.clone(), "https://ghe.example.com");
        block_on(c.create_issue_branch(&NewIssueBranch {
            project: "o/r".into(),
            issue_number: 7,
            name: "b".into(),
            from: "main".into(),
        }))
        .unwrap();
        assert_eq!(t.requests()[2].url, "https://ghe.example.com/api/graphql");
    }

    #[test]
    fn graphql_errors_are_reported() {
        let t = Arc::new(
            RecordedTransport::new()
                .on("/issues/7", 200, r#"{"node_id":"I"}"#)
                .on("/git/ref/heads/main", 200, r#"{"object":{"sha":"a"}}"#)
                .on(
                    "/graphql",
                    200,
                    r#"{"errors":[{"message":"Could not resolve"}]}"#,
                ),
        );
        let c = client(t, "https://github.com");
        let err = block_on(c.create_issue_branch(&NewIssueBranch {
            project: "o/r".into(),
            issue_number: 7,
            name: "b".into(),
            from: "main".into(),
        }))
        .unwrap_err();
        assert!(err.to_string().contains("Could not resolve"), "{err}");
    }

    #[test]
    fn the_links_are_the_linked_branches_and_the_pull_requests_of_the_timeline() {
        let body = r#"{"data":{"repository":{"issue":{
            "linkedBranches":{"nodes":[{"ref":{"name":"feature/7-x"}}]},
            "timelineItems":{"nodes":[
              {"__typename":"CrossReferencedEvent","source":{"__typename":"PullRequest","number":9,"title":"t","state":"OPEN","isDraft":true,"url":"https://github.com/o/r/pull/9","headRefName":"feature/7-x","baseRefName":"main","updatedAt":"2026-01-01T00:00:00Z","author":{"login":"bot"}}},
              {"__typename":"ConnectedEvent","subject":{"__typename":"PullRequest","number":9,"title":"t","state":"OPEN","isDraft":true,"url":"u","headRefName":"feature/7-x","baseRefName":"main","updatedAt":"","author":null}},
              {"__typename":"CrossReferencedEvent","source":{"__typename":"Issue"}}
            ]}}}}}"#;
        let t = Arc::new(RecordedTransport::new().on("/graphql", 200, body));
        let links = block_on(client(t, "https://github.com").issue_links("o/r", 7)).unwrap();
        assert_eq!(links.branches, ["feature/7-x"]);
        assert_eq!(links.requests.len(), 1);
        assert_eq!(links.requests[0].number, 9);
        assert_eq!(links.requests[0].state, PullRequestState::Draft);
        assert_eq!(links.requests[0].target_branch, "main");
    }

    #[test]
    fn branches_are_matched_by_prefix_and_lose_refs_heads() {
        let t = Arc::new(RecordedTransport::new().on(
            "/git/matching-refs/heads/feature/7",
            200,
            r#"[{"ref":"refs/heads/feature/7-x"}]"#,
        ));
        let names =
            block_on(client(t, "https://github.com").branch_names("o/r", "feature/7")).unwrap();
        assert_eq!(names, ["feature/7-x"]);
    }
}
