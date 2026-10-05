//! The code of a request through Gitea's API, over `RecordedTransport`
//! (the first registered fragment a URL contains answers).

use std::sync::Arc;

use gitpoulpi_core::domain::FileStatus as Core;

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

fn gitea(t: Arc<RecordedTransport>) -> Box<dyn HostingClient> {
    client_for(
        Integration::new("g", IntegrationKind::Gitea, "https://git.example.com/"),
        "tok".into(),
        t,
    )
}

fn forgejo(t: Arc<RecordedTransport>) -> Box<dyn HostingClient> {
    client_for(
        Integration::new("c", IntegrationKind::Forgejo, "https://codeberg.org"),
        "tok".into(),
        t,
    )
}

const PULL_DIFF: &str = "diff --git a/src/a.rs b/src/a.rs\nindex 111..222 100644\n--- a/src/a.rs\n+++ b/src/a.rs\n@@ -1,3 +1,3 @@\n keep\n-old\n+new\n tail\ndiff --git a/docs/new.md b/docs/new.md\nnew file mode 100644\nindex 000..333\n--- /dev/null\n+++ b/docs/new.md\n@@ -0,0 +1,2 @@\n+# New\n+text\ndiff --git a/gone.txt b/gone.txt\ndeleted file mode 100644\nindex 444..000\n--- a/gone.txt\n+++ /dev/null\n@@ -1 +0,0 @@\n-bye\ndiff --git a/old name.rs b/new name.rs\nsimilarity index 100%\nrename from old name.rs\nrename to new name.rs\ndiff --git a/logo.png b/logo.png\nindex 555..666 100644\nBinary files a/logo.png and b/logo.png differ\n";

#[test]
fn the_pull_diff_is_split_per_file_with_status_counts_and_renames() {
    let t = Arc::new(RecordedTransport::new().on("/pulls/3.diff", 200, PULL_DIFF));
    let files = block_on(gitea(t.clone()).pull_request_diff("o/p", 3, None, None)).unwrap();
    assert_eq!(
        t.requests()[0].url,
        "https://git.example.com/api/v1/repos/o/p/pulls/3.diff"
    );
    let summary: Vec<(String, Core, u32, u32, usize)> = files
        .iter()
        .map(|f| {
            (
                f.diff.path.to_string_lossy().into_owned(),
                f.diff.status,
                f.additions,
                f.deletions,
                f.diff.hunks.len(),
            )
        })
        .collect();
    assert_eq!(
        summary,
        vec![
            ("src/a.rs".to_string(), Core::Modified, 1, 1, 1),
            ("docs/new.md".to_string(), Core::Added, 2, 0, 1),
            ("gone.txt".to_string(), Core::Deleted, 0, 1, 1),
            ("new name.rs".to_string(), Core::Renamed, 0, 0, 0),
            ("logo.png".to_string(), Core::Modified, 0, 0, 0),
        ]
    );
    assert_eq!(files[3].previous_path.as_deref(), Some("old name.rs"));
    assert_eq!(files[0].previous_path, None);
    assert!(files.iter().all(|f| f.omitted.is_none()));
}

#[test]
fn pull_commits_come_oldest_first_whatever_order_the_server_uses() {
    let row = |sha: &str, parent: &str| {
        format!(
            r#"{{"sha":"{sha}","parents":[{{"sha":"{parent}"}}],"author":{{"login":"lena"}},
            "commit":{{"message":"{sha}\n\nbody","author":{{"name":"Lena","date":"2026-10-10T08:00:00Z"}}}}}}"#
        )
    };
    let (c1, c2) = (row("c1", "base"), row("c2", "c1"));
    for (name, body) in [
        ("oldest first", format!("[{c1},{c2}]")),
        ("newest first", format!("[{c2},{c1}]")),
    ] {
        let t = Arc::new(RecordedTransport::new().on("/pulls/3/commits", 200, &body));
        let commits = block_on(forgejo(t).pull_request_commits("o/p", 3)).unwrap();
        let shas: Vec<&str> = commits.iter().map(|c| c.sha.as_str()).collect();
        assert_eq!(shas, ["c1", "c2"], "{name}");
        assert_eq!(commits[1].parents, ["c1"]);
        assert_eq!(commits[0].author, "lena");
        assert_eq!(commits[0].date, "2026-10-10T08:00:00Z");
        assert_eq!(commits[0].summary, "c1");
    }
}

#[test]
fn a_commit_diff_reads_the_git_commit_diff_route() {
    let t = Arc::new(RecordedTransport::new().on("/git/commits/abc123.diff", 200, PULL_DIFF));
    let files = block_on(gitea(t.clone()).commit_diff("o/p", "abc123")).unwrap();
    assert_eq!(files.len(), 5);
    assert!(t.requests()[0]
        .url
        .ends_with("/repos/o/p/git/commits/abc123.diff"));
}

#[test]
fn a_file_is_read_through_the_json_contents_route_and_survives_binary() {
    let t = Arc::new(
        RecordedTransport::new()
            .on("contents/none.md", 404, r#"{"message":"not found"}"#)
            .on("contents/docs", 200, r#"[{"name":"a.md","type":"file"}]"#)
            .on(
                "contents/bin/blob.dat",
                200,
                r#"{"type":"file","encoding":"base64","size":4,"content":"AP9B\nCg=="}"#,
            ),
    );
    let client = gitea(t.clone());
    let bytes = block_on(client.file_content("o/p", "bin/blob.dat", Some("feature/x")))
        .unwrap()
        .unwrap();
    assert_eq!(bytes, vec![0u8, 0xFF, 0x41, 0x0A]);
    assert!(
        t.requests()[0]
            .url
            .ends_with("/repos/o/p/contents/bin/blob.dat?ref=feature%2Fx"),
        "{}",
        t.requests()[0].url
    );
    assert_eq!(
        block_on(client.file_content("o/p", "none.md", None)).unwrap(),
        None
    );
    assert!(block_on(client.file_content("o/p", "docs", None)).is_err());
}
