//! A loopback HTTP mock with a route table, and the CLI driver.
#![allow(dead_code)] // each provider module uses a subset, and builds may drop providers

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::sync::{Arc, Mutex};

use assert_cmd::Command;
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::Value;

/// One request the mock saw.
#[derive(Debug, Clone)]
pub struct Seen {
    pub method: String,
    /// Path and query, as sent.
    pub target: String,
    /// Header names lowercased.
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl Seen {
    pub fn path(&self) -> &str {
        self.target.split('?').next().unwrap_or_default()
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }
}

/// `(method, path contains[|body contains], status, json)`: the first
/// matching route answers; anything else is a 404.
pub type Route = (&'static str, String, u16, String);

pub struct Mock {
    pub port: u16,
    seen: Arc<Mutex<Vec<Seen>>>,
}

impl Mock {
    pub fn start(routes: Vec<Route>) -> Mock {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind the mock");
        let port = listener.local_addr().unwrap().port();
        let seen: Arc<Mutex<Vec<Seen>>> = Arc::default();
        let log = Arc::clone(&seen);
        let routes = Arc::new(routes);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let log = Arc::clone(&log);
                let routes = Arc::clone(&routes);
                std::thread::spawn(move || {
                    let mut buf = Vec::new();
                    let mut chunk = [0u8; 4096];
                    let (head_end, content_len) = loop {
                        let n = stream.read(&mut chunk).unwrap_or(0);
                        if n == 0 {
                            return;
                        }
                        buf.extend_from_slice(&chunk[..n]);
                        if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                            let head = String::from_utf8_lossy(&buf[..pos]).to_lowercase();
                            let len = head
                                .lines()
                                .find_map(|l| l.strip_prefix("content-length:"))
                                .and_then(|v| v.trim().parse::<usize>().ok())
                                .unwrap_or(0);
                            break (pos + 4, len);
                        }
                    };
                    while buf.len() < head_end + content_len {
                        let n = stream.read(&mut chunk).unwrap_or(0);
                        if n == 0 {
                            break;
                        }
                        buf.extend_from_slice(&chunk[..n]);
                    }
                    let head = String::from_utf8_lossy(&buf[..head_end]).to_string();
                    let body = String::from_utf8_lossy(&buf[head_end..]).to_string();
                    let mut lines = head.lines();
                    let line = lines.next().unwrap_or_default();
                    let method = line.split(' ').next().unwrap_or_default().to_string();
                    let target = line.split(' ').nth(1).unwrap_or_default().to_string();
                    let headers = lines
                        .filter_map(|l| l.split_once(':'))
                        .map(|(n, v)| (n.trim().to_lowercase(), v.trim().to_string()))
                        .collect();
                    let seen = Seen {
                        method,
                        target,
                        headers,
                        body,
                    };
                    let (status, json) = routes
                        .iter()
                        .find(|(m, p, _, _)| {
                            // `path|needle`: the body must contain `needle` too
                            // (one GraphQL endpoint, many documents).
                            let (path, needle) = p.split_once('|').unwrap_or((p.as_str(), ""));
                            *m == seen.method
                                && seen.path().contains(path)
                                && seen.body.contains(needle)
                        })
                        .map(|(_, _, s, j)| (*s, j.clone()))
                        .unwrap_or((404, r#"{"message":"no such route"}"#.to_string()));
                    log.lock().unwrap().push(seen);
                    let reply = format!(
                        "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{json}",
                        json.len()
                    );
                    let _ = stream.write_all(reply.as_bytes());
                });
            }
        });
        Mock { port, seen }
    }

    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    pub fn requests(&self) -> Vec<Seen> {
        self.seen.lock().unwrap().clone()
    }

    /// The requests to paths containing `needle`.
    pub fn to(&self, needle: &str) -> Vec<Seen> {
        self.requests()
            .into_iter()
            .filter(|r| r.path().contains(needle))
            .collect()
    }
}

pub fn route(method: &'static str, path: &str, json: &str) -> Route {
    (method, path.to_string(), 200, json.to_string())
}

pub struct Cli {
    pub config: tempfile::TempDir,
    session: String,
    tokens: Vec<(String, String)>,
}

impl Cli {
    /// `config` is the whole `config.toml`; `tokens` the
    /// `GITPOULPI_TOKEN_<ID>` variables to set.
    pub fn new(session: &str, config: &str, tokens: &[(&str, &str)]) -> Self {
        let dir = tempfile::tempdir().expect("temp config dir");
        std::fs::write(dir.path().join("config.toml"), config).unwrap();
        Self {
            config: dir,
            session: format!("hp-{session}"),
            tokens: tokens
                .iter()
                .map(|(id, token)| {
                    (
                        format!("GITPOULPI_TOKEN_{}", id.to_uppercase().replace('-', "_")),
                        token.to_string(),
                    )
                })
                .collect(),
        }
    }

    pub fn output(&self, cwd: &Path, args: &[&str]) -> (bool, Value) {
        let output = Command::cargo_bin("gitpoulpi")
            .unwrap()
            .env("GITPOULPI_CONFIG_DIR", self.config.path())
            .envs(self.tokens.iter().map(|(k, v)| (k, v)))
            .arg("--json")
            .args(["--session", &self.session])
            .args(args)
            .current_dir(cwd)
            .output()
            .expect("run gitpoulpi");
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let json = serde_json::from_str(&stdout).unwrap_or_else(|e| {
            panic!(
                "not JSON ({e}): {stdout} / {}",
                String::from_utf8_lossy(&output.stderr)
            )
        });
        (output.status.success(), json)
    }

    pub fn json(&self, cwd: &Path, args: &[&str]) -> Value {
        let (ok, json) = self.output(cwd, args);
        assert!(ok, "`{}` failed: {json}", args.join(" "));
        json
    }

    pub fn error(&self, cwd: &Path, args: &[&str]) -> String {
        let (ok, json) = self.output(cwd, args);
        assert!(!ok, "`{}` unexpectedly succeeded: {json}", args.join(" "));
        json["error"].as_str().expect("an error string").to_string()
    }

    pub fn config_text(&self) -> String {
        std::fs::read_to_string(self.config.path().join("config.toml")).unwrap_or_default()
    }
}

/// A repository with a commit and an `origin` (a local bare repository).
pub fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "start")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .build()
}

/// Writes `.gitpoulpi/repo-state.toml`.
pub fn write_state(repo: &FixtureRepo, text: &str) {
    let dir = repo.state_dir();
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("repo-state.toml"), text).unwrap();
}

pub fn read_state(repo: &FixtureRepo) -> String {
    std::fs::read_to_string(repo.state_dir().join("repo-state.toml")).unwrap_or_default()
}

pub fn integration(id: &str, kind: &str, url: &str, extra: &str) -> String {
    format!(
        "[[integrations]]\nid = \"{id}\"\nkind = \"{kind}\"\nurl = \"{url}\"\ntoken_ref = \"integration:{id}\"\nname = \"{id}\"\n{extra}\n"
    )
}

/// `%XX` and `+` decoded, to compare a query string with what was meant.
pub fn decoded(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("");
                match u8::from_str_radix(hex, 16) {
                    Ok(b) => {
                        out.push(b);
                        i += 3;
                    }
                    Err(_) => {
                        out.push(b'%');
                        i += 1;
                    }
                }
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Pushes a branch `name` with one commit to `origin`, then removes the
/// local branch so only the host has it. Returns its tip.
pub fn host_only_branch(repo: &FixtureRepo, name: &str) -> String {
    repo.git(&["checkout", "-q", "-b", name]);
    std::fs::write(repo.root().join("f.txt"), "f\n").unwrap();
    repo.git(&["add", "f.txt"]);
    repo.git(&["commit", "-q", "-m", "work on the branch"]);
    let tip = repo.git(&["rev-parse", "HEAD"]);
    repo.git(&["push", "-q", "origin", name]);
    repo.git(&["checkout", "-q", "main"]);
    repo.git(&["branch", "-D", name]);
    repo.git(&["update-ref", "-d", &format!("refs/remotes/origin/{name}")]);
    tip
}
