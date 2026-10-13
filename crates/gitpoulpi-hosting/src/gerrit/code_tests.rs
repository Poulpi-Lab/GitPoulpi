//! The code of a change through Gerrit's API, over `RecordedTransport`
//! (the first registered fragment a URL contains answers).

use std::sync::Arc;

use gitpoulpi_core::domain::{FileStatus as Core, LineOrigin};

use crate::client::{client_for, HostingClient};
use crate::http::RecordedTransport;
use crate::integration::{Integration, IntegrationKind};

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
    client_for(i, "http-pass".into(), t)
}

fn gerrit(json: impl std::fmt::Display) -> String {
    format!(")]}}'\n{json}")
}

const FILES: &str = r#"{
  "/COMMIT_MSG": {"status":"A","lines_inserted":9},
  "src/a.rs": {"lines_inserted":2,"lines_deleted":1},
  "docs/new.md": {"status":"A","lines_inserted":2},
  "gone.txt": {"status":"D","lines_deleted":1},
  "new.rs": {"status":"R","old_path":"old.rs"},
  "logo.png": {"status":"A","binary":true,"size_delta":10}
}"#;

fn transport() -> Arc<RecordedTransport> {
    Arc::new(
        RecordedTransport::new()
            // The more specific fragments first.
            .on(
                "/files/src%2Fa.rs/diff",
                200,
                &gerrit(
                    r#"{"content":[{"ab":["keep"]},{"a":["old"],"b":["new","more"]},{"ab":["tail"]}]}"#,
                ),
            )
            .on(
                "/files/docs%2Fnew.md/diff",
                200,
                &gerrit(r##"{"content":[{"b":["# New","text"]}]}"##),
            )
            .on(
                "/files/gone.txt/diff",
                200,
                &gerrit(r#"{"content":[{"a":["bye"]}]}"#),
            )
            .on(
                "/files/new.rs/diff",
                200,
                &gerrit(r#"{"content":[{"ab":["same"]}]}"#),
            )
            .on(
                "/files/logo.png/diff",
                200,
                &gerrit(r#"{"binary":true,"content":[]}"#),
            )
            .on("/files/", 200, &gerrit(FILES)),
    )
}

#[test]
fn a_change_diff_lists_the_files_and_turns_each_diff_json_into_hunks() {
    let t = transport();
    let files = block_on(client(t.clone()).pull_request_diff("sandbox", 7, None, None)).unwrap();
    let paths: Vec<String> = files
        .iter()
        .map(|f| f.diff.path.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        paths,
        ["docs/new.md", "gone.txt", "logo.png", "new.rs", "src/a.rs"],
        "the magic /COMMIT_MSG file is not code"
    );
    let by = |p: &str| {
        files
            .iter()
            .find(|f| f.diff.path.to_str() == Some(p))
            .unwrap()
    };

    let a = by("src/a.rs");
    assert_eq!(a.diff.status, Core::Modified);
    assert_eq!((a.additions, a.deletions), (2, 1));
    let hunk = &a.diff.hunks[0];
    assert_eq!(
        (
            hunk.old_start,
            hunk.old_lines,
            hunk.new_start,
            hunk.new_lines
        ),
        (1, 3, 1, 4)
    );
    let shown: Vec<(LineOrigin, &str)> = hunk
        .lines
        .iter()
        .map(|l| (l.origin.clone(), l.content.as_str()))
        .collect();
    assert_eq!(
        shown,
        [
            (LineOrigin::Context, "keep"),
            (LineOrigin::Delete, "old"),
            (LineOrigin::Add, "new"),
            (LineOrigin::Add, "more"),
            (LineOrigin::Context, "tail"),
        ]
    );
    assert_eq!(by("docs/new.md").diff.status, Core::Added);
    assert_eq!(by("docs/new.md").diff.hunks[0].new_lines, 2);
    assert_eq!(by("gone.txt").diff.status, Core::Deleted);
    let renamed = by("new.rs");
    assert_eq!(renamed.diff.status, Core::Renamed);
    assert_eq!(renamed.previous_path.as_deref(), Some("old.rs"));
    let binary = by("logo.png");
    assert!(binary.diff.hunks.is_empty() && binary.omitted.is_none());

    let urls: Vec<String> = t.requests().into_iter().map(|r| r.url).collect();
    assert!(
        urls[0].contains("/changes/sandbox~7/revisions/current/files/"),
        "{urls:?}"
    );
    assert!(
        urls.iter()
            .any(|u| u.contains("/files/src%2Fa.rs/diff?context=3")),
        "{urls:?}"
    );
}

#[test]
fn a_patch_set_and_a_comparison_are_asked_of_the_server() {
    let t = transport();
    block_on(client(t.clone()).pull_request_diff("sandbox", 7, Some(2), Some(1))).unwrap();
    let urls: Vec<String> = t.requests().into_iter().map(|r| r.url).collect();
    assert!(
        urls[0].contains("/revisions/2/files/?base=1"),
        "{}",
        urls[0]
    );
    assert!(
        urls.iter()
            .skip(1)
            .all(|u| u.contains("/revisions/2/") && u.contains("base=1")),
        "{urls:?}"
    );
}

#[test]
fn the_commit_of_a_change_is_its_current_patch_sets() {
    let t = Arc::new(RecordedTransport::new().on(
        "/revisions/current/commit",
        200,
        &gerrit(
            r#"{"commit":"2222","parents":[{"commit":"bbbb","subject":"base"}],
                "author":{"name":"Ines","email":"i@x","date":"2026-10-14 10:00:00.000000000"},
                "subject":"Add moons","message":"Add moons\n\nbody"}"#,
        ),
    ));
    let commits = block_on(client(t.clone()).pull_request_commits("sandbox", 7)).unwrap();
    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].sha, "2222");
    assert_eq!(commits[0].parents, ["bbbb"]);
    assert_eq!(commits[0].author, "Ines");
    assert_eq!(commits[0].date, "2026-10-14T10:00:00Z");
    assert_eq!(commits[0].summary, "Add moons");
    assert!(t.requests()[0]
        .url
        .contains("/changes/sandbox~7/revisions/current/commit"));
}

#[test]
fn a_commit_diff_uses_the_project_commit_routes() {
    let t = transport();
    let files = block_on(client(t.clone()).commit_diff("group/sandbox", "abc1234")).unwrap();
    assert_eq!(files.len(), 5);
    assert!(
        t.requests()[0]
            .url
            .contains("/projects/group%2Fsandbox/commits/abc1234/files/"),
        "{}",
        t.requests()[0].url
    );
}

#[test]
fn a_file_is_the_base64_body_of_the_content_route() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/files/gone.txt/content", 404, "Not Found")
            .on("/files/bin%2Fblob.dat/content", 200, "AP9B\nCg==\n"),
    );
    let c = client(t.clone());
    let bytes = block_on(c.file_content("sandbox", "bin/blob.dat", Some("stable/1.0")))
        .unwrap()
        .unwrap();
    assert_eq!(bytes, vec![0u8, 0xFF, 0x41, 0x0A]);
    assert!(
        t.requests()[0]
            .url
            .contains("/projects/sandbox/branches/stable%2F1.0/files/bin%2Fblob.dat/content"),
        "{}",
        t.requests()[0].url
    );
    // Without a revision: the project's HEAD branch; a commit id: its route.
    block_on(c.file_content("sandbox", "bin/blob.dat", None)).unwrap();
    assert!(t.requests()[1].url.contains("/branches/HEAD/files/"));
    block_on(c.file_content("sandbox", "bin/blob.dat", Some("0123abcd"))).unwrap();
    assert!(t.requests()[2].url.contains("/commits/0123abcd/files/"));
    assert_eq!(
        block_on(c.file_content("sandbox", "gone.txt", None)).unwrap(),
        None
    );
}
