//! Git operations: files, sync, stash, copy, file views, commit and rebase
//! rows, LFS, conflicts.

use crate::event_loop::*;
use gitpoulpi_core::domain::ConflictSide;
use gitpoulpi_i18n::msg;
use gitpoulpi_session::PendingAutoStashState;

use super::{pane::*, shared::*, CommandCtx};

pub(super) fn run(cx: &mut CommandCtx<'_>, cmd: PaletteCommand, sel_ref: &Option<String>) -> bool {
    let sel_ref = sel_ref.clone();
    match cmd {
        // -- Git -------------------------------
        PaletteCommand::FileMenu(action) => {
            let other = match *cx.active_status_section {
                StatusSection::Unstaged => StatusSection::Staged,
                StatusSection::Staged => StatusSection::Unstaged,
            };
            let selected = [*cx.active_status_section, other]
                .into_iter()
                .find_map(|section| {
                    selected_status_path_in(
                        &*cx.status_result,
                        &*cx.status_states,
                        section,
                        *cx.status_list_mode,
                        &*cx.collapsed_status_folders,
                    )
                    .map(|path| (section, path))
                });
            match selected {
                Some((section, path)) => {
                    apply_file_menu_action(cx, section, path, action);
                }
                None => {
                    *cx.last_message = Some(msg::input::SELECT_CHANGED_FILE.get().into());
                }
            }
        }
        PaletteCommand::FilesOpenMenu => {
            if !open_menu_from_keyboard(cx) {
                match *cx.focus {
                    Focus::Status => open_status_row_menu(cx),
                    _ => *cx.last_message = Some(msg::pane_menu::SELECT_FILES_FIRST.get().into()),
                }
            }
        }
        PaletteCommand::FilesSelectAll => select_all(cx),
        PaletteCommand::FilesClearSelection => clear_selection(cx),
        PaletteCommand::FilesToggleSelection => toggle_cursor(cx),
        PaletteCommand::CreateFile => {
            start_create_file(Path::new(""), &mut *cx.input_mode);
        }
        PaletteCommand::OpenFolderMenu => {
            let other = match *cx.active_status_section {
                StatusSection::Unstaged => StatusSection::Staged,
                StatusSection::Staged => StatusSection::Unstaged,
            };
            let found = [*cx.active_status_section, other]
                .into_iter()
                .find_map(|section| {
                    let state = cx.status_states.state(section);
                    let i = state.selected()?;
                    match section_lines(&*cx.status_section_lines, section).get(i) {
                        Some(StatusPaneLine::Folder { path, .. }) => {
                            Some((section, path.clone(), i))
                        }
                        _ => None,
                    }
                });
            match found {
                Some((section, path, i)) => {
                    let area = cx
                        .status_section_areas
                        .iter()
                        .find(|(s, _)| *s == section)
                        .map(|(_, a)| *a)
                        .unwrap_or_default();
                    let offset = cx.status_states.state(section).offset().min(i);
                    let row = area.y + 1 + (i - offset) as u16;
                    let menu = open_folder_menu(section, &path, area.x + 2, row);
                    *cx.active_status_section = section;
                    *cx.open_menu = OpenMenu::Folder((section, path, menu));
                }
                None => {
                    *cx.last_message = Some(msg::input::SELECT_FOLDER_ROW.get().into());
                }
            }
        }
        PaletteCommand::StageSelectedFile => {
            if let Some(p) = selected_status_path_in(
                &*cx.status_result,
                &*cx.status_states,
                StatusSection::Unstaged,
                *cx.status_list_mode,
                &*cx.collapsed_status_folders,
            ) {
                cx.link.submit_action(Action::StageFile(p));
            }
        }
        PaletteCommand::UnstageSelectedFile => {
            if let Some(p) = selected_status_path_in(
                &*cx.status_result,
                &*cx.status_states,
                StatusSection::Staged,
                *cx.status_list_mode,
                &*cx.collapsed_status_folders,
            ) {
                cx.link.submit_action(Action::UnstageFile(p));
            }
        }
        PaletteCommand::StageAll => cx.link.submit_action(Action::StageAll),
        PaletteCommand::DiscardAllChanges => {
            discard_all_changes(
                cx.status_result.as_ref().ok(),
                &mut *cx.input_mode,
                &mut *cx.last_message,
            );
        }
        // `Ctrl+G c` and `Ctrl+G Enter` are one command: commit when the
        // message field holds a message, else take the keyboard there.
        PaletteCommand::CommitNow | PaletteCommand::Commit => commit_now(cx),
        // Everything the per-frame guard needs
        // for the field to keep the keyboard:
        // the WIP row selected, the Log focused
        // and on screen.
        PaletteCommand::EditWipMessage
            if !has_wip_row(&*cx.status_result)
                && matches!(
                    cx.operation_state,
                    Some(gitpoulpi_core::domain::OperationState::Merge { .. })
                ) =>
        {
            // A merge whose result equals HEAD has no WIP row (nothing is
            // changed), yet can still be concluded: the message field
            // takes the merge commit's message.
            let message = merge_message(cx.operation_state);
            focus_merge_message(cx, message);
        }
        PaletteCommand::EditWipMessage
            if !has_wip_row(&*cx.status_result)
                && matches!(
                    cx.operation_state,
                    Some(
                        gitpoulpi_core::domain::OperationState::CherryPick { .. }
                            | gitpoulpi_core::domain::OperationState::Revert { .. }
                    )
                ) =>
        {
            // git itself refuses to commit an empty pick: say so, and
            // point to Skip (the banner's other button).
            *cx.last_message = Some(gitpoulpi_i18n::msg::merge::MSG_EMPTY_PICK.get().to_string());
        }
        PaletteCommand::EditWipMessage => {
            *cx.focus = Focus::Log;
            *cx.central_view = CentralView::Log;
            *cx.diff_view_state = None;
            cx.log_state.select(Some(0));
            *cx.log_follow = true;
            *cx.composer_focus = Some(ComposerField::WipSummary);
        }
        // The one-off grouped push: the selection step of the push-group
        // dialog for the checked-out branch.
        PaletteCommand::PushToSelectedRemotes => {
            match cx
                .refs_result
                .as_ref()
                .ok()
                .and_then(|refs| crate::remotes_ui::head_branch(refs))
            {
                Some(head) => {
                    crate::push_group_ui::PushGroupUi::open_selection(cx.repo_root, &head.name)
                }
                None => {
                    *cx.last_message = Some(msg::remotes::DETACHED_PUSH.get().into());
                }
            }
        }
        // Every sync and remote command, in
        // one place (`remotes_ui.rs`), so the
        // palette cannot drift from the
        // toolbar, the menus and `Ctrl+G`.
        PaletteCommand::Fetch
        | PaletteCommand::Pull
        | PaletteCommand::PullWithMode(_)
        | PaletteCommand::ChoosePullButtonAction
        | PaletteCommand::LfsMenu
        | PaletteCommand::Push
        | PaletteCommand::PushToRemote
        | PaletteCommand::ForcePushWithLease
        | PaletteCommand::SetUpstream
        | PaletteCommand::AddRemote
        | PaletteCommand::RemoveRemote
        | PaletteCommand::EditRemote
        | PaletteCommand::SetDefaultRemote
        | PaletteCommand::CopyRemoteName
        | PaletteCommand::CopyRemoteLink
        | PaletteCommand::FetchRemote
        | PaletteCommand::RemoteActions
        | PaletteCommand::HideRemoteInGraph
        | PaletteCommand::ShowRemoteInGraph => {
            let selected_remote = selected_sidebar_remote(
                &*cx.refs_result,
                &*cx.refs_filter,
                *cx.active_ref_section,
                cx.sidebar_states.state(*cx.active_ref_section).selected(),
                *cx.hierarchical_refs,
                &*cx.collapsed_ref_folders,
                &*cx.remote_names,
            );
            let selected_branch = sel_ref
                .clone()
                .filter(|_| *cx.active_ref_section == SidebarSection::Local);
            run_remote_palette_command(
                cmd,
                selected_remote,
                selected_branch,
                RemoteUiCtx {
                    ui: &mut *cx.remote_ui,
                    refs: cx.refs_result.as_deref().unwrap_or(&[]),
                    link: cx.link,
                    clipboard: &mut *cx.clipboard,
                    input_mode: &mut *cx.input_mode,
                    last_message: &mut *cx.last_message,
                    hidden_from_graph: &mut *cx.hidden_from_graph,
                    solo_refs: &*cx.solo_refs,
                },
            );
        }
        PaletteCommand::NewBranch => {
            *cx.input_mode = InputMode::InlineName {
                op: RefNameOp::NewBranch,
                value: String::new(),
            };
        }
        PaletteCommand::NewTag => {
            *cx.input_mode = InputMode::InlineName {
                op: RefNameOp::NewTag,
                value: String::new(),
            };
        }
        PaletteCommand::StashPush => {
            cx.link.submit_action(Action::StashPush);
            *cx.last_message = Some(msg::input::STASHING.get().into());
        }
        // All three act on the stash row selected
        // in the Log when the Log has focus, then
        // go through `target_stash`: the stash
        // *selected* in the STASHES pane when it
        // has focus, `stash@{0}` otherwise.
        PaletteCommand::StashPop(StashTarget::Selected) => {
            if let Some(s) = selected_log_stash(&*cx.stashes_result, &*cx.log_state, &*cx.log_order)
                .filter(|_| *cx.focus == Focus::Log)
                .or_else(|| {
                    target_stash(
                        &*cx.stashes_result,
                        *cx.focus,
                        *cx.active_ref_section,
                        cx.sidebar_states.state_mut(SidebarSection::Stashes),
                    )
                })
            {
                cx.link.submit_action(Action::StashPop(s.id));
            }
        }
        PaletteCommand::StashPop(StashTarget::LatestOrLogRow) => {
            // The stash row selected in the Log, if the Log has focus and is
            // on one; the latest stash otherwise.
            let chosen = selected_log_stash(&*cx.stashes_result, &*cx.log_state, &*cx.log_order)
                .filter(|_| *cx.focus == Focus::Log)
                .or_else(|| {
                    cx.stashes_result
                        .as_ref()
                        .ok()
                        .and_then(|s| s.first().cloned())
                });
            match chosen {
                Some(stash) => {
                    cx.link.submit_action(Action::StashPop(stash.id));
                    *cx.last_message = Some(msg::input::popping_stash(stash.id.0));
                }
                None => *cx.last_message = Some(msg::input::NO_STASH_TO_POP.get().into()),
            }
        }
        PaletteCommand::StashPop(StashTarget::Latest) => {
            match cx.stashes_result.as_deref().ok().and_then(|s| s.first()) {
                Some(s) => {
                    cx.link.submit_action(Action::StashPop(s.id));
                    *cx.last_message = Some(msg::input::popping_stash(s.id.0));
                }
                None => {
                    *cx.last_message = Some(msg::input::NO_STASH_TO_POP.get().into());
                }
            }
        }
        PaletteCommand::StashApply => {
            if let Some(s) = selected_log_stash(&*cx.stashes_result, &*cx.log_state, &*cx.log_order)
                .filter(|_| *cx.focus == Focus::Log)
                .or_else(|| {
                    target_stash(
                        &*cx.stashes_result,
                        *cx.focus,
                        *cx.active_ref_section,
                        cx.sidebar_states.state_mut(SidebarSection::Stashes),
                    )
                })
            {
                cx.link.submit_action(Action::StashApply(s.id));
            }
        }
        PaletteCommand::StashDrop => {
            if let Some(s) = selected_log_stash(&*cx.stashes_result, &*cx.log_state, &*cx.log_order)
                .filter(|_| *cx.focus == Focus::Log)
                .or_else(|| {
                    target_stash(
                        &*cx.stashes_result,
                        *cx.focus,
                        *cx.active_ref_section,
                        cx.sidebar_states.state_mut(SidebarSection::Stashes),
                    )
                })
            {
                *cx.input_mode = InputMode::Confirm {
                    prompt: msg::input::drop_stash(s.id.0),
                    action: ConfirmAction::Destructive(PendingDestructive::StashDrop(s.id)),
                };
            }
        }
        // The File View has
        // no hunks; its Copy Line is its
        // selected row.
        PaletteCommand::CopySelectedText | PaletteCommand::CopyLine | PaletteCommand::CopyHunk => {
            let what = match cmd {
                PaletteCommand::CopySelectedText => diff_view::DiffCopy::Selection,
                PaletteCommand::CopyLine => diff_view::DiffCopy::Line,
                _ => diff_view::DiffCopy::Hunk,
            };
            if *cx.central_view == CentralView::File {
                if let Some(insight) = cx.file_insight.as_ref() {
                    let text = match what {
                        diff_view::DiffCopy::Line => insight.copy_line(),
                        _ => insight.yank(),
                    };
                    *cx.last_message = Some(match text {
                        Ok((label, t)) => cx.clipboard.copy(label, &t),
                        Err(why) => why.to_string(),
                    });
                }
            } else {
                *cx.last_message = copy_from_diff(
                    &*cx.diff_view_state,
                    &*cx.diff_render,
                    what,
                    &mut *cx.clipboard,
                );
            }
        }
        // The File View's or the
        // diff's file, else the file the Status
        // pane or the commit points at -- its
        // working-tree copy either way.
        PaletteCommand::EditInWorkingDirectory => {
            let shown = match *cx.central_view {
                CentralView::File => cx.file_insight.as_ref().map(|f| f.request.path.clone()),
                CentralView::Diff => cx.diff_view_state.as_ref().map(|d| d.path.clone()),
                _ => None,
            };
            let path = shown.or_else(|| {
                insight_target_path(
                    None,
                    &*cx.status_result,
                    &*cx.status_states,
                    *cx.active_status_section,
                    *cx.status_list_mode,
                    &*cx.collapsed_status_folders,
                    cx.commit_file_state
                        .selected()
                        .and_then(|i| cx.commit_file_rows.get(i))
                        .and_then(TreeRow::file_index)
                        .and_then(|pi| cx.commit_paths_cache.get(pi))
                        .cloned(),
                )
            });
            match path {
                Some(path) => {
                    if let Err(why) = open_editor(
                        cx.repo_root,
                        &path,
                        head_oid(cx.refs_result.as_deref().unwrap_or(&[])),
                        cx.link,
                        &mut *cx.editor,
                        &mut *cx.file_insight,
                        &*cx.diff_view_state,
                    ) {
                        *cx.last_message = Some(why);
                    }
                }
                None => *cx.last_message = Some(msg::input::SELECT_FILE_FIRST.get().into()),
            }
        }
        // The editor's status menus: the keyboard twin of the chips.
        PaletteCommand::EditorMenu(menu) => {
            if let (CentralView::Editor, Some(ed)) = (*cx.central_view, cx.editor.as_mut()) {
                ed.open_menu(menu);
            } else {
                *cx.last_message = Some(msg::editor::REASON_CLOSED.get().into());
            }
        }
        PaletteCommand::Blame | PaletteCommand::FileView if super::history::is_open(cx) => {
            // Inside the File History these pick the right side's mode.
            let mode = if cmd == PaletteCommand::Blame {
                crate::history_view::ViewMode::Blame
            } else {
                crate::history_view::ViewMode::File
            };
            super::history::set_mode(cx, mode);
        }
        PaletteCommand::Blame | PaletteCommand::History | PaletteCommand::FileView => {
            let kind = match cmd {
                PaletteCommand::Blame => FileInsightKind::Blame,
                PaletteCommand::History => FileInsightKind::History,
                _ => FileInsightKind::Content,
            };
            // Same file *and* same revision as
            // the Diff view's own bar buttons
            // when a diff is open -- two ways
            // to run one command must not
            // answer different questions.
            let open_diff = cx
                .diff_view_state
                .as_ref()
                .filter(|_| *cx.central_view == CentralView::Diff);
            let refs = cx.refs_result.as_deref().unwrap_or(&[]);
            let commit_file_path = cx
                .commit_file_state
                .selected()
                .and_then(|i| cx.commit_file_rows.get(i))
                .and_then(TreeRow::file_index)
                .and_then(|pi| cx.commit_paths_cache.get(pi))
                .cloned();
            // A file selected in the Changed files
            // lists is a WIP file: its section says
            // whether the index or the disk is read.
            let wip_path = if *cx.right_panel == RightPanel::CommitDetail {
                None
            } else {
                selected_status_path_in(
                    &*cx.status_result,
                    &*cx.status_states,
                    *cx.active_status_section,
                    *cx.status_list_mode,
                    &*cx.collapsed_status_folders,
                )
            };
            let request = if let Some(s) = open_diff {
                Ok(diff_insight_request(s, kind, refs))
            } else if let Some(path) = wip_path {
                Ok(wip_insight_request(
                    kind,
                    path,
                    *cx.active_status_section == StatusSection::Staged,
                    head_oid(refs),
                ))
            } else {
                match (commit_file_path, head_oid(refs)) {
                    (Some(path), Some(at)) => Ok(FileInsightRequest {
                        path,
                        at,
                        kind,
                        worktree: false,
                        staged: false,
                    }),
                    (None, _) => Err(msg::input::SELECT_FILE_FIRST.get()),
                    (_, None) => Err(msg::input::HEAD_NO_TIP.get()),
                }
            };
            match request {
                Ok(request) => open_insight_request(request, cx.link, &mut *cx.file_insight),
                Err(why) => *cx.last_message = Some(why.into()),
            }
        }
        // Unreachable: `unavailable_reason`
        // returns Some for these, and the guard
        // above returns before we get here.
        // All three read the Log's selection and
        // run through the same function the
        // right-click menu and the Ctrl+G
        // chords do, so "does this confirm?"
        // has exactly one answer per action.
        PaletteCommand::CherryPickSelectedCommit
        | PaletteCommand::RevertSelectedCommit
        | PaletteCommand::EditSelectedCommitMessage
        | PaletteCommand::AmendSelectedCommit(_)
        | PaletteCommand::SplitSelectedCommit
        | PaletteCommand::InteractiveRebaseFromSelectedCommit
        | PaletteCommand::ResetToSelectedCommit(_)
        | PaletteCommand::CreateBranchAtSelectedCommit
        | PaletteCommand::CreateTagAtSelectedCommit
        | PaletteCommand::CheckoutSelectedCommit
        | PaletteCommand::CreateWorktreeAtSelectedCommit
        | PaletteCommand::CreatePatchAtSelectedCommit
        | PaletteCommand::CreatePatchFromSelectedCommitToHead
        | PaletteCommand::CreateAnnotatedTagAtSelectedCommit
        | PaletteCommand::CopySelectedCommitLink
        | PaletteCommand::CopySelectedAsMermaid
        | PaletteCommand::CopyCommitField(_) => {
            let action = match cmd {
                PaletteCommand::CreateAnnotatedTagAtSelectedCommit => {
                    LogContextAction::CreateAnnotatedTagHere
                }
                PaletteCommand::CopySelectedCommitLink => LogContextAction::CopyCommitLink,
                PaletteCommand::CopySelectedAsMermaid => LogContextAction::CopyAsMermaid,
                PaletteCommand::CreateWorktreeAtSelectedCommit => LogContextAction::CreateWorktree,
                PaletteCommand::CreatePatchAtSelectedCommit => LogContextAction::CreatePatch,
                PaletteCommand::CreatePatchFromSelectedCommitToHead => {
                    LogContextAction::CreatePatchToHead
                }
                PaletteCommand::CherryPickSelectedCommit => LogContextAction::CherryPick,
                PaletteCommand::EditSelectedCommitMessage => LogContextAction::EditMessage,
                PaletteCommand::AmendSelectedCommit(scope) => LogContextAction::Amend(scope),
                PaletteCommand::SplitSelectedCommit => LogContextAction::SplitCommit,
                PaletteCommand::RevertSelectedCommit => LogContextAction::Revert,
                PaletteCommand::ResetToSelectedCommit(mode) => LogContextAction::Reset(mode),
                PaletteCommand::CreateBranchAtSelectedCommit => LogContextAction::CreateBranchHere,
                PaletteCommand::CreateTagAtSelectedCommit => LogContextAction::CreateTagHere,
                PaletteCommand::CheckoutSelectedCommit => LogContextAction::CheckoutCommit,
                PaletteCommand::CopyCommitField(f) => LogContextAction::Copy(f),
                PaletteCommand::InteractiveRebaseFromSelectedCommit => {
                    LogContextAction::InteractiveRebase
                }
                _ => unreachable!("guarded by the match arm"),
            };
            if let Some(commit) =
                selected_log_commit(&*cx.log_result, &*cx.log_state, &*cx.log_order)
            {
                apply_log_context_action(
                    action,
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
                );
            }
        }
        PaletteCommand::RebaseContinue => {
            cx.link.submit_action(Action::RebaseContinue);
            *cx.last_message = Some(msg::input::CONTINUING_REBASE.get().into());
        }
        PaletteCommand::RebaseSkip => {
            cx.link.submit_action(Action::RebaseSkip);
            *cx.last_message = Some(msg::input::SKIPPING.get().into());
        }
        PaletteCommand::AutoStashPop => match cx.pending_auto_stash.as_ref() {
            Some(pending) => {
                cx.link
                    .submit_action(Action::AutoStashPop(pending.stash_id, pending.stash_commit));
                *cx.last_message = Some(msg::input::popping_autostash(pending.stash_id.0));
            }
            None => *cx.last_message = Some(msg::input::NO_AUTOSTASH.get().into()),
        },
        // A conflicted pop already put the changes back: once the conflicts
        // are resolved the stash is only a copy. Unstaged, as a clean pop
        // leaves them; then the drop, which clears the record (`settle_manual`).
        PaletteCommand::AutoStashFinish => match cx.pending_auto_stash.as_ref() {
            Some(pending) if pending.state != PendingAutoStashState::Conflicted => {
                *cx.last_message = Some(msg::autostash::NOT_BACK_YET.get().into());
            }
            Some(_) if !cx.conflicts.is_empty() => {
                *cx.last_message = Some(msg::autostash::finish_conflicts_left(cx.conflicts.len()));
            }
            Some(pending) => {
                cx.link.submit_action(Action::UnstageAll);
                cx.link.submit_action(Action::AutoStashDrop(
                    pending.stash_id,
                    pending.stash_commit,
                ));
                *cx.last_message = Some(msg::autostash::FINISHING.get().into());
            }
            None => *cx.last_message = Some(msg::input::NO_AUTOSTASH.get().into()),
        },
        PaletteCommand::ConflictsShow => {
            let head = cx
                .refs_result
                .as_ref()
                .ok()
                .and_then(|r| crate::refs_sidebar::head_oid(r));
            match cx.conflict_badge.shown(head) {
                Some(prediction) => crate::conflict_popup::open(
                    prediction.base_ref.as_deref().unwrap_or(""),
                    &prediction.files,
                ),
                None if cx.conflict_badge.is_off() => {
                    *cx.last_message = Some(msg::conflict_prevention::OFF_NOTE.get().into());
                }
                None => {
                    *cx.last_message = Some(msg::conflict_prevention::NONE_NOTE.get().into());
                }
            }
        }
        PaletteCommand::AutoStashShow => match cx.pending_auto_stash.as_ref() {
            Some(pending) => {
                let id = pending.stash_id;
                // By commit: the index in the record may be one refresh old.
                let index = cx.stashes_result.as_ref().ok().and_then(|s| {
                    s.iter().position(|st| match pending.stash_commit {
                        Some(commit) => st.commit == Some(commit),
                        None => st.id == id,
                    })
                });
                match index {
                    Some(i) => {
                        *cx.focus = Focus::Refs;
                        *cx.active_ref_section = SidebarSection::Stashes;
                        cx.sidebar_states
                            .state_mut(SidebarSection::Stashes)
                            .select(Some(i));
                        *cx.last_message = Some(msg::input::autostash_selected(id.0));
                    }
                    None => *cx.last_message = Some(msg::input::autostash_gone(id.0)),
                }
            }
            None => *cx.last_message = Some(msg::input::NO_AUTOSTASH.get().into()),
        },
        PaletteCommand::AutoStashStartOver => {
            if cx.pending_auto_stash.is_some() {
                *cx.input_mode = InputMode::Confirm {
                    prompt: msg::input::START_OVER_AUTOSTASH.get().to_string(),
                    action: ConfirmAction::AutoStashStartOver,
                };
            } else {
                *cx.last_message = Some(msg::input::NO_AUTOSTASH.get().into());
            }
        }
        PaletteCommand::AutoStashDrop => {
            if let Some(pending) = cx.pending_auto_stash.as_ref() {
                // After a conflicted pop the changes are in the working tree
                // already: the stash is a copy, and the prompt must not say
                // they would be lost.
                let prompt = if pending.state == PendingAutoStashState::Conflicted {
                    msg::input::DROP_AUTOSTASH_CONFLICTED.get().to_string()
                } else if pending.state == PendingAutoStashState::PopFailed {
                    // Nothing was restored: the stash holds the only copy.
                    msg::input::DROP_AUTOSTASH_POP_FAILED.get().to_string()
                } else {
                    msg::input::drop_autostash(pending.stash_id.0)
                };
                *cx.input_mode = InputMode::Confirm {
                    prompt,
                    action: ConfirmAction::Destructive(PendingDestructive::AutoStashDrop(
                        pending.stash_id,
                        pending.stash_commit,
                    )),
                };
            } else {
                *cx.last_message = Some(msg::input::NO_AUTOSTASH.get().into());
            }
        }
        PaletteCommand::AmContinue => {
            cx.link.submit_action(Action::AmContinue);
            *cx.last_message = Some(msg::stuck_states::AM_CONTINUING.get().into());
        }
        PaletteCommand::AmSkip => {
            cx.link.submit_action(Action::AmSkip);
            *cx.last_message = Some(msg::stuck_states::AM_SKIPPING.get().into());
        }
        PaletteCommand::AmAbort => {
            if matches!(
                cx.operation_state,
                Some(gitpoulpi_core::domain::OperationState::Am { .. })
            ) {
                *cx.input_mode = InputMode::Confirm {
                    prompt: msg::stuck_states::AM_ABORT_CONFIRM.get().to_string(),
                    action: ConfirmAction::Destructive(PendingDestructive::AbortAm),
                };
            } else {
                *cx.last_message = Some(msg::stuck_states::NO_AM.get().into());
            }
        }
        PaletteCommand::IndexLockRemove => match cx.index_lock {
            Some(lock) if lock.stale => {
                *cx.input_mode = InputMode::Confirm {
                    prompt: msg::stuck_states::LOCK_CONFIRM.get().to_string(),
                    action: ConfirmAction::Destructive(PendingDestructive::RemoveIndexLock),
                };
            }
            Some(_) => {
                *cx.last_message = Some(msg::stuck_states::lock_fresh(
                    gitpoulpi_core::domain::STALE_INDEX_LOCK_SECS,
                ));
            }
            None => *cx.last_message = Some(msg::stuck_states::LOCK_NONE.get().into()),
        },
        PaletteCommand::RebaseAbort => {
            cx.link.submit_action(Action::RebaseAbort);
            *cx.last_message = Some(msg::input::ABORTING_REBASE.get().into());
        }
        PaletteCommand::CreatePatchFromWip => {
            let branch = cx.refs_result.as_ref().ok().and_then(|rs| {
                rs.iter().find_map(|r| match (&r.kind, r.is_head) {
                    (RefKind::LocalBranch { name }, true) => Some(name.clone()),
                    _ => None,
                })
            });
            let name = match branch {
                Some(branch) => format!("wip-{branch}"),
                None => "wip".to_string(),
            };
            *cx.patch_seed = Some(repo_dialogs::PatchSeed {
                source: gitpoulpi_core::domain::PatchSource::WorkingTree { paths: vec![] },
                name,
            });
        }
        PaletteCommand::ApplyPatch => {
            cx.repo_dialogs.open_apply_patch(&cx.repo.root);
        }
        PaletteCommand::SparseCheckout => cx.repo_dialogs.open_sparse(),
        // Git LFS (`lfs_ui.rs`). The reasons a
        // command cannot run were checked when
        // the palette listed it (grayed out); the
        // dispatch only resolves its arguments.
        PaletteCommand::LfsInitialize => {
            cx.link.submit_action(Action::LfsInstall);
            *cx.last_message = Some(msg::input::LFS_SETTING_UP.get().into());
        }
        PaletteCommand::LfsTrackPattern => {
            *cx.input_mode =
                InputMode::Prompt(Box::new(crate::prompt_dialog::PromptDialog::lfs_track()));
        }
        // The tracked patterns are offered in a list (a typed one is allowed).
        PaletteCommand::LfsUntrackPattern => {
            let tracked: Vec<String> = cx
                .lfs
                .status
                .as_ref()
                .map(|s| s.patterns.iter().map(|p| p.pattern.clone()).collect())
                .unwrap_or_default();
            *cx.input_mode = InputMode::Prompt(Box::new(
                crate::prompt_dialog::PromptDialog::lfs_untrack(&tracked),
            ));
        }
        PaletteCommand::LfsPrune => {
            cx.link.submit_action(Action::LfsPrune);
            *cx.last_message = Some(msg::input::LFS_PRUNING.get().into());
        }
        // The toolbar dropdown's rows that have no older command: the same
        // `LfsMenuItem::run` the dropdown calls.
        PaletteCommand::LfsFetchAll | PaletteCommand::LfsCheckout | PaletteCommand::LfsPushAll => {
            let item = match cmd {
                PaletteCommand::LfsFetchAll => lfs_ui::LfsMenuItem::FetchAll,
                PaletteCommand::LfsCheckout => lfs_ui::LfsMenuItem::Checkout,
                _ => lfs_ui::LfsMenuItem::PushAll,
            };
            let remote = cx
                .remote_ui
                .default_remote(cx.refs_result.as_deref().unwrap_or(&[]));
            *cx.last_message = Some(item.run(remote, cx.link));
        }
        PaletteCommand::LfsFetch
        | PaletteCommand::LfsPull
        | PaletteCommand::LfsPush
        | PaletteCommand::LfsShowLocks => {
            match cx
                .remote_ui
                .default_remote(cx.refs_result.as_deref().unwrap_or(&[]))
            {
                Some(remote) => {
                    let (action, text) = match cmd {
                        PaletteCommand::LfsFetch => {
                            (Action::LfsFetch(remote), msg::input::LFS_FETCHING)
                        }
                        PaletteCommand::LfsPull => {
                            (Action::LfsPull(remote), msg::input::LFS_PULLING)
                        }
                        PaletteCommand::LfsPush => {
                            (Action::LfsPush(remote), msg::input::LFS_PUSHING)
                        }
                        _ => (Action::LfsLocks(remote), msg::input::LFS_LOCKS),
                    };
                    cx.link.submit_action(action);
                    *cx.last_message = Some(text.get().into());
                }
                None => {
                    *cx.last_message = Some(msg::input::NO_DEFAULT_REMOTE.get().into());
                }
            }
        }
        PaletteCommand::LfsOpenInstallPage => {
            *cx.last_message = Some(open_lfs_install_page());
        }
        PaletteCommand::LfsToggleRawPointer => {
            if let Some(s) = cx.diff_view_state.as_mut() {
                s.lfs_raw = !s.lfs_raw;
                s.reset_position();
                *cx.last_message = Some(
                    if s.lfs_raw {
                        msg::input::SHOWING_RAW_POINTER
                    } else {
                        msg::input::SHOWING_LFS_CARD
                    }
                    .get()
                    .into(),
                );
            }
        }
        PaletteCommand::LfsTrackSelectedFile
        | PaletteCommand::LfsTrackSelectedExtension
        | PaletteCommand::LfsLockSelectedFile
        | PaletteCommand::LfsUnlockSelectedFile
        | PaletteCommand::LfsForceUnlockSelectedFile => {
            let selected = [StatusSection::Unstaged, StatusSection::Staged]
                .into_iter()
                .find_map(|section| {
                    selected_status_path_in(
                        &*cx.status_result,
                        &*cx.status_states,
                        section,
                        *cx.status_list_mode,
                        &*cx.collapsed_status_folders,
                    )
                });
            let action = match cmd {
                PaletteCommand::LfsTrackSelectedFile => FileContextAction::LfsTrackFile,
                PaletteCommand::LfsTrackSelectedExtension => FileContextAction::LfsTrackExtension,
                PaletteCommand::LfsLockSelectedFile => FileContextAction::LfsLock,
                PaletteCommand::LfsUnlockSelectedFile => FileContextAction::LfsUnlock,
                _ => FileContextAction::LfsForceUnlock,
            };
            match selected {
                Some(path) => {
                    apply_lfs_file_action(
                        action,
                        &path,
                        cx.remote_ui
                            .default_remote(cx.refs_result.as_deref().unwrap_or(&[])),
                        cx.link,
                        &mut *cx.input_mode,
                        &mut *cx.last_message,
                    );
                }
                None => {
                    *cx.last_message = Some(msg::input::SELECT_CHANGED_FILE.get().into());
                }
            }
        }
        PaletteCommand::ResolveConflictWith(side) => {
            if let Some(path) = selected_conflicted_path(
                &*cx.status_result,
                &*cx.status_states,
                *cx.status_list_mode,
                &*cx.collapsed_status_folders,
                &*cx.conflicts,
            ) {
                cx.link.submit_action(Action::ResolveConflict {
                    path: path.clone(),
                    side,
                });
                *cx.last_message = Some(match side {
                    ConflictSide::Ours => msg::input::resolving_ours(path.display()),
                    ConflictSide::Theirs => msg::input::resolving_theirs(path.display()),
                    ConflictSide::Base => msg::input::resolving_base(path.display()),
                });
            }
        }
        _ => return false,
    }
    true
}

/// git's own default message for the merge in progress, for the prompt
/// that concludes a merge whose result equals HEAD.
fn merge_message(state: &Option<gitpoulpi_core::domain::OperationState>) -> String {
    use gitpoulpi_core::domain::OperationState as O;
    match state {
        // A commit message, git's own wording.
        Some(O::Merge {
            head_name: Some(head),
            their_label,
        }) => format!("Merge {their_label} into {head}"), // i18n-english
        Some(O::Merge { their_label, .. }) => format!("Merge {their_label}"), // i18n-english
        _ => String::new(),
    }
}
