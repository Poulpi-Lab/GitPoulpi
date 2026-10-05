//! Gitea and Forgejo through the CLI (a loopback host standing for the
//! instance's `/api/v1`): token scheme, pull requests, issues, metadata and
//! the review commands. Assertions are on deserialized JSON.

use crate::common::*;

const PULL: &str = r#"{"number":3,"title":"Greet","state":"open","draft":false,"merged":false,
    "merged_at":null,"html_url":"https://git.example/o/p/pulls/3","updated_at":"2026-10-14T20:00:00Z",
    "user":{"login":"poulpi"},"head":{"ref":"feature","sha":"h1","repo":{"full_name":"o/p"}},
    "base":{"ref":"master","sha":"b1","repo":{"full_name":"o/p"}},"assignees":null,
    "requested_reviewers":[{"login":"alice"}],"mergeable":true,"body":"text",
    "created_at":"2026-10-14T19:00:00Z","additions":7,"deletions":1,"changed_files":2,
    "labels":[],"milestone":null}"#;

fn mock() -> Mock {
    Mock::start(vec![
        route(
            "GET",
            "/api/v1/user/repos",
            r#"[{"full_name":"o/p","name":"p","private":true,"clone_url":"https://git.example/o/p.git"}]"#,
        ),
        route(
            "GET",
            "/api/v1/user",
            r#"{"id":1,"login":"poulpi","full_name":"Poulpi"}"#,
        ),
        route(
            "GET",
            "/api/v1/repos/o/p/pulls/3/files",
            r#"[{"filename":"main.rs","status":"changed","additions":4,"deletions":1}]"#,
        ),
        route(
            "GET",
            "/api/v1/repos/o/p/pulls/3/reviews/2/comments",
            r#"[{"id":4,"body":"here","user":{"login":"poulpi"},"resolver":null,"path":"main.rs","commit_id":"h1","position":2,"original_position":0,"created_at":"x"}]"#,
        ),
        route(
            "GET",
            "/api/v1/repos/o/p/pulls/3/reviews",
            r#"[{"id":2,"state":"COMMENT","comments_count":1}]"#,
        ),
        route(
            "GET",
            "/api/v1/repos/o/p/commits/h1/status",
            r#"{"state":"success","total_count":1}"#,
        ),
        route("GET", "/api/v1/repos/o/p/issues/3/comments", r#"[]"#),
        route("POST", "/api/v1/repos/o/p/pulls/3/reviews", r#"{"id":9}"#),
        route(
            "POST",
            "/api/v1/repos/o/p/pulls/comments/4/resolve",
            r#"{}"#,
        ),
        route("GET", "/api/v1/repos/o/p/pulls/3", PULL),
        route("GET", "/api/v1/repos/o/p/pulls", &format!("[{PULL}]")),
        route(
            "GET",
            "/api/v1/repos/o/p/issues/1",
            r#"{"assignees":[],"labels":[],"milestone":null}"#,
        ),
        route(
            "GET",
            "/api/v1/repos/o/p/issues",
            r#"[{"number":1,"title":"Crash","state":"open","html_url":"https://git.example/o/p/issues/1","user":{"login":"poulpi"},"assignees":null,"labels":[{"name":"bug"}]}]"#,
        ),
        route("PATCH", "/api/v1/repos/o/p/issues/1", r#"{}"#),
        route("PUT", "/api/v1/repos/o/p/issues/1/labels", r#"[]"#),
        route(
            "GET",
            "/api/v1/repos/o/p/labels",
            r#"[{"id":7,"name":"bug","color":"d73a4a"}]"#,
        ),
        route(
            "GET",
            "/api/v1/repos/o/p/milestones",
            r#"[{"id":2,"title":"v0.2","state":"open"}]"#,
        ),
        route(
            "GET",
            "/api/v1/repos/o/p/assignees",
            r#"[{"id":2,"login":"alice"}]"#,
        ),
    ])
}

fn cli(name: &str, kind: &str, mock: &Mock) -> Cli {
    Cli::new(
        name,
        &integration("gt", kind, &mock.url(), ""),
        &[("gt", "tok")],
    )
}

fn link(cli: &Cli, repo: &std::path::Path) {
    for what in ["pulls", "issues"] {
        cli.json(
            repo,
            &["hosting", "link", what, "--id", "gt", "--project", "o/p"],
        );
    }
}

#[test]
fn projects_pull_requests_and_issues_over_the_token_scheme() {
    let mock = mock();
    let cli = cli("gitea-lists", "gitea", &mock);
    let repo = fixture();

    let projects = cli.json(repo.root(), &["hosting", "projects", "--id", "gt"]);
    assert_eq!(projects[0]["full_name"], "o/p");
    assert_eq!(
        mock.to("/api/v1/user/repos")[0].header("authorization"),
        Some("token tok")
    );

    link(&cli, repo.root());
    let prs = cli.json(repo.root(), &["hosting", "prs"]);
    assert_eq!(prs["pull_requests"][0]["iid_label"], "#3");
    assert_eq!(prs["pull_requests"][0]["reviewers"][0], "alice");
    let issues = cli.json(repo.root(), &["hosting", "issues"]);
    assert_eq!(issues["issues"][0]["reference"], "#1");
    assert_eq!(issues["issues"][0]["labels"][0], "bug");

    let opened = cli.json(repo.root(), &["hosting", "open", "pr", "#3"]);
    assert_eq!(opened["url"], format!("{}/o/p/pulls/3", mock.url()));
}

#[test]
fn forgejo_is_the_same_client_and_metadata_edits_go_through_the_issue() {
    let mock = mock();
    let cli = cli("forgejo-meta", "forgejo", &mock);
    let repo = fixture();
    link(&cli, repo.root());

    let labels = cli.json(repo.root(), &["hosting", "labels"]);
    assert_eq!(labels["labels"][0]["name"], "bug");
    let members = cli.json(repo.root(), &["hosting", "members"]);
    assert_eq!(members["members"][0]["login"], "alice");
    let milestones = cli.json(repo.root(), &["hosting", "milestones"]);
    assert_eq!(milestones["milestones"][0]["title"], "v0.2");

    cli.json(
        repo.root(),
        &[
            "hosting",
            "issue",
            "edit",
            "1",
            "--add-label",
            "bug",
            "--add-assignee",
            "alice",
            "--milestone",
            "v0.2",
        ],
    );
    let patch = &mock
        .to("/repos/o/p/issues/1")
        .into_iter()
        .find(|r| r.method == "PATCH")
        .expect("a PATCH on the issue");
    let body: serde_json::Value = serde_json::from_str(&patch.body).unwrap();
    assert_eq!(body["assignees"][0], "alice");
    assert_eq!(body["milestone"], 2);
    assert!(mock
        .requests()
        .iter()
        .any(|r| r.method == "PUT" && r.path().ends_with("/issues/1/labels")));
}

#[test]
fn review_reads_threads_and_submits_with_the_servers_event_words() {
    let mock = mock();
    let cli = cli("gitea-review", "gitea", &mock);
    let repo = fixture();
    link(&cli, repo.root());

    let shown = cli.json(repo.root(), &["hosting", "review", "show", "3"]);
    assert_eq!(shown["detail"]["head_sha"], "h1");
    assert_eq!(shown["detail"]["checks"], "passing");
    assert_eq!(shown["detail"]["capabilities"]["resolve"], true);
    assert_eq!(shown["detail"]["capabilities"]["line_ranges"], false);
    assert_eq!(shown["files"][0]["path"], "main.rs");
    assert_eq!(shown["head_ref"], "refs/gitpoulpi/pr/3");

    let threads = cli.json(repo.root(), &["hosting", "review", "threads", "3"]);
    let id = threads["threads"][0]["id"].as_str().unwrap().to_string();
    assert_eq!(id, "4|n|2|main.rs");

    cli.json(repo.root(), &["hosting", "review", "resolve", "3", &id]);

    cli.json(
        repo.root(),
        &[
            "hosting",
            "review",
            "submit",
            "3",
            "--event",
            "approve",
            "--comment",
            "main.rs:2:new:fine",
        ],
    );
    let post = mock
        .to("/pulls/3/reviews")
        .into_iter()
        .find(|r| r.method == "POST")
        .expect("the review");
    let body: serde_json::Value = serde_json::from_str(&post.body).unwrap();
    assert_eq!(body["event"], "APPROVED");
    assert_eq!(body["comments"][0]["new_position"], 2);
}
