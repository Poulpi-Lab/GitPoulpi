//! Branches of account-wide requests: the search endpoints of GitHub and
//! Gitea carry none, so they are read afterwards (batched, then remembered).

use std::sync::Arc;

use super::*;
use crate::client::{client_for, HostingClient};
use crate::http::RecordedTransport;
use crate::{Integration, IntegrationKind};

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

fn client(kind: IntegrationKind, url: &str, t: &Arc<RecordedTransport>) -> Box<dyn HostingClient> {
    let mut integration = Integration::new("acct", kind, url);
    integration.user = Some(gitpoulpi_config::ConnectedUser {
        login: "octo".to_string(),
        ..Default::default()
    });
    client_for(
        integration,
        "tok".into(),
        t.clone(),
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap()
}

fn review() -> AccountQuery {
    AccountQuery {
        review_requests: true,
        ..AccountQuery::default()
    }
}

fn gh_item(n: u64, repo: &str) -> String {
    format!(
        r#"{{"number":{n},"node_id":"PR_{n}","title":"t{n}","html_url":"https://github.com/{repo}/pull/{n}",
        "updated_at":"2026-10-02T00:00:00Z","repository_url":"https://api.github.com/repos/{repo}",
        "user":{{"login":"alice"}},"assignees":[],"labels":[],"milestone":null}}"#
    )
}

fn graphql_calls(t: &RecordedTransport) -> usize {
    t.requests()
        .iter()
        .filter(|r| r.url.contains("/graphql"))
        .count()
}

#[test]
fn github_search_rows_get_their_branches_from_one_batched_graphql_call() {
    let search = format!(
        r#"{{"items":[{},{}]}}"#,
        gh_item(9101, "octo/app"),
        gh_item(9102, "octo/lib")
    );
    let nodes = r#"{"data":{"nodes":[
        {"number":9101,"repository":{"nameWithOwner":"octo/app"},"headRefName":"feature/login","baseRefName":"main","headRepository":{"nameWithOwner":"octo/app"}},
        {"number":9102,"repository":{"nameWithOwner":"octo/lib"},"headRefName":"fix","baseRefName":"develop","headRepository":{"nameWithOwner":"alice/lib"}}]}}"#;
    let t = Arc::new(
        RecordedTransport::new()
            .on("/search/issues", 200, &search)
            .on("/graphql", 200, nodes),
    );
    let c = client(IntegrationKind::Github, "https://github.com", &t);
    let items = block_on(c.account_items(&review())).unwrap();
    let app = items.prs.iter().find(|p| p.pr.number == 9101).unwrap();
    assert_eq!(app.pr.source_branch, "feature/login");
    assert_eq!(app.pr.target_branch, "main");
    assert_eq!(app.pr.source_project, None);
    let lib = items.prs.iter().find(|p| p.pr.number == 9102).unwrap();
    assert_eq!(lib.pr.target_branch, "develop");
    assert_eq!(lib.pr.source_project.as_deref(), Some("alice/lib"));
    assert_eq!(graphql_calls(&t), 1, "one call for both rows");

    // A second round with the same rows asks nothing more about them.
    let again = block_on(c.account_items(&review())).unwrap();
    assert!(!again.prs[0].pr.target_branch.is_empty());
    assert_eq!(
        graphql_calls(&t),
        1,
        "remembered per request and updated_at"
    );
}

#[test]
fn a_failing_branch_lookup_keeps_the_rows() {
    let search = format!(r#"{{"items":[{}]}}"#, gh_item(9201, "octo/app"));
    let t = Arc::new(
        RecordedTransport::new()
            .on("/search/issues", 200, &search)
            .on("/graphql", 502, "bad gateway"),
    );
    let c = client(IntegrationKind::Github, "https://github.com", &t);
    let items = block_on(c.account_items(&review())).unwrap();
    assert_eq!(items.prs.len(), 1);
    assert!(items.prs[0].pr.source_branch.is_empty());
}

#[test]
fn gitea_search_rows_get_their_branches_from_the_request_detail() {
    let row = r#"{"number":9301,"title":"t","html_url":"https://git.example/octo/app/pulls/9301",
        "updated_at":"2026-10-03T00:00:00Z","user":{"login":"alice"},"assignees":null,
        "labels":null,"milestone":null,"repository":{"full_name":"octo/app"},"pull_request":{"draft":false}}"#;
    let detail = r#"{"number":9301,"title":"t","state":"open","html_url":"u","updated_at":"x",
        "head":{"ref":"feature/login","sha":"a","repo":{"full_name":"alice/app"}},
        "base":{"ref":"main","sha":"b","repo":{"full_name":"octo/app"}}}"#;
    let t = Arc::new(
        RecordedTransport::new()
            .on("/repos/octo/app/pulls/9301", 200, detail)
            .on("type=pulls", 200, &format!("[{row}]")),
    );
    let c = client(IntegrationKind::Forgejo, "https://git.example", &t);
    let items = block_on(c.account_items(&review())).unwrap();
    let pr = &items.prs[0].pr;
    assert_eq!(pr.source_branch, "feature/login");
    assert_eq!(pr.target_branch, "main");
    assert_eq!(pr.source_project.as_deref(), Some("alice/app"));
}

#[test]
fn gitlab_lists_carry_the_branches() {
    let mr = r#"[{"iid":7,"title":"mr","state":"opened","source_branch":"feature/login","target_branch":"main",
        "source_project_id":1,"target_project_id":1,"author":{"username":"alice"},
        "web_url":"https://gitlab.example/octo/app/-/merge_requests/7","updated_at":"2026-10-02T00:00:00Z",
        "assignees":[],"reviewers":[{"username":"octo"}],"references":{"full":"octo/app!7"}}]"#;
    let t = Arc::new(RecordedTransport::new().on("/merge_requests", 200, mr));
    let c = client(
        IntegrationKind::GitlabSelfManaged,
        "https://gitlab.example",
        &t,
    );
    let items = block_on(c.account_items(&review())).unwrap();
    assert_eq!(items.prs[0].pr.source_branch, "feature/login");
    assert_eq!(items.prs[0].pr.target_branch, "main");
}

#[test]
fn a_cache_entry_from_before_the_branches_existed_is_not_reused() {
    // The fingerprint of the old spelling (no schema tag): its entries carry
    // no branches and would be served until they aged out.
    let text = "r1o0i0d0l300x0|";
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in text.bytes() {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    assert_ne!(review().fingerprint(), format!("{hash:016x}"));
}
