//! `gitpoulpi session list|new|delete|rename`.
//!
//! These answer from the files under `<config>/sessions/` -- never from a
//! daemon -- so they run without connecting (or auto-starting) one, in the
//! CLI and, through `execute`, in the MCP server alike.
//!
//! A session is `sessions/<name>.toml` (tabs + layout, `GlobalSession`)
//! plus, while its daemon runs, `<name>.port` (and `<name>.pid` for a
//! standalone one).
//!
//! Liveness is *probed*, never inferred from the discovery file existing:
//! a daemon that crashed leaves its `.port` file behind, and a stale file
//! reported as a running session is worse than no listing at all.
//!
//! `delete` and `rename` refuse while the session's daemon is live: its
//! daemon (or the TUI embedding it) is keyed by the name and would rewrite
//! the session file on exit, silently undoing the change. The message says
//! how to stop it. Stale `.port`/`.pid` leftovers are cleaned up.

use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::time::Duration;

use gitpoulpi_config::{list_session_port_files, sanitize_session_name, ConfigPaths};
use gitpoulpi_session::GlobalSession;
use serde_json::{json, Value};

use crate::cli::SessionAction;
use crate::repo_cmd::Emitted;

/// Short on purpose: the daemon is on loopback, so a connect that has not
/// completed in this long is not a daemon we could have used anyway, and
/// `session list` must stay instant even with a dozen stale files.
const PROBE_TIMEOUT: Duration = Duration::from_millis(250);

pub fn run(current_session: &str, action: SessionAction) -> Result<Emitted, String> {
    let paths = ConfigPaths::discover()
        .ok_or_else(|| "could not determine the config directory for this OS".to_string())?;
    match action {
        SessionAction::List => Ok(list(&paths, current_session)),
        SessionAction::New { name } => new(&paths, &name),
        SessionAction::Delete { name } => delete(&paths, &name),
        SessionAction::Rename { old_name, new_name } => rename(&paths, &old_name, &new_name),
    }
}

/// A TCP connect, not a file check. `connect_timeout` needs a concrete
/// `SocketAddr`, which is why this builds one instead of using the
/// `(host, port)` tuple form.
pub fn daemon_is_listening(port: u16) -> bool {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    TcpStream::connect_timeout(&addr, PROBE_TIMEOUT).is_ok()
}

fn port_of(paths: &ConfigPaths, name: &str) -> Option<u16> {
    std::fs::read_to_string(paths.ipc_port_file(name))
        .ok()?
        .trim()
        .parse()
        .ok()
}

fn is_live(paths: &ConfigPaths, name: &str) -> bool {
    port_of(paths, name).is_some_and(daemon_is_listening)
}

fn exists(paths: &ConfigPaths, name: &str) -> bool {
    paths.session_file(name).exists() || paths.ipc_port_file(name).exists()
}

fn list(paths: &ConfigPaths, current: &str) -> Emitted {
    let sessions_dir = paths.sessions_dir();
    let mut names: std::collections::BTreeMap<String, Option<u16>> =
        std::collections::BTreeMap::new();
    for (name, port) in list_session_port_files(&sessions_dir) {
        names.insert(name, Some(port));
    }
    if let Ok(entries) = std::fs::read_dir(&sessions_dir) {
        for path in entries.flatten().map(|e| e.path()) {
            if path.extension().is_some_and(|e| e == "toml") {
                if let Some(stem) = path.file_stem() {
                    names
                        .entry(stem.to_string_lossy().into_owned())
                        .or_default();
                }
            }
        }
    }

    let rows: Vec<Value> = names
        .into_iter()
        .map(|(name, port)| {
            let live = port.is_some_and(daemon_is_listening);
            json!({
                "name": name,
                "port": port,
                "live": live,
                "saved": paths.session_file(&name).exists(),
                "current": name == current,
            })
        })
        .collect();

    let mut human = format!("sessions in {}", sessions_dir.display());
    if rows.is_empty() {
        human.push_str("\n  (none - no session has been created or run a daemon here)");
    }
    for row in &rows {
        let marker = if row["current"] == true { "*" } else { " " };
        let state = if row["live"] == true {
            "live"
        } else if row["port"].is_null() {
            "saved"
        } else {
            "stale"
        };
        let port = row["port"]
            .as_u64()
            .map_or_else(|| "-".to_string(), |p| p.to_string());
        human.push_str(&format!(
            "\n{marker} {:<40} port {port:<6} {state}",
            row["name"].as_str().unwrap_or_default(),
        ));
    }
    if !rows.iter().any(|row| row["current"] == true) {
        human.push_str(&format!(
            "\n  (current session {current} has no daemon yet)"
        ));
    }

    Emitted {
        exit_code: 0,
        json: json!({
            "current": current,
            "sessions_dir": sessions_dir,
            "sessions": rows,
        }),
        human,
    }
}

/// The name as it will be stored; an empty or all-separator name is
/// refused rather than turned into a surprise.
fn clean(raw: &str) -> Result<String, String> {
    if raw.trim().is_empty() {
        return Err("a session name cannot be empty".into());
    }
    Ok(sanitize_session_name(raw))
}

fn new(paths: &ConfigPaths, raw: &str) -> Result<Emitted, String> {
    let name = clean(raw)?;
    if exists(paths, &name) {
        return Err(format!("session `{name}` already exists"));
    }
    GlobalSession::default()
        .save(&paths.session_file(&name))
        .map_err(|e| e.to_string())?;
    Ok(Emitted {
        exit_code: 0,
        json: json!({ "created": name, "path": paths.session_file(&name) }),
        human: format!("created session {name}"),
    })
}

fn refuse_live(paths: &ConfigPaths, name: &str, verb: &str) -> Result<(), String> {
    if is_live(paths, name) {
        return Err(format!(
            "session `{name}` has a running daemon (or an open TUI); refusing to {verb} it. \
             Stop it first with `gitpoulpi -s {name} daemon stop` (close the TUI if one is \
             open), then retry."
        ));
    }
    Ok(())
}

/// Leftover discovery files of a daemon that is no longer there.
fn remove_stale_discovery(paths: &ConfigPaths, name: &str) {
    let _ = std::fs::remove_file(paths.ipc_port_file(name));
    let _ = std::fs::remove_file(paths.ipc_pid_file(name));
    let _ = std::fs::remove_file(paths.ipc_port_file(name).with_extension("host"));
}

fn delete(paths: &ConfigPaths, raw: &str) -> Result<Emitted, String> {
    let name = clean(raw)?;
    if !exists(paths, &name) {
        return Err(format!("no session named `{name}`"));
    }
    refuse_live(paths, &name, "delete")?;
    remove_stale_discovery(paths, &name);
    let file: PathBuf = paths.session_file(&name);
    match std::fs::remove_file(&file) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(format!("could not delete {}: {e}", file.display())),
    }
    Ok(Emitted {
        exit_code: 0,
        json: json!({ "deleted": name }),
        human: format!("deleted session {name}"),
    })
}

fn rename(paths: &ConfigPaths, old_raw: &str, new_raw: &str) -> Result<Emitted, String> {
    let old = clean(old_raw)?;
    let new = clean(new_raw)?;
    if !exists(paths, &old) {
        return Err(format!("no session named `{old}`"));
    }
    if old == new {
        return Err("the old and new session names are the same".into());
    }
    if exists(paths, &new) {
        return Err(format!("session `{new}` already exists"));
    }
    refuse_live(paths, &old, "rename")?;
    remove_stale_discovery(paths, &old);
    let from = paths.session_file(&old);
    let to = paths.session_file(&new);
    if from.exists() {
        // A rename of the file, so tabs and layout travel untouched.
        std::fs::rename(&from, &to)
            .map_err(|e| format!("could not rename {}: {e}", from.display()))?;
    } else {
        GlobalSession::default()
            .save(&to)
            .map_err(|e| e.to_string())?;
    }
    Ok(Emitted {
        exit_code: 0,
        json: json!({ "renamed": { "from": old, "to": new } }),
        human: format!("renamed session {old} to {new}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn a_bound_port_probes_live_and_a_released_one_does_not() {
        // A released ephemeral port can be re-bound at once by anything else
        // on the machine (another test's daemon, a parallel agent), so a
        // single "released" probe is racy: ask over fresh ports and require
        // that a released one reads as dead at least once. A probe that
        // always said "live" would still fail every attempt.
        let mut dead = false;
        for _ in 0..20 {
            let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
            let port = listener.local_addr().unwrap().port();
            assert!(daemon_is_listening(port));
            drop(listener);
            if !daemon_is_listening(port) {
                dead = true;
                break;
            }
        }
        assert!(dead, "a released port always probed as live");
    }
}
