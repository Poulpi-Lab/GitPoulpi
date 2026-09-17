//! The code-review endpoints of `mock-host`: one pull request on the
//! GitHub side (`#9`, any repository) and one merge request on the GitLab
//! side (`!58`, any project), each with two changed files, two threads
//! (one resolved), conversation comments and the write endpoints a review
//! uses. Writes answer with the canned success the real host would, and
//! are recorded by the server's request log so a test can assert on the
//! bodies. The data is fictional.
//!
//! The code itself is served too, for review without a clone: a `patch` per
//! file (GitHub) / a `diff` per file (GitLab), the request's commits, one
//! commit's diff, and raw files at any ref (`CLAUDE.md`, `src/planets.rs`
//! and a binary `logo.png`; anything else is a 404).

use serde_json::{json, Value};

use super::http::Request;

/// The head commit both fixtures report.
pub const HEAD_SHA: &str = "1111111111111111111111111111111111111111";
/// The merge base GitLab reports (`diff_refs.base_sha`) and GitHub's base.
pub const BASE_SHA: &str = "2222222222222222222222222222222222222222";
/// The pull request number on the GitHub side.
pub const GITHUB_PR: u64 = 9;
/// The merge request `iid` on the GitLab side.
pub const GITLAB_MR: u64 = 58;

/// The commits of the request, oldest first (GitHub's order).
pub const COMMIT_SHAS: [&str; 2] = ["aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", HEAD_SHA];

/// The bytes the mock serves at `path` (any ref), if it has such a file.
pub fn file_bytes(path: &str) -> Option<Vec<u8>> {
    match path {
        "CLAUDE.md" => Some(
            b"# Conventions\n\nPrefer small functions.\nNever use unwrap in library code.\n"
                .to_vec(),
        ),
        "src/planets.rs" => {
            Some(b"pub fn planets() -> Vec<&'static str> {\n    vec![\"Mars\"]\n}\n".to_vec())
        }
        // Not UTF-8: a PNG signature fragment.
        "logo.png" => Some(vec![
            0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0xFF,
        ]),
        _ => None,
    }
}

pub(super) const PLANETS_PATCH: &str = "@@ -10,3 +10,6 @@ fn planets()\n     let a = 1;\n-    let b = 2;\n+    let b = 3;\n+    let moons = 4;\n+    let c = 5;\n     let d = 6;\n@@ -30 +33 @@\n-    old();\n\\ No newline at end of file\n+    new();\n\\ No newline at end of file";
pub(super) const MOONS_PATCH: &str = "@@ -0,0 +1,2 @@\n+# Moons\n+text";
/// GitLab's version: hunks only, no header, the way its `/diffs` sends them.
const GITLAB_PLANETS_DIFF: &str = "@@ -10,3 +10,14 @@\n a\n+b\n+c\n-d\n";

pub(super) fn base64(bytes: &[u8]) -> String {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let n = (u32::from(chunk[0]) << 16)
            | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8)
            | u32::from(*chunk.get(2).unwrap_or(&0));
        out.push(A[(n >> 18) as usize & 63] as char);
        out.push(A[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            A[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            A[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

type Reply = Option<(u16, String)>;

fn ok(value: Value) -> Reply {
    Some((200, value.to_string()))
}

fn created(value: Value) -> Reply {
    Some((201, value.to_string()))
}

/// Answers a review endpoint, or `None` for any other request.
pub fn route(request: &Request, path: &str, port: u16) -> Reply {
    if let Some(rest) = path.strip_prefix("/api/v3/repos/") {
        return github(request, rest, port);
    }
    if path == "/api/graphql" {
        return graphql(request);
    }
    if let Some(rest) = path.strip_prefix("/api/v4/projects/") {
        return gitlab(request, rest, port);
    }
    None
}

fn github(request: &Request, rest: &str, port: u16) -> Reply {
    // `rest` is `owner/name/<endpoint>`.
    let mut parts = rest.splitn(3, '/');
    let (owner, name, endpoint) = (parts.next()?, parts.next()?, parts.next()?);
    let method = request.method.as_str();
    let pull = format!("pulls/{GITHUB_PR}");
    match (method, endpoint) {
        ("GET", e) if e == pull => ok(json!({
            "number": GITHUB_PR, "title": "Add moons to the planet table", "state": "open",
            "draft": false, "merged_at": null,
            "html_url": format!("http://127.0.0.1:{port}/{owner}/{name}/pull/{GITHUB_PR}"),
            "created_at": "2026-10-10T09:00:00Z", "updated_at": "2026-10-12T16:30:00Z",
            "body": "Adds the **moons** column and the data behind it.\n\n- sorted by planet\n- tested",
            "user": {"login": "ines"},
            "head": {"ref": "feature/moons", "sha": HEAD_SHA, "repo": {"full_name": format!("{owner}/{name}")}},
            "base": {"ref": "main", "sha": BASE_SHA, "repo": {"full_name": format!("{owner}/{name}")}},
            "mergeable": true, "mergeable_state": "clean",
            "additions": 22, "deletions": 3, "changed_files": 2,
            "assignees": [], "requested_reviewers": [{"login": "mara-quinn"}],
        })),
        ("GET", e) if e == format!("{pull}/files") => ok(json!([
            {"filename": "src/planets.rs", "status": "modified", "additions": 14, "deletions": 3,
             "patch": PLANETS_PATCH},
            {"filename": "docs/moons.md", "status": "added", "additions": 8, "deletions": 0,
             "patch": MOONS_PATCH},
        ])),
        ("GET", e) if e == format!("{pull}/commits") => ok(json!([
            {"sha": COMMIT_SHAS[0], "parents": [{"sha": BASE_SHA}], "author": {"login": "ines"},
             "commit": {"message": "feat: add the moons column\n\nFirst step.",
                        "author": {"name": "Ines", "date": "2026-10-09T09:00:00Z"}}},
            {"sha": COMMIT_SHAS[1], "parents": [{"sha": COMMIT_SHAS[0]}], "author": {"login": "ines"},
             "commit": {"message": "docs: describe the moons",
                        "author": {"name": "Ines", "date": "2026-10-10T08:00:00Z"}}},
        ])),
        ("GET", e) if e.starts_with("commits/") && !e.contains("/check-runs") => {
            let sha = e.trim_start_matches("commits/");
            if !COMMIT_SHAS.contains(&sha) {
                return None;
            }
            ok(json!({"sha": sha, "files": [
                {"filename": "docs/moons.md", "status": "added", "additions": 2, "deletions": 0,
                 "patch": MOONS_PATCH},
            ]}))
        }
        ("GET", e) if e.starts_with("contents/") => {
            Some(match file_bytes(&percent_decode(&e["contents/".len()..])) {
                Some(bytes) => (
                    200,
                    json!({"type": "file", "encoding": "base64", "size": bytes.len(),
                       "content": base64(&bytes)})
                    .to_string(),
                ),
                None => (404, json!({"message": "Not Found"}).to_string()),
            })
        }
        ("GET", e) if e.starts_with("commits/") && e.contains("/check-runs") => ok(json!({
            "check_runs": [{"status": "completed", "conclusion": "success"}]
        })),
        ("GET", e) if e == format!("issues/{GITHUB_PR}/comments") => ok(json!([
            {"id": 3001, "body": "Looks promising, one question inline.", "created_at": "2026-10-11T08:00:00Z",
             "user": {"login": "tomas"}, "html_url": "http://127.0.0.1/c/3001"},
            {"id": 3002, "body": "Thanks, answering below.", "created_at": "2026-10-11T09:00:00Z",
             "user": {"login": "ines"}, "html_url": "http://127.0.0.1/c/3002"},
        ])),
        ("POST", e) if e == format!("issues/{GITHUB_PR}/comments") => {
            let body = body_text(request);
            created(
                json!({"id": 3003, "body": body, "created_at": "2026-10-13T10:00:00Z",
                "user": {"login": super::GITHUB_LOGIN}, "html_url": "http://127.0.0.1/c/3003"}),
            )
        }
        ("GET", e) if e == format!("{pull}/reviews") => ok(json!([
            {"id": 9100, "state": "APPROVED", "body": "Looks good overall.",
             "submitted_at": "2026-10-12T09:00:00Z", "user": {"login": "mara-quinn"}},
            {"id": 9101, "state": "CHANGES_REQUESTED", "body": "Please look at the inline notes.",
             "submitted_at": "2026-10-12T10:00:00Z", "user": {"login": "tomas"}},
        ])),
        ("POST", e) if e == format!("{pull}/reviews") => {
            ok(json!({"id": 9001, "state": "COMMENTED"}))
        }
        _ => None,
    }
}

/// The GraphQL endpoint, told apart by the document it carries.
fn graphql(request: &Request) -> Reply {
    if request.method != "POST" {
        return None;
    }
    let query = serde_json::from_str::<Value>(&request.body)
        .ok()
        .and_then(|v| v["query"].as_str().map(str::to_string))
        .unwrap_or_default();
    if query.contains("reviewThreads") {
        let node =
            |id: &str, resolved: bool, path: &str, line: u32, side: &str, comments: Value| {
                json!({"id": id, "isResolved": resolved, "isOutdated": false, "path": path,
                   "line": line, "originalLine": line, "startLine": null, "originalStartLine": null,
                   "diffSide": side, "comments": {"nodes": comments}})
            };
        return ok(
            json!({"data": {"repository": {"pullRequest": {"reviewThreads": {
            "pageInfo": {"hasNextPage": false, "endCursor": null},
            "nodes": [
                node("PRRT_open", false, "src/planets.rs", 12, "RIGHT", json!([
                    {"databaseId": 4001, "author": {"login": "tomas"}, "body": "Why a Vec here and not a slice?",
                     "createdAt": "2026-10-11T08:05:00Z", "url": "http://127.0.0.1/r/4001", "replyTo": null},
                    {"databaseId": 4002, "author": {"login": "ines"}, "body": "It is built once at start-up.",
                     "createdAt": "2026-10-11T09:05:00Z", "url": "http://127.0.0.1/r/4002", "replyTo": {"databaseId": 4001}},
                ])),
                node("PRRT_done", true, "src/planets.rs", 30, "RIGHT", json!([
                    {"databaseId": 4003, "author": {"login": "tomas"}, "body": "Typo in the name.",
                     "createdAt": "2026-10-11T08:10:00Z", "url": "http://127.0.0.1/r/4003", "replyTo": null},
                ])),
            ]}}}}}),
        );
    }
    if query.contains("addPullRequestReviewThreadReply") {
        return ok(
            json!({"data": {"addPullRequestReviewThreadReply": {"comment": {
            "databaseId": 4100, "author": {"login": super::GITHUB_LOGIN}, "body": "reply",
            "createdAt": "2026-10-13T10:00:00Z", "url": "http://127.0.0.1/r/4100",
            "replyTo": {"databaseId": 4001}}}}}),
        );
    }
    if query.contains("resolveReviewThread") || query.contains("unresolveReviewThread") {
        let resolved = !query.contains("unresolveReviewThread");
        return ok(json!({"data": {"resolveReviewThread": {"thread": {"isResolved": resolved}}}}));
    }
    None
}

/// `%2F` and friends back to characters (the mock's paths are not decoded).
pub(super) fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Some(v) = std::str::from_utf8(&bytes[i + 1..i + 3])
                .ok()
                .and_then(|h| u8::from_str_radix(h, 16).ok())
            {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn body_text(request: &Request) -> String {
    serde_json::from_str::<Value>(&request.body)
        .ok()
        .and_then(|v| v["body"].as_str().map(str::to_string))
        .unwrap_or_default()
}

fn gitlab(request: &Request, rest: &str, port: u16) -> Reply {
    // The project itself (its default branch) and its raw files.
    if !rest.contains('/') && request.method == "GET" {
        return ok(
            json!({"id": 1, "default_branch": "main", "path_with_namespace": rest.replace("%2F", "/")}),
        );
    }
    if let Some((_, file)) = rest.split_once("/repository/files/") {
        return Some(match file_bytes(&percent_decode(file)) {
            Some(bytes) => (
                200,
                json!({"encoding": "base64", "size": bytes.len(), "content": base64(&bytes)})
                    .to_string(),
            ),
            None => (404, json!({"message": "404 File Not Found"}).to_string()),
        });
    }
    if let Some((_, sha)) = rest.split_once("/repository/commits/") {
        let sha = sha.strip_suffix("/diff")?;
        if !COMMIT_SHAS.contains(&sha) {
            return None;
        }
        return ok(json!([
            {"old_path": "docs/moons.md", "new_path": "docs/moons.md", "new_file": true,
             "diff": MOONS_PATCH},
        ]));
    }
    // `rest` is `<encoded project>/merge_requests/<iid>[/endpoint]`.
    let (project, tail) = rest.split_once("/merge_requests/")?;
    let (iid, endpoint) = tail.split_once('/').unwrap_or((tail, ""));
    if iid.parse::<u64>().ok()? != GITLAB_MR {
        return None;
    }
    let path = project.replace("%2F", "/");
    let method = request.method.as_str();
    let note = |id: u64, body: &str, author: &str| {
        json!({"id": id, "body": body, "created_at": "2026-10-13T10:00:00Z",
               "author": {"username": author}})
    };
    match (method, endpoint) {
        ("GET", "") => ok(json!({
            "iid": GITLAB_MR, "title": "Add moons to the planet table", "state": "opened",
            "draft": false, "source_branch": "feature/moons", "target_branch": "main",
            "source_project_id": 1, "target_project_id": 1,
            "author": {"username": "ines"},
            "web_url": format!("http://127.0.0.1:{port}/{path}/-/merge_requests/{GITLAB_MR}"),
            "created_at": "2026-10-10T09:00:00Z", "updated_at": "2026-10-12T16:30:00Z",
            "description": "Adds the **moons** column and the data behind it.",
            "assignees": [], "reviewers": [{"username": "mara"}], "sha": HEAD_SHA,
            "diff_refs": {"base_sha": BASE_SHA, "start_sha": "3333333333333333333333333333333333333333", "head_sha": HEAD_SHA},
            "detailed_merge_status": "mergeable", "has_conflicts": false,
            "head_pipeline": {"status": "success"}, "changes_count": "2",
        })),
        ("GET", "diffs") => ok(json!([
            {"old_path": "src/planets.rs", "new_path": "src/planets.rs",
             "diff": GITLAB_PLANETS_DIFF},
            {"old_path": "/dev/null", "new_path": "docs/moons.md", "new_file": true,
             "diff": MOONS_PATCH},
        ])),
        ("GET", "discussions") => ok(json!([
            {"id": "d_open", "individual_note": false, "notes": [
                {"id": 5001, "body": "Why a Vec here and not a slice?", "created_at": "2026-10-11T08:05:00Z",
                 "author": {"username": "tomas"}, "system": false, "resolvable": true, "resolved": false,
                 "type": "DiffNote",
                 "position": {"old_path": "src/planets.rs", "new_path": "src/planets.rs", "new_line": 12,
                              "old_line": null, "head_sha": HEAD_SHA}},
                {"id": 5002, "body": "It is built once at start-up.", "created_at": "2026-10-11T09:05:00Z",
                 "author": {"username": "ines"}, "system": false, "resolvable": true, "resolved": false,
                 "position": null}]},
            {"id": "d_done", "individual_note": false, "notes": [
                {"id": 5003, "body": "Typo in the name.", "created_at": "2026-10-11T08:10:00Z",
                 "author": {"username": "tomas"}, "system": false, "resolvable": true, "resolved": true,
                 "type": "DiffNote",
                 "position": {"old_path": "src/planets.rs", "new_path": "src/planets.rs", "new_line": 30,
                              "old_line": null, "head_sha": HEAD_SHA}}]},
            {"id": "d_talk", "individual_note": true, "notes": [
                {"id": 5004, "body": "Looks promising, one question inline.", "created_at": "2026-10-11T08:00:00Z",
                 "author": {"username": "tomas"}, "system": false, "resolvable": false, "resolved": false,
                 "position": null}]},
            {"id": "d_talk2", "individual_note": true, "notes": [
                {"id": 5005, "body": "Thanks, answering below.", "created_at": "2026-10-11T09:00:00Z",
                 "author": {"username": "ines"}, "system": false, "resolvable": false, "resolved": false,
                 "position": null}]},
        ])),
        ("GET", "commits") => ok(json!([
            {"id": COMMIT_SHAS[1], "parent_ids": [COMMIT_SHAS[0]], "author_name": "Ines",
             "authored_date": "2026-10-10T08:00:00Z", "title": "docs: describe the moons"},
            {"id": COMMIT_SHAS[0], "parent_ids": [BASE_SHA], "author_name": "Ines",
             "authored_date": "2026-10-09T09:00:00Z", "title": "feat: add the moons column"},
        ])),
        ("GET", "approvals") => ok(json!({"approved_by": [{"user": {"username": "mara"}}]})),
        ("POST", "notes") => created(note(5100, &body_text(request), super::GITLAB_LOGIN)),
        ("POST", e) if e.starts_with("discussions/") && e.ends_with("/notes") => {
            created(note(5101, &body_text(request), super::GITLAB_LOGIN))
        }
        ("PUT", e) if e.starts_with("discussions/") => ok(json!({"id": "d_open"})),
        ("POST", "draft_notes") => created(json!({"id": 6001})),
        ("POST", "draft_notes/bulk_publish") => ok(json!({})),
        ("POST", "approve") => created(json!({"approved": true})),
        _ => None,
    }
}
