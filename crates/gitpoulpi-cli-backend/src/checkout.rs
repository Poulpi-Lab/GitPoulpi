use gitpoulpi_core::domain::{CheckoutTarget, RepoRef};
use gitpoulpi_core::Result;

use crate::process::run;

pub fn checkout(repo: &RepoRef, target: &CheckoutTarget) -> Result<()> {
    match target {
        CheckoutTarget::Branch(name) => {
            run(&repo.root, &["checkout", name])?;
        }
        CheckoutTarget::Commit(oid) => {
            run(&repo.root, &["checkout", &oid.to_hex()])?;
        }
        CheckoutTarget::File { path, source } => {
            run(
                &repo.root,
                &["checkout", &source.to_hex(), "--", path.as_str()],
            )?;
        }
        CheckoutTarget::Files { paths, source } => {
            if paths.is_empty() {
                return Ok(());
            }
            let hex = source.to_hex();
            let mut args = vec!["checkout", hex.as_str(), "--"];
            args.extend(paths.iter().map(String::as_str));
            run(&repo.root, &args)?;
        }
    }
    Ok(())
}
