use std::sync::Arc;

use serde_json::{json, Value};

use crate::client::{client_for, HostingClient, NewPullRequest};
use crate::http::{HttpRequest, Method, RecordedTransport};
use crate::integration::{Integration, IntegrationKind};
use crate::model::{PrFilter, PullRequestState, StateFilter};
use crate::review::model::{
    DiffSide, MergeState, PendingComment, ReviewEvent, ReviewSubmission, ReviewVote,
};

use super::{change_ref, patch_set_of_ref};

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

fn client(t: Arc<RecordedTransport>) -> Box<dyn HostingClient> {
    let mut i = Integration::new("gr", IntegrationKind::Gerrit, "http://127.0.0.1:8081/");
    i.account = "admin".into();
    client_for(
        i,
        "http-pass".into(),
        t,
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap()
}

/// A Gerrit JSON answer: the guard line, then the JSON.
fn gerrit(json: impl std::fmt::Display) -> String {
    format!(")]}}'\n{json}")
}

fn body(r: &HttpRequest) -> Value {
    serde_json::from_str(r.body.as_deref().unwrap()).unwrap()
}

const ME: &str = r#"{"_account_id":1000000,"name":"Administrator","username":"admin"}"#;

fn change_json() -> Value {
    json!({
        "_number": 7, "project": "sandbox", "branch": "master", "subject": "Add a main",
        "status": "NEW", "created": "2026-10-15 06:29:50.000000000",
        "updated": "2026-10-15 07:00:00.000000000",
        "owner": { "_account_id": 5, "name": "Olga", "username": "olga" },
        "submittable": false, "insertions": 3, "deletions": 1,
        "current_revision": "bbb",
        "revisions": {
            "aaa": { "_number": 1, "created": "2026-10-15 06:29:50.000000000",
                     "commit": { "message": "Add a main\n\nFirst try.\n\nChange-Id: I1\n",
                                 "parents": [ { "commit": "base1" } ] } },
            "bbb": { "_number": 2, "created": "2026-10-15 07:00:00.000000000",
                     "commit": { "message": "Add a main\n\nSecond try.\nIt works now.\n\nChange-Id: I1\nSigned-off-by: Olga <o@x>\n",
                                 "parents": [ { "commit": "base2" } ] } }
        },
        "labels": {
            "Code-Review": {
                "values": { "-2": "Do not submit", "-1": "Prefer not", " 0": "No score",
                            "+1": "Looks good", "+2": "Approved" },
                "all": [ { "_account_id": 1000000, "value": 1, "username": "admin", "date": "2026-10-15 07:10:00.000000000" },
                         { "_account_id": 9, "value": -1, "username": "bob", "date": "2026-10-15 07:11:00.000000000" } ],
                "default_value": 0
            },
            "Verified": {
                "values": { "-1": "Fails", " 0": "No score", "+1": "Verified" },
                "all": [], "default_value": 0
            }
        },
        "permitted_labels": { "Code-Review": ["-2", "-1", " 0", "+1"], "Verified": ["-1", " 0", "+1"] },
        "reviewers": { "REVIEWER": [ { "_account_id": 1000000, "username": "admin" } ] },
        "messages": [
            { "id": "m1", "author": { "_account_id": 5, "username": "olga" },
              "date": "2026-10-15 06:30:00.000000000", "message": "Uploaded patch set 1." },
            { "id": "m2", "author": { "_account_id": 9, "username": "bob" },
              "date": "2026-10-15 07:11:00.000000000", "message": "Patch Set 2: Code-Review-1" }
        ]
    })
}

#[test]
fn the_guard_line_is_stripped_and_basic_auth_carries_the_user() {
    let t = Arc::new(RecordedTransport::new().on("/a/accounts/self", 200, &gerrit(ME)));
    let check = block_on(client(t.clone()).check_token()).unwrap();
    assert_eq!(check.user.login, "admin");
    assert_eq!(check.user.name, "Administrator");
    let req = &t.requests()[0];
    assert_eq!(req.url, "http://127.0.0.1:8081/a/accounts/self");
    let auth = req
        .headers
        .iter()
        .find(|(k, _)| k == "Authorization")
        .map(|(_, v)| v.as_str());
    // base64("admin:http-pass")
    assert_eq!(auth, Some("Basic YWRtaW46aHR0cC1wYXNz"));
}

#[test]
fn a_bad_password_is_unauthorized() {
    let t = Arc::new(RecordedTransport::new().on("/a/accounts/self", 401, "Unauthorized"));
    let err = block_on(client(t).check_token()).unwrap_err();
    assert!(matches!(err, crate::Error::Unauthorized { .. }), "{err:?}");
}

#[test]
fn change_refs_use_the_last_two_digits() {
    assert_eq!(change_ref(1, 1), "refs/changes/01/1/1");
    assert_eq!(change_ref(12345, 3), "refs/changes/45/12345/3");
    assert_eq!(change_ref(100, 2), "refs/changes/00/100/2");
    assert_eq!(patch_set_of_ref("refs/changes/45/12345/3"), Some(3));
    assert_eq!(patch_set_of_ref("feature"), None);
}

#[test]
fn projects_come_from_the_code_projects_map() {
    let t = Arc::new(RecordedTransport::new().on(
        "/a/projects/",
        200,
        &gerrit(
            r#"{"sandbox":{"id":"sandbox","description":"A sandbox","state":"ACTIVE"},
                "team/tool":{"id":"team%2Ftool","state":"ACTIVE"},
                "old":{"id":"old","state":"READ_ONLY"}}"#,
        ),
    ));
    let projects = block_on(client(t.clone()).list_projects()).unwrap();
    let names: Vec<_> = projects.iter().map(|p| p.full_name.as_str()).collect();
    assert_eq!(
        names,
        ["sandbox", "team/tool"],
        "read-only projects are hidden"
    );
    assert_eq!(
        projects[0].https_url.as_deref(),
        Some("http://127.0.0.1:8081/sandbox")
    );
    assert_eq!(projects[1].name, "tool");
    assert!(t.requests()[0].url.contains("type=CODE"));
}

#[test]
fn the_list_asks_for_open_changes_of_the_project() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/a/accounts/self", 200, &gerrit(ME))
            .on("/a/changes/", 200, &gerrit(json!([change_json()]))),
    );
    let rows = block_on(client(t.clone()).list_pull_requests(
        "sandbox",
        &PrFilter {
            state: StateFilter::Open,
            ..Default::default()
        },
    ))
    .unwrap();
    let url = t
        .requests()
        .iter()
        .find(|r| r.url.contains("/a/changes/"))
        .unwrap()
        .url
        .clone();
    assert!(url.contains("q=project%3Asandbox%20status%3Aopen"), "{url}");
    let row = &rows[0];
    assert_eq!(row.number, 7);
    assert_eq!(row.iid_label, "#7");
    assert_eq!(row.title, "Add a main");
    assert_eq!(row.state, PullRequestState::Open);
    assert_eq!(row.target_branch, "master");
    assert_eq!(
        row.source_branch, "refs/changes/07/7/2",
        "the newest patch set"
    );
    assert_eq!(row.author, "olga");
    assert_eq!(row.web_url, "http://127.0.0.1:8081/c/sandbox/+/7");
    assert_eq!(row.updated_at, "2026-10-15T07:00:00Z");
    assert!(row.review_requested, "the user is a REVIEWER");
    assert_eq!(row.reviewers, ["admin"]);
    assert_eq!(row.checks, Some(crate::model::ChecksState::Pending));
}

#[test]
fn filters_become_gerrit_query_operators() {
    for (filter, want) in [
        (
            PrFilter {
                state: StateFilter::Closed,
                ..Default::default()
            },
            "status%3Amerged%20OR%20status%3Aabandoned",
        ),
        (
            PrFilter {
                state: StateFilter::Open,
                mine: true,
                ..Default::default()
            },
            "owner%3Aself",
        ),
        (
            PrFilter {
                state: StateFilter::Open,
                review_requested: true,
                ..Default::default()
            },
            "reviewer%3Aself",
        ),
    ] {
        let t = Arc::new(
            RecordedTransport::new()
                .on("/a/accounts/self", 200, &gerrit(ME))
                .on("/a/changes/", 200, &gerrit("[]")),
        );
        block_on(client(t.clone()).list_pull_requests("sandbox", &filter)).unwrap();
        let url = t
            .requests()
            .iter()
            .find(|r| r.url.contains("/a/changes/"))
            .unwrap()
            .url
            .clone();
        assert!(url.contains(want), "{want} in {url}");
    }
}

#[test]
fn work_in_progress_is_a_draft_and_merged_and_abandoned_are_final() {
    let mut wip = change_json();
    wip["work_in_progress"] = json!(true);
    let mut merged = change_json();
    merged["status"] = json!("MERGED");
    let mut dropped = change_json();
    dropped["status"] = json!("ABANDONED");
    let t = Arc::new(
        RecordedTransport::new()
            .on("/a/accounts/self", 200, &gerrit(ME))
            .on("/a/changes/", 200, &gerrit(json!([wip, merged, dropped]))),
    );
    let rows = block_on(client(t).list_pull_requests("sandbox", &PrFilter::default())).unwrap();
    let states: Vec<_> = rows.iter().map(|r| r.state).collect();
    assert_eq!(
        states,
        [
            PullRequestState::Draft,
            PullRequestState::Merged,
            PullRequestState::Closed
        ]
    );
}

#[test]
fn the_detail_names_the_patch_sets_and_the_votes_the_server_permits() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/a/accounts/self", 200, &gerrit(ME))
            .on("/a/changes/sandbox~7?", 200, &gerrit(change_json())),
    );
    let c = client(t.clone());
    let latest = block_on(c.pull_request_detail("sandbox", 7)).unwrap();
    assert_eq!(latest.head_sha, "bbb");
    assert_eq!(latest.base_sha, "base2");
    assert_eq!(latest.shown_patch_set, Some(2));
    assert_eq!(latest.body, "Second try.\nIt works now.");
    assert_eq!(latest.merge_state, MergeState::Blocked);
    assert_eq!(latest.additions, Some(3));
    assert_eq!(latest.summary.source_branch, "refs/changes/07/7/2");
    let numbers: Vec<_> = latest.patch_sets.iter().map(|p| p.number).collect();
    assert_eq!(numbers, [1, 2]);
    assert_eq!(latest.patch_sets[0].sha, "aaa");
    assert_eq!(latest.patch_sets[1].created_at, "2026-10-15T07:00:00Z");

    let labels = &latest.vote_labels;
    assert_eq!(labels[0].name, "Code-Review", "Code-Review first");
    let cr: Vec<_> = labels[0].values.iter().map(|v| v.value).collect();
    assert_eq!(cr, [-2, -1, 0, 1], "+2 is not permitted: it is not offered");
    assert_eq!(labels[0].values[0].text, "Do not submit");
    assert_eq!(labels[0].current, 1, "my own vote");
    assert_eq!(labels[1].name, "Verified");
    assert_eq!(labels[1].current, 0);

    let first = block_on(c.pull_request_detail_at("sandbox", 7, Some(1))).unwrap();
    assert_eq!(first.head_sha, "aaa");
    assert_eq!(first.base_sha, "base1");
    assert_eq!(first.shown_patch_set, Some(1));
    assert_eq!(first.summary.source_branch, "refs/changes/07/7/1");
    assert!(block_on(c.pull_request_detail_at("sandbox", 7, Some(9))).is_err());
}

#[test]
fn files_are_listed_for_the_chosen_patch_set_without_the_commit_message() {
    let t = Arc::new(RecordedTransport::new().on(
        "/files/",
        200,
        &gerrit(
            r#"{"/COMMIT_MSG":{"status":"A","lines_inserted":11},
                "src/new.rs":{"status":"A","lines_inserted":3},
                "src/gone.rs":{"status":"D","lines_deleted":5},
                "src/moved.rs":{"status":"R","old_path":"src/was.rs","lines_inserted":1,"lines_deleted":1},
                "src/main.rs":{"lines_inserted":2,"lines_deleted":1}}"#,
        ),
    ));
    let c = client(t.clone());
    let files = block_on(c.pull_request_files_at("sandbox", 7, Some(2))).unwrap();
    assert!(
        t.requests()[0].url.contains("/revisions/2/files/"),
        "{}",
        t.requests()[0].url
    );
    let by_path = |p: &str| files.iter().find(|f| f.path == p).unwrap();
    assert!(files.iter().all(|f| f.path != "/COMMIT_MSG"));
    assert_eq!(
        by_path("src/new.rs").status,
        crate::review::FileStatus::Added
    );
    assert_eq!(
        by_path("src/gone.rs").status,
        crate::review::FileStatus::Removed
    );
    let moved = by_path("src/moved.rs");
    assert_eq!(moved.status, crate::review::FileStatus::Renamed);
    assert_eq!(moved.previous_path.as_deref(), Some("src/was.rs"));
    assert_eq!(by_path("src/main.rs").additions, 2);
    block_on(c.pull_request_files("sandbox", 7)).unwrap();
    assert!(t.requests()[1].url.contains("/revisions/current/files/"));
}

fn comments_json() -> String {
    gerrit(
        r#"{"src/main.rs":[
          {"id":"c1","patch_set":2,"line":3,"message":"Why hello?","updated":"2026-10-15 07:12:00.000000000",
           "author":{"_account_id":9,"username":"bob"},"unresolved":true},
          {"id":"c2","patch_set":2,"line":3,"in_reply_to":"c1","message":"Because.","updated":"2026-10-15 07:13:00.000000000",
           "author":{"_account_id":5,"username":"olga"},"unresolved":true},
          {"id":"c3","patch_set":1,"line":2,"message":"Old remark","updated":"2026-10-15 06:40:00.000000000",
           "author":{"_account_id":9,"username":"bob"},"unresolved":true},
          {"id":"c4","patch_set":2,"line":1,"side":"PARENT","message":"On the base","updated":"2026-10-15 07:14:00.000000000",
           "author":{"_account_id":9,"username":"bob"},"unresolved":false},
          {"id":"c5","patch_set":2,"message":"Whole file","updated":"2026-10-15 07:15:00.000000000",
           "author":{"_account_id":9,"username":"bob"}}],
         "/PATCHSET_LEVEL":[
          {"id":"c6","patch_set":2,"message":"General","updated":"2026-10-15 07:16:00.000000000",
           "author":{"_account_id":9,"username":"bob"}}]}"#,
    )
}

#[test]
fn comments_chain_into_threads_and_the_newest_decides_resolution() {
    let drafts = gerrit(
        r#"{"src/main.rs":[
          {"id":"d1","patch_set":2,"line":3,"in_reply_to":"c2","message":"Fixing it","updated":"2026-10-15 07:20:00.000000000","unresolved":false}]}"#,
    );
    let t = Arc::new(
        RecordedTransport::new()
            .on("/a/accounts/self", 200, &gerrit(ME))
            .on("/drafts", 200, &drafts)
            .on("/comments", 200, &comments_json()),
    );
    let threads = block_on(client(t).pull_request_threads_at("sandbox", 7, Some(2))).unwrap();
    assert_eq!(
        threads.len(),
        4,
        "the patch-set-level comment is not a code thread"
    );
    let main = threads.iter().find(|t| t.comments[0].id == "c1").unwrap();
    assert_eq!(main.path, "src/main.rs");
    assert_eq!(main.line, Some(3));
    assert_eq!(main.side, DiffSide::New);
    assert_eq!(main.comments.len(), 3);
    assert_eq!(
        main.comments[2].author, "admin (draft)",
        "a draft is marked"
    );
    assert!(main.resolved, "the draft reply says unresolved: false");
    assert!(!main.outdated);
    assert!(main.resolvable);
    // Replies answer the newest published comment, on the root's patch set.
    assert!(main.id.starts_with("c2|2|0|n|3|src/main.rs"), "{}", main.id);

    let old = threads.iter().find(|t| t.comments[0].id == "c3").unwrap();
    assert!(old.outdated, "written on patch set 1, patch set 2 is shown");
    assert_eq!(old.line, None);
    assert_eq!(old.original_line, Some(2));
    assert!(!old.resolved);

    let base = threads.iter().find(|t| t.comments[0].id == "c4").unwrap();
    assert_eq!(base.side, DiffSide::Old);
    assert!(base.resolved);

    let file = threads.iter().find(|t| t.comments[0].id == "c5").unwrap();
    assert_eq!(
        file.line,
        Some(1),
        "a file comment sits at the top of the file"
    );
    assert!(file.resolved, "no `unresolved` flag counts as resolved");
}

#[test]
fn replying_posts_a_comment_on_the_threads_patch_set() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/a/accounts/self", 200, &gerrit(ME))
            .on("/review", 200, &gerrit("{}")),
    );
    let c = client(t.clone());
    block_on(c.reply_to_thread("sandbox", 7, "c2|2|1|n|3|src/main.rs", "Thanks")).unwrap();
    let post = t
        .requests()
        .into_iter()
        .find(|r| r.method == Method::Post)
        .unwrap();
    assert_eq!(
        post.url,
        "http://127.0.0.1:8081/a/changes/sandbox~7/revisions/2/review"
    );
    assert_eq!(
        body(&post),
        json!({ "comments": { "src/main.rs": [
            { "in_reply_to": "c2", "message": "Thanks", "unresolved": true, "line": 3 }
        ]}})
    );
}

#[test]
fn resolving_is_a_reply_with_unresolved_false() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/a/accounts/self", 200, &gerrit(ME))
            .on("/review", 200, &gerrit("{}")),
    );
    let c = client(t.clone());
    block_on(c.resolve_thread("sandbox", 7, "c4|2|1|o|1|src/lib.rs", true)).unwrap();
    block_on(c.resolve_thread("sandbox", 7, "c4|2|0|o|1|src/lib.rs", false)).unwrap();
    let posts: Vec<Value> = t
        .requests()
        .iter()
        .filter(|r| r.method == Method::Post)
        .map(body)
        .collect();
    let first = &posts[0]["comments"]["src/lib.rs"][0];
    assert_eq!(first["unresolved"], false);
    assert_eq!(first["in_reply_to"], "c4");
    assert_eq!(first["side"], "PARENT");
    assert_eq!(first["message"], "Done");
    assert_eq!(posts[1]["comments"]["src/lib.rs"][0]["unresolved"], true);
    assert!(c.resolve_thread("sandbox", 7, "garbage", true).await_err());
}

trait AwaitErr {
    fn await_err(self) -> bool;
}

impl<F: std::future::Future<Output = crate::Result<()>>> AwaitErr for F {
    fn await_err(self) -> bool {
        block_on(self).is_err()
    }
}

#[test]
fn a_review_sends_votes_comments_and_publishes_the_drafts() {
    let t = Arc::new(RecordedTransport::new().on("/review", 200, &gerrit("{}")));
    let outcome = block_on(client(t.clone()).submit_review(
        "sandbox",
        7,
        &ReviewSubmission {
            event: ReviewEvent::Comment,
            body: "Looks mostly fine".into(),
            comments: vec![
                PendingComment {
                    path: "src/main.rs".into(),
                    previous_path: None,
                    line: 3,
                    side: DiffSide::New,
                    start_line: Some(2),
                    other_line: None,
                    body: "Rename this".into(),
                },
                PendingComment {
                    path: "src/main.rs".into(),
                    previous_path: None,
                    line: 1,
                    side: DiffSide::Old,
                    start_line: None,
                    other_line: None,
                    body: "Was better".into(),
                },
            ],
            head_sha: Some("aaa".into()),
            votes: vec![
                ReviewVote {
                    label: "Code-Review".into(),
                    value: 1,
                },
                ReviewVote {
                    label: "Verified".into(),
                    value: -1,
                },
            ],
        },
    ))
    .unwrap();
    assert_eq!(outcome.comments, 2);
    let post = &t.requests()[0];
    assert_eq!(
        post.url, "http://127.0.0.1:8081/a/changes/sandbox~7/revisions/aaa/review",
        "the sha of the patch set that was read"
    );
    assert_eq!(
        body(post),
        json!({
            "drafts": "PUBLISH_ALL_REVISIONS",
            "message": "Looks mostly fine",
            "labels": { "Code-Review": 1, "Verified": -1 },
            "comments": { "src/main.rs": [
                { "line": 3, "message": "Rename this", "unresolved": true,
                  "range": { "start_line": 2, "start_character": 0, "end_line": 3, "end_character": 0 } },
                { "line": 1, "message": "Was better", "unresolved": true, "side": "PARENT" }
            ]}
        })
    );
}

#[test]
fn approve_votes_the_highest_permitted_code_review_and_request_changes_the_mildest_negative() {
    for (event, want) in [(ReviewEvent::Approve, 1), (ReviewEvent::RequestChanges, -1)] {
        let t = Arc::new(
            RecordedTransport::new()
                .on("/a/accounts/self", 200, &gerrit(ME))
                .on("/a/changes/sandbox~7?", 200, &gerrit(change_json()))
                .on("/review", 200, &gerrit("{}")),
        );
        block_on(client(t.clone()).submit_review(
            "sandbox",
            7,
            &ReviewSubmission {
                event,
                body: "words".into(),
                comments: vec![],
                head_sha: None,
                votes: vec![],
            },
        ))
        .unwrap();
        let post = t
            .requests()
            .into_iter()
            .find(|r| r.method == Method::Post)
            .unwrap();
        assert!(
            post.url.ends_with("/revisions/current/review"),
            "{}",
            post.url
        );
        assert_eq!(body(&post)["labels"], json!({ "Code-Review": want }));
    }
}

#[test]
fn an_empty_comment_review_is_refused_before_sending() {
    let t = Arc::new(RecordedTransport::new());
    let err = block_on(client(t.clone()).submit_review(
        "sandbox",
        7,
        &ReviewSubmission {
            event: ReviewEvent::Comment,
            body: String::new(),
            comments: vec![],
            head_sha: None,
            votes: vec![],
        },
    ))
    .unwrap_err();
    assert!(err.to_string().contains("needs a body"), "{err}");
    assert!(t.requests().is_empty());
}

#[test]
fn a_conversation_comment_is_a_change_message() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/a/accounts/self", 200, &gerrit(ME))
            .on("/review", 200, &gerrit("{}")),
    );
    let c = client(t.clone());
    let comment = block_on(c.add_comment("sandbox", 7, "Please rebase")).unwrap();
    assert_eq!(comment.author, "admin");
    let post = t
        .requests()
        .into_iter()
        .find(|r| r.method == Method::Post)
        .unwrap();
    assert!(post.url.ends_with("/revisions/current/review"));
    assert_eq!(body(&post), json!({ "message": "Please rebase" }));
}

#[test]
fn the_conversation_is_the_change_messages_and_reviews_are_the_code_review_votes() {
    let t =
        Arc::new(RecordedTransport::new().on("/a/changes/sandbox~7?", 200, &gerrit(change_json())));
    let c = client(t);
    let conversation = block_on(c.pull_request_conversation("sandbox", 7)).unwrap();
    assert_eq!(conversation.len(), 2);
    assert_eq!(conversation[1].author, "bob");
    let reviews = block_on(c.pull_request_reviews("sandbox", 7)).unwrap();
    assert_eq!(reviews.len(), 2);
    assert_eq!(reviews[0].author, "admin");
    assert_eq!(reviews[0].state, crate::review::ReviewState::Approved);
    assert_eq!(
        reviews[1].state,
        crate::review::ReviewState::ChangesRequested
    );
    assert_eq!(reviews[1].body, "Code-Review -1");
}

#[test]
fn a_change_is_never_created_through_the_api() {
    let t = Arc::new(RecordedTransport::new());
    let err =
        block_on(client(t.clone()).create_pull_request(&NewPullRequest::default())).unwrap_err();
    assert!(err.to_string().contains("refs/for"), "{err}");
    assert!(t.requests().is_empty());
}

#[test]
fn the_spec_knows_change_pages_remotes_and_the_change_ref() {
    use crate::spec::{spec_for, CheckoutSource};
    use gitpoulpi_config::ApiFamily;
    let spec = spec_for(ApiFamily::Gerrit);
    assert_eq!(spec.tag, "GR");
    assert_eq!(
        (spec.pr_web_url)("http://h:8081/", "team/tool", 7).as_deref(),
        Some("http://h:8081/c/team/tool/+/7")
    );
    for (url, project) in [
        ("ssh://admin@h:29418/sandbox", "sandbox"),
        ("https://h/sandbox", "sandbox"),
        ("https://h/a/team/tool", "team/tool"),
        ("https://h/a/sandbox.git", "sandbox"),
    ] {
        assert_eq!((spec.parse_remote)(url).unwrap().project, project, "{url}");
    }
    let pr = crate::model::PullRequestSummary {
        number: 7,
        iid_label: "#7".into(),
        title: "t".into(),
        state: PullRequestState::Open,
        source_branch: "refs/changes/07/7/3".into(),
        target_branch: "master".into(),
        source_project: None,
        author: "a".into(),
        web_url: String::new(),
        updated_at: String::new(),
        review_requested: false,
        assignees: vec![],
        reviewers: vec![],
        checks: None,
        has_conflicts: None,
    };
    assert_eq!(
        (spec.checkout)(&pr),
        CheckoutSource::HeadRef {
            remote_ref: "refs/changes/07/7/3".into(),
            local: "change/7".into()
        }
    );
}

/// Reads against a real Gerrit (the Docker bench: `scripts/test-bench.sh up
/// gerrit`, then `seed gerrit`). Run with `cargo test -p gitpoulpi-hosting
/// live_gerrit -- --ignored --nocapture`, with `GITPOULPI_LIVE_GERRIT` (the
/// URL), `GITPOULPI_LIVE_GERRIT_USER`, `GITPOULPI_LIVE_GERRIT_TOKEN` and
/// optionally `GITPOULPI_LIVE_GERRIT_PROJECT` (default `gitpoulpi-sandbox`).
/// Reads only. Never part of the gate.
#[test]
#[ignore = "live: set GITPOULPI_LIVE_TESTS=1 and run with --ignored"]
fn live_gerrit_reads() {
    if !gitpoulpi_testkit::live::enabled("live_gerrit_reads") {
        return;
    }
    let (Ok(url), Ok(user), Ok(token)) = (
        std::env::var("GITPOULPI_LIVE_GERRIT"),
        std::env::var("GITPOULPI_LIVE_GERRIT_USER"),
        std::env::var("GITPOULPI_LIVE_GERRIT_TOKEN"),
    ) else {
        panic!(
            "set GITPOULPI_LIVE_GERRIT, GITPOULPI_LIVE_GERRIT_USER and GITPOULPI_LIVE_GERRIT_TOKEN"
        );
    };
    let project = std::env::var("GITPOULPI_LIVE_GERRIT_PROJECT")
        .unwrap_or_else(|_| "gitpoulpi-sandbox".to_string());
    let transport = Arc::new(crate::ReqwestTransport::new().expect("transport"));
    let mut integration = Integration::new("live", IntegrationKind::Gerrit, url);
    integration.account = user;
    let client = client_for(
        integration,
        token,
        transport,
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap();
    block_on(async {
        let check = client.check_token().await.expect("credentials");
        println!("user: {}", check.user.login);
        let projects = client.list_projects().await.expect("projects");
        assert!(projects.iter().any(|p| p.full_name == project));
        let changes = client
            .list_pull_requests(&project, &PrFilter::default())
            .await
            .expect("changes");
        println!("open changes: {}", changes.len());
        let first = changes
            .first()
            .expect("a change (scripts/test-bench.sh seed gerrit)");
        let detail = client
            .pull_request_detail(&project, first.number)
            .await
            .expect("detail");
        assert!(!detail.head_sha.is_empty());
        assert!(!detail.patch_sets.is_empty());
        println!(
            "patch sets: {}, labels: {:?}",
            detail.patch_sets.len(),
            detail
                .vote_labels
                .iter()
                .map(|l| &l.name)
                .collect::<Vec<_>>()
        );
        client
            .pull_request_files(&project, first.number)
            .await
            .expect("files");
        client
            .pull_request_threads(&project, first.number)
            .await
            .expect("threads");
    });
}

#[test]
fn files_between_two_patch_sets_ask_the_server_for_the_difference() {
    let t = Arc::new(RecordedTransport::new().on(
        "/files/",
        200,
        &gerrit(r#"{"src/main.rs":{"lines_inserted":1,"lines_deleted":1}}"#),
    ));
    let c = client(t.clone());
    let files = block_on(c.pull_request_files_against("sandbox", 7, Some(2), Some(1))).unwrap();
    assert_eq!(files.len(), 1);
    assert!(
        t.requests()[0].url.ends_with("/revisions/2/files/?base=1"),
        "{}",
        t.requests()[0].url
    );
}

#[test]
fn the_detail_says_when_the_change_can_be_submitted_or_abandoned() {
    let mut change = change_json();
    change["submittable"] = json!(true);
    change["actions"] = json!({ "abandon": { "method": "POST", "enabled": true } });
    let t = Arc::new(
        RecordedTransport::new()
            .on("/accounts/self", 200, &gerrit(ME))
            .on("/changes/sandbox~7", 200, &gerrit(change)),
    );
    let c = client(t.clone());
    let d = block_on(c.pull_request_detail_at("sandbox", 7, None)).unwrap();
    assert!(d.can_submit && d.can_abandon);
    assert!(
        t.requests()
            .iter()
            .any(|r| r.url.contains("o=CHANGE_ACTIONS")),
        "the change actions are asked for"
    );
    // The fixture's own change is not submittable and offers no action.
    let t = Arc::new(
        RecordedTransport::new()
            .on("/accounts/self", 200, &gerrit(ME))
            .on("/changes/sandbox~7", 200, &gerrit(change_json())),
    );
    let d = block_on(client(t).pull_request_detail_at("sandbox", 7, None)).unwrap();
    assert!(!d.can_submit && !d.can_abandon);
}

#[test]
fn submitting_and_abandoning_post_to_the_change() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/submit", 200, &gerrit("{}"))
            .on("/abandon", 200, &gerrit("{}")),
    );
    let c = client(t.clone());
    block_on(c.submit_change("sandbox", 7)).unwrap();
    block_on(c.abandon_change("sandbox", 7, Some("superseded by 9"))).unwrap();
    block_on(c.abandon_change("sandbox", 7, Some("  "))).unwrap();
    let r = t.requests();
    assert_eq!(r[0].url, "http://127.0.0.1:8081/a/changes/sandbox~7/submit");
    assert_eq!(r[0].method, Method::Post);
    assert_eq!(
        r[1].url,
        "http://127.0.0.1:8081/a/changes/sandbox~7/abandon"
    );
    assert_eq!(body(&r[1]), json!({ "message": "superseded by 9" }));
    assert_eq!(body(&r[2]), json!({}), "a blank message sends none");
}

#[test]
fn a_refused_submit_reports_the_servers_words() {
    let t = Arc::new(RecordedTransport::new().on("/submit", 409, "needs Code-Review"));
    let c = client(t);
    let err = block_on(c.submit_change("sandbox", 7))
        .unwrap_err()
        .to_string();
    assert!(err.contains("needs Code-Review"), "{err}");
}
