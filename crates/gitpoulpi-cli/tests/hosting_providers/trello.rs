//! Trello through the CLI: boards are the projects, lists the choices.

use crate::common::*;

fn mock() -> Mock {
    Mock::start(vec![
        route("GET", "/1/members/me/boards", r#"[{"id":"b1","name":"Roadmap","shortLink":"AbCd1234","url":"https://trello.com/b/AbCd1234/roadmap","prefs":{"permissionLevel":"private"}}]"#),
        route("GET", "/1/members/me", r#"{"id":"m1","username":"me","fullName":"Me Myself"}"#),
        route("GET", "/lists", r#"[{"id":"l1","name":"To do"},{"id":"l2","name":"Doing"}]"#),
        route("GET", "/cards/open", r#"[{"id":"c1","name":"Ship it","idShort":7,"shortUrl":"https://trello.com/c/Zz9Yy8Xx","url":"https://trello.com/c/Zz9Yy8Xx/7-ship-it","closed":false,"idList":"l2","idMembers":[],"labels":[],"dateLastActivity":"2026-10-01T10:00:00.000Z"}]"#),
        ("POST", "/1/cards".to_string(), 200, r#"{"id":"c2","idShort":8,"shortUrl":"https://trello.com/c/New12345","url":"https://trello.com/c/New12345/8-x"}"#.to_string()),
    ])
}

fn cli(name: &str, mock: &Mock) -> Cli {
    Cli::new(
        name,
        &integration("trello", "trello", &mock.url(), "account = \"KEY123\""),
        &[("trello", "tok")],
    )
}

#[test]
fn boards_are_projects_and_the_token_never_rides_in_the_url() {
    let mock = mock();
    let cli = cli("trello-boards", &mock);
    let projects = cli.json(
        std::path::Path::new("."),
        &["hosting", "projects", "--id", "trello"],
    );
    assert_eq!(projects[0]["full_name"], "AbCd1234");
    assert_eq!(projects[0]["name"], "Roadmap");
    for request in mock.requests() {
        assert!(!request.target.contains("token="), "{}", request.target);
        assert!(!request.target.contains("key="), "{}", request.target);
        let auth = request.header("authorization").unwrap_or_default();
        assert!(
            auth.contains("oauth_consumer_key=\"KEY123\"") && auth.contains("oauth_token=\"tok\""),
            "{auth}"
        );
    }
}

#[test]
fn cards_list_with_a_list_filter_and_a_card_is_created_in_a_named_list() {
    let mock = mock();
    let cli = cli("trello-cards", &mock);
    let repo = fixture();
    cli.json(
        repo.root(),
        &[
            "hosting",
            "link",
            "issues",
            "--id",
            "trello",
            "--project",
            "AbCd1234",
            "--filter",
            "Doing",
        ],
    );
    let out = cli.json(repo.root(), &["hosting", "issues"]);
    let row = &out["issues"][0];
    assert_eq!(row["reference"], "https://trello.com/c/Zz9Yy8Xx");
    assert_eq!(row["number"], 7);
    assert_eq!(row["kind"], "Doing");

    // A Trello short link opens its card.
    let opened = cli.json(repo.root(), &["hosting", "open", "issue", "Zz9Yy8Xx"]);
    assert_eq!(opened["url"], format!("{}/c/Zz9Yy8Xx", mock.url()));

    let options = cli.json(repo.root(), &["hosting", "issue", "options"]);
    assert_eq!(options["lists"].as_array().unwrap().len(), 2, "{options}");
    assert_eq!(options["types"].as_array().unwrap().len(), 0);

    let created = cli.json(
        repo.root(),
        &[
            "hosting", "issue", "create", "--title", "New card", "--list", "doing",
        ],
    );
    assert_eq!(created["reference"], "https://trello.com/c/New12345");
    let post = &mock.to("/1/cards")[0];
    let body: serde_json::Value = serde_json::from_str(&post.body).unwrap();
    assert_eq!(body["idList"], "l2");
    assert_eq!(body["name"], "New card");
}
