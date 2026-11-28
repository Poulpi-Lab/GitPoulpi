//! The first writes: Fetch, Pull, Push, Undo and Redo as link actions.
//!
//! One press is one submitted action, so one journal entry for Undo to take back. The link reports
//! progress on its receivers and through the host; this file only decides *what* to submit.

use gitpoulpi_core::domain::{Direction, PullOptions, PushOptions, Ref, RefKind, Remote};
use gitpoulpi_link::Action;

use super::Backend;
use crate::strings;

/// What a toolbar button or shortcut asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transfer {
    Fetch,
    Pull,
    Push,
    Undo,
    Redo,
}

/// Where a push of the checked-out branch goes.
///
/// The branch's upstream when it has one (the remote is the longest known remote name that prefixes the
/// upstream, since a remote may be named `grp/sub`); otherwise `origin`, or the only remote, with the
/// upstream set so the next push needs no choice. Errors are status-line texts.
pub fn push_action(refs: &[Ref], remotes: &[Remote]) -> Result<Action, String> {
    let head = refs
        .iter()
        .find_map(|r| match (&r.kind, r.is_head) {
            (RefKind::LocalBranch { name }, true) => Some((name.clone(), r.upstream.clone())),
            _ => None,
        })
        .ok_or_else(|| strings::PUSH_DETACHED.to_string())?;
    let (branch, upstream) = head;
    if let Some(upstream) = upstream {
        let remote = remotes
            .iter()
            .map(|r| r.name.as_str())
            .filter(|n| {
                upstream
                    .strip_prefix(n)
                    .is_some_and(|rest| rest.starts_with('/'))
            })
            .max_by_key(|n| n.len());
        if let Some(remote) = remote {
            return Ok(Action::Push {
                remote: remote.to_string(),
                opts: PushOptions {
                    branch: Some(branch),
                    ..PushOptions::default()
                },
            });
        }
    }
    let remote = remotes
        .iter()
        .find(|r| r.name == "origin")
        .or_else(|| (remotes.len() == 1).then(|| &remotes[0]))
        .ok_or_else(|| {
            if remotes.is_empty() {
                strings::PUSH_NO_REMOTE.to_string()
            } else {
                strings::PUSH_CHOOSE_REMOTE.to_string()
            }
        })?;
    Ok(Action::Push {
        remote: remote.name.clone(),
        opts: PushOptions {
            branch: Some(branch),
            set_upstream: true,
            ..PushOptions::default()
        },
    })
}

impl Backend {
    /// Starts `what` and returns the status-line text for it: what started, or why nothing did. A
    /// refusal changes nothing. The outcome arrives later (see `app::live`).
    pub async fn start_transfer(&self, what: Transfer) -> Result<String, String> {
        use gitpoulpi_i18n::msg::link as l;
        match what {
            Transfer::Fetch => {
                self.with_link(|link| link.submit_action(Action::FetchAll));
                Ok(l::ACT_FETCHING.get().to_string())
            }
            Transfer::Pull => {
                self.with_link(|link| link.submit_action(Action::Pull(PullOptions::default())));
                Ok(l::ACT_PULLING.get().to_string())
            }
            Transfer::Push => {
                let refs = self.refs().await?;
                let remotes: Vec<Remote> = self
                    .call(
                        gitpoulpi_core::domain::OperationKind::Remotes,
                        serde_json::json!({}),
                    )
                    .await?;
                let action = push_action(&refs, &remotes)?;
                self.with_link(|link| link.submit_action(action));
                Ok(l::ACT_PUSHING.get().to_string())
            }
            Transfer::Undo => self.start_undo_redo(Direction::Undo),
            Transfer::Redo => self.start_undo_redo(Direction::Redo),
        }
    }

    /// The link's own check first: a refusal (nothing to undo, busy, blocked) is an `Err` with its
    /// sentence and sends nothing.
    fn start_undo_redo(&self, direction: Direction) -> Result<String, String> {
        self.with_link(|link| {
            let state = link.journal_state();
            match gitpoulpi_link::undo_ui::reason(&state, direction, link.undo_busy()) {
                Some(why) => Err(why),
                None => Ok(link.start_undo_redo(direction)),
            }
        })
    }

    /// Whether Undo (or Redo) has an entry to act on right now: the toolbar button's enabled state.
    pub fn can_undo_redo(&self, direction: Direction) -> bool {
        self.with_link(|link| {
            gitpoulpi_link::undo_ui::reason(&link.journal_state(), direction, link.undo_busy())
                .is_none()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::{Oid, RefVisibility};

    fn branch(name: &str, head: bool, upstream: Option<&str>) -> Ref {
        Ref {
            kind: RefKind::LocalBranch { name: name.into() },
            target: Oid([0; 20]),
            visibility: RefVisibility::Shown,
            is_head: head,
            upstream: upstream.map(str::to_string),
            tag_sync: None,
            multi_remote: None,
        }
    }

    fn remote(name: &str) -> Remote {
        Remote {
            name: name.into(),
            fetch_url: String::new(),
            push_url: None,
        }
    }

    fn pushed(action: Result<Action, String>) -> (String, PushOptions) {
        match action.unwrap() {
            Action::Push { remote, opts } => (remote, opts),
            other => panic!("not a push: {other:?}"),
        }
    }

    #[test]
    fn a_branch_with_an_upstream_pushes_there() {
        let refs = [
            branch("main", true, Some("grp/sub/main")),
            branch("x", false, None),
        ];
        let remotes = [remote("grp"), remote("grp/sub"), remote("origin")];
        let (remote, opts) = pushed(push_action(&refs, &remotes));
        assert_eq!(
            remote, "grp/sub",
            "the longest remote name that prefixes it"
        );
        assert_eq!(opts.branch.as_deref(), Some("main"));
        assert!(!opts.set_upstream);
    }

    #[test]
    fn a_branch_without_an_upstream_goes_to_origin_and_sets_it() {
        let refs = [branch("topic", true, None)];
        let (to, opts) = pushed(push_action(&refs, &[remote("up"), remote("origin")]));
        assert_eq!(to, "origin");
        assert!(opts.set_upstream);
        let (only, _) = pushed(push_action(&refs, &[remote("up")]));
        assert_eq!(only, "up", "the only remote is the one");
    }

    #[test]
    fn nothing_to_push_to_is_said_not_guessed() {
        let refs = [branch("topic", true, None)];
        assert_eq!(
            push_action(&refs, &[]).unwrap_err(),
            strings::PUSH_NO_REMOTE
        );
        assert_eq!(
            push_action(&refs, &[remote("a"), remote("b")]).unwrap_err(),
            strings::PUSH_CHOOSE_REMOTE
        );
        assert_eq!(
            push_action(&[branch("m", false, None)], &[remote("origin")]).unwrap_err(),
            strings::PUSH_DETACHED
        );
    }
}
