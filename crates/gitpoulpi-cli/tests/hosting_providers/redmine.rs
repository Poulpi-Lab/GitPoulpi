//! Redmine through the CLI: a loopback mock answers the REST routes.

use std::path::Path;

use crate::common::*;

const ISSUES: &str = r#"{"issues":[
  {"id":12,"project":{"id":1,"name":"Bench"},"tracker":{"id":1,"name":"Bug"},
   "status":{"id":2,"name":"In Progress","is_closed":false},"author":{"id":1,"name":"Bob"},
   "assigned_to":{"id":4,"name":"Ann Lee"},"fixed_version":{"id":7,"name":"v0.1"},
   "subject":"Crash on start","updated_on":"2026-10-15T06:13:39Z","closed_on":null}],
  "total_count":1,"offset":0,"limit":100}"#;

const STATUSES: &str = r#"{"issue_statuses":[
  {"id":1,"name":"New","is_closed":false},{"id":2,"name":"In Progress","is_closed":false},
  {"id":5,"name":"Closed","is_closed":true}]}"#;

fn mock() -> Mock {
    Mock::start(vec![
        route(
            "GET",
            "/users/current.json",
            r#"{"user":{"id":4,"login":"ann","firstname":"Ann","lastname":"Lee"}}"#,
        ),
        route(
            "GET",
            "/projects/bench/memberships",
            r#"{"memberships":[{"id":1,"user":{"id":4,"name":"Ann Lee"}}],"total_count":1}"#,
        ),
        route(
            "GET",
            "/projects/bench/versions",
            r#"{"versions":[{"id":7,"name":"v0.1","status":"open","due_date":"2026-11-14"}]}"#,
        ),
        route(
            "GET",
            "/projects/bench.json",
            r#"{"project":{"id":1,"trackers":[{"id":1,"name":"Bug"},{"id":2,"name":"Feature"}]}}"#,
        ),
        route(
            "GET",
            "/projects.json",
            r#"{"projects":[{"id":1,"name":"Bench","identifier":"bench","is_public":false}],"total_count":1}"#,
        ),
        route("GET", "/issue_statuses.json", STATUSES),
        route("GET", "/issues.json", ISSUES),
        (
            "POST",
            "/issues.json".to_string(),
            201,
            r#"{"issue":{"id":13}}"#.to_string(),
        ),
        ("PUT", "/issues/12.json".to_string(), 204, String::new()),
    ])
}

fn cli(name: &str, mock: &Mock) -> Cli {
    Cli::new(
        name,
        &integration("rm", "redmine", &mock.url(), ""),
        &[("rm", "the-api-key")],
    )
}

fn link(cli: &Cli, repo: &Path) {
    cli.json(
        repo,
        &[
            "hosting",
            "link",
            "issues",
            "--id",
            "rm",
            "--project",
            "bench",
        ],
    );
}

#[test]
fn the_key_is_its_own_header_and_projects_are_listed_by_identifier() {
    let mock = mock();
    let cli = cli("rm-conn", &mock);
    let tested = cli.json(
        Path::new("."),
        &["hosting", "integrations", "test", "--id", "rm"],
    );
    assert_eq!(tested["user"]["login"], "ann");
    let seen = &mock.to("/users/current.json")[0];
    assert_eq!(seen.header("x-redmine-api-key"), Some("the-api-key"));
    assert_eq!(seen.header("authorization"), None);
    let projects = cli.json(Path::new("."), &["hosting", "projects", "--id", "rm"]);
    let rows = projects.as_array().unwrap();
    assert_eq!(rows.len(), 1, "{projects}");
    assert_eq!(rows[0]["full_name"], "bench");
    assert_eq!(rows[0]["web_url"], format!("{}/projects/bench", mock.url()));
}

#[test]
fn a_linked_project_lists_open_issues_assigned_to_me() {
    let mock = mock();
    let cli = cli("rm-issues", &mock);
    let repo = fixture();
    link(&cli, repo.root());

    let out = cli.json(repo.root(), &["hosting", "issues", "--assigned"]);
    let row = &out["issues"][0];
    assert_eq!(row["reference"], "#12");
    assert_eq!(row["number"], 12);
    assert_eq!(row["status"], "In Progress");
    assert_eq!(row["kind"], "Bug");
    assert_eq!(row["assignees"][0], "Ann Lee");
    assert_eq!(row["state"], "open");
    let list = mock
        .to("/issues.json")
        .into_iter()
        .find(|r| r.method == "GET")
        .unwrap();
    for part in [
        "project_id=bench",
        "status_id=open",
        "assigned_to_id=me",
        "limit=100",
    ] {
        assert!(list.target.contains(part), "{part} in {}", list.target);
    }

    let opened = cli.json(repo.root(), &["hosting", "open", "issue", "12"]);
    assert_eq!(opened["url"], format!("{}/issues/12", mock.url()));

    // Redmine has no pull requests.
    let err = cli.error(
        repo.root(),
        &[
            "hosting",
            "link",
            "pulls",
            "--id",
            "rm",
            "--project",
            "bench",
        ],
    );
    assert!(err.to_lowercase().contains("pull"), "{err}");
}

#[test]
fn issue_options_are_the_trackers_and_create_sends_ids() {
    let mock = mock();
    let cli = cli("rm-create", &mock);
    let repo = fixture();
    link(&cli, repo.root());

    let options = cli.json(repo.root(), &["hosting", "issue", "options"]);
    let types = options["types"].as_array().unwrap();
    assert_eq!(types.len(), 2, "{options}");
    assert_eq!(types[1]["label"], "Feature");

    let created = cli.json(
        repo.root(),
        &[
            "hosting",
            "issue",
            "create",
            "--title",
            "Broken",
            "--body",
            "Steps",
            "--type",
            "feature",
            "--assignee",
            "Ann Lee",
            "--milestone",
            "v0.1",
        ],
    );
    assert_eq!(created["reference"], "#13");
    assert_eq!(created["web_url"], format!("{}/issues/13", mock.url()));
    let post = mock
        .to("/issues.json")
        .into_iter()
        .find(|r| r.method == "POST")
        .unwrap();
    let body: serde_json::Value = serde_json::from_str(&post.body).unwrap();
    assert_eq!(body["issue"]["project_id"], "bench");
    assert_eq!(body["issue"]["subject"], "Broken");
    assert_eq!(body["issue"]["tracker_id"], 2);
    assert_eq!(body["issue"]["assigned_to_id"], 4);
    assert_eq!(body["issue"]["fixed_version_id"], 7);

    // No labels in core Redmine: refused before anything is posted.
    let err = cli.error(
        repo.root(),
        &[
            "hosting", "issue", "create", "--title", "x", "--label", "bug",
        ],
    );
    assert!(err.to_lowercase().contains("label"), "{err}");
}

#[test]
fn move_and_close_change_the_status() {
    let mock = mock();
    let cli = cli("rm-move", &mock);
    let repo = fixture();
    link(&cli, repo.root());

    let moved = cli.json(
        repo.root(),
        &["hosting", "issue", "move", "12", "--status", "in progress"],
    );
    assert_eq!(moved["number"], 12);
    let closed = cli.json(repo.root(), &["hosting", "issue", "close", "12"]);
    assert_eq!(closed["number"], 12);
    let bodies: Vec<serde_json::Value> = mock
        .to("/issues/12.json")
        .into_iter()
        .filter(|r| r.method == "PUT")
        .map(|r| serde_json::from_str(&r.body).unwrap())
        .collect();
    assert_eq!(bodies[0]["issue"]["status_id"], 2);
    assert_eq!(
        bodies[1]["issue"]["status_id"], 5,
        "the first closed status"
    );
}

#[test]
fn a_redmine_profile_needs_its_url() {
    let cli = Cli::new("rm-add", "[general]\n", &[]);
    let err = cli.error(
        Path::new("."),
        &["hosting", "integrations", "add", "--kind", "redmine"],
    );
    assert!(err.contains("--url"), "{err}");
    let added = cli.json(
        Path::new("."),
        &[
            "hosting",
            "integrations",
            "add",
            "--kind",
            "redmine",
            "--url",
            "https://redmine.example.com",
        ],
    );
    assert_eq!(added["added"]["kind"], "redmine");
    assert_eq!(added["added"]["category"], "issue_tracker");
    assert_eq!(added["added"]["api_base"], "https://redmine.example.com");
}
