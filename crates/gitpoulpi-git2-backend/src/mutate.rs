use gitpoulpi_core::domain::{CheckoutTarget, CommitOptions, Oid, RepoRef, ResetMode};
use gitpoulpi_core::{Error, Result};

use crate::repo::{map_err, open, to_domain_oid, to_git2_oid};

/// True while a merge, cherry-pick or revert is stopped waiting for its
/// concluding commit.
pub fn has_pending_sequencer_commit(repo: &RepoRef) -> bool {
    let Ok(git_repo) = open(repo) else {
        return false;
    };
    matches!(
        git_repo.state(),
        git2::RepositoryState::Merge
            | git2::RepositoryState::CherryPick
            | git2::RepositoryState::CherryPickSequence
            | git2::RepositoryState::Revert
            | git2::RepositoryState::RevertSequence
    )
}

/// The hooks `git commit` may run. libgit2 runs none of them, so a repository
/// with one of these is committed through the system git (the decorator's
/// fallback), `--no-verify` included: that flag skips only `pre-commit` and
/// `commit-msg`, and the other two keep running exactly as with `git commit`.
const COMMIT_HOOKS: [&str; 4] = [
    "pre-commit",
    "prepare-commit-msg",
    "commit-msg",
    "post-commit",
];

/// Whether this repository has a commit hook git would run (an executable
/// file on Unix, a file on Windows), looked up in `core.hooksPath` or the
/// common git directory's `hooks`.
pub fn has_commit_hooks(repo: &RepoRef) -> bool {
    let Ok(git_repo) = open(repo) else {
        return false;
    };
    let dir = git_repo
        .config()
        .ok()
        .and_then(|c| c.get_path("core.hooksPath").ok())
        .map(|p| {
            if p.is_absolute() {
                p
            } else {
                repo.root.join(p)
            }
        })
        .unwrap_or_else(|| {
            let git_dir = git_repo.path().to_path_buf();
            let common = std::fs::read_to_string(git_dir.join("commondir"))
                .ok()
                .map(|rel| git_dir.join(rel.trim()))
                .unwrap_or(git_dir);
            common.join("hooks")
        });
    COMMIT_HOOKS.iter().any(|name| {
        let path = dir.join(name);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::metadata(&path)
                .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        }
        #[cfg(not(unix))]
        {
            path.is_file()
        }
    })
}

pub fn commit(repo: &RepoRef, message: &str, opts: &CommitOptions) -> Result<Oid> {
    let git_repo = open(repo)?;
    let mut index = git_repo.index().map_err(map_err)?;
    let tree_id = index.write_tree().map_err(map_err)?;
    let tree = git_repo.find_tree(tree_id).map_err(map_err)?;
    // `opts.author`: `general.active_profile`'s identity, resolved by
    // the caller. `None` falls back to whatever `git config user.*` says,
    // via `signature()`.
    let sig = match &opts.author {
        Some(author) => git2::Signature::now(&author.name, &author.email).map_err(map_err)?,
        None => git_repo.signature().map_err(map_err)?,
    };
    let head_commit = git_repo.head().ok().and_then(|h| h.peel_to_commit().ok());

    if opts.amend {
        let parent = head_commit.ok_or_else(|| Error::Backend("no HEAD commit to amend".into()))?;
        // `None` for author keeps the amended commit's existing author
        // (`git commit --amend`'s default). An override replaces just the
        // author; the committer is always whoever is running the amend
        // (`git config user.*`).
        let author = opts.author.is_some().then_some(&sig);
        let new_oid = parent
            .amend(Some("HEAD"), author, None, None, Some(message), Some(&tree))
            .map_err(map_err)?;
        return to_domain_oid(new_oid);
    }

    let parents: Vec<&git2::Commit> = head_commit.iter().collect();
    let new_oid = git_repo
        .commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
        .map_err(map_err)?;
    to_domain_oid(new_oid)
}

pub fn checkout(repo: &RepoRef, target: &CheckoutTarget) -> Result<()> {
    let git_repo = open(repo)?;
    match target {
        CheckoutTarget::Branch(name) => {
            let (object, reference) = git_repo.revparse_ext(name).map_err(map_err)?;
            let mut builder = git2::build::CheckoutBuilder::new();
            builder.safe();
            git_repo
                .checkout_tree(&object, Some(&mut builder))
                .map_err(map_err)?;
            match reference {
                Some(r) if r.is_branch() => {
                    git_repo
                        .set_head(
                            r.name()
                                .ok_or_else(|| Error::Backend("branch ref had no name".into()))?,
                        )
                        .map_err(map_err)?;
                }
                _ => {
                    git_repo.set_head_detached(object.id()).map_err(map_err)?;
                }
            }
        }
        CheckoutTarget::Commit(oid) => {
            let git2_oid = to_git2_oid(*oid)?;
            let object = git_repo.find_object(git2_oid, None).map_err(map_err)?;
            let mut builder = git2::build::CheckoutBuilder::new();
            builder.safe();
            git_repo
                .checkout_tree(&object, Some(&mut builder))
                .map_err(map_err)?;
            git_repo.set_head_detached(git2_oid).map_err(map_err)?;
        }
        CheckoutTarget::File { .. } | CheckoutTarget::Files { .. } => {
            // Single-file checkout-from-commit into the workdir *and* index
            // needs careful index-entry construction; the CLI backend
            // (`git checkout <src> -- <path>`) already does this correctly,
            // so this variant is dispatched to the fallback rather than
            // reimplemented here.
            return Err(Error::Unsupported {
                operation: "Git2Backend::checkout(File) - dispatched via decorator fallback",
            });
        }
    }
    Ok(())
}

pub fn branch_create(repo: &RepoRef, name: &str, at: Oid) -> Result<()> {
    let git_repo = open(repo)?;
    let commit = git_repo.find_commit(to_git2_oid(at)?).map_err(map_err)?;
    git_repo.branch(name, &commit, false).map_err(map_err)?;
    Ok(())
}

/// `git checkout -b <local_name> --track <upstream>`, the libgit2 way.
///
/// Three steps, in this order, because each one is a separate way to
/// half-do it: create the branch at the remote branch's tip (`force =
/// false`, so an existing name is an error rather than a silent reset),
/// record the upstream, then move HEAD and the working tree onto it.
/// `set_upstream` is what makes `%(upstream:short)` - and therefore the
/// UI's "a local branch already tracks this" join - answer at all.
pub fn branch_create_tracking(repo: &RepoRef, local_name: &str, upstream: &str) -> Result<()> {
    let git_repo = open(repo)?;
    let remote_branch = git_repo
        .find_branch(upstream, git2::BranchType::Remote)
        .map_err(map_err)?;
    let commit = remote_branch.get().peel_to_commit().map_err(map_err)?;
    let mut branch = git_repo
        .branch(local_name, &commit, false)
        .map_err(map_err)?;
    branch.set_upstream(Some(upstream)).map_err(map_err)?;

    let object = commit.as_object();
    let mut builder = git2::build::CheckoutBuilder::new();
    builder.safe();
    git_repo
        .checkout_tree(object, Some(&mut builder))
        .map_err(map_err)?;
    let head_ref = branch
        .get()
        .name()
        .ok_or_else(|| Error::Backend("new branch ref had no name".into()))?
        .to_string();
    git_repo.set_head(&head_ref).map_err(map_err)?;
    Ok(())
}

pub fn branch_delete(repo: &RepoRef, name: &str) -> Result<()> {
    let git_repo = open(repo)?;
    let mut branch = git_repo
        .find_branch(name, git2::BranchType::Local)
        .map_err(map_err)?;
    branch.delete().map_err(map_err)?;
    Ok(())
}

pub fn branch_rename(repo: &RepoRef, old_name: &str, new_name: &str) -> Result<()> {
    let git_repo = open(repo)?;
    let mut branch = git_repo
        .find_branch(old_name, git2::BranchType::Local)
        .map_err(map_err)?;
    branch.rename(new_name, false).map_err(map_err)?;
    Ok(())
}

/// libgit2's `git_branch_set_upstream`, which takes the upstream by its
/// short remote-branch name (`origin/main`) and writes the
/// `remote`/`merge` pair itself.
///
/// The remote-tracking ref is looked up first: libgit2 would also accept
/// a *local* branch of that name, and the contract is "a remote branch
/// that exists".
pub fn branch_set_upstream(repo: &RepoRef, branch: &str, upstream: Option<&str>) -> Result<()> {
    let git_repo = open(repo)?;
    let mut local = git_repo
        .find_branch(branch, git2::BranchType::Local)
        .map_err(map_err)?;
    match upstream {
        Some(upstream) => {
            git_repo
                .find_reference(&format!("refs/remotes/{upstream}"))
                .map_err(|_| Error::Backend(format!("no such remote branch: {upstream}")))?;
            local.set_upstream(Some(upstream)).map_err(map_err)?;
        }
        None => {
            if local.upstream().is_ok() {
                local.set_upstream(None).map_err(map_err)?;
            }
        }
    }
    Ok(())
}

/// `GitBackend::fast_forward`, natively.
///
/// Ancestry through `graph_descendant_of`; the checked-out branch moves
/// by a *safe* checkout of the target's tree (which refuses to overwrite
/// local changes, and updates the index) followed by the ref, exactly the
/// order `git merge --ff-only` uses - so a refused checkout leaves the
/// ref where it was. Any other branch moves by its ref alone, unless
/// another worktree has it checked out (`reads::worktrees` knows which,
/// main worktree included).
pub fn fast_forward(
    repo: &RepoRef,
    branch: &str,
    target: Oid,
) -> Result<gitpoulpi_core::domain::FastForwardOutcome> {
    use gitpoulpi_core::domain::FastForwardOutcome;
    let git_repo = open(repo)?;
    let local = format!("refs/heads/{branch}");
    let mut reference = git_repo
        .find_reference(&local)
        .map_err(|_| Error::Backend(format!("there is no local branch {branch}")))?;
    let from = reference.peel_to_commit().map_err(map_err)?.id();
    let to = to_git2_oid(target)?;
    git_repo.find_commit(to).map_err(map_err)?;
    if from == to || git_repo.graph_descendant_of(from, to).map_err(map_err)? {
        return Ok(FastForwardOutcome::UpToDate);
    }
    if !git_repo.graph_descendant_of(to, from).map_err(map_err)? {
        return Err(Error::NotFastForward);
    }
    let message = format!("fast-forward {branch} to {}", &target.to_hex()[..7]);
    let checked_out_here = !git_repo.head_detached().unwrap_or(true)
        && git_repo
            .head()
            .ok()
            .and_then(|h| h.name().map(|n| n == local))
            .unwrap_or(false);
    if checked_out_here {
        let commit = git_repo.find_commit(to).map_err(map_err)?;
        let mut builder = git2::build::CheckoutBuilder::new();
        builder.safe();
        git_repo
            .checkout_tree(commit.as_object(), Some(&mut builder))
            .map_err(map_err)?;
    } else if let Some(elsewhere) = crate::reads::worktrees(repo)?
        .into_iter()
        .find(|wt| wt.branch.as_deref() == Some(branch))
    {
        return Err(Error::Backend(format!(
            "{branch} is checked out in the worktree at {} - fast-forward it there",
            elsewhere.path.display()
        )));
    }
    reference.set_target(to, &message).map_err(map_err)?;
    Ok(FastForwardOutcome::FastForwarded {
        from: to_domain_oid(from)?,
        to: target,
    })
}

/// `remote.<name>.url`. libgit2 would happily write config for a remote
/// that does not exist, so its existence is checked first.
pub fn remote_set_url(repo: &RepoRef, name: &str, url: &str) -> Result<()> {
    let git_repo = open(repo)?;
    git_repo.find_remote(name).map_err(map_err)?;
    git_repo.remote_set_url(name, url).map_err(map_err)?;
    Ok(())
}

/// `remote.<name>.pushurl`, set or removed.
pub fn remote_set_push_url(repo: &RepoRef, name: &str, url: Option<&str>) -> Result<()> {
    let git_repo = open(repo)?;
    git_repo.find_remote(name).map_err(map_err)?;
    match url {
        Some(url) => git_repo
            .remote_set_pushurl(name, Some(url))
            .map_err(map_err)?,
        None => {
            let has_one = git_repo
                .find_remote(name)
                .map_err(map_err)?
                .pushurl()
                .is_some();
            if has_one {
                git_repo.remote_set_pushurl(name, None).map_err(map_err)?;
            }
        }
    }
    Ok(())
}

/// The repository-local value of an allow-listed key, or `None`.
pub fn repo_config(repo: &RepoRef, key: &str) -> Result<Option<String>> {
    gitpoulpi_core::repo_config::check_repo_config_key(key)?;
    let git_repo = open(repo)?;
    let config = git_repo
        .config()
        .map_err(map_err)?
        .open_level(git2::ConfigLevel::Local)
        .map_err(map_err)?;
    Ok(config
        .get_string(key)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty()))
}

/// See `GitBackend::config_entries`.
pub fn config_entries(repo: &RepoRef, prefixes: &[&str]) -> Result<Vec<(String, String)>> {
    let git_repo = open(repo)?;
    let config = git_repo
        .config()
        .map_err(map_err)?
        .open_level(git2::ConfigLevel::Local)
        .map_err(map_err)?;
    let prefixes: Vec<String> = prefixes.iter().map(|p| p.to_ascii_lowercase()).collect();
    let mut out = Vec::new();
    let mut entries = config.entries(None).map_err(map_err)?;
    while let Some(entry) = entries.next() {
        let entry = entry.map_err(map_err)?;
        let (Some(name), Some(value)) = (entry.name(), entry.value()) else {
            continue;
        };
        let lower = name.to_ascii_lowercase();
        if prefixes.iter().any(|p| lower.starts_with(p.as_str())) {
            out.push((name.to_string(), value.to_string()));
        }
    }
    Ok(out)
}

/// Sets or clears an allow-listed repository-local key.
pub fn set_repo_config(repo: &RepoRef, key: &str, value: Option<&str>) -> Result<()> {
    let git_repo = open(repo)?;
    let mut config = git_repo
        .config()
        .map_err(map_err)?
        .open_level(git2::ConfigLevel::Local)
        .map_err(map_err)?;
    match value {
        Some(v) => {
            gitpoulpi_core::repo_config::check_repo_config_value(key, v)?;
            config.set_str(key, v).map_err(map_err)?;
        }
        None => {
            gitpoulpi_core::repo_config::check_repo_config_key(key)?;
            if config.get_entry(key).is_ok() {
                config.remove(key).map_err(map_err)?;
            }
        }
    }
    Ok(())
}

/// The repository-local `core.sshCommand` (the repo's own config file,
/// not the global or system one), or `None`.
pub fn ssh_command(repo: &RepoRef) -> Result<Option<String>> {
    let git_repo = open(repo)?;
    let config = git_repo
        .config()
        .map_err(map_err)?
        .open_level(git2::ConfigLevel::Local)
        .map_err(map_err)?;
    Ok(config
        .get_string("core.sshCommand")
        .ok()
        .filter(|v| !v.trim().is_empty()))
}

/// Sets or clears the repository-local `core.sshCommand`.
pub fn set_ssh_command(repo: &RepoRef, command: Option<&str>) -> Result<()> {
    let git_repo = open(repo)?;
    let mut config = git_repo
        .config()
        .map_err(map_err)?
        .open_level(git2::ConfigLevel::Local)
        .map_err(map_err)?;
    match command {
        Some(cmd) => config.set_str("core.sshCommand", cmd).map_err(map_err)?,
        None => {
            if config.get_entry("core.sshCommand").is_ok() {
                config.remove("core.sshCommand").map_err(map_err)?;
            }
        }
    }
    Ok(())
}

pub fn tag_create(
    repo: &RepoRef,
    name: &str,
    at: Oid,
    annotated_message: Option<&str>,
) -> Result<()> {
    let git_repo = open(repo)?;
    let object = git_repo
        .find_object(to_git2_oid(at)?, None)
        .map_err(map_err)?;
    match annotated_message {
        Some(message) => {
            let sig = git_repo.signature().map_err(map_err)?;
            git_repo
                .tag(name, &object, &sig, message, false)
                .map_err(map_err)?;
        }
        None => {
            git_repo
                .tag_lightweight(name, &object, false)
                .map_err(map_err)?;
        }
    }
    Ok(())
}

pub fn tag_delete(repo: &RepoRef, name: &str) -> Result<()> {
    let git_repo = open(repo)?;
    git_repo.tag_delete(name).map_err(map_err)?;
    Ok(())
}

pub fn reset(repo: &RepoRef, target: Oid, mode: ResetMode) -> Result<()> {
    let git_repo = open(repo)?;
    let object = git_repo
        .find_object(to_git2_oid(target)?, None)
        .map_err(map_err)?;
    let git2_mode = match mode {
        ResetMode::Soft => git2::ResetType::Soft,
        ResetMode::Mixed => git2::ResetType::Mixed,
        ResetMode::Hard => git2::ResetType::Hard,
    };
    git_repo.reset(&object, git2_mode, None).map_err(map_err)?;
    Ok(())
}

/// Replaces `name` with an annotated tag carrying `message`, at the
/// commit it already points at.
///
/// Three things are load-bearing, and each is a way to diverge from
/// `SystemGitBackend`:
///
/// - **`find_reference` first.** `Repository::tag(.., force = true)`
///   will happily *create* a tag, so without the lookup "annotate a tag
///   that isn't there" would silently invent one at whatever object was
///   passed.
/// - **The target comes from the tag itself**, peeled to a commit - so
///   the tag cannot move, whatever the caller believed it pointed at.
/// - **`force = true`**, because the ref exists by definition here.
pub fn tag_annotate(repo: &RepoRef, name: &str, message: &str) -> Result<()> {
    let git_repo = open(repo)?;
    let reference = git_repo
        .find_reference(&format!("refs/tags/{name}"))
        .map_err(|_| Error::Backend(format!("no tag named `{name}`")))?;
    let commit = reference.peel_to_commit().map_err(map_err)?;
    let object = commit.as_object();
    let sig = git_repo.signature().map_err(map_err)?;
    git_repo
        .tag(name, object, &sig, message, true)
        .map_err(map_err)?;
    Ok(())
}
