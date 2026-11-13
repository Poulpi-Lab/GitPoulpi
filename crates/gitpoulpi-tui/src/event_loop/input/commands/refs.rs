//! Commands on the Refs pane: checkout, ref menus, submodules, solo/hide, filter.

use crate::event_loop::*;
use gitpoulpi_i18n::msg;

use super::CommandCtx;

pub(super) fn run(cx: &mut CommandCtx<'_>, cmd: PaletteCommand, sel_ref: &Option<String>) -> bool {
    let sel_ref = sel_ref.clone();
    match cmd {
        // -- Refs ------------------------------
        PaletteCommand::CheckoutSelectedRef => {
            if let Some(n) = sel_ref {
                let msg = checkout_ref_now(
                    &n,
                    &*cx.refs_result,
                    &*cx.remote_names,
                    cx.link,
                    &mut *cx.input_mode,
                );
                if !msg.is_empty() {
                    *cx.last_message = Some(msg);
                }
            }
        }
        // The ref menu's own rows, run on the
        // Refs pane's selection through the
        // same function the menu runs.
        PaletteCommand::CopyRefField(_)
        | PaletteCommand::DeleteSelectedRemoteBranch
        | PaletteCommand::DeleteSelectedBranchAndRemote
        | PaletteCommand::CopySelectedBranchLink
        | PaletteCommand::FastForwardSelectedToCurrent
        | PaletteCommand::FastForwardCurrentToSelected
        | PaletteCommand::CreateWorktreeFromSelectedRef
        | PaletteCommand::ReflogSelectedBranch
        | PaletteCommand::StartPullRequestFromSelectedBranch => {
            let action = match cmd {
                PaletteCommand::CopyRefField(field) => RefContextAction::Copy(field),
                PaletteCommand::DeleteSelectedRemoteBranch => RefContextAction::DeleteRemoteBranch,
                PaletteCommand::DeleteSelectedBranchAndRemote => {
                    RefContextAction::DeleteBranchAndRemote
                }
                PaletteCommand::CopySelectedBranchLink => RefContextAction::CopyBranchLink,
                PaletteCommand::FastForwardSelectedToCurrent => {
                    RefContextAction::FastForwardToCurrent
                }
                PaletteCommand::FastForwardCurrentToSelected => {
                    RefContextAction::FastForwardCurrentToThis
                }
                PaletteCommand::StartPullRequestFromSelectedBranch => {
                    RefContextAction::StartPullRequest
                }
                PaletteCommand::ReflogSelectedBranch => RefContextAction::Reflog,
                _ => RefContextAction::CreateWorktree,
            };
            match (sel_ref.clone(), *cx.active_ref_section) {
                (Some(n), section) => apply_ref_context_action(
                    action,
                    section,
                    &n,
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
                        anchor: (0, 1),
                        worktrees: cx.worktrees_result.as_deref().unwrap_or(&[]),
                        worktree_seed: &mut *cx.worktree_seed,
                        pull_request_seed: &mut *cx.pull_request_seed,
                        drop_source: cx.ref_drop_source.as_ref(),
                        submodules: &*cx.submodules_result,
                        repo_root: &cx.repo.root,
                        open_in_tab: &mut *cx.submodule_open,
                        reveal: &mut *cx.reveal,
                    },
                ),
                (None, _) => *cx.last_message = Some(msg::input::SELECT_REF_REFS.get().into()),
            }
        }
        PaletteCommand::SubmoduleOpenSelected => {
            let selected = cx
                .sidebar_states
                .state(SidebarSection::Submodules)
                .selected();
            if let Some(message) = open_submodule_in_tab(
                &*cx.submodules_result,
                selected,
                &cx.repo.root,
                &mut tab_ctx!(cx),
            ) {
                *cx.last_message = Some(message);
            }
        }
        PaletteCommand::SubmoduleUpdateSelected | PaletteCommand::SubmoduleInitSelected => {
            let selected = cx
                .sidebar_states
                .state(SidebarSection::Submodules)
                .selected();
            match selected_path(&*cx.submodules_result, selected) {
                Some(path) => {
                    let update = cmd == PaletteCommand::SubmoduleUpdateSelected;
                    cx.link.submit_action(if update {
                        Action::SubmoduleUpdate {
                            paths: vec![path.clone()],
                            init: true,
                        }
                    } else {
                        Action::SubmoduleInit {
                            paths: vec![path.clone()],
                        }
                    });
                    *cx.last_message = Some(if update {
                        msg::input::submodule_updating(&path)
                    } else {
                        msg::input::submodule_registering(&path)
                    });
                }
                None => {
                    *cx.last_message = Some(msg::input::SELECT_SUBMODULE_REFS.get().into());
                }
            }
        }
        PaletteCommand::SubmoduleUpdateAll => {
            cx.link.submit_action(Action::SubmoduleUpdate {
                paths: Vec::new(),
                init: true,
            });
            *cx.last_message = Some(msg::input::UPDATING_SUBMODULES.get().into());
        }
        PaletteCommand::OpenWorktreeInTab => {
            let selected = cx
                .sidebar_states
                .state(SidebarSection::Worktrees)
                .selected();
            let list = cx.worktrees_result.as_deref().unwrap_or(&[]).to_vec();
            if let Some(message) = open_worktree_in_tab(&list, selected, &mut tab_ctx!(cx)) {
                *cx.last_message = Some(message);
            }
        }
        PaletteCommand::DropSelectedRefOnCurrent => {
            let refs = cx.refs_result.as_deref().unwrap_or(&[]);
            match (
                sel_ref.clone(),
                cx.active_ref_section.ref_kind(),
                head_branch(refs),
            ) {
                (Some(n), Some(kind), Some(head)) => {
                    let entries = drop_menu_entries(
                        &n,
                        kind,
                        &head.name,
                        RefRowKind::Local,
                        refs,
                        cx.log_result.as_ref().ok(),
                        cx.commit_settings.merge_ff,
                    );
                    if entries.is_empty() {
                        *cx.last_message = Some(msg::input::nothing_to_do(&n, &head.name));
                    } else {
                        let title = drop_menu_title(&n, &head.name);
                        *cx.ref_drop_source = Some((n, kind));
                        *cx.open_menu = OpenMenu::Ref((
                            SidebarSection::Local,
                            head.name,
                            ContextMenu::new(title, entries, 0, 1),
                        ));
                    }
                }
                (None, _, _) => *cx.last_message = Some(msg::input::SELECT_REF_REFS.get().into()),
                (_, _, None) => {
                    *cx.last_message = Some(msg::input::HEAD_DETACHED_CHECKOUT.get().into())
                }
                _ => *cx.last_message = Some(msg::input::SELECT_BRANCH_REFS.get().into()),
            }
        }
        PaletteCommand::MarkRefAsDropSource => match (sel_ref, cx.active_ref_section.ref_kind()) {
            (Some(n), Some(kind)) => {
                let same = cx
                    .ref_drop_mark
                    .as_ref()
                    .is_some_and(|(m, k)| *m == n && *k == kind);
                if same {
                    *cx.ref_drop_mark = None;
                    *cx.last_message = Some(msg::input::ref_drop_mark_cleared(&n));
                } else {
                    let key = crate::keymap::current().shown(crate::keymap::Action::RefsDropMarked);
                    *cx.last_message = Some(msg::input::ref_marked_for_drop(&n, &key));
                    *cx.ref_drop_mark = Some((n, kind));
                }
            }
            _ => *cx.last_message = Some(msg::input::SELECT_REF_REFS.get().into()),
        },
        // Lets go of the marked ref on the selected one: the very menu a
        // drag opens (`drop_menu_entries`), anchored at the top left of the
        // pane. The mark is spent once the menu is up; a pair that offers
        // nothing keeps it so another target can be tried.
        PaletteCommand::DropMarkedRefHere => {
            let marked = cx.ref_drop_mark.clone();
            match (marked, sel_ref, cx.active_ref_section.ref_kind()) {
                (None, _, _) => {
                    *cx.last_message = Some(msg::input::NO_REF_MARKED.get().into());
                }
                (Some(_), None, _) | (Some(_), _, None) => {
                    *cx.last_message = Some(msg::input::SELECT_REF_REFS.get().into());
                }
                (Some((source, source_kind)), Some(target), Some(kind)) => {
                    if !crate::refs_sidebar::ref_drop_is_valid(&source, source_kind, &target, kind)
                    {
                        *cx.last_message = Some(if source == target && source_kind == kind {
                            msg::input::nothing_to_do(&source, &target)
                        } else {
                            msg::input::drop_marked_refused(&target, &source)
                        });
                        return true;
                    }
                    let entries = drop_menu_entries(
                        &source,
                        source_kind,
                        &target,
                        kind,
                        cx.refs_result.as_deref().unwrap_or(&[]),
                        cx.log_result.as_ref().ok(),
                        cx.commit_settings.merge_ff,
                    );
                    if entries.is_empty() {
                        *cx.last_message = Some(msg::input::nothing_to_do(&source, &target));
                    } else {
                        let title = drop_menu_title(&source, &target);
                        *cx.ref_drop_source = Some((source, source_kind));
                        *cx.ref_drop_mark = None;
                        *cx.open_menu = OpenMenu::Ref((
                            *cx.active_ref_section,
                            target,
                            ContextMenu::new(title, entries, 0, 1),
                        ));
                    }
                }
            }
        }
        PaletteCommand::MergeSelectedRef => {
            if let Some(n) = sel_ref {
                match find_local_branch_ref(&*cx.refs_result, &n) {
                    Some(from) => {
                        cx.link.submit_action(Action::Merge(from));
                        *cx.last_message = Some(msg::input::merging(&n));
                    }
                    None => *cx.last_message = Some(NO_SUCH_LOCAL_BRANCH.get().into()),
                }
            }
        }
        PaletteCommand::RebaseOntoSelectedRef => {
            if let Some(n) = sel_ref {
                match find_local_branch_ref(&*cx.refs_result, &n) {
                    Some(onto) => {
                        cx.link.submit_action(Action::Rebase(onto));
                        *cx.last_message = Some(msg::input::rebasing_onto(&n));
                    }
                    None => *cx.last_message = Some(NO_SUCH_LOCAL_BRANCH.get().into()),
                }
            }
        }
        PaletteCommand::RenameSelectedBranch => {
            if let Some(n) = sel_ref {
                *cx.input_mode = InputMode::ComposingRefName {
                    op: RefNameOp::RenameBranch {
                        name: n.clone(),
                        spot: RenameSpot::Auto,
                    },
                    value: n,
                };
            }
        }
        PaletteCommand::DeleteSelectedRef => {
            if let Some(n) = sel_ref {
                if cx.active_ref_section.ref_kind() == Some(RefRowKind::Tag) {
                    // Asks which side when a remote has the tag.
                    crate::tag_delete_dialog::begin(
                        &n,
                        false,
                        &mut *cx.input_mode,
                        &mut *cx.last_message,
                    );
                } else if let Some(why) =
                    checked_out_refusal(cx.refs_result.as_deref().unwrap_or(&[]), &n)
                {
                    *cx.last_message = Some(why);
                } else {
                    *cx.input_mode = InputMode::Confirm {
                        prompt: msg::input::delete_branch(&n),
                        action: ConfirmAction::Destructive(PendingDestructive::DeleteBranch(n)),
                    };
                }
            }
        }
        PaletteCommand::PushSelectedTag => {
            if let Some(n) = sel_ref {
                *cx.last_message = Some(push_tag_now(
                    &n,
                    &*cx.remote_ui,
                    cx.refs_result.as_deref().unwrap_or(&[]),
                    cx.link,
                ));
            }
        }
        PaletteCommand::DeleteSelectedTagFromRemote => {
            if let Some(n) = sel_ref {
                crate::tag_delete_dialog::begin(
                    &n,
                    true,
                    &mut *cx.input_mode,
                    &mut *cx.last_message,
                );
            }
        }
        PaletteCommand::FetchSelectedTag => {
            if let Some(n) = sel_ref {
                let remotes = crate::tag_where_ui::lookup(&n)
                    .map(|i| i.all_remotes())
                    .unwrap_or_default();
                let refs = cx.refs_result.as_deref().unwrap_or(&[]);
                let default = cx.remote_ui.default_remote(refs);
                let remote = match default {
                    Some(d) if remotes.contains(&d) => Some(d),
                    _ => remotes.into_iter().next(),
                };
                *cx.last_message = Some(match remote {
                    Some(remote) => {
                        let said = msg::tag_where::msg_fetching(&n, &remote);
                        cx.link.submit_action(Action::TagFetch { remote, name: n });
                        said
                    }
                    None => msg::input::no_remote_has(&n),
                });
            }
        }
        PaletteCommand::AnnotateSelectedTag => {
            if let Some(n) = sel_ref {
                *cx.input_mode = InputMode::ComposingRefName {
                    op: RefNameOp::AnnotateTag(n),
                    value: String::new(),
                };
            }
        }
        // The one repository-wide tag
        // command, and the reason
        // `TagSelection::AllUnpushed`
        // exists: the set is resolved in
        // the backend at push time, so a
        // tag created since this frame's
        // refresh is included rather than
        // silently left behind.
        PaletteCommand::PushAllUnpushedTags => {
            *cx.last_message = Some(push_all_tags_now(
                &*cx.remote_ui,
                cx.refs_result.as_deref().unwrap_or(&[]),
                cx.link,
            ));
        }
        // The keyboard and palette twin of the ref menu's Reveal in graph,
        // on the Refs pane's selection (a ref, or a stash row).
        PaletteCommand::RevealSelectedRefInGraph => {
            let section = *cx.active_ref_section;
            let name = if section == SidebarSection::Stashes {
                let selected = cx.sidebar_states.state(section).selected();
                cx.stashes_result
                    .as_deref()
                    .ok()
                    .and_then(|list| selected.and_then(|i| list.get(i)))
                    .map(|s| s.id.0.to_string())
            } else {
                sel_ref
            };
            match name {
                Some(name) => {
                    let sources = crate::ref_reveal::RevealSources {
                        refs: &*cx.refs_result,
                        stashes: &*cx.stashes_result,
                        hidden: &*cx.hidden_from_graph,
                    };
                    if let Some(note) = cx.reveal.request_ref(section, &name, &sources) {
                        *cx.last_message = Some(note);
                    }
                }
                None => *cx.last_message = Some(msg::input::SELECT_REF_REFS.get().into()),
            }
        }
        PaletteCommand::ToggleHideInGraph => {
            if let Some(n) = sel_ref {
                if let Some(note) = refs_tree::toggle_hide_ref(
                    &mut *cx.hidden_from_graph,
                    cx.active_ref_section.ref_kind(),
                    &n,
                ) {
                    *cx.last_message = Some(note);
                }
                refs_tree::submit_ref_visibility(cx.link, &*cx.hidden_from_graph, &*cx.solo_refs);
            }
        }
        PaletteCommand::ToggleSolo => {
            if let Some(n) = sel_ref {
                toggle_in(&mut *cx.solo_refs, n);
                refs_tree::submit_ref_visibility(cx.link, &*cx.hidden_from_graph, &*cx.solo_refs);
                *cx.log_order = log_rows_now(
                    &*cx.log_result,
                    &*cx.status_result,
                    &*cx.stashes_result,
                    stash_rows_shown(*cx.graph_show_stashes, &*cx.solo_refs),
                    &*cx.solo_refs,
                    &*cx.refs_result,
                );
                *cx.log_len = log_len_now(
                    &*cx.log_result,
                    &*cx.solo_refs,
                    &*cx.refs_result,
                    &*cx.log_order,
                );
                move_table_selection(&mut *cx.log_state, *cx.log_len, 0);
                *cx.log_follow = true;
            }
        }
        PaletteCommand::StopSoloing => {
            cx.solo_refs.clear();
            refs_tree::submit_ref_visibility(cx.link, &*cx.hidden_from_graph, &*cx.solo_refs);
            *cx.log_order = log_rows_now(
                &*cx.log_result,
                &*cx.status_result,
                &*cx.stashes_result,
                stash_rows_shown(*cx.graph_show_stashes, &*cx.solo_refs),
                &*cx.solo_refs,
                &*cx.refs_result,
            );
            *cx.log_len = log_len_now(
                &*cx.log_result,
                &*cx.solo_refs,
                &*cx.refs_result,
                &*cx.log_order,
            );
            move_table_selection(&mut *cx.log_state, *cx.log_len, 0);
            *cx.log_follow = true;
        }
        PaletteCommand::ShowAllHiddenRefs => {
            cx.hidden_from_graph.clear();
            refs_tree::submit_ref_visibility(cx.link, &*cx.hidden_from_graph, &*cx.solo_refs);
            *cx.last_message = Some(msg::input::HIDDEN_REFS_SHOWN.get().to_string());
        }
        PaletteCommand::ToggleHideSelectedFolder | PaletteCommand::ToggleSoloSelectedFolder => {
            let selected_folder = selected_ref_folder(
                &*cx.refs_result,
                &*cx.refs_filter,
                *cx.active_ref_section,
                cx.sidebar_states.state(*cx.active_ref_section).selected(),
                *cx.hierarchical_refs,
                &*cx.collapsed_ref_folders,
                &*cx.remote_names,
            );
            match (selected_folder, cx.active_ref_section.ref_kind()) {
                (Some((path, is_remote_root)), Some(kind)) => {
                    // The keyboard twin of the remote row's eye.
                    if cmd == PaletteCommand::ToggleHideSelectedFolder {
                        if let refs_tree::FolderHide::Done(note) = refs_tree::hide_folder(
                            &mut *cx.hidden_from_graph,
                            Some(kind),
                            &path,
                            is_remote_root,
                        ) {
                            *cx.last_message = Some(note);
                            refs_tree::submit_ref_visibility(
                                cx.link,
                                &*cx.hidden_from_graph,
                                &*cx.solo_refs,
                            );
                            return true;
                        }
                    }
                    let names = cx
                        .refs_result
                        .as_ref()
                        .map(|refs| group_refs(refs, "").names(kind).to_vec())
                        .unwrap_or_default();
                    let under = refs_under_folder(&names, &path);
                    if cmd == PaletteCommand::ToggleSoloSelectedFolder {
                        toggle_folder_membership(&mut *cx.solo_refs, &under);
                        *cx.log_order = log_rows_now(
                            &*cx.log_result,
                            &*cx.status_result,
                            &*cx.stashes_result,
                            stash_rows_shown(*cx.graph_show_stashes, &*cx.solo_refs),
                            &*cx.solo_refs,
                            &*cx.refs_result,
                        );
                        *cx.log_len = log_len_now(
                            &*cx.log_result,
                            &*cx.solo_refs,
                            &*cx.refs_result,
                            &*cx.log_order,
                        );
                        move_table_selection(&mut *cx.log_state, *cx.log_len, 0);
                    } else {
                        toggle_folder_membership(&mut *cx.hidden_from_graph, &under);
                    }
                    refs_tree::submit_ref_visibility(
                        cx.link,
                        &*cx.hidden_from_graph,
                        &*cx.solo_refs,
                    );
                }
                _ => *cx.last_message = Some(msg::input::SELECT_FOLDER_OR_REMOTE.get().into()),
            }
        }
        PaletteCommand::FilterRefs => {
            *cx.focus = Focus::Refs;
            *cx.input_mode = InputMode::FilteringRefs(cx.refs_filter.clone());
        }
        PaletteCommand::ClearRefsFilter => cx.refs_filter.clear(),
        PaletteCommand::NextRefSection => {
            step_sidebar_section(&mut *cx.active_ref_section, cx.visible_sidebar, 1)
        }
        PaletteCommand::PrevRefSection => {
            step_sidebar_section(&mut *cx.active_ref_section, cx.visible_sidebar, -1)
        }
        PaletteCommand::ToggleSectionCollapse => {
            let k = cx.active_ref_section.title().to_string();
            toggle_in(&mut *cx.collapsed_ref_sections, k);
        }
        PaletteCommand::ToggleHierarchy => {
            *cx.last_message = Some(toggle_ref_tree(&mut *cx.hierarchical_refs));
        }

        _ => return false,
    }
    true
}
