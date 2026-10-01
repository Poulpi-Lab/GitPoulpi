//! Bitbucket Data Center through the CLI: Bearer token, `isLastPage` paging
//! and a request head fetched from `refs/pull-requests/<n>/from`.

use crate::common::*;

fn mock() -> Mock {
    route_pr(7)
}

fn route_pr(id: u64) -> Mock {
    let row = format!(
        r#"{{"id":{id},"title":"Add it","state":"OPEN","draft":false,"updatedDate":1760000000000,
        "fromRef":{{"id":"refs/heads/feature","displayId":"feature","repository":{{"slug":"repo","project":{{"key":"PRJ"}}}}}},
        "toRef":{{"id":"refs/heads/main","displayId":"main","repository":{{"slug":"repo","project":{{"key":"PRJ"}}}}}},
        "author":{{"user":{{"name":"ann","displayName":"Ann"}},"role":"AUTHOR"}},"reviewers":[],
        "links":{{"self":[{{"href":"https://git.example.com/projects/PRJ/repos/repo/pull-requests/{id}"}}]}}}}"#
    );
    Mock::start(vec![route(
        "GET",
        "/rest/api/1.0/projects/PRJ/repos/repo/pull-requests",
        &format!(r#"{{"size":1,"limit":100,"isLastPage":true,"values":[{row}]}}"#),
    )])
}

fn cli(name: &str, mock: &Mock) -> Cli {
    Cli::new(
        name,
        &integration("dc", "bitbucket_data_center", &mock.url(), ""),
        &[("dc", "tok")],
    )
}

#[test]
fn a_data_center_profile_needs_a_url_but_no_account() {
    let cli = Cli::new("add-dc", "", &[]);
    let here = std::path::Path::new(".");
    let err = cli.error(
        here,
        &[
            "hosting",
            "integrations",
            "add",
            "--kind",
            "bitbucket_data_center",
        ],
    );
    assert!(err.contains("--url"), "{err}");
    let added = cli.json(
        here,
        &[
            "hosting",
            "integrations",
            "add",
            "--kind",
            "bitbucket_data_center",
            "--url",
            "https://git.corp",
        ],
    );
    assert_eq!(added["added"]["api_base"], "https://git.corp/rest/api/1.0");
}

#[test]
fn pull_requests_are_listed_and_checked_out_from_the_hosts_head_ref() {
    let mock = mock();
    let cli = cli("bitbucket-dc", &mock);
    let repo = fixture();
    let tip = host_only_branch(&repo, "feature");
    cli.json(
        repo.root(),
        &[
            "hosting",
            "link",
            "pulls",
            "--id",
            "dc",
            "--project",
            "PRJ/repo",
        ],
    );

    let out = cli.json(repo.root(), &["hosting", "prs"]);
    assert_eq!(out["pull_requests"][0]["number"], 7);
    assert_eq!(
        mock.requests()[0].header("authorization"),
        Some("Bearer tok")
    );

    // The host keeps the head in its special ref.
    repo.git(&[
        "push",
        "-q",
        "origin",
        &format!("{tip}:refs/pull-requests/7/from"),
    ]);
    let out = cli.json(repo.root(), &["hosting", "checkout-pr", "7"]);
    assert_eq!(out["ref"], "refs/pull-requests/7/from");
    assert_eq!(out["branch"], "pr/7");
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), tip);
}
