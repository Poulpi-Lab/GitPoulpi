//! Azure DevOps Server through the CLI (a loopback collection root): the
//! pull request list, branch-based checkout and work items.

use crate::common::*;

const PULL: &str = r#"{"pullRequestId":12,"title":"Add the parser","status":"active","isDraft":false,
    "sourceRefName":"refs/heads/feature","targetRefName":"refs/heads/main",
    "createdBy":{"displayName":"Kai","id":"u1"},"creationDate":"2026-10-01T10:00:00Z",
    "repository":{"id":"rid-1","name":"Repo","project":{"name":"P"}},"reviewers":[]}"#;

fn mock() -> Mock {
    let c = "/tfs/Coll";
    Mock::start(vec![
        route(
            "GET",
            &format!("{c}/P/_apis/git/repositories/rid-1/pullrequests"),
            &format!(r#"{{"value":[{PULL}]}}"#),
        ),
        route(
            "GET",
            &format!("{c}/P/_apis/git/repositories/Repo"),
            r#"{"id":"rid-1","name":"Repo","project":{"name":"P"}}"#,
        ),
        route(
            "GET",
            &format!("{c}/_apis/git/repositories"),
            r#"{"value":[{"id":"rid-1","name":"Repo","project":{"name":"P"},"remoteUrl":"https://tfs.corp/tfs/Coll/P/_git/Repo"}]}"#,
        ),
        route(
            "GET",
            &format!("{c}/_apis/connectionData"),
            r#"{"authenticatedUser":{"id":"u1","providerDisplayName":"Kai","properties":{"Account":{"$value":"kai@corp"}}}}"#,
        ),
        route(
            "POST",
            &format!("{c}/P/_apis/wit/wiql"),
            r#"{"workItems":[{"id":7}]}"#,
        ),
        route(
            "GET",
            &format!("{c}/P/_apis/wit/workitemtypes"),
            r#"{"value":[{"name":"Bug"},{"name":"Task"},{"name":"Off","isDisabled":true}]}"#,
        ),
        route(
            "GET",
            &format!("{c}/P/_apis/wit/workitems"),
            r#"{"value":[{"id":7,"fields":{"System.Title":"Crash","System.State":"Active","System.WorkItemType":"Bug","System.TeamProject":"P","System.ChangedDate":"2026-10-01T10:00:00Z"}}]}"#,
        ),
        (
            "POST",
            format!("{c}/P/_apis/wit/workitems/"),
            200,
            r#"{"id":99}"#.to_string(),
        ),
    ])
}

fn cli(name: &str, mock: &Mock) -> Cli {
    Cli::new(
        name,
        &integration(
            "az",
            "azure_devops_server",
            &format!("{}/tfs/Coll", mock.url()),
            "",
        ),
        &[("az", "pat")],
    )
}

#[test]
fn pull_requests_are_listed_and_found_by_branch_with_a_bang_label() {
    let mock = mock();
    let cli = cli("azure-prs", &mock);
    let repo = fixture();

    let tested = cli.json(
        repo.root(),
        &["hosting", "integrations", "test", "--id", "az"],
    );
    assert_eq!(tested["user"]["login"], "kai@corp");
    assert_eq!(
        mock.to("connectionData")[0].header("authorization"),
        Some("Basic OnBhdA==")
    );
    let projects = cli.json(repo.root(), &["hosting", "projects", "--id", "az"]);
    assert_eq!(projects[0]["full_name"], "P/Repo");

    cli.json(
        repo.root(),
        &[
            "hosting",
            "link",
            "pulls",
            "--id",
            "az",
            "--project",
            "P/Repo",
        ],
    );
    assert!(read_state(&repo).contains("project = \"P/Repo\""));
    let out = cli.json(repo.root(), &["hosting", "prs"]);
    let pr = &out["pull_requests"][0];
    assert_eq!(pr["iid_label"], "!12");
    assert_eq!(pr["source_branch"], "feature");
    assert_eq!(
        pr["web_url"],
        format!("{}/tfs/Coll/P/_git/Repo/pullrequest/12", mock.url())
    );

    let found = cli.json(repo.root(), &["hosting", "pr-for-branch", "feature"]);
    assert_eq!(found["pull_request"]["number"], 12);
    let requests = mock.to("/pullrequests");
    let last = decoded(&requests.last().unwrap().target);
    assert!(
        last.contains("searchCriteria.sourceRefName=refs/heads/feature"),
        "{last}"
    );
}

#[test]
fn checking_out_a_pull_request_tracks_its_source_branch_as_pr_n() {
    let mock = mock();
    let cli = cli("azure-checkout", &mock);
    let repo = fixture();
    let tip = host_only_branch(&repo, "feature");
    cli.json(
        repo.root(),
        &[
            "hosting",
            "link",
            "pulls",
            "--id",
            "az",
            "--project",
            "P/Repo",
        ],
    );

    let out = cli.json(repo.root(), &["hosting", "checkout-pr", "12"]);
    assert_eq!(out["branch"], "pr/12");
    assert_eq!(out["ref"], "refs/heads/feature");
    assert_eq!(out["checked_out"], true);
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), tip);
    assert_eq!(repo.git(&["rev-parse", "--abbrev-ref", "HEAD"]), "pr/12");
    assert_eq!(repo.git(&["config", "branch.pr/12.remote"]), "origin");

    // Not a pull request of the project: nothing is fetched.
    let err = cli.error(repo.root(), &["hosting", "checkout-pr", "404"]);
    assert!(err.contains("404"), "{err}");
}

#[test]
fn work_items_list_open_and_create_with_the_chosen_type() {
    let mock = mock();
    let cli = cli("azure-items", &mock);
    let repo = fixture();
    cli.json(
        repo.root(),
        &[
            "hosting",
            "link",
            "issues",
            "--id",
            "az",
            "--project",
            "P",
            "--filter",
            "[System.Tags] CONTAINS 'ui'",
        ],
    );

    let out = cli.json(repo.root(), &["hosting", "issues"]);
    assert_eq!(out["issues"][0]["number"], 7);
    assert_eq!(out["issues"][0]["kind"], "Bug");
    assert_eq!(out["issues"][0]["status"], "Active");
    let wiql: serde_json::Value = serde_json::from_str(&mock.to("/wiql")[0].body).unwrap();
    assert!(
        wiql["query"]
            .as_str()
            .unwrap()
            .contains("([System.Tags] CONTAINS 'ui')"),
        "{wiql}"
    );

    let opened = cli.json(repo.root(), &["hosting", "open", "issue", "AB#7"]);
    assert_eq!(
        opened["url"],
        format!("{}/tfs/Coll/P/_workitems/edit/7", mock.url())
    );
    let opened = cli.json(repo.root(), &["hosting", "open", "issue", "#7"]);
    assert_eq!(
        opened["url"],
        format!("{}/tfs/Coll/P/_workitems/edit/7", mock.url())
    );

    let options = cli.json(repo.root(), &["hosting", "issue", "options"]);
    assert_eq!(options["types"].as_array().unwrap().len(), 2, "{options}");

    let created = cli.json(
        repo.root(),
        &[
            "hosting", "issue", "create", "--title", "New", "--type", "Bug",
        ],
    );
    assert_eq!(created["number"], 99);
    let post = mock
        .requests()
        .into_iter()
        .find(|r| r.method == "POST" && r.path().contains("workitems/"))
        .unwrap();
    assert!(post.path().ends_with("/workitems/$Bug"), "{}", post.path());
    assert_eq!(
        post.header("content-type"),
        Some("application/json-patch+json")
    );
}
