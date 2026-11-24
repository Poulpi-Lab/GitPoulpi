//! The writes the Radar's row actions make on a GitLab merge request
//! (`PUT .../merge_requests/:iid/merge`, `PUT .../merge_requests/:iid` with a
//! `state_event` or a `title`) and what they change in the account-wide
//! lists: a merged or closed request leaves them, a request whose title lost
//! its `Draft:` marker is no draft any more.
//!
//! State is per mock (keyed by its port) so parallel tests never see each
//! other's writes.

use std::sync::Mutex;

use serde_json::{json, Value};

use super::http::Request;

type Reply = Option<(u16, String)>;

/// `(port, iid, what)`: what = `gone` (merged or closed) or `ready`.
static DONE: Mutex<Vec<(u16, u64, &'static str)>> = Mutex::new(Vec::new());

fn done() -> std::sync::MutexGuard<'static, Vec<(u16, u64, &'static str)>> {
    DONE.lock().unwrap_or_else(|e| e.into_inner())
}

/// `<encoded project>/merge_requests/<iid>[/merge]` -> `(iid, is_merge)`.
fn parse(rest: &str) -> Option<(u64, bool)> {
    let (_, tail) = rest.split_once("/merge_requests/")?;
    let (iid, more) = match tail.split_once('/') {
        Some((iid, more)) => (iid, Some(more)),
        None => (tail, None),
    };
    let iid = iid.parse().ok()?;
    match more {
        None => Some((iid, false)),
        Some("merge") => Some((iid, true)),
        Some(_) => None,
    }
}

/// Answers the writes, and the read a "mark as ready" makes first.
pub(super) fn route(request: &Request, path: &str, port: u16) -> Reply {
    let rest = path.strip_prefix("/api/v4/projects/")?;
    let (iid, is_merge) = parse(rest)?;
    let body: Value = serde_json::from_str(&request.body).unwrap_or(Value::Null);
    match (request.method.as_str(), is_merge) {
        ("PUT", true) => {
            done().push((port, iid, "gone"));
            Some((200, json!({"iid": iid, "state": "merged"}).to_string()))
        }
        ("PUT", false) => {
            if body["state_event"] == "close" {
                done().push((port, iid, "gone"));
            } else if body["title"].is_string() {
                done().push((port, iid, "ready"));
            }
            Some((200, json!({"iid": iid}).to_string()))
        }
        // Only the draft of the account lists has a title worth reading.
        ("GET", false) if iid == 6 => Some((
            200,
            json!({"iid": iid, "title": "Draft: runbook for restores"}).to_string(),
        )),
        _ => None,
    }
}

/// An account-wide list as this mock's writes left it.
pub(super) fn apply(port: u16, list: Value) -> Value {
    let log = done();
    let Value::Array(rows) = list else {
        return list;
    };
    let kept = rows
        .into_iter()
        .filter(|row| {
            let iid = row["iid"].as_u64().unwrap_or(0);
            !log.iter()
                .any(|(p, i, what)| *p == port && *i == iid && *what == "gone")
        })
        .map(|mut row| {
            let iid = row["iid"].as_u64().unwrap_or(0);
            if log
                .iter()
                .any(|(p, i, what)| *p == port && *i == iid && *what == "ready")
            {
                row["draft"] = json!(false);
                if let Some(t) = row["title"].as_str() {
                    row["title"] = json!(t.trim_start_matches("Draft: "));
                }
            }
            row
        })
        .collect();
    Value::Array(kept)
}
