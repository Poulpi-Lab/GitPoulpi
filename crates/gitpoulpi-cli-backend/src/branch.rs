use gitpoulpi_core::domain::{FastForwardOutcome, Oid, RepoRef};
use gitpoulpi_core::{Error, Result};

use crate::process::{run, run_async};

/// Deletes `refs/heads/<name>` on `remote`. The fully-qualified ref keeps
/// a same-named tag from being the one deleted; the remote-tracking ref
/// is dropped by git itself once the remote accepts the deletion.
pub async fn branch_delete_remote(repo: &RepoRef, remote: &str, name: &str) -> Result<()> {
    let target = format!("refs/heads/{name}");
    run_async(&repo.root, &["push", remote, "--delete", &target]).await?;
    Ok(())
}

pub fn branch_create(repo: &RepoRef, name: &str, at: Oid) -> Result<()> {
    run(&repo.root, &["branch", name, &at.to_hex()])?;
    Ok(())
}

/// `git checkout -b <local_name> --track <upstream>`.
///
/// `--track` writes the `branch.<name>.remote`/`.merge` pair that
/// `%(upstream:short)` reads back; `-b` without `--force` makes a taken
/// name a non-zero exit instead of a silent checkout.
///
/// The options come first and the start point is fully qualified
/// (`refs/remotes/<upstream>`): git before 2.35 does not know
/// `--end-of-options` in `checkout` and reads it as a path ("'--track'
/// cannot be used with updating paths"), and the `refs/` prefix is what
/// keeps a start point from being parsed as a flag.
pub fn branch_create_tracking(repo: &RepoRef, local_name: &str, upstream: &str) -> Result<()> {
    let start = format!("refs/remotes/{upstream}");
    run(
        &repo.root,
        &["checkout", "--track", "-b", local_name, &start],
    )?;
    Ok(())
}

pub fn branch_delete(repo: &RepoRef, name: &str) -> Result<()> {
    run(&repo.root, &["branch", "-D", name])?;
    Ok(())
}

pub fn branch_rename(repo: &RepoRef, old_name: &str, new_name: &str) -> Result<()> {
    run(&repo.root, &["branch", "-m", old_name, new_name])?;
    Ok(())
}

/// `git branch --set-upstream-to` / `--unset-upstream`.
///
/// The upstream is passed fully qualified (`refs/remotes/origin/main`) so
/// a local branch *named* `origin/main` cannot be picked instead.
/// `--unset-upstream` errors in git when nothing is set, so the current
/// upstream is read first and "already none" is a success.
pub fn branch_set_upstream(repo: &RepoRef, branch: &str, upstream: Option<&str>) -> Result<()> {
    let local = format!("refs/heads/{branch}");
    match upstream {
        Some(upstream) => {
            let target = format!("refs/remotes/{upstream}");
            run(
                &repo.root,
                &[
                    "branch",
                    &format!("--set-upstream-to={target}"),
                    "--end-of-options",
                    branch,
                ],
            )?;
        }
        None => {
            let current = run(
                &repo.root,
                &["for-each-ref", "--format=%(upstream)", &local],
            )?;
            if !current.trim().is_empty() {
                run(
                    &repo.root,
                    &["branch", "--unset-upstream", "--end-of-options", branch],
                )?;
            }
        }
    }
    Ok(())
}

/// `GitBackend::fast_forward`: the ancestry is checked first, so the
/// verdict (up to date / fast-forward / diverged) never depends on parsing
/// what `git merge` printed.
///
/// - The branch checked out *here*: `git merge --ff-only <target>`, which
///   moves the ref, the index and the working tree together and refuses
///   on local changes it would overwrite.
/// - Any other branch: `git update-ref` with the old value as the
///   expected one, so a branch that moved since it was read is not
///   clobbered. Refused when another worktree has it checked out, as
///   `git fetch . a:b` refuses.
pub fn fast_forward(repo: &RepoRef, branch: &str, target: Oid) -> Result<FastForwardOutcome> {
    let local = format!("refs/heads/{branch}");
    let from = run(
        &repo.root,
        &[
            "rev-parse",
            "--verify",
            "--end-of-options",
            &format!("{local}^{{commit}}"),
        ],
    )
    .map_err(|_| Error::Backend(format!("there is no local branch {branch}")))?;
    let from = Oid::from_hex(from.trim())
        .ok_or_else(|| Error::Backend(format!("unreadable tip for {branch}: {from}")))?;
    let (from_hex, to_hex) = (from.to_hex(), target.to_hex());
    if from == target || is_ancestor(repo, &to_hex, &from_hex)? {
        return Ok(FastForwardOutcome::UpToDate);
    }
    if !is_ancestor(repo, &from_hex, &to_hex)? {
        return Err(Error::NotFastForward);
    }
    let head = run(&repo.root, &["symbolic-ref", "-q", "HEAD"]).unwrap_or_default();
    if head.trim() == local {
        run(&repo.root, &["merge", "--ff-only", "--no-edit", &to_hex])?;
    } else {
        if let Some(elsewhere) = crate::worktree_list::worktrees(repo)?
            .into_iter()
            .find(|wt| wt.branch.as_deref() == Some(branch))
        {
            return Err(Error::Backend(format!(
                "{branch} is checked out in the worktree at {} - fast-forward it there",
                elsewhere.path.display()
            )));
        }
        let message = format!("fast-forward {branch} to {}", &to_hex[..7]);
        run(
            &repo.root,
            &["update-ref", "-m", &message, &local, &to_hex, &from_hex],
        )?;
    }
    Ok(FastForwardOutcome::FastForwarded { from, to: target })
}

/// `git merge-base --is-ancestor`: exit 0 yes, 1 no, anything else an
/// error (an unknown commit).
fn is_ancestor(repo: &RepoRef, ancestor: &str, descendant: &str) -> Result<bool> {
    let out = crate::process::run_status(
        &repo.root,
        &["merge-base", "--is-ancestor", ancestor, descendant],
    )?;
    if out.success {
        Ok(true)
    } else if out.stderr.trim().is_empty() {
        Ok(false)
    } else {
        Err(Error::Backend(format!(
            "could not compare {ancestor} and {descendant}: {}",
            out.stderr.trim()
        )))
    }
}

pub fn tag_create(
    repo: &RepoRef,
    name: &str,
    at: Oid,
    annotated_message: Option<&str>,
) -> Result<()> {
    match annotated_message {
        Some(message) => {
            run(
                &repo.root,
                &["tag", "-a", name, "-m", message, &at.to_hex()],
            )?;
        }
        None => {
            run(&repo.root, &["tag", name, &at.to_hex()])?;
        }
    }
    Ok(())
}

pub fn tag_delete(repo: &RepoRef, name: &str) -> Result<()> {
    run(&repo.root, &["tag", "-d", name])?;
    Ok(())
}
