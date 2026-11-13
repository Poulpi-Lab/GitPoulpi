use super::*;
use gitpoulpi_i18n::msg::menu_log as msg;

/// Runs one row of the commit context menu, from the mouse, from the
/// keyboard, from the `Ctrl+G` chords or from the palette. One function
/// so the paths cannot disagree about which rows confirm and which don't.
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_log_context_action(
    action: LogContextAction,
    commit: Oid,
    log_result: &gitpoulpi_core::Result<gitpoulpi_core::domain::CommitGraph>,
    refs: &gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Ref>>,
    link: &DaemonLink,
    clipboard: &mut clipboard::Clipboard,
    input_mode: &mut InputMode,
    last_message: &mut Option<String>,
    worktree_seed: &mut Option<WorktreeSeed>,
    patch_seed: &mut Option<crate::repo_dialogs::PatchSeed>,
    selected: &[Oid],
    remotes: &[gitpoulpi_core::domain::Remote],
) {
    // A merge commit needs its mainline parent first: the dropdown, whose
    // pick then goes on to the confirm (`apply_revert_mainline`).
    if action == LogContextAction::Revert {
        if let Ok(graph) = log_result.as_ref() {
            if let Some(parents) = crate::revert_mainline::merge_parents(graph, commit) {
                crate::revert_mainline::open(graph, commit, &parents);
                return;
            }
        }
    }
    if let Some((prompt, pending)) = log_action_confirmation(action, commit) {
        *input_mode = InputMode::Confirm {
            prompt,
            action: ConfirmAction::Destructive(pending),
        };
        return;
    }
    let short = &commit.to_hex()[..SHORT_SHA_LEN];
    match action {
        // Several commits selected: the interactive view, straight away
        // (no intermediate choice); one commit keeps the direct action.
        LogContextAction::CherryPick if selected.len() > 1 => {
            match cherry_pick_form_for(log_result, refs, selected) {
                Ok(form) => *input_mode = InputMode::InteractiveRebase(Box::new(form)),
                Err(reason) => *last_message = Some(reason),
            }
        }
        LogContextAction::CherryPick => {
            link.submit_action(Action::CherryPick(commit));
            *last_message = Some(msg::cherry_picking(short));
        }
        LogContextAction::Reset(mode) => {
            link.submit_action(Action::Reset {
                target: commit,
                mode,
            });
            *last_message = Some(msg::resetting(format!("{mode:?}"), short));
        }
        // Both reuse the existing name prompt rather than inventing a
        // second one -- `RefNameOp::NewBranchAt`/`NewTagAt` differ from
        // `NewBranch`/`NewTag` only in pointing at this commit instead of
        // at HEAD, so Enter, Esc and the field's editing behave exactly
        // as they already do everywhere else.
        // Detaching HEAD is not destructive -- git keeps every commit,
        // and the branch you left is still where it was -- so it does not
        // take the confirm modal. It does say so in the status line,
        // because "detached HEAD" surprises people.
        LogContextAction::CheckoutCommit => {
            link.submit_action(Action::CheckoutCommit(commit));
            *last_message = Some(msg::checking_out(short));
        }
        LogContextAction::CreateBranchHere => {
            *input_mode = InputMode::InlineName {
                op: RefNameOp::NewBranchAt(commit),
                value: String::new(),
            };
        }
        LogContextAction::CreateTagHere => {
            *input_mode = InputMode::InlineName {
                op: RefNameOp::NewTagAt(commit),
                value: String::new(),
            };
        }
        // The dialog is `repo_dialogs`', opened by the event loop from
        // the seed: detached here unless a new branch is named.
        LogContextAction::CreateWorktree => {
            *worktree_seed = Some(WorktreeSeed {
                start: gitpoulpi_core::domain::WorktreeStart::Commit(commit),
                label: short.to_string(),
                new_branch: String::new(),
                branch_busy: false,
            });
        }
        LogContextAction::CreatePatch => {
            let slug = loaded_commit(log_result, commit)
                .map(|n| slugify(&n.summary))
                .unwrap_or_default();
            let name = if slug.is_empty() {
                short.to_string()
            } else {
                format!("{short}-{slug}")
            };
            *patch_seed = Some(crate::repo_dialogs::PatchSeed {
                source: gitpoulpi_core::domain::PatchSource::Commits(vec![commit]),
                name,
            });
        }
        LogContextAction::CreatePatchToHead => {
            match refs.as_ref().ok().and_then(|rs| refs_sidebar::head_oid(rs)) {
                Some(head) => {
                    *patch_seed = Some(crate::repo_dialogs::PatchSeed {
                        source: gitpoulpi_core::domain::PatchSource::Range {
                            base: commit,
                            tip: head,
                        },
                        name: format!("{short}..{}", &head.to_hex()[..SHORT_SHA_LEN]),
                    });
                }
                None => *last_message = Some(msg::NO_HEAD.get().into()),
            }
        }
        LogContextAction::EditMessage => match reword_form_for(log_result, refs, commit) {
            Ok(form) => *input_mode = InputMode::EditingCommitMessage(Box::new(form)),
            Err(why) => *last_message = Some(why),
        },
        LogContextAction::Amend(scope) => match amend_form_for(log_result, refs, commit, scope) {
            Ok(form) => *input_mode = InputMode::EditingCommitMessage(Box::new(form)),
            Err(why) => *last_message = Some(why),
        },
        LogContextAction::SplitCommit => match split_precheck(log_result, refs, commit) {
            Ok(()) => link.request_split_source(commit),
            Err(why) => *last_message = Some(why),
        },
        LogContextAction::InteractiveRebase => match rebase_form_for(log_result, refs, commit) {
            Ok(form) => *input_mode = InputMode::InteractiveRebase(Box::new(form)),
            Err(why) => *last_message = Some(why),
        },
        LogContextAction::Copy(field) => {
            *last_message = Some(match loaded_commit(log_result, commit) {
                Some(node) => clipboard.copy(field.label(), &field.value(node)),
                // The short/full SHA are still copyable without the node,
                // because the `Oid` *is* the value -- refusing there would
                // be a dead entry for no reason.
                None => match field {
                    CommitField::ShortSha => clipboard.copy(field.label(), short),
                    CommitField::LongSha => clipboard.copy(field.label(), &commit.to_hex()),
                    _ => msg::not_loaded(short),
                },
            });
        }
        LogContextAction::CopyAsMermaid => {
            let refs = refs.as_ref().map(Vec::as_slice).unwrap_or(&[]);
            match log_result.as_ref() {
                Ok(graph) => {
                    if let Err(why) = crate::mermaid_popup::open(graph, refs, selected, commit) {
                        *last_message = Some(why);
                    }
                }
                Err(_) => *last_message = Some(msg::not_loaded(short)),
            }
        }
        LogContextAction::CreateAnnotatedTagHere => {
            *input_mode = InputMode::InlineName {
                op: RefNameOp::NewAnnotatedTagAt(commit),
                value: String::new(),
            };
        }
        LogContextAction::CopyCommitLink => {
            *last_message = Some(copy_remote_link(
                &LinkTarget::Commit(commit),
                remotes,
                None,
                clipboard,
            ));
        }
        // `Ref` and `ApplyPatch` need state this function does not take
        // (the ref table, the dialogs); the event loop runs them.
        LogContextAction::Ref(..)
        | LogContextAction::ApplyPatch
        | LogContextAction::RecoverLost => {}
        // Returned above, via `log_action_confirmation`.
        LogContextAction::Revert => {}
        // Run by the event loop, which owns the AI worker.
        LogContextAction::AiExplain | LogContextAction::AiSplitCommit => {}
        // Run by the event loop, which owns the open view.
        LogContextAction::Column(_) => {}
        // Run by the event loop, which owns the workbench's context.
        LogContextAction::RewriteHistory(_) => {}
    }
}

/// What picking a parent in the merge-revert dropdown does: the confirm
/// modal, as for any revert, now naming the mainline.
pub(crate) fn apply_revert_mainline(
    pick: crate::revert_mainline::Pick,
    input_mode: &mut InputMode,
) {
    let short = &pick.commit.to_hex()[..SHORT_SHA_LEN];
    *input_mode = InputMode::Confirm {
        prompt: msg::confirm_revert_merge(short, pick.parent),
        action: ConfirmAction::Destructive(PendingDestructive::Revert {
            commit: pick.commit,
            mainline: Some(pick.parent),
        }),
    };
}

/// Runs one row of the WIP row's context menu - the same reuse
/// `apply_log_context_action`'s doc comment asks for, but over
/// `WipContextAction` instead: `CreatePatch` seeds `patch_seed` exactly
/// the way `PaletteCommand::CreatePatchFromWip` does, `ApplyPatch` opens
/// the same file picker `PaletteCommand::ApplyPatch` does, and the three
/// whole-tree actions submit the same `Action`s their toolbar/palette
/// twins do.
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_wip_context_action(
    action: WipContextAction,
    repo_root: &Path,
    refs: &gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Ref>>,
    status_result: &gitpoulpi_core::Result<WorkingTree>,
    link: &DaemonLink,
    repo_dialogs: &mut RepoDialogs,
    input_mode: &mut InputMode,
    last_message: &mut Option<String>,
    patch_seed: &mut Option<crate::repo_dialogs::PatchSeed>,
) {
    match action {
        WipContextAction::CreatePatch => {
            let branch = refs.as_ref().ok().and_then(|rs| {
                rs.iter().find_map(|r| match (&r.kind, r.is_head) {
                    (RefKind::LocalBranch { name }, true) => Some(name.clone()),
                    _ => None,
                })
            });
            let name = match branch {
                Some(branch) => format!("wip-{branch}"),
                None => "wip".to_string(),
            };
            *patch_seed = Some(crate::repo_dialogs::PatchSeed {
                source: gitpoulpi_core::domain::PatchSource::WorkingTree { paths: vec![] },
                name,
            });
        }
        WipContextAction::ApplyPatch => {
            repo_dialogs.open_apply_patch(repo_root);
        }
        WipContextAction::AiExplain | WipContextAction::AiCompose => {}
        WipContextAction::StashAll => {
            link.submit_action(Action::StashPush);
            *last_message = Some(msg::STASHING.get().into());
        }
        WipContextAction::StageAll => {
            link.submit_action(Action::StageAll);
        }
        WipContextAction::DiscardAll => {
            discard_all_changes(status_result.as_ref().ok(), input_mode, last_message);
        }
    }
}
