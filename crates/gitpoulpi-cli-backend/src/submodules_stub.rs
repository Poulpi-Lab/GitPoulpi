//! Stand-in for `submodules.rs` when the `submodule` feature is off (custom
//! builds): the same signatures, every operation answers
//! `NotBuilt { feature: "submodule" }`.

use gitpoulpi_core::domain::{RepoRef, Submodule, SubmoduleUpdateOptions};
use gitpoulpi_core::{Error, Result};

fn not_built<T>() -> Result<T> {
    Err(Error::NotBuilt {
        feature: "submodule",
    })
}

pub fn submodules(_repo: &RepoRef) -> Result<Vec<Submodule>> {
    not_built()
}

pub fn submodule_init(_repo: &RepoRef, _paths: &[String]) -> Result<()> {
    not_built()
}

pub fn submodule_update(_repo: &RepoRef, _opts: &SubmoduleUpdateOptions) -> Result<()> {
    not_built()
}
