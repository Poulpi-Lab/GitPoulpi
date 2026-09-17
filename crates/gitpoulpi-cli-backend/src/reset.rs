use gitpoulpi_core::domain::{Oid, RepoRef, ResetMode};
use gitpoulpi_core::Result;

use crate::process::run;

pub fn reset(repo: &RepoRef, target: Oid, mode: ResetMode) -> Result<()> {
    let mode_flag = match mode {
        ResetMode::Soft => "--soft",
        ResetMode::Mixed => "--mixed",
        ResetMode::Hard => "--hard",
    };
    run(&repo.root, &["reset", mode_flag, &target.to_hex()])?;
    Ok(())
}
