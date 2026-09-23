//! Without the `lfs` feature every LFS operation answers a typed
//! `NotBuilt`, and a clone is a plain one.
#![cfg(not(feature = "lfs"))]

use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_core::domain::{LfsFetchOptions, LfsInstallScope};
use gitpoulpi_core::{Error, GitBackend};
use gitpoulpi_testkit::FixtureRepoBuilder;

#[test]
fn lfs_operations_are_not_built() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("c")
        .unwrap()
        .build();
    let backend = SystemGitBackend::new();
    let r = repo.repo_ref();
    assert!(matches!(
        backend.lfs_status(&r),
        Err(Error::NotBuilt { feature: "lfs" })
    ));
    assert!(matches!(
        backend.lfs_files(&r, None),
        Err(Error::NotBuilt { feature: "lfs" })
    ));
    assert!(matches!(
        backend.lfs_install(&r, LfsInstallScope::Repository),
        Err(Error::NotBuilt { feature: "lfs" })
    ));
    let res = gitpoulpi_testkit::contract::block_on(backend.lfs_fetch(
        &r,
        "origin",
        &LfsFetchOptions::default(),
        &|_| {},
    ));
    assert!(matches!(res, Err(Error::NotBuilt { feature: "lfs" })));
}
