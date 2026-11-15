//! The conversation endpoints of `mock-host`: what the item's timeline, the
//! post / reply / edit / delete of a comment and the token's identity read
//! and write, on four hosts.
//!
//! - GitHub side: issue `#7` and pull request `#9` of any repository (the
//!   pull request's code threads come from the review module's GraphQL).
//!   A token that starts with `ghs_` is an App installation token: `GET /user`
//!   answers 403 "Resource not accessible by integration", GraphQL `viewer`
//!   is not served, and a post comes back authored by `my-app[bot]`.
//! - GitLab side: issue `#31` of any project and the merge request `!58`
//!   of the review module (discussions are the review module's).
//! - Azure DevOps Server (`kind = "azure_devops_server"`, `url =
//!   http://127.0.0.1:PORT/tfs/Collection`): pull request `9` of repository
//!   `Proj/repo` with the threads of the hosting crate's fixture.
//!
//! Writes answer the canned success the real host would and are recorded by
//! the server's request log. The data is fictional.

use serde_json::{json, Value};

use super::http::Request;

/// The issue number on the GitHub side.
pub const GITHUB_ISSUE: u64 = 7;
/// The issue `iid` on the GitLab side.
pub const GITLAB_ISSUE: u64 = 31;
/// The pull request id on the Azure DevOps side.
pub const AZURE_PR: u64 = 9;
/// Who an App installation token posts as.
pub const BOT_LOGIN: &str = "my-app[bot]";

const AZURE_THREADS: &str = include_str!(
    "../../../crates/gitpoulpi-hosting/src/azure_devops/fixtures/conversation/threads.json"
);

type Reply = Option<(u16, String)>;

fn ok(value: Value) -> Reply {
    Some((200, value.to_string()))
}

fn created(value: Value) -> Reply {
    Some((201, value.to_string()))
}

fn no_content() -> Reply {
    Some((204, String::new()))
}

fn is_bot(request: &Request) -> bool {
    request.headers.contains("ghs_")
}

fn body_of(request: &Request, key: &str) -> String {
    serde_json::from_str::<Value>(&request.body)
        .ok()
        .and_then(|v| v[key].as_str().map(str::to_string))
        .unwrap_or_default()
}

/// Answers a conversation endpoint, or `None` for any other request.
pub fn route(request: &Request, path: &str, port: u16) -> Reply {
    if path == "/api/v3/user" && request.method == "GET" && is_bot(request) {
        return Some((
            403,
            json!({"message": "Resource not accessible by integration"}).to_string(),
        ));
    }
    if let Some(rest) = path.strip_prefix("/api/v3/repos/") {
        return github(request, rest, port);
    }
    if let Some(rest) = path.strip_prefix("/api/v4/projects/") {
        return gitlab(request, rest, port);
    }
    if path.contains("/_apis/") {
        return azure(request, path);
    }
    None
}

fn github_comment(id: u64, body: &str, login: &str, bot: bool, owner_repo: &str) -> Value {
    json!({
        "id": id, "body": body,
        "created_at": "2026-10-13T10:00:00Z", "updated_at": "2026-10-13T10:00:00Z",
        "user": {"login": login, "type": if bot { "Bot" } else { "User" }},
        "html_url": format!("http://127.0.0.1/{owner_repo}/issues/comments/{id}"),
    })
}

fn github(request: &Request, rest: &str, port: u16) -> Reply {
    let mut parts = rest.splitn(3, '/');
    let (owner, name, endpoint) = (parts.next()?, parts.next()?, parts.next()?);
    let owner_repo = format!("{owner}/{name}");
    let method = request.method.as_str();
    let item = |n: u64| {
        let pull = n == super::review::GITHUB_PR;
        json!({
            "number": n, "state": "open",
            "title": if pull { "Add moons to the planet table" } else { "Crash when the planet list is empty" },
            "body": if pull { "Adds the **moons** column, see #7." } else { "Starts with `planets()` returning nothing.\n\nRelated: #9" },
            "user": {"login": "ines", "type": "User"},
            "created_at": "2026-10-10T09:00:00Z",
            "html_url": format!("http://127.0.0.1:{port}/{owner_repo}/{}/{n}", if pull { "pull" } else { "issues" }),
            "labels": [{"name": "bug"}],
            "pull_request": if pull { json!({"merged_at": null}) } else { Value::Null },
        })
    };
    let n_of = |text: &str| text.parse::<u64>().ok();
    match (
        method,
        endpoint
            .split('?')
            .next()
            .unwrap_or_default()
            .split('/')
            .collect::<Vec<_>>()
            .as_slice(),
    ) {
        // The ISSUES list of the sidebar: the one issue this module serves.
        ("GET", ["issues"]) => {
            let mut row = item(GITHUB_ISSUE);
            row["updated_at"] = json!("2026-10-11T09:30:00Z");
            row["assignees"] = json!([]);
            ok(json!([row]))
        }
        ("GET", ["issues", n]) => {
            let n = n_of(n)?;
            (n == GITHUB_ISSUE || n == super::review::GITHUB_PR).then(|| ok(item(n)))?
        }
        ("GET", ["issues", n, "timeline"]) => {
            let n = n_of(n)?;
            if n != GITHUB_ISSUE && n != super::review::GITHUB_PR {
                return None;
            }
            ok(json!([
                {"id": 2001, "event": "commented", "actor": {"login": "tomas", "type": "User"},
                 "user": {"login": "tomas", "type": "User"}, "created_at": "2026-10-11T08:00:00Z",
                 "updated_at": "2026-10-11T08:00:00Z", "body": "I can reproduce this on #9.",
                 "html_url": "http://127.0.0.1/c/2001"},
                {"id": 2002, "event": "labeled", "actor": {"login": "ines", "type": "User"},
                 "created_at": "2026-10-11T08:30:00Z", "label": {"name": "bug"}},
                {"id": 2003, "event": "commented", "actor": {"login": "ines", "type": "User"},
                 "user": {"login": "ines", "type": "User"}, "created_at": "2026-10-11T09:00:00Z",
                 "updated_at": "2026-10-11T09:30:00Z", "body": "Thanks, fixing it.",
                 "html_url": "http://127.0.0.1/c/2003"},
            ]))
        }
        ("POST", ["issues", n, "comments"]) if n_of(n) == Some(GITHUB_ISSUE) => {
            let bot = is_bot(request);
            created(github_comment(
                2100,
                &body_of(request, "body"),
                if bot { BOT_LOGIN } else { super::GITHUB_LOGIN },
                bot,
                &owner_repo,
            ))
        }
        ("PATCH", ["issues", "comments", id]) => {
            let bot = is_bot(request);
            ok(github_comment(
                n_of(id)?,
                &body_of(request, "body"),
                if bot { BOT_LOGIN } else { super::GITHUB_LOGIN },
                bot,
                &owner_repo,
            ))
        }
        ("DELETE", ["issues", "comments", _]) => no_content(),
        _ => None,
    }
}

fn gitlab(request: &Request, rest: &str, port: u16) -> Reply {
    // A merge request's notes can be edited and deleted too.
    if let Some((_, tail)) = rest.split_once("/merge_requests/") {
        let (_, endpoint) = tail.split_once('/')?;
        return match (request.method.as_str(), endpoint) {
            ("PUT", e) if e.starts_with("notes/") => ok(json!({
                "id": 5100, "body": body_of(request, "body"), "system": false,
                "created_at": "2026-10-13T10:00:00Z", "updated_at": "2026-10-13T10:05:00Z",
                "author": {"username": super::GITLAB_LOGIN}})),
            ("DELETE", e) if e.starts_with("notes/") => no_content(),
            _ => None,
        };
    }
    // `rest` is `<encoded project>/issues/<iid>[/endpoint]`.
    let (project, tail) = rest.split_once("/issues/")?;
    let path = project.replace("%2F", "/");
    let (iid, endpoint) = tail.split_once('/').unwrap_or((tail, ""));
    if iid.parse::<u64>().ok()? != GITLAB_ISSUE {
        return None;
    }
    let note = |id: u64, body: &str| {
        json!({"id": id, "body": body, "created_at": "2026-10-13T10:00:00Z",
               "updated_at": "2026-10-13T10:00:00Z", "system": false,
               "author": {"username": super::GITLAB_LOGIN}})
    };
    match (request.method.as_str(), endpoint) {
        ("GET", "") => ok(json!({
            "iid": GITLAB_ISSUE, "title": "Crash when the planet list is empty", "state": "opened",
            "author": {"username": "ines"},
            "web_url": format!("http://127.0.0.1:{port}/{path}/-/issues/{GITLAB_ISSUE}"),
            "created_at": "2026-10-10T09:00:00Z",
            "description": issue_description(&path), "labels": ["bug"],
        })),
        ("GET", "discussions") => ok(json!([
            {"id": "d_issue1", "individual_note": false, "notes": [
                {"id": 7001, "body": "I can reproduce this.", "created_at": "2026-10-11T08:00:00Z",
                 "updated_at": "2026-10-11T08:00:00Z", "author": {"username": "tomas"},
                 "system": false, "resolvable": false, "resolved": false},
                {"id": 7002, "body": "Fix is on its way.", "created_at": "2026-10-11T09:00:00Z",
                 "updated_at": "2026-10-11T09:00:00Z", "author": {"username": "ines"},
                 "system": false, "resolvable": false, "resolved": false}]},
            {"id": "d_issue2", "individual_note": true, "notes": [
                {"id": 7003, "body": "added ~bug label", "created_at": "2026-10-11T10:00:00Z",
                 "author": {"username": "ines"}, "system": true, "resolvable": false}]},
        ])),
        ("POST", "notes") => created(note(7100, &body_of(request, "body"))),
        ("POST", e) if e.starts_with("discussions/") && e.ends_with("/notes") => {
            created(note(7101, &body_of(request, "body")))
        }
        ("PUT", e) if e.starts_with("notes/") => ok(note(7001, &body_of(request, "body"))),
        ("DELETE", e) if e.starts_with("notes/") => no_content(),
        _ => None,
    }
}

/// The issue's text: the account-wide Radar row of `northwind/ledger-ui`
/// carries a Mermaid diagram, the other projects do not.
fn issue_description(project: &str) -> String {
    let mut text = String::from("Starts with `planets()` returning nothing.");
    if project == "northwind/ledger-ui" {
        text.push_str(
            "\n\n```mermaid\ngraph TD\n  Export[Export job] --> Ledger[Ledger table]\n```\n",
        );
    }
    text
}

fn azure(request: &Request, path: &str) -> Reply {
    let (_, api) = path.split_once("/_apis/")?;
    let method = request.method.as_str();
    let segments: Vec<&str> = api.split('/').collect();
    let comment = |id: u64, content: &str| {
        json!({"id": id, "parentCommentId": 1, "content": content, "commentType": "text",
               "author": {"displayName": "Mara Q", "uniqueName": "mara@acme.test"},
               "publishedDate": "2026-10-13T10:00:00Z",
               "lastUpdatedDate": "2026-10-13T10:00:00Z"})
    };
    match (method, segments.as_slice()) {
        ("GET", ["git", "repositories", "repo"]) => {
            ok(json!({"id": "repo-guid-1", "name": "repo"}))
        }
        ("GET", ["git", "repositories", "repo-guid-1", "pullRequests", id])
            if id.parse::<u64>().ok() == Some(AZURE_PR) =>
        {
            ok(
                json!({"pullRequestId": AZURE_PR, "title": "Add retries", "status": "active",
                      "isDraft": false, "description": "Retries **three** times.",
                      "creationDate": "2026-10-01T09:00:00Z",
                      "createdBy": {"displayName": "Ann A", "uniqueName": "ann@acme.test"},
                      "labels": [{"name": "backend"}]}),
            )
        }
        ("GET", ["git", "repositories", "repo-guid-1", "pullRequests", _, "threads"]) => {
            Some((200, AZURE_THREADS.to_string()))
        }
        ("POST", ["git", "repositories", "repo-guid-1", "pullRequests", _, "threads"]) => {
            let content = serde_json::from_str::<Value>(&request.body)
                .ok()
                .and_then(|v| v["comments"][0]["content"].as_str().map(str::to_string))
                .unwrap_or_default();
            ok(json!({"id": 8, "status": "active", "comments": [comment(1, &content)]}))
        }
        (
            "POST",
            ["git", "repositories", "repo-guid-1", "pullRequests", _, "threads", thread, "comments"],
        ) => {
            let _ = thread;
            ok(comment(2, &body_of(request, "content")))
        }
        (
            "PATCH",
            ["git", "repositories", "repo-guid-1", "pullRequests", _, "threads", _, "comments", c],
        ) => ok(comment(c.parse().ok()?, &body_of(request, "content"))),
        (
            "DELETE",
            ["git", "repositories", "repo-guid-1", "pullRequests", _, "threads", _, "comments", _],
        ) => no_content(),
        _ => None,
    }
}
