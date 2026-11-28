//! What dropping one ref on another does, and the branch-level actions the
//! drop shares with the Pull and Push buttons.
//!
//! A frontend shows the rows it likes (they depend on its menu widgets); once
//! a row is chosen, [`plan_drop`] turns it into one [`Action`] and the line to
//! say while it runs, so the terminal UI and the desktop run the same thing.
//! The one exception is the interactive rebase of a dropped branch, which
//! [`plan_drop`] hands back as a [`RebaseDrop`] for the frontend's own rebase
//! form.

use crate::Action;
use gitpoulpi_app::refs_tree::RefRowKind;
use gitpoulpi_config::PullButtonMode;
use gitpoulpi_core::domain::{CommitGraph, Oid, PullMode, PullOptions, PushOptions, Ref, RefKind};
use gitpoulpi_i18n::msg::{menu_drop as msg, remotes};

/// The checked-out branch (or any local branch): its name and its upstream
/// (`origin/main`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadBranch {
    pub name: String,
    pub upstream: Option<String>,
}

/// The checked-out *branch*, or `None` on a detached HEAD.
pub fn head_branch(refs: &[Ref]) -> Option<HeadBranch> {
    refs.iter().find_map(|r| match &r.kind {
        RefKind::LocalBranch { name } if r.is_head => Some(HeadBranch {
            name: name.clone(),
            upstream: r.upstream.clone(),
        }),
        _ => None,
    })
}

/// A local branch by name, as a [`HeadBranch`] (name + upstream).
pub fn local_branch(refs: &[Ref], branch: &str) -> Option<HeadBranch> {
    refs.iter().find_map(|r| match &r.kind {
        RefKind::LocalBranch { name } if name == branch => Some(HeadBranch {
            name: name.clone(),
            upstream: r.upstream.clone(),
        }),
        _ => None,
    })
}

/// The action the Pull button (and `Ctrl+G p`) runs in `mode`, with its
/// status line - or why it cannot run.
///
/// A branch with no upstream pulls its same-named branch from the
/// default remote, rather than failing the way a bare `git pull` would.
pub fn pull_action(
    mode: PullButtonMode,
    head: Option<&HeadBranch>,
    default_remote: Option<&str>,
) -> Result<(Action, String), String> {
    let mode = match mode {
        PullButtonMode::Merge => PullMode::Merge,
        PullButtonMode::FfOnly => PullMode::FastForwardOnly,
        PullButtonMode::Rebase => PullMode::Rebase,
    };
    let head = head.ok_or_else(|| remotes::DETACHED_PULL.get().to_string())?;
    let remote = if head.upstream.is_some() {
        None
    } else {
        Some(
            default_remote
                .map(str::to_string)
                .ok_or_else(|| remotes::no_upstream_no_default(&head.name))?,
        )
    };
    let from = match (&head.upstream, &remote) {
        (Some(up), _) => up.clone(),
        (None, Some(r)) => format!("{r}/{}", head.name),
        (None, None) => unreachable!("handled above"),
    };
    Ok((
        Action::Pull(PullOptions {
            mode,
            remote,
            ..PullOptions::default()
        }),
        remotes::pulling_from(from, &head.name),
    ))
}

/// Pushing `branch` to `remote`, setting the upstream when the branch
/// has none (the first push of a new branch).
pub fn push_action(remote: &str, branch: &HeadBranch, force_with_lease: bool) -> Action {
    Action::Push {
        remote: remote.to_string(),
        opts: PushOptions {
            force_with_lease,
            set_upstream: branch.upstream.is_none(),
            branch: Some(branch.name.clone()),
            ..PushOptions::default()
        },
    }
}

/// One choice of the menu a drop opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropOp {
    /// Merge the dragged ref into the target branch.
    Merge,
    /// Local X onto Y: replay X on Y. Remote R onto Y: replay Y on R.
    Rebase,
    /// Open the interactive rebase popup for X's commits onto Y.
    InteractiveRebase,
    /// Move the target branch forward to the dragged ref.
    FastForward,
    /// Push the dragged local branch to the target's remote.
    Push,
    /// Pull the dragged remote branch into the (local) target.
    Pull,
    /// Move the dragged tag to the target's commit (a fast-forward of the
    /// tag: the target descends from the tag's commit).
    MoveTag,
    /// The same move when it is not a fast-forward, offered with a warning.
    MoveTagForced,
    /// `git merge <tag>` on the target branch.
    MergeTagIntoBranch,
    /// Merge the target branch into the tag's commit, detached, then move
    /// the tag onto the merge commit.
    MergeBranchIntoTag,
}

/// The inputs of the interactive-rebase form a drop opens: the frontend
/// builds its rows from the loaded graph (`source_tip` back to the merge base
/// with `target_tip`) and replays them onto `target_tip`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RebaseDrop {
    /// The dragged branch's name.
    pub source: String,
    pub source_tip: Oid,
    pub target_tip: Oid,
    /// The branch to check out before starting, `None` when `source` already
    /// is the checked-out one.
    pub checkout: Option<String>,
    /// The target's name, for the header ("onto <target>").
    pub target: String,
}

/// What a chosen [`DropOp`] turns into.
#[derive(Debug, Clone, PartialEq)]
pub enum DropPlan {
    /// Submit this action and say this on the status line.
    Submit(Action, String),
    /// Open the interactive rebase form for these commits.
    Rebase(RebaseDrop),
}

/// `then`, preceded by checking `branch` out unless it already is.
fn after_checkout(branch: &str, head: Option<&HeadBranch>, then: Action) -> Action {
    if head.is_some_and(|h| h.name == branch) {
        then
    } else {
        Action::CheckoutThen {
            branch: branch.to_string(),
            then: Box::new(then),
        }
    }
}

/// "<name> no longer exists - refresh (r)", with the effective Refresh key.
fn gone(name: &str) -> String {
    gitpoulpi_i18n::msg::git_actions::gone(
        name,
        gitpoulpi_app::keymap::paren(
            &gitpoulpi_app::keymap::current(),
            gitpoulpi_app::keymap::Action::GlobalRefresh,
        ),
    )
}

/// What `op` does for this drop. `Err` is a status line saying why not.
/// `remotes` are the repository's remote names (to split `origin/x`),
/// `pull_mode` the Pull button's mode (a dropped remote branch pulls with it).
#[allow(clippy::too_many_arguments)]
pub fn plan_drop(
    op: DropOp,
    source: &str,
    source_kind: RefRowKind,
    target: &str,
    target_kind: RefRowKind,
    refs: &[Ref],
    graph: Option<&CommitGraph>,
    remotes: &[String],
    pull_mode: PullButtonMode,
) -> Result<DropPlan, String> {
    use gitpoulpi_app::ref_rules::{find_ref, tag_remotes};
    let src = find_ref(refs, source_kind, source).ok_or_else(|| gone(source))?;
    let tgt = find_ref(refs, target_kind, target).ok_or_else(|| gone(target))?;
    let head = head_branch(refs);
    match (op, source_kind, target_kind) {
        // Both directions merge `src` into the local branch `target`.
        (DropOp::Merge, _, RefRowKind::Local) => Ok(DropPlan::Submit(
            after_checkout(target, head.as_ref(), Action::Merge(src.clone())),
            msg::merging(source, target),
        )),
        (DropOp::Rebase, RefRowKind::Local, RefRowKind::Local) => Ok(DropPlan::Submit(
            after_checkout(source, head.as_ref(), Action::Rebase(tgt.clone())),
            msg::rebasing_source(source, target),
        )),
        (DropOp::Rebase, RefRowKind::Remote, RefRowKind::Local) => Ok(DropPlan::Submit(
            after_checkout(target, head.as_ref(), Action::Rebase(src.clone())),
            msg::rebasing_target(target, source),
        )),
        (DropOp::InteractiveRebase, RefRowKind::Local, RefRowKind::Local) => {
            graph.ok_or(msg::NO_HISTORY.get())?;
            let checkout = (head.as_ref().map(|h| h.name.as_str()) != Some(source))
                .then(|| source.to_string());
            Ok(DropPlan::Rebase(RebaseDrop {
                source: source.to_string(),
                source_tip: src.target,
                target_tip: tgt.target,
                checkout,
                target: target.to_string(),
            }))
        }
        (DropOp::MoveTag | DropOp::MoveTagForced, RefRowKind::Tag, _) => {
            // Where the tag lives besides here: the remote keeps its old
            // position until the owner pushes (the tag turns `differs`).
            let remotes = tag_remotes(src);
            let said = if remotes.is_empty() {
                msg::moving_tag(source, target)
            } else {
                msg::moving_tag_remote(source, target, remotes.join(", "))
            };
            Ok(DropPlan::Submit(
                Action::TagMove {
                    name: source.to_string(),
                    target: tgt.target,
                    force: op == DropOp::MoveTagForced,
                    target_label: target.to_string(),
                    remotes,
                },
                said,
            ))
        }
        (DropOp::MergeTagIntoBranch, RefRowKind::Tag, RefRowKind::Local) => Ok(DropPlan::Submit(
            after_checkout(target, head.as_ref(), Action::Merge(src.clone())),
            msg::merging(source, target),
        )),
        (DropOp::MergeBranchIntoTag, RefRowKind::Tag, RefRowKind::Local) => Ok(DropPlan::Submit(
            Action::TagMergeBranch {
                name: source.to_string(),
                branch: target.to_string(),
            },
            msg::merging_into_tag(target, source),
        )),
        (DropOp::FastForward, _, RefRowKind::Local) => Ok(DropPlan::Submit(
            Action::FastForward {
                branch: target.to_string(),
                target: src.target,
                target_label: source.to_string(),
            },
            gitpoulpi_i18n::msg::menu_ref::fast_forwarding(target, source),
        )),
        (DropOp::Push, RefRowKind::Local, RefRowKind::Remote) => {
            let (remote, _) = gitpoulpi_core::domain::split_remote_ref(target, remotes)
                .ok_or_else(|| msg::not_on_remote(target))?;
            let branch = HeadBranch {
                name: source.to_string(),
                upstream: src.upstream.clone(),
            };
            Ok(DropPlan::Submit(
                push_action(&remote, &branch, false),
                gitpoulpi_i18n::msg::menu_ref::pushing(source, &remote),
            ))
        }
        (DropOp::Pull, RefRowKind::Remote, RefRowKind::Local) => {
            let branch = HeadBranch {
                name: target.to_string(),
                upstream: tgt.upstream.clone(),
            };
            let (pull, said) = pull_action(pull_mode, Some(&branch), None)?;
            Ok(DropPlan::Submit(
                after_checkout(target, head.as_ref(), pull),
                said,
            ))
        }
        _ => Err(msg::cannot_drop(source, target)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::RefVisibility;

    fn branch(name: &str, n: u8, is_head: bool) -> Ref {
        Ref {
            kind: RefKind::LocalBranch { name: name.into() },
            target: Oid([n; 20]),
            visibility: RefVisibility::Shown,
            is_head,
            upstream: None,
            tag_sync: None,
            multi_remote: None,
        }
    }

    fn plan(op: DropOp, source: &str, target: &str, refs: &[Ref]) -> Result<DropPlan, String> {
        plan_drop(
            op,
            source,
            RefRowKind::Local,
            target,
            RefRowKind::Local,
            refs,
            None,
            &[],
            PullButtonMode::Merge,
        )
    }

    #[test]
    fn merging_into_a_branch_checks_it_out_first_unless_it_is_checked_out() {
        let refs = vec![branch("main", 1, true), branch("feat", 2, false)];
        let Ok(DropPlan::Submit(Action::Merge(_), _)) = plan(DropOp::Merge, "feat", "main", &refs)
        else {
            panic!("main is checked out: a plain merge");
        };
        let Ok(DropPlan::Submit(Action::CheckoutThen { branch, .. }, _)) =
            plan(DropOp::Merge, "main", "feat", &refs)
        else {
            panic!("feat is not checked out: a checkout first");
        };
        assert_eq!(branch, "feat");
    }

    #[test]
    fn a_vanished_ref_and_an_impossible_pair_are_refused() {
        let refs = vec![branch("main", 1, true)];
        assert!(plan(DropOp::Merge, "gone", "main", &refs).is_err());
        assert!(plan(DropOp::Push, "main", "main", &refs).is_err());
    }

    #[test]
    fn an_interactive_rebase_needs_the_loaded_history() {
        let refs = vec![branch("main", 1, true), branch("feat", 2, false)];
        assert!(plan(DropOp::InteractiveRebase, "feat", "main", &refs).is_err());
    }
}
