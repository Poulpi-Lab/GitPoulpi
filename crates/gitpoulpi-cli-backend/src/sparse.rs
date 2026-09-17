//! Sparse checkout (cone mode) through `git sparse-checkout`.

use gitpoulpi_core::domain::{normalize_sparse_dirs, RepoRef, SparseState};
use gitpoulpi_core::Result;

use crate::process::{run, run_lenient};

/// `core.sparseCheckout` is on, in which mode, and what `git
/// sparse-checkout list` says. A repository that never enabled it is the
/// default (disabled) state, not an error: `list` itself fails there.
pub fn status(repo: &RepoRef) -> Result<SparseState> {
    let (_, enabled) = run_lenient(&repo.root, &["config", "--get", "core.sparseCheckout"])?;
    if enabled.trim() != "true" {
        return Ok(SparseState::default());
    }
    let (_, cone) = run_lenient(&repo.root, &["config", "--get", "core.sparseCheckoutCone"])?;
    let (ok, list) = run_lenient(&repo.root, &["sparse-checkout", "list"])?;
    let dirs = if ok {
        list.lines()
            .map(str::trim_end)
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect()
    } else {
        Vec::new()
    };
    Ok(SparseState {
        enabled: true,
        cone: cone.trim() == "true",
        dirs,
    })
}

/// `git sparse-checkout set --cone -- <dirs>`: enables the mode when it is
/// off and replaces the set when it is on.
///
/// Before git 2.35 `set` knows neither `--cone` nor `--`: it is
/// `init --cone` (a no-op when already on) then `set <dirs>`, and the
/// directories are checked not to look like flags by
/// `normalize_sparse_dirs`.
pub fn set(repo: &RepoRef, dirs: &[String]) -> Result<()> {
    let dirs = normalize_sparse_dirs(dirs)?;
    if !crate::process::git_at_least(2, 35) {
        run(&repo.root, &["sparse-checkout", "init", "--cone"])?;
        let mut args = vec!["sparse-checkout", "set"];
        args.extend(dirs.iter().map(String::as_str));
        run(&repo.root, &args)?;
        return Ok(());
    }
    let mut args = vec!["sparse-checkout", "set", "--cone", "--"];
    args.extend(dirs.iter().map(String::as_str));
    run(&repo.root, &args)?;
    Ok(())
}

/// `git sparse-checkout disable`: every file comes back. Harmless on a
/// repository that is not sparse.
pub fn disable(repo: &RepoRef) -> Result<()> {
    run(&repo.root, &["sparse-checkout", "disable"])?;
    Ok(())
}
