//! Stand-in for `lfs_locks.rs` when the `lfs` feature is off.

use std::path::Path;

use gitpoulpi_core::domain::{LfsLock, RepoRef};
use gitpoulpi_core::{Error, Result};

use crate::lfs::LfsProbe;

pub async fn lfs_locks(_repo: &RepoRef, _probe: &LfsProbe, _remote: &str) -> Result<Vec<LfsLock>> {
    Err(Error::NotBuilt { feature: "lfs" })
}

pub async fn lfs_lock(
    _repo: &RepoRef,
    _probe: &LfsProbe,
    _remote: &str,
    _path: &Path,
) -> Result<LfsLock> {
    Err(Error::NotBuilt { feature: "lfs" })
}

pub async fn lfs_unlock(
    _repo: &RepoRef,
    _probe: &LfsProbe,
    _remote: &str,
    _path: &Path,
    _force: bool,
) -> Result<()> {
    Err(Error::NotBuilt { feature: "lfs" })
}
