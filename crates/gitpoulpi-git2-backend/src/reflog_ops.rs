//! Native reflog and blob reads (the lost-commit scan is `CliFallback`: one
//! implementation of the reachability walk, git's own).

use gitpoulpi_core::domain::{reflog_refname, Oid, ReflogEntry, ReflogPage, RepoRef};
use gitpoulpi_core::Result;

use crate::repo::{map_err, open, to_domain_oid, to_git2_oid};

pub fn blob_bytes(repo: &RepoRef, oid: Oid) -> Result<Vec<u8>> {
    let git_repo = open(repo)?;
    let blob = git_repo.find_blob(to_git2_oid(oid)?).map_err(map_err)?;
    Ok(blob.content().to_vec())
}

pub fn reflog(repo: &RepoRef, refname: &str, skip: usize, limit: usize) -> Result<ReflogPage> {
    // libgit2 cannot open a reftable repository: git reads its reflog.
    if gitpoulpi_core::paths::uses_reftable(&repo.root) {
        return gitpoulpi_cli_backend::reflog_via_git(repo, refname, skip, limit);
    }
    let refname = reflog_refname(refname);
    let git_repo = open(repo)?;
    // A ref without a log (a deleted branch) reads as an empty reflog.
    let log = git_repo.reflog(&refname).map_err(map_err)?;
    let total = log.len();
    let mut entries = Vec::new();
    for index in skip..total.min(skip.saturating_add(limit)) {
        let Some(entry) = log.get(index) else { break };
        let who = entry.committer();
        entries.push(ReflogEntry {
            old: to_domain_oid(entry.id_old())?,
            new: to_domain_oid(entry.id_new())?,
            committer: format!(
                "{} <{}>",
                who.name().unwrap_or_default(),
                who.email().unwrap_or_default()
            ),
            time: who.when().seconds(),
            message: entry
                .message()
                .ok()
                .flatten()
                .unwrap_or_default()
                .trim_end_matches(['\r', '\n'])
                .to_string(),
        });
    }
    Ok(ReflogPage {
        refname,
        entries,
        more: total > skip.saturating_add(limit),
    })
}
