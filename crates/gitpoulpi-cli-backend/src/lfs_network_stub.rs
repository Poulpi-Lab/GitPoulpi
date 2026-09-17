//! Stand-in for `lfs_network.rs` when the `lfs` feature is off.

use std::path::PathBuf;

use gitpoulpi_core::domain::{LfsFetchOptions, LfsProgressSink, LfsPushOptions, RepoRef};
use gitpoulpi_core::{Error, Result};

use crate::lfs::LfsProbe;

pub async fn lfs_fetch(
    _repo: &RepoRef,
    _probe: &LfsProbe,
    _remote: &str,
    _opts: &LfsFetchOptions,
    _progress: LfsProgressSink<'_>,
) -> Result<()> {
    Err(Error::NotBuilt { feature: "lfs" })
}

pub async fn lfs_pull(
    _repo: &RepoRef,
    _probe: &LfsProbe,
    _remote: &str,
    _progress: LfsProgressSink<'_>,
) -> Result<()> {
    Err(Error::NotBuilt { feature: "lfs" })
}

pub async fn lfs_checkout(
    _repo: &RepoRef,
    _probe: &LfsProbe,
    _paths: &[PathBuf],
    _progress: LfsProgressSink<'_>,
) -> Result<()> {
    Err(Error::NotBuilt { feature: "lfs" })
}

pub async fn lfs_push(
    _repo: &RepoRef,
    _probe: &LfsProbe,
    _remote: &str,
    _opts: &LfsPushOptions,
    _progress: LfsProgressSink<'_>,
) -> Result<()> {
    Err(Error::NotBuilt { feature: "lfs" })
}
