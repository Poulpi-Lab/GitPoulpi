//! `git am` in progress and a stale `index.lock`: the two states of plan
//! `stuck-states.md` slice S8 that only plain git used to clear.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use gitpoulpi_core::domain::{
    AmOutcome, ConflictRefusal, IndexLock, OperationState, RepoRef, STALE_INDEX_LOCK_SECS,
};
use gitpoulpi_core::{Error, Result};

use crate::conflicts::{git_dir, read_trimmed};
use crate::process::run_status_with_env;

/// `rebase-apply` holds both a plain `git rebase` and a `git am`; only the
/// latter has an `applying` file.
pub fn am_dir(root: &Path) -> Option<PathBuf> {
    let dir = git_dir(root).join("rebase-apply");
    dir.join("applying").exists().then_some(dir)
}

fn counter(dir: &Path, name: &str) -> u32 {
    read_trimmed(&dir.join(name))
        .and_then(|t| t.parse().ok())
        .unwrap_or(0)
}

/// The `am` in progress as an [`OperationState`], if any.
pub fn am_state(repo: &RepoRef) -> Option<OperationState> {
    let dir = am_dir(&repo.root)?;
    Some(OperationState::Am {
        step: counter(&dir, "next"),
        total: counter(&dir, "last"),
    })
}

fn require_am(repo: &RepoRef) -> Result<PathBuf> {
    am_dir(&repo.root).ok_or(Error::ConflictRefused(ConflictRefusal::NoOperation))
}

fn am_step(repo: &RepoRef, flag: &str) -> Result<AmOutcome> {
    let dir = require_am(repo)?;
    let before = counter(&dir, "next");
    let out = run_status_with_env(&repo.root, &["am", flag], &[("GIT_EDITOR", ":")])?;
    match am_dir(&repo.root) {
        None if out.success => Ok(AmOutcome::Completed),
        None => Err(Error::Backend(format!(
            "git am {flag} failed: {}",
            out.stderr
        ))),
        // Stopped again, on a later patch: progress. The same patch with an
        // error means nothing happened (e.g. paths still unmerged).
        Some(dir) if counter(&dir, "next") != before => Ok(AmOutcome::Stopped),
        Some(_) => Err(Error::Backend(format!(
            "git am {flag} failed: {}",
            out.stderr
        ))),
    }
}

pub fn am_continue(repo: &RepoRef) -> Result<AmOutcome> {
    am_step(repo, "--continue")
}

pub fn am_skip(repo: &RepoRef) -> Result<AmOutcome> {
    am_step(repo, "--skip")
}

pub fn am_abort(repo: &RepoRef) -> Result<()> {
    require_am(repo)?;
    let out = crate::process::run_abort(&repo.root, &["am", "--abort"])?;
    if out.success {
        Ok(())
    } else if crate::process::not_uptodate_paths(&out.stderr).is_empty() {
        Err(Error::Backend(format!(
            "git am --abort failed: {}",
            out.stderr
        )))
    } else {
        Err(crate::process::abort_failed(&["am", "--abort"], &out))
    }
}

fn lock_path(repo: &RepoRef) -> PathBuf {
    git_dir(&repo.root).join("index.lock")
}

pub fn index_lock(repo: &RepoRef) -> Result<Option<IndexLock>> {
    let path = lock_path(repo);
    let Ok(meta) = std::fs::metadata(&path) else {
        return Ok(None);
    };
    let age_secs = meta
        .modified()
        .ok()
        .and_then(|m| SystemTime::now().duration_since(m).ok())
        .map_or(0, |d| d.as_secs());
    Ok(Some(IndexLock {
        path,
        age_secs,
        stale: age_secs >= STALE_INDEX_LOCK_SECS,
    }))
}

pub fn index_lock_remove(repo: &RepoRef) -> Result<()> {
    let Some(lock) = index_lock(repo)? else {
        return Err(Error::Backend("there is no index.lock".into()));
    };
    if !lock.stale {
        return Err(Error::Backend(format!(
            "index.lock is only {}s old: a git command may still be running",
            lock.age_secs
        )));
    }
    std::fs::remove_file(&lock.path)?;
    Ok(())
}
