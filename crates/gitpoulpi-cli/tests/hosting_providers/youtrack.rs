//! YouTrack through the CLI: a loopback mock answers the REST routes.

use std::path::Path;

use crate::common::*;

const ISSUES: &str = r#"[
  {"id":"2-12","idReadable":"ABC-12","summary":"Crash on start","resolved":null,"updated":1760000000000,
   "reporter":{"login":"bob","fullName":"Bob"},"tags":[{"name":"ui"}],
   "customFields":[{"name":"State","value":{"name":"In Progress"}},{"name":"Type","value":{"name":"Bug"}},
                   {"name":"Assignee","value":{"login":"ann","fullName":"Ann"}}]}]"#;

const FIELDS: &str = r#"[{"field":{"name":"State"},"bundle":{"values":[
  {"name":"Submitted","isResolved":false},{"name":"Done","isResolved":true}]}}]"#;

fn mock() -> Mock {
    Mock::start(vec![
        route(
            "GET",
            "/api/users/me",
            r#"{"id":"1-1","login":"ann","fullName":"Ann Lee"}"#,
        ),
        route("GET", "/api/admin/projects/0-1/customFields", FIELDS),
        route(
            "GET",
            "/api/admin/projects",
            r#"[{"id":"0-1","shortName":"ABC","name":"Alphabet"},{"id":"0-2","shortName":"XYZ","name":"Xray"}]"#,
        ),
        route("GET", "/api/issues", ISSUES),
        (
            "POST",
            "/api/issues".to_string(),
            200,
            r#"{"id":"2-13","idReadable":"ABC-13"}"#.to_string(),
        ),
    ])
}

fn cli(name: &str, mock: &Mock) -> Cli {
    Cli::new(
        name,
        &integration("yt", "youtrack", &mock.url(), ""),
        &[("yt", "perm:secret")],
    )
}

fn link(cli: &Cli, repo: &Path, extra: &[&str]) {
    let mut args = vec![
        "hosting",
        "link",
        "issues",
        "--id",
        "yt",
        "--project",
        "ABC",
    ];
    args.extend_from_slice(extra);
    cli.json(repo, &args);
}

#[test]
fn the_token_is_a_bearer_and_projects_are_listed_by_short_name() {
    let mock = mock();
    let cli = cli("yt-conn", &mock);
    let tested = cli.json(
        Path::new("."),
        &["hosting", "integrations", "test", "--id", "yt"],
    );
    assert_eq!(tested["user"]["login"], "ann");
    assert_eq!(
        mock.to("/api/users/me")[0].header("authorization"),
        Some("Bearer perm:secret")
    );
    let projects = cli.json(
        Path::new("."),
        &["hosting", "projects", "--id", "yt", "--filter", "alph"],
    );
    let rows = projects.as_array().unwrap();
    assert_eq!(rows.len(), 1, "{projects}");
    assert_eq!(rows[0]["full_name"], "ABC");
    assert_eq!(rows[0]["web_url"], format!("{}/projects/ABC", mock.url()));
}

#[test]
fn a_linked_project_with_a_query_filter_lists_issues() {
    let mock = mock();
    let cli = cli("yt-issues", &mock);
    let repo = fixture();
    link(&cli, repo.root(), &["--filter", "Type: Bug"]);
    assert!(
        read_state(&repo).contains("filter = \"Type: Bug\""),
        "{}",
        read_state(&repo)
    );

    let out = cli.json(repo.root(), &["hosting", "issues", "--assigned"]);
    let row = &out["issues"][0];
    assert_eq!(row["reference"], "ABC-12");
    assert_eq!(row["number"], 12);
    assert_eq!(row["status"], "In Progress");
    assert_eq!(row["kind"], "Bug");
    assert_eq!(row["assignees"][0], "ann");
    assert_eq!(row["labels"][0], "ui");
    assert_eq!(row["state"], "open");

    let list = mock
        .to("/api/issues")
        .into_iter()
        .find(|r| r.method == "GET")
        .unwrap();
    let query = decoded(list.target.split_once("&query=").unwrap().1);
    assert!(
        query.starts_with("project: {ABC} #Unresolved for: me and (Type: Bug) sort by:"),
        "{query}"
    );

    let opened = cli.json(repo.root(), &["hosting", "open", "issue", "ABC-12"]);
    assert_eq!(opened["url"], format!("{}/issue/ABC-12", mock.url()));

    // YouTrack has no pull requests.
    let err = cli.error(
        repo.root(),
        &["hosting", "link", "pulls", "--id", "yt", "--project", "ABC"],
    );
    assert!(err.to_lowercase().contains("pull"), "{err}");
}

#[test]
fn issue_options_are_the_open_states_and_create_sends_the_project_id() {
    let mock = mock();
    let cli = cli("yt-create", &mock);
    let repo = fixture();
    link(&cli, repo.root(), &[]);

    let options = cli.json(repo.root(), &["hosting", "issue", "options"]);
    let lists = options["lists"].as_array().unwrap();
    assert_eq!(lists.len(), 1, "resolved states are not offered: {options}");
    assert_eq!(lists[0]["label"], "Submitted");

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
            "--list",
            "submitted",
        ],
    );
    assert_eq!(created["reference"], "ABC-13");
    assert_eq!(created["web_url"], format!("{}/issue/ABC-13", mock.url()));
    let post = mock
        .to("/api/issues")
        .into_iter()
        .find(|r| r.method == "POST")
        .unwrap();
    let body: serde_json::Value = serde_json::from_str(&post.body).unwrap();
    assert_eq!(body["project"]["id"], "0-1");
    assert_eq!(body["summary"], "Broken");
    assert_eq!(body["customFields"][0]["value"]["name"], "Submitted");
}

#[test]
fn a_youtrack_profile_needs_its_url() {
    let cli = Cli::new("yt-add", "[general]\n", &[]);
    let err = cli.error(
        Path::new("."),
        &["hosting", "integrations", "add", "--kind", "youtrack"],
    );
    assert!(err.contains("--url"), "{err}");
    let added = cli.json(
        Path::new("."),
        &[
            "hosting",
            "integrations",
            "add",
            "--kind",
            "youtrack",
            "--url",
            "https://acme.youtrack.cloud",
        ],
    );
    assert_eq!(added["added"]["kind"], "youtrack");
    assert_eq!(added["added"]["category"], "issue_tracker");
    assert_eq!(
        added["added"]["api_base"],
        "https://acme.youtrack.cloud/api"
    );
}
