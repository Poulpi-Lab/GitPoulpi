//! Stand-in for `lfs.rs` when the `lfs` feature is off (custom builds):
//! the same signatures, every operation answers `NotBuilt { feature: "lfs" }`.

use gitpoulpi_core::domain::{
    LfsBinary, LfsFile, LfsInstallScope, LfsPattern, LfsRepoStatus, LfsTrackOptions, Oid, RepoRef,
};
use gitpoulpi_core::{Error, Result};

fn not_built<T>() -> Result<T> {
    Err(Error::NotBuilt { feature: "lfs" })
}

/// Kept so `SystemGitBackend` has the same fields in every build.
#[derive(Default)]
pub struct LfsProbe {
    _unused: (),
}

impl LfsProbe {
    pub fn forced(_binary: LfsBinary) -> Self {
        Self { _unused: () }
    }
}

pub fn lfs_status(_repo: &RepoRef, _probe: &LfsProbe) -> Result<LfsRepoStatus> {
    not_built()
}

pub fn lfs_install(_repo: &RepoRef, _probe: &LfsProbe, _scope: LfsInstallScope) -> Result<()> {
    not_built()
}

pub fn lfs_track(
    _repo: &RepoRef,
    _probe: &LfsProbe,
    _patterns: &[String],
    _opts: &LfsTrackOptions,
) -> Result<Vec<LfsPattern>> {
    not_built()
}

pub fn lfs_untrack(
    _repo: &RepoRef,
    _probe: &LfsProbe,
    _patterns: &[String],
) -> Result<Vec<LfsPattern>> {
    not_built()
}

pub fn lfs_prune(_repo: &RepoRef, _probe: &LfsProbe) -> Result<()> {
    not_built()
}

pub fn lfs_files(_repo: &RepoRef, _at: Option<Oid>) -> Result<Vec<LfsFile>> {
    not_built()
}

pub(crate) fn filters_configured(_repo: &RepoRef) -> Result<bool> {
    not_built()
}
