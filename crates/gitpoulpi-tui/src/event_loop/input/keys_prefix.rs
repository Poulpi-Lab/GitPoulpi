//! The two prefix modes: the leader chord (`Ctrl+G`-style sub-commands run
//! through the command layer) and the navigation prefix.

use super::commands::{CommandCtx, Flow};
use super::KeyEnv;
use crate::event_loop::*;
use gitpoulpi_i18n::msg;

/// The leader chord is armed: the next key picks the command.
pub(super) fn leader(cx: &mut CommandCtx<'_>, env: &KeyEnv<'_>, key: KeyEvent) -> io::Result<Flow> {
    let km = env.km;
    *cx.input_mode = InputMode::Normal;
    match km.action_in(keymap::Ctx::GitPrefix, &key) {
        // `Ctrl+G c`: the same command as `Ctrl+G Enter` and the palette's
        // two commit entries.
        Some(Bind::GitCompose) => {
            commands::run_command(cx, PaletteCommand::Commit);
        }
        // `Ctrl+G C`: AI proposes a split of the working
        // changes into commits (an interactive review).
        Some(Bind::GitAiCompose) if cx.ai_ui.enabled() => {
            if has_wip_row(&*cx.status_result) {
                commands::start_ai(cx, AiRequest::Compose);
            } else {
                *cx.last_message = Some(msg::input::NO_CHANGES_TO_SPLIT.get().into());
            }
        }
        // `Ctrl+G w`: AI explains what changed -- the selected
        // commit, else the working changes.
        Some(Bind::GitAiExplain) if cx.ai_ui.enabled() => {
            if let Some(commit) =
                selected_log_commit(&*cx.log_result, &*cx.log_state, &*cx.log_order)
            {
                commands::explain_commit(cx, commit);
            } else if has_wip_row(&*cx.status_result) {
                commands::explain_wip(cx);
            } else {
                *cx.last_message = Some(msg::input::NOTHING_TO_EXPLAIN.get().into());
            }
        }
        // `Ctrl+G Enter`: commit what is staged with the composer's message,
        // from anywhere (the palette runs the same command).
        Some(Bind::GitCommitNow) => {
            commands::run_command(cx, PaletteCommand::CommitNow);
        }
        // `Ctrl+G Shift+H`: the history rewrite workbench (advanced, so no
        // toolbar button; the palette and the Log menu open it too).
        Some(Bind::GitRewriteWorkbench) if crate::build_features::HISTORY_REWRITE => {
            open_rewrite(cx, crate::rewrite_view::RewriteEntry::Workbench);
        }
        // `Ctrl+G m`: stash with a message (prompt).
        Some(Bind::GitStashMessage) => {
            *cx.input_mode = InputMode::ComposingRefName {
                op: RefNameOp::StashMessage,
                value: String::new(),
            };
        }
        // `Ctrl+G i`: AI writes the commit message.
        // Silent when AI is off, like its button.
        Some(Bind::GitAiCommitMessage) if cx.ai_ui.enabled() => {
            commands::generate_commit_message(cx);
        }
        Some(Bind::GitStageAll) => {
            cx.link.submit_action(Action::StageAll);
            *cx.last_message = Some(msg::input::STAGING_ALL.get().into());
        }
        // The leader-key twin of the
        // toolbar's trash button and the palette's
        // `Working Changes: Discard All Changes...`.
        Some(Bind::GitDiscardAll) => {
            discard_all_changes(
                cx.status_result.as_ref().ok(),
                &mut *cx.input_mode,
                &mut *cx.last_message,
            );
        }
        Some(Bind::GitStageFile) => {
            if let Some(path) = selected_status_path(
                section_lines(&*cx.status_section_lines, StatusSection::Unstaged),
                cx.status_states.state(StatusSection::Unstaged).selected(),
                false,
            ) {
                cx.link.submit_action(Action::StageFile(path));
            } else {
                *cx.last_message = Some(msg::input::SELECT_UNSTAGED_FILE.get().into());
            }
        }
        Some(Bind::GitUnstageFile) => {
            if let Some(path) = selected_status_path(
                section_lines(&*cx.status_section_lines, StatusSection::Staged),
                cx.status_states.state(StatusSection::Staged).selected(),
                true,
            ) {
                cx.link.submit_action(Action::UnstageFile(path));
            } else {
                *cx.last_message = Some(msg::input::SELECT_STAGED_FILE.get().into());
            }
        }
        // The same commands the palette runs (`commands.rs`),
        // so the chord and the palette cannot drift.
        Some(
            bind @ (Bind::GitPull
            | Bind::GitPush
            | Bind::GitOpenLinkedPr
            | Bind::GitPushTags
            | Bind::GitFetchAll
            | Bind::GitNewTag
            | Bind::GitNewBranch
            | Bind::GitStashPush
            | Bind::GitLfsMenu
            | Bind::GitFlowMenu
            | Bind::GitLfsFetchAll
            | Bind::GitLfsCheckout
            | Bind::GitLfsPull
            | Bind::GitLfsPushAll
            | Bind::GitLfsPrune),
        ) => {
            let cmd = match bind {
                Bind::GitPull => PaletteCommand::Pull,
                Bind::GitPush => PaletteCommand::Push,
                // The linked pull/merge request of the
                // checked-out branch, in the browser: the
                // toolbar chip's click.
                Bind::GitOpenLinkedPr => PaletteCommand::Hosting(HostingCommand::OpenLinkedPr),
                Bind::GitPushTags => PaletteCommand::PushAllUnpushedTags,
                Bind::GitFetchAll => PaletteCommand::Fetch,
                Bind::GitNewTag => PaletteCommand::NewTag,
                Bind::GitNewBranch => PaletteCommand::NewBranch,
                Bind::GitLfsMenu => PaletteCommand::LfsMenu,
                Bind::GitFlowMenu => PaletteCommand::Flow(crate::flow::FlowCmd::Menu),
                Bind::GitLfsFetchAll => PaletteCommand::LfsFetchAll,
                Bind::GitLfsCheckout => PaletteCommand::LfsCheckout,
                Bind::GitLfsPull => PaletteCommand::LfsPull,
                Bind::GitLfsPushAll => PaletteCommand::LfsPushAll,
                Bind::GitLfsPrune => PaletteCommand::LfsPrune,
                _ => PaletteCommand::StashPush,
            };
            commands::run_command(cx, cmd);
        }
        // The three commit-targeted operations, all
        // reading the Log's selection, all going
        // through `apply_log_context_action` so the
        // chord and the right-click menu confirm the
        // same things.
        Some(
            git_op @ (Bind::GitCherryPick
            | Bind::GitRevert
            | Bind::GitEditMessage
            | Bind::GitAmend
            | Bind::GitEditAuthor
            | Bind::GitResetMixed),
        ) => {
            let cmd = match git_op {
                Bind::GitCherryPick => PaletteCommand::CherryPickSelectedCommit,
                Bind::GitRevert => PaletteCommand::RevertSelectedCommit,
                Bind::GitEditMessage => PaletteCommand::EditSelectedCommitMessage,
                Bind::GitAmend => PaletteCommand::AmendSelectedCommit(AmendScope::All),
                Bind::GitEditAuthor => PaletteCommand::AmendSelectedCommit(AmendScope::Author),
                // Mixed is git's own default, and the
                // only one of the three that is safe
                // to put on a bare chord: soft and
                // hard stay in the menu and the
                // palette, where their consequences
                // are spelled out in the label.
                _ => {
                    PaletteCommand::ResetToSelectedCommit(gitpoulpi_core::domain::ResetMode::Mixed)
                }
            };
            // The palette grays these out without a
            // commit; the chord has no popup to gray, so
            // it says why instead.
            if selected_log_commit(&*cx.log_result, &*cx.log_state, &*cx.log_order).is_some() {
                commands::run_command(cx, cmd);
            } else {
                *cx.last_message = Some(msg::input::SELECT_LOG_COMMIT_NOT_WIP.get().into());
            }
        }
        // Rebase reads the *Refs* sidebar's selection,
        // not the Log's: its argument is the branch to
        // replay onto, which is the same thing the
        // sidebar's "Rebase current onto this" row
        // names.
        Some(Bind::GitRebaseOnto) => {
            let selected = selected_ref_leaf(
                &*cx.refs_result,
                &*cx.refs_filter,
                *cx.active_ref_section,
                cx.sidebar_states
                    .state_mut(*cx.active_ref_section)
                    .selected(),
                *cx.hierarchical_refs,
                &*cx.collapsed_ref_folders,
                &*cx.remote_names,
            );
            match selected
                .filter(|_| cx.active_ref_section.ref_kind() == Some(RefRowKind::Local))
                .and_then(|n| find_local_branch_ref(&*cx.refs_result, &n).map(|r| (n, r)))
            {
                Some((name, onto)) => {
                    cx.link.submit_action(Action::Rebase(onto));
                    *cx.last_message = Some(msg::input::rebasing_onto(&name));
                }
                None => {
                    *cx.last_message = Some(msg::input::SELECT_LOCAL_BRANCH_REBASE.get().into())
                }
            }
        }
        Some(Bind::GitStashPop) => {
            commands::run_command(cx, PaletteCommand::StashPop(StashTarget::LatestOrLogRow));
        }
        // Keyboard path for deleting whichever ref is
        // currently selected in the Refs sidebar -- the
        // mouse path (right-click -> Delete) already
        // opens the same real confirm modal; this just
        // reaches it without a mouse. Kept behind the
        // `Ctrl+G` leader like every other git-mutating
        // action in this app, not a bare key.
        Some(Bind::GitDeleteRef) => {
            // The STASHES pane reuses this same
            // leader key for "drop the selected
            // stash", through the same confirm modal.
            if *cx.active_ref_section == SidebarSection::Stashes {
                let entries = cx.stashes_result.as_deref().unwrap_or(&[]).to_vec();
                let selected = cx
                    .sidebar_states
                    .state_mut(*cx.active_ref_section)
                    .selected()
                    .and_then(|i| entries.get(i));
                match selected {
                    Some(st) => {
                        *cx.input_mode = InputMode::Confirm {
                            prompt: msg::input::drop_stash_final(st.id.0),
                            action: ConfirmAction::Destructive(PendingDestructive::StashDrop(
                                st.id,
                            )),
                        };
                    }
                    None => {
                        *cx.last_message = Some(msg::input::NO_STASH_SELECTED.get().into());
                    }
                }
            } else if let (Ok(refs), Some(kind)) =
                (cx.refs_result.as_ref(), cx.active_ref_section.ref_kind())
            {
                let groups = group_refs(refs, &*cx.refs_filter);
                let state = cx.sidebar_states.state_mut(*cx.active_ref_section);
                let rows = ref_display_rows(
                    groups.names(kind),
                    kind,
                    *cx.hierarchical_refs,
                    &*cx.collapsed_ref_folders,
                    &*cx.remote_names,
                );
                let selected_leaf =
                    state
                        .selected()
                        .and_then(|i| rows.get(i))
                        .and_then(|row| match row {
                            RefTreeRow::Leaf { full_name, .. } => Some(full_name.clone()),
                            RefTreeRow::Folder { .. } => None,
                        });
                match (kind, selected_leaf) {
                    (RefRowKind::Local, Some(full_name)) => {
                        match checked_out_refusal(refs, &full_name) {
                            Some(why) => *cx.last_message = Some(why),
                            None => {
                                *cx.input_mode = InputMode::Confirm {
                                    prompt: msg::input::delete_branch(&full_name),
                                    action: ConfirmAction::Destructive(
                                        PendingDestructive::DeleteBranch(full_name),
                                    ),
                                };
                            }
                        }
                    }
                    (RefRowKind::Tag, Some(full_name)) => {
                        *cx.input_mode = InputMode::Confirm {
                            prompt: msg::input::delete_tag(&full_name),
                            action: ConfirmAction::Destructive(PendingDestructive::DeleteTag(
                                full_name,
                            )),
                        };
                    }
                    (RefRowKind::Remote, Some(full_name)) => {
                        // Deletes the branch on its remote,
                        // after a confirm (see the ref menu).
                        match delete_targets(refs, kind, &full_name).remote {
                            Some((remote, name)) => {
                                *cx.input_mode = InputMode::Confirm {
                                    prompt: msg::input::delete_remote_branch(&name, &remote),
                                    action: ConfirmAction::Destructive(
                                        PendingDestructive::DeleteRemoteBranch {
                                            remote,
                                            name,
                                            and_local: None,
                                        },
                                    ),
                                };
                            }
                            None => {
                                *cx.last_message =
                                    Some(msg::input::no_remote_branch_for(&full_name))
                            }
                        }
                    }
                    _ => {
                        *cx.last_message = Some(msg::input::NO_REF_TO_DELETE.get().into());
                    }
                }
            }
        }
        None if key.code == KeyCode::Esc => {}
        _ => *cx.last_message = Some(msg::input::unknown_action(km.shown(Bind::GlobalGitPrefix))),
    }
    Ok(Flow::Handled)
}

// The `Ctrl+N` navigation/display prefix's second key.
// `navigation_key_action` (`chrome.rs`) does the pure
// lookup -- unit-tested there without a terminal --
// this just applies whichever action it names.
// `FocusByBadge` is resolved against
// `nav_badge_targets`, the exact list the last render
// pass drew badges from, so the glyph on screen and
// what pressing it does can never disagree.
/// The navigation prefix is armed: the next key moves focus or resizes.
pub(super) fn navigation(cx: &mut CommandCtx<'_>, key: KeyEvent) -> io::Result<Flow> {
    *cx.input_mode = InputMode::Normal;
    let action = crate::chrome::navigation_key_action_mods(key.code, key.modifiers);
    // Settings covers the window: only the app-wide chords work over it
    // (the same set as `PaletteCommand::stays_in_settings`).
    if cx.preferences.is_some()
        && !matches!(
            action,
            NavigationAction::ChangeTheme
                | NavigationAction::ChangeLanguage
                | NavigationAction::ChangeGraphColorTheme
                | NavigationAction::ToggleGraphRowTints
                | NavigationAction::OpenAbout
                | NavigationAction::ToggleHelp
                | NavigationAction::FocusByBadge(_)
                | NavigationAction::Cancel
        )
    {
        commands::refuse_over_settings(cx);
        return Ok(Flow::Handled);
    }
    // A dialog or the Activity Log is up: only a badge (or Esc) means anything,
    // the other chords would act on the window under it.
    if (cx.repo_dialogs.is_open() || crate::activity_log::is_open())
        && !matches!(
            action,
            NavigationAction::FocusByBadge(_)
                | NavigationAction::Cancel
                | NavigationAction::ArrangeDialog
        )
    {
        *cx.last_message = Some(msg::input::NAVIGATION_CANCELED.get().into());
        return Ok(Flow::Handled);
    }
    match action {
        NavigationAction::CollapseLeft => {
            *cx.left_pane_collapsed = !*cx.left_pane_collapsed;
            if *cx.left_pane_collapsed && *cx.focus == Focus::Refs {
                *cx.focus = Focus::Log;
            }
            *cx.last_message = Some(if *cx.left_pane_collapsed {
                msg::input::REFS_PANE_COLLAPSED.get().into()
            } else {
                msg::input::REFS_PANE_RESTORED.get().into()
            });
            persist_pane_layout(
                cx.session,
                *cx.divider1_pct,
                *cx.divider2_pct,
                cx.left_auto_collapse.persisted(*cx.left_pane_collapsed),
                cx.right_auto_collapse.persisted(*cx.right_pane_collapsed),
            );
        }
        NavigationAction::CollapseRight => {
            *cx.right_pane_collapsed = !*cx.right_pane_collapsed;
            if *cx.right_pane_collapsed && *cx.focus == Focus::Status {
                *cx.focus = Focus::Log;
            }
            *cx.last_message = Some(if *cx.right_pane_collapsed {
                msg::input::STATUS_PANE_COLLAPSED.get().into()
            } else {
                msg::input::STATUS_PANE_RESTORED.get().into()
            });
            persist_pane_layout(
                cx.session,
                *cx.divider1_pct,
                *cx.divider2_pct,
                cx.left_auto_collapse.persisted(*cx.left_pane_collapsed),
                cx.right_auto_collapse.persisted(*cx.right_pane_collapsed),
            );
        }
        // Look the pressed badge up in the list the
        // last render pass built (`nav_badge_targets`)
        // -- it is the render's own answer to "which
        // sub-pane does this key mean right now",
        // never a second, independently-guessed
        // mapping. A key with nothing bound to it
        // (mistyped, or badges changed since the
        // frame that showed them) just reports that.
        NavigationAction::FocusByBadge(c) => {
            // A control of the dialog or the Activity Log on top: a click on it.
            let over = cx
                .nav_badge_targets
                .iter()
                .find(|(badge, _)| *badge == c)
                .map(|(_, target)| *target);
            match over {
                Some(FocusTarget::Dialog(col, row)) => {
                    if let Some(effect) = cx.repo_dialogs.nav_press((col, row)) {
                        super::menu_actions::apply_effect(cx, effect);
                    }
                    if !cx.repo_dialogs.is_open() {
                        refresh_management(cx.session, &mut *cx.management);
                    }
                    *cx.should_quit |= cx.switch_to.is_some() || *cx.close_window;
                    return Ok(Flow::Handled);
                }
                Some(FocusTarget::Activity(col, row)) => {
                    crate::activity_log::nav_press((col, row));
                    return Ok(Flow::Handled);
                }
                _ => {}
            }
            let target = cx
                .nav_badge_targets
                .iter()
                .find(|(badge, _)| *badge == c)
                .map(|(_, target)| *target);
            // Whatever the badge names, the column gives the keyboard up; its
            // own badge takes it below.
            cx.commit_col.keys = false;
            if matches!(target, Some(FocusTarget::DiffView)) {
                commands::claim_diff_keys(cx);
            } else if target.is_some() {
                commands::release_diff_keys(cx);
            }
            match cx
                .nav_badge_targets
                .iter()
                .find(|(badge, _)| *badge == c)
                .map(|(_, target)| *target)
            {
                Some(FocusTarget::RefsFilter) => {
                    *cx.focus = Focus::Refs;
                    *cx.input_mode = InputMode::FilteringRefs(cx.refs_filter.clone());
                }
                Some(FocusTarget::RefsSection(section)) => {
                    *cx.focus = Focus::Refs;
                    *cx.active_ref_section = section;
                }
                Some(FocusTarget::Log) => {
                    *cx.focus = Focus::Log;
                    *cx.right_panel = RightPanel::CommitDetail;
                }
                Some(FocusTarget::CommitColumn) => commands::commit_column::focus(cx),
                Some(FocusTarget::StatusSection(section)) => {
                    *cx.focus = Focus::Status;
                    *cx.active_status_section = section;
                    *cx.right_panel = RightPanel::Status;
                }
                // Same two fields a click on the
                // Summary field itself sets
                // (`HitTarget::CommitSummaryField`'s
                // own arm) -- no `right_panel` change,
                // since the composer sits under
                // whichever the Status pane is
                // already showing.
                Some(FocusTarget::CommitFiles) => {
                    *cx.focus = Focus::Log;
                    *cx.right_panel = RightPanel::CommitDetail;
                    if *cx.commit_files_len > 0 {
                        *cx.commit_file_focus = true;
                        move_selection(&mut *cx.commit_file_state, *cx.commit_files_len, 0);
                        *cx.commit_files_follow = true;
                    }
                }
                Some(FocusTarget::CommitFilter) => {
                    *cx.focus = Focus::Log;
                    *cx.right_panel = RightPanel::CommitDetail;
                    *cx.commit_filter_focus = true;
                }
                Some(FocusTarget::CommitPathTree) => {
                    *cx.focus = Focus::Log;
                    commands::toggle_file_list_mode(cx);
                }
                Some(FocusTarget::CommitSort) => {
                    *cx.focus = Focus::Log;
                    commands::toggle_commit_sort(cx);
                }
                Some(FocusTarget::CommitViewAll) => {
                    *cx.focus = Focus::Log;
                    commands::toggle_view_all_files(cx);
                }
                Some(FocusTarget::ComposerSummary) => {
                    *cx.composer_focus = Some(ComposerField::Summary);
                    *cx.focus = Focus::Status;
                }
                // Both of these already own every key
                // the moment they're open (see
                // `FocusTarget::DiffView`/`FileView`'s
                // own doc comment) -- the badge exists
                // so `Ctrl+N` can name them, not
                // because there is a `focus` to set.
                Some(FocusTarget::DiffView) => {
                    *cx.last_message = Some(msg::input::DIFF_HAS_KEYBOARD.get().into());
                }
                Some(FocusTarget::FileView) => {
                    *cx.last_message = Some(msg::input::FILE_VIEW_HAS_KEYBOARD.get().into());
                }
                Some(FocusTarget::MergePane(pane)) => {
                    if let Some(view) = cx.merge.as_mut() {
                        let out = view.run(crate::merge_view::MergeCmd::FocusPane(pane));
                        commands::apply_merge_outcome(cx, out);
                    }
                }
                Some(FocusTarget::Editor) => {
                    *cx.last_message = Some(msg::input::EDITOR_HAS_KEYBOARD.get().into());
                }
                Some(FocusTarget::RepoManagerSearch) => {
                    if let Some(manager) = cx.management.as_mut() {
                        manager.search_focused = true;
                    }
                }
                Some(FocusTarget::RepoManagerSection(section)) => {
                    if let Some(manager) = cx.management.as_mut() {
                        if let Some(idx) = manager.rows().iter().position(|r| {
                            matches!(
                                r,
                                repo_manager::ManagerRow::Header(s) if *s == section
                            )
                        }) {
                            manager.selected = idx;
                        }
                    }
                }
                // A Home control is pressed like a click on it; the action it
                // returns runs through the same path a click's does.
                Some(FocusTarget::HomeControl(hit)) => {
                    let action = cx
                        .management
                        .as_mut()
                        .map(|manager| repo_manager::click(manager, hit, false));
                    if let Some(action) = action {
                        let anchor = (4, 6);
                        if let Some(message) =
                            apply_manager_action(action, anchor, &mut tab_ctx!(cx))
                        {
                            *cx.last_message = Some(message.clone());
                            if let Some(m) = cx.management.as_mut() {
                                m.message = Some(message);
                            }
                        }
                        *cx.should_quit |= cx.switch_to.is_some() || *cx.close_window;
                    }
                }
                Some(FocusTarget::Radar(spot)) => {
                    if let Some(manager) = cx.management.as_mut() {
                        let outcome = manager.radar.nav_focus(spot);
                        commands::apply_radar_outcome(cx, outcome);
                    }
                }
                // A control of Settings: the badge is a click on it, and a
                // click that closes the screen closes it the way the mouse's
                // does.
                Some(FocusTarget::Settings(col, row)) => {
                    if let Some(prefs) = cx.preferences.as_mut() {
                        let stay_open = prefs.nav_press((col, row));
                        if let Some(message) = prefs.take_message() {
                            *cx.last_message = Some(message);
                        }
                        if prefs.take_open_about() {
                            crate::about::open();
                        }
                        if !stay_open {
                            super::modal::settings_closed(cx);
                        }
                    }
                }
                // The tab bar's scroll arrow: one tab, as its click does.
                Some(FocusTarget::TabScroll(right)) => {
                    crate::tab_scroll::scroll_by(if right { 1 } else { -1 });
                }
                // Answered above, before the window's own targets.
                Some(FocusTarget::Dialog(..) | FocusTarget::Activity(..)) => {}
                // No sub-pane wears this badge right now (mistyped, or the badges shifted
                // since the frame that showed them): same outcome and wording as any
                // other unrecognized key here.
                None => {
                    *cx.last_message = Some(msg::input::NAVIGATION_CANCELED.get().into());
                }
            }
        }
        NavigationAction::Resize(resize) => commands::resize_pane(cx, resize),
        NavigationAction::GrowMessage => commands::resize_commit_message(cx, Resize::By(1)),
        NavigationAction::ShrinkMessage => commands::resize_commit_message(cx, Resize::By(-1)),
        NavigationAction::GrowSection => commands::resize_sidebar_section(cx, Resize::By(1)),
        NavigationAction::ShrinkSection => commands::resize_sidebar_section(cx, Resize::By(-1)),
        NavigationAction::OpenSettings => {
            *cx.preferences = Some(PreferencesState::open(cx.repo_root));
        }
        NavigationAction::OpenAbout => crate::about::open(),
        NavigationAction::OpenActivityLog => crate::activity_log::open_panel(),
        NavigationAction::OpenGlobalLog => crate::activity_log::open_global_panel(),
        NavigationAction::ShowJobs => crate::jobs_ui::run(crate::jobs_ui::JobsCmd::Show),
        NavigationAction::ShowUpdates => crate::updates::panel::open(),
        NavigationAction::ArrangeDialog => {
            *cx.last_message = Some(if crate::dialog_kit::frame::arrange_top() {
                msg::dialog_frame::ARRANGING.get().into()
            } else {
                msg::dialog_frame::WHY_NO_DIALOG.get().into()
            });
        }
        NavigationAction::ToggleHelp => {
            *cx.show_help = true;
        }
        NavigationAction::SelectTab => {
            *cx.input_mode = crate::tabs::open_tab_picker(&*cx.repo_tabs, *cx.active_tab);
        }
        NavigationAction::ChangeTheme => {
            commands::run_command(cx, PaletteCommand::ChangeTheme);
        }
        NavigationAction::ChangeLanguage => {
            commands::run_command(cx, PaletteCommand::ChangeLanguage);
        }
        NavigationAction::ChangeGraphColorTheme => {
            commands::run_command(cx, PaletteCommand::ChangeGraphColorTheme);
        }
        NavigationAction::ToggleGraphRowTints => {
            commands::run_command(cx, PaletteCommand::ToggleGraphRowTints);
        }
        NavigationAction::AgentShow => cx.ui_viewer.accept_pending(),
        NavigationAction::AgentBack => {
            if cx.ui_viewer.has_pending() {
                cx.ui_viewer.dismiss_pending();
            } else {
                cx.ui_viewer.request_back();
            }
        }
        NavigationAction::Cancel => {
            *cx.last_message = Some(msg::input::NAVIGATION_CANCELED.get().into());
        }
    }
    Ok(Flow::Handled)
}

/// Opens the history rewrite workbench for a palette row or the chord.
pub(super) fn open_rewrite(cx: &mut CommandCtx<'_>, entry: crate::rewrite_view::RewriteEntry) {
    use crate::rewrite_view::RewriteEntry;
    if !crate::build_features::HISTORY_REWRITE {
        return;
    }
    if entry == RewriteEntry::FromSelectedCommit {
        match selected_log_commit(&*cx.log_result, &*cx.log_state, &*cx.log_order) {
            Some(commit) => {
                open_rewrite_from_menu(cx, crate::log_menu::RewriteFrom::FromHere, commit)
            }
            None => {
                *cx.last_message = Some(
                    msg::palette::WHY_SELECT_A_COMMIT_IN_THE_LOG_FIRST_THE_WIP_ROW_ISN
                        .get()
                        .into(),
                );
            }
        }
        return;
    }
    let mut ctx = rewrite_scope_context(cx);
    if entry == RewriteEntry::SelectedCommits {
        if ctx.selection.is_empty() {
            match selected_log_commit(&*cx.log_result, &*cx.log_state, &*cx.log_order) {
                Some(commit) => ctx.selection.push(commit),
                None => {
                    *cx.last_message = Some(
                        msg::palette::WHY_SELECT_A_COMMIT_IN_THE_LOG_FIRST_THE_WIP_ROW_ISN
                            .get()
                            .into(),
                    );
                    return;
                }
            }
        }
        ctx.use_selection = true;
    }
    let view = rewrite_workbench(cx, ctx);
    let view = match entry {
        RewriteEntry::Rule(template) => view.with_rule(template),
        RewriteEntry::Purge => view.with_purge(),
        RewriteEntry::Identities => view.with_identities(),
        RewriteEntry::ScriptAi => view.with_script_ai(cx.ai_ui.enabled()),
        RewriteEntry::RulesAi => view.with_rules_ai(cx.ai_ui.enabled()),
        _ => view,
    };
    *cx.input_mode = InputMode::Rewrite(Box::new(view));
}

/// Opens the workbench with an agent's recipe in it (a proposal): nothing is
/// previewed or applied; the user does both.
pub(in crate::event_loop) fn open_rewrite_proposed(
    cx: &mut CommandCtx<'_>,
    recipe: gitpoulpi_core::domain::rewrite::Recipe,
    client: &str,
) {
    if !crate::build_features::HISTORY_REWRITE {
        return;
    }
    let ctx = rewrite_scope_context(cx);
    let view = rewrite_workbench(cx, ctx).with_proposed(recipe, client);
    *cx.input_mode = InputMode::Rewrite(Box::new(view));
}

/// The Log menu's "Rewrite history" rows.
pub(super) fn open_rewrite_from_menu(
    cx: &mut CommandCtx<'_>,
    from: crate::log_menu::RewriteFrom,
    commit: Oid,
) {
    use crate::log_menu::RewriteFrom;
    if !crate::build_features::HISTORY_REWRITE {
        return;
    }
    let mut ctx = rewrite_scope_context(cx);
    match from {
        RewriteFrom::Workbench => {}
        RewriteFrom::Selected => {
            if ctx.selection.len() < 2 {
                ctx.selection = vec![commit];
            }
            ctx.use_selection = true;
        }
        RewriteFrom::FromHere => {
            // The commit and everything after it: the scope starts after
            // its (first) parent. A root commit starts at the beginning.
            ctx.since = cx.log_result.as_ref().ok().and_then(|graph| {
                graph
                    .nodes
                    .iter()
                    .find(|n| n.id == commit)
                    .and_then(|n| n.parents.first().copied())
            });
        }
    }
    let view = rewrite_workbench(cx, ctx);
    *cx.input_mode = InputMode::Rewrite(Box::new(view));
}

/// What the scope bar offers: the Log's selection and the branches and tags
/// the Refs sidebar lists.
fn rewrite_scope_context(cx: &mut CommandCtx<'_>) -> crate::rewrite_view::ScopeContext {
    use gitpoulpi_core::domain::RefKind;
    let mut ctx = crate::rewrite_view::ScopeContext::default();
    let selected = cx.log_multi.newest_first(&*cx.log_result);
    // "Test on selection" (a Script rule) runs on what is selected, else on
    // the newest commits the Log shows.
    if selected.is_empty() {
        if let Ok(graph) = &*cx.log_result {
            ctx.test_commits = graph
                .nodes
                .iter()
                .take(crate::rewrite_view::TEST_SAMPLE)
                .map(|n| n.id)
                .collect();
        }
    } else {
        ctx.test_commits = selected.clone();
        ctx.test_from_selection = true;
    }
    // The sample the ✦ script dialog may send (author, date, subject only):
    // the same commits as the test.
    if let Ok(graph) = &*cx.log_result {
        let wanted: Vec<_> = ctx.test_commits.to_vec();
        ctx.sample = graph
            .nodes
            .iter()
            .filter(|n| wanted.contains(&n.id))
            .take(crate::rewrite_view::SAMPLE_MAX)
            .map(|n| crate::rewrite_view::SampleRow {
                author_name: n.author_name.clone(),
                author_email: n.author_email.clone(),
                timestamp: n.timestamp,
                subject: n.summary.clone(),
            })
            .collect();
    }
    if selected.len() > 1 {
        ctx.selection = selected;
    }
    if let Ok(refs) = &*cx.refs_result {
        for r in refs {
            match &r.kind {
                RefKind::LocalBranch { name } => {
                    if r.is_head {
                        ctx.head_branch = Some(name.clone());
                    }
                    ctx.branches.push(name.clone());
                }
                RefKind::Tag { name } => ctx.tags.push(name.clone()),
                RefKind::RemoteBranch { .. } => {}
            }
        }
    }
    ctx
}

/// A workbench on this repository: recipes saved next to the repository or
/// under the user's config dir.
fn rewrite_workbench(
    cx: &mut CommandCtx<'_>,
    ctx: crate::rewrite_view::ScopeContext,
) -> crate::rewrite_view::RewriteView {
    crate::rewrite_view::RewriteView::open(
        cx.link.activity_handle().raw.requester(),
        crate::rewrite_view::RecipeStore {
            repo_root: cx.repo_root.clone(),
            config_dir: crate::session_store::config_dir(),
        },
        ctx,
    )
}
