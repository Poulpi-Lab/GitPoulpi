//! The passive half of host-driven flows: issues a flow made are remembered
//! (`RepoState::watched_issues`); after every fetch (any change of the refs)
//! the remote branches are matched against the workflow's `branch_from_issue`
//! and a branch that appeared for a watched issue is offered with a Pick up
//! action. No network: the fetch already brought the branch. Asked off the
//! render thread (the state file is read on a worker).

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};

use gitpoulpi_core::workflow::{branches_for_issue, Workflow};

/// A watched issue whose branch is now on the remote.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Appeared {
    pub(crate) number: u64,
    pub(crate) title: String,
    pub(crate) branches: Vec<String>,
}

/// Pure: which watched issues have a remote branch that is not local yet.
pub(crate) fn appeared(
    workflow: &Workflow,
    watched: &[gitpoulpi_session::WatchedIssue],
    remote_branches: &[String],
    local_branches: &[String],
) -> Vec<Appeared> {
    let flow = workflow.issue_flow.clone().unwrap_or_default();
    let ids: Vec<&str> = workflow.topics.iter().map(|t| t.id.as_str()).collect();
    watched
        .iter()
        .filter_map(|w| {
            let branches: Vec<String> = branches_for_issue(&flow, &ids, remote_branches, w.number)
                .into_iter()
                .filter(|b| !local_branches.contains(b))
                .collect();
            (!branches.is_empty()).then(|| Appeared {
                number: w.number,
                title: w.title.clone(),
                branches,
            })
        })
        .collect()
}

#[derive(Default)]
pub(crate) struct IssueWatch {
    asked_at: Option<u64>,
    rx: Option<Receiver<Vec<Appeared>>>,
    pending: Vec<Appeared>,
    /// Already shown in this window: a "Later" is not asked again until the
    /// next restart.
    notified: HashSet<u64>,
}

impl IssueWatch {
    /// Called every frame with the refs generation. Asks again when the refs
    /// changed; returns the next appeared branch once `can_show` (no dialog
    /// is open).
    pub(crate) fn tick(
        &mut self,
        refs_gen: u64,
        can_show: bool,
        inputs: impl FnOnce() -> Option<(Workflow, PathBuf, Vec<String>, Vec<String>)>,
    ) -> Option<Appeared> {
        if let Some(rx) = &self.rx {
            if let Ok(found) = rx.try_recv() {
                self.rx = None;
                self.pending = found
                    .into_iter()
                    .filter(|a| !self.notified.contains(&a.number))
                    .collect();
            }
        }
        if self.rx.is_none() && self.asked_at != Some(refs_gen) {
            self.asked_at = Some(refs_gen);
            if let Some((workflow, state_path, remote, local)) = inputs() {
                let (tx, rx) = channel();
                self.rx = Some(rx);
                std::thread::spawn(move || {
                    let watched = gitpoulpi_session::RepoState::load(&state_path)
                        .map(|s| s.watched_issues)
                        .unwrap_or_default();
                    let _ = tx.send(appeared(&workflow, &watched, &remote, &local));
                    crate::redraw::poke();
                });
            }
        }
        if can_show && !self.pending.is_empty() {
            let next = self.pending.remove(0);
            self.notified.insert(next.number);
            return Some(next);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_session::WatchedIssue;

    fn workflow() -> Workflow {
        gitpoulpi_core::workflow::WorkflowTemplate::GithubFlow.workflow()
    }

    fn watched(number: u64) -> WatchedIssue {
        WatchedIssue {
            number,
            type_id: "feature".into(),
            title: "Login".into(),
            created: 0,
        }
    }

    #[test]
    fn a_remote_branch_of_a_watched_issue_appears_once_and_a_local_one_does_not() {
        let remote = vec!["feature/12-login".to_string(), "feature/13-x".to_string()];
        let found = appeared(&workflow(), &[watched(12), watched(14)], &remote, &[]);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].number, 12);
        assert_eq!(found[0].branches, ["feature/12-login"]);
        // Picked up already: nothing to offer.
        let local = vec!["feature/12-login".to_string()];
        assert!(appeared(&workflow(), &[watched(12)], &remote, &local).is_empty());
    }

    #[test]
    fn a_notified_issue_is_not_offered_again_and_a_dialog_in_the_way_defers() {
        let mut watch = IssueWatch::default();
        let inputs = || {
            Some((
                workflow(),
                std::path::PathBuf::from("/nonexistent/repo-state.toml"),
                vec!["feature/1-a".to_string()],
                Vec::new(),
            ))
        };
        // The state file does not exist: nothing is watched, nothing appears.
        let mut shown = None;
        for _ in 0..50 {
            shown = watch.tick(1, true, inputs);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(shown.is_none());
        // Seed the pending list by hand: a dialog open defers, then it shows once.
        watch.pending = vec![Appeared {
            number: 1,
            title: "a".into(),
            branches: vec!["feature/1-a".into()],
        }];
        assert!(watch.tick(1, false, inputs).is_none());
        assert_eq!(watch.tick(1, true, inputs).unwrap().number, 1);
        assert!(watch.tick(1, true, inputs).is_none());
    }
}
