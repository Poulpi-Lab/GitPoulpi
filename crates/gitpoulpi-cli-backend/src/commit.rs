use gitpoulpi_core::domain::{CommitOptions, Oid, RepoRef};
use gitpoulpi_core::{Error, Result};

use crate::process::run;

pub fn commit(repo: &RepoRef, message: &str, opts: &CommitOptions) -> Result<Oid> {
    let mut args = vec!["commit", "-m", message];
    if opts.amend {
        args.push("--amend");
    }
    if opts.no_verify {
        args.push("--no-verify");
    }
    if opts.sign {
        args.push("-S");
    } else {
        args.push("--no-gpg-sign");
    }
    // Same rule as `Git2Backend`: `opts.author` overrides the identity
    // this one commit is made with, `git config user.*` otherwise.
    let author_arg = opts
        .author
        .as_ref()
        .map(|a| format!("{} <{}>", a.name, a.email));
    if let Some(author_arg) = &author_arg {
        args.push("--author");
        args.push(author_arg);
    }
    run(&repo.root, &args)?;

    let head = run(&repo.root, &["rev-parse", "HEAD"])?;
    Oid::from_hex(head.trim()).ok_or_else(|| Error::Backend("HEAD was not a valid oid".into()))
}
