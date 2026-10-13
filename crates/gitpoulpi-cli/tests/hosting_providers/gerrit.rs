//! Gerrit through the CLI (a loopback host standing for the server's `/a/`
//! REST API): HTTP basic with the user name in `account`, the `)]}'` guard
//! line, changes as pull requests, patch sets, votes and the review push.
//! Assertions are on deserialized JSON.

use std::path::Path;

use crate::common::*;

/// A Gerrit answer: the guard line, then the JSON.
fn g(json: &str) -> String {
    format!(")]}}'\n{json}")
}

const CHANGE: &str = r#"{
  "_number": 7, "project": "sandbox", "branch": "main", "subject": "Add moons",
  "status": "NEW", "created": "2026-10-15 06:29:50.000000000",
  "updated": "2026-10-15 07:00:00.000000000",
  "owner": {"_account_id": 5, "username": "olga"},
  "submittable": false, "insertions": 3, "deletions": 1, "current_revision": "bbb",
  "revisions": {
    "aaa": {"_number": 1, "created": "2026-10-15 06:29:50.000000000",
            "commit": {"message": "Add moons\n\nOne.\n", "parents": [{"commit": "base"}]}},
    "bbb": {"_number": 2, "created": "2026-10-15 07:00:00.000000000",
            "commit": {"message": "Add moons\n\nTwo.\n", "parents": [{"commit": "base"}]}}
  },
  "labels": {"Code-Review": {"values": {"-2": "Do not submit", "-1": "Prefer not", " 0": "No score",
                                         "+1": "Looks good", "+2": "Approved"}, "all": [], "default_value": 0}},
  "permitted_labels": {"Code-Review": ["-2", "-1", " 0", "+1"]},
  "reviewers": {}, "messages": []
}"#;

fn mock() -> Mock {
    Mock::start(vec![
        route(
            "GET",
            "/a/accounts/self",
            &g(r#"{"_account_id":1000000,"name":"Admin","username":"admin"}"#),
        ),
        route(
            "GET",
            "/a/projects/",
            &g(r#"{"sandbox":{"id":"sandbox","description":"d","state":"ACTIVE"}}"#),
        ),
        route(
            "GET",
            "/a/changes/sandbox~7/revisions/1/files/",
            &g(r#"{"/COMMIT_MSG":{"status":"A"},"src/main.rs":{"lines_inserted":3}}"#),
        ),
        route(
            "GET",
            "/a/changes/sandbox~7/revisions/2/files/",
            &g(
                r#"{"/COMMIT_MSG":{"status":"A"},"src/main.rs":{"lines_inserted":3},"docs/a.md":{"status":"A","lines_inserted":1}}"#,
            ),
        ),
        route(
            "GET",
            "/a/changes/sandbox~7/comments",
            &g(
                r#"{"src/main.rs":[{"id":"c1","patch_set":2,"line":2,"message":"why?","updated":"2026-10-15 07:10:00.000000000","author":{"_account_id":9,"username":"bob"},"unresolved":true}]}"#,
            ),
        ),
        route("GET", "/a/changes/sandbox~7/drafts", &g("{}")),
        route("POST", "/a/changes/sandbox~7/revisions/", &g("{}")),
        route("POST", "/a/changes/sandbox~7/submit", &g(CHANGE)),
        route("POST", "/a/changes/sandbox~7/abandon", &g(CHANGE)),
        route("GET", "/a/changes/sandbox~7", &g(CHANGE)),
        route("GET", "/a/changes/", &g(&format!("[{CHANGE}]"))),
    ])
}

fn cli(name: &str, mock: &Mock) -> Cli {
    Cli::new(
        name,
        &integration("gr", "gerrit", &mock.url(), "account = \"admin\""),
        &[("gr", "http-pass")],
    )
}

fn link(cli: &Cli, repo: &Path) {
    cli.json(
        repo,
        &[
            "hosting",
            "link",
            "pulls",
            "--id",
            "gr",
            "--project",
            "sandbox",
        ],
    );
}

#[test]
fn http_basic_carries_the_user_name_and_the_guard_line_is_ignored() {
    let mock = mock();
    let cli = cli("gr-conn", &mock);
    let tested = cli.json(
        Path::new("."),
        &["hosting", "integrations", "test", "--id", "gr"],
    );
    assert_eq!(tested["user"]["login"], "admin");
    // base64("admin:http-pass")
    assert_eq!(
        mock.to("/a/accounts/self")[0].header("authorization"),
        Some("Basic YWRtaW46aHR0cC1wYXNz")
    );
    let projects = cli.json(Path::new("."), &["hosting", "projects", "--id", "gr"]);
    assert_eq!(projects[0]["full_name"], "sandbox");
}

#[test]
fn changes_are_listed_as_pull_requests_with_their_change_ref() {
    let mock = mock();
    let cli = cli("gr-list", &mock);
    let repo = fixture();
    link(&cli, repo.root());
    let out = cli.json(repo.root(), &["hosting", "prs"]);
    let row = &out["pull_requests"][0];
    assert_eq!(row["number"], 7);
    assert_eq!(row["target_branch"], "main");
    assert_eq!(row["source_branch"], "refs/changes/07/7/2");
    assert_eq!(row["author"], "olga");
    let query = decoded(&mock.to("/a/changes/")[0].target);
    assert!(
        query.contains("project:sandbox status:open"),
        "open changes of the project: {query}"
    );
}

#[test]
fn review_show_names_the_patch_sets_and_the_votes_the_server_permits() {
    let mock = mock();
    let cli = cli("gr-show", &mock);
    let repo = fixture();
    link(&cli, repo.root());
    let out = cli.json(repo.root(), &["hosting", "review", "show", "7"]);
    let detail = &out["detail"];
    assert_eq!(detail["shown_patch_set"], 2);
    assert_eq!(detail["patch_sets"].as_array().unwrap().len(), 2);
    assert_eq!(detail["head_sha"], "bbb");
    let values: Vec<i64> = detail["vote_labels"][0]["values"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["value"].as_i64().unwrap())
        .collect();
    assert_eq!(values, [-2, -1, 0, 1], "+2 is not permitted");
    let files: Vec<_> = out["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["path"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(files, ["docs/a.md", "src/main.rs"]);

    let first = cli.json(
        repo.root(),
        &["hosting", "review", "show", "7", "--patch-set", "1"],
    );
    assert_eq!(first["detail"]["head_sha"], "aaa");
    assert_eq!(first["files"].as_array().unwrap().len(), 1);

    let threads = cli.json(
        repo.root(),
        &["hosting", "review", "threads", "7", "--patch-set", "1"],
    );
    assert_eq!(
        threads["threads"][0]["outdated"], true,
        "written on patch set 2"
    );
}

#[test]
fn a_vote_is_checked_against_the_permitted_values_and_sent_on_the_patch_set() {
    let mock = mock();
    let cli = cli("gr-vote", &mock);
    let repo = fixture();
    link(&cli, repo.root());

    let err = cli.error(
        repo.root(),
        &[
            "hosting",
            "review",
            "submit",
            "7",
            "--event",
            "comment",
            "--vote",
            "Code-Review=+2",
        ],
    );
    assert!(err.contains("does not permit +2"), "{err}");
    assert!(
        err.contains("-2 -1 +0 +1") || err.contains("permitted"),
        "{err}"
    );
    let err = cli.error(
        repo.root(),
        &[
            "hosting",
            "review",
            "submit",
            "7",
            "--event",
            "comment",
            "--vote",
            "Verified=1",
        ],
    );
    assert!(err.contains("cannot vote on `Verified`"), "{err}");
    assert!(
        mock.requests().iter().all(|r| r.method == "GET"),
        "nothing was posted for a refused vote"
    );

    let out = cli.json(
        repo.root(),
        &[
            "hosting",
            "review",
            "submit",
            "7",
            "--event",
            "comment",
            "--body",
            "Fine",
            "--vote",
            "code-review=+1",
            "--patch-set",
            "1",
        ],
    );
    assert_eq!(out["votes"], 1);
    assert_eq!(out["patch_set"], 1);
    let post = mock
        .requests()
        .into_iter()
        .find(|r| r.method == "POST")
        .unwrap();
    assert!(
        post.target
            .ends_with("/a/changes/sandbox~7/revisions/aaa/review"),
        "{}",
        post.target
    );
    let body: serde_json::Value = serde_json::from_str(&post.body).unwrap();
    assert_eq!(body["labels"]["Code-Review"], 1, "the label's own spelling");
    assert_eq!(body["message"], "Fine");
}

#[test]
fn review_push_sends_head_to_the_review_ref_and_says_it_cannot_be_undone() {
    let mock = mock();
    let cli = cli("gr-push", &mock);
    let repo = fixture();
    link(&cli, repo.root());
    let out = cli.json(
        repo.root(),
        &[
            "hosting", "review", "push", "--branch", "main", "--topic", "moons",
        ],
    );
    assert_eq!(out["pushed_refs"][0], "refs/for/main%topic=moons");
    assert_eq!(out["undoable"], false);
    assert_eq!(
        repo.git_in_remote(&["rev-parse", "refs/for/main%topic=moons"]),
        repo.git(&["rev-parse", "HEAD"])
    );
    let err = cli.error(
        repo.root(),
        &["hosting", "review", "push", "--branch", "ma in"],
    );
    assert!(err.contains("not a usable branch"), "{err}");
}

#[test]
fn a_gerrit_profile_needs_its_url_and_user_name() {
    let cli = Cli::new("gr-add", "[general]\n", &[]);
    let err = cli.error(
        Path::new("."),
        &["hosting", "integrations", "add", "--kind", "gerrit"],
    );
    assert!(err.contains("--account"), "{err}");
    let err = cli.error(
        Path::new("."),
        &[
            "hosting",
            "integrations",
            "add",
            "--kind",
            "gerrit",
            "--account",
            "admin",
        ],
    );
    assert!(err.contains("--url"), "{err}");
    let added = cli.json(
        Path::new("."),
        &[
            "hosting",
            "integrations",
            "add",
            "--kind",
            "gerrit",
            "--url",
            "https://g.example/",
            "--account",
            "admin",
        ],
    );
    assert_eq!(added["added"]["kind"], "gerrit");
    assert_eq!(added["added"]["category"], "hosting");
    assert_eq!(added["added"]["account"], "admin");
    assert_eq!(added["added"]["api_base"], "https://g.example/a");
}

#[test]
fn submit_change_and_abandon_post_to_the_change_and_say_they_cannot_be_undone() {
    let mock = mock();
    let cli = cli("gr-act", &mock);
    let repo = fixture();
    link(&cli, repo.root());
    let out = cli.json(repo.root(), &["hosting", "review", "submit-change", "7"]);
    assert_eq!(out["submitted"], true);
    assert_eq!(out["undoable"], false);
    let out = cli.json(
        repo.root(),
        &[
            "hosting",
            "review",
            "abandon",
            "7",
            "--message",
            "superseded by 9",
        ],
    );
    assert_eq!(out["abandoned"], true);
    assert_eq!(out["undoable"], false);
    let posts: Vec<_> = mock
        .requests()
        .into_iter()
        .filter(|r| r.method == "POST")
        .collect();
    assert_eq!(posts.len(), 2);
    assert!(posts[0].target.ends_with("/a/changes/sandbox~7/submit"));
    assert!(posts[1].target.ends_with("/a/changes/sandbox~7/abandon"));
    let body: serde_json::Value = serde_json::from_str(&posts[1].body).unwrap();
    assert_eq!(body["message"], "superseded by 9");
}

#[test]
fn show_can_compare_the_files_with_an_older_patch_set() {
    let mock = mock();
    let cli = cli("gr-compare", &mock);
    let repo = fixture();
    link(&cli, repo.root());
    let out = cli.json(
        repo.root(),
        &[
            "hosting",
            "review",
            "show",
            "7",
            "--patch-set",
            "2",
            "--compare-with",
            "1",
        ],
    );
    assert_eq!(out["compare_with"], 1);
    assert!(
        mock.requests()
            .iter()
            .any(|r| r.target.ends_with("/revisions/2/files/?base=1")),
        "{:?}",
        mock.requests()
            .iter()
            .map(|r| &r.target)
            .collect::<Vec<_>>()
    );
}
