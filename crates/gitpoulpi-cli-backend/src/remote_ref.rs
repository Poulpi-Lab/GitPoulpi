//! A single branch or tag on a remote, read and set exactly: what undoing
//! (or redoing) the deletion of a remote branch or tag stands on.
//!
//! Never forced: every change carries a lease on the value the caller saw
//! (`--force-with-lease=<ref>:<value>`, empty value = "must not exist"), so a
//! remote that moved in between refuses the push instead of being
//! overwritten.

use gitpoulpi_core::domain::{remote_tag_mirror_ref, Oid, PushRejection, RepoRef, UndoRefusal};
use gitpoulpi_core::{Error, Result};

use crate::history::push::rejection_reason;
use crate::process::{run_full_async, run_status_async, run_with_stdin};

fn check(remote: &str, full_ref: &str) -> Result<()> {
    gitpoulpi_core::guard::check_remote_arg("remote", remote)?;
    let ok = (full_ref.starts_with("refs/heads/") || full_ref.starts_with("refs/tags/"))
        && !full_ref.contains(char::is_whitespace);
    if !ok {
        return Err(Error::Backend(format!(
            "`{full_ref}` is not a branch or tag name"
        )));
    }
    Ok(())
}

/// What `remote` holds at `full_ref` right now (`ls-remote`), or `None`
/// when it has no such ref. For an annotated tag that is the tag object, not
/// the commit it points at.
pub async fn remote_ref_value(repo: &RepoRef, remote: &str, full_ref: &str) -> Result<Option<Oid>> {
    check(remote, full_ref)?;
    let args = ["ls-remote", remote, full_ref];
    let out = run_full_async(&repo.root, &args).await?;
    if !out.success {
        return Err(crate::process::failed_with(
            &args,
            out.exit_code,
            out.stderr.trim(),
        ));
    }
    // `ls-remote <pattern>` matches by tail, and lists an annotated tag's
    // peeled commit as `<ref>^{}`: only the exact name counts.
    for line in out.stdout.lines() {
        let Some((oid, name)) = line.split_once('\t') else {
            continue;
        };
        if name == full_ref {
            return Ok(Oid::from_hex(oid.trim()));
        }
    }
    Ok(None)
}

/// Moves `full_ref` on `remote` from `expect` to `new`: `Some` pushes that
/// object there (re-creating the ref when `expect` is `None`), `None` deletes
/// it. The remote must hold `expect` (a lease) or [`Error::PushRejected`]
/// with `StaleLease` comes back and nothing changed. Pushing an object this
/// repository no longer has is [`UndoRefusal::RemoteObjectMissing`].
///
/// A branch's remote-tracking ref follows by itself (the push writes it); a
/// tag's mirror ref (see `tag_push`) is kept here.
pub async fn remote_ref_update(
    repo: &RepoRef,
    remote: &str,
    full_ref: &str,
    new: Option<Oid>,
    expect: Option<Oid>,
) -> Result<()> {
    check(remote, full_ref)?;
    if let Some(oid) = new {
        let probe = format!("{}^{{object}}", oid.to_hex());
        let present = run_status_async(&repo.root, &["cat-file", "-e", &probe]).await?;
        if !present.success {
            return Err(Error::UndoRefused(UndoRefusal::RemoteObjectMissing { oid }));
        }
    }
    let lease = format!(
        "--force-with-lease={full_ref}:{}",
        expect.map(|o| o.to_hex()).unwrap_or_default()
    );
    let refspec = new.map(|oid| format!("{}:{full_ref}", oid.to_hex()));
    let mut args: Vec<&str> = vec!["push", "--porcelain", &lease, remote];
    match &refspec {
        Some(spec) => args.push(spec),
        None => {
            args.push("--delete");
            args.push(full_ref);
        }
    }
    let out = run_full_async(&repo.root, &args).await?;
    if !out.success {
        if let Some(reason) = rejection_reason(&out.stdout, &out.stderr) {
            let detail = match crate::process::failed_with(&args, out.exit_code, &out.stderr) {
                Error::Git { detail, .. } => Some(detail),
                _ => None,
            };
            return Err(Error::PushRejected(PushRejection {
                remote: remote.to_string(),
                branch: full_ref.to_string(),
                reason,
                detail,
            }));
        }
        return Err(crate::process::failed_with(
            &args,
            out.exit_code,
            out.stderr.trim(),
        ));
    }
    if let Some(tag) = full_ref.strip_prefix("refs/tags/") {
        let mirror = remote_tag_mirror_ref(remote, tag);
        let script = match new {
            Some(oid) => format!("update {mirror} {}\n", oid.to_hex()),
            None => format!("delete {mirror}\n"),
        };
        // A cache of what the remote says: a failure here is the next
        // fetch's to repair, not the operation's.
        let _ = run_with_stdin(&repo.root, &["update-ref", "--stdin"], &script);
    }
    Ok(())
}
