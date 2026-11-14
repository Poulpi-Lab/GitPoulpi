use std::path::Path;

use gitpoulpi_core::domain::REMOTE_TAG_MIRROR_PREFIX;
use gitpoulpi_core::domain::{
    CloneOptions, ClonePhase, CloneProgress, CloneProgressSink, FetchOptions, FetchOutcome,
    PullMode, PullOptions, PullOutcome, PushLease, PushOptions, PushOutcome, PushRejection,
    RepoRef,
};
#[cfg(feature = "lfs")]
use gitpoulpi_core::domain::{LfsCloneMode, LfsDirection, LfsProgress};
use gitpoulpi_core::{Error, Result};

use crate::lfs::LfsProbe;
use crate::process::{run, run_async, run_full_async, run_status_async, run_streaming_stderr};

/// Remote-tracking `(refname, oid)` pairs for `remote`, used to diff
/// before/after a fetch since `git fetch` has no machine-readable
/// "what changed" output - a ref can keep its name but move to a new oid.
fn remote_tracking_refs(repo: &RepoRef, remote: &str) -> Result<Vec<(String, String)>> {
    refs_under(repo, &format!("refs/remotes/{remote}"))
}

/// `(refname, oid)` of every ref under `prefix`.
fn refs_under(repo: &RepoRef, prefix: &str) -> Result<Vec<(String, String)>> {
    let raw = run(
        &repo.root,
        &["for-each-ref", "--format=%(refname) %(objectname)", prefix],
    )?;
    Ok(raw
        .lines()
        .filter_map(|line| {
            let (name, oid) = line.split_once(' ')?;
            Some((name.to_string(), oid.to_string()))
        })
        .collect())
}

/// Whether the installed git has `git fetch --porcelain` (2.41+), which
/// reports every ref it touched on stdout.
fn fetch_porcelain_supported() -> bool {
    crate::process::git_at_least(2, 41)
}

/// Refs a `git fetch --porcelain` run moved: lines are
/// `<flag> <old> <new> <ref>`, where a blank flag is a fast-forward, `*` a
/// new ref and `+` a forced update. `=` (up to date), `-` (pruned), `t`
/// (tag update) and `!` (rejected) are not "updated remote-tracking refs".
fn porcelain_updated_refs(stdout: &str, prefix: &str) -> Vec<String> {
    stdout
        .lines()
        .filter_map(|line| {
            let flag = line.chars().next()?;
            if !matches!(flag, ' ' | '*' | '+') {
                return None;
            }
            let name = line.split_whitespace().last()?;
            name.starts_with(prefix).then(|| name.to_string())
        })
        .collect()
}

/// `(ref, old oid)` of every ref a `git fetch --porcelain --prune` run
/// pruned (`- <old> <new> <ref>`) under `prefix`.
fn porcelain_pruned(stdout: &str, prefix: &str) -> Vec<(String, String)> {
    stdout
        .lines()
        .filter_map(|line| {
            let mut it = line.split_whitespace();
            if it.next()? != "-" {
                return None;
            }
            let old = it.next()?;
            let name = it.last()?;
            name.starts_with(prefix)
                .then(|| (name.to_string(), old.to_string()))
        })
        .collect()
}

/// One `git fetch` of the user's own refspecs. The remote-tag mirror (see
/// `tags.rs`) is a separate fetch, `fetch_mirror`, run beside this one.
///
/// `target` is `[remote]` or `["--all"]`.
async fn fetch_once(repo: &RepoRef, target: &[&str], prune: bool) -> Result<Vec<String>> {
    let all = target[0] == "--all";
    let prefix = if all {
        "refs/remotes/".to_string()
    } else {
        format!("refs/remotes/{}/", target[0])
    };
    let porcelain = fetch_porcelain_supported();
    let scope = if all { "" } else { target[0] };
    let before = if porcelain {
        Vec::new()
    } else {
        remote_tracking_refs(repo, scope)?
    };

    let mut args: Vec<String> = Vec::new();
    args.push("fetch".into());
    if porcelain {
        args.push("--porcelain".into());
    }
    args.extend(target.iter().map(|t| t.to_string()));
    if prune {
        args.push("--prune".into());
    }
    let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
    let output = run_full_async(&repo.root, &args_ref).await?;
    if !output.success {
        return Err(crate::process::failed_with(
            &args_ref,
            output.exit_code,
            &output.stderr,
        ));
    }

    if porcelain {
        return Ok(porcelain_updated_refs(&output.stdout, &prefix));
    }
    let after = remote_tracking_refs(repo, scope)?;
    Ok(after
        .into_iter()
        .filter(|pair| !before.contains(pair))
        .map(|(name, _)| name)
        .collect())
}

/// Refreshes `remote`'s tag mirror: `git fetch --prune --no-tags <remote>
/// <mirror refspec>`. Refspecs given on the command line are the only ones
/// `--prune` considers, so this prunes the mirror and never the user's
/// remote-tracking branches (whose pruning stays the user's opt-in). A tag
/// deleted on the remote by someone else therefore stops reading "on
/// origin" after any fetch, whatever the user's prune setting.
///
/// It runs *beside* the user's fetch (`fetch_remotes` joins the two), not
/// after it: a second round trip in series doubled the time of a fetch from
/// a slow remote.
///
/// Returns the mirror refs it pruned, `(ref, old oid)` - only when
/// `want_pruned` (the caller will delete the matching local tags). Without
/// `fetch --porcelain` (git before 2.41) they are the mirror refs that
/// vanished between a listing before and one after the fetch, two more
/// spawns that only a `prune_tags` fetch pays.
async fn fetch_mirror(
    repo: &RepoRef,
    remote: &str,
    want_pruned: bool,
) -> Result<Vec<(String, String)>> {
    let spec = crate::tags::mirror_refspec(remote);
    let mut args = vec!["fetch"];
    let porcelain = fetch_porcelain_supported();
    let mirror_prefix = format!("{REMOTE_TAG_MIRROR_PREFIX}{remote}/");
    let listed_before = if want_pruned && !porcelain {
        refs_under(repo, &mirror_prefix)?
    } else {
        Vec::new()
    };
    if porcelain {
        args.push("--porcelain");
    }
    args.extend(["--prune", "--no-tags", remote, spec.as_str()]);
    let output = run_full_async(&repo.root, &args).await?;
    if !output.success {
        return Err(crate::process::failed_with(
            &args,
            output.exit_code,
            &output.stderr,
        ));
    }
    Ok(if porcelain {
        porcelain_pruned(&output.stdout, REMOTE_TAG_MIRROR_PREFIX)
    } else if want_pruned {
        let after = refs_under(repo, &mirror_prefix)?;
        listed_before
            .into_iter()
            .filter(|(name, _)| !after.iter().any(|(kept, _)| kept == name))
            .collect()
    } else {
        Vec::new()
    })
}

/// The whole of a fetch: the user's refspecs and, beside them, the tag
/// mirror of every remote; then - only when asked - the deletion of the
/// local tags the remote just lost.
///
/// A mirror failure does not fail the fetch (it is a cache: the next fetch
/// retries, and the user's fetch already answered); the user's own fetch
/// error is returned as before.
async fn fetch_remotes(
    repo: &RepoRef,
    remotes: &[String],
    target: &[&str],
    opts: &FetchOptions,
) -> Result<FetchOutcome> {
    let mirrors = async {
        let mut pruned = Vec::new();
        for remote in remotes {
            if let Ok(p) = fetch_mirror(repo, remote, opts.prune_tags).await {
                pruned.extend(p);
            }
        }
        pruned
    };
    let (updated, pruned) = tokio::join!(fetch_once(repo, target, opts.prune), mirrors);
    let updated_refs = updated?;
    let pruned_tags = if opts.prune_tags {
        crate::tags::delete_local_tags_lost_on_remote(repo, &pruned)?
    } else {
        Vec::new()
    };
    Ok(FetchOutcome {
        updated_refs,
        pruned_tags,
    })
}

/// `git fetch <remote>`, which also refreshes the remote-tag mirror: git
/// keeps no remote-tracking refs for tags, so "is this tag pushed?" has no
/// local answer otherwise. A fetch is the gesture that already means "sync
/// with the remote"; `refs()`, called on every render, never touches the
/// network.
pub async fn fetch(repo: &RepoRef, remote: &str, opts: &FetchOptions) -> Result<FetchOutcome> {
    gitpoulpi_core::guard::check_remote_arg("remote", remote)?;
    fetch_remotes(repo, &[remote.to_string()], &[remote], opts).await
}

/// `git fetch --all`, with every remote's tag mirror refreshed beside it.
pub async fn fetch_all(repo: &RepoRef, opts: &FetchOptions) -> Result<FetchOutcome> {
    let names: Vec<String> = crate::remotes::remotes(repo)?
        .into_iter()
        .map(|r| r.name)
        .collect();
    fetch_remotes(repo, &names, &["--all"], opts).await
}

/// See `GitBackend::fetch_refspec`.
pub async fn fetch_refspec(repo: &RepoRef, remote: &str, refspec: &str) -> Result<FetchOutcome> {
    let (_, dst) = gitpoulpi_core::domain::parse_fetch_refspec(refspec).ok_or_else(|| {
        Error::Backend(format!(
            "`{refspec}` is not a fetchable spec (expected refs/<src>:{}<dst>)",
            gitpoulpi_core::domain::FETCH_REFSPEC_NAMESPACE
        ))
    })?;
    gitpoulpi_core::guard::check_remote_arg("remote", remote)?;
    let forced = if refspec.starts_with('+') {
        refspec.to_string()
    } else {
        format!("+{refspec}")
    };
    run_async(
        &repo.root,
        &["fetch", "--quiet", "--no-tags", remote, &forced],
    )
    .await?;
    Ok(FetchOutcome {
        updated_refs: vec![dst.to_string()],
        pruned_tags: Vec::new(),
    })
}

/// See `GitBackend::fetch_ref_as_branch`.
pub async fn fetch_ref_as_branch(
    repo: &RepoRef,
    remote: &str,
    remote_ref: &str,
    branch: &str,
) -> Result<FetchOutcome> {
    gitpoulpi_core::guard::check_remote_arg("remote", remote)?;
    if !remote_ref.starts_with("refs/") || remote_ref.contains("..") || remote_ref.contains(' ') {
        return Err(Error::Backend(format!(
            "`{remote_ref}` is not a full ref name (expected refs/...)"
        )));
    }
    run_async(&repo.root, &["check-ref-format", "--branch", branch])
        .await
        .map_err(|_| Error::Backend(format!("`{branch}` is not a valid branch name")))?;
    let tracking = format!("refs/remotes/{remote}/{branch}");
    run_async(
        &repo.root,
        &[
            "fetch",
            "--quiet",
            remote,
            &format!("+{remote_ref}:{tracking}"),
        ],
    )
    .await?;

    let local = format!("refs/heads/{branch}");
    let exists = run_async(&repo.root, &["rev-parse", "--verify", "--quiet", &local])
        .await
        .is_ok();
    if !exists {
        run_async(&repo.root, &["branch", "--track", branch, &tracking]).await?;
    } else {
        let ff = run_async(
            &repo.root,
            &["merge-base", "--is-ancestor", &local, &tracking],
        )
        .await
        .is_ok();
        if ff {
            if current_branch(repo).await.ok().as_deref() == Some(branch) {
                run_async(&repo.root, &["merge", "--ff-only", "--quiet", &tracking]).await?;
            } else {
                run_async(&repo.root, &["branch", "-f", branch, &tracking]).await?;
            }
        }
    }
    Ok(FetchOutcome {
        updated_refs: vec![tracking],
        pruned_tags: Vec::new(),
    })
}

/// The checked-out branch's short name, or an error on a detached HEAD -
/// there is no branch to pull into or push from.
async fn current_branch(repo: &RepoRef) -> Result<String> {
    let branch = run_async(&repo.root, &["rev-parse", "--abbrev-ref", "HEAD"]).await?;
    let branch = branch.trim().to_string();
    if branch == "HEAD" {
        return Err(Error::Backend(
            "HEAD is detached: check a branch out first".into(),
        ));
    }
    Ok(branch)
}

/// The integration flag for a pull mode. Always explicit, so the user's
/// `pull.rebase` / `pull.ff` cannot turn one mode into another.
fn pull_mode_args(mode: PullMode) -> &'static [&'static str] {
    match mode {
        PullMode::Merge => &["--no-rebase", "--ff", "--no-edit"],
        PullMode::FastForwardOnly => &["--ff-only"],
        PullMode::Rebase => &["--rebase"],
    }
}

pub async fn pull(repo: &RepoRef, opts: &PullOptions) -> Result<PullOutcome> {
    if let Some(remote) = &opts.remote {
        gitpoulpi_core::guard::check_remote_arg("remote", remote)?;
    }
    let before = run_async(&repo.root, &["rev-parse", "HEAD"]).await?;

    let mut args: Vec<String> = vec!["pull".into()];
    args.extend(pull_mode_args(opts.mode).iter().map(|a| a.to_string()));
    if let Some(remote) = &opts.remote {
        let branch = match &opts.remote_branch {
            Some(b) => b.clone(),
            None => current_branch(repo).await?,
        };
        args.push(remote.clone());
        args.push(branch);
    }
    finish_pull(repo, opts, before, args).await
}

/// Where a pull takes its commits from.
struct PullSource {
    remote: String,
    /// The branch asked for with an explicit remote; `None`: the branch's
    /// configured upstream.
    branch: Option<String>,
    /// The remote-tracking ref the fork point of a rebase is looked up on.
    tracking: Option<String>,
}

/// The source `git pull` would use, or `None` when git has to say what is
/// wrong (no upstream, a detached HEAD): the split then runs the whole
/// `git pull` in the integrate step, so those errors read as they always did.
async fn pull_source(repo: &RepoRef, opts: &PullOptions) -> Result<Option<PullSource>> {
    if let Some(remote) = &opts.remote {
        gitpoulpi_core::guard::check_remote_arg("remote", remote)?;
    }
    if let Some(remote) = &opts.remote {
        let branch = match &opts.remote_branch {
            Some(b) => b.clone(),
            None => current_branch(repo).await?,
        };
        return Ok(Some(PullSource {
            remote: remote.clone(),
            tracking: Some(format!("refs/remotes/{remote}/{branch}")),
            branch: Some(branch),
        }));
    }
    let Ok(branch) = current_branch(repo).await else {
        return Ok(None);
    };
    let config = |key: &str| -> Result<Option<String>> {
        let (ok, text) = crate::process::run_lenient(
            &repo.root,
            &["config", "--get", &format!("branch.{branch}.{key}")],
        )?;
        let text = text.trim().to_string();
        Ok((ok && !text.is_empty()).then_some(text))
    };
    let (Some(remote), Some(_merge)) = (config("remote")?, config("merge")?) else {
        return Ok(None);
    };
    let (ok, tracking) = crate::process::run_lenient(
        &repo.root,
        &["rev-parse", "--symbolic-full-name", "@{upstream}"],
    )?;
    Ok(Some(PullSource {
        remote,
        branch: None,
        tracking: ok.then(|| tracking.trim().to_string()),
    }))
}

/// The download half of a pull: `git fetch <remote> [<branch>]`, the fetch
/// `git pull` makes first. It writes refs, objects and `FETCH_HEAD` only, so
/// dropping it (a canceled job) leaves the index, the working tree and `HEAD`
/// as they were.
pub async fn pull_fetch(repo: &RepoRef, opts: &PullOptions) -> Result<()> {
    let Some(source) = pull_source(repo, opts).await? else {
        return Ok(());
    };
    let mut args = vec!["fetch".to_string(), source.remote];
    args.extend(source.branch);
    let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
    let output = run_status_async(&repo.root, &args_ref).await?;
    if output.success {
        Ok(())
    } else {
        Err(crate::process::failed_with(
            &args_ref,
            output.exit_code,
            &output.stderr,
        ))
    }
}

/// The commits `FETCH_HEAD` marks for merging (the ones `git pull` merges).
fn merge_heads(repo: &RepoRef) -> Vec<String> {
    let Ok((true, path)) =
        crate::process::run_lenient(&repo.root, &["rev-parse", "--git-path", "FETCH_HEAD"])
    else {
        return Vec::new();
    };
    let path = repo.root.join(path.trim());
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines()
        .filter_map(|line| {
            let mut fields = line.split('\t');
            let oid = fields.next()?.trim();
            let marker = fields.next().unwrap_or("");
            (marker.is_empty() && !oid.is_empty()).then(|| oid.to_string())
        })
        .collect()
}

/// The integrate half of a pull, from the `FETCH_HEAD` that [`pull_fetch`]
/// wrote: no network, same outcomes and errors as [`pull`]. Past this point
/// the index and working tree are being rewritten.
pub async fn pull_integrate(repo: &RepoRef, opts: &PullOptions) -> Result<PullOutcome> {
    let source = pull_source(repo, opts).await.ok().flatten();
    let heads = merge_heads(repo);
    let (Some(source), Some(head)) = (source, heads.first()) else {
        return pull(repo, opts).await;
    };
    let before = run_async(&repo.root, &["rev-parse", "HEAD"]).await?;
    let args: Vec<String> = match opts.mode {
        PullMode::Merge => ["merge", "--ff", "--no-edit", "FETCH_HEAD"]
            .map(String::from)
            .to_vec(),
        PullMode::FastForwardOnly => ["merge", "--ff-only", "FETCH_HEAD"]
            .map(String::from)
            .to_vec(),
        PullMode::Rebase => {
            // `git pull --rebase` replays onto the fetched commit from the
            // fork point of the remote-tracking branch, when it has one.
            let fork = source.tracking.as_deref().and_then(|tracking| {
                let (ok, out) = crate::process::run_lenient(
                    &repo.root,
                    &["merge-base", "--fork-point", tracking, "HEAD"],
                )
                .ok()?;
                let out = out.trim().to_string();
                (ok && !out.is_empty()).then_some(out)
            });
            vec![
                "rebase".into(),
                "--onto".into(),
                head.clone(),
                fork.unwrap_or_else(|| head.clone()),
            ]
        }
    };
    finish_pull(repo, opts, before, args).await
}

/// Runs the integrating command `args` and reads its outcome (shared by the
/// one-step [`pull`] and [`pull_integrate`]).
async fn finish_pull(
    repo: &RepoRef,
    opts: &PullOptions,
    before: String,
    args: Vec<String>,
) -> Result<PullOutcome> {
    let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
    let output = run_status_async(&repo.root, &args_ref).await?;

    if !output.success {
        let conflicts = run_async(&repo.root, &["diff", "--name-only", "--diff-filter=U"]).await;
        if matches!(&conflicts, Ok(s) if !s.trim().is_empty()) {
            return Ok(PullOutcome::Conflicted);
        }
        // Decided from the history, never from git's (localised) stderr:
        // the fetch half of the pull wrote FETCH_HEAD, and a HEAD that is
        // not its ancestor is a divergence a fast-forward cannot settle.
        if opts.mode == PullMode::FastForwardOnly {
            let ancestor = run_status_async(
                &repo.root,
                &["merge-base", "--is-ancestor", "HEAD", "FETCH_HEAD"],
            )
            .await;
            if matches!(ancestor, Ok(out) if !out.success) {
                return Err(Error::NotFastForward);
            }
        }
        return Err(crate::process::failed_with(
            &args_ref,
            output.exit_code,
            &output.stderr,
        ));
    }

    let after = run_async(&repo.root, &["rev-parse", "HEAD"]).await?;
    if after.trim() == before.trim() {
        return Ok(PullOutcome::UpToDate);
    }
    let oid = gitpoulpi_core::domain::Oid::from_hex(after.trim())
        .ok_or_else(|| Error::Backend("HEAD was not a valid oid".into()))?;
    // A merge commit whose first parent is the old HEAD also has the old
    // HEAD as its merge base, so "merge base == before" alone reported
    // every merge pull as a fast-forward. The parents decide it.
    let parents = run_async(
        &repo.root,
        &["rev-list", "--parents", "-n", "1", after.trim()],
    )
    .await?;
    let parents: Vec<&str> = parents.split_whitespace().skip(1).collect();
    let is_new_merge = parents.len() >= 2 && parents[0] == before.trim();
    // Fast-forward = the old HEAD is an ancestor of the new one, minus the
    // merge case above. Only reached when HEAD moved, so the spawn is never
    // paid on an up-to-date pull.
    let before_is_ancestor = run_status_async(
        &repo.root,
        &["merge-base", "--is-ancestor", before.trim(), after.trim()],
    )
    .await?
    .success;
    if before_is_ancestor && !is_new_merge {
        Ok(PullOutcome::FastForwarded(oid))
    } else {
        Ok(PullOutcome::Merged(oid))
    }
}

/// `git push --porcelain <remote> refs/heads/<b>:refs/heads/<b>`.
///
/// `--porcelain` because a rejection has to be *recognized*, and the
/// porcelain status line (`!\t<src>:<dst>\t[rejected] (non-fast-forward)`)
/// is a fixed, untranslated format where the human stderr is neither.
/// The force flag is `--force-with-lease` and nothing else (see
/// `PushOptions::force_with_lease`).
pub async fn push(repo: &RepoRef, remote: &str, opts: &PushOptions) -> Result<PushOutcome> {
    gitpoulpi_core::guard::check_remote_arg("remote", remote)?;
    let branch = match &opts.branch {
        Some(b) => b.clone(),
        None => current_branch(repo).await?,
    };

    let mut args = vec!["push".to_string(), "--porcelain".to_string()];
    if opts.set_upstream {
        args.push("--set-upstream".to_string());
    }
    let dst = opts.remote_branch.clone().unwrap_or_else(|| branch.clone());
    if let Some(dst) = &opts.remote_branch {
        gitpoulpi_core::guard::check_remote_arg("remote branch", dst)?;
    }
    let lease = opts
        .lease
        .or(opts.force_with_lease.then_some(PushLease::Tracking));
    match lease {
        None => {}
        Some(PushLease::Tracking) => args.push("--force-with-lease".to_string()),
        Some(PushLease::Expect(oid)) => {
            args.push(format!(
                "--force-with-lease=refs/heads/{dst}:{}",
                oid.to_hex()
            ));
        }
        Some(PushLease::Absent) => args.push(format!("--force-with-lease=refs/heads/{dst}:")),
    }
    args.push(remote.to_string());
    args.push(format!("refs/heads/{branch}:refs/heads/{dst}"));
    let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
    let output = run_full_async(&repo.root, &args_ref).await?;

    if !output.success {
        if let Some(reason) = crate::history::push::rejection_reason(&output.stdout, &output.stderr)
        {
            let detail =
                match crate::process::failed_with(&args_ref, output.exit_code, &output.stderr) {
                    Error::Git { detail, .. } => Some(detail),
                    _ => None,
                };
            return Err(Error::PushRejected(PushRejection {
                remote: remote.to_string(),
                branch,
                reason,
                detail,
            }));
        }
        return Err(crate::process::failed_with(
            &args_ref,
            output.exit_code,
            output.stderr.trim(),
        ));
    }

    Ok(PushOutcome {
        pushed_refs: vec![branch],
    })
}

/// `git push <remote> HEAD:refs/for/<branch>[%topic=<topic>]`: how a Gerrit
/// change is created. `branch` is the target branch on the server; the
/// names are checked before git runs (a Gerrit option string is
/// `%key=value`, so `%`, spaces and the like cannot be allowed in them).
/// Never forced.
pub async fn push_for_review(
    repo: &RepoRef,
    remote: &str,
    branch: &str,
    topic: Option<&str>,
) -> Result<PushOutcome> {
    fn check(what: &str, value: &str) -> Result<()> {
        let bad = value.is_empty()
            || value.starts_with('-')
            || value.starts_with('/')
            || value.ends_with('/')
            || value.contains("..")
            || value
                .chars()
                .any(|c| c.is_whitespace() || c.is_control() || "%:?*[\\^~@{}".contains(c));
        if bad {
            return Err(Error::Backend(format!(
                "`{value}` is not a usable {what} for a review push"
            )));
        }
        Ok(())
    }
    let branch = branch.trim();
    let branch = branch.strip_prefix("refs/heads/").unwrap_or(branch);
    check("branch", branch)?;
    check("remote", remote)?;
    gitpoulpi_core::guard::check_remote_arg("remote", remote)?;
    let mut target = format!("refs/for/{branch}");
    if let Some(topic) = topic.map(str::trim).filter(|t| !t.is_empty()) {
        check("topic", topic)?;
        target.push_str(&format!("%topic={topic}"));
    }
    let refspec = format!("HEAD:{target}");
    let args = ["push", "--porcelain", remote, refspec.as_str()];
    let output = run_full_async(&repo.root, &args).await?;
    if !output.success {
        return Err(crate::process::failed_with(
            &args,
            output.exit_code,
            output.stderr.trim(),
        ));
    }
    Ok(PushOutcome {
        pushed_refs: vec![target],
    })
}

/// `git clone`'s argument list, in the one place that builds it, so the
/// progress-reporting and plain variants cannot drift apart in what they
/// actually ask git to do.
fn clone_args(url: &str, dest: &Path, opts: &CloneOptions, progress: bool) -> Vec<String> {
    let mut args = vec!["clone".to_string()];
    if progress {
        // git only emits progress when stderr is a terminal, and it is a
        // pipe here. Without this the stderr stream is silent until the
        // clone ends, which is exactly the period a progress bar exists
        // to cover.
        args.push("--progress".to_string());
    }
    if let Some(depth) = opts.shallow_depth {
        args.push("--depth".to_string());
        args.push(depth.to_string());
    }
    if opts.recurse_submodules {
        args.push("--recurse-submodules".to_string());
    }
    if opts.sparse.is_some() {
        // Root files only at first: the chosen directories are added right
        // after (`sparse_after_clone`), so the clone never writes the rest.
        args.push("--sparse".to_string());
    }
    if let Some(branch) = &opts.branch {
        args.push("--branch".to_string());
        args.push(branch.clone());
    }
    if let Some(remote) = &opts.remote_name {
        args.push("--origin".to_string());
        args.push(remote.clone());
    }
    // `--` ends the options: the address and the folder are operands.
    args.push("--".to_string());
    args.push(url.to_string());
    args.push(dest.to_string_lossy().to_string());
    args
}

/// The directory `git clone` is spawned in: `dest` does not exist yet, so
/// the child's cwd is its parent, which is created here (the user may have
/// typed a path two levels deep) rather than letting git fail with
/// `could not create leading directories`.
fn clone_cwd(dest: &Path) -> Result<std::path::PathBuf> {
    let parent = dest
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    std::fs::create_dir_all(&parent)
        .map_err(|e| Error::Backend(format!("could not create {}: {e}", parent.display())))?;
    Ok(parent)
}

/// Config that keeps the clone's own checkout from running the LFS
/// filters (`git-lfs` does the same for its internal git calls). Objects
/// are fetched afterwards by [`lfs_after_clone`] - with the progress
/// GitPoulpi can show - or deliberately not fetched (`LfsCloneMode::Skip`).
/// It also makes sure a clone never fails because `git-lfs` is missing
/// while the user's config still names it as a *required* filter.
#[cfg(feature = "lfs")]
const CLONE_WITHOUT_LFS_FILTERS: [&str; 6] = [
    "-c",
    "filter.lfs.smudge=",
    "-c",
    "filter.lfs.process=",
    "-c",
    "filter.lfs.required=false",
];

/// `clone_args` with [`CLONE_WITHOUT_LFS_FILTERS`] in front (they are
/// options of `git`, not of `git clone`).
#[cfg(feature = "lfs")]
fn clone_command_args(url: &str, dest: &Path, opts: &CloneOptions, progress: bool) -> Vec<String> {
    let mut args: Vec<String> = CLONE_WITHOUT_LFS_FILTERS
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    args.extend(clone_args(url, dest, opts, progress));
    args
}

/// Without LFS support the clone is a plain `git clone`: no filter overrides,
/// so an installed `git-lfs` smudges files as usual.
#[cfg(not(feature = "lfs"))]
fn clone_command_args(url: &str, dest: &Path, opts: &CloneOptions, progress: bool) -> Vec<String> {
    clone_args(url, dest, opts, progress)
}

/// A sparse clone's directories, validated *before* anything is cloned.
fn sparse_dirs_of(opts: &CloneOptions) -> Result<Option<Vec<String>>> {
    opts.sparse
        .as_deref()
        .map(gitpoulpi_core::domain::normalize_sparse_dirs)
        .transpose()
}

/// After `git clone --sparse`: widens the root-only checkout to the chosen
/// directories.
fn sparse_after_clone(repo: &RepoRef, dirs: Option<&[String]>) -> Result<()> {
    match dirs {
        Some(dirs) => crate::sparse::set(repo, dirs),
        None => Ok(()),
    }
}

/// After a clone: when LFS is wanted (`Auto`), usable, and the repository
/// uses it, download the objects (installing the filters for this
/// repository first if none are configured) and report it as
/// [`ClonePhase::LfsObjects`].
///
/// **Never an error.** The clone itself succeeded; a missing program or a
/// failed download leaves pointer files and the TUI's LFS status says
/// what to do about it.
#[cfg(feature = "lfs")]
async fn lfs_after_clone(
    repo: &RepoRef,
    opts: &CloneOptions,
    probe: &LfsProbe,
    progress: CloneProgressSink<'_>,
) {
    if opts.lfs != LfsCloneMode::Auto || !probe.get(&repo.root).is_usable() {
        return;
    }
    if !crate::lfs::repo_uses_lfs(repo) {
        return;
    }
    progress(CloneProgress::new(ClonePhase::LfsObjects, 0));

    // Only the download counts toward the bar: the checkout that follows
    // restarts the counters, and a bar that goes backwards reads as a bug.
    let highest = std::sync::atomic::AtomicU8::new(0);
    let sink = |p: LfsProgress| {
        if p.direction != LfsDirection::Download {
            return;
        }
        let percent = p.percent();
        if percent < highest.fetch_max(percent, std::sync::atomic::Ordering::Relaxed) {
            return;
        }
        progress(
            CloneProgress::new(ClonePhase::LfsObjects, percent)
                .with_counts(p.files_done, p.files_total),
        );
    };
    let remote = opts.remote_name.as_deref().unwrap_or("origin");
    let _ = crate::lfs_network::lfs_pull(repo, probe, remote, &sink).await;
}

#[cfg(not(feature = "lfs"))]
async fn lfs_after_clone(
    _repo: &RepoRef,
    _opts: &CloneOptions,
    _probe: &LfsProbe,
    _progress: CloneProgressSink<'_>,
) {
}

pub async fn clone(
    url: &str,
    dest: &Path,
    opts: &CloneOptions,
    probe: &LfsProbe,
) -> Result<RepoRef> {
    gitpoulpi_core::guard::check_remote_arg("clone address", url)?;
    let sparse = sparse_dirs_of(opts)?;
    let args = clone_command_args(url, dest, opts, false);
    let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
    let cwd = clone_cwd(dest)?;
    // Dropped mid-clone (a canceled job): git and its helpers are killed and
    // the destination is put back as it was.
    let mut undo = crate::cancel_cleanup::CloneCleanup::new(dest);
    let cloned = run_async(&cwd, &args_ref).await;
    undo.disarm();
    cloned?;
    let repo = cloned_ref(dest);
    sparse_after_clone(&repo, sparse.as_deref())?;
    lfs_after_clone(&repo, opts, probe, &|_| {}).await;
    Ok(repo)
}

pub async fn clone_with_progress(
    url: &str,
    dest: &Path,
    opts: &CloneOptions,
    probe: &LfsProbe,
    progress: CloneProgressSink<'_>,
) -> Result<RepoRef> {
    gitpoulpi_core::guard::check_remote_arg("clone address", url)?;
    let sparse = sparse_dirs_of(opts)?;
    let args = clone_command_args(url, dest, opts, true);
    let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
    let cwd = clone_cwd(dest)?;

    progress(CloneProgress::new(ClonePhase::Connecting, 0));

    // Monotonic within a phase: git can re-report a lower percentage when
    // it restarts a counter, and a bar that jumps backwards reads as a
    // bug. Tracked here rather than in the parser so the parser stays a
    // pure function of one chunk.
    let mut last: Option<CloneProgress> = None;
    // Dropped mid-clone (a canceled job): git and its helpers are killed and
    // the destination is put back as it was. Not armed past the `git clone`
    // itself: a canceled LFS download keeps the (valid) clone.
    let mut undo = crate::cancel_cleanup::CloneCleanup::new(dest);
    let cloned = run_streaming_stderr(&cwd, &args_ref, |chunk| {
        for report in parse_clone_progress(chunk) {
            let keep = match last {
                Some(prev) if prev.phase == report.phase => report.percent >= prev.percent,
                _ => true,
            };
            if keep {
                last = Some(report);
                progress(report);
            }
        }
    })
    .await;
    undo.disarm();
    cloned?;

    let repo = cloned_ref(dest);
    sparse_after_clone(&repo, sparse.as_deref())?;
    lfs_after_clone(&repo, opts, probe, progress).await;
    progress(CloneProgress::done());
    Ok(repo)
}

fn cloned_ref(dest: &Path) -> RepoRef {
    RepoRef::from_root(dest)
}

/// Turns a chunk of `git clone --progress` stderr into progress reports.
///
/// git writes lines like `Receiving objects:  45% (450/1000), ...`
/// separated by `\r`, so a chunk can hold several updates, a partial one,
/// or none. A partial trailing update is dropped: the next chunk carries a
/// newer one. Pure, so the phase table and percent parsing are unit-testable.
fn parse_clone_progress(chunk: &str) -> Vec<CloneProgress> {
    const PHASES: &[(&str, ClonePhase)] = &[
        ("remote: Counting objects:", ClonePhase::Counting),
        ("Counting objects:", ClonePhase::Counting),
        ("remote: Compressing objects:", ClonePhase::Compressing),
        ("Compressing objects:", ClonePhase::Compressing),
        ("Receiving objects:", ClonePhase::Receiving),
        ("Resolving deltas:", ClonePhase::Resolving),
        ("Updating files:", ClonePhase::CheckingOut),
        ("Checking out files:", ClonePhase::CheckingOut),
    ];

    let mut out = Vec::new();
    for raw in chunk.split(['\r', '\n']) {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let Some((prefix, phase)) = PHASES.iter().find(|(prefix, _)| line.starts_with(prefix))
        else {
            continue;
        };
        // Strip the matched prefix by *length*, not by splitting on the first
        // `:`: `remote: Counting objects:  50%` has two colons, and the
        // split would leave `Counting objects:  50%`, whose percent parse
        // fails.
        let rest = &line[prefix.len()..];
        let Some(percent) = rest
            .split('%')
            .next()
            .map(str::trim)
            .and_then(|n| n.parse::<u32>().ok())
        else {
            continue;
        };
        let mut report = CloneProgress::new(*phase, percent.min(100) as u8);
        if let Some(counts) = rest
            .split_once('(')
            .and_then(|(_, r)| r.split_once(')'))
            .map(|(c, _)| c)
        {
            if let Some((cur, total)) = counts.split_once('/') {
                if let (Ok(cur), Ok(total)) = (cur.trim().parse(), total.trim().parse()) {
                    report = report.with_counts(cur, total);
                }
            }
        }
        out.push(report);
    }
    out
}

pub fn remote_add(repo: &RepoRef, name: &str, url: &str) -> Result<()> {
    gitpoulpi_core::guard::check_remote_arg("remote name", name)?;
    gitpoulpi_core::guard::check_remote_arg("remote address", url)?;
    run(&repo.root, &["remote", "add", "--", name, url])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::PushRejectReason;

    /// The three rejections that are the user's to resolve are typed;
    /// a hook refusing the push (`[remote rejected]`) and a success are
    /// not. The lines are real `git push --porcelain` output.
    #[test]
    fn porcelain_rejections_are_recognized_and_nothing_else_is() {
        let line = |summary: &str| {
            format!("To ../origin.git\n!\trefs/heads/main:refs/heads/main\t{summary}\nDone\n")
        };
        assert_eq!(
            crate::history::push::rejection_reason(&line("[rejected] (non-fast-forward)"), ""),
            Some(PushRejectReason::NonFastForward)
        );
        assert_eq!(
            crate::history::push::rejection_reason(&line("[rejected] (fetch first)"), ""),
            Some(PushRejectReason::NonFastForward)
        );
        assert_eq!(
            crate::history::push::rejection_reason(&line("[rejected] (stale info)"), ""),
            Some(PushRejectReason::StaleLease)
        );
        assert_eq!(
            crate::history::push::rejection_reason(
                &line("[remote rejected] (pre-receive hook declined)"),
                ""
            ),
            Some(PushRejectReason::RemoteRefused)
        );
        assert_eq!(
            crate::history::push::rejection_reason(
                "To ../o.git\n \trefs/heads/main:refs/heads/main\t1a2b..3c4d\nDone\n",
                ""
            ),
            None
        );
    }

    #[test]
    fn a_protected_branch_refusal_is_typed_from_the_hosts_words() {
        let porcelain = "!\trefs/heads/main:refs/heads/main\t[remote rejected] (protected branch hook declined)\n";
        assert_eq!(
            crate::history::push::rejection_reason(porcelain, ""),
            Some(PushRejectReason::ProtectedByHost)
        );
    }

    #[test]
    fn a_chunk_of_real_git_progress_becomes_reports() {
        let chunk = "Receiving objects:  45% (450/1000), 1.20 MiB | 3.00 MiB/s\r\
                     Receiving objects: 100% (1000/1000), 2.40 MiB | 3.00 MiB/s, done.\r\n\
                     Resolving deltas:  10% (12/120)\r";
        let got = parse_clone_progress(chunk);
        assert_eq!(got.len(), 3);
        assert_eq!(got[0].phase, ClonePhase::Receiving);
        assert_eq!(got[0].percent, 45);
        assert_eq!(got[0].current, Some(450));
        assert_eq!(got[0].total, Some(1000));
        assert_eq!(got[1].percent, 100);
        assert_eq!(got[2].phase, ClonePhase::Resolving);
        assert_eq!(got[2].current, Some(12));
    }

    /// The `remote:`-prefixed variants are the server's side of the same
    /// phases, and git prints both spellings in one clone.
    #[test]
    fn the_remote_prefixed_phases_are_recognized() {
        let got = parse_clone_progress(
            "remote: Counting objects:  50% (5/10)\rremote: Compressing objects: 100% (10/10), done.\r",
        );
        assert_eq!(got[0].phase, ClonePhase::Counting);
        assert_eq!(got[1].phase, ClonePhase::Compressing);
    }

    /// Everything git says that is not progress - warnings, the
    /// `Cloning into` banner, a partial line at the end of a chunk --
    /// must produce nothing rather than a bogus 0%.
    #[test]
    fn non_progress_output_produces_no_reports() {
        for noise in [
            "Cloning into 'thing'...\n",
            "warning: redirecting to https://example.invalid/thing.git\n",
            "Receiving objec", // torn mid-word across a read boundary
            "",
        ] {
            assert!(
                parse_clone_progress(noise).is_empty(),
                "{noise:?} must not parse as progress"
            );
        }
    }

    /// `--branch` and `--origin` have to reach git, and `--progress` has
    /// to be absent from the non-progress spelling (it would put git's
    /// counters into stderr that nothing is reading).
    #[test]
    fn the_option_mapping_is_the_one_git_understands() {
        let opts = CloneOptions {
            shallow_depth: Some(1),
            recurse_submodules: true,
            branch: Some("side".into()),
            remote_name: Some("upstream".into()),
            ..CloneOptions::default()
        };
        let args = clone_args("url", Path::new("dest"), &opts, true);
        assert!(args.windows(2).any(|w| w == ["--depth", "1"]));
        assert!(args.windows(2).any(|w| w == ["--branch", "side"]));
        assert!(args.windows(2).any(|w| w == ["--origin", "upstream"]));
        assert!(args.iter().any(|a| a == "--recurse-submodules"));
        assert!(args.iter().any(|a| a == "--progress"));

        let plain = clone_args("url", Path::new("dest"), &CloneOptions::default(), false);
        assert!(!plain.iter().any(|a| a == "--progress"));
        assert_eq!(plain, vec!["clone", "--", "url", "dest"]);
    }

    /// The command actually run is preceded by the config that keeps the
    /// clone's checkout from running LFS filters - as options of `git`
    /// (before `clone`), or a missing `git-lfs` could fail the clone.
    #[test]
    #[cfg(feature = "lfs")]
    fn the_lfs_filters_are_switched_off_for_the_clone_itself() {
        let args = clone_command_args("url", Path::new("dest"), &CloneOptions::default(), false);
        let clone_at = args.iter().position(|a| a == "clone").unwrap();
        for setting in [
            "filter.lfs.smudge=",
            "filter.lfs.process=",
            "filter.lfs.required=false",
        ] {
            let at = args.iter().position(|a| a == setting).unwrap();
            assert!(at < clone_at, "{setting} must precede `clone`");
            assert_eq!(args[at - 1], "-c");
        }
        assert_eq!(&args[clone_at..], ["clone", "--", "url", "dest"]);
    }

    /// Custom build without LFS: a plain clone, no `-c filter.lfs.*`.
    #[cfg(not(feature = "lfs"))]
    #[test]
    fn a_clone_without_lfs_passes_no_lfs_filter_config() {
        let args = clone_command_args("url", Path::new("dest"), &CloneOptions::default(), false);
        assert_eq!(args, ["clone", "--", "url", "dest"]);
        assert!(!args.iter().any(|a| a.contains("filter.lfs")));
    }
}
