//! Jira Cloud through the CLI: a loopback mock answers the REST routes.

use crate::common::*;

const ISSUE: &str = r#"{"issues":[{"id":"10001","key":"ABC-12","fields":{"summary":"Crash on start",
    "status":{"name":"In Progress","statusCategory":{"key":"indeterminate"}},
    "issuetype":{"name":"Bug"},"labels":[],"updated":"2026-10-01T10:00:00.000+0000"}}],"isLast":true}"#;

fn mock() -> Mock {
    Mock::start(vec![
        route(
            "GET",
            "/rest/api/3/myself",
            r#"{"accountId":"a1","displayName":"Me","emailAddress":"me@x.io"}"#,
        ),
        route(
            "GET",
            "/rest/api/3/project/search",
            r#"{"values":[{"key":"ABC","name":"Alpha"},{"key":"XYZ","name":"Xray"}],"isLast":true}"#,
        ),
        route("GET", "/rest/api/3/search/jql", ISSUE),
        route(
            "GET",
            "/rest/api/3/issue/createmeta/ABC/issuetypes",
            r#"{"issueTypes":[{"id":"10001","name":"Task"},{"id":"10002","name":"Bug"},{"id":"10003","name":"Sub-task","subtask":true}],"total":3}"#,
        ),
        (
            "POST",
            "/rest/api/2/issue".to_string(),
            201,
            r#"{"id":"10050","key":"ABC-13"}"#.to_string(),
        ),
    ])
}

fn cli(name: &str, mock: &Mock) -> Cli {
    Cli::new(
        name,
        &integration("jira", "jira_cloud", &mock.url(), "account = \"me@x.io\""),
        &[("jira", "tok")],
    )
}

#[test]
fn the_token_check_and_the_projects_use_the_email_and_token() {
    let mock = mock();
    let cli = cli("jira-conn", &mock);
    let tested = cli.json(
        std::path::Path::new("."),
        &["hosting", "integrations", "test", "--id", "jira"],
    );
    assert_eq!(tested["user"]["login"], "me@x.io");
    let me = mock.to("/myself");
    assert_eq!(
        me[0].header("authorization"),
        Some("Basic bWVAeC5pbzp0b2s=")
    );

    let projects = cli.json(
        std::path::Path::new("."),
        &["hosting", "projects", "--id", "jira", "--filter", "alp"],
    );
    let rows = projects.as_array().unwrap();
    assert_eq!(rows.len(), 1, "{projects}");
    assert_eq!(rows[0]["full_name"], "ABC");
    assert_eq!(rows[0]["web_url"], format!("{}/browse/ABC", mock.url()));
}

#[test]
fn a_linked_project_with_a_filter_lists_issues_with_their_reference() {
    let mock = mock();
    let cli = cli("jira-issues", &mock);
    let repo = fixture();

    let linked = cli.json(
        repo.root(),
        &[
            "hosting",
            "link",
            "issues",
            "--id",
            "jira",
            "--project",
            "ABC",
            "--filter",
            "component = UI",
        ],
    );
    assert_eq!(linked["linked"], true);
    assert_eq!(linked["project"], "ABC");
    assert_eq!(linked["filter"], "component = UI");
    assert!(
        read_state(&repo).contains("filter = \"component = UI\""),
        "{}",
        read_state(&repo)
    );
    // Showing the link keeps the filter.
    let shown = cli.json(repo.root(), &["hosting", "link", "issues"]);
    assert_eq!(shown["filter"], "component = UI");

    let out = cli.json(repo.root(), &["hosting", "issues"]);
    let row = &out["issues"][0];
    assert_eq!(row["reference"], "ABC-12");
    assert_eq!(row["number"], 12);
    assert_eq!(row["kind"], "Bug");
    assert_eq!(row["status"], "In Progress");
    let search = mock.to("/search/jql");
    let query = decoded(search[0].target.split_once('?').unwrap().1);
    assert!(query.contains("project = \"ABC\""), "{query}");
    assert!(query.contains("(component = UI)"), "{query}");

    // A reference, not a number, opens an issue.
    let opened = cli.json(repo.root(), &["hosting", "open", "issue", "ABC-12"]);
    assert_eq!(opened["url"], format!("{}/browse/ABC-12", mock.url()));

    // Jira has no pull requests.
    let err = cli.error(
        repo.root(),
        &[
            "hosting",
            "link",
            "pulls",
            "--id",
            "jira",
            "--project",
            "ABC",
        ],
    );
    assert!(err.to_lowercase().contains("pull"), "{err}");

    // Unlinking drops the filter with the source.
    cli.json(repo.root(), &["hosting", "link", "issues", "--clear"]);
    assert!(!read_state(&repo).contains("filter"));
}

#[test]
fn issue_options_and_create_send_the_chosen_type_id() {
    let mock = mock();
    let cli = cli("jira-create", &mock);
    let repo = fixture();
    write_state(
        &repo,
        "[issues_source]\nintegration = \"jira\"\nproject = \"ABC\"\n",
    );

    let options = cli.json(repo.root(), &["hosting", "issue", "options"]);
    let types = options["types"].as_array().unwrap();
    assert_eq!(types.len(), 2, "sub-tasks are not offered: {options}");
    assert_eq!(types[1], serde_json::json!({"id": "10002", "label": "Bug"}));
    assert_eq!(options["lists"].as_array().unwrap().len(), 0);

    let created = cli.json(
        repo.root(),
        &[
            "hosting", "issue", "create", "--title", "Broken", "--body", "Steps", "--type", "bug",
        ],
    );
    assert_eq!(created["reference"], "ABC-13");
    assert_eq!(created["web_url"], format!("{}/browse/ABC-13", mock.url()));
    let post = &mock.to("/rest/api/2/issue")[0];
    let body: serde_json::Value = serde_json::from_str(&post.body).unwrap();
    assert_eq!(body["fields"]["issuetype"]["id"], "10002");
    assert_eq!(body["fields"]["project"]["key"], "ABC");

    // By id too; an unknown type names the choices and posts nothing.
    cli.json(
        repo.root(),
        &[
            "hosting", "issue", "create", "--title", "T", "--type", "10001",
        ],
    );
    let err = cli.error(
        repo.root(),
        &[
            "hosting", "issue", "create", "--title", "T", "--type", "Epic",
        ],
    );
    assert!(err.contains("Task") && err.contains("Bug"), "{err}");
    assert_eq!(mock.to("/rest/api/2/issue").len(), 2);

    // A list means nothing to Jira.
    let err = cli.error(
        repo.root(),
        &[
            "hosting", "issue", "create", "--title", "T", "--list", "Doing",
        ],
    );
    assert!(err.contains("--list"), "{err}");
}
