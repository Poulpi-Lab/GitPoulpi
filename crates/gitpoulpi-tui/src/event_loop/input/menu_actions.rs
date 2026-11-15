//! Menu and dialog actions that need most of the window's state, shared by
//! the keyboard and mouse paths so the two cannot drift.

use super::commands::CommandCtx;
use crate::event_loop::*;
use gitpoulpi_i18n::msg;

pub(in crate::event_loop) fn apply_effect(cx: &mut CommandCtx<'_>, effect: DialogEffect) {
    match effect {
        DialogEffect::GeneratePullRequest { source, target } => {
            cx.ai_ui.start_into(
                cx.session,
                cx.repo,
                AiRequest::PullRequest { source, target },
                &mut *cx.last_message,
            );
            // Not started (AI off, another request running): the
            // form must not stay "generating".
            if !cx.ai_ui.running_pull_request() {
                cx.repo_dialogs.pull_request_generated(Err(cx
                    .last_message
                    .take()
                    .unwrap_or_else(|| msg::input::COULD_NOT_START.get().to_string())));
            }
        }
        DialogEffect::NameWorktree {
            description,
            branches,
        } => {
            cx.ai_ui.start_into(
                cx.session,
                cx.repo,
                AiRequest::WorktreeName {
                    description,
                    branches,
                },
                &mut *cx.last_message,
            );
            // Not started (AI off, another request running): the dialog
            // must not stay "asking".
            if !cx.ai_ui.running_worktree_name() {
                cx.repo_dialogs.worktree_named(Err(cx
                    .last_message
                    .take()
                    .unwrap_or_else(|| msg::input::COULD_NOT_START.get().to_string())));
            }
        }
        DialogEffect::StopWorktreeAi => cx.ai_ui.cancel(),
        DialogEffect::OpenIntegrations => {
            super::apply_card_command(cx, crate::activity_log::Command::OpenIntegrations);
        }
        DialogEffect::ShowCommit(commit) => {
            if select_log_commit(&*cx.log_result, &*cx.log_order, &mut *cx.log_state, commit) {
                *cx.focus = Focus::Log;
                *cx.right_panel = RightPanel::CommitDetail;
            } else {
                *cx.last_message = Some(msg::input::not_in_loaded_log(&commit.to_hex()[..8]));
            }
        }
        DialogEffect::ShowCommitPrefix(sha) => {
            let found = cx.log_result.as_ref().ok().and_then(|g| {
                g.nodes
                    .iter()
                    .find(|n| n.id.to_hex().starts_with(&sha))
                    .map(|n| n.id)
            });
            match found {
                Some(commit)
                    if select_log_commit(
                        &*cx.log_result,
                        &*cx.log_order,
                        &mut *cx.log_state,
                        commit,
                    ) =>
                {
                    *cx.focus = Focus::Log;
                    *cx.right_panel = RightPanel::CommitDetail;
                }
                _ => {
                    let short: String = sha.chars().take(8).collect();
                    *cx.last_message = Some(msg::input::not_in_loaded_log(&short));
                }
            }
        }
        #[cfg(feature = "hosting")]
        DialogEffect::CheckoutPullRequest(number) => {
            let refs = cx.refs_result.as_deref().unwrap_or(&[]);
            let effect = cx
                .hosting
                .checkout_pr_effect(number, refs, &mut *cx.clipboard);
            commands::apply_hosting_effect(cx, effect);
        }
        other => {
            if let Some(message) = apply_dialog_effect(other, &mut tab_ctx!(cx)) {
                // The Home tab covers the status bar: it says it in its footer.
                if let Some(manager) = cx.management.as_mut() {
                    manager.message = Some(message.clone());
                }
                *cx.last_message = Some(message);
            }
        }
    }
}

pub(in crate::event_loop) fn apply_wip_action(cx: &mut CommandCtx<'_>, action: WipContextAction) {
    if action == WipContextAction::AiExplain {
        commands::explain_wip(cx);
    } else if action == WipContextAction::AiCompose {
        commands::start_ai(cx, AiRequest::Compose);
    } else {
        apply_wip_context_action(
            action,
            &cx.repo.root,
            &*cx.refs_result,
            &*cx.status_result,
            cx.link,
            &mut *cx.repo_dialogs,
            &mut *cx.input_mode,
            &mut *cx.last_message,
            &mut *cx.patch_seed,
        );
    }
}

pub(in crate::event_loop) fn apply_folder_menu_action(
    cx: &mut CommandCtx<'_>,
    section: StatusSection,
    path: PathBuf,
    action: FolderContextAction,
) {
    if let FolderContextAction::Fold(op) = action {
        let list = match section {
            StatusSection::Unstaged => FileList::Unstaged,
            StatusSection::Staged => FileList::Staged,
        };
        let root = (!path.as_os_str().is_empty()).then_some(path.as_path());
        commands::fold_files(cx, list, op, root);
        return;
    }
    if matches!(action, FolderContextAction::Convert(_)) {
        let under: Vec<PathBuf> = cx
            .status_result
            .as_ref()
            .ok()
            .map(|t| {
                t.files
                    .iter()
                    .filter(|f| f.path.starts_with(&path))
                    .map(|f| f.path.clone())
                    .collect()
            })
            .unwrap_or_default();
        if let Some(open) = commands::editor_blocks(cx, under.iter().map(PathBuf::as_path)) {
            *cx.last_message = Some(msg::pane_menu::editor_open(open.display()));
            return;
        }
    }
    match cx.status_result.as_ref().ok() {
        Some(tree) => run_folder_context_action(
            action,
            cx.repo_root,
            &path,
            tree,
            cx.link,
            &mut *cx.input_mode,
            &mut *cx.last_message,
            &mut *cx.patch_seed,
        ),
        None if action == FolderContextAction::CreateFile => {
            start_create_file(&path, &mut *cx.input_mode)
        }
        None => *cx.last_message = Some(msg::input::NO_WORKING_TREE.get().into()),
    }
}

pub(in crate::event_loop) fn run_log_menu_action(
    cx: &mut CommandCtx<'_>,
    action: LogContextAction,
    commit: Oid,
) {
    match action {
        // Renamed where the pill is: the box opens in the tip's Log row.
        LogContextAction::Ref(RefContextAction::RenameBranch, index) => {
            if let Some((_, name)) = cx.log_menu_refs.get(index as usize).cloned() {
                *cx.input_mode = InputMode::InlineName {
                    op: RefNameOp::RenameBranch {
                        name: name.clone(),
                        spot: RenameSpot::Pill,
                    },
                    value: name,
                };
            }
        }
        LogContextAction::Ref(ref_action, index) => {
            if let Some((section, name)) = cx.log_menu_refs.get(index as usize).cloned() {
                apply_ref_context_action(
                    ref_action,
                    section,
                    &name,
                    RefActionCtx {
                        refs_result: &*cx.refs_result,
                        log_result: &*cx.log_result,
                        status_result: &*cx.status_result,
                        hidden_from_graph: &mut *cx.hidden_from_graph,
                        solo_refs: &mut *cx.solo_refs,
                        log_len: &mut *cx.log_len,
                        log_order: &mut *cx.log_order,
                        stashes_result: &*cx.stashes_result,
                        graph_show_stashes: *cx.graph_show_stashes,
                        log_state: &mut *cx.log_state,
                        link: cx.link,
                        clipboard: &mut *cx.clipboard,
                        input_mode: &mut *cx.input_mode,
                        last_message: &mut *cx.last_message,
                        remotes: &*cx.remote_names,
                        remote_ui: &mut *cx.remote_ui,
                        worktrees: cx.worktrees_result.as_deref().unwrap_or(&[]),
                        worktree_seed: &mut *cx.worktree_seed,
                        pull_request_seed: &mut *cx.pull_request_seed,
                        drop_source: cx.ref_drop_source.as_ref(),
                        submodules: &*cx.submodules_result,
                        repo_root: &cx.repo.root,
                        open_in_tab: &mut *cx.submodule_open,
                        reveal: &mut *cx.reveal,
                    },
                );
                refs_tree::submit_ref_visibility(cx.link, &*cx.hidden_from_graph, &*cx.solo_refs);
            }
        }
        LogContextAction::ApplyPatch => cx.repo_dialogs.open_apply_patch(&cx.repo.root),
        LogContextAction::RecoverLost => {
            cx.repo_dialogs
                .open_safety(cx.link, crate::safety_panel::Tab::Lost, None)
        }
        LogContextAction::RewriteHistory(from) => {
            super::keys_prefix::open_rewrite_from_menu(cx, from, commit)
        }
        LogContextAction::Column(op) => commands::commit_column::run_op(cx, op, commit),
        LogContextAction::AiExplain => commands::explain_commit(cx, commit),
        LogContextAction::AiSplitCommit => commands::split_commit_with_ai(cx, commit),
        other => apply_log_context_action(
            other,
            commit,
            &*cx.log_result,
            &*cx.refs_result,
            cx.link,
            &mut *cx.clipboard,
            &mut *cx.input_mode,
            &mut *cx.last_message,
            &mut *cx.worktree_seed,
            &mut *cx.patch_seed,
            &cx.log_multi.newest_first(&*cx.log_result),
            &cx.remote_ui.remotes,
        ),
    }
}

/// The toolbar's and the Status pane's "compose commits" button: refuses (with a
/// status line) when AI is off or nothing is uncommitted.
pub(in crate::event_loop) fn compose_commits(cx: &mut CommandCtx<'_>) {
    if !cx.ai_ui.enabled() {
        *cx.last_message = Some(msg::input::AI_OFF.get().into());
    } else if !has_wip_row(&*cx.status_result) {
        *cx.last_message = Some(msg::input::NO_CHANGES_TO_SPLIT.get().into());
    } else {
        commands::start_ai(cx, AiRequest::Compose);
    }
}
