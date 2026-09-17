//! Stand-in for `lfs.rs` when the `lfs` feature is off (custom builds).
//!
//! `is_active` stays real: libgit2 has no LFS filters, so a repository that
//! uses LFS must still route working-tree writes to the system git, which
//! honours whatever filters the user installed. Only the LFS *management*
//! read answers `NotBuilt`.

use gitpoulpi_core::domain::{attributes_mention_lfs, LfsFile, Oid, RepoRef};
use gitpoulpi_core::{Error, Result};

use crate::repo::open;

fn uses_lfs(repo: &RepoRef, common: &std::path::Path) -> bool {
    if common.join("lfs").is_dir() {
        return true;
    }
    [
        repo.root.join(".gitattributes"),
        common.join("info").join("attributes"),
    ]
    .iter()
    .any(|file| std::fs::read_to_string(file).is_ok_and(|t| attributes_mention_lfs(&t)))
}

pub fn is_active(repo: &RepoRef) -> bool {
    match open(repo) {
        Ok(git) => uses_lfs(repo, &gitpoulpi_core::paths::common_dir(git.path())),
        Err(_) => false,
    }
}

pub fn lfs_files(_repo: &RepoRef, _at: Option<Oid>) -> Result<Vec<LfsFile>> {
    Err(Error::NotBuilt { feature: "lfs" })
}
