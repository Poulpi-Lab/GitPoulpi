//! `cargo run -p xtask -- mock-host [--port N] [--bare-root DIR]`: a fake
//! hosting service on 127.0.0.1, so the Clone dialog (and anything else
//! that lists a connected account's repositories) can be tested and
//! screenshotted without a network, an account or a token.
//!
//! One server answers both shapes the app talks to:
//!
//! * GitHub, as a GitHub Enterprise Server would (`/api/v3/...`): connect it
//!   with `kind = "github_enterprise"`, `url = "http://127.0.0.1:PORT"`;
//! * GitLab self-managed (`/api/v4/...`): `kind = "gitlab_self_managed"`,
//!   same URL.
//!
//! * Gitea or Forgejo (`/api/v1/...`, read-only, see [`gitea`]):
//!   `kind = "gitea"` / `"forgejo"`, same URL.
//!
//! * Gerrit (`/a/...`, see [`gerrit`]): `kind = "gerrit"`, `account = "mara"`,
//!   same URL; one project (`sandbox`) with change `#7` and its two patch sets.
//!
//! Each side has its own signed-in user and several owners (the user, then
//! organisations or groups) with a mix of public and private repositories,
//! so the Clone list shows one section per owner, per source. The data is
//! fictional. A token that contains `revoked` is answered with a 401.
//!
//! Metadata (slice M, `mock_host/meta.rs`): both sides also serve who can be
//! assigned, the labels and the milestones, and accept the edit calls
//! (GitLab `PUT`, GitHub `PATCH` and `requested_reviewers`), keeping them in
//! memory so a list fetched afterwards shows them.
//!
//! Clone URLs: with `--bare-root DIR`, every fixture repository exists as a
//! tiny bare repository at `DIR/<owner>/<name>.git` and is advertised as
//! `file://DIR/<owner>/<name>.git`, so a clone really happens (no network).
//! Without it the URLs are `http://127.0.0.1:PORT/<owner>/<name>.git`,
//! which nothing serves: fine for a screenshot of the list.
//!
//! Code review: pull request `#9` (any GitHub repository) and merge request
//! `!58` (any GitLab project) have two changed files, two threads (one
//! resolved), conversation comments, and accept comments, replies,
//! resolution and review submissions (see [`review`]).
//!
//! The library half is for the workspace's tests (`xtask::mock_host::start`,
//! and [`http::serve`], the tiny HTTP loop `pty_hosting/mock.rs` reuses).

mod account;
pub mod conversation;
pub mod flow;
pub mod gerrit;
mod gitea;
mod host_actions;
mod meta;

use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde_json::json;

pub mod review;

const USAGE: &str = "\
Usage: cargo run -p xtask -- mock-host [--port N] [--bare-root DIR]

  --port N         listen on 127.0.0.1:N (default: any free port)
  --flow-bare DIR  instead: serve a stateful GitLab-shaped host for host-driven
                   flows over the existing bare repository DIR, with a bot that
                   makes `{type}/{iid}-{slug}` and a merge request for each new
                   issue (see mock_host/flow.rs); --bot-delay-ms N (default 300)
  --bare-root DIR  also create every fixture repository as a bare repository
                   under DIR and advertise file:// clone URLs for them

Prints `PORT` on the first line, then serves until killed. Connect profiles:
  kind = \"github_enterprise\"     url = \"http://127.0.0.1:PORT\"
  kind = \"gitlab_self_managed\"   url = \"http://127.0.0.1:PORT\"
  kind = \"gitea\" or \"forgejo\"  url = \"http://127.0.0.1:PORT\"
  kind = \"gerrit\"            url = \"http://127.0.0.1:PORT\"  account = \"mara\"
(a token containing `revoked` is answered with 401).";

/// The signed-in user of the GitHub side.
pub const GITHUB_LOGIN: &str = "mara-quinn";
/// The signed-in user of the GitLab side.
pub const GITLAB_LOGIN: &str = "mara";

/// One fixture repository: `(full_name, description, private, fork)`.
type Repo = (&'static str, &'static str, bool, bool);

/// The GitHub side: the user, then three organisations (two repositories
/// each for the first two sections, so a Clone list screenshot shows two
/// sections at once).
pub const GITHUB_REPOS: &[Repo] = &[
    (
        "mara-quinn/dotfiles",
        "Shell and editor settings",
        false,
        false,
    ),
    ("mara-quinn/notes-vault", "Private notes", true, false),
    ("harbor-guild/harbor-ui", "Component library", false, false),
    ("harbor-guild/harbor-tokens", "Design tokens", false, false),
    ("lumen-labs/lumen-core", "The core engine", false, false),
    (
        "lumen-labs/lumen-cli",
        "Command line front end",
        false,
        false,
    ),
    ("lumen-labs/lumen-site", "Marketing site", true, false),
    (
        "lumen-labs/lumen-docs",
        "Documentation sources",
        false,
        false,
    ),
    (
        "lumen-labs/pebble-fork",
        "Fork of the pebble parser",
        false,
        true,
    ),
    (
        "tidepool/tidepool-agent",
        "Metrics collection agent",
        false,
        false,
    ),
    ("tidepool/tidepool-ops", "Deployment runbooks", true, false),
];

/// The GitLab side: the user, a group, a sub-group and another group.
pub const GITLAB_REPOS: &[Repo] = &[
    ("mara/scratch-pad", "Throwaway experiments", false, false),
    ("mara/kitchen-sink", "Odds and ends", true, false),
    ("northwind/billing", "Invoices and receipts", true, false),
    ("northwind/ledger-ui", "Accounting screens", true, false),
    (
        "northwind/infra-notes",
        "Infrastructure notes",
        false,
        false,
    ),
    ("quarry/tools/rockhound", "Dependency scanner", false, false),
    ("quarry/tools/chisel", "Code generator", false, false),
    ("saltmarsh/field-kit", "Survey helpers", false, false),
];

/// What `start` needs.
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// 0: any free port.
    pub port: u16,
    /// Where the bare repositories live (see the module docs); `None`: the
    /// advertised clone URLs are not served.
    pub bare_root: Option<PathBuf>,
}

/// A started mock: its port and every request it has seen.
pub struct MockHost {
    pub port: u16,
    pub seen: Arc<Mutex<Vec<http::Request>>>,
    gate: Arc<Gate>,
}

/// What [`MockHost::hold_lists`] switches: while `held`, repository list
/// requests wait (and are counted in `waiting`) until released.
#[derive(Default)]
struct Gate {
    state: Mutex<GateState>,
    cv: std::sync::Condvar,
}

#[derive(Default)]
struct GateState {
    held: bool,
    waiting: usize,
}

impl Gate {
    fn wait_if_held(&self) {
        let mut state = self.state.lock().unwrap();
        if !state.held {
            return;
        }
        state.waiting += 1;
        let mut state = self.cv.wait_while(state, |s| s.held).unwrap();
        state.waiting -= 1;
    }
}

/// Holds the repository list endpoints (`/api/v3/user/repos`,
/// `/api/v4/projects`) shut until [`ListGate::release`] or drop (a test that
/// panics never leaves the server stuck).
pub struct ListGate {
    gate: Arc<Gate>,
}

impl ListGate {
    /// How many list requests are waiting at the gate.
    pub fn waiting(&self) -> usize {
        self.gate.state.lock().unwrap().waiting
    }

    /// Lets the held requests (and later ones) through.
    pub fn release(&self) {
        self.gate.state.lock().unwrap().held = false;
        self.gate.cv.notify_all();
    }
}

impl Drop for ListGate {
    fn drop(&mut self) {
        self.release();
    }
}

impl MockHost {
    pub fn requests(&self) -> Vec<http::Request> {
        self.seen.lock().unwrap().clone()
    }

    /// Whether a request with this method and a target containing `needle`
    /// was seen.
    /// Makes the repository lists wait until the returned gate is released.
    pub fn hold_lists(&self) -> ListGate {
        self.gate.state.lock().unwrap().held = true;
        ListGate {
            gate: Arc::clone(&self.gate),
        }
    }

    pub fn saw(&self, method: &str, needle: &str) -> bool {
        self.requests()
            .iter()
            .any(|r| r.method == method && r.target.contains(needle))
    }
}

/// Binds and serves in background threads.
pub fn start(options: Options) -> std::io::Result<MockHost> {
    let listener = TcpListener::bind(("127.0.0.1", options.port))?;
    let port = listener.local_addr()?.port();
    // Edits to labels, assignees and milestones live here for the life of
    // the mock (see `meta`).
    let meta = Arc::new(meta::Meta::default());
    let gate = Arc::new(Gate::default());
    let handler_gate = Arc::clone(&gate);
    let seen = http::serve_with_headers(listener, move |request| {
        if request.method == "GET" {
            let path = request.target.split('?').next().unwrap_or_default();
            if path == "/api/v3/user/repos" || path == "/api/v4/projects" {
                handler_gate.wait_if_held();
            }
        }
        let (status, body) = route(request, port, &options, &meta);
        (status, review::extra_headers(request), body)
    });
    Ok(MockHost { port, seen, gate })
}

/// The tiny HTTP/1.1 loop under every mock of this workspace.
pub mod http {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Clone)]
    pub struct Request {
        pub method: String,
        pub target: String,
        /// The header lines, raw.
        pub headers: String,
        pub body: String,
    }

    /// Serves `listener` on background threads: `handler` maps each request
    /// to `(status, JSON body)`. Returns the log of requests, in order of
    /// arrival (logged after the handler returned).
    pub fn serve<F>(listener: TcpListener, handler: F) -> Arc<Mutex<Vec<Request>>>
    where
        F: Fn(&Request) -> (u16, String) + Send + Sync + 'static,
    {
        serve_with_headers(listener, move |request| {
            let (status, body) = handler(request);
            (status, Vec::new(), body)
        })
    }

    /// [`serve`] for a handler that also sets response headers (a paged
    /// list says how many pages there are).
    pub fn serve_with_headers<F>(listener: TcpListener, handler: F) -> Arc<Mutex<Vec<Request>>>
    where
        F: Fn(&Request) -> (u16, Vec<(&'static str, String)>, String) + Send + Sync + 'static,
    {
        let seen: Arc<Mutex<Vec<Request>>> = Arc::default();
        let log = Arc::clone(&seen);
        let handler = Arc::new(handler);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let log = Arc::clone(&log);
                let handler = Arc::clone(&handler);
                std::thread::spawn(move || {
                    let Some(request) = read_request(&mut stream) else {
                        return;
                    };
                    let (status, headers, json) = handler(&request);
                    log.lock().unwrap().push(request);
                    let extra: String = headers
                        .iter()
                        .map(|(k, v)| format!("{k}: {v}\r\n"))
                        .collect();
                    let reply = format!(
                        "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\n{extra}Content-Length: {}\r\nConnection: close\r\n\r\n{json}",
                        json.len()
                    );
                    let _ = stream.write_all(reply.as_bytes());
                    let _ = stream.flush();
                });
            }
        });
        seen
    }

    fn read_request(stream: &mut std::net::TcpStream) -> Option<Request> {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 4096];
        let header_end = loop {
            let n = stream.read(&mut chunk).unwrap_or(0);
            if n == 0 {
                return None;
            }
            buf.extend_from_slice(&chunk[..n]);
            if let Some(at) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                break at + 4;
            }
        };
        let head = String::from_utf8_lossy(&buf[..header_end]).to_string();
        let mut lines = head.lines();
        let request_line = lines.next().unwrap_or_default().to_string();
        let mut parts = request_line.split(' ');
        let method = parts.next().unwrap_or_default().to_string();
        let target = parts.next().unwrap_or_default().to_string();
        let headers = lines.collect::<Vec<_>>().join("\n");
        let length = head
            .lines()
            .find_map(|l| {
                let (k, v) = l.split_once(':')?;
                k.eq_ignore_ascii_case("content-length")
                    .then(|| v.trim().parse::<usize>().ok())?
            })
            .unwrap_or(0);
        while buf.len() < header_end + length {
            let n = stream.read(&mut chunk).unwrap_or(0);
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&chunk[..n]);
        }
        let body = String::from_utf8_lossy(&buf[header_end..]).to_string();
        Some(Request {
            method,
            target,
            headers,
            body,
        })
    }
}

/// The advertised clone URL of `full_name`.
fn clone_url(full_name: &str, port: u16, options: &Options) -> String {
    match &options.bare_root {
        Some(root) => format!(
            "file://{}/{full_name}.git",
            root.display().to_string().replace('\\', "/")
        ),
        None => format!("http://127.0.0.1:{port}/{full_name}.git"),
    }
}

fn github_repo(repo: &Repo, port: u16, options: &Options) -> serde_json::Value {
    let (full, description, private, fork) = *repo;
    let name = full.rsplit('/').next().unwrap();
    json!({
        "full_name": full,
        "name": name,
        "description": description,
        "default_branch": "main",
        "ssh_url": format!("git@127.0.0.1:{full}.git"),
        "clone_url": clone_url(full, port, options),
        "html_url": format!("http://127.0.0.1:{port}/{full}"),
        "private": private,
        "fork": fork,
    })
}

fn gitlab_project(index: usize, repo: &Repo, port: u16, options: &Options) -> serde_json::Value {
    let (full, description, private, _) = *repo;
    let name = full.rsplit('/').next().unwrap();
    json!({
        "id": 100 + index,
        "name": name,
        "path_with_namespace": full,
        "description": description,
        "default_branch": "main",
        "ssh_url_to_repo": format!("git@127.0.0.1:{full}.git"),
        "http_url_to_repo": clone_url(full, port, options),
        "web_url": format!("http://127.0.0.1:{port}/{full}"),
        "visibility": if private { "private" } else { "public" },
    })
}

/// One `per_page` / `page` slice of `rows`, as the real APIs page.
fn page_of(rows: Vec<serde_json::Value>, query: &str) -> String {
    let param = |key: &str, default: usize| -> usize {
        query
            .split('&')
            .find_map(|kv| kv.strip_prefix(key)?.strip_prefix('=')?.parse().ok())
            .unwrap_or(default)
    };
    let per_page = param("per_page", 30).max(1);
    let page = param("page", 1).max(1);
    let slice: Vec<_> = rows
        .into_iter()
        .skip((page - 1) * per_page)
        .take(per_page)
        .collect();
    serde_json::Value::Array(slice).to_string()
}

fn route(
    request: &http::Request,
    port: u16,
    options: &Options,
    meta: &meta::Meta,
) -> (u16, String) {
    let (path, query) = request
        .target
        .split_once('?')
        .unwrap_or((request.target.as_str(), ""));
    if request.headers.to_lowercase().contains("revoked") {
        return (401, r#"{"message":"401 Unauthorized"}"#.into());
    }
    // Merge, close and mark ready on a merge request.
    if let Some(reply) = host_actions::route(request, path, port) {
        return reply;
    }
    // Conversations: timelines, comments and the token's identity.
    if let Some(reply) = conversation::route(request, path, port) {
        return reply;
    }
    // Code review of one request per side (pull request #9 / merge request !58).
    if let Some(reply) = review::route(request, path, port) {
        return reply;
    }
    // Account-wide lists: search, global merge request / issue lists, groups.
    if let Some(reply) = account::route(request, path, query, port) {
        return reply;
    }
    // Members, labels, milestones and the edits (slice M).
    let rows = |list_path: &str| -> Option<serde_json::Value> {
        let body = if list_path.ends_with("/merge_requests") {
            gitlab_merge_requests(list_path, port)
        } else {
            gitlab_issues(list_path, port)
        };
        serde_json::from_str(&body).ok()
    };
    if let Some(answer) = meta.handle(request, &rows) {
        return answer;
    }
    // Gerrit (`/a/...`).
    if let Some(answer) = gerrit::route(request, path) {
        return answer;
    }
    // Gitea / Forgejo (`/api/v1`), read-only.
    if let Some(answer) = gitea::route(request, path, query, port, options) {
        return answer;
    }
    if request.method != "GET" {
        return (404, r#"{"message":"Not Found"}"#.into());
    }
    match path {
        "/api/v3/user" => (
            200,
            json!({"login": GITHUB_LOGIN, "name": "Mara Quinn"}).to_string(),
        ),
        "/api/v3/user/repos" => {
            let rows = GITHUB_REPOS
                .iter()
                .map(|r| github_repo(r, port, options))
                .collect();
            (200, page_of(rows, query))
        }
        "/api/v4/user" => (
            200,
            json!({"id": 9, "username": GITLAB_LOGIN, "name": "Mara"}).to_string(),
        ),
        "/api/v4/projects" => {
            let rows = GITLAB_REPOS
                .iter()
                .enumerate()
                .map(|(i, r)| gitlab_project(i, r, port, options))
                .collect();
            (200, page_of(rows, query))
        }
        p if p.starts_with("/api/v4/projects/") && p.ends_with("/merge_requests") => {
            (200, meta.decorate_list(p, gitlab_merge_requests(p, port)))
        }
        p if p.starts_with("/api/v4/projects/") && p.ends_with("/issues") => {
            (200, meta.decorate_issues(p, gitlab_issues(p, port), query))
        }
        "/api/v4/todos" => (200, gitlab_todos(query, port)),
        _ => (404, r#"{"message":"404 Not Found"}"#.into()),
    }
}

/// `YYYY-MM-DDTHH:MM:SSZ` for `hours` hours before now (the Radar shows
/// ages, so fixed dates would drift into "months old").
fn hours_ago(hours: u64) -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
        .saturating_sub(hours * 3600);
    let (days, rem) = ((secs / 86400) as i64, secs % 86400);
    // Howard Hinnant's civil-from-days.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

/// Which fixture project a GitLab path names: the one whose name ends in
/// `-conflict` is the second, the rest the first.
fn second_project(path: &str) -> bool {
    path.contains("conflict")
}

/// Merge requests of any project, as GitLab lists them. The signed-in user
/// is [`GITLAB_LOGIN`]: some are hers, one asks for her review.
fn gitlab_merge_requests(path: &str, port: u16) -> String {
    let rows = if second_project(path) {
        vec![
            (
                31,
                "Resolve the ring mesh merge",
                "mara",
                "fix/ring-mesh",
                &[][..],
                &[][..],
                3,
            ),
            (
                30,
                "Draft: retire the legacy export",
                "ines",
                "chore/legacy-export",
                &[][..],
                &["mara"][..],
                26,
            ),
        ]
    } else {
        vec![
            (
                58,
                "Add moons to the planet table",
                "mara",
                "feature/moons",
                &[][..],
                &[][..],
                2,
            ),
            (
                57,
                "Speed up the orbit solver",
                "tomas",
                "perf/solver",
                &[][..],
                &["mara"][..],
                7,
            ),
            (
                55,
                "Docs: how to read the planet table",
                "ines",
                "docs/planets",
                &["mara"][..],
                &[][..],
                50,
            ),
            (
                52,
                "Bump the ephemeris data",
                "tomas",
                "chore/ephemeris",
                &[][..],
                &[][..],
                120,
            ),
        ]
    };
    let list: Vec<_> = rows
        .into_iter()
        .map(|(iid, title, author, branch, assignees, reviewers, hours)| {
            let users = |names: &[&str]| -> Vec<_> {
                names.iter().map(|n| json!({"username": n})).collect()
            };
            let (pipeline, conflicts) = checks_of(iid);
            json!({
                "iid": iid, "title": title, "state": "opened",
                "draft": title.starts_with("Draft:"),
                "has_conflicts": conflicts,
                "head_pipeline": pipeline.map(|status| json!({"status": status})),
                "source_branch": branch, "target_branch": "main",
                "source_project_id": 1, "target_project_id": 1,
                "author": {"username": author},
                "web_url": format!("http://127.0.0.1:{port}/orbit-co/orbit/-/merge_requests/{iid}"),
                "updated_at": hours_ago(hours),
                "assignees": users(assignees), "reviewers": users(reviewers),
                "labels": [],
            })
        })
        .collect();
    serde_json::Value::Array(list).to_string()
}

/// What the fixture merge requests say about their pipeline and whether they
/// conflict, so a Radar screenshot shows every glyph: (pipeline status,
/// has_conflicts); `None` = the host says nothing.
fn checks_of(iid: u32) -> (Option<&'static str>, Option<bool>) {
    match iid {
        58 => (Some("success"), Some(false)),
        57 => (Some("failed"), Some(false)),
        55 => (Some("running"), Some(false)),
        52 => (Some("success"), Some(true)),
        31 => (Some("failed"), Some(true)),
        _ => (None, None),
    }
}

/// The to-do list: what mentions [`GITLAB_LOGIN`], one entry per mention,
/// with the whole item inside (what `action=mentioned&type=...` returns).
/// Only the first fixture project has any.
fn gitlab_todos(query: &str, port: u16) -> String {
    let wants = |key: &str, value: &str| query.split('&').any(|kv| kv == format!("{key}={value}"));
    if !wants("action", "mentioned") {
        return "[]".into();
    }
    let project = json!({"path_with_namespace": "o/orbit"});
    let mut list: Vec<serde_json::Value> = Vec::new();
    if wants("type", "MergeRequest") {
        let mrs: Vec<serde_json::Value> =
            serde_json::from_str(&gitlab_merge_requests("o/orbit", port)).unwrap_or_default();
        list.extend(mrs.into_iter().filter(|m| m["iid"] == 52).map(|target| {
            json!({"action": "mentioned", "target_type": "MergeRequest",
                   "project": project, "target": target})
        }));
    }
    if wants("type", "Issue") {
        let issues: Vec<serde_json::Value> =
            serde_json::from_str(&gitlab_issues("o/orbit", port)).unwrap_or_default();
        list.extend(
            issues
                .into_iter()
                .filter(|i| i["iid"] == 190)
                .map(|target| {
                    json!({"action": "mentioned", "target_type": "Issue",
                   "project": project, "target": target})
                }),
        );
    }
    serde_json::Value::Array(list).to_string()
}

/// Issues of any project, as GitLab lists them.
fn gitlab_issues(path: &str, port: u16) -> String {
    // (iid, title, author, assignees, labels, hours since update)
    type Row<'a> = (u32, &'a str, &'a str, &'a [&'a str], &'a [&'a str], u64);
    let rows: Vec<Row> = if second_project(path) {
        vec![(
            9,
            "Merge leaves a stale lock file",
            "ines",
            &["mara"],
            &["bug"],
            30,
        )]
    } else {
        vec![
            (
                214,
                "Moons of Jupiter are missing",
                "tomas",
                &["mara"],
                &["data", "good first issue"],
                9,
            ),
            (
                209,
                "Orbit chart flickers on resize",
                "mara",
                &[],
                &["bug"],
                73,
            ),
            // Not hers and not assigned to her, but it mentions her.
            (
                190,
                "Clarify the moon naming rule",
                "tomas",
                &[],
                &["docs"],
                40,
            ),
        ]
    };
    let list: Vec<_> = rows
        .into_iter()
        .map(|(iid, title, author, assignees, labels, hours)| {
            json!({
                "iid": iid, "title": title, "state": "opened",
                "web_url": format!("http://127.0.0.1:{port}/orbit-co/orbit/-/issues/{iid}"),
                "updated_at": hours_ago(hours),
                "author": {"username": author},
                "assignees": assignees.iter().map(|n| json!({"username": n})).collect::<Vec<_>>(),
                "labels": labels,
            })
        })
        .collect();
    serde_json::Value::Array(list).to_string()
}

fn git(dir: &Path, args: &[&str]) -> Result<(), String> {
    let out = std::process::Command::new("git")
        .current_dir(dir)
        .env(
            "GIT_CONFIG_GLOBAL",
            if cfg!(windows) { "NUL" } else { "/dev/null" },
        )
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .args([
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.com",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .output()
        .map_err(|e| format!("could not run git: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!(
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        ))
    }
}

/// Creates every fixture repository as a bare repository (one commit on
/// `main`) at `root/<full_name>.git`, the paths [`start`] advertises.
pub fn seed_bare_repos(root: &Path) -> Result<(), String> {
    std::fs::create_dir_all(root).map_err(|e| e.to_string())?;
    let work = root.join(".seed-work");
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&work).map_err(|e| e.to_string())?;
    git(&work, &["init", "-q", "-b", "main"])?;
    std::fs::write(work.join("README.md"), "A fixture repository.\n").map_err(|e| e.to_string())?;
    git(&work, &["add", "."])?;
    git(&work, &["commit", "-q", "-m", "Initial commit"])?;
    for (full, ..) in GITHUB_REPOS.iter().chain(GITLAB_REPOS) {
        let dest = root.join(format!("{full}.git"));
        if dest.exists() {
            continue;
        }
        std::fs::create_dir_all(dest.parent().unwrap()).map_err(|e| e.to_string())?;
        let dest_str = dest.display().to_string();
        git(&work, &["clone", "-q", "--bare", ".", &dest_str])?;
    }
    std::fs::remove_dir_all(&work).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn run(args: &[String]) -> Result<i32, String> {
    let mut options = Options::default();
    let mut flow_bare: Option<PathBuf> = None;
    let mut bot_delay_ms: u64 = 300;
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--port" => {
                options.port = it
                    .next()
                    .and_then(|v| v.parse().ok())
                    .ok_or("--port needs a number")?;
            }
            "--flow-bare" => {
                flow_bare = Some(PathBuf::from(it.next().ok_or("--flow-bare needs a DIR")?));
            }
            "--bot-delay-ms" => {
                bot_delay_ms = it
                    .next()
                    .and_then(|v| v.parse().ok())
                    .ok_or("--bot-delay-ms needs a number")?;
            }
            "--bare-root" => {
                options.bare_root =
                    Some(PathBuf::from(it.next().ok_or("--bare-root needs a DIR")?));
            }
            "-h" | "--help" => {
                println!("{USAGE}");
                return Ok(0);
            }
            other => return Err(format!("unknown option '{other}'\n\n{USAGE}")),
        }
    }
    if let Some(bare) = flow_bare {
        let mut flow = flow::FlowHostOptions::new(bare);
        flow.bot_delay = Some(std::time::Duration::from_millis(bot_delay_ms));
        let host = flow::start(flow).map_err(|e| format!("could not listen: {e}"))?;
        println!("{}", host.port);
        {
            use std::io::Write;
            let _ = std::io::stdout().flush();
        }
        loop {
            std::thread::park();
        }
    }
    if let Some(root) = &options.bare_root {
        seed_bare_repos(root)?;
    }
    let host = start(options).map_err(|e| format!("could not listen: {e}"))?;
    println!("{}", host.port);
    {
        use std::io::Write;
        let _ = std::io::stdout().flush();
    }
    loop {
        std::thread::park();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get(port: u16, target: &str, auth: &str) -> (u16, serde_json::Value) {
        use std::io::{Read, Write};
        let mut s = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
        write!(
            s,
            "GET {target} HTTP/1.1\r\nHost: x\r\nAuthorization: Bearer {auth}\r\nConnection: close\r\n\r\n"
        )
        .unwrap();
        let mut out = String::new();
        s.read_to_string(&mut out).unwrap();
        let status = out[9..12].parse().unwrap();
        let body = out.split("\r\n\r\n").nth(1).unwrap_or("null");
        (status, serde_json::from_str(body).unwrap())
    }

    /// A request whose answer is read as text (Gerrit's starts with a guard line).
    fn text_of(port: u16, method: &str, target: &str, body: &str) -> (u16, String) {
        use std::io::{Read, Write};
        let mut s = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
        write!(
            s,
            "{method} {target} HTTP/1.1\r\nHost: x\r\nAuthorization: Basic dDp0\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .unwrap();
        let mut out = String::new();
        s.read_to_string(&mut out).unwrap();
        let status = out[9..12].parse().unwrap();
        let text = out
            .split_once("\r\n\r\n")
            .map_or("", |(_, b)| b)
            .to_string();
        (status, text)
    }

    #[test]
    fn serves_both_sides_pages_and_rejects_a_revoked_token() {
        let host = start(Options::default()).unwrap();
        let (status, repos) = get(host.port, "/api/v3/user/repos?per_page=100&page=1", "t");
        assert_eq!(status, 200);
        assert_eq!(repos.as_array().unwrap().len(), GITHUB_REPOS.len());
        let (_, second) = get(host.port, "/api/v3/user/repos?per_page=5&page=3", "t");
        assert_eq!(second.as_array().unwrap().len(), GITHUB_REPOS.len() - 10);
        let (_, projects) = get(host.port, "/api/v4/projects?per_page=100&page=1", "t");
        assert_eq!(projects[5]["path_with_namespace"], "quarry/tools/rockhound");
        let (status, _) = get(host.port, "/api/v4/projects?per_page=100", "revoked");
        assert_eq!(status, 401);
    }

    #[test]
    fn the_review_request_lists_the_verdicts_already_given_on_both_sides() {
        let host = start(Options::default()).unwrap();
        let (status, reviews) = get(host.port, "/api/v3/repos/o/r/pulls/9/reviews", "t");
        assert_eq!(status, 200);
        assert_eq!(reviews[0]["state"], "APPROVED");
        assert_eq!(reviews[1]["state"], "CHANGES_REQUESTED");
        let (status, approvals) = get(
            host.port,
            "/api/v4/projects/o%2Forbit/merge_requests/58/approvals",
            "t",
        );
        assert_eq!(status, 200);
        assert_eq!(approvals["approved_by"][0]["user"]["username"], "mara");
    }

    #[test]
    fn gerrit_serves_one_change_with_two_patch_sets_behind_the_guard_line() {
        let host = start(Options::default()).unwrap();
        let raw = |path: &str| {
            let (status, text) = text_of(host.port, "GET", path, "");
            assert_eq!(status, 200, "{path}");
            assert!(text.starts_with(")]}'\n"), "{path}: {text}");
            serde_json::from_str::<serde_json::Value>(&text[5..]).unwrap()
        };
        assert_eq!(raw("/a/accounts/self")["username"], gerrit::GERRIT_LOGIN);
        let change = raw("/a/changes/sandbox~7?o=ALL_REVISIONS");
        assert_eq!(change["revisions"].as_object().unwrap().len(), 2);
        assert_eq!(change["current_revision"], gerrit::sha_of(2));
        assert_eq!(change["permitted_labels"]["Code-Review"][4], "+2");
        let one = raw("/a/changes/sandbox~7/revisions/1/files/");
        assert!(one.get("docs/moons.md").is_none());
        let two = raw("/a/changes/sandbox~7/revisions/2/files/");
        assert!(two.get("docs/moons.md").is_some());
        let (status, _) = text_of(
            host.port,
            "POST",
            "/a/changes/sandbox~7/revisions/1/review",
            "{}",
        );
        assert_eq!(status, 200);
    }

    #[test]
    fn gitea_serves_the_user_repositories_pulls_and_issues_read_only() {
        let host = start(Options::default()).unwrap();
        let (_, user) = get(host.port, "/api/v1/user", "t");
        assert_eq!(user["login"], gitea::GITEA_LOGIN);
        let (_, repos) = get(host.port, "/api/v1/user/repos?limit=50&page=1", "t");
        assert!(!repos.as_array().unwrap().is_empty());
        let (_, pulls) = get(host.port, "/api/v1/repos/o/orbit/pulls?state=open", "t");
        assert_eq!(pulls[0]["head"]["ref"], "feature");
        assert_eq!(pulls[1]["draft"], true);
        let (_, issues) = get(host.port, "/api/v1/repos/o/orbit/issues?type=issues", "t");
        assert_eq!(issues[0]["labels"][0]["name"], "bug");
        let (_, labels) = get(host.port, "/api/v1/repos/o/orbit/labels", "t");
        assert_eq!(labels[0]["name"], "bug");
        let (status, _) = send(host.port, "POST", "/api/v1/repos/o/orbit/issues", "{}");
        assert_eq!(status, 404, "read-only");
        let (status, _) = get(host.port, "/api/v1/user", "revoked");
        assert_eq!(status, 401);
    }

    #[test]
    fn gitlab_serves_merge_requests_and_issues_per_project() {
        let host = start(Options::default()).unwrap();
        let (status, mrs) = get(host.port, "/api/v4/projects/o%2Forbit/merge_requests", "t");
        assert_eq!(status, 200);
        assert_eq!(mrs[0]["iid"], 58);
        let (_, other) = get(
            host.port,
            "/api/v4/projects/o%2Forbit-conflict/merge_requests",
            "t",
        );
        assert_eq!(other[0]["iid"], 31);
        let (_, issues) = get(
            host.port,
            "/api/v4/projects/o%2Forbit/issues?state=opened",
            "t",
        );
        assert_eq!(issues[0]["iid"], 214);
        assert!(mrs[0]["updated_at"].as_str().unwrap().ends_with('Z'));
    }

    fn send(port: u16, method: &str, target: &str, body: &str) -> (u16, serde_json::Value) {
        use std::io::{Read, Write};
        let mut s = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
        write!(
            s,
            "{method} {target} HTTP/1.1\r\nHost: x\r\nAuthorization: Bearer t\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .unwrap();
        let mut out = String::new();
        s.read_to_string(&mut out).unwrap();
        let status = out[9..12].parse().unwrap();
        let body = out.split("\r\n\r\n").nth(1).unwrap_or("null");
        (
            status,
            serde_json::from_str(body).unwrap_or(serde_json::Value::Null),
        )
    }

    #[test]
    fn gitlab_metadata_is_served_and_an_edit_shows_in_the_next_list() {
        let host = start(Options::default()).unwrap();
        let p = host.port;
        let (_, members) = get(p, "/api/v4/projects/o%2Forbit/members/all?query=len", "t");
        assert_eq!(members[0]["username"], "lena");
        let (_, labels) = get(
            p,
            "/api/v4/projects/o%2Forbit/labels?per_page=100&page=1",
            "t",
        );
        assert_eq!(labels[3]["name"], "needs-design");
        assert_eq!(labels[3]["color"], "#a295d6");
        let (_, milestones) = get(p, "/api/v4/projects/o%2Forbit/milestones?state=active", "t");
        assert_eq!(milestones[0]["id"], 301);

        let (status, _) = send(
            p,
            "PUT",
            "/api/v4/projects/o%2Forbit/issues/209",
            r#"{"labels":"bug,needs-design","assignee_ids":[22],"milestone_id":302}"#,
        );
        assert_eq!(status, 200);
        let (_, issues) = get(p, "/api/v4/projects/o%2Forbit/issues?state=opened", "t");
        let row = issues
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["iid"] == 209)
            .unwrap();
        assert_eq!(row["labels"], serde_json::json!(["bug", "needs-design"]));
        assert_eq!(row["assignees"][0]["username"], "ines");
        let (_, one) = get(p, "/api/v4/projects/o%2Forbit/issues/209", "t");
        assert_eq!(one["milestone"]["title"], "v1.1");
        // The other issue is untouched.
        let other = issues
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["iid"] == 214)
            .unwrap();
        assert_eq!(
            other["labels"],
            serde_json::json!(["data", "good first issue"])
        );
    }

    #[test]
    fn github_metadata_is_served_and_reviewers_are_added_and_removed() {
        let host = start(Options::default()).unwrap();
        let p = host.port;
        let (_, people) = get(p, "/api/v3/repos/o/r/assignees?per_page=100&page=1", "t");
        assert_eq!(people[1]["login"], "theo-park");
        let (status, _) = send(
            p,
            "PATCH",
            "/api/v3/repos/o/r/issues/7",
            r#"{"labels":["docs"],"milestone":1,"assignees":["ivy-ng"]}"#,
        );
        assert_eq!(status, 200);
        send(
            p,
            "POST",
            "/api/v3/repos/o/r/pulls/7/requested_reviewers",
            r#"{"reviewers":["theo-park","ivy-ng"]}"#,
        );
        send(
            p,
            "DELETE",
            "/api/v3/repos/o/r/pulls/7/requested_reviewers",
            r#"{"reviewers":["ivy-ng"]}"#,
        );
        let (_, pull) = get(p, "/api/v3/repos/o/r/pulls/7", "t");
        assert_eq!(pull["requested_reviewers"][0]["login"], "theo-park");
        assert_eq!(pull["requested_reviewers"].as_array().unwrap().len(), 1);
        assert_eq!(pull["labels"][0]["name"], "docs");
        assert_eq!(pull["assignees"][0]["login"], "ivy-ng");
    }

    #[test]
    fn gitlab_merge_requests_carry_their_pipeline_and_conflicts() {
        let host = start(Options::default()).unwrap();
        let (_, mrs) = get(host.port, "/api/v4/projects/o%2Forbit/merge_requests", "t");
        let by_iid = |iid: u64| {
            mrs.as_array()
                .unwrap()
                .iter()
                .find(|m| m["iid"] == iid)
                .unwrap()
                .clone()
        };
        assert_eq!(by_iid(58)["head_pipeline"]["status"], "success");
        assert_eq!(by_iid(57)["head_pipeline"]["status"], "failed");
        assert_eq!(by_iid(52)["has_conflicts"], true);
        assert_eq!(by_iid(58)["has_conflicts"], false);
    }

    #[test]
    fn gitlab_todos_list_the_mentions_with_their_items() {
        let host = start(Options::default()).unwrap();
        let (_, mrs) = get(
            host.port,
            "/api/v4/todos?action=mentioned&state=pending&type=MergeRequest",
            "t",
        );
        assert_eq!(mrs.as_array().unwrap().len(), 1);
        assert_eq!(mrs[0]["project"]["path_with_namespace"], "o/orbit");
        assert_eq!(mrs[0]["target"]["iid"], 52);
        let (_, issues) = get(
            host.port,
            "/api/v4/todos?action=mentioned&state=pending&type=Issue",
            "t",
        );
        assert_eq!(issues[0]["target"]["iid"], 190);
        let (_, none) = get(host.port, "/api/v4/todos?action=assigned", "t");
        assert!(none.as_array().unwrap().is_empty());
    }

    #[test]
    fn bare_repositories_are_advertised_as_file_urls_and_exist() {
        let root = tempfile::tempdir().unwrap();
        seed_bare_repos(root.path()).unwrap();
        let host = start(Options {
            port: 0,
            bare_root: Some(root.path().to_path_buf()),
        })
        .unwrap();
        let (_, repos) = get(host.port, "/api/v3/user/repos?per_page=100&page=1", "t");
        let url = repos[0]["clone_url"].as_str().unwrap();
        let path = url.strip_prefix("file://").unwrap();
        assert!(Path::new(path).join("HEAD").is_file(), "{path}");
    }
}
