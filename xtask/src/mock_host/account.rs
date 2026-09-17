//! The account-wide endpoints of `mock-host` (what the Radar and `hosting
//! radar` ask once a profile turns on review requests, open requests or
//! assigned issues): GitHub's search API and organizations (under
//! `/api/v3`), GitLab's global merge request and issue lists and its groups
//! (under `/api/v4`), and Gitea's `/repos/issues/search` and organizations
//! (under `/api/v1`), Azure DevOps Server's org-level pull requests and work
//! items (`kind = "azure_devops_server"`) and Bitbucket Data Center's
//! dashboard (`/rest/api/1.0`). Gerrit's change queries are answered by
//! `gerrit.rs` (one change of the `sandbox` project). Most rows belong to the fixture repositories (so a
//! row is "cloned" when that repository is open), a few to repositories that
//! exist nowhere else (`orion-labs/telemetry`, `harborside/atlas`,
//! `tide-works/signal`) so there is something that is not cloned. The data is
//! fictional.

use serde_json::{json, Value};

use super::{hours_ago, http};

type Reply = Option<(u16, String)>;

fn ok(value: Value) -> Reply {
    Some((200, value.to_string()))
}

/// Answers an account-wide endpoint, or `None` for any other request.
pub(super) fn route(request: &http::Request, path: &str, query: &str, port: u16) -> Reply {
    let web = format!("http://127.0.0.1:{port}");
    // Azure DevOps Server (`kind = "azure_devops_server"`, the URL may carry a
    // collection) and Bitbucket Data Center: matched by their route suffix.
    if request.method == "POST" && path.ends_with("/_apis/wit/wiql") {
        return ok(json!({"workItems": [{"id": 4101}, {"id": 4087}]}));
    }
    if request.method != "GET" {
        return None;
    }
    if let Some(reply) = azure_route(path, query, &web) {
        return Some(reply);
    }
    if path.ends_with("/rest/api/1.0/dashboard/pull-requests") {
        return ok(bitbucket_dashboard(&decode(query), &web));
    }
    match path {
        "/api/v3/search/issues" => ok(github_search(&decode(query), &web)),
        "/api/v3/user/orgs" => ok(json!([
            {"login": "harbor-guild"}, {"login": "lumen-labs"}, {"login": "orion-labs"}
        ])),
        "/api/v4/merge_requests" => ok(gitlab_review(&web)),
        "/api/v4/issues" => ok(gitlab_issues(&web)),
        "/api/v4/groups" => ok(json!([{"id": 31, "full_path": "northwind"}])),
        "/api/v4/groups/31/merge_requests" => ok(gitlab_group(&web)),
        "/api/v1/repos/issues/search" => ok(gitea_search(&decode(query), &web)),
        "/api/v1/user/orgs" => ok(json!([{"username": "tide-works"}])),
        _ => None,
    }
}

fn azure_pull(web: &str, id: u64, project: &str, title: &str, vote: i32, h: u64) -> Value {
    json!({
        "pullRequestId": id, "title": title, "status": "active", "isDraft": title.starts_with("WIP"),
        "sourceRefName": "refs/heads/feature", "targetRefName": "refs/heads/main",
        "createdBy": {"displayName": "Lena Park"}, "creationDate": hours_ago(h),
        "reviewers": [{"id": "u-mara", "displayName": "Mara", "vote": vote}],
        "repository": {
            "id": format!("repo-{id}"), "name": "services",
            "project": {"name": project},
            "remoteUrl": format!("{web}/{project}/_git/services"),
        },
    })
}

/// Azure DevOps Server's org-level routes: the token's user, active pull
/// requests (`reviewerId` narrows to mine), projects, work items.
fn azure_route(path: &str, query: &str, web: &str) -> Reply {
    if path.ends_with("/_apis/connectionData") {
        return ok(json!({"authenticatedUser": {"id": "u-mara", "providerDisplayName": "Mara"}}));
    }
    if path.ends_with("/_apis/git/pullrequests") {
        let rows = if query.contains("reviewerId=u-mara") {
            vec![
                azure_pull(web, 311, "Harbor", "Cache the token lookups", 0, 4),
                azure_pull(web, 305, "Harbor", "Already approved change", 10, 30),
            ]
        } else {
            vec![
                azure_pull(web, 311, "Harbor", "Cache the token lookups", 0, 4),
                azure_pull(web, 322, "Beacon", "WIP: split the worker", 0, 11),
            ]
        };
        return ok(json!({"count": rows.len(), "value": rows}));
    }
    if path.ends_with("/_apis/projects") {
        return ok(json!({"value": [{"name": "Harbor"}, {"name": "Beacon"}]}));
    }
    if path.ends_with("/_apis/wit/workitems") {
        let item = |id: u64, title: &str, project: &str, h: u64| {
            json!({"id": id, "fields": {
                "System.Title": title, "System.State": "Active", "System.WorkItemType": "Bug",
                "System.TeamProject": project, "System.ChangedDate": hours_ago(h),
                "System.AssignedTo": {"displayName": "Mara"},
            }})
        };
        return ok(json!({"value": [
            item(4087, "Timeout on the export job", "Beacon", 20),
            item(4101, "Wrong rounding in the invoice total", "Harbor", 5),
        ]}));
    }
    None
}

fn bitbucket_pull(web: &str, id: u64, key: &str, title: &str, draft: bool, h: u64) -> Value {
    let lower = key.to_lowercase();
    json!({
        "id": id, "title": title, "state": "OPEN", "draft": draft,
        "updatedDate": 1_760_000_000_000_u64 - h * 3_600_000,
        "fromRef": {"displayId": "feature", "repository": {"slug": "ledger", "project": {"key": key}}},
        "toRef": {"displayId": "main", "repository": {
            "slug": "ledger", "project": {"key": key},
            "links": {"clone": [{"name": "http", "href": format!("{web}/scm/{lower}/ledger.git")}]},
        }},
        "author": {"user": {"name": "lena", "displayName": "Lena Park"}},
        "reviewers": [],
        "links": {"self": [{"href": format!("{web}/projects/{key}/repos/ledger/pull-requests/{id}")}]},
    })
}

/// Bitbucket Data Center's dashboard, one role at a time.
fn bitbucket_dashboard(q: &str, web: &str) -> Value {
    let rows = if q.contains("role=AUTHOR") {
        vec![bitbucket_pull(
            web,
            60,
            "FIN",
            "WIP: ledger export",
            true,
            8,
        )]
    } else {
        vec![bitbucket_pull(
            web,
            52,
            "FIN",
            "Add the audit trail",
            false,
            3,
        )]
    };
    json!({"size": rows.len(), "limit": 100, "isLastPage": true, "values": rows})
}

/// `%XX` and `+` decoded, to read a search query.
fn decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => match u8::from_str_radix(&text[i + 1..i + 3], 16) {
                Ok(b) => {
                    out.push(b);
                    i += 3;
                    continue;
                }
                Err(_) => out.push(b'%'),
            },
            b'+' => out.push(b' '),
            b => out.push(b),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn github_row(
    web: &str,
    repo: &str,
    n: u64,
    title: &str,
    author: &str,
    draft: bool,
    h: u64,
) -> Value {
    json!({
        "number": n, "title": title, "state": "open", "draft": draft,
        "html_url": format!("{web}/{repo}/pull/{n}"),
        "updated_at": hours_ago(h),
        "repository_url": format!("{web}/api/v3/repos/{repo}"),
        "user": {"login": author}, "assignees": [], "labels": [], "milestone": null,
    })
}

fn github_search(q: &str, web: &str) -> Value {
    let items = if q.contains("review-requested:@me") {
        vec![
            github_row(
                web,
                "harbor-guild/harbor-ui",
                41,
                "Harden the token build",
                "lena",
                false,
                3,
            ),
            github_row(
                web,
                "lumen-labs/lumen-core",
                208,
                "Lower the render budget",
                "tomas",
                false,
                6,
            ),
            github_row(
                web,
                "orion-labs/telemetry",
                12,
                "Batch the metric flushes",
                "ines",
                false,
                20,
            ),
        ]
    } else if q.contains("is:issue") {
        let mut a = github_row(
            web,
            "harbor-guild/harbor-ui",
            37,
            "Focus ring clipped in the menu",
            "lena",
            false,
            5,
        );
        a["html_url"] = json!(format!("{web}/harbor-guild/harbor-ui/issues/37"));
        let mut b = github_row(
            web,
            "orion-labs/telemetry",
            5,
            "Dashboard times out on large orgs",
            "tomas",
            false,
            30,
        );
        b["html_url"] = json!(format!("{web}/orion-labs/telemetry/issues/5"));
        vec![a, b]
    } else {
        vec![
            github_row(
                web,
                "harbor-guild/harbor-tokens",
                14,
                "Rename the spacing scale",
                "mara-quinn",
                false,
                2,
            ),
            github_row(
                web,
                "orion-labs/telemetry",
                12,
                "Batch the metric flushes",
                "ines",
                false,
                20,
            ),
            github_row(
                web,
                "orion-labs/telemetry",
                15,
                "WIP sampling rework",
                "ines",
                true,
                40,
            ),
        ]
    };
    json!({"total_count": items.len(), "incomplete_results": false, "items": items})
}

fn gitlab_mr(
    web: &str,
    (project, iid): (&str, u64),
    title: &str,
    author: &str,
    draft: bool,
    reviewers: &[&str],
    h: u64,
) -> Value {
    json!({
        "iid": iid, "title": title, "state": "opened", "draft": draft,
        "source_branch": format!("feature/{iid}"), "target_branch": "main",
        "source_project_id": 1, "target_project_id": 1,
        "author": {"username": author},
        "web_url": format!("{web}/{project}/-/merge_requests/{iid}"),
        "updated_at": hours_ago(h), "assignees": [],
        "reviewers": reviewers.iter().map(|r| json!({"username": r})).collect::<Vec<_>>(),
        "references": {"full": format!("{project}!{iid}")},
    })
}

fn gitlab_review(web: &str) -> Value {
    json!([
        gitlab_mr(
            web,
            ("northwind/billing", 58),
            "Round invoice totals",
            "dario",
            false,
            &["mara"],
            4
        ),
        gitlab_mr(
            web,
            ("harborside/atlas", 9),
            "Cache the tile index",
            "noor",
            false,
            &["mara"],
            26
        ),
    ])
}

fn gitlab_group(web: &str) -> Value {
    json!([
        gitlab_mr(
            web,
            ("northwind/ledger-ui", 52),
            "Keyboard shortcuts for the grid",
            "dario",
            false,
            &[],
            8
        ),
        gitlab_mr(
            web,
            ("northwind/billing", 58),
            "Round invoice totals",
            "dario",
            false,
            &["mara"],
            4
        ),
        gitlab_mr(
            web,
            ("northwind/infra-notes", 6),
            "Draft: runbook for restores",
            "noor",
            true,
            &[],
            50
        ),
    ])
}

fn gitlab_issues(web: &str) -> Value {
    json!([{
        "iid": 31, "title": "Ledger export drops the last row",
        "web_url": format!("{web}/northwind/ledger-ui/-/issues/31"),
        "updated_at": hours_ago(10), "author": {"username": "dario"},
        "assignees": [{"username": "mara"}], "labels": ["bug"], "milestone": null,
        "references": {"full": "northwind/ledger-ui#31"},
    }])
}

fn gitea_row(web: &str, repo: &str, n: u64, title: &str, pull: bool, draft: bool, h: u64) -> Value {
    let mut row = json!({
        "number": n, "title": title,
        "html_url": format!("{web}/{repo}/{}/{n}", if pull { "pulls" } else { "issues" }),
        "updated_at": hours_ago(h), "user": {"login": "lena"}, "assignees": null,
        "labels": [], "milestone": null, "repository": {"full_name": repo},
    });
    if pull {
        row["pull_request"] = json!({"draft": draft, "merged": false});
    }
    row
}

fn gitea_search(q: &str, web: &str) -> Value {
    if q.contains("review_requested=true") {
        json!([gitea_row(
            web,
            "mara-quinn/dotfiles",
            4,
            "Switch the prompt to starship",
            true,
            false,
            7
        )])
    } else if q.contains("type=issues") {
        json!([gitea_row(
            web,
            "tide-works/signal",
            18,
            "Retry the webhook on 502",
            false,
            false,
            12
        )])
    } else if q.contains("owner=tide-works") {
        json!([
            gitea_row(
                web,
                "tide-works/signal",
                21,
                "Add a dead-letter queue",
                true,
                false,
                9
            ),
            gitea_row(
                web,
                "tide-works/signal",
                22,
                "WIP: tracing spans",
                true,
                true,
                15
            ),
        ])
    } else {
        json!([])
    }
}

#[cfg(test)]
mod tests {
    use super::super::{start, Options};
    use serde_json::Value;

    fn get(port: u16, target: &str) -> Value {
        use std::io::{Read, Write};
        let mut s = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
        write!(
            s,
            "GET {target} HTTP/1.1\r\nHost: x\r\nAuthorization: Bearer t\r\nConnection: close\r\n\r\n"
        )
        .unwrap();
        let mut out = String::new();
        s.read_to_string(&mut out).unwrap();
        assert!(out.starts_with("HTTP/1.1 200"), "{target}: {out}");
        serde_json::from_str(out.split("\r\n\r\n").nth(1).unwrap()).unwrap()
    }

    #[test]
    fn every_host_serves_its_account_wide_lists() {
        let host = start(Options::default()).unwrap();
        let p = host.port;
        let review = get(
            p,
            "/api/v3/search/issues?q=is%3Apr%20review-requested%3A%40me",
        );
        assert_eq!(review["items"].as_array().unwrap().len(), 3);
        let issues = get(p, "/api/v3/search/issues?q=is%3Aissue%20assignee%3A%40me");
        assert_eq!(issues["items"][0]["number"], 37);
        let open = get(p, "/api/v3/search/issues?q=is%3Apr%20org%3Aorion-labs");
        assert!(open["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["draft"] == true));
        assert_eq!(get(p, "/api/v3/user/orgs").as_array().unwrap().len(), 3);

        let mrs = get(p, "/api/v4/merge_requests?scope=all&reviewer_username=mara");
        assert_eq!(mrs[0]["references"]["full"], "northwind/billing!58");
        assert_eq!(get(p, "/api/v4/groups?min_access_level=10")[0]["id"], 31);
        assert_eq!(
            get(p, "/api/v4/groups/31/merge_requests?state=opened")
                .as_array()
                .unwrap()
                .len(),
            3
        );
        assert_eq!(get(p, "/api/v4/issues?scope=assigned_to_me")[0]["iid"], 31);

        let pulls = get(
            p,
            "/api/v1/repos/issues/search?type=pulls&review_requested=true",
        );
        assert_eq!(pulls[0]["repository"]["full_name"], "mara-quinn/dotfiles");
        let theirs = get(p, "/api/v1/repos/issues/search?type=pulls&owner=tide-works");
        assert_eq!(theirs.as_array().unwrap().len(), 2);
        let assigned = get(p, "/api/v1/repos/issues/search?type=issues&assigned=true");
        assert_eq!(assigned[0]["number"], 18);
        assert_eq!(get(p, "/api/v1/user/orgs")[0]["username"], "tide-works");

        let az = get(
            p,
            "/DefaultCollection/_apis/git/pullrequests?searchCriteria.reviewerId=u-mara",
        );
        assert_eq!(az["value"][0]["pullRequestId"], 311);
        assert_eq!(
            get(p, "/_apis/connectionData")["authenticatedUser"]["id"],
            "u-mara"
        );
        let items = get(p, "/_apis/wit/workitems?ids=4101,4087");
        assert_eq!(items["value"].as_array().unwrap().len(), 2);
        let dc = get(
            p,
            "/rest/api/1.0/dashboard/pull-requests?state=OPEN&role=REVIEWER",
        );
        assert_eq!(dc["values"][0]["id"], 52);
    }
}
