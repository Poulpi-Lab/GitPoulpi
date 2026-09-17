//! The Gitea / Forgejo flavour of the mock host (`/api/v1/...`): connect it
//! with `kind = "gitea"` (or `"forgejo"`), `url = "http://127.0.0.1:PORT"`.
//! Read-only and small on purpose: the signed-in user and repositories (the
//! GitHub side's fixtures, same owners), two pull requests and two issues
//! per repository, and who can be assigned, labels and milestones. The
//! write calls answer 404; pull request 12's code is served for review
//! without a clone (diff, commits, `contents`); the real servers are covered by the local
//! containers (the integrations-wave-2 design).

use serde_json::{json, Value};

use super::{github_repo, http, page_of, Options, GITHUB_REPOS};

/// The signed-in user of the Gitea side.
pub const GITEA_LOGIN: &str = "mara-quinn";

pub(super) fn route(
    request: &http::Request,
    path: &str,
    query: &str,
    port: u16,
    options: &Options,
) -> Option<(u16, String)> {
    let rest = path.strip_prefix("/api/v1/")?;
    if request.method != "GET" {
        return Some((404, r#"{"message":"Not Found"}"#.into()));
    }
    let web = format!("http://127.0.0.1:{port}");
    let user = |login: &str| json!({ "id": 1, "login": login, "full_name": "", "html_url": format!("{web}/{login}") });
    Some(match rest {
        "user" => (200, user(GITEA_LOGIN).to_string()),
        "user/repos" => {
            let rows = GITHUB_REPOS
                .iter()
                .map(|r| github_repo(r, port, options))
                .collect();
            (200, page_of(rows, query))
        }
        p => {
            let p = p.strip_prefix("repos/")?;
            let (project, tail) = split_project(p)?;
            let tail = tail.trim_start_matches('/');
            if let Some(reply) = code(project, tail, &web) {
                return Some(reply);
            }
            match tail {
                "pulls" => (200, pulls(project, &web)),
                "issues" => (200, search(project, &web, query)),
                t if t.starts_with("commits/") && t.ends_with("/status") => {
                    (200, commit_status(t).to_string())
                }
                "labels" => (
                    200,
                    json!([
                        {"id": 1, "name": "bug", "color": "d73a4a", "description": "Broken"},
                        {"id": 2, "name": "enhancement", "color": "a2eeef", "description": ""},
                    ])
                    .to_string(),
                ),
                "assignees" => (200, json!([user(GITEA_LOGIN), user("lena")]).to_string()),
                "milestones" => (
                    200,
                    json!([{"id": 1, "title": "v0.1", "state": "open", "due_on": "2026-12-01T12:00:00Z"}])
                        .to_string(),
                ),
                _ => return None,
            }
        }
    })
}

/// The request whose code is served (the first of the list), its commits.
pub const CODE_PR: u64 = 12;

/// The code of request [`CODE_PR`], for review without a clone: the detail,
/// the files, the whole diff, the commits, one commit's diff and the files
/// of the repository through the JSON `contents` route.
fn code(project: &str, tail: &str, web: &str) -> Option<(u16, String)> {
    use super::review::{
        base64, file_bytes, percent_decode, COMMIT_SHAS, MOONS_PATCH, PLANETS_PATCH,
    };
    let diff = format!(
        "diff --git a/src/planets.rs b/src/planets.rs\nindex 111..222 100644\n--- a/src/planets.rs\n+++ b/src/planets.rs\n{PLANETS_PATCH}\ndiff --git a/docs/moons.md b/docs/moons.md\nnew file mode 100644\nindex 000..333\n--- /dev/null\n+++ b/docs/moons.md\n{MOONS_PATCH}\n"
    );
    let ok = |value: Value| Some((200, value.to_string()));
    if tail == format!("pulls/{CODE_PR}") {
        let mut row: Value = serde_json::from_str(&pulls(project, web)).ok()?;
        let mut row = row[0].take();
        row["body"] = json!("Adds the **moons** column.");
        row["created_at"] = json!("2026-10-10T09:00:00Z");
        row["additions"] = json!(22);
        row["deletions"] = json!(3);
        row["changed_files"] = json!(2);
        return ok(row);
    }
    if tail == format!("pulls/{CODE_PR}/files") {
        return ok(json!([
            {"filename": "src/planets.rs", "status": "modified", "additions": 3, "deletions": 2},
            {"filename": "docs/moons.md", "status": "added", "additions": 2, "deletions": 0},
        ]));
    }
    if tail == format!("pulls/{CODE_PR}.diff") {
        return Some((200, diff));
    }
    if tail == format!("pulls/{CODE_PR}/commits") {
        return ok(json!([
            {"sha": COMMIT_SHAS[0], "parents": [{"sha": "b".repeat(40)}], "author": {"login": "lena"},
             "commit": {"message": "feat: add the moons column\n\nFirst step.",
                        "author": {"name": "Lena", "date": "2026-10-09T09:00:00Z"}}},
            {"sha": COMMIT_SHAS[1], "parents": [{"sha": COMMIT_SHAS[0]}], "author": {"login": "lena"},
             "commit": {"message": "docs: describe the moons",
                        "author": {"name": "Lena", "date": "2026-10-10T08:00:00Z"}}},
        ]));
    }
    if let Some(sha) = tail
        .strip_prefix("git/commits/")
        .and_then(|s| s.strip_suffix(".diff"))
    {
        if !COMMIT_SHAS.contains(&sha) {
            return Some((404, r#"{"message":"Not Found"}"#.into()));
        }
        let moons = diff
            .split_once("diff --git a/docs/")
            .map(|(_, rest)| rest)?;
        return Some((200, format!("diff --git a/docs/{moons}")));
    }
    if let Some(path) = tail.strip_prefix("contents/") {
        return Some(match file_bytes(&percent_decode(path)) {
            Some(bytes) => (
                200,
                json!({"type": "file", "encoding": "base64", "size": bytes.len(),
                       "content": base64(&bytes)})
                .to_string(),
            ),
            None => (
                404,
                json!({"message": "The target couldn't be found."}).to_string(),
            ),
        });
    }
    None
}

/// `owner/name` and what follows it.
fn split_project(path: &str) -> Option<(&str, &str)> {
    let mut parts = path.splitn(3, '/');
    let owner = parts.next()?;
    let name = parts.next()?;
    let len = owner.len() + 1 + name.len();
    Some((&path[..len], &path[len..]))
}

fn pulls(project: &str, web: &str) -> String {
    let row = |n: u64, title: &str, branch: &str, draft: bool| -> Value {
        json!({
            "number": n, "title": title, "state": "open", "draft": draft, "merged": false,
            "merged_at": null, "html_url": format!("{web}/{project}/pulls/{n}"),
            "updated_at": "2026-10-14T20:00:00Z", "user": {"login": "lena"},
            "head": {"ref": branch, "sha": format!("{n:040}"), "repo": {"full_name": project}},
            "base": {"ref": "main", "sha": "b".repeat(40), "repo": {"full_name": project}},
            "assignees": null, "requested_reviewers": [{"login": GITEA_LOGIN}], "mergeable": n != 12,
        })
    };
    json!([
        row(12, "Greet by name", "feature", false),
        row(13, "WIP: rework the log", "wip/log", true),
    ])
    .to_string()
}

/// `issues?type=pulls|issues[&mentioned_by=...]`: the search the Radar
/// uses for "mentioning me" (request 13 and issue 8 mention the user), else
/// the plain issue list.
fn search(project: &str, web: &str, query: &str) -> String {
    let param = |key: &str| {
        query
            .split('&')
            .find_map(|kv| kv.strip_prefix(&format!("{key}=")))
            .unwrap_or("")
            .to_string()
    };
    if param("type") == "pulls" {
        let rows: Vec<Value> = if param("mentioned_by").is_empty() {
            vec![json!({"number": 12}), json!({"number": 13})]
        } else {
            vec![json!({"number": 13})]
        };
        return Value::Array(rows).to_string();
    }
    let all = issues(project, web);
    if param("mentioned_by").is_empty() {
        return all;
    }
    let rows: Vec<Value> = serde_json::from_str(&all).unwrap_or_default();
    Value::Array(rows.into_iter().filter(|r| r["number"] == 8).collect()).to_string()
}

/// The combined commit status of the pull request whose head sha is
/// `{number:040}`: #12 fails, #13 passes.
fn commit_status(path: &str) -> Value {
    let sha = path
        .trim_start_matches("commits/")
        .trim_end_matches("/status");
    let state = if sha.trim_start_matches('0') == "12" {
        "failure"
    } else {
        "success"
    };
    json!({"state": state, "total_count": 1})
}

fn issues(project: &str, web: &str) -> String {
    let row = |n: u64, title: &str, labels: Value| -> Value {
        json!({
            "number": n, "title": title, "state": "open",
            "html_url": format!("{web}/{project}/issues/{n}"),
            "updated_at": "2026-10-14T19:00:00Z", "user": {"login": "lena"},
            "assignees": null, "labels": labels, "pull_request": null,
        })
    };
    json!([
        row(7, "Crash on empty name", json!([{"name": "bug"}])),
        row(8, "Greeting should be configurable", json!([])),
    ])
    .to_string()
}
