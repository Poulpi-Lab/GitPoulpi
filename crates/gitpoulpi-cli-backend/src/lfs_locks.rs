//! Git LFS file locks: `git lfs locks | lock | unlock --json`.
//!
//! Locking is an LFS *server* feature (GitHub, GitLab and Gitea have it),
//! reached with the remote's own credentials -- GitPoulpi has no account
//! of its own. A server without it is reported as
//! [`Error::LfsLockingUnsupported`] where git-lfs says so (`lock` and
//! `unlock`); git-lfs itself answers `locks` on such a server with an
//! empty list, so that is what `lfs_locks` returns.

use std::path::Path;

use gitpoulpi_core::domain::{LfsLock, RepoRef};
use gitpoulpi_core::{Error, Result};
use serde_json::Value;

use crate::lfs::LfsProbe;
use crate::process::{run_full_async, run_lenient};

/// Fragments of git-lfs's error text (which carries the server's own
/// message) that mean the endpoint does not do locking. Lower-case.
const UNSUPPORTED: &[&str] = &[
    "not supported",
    "not implemented",
    "does not support",
    "no lock api",
    "not found", // a 404 on POST /locks: there is no locking endpoint
];

fn is_unsupported(text: &str) -> bool {
    let lower = text.to_lowercase();
    UNSUPPORTED.iter().any(|needle| lower.contains(needle))
}

fn lock_from_json(value: &Value, ours: bool) -> Option<LfsLock> {
    Some(LfsLock {
        id: value["id"].as_str()?.to_string(),
        path: value["path"].as_str()?.into(),
        owner: value["owner"]["name"].as_str().map(str::to_string),
        locked_at: value["locked_at"].as_str().unwrap_or_default().to_string(),
        ours,
    })
}

/// `git lfs locks --json --verify` prints `{"ours": [...], "theirs":
/// [...]}`; plain `--json` prints a bare array.
fn parse_verified(stdout: &str) -> Option<Vec<LfsLock>> {
    let value: Value = serde_json::from_str(stdout).ok()?;
    let mut locks = Vec::new();
    for (key, ours) in [("ours", true), ("theirs", false)] {
        for item in value[key].as_array()? {
            locks.extend(lock_from_json(item, ours));
        }
    }
    Some(locks)
}

fn parse_list(stdout: &str, ours: impl Fn(&LfsLock) -> bool) -> Option<Vec<LfsLock>> {
    let value: Value = serde_json::from_str(stdout).ok()?;
    let mut locks: Vec<LfsLock> = value
        .as_array()?
        .iter()
        .filter_map(|item| lock_from_json(item, false))
        .collect();
    for lock in &mut locks {
        lock.ours = ours(lock);
    }
    Some(locks)
}

/// The error for a failed `lock` / `unlock` / `locks`.
fn lock_error(remote: &str, what: &str, stderr: &str) -> Error {
    if is_unsupported(stderr) {
        Error::LfsLockingUnsupported {
            remote: remote.to_string(),
        }
    } else {
        Error::Backend(format!("git lfs {what} failed: {}", stderr.trim()))
    }
}

fn slashes(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// See `GitBackend::lfs_locks`.
pub async fn lfs_locks(repo: &RepoRef, probe: &LfsProbe, remote: &str) -> Result<Vec<LfsLock>> {
    probe.require(&repo.root)?;
    let verified = run_full_async(
        &repo.root,
        &["lfs", "locks", "--json", "--verify", "--remote", remote],
    )
    .await?;
    if verified.success {
        if let Some(locks) = parse_verified(&verified.stdout) {
            return Ok(locks);
        }
    }

    // Some servers do not implement the verify endpoint: fall back to the
    // plain list and call a lock ours when its owner is the git user.
    let plain = run_full_async(&repo.root, &["lfs", "locks", "--json", "--remote", remote]).await?;
    if !plain.success {
        return Err(lock_error(remote, "locks", &plain.stderr));
    }
    let me = run_lenient(&repo.root, &["config", "--get", "user.name"])
        .ok()
        .filter(|(ok, _)| *ok)
        .map(|(_, name)| name.trim().to_string());
    parse_list(&plain.stdout, |lock| {
        lock.owner.is_some() && lock.owner.as_deref() == me.as_deref()
    })
    .ok_or_else(|| Error::Backend("git lfs locks gave output that is not JSON".into()))
}

/// See `GitBackend::lfs_lock`.
pub async fn lfs_lock(
    repo: &RepoRef,
    probe: &LfsProbe,
    remote: &str,
    path: &Path,
) -> Result<LfsLock> {
    probe.require(&repo.root)?;
    let path = slashes(path);
    let out = run_full_async(
        &repo.root,
        &["lfs", "lock", "--json", "--remote", remote, "--", &path],
    )
    .await?;
    if !out.success {
        return Err(lock_error(remote, "lock", &out.stderr));
    }
    let value: Value = serde_json::from_str(&out.stdout)
        .map_err(|e| Error::Backend(format!("git lfs lock gave output that is not JSON: {e}")))?;
    value
        .as_array()
        .and_then(|a| a.first())
        .or(Some(&value))
        .and_then(|v| lock_from_json(v, true))
        .ok_or_else(|| Error::Backend("git lfs lock did not report the lock".into()))
}

/// See `GitBackend::lfs_unlock`.
pub async fn lfs_unlock(
    repo: &RepoRef,
    probe: &LfsProbe,
    remote: &str,
    path: &Path,
    force: bool,
) -> Result<()> {
    probe.require(&repo.root)?;
    let path = slashes(path);
    let mut args = vec!["lfs", "unlock", "--json", "--remote", remote];
    if force {
        args.push("--force");
    }
    args.push("--");
    args.push(&path);
    let out = run_full_async(&repo.root, &args).await?;
    if out.success {
        return Ok(());
    }
    // `[{"path": ..., "unlocked": false, "reason": "..."}]` carries the
    // most specific message.
    let reason = serde_json::from_str::<Value>(&out.stdout)
        .ok()
        .and_then(|v| v[0]["reason"].as_str().map(str::to_string))
        .unwrap_or_else(|| out.stderr.clone());
    Err(lock_error(remote, "unlock", &reason))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_verified_listing_marks_ours_and_theirs() {
        let json = r#"{"ours":[{"id":"2","path":"x.bin","owner":{"name":"me"},"locked_at":"t"}],
                       "theirs":[{"id":"1","path":"y.bin","owner":{"name":"other"},"locked_at":"t"}]}"#;
        let locks = parse_verified(json).unwrap();
        assert_eq!(locks.len(), 2);
        assert!(locks[0].ours && locks[0].path == Path::new("x.bin"));
        assert!(!locks[1].ours && locks[1].owner.as_deref() == Some("other"));
        assert!(
            parse_verified("[]").is_none(),
            "a bare array is not the verify shape"
        );
    }

    #[test]
    fn a_plain_listing_uses_the_owner_to_decide_ours() {
        let json = r#"[{"id":"1","path":"a","owner":{"name":"me"},"locked_at":"t"},
                       {"id":"2","path":"b","owner":{"name":"you"},"locked_at":"t"}]"#;
        let locks = parse_list(json, |l| l.owner.as_deref() == Some("me")).unwrap();
        assert!(locks[0].ours && !locks[1].ours);
    }

    #[test]
    fn server_refusals_that_mean_no_locking_are_typed() {
        for text in [
            "Locking x.bin failed: locking is not supported",
            "Locking x.bin failed: not found",
            "Server does not support locking",
            "501 Not Implemented",
        ] {
            assert!(
                matches!(
                    lock_error("origin", "lock", text),
                    Error::LfsLockingUnsupported { .. }
                ),
                "{text}"
            );
        }
        assert!(matches!(
            lock_error(
                "origin",
                "lock",
                "Locking x.bin failed: already locked by bob"
            ),
            Error::Backend(_)
        ));
    }
}
