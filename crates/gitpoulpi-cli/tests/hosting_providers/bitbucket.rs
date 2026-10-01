//! Bitbucket Cloud through the CLI (a loopback host standing for
//! bitbucket.org's API): workspaces, pull requests, branch checkout.

use crate::common::*;

fn mock() -> Mock {
    Mock::start(vec![
        route(
            "GET",
            "/2.0/repositories/ws/repo/pullrequests",
            r#"{"values":[{"id":5,"title":"Add it","state":"OPEN","source":{"branch":{"name":"feature"}},
                "destination":{"branch":{"name":"main"}},"author":{"display_name":"Me"},
                "updated_on":"2026-10-01T10:00:00Z",
                "links":{"html":{"href":"https://bitbucket.org/ws/repo/pull-requests/5"}}}]}"#,
        ),
        route(
            "GET",
            "/2.0/repositories/ws",
            r#"{"values":[{"full_name":"ws/repo","name":"repo","links":{"html":{"href":"https://bitbucket.org/ws/repo"},"clone":[{"name":"https","href":"https://me@bitbucket.org/ws/repo.git"}]}}]}"#,
        ),
        route(
            "GET",
            "/2.0/user/workspaces",
            r#"{"values":[{"workspace":{"slug":"ws"}}]}"#,
        ),
        route(
            "GET",
            "/2.0/user",
            r#"{"uuid":"{u1}","nickname":"me","display_name":"Me"}"#,
        ),
    ])
}

fn cli(name: &str, mock: &Mock) -> Cli {
    Cli::new(
        name,
        &integration("bb", "bitbucket", &mock.url(), "account = \"me@x.io\""),
        &[("bb", "tok")],
    )
}

#[test]
fn projects_and_pull_requests_with_a_hash_label() {
    let mock = mock();
    let cli = cli("bitbucket-prs", &mock);
    let repo = fixture();

    let projects = cli.json(repo.root(), &["hosting", "projects", "--id", "bb"]);
    assert_eq!(projects[0]["full_name"], "ws/repo");
    assert_eq!(
        mock.to("/user/workspaces")[0].header("authorization"),
        Some("Basic bWVAeC5pbzp0b2s=")
    );

    cli.json(
        repo.root(),
        &[
            "hosting",
            "link",
            "pulls",
            "--id",
            "bb",
            "--project",
            "ws/repo",
        ],
    );
    let out = cli.json(repo.root(), &["hosting", "prs"]);
    assert_eq!(out["pull_requests"][0]["iid_label"], "#5");
    let found = cli.json(repo.root(), &["hosting", "pr-for-branch", "feature"]);
    assert_eq!(found["pull_request"]["number"], 5);
    let last = decoded(&mock.to("/pullrequests").last().unwrap().target);
    assert!(last.contains("source.branch.name=\"feature\""), "{last}");

    // The pull request page, from the host's own scheme.
    let opened = cli.json(repo.root(), &["hosting", "open", "pr", "#5"]);
    assert_eq!(
        opened["url"],
        format!("{}/ws/repo/pull-requests/5", mock.url())
    );

    // Bitbucket retired its issues: linking them says where to go instead.
    let err = cli.error(
        repo.root(),
        &[
            "hosting",
            "link",
            "issues",
            "--id",
            "bb",
            "--project",
            "ws/repo",
        ],
    );
    assert!(err.contains("Jira"), "{err}");
}

#[test]
fn checking_out_a_pull_request_tracks_its_source_branch_as_pr_n() {
    let mock = mock();
    let cli = cli("bitbucket-checkout", &mock);
    let repo = fixture();
    let tip = host_only_branch(&repo, "feature");
    cli.json(
        repo.root(),
        &[
            "hosting",
            "link",
            "pulls",
            "--id",
            "bb",
            "--project",
            "ws/repo",
        ],
    );

    let out = cli.json(repo.root(), &["hosting", "checkout-pr", "5"]);
    assert_eq!(out["branch"], "pr/5");
    assert_eq!(out["ref"], "refs/heads/feature");
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), tip);
    assert_eq!(repo.git(&["rev-parse", "--abbrev-ref", "HEAD"]), "pr/5");
}
