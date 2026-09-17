//! Gathering what the planner needs: the effective workflow and the
//! repository's branches, remotes, recorded parents and working tree.

use gitpoulpi_core::domain::{RefKind, RepoRef};
use gitpoulpi_core::time::civil_date;
use gitpoulpi_core::workflow::{
    parse_recorded_parents, FlowFacts, Workflow, WORKFLOW_CONFIG_PREFIXES,
};
use gitpoulpi_core::GitBackend;
use gitpoulpi_session::LoadedWorkflow;

use crate::dispatch::Failure;

pub(super) fn backend_failure(e: gitpoulpi_core::Error) -> Failure {
    (e.to_string(), e.typed(), e.detail().cloned())
}

pub(super) fn plain_failure(message: impl Into<String>) -> Failure {
    (message.into(), None, None)
}

/// The effective workflow and the facts, read now.
#[allow(clippy::result_large_err)]
pub(super) fn gather(
    backend: &dyn GitBackend,
    repo: &RepoRef,
) -> Result<(Workflow, FlowFacts), Failure> {
    let loaded =
        LoadedWorkflow::load(&repo.root, &crate::workflow::state_dir()).map_err(plain_failure)?;
    let refs = backend.refs(repo).map_err(backend_failure)?;
    let mut facts = FlowFacts::default();
    for r in &refs {
        match &r.kind {
            RefKind::LocalBranch { name } => {
                facts.branches.insert(name.clone(), r.target);
                if r.is_head {
                    facts.head = Some(name.clone());
                }
                if let Some(up) = &r.upstream {
                    facts.upstreams.insert(name.clone(), up.clone());
                }
            }
            RefKind::RemoteBranch { remote, name } if name != "HEAD" => {
                facts
                    .remote_branches
                    .insert(format!("{remote}/{name}"), r.target);
            }
            RefKind::Tag { name } => {
                facts.tags.insert(name.clone());
            }
            RefKind::RemoteBranch { .. } => {}
        }
    }
    facts.remotes = backend
        .remotes(repo)
        .map_err(backend_failure)?
        .into_iter()
        .map(|r| r.name)
        .collect();
    let entries = backend
        .config_entries(repo, WORKFLOW_CONFIG_PREFIXES)
        .map_err(backend_failure)?;
    facts.recorded = parse_recorded_parents(&entries);
    facts.dirty = !backend
        .status(repo)
        .map_err(backend_failure)?
        .files
        .is_empty();
    facts.in_progress =
        crate::journal::git_dir(&repo.root).and_then(|d| crate::journal::in_progress(&d));
    facts.user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .ok()
        .filter(|u| !u.trim().is_empty());
    facts.today = civil_date(gitpoulpi_core::time::unix_now());
    Ok((loaded.effective(), facts))
}

#[cfg(test)]
mod tests {
    use super::civil_date;

    #[test]
    fn civil_dates_are_right_around_the_epoch_and_a_leap_day() {
        assert_eq!(civil_date(0), "1970-01-01");
        assert_eq!(civil_date(951_782_400), "2000-02-29");
        assert_eq!(civil_date(1_781_136_000), "2026-06-11");
        assert_eq!(civil_date(-86_400), "1969-12-31");
    }
}
