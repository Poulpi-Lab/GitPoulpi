//! Git actions the loop runs directly: checking out a ref (local, or a
//! remote branch pill), and running a confirmed action.

use super::*;

/// What to say when a branch name the UI is holding no longer names a
/// ref - the sidebar row can outlive the branch by up to one refresh.
pub(crate) const NO_SUCH_LOCAL_BRANCH: gitpoulpi_i18n::M =
    gitpoulpi_i18n::msg::git_actions::NO_SUCH_LOCAL_BRANCH;

/// "<name> no longer exists - refresh (r)", with the effective Refresh key.
pub(crate) fn gone(name: &str) -> String {
    gitpoulpi_i18n::msg::git_actions::gone(
        name,
        crate::keymap::paren(
            &crate::keymap::current(),
            crate::keymap::Action::GlobalRefresh,
        ),
    )
}

/// What a one-click network action says when the repository has no default
/// remote to aim at. The default remote is per repository
/// (`RemoteUi::default_remote`, `remotes_ui.rs`), and "none" is a real
/// answer when there are several remotes and none is `origin`.
pub(crate) fn no_default_remote() -> String {
    gitpoulpi_i18n::msg::git_actions::no_default_remote_long(
        gitpoulpi_i18n::msg::sidebar::SECTION_REMOTE,
        gitpoulpi_i18n::msg::remotes::SET_DEFAULT,
        gitpoulpi_i18n::msg::palette::REMOTE_SET_DEFAULT_REMOTE,
    )
}

/// The `Ref` the daemon wants for a local branch the UI knows by name.
///
/// `merge` and `rebase` both take a whole `Ref` rather than a string
/// (`GitBackend` never parses revisions on a frontend's behalf; see the
/// `resolve` note in `git-backend-trait.md`), and both are reachable from
/// the palette *and* the sidebar's context menu: four call sites for one
/// walk, which is how two of them start disagreeing about what matches.
pub(crate) fn find_local_branch_ref(
    refs_result: &gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Ref>>,
    name: &str,
) -> Option<gitpoulpi_core::domain::Ref> {
    refs_result
        .as_ref()
        .ok()?
        .iter()
        .find(|r| matches!(&r.kind, RefKind::LocalBranch { name: n } if n == name))
        .cloned()
}

/// What "check this ref out" means, for **every** path that offers it: the
/// Refs sidebar's double-click, its `Enter`, and the palette's
/// `Refs: Checkout selected ref`. One function, so a mouse gesture and a
/// keystroke cannot reach different conclusions about the same row.
///
/// A local branch or a tag is checked out directly. A **remote** branch
/// cannot be "checked out": git would detach HEAD or DWIM a local branch
/// into existence behind the user's back. So it goes through
/// `refs_tree::remote_checkout_plan`, which gives three outcomes: check out
/// the local branch that already tracks it, offer to create one, or say who
/// has taken the name.
///
/// Returns the sentence for the status line; opens the confirm modal
/// through `input_mode` when a branch would be created.
pub(crate) fn checkout_ref_now(
    name: &str,
    refs_result: &gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Ref>>,
    remotes: &[String],
    link: &DaemonLink,
    input_mode: &mut InputMode,
) -> String {
    let Ok(refs) = refs_result.as_ref() else {
        // Refs failed to load, so nothing can be decided about tracking.
        // Asking for the plain checkout is what the user pressed, and the
        // backend will say no if it cannot.
        link.submit_action(Action::CheckoutBranch(name.to_string()));
        return gitpoulpi_i18n::msg::input::checking_out(name);
    };

    let is_remote_branch = refs.iter().any(|r| match &r.kind {
        RefKind::RemoteBranch { remote, name: n } => format!("{remote}/{n}") == name,
        _ => false,
    });
    if !is_remote_branch {
        link.submit_action(Action::CheckoutBranch(name.to_string()));
        return gitpoulpi_i18n::msg::input::checking_out(name);
    }

    match refs_tree::remote_checkout_plan(name, refs, remotes) {
        refs_tree::RemoteCheckout::CheckoutLocal { local } => {
            link.submit_action(Action::CheckoutBranch(local.clone()));
            gitpoulpi_i18n::msg::git_actions::checking_out_tracks(&local, name)
        }
        refs_tree::RemoteCheckout::AlreadyOnIt { local } => {
            gitpoulpi_i18n::msg::git_actions::already_on(&local, name)
        }
        refs_tree::RemoteCheckout::OfferCreate { local, upstream } => {
            *input_mode = InputMode::Confirm {
                prompt: // Two lines: the shared confirm modal's message area
                // holds four at its smallest (see `close_tab_prompt`),
                // and a long branch name wraps.
                gitpoulpi_i18n::msg::git_actions::offer_create(&upstream, &local),
                action: ConfirmAction::CreateTrackingBranch { local, upstream },
            };
            String::new()
        }
        refs_tree::RemoteCheckout::NameTaken { local, tracks } => match tracks {
            Some(other) => gitpoulpi_i18n::msg::git_actions::name_taken_other(&local, other, name),
            None => gitpoulpi_i18n::msg::git_actions::name_taken_none(&local),
        },
    }
}
/// The `remote/branch` names a **remote pill** may stand for.
///
/// A pill beside the graph shows a remote branch's name *without* its
/// remote (the icon says "remote"), so `main` on a commit may be
/// `origin/main`, `gh/main` or both. `at`, when the caller knows which
/// commit the pill sits on, narrows it to the refs on that commit.
pub(crate) fn remote_pill_candidates(
    branch: &str,
    at: Option<Oid>,
    refs: &[gitpoulpi_core::domain::Ref],
) -> Vec<String> {
    refs.iter()
        .filter(|r| at.is_none_or(|c| r.target == c))
        .filter_map(|r| match &r.kind {
            RefKind::RemoteBranch { remote, name } if name == branch => {
                Some(format!("{remote}/{name}"))
            }
            _ => None,
        })
        .collect()
}

/// Double-click / `Enter` on a remote **pill** (the Log's, or the
/// ref-reveal popup's): the same `checkout_ref_now` the sidebar calls,
/// once the pill's bare name is resolved back to one remote branch.
/// Two remotes with the same branch on the pill's commit cannot be told
/// apart from the pill, so that case says where to go instead of
/// guessing.
pub(crate) fn checkout_remote_pill(
    branch: &str,
    at: Option<Oid>,
    refs_result: &gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Ref>>,
    remotes: &[String],
    link: &DaemonLink,
    input_mode: &mut InputMode,
) -> String {
    let candidates = refs_result
        .as_ref()
        .map(|refs| remote_pill_candidates(branch, at, refs))
        .unwrap_or_default();
    match candidates.as_slice() {
        [one] => checkout_ref_now(one, refs_result, remotes, link, input_mode),
        [] => gitpoulpi_i18n::msg::git_actions::no_remote_branch(branch),
        many => gitpoulpi_i18n::msg::git_actions::several_remotes(
            branch,
            many.join(", "),
            gitpoulpi_i18n::msg::sidebar::SECTION_REMOTE,
        ),
    }
}

/// Carries out a confirmed [`ConfirmAction`], from the keyboard arm and the
/// mouse arm alike. One function because a confirmation can close a tab as
/// well as submit a daemon action, and two copies would be two behaviors.
#[allow(clippy::too_many_arguments)]
pub(crate) fn confirm_now(
    action: ConfirmAction,
    session: &str,
    link: &DaemonLink,
    current: &RepoRef,
    dialogs: &mut RepoDialogs,
    tabs: &mut Vec<repo_switcher::RepoTab>,
    active_tab: &mut usize,
    switch_to: &mut Option<RepoRef>,
    close_window: &mut bool,
) -> Option<String> {
    match action {
        ConfirmAction::Destructive(pending) => {
            let ticket = pending.ticket();
            link.submit_as(ticket, pending.into_action());
            Some(gitpoulpi_i18n::msg::git_actions::CONFIRMED.get().into())
        }
        ConfirmAction::CreateTrackingBranch { local, upstream } => {
            link.submit_action(Action::BranchCreateTracking {
                local_name: local.clone(),
                upstream: upstream.clone(),
            });
            Some(gitpoulpi_i18n::msg::git_actions::creating_tracking(
                &local, &upstream,
            ))
        }
        ConfirmAction::MoveTag {
            name,
            target,
            target_label,
            remotes,
        } => {
            let said = gitpoulpi_i18n::msg::menu_drop::moving_tag(&name, &target_label);
            link.submit_action(Action::TagMove {
                name,
                target,
                force: true,
                target_label,
                remotes,
            });
            Some(said)
        }
        ConfirmAction::OpenLfsInstallPage => Some(open_lfs_install_page()),
        ConfirmAction::AutoStashStartOver => {
            link.submit_action(Action::AutoStashStartOver { confirmed: true });
            Some(
                gitpoulpi_i18n::msg::input::STARTING_OVER_AUTOSTASH
                    .get()
                    .into(),
            )
        }
        ConfirmAction::WriteProfileIdentity { name, email } => {
            link.submit_action(Action::SetRepoConfig {
                key: gitpoulpi_core::repo_config::USER_NAME.into(),
                value: Some(name),
            });
            link.submit_action(Action::SetRepoConfig {
                key: gitpoulpi_core::repo_config::USER_EMAIL.into(),
                value: Some(email),
            });
            Some(gitpoulpi_i18n::msg::profile_bar::written(
                crate::profile_bar::effective_name().unwrap_or_default(),
            ))
        }
        ConfirmAction::ReflogRestore { refname, oid } => {
            let said = gitpoulpi_i18n::msg::undo_safety::RUNNING.get().to_string();
            link.submit_action(Action::ReflogRestore { refname, oid });
            Some(said)
        }
        ConfirmAction::UndoForget { expect } => {
            link.submit_action(Action::UndoForget { expect });
            Some(gitpoulpi_i18n::msg::undo_journal::FORGETTING.get().into())
        }
        ConfirmAction::UndoSequence { group } => {
            link.send_undo_sequence(group, true);
            Some(gitpoulpi_i18n::msg::undo_ui::UNDOING_SEQUENCE.get().into())
        }
        ConfirmAction::UndoRedo { direction, expect } => {
            link.send_undo_redo(direction, expect, true);
            Some(match direction {
                gitpoulpi_core::domain::Direction::Undo => {
                    gitpoulpi_i18n::msg::git_actions::UNDOING.get().into()
                }
                gitpoulpi_core::domain::Direction::Redo => {
                    gitpoulpi_i18n::msg::git_actions::REDOING.get().into()
                }
            })
        }
        ConfirmAction::DeleteWorkspace(id) => crate::workspace_ui::apply::delete_now(
            &id,
            session,
            current,
            tabs,
            active_tab,
            switch_to,
            close_window,
        ),
        ConfirmAction::CloseTabs(paths) => close_tabs_now(
            &paths,
            session,
            current,
            tabs,
            active_tab,
            switch_to,
            close_window,
        ),
        ConfirmAction::CloseTab(index) => apply_switcher_action(
            repo_switcher::SwitcherAction::Close(index),
            session,
            link,
            current,
            dialogs,
            tabs,
            active_tab,
            switch_to,
            close_window,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::*;

    /// A remote pill shows the branch without its remote, so it has to
    /// be resolved back to one `remote/branch` before anything can be
    /// checked out - and two remotes on the same commit cannot be.
    #[test]
    fn a_remote_pill_resolves_to_its_remote_branch_or_to_the_ambiguity() {
        let refs = vec![
            remote_branch_at("origin", "main", oid(1)),
            remote_branch_at("gh", "main", oid(2)),
            remote_branch_at("origin", "feature/login", oid(3)),
            local_branch_at("main", oid(1)),
        ];
        assert_eq!(
            remote_pill_candidates("feature/login", None, &refs),
            vec!["origin/feature/login".to_string()]
        );
        assert_eq!(
            remote_pill_candidates("main", None, &refs),
            vec!["origin/main".to_string(), "gh/main".to_string()]
        );
        // Knowing the pill's commit settles it.
        assert_eq!(
            remote_pill_candidates("main", Some(oid(2)), &refs),
            vec!["gh/main".to_string()]
        );
        // A local branch of the name is not a candidate.
        assert!(remote_pill_candidates("main", Some(oid(3)), &refs).is_empty());
    }
}
