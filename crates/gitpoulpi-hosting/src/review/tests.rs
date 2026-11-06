//! The review operations over `RecordedTransport`: one fake host per
//! family, one test per behaviour, and assertions on the requests sent as
//! well as the values returned. `RecordedTransport` answers the first
//! registered fragment a URL contains, so the specific endpoints come
//! before `pulls/7` / `merge_requests/7`.

use std::sync::Arc;

use serde_json::Value;

use super::*;
use crate::client::{client_for, HostingClient};
use crate::http::{HttpRequest, Method, RecordedTransport};
use crate::integration::{Integration, IntegrationKind};
use crate::model::ChecksState;
use crate::Error;

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

fn github(url: &str, kind: IntegrationKind, t: Arc<RecordedTransport>) -> Box<dyn HostingClient> {
    client_for(
        Integration::new("gh", kind, url),
        "tok".into(),
        t,
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap()
}

fn gitlab(t: Arc<RecordedTransport>) -> Box<dyn HostingClient> {
    client_for(
        Integration::new(
            "gl",
            IntegrationKind::GitlabSelfManaged,
            "https://gitlab.example.com",
        ),
        "tok".into(),
        t,
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap()
}

fn dotcom(t: Arc<RecordedTransport>) -> Box<dyn HostingClient> {
    github("https://github.com", IntegrationKind::Github, t)
}

fn body_of(request: &HttpRequest) -> Value {
    serde_json::from_str(request.body.as_deref().unwrap_or("null")).unwrap()
}

fn sent(t: &RecordedTransport, method: Method, needle: &str) -> Vec<HttpRequest> {
    t.requests()
        .into_iter()
        .filter(|r| r.method == method && r.url.contains(needle))
        .collect()
}

fn submission(event: ReviewEvent, body: &str, comments: Vec<PendingComment>) -> ReviewSubmission {
    ReviewSubmission {
        votes: Vec::new(),
        event,
        body: body.to_string(),
        comments,
        head_sha: Some("h1".to_string()),
    }
}

fn comment(path: &str, line: u32, side: DiffSide, body: &str) -> PendingComment {
    PendingComment {
        path: path.to_string(),
        previous_path: None,
        line,
        side,
        start_line: None,
        other_line: None,
        body: body.to_string(),
    }
}

// ---- shared pieces ----

#[test]
fn refspecs_name_the_hosts_head_and_a_namespaced_destination() {
    assert_eq!(
        review_head_refspec(IntegrationKind::Github, 7).as_deref(),
        Some("+refs/pull/7/head:refs/gitpoulpi/pr/7")
    );
    assert_eq!(
        review_head_refspec(IntegrationKind::GitlabSelfManaged, 7).as_deref(),
        Some("+refs/merge-requests/7/head:refs/gitpoulpi/pr/7")
    );
    assert_eq!(
        review_base_refspec("main", 7),
        "+refs/heads/main:refs/gitpoulpi/pr/7-base"
    );
    for spec in [
        review_head_refspec(IntegrationKind::Github, 7).unwrap(),
        review_base_refspec("release/1.x", 7),
    ] {
        assert!(
            gitpoulpi_core::domain::parse_fetch_refspec(&spec).is_some(),
            "{spec} must pass the backend's check"
        );
    }
}

#[test]
fn diff_stats_count_hunk_lines_only() {
    let diff =
        "@@ -1,3 +1,3 @@\n a\n-b\n+c\n+d\n--- not a file header inside a hunk is a removal\n";
    assert_eq!(count_changes(diff), (2, 2));
    assert_eq!(count_changes("--- a/x\n+++ b/x\n"), (0, 0));
}

#[test]
fn events_and_sides_parse_the_cli_spellings() {
    assert_eq!(
        ReviewEvent::parse("request-changes"),
        Some(ReviewEvent::RequestChanges)
    );
    assert_eq!(
        ReviewEvent::parse("REQUEST_CHANGES"),
        Some(ReviewEvent::RequestChanges)
    );
    assert_eq!(ReviewEvent::parse("merge"), None);
    assert_eq!(DiffSide::parse("old"), Some(DiffSide::Old));
    assert_eq!(DiffSide::parse("Right"), Some(DiffSide::New));
    assert_eq!(DiffSide::parse("middle"), None);
}

#[test]
fn a_client_without_review_support_says_so() {
    let bitbucket = client_for(
        Integration::new("bb", IntegrationKind::Bitbucket, "https://bitbucket.org"),
        "tok".into(),
        Arc::new(RecordedTransport::new()),
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap();
    assert_eq!(
        bitbucket.review_capabilities(),
        ReviewCapabilities::default()
    );
    assert!(matches!(
        block_on(bitbucket.pull_request_threads("a/b", 1)),
        Err(Error::Unsupported(_))
    ));
    assert!(!ReviewCapabilities::default().allows(ReviewEvent::Comment));
}

// ---- GitHub ----

const GH_PULL: &str = r#"{"number":7,"title":"Add moons","state":"open","draft":false,
 "merged_at":null,"html_url":"https://github.com/o/r/pull/7","updated_at":"2026-10-01T10:00:00Z",
 "created_at":"2026-09-30T10:00:00Z","body":"Adds **moons**.","user":{"login":"ines"},
 "head":{"ref":"feature/moons","sha":"h1","repo":{"full_name":"o/r"}},
 "base":{"ref":"main","sha":"b1","repo":{"full_name":"o/r"}},
 "mergeable":true,"mergeable_state":"blocked","additions":12,"deletions":3,"changed_files":2,
 "assignees":[],"requested_reviewers":[{"login":"mara"}]}"#;

#[test]
fn github_detail_maps_the_request_and_its_checks() {
    let t = Arc::new(
        RecordedTransport::new()
            .on(
                "commits/h1/check-runs",
                200,
                r#"{"check_runs":[{"status":"completed","conclusion":"success"},{"status":"completed","conclusion":"failure"}]}"#,
            )
            .on("pulls/7", 200, GH_PULL),
    );
    let detail = block_on(dotcom(t.clone()).pull_request_detail("o/r", 7)).unwrap();
    assert_eq!(detail.summary.number, 7);
    assert_eq!(detail.summary.source_branch, "feature/moons");
    assert_eq!(detail.body, "Adds **moons**.");
    assert_eq!(
        (detail.head_sha.as_str(), detail.base_sha.as_str()),
        ("h1", "b1")
    );
    assert_eq!(detail.merge_state, MergeState::Blocked);
    assert_eq!(detail.checks, Some(ChecksState::Failing));
    assert_eq!(detail.additions, Some(12));
    assert_eq!(detail.changed_files, Some(2));
    assert!(detail.capabilities.request_changes && detail.capabilities.resolve);
    let first = &t.requests()[0];
    assert_eq!(first.url, "https://api.github.com/repos/o/r/pulls/7");
}

#[test]
fn github_detail_survives_a_token_that_cannot_read_checks_and_flags_conflicts() {
    let t = Arc::new(
        RecordedTransport::new()
            .on(
                "check-runs",
                403,
                r#"{"message":"Resource not accessible"}"#,
            )
            .on(
                "pulls/7",
                200,
                &GH_PULL.replace("\"mergeable\":true", "\"mergeable\":false"),
            ),
    );
    let detail = block_on(dotcom(t).pull_request_detail("o/r", 7)).unwrap();
    assert_eq!(detail.checks, None);
    assert_eq!(detail.merge_state, MergeState::Conflicting);
}

#[test]
fn github_files_carry_status_counts_and_the_previous_path() {
    let t = Arc::new(RecordedTransport::new().on(
        "pulls/7/files",
        200,
        r#"[{"filename":"a.rs","status":"modified","additions":3,"deletions":1},
            {"filename":"new.rs","status":"added","additions":9,"deletions":0},
            {"filename":"b.rs","status":"renamed","additions":0,"deletions":0,"previous_filename":"old.rs"},
            {"filename":"gone.rs","status":"removed","additions":0,"deletions":4}]"#,
    ));
    let files = block_on(dotcom(t).pull_request_files("o/r", 7)).unwrap();
    let kinds: Vec<_> = files.iter().map(|f| f.status).collect();
    assert_eq!(
        kinds,
        [
            FileStatus::Modified,
            FileStatus::Added,
            FileStatus::Renamed,
            FileStatus::Removed
        ]
    );
    assert_eq!(files[2].previous_path.as_deref(), Some("old.rs"));
    assert_eq!((files[0].additions, files[0].deletions), (3, 1));
}

const GH_THREADS: &str = r#"{"data":{"repository":{"pullRequest":{"reviewThreads":{
 "pageInfo":{"hasNextPage":false,"endCursor":null},"nodes":[
 {"id":"PRRT_1","isResolved":false,"isOutdated":false,"path":"src/a.rs","line":12,"originalLine":12,
  "startLine":10,"originalStartLine":10,"diffSide":"RIGHT","comments":{"nodes":[
   {"databaseId":501,"author":{"login":"ines"},"body":"why?","createdAt":"2026-10-01T10:00:00Z","url":"https://x/1","replyTo":null},
   {"databaseId":502,"author":{"login":"tomas"},"body":"because","createdAt":"2026-10-01T11:00:00Z","url":"https://x/2","replyTo":{"databaseId":501}}]}},
 {"id":"PRRT_2","isResolved":true,"isOutdated":true,"path":"src/b.rs","line":null,"originalLine":4,
  "startLine":null,"originalStartLine":null,"diffSide":"LEFT","comments":{"nodes":[
   {"databaseId":503,"author":null,"body":"old","createdAt":"2026-09-30T10:00:00Z","url":null,"replyTo":null}]}}]}}}}}"#;

#[test]
fn github_threads_come_from_graphql_with_ids_sides_and_resolution() {
    let t = Arc::new(RecordedTransport::new().on("/graphql", 200, GH_THREADS));
    let threads = block_on(dotcom(t.clone()).pull_request_threads("o/r", 7)).unwrap();

    let request = &t.requests()[0];
    assert_eq!(request.method, Method::Post);
    assert_eq!(request.url, "https://api.github.com/graphql");
    let vars = &body_of(request)["variables"];
    assert_eq!(
        (
            vars["owner"].as_str(),
            vars["name"].as_str(),
            vars["number"].as_u64()
        ),
        (Some("o"), Some("r"), Some(7))
    );

    assert_eq!(threads.len(), 2);
    let live = &threads[0];
    assert_eq!(live.id, "PRRT_1");
    assert_eq!(live.anchor(), Some((DiffSide::New, 12)));
    assert_eq!(live.start_line, Some(10));
    assert!(!live.resolved && !live.outdated);
    assert_eq!(live.comments.len(), 2);
    assert_eq!(live.comments[1].in_reply_to.as_deref(), Some("501"));

    let old = &threads[1];
    assert!(old.resolved && old.outdated);
    assert_eq!(
        old.anchor(),
        None,
        "an outdated thread has no inline anchor"
    );
    assert_eq!(old.original_line, Some(4));
    assert_eq!(old.side, DiffSide::Old);
    assert_eq!(old.comments[0].author, "", "a deleted account has no login");
}

#[test]
fn github_enterprise_graphql_lives_under_api_graphql() {
    let t = Arc::new(RecordedTransport::new().on("/graphql", 200, GH_THREADS));
    let ghes = github(
        "https://ghe.example.com",
        IntegrationKind::GithubEnterprise,
        t.clone(),
    );
    block_on(ghes.pull_request_threads("o/r", 7)).unwrap();
    assert_eq!(t.requests()[0].url, "https://ghe.example.com/api/graphql");
}

#[test]
fn a_graphql_error_inside_a_200_is_an_error() {
    let t = Arc::new(RecordedTransport::new().on(
        "/graphql",
        200,
        r#"{"data":null,"errors":[{"message":"Could not resolve to a PullRequest"}]}"#,
    ));
    let err = block_on(dotcom(t).pull_request_threads("o/r", 99)).unwrap_err();
    assert!(err.to_string().contains("Could not resolve"), "{err}");
}

#[test]
fn github_resolve_and_reopen_use_the_two_mutations() {
    let t = Arc::new(RecordedTransport::new().on(
        "/graphql",
        200,
        r#"{"data":{"resolveReviewThread":{"thread":{"isResolved":true}}}}"#,
    ));
    let c = dotcom(t.clone());
    block_on(c.resolve_thread("o/r", 7, "PRRT_1", true)).unwrap();
    block_on(c.resolve_thread("o/r", 7, "PRRT_1", false)).unwrap();
    let sent = t.requests();
    let query = |i: usize| body_of(&sent[i])["query"].as_str().unwrap().to_string();
    assert!(query(0).contains("resolveReviewThread") && !query(0).contains("unresolve"));
    assert!(query(1).contains("unresolveReviewThread"));
    assert_eq!(body_of(&sent[0])["variables"]["thread"], "PRRT_1");
}

#[test]
fn github_reply_goes_through_the_thread_mutation() {
    let t = Arc::new(RecordedTransport::new().on(
        "/graphql",
        200,
        r#"{"data":{"addPullRequestReviewThreadReply":{"comment":{"databaseId":777,
           "author":{"login":"mara"},"body":"done","createdAt":"2026-10-02T10:00:00Z",
           "url":"https://x/7","replyTo":{"databaseId":501}}}}}"#,
    ));
    let c = dotcom(t.clone());
    let reply = block_on(c.reply_to_thread("o/r", 7, "PRRT_1", "done")).unwrap();
    assert_eq!((reply.id.as_str(), reply.author.as_str()), ("777", "mara"));
    assert_eq!(reply.in_reply_to.as_deref(), Some("501"));
    let vars = &body_of(&t.requests()[0])["variables"];
    assert_eq!(
        (vars["thread"].as_str(), vars["body"].as_str()),
        (Some("PRRT_1"), Some("done"))
    );
    assert!(block_on(c.reply_to_thread("o/r", 7, "PRRT_1", "  ")).is_err());
    assert_eq!(
        t.requests().len(),
        1,
        "an empty reply is refused before any request"
    );
}

#[test]
fn github_conversation_lists_and_posts_issue_comments() {
    let t = Arc::new(RecordedTransport::new().on(
        "issues/7/comments",
        200,
        r#"[{"id":1,"body":"hello","created_at":"2026-10-01T10:00:00Z","user":{"login":"ines"},"html_url":"https://x/c1"}]"#,
    ));
    let list = block_on(dotcom(t).pull_request_conversation("o/r", 7)).unwrap();
    assert_eq!(
        (list[0].id.as_str(), list[0].author.as_str()),
        ("1", "ines")
    );

    let t = Arc::new(RecordedTransport::new().on(
        "issues/7/comments",
        201,
        r#"{"id":2,"body":"thanks","created_at":"x","user":{"login":"mara"}}"#,
    ));
    let c = dotcom(t.clone());
    let posted = block_on(c.add_comment("o/r", 7, "thanks")).unwrap();
    assert_eq!(posted.id, "2");
    assert_eq!(
        body_of(&sent(&t, Method::Post, "issues/7/comments")[0])["body"],
        "thanks"
    );
    assert!(block_on(c.add_comment("o/r", 7, " ")).is_err());
    assert_eq!(t.requests().len(), 1);
}

#[test]
fn github_submit_sends_one_review_with_sides_ranges_and_the_head() {
    let t = Arc::new(RecordedTransport::new().on("pulls/7/reviews", 200, r#"{"id":4242}"#));
    let mut ranged = comment("src/a.rs", 14, DiffSide::New, "extract this");
    ranged.start_line = Some(10);
    let review = submission(
        ReviewEvent::RequestChanges,
        "needs work",
        vec![ranged, comment("src/b.rs", 4, DiffSide::Old, "why remove")],
    );
    let outcome = block_on(dotcom(t.clone()).submit_review("o/r", 7, &review)).unwrap();
    assert_eq!(outcome.review_id.as_deref(), Some("4242"));
    assert_eq!(outcome.comments, 2);

    let posts = sent(&t, Method::Post, "pulls/7/reviews");
    assert_eq!(posts.len(), 1, "one round trip however many comments");
    let body = body_of(&posts[0]);
    assert_eq!(body["event"], "REQUEST_CHANGES");
    assert_eq!(body["body"], "needs work");
    assert_eq!(body["commit_id"], "h1");
    let comments = body["comments"].as_array().unwrap();
    assert_eq!(comments[0]["side"], "RIGHT");
    assert_eq!(comments[0]["line"], 14);
    assert_eq!(comments[0]["start_line"], 10);
    assert_eq!(comments[0]["start_side"], "RIGHT");
    assert_eq!(comments[1]["side"], "LEFT");
    assert!(comments[1].get("start_line").is_none());
}

#[test]
fn github_submit_refuses_an_empty_review_before_sending_anything() {
    let t = Arc::new(RecordedTransport::new());
    let c = dotcom(t.clone());
    for event in [ReviewEvent::Comment, ReviewEvent::RequestChanges] {
        let err = block_on(c.submit_review("o/r", 7, &submission(event, " ", vec![]))).unwrap_err();
        assert!(matches!(err, Error::Api { status: 422, .. }), "{err}");
    }
    let backwards = {
        let mut c = comment("a", 3, DiffSide::New, "x");
        c.start_line = Some(5);
        c
    };
    assert!(block_on(c.submit_review(
        "o/r",
        7,
        &submission(ReviewEvent::Comment, "", vec![backwards])
    ))
    .is_err());
    // An approval needs no words.
    assert!(t.requests().is_empty());
}

#[test]
fn github_surfaces_a_rejected_line_comment() {
    let t = Arc::new(RecordedTransport::new().on(
        "pulls/7/reviews",
        422,
        r#"{"message":"Unprocessable Entity","errors":["Line must be part of the diff"]}"#,
    ));
    let review = submission(
        ReviewEvent::Comment,
        "",
        vec![comment("a.rs", 999, DiffSide::New, "x")],
    );
    let err = block_on(dotcom(t).submit_review("o/r", 7, &review)).unwrap_err();
    assert!(
        err.to_string().contains("Line must be part of the diff"),
        "{err}"
    );
}

// ---- GitLab ----

const GL_MR: &str = r#"{"iid":7,"title":"Add moons","state":"opened","draft":false,
 "source_branch":"feature/moons","target_branch":"main","source_project_id":1,"target_project_id":1,
 "author":{"username":"ines"},"web_url":"https://gitlab.example.com/g/p/-/merge_requests/7",
 "updated_at":"2026-10-01T10:00:00Z","created_at":"2026-09-30T10:00:00Z","description":"Adds moons.",
 "assignees":[],"reviewers":[{"username":"mara"}],"sha":"h1",
 "diff_refs":{"base_sha":"b0","start_sha":"s0","head_sha":"h1"},
 "detailed_merge_status":"not_approved","has_conflicts":false,
 "head_pipeline":{"status":"running"},"changes_count":"2"}"#;

#[test]
fn gitlab_detail_uses_diff_refs_and_the_pipeline() {
    let t = Arc::new(RecordedTransport::new().on("merge_requests/7", 200, GL_MR));
    let detail = block_on(gitlab(t.clone()).pull_request_detail("g/p", 7)).unwrap();
    assert_eq!(detail.summary.iid_label, "!7");
    assert_eq!(
        (detail.head_sha.as_str(), detail.base_sha.as_str()),
        ("h1", "b0")
    );
    assert_eq!(detail.merge_state, MergeState::Blocked);
    assert_eq!(detail.checks, Some(ChecksState::Pending));
    assert_eq!(detail.changed_files, Some(2));
    assert!(!detail.capabilities.request_changes);
    assert_eq!(
        t.requests()[0].url,
        "https://gitlab.example.com/api/v4/projects/g%2Fp/merge_requests/7"
    );
}

#[test]
fn gitlab_files_count_lines_and_fall_back_to_changes_on_an_old_instance() {
    let rows = r#"[{"old_path":"a.rs","new_path":"a.rs","diff":"@@ -1 +1,2 @@\n a\n+b\n"},
        {"old_path":"old.rs","new_path":"new.rs","renamed_file":true,"diff":""},
        {"old_path":"/dev/null","new_path":"n.rs","new_file":true,"diff":"@@ -0,0 +1 @@\n+x\n"},
        {"old_path":"d.rs","new_path":"d.rs","deleted_file":true,"diff":"@@ -1 +0,0 @@\n-x\n"}]"#;
    let t = Arc::new(RecordedTransport::new().on("/diffs", 200, rows));
    let files = block_on(gitlab(t).pull_request_files("g/p", 7)).unwrap();
    assert_eq!(files[0].additions, 1);
    assert_eq!(files[1].status, FileStatus::Renamed);
    assert_eq!(files[1].previous_path.as_deref(), Some("old.rs"));
    assert_eq!(files[2].status, FileStatus::Added);
    assert_eq!(
        (files[3].status, files[3].deletions),
        (FileStatus::Removed, 1)
    );

    let old = Arc::new(
        RecordedTransport::new()
            .on("/diffs", 404, r#"{"message":"404 Not Found"}"#)
            .on("/changes", 200, &format!(r#"{{"changes":{rows}}}"#)),
    );
    let files = block_on(gitlab(old).pull_request_files("g/p", 7)).unwrap();
    assert_eq!(files.len(), 4);
}

const GL_DISCUSSIONS: &str = r#"[
 {"id":"d1","individual_note":false,"notes":[
  {"id":11,"body":"why?","created_at":"2026-10-01T10:00:00Z","author":{"username":"ines"},"system":false,
   "resolvable":true,"resolved":false,"type":"DiffNote",
   "position":{"old_path":"a.rs","new_path":"a.rs","new_line":12,"old_line":null,"head_sha":"h1",
     "line_range":{"start":{"new_line":10,"old_line":null}}}},
  {"id":12,"body":"because","created_at":"2026-10-01T11:00:00Z","author":{"username":"tomas"},"system":false,
   "resolvable":true,"resolved":false,"position":null}]},
 {"id":"d2","individual_note":false,"notes":[
  {"id":21,"body":"old","created_at":"2026-09-30T10:00:00Z","author":{"username":"ines"},"system":false,
   "resolvable":true,"resolved":true,"position":{"old_path":"b.rs","new_path":"b.rs","new_line":null,"old_line":4,"head_sha":"h0"}}]},
 {"id":"d3","individual_note":true,"notes":[
  {"id":31,"body":"general remark","created_at":"2026-10-01T09:00:00Z","author":{"username":"mara"},"system":false,
   "resolvable":false,"resolved":false,"position":null}]},
 {"id":"d4","individual_note":true,"notes":[
  {"id":41,"body":"assigned to @mara","created_at":"2026-10-01T08:00:00Z","author":{"username":"ines"},"system":true,
   "resolvable":false,"resolved":false,"position":null}]}]"#;

#[test]
fn gitlab_threads_and_conversation_split_the_discussions() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/discussions", 200, GL_DISCUSSIONS)
            .on("merge_requests/7", 200, GL_MR),
    );
    let c = gitlab(t);
    let threads = block_on(c.pull_request_threads("g/p", 7)).unwrap();
    assert_eq!(
        threads.len(),
        2,
        "the conversation and system notes are not threads"
    );
    let live = &threads[0];
    assert_eq!(live.id, "d1");
    assert_eq!(live.anchor(), Some((DiffSide::New, 12)));
    assert_eq!(live.start_line, Some(10));
    assert!(!live.resolved && !live.outdated && live.resolvable);
    assert_eq!(live.comments[1].in_reply_to.as_deref(), Some("11"));

    let old = &threads[1];
    assert!(old.outdated, "written against h0, now at h1, never moved");
    assert!(old.resolved);
    assert_eq!(old.side, DiffSide::Old);
    assert_eq!(old.original_line, Some(4));
    assert_eq!(old.anchor(), None);

    let talk = block_on(c.pull_request_conversation("g/p", 7)).unwrap();
    assert_eq!(talk.len(), 1);
    assert_eq!(talk[0].body, "general remark");
}

#[test]
fn gitlab_reply_resolve_and_comment_hit_the_right_endpoints() {
    let t = Arc::new(
        RecordedTransport::new()
            .on(
                "/discussions/d1/notes",
                201,
                r#"{"id":99,"body":"done","created_at":"x","author":{"username":"mara"}}"#,
            )
            .on("/discussions/d1", 200, r#"{"id":"d1"}"#)
            .on(
                "/notes",
                201,
                r#"{"id":98,"body":"hi","created_at":"x","author":{"username":"mara"}}"#,
            ),
    );
    let c = gitlab(t.clone());
    let reply = block_on(c.reply_to_thread("g/p", 7, "d1", "done")).unwrap();
    assert_eq!(reply.in_reply_to.as_deref(), Some("d1"));
    block_on(c.resolve_thread("g/p", 7, "d1", true)).unwrap();
    block_on(c.resolve_thread("g/p", 7, "d1", false)).unwrap();
    let note = block_on(c.add_comment("g/p", 7, "hi")).unwrap();
    assert_eq!(note.id, "98");

    let puts: Vec<_> = t
        .requests()
        .into_iter()
        .filter(|r| r.method == Method::Put)
        .collect();
    assert_eq!(puts.len(), 2);
    assert!(puts[0].url.ends_with("/discussions/d1?resolved=true"));
    assert_eq!(body_of(&puts[0])["resolved"], true);
    assert!(puts[1].url.ends_with("?resolved=false"));
    assert_eq!(
        body_of(&sent(&t, Method::Post, "/discussions/d1/notes")[0])["body"],
        "done"
    );
}

#[test]
fn gitlab_submit_publishes_draft_notes_then_approves() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("bulk_publish", 200, "{}")
            .on("/draft_notes", 201, r#"{"id":1}"#)
            .on("/approve", 201, "{}")
            .on("merge_requests/7", 200, GL_MR),
    );
    let mut ctx = comment("a.rs", 12, DiffSide::New, "context line");
    ctx.other_line = Some(11);
    let review = submission(
        ReviewEvent::Approve,
        "looks good",
        vec![ctx, comment("b.rs", 4, DiffSide::Old, "removed why")],
    );
    let outcome = block_on(gitlab(t.clone()).submit_review("g/p", 7, &review)).unwrap();
    assert_eq!(outcome.comments, 2);

    let drafts = sent(&t, Method::Post, "/draft_notes");
    let drafts: Vec<_> = drafts.iter().filter(|r| !r.url.contains("bulk")).collect();
    assert_eq!(drafts.len(), 3, "two comments and the summary");
    let first = body_of(drafts[0]);
    assert_eq!(first["note"], "context line");
    let pos = &first["position"];
    assert_eq!(pos["position_type"], "text");
    assert_eq!(pos["new_line"], 12);
    assert_eq!(pos["old_line"], 11);
    assert_eq!(pos["new_path"], "a.rs");
    assert_eq!(
        (
            pos["base_sha"].as_str(),
            pos["start_sha"].as_str(),
            pos["head_sha"].as_str()
        ),
        (Some("b0"), Some("s0"), Some("h1"))
    );
    let removed = body_of(drafts[1]);
    assert_eq!(removed["position"]["old_line"], 4);
    assert!(removed["position"].get("new_line").is_none());
    assert!(body_of(drafts[2]).get("position").is_none());

    let order: Vec<_> = t
        .requests()
        .into_iter()
        .filter(|r| r.method == Method::Post)
        .map(|r| r.url)
        .collect();
    let at = |needle: &str| order.iter().position(|u| u.contains(needle)).unwrap();
    assert!(at("bulk_publish") > at("/draft_notes"));
    assert!(
        at("/approve") > at("bulk_publish"),
        "approve after the notes are out"
    );
    assert_eq!(body_of(&sent(&t, Method::Post, "/approve")[0])["sha"], "h1");
}

#[test]
fn gitlab_has_no_request_changes_and_sends_nothing_for_it() {
    let t = Arc::new(RecordedTransport::new());
    let c = gitlab(t.clone());
    assert!(!c.review_capabilities().allows(ReviewEvent::RequestChanges));
    assert!(c.review_capabilities().allows(ReviewEvent::Approve));
    let err = block_on(c.submit_review(
        "g/p",
        7,
        &submission(ReviewEvent::RequestChanges, "no", vec![]),
    ))
    .unwrap_err();
    assert!(matches!(err, Error::Unsupported(_)), "{err}");
    assert!(t.requests().is_empty());
}

#[test]
fn gitlab_submit_deletes_the_drafts_it_made_when_one_fails() {
    // The second draft is refused: the first must not stay pending.
    struct Seq(std::sync::Mutex<u32>, Arc<RecordedTransport>);
    #[async_trait::async_trait]
    impl crate::http::HttpTransport for Seq {
        async fn send(&self, r: HttpRequest) -> crate::Result<crate::http::HttpResponse> {
            if r.method == Method::Post && r.url.ends_with("/draft_notes") {
                let mut n = self.0.lock().unwrap();
                *n += 1;
                if *n == 2 {
                    return Ok(crate::http::HttpResponse {
                        status: 400,
                        headers: Default::default(),
                        body: r#"{"message":"line_code is invalid"}"#.into(),
                    });
                }
            }
            self.1.send(r).await
        }
    }
    let inner = Arc::new(
        RecordedTransport::new()
            .on("/draft_notes/1", 204, "")
            .on("/draft_notes", 201, r#"{"id":1}"#)
            .on("merge_requests/7", 200, GL_MR),
    );
    let c = client_for(
        Integration::new(
            "gl",
            IntegrationKind::GitlabSelfManaged,
            "https://gitlab.example.com",
        ),
        "tok".into(),
        Arc::new(Seq(Default::default(), inner.clone())),
        &crate::trust::TrustGate::for_tests(),
    )
    .unwrap();
    let review = submission(
        ReviewEvent::Comment,
        "",
        vec![
            comment("a.rs", 1, DiffSide::New, "one"),
            comment("a.rs", 99, DiffSide::New, "two"),
        ],
    );
    let err = block_on(c.submit_review("g/p", 7, &review)).unwrap_err();
    assert!(err.to_string().contains("line_code is invalid"), "{err}");
    assert_eq!(sent(&inner, Method::Delete, "/draft_notes/1").len(), 1);
    assert!(sent(&inner, Method::Post, "bulk_publish").is_empty());
}

#[test]
fn gitlab_refuses_a_review_of_a_moved_request() {
    let t = Arc::new(RecordedTransport::new().on("merge_requests/7", 200, GL_MR));
    let mut review = submission(
        ReviewEvent::Comment,
        "x",
        vec![comment("a.rs", 1, DiffSide::New, "c")],
    );
    review.head_sha = Some("stale".into());
    let err = block_on(gitlab(t.clone()).submit_review("g/p", 7, &review)).unwrap_err();
    assert!(err.to_string().contains("moved"), "{err}");
    assert!(sent(&t, Method::Post, "draft_notes").is_empty());
}

#[test]
fn gitlab_without_draft_notes_posts_discussions_instead() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/draft_notes", 404, r#"{"message":"404 Not Found"}"#)
            .on("/discussions", 201, r#"{"id":"dx"}"#)
            .on(
                "/notes",
                201,
                r#"{"id":5,"body":"sum","author":{"username":"m"}}"#,
            )
            .on("merge_requests/7", 200, GL_MR),
    );
    let review = submission(
        ReviewEvent::Comment,
        "sum",
        vec![comment("a.rs", 3, DiffSide::New, "c")],
    );
    block_on(gitlab(t.clone()).submit_review("g/p", 7, &review)).unwrap();
    let discussion = &sent(&t, Method::Post, "/discussions")[0];
    assert_eq!(body_of(discussion)["position"]["new_line"], 3);
    assert_eq!(body_of(&sent(&t, Method::Post, "/notes")[0])["body"], "sum");
}

#[test]
fn github_lists_submitted_reviews_with_their_verdicts_and_skips_drafts() {
    let t = Arc::new(RecordedTransport::new().on(
        "pulls/7/reviews",
        200,
        r#"[{"id":5,"state":"APPROVED","body":"","submitted_at":"2026-10-02T10:00:00Z","user":{"login":"ines"}},
            {"id":6,"state":"CHANGES_REQUESTED","body":"fix it","submitted_at":"2026-10-03T10:00:00Z","user":{"login":"mara"}},
            {"id":7,"state":"PENDING","body":"draft","user":{"login":"me"}},
            {"id":8,"state":"COMMENTED","body":"","user":{"login":"tomas"}},
            {"id":9,"state":"COMMENTED","body":"thoughts","user":{"login":"tomas"}}]"#,
    ));
    let list = block_on(dotcom(t).pull_request_reviews("o/r", 7)).unwrap();
    assert_eq!(list.len(), 3, "no draft, no empty commented row");
    assert_eq!(list[2].state, ReviewState::Commented);
    assert_eq!(list[2].body, "thoughts");
    assert_eq!(list[0].state, ReviewState::Approved);
    assert_eq!(list[1].state, ReviewState::ChangesRequested);
    assert_eq!(list[1].body, "fix it");
}

#[test]
fn gitlab_lists_its_approvals_as_approved_reviews() {
    let t = Arc::new(RecordedTransport::new().on(
        "merge_requests/7/approvals",
        200,
        r#"{"approved_by":[{"user":{"username":"ines"}}]}"#,
    ));
    let list = block_on(gitlab(t).pull_request_reviews("g/p", 7)).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(
        (list[0].author.as_str(), list[0].state),
        ("ines", ReviewState::Approved)
    );
}
