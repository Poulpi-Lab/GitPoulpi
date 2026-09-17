//! A stateful GitLab-shaped host for host-driven flows (`gitpoulpi flow
//! issue ...`), with a scripted **bot**: when an issue is created (optionally
//! only with a given label) it waits `delay`, then makes the branch
//! `{type}/{iid}-{slug}` in a real bare repository (so `git fetch` finds it)
//! and a draft merge request that closes the issue. Whatever project path a
//! request names is the same project; the bare repository is its code.
//!
//! Endpoints (under `/api/v4`): `GET /user`, `POST projects/:p/issues`,
//! `GET projects/:p/issues/:iid`, `.../issues/:iid/related_merge_requests`,
//! `.../issues/:iid/closed_by`, `GET|POST projects/:p/repository/branches`,
//! `GET|POST projects/:p/merge_requests`. The library half is for the
//! workspace's tests; `scripts/fake-gitlab-bot.sh` is the same bot against a
//! real GitLab.

use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{json, Value};

use super::http;

/// What `start` needs.
#[derive(Debug, Clone)]
pub struct FlowHostOptions {
    /// The bare repository the host serves as the project's code.
    pub bare: PathBuf,
    /// The bot makes its branch this long after the issue was created;
    /// `None`: no bot.
    pub bot_delay: Option<Duration>,
    /// The bot only reacts to issues carrying this label (`None`: any).
    pub bot_label: Option<String>,
    /// The branch the bot starts from and its merge request targets.
    pub bot_target: String,
    /// The bot also opens the merge request.
    pub bot_opens_request: bool,
}

impl FlowHostOptions {
    pub fn new(bare: impl Into<PathBuf>) -> Self {
        Self {
            bare: bare.into(),
            bot_delay: Some(Duration::from_millis(300)),
            bot_label: None,
            bot_target: "main".to_string(),
            bot_opens_request: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Issue {
    pub iid: u64,
    pub title: String,
    pub description: String,
    pub labels: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MergeRequest {
    pub iid: u64,
    pub title: String,
    pub description: String,
    pub source: String,
    pub target: String,
}

#[derive(Default)]
struct State {
    issues: Vec<Issue>,
    requests: Vec<MergeRequest>,
}

/// A started flow host.
pub struct FlowHost {
    pub port: u16,
    pub seen: Arc<Mutex<Vec<http::Request>>>,
    state: Arc<Mutex<State>>,
}

impl FlowHost {
    pub fn requests(&self) -> Vec<http::Request> {
        self.seen.lock().unwrap().clone()
    }

    pub fn issues(&self) -> Vec<Issue> {
        self.state.lock().unwrap().issues.clone()
    }

    pub fn merge_requests(&self) -> Vec<MergeRequest> {
        self.state.lock().unwrap().requests.clone()
    }

    /// Bodies of the requests with this method whose target contains `needle`.
    pub fn bodies(&self, method: &str, needle: &str) -> Vec<Value> {
        self.requests()
            .iter()
            .filter(|r| r.method == method && r.target.contains(needle))
            .map(|r| serde_json::from_str(&r.body).unwrap_or(Value::Null))
            .collect()
    }
}

fn run_git(bare: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(bare)
        .env(
            "GIT_CONFIG_GLOBAL",
            if cfg!(windows) { "NUL" } else { "/dev/null" },
        )
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .args([
            "-c",
            "user.name=Bot",
            "-c",
            "user.email=bot@example.com",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .output()
        .map_err(|e| format!("could not run git: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Err(format!(
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        ))
    }
}

/// `slug` of an issue title: the `[type] ` prefix of the title template is
/// dropped, then lowercase words joined by dashes.
pub fn slug_of(title: &str) -> String {
    let title = match title.strip_prefix('[').and_then(|t| t.split_once(']')) {
        Some((_, rest)) => rest,
        None => title,
    };
    let mut out = String::new();
    let mut dash = false;
    for c in title.chars() {
        if c.is_ascii_alphanumeric() {
            if dash && !out.is_empty() {
                out.push('-');
            }
            dash = false;
            out.push(c.to_ascii_lowercase());
        } else {
            dash = true;
        }
    }
    out
}

/// The topic type an issue is for: its `workflow::<type>` label, else
/// `feature`.
fn type_of(issue: &Issue) -> String {
    issue
        .labels
        .iter()
        .find_map(|l| l.strip_prefix("workflow::"))
        .unwrap_or("feature")
        .to_string()
}

/// The bot's work: branch + (optionally) merge request.
fn bot_work(bare: &Path, state: &Mutex<State>, issue: &Issue, options: &FlowHostOptions) {
    let branch = format!("{}/{}-{}", type_of(issue), issue.iid, slug_of(&issue.title));
    let target = &options.bot_target;
    let Ok(tip) = run_git(bare, &["rev-parse", &format!("refs/heads/{target}")]) else {
        return;
    };
    // An empty commit on top of the target: the branch has a commit of its own.
    let Ok(tree) = run_git(bare, &["rev-parse", &format!("{tip}^{{tree}}")]) else {
        return;
    };
    let message = format!("Start work on #{}", issue.iid);
    let Ok(commit) = run_git(bare, &["commit-tree", &tree, "-p", &tip, "-m", &message]) else {
        return;
    };
    let _ = run_git(
        bare,
        &["update-ref", &format!("refs/heads/{branch}"), &commit],
    );
    if options.bot_opens_request {
        let mut s = state.lock().unwrap();
        let iid = s.requests.len() as u64 + 1;
        s.requests.push(MergeRequest {
            iid,
            title: format!("Draft: Resolve \"{}\"", issue.title),
            description: format!("Closes #{}", issue.iid),
            source: branch,
            target: target.clone(),
        });
    }
}

fn user() -> Value {
    json!({"id": 1, "username": "mara", "name": "Mara", "web_url": "http://localhost/mara"})
}

fn issue_json(i: &Issue, port: u16) -> Value {
    json!({
        "iid": i.iid, "id": 1000 + i.iid, "title": i.title, "description": i.description,
        "state": "opened", "labels": i.labels, "assignees": [], "author": {"username": "mara"},
        "web_url": format!("http://127.0.0.1:{port}/o/p/-/issues/{}", i.iid),
        "updated_at": "2026-10-16T10:00:00Z", "created_at": "2026-10-16T10:00:00Z",
    })
}

fn request_json(m: &MergeRequest, port: u16) -> Value {
    json!({
        "iid": m.iid, "id": 2000 + m.iid, "title": m.title, "description": m.description,
        "state": "opened", "draft": m.title.starts_with("Draft:"),
        "source_branch": m.source, "target_branch": m.target,
        "source_project_id": 1, "target_project_id": 1,
        "author": {"username": "bot"}, "assignees": [], "reviewers": [],
        "web_url": format!("http://127.0.0.1:{port}/o/p/-/merge_requests/{}", m.iid),
        "updated_at": "2026-10-16T10:00:00Z",
    })
}

fn route(
    request: &http::Request,
    port: u16,
    options: &FlowHostOptions,
    state: &Arc<Mutex<State>>,
) -> (u16, String) {
    let (path, query) = request
        .target
        .split_once('?')
        .unwrap_or((request.target.as_str(), ""));
    let Some(path) = path.strip_prefix("/api/v4") else {
        return (404, r#"{"message":"404 Not Found"}"#.into());
    };
    if path == "/user" {
        return (200, user().to_string());
    }
    // /projects/<encoded path>/<rest>
    let Some(rest) = path.strip_prefix("/projects/") else {
        return (404, r#"{"message":"404 Not Found"}"#.into());
    };
    let rest = rest.split_once('/').map(|(_, r)| r).unwrap_or("");
    let body: Value = serde_json::from_str(&request.body).unwrap_or(Value::Null);
    let text = |k: &str| body[k].as_str().unwrap_or_default().to_string();
    let method = request.method.as_str();
    let related = |iid: u64| -> String {
        let s = state.lock().unwrap();
        let needle = format!("#{iid}");
        Value::Array(
            s.requests
                .iter()
                .filter(|m| m.description.contains(&needle))
                .map(|m| request_json(m, port))
                .collect(),
        )
        .to_string()
    };
    match (method, rest) {
        ("POST", "issues") => {
            let labels: Vec<String> = text("labels")
                .split(',')
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect();
            let issue = {
                let mut s = state.lock().unwrap();
                let issue = Issue {
                    iid: s.issues.len() as u64 + 1,
                    title: text("title"),
                    description: text("description"),
                    labels,
                };
                s.issues.push(issue.clone());
                issue
            };
            let triggers = options
                .bot_label
                .as_ref()
                .map_or(true, |l| issue.labels.iter().any(|x| x == l));
            if let (Some(delay), true) = (options.bot_delay, triggers) {
                let (bare, st, opts, issue) = (
                    options.bare.clone(),
                    Arc::clone(state),
                    options.clone(),
                    issue.clone(),
                );
                std::thread::spawn(move || {
                    std::thread::sleep(delay);
                    bot_work(&bare, &st, &issue, &opts);
                });
            }
            (201, issue_json(&issue, port).to_string())
        }
        ("GET", r) if r.starts_with("issues/") => {
            let tail = &r["issues/".len()..];
            let (iid, sub) = tail.split_once('/').unwrap_or((tail, ""));
            let Ok(iid) = iid.parse::<u64>() else {
                return (404, r#"{"message":"404 Not Found"}"#.into());
            };
            match sub {
                "related_merge_requests" | "closed_by" => (200, related(iid)),
                "" => match state.lock().unwrap().issues.iter().find(|i| i.iid == iid) {
                    Some(i) => (200, issue_json(i, port).to_string()),
                    None => (404, r#"{"message":"404 Not found"}"#.into()),
                },
                _ => (404, r#"{"message":"404 Not Found"}"#.into()),
            }
        }
        ("GET", "repository/branches") => {
            let search = query
                .split('&')
                .find_map(|kv| kv.strip_prefix("search="))
                .map(percent_decode)
                .unwrap_or_default();
            let names = run_git(
                &options.bare,
                &["for-each-ref", "--format=%(refname:short)", "refs/heads"],
            )
            .unwrap_or_default();
            let rows: Vec<Value> = names
                .lines()
                .filter(|n| n.contains(&search))
                .map(|n| json!({"name": n, "web_url": format!("http://127.0.0.1:{port}/o/p/-/tree/{n}")}))
                .collect();
            (200, Value::Array(rows).to_string())
        }
        ("POST", "repository/branches") => {
            let (branch, from) = (text("branch"), text("ref"));
            match run_git(&options.bare, &["branch", &branch, &from]) {
                Ok(_) => (
                    201,
                    json!({"name": branch, "web_url": format!("http://127.0.0.1:{port}/o/p/-/tree/{branch}")})
                        .to_string(),
                ),
                Err(e) => (400, json!({"message": e}).to_string()),
            }
        }
        ("GET", "merge_requests") => {
            let s = state.lock().unwrap();
            (
                200,
                Value::Array(s.requests.iter().map(|m| request_json(m, port)).collect())
                    .to_string(),
            )
        }
        ("POST", "merge_requests") => {
            let mut s = state.lock().unwrap();
            let m = MergeRequest {
                iid: s.requests.len() as u64 + 1,
                title: text("title"),
                description: text("description"),
                source: text("source_branch"),
                target: text("target_branch"),
            };
            s.requests.push(m.clone());
            (201, request_json(&m, port).to_string())
        }
        _ => (404, r#"{"message":"404 Not Found"}"#.into()),
    }
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
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

/// Binds and serves in background threads.
pub fn start(options: FlowHostOptions) -> std::io::Result<FlowHost> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    let port = listener.local_addr()?.port();
    let state: Arc<Mutex<State>> = Arc::default();
    let handler_state = Arc::clone(&state);
    let seen = http::serve(listener, move |request| {
        route(request, port, &options, &handler_state)
    });
    Ok(FlowHost { port, seen, state })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_drop_the_type_prefix() {
        assert_eq!(slug_of("[feature] Login page!"), "login-page");
        assert_eq!(slug_of("Plain title"), "plain-title");
    }

    #[test]
    fn percent_decoding() {
        assert_eq!(percent_decode("feature%2F7"), "feature/7");
        assert_eq!(percent_decode("a%2"), "a%2");
    }
}
