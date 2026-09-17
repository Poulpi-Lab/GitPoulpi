//! The mock host's metadata half (slice M): who can be assigned, the
//! project's labels and milestones, and the edit calls. Edits are kept in
//! memory, so a list fetched afterwards shows them, exactly what a real
//! host does and what the PTY test for the picker relies on.
//!
//! Both sides answer: GitLab (`/api/v4/projects/<path>/...`: `members/all`,
//! `labels`, `milestones`, `issues/<iid>` and `merge_requests/<iid>` with
//! `PUT`), and GitHub as an Enterprise Server (`/api/v3/repos/<o>/<r>/...`:
//! `assignees`, `labels`, `milestones`, `issues/<n>` with `PATCH`,
//! `pulls/<n>`, and `pulls/<n>/requested_reviewers` with `POST` / `DELETE`).

use std::collections::HashMap;
use std::sync::Mutex;

use serde_json::{json, Value};

use super::http::Request;

/// `(id, username, name)`: GitLab's candidates.
pub const GITLAB_MEMBERS: &[(i64, &str, &str)] = &[
    (9, "mara", "Mara"),
    (21, "tomas", "Tomas"),
    (22, "ines", "Ines"),
    (23, "lena", "Lena Hart"),
];

/// `(login, id)`: GitHub's candidates.
pub const GITHUB_MEMBERS: &[(&str, i64)] = &[("mara-quinn", 1), ("theo-park", 2), ("ivy-ng", 3)];

/// `(name, colour, description)`, both sides.
pub const LABELS: &[(&str, &str, &str)] = &[
    ("bug", "d9534f", "Something is broken"),
    ("data", "428bca", "Reference data"),
    ("good first issue", "5cb85c", "A gentle start"),
    ("needs-design", "a295d6", "Waiting for a mock-up"),
    ("docs", "f0ad4e", "Documentation"),
];

/// `(id, title, due date)`, both sides.
pub const MILESTONES: &[(u64, &str, Option<&str>)] =
    &[(301, "v1.0", Some("2026-12-01")), (302, "v1.1", None)];

/// What an edit changed on one item. `None`: left as the fixture has it.
#[derive(Default)]
struct Edited {
    assignees: Option<Vec<String>>,
    reviewers: Option<Vec<String>>,
    labels: Option<Vec<String>>,
    milestone: Option<Option<u64>>,
    /// `Some(true)`: closed by `state_event=close` (GitLab) or `state`
    /// (GitHub); `Some(false)`: reopened.
    closed: Option<bool>,
}

#[derive(Default)]
pub struct Meta {
    edits: Mutex<HashMap<String, Edited>>,
}

fn milestone_json(id: u64) -> Value {
    MILESTONES
        .iter()
        .find(|m| m.0 == id)
        .map(|(id, title, due)| {
            json!({"id": id, "number": id, "title": title, "due_date": due,
            "due_on": due.map(|d| format!("{d}T00:00:00Z")), "state": "active"})
        })
        .unwrap_or(Value::Null)
}

fn list(rows: Vec<Value>) -> (u16, String) {
    (200, Value::Array(rows).to_string())
}

/// The path segments after `prefix`, when `path` starts with it.
fn rest<'a>(path: &'a str, prefix: &str) -> Option<Vec<&'a str>> {
    Some(path.strip_prefix(prefix)?.split('/').collect())
}

fn query_param(query: &str, key: &str) -> String {
    query
        .split('&')
        .find_map(|kv| kv.strip_prefix(key)?.strip_prefix('='))
        .unwrap_or("")
        .to_lowercase()
}

impl Meta {
    /// Answers the metadata routes; `None` for anything else.
    pub fn handle(
        &self,
        request: &Request,
        list_rows: &dyn Fn(&str) -> Option<Value>,
    ) -> Option<(u16, String)> {
        let (path, query) = request
            .target
            .split_once('?')
            .unwrap_or((request.target.as_str(), ""));
        if let Some(seg) = rest(path, "/api/v4/projects/") {
            return self.gitlab(request, path, query, &seg, list_rows);
        }
        if let Some(seg) = rest(path, "/api/v3/repos/") {
            return self.github(request, path, query, &seg);
        }
        None
    }

    // -- GitLab -------------------------------------------------------------

    fn gitlab(
        &self,
        request: &Request,
        path: &str,
        query: &str,
        seg: &[&str],
        list_rows: &dyn Fn(&str) -> Option<Value>,
    ) -> Option<(u16, String)> {
        match (request.method.as_str(), seg) {
            ("GET", [_, "members", "all"]) => {
                let q = query_param(query, "query");
                Some(list(
                    GITLAB_MEMBERS
                        .iter()
                        .filter(|(_, user, name)| {
                            q.is_empty()
                                || user.to_lowercase().contains(&q)
                                || name.to_lowercase().contains(&q)
                        })
                        .map(|(id, user, name)| json!({"id": id, "username": user, "name": name}))
                        .collect(),
                ))
            }
            ("GET", [_, "labels"]) => Some(list(
                LABELS
                    .iter()
                    .map(|(n, c, d)| json!({"name": n, "color": format!("#{c}"), "description": d}))
                    .collect(),
            )),
            ("GET", [_, "milestones"]) => Some(list(
                MILESTONES.iter().map(|m| milestone_json(m.0)).collect(),
            )),
            ("GET", [_, kind @ ("issues" | "merge_requests"), iid]) => {
                let list_path = path.rsplit_once('/')?.0;
                let _ = kind;
                let row = list_rows(list_path)?
                    .as_array()?
                    .iter()
                    .find(|r| r["iid"].as_u64() == iid.parse::<u64>().ok())
                    .cloned()?;
                Some((200, self.decorate_row(list_path, row).to_string()))
            }
            ("PUT", [_, "issues" | "merge_requests", _]) => {
                let body: Value = serde_json::from_str(&request.body).unwrap_or(Value::Null);
                let mut edit = Edited::default();
                let names = |ids: &Value| -> Vec<String> {
                    ids.as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|id| {
                            GITLAB_MEMBERS
                                .iter()
                                .find(|m| Some(m.0) == id.as_i64())
                                .map(|m| m.1.to_string())
                        })
                        .collect()
                };
                if body.get("assignee_ids").is_some() {
                    edit.assignees = Some(names(&body["assignee_ids"]));
                }
                if body.get("reviewer_ids").is_some() {
                    edit.reviewers = Some(names(&body["reviewer_ids"]));
                }
                if let Some(labels) = body.get("labels").and_then(Value::as_str) {
                    edit.labels = Some(
                        labels
                            .split(',')
                            .filter(|l| !l.is_empty())
                            .map(str::to_string)
                            .collect(),
                    );
                }
                if let Some(id) = body.get("milestone_id").and_then(Value::as_u64) {
                    edit.milestone = Some((id != 0).then_some(id));
                }
                match body.get("state_event").and_then(Value::as_str) {
                    Some("close") => edit.closed = Some(true),
                    Some("reopen") => edit.closed = Some(false),
                    _ => {}
                }
                self.merge(path, edit);
                Some((200, "{}".into()))
            }
            _ => None,
        }
    }

    // -- GitHub -------------------------------------------------------------

    fn github(
        &self,
        request: &Request,
        path: &str,
        query: &str,
        seg: &[&str],
    ) -> Option<(u16, String)> {
        let _ = query;
        match (request.method.as_str(), seg) {
            ("GET", [_, _, "assignees"]) => Some(list(
                GITHUB_MEMBERS
                    .iter()
                    .map(|(login, id)| json!({"login": login, "id": id}))
                    .collect(),
            )),
            ("GET", [_, _, "labels"]) => Some(list(
                LABELS
                    .iter()
                    .map(|(n, c, d)| json!({"name": n, "color": c, "description": d}))
                    .collect(),
            )),
            ("GET", [_, _, "milestones"]) => Some(list(
                MILESTONES.iter().map(|m| milestone_json(m.0)).collect(),
            )),
            ("GET", [_, _, "issues" | "pulls", _]) => {
                let edits = self.edits.lock().ok()?;
                let e = edits.get(&Self::item_key(path));
                let people = |v: Option<&Vec<String>>| -> Vec<Value> {
                    v.into_iter()
                        .flatten()
                        .map(|l| json!({"login": l, "id": 0}))
                        .collect()
                };
                Some((
                    200,
                    json!({
                        "assignees": people(e.and_then(|e| e.assignees.as_ref())),
                        "requested_reviewers": people(e.and_then(|e| e.reviewers.as_ref())),
                        "labels": e.and_then(|e| e.labels.as_ref()).into_iter().flatten()
                            .map(|l| json!({"name": l})).collect::<Vec<_>>(),
                        "milestone": e.and_then(|e| e.milestone).flatten().map_or(Value::Null, milestone_json),
                    })
                    .to_string(),
                ))
            }
            ("PATCH", [_, _, "issues", _]) => {
                let body: Value = serde_json::from_str(&request.body).unwrap_or(Value::Null);
                let strings = |v: &Value| -> Vec<String> {
                    v.as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|s| s.as_str().map(str::to_string))
                        .collect()
                };
                let mut edit = Edited::default();
                if body.get("assignees").is_some() {
                    edit.assignees = Some(strings(&body["assignees"]));
                }
                if body.get("labels").is_some() {
                    edit.labels = Some(strings(&body["labels"]));
                }
                if body.get("milestone").is_some() {
                    edit.milestone = Some(body["milestone"].as_u64());
                }
                // Pull requests are issues to GitHub: one key for both.
                self.merge(
                    &format!("{}/issues/{}", path.rsplit_once("/issues/")?.0, seg[3]),
                    edit,
                );
                Some((200, "{}".into()))
            }
            ("POST" | "DELETE", [_, _, "pulls", n, "requested_reviewers"]) => {
                let body: Value = serde_json::from_str(&request.body).unwrap_or(Value::Null);
                let asked: Vec<String> = body["reviewers"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|s| s.as_str().map(str::to_string))
                    .collect();
                let key = Self::item_key(&path.replace("/requested_reviewers", ""));
                let mut edits = self.edits.lock().ok()?;
                let _ = n;
                let have = edits.entry(key).or_default();
                let mut reviewers = have.reviewers.take().unwrap_or_default();
                if request.method == "POST" {
                    reviewers.extend(asked);
                } else {
                    reviewers.retain(|r| !asked.contains(r));
                }
                have.reviewers = Some(reviewers);
                Some((201, "{}".into()))
            }
            _ => None,
        }
    }

    // -- shared -------------------------------------------------------------

    /// One key for an item, whichever of its URLs was used.
    fn item_key(path: &str) -> String {
        path.replace("/pulls/", "/issues/")
    }

    fn merge(&self, path: &str, edit: Edited) {
        let Ok(mut edits) = self.edits.lock() else {
            return;
        };
        let have = edits.entry(Self::item_key(path)).or_default();
        if edit.assignees.is_some() {
            have.assignees = edit.assignees;
        }
        if edit.reviewers.is_some() {
            have.reviewers = edit.reviewers;
        }
        if edit.labels.is_some() {
            have.labels = edit.labels;
        }
        if edit.milestone.is_some() {
            have.milestone = edit.milestone;
        }
        if edit.closed.is_some() {
            have.closed = edit.closed;
        }
    }

    /// A GitLab list row with the edits applied (`list_path` is the list's
    /// own path, `.../issues`).
    fn decorate_row(&self, list_path: &str, mut row: Value) -> Value {
        let key = format!("{list_path}/{}", row["iid"]);
        let Ok(edits) = self.edits.lock() else {
            return row;
        };
        if let Some(e) = edits.get(&key) {
            let users = |names: &Vec<String>| -> Value {
                names.iter().map(|n| json!({"username": n})).collect()
            };
            if let Some(a) = &e.assignees {
                row["assignees"] = users(a);
            }
            if let Some(r) = &e.reviewers {
                row["reviewers"] = users(r);
            }
            if let Some(l) = &e.labels {
                row["labels"] = json!(l);
            }
            if let Some(m) = e.milestone {
                row["milestone"] = m.map_or(Value::Null, milestone_json);
            }
            if let Some(closed) = e.closed {
                row["state"] = json!(if closed { "closed" } else { "opened" });
            }
        }
        row
    }

    /// GitLab's issue list: the edits applied, then the rows `?state=`
    /// asks for (`opened` by default, `closed`, `all`).
    pub fn decorate_issues(&self, list_path: &str, body: String, query: &str) -> String {
        let want = query_param(query, "state");
        let Ok(Value::Array(rows)) =
            serde_json::from_str::<Value>(&self.decorate_list(list_path, body))
        else {
            return "[]".into();
        };
        Value::Array(
            rows.into_iter()
                .filter(|r| {
                    let closed = r["state"] == "closed";
                    match want.as_str() {
                        "closed" => closed,
                        "all" => true,
                        _ => !closed,
                    }
                })
                .collect(),
        )
        .to_string()
    }

    /// A GitLab list body with the edits applied to its rows.
    pub fn decorate_list(&self, list_path: &str, body: String) -> String {
        let Ok(Value::Array(rows)) = serde_json::from_str::<Value>(&body) else {
            return body;
        };
        Value::Array(
            rows.into_iter()
                .map(|r| self.decorate_row(list_path, r))
                .collect(),
        )
        .to_string()
    }
}
