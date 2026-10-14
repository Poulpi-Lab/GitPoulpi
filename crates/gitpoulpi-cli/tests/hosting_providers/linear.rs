//! Linear through the CLI: a loopback mock answers the one GraphQL endpoint;
//! routes are told apart by a word of the document (`path|needle`).

use std::path::Path;

use crate::common::*;

const ISSUES: &str = r#"{"data":{"issues":{"nodes":[
  {"id":"u1","identifier":"ENG-12","number":12,"title":"Crash on start","url":"https://linear.app/acme/issue/ENG-12/crash",
   "branchName":"kai/eng-12-crash-on-start","updatedAt":"2026-10-01T10:00:00.000Z","priorityLabel":"Urgent",
   "state":{"name":"In Progress","type":"started"},"creator":{"name":"Bob","displayName":"bob"},
   "assignee":{"name":"Ann","displayName":"ann"},"labels":{"nodes":[{"name":"bug"}]}}],
  "pageInfo":{"hasNextPage":false,"endCursor":null}}}}"#;

fn mock() -> Mock {
    let post = |needle: &str, json: &str| route("POST", &format!("/graphql|{needle}"), json);
    Mock::start(vec![
        post(
            "query Viewer",
            r#"{"data":{"viewer":{"id":"v1","name":"Ann Lee","displayName":"ann","email":"a@x.io","url":"https://linear.app/acme/profiles/ann"}}}"#,
        ),
        post(
            "query Teams",
            r#"{"data":{"organization":{"urlKey":"acme"},"teams":{"nodes":[
              {"id":"t1","key":"ENG","name":"Engineering"},{"id":"t2","key":"OPS","name":"Operations"}],
              "pageInfo":{"hasNextPage":false,"endCursor":null}}}}"#,
        ),
        post("query Issues", ISSUES),
        post(
            "query Team(",
            r#"{"data":{"teams":{"nodes":[{"id":"t1","key":"ENG","name":"Engineering"}]}}}"#,
        ),
        post(
            "query States",
            r#"{"data":{"team":{"defaultIssueState":{"id":"s2"},"states":{"nodes":[
              {"id":"s1","name":"Backlog","type":"backlog","position":0},
              {"id":"s2","name":"Todo","type":"unstarted","position":0},
              {"id":"s3","name":"Done","type":"completed","position":0}],
              "pageInfo":{"hasNextPage":false}}}}}"#,
        ),
        post(
            "mutation IssueCreate",
            r#"{"data":{"issueCreate":{"success":true,"issue":{"id":"u9","identifier":"ENG-13","number":13,"url":"https://linear.app/acme/issue/ENG-13/x"}}}}"#,
        ),
    ])
}

fn cli(name: &str, mock: &Mock) -> Cli {
    Cli::new(
        name,
        &integration("lin", "linear", &mock.url(), ""),
        &[("lin", "lin_api_secret")],
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
            "lin",
            "--project",
            "ENG",
        ],
    );
}

#[test]
fn the_key_goes_bare_in_the_authorization_header_and_teams_are_the_projects() {
    let mock = mock();
    let cli = cli("linear-conn", &mock);
    let tested = cli.json(
        Path::new("."),
        &["hosting", "integrations", "test", "--id", "lin"],
    );
    assert_eq!(tested["user"]["login"], "ann");
    let seen = mock.to("/graphql");
    assert_eq!(seen[0].header("authorization"), Some("lin_api_secret"));

    let projects = cli.json(
        Path::new("."),
        &["hosting", "projects", "--id", "lin", "--filter", "eng"],
    );
    let rows = projects.as_array().unwrap();
    assert_eq!(rows.len(), 1, "{projects}");
    assert_eq!(rows[0]["full_name"], "ENG");
    assert_eq!(
        rows[0]["web_url"],
        format!("{}/acme/team/ENG/all", mock.url())
    );
}

#[test]
fn a_linked_team_lists_issues_with_reference_status_and_branch() {
    let mock = mock();
    let cli = cli("linear-issues", &mock);
    let repo = fixture();
    link(&cli, repo.root());

    let out = cli.json(repo.root(), &["hosting", "issues", "--assigned"]);
    let row = &out["issues"][0];
    assert_eq!(row["reference"], "ENG-12");
    assert_eq!(row["number"], 12);
    assert_eq!(row["status"], "In Progress");
    assert_eq!(row["kind"], "Urgent");
    assert_eq!(row["assignees"][0], "ann");
    assert_eq!(row["branch_name"], "kai/eng-12-crash-on-start");
    assert_eq!(row["state"], "open");

    let issues = mock
        .to("/graphql")
        .into_iter()
        .find(|r| r.body.contains("query Issues"))
        .unwrap();
    let body: serde_json::Value = serde_json::from_str(&issues.body).unwrap();
    assert_eq!(body["variables"]["filter"]["team"]["key"]["eq"], "ENG");
    assert_eq!(body["variables"]["filter"]["assignee"]["isMe"]["eq"], true);
    assert_eq!(
        body["variables"]["filter"]["state"]["type"]["nin"],
        serde_json::json!(["completed", "canceled"])
    );

    // A reference opens an issue; a bare number is not Linear's spelling.
    let opened = cli.json(repo.root(), &["hosting", "open", "issue", "ENG-12"]);
    assert_eq!(opened["url"], format!("{}/issue/ENG-12", mock.url()));
    let err = cli.error(repo.root(), &["hosting", "open", "issue", "12"]);
    assert!(err.contains("ENG-12") || err.contains("key"), "{err}");

    // Linear has no pull requests.
    let err = cli.error(
        repo.root(),
        &[
            "hosting",
            "link",
            "pulls",
            "--id",
            "lin",
            "--project",
            "ENG",
        ],
    );
    assert!(err.to_lowercase().contains("pull"), "{err}");
}

#[test]
fn issue_options_are_the_open_states_and_create_sends_the_state_id() {
    let mock = mock();
    let cli = cli("linear-create", &mock);
    let repo = fixture();
    link(&cli, repo.root());

    let options = cli.json(repo.root(), &["hosting", "issue", "options"]);
    let lists = options["lists"].as_array().unwrap();
    let names: Vec<&str> = lists.iter().map(|l| l["label"].as_str().unwrap()).collect();
    assert_eq!(names, vec!["Backlog", "Todo"], "{options}");
    assert_eq!(options["default_list"], "s2");

    let created = cli.json(
        repo.root(),
        &[
            "hosting", "issue", "create", "--title", "Broken", "--body", "Steps", "--list", "s1",
        ],
    );
    assert_eq!(created["reference"], "ENG-13");
    let post = mock
        .to("/graphql")
        .into_iter()
        .find(|r| r.body.contains("mutation IssueCreate"))
        .unwrap();
    let body: serde_json::Value = serde_json::from_str(&post.body).unwrap();
    let input = &body["variables"]["input"];
    assert_eq!(input["teamId"], "t1");
    assert_eq!(input["title"], "Broken");
    assert_eq!(input["description"], "Steps");
    assert_eq!(input["stateId"], "s1");
}

#[test]
fn commit_messages_name_only_the_linked_team() {
    let cli = Cli::new(
        "refs-linear",
        &integration("lin", "linear", "https://linear.app/acme", ""),
        &[],
    );
    let repo = fixture();
    write_state(
        &repo,
        "[issues_source]\nintegration = \"lin\"\nproject = \"ENG\"\n",
    );
    let out = cli.json(
        repo.root(),
        &[
            "hosting",
            "refs",
            "--message",
            "Fixes ENG-12, not UTF-8 or OPS-3",
        ],
    );
    let refs = out["refs"].as_array().unwrap();
    assert_eq!(refs.len(), 1, "{out}");
    assert_eq!(refs[0]["reference"], "ENG-12");
    assert_eq!(refs[0]["url"], "https://linear.app/acme/issue/ENG-12");
}

#[test]
fn a_bad_key_is_reported_as_a_rejected_token() {
    let mock = Mock::start(vec![(
        "POST",
        "/graphql".to_string(),
        401,
        r#"{"errors":[{"message":"Authentication required","extensions":{"type":"authentication error"}}]}"#
            .to_string(),
    )]);
    let cli = cli("linear-401", &mock);
    let err = cli.error(
        Path::new("."),
        &["hosting", "integrations", "test", "--id", "lin"],
    );
    assert!(err.contains("rejected the token"), "{err}");
}

#[test]
fn linear_needs_no_url() {
    let cli = Cli::new("linear-add", "[general]\n", &[]);
    let added = cli.json(
        Path::new("."),
        &["hosting", "integrations", "add", "--kind", "linear"],
    );
    assert_eq!(added["added"]["url"], "https://linear.app");
    assert_eq!(added["added"]["api_base"], "https://api.linear.app");
    assert_eq!(added["added"]["category"], "issue_tracker");
}
