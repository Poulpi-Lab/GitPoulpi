use gitpoulpi_core::domain::{
    CherryPickOptions, CherryPickOutcome, MergeOptions, MergeOutcome, Oid, RebaseOptions,
    RebaseOutcome, Ref, RefKind, RepoRef, RevertOutcome,
};
use gitpoulpi_core::{Error, Result};

use crate::process::{run, run_status};

fn ref_name(kind: &RefKind) -> String {
    match kind {
        RefKind::LocalBranch { name } => name.clone(),
        RefKind::RemoteBranch { remote, name } => format!("{remote}/{name}"),
        RefKind::Tag { name } => name.clone(),
    }
}

pub(crate) fn head_oid(repo: &RepoRef) -> Result<Oid> {
    let head = run(&repo.root, &["rev-parse", "HEAD"])?;
    Oid::from_hex(head.trim()).ok_or_else(|| Error::Backend("HEAD was not a valid oid".into()))
}

pub(crate) fn conflicted_paths(repo: &RepoRef) -> Result<Vec<String>> {
    let raw = run(&repo.root, &["diff", "--name-only", "--diff-filter=U"])?;
    Ok(raw.lines().map(str::to_string).collect())
}

pub fn merge(repo: &RepoRef, from: &Ref, opts: &MergeOptions) -> Result<MergeOutcome> {
    let name = ref_name(&from.kind);
    let before = head_oid(repo)?;

    if opts.squash {
        return squash_merge(repo, &name, opts);
    }
    let mut args = vec!["merge".to_string(), name];
    if opts.fast_forward_only {
        args.push("--ff-only".to_string());
    }
    if opts.no_ff {
        args.push("--no-ff".to_string());
    }
    if let Some(message) = opts.message.as_deref().filter(|m| !m.trim().is_empty()) {
        args.push("-m".to_string());
        args.push(message.to_string());
    }
    if opts.no_commit {
        args.push("--no-commit".to_string());
    } else {
        args.push("--no-edit".to_string());
    }
    let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
    let output = run_status(&repo.root, &args_ref)?;

    if !output.success {
        let conflicts = conflicted_paths(repo)?;
        if !conflicts.is_empty() {
            return Ok(MergeOutcome::Conflicted(conflicts));
        }
        // `--ff-only` on a branch that cannot fast-forward: the typed reading
        // (asked of the graph, not of git's wording, which is translated).
        if opts.fast_forward_only
            && !crate::branch::is_ancestor(repo, &before.to_hex(), &from.target.to_hex())?
        {
            return Err(Error::NotFastForward);
        }
        return Err(Error::Backend(format!(
            "git merge failed: {}",
            output.stderr
        )));
    }

    if opts.no_commit {
        // left staged for the caller to commit -- report as merged at the
        // target commit since no new commit oid exists yet.
        return Ok(MergeOutcome::Merged(from.target));
    }

    let after = head_oid(repo)?;
    if after == before {
        return Ok(MergeOutcome::UpToDate);
    }
    if after == from.target {
        return Ok(MergeOutcome::FastForwarded(after));
    }
    Ok(MergeOutcome::Merged(after))
}

/// `git merge --squash`, then one commit with the squash message. Conflicts
/// stop it with the index conflicted and no commit (there is no `MERGE_HEAD`
/// for a squash); an empty squash is `UpToDate`.
fn squash_merge(repo: &RepoRef, name: &str, opts: &MergeOptions) -> Result<MergeOutcome> {
    let output = run_status(&repo.root, &["merge", "--squash", name])?;
    if !output.success {
        let conflicts = conflicted_paths(repo)?;
        if !conflicts.is_empty() {
            return Ok(MergeOutcome::Conflicted(conflicts));
        }
        return Err(Error::Backend(format!(
            "git merge --squash failed: {}",
            output.stderr
        )));
    }
    if opts.no_commit {
        return Ok(MergeOutcome::Merged(head_oid(repo)?));
    }
    let staged = run_status(&repo.root, &["diff", "--cached", "--quiet"])?;
    if staged.success {
        return Ok(MergeOutcome::UpToDate);
    }
    let message = opts
        .message
        .as_deref()
        .filter(|m| !m.trim().is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| format!("Squashed commit of '{name}'"));
    run(&repo.root, &["commit", "-m", &message])?;
    Ok(MergeOutcome::Merged(head_oid(repo)?))
}

pub fn rebase(repo: &RepoRef, onto: &Ref, opts: &RebaseOptions) -> Result<RebaseOutcome> {
    let name = ref_name(&onto.kind);
    let output = if opts.autosquash {
        // No explicit plan: let git itself do the autosquash reordering
        // (`--autosquash`), accepting whatever todo list it built by
        // never opening an editor for it.
        crate::process::run_status_with_env(
            &repo.root,
            &["rebase", "-i", "--autosquash", &name],
            &[("GIT_SEQUENCE_EDITOR", ":"), ("GIT_EDITOR", ":")],
        )?
    } else {
        run_status(&repo.root, &["rebase", &name])?
    };

    if output.success {
        return Ok(RebaseOutcome::Completed);
    }

    let paused_at = run(&repo.root, &["rev-parse", "REBASE_HEAD"])
        .ok()
        .and_then(|s| Oid::from_hex(s.trim()));
    match paused_at {
        Some(paused_at) => Ok(RebaseOutcome::Conflicted {
            paused_at,
            remaining: Vec::new(),
        }),
        None => Err(Error::Backend(format!(
            "git rebase failed: {}",
            output.stderr
        ))),
    }
}

pub fn cherry_pick(
    repo: &RepoRef,
    commits: &[Oid],
    opts: &CherryPickOptions,
) -> Result<CherryPickOutcome> {
    let mut applied = Vec::new();
    for (i, commit) in commits.iter().enumerate() {
        let hex = commit.to_hex();
        let mut args = vec!["cherry-pick", hex.as_str()];
        if opts.no_commit {
            args.push("--no-commit");
        }
        let output = run_status(&repo.root, &args)?;
        if !output.success {
            let conflicts = conflicted_paths(repo)?;
            if !conflicts.is_empty() {
                return Ok(CherryPickOutcome::Conflicted {
                    paused_at: *commit,
                    remaining: commits[i + 1..].to_vec(),
                });
            }
            return Err(Error::Backend(format!(
                "git cherry-pick failed: {}",
                output.stderr
            )));
        }
        if !opts.no_commit {
            applied.push(head_oid(repo)?);
        }
    }
    Ok(CherryPickOutcome::Completed(applied))
}

pub fn revert(repo: &RepoRef, commit: Oid, mainline: Option<u32>) -> Result<RevertOutcome> {
    let hex = commit.to_hex();
    let parent = mainline.map(|n| n.to_string());
    let mut args = vec!["revert", "--no-edit"];
    if let Some(n) = parent.as_ref() {
        args.extend(["-m", n]);
    }
    args.push(&hex);
    let output = run_status(&repo.root, &args)?;
    if !output.success {
        let conflicts = conflicted_paths(repo)?;
        if !conflicts.is_empty() {
            return Ok(RevertOutcome::Conflicted);
        }
        return Err(Error::Backend(format!(
            "git revert failed: {}",
            output.stderr
        )));
    }
    Ok(RevertOutcome::Completed(head_oid(repo)?))
}
