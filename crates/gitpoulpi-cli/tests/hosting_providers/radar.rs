//! `hosting radar` and `hosting integrations radar` through the CLI against a
//! loopback self-managed GitLab: the account-wide lists, the five-minute
//! cache file, the per-profile settings and the failure modes. Assertions are
//! on deserialized JSON; nothing reaches the internet.

use std::path::Path;

use crate::common::*;

const MR: &str = r#"{"iid":5,"title":"Add parser","state":"opened","draft":false,
    "source_branch":"feat","target_branch":"main","source_project_id":1,"target_project_id":1,
    "author":{"username":"ann"},"web_url":"https://lab.example/team/tool/-/merge_requests/5",
    "updated_at":"2026-10-02T10:00:00Z","assignees":[],"reviewers":[{"username":"me"}],
    "references":{"full":"team/tool!5"}}"#;

const ISSUE: &str = r#"{"iid":9,"title":"Crash","web_url":"https://lab.example/team/tool/-/issues/9",
    "updated_at":"2026-10-03T10:00:00Z","author":{"username":"ann"},"assignees":[{"username":"me"}],
    "labels":["bug"],"milestone":null,"references":{"full":"team/tool#9"}}"#;

fn mock() -> Mock {
    Mock::start(vec![
        route(
            "GET",
            "/api/v4/user",
            r#"{"id":1,"username":"me","name":"Me"}"#,
        ),
        route("GET", "/api/v4/merge_requests", &format!("[{MR}]")),
        route("GET", "/api/v4/issues", &format!("[{ISSUE}]")),
    ])
}

fn cli(name: &str, mock: &Mock, extra: &str) -> Cli {
    Cli::new(
        name,
        &integration("lab", "gitlab_self_managed", &mock.url(), extra),
        &[("lab", "tok")],
    )
}

fn here() -> &'static Path {
    Path::new(".")
}

#[test]
fn review_requests_come_from_the_host_then_from_the_cache() {
    let mock = mock();
    let cli = cli("radar-cache", &mock, "");
    let args = ["hosting", "radar", "--id", "lab", "--what", "review"];

    let first = cli.json(here(), &args);
    let profile = &first["profiles"][0];
    assert_eq!(profile["id"], "lab");
    assert_eq!(profile["cached"], false);
    assert_eq!(profile["prs"][0]["project"], "team/tool");
    assert_eq!(profile["prs"][0]["number"], 5);
    assert_eq!(profile["prs"][0]["review_requested"], true);
    assert_eq!(
        profile["prs"][0]["clone_url"],
        format!("{}/team/tool.git", mock.url())
    );
    assert_eq!(profile["issues"].as_array().unwrap().len(), 0);
    assert_eq!(profile["truncated"], false);
    assert!(profile["fetched_at"].as_u64().unwrap() > 0);
    assert!(cli.config.path().join("radar-account.json").exists());
    let asked = mock
        .requests()
        .into_iter()
        .find(|r| r.path().ends_with("/merge_requests"))
        .unwrap();
    assert!(
        decoded(&asked.target).contains("scope=all&state=opened"),
        "{}",
        asked.target
    );
    assert!(
        asked.target.contains("reviewer_username=me"),
        "{}",
        asked.target
    );

    // Within five minutes the file answers.
    let second = cli.json(here(), &args);
    assert_eq!(second["profiles"][0]["cached"], true);
    assert_eq!(second["profiles"][0]["fetched_at"], profile["fetched_at"]);
    assert_eq!(mock.to("/merge_requests").len(), 1);

    // `--refresh` asks again.
    let third = cli.json(
        here(),
        &[
            "hosting",
            "radar",
            "--id",
            "lab",
            "--what",
            "review",
            "--refresh",
        ],
    );
    assert_eq!(third["profiles"][0]["cached"], false);
    assert_eq!(mock.to("/merge_requests").len(), 2);
}

#[test]
fn the_profile_settings_decide_what_is_listed_without_what() {
    let mock = mock();
    let cli = cli("radar-settings", &mock, "");

    // Nothing turned on: no profile is asked.
    let none = cli.json(here(), &["hosting", "radar"]);
    assert_eq!(none["profiles"].as_array().unwrap().len(), 0);
    assert!(mock.requests().is_empty());

    let set = cli.json(
        here(),
        &[
            "hosting",
            "integrations",
            "radar",
            "lab",
            "--assigned-issues",
            "on",
            "--namespace",
            "acme, tools",
            "--review-mode",
            "api",
        ],
    );
    assert_eq!(set["changed"], true);
    assert_eq!(set["radar"]["assigned_issues"], true);
    assert_eq!(set["radar"]["review_requests"], false);
    assert_eq!(
        set["radar"]["namespaces"],
        serde_json::json!(["acme", "tools"])
    );
    assert_eq!(set["radar"]["review_mode"], "api");
    let text = cli.config_text();
    assert!(text.contains("[integrations.radar]"), "{text}");
    assert!(text.contains("assigned_issues = true"), "{text}");

    // Reading changes nothing and reports it.
    let shown = cli.json(here(), &["hosting", "integrations", "radar", "lab"]);
    assert_eq!(shown["changed"], false);
    assert_eq!(shown["radar"]["review_mode"], "api");

    let listed = cli.json(here(), &["hosting", "radar"]);
    let profile = &listed["profiles"][0];
    assert_eq!(profile["issues"][0]["project"], "team/tool");
    assert_eq!(profile["issues"][0]["number"], 9);
    assert_eq!(profile["prs"].as_array().unwrap().len(), 0);
    assert!(
        mock.to("/merge_requests").is_empty(),
        "only issues were asked for"
    );

    // Back to the defaults leaves no table behind.
    cli.json(
        here(),
        &[
            "hosting",
            "integrations",
            "radar",
            "lab",
            "--assigned-issues",
            "off",
            "--clear-namespaces",
            "--review-mode",
            "ask",
        ],
    );
    assert!(
        !cli.config_text().contains("radar"),
        "{}",
        cli.config_text()
    );
    let rows = cli.json(here(), &["hosting", "integrations", "list"]);
    assert!(rows["integrations"][0]["radar"].is_null());
}

#[test]
fn a_rejected_token_fails_a_single_profile_but_not_the_other_one() {
    let mock = Mock::start(vec![
        route("GET", "/api/v4/user", r#"{"id":1,"username":"me"}"#),
        (
            "GET",
            "/api/v4/merge_requests".to_string(),
            401,
            r#"{"message":"401 Unauthorized"}"#.to_string(),
        ),
    ]);
    let good = Mock::start(vec![
        route("GET", "/api/v4/user", r#"{"id":1,"username":"me"}"#),
        route("GET", "/api/v4/merge_requests", &format!("[{MR}]")),
    ]);
    let config = format!(
        "{}{}",
        integration("bad", "gitlab_self_managed", &mock.url(), ""),
        integration("good", "gitlab_self_managed", &good.url(), ""),
    );
    let cli = Cli::new("radar-401", &config, &[("bad", "tok"), ("good", "tok")]);
    let err = cli.error(
        here(),
        &["hosting", "radar", "--id", "bad", "--what", "review"],
    );
    assert!(err.contains("rejected the token"), "{err}");
    assert!(
        !cli.config.path().join("radar-account.json").exists(),
        "a failed round caches nothing"
    );

    let both = cli.json(here(), &["hosting", "radar", "--what", "review"]);
    let profiles = both["profiles"].as_array().unwrap();
    assert_eq!(profiles.len(), 2);
    let bad = profiles.iter().find(|p| p["id"] == "bad").unwrap();
    let ok = profiles.iter().find(|p| p["id"] == "good").unwrap();
    assert!(bad["error"]
        .as_str()
        .unwrap()
        .contains("rejected the token"));
    assert!(bad["prs"].as_array().unwrap().is_empty());
    assert!(ok["error"].is_null());
    assert_eq!(ok["prs"][0]["project"], "team/tool");
}

#[test]
fn a_profile_without_account_lists_is_refused_by_name() {
    let mock = mock();
    let config = format!(
        "{}{}",
        integration("lab", "gitlab_self_managed", &mock.url(), ""),
        integration(
            "board",
            "trello",
            "https://trello.com",
            "account = \"key\"\n"
        ),
    );
    let cli = Cli::new("radar-kinds", &config, &[("lab", "tok")]);
    let err = cli.error(
        here(),
        &["hosting", "radar", "--id", "board", "--what", "all"],
    );
    assert!(err.contains("no account-wide lists"), "{err}");
    let err = cli.error(
        here(),
        &[
            "hosting",
            "integrations",
            "radar",
            "board",
            "--review-requests",
            "on",
        ],
    );
    assert!(err.contains("issue tracker"), "{err}");
    let err = cli.error(here(), &["hosting", "radar", "--id", "nope"]);
    assert!(err.contains("no integration `nope`"), "{err}");
    // A profile with every list off and no --what is told how to turn one on.
    let err = cli.error(here(), &["hosting", "radar", "--id", "lab"]);
    assert!(err.contains("no Radar list turned on"), "{err}");
}

#[test]
fn gerrit_lists_its_review_requests_account_wide() {
    let change = r#"{"_number":7,"project":"tools/cli","branch":"main","subject":"Add flag",
        "status":"NEW","work_in_progress":false,"updated":"2026-10-02 10:00:00.000000000",
        "owner":{"_account_id":2,"username":"ann"}}"#;
    let mock = Mock::start(vec![route(
        "GET",
        "/a/changes/",
        &format!(")]}}'\n[{change}]"),
    )]);
    let config = integration("gr", "gerrit", &mock.url(), "account = \"mara\"\n");
    let cli = Cli::new("radar-gerrit", &config, &[("gr", "tok")]);
    let out = cli.json(
        here(),
        &["hosting", "radar", "--id", "gr", "--what", "review"],
    );
    let profile = &out["profiles"][0];
    assert_eq!(profile["prs"][0]["project"], "tools/cli");
    assert_eq!(profile["prs"][0]["number"], 7);
    assert_eq!(profile["prs"][0]["review_requested"], true);
    let asked = mock
        .requests()
        .into_iter()
        .find(|r| r.path().ends_with("/changes/"))
        .unwrap();
    assert!(
        decoded(&asked.target).contains("q=is:open reviewer:self -owner:self -is:ignored -is:wip"),
        "{}",
        asked.target
    );
}

#[test]
fn bitbucket_cloud_is_refused_with_the_reason() {
    let config = integration(
        "bb",
        "bitbucket",
        "https://bitbucket.org",
        "account = \"me@example.com\"\n",
    );
    let cli = Cli::new("radar-bb-cloud", &config, &[("bb", "tok")]);
    let err = cli.error(here(), &["hosting", "radar", "--id", "bb", "--what", "all"]);
    assert!(err.contains("Bitbucket Cloud has no account-wide"), "{err}");
    assert!(err.contains("Bitbucket Data Center"), "{err}");
}
