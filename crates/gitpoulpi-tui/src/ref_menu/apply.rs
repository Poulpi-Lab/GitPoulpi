use super::*;
use gitpoulpi_i18n::msg::menu_ref as msg;

/// Everything a ref context menu's rows need to act on.
///
/// A struct of borrows rather than fourteen parameters.
/// `apply_ref_context_action` is the one place a ref row runs, so the
/// mouse path and the keyboard path cannot implement different subsets.
pub(crate) struct RefActionCtx<'a> {
    pub(crate) refs_result: &'a gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Ref>>,
    pub(crate) log_result: &'a gitpoulpi_core::Result<gitpoulpi_core::domain::CommitGraph>,
    pub(crate) status_result: &'a gitpoulpi_core::Result<WorkingTree>,
    pub(crate) hidden_from_graph: &'a mut std::collections::HashSet<String>,
    pub(crate) solo_refs: &'a mut std::collections::HashSet<String>,
    pub(crate) log_len: &'a mut usize,
    /// The Log's display order (`graph::LogRows`) and what it is built
    /// from, so a solo change recomputes it - and `log_len` - with the
    /// stash rows it adds or removes, never a hand-rolled `+ has_wip`.
    pub(crate) log_order: &'a mut graph::LogRows,
    pub(crate) stashes_result: &'a gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Stash>>,
    pub(crate) graph_show_stashes: bool,
    pub(crate) log_state: &'a mut TableState,
    pub(crate) link: &'a DaemonLink,
    pub(crate) clipboard: &'a mut clipboard::Clipboard,
    pub(crate) input_mode: &'a mut InputMode,
    pub(crate) last_message: &'a mut Option<String>,
    /// Known remote names, for `checkout_ref_now`'s tracking-branch
    /// resolution when `full_name` is a remote branch.
    pub(crate) remotes: &'a [String],
    /// Remotes, the default remote and the remote menus, for the
    /// branch rows' Push / Pull / Fetch / Set upstream and the tag rows.
    pub(crate) remote_ui: &'a mut RemoteUi,
    /// Where the menu was, for a follow-up menu (Set upstream).
    pub(crate) anchor: (u16, u16),
    /// Every worktree, for Create worktree to know whether the branch is
    /// already checked out somewhere (git would refuse it again).
    pub(crate) worktrees: &'a [gitpoulpi_core::domain::Worktree],
    /// Where Create worktree leaves its request; the event loop opens the
    /// dialog from it.
    pub(crate) worktree_seed: &'a mut Option<WorktreeSeed>,
    /// Where Start a pull request leaves its request when it needs a
    /// dialog (a matched integration with a usable token); the event
    /// loop opens it from here the same way it does `worktree_seed`. Left
    /// `None` when the action instead opened a browser URL directly, or
    /// reported why it could not.
    pub(crate) pull_request_seed: &'a mut Option<PullRequestSeed>,
    /// The ref carried onto the menu's target, for `Drop` rows: its name
    /// and kind. `None` for every menu that is not a drop menu.
    pub(crate) drop_source: Option<&'a (String, RefRowKind)>,
    /// The submodules, for Open in new tab to find the working tree; the
    /// request it leaves in `open_in_tab` is carried out by the event loop.
    pub(crate) submodules: &'a [gitpoulpi_core::domain::Submodule],
    pub(crate) repo_root: &'a std::path::Path,
    pub(crate) open_in_tab: &'a mut Option<std::path::PathBuf>,
    /// Where Reveal in graph leaves its request; the window ticks it.
    pub(crate) reveal: &'a mut crate::ref_reveal::RevealState,
}

/// Runs one row of a ref's context menu. `full_name` is the row's
/// identity - a ref's displayed name, or a stash's numeric id rendered
/// as a string for STASHES rows.
pub(crate) fn apply_ref_context_action(
    action: RefContextAction,
    section: SidebarSection,
    full_name: &str,
    ctx: RefActionCtx<'_>,
) {
    let RefActionCtx {
        refs_result,
        log_result,
        status_result,
        hidden_from_graph,
        solo_refs,
        log_len,
        log_order,
        stashes_result,
        graph_show_stashes,
        log_state,
        link,
        clipboard,
        input_mode,
        last_message,
        remotes,
        remote_ui,
        anchor,
        worktrees,
        worktree_seed,
        pull_request_seed,
        drop_source,
        submodules,
        repo_root,
        open_in_tab,
        reveal,
    } = ctx;
    let full_name = full_name.to_string();
    match action {
        // The same `checkout_ref_now` the sidebar's double-click and the
        // ref-reveal popup use: a local branch or tag is checked out
        // directly, a remote branch goes through the tracking-branch
        // resolution (check out the local that already tracks it, offer
        // to create one, or say who has taken the name) since a plain
        // checkout of a remote ref would detach HEAD.
        RefContextAction::Checkout => {
            let msg = checkout_ref_now(&full_name, refs_result, remotes, link, input_mode);
            if !msg.is_empty() {
                *last_message = Some(msg);
            }
        }
        RefContextAction::RevealInGraph => {
            let sources = crate::ref_reveal::RevealSources {
                refs: refs_result,
                stashes: stashes_result,
                hidden: hidden_from_graph,
            };
            if let Some(note) = reveal.request_ref(section, &full_name, &sources) {
                *last_message = Some(note);
            }
        }
        RefContextAction::ToggleHideInGraph => {
            if let Some(note) =
                crate::refs_tree::toggle_hide_ref(hidden_from_graph, section.ref_kind(), &full_name)
            {
                *last_message = Some(note);
            }
        }
        RefContextAction::ToggleSolo => {
            // Accumulates: toggling Solo on a ref that's already soloed
            // un-solos just that one; toggling it on a new ref adds it
            // alongside whatever's already soloed instead of replacing it
            // (multi-solo).
            if !solo_refs.remove(&full_name) {
                solo_refs.insert(full_name.clone());
            }
            // Recomputed immediately, not left for the next daemon poll, so
            // keyboard nav bounds (`log_len`) match what was just drawn.
            *log_order = log_rows_now(
                log_result,
                status_result,
                stashes_result,
                stash_rows_shown(graph_show_stashes, solo_refs),
                solo_refs,
                refs_result,
            );
            *log_len = log_len_now(log_result, solo_refs, refs_result, log_order);
            move_table_selection(log_state, *log_len, 0);
        }
        // `full_name` is the folder's collapse-key path here (see the
        // action's own doc comment) -- every ref under it, in this
        // section, is what the bulk toggle touches. Unfiltered (`""`):
        // the sidebar's search box filters what is *drawn*, not what a
        // bulk hide/solo reaches, or a filtered-out sibling would be
        // silently skipped.
        RefContextAction::ToggleHideFolder | RefContextAction::ToggleSoloFolder => {
            // A remote's own row hides the whole remote, and a folder under a
            // hidden remote only says so (`refs_tree::hide_folder`).
            if action == RefContextAction::ToggleHideFolder {
                let is_remote_root =
                    section == SidebarSection::Remote && remotes.contains(&full_name);
                if let crate::refs_tree::FolderHide::Done(note) = crate::refs_tree::hide_folder(
                    hidden_from_graph,
                    section.ref_kind(),
                    &full_name,
                    is_remote_root,
                ) {
                    *last_message = Some(note);
                    return;
                }
            }
            if let (Ok(refs), Some(kind)) = (refs_result.as_ref(), section.ref_kind()) {
                let names = group_refs(refs, "").names(kind).to_vec();
                let under = refs_under_folder(&names, &full_name);
                if !under.is_empty() {
                    if action == RefContextAction::ToggleSoloFolder {
                        toggle_folder_membership(solo_refs, &under);
                        *log_order = log_rows_now(
                            log_result,
                            status_result,
                            stashes_result,
                            stash_rows_shown(graph_show_stashes, solo_refs),
                            solo_refs,
                            refs_result,
                        );
                        *log_len = log_len_now(log_result, solo_refs, refs_result, log_order);
                        move_table_selection(log_state, *log_len, 0);
                    } else {
                        toggle_folder_membership(hidden_from_graph, &under);
                    }
                }
            }
        }
        RefContextAction::MergeIntoCurrent => {
            match find_local_branch_ref(refs_result, &full_name) {
                Some(from) => {
                    link.submit_action(Action::Merge(from));
                    *last_message = Some(msg::merging(&full_name));
                }
                None => *last_message = Some(NO_SUCH_LOCAL_BRANCH.get().into()),
            }
        }
        RefContextAction::RebaseOntoThis => match find_local_branch_ref(refs_result, &full_name) {
            Some(onto) => {
                link.submit_action(Action::Rebase(onto));
                *last_message = Some(msg::rebasing_onto(&full_name));
            }
            None => *last_message = Some(NO_SUCH_LOCAL_BRANCH.get().into()),
        },
        RefContextAction::RenameBranch => {
            *input_mode = InputMode::ComposingRefName {
                op: RefNameOp::RenameBranch(full_name.clone()),
                value: full_name.clone(),
            };
        }
        RefContextAction::DeleteBranch => {
            let refs = refs_result.as_deref().unwrap_or(&[]);
            match checked_out_refusal(refs, &full_name) {
                Some(why) => *last_message = Some(why),
                None => {
                    *input_mode = InputMode::Confirm {
                        prompt: msg::confirm_delete_branch(&full_name),
                        action: ConfirmAction::Destructive(PendingDestructive::DeleteBranch(
                            full_name.clone(),
                        )),
                    };
                }
            }
        }
        RefContextAction::DeleteRemoteBranch | RefContextAction::DeleteBranchAndRemote => {
            let refs = refs_result.as_deref().unwrap_or(&[]);
            let Some(kind) = section.ref_kind() else {
                return;
            };
            let targets = delete_targets(refs, kind, &full_name);
            let both = action == RefContextAction::DeleteBranchAndRemote;
            match (targets.local, targets.remote) {
                (_, None) => {
                    *last_message = Some(msg::no_remote_branch(&full_name));
                }
                (None, Some(_)) if both => {
                    *last_message = Some(msg::no_local_branch(&full_name));
                }
                (local, Some((remote, branch))) => {
                    let refusal = both
                        .then(|| local.as_deref().and_then(|l| checked_out_refusal(refs, l)))
                        .flatten();
                    match refusal {
                        Some(why) => *last_message = Some(why),
                        None => {
                            let and_local = both.then_some(local).flatten();
                            let prompt = match &and_local {
                                Some(l) => msg::confirm_delete_both(l, &remote, &branch),
                                None => msg::confirm_delete_remote(&branch, &remote),
                            };
                            *input_mode = InputMode::Confirm {
                                prompt,
                                action: ConfirmAction::Destructive(
                                    PendingDestructive::DeleteRemoteBranch {
                                        remote,
                                        name: branch,
                                        and_local,
                                    },
                                ),
                            };
                        }
                    }
                }
            }
        }
        RefContextAction::CopyBranchLink => {
            let refs = refs_result.as_deref().unwrap_or(&[]);
            let Some(kind) = section.ref_kind() else {
                return;
            };
            let targets = delete_targets(refs, kind, &full_name);
            *last_message = Some(match targets.remote {
                Some((remote, branch)) => copy_remote_link(
                    &LinkTarget::Branch(branch),
                    &remote_ui.remotes,
                    Some(&remote),
                    clipboard,
                ),
                None => msg::no_remote_link(&full_name),
            });
        }
        // Asks which side when a remote has the tag (`tag_delete_dialog`),
        // else the plain confirm of a tag that exists only here.
        RefContextAction::DeleteTag => {
            crate::tag_delete_dialog::begin(&full_name, false, input_mode, last_message);
        }
        RefContextAction::FetchTag => {
            let refs = refs_result.as_deref().unwrap_or(&[]);
            let remotes = crate::tag_where_ui::lookup(&full_name)
                .map(|i| i.all_remotes())
                .unwrap_or_default();
            let default = remote_ui.default_remote(refs);
            let remote = match default {
                Some(d) if remotes.contains(&d) => Some(d),
                _ => remotes.into_iter().next(),
            };
            *last_message = Some(match remote {
                Some(remote) => {
                    let said = gitpoulpi_i18n::msg::tag_where::msg_fetching(&full_name, &remote);
                    link.submit_action(Action::TagFetch {
                        remote,
                        name: full_name.clone(),
                    });
                    said
                }
                None => msg::no_remote_has(&full_name),
            });
        }
        // Network, so it goes through `submit_action` like fetch/pull/
        // push and narrates through `last_message` -- the input loop
        // never waits on it.
        RefContextAction::PushTag => {
            let refs = refs_result.as_deref().unwrap_or(&[]);
            *last_message = Some(push_tag_now(&full_name, remote_ui, refs, link));
        }
        // Destructive, and destructive on a *shared* remote, so it
        // confirms through the same modal every other destruction in
        // this app uses.
        RefContextAction::DeleteTagFromRemote => {
            crate::tag_delete_dialog::begin(&full_name, true, input_mode, last_message);
        }
        RefContextAction::SetUpstream => {
            let refs = refs_result.as_deref().unwrap_or(&[]);
            match remotes_ui::local_branch(refs, &full_name) {
                Some(branch) => remote_ui.open_upstream_menu(&branch, refs, anchor.0, anchor.1),
                None => *last_message = Some(NO_SUCH_LOCAL_BRANCH.get().into()),
            }
        }
        RefContextAction::PushBranch => {
            let refs = refs_result.as_deref().unwrap_or(&[]);
            *last_message = Some(match remotes_ui::local_branch(refs, &full_name) {
                Some(branch) => {
                    let default = remote_ui.default_remote(refs);
                    match remote_for_branch(&branch, remotes, default.as_deref()) {
                        Some(remote) => {
                            link.submit_action(push_action(&remote, &branch, false));
                            msg::pushing(&full_name, &remote)
                        }
                        None => no_default_remote(),
                    }
                }
                None => NO_SUCH_LOCAL_BRANCH.get().to_string(),
            });
        }
        RefContextAction::PullBranch => {
            let refs = refs_result.as_deref().unwrap_or(&[]);
            *last_message = Some(remote_ui.run_pull_button(refs, link));
        }
        RefContextAction::FetchBranchRemote => {
            let refs = refs_result.as_deref().unwrap_or(&[]);
            let default = remote_ui.default_remote(refs);
            let remote = remotes_ui::local_branch(refs, &full_name)
                .and_then(|b| remote_for_branch(&b, remotes, default.as_deref()));
            *last_message = Some(match remote {
                Some(remote) => {
                    link.submit_action(Action::Fetch(remote.clone()));
                    msg::fetching(&remote)
                }
                None => no_default_remote(),
            });
        }
        RefContextAction::FastForwardToCurrent | RefContextAction::FastForwardCurrentToThis => {
            let refs = refs_result.as_deref().unwrap_or(&[]);
            let head = refs.iter().find_map(|r| match (&r.kind, r.is_head) {
                (RefKind::LocalBranch { name }, true) => Some((name.clone(), r.target)),
                _ => None,
            });
            let this = section
                .ref_kind()
                .and_then(|kind| ref_by_display_name(refs_result, kind, &full_name))
                .map(|r| r.target);
            *last_message = Some(match (head, this) {
                (None, _) => msg::FF_DETACHED.get().into(),
                (_, None) => crate::git_actions::gone(&full_name),
                (Some((head_name, head_target)), Some(this_target)) => {
                    let (branch, target, target_label) =
                        if action == RefContextAction::FastForwardToCurrent {
                            (full_name.clone(), head_target, head_name)
                        } else {
                            (head_name, this_target, full_name.clone())
                        };
                    let message = msg::fast_forwarding(&branch, &target_label);
                    link.submit_action(Action::FastForward {
                        branch,
                        target,
                        target_label,
                    });
                    message
                }
            });
        }
        // The section checklist runs through `apply_sidebar_sections_action`
        // (it keeps its menu open); it never reaches here.
        RefContextAction::ToggleSidebarSection(_)
        | RefContextAction::ResetSidebarSections
        | RefContextAction::UseSidebarSectionsForAll
        | RefContextAction::Fold(_)
        | RefContextAction::OpenSectionsChecklist => {}
        RefContextAction::Drop(op) => {
            let (Some((source, source_kind)), Some(target_kind)) =
                (drop_source, section.ref_kind())
            else {
                *last_message = Some(msg::NOTHING_DROPPED.get().into());
                return;
            };
            let refs = refs_result.as_deref().unwrap_or(&[]);
            match plan_drop(
                op,
                source,
                *source_kind,
                &full_name,
                target_kind,
                refs,
                log_result.as_ref().ok(),
                remotes,
                sync_settings().0,
            ) {
                Ok(DropPlan::Submit(action, said)) => {
                    link.submit_action(action);
                    *last_message = Some(said);
                }
                Ok(DropPlan::Rebase(form)) => *input_mode = InputMode::InteractiveRebase(form),
                Err(why) => *last_message = Some(why),
            }
        }
        RefContextAction::Reflog => {
            crate::safety_panel::ask_to_open(
                crate::safety_panel::Tab::Reflog,
                Some(full_name.clone()),
            );
        }
        RefContextAction::CreateWorktree => {
            let refs = refs_result.as_deref().unwrap_or(&[]);
            let found = section
                .ref_kind()
                .and_then(|kind| ref_by_display_name(refs_result, kind, &full_name));
            *worktree_seed = match found.map(|r| (&r.kind, r.target)) {
                Some((RefKind::LocalBranch { name }, _)) => Some(WorktreeSeed {
                    start: gitpoulpi_core::domain::WorktreeStart::Branch(name.clone()),
                    label: name.clone(),
                    new_branch: String::new(),
                    branch_busy: worktrees
                        .iter()
                        .any(|w| w.branch.as_deref() == Some(name.as_str()))
                        || refs.iter().any(|r| {
                            r.is_head && r.kind == RefKind::LocalBranch { name: name.clone() }
                        }),
                }),
                // A remote branch: a new local branch of the same name,
                // started there -- unless that name is taken, in which
                // case the field starts empty (detached) for the user to
                // fill.
                Some((RefKind::RemoteBranch { name, .. }, target)) => Some(WorktreeSeed {
                    start: gitpoulpi_core::domain::WorktreeStart::Commit(target),
                    label: full_name.clone(),
                    new_branch: if remotes_ui::local_branch(refs, name).is_some() {
                        String::new()
                    } else {
                        name.clone()
                    },
                    branch_busy: false,
                }),
                Some((_, target)) => Some(WorktreeSeed {
                    start: gitpoulpi_core::domain::WorktreeStart::Commit(target),
                    label: full_name.clone(),
                    new_branch: String::new(),
                    branch_busy: false,
                }),
                None => {
                    *last_message = Some(crate::git_actions::gone(&full_name));
                    None
                }
            };
        }
        RefContextAction::StartPullRequest => {
            let refs = refs_result.as_deref().unwrap_or(&[]);
            *last_message = Some(start_pull_request_now(
                &full_name,
                refs,
                &remote_ui.remotes,
                pull_request_seed,
            ));
        }
        RefContextAction::AnnotateTag => {
            *input_mode = InputMode::Prompt(Box::new(
                crate::prompt_dialog::PromptDialog::annotate_tag(&full_name),
            ));
        }
        RefContextAction::PushAllUnpushedTags => {
            let refs = refs_result.as_deref().unwrap_or(&[]);
            *last_message = Some(push_all_tags_now(remote_ui, refs, link));
        }
        // `full_name` is the stash's numeric id for STASHES rows.
        RefContextAction::StashPop => {
            if let Some(id) = parse_stash_id(&full_name) {
                link.submit_action(Action::StashPop(id));
                *last_message = Some(msg::popping_stash(id.0));
            }
        }
        RefContextAction::StashApply => {
            if let Some(id) = parse_stash_id(&full_name) {
                link.submit_action(Action::StashApply(id));
                *last_message = Some(msg::applying_stash(id.0));
            }
        }
        RefContextAction::StashDrop => {
            if let Some(id) = parse_stash_id(&full_name) {
                *input_mode = InputMode::Confirm {
                    prompt: msg::confirm_drop_stash(
                        id.0,
                        crate::keymap::current().shown(crate::keymap::Action::GlobalUndo),
                    ),
                    action: ConfirmAction::Destructive(PendingDestructive::StashDrop(id)),
                };
            }
        }
        // `full_name` is the submodule's path for SUBMODULES rows.
        RefContextAction::SubmoduleUpdate => {
            link.submit_action(Action::SubmoduleUpdate {
                paths: vec![full_name.clone()],
                init: true,
            });
            *last_message = Some(msg::updating_submodule(&full_name));
        }
        RefContextAction::SubmoduleInit => {
            link.submit_action(Action::SubmoduleInit {
                paths: vec![full_name.clone()],
            });
            *last_message = Some(msg::registering_submodule(&full_name));
        }
        RefContextAction::SubmoduleOpen => {
            match crate::submodules_ui::open_target(submodules, repo_root, &full_name) {
                Ok(path) => *open_in_tab = Some(path),
                Err(why) => *last_message = Some(why),
            }
        }
        RefContextAction::Copy(field) => {
            // `ref_menu_entries` only offers Copy under a ref section, so
            // `ref_kind()` is always `Some` here; a stash row has no Copy
            // submenu to reach this from.
            let Some(kind) = section.ref_kind() else {
                return;
            };
            *last_message = Some(match field {
                RefField::Name => clipboard.copy(field.label(), &full_name),
                RefField::FullRef => {
                    clipboard.copy(field.label(), &qualified_ref_name(kind, &full_name))
                }
                RefField::TargetSha => match ref_by_display_name(refs_result, kind, &full_name) {
                    Some(r) => clipboard.copy(field.label(), &r.target.to_hex()),
                    None => crate::git_actions::gone(&full_name),
                },
            });
        }
    }
}
