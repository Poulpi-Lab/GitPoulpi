use super::fake::*;
use super::*;
use gitpoulpi_hosting::PullRequestState;

mod cache;
mod fetch;
mod policy;

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

fn gitlab_profile() -> Integration {
    Integration::new(
        "corp",
        IntegrationKind::GitlabSelfManaged,
        "https://gitlab.example.com",
    )
}

fn request(remote_url: &str) -> HostingRequest {
    HostingRequest {
        remotes: vec![ResolveRemote::new("origin", remote_url)],
        integrations: vec![gitlab_profile()],
        override_id: None,
        head_branch: Some("feature".into()),
        pr_view: PrView::Open,
        issue_view: IssueView::Open,
        issues_source: Some(gitpoulpi_session::IssuesSource {
            integration: "corp".into(),
            project: "g/p".into(),
            filter: None,
        }),
        pulls_source: Some(gitpoulpi_session::IssuesSource {
            integration: "corp".into(),
            project: "g/p".into(),
            filter: None,
        }),
    }
}

fn unlinked_request(remote_url: &str) -> HostingRequest {
    HostingRequest {
        issues_source: None,
        pulls_source: None,
        ..request(remote_url)
    }
}
