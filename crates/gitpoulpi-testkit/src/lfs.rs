//! Git LFS test infrastructure: the availability probe and its skip
//! macro, and a loopback lock server. Offline by construction: transfers
//! use git-lfs's built-in `file://` adapter (see
//! `FixtureRepoBuilder::with_lfs_origin`), and locking (and, when a check
//! needs real HTTP transfers, the basic adapter) talks to a `127.0.0.1`
//! server started here.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::Command;
use std::sync::{Arc, Mutex, OnceLock};

use serde_json::{json, Value};

/// Is a usable `git-lfs` installed? Asked once per test binary.
///
/// Runs `git lfs version` with the same hermetic environment the fixtures
/// use, so a user's global config cannot change the answer.
pub fn git_lfs_available() -> bool {
    static AVAILABLE: OnceLock<bool> = OnceLock::new();
    *AVAILABLE.get_or_init(|| {
        let mut cmd = Command::new("git");
        cmd.args(["lfs", "version"]);
        crate::fixture::isolate_from_user_config(&mut cmd);
        cmd.output()
            .map(|o| o.status.success() && String::from_utf8_lossy(&o.stdout).contains("git-lfs/"))
            .unwrap_or(false)
    })
}

/// Runs `git <args>` in `dir` with the fixtures' hermetic environment and
/// panics with git's stderr on failure. For arranging LFS state (a clone,
/// a pull) that the check then observes through the backend under test.
pub fn hermetic_git(dir: &std::path::Path, args: &[&str]) -> String {
    let mut cmd = Command::new("git");
    cmd.args(args).current_dir(dir);
    crate::fixture::isolate_from_user_config(&mut cmd);
    cmd.env("GIT_TERMINAL_PROMPT", "0");
    let out = cmd.output().expect("failed to spawn git");
    assert!(
        out.status.success(),
        "git {} failed in {}: {}",
        args.join(" "),
        dir.display(),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// Clones `origin` into `dest` the way a machine with `GIT_LFS_SKIP_SMUDGE`
/// set would: the working tree holds pointer text, the local LFS store is
/// empty. (Under the hermetic environment no LFS filter is configured, so
/// this holds whether or not the variable is honoured.)
pub fn clone_leaving_pointers(origin: &std::path::Path, dest: &std::path::Path) {
    let parent = dest.parent().expect("dest has a parent");
    let mut cmd = Command::new("git");
    cmd.arg("clone")
        .arg(origin)
        .arg(dest)
        .current_dir(parent)
        .env("GIT_LFS_SKIP_SMUDGE", "1");
    crate::fixture::isolate_from_user_config(&mut cmd);
    let out = cmd.output().expect("failed to spawn git");
    assert!(
        out.status.success(),
        "clone failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// The decision behind [`require_git_lfs!`], with its two inputs
/// injectable so the skip logic itself can be unit-tested.
///
/// Returns `true` to run the check. When `git-lfs` is missing it returns
/// `false` after printing `SKIPPED <check>: git-lfs not installed`, unless
/// `require` is set (`GITPOULPI_REQUIRE_LFS=1`), in which case it panics:
/// a machine that is supposed to have git-lfs (the local gate, CI) must
/// not pass silently without it.
pub fn lfs_gate(check: &str, available: bool, require: bool) -> bool {
    if available {
        return true;
    }
    if require {
        panic!("{check}: git-lfs is not installed but GITPOULPI_REQUIRE_LFS=1");
    }
    eprintln!("SKIPPED {check}: git-lfs not installed");
    false
}

/// Whether `GITPOULPI_REQUIRE_LFS=1` is set.
pub fn lfs_required() -> bool {
    std::env::var("GITPOULPI_REQUIRE_LFS").is_ok_and(|v| v == "1")
}

/// First line of every check that needs the `git-lfs` binary: returns
/// from the enclosing function (skipping the check) when it is missing.
#[macro_export]
macro_rules! require_git_lfs {
    ($check:expr) => {
        if !$crate::lfs::lfs_gate(
            $check,
            $crate::lfs::git_lfs_available(),
            $crate::lfs::lfs_required(),
        ) {
            return;
        }
    };
}

/// One lock held by the test server.
#[derive(Debug, Clone)]
struct ServerLock {
    id: String,
    path: String,
    owner: String,
    locked_at: String,
}

struct ServerState {
    port: u16,
    /// Object store of the basic transfer adapter: oid -> bytes.
    objects: std::collections::HashMap<String, Vec<u8>>,
    locks: Vec<ServerLock>,
    next_id: u32,
    supports_locking: bool,
}

/// The identity the server gives to locks made through it.
pub const LOCK_SERVER_USER: &str = "tester";

/// A minimal Git LFS server on a loopback port, with in-memory state and
/// no authentication:
///
/// - the locking API: `GET /locks`, `POST /locks`, `POST /locks/verify`,
///   `POST /locks/{id}/unlock`;
/// - the batch API and the *basic* transfer adapter over plain HTTP:
///   `POST /objects/batch`, `PUT` / `GET /objects/{oid}`. A real HTTP
///   transfer is the only way to get git-lfs to write progress lines (its
///   built-in `file://` adapter reports none).
pub struct LfsLockServer {
    port: u16,
    state: Arc<Mutex<ServerState>>,
}

impl LfsLockServer {
    /// Start a server that supports locking.
    pub fn start() -> Self {
        Self::start_with(true)
    }

    /// Start a server that answers every locking request with `501`, like
    /// an LFS server without the locking API.
    pub fn start_without_locking() -> Self {
        Self::start_with(false)
    }

    fn start_with(supports_locking: bool) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
        let port = listener.local_addr().unwrap().port();
        let state = Arc::new(Mutex::new(ServerState {
            port,
            objects: std::collections::HashMap::new(),
            locks: Vec::new(),
            next_id: 1,
            supports_locking,
        }));
        let shared = Arc::clone(&state);
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let shared = Arc::clone(&shared);
                std::thread::spawn(move || {
                    let _ = handle(stream, &shared);
                });
            }
        });
        Self { port, state }
    }

    /// The value for `lfs.url`.
    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    /// Pre-create a lock held by someone else (or anyone).
    pub fn seed_lock(&self, path: &str, owner: &str) -> String {
        let mut state = self.state.lock().unwrap();
        let id = state.next_id.to_string();
        state.next_id += 1;
        state.locks.push(ServerLock {
            id: id.clone(),
            path: path.to_string(),
            owner: owner.to_string(),
            locked_at: "2026-01-01T00:00:00Z".to_string(),
        });
        id
    }

    /// The object ids stored through the basic transfer adapter, sorted.
    pub fn stored_objects(&self) -> Vec<String> {
        let state = self.state.lock().unwrap();
        let mut oids: Vec<String> = state.objects.keys().cloned().collect();
        oids.sort();
        oids
    }

    /// The paths currently locked, in creation order.
    pub fn locked_paths(&self) -> Vec<String> {
        let state = self.state.lock().unwrap();
        state.locks.iter().map(|l| l.path.clone()).collect()
    }
}

fn lock_json(lock: &ServerLock) -> Value {
    json!({
        "id": lock.id,
        "path": lock.path,
        "locked_at": lock.locked_at,
        "owner": { "name": lock.owner },
    })
}

fn respond_bytes(stream: &mut TcpStream, status: &str, body: &[u8]) -> std::io::Result<()> {
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: application/octet-stream\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    stream.write_all(body)?;
    stream.flush()
}

fn respond(stream: &mut TcpStream, status: &str, body: &Value) -> std::io::Result<()> {
    let body = body.to_string();
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: application/vnd.git-lfs+json\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )?;
    stream.flush()
}

fn handle(mut stream: TcpStream, state: &Mutex<ServerState>) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let target = parts.next().unwrap_or("").to_string();

    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 || line == "\r\n" || line == "\n" {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("content-length") {
                content_length = value.trim().parse().unwrap_or(0);
            }
        }
    }
    let mut body = vec![0u8; content_length];
    reader.read_exact(&mut body)?;
    let raw = body;
    let body: Value = serde_json::from_slice(&raw).unwrap_or(Value::Null);
    let path = target.split('?').next().unwrap_or("");

    let mut state = state.lock().unwrap();

    // Transfers: the batch API points every action back at this server.
    match (method.as_str(), path) {
        ("POST", "/objects/batch") => {
            let upload = body["operation"] == "upload";
            let port = state.port;
            let objects: Vec<Value> = body["objects"]
                .as_array()
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|object| {
                    let oid = object["oid"].as_str().unwrap_or("").to_string();
                    let size = object["size"].as_u64().unwrap_or(0);
                    let href = format!("http://127.0.0.1:{port}/objects/{oid}");
                    let have = state.objects.contains_key(&oid);
                    let action = if upload && !have {
                        Some(json!({ "upload": { "href": href } }))
                    } else if !upload && have {
                        Some(json!({ "download": { "href": href } }))
                    } else if !upload {
                        return json!({
                            "oid": oid, "size": size,
                            "error": { "code": 404, "message": "object not found" }
                        });
                    } else {
                        None
                    };
                    match action {
                        Some(actions) => json!({ "oid": oid, "size": size, "actions": actions }),
                        None => json!({ "oid": oid, "size": size }),
                    }
                })
                .collect();
            return respond(
                &mut stream,
                "200 OK",
                &json!({ "transfer": "basic", "objects": objects }),
            );
        }
        ("PUT", p) if p.starts_with("/objects/") => {
            let oid = p.trim_start_matches("/objects/").to_string();
            state.objects.insert(oid, raw);
            return respond_bytes(&mut stream, "200 OK", b"");
        }
        ("GET", p) if p.starts_with("/objects/") => {
            let oid = p.trim_start_matches("/objects/");
            return match state.objects.get(oid) {
                Some(bytes) => respond_bytes(&mut stream, "200 OK", bytes),
                None => respond_bytes(&mut stream, "404 Not Found", b""),
            };
        }
        _ => {}
    }

    if !state.supports_locking {
        return respond(
            &mut stream,
            "501 Not Implemented",
            &json!({"message": "locking is not supported"}),
        );
    }

    match (method.as_str(), path) {
        ("GET", "/locks") => {
            // `?path=` and `?id=` filter, as on a real server: git-lfs
            // finds a lock's id by asking for its path before unlocking.
            let query = target.split_once('?').map_or("", |(_, q)| q);
            let wanted = |key: &str| {
                query
                    .split('&')
                    .find_map(|kv| kv.strip_prefix(&format!("{key}=")))
                    .map(|v| v.replace("%2F", "/").replace("%2f", "/"))
            };
            let (by_path, by_id) = (wanted("path"), wanted("id"));
            let locks: Vec<Value> = state
                .locks
                .iter()
                .filter(|l| by_path.as_ref().map_or(true, |p| &l.path == p))
                .filter(|l| by_id.as_ref().map_or(true, |i| &l.id == i))
                .map(lock_json)
                .collect();
            respond(&mut stream, "200 OK", &json!({ "locks": locks }))
        }
        ("POST", "/locks/verify") => {
            let (ours, theirs): (Vec<_>, Vec<_>) = state
                .locks
                .iter()
                .partition(|l| l.owner == LOCK_SERVER_USER);
            respond(
                &mut stream,
                "200 OK",
                &json!({
                    "ours": ours.iter().map(|l| lock_json(l)).collect::<Vec<_>>(),
                    "theirs": theirs.iter().map(|l| lock_json(l)).collect::<Vec<_>>(),
                }),
            )
        }
        ("POST", "/locks") => {
            let wanted = body["path"].as_str().unwrap_or("").to_string();
            if let Some(existing) = state.locks.iter().find(|l| l.path == wanted) {
                return respond(
                    &mut stream,
                    "409 Conflict",
                    &json!({ "lock": lock_json(existing), "message": "already created lock" }),
                );
            }
            let id = state.next_id.to_string();
            state.next_id += 1;
            let lock = ServerLock {
                id,
                path: wanted,
                owner: LOCK_SERVER_USER.to_string(),
                locked_at: "2026-09-30T12:00:00Z".to_string(),
            };
            let reply = json!({ "lock": lock_json(&lock) });
            state.locks.push(lock);
            respond(&mut stream, "201 Created", &reply)
        }
        ("POST", p) if p.starts_with("/locks/") && p.ends_with("/unlock") => {
            let id = p
                .trim_start_matches("/locks/")
                .trim_end_matches("/unlock")
                .to_string();
            let force = body["force"].as_bool().unwrap_or(false);
            let Some(pos) = state.locks.iter().position(|l| l.id == id) else {
                return respond(
                    &mut stream,
                    "404 Not Found",
                    &json!({"message": "no such lock"}),
                );
            };
            if state.locks[pos].owner != LOCK_SERVER_USER && !force {
                return respond(
                    &mut stream,
                    "403 Forbidden",
                    &json!({"message": "lock is held by someone else, force required"}),
                );
            }
            let removed = state.locks.remove(pos);
            respond(
                &mut stream,
                "200 OK",
                &json!({ "lock": lock_json(&removed) }),
            )
        }
        _ => respond(
            &mut stream,
            "404 Not Found",
            &json!({"message": "not found"}),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_gate_skips_when_missing_and_runs_when_present() {
        assert!(lfs_gate("x", true, false));
        assert!(lfs_gate("x", true, true));
        assert!(!lfs_gate("x", false, false));
    }

    #[test]
    #[should_panic(expected = "GITPOULPI_REQUIRE_LFS")]
    fn the_gate_panics_when_git_lfs_is_required_but_missing() {
        lfs_gate("x", false, true);
    }
}
