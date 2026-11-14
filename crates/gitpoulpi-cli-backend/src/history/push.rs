//! Pushing a rewrite: `git push --porcelain --atomic
//! --force-with-lease=<ref>:<expected> ... <remote> <new>:<ref> ...`, one push
//! per remote.
//!
//! The lease of each ref is the remote id the preview recorded
//! ([`PushState::OnRemote::remote_oid`]), never "whatever the tracking ref
//! says now": an auto-fetch between the preview and the push would otherwise
//! lease against the new remote state and overwrite a collaborator's push.
//! There is no plain `--force` in this file.
//!
//! The same-named remote ref is assumed (`refs/heads/x` on both sides). A
//! branch whose upstream has another name fails safe: the lease names a ref
//! the remote does not hold at that id, and git refuses.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use gitpoulpi_core::domain::rewrite::{
    PushRefResult, PushRefStatus, PushRewritten, PushRewrittenOutcome, PushState, PushedRef,
    RefChange, PUSHED_FILE,
};
use gitpoulpi_core::domain::{Oid, PushRejectReason, RepoRef};
use gitpoulpi_core::Result;

use super::{git_dir, store};
use crate::process::{failed_with, run_full_async_english, run_status_async};

pub(crate) async fn push_rewritten(
    repo: &RepoRef,
    req: &PushRewritten,
) -> Result<PushRewrittenOutcome> {
    if !store::valid_run(&req.run) {
        return Err(super::stale("the request names no run"));
    }
    let git_dir = git_dir(&repo.root)?;
    let run = store::resolve_run(&git_dir, &req.run)?;
    let dir = store::run_dir(&git_dir, &run);
    let moved = store::read_moved(&dir)
        .ok_or_else(|| super::stale("this run was not applied here, or its files are gone"))?;

    // Which refs, in which remote, with which lease.
    let mut results: Vec<PushRefResult> = Vec::new();
    let mut by_remote: Vec<(String, Vec<Planned>)> = Vec::new();
    let tracked = tracking_refs(repo)?;
    for change in &moved {
        let PushState::OnRemote {
            remote, remote_oid, ..
        } = &change.pushed
        else {
            continue;
        };
        if req.remote.as_ref().is_some_and(|only| only != remote) {
            continue;
        }
        // Unticked by the user: not part of this push, not in the answer.
        if !req.selects(&change.name) {
            continue;
        }
        // A branch the remote does not have by that name was never "on" it
        // (the preview falls back to some other tracking ref for the remote's
        // name): not ours to push.
        let on_remote = is_on_remote(&tracked, change.name.as_str(), remote);
        if !on_remote {
            continue;
        }
        let mut result = PushRefResult {
            name: change.name.clone(),
            remote: remote.clone(),
            expected_old: *remote_oid,
            new: change.new,
            status: PushRefStatus::NotPushed,
            detail: None,
        };
        // `protected` was decided by the preview (the config's patterns and the
        // workflow's protected bases): the push honours the same flag.
        if change.protected && !req.includes(&change.name) {
            result.status = PushRefStatus::SkippedProtected;
            results.push(result);
            continue;
        }
        if !still_there(repo, change).await? {
            result.detail = Some(
                "the local ref no longer holds the rewritten commit (undone or changed since)"
                    .into(),
            );
            results.push(result);
            continue;
        }
        let slot = results.len();
        results.push(result);
        match by_remote.iter_mut().find(|(r, _)| r == remote) {
            Some((_, list)) => list.push(Planned { slot }),
            None => by_remote.push((remote.clone(), vec![Planned { slot }])),
        }
    }

    let mut atomic_fallback = false;
    for (remote, planned) in &by_remote {
        let used_fallback = push_remote(repo, req, remote, planned, &mut results).await?;
        atomic_fallback |= used_fallback;
    }

    if !req.dry_run {
        let pushed: Vec<PushedRef> = results
            .iter()
            .filter(|r| r.status == PushRefStatus::Pushed)
            .filter_map(|r| {
                Some(PushedRef {
                    name: r.name.clone(),
                    remote: r.remote.clone(),
                    new: r.new?,
                })
            })
            .collect();
        record_pushed(&git_dir, pushed);
    }
    Ok(PushRewrittenOutcome {
        run,
        dry_run: req.dry_run,
        results,
        atomic_fallback,
    })
}

struct Planned {
    /// Index in the results.
    slot: usize,
}

/// Whether `remote` has the ref under that name: a branch with a same-named
/// remote-tracking ref, or a tag (known from the tag mirror).
pub(crate) fn is_on_remote(tracked: &HashSet<String>, name: &str, remote: &str) -> bool {
    match name.strip_prefix("refs/heads/") {
        Some(short) => tracked.contains(&format!("refs/remotes/{remote}/{short}")),
        None => name.starts_with("refs/tags/"),
    }
}

/// The names of `moved` that `remote` has, for the record.
pub(crate) fn on_remote_names(repo: &RepoRef, moved: &[RefChange]) -> Result<Vec<String>> {
    let tracked = tracking_refs(repo)?;
    Ok(moved
        .iter()
        .filter(|c| match &c.pushed {
            PushState::OnRemote { remote, .. } => is_on_remote(&tracked, &c.name, remote),
            PushState::LocalOnly => false,
        })
        .map(|c| c.name.clone())
        .collect())
}

/// Every remote-tracking ref, one `for-each-ref`.
fn tracking_refs(repo: &RepoRef) -> Result<HashSet<String>> {
    let out = crate::process::run(
        &repo.root,
        &["for-each-ref", "--format=%(refname)", "refs/remotes"],
    )?;
    Ok(out.lines().map(str::to_string).collect())
}

/// Whether the local ref still is the rewritten one (or has it in its
/// history): pushing a rewrite that was undone since would be pushing
/// nothing anybody asked for.
async fn still_there(repo: &RepoRef, change: &RefChange) -> Result<bool> {
    let Some(new) = change.new else {
        return Ok(true);
    };
    let name = change.name.as_str();
    let new_hex = new.to_hex();
    let is_tag = name.starts_with("refs/tags/");
    let args: Vec<&str> = if is_tag {
        vec!["rev-parse", "--verify", "--quiet", name]
    } else {
        vec!["merge-base", "--is-ancestor", new_hex.as_str(), name]
    };
    if is_tag {
        let out = crate::process::run_lenient(&repo.root, &args)?;
        return Ok(out.0 && out.1.trim() == new_hex);
    }
    Ok(run_status_async(&repo.root, &args).await?.success)
}

/// One push for `remote`. Returns whether the atomic push had to be given
/// up for one push per ref.
async fn push_remote(
    repo: &RepoRef,
    req: &PushRewritten,
    remote: &str,
    planned: &[Planned],
    results: &mut [PushRefResult],
) -> Result<bool> {
    if planned.is_empty() {
        return Ok(false);
    }
    let atomic = req.atomic && planned.len() > 1;
    let whole = run_push(repo, req, remote, planned, atomic, results).await?;
    match whole {
        PushRun::Done => Ok(false),
        PushRun::NoAtomic => {
            // The server cannot do all-or-nothing: one push per ref, each
            // with its own lease, so one stale ref blocks only itself.
            for one in planned {
                let single = std::slice::from_ref(one);
                run_push(repo, req, remote, single, false, results).await?;
            }
            Ok(true)
        }
    }
}

enum PushRun {
    Done,
    NoAtomic,
}

async fn run_push(
    repo: &RepoRef,
    req: &PushRewritten,
    remote: &str,
    planned: &[Planned],
    atomic: bool,
    results: &mut [PushRefResult],
) -> Result<PushRun> {
    let mut args: Vec<String> = vec!["push".into(), "--porcelain".into()];
    if atomic {
        args.push("--atomic".into());
    }
    if req.dry_run {
        args.push("--dry-run".into());
    }
    for p in planned {
        let r = &results[p.slot];
        args.push(format!(
            "--force-with-lease={}:{}",
            r.name,
            r.expected_old.to_hex()
        ));
    }
    args.push(remote.to_string());
    for p in planned {
        let r = &results[p.slot];
        match r.new {
            Some(new) => args.push(format!("{}:{}", new.to_hex(), r.name)),
            None => args.push(format!(":{}", r.name)),
        }
    }
    let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
    let out = run_full_async_english(&repo.root, &args_ref).await?;

    if atomic && !out.success && out.stderr.contains("does not support --atomic") {
        return Ok(PushRun::NoAtomic);
    }
    let lines = parse(&out.stdout);
    let mut seen = 0;
    for p in planned {
        let name = results[p.slot].name.clone();
        let Some(line) = lines.iter().find(|l| l.to == name) else {
            continue;
        };
        seen += 1;
        let (status, detail) = classify(line, req.dry_run, &out.stderr);
        results[p.slot].status = status;
        results[p.slot].detail = detail;
    }
    if seen == 0 && !out.success {
        // No ref was even tried: the remote is unreachable, the credentials
        // were refused, a hook stopped the push. The command's own error.
        return Err(failed_with(&args_ref, out.exit_code, out.stderr.trim()));
    }
    if seen < planned.len() {
        let why = out.stderr.trim().to_string();
        for p in planned {
            let r = &mut results[p.slot];
            if r.status == PushRefStatus::NotPushed && r.detail.is_none() && !why.is_empty() {
                r.detail = Some(why.clone());
            }
        }
    }
    Ok(PushRun::Done)
}

/// A ref line of `git push --porcelain`:
/// `<flag> TAB <from>:<to> TAB <summary> [(<reason>)]`.
#[derive(Debug, PartialEq, Eq)]
struct PorcelainLine {
    flag: char,
    to: String,
    summary: String,
}

fn parse(porcelain: &str) -> Vec<PorcelainLine> {
    porcelain
        .lines()
        .filter_map(|line| {
            let mut fields = line.splitn(3, '\t');
            let flag = fields.next()?;
            let refs = fields.next()?;
            let summary = fields.next().unwrap_or_default();
            let mut chars = flag.chars();
            let flag = chars.next().unwrap_or(' ');
            if chars.next().is_some() || !" +-*=!".contains(flag) {
                return None;
            }
            let (_, to) = refs.split_once(':')?;
            Some(PorcelainLine {
                flag,
                to: to.to_string(),
                summary: summary.to_string(),
            })
        })
        .collect()
}

/// What a host says when its branch protection refuses a push. GitHub puts
/// it in the porcelain summary; GitLab, Azure DevOps and Bitbucket only say
/// `pre-receive hook declined` there and explain on stderr (`remote: ...`),
/// so a hook-declined line is read together with the remote's own words.
fn says_protected(text: &str) -> bool {
    text.contains("protected")
        || text.contains("gh006")
        || text.contains("gh013")
        || text.contains("not allowed to force push")
        || text.contains("tf402455")
        || text.contains("pushes to this branch are not permitted")
        || text.contains("branch restriction")
}

/// The one reading of a failed single-ref push, shared with the plain push
/// (`network::push`) so the two cannot diverge: the first refused ref of
/// `porcelain` as a typed reason, `stderr` being the remote's own words.
pub(crate) fn rejection_reason(porcelain: &str, stderr: &str) -> Option<PushRejectReason> {
    parse(porcelain)
        .iter()
        .filter(|line| line.flag == '!')
        .find_map(|line| {
            let (status, _) = classify(line, false, stderr);
            match status {
                PushRefStatus::RejectedProtectedByHost => Some(PushRejectReason::ProtectedByHost),
                PushRefStatus::RejectedRemoteMoved => {
                    if line.summary.to_ascii_lowercase().contains("stale info") {
                        Some(PushRejectReason::StaleLease)
                    } else {
                        Some(PushRejectReason::NonFastForward)
                    }
                }
                PushRefStatus::RejectedOther => Some(PushRejectReason::RemoteRefused),
                _ => None,
            }
        })
}

fn classify(line: &PorcelainLine, dry_run: bool, stderr: &str) -> (PushRefStatus, Option<String>) {
    match line.flag {
        '=' => (PushRefStatus::UpToDate, None),
        '!' => {
            let summary = line.summary.trim();
            let lower = summary.to_ascii_lowercase();
            let protected = says_protected(&lower)
                || (lower.contains("hook declined")
                    && says_protected(&stderr.to_ascii_lowercase()));
            let status = if lower.contains("atomic push failed") {
                return (
                    PushRefStatus::NotPushed,
                    Some("not sent: another ref of the same atomic push was refused".into()),
                );
            } else if protected {
                PushRefStatus::RejectedProtectedByHost
            } else if lower.contains("stale info")
                || lower.contains("fetch first")
                || lower.contains("non-fast-forward")
            {
                PushRefStatus::RejectedRemoteMoved
            } else {
                PushRefStatus::RejectedOther
            };
            (status, Some(summary.to_string()))
        }
        _ if dry_run => (PushRefStatus::WouldPush, None),
        _ => (PushRefStatus::Pushed, None),
    }
}

/// Appends what was pushed to `<git-dir>/gitpoulpi/rewrite/pushed.json`, the
/// record a later undo of the rewrite reads. Best effort: failing to write it
/// must not turn a successful push into an error.
fn record_pushed(git_dir: &Path, new: Vec<PushedRef>) {
    if new.is_empty() {
        return;
    }
    let path = git_dir.join("gitpoulpi").join("rewrite").join(PUSHED_FILE);
    let mut all: Vec<PushedRef> = std::fs::read(&path)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    let mut index: HashMap<(String, Oid), usize> = HashMap::new();
    for (i, p) in all.iter().enumerate() {
        index.insert((p.name.clone(), p.new), i);
    }
    for p in new {
        if !index.contains_key(&(p.name.clone(), p.new)) {
            all.push(p);
        }
    }
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(text) = serde_json::to_vec(&all) {
        let _ = std::fs::write(path, text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn porcelain_lines_are_read_by_flag_and_destination() {
        let text = "To /tmp/o.git\n \trefs/heads/a:refs/heads/a\t1..2\n\
                    !\trefs/heads/b:refs/heads/b\t[rejected] (stale info)\n\
                    !\trefs/heads/c:refs/heads/c\t[remote rejected] (protected branch hook declined)\n\
                    =\trefs/heads/d:refs/heads/d\t[up to date]\nDone\n";
        let lines = parse(text);
        assert_eq!(lines.len(), 4);
        assert_eq!(classify(&lines[0], false, "").0, PushRefStatus::Pushed);
        assert_eq!(classify(&lines[0], true, "").0, PushRefStatus::WouldPush);
        assert_eq!(
            classify(&lines[1], false, "").0,
            PushRefStatus::RejectedRemoteMoved
        );
        assert_eq!(
            classify(&lines[2], false, "").0,
            PushRefStatus::RejectedProtectedByHost
        );
        assert_eq!(classify(&lines[3], false, "").0, PushRefStatus::UpToDate);
    }

    #[test]
    fn a_hook_declined_line_is_read_with_the_remotes_own_words() {
        let line = PorcelainLine {
            flag: '!',
            to: "refs/heads/main".into(),
            summary: "[remote rejected] (pre-receive hook declined)".into(),
        };
        let gitlab = "remote: GitLab: You are not allowed to force push code to a protected branch on this project.";
        assert_eq!(
            classify(&line, false, gitlab).0,
            PushRefStatus::RejectedProtectedByHost
        );
        let azure = "remote: error: TF402455: Pushes to this branch are not permitted; you must use a pull request";
        assert_eq!(
            classify(&line, false, azure).0,
            PushRefStatus::RejectedProtectedByHost
        );
        assert_eq!(
            classify(&line, false, "remote: lint failed").0,
            PushRefStatus::RejectedOther,
            "an ordinary hook is not the host's protection"
        );
    }

    #[test]
    fn an_atomic_victim_is_not_pushed_rather_than_rejected() {
        let line = PorcelainLine {
            flag: '!',
            to: "refs/heads/x".into(),
            summary: "[rejected] (atomic push failed)".into(),
        };
        assert_eq!(classify(&line, false, "").0, PushRefStatus::NotPushed);
    }
}
