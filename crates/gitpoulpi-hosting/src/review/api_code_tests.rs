//! The code-through-the-API operations (`pull_request_diff`,
//! `pull_request_commits`, `commit_diff`, `file_content`) over
//! `RecordedTransport`. `RecordedTransport` answers the first registered
//! fragment a URL contains, so the specific endpoints come first.

use std::sync::Arc;

use gitpoulpi_core::domain::{FileStatus as Core, LineOrigin};

use super::*;
use crate::client::{client_for, HostingClient};
use crate::http::RecordedTransport;
use crate::integration::{Integration, IntegrationKind};
use crate::Error;

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}

fn github(t: Arc<RecordedTransport>) -> Box<dyn HostingClient> {
    client_for(
        Integration::new("gh", IntegrationKind::Github, "https://github.com"),
        "tok".into(),
        t,
    )
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
    )
}

fn lines(file: &ApiFileDiff) -> Vec<(LineOrigin, String)> {
    file.diff
        .hunks
        .iter()
        .flat_map(|h| h.lines.iter())
        .map(|l| (l.origin.clone(), l.content.clone()))
        .collect()
}

// ---- GitHub ----

#[test]
fn github_diff_parses_each_patch_and_marks_the_ones_the_host_left_out() {
    let body = serde_json::json!([
        {"filename": "src/a.rs", "status": "modified", "additions": 1, "deletions": 1,
         "patch": "@@ -1,2 +1,2 @@\n keep\n-old\n\\ No newline at end of file\n+new\n\\ No newline at end of file"},
        {"filename": "src/new_name.rs", "previous_filename": "src/old_name.rs", "status": "renamed",
         "additions": 1, "deletions": 0, "patch": "@@ -1 +1,2 @@\n x\r\n+y\r\n"},
        {"filename": "gone.txt", "status": "removed", "additions": 0, "deletions": 1,
         "patch": "@@ -1 +0,0 @@\n-bye"},
        {"filename": "big.json", "status": "modified", "additions": 4000, "deletions": 10},
        {"filename": "logo.png", "status": "modified", "additions": 0, "deletions": 0},
        {"filename": "moved.rs", "previous_filename": "was.rs", "status": "renamed",
         "additions": 0, "deletions": 0}
    ])
    .to_string();
    let t = Arc::new(RecordedTransport::new().on("pulls/7/files", 200, &body));
    let files = block_on(github(t.clone()).pull_request_diff("o/r", 7, None, None)).unwrap();
    assert_eq!(files.len(), 6);
    assert!(t.requests()[0].url.contains("/repos/o/r/pulls/7/files"));

    let a = &files[0];
    assert_eq!(
        (a.diff.path.to_str(), a.diff.status),
        (Some("src/a.rs"), Core::Modified)
    );
    assert_eq!(
        lines(a),
        vec![
            (LineOrigin::Context, "keep".to_string()),
            (LineOrigin::Delete, "old".to_string()),
            (LineOrigin::Add, "new".to_string())
        ],
        "the no-newline markers are not lines"
    );
    assert_eq!(a.omitted, None);

    let renamed = &files[1];
    assert_eq!(renamed.diff.status, Core::Renamed);
    assert_eq!(renamed.previous_path.as_deref(), Some("src/old_name.rs"));
    assert_eq!(
        lines(renamed),
        vec![
            (LineOrigin::Context, "x".to_string()),
            (LineOrigin::Add, "y".to_string())
        ],
        "CRLF line ends are not part of the text"
    );

    assert_eq!(files[2].diff.status, Core::Deleted);

    assert_eq!(files[3].omitted.as_deref(), Some(NOT_SHOWN_BY_THE_HOST));
    assert!(files[3].diff.hunks.is_empty());
    // A binary file and a pure rename change no line: nothing was left out.
    assert_eq!(files[4].omitted, None);
    assert_eq!(files[5].omitted, None);
    assert_eq!(files[5].diff.status, Core::Renamed);
}

#[test]
fn github_commits_come_oldest_first_with_parents_and_a_summary() {
    let t = Arc::new(RecordedTransport::new().on(
        "pulls/7/commits",
        200,
        r#"[
          {"sha":"c1","parents":[{"sha":"p0"}],"author":{"login":"ines"},
           "commit":{"message":"feat: moons\n\nlong body","author":{"name":"Ines Q","date":"2026-10-01T10:00:00Z"}}},
          {"sha":"c2","parents":[{"sha":"c1"},{"sha":"m9"}],"author":null,
           "commit":{"message":"merge","author":{"name":"Tomas","date":"2026-10-02T10:00:00Z"}}}
        ]"#,
    ));
    let commits = block_on(github(t).pull_request_commits("o/r", 7)).unwrap();
    assert_eq!(commits.len(), 2);
    assert_eq!(
        commits[0],
        ApiCommit {
            sha: "c1".into(),
            parents: vec!["p0".into()],
            author: "ines".into(),
            date: "2026-10-01T10:00:00Z".into(),
            summary: "feat: moons".into(),
        }
    );
    assert_eq!(commits[1].author, "Tomas", "no account: the commit's name");
    assert_eq!(commits[1].parents, vec!["c1", "m9"]);
}

#[test]
fn github_commit_diff_reads_the_files_of_one_commit() {
    let t = Arc::new(RecordedTransport::new().on(
        "commits/abc123",
        200,
        r#"{"sha":"abc123","files":[{"filename":"a.txt","status":"added","additions":1,"deletions":0,
            "patch":"@@ -0,0 +1 @@\n+hello"}]}"#,
    ));
    let files = block_on(github(t.clone()).commit_diff("o/r", "abc123")).unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].diff.status, Core::Added);
    assert_eq!(
        lines(&files[0]),
        vec![(LineOrigin::Add, "hello".to_string())]
    );
    assert!(t.requests()[0].url.ends_with("/repos/o/r/commits/abc123"));
}

#[test]
fn github_file_content_decodes_base64_binary_and_a_missing_file_is_none() {
    // 00 FF 41 0A: not UTF-8, and a line break inside the base64 text.
    let t = Arc::new(
        RecordedTransport::new()
            .on(
                "contents/docs/ARCH%20ITECTURE.md",
                404,
                r#"{"message":"Not Found"}"#,
            )
            .on(
                "contents/bin/blob.dat",
                200,
                r#"{"type":"file","encoding":"base64","size":4,"content":"AP9B\nCg==\n"}"#,
            ),
    );
    let client = github(t.clone());
    let bytes = block_on(client.file_content("o/r", "bin/blob.dat", Some("feature/x")))
        .unwrap()
        .unwrap();
    assert_eq!(bytes, vec![0u8, 0xFF, 0x41, 0x0A]);
    let url = t.requests()[0].url.clone();
    assert!(
        url.contains("/repos/o/r/contents/bin/blob.dat?ref=feature%2Fx"),
        "{url}"
    );

    let missing = block_on(client.file_content("o/r", "docs/ARCH ITECTURE.md", None)).unwrap();
    assert_eq!(missing, None, "404 is an absent file, not an error");
}

#[test]
fn github_file_content_refuses_a_file_the_contents_api_does_not_inline() {
    let t = Arc::new(RecordedTransport::new().on(
        "contents/big.bin",
        200,
        r#"{"type":"file","encoding":"none","size":5000000,"content":""}"#,
    ));
    let err = block_on(github(t).file_content("o/r", "big.bin", None)).unwrap_err();
    assert!(
        matches!(&err, Error::Api { detail, .. } if detail.contains("too large")),
        "{err}"
    );
}

#[test]
fn github_server_errors_on_a_file_are_errors() {
    let t = Arc::new(RecordedTransport::new().on("contents/", 500, "boom"));
    assert!(block_on(github(t).file_content("o/r", "a", None)).is_err());
}

// ---- GitLab ----

#[test]
fn gitlab_diff_without_headers_gets_its_status_from_the_row_flags() {
    let t = Arc::new(RecordedTransport::new().on(
        "merge_requests/7/diffs",
        200,
        r#"[
          {"old_path":"a.rs","new_path":"a.rs","diff":"@@ -1,2 +1,2 @@\n x\n-a\n+b\n"},
          {"old_path":"/dev/null","new_path":"n.md","new_file":true,"diff":"@@ -0,0 +1 @@\n+# N\n"},
          {"old_path":"d.md","new_path":"d.md","deleted_file":true,"diff":"@@ -1 +0,0 @@\n-# D\n"},
          {"old_path":"o.rs","new_path":"p.rs","renamed_file":true,"diff":""},
          {"old_path":"huge.json","new_path":"huge.json","diff":"","too_large":true},
          {"old_path":"lazy.json","new_path":"lazy.json","diff":"","collapsed":true}
        ]"#,
    ));
    let files = block_on(gitlab(t).pull_request_diff("g/sub/p", 7, None, None)).unwrap();
    assert_eq!(files.len(), 6);
    assert_eq!(files[0].diff.status, Core::Modified);
    assert_eq!(
        lines(&files[0]),
        vec![
            (LineOrigin::Context, "x".to_string()),
            (LineOrigin::Delete, "a".to_string()),
            (LineOrigin::Add, "b".to_string())
        ]
    );
    assert_eq!((files[0].additions, files[0].deletions), (1, 1));
    assert_eq!(files[1].diff.status, Core::Added);
    assert_eq!(files[2].diff.status, Core::Deleted);
    assert_eq!(files[2].diff.path.to_str(), Some("d.md"));
    assert_eq!(files[3].diff.status, Core::Renamed);
    assert_eq!(files[3].diff.path.to_str(), Some("p.rs"));
    assert_eq!(files[3].previous_path.as_deref(), Some("o.rs"));
    assert_eq!(files[3].omitted, None);
    assert_eq!(files[4].omitted.as_deref(), Some(NOT_SHOWN_BY_THE_HOST));
    assert_eq!(files[5].omitted.as_deref(), Some(NOT_SHOWN_BY_THE_HOST));
}

#[test]
fn gitlab_diff_falls_back_to_changes_on_an_older_instance() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("merge_requests/7/diffs", 404, "{}")
            .on(
                "merge_requests/7/changes",
                200,
                r#"{"changes":[{"old_path":"a","new_path":"a","diff":"@@ -1 +1 @@\n-x\n+y\n"}]}"#,
            ),
    );
    let files = block_on(gitlab(t).pull_request_diff("g/p", 7, None, None)).unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].diff.hunks.len(), 1);
}

#[test]
fn gitlab_commits_are_reversed_to_oldest_first() {
    let t = Arc::new(RecordedTransport::new().on(
        "merge_requests/7/commits",
        200,
        r#"[
          {"id":"c2","parent_ids":["c1"],"author_name":"Ines","authored_date":"2026-10-02T10:00:00Z","title":"second"},
          {"id":"c1","parent_ids":["p0"],"author_name":"Ines","created_at":"2026-10-01T11:00:00Z","title":"","message":"first\n\nbody"}
        ]"#,
    ));
    let commits = block_on(gitlab(t).pull_request_commits("g/p", 7)).unwrap();
    assert_eq!(
        commits.iter().map(|c| c.sha.as_str()).collect::<Vec<_>>(),
        ["c1", "c2"]
    );
    assert_eq!(commits[0].summary, "first");
    assert_eq!(commits[0].date, "2026-10-01T11:00:00Z");
    assert_eq!(commits[1].parents, vec!["c1"]);
    assert_eq!(commits[1].date, "2026-10-02T10:00:00Z");
}

#[test]
fn gitlab_commit_diff_addresses_the_project_by_its_encoded_path() {
    let t = Arc::new(RecordedTransport::new().on(
        "repository/commits/abc/diff",
        200,
        r#"[{"old_path":"a","new_path":"a","diff":"@@ -1 +1 @@\n-x\n+y\n"}]"#,
    ));
    let files = block_on(gitlab(t.clone()).commit_diff("g/sub/p", "abc")).unwrap();
    assert_eq!(files.len(), 1);
    assert!(t.requests()[0]
        .url
        .contains("/projects/g%2Fsub%2Fp/repository/commits/abc/diff"));
}

#[test]
fn gitlab_file_content_encodes_the_path_and_uses_the_default_branch_without_a_ref() {
    let t = Arc::new(
        RecordedTransport::new()
            .on(
                "repository/files/docs%2Fmissing.md",
                404,
                r#"{"message":"404 File Not Found"}"#,
            )
            .on(
                "repository/files/CLAUDE.md",
                200,
                r#"{"encoding":"base64","content":"IyBSdWxlcwo="}"#,
            )
            .on("/projects/g%2Fp", 200, r#"{"default_branch":"trunk"}"#),
    );
    let client = gitlab(t.clone());
    let bytes = block_on(client.file_content("g/p", "CLAUDE.md", None))
        .unwrap()
        .unwrap();
    assert_eq!(bytes, b"# Rules\n");
    let urls: Vec<String> = t.requests().iter().map(|r| r.url.clone()).collect();
    assert!(urls[0].ends_with("/projects/g%2Fp"), "{urls:?}");
    assert!(
        urls[1].contains("repository/files/CLAUDE.md?ref=trunk"),
        "{urls:?}"
    );

    assert_eq!(
        block_on(client.file_content("g/p", "docs/missing.md", Some("v1"))).unwrap(),
        None
    );
    assert!(t
        .requests()
        .last()
        .unwrap()
        .url
        .contains("repository/files/docs%2Fmissing.md?ref=v1"));
}

// ---- other hosts ----

#[test]
fn a_host_without_the_api_code_says_so() {
    let t = Arc::new(RecordedTransport::new());
    let client = client_for(
        Integration::new("t", IntegrationKind::Trello, "https://api.trello.com"),
        "tok".into(),
        t.clone(),
    );
    assert!(block_on(client.pull_request_diff("p", 1, None, None)).is_err());
    assert!(block_on(client.pull_request_commits("p", 1)).is_err());
    assert!(block_on(client.commit_diff("p", "abc")).is_err());
    assert!(block_on(client.file_content("p", "a", None)).is_err());
    assert!(t.requests().is_empty());
}
