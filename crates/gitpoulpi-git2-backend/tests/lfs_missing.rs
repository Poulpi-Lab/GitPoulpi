//! With `git-lfs` missing, `Git2Backend` (which delegates every LFS
//! mutation to its system-git fallback) reports the same typed error.

#![cfg(feature = "lfs")]

use gitpoulpi_git2_backend::Git2Backend;
use gitpoulpi_testkit::contract;

#[test]
fn lfs_mutations_without_the_program_are_typed_unavailable() {
    contract::assert_lfs_mutations_without_binary_are_typed_unavailable(&Git2Backend::without_lfs());
}

#[test]
fn a_clone_without_the_program_succeeds_and_keeps_pointers() {
    contract::assert_clone_without_lfs_program_succeeds_and_keeps_pointers(
        &Git2Backend::without_lfs(),
    );
}
