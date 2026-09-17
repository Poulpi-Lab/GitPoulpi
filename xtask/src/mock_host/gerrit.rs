//! The Gerrit flavour of the mock host (`/a/...`): connect it with
//! `kind = "gerrit"`, `url = "http://127.0.0.1:PORT"`, `account = "mara"`.
//!
//! One project (`sandbox`) with one open change, `#7`, that has **two patch
//! sets** (`refs/changes/07/7/1` and `/2`; the sha of patch set N is the
//! digit N repeated 40 times), two labels the user may vote on (`Code-Review`
//! with the full range, `Verified` with `-1..+1`), a message, a resolved and an
//! unresolved comment, and the write endpoint a review uses
//! (`POST .../revisions/<rev>/review`, answered like the real server and
//! recorded by the server's request log so a test can assert on the body),
//! `files/?base=<ps>` (the difference between two patch sets: only the
//! documentation file) and the change actions `submit` and `abandon`. The
//! change is submittable and abandonable. The code is served too (review
//! without a clone): `.../files/<path>/diff` sections for the two files,
//! `revisions/<ps>/commit`, the project's `commits/<sha>/files/` and
//! `branches/<b>/files/<path>/content` (a base64 body).
//! Every JSON answer starts with the `)]}'` guard line, as Gerrit's does.
//! The data is fictional.

use serde_json::{json, Value};

use super::http::Request;
use super::review::{base64, file_bytes, percent_decode};

/// The signed-in user.
pub const GERRIT_LOGIN: &str = "mara";
/// The project.
pub const PROJECT: &str = "sandbox";
/// The change number.
pub const CHANGE: u64 = 7;

/// The sha of patch set `n`.
pub fn sha_of(n: u32) -> String {
    n.to_string().repeat(40)
}

fn guarded(value: Value) -> Option<(u16, String)> {
    Some((200, format!(")]}}'\n{value}")))
}

fn account(id: u64, username: &str, name: &str) -> Value {
    json!({ "_account_id": id, "username": username, "name": name })
}

fn change(detail: bool) -> Value {
    let revision = |n: u32| {
        json!({
            "_number": n, "created": format!("2026-10-14 1{n}:00:00.000000000"),
            "commit": {
                "message": "Add moons to the table\n\nAdds the moons column.\n\nChange-Id: I7\n",
                "parents": [ { "commit": "b".repeat(40) } ]
            }
        })
    };
    let mut value = json!({
        "_number": CHANGE, "project": PROJECT, "branch": "main", "subject": "Add moons to the table",
        "status": "NEW", "created": "2026-10-14 10:00:00.000000000",
        "updated": "2026-10-14 12:30:00.000000000",
        "owner": account(5, "ines", "Ines"),
        "submittable": true, "insertions": 22, "deletions": 3,
        "actions": { "abandon": { "method": "POST", "enabled": true } },
        "current_revision": sha_of(2),
        "revisions": { sha_of(1): revision(1), sha_of(2): revision(2) },
        "labels": {
            "Code-Review": {
                "values": { "-2": "This shall not be submitted", "-1": "I would prefer this is not submitted as is",
                            " 0": "No score", "+1": "Looks good to me, but someone else must approve",
                            "+2": "Looks good to me, approved" },
                "all": [ { "_account_id": 9, "username": "tomas", "value": 1, "date": "2026-10-14 11:00:00.000000000" } ],
                "default_value": 0
            },
            "Verified": {
                "values": { "-1": "Fails", " 0": "No score", "+1": "Verified" },
                "all": [], "default_value": 0
            }
        },
        "reviewers": { "REVIEWER": [ account(1000000, GERRIT_LOGIN, "Mara Quinn") ] },
        "messages": [
            { "id": "m1", "author": account(5, "ines", "Ines"), "date": "2026-10-14 10:00:00.000000000",
              "message": "Uploaded patch set 1.", "_revision_number": 1 },
            { "id": "m2", "author": account(9, "tomas", "Tomas"), "date": "2026-10-14 11:00:00.000000000",
              "message": "Patch Set 1: Code-Review+1\n\nLooks promising, one question inline.", "_revision_number": 1 }
        ]
    });
    if detail {
        value["permitted_labels"] = json!({
            "Code-Review": ["-2", "-1", " 0", "+1", "+2"],
            "Verified": ["-1", " 0", "+1"]
        });
    }
    value
}

/// `.../files/<encoded path>/diff`: Gerrit's section list for the two
/// files of the change (and nothing for any other).
fn file_diff(route: &str) -> Option<(u16, String)> {
    let (_, tail) = route.split_once("/files/")?;
    let path = percent_decode(tail.trim_end_matches("/diff"));
    match path.as_str() {
        "src/planets.rs" => guarded(json!({
            "meta_b": { "name": path, "content_type": "text/x-rust", "lines": 40 },
            "content": [
                { "skip": 9 },
                { "ab": ["    let a = 1;"] },
                { "a": ["    let b = 2;"], "b": ["    let b = 3;", "    let moons = 4;"] },
                { "ab": ["    let d = 6;"] }
            ]
        })),
        "docs/moons.md" => guarded(json!({
            "meta_b": { "name": path, "content_type": "text/markdown", "lines": 2 },
            "content": [ { "b": ["# Moons", "text"] } ]
        })),
        _ => Some((404, "Not found".into())),
    }
}

/// Answers a Gerrit endpoint, or `None` for any other request.
pub(super) fn route(request: &Request, path: &str) -> Option<(u16, String)> {
    let rest = path.strip_prefix("/a/")?;
    let method = request.method.as_str();
    let id = format!("{PROJECT}~{CHANGE}");
    match (method, rest) {
        ("GET", "accounts/self") => guarded(account(1000000, GERRIT_LOGIN, "Mara Quinn")),
        ("GET", "projects/") => guarded(json!({
            PROJECT: { "id": PROJECT, "description": "Fictional sandbox", "state": "ACTIVE" }
        })),
        ("GET", "changes/") => guarded(json!([change(false)])),
        ("GET", r) if r == format!("changes/{id}") => guarded(change(true)),
        ("GET", r)
            if r.starts_with(&format!("changes/{id}/revisions/")) && r.ends_with("/files/") =>
        {
            let revision = r
                .trim_start_matches(&format!("changes/{id}/revisions/"))
                .trim_end_matches("/files/");
            // Between two patch sets: what patch set 2 added to patch set 1.
            if request.target.contains("base=1") {
                return guarded(json!({
                    "/COMMIT_MSG": { "lines_inserted": 1 },
                    "docs/moons.md": { "status": "A", "lines_inserted": 8 }
                }));
            }
            let mut files = json!({
                "/COMMIT_MSG": { "status": "A", "lines_inserted": 9 },
                "src/planets.rs": { "lines_inserted": 14, "lines_deleted": 3 }
            });
            // Patch set 2 also documents the moons.
            if revision != "1" {
                files["docs/moons.md"] = json!({ "status": "A", "lines_inserted": 8 });
            }
            guarded(files)
        }
        ("GET", r)
            if r.starts_with(&format!("changes/{id}/revisions/")) && r.ends_with("/commit") =>
        {
            guarded(json!({
                "commit": sha_of(2), "parents": [ { "commit": "b".repeat(40), "subject": "Base" } ],
                "author": { "name": "Ines", "email": "ines@example.com", "date": "2026-10-14 11:00:00.000000000" },
                "subject": "Add moons to the table",
                "message": "Add moons to the table\n\nAdds the moons column.\n\nChange-Id: I7\n"
            }))
        }
        ("GET", r)
            if r.starts_with(&format!("changes/{id}/revisions/")) && r.ends_with("/diff") =>
        {
            file_diff(r)
        }
        ("GET", r)
            if r.starts_with(&format!("projects/{PROJECT}/commits/")) && r.ends_with("/files/") =>
        {
            guarded(json!({
                "/COMMIT_MSG": { "status": "A", "lines_inserted": 9 },
                "docs/moons.md": { "status": "A", "lines_inserted": 2 }
            }))
        }
        ("GET", r)
            if r.starts_with(&format!("projects/{PROJECT}/commits/")) && r.ends_with("/diff") =>
        {
            file_diff(r)
        }
        ("GET", r)
            if r.starts_with(&format!("projects/{PROJECT}/branches/"))
                && r.ends_with("/content") =>
        {
            let (_, file) = r.split_once("/files/")?;
            let path = percent_decode(file.trim_end_matches("/content"));
            Some(match file_bytes(&path) {
                Some(bytes) => (200, base64(&bytes)),
                None => (404, "Not found: ".to_string() + &path),
            })
        }
        ("GET", r) if r == format!("changes/{id}/comments") => guarded(json!({
            "src/planets.rs": [
                { "id": "c1", "patch_set": 2, "line": 12, "message": "Why a Vec here and not a slice?",
                  "updated": "2026-10-14 11:30:00.000000000", "author": account(9, "tomas", "Tomas"),
                  "unresolved": true },
                { "id": "c2", "patch_set": 2, "line": 1, "message": "Nit: a blank line.",
                  "updated": "2026-10-14 11:40:00.000000000", "author": account(9, "tomas", "Tomas"),
                  "unresolved": false }
            ]
        })),
        ("GET", r) if r == format!("changes/{id}/drafts") => guarded(json!({})),
        ("POST", r)
            if r == format!("changes/{id}/submit") || r == format!("changes/{id}/abandon") =>
        {
            guarded(change(true))
        }
        ("POST", r)
            if r.starts_with(&format!("changes/{id}/revisions/")) && r.ends_with("/review") =>
        {
            guarded(json!({}))
        }
        _ => Some((404, "Not found".into())),
    }
}
