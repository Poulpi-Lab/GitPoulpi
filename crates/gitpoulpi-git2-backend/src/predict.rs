//! `GitBackend::predict_merge_conflicts` over libgit2's in-memory merge.

use gitpoulpi_core::domain::{Oid, RepoRef};
use gitpoulpi_core::Result;

use crate::repo::{map_err, open, to_git2_oid};

pub fn predict_merge_conflicts(repo: &RepoRef, ours: Oid, theirs: Oid) -> Result<Vec<String>> {
    let git_repo = open(repo)?;
    let ours = git_repo.find_commit(to_git2_oid(ours)?).map_err(map_err)?;
    let theirs = git_repo
        .find_commit(to_git2_oid(theirs)?)
        .map_err(map_err)?;
    // Renames are followed (like `merge-tree`'s default), so rename/modify
    // merges cleanly here too. No work tree, index or ref is touched.
    let mut opts = git2::MergeOptions::new();
    opts.find_renames(true);
    let index = git_repo
        .merge_commits(&ours, &theirs, Some(&opts))
        .map_err(map_err)?;
    if !index.has_conflicts() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for conflict in index.conflicts().map_err(map_err)? {
        let conflict = conflict.map_err(map_err)?;
        let entry = conflict.our.or(conflict.their).or(conflict.ancestor);
        if let Some(entry) = entry {
            names.push(String::from_utf8_lossy(&entry.path).into_owned());
        }
    }
    names.sort();
    names.dedup();
    Ok(names)
}
