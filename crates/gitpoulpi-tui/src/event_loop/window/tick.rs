//! The pre-frame step: everything the loop does before drawing -- timers
//! (auto-fetch, hosting, column-layout persistence), worker polls (AI,
//! hosting, avatars, token probes) and the per-frame normalisation of the
//! selection, panes and views the draw pass and the input drain rely on.

use crate::event_loop::*;
use init::WindowState;

/// What the pre-frame step worked out for this iteration, read by the draw
/// pass and the input drain so what the popup grays out and what Enter
/// refuses can't disagree.
pub(in crate::event_loop) struct FrameInputs {
    pub(in crate::event_loop) journal_state: gitpoulpi_core::domain::JournalState,
    pub(in crate::event_loop) undo_busy: bool,
    pub(in crate::event_loop) undo_avail: crate::undo_ui::Avail,
    pub(in crate::event_loop) redo_avail: crate::undo_ui::Avail,
    pub(in crate::event_loop) unpushed_tags: std::collections::HashSet<String>,
    pub(in crate::event_loop) palette_ctx: PaletteContext,
}

/// How long the input drain's first `poll` may block: one frame while
/// something animates (the pacing of the spinner), nothing otherwise --
/// `wait_for_work` has already blocked until there was a reason to draw.
/// Every later poll in the same drain uses a zero timeout, so an empty queue
/// ends the drain immediately instead of costing a frame.
pub(in crate::event_loop) fn poll_timeout(
    first_poll: bool,
    animating: bool,
) -> std::time::Duration {
    if first_poll && animating {
        super::idle::FRAME
    } else {
        std::time::Duration::ZERO
    }
}

impl WindowState {
    /// The WIP row is not a commit, so selecting it -- by any path -- shows
    /// the working-changes pane, never an empty commit detail. Every handler
    /// that moved the selection set `CommitDetail`; this is applied on top of
    /// them once per frame AND before each input event of a drain, so an
    /// event queued behind a click on the WIP row (a right-click, a key) never
    /// sees the transient `CommitDetail` the click left until the next frame.
    /// See `right_panel_for_log_selection`. Returns whether the WIP row is the
    /// selection.
    pub(in crate::event_loop) fn normalize_right_panel(&mut self) -> bool {
        self.refresh_compare();
        // A comparison with the WIP row in it is the commit-detail view's:
        // its files are the answer, not the working-changes composer.
        let wip_selected =
            wip_row_selected(&self.status_result, &self.log_state) && self.compare.is_none();
        let normalized = right_panel_for_log_selection(self.right_panel, wip_selected);
        if normalized != self.right_panel {
            self.right_panel = normalized;
            self.commit_file_focus = false;
        }
        wip_selected
    }

    /// Runs the timers and polls, then settles the state this frame will draw.
    pub(in crate::event_loop) fn tick(&mut self) -> FrameInputs {
        self.note_message();
        crate::activity_log::set_active_repo(&self.repo.root);
        if let Some(note) = crate::launcher_sync::poll() {
            self.last_message = Some(note);
        }
        // What a check the user asked for found.
        if let Some(note) = crate::updates::take_message() {
            self.last_message = Some(note);
        }
        self.sync_external_tabs();
        // "Open to commit" from the Remove worktree dialog: this window is
        // the worktree it asked for, so the commit message takes the caret.
        #[cfg(feature = "worktree")]
        if crate::worktree_remove_dialog::take_commit_focus(&self.repo.root) {
            self.right_panel = RightPanel::Status;
            self.right_pane_collapsed = false;
            self.focus = Focus::Status;
            self.composer_focus = Some(ComposerField::Summary);
        }
        // `workspaces.toml` edited from a shell or the CLI.
        crate::workspace_ui::watch::poll(&self.session);
        self.agent_view_tick();
        // What the Radar asked of this repository's window.
        #[cfg(feature = "hosting")]
        self.radar_pending_tick();
        if let InputMode::WorkspaceSearch(search) = &mut self.input_mode {
            search.tick(Instant::now());
        }
        self.tick_reveal();
        self.tick_history();
        self.tick_blame_diff();
        // Persist a changed Log column layout (not mid-drag: a resize
        // fires per mouse move). The write runs on its own thread.
        if self.dragging_log_boundary.is_none() && self.dragging_log_header.is_none() {
            let now = LogColumnPrefs {
                order: self.log_column_order.clone(),
                hidden: self.log_column_hidden.clone(),
                widths: self.log_column_overrides.clone(),
                scroll_widths: self.log_column_scroll_widths.clone(),
            };
            if now != self.last_saved_log_columns {
                if let Some(path) = &self.diff_config_path {
                    now.save_in_background(path);
                }
                self.last_saved_log_columns = now;
            }
        }
        // Persist a changed left-pane section visibility: this repository's
        // own choice in its state file, the global default (`[ui]`) only
        // after "Use for all repositories".
        self.sidebar_scope.follow_scope(
            &mut self.hidden_sidebar_sections,
            crate::live_settings::sidebar_sections_global(),
        );
        let (default, own) = self.sidebar_scope.take_changes();
        if let (Some(default), Some(path)) = (default, &self.diff_config_path) {
            crate::refs_sidebar::save_hidden_in_background(path, &default);
        }
        if let (Some(own), false) = (own, self.no_repository) {
            crate::sections_scope::save_own(&crate::fold_state::state_path(&self.repo_root), own);
        }
        // Folded panes are remembered per repository: a tab switch
        // rebuilds this window. One comparison per frame; a write only
        // when something was folded or unfolded.
        // An active filter shows every folder with a match open, in a view
        // of its own: the user's folds are parked meanwhile and are what is
        // saved (`tree_fold::FilterOverlay`).
        self.ref_fold_overlay
            .sync(&self.refs_filter, &mut self.collapsed_ref_folders);
        self.commit_fold_overlay.sync(
            &self.commit_files_filter,
            &mut self.collapsed_commit_folders,
        );
        let real_ref_folders = self.ref_fold_overlay.real(&self.collapsed_ref_folders);
        if *real_ref_folders != self.last_saved_ref_folders && !self.no_repository {
            crate::refs_tree::save_collapsed(&self.repo_root, real_ref_folders);
            self.last_saved_ref_folders = real_ref_folders.clone();
        }
        let folds = crate::fold_state::Folds::capture(
            &self.collapsed_ref_sections,
            &self.collapsed_status_sections,
            self.hosting.folded_groups(),
        )
        .with_trees(
            &mut self.collapsed_status_folders,
            self.commit_fold_overlay
                .real_mut(&mut self.collapsed_commit_folders),
            &self.last_saved_folds,
        );
        if folds != self.last_saved_folds && !self.no_repository {
            crate::fold_state::save(&self.repo_root, &folds);
            self.last_saved_folds = folds;
        }
        self.persist_sizes();
        // `general.auto_fetch_interval_minutes`. Fired through `submit_action`,
        // the seam the toolbar's Fetch button uses, so auto-fetch cannot disagree
        // with a manual fetch about what "fetch" does.
        self.fetch_on_open_once();
        if let Some(interval) = self.auto_fetch_interval {
            // A window with no repository (a New tab, Home) has no folder to
            // run git in: on Windows the spawn failed with os error 267.
            if fetch_policy::auto_fetch_due(self.no_repository, self.last_auto_fetch, interval) {
                self.last_auto_fetch = Instant::now();
                fetch_policy::note_fetch(&self.repo_root);
                self.link.submit_auto_fetch();
                self.last_message =
                    Some(gitpoulpi_i18n::msg::window::AUTO_FETCHING.get().to_string());
            }
        }

        // The history rewrite workbench: collect what the daemon answered.
        if let InputMode::Rewrite(view) = &mut self.input_mode {
            view.poll(self.link.activity_handle().rewrite_progress());
            // S23: what the host protects (asked once, only for a linked
            // repository; the answer arrives on the view's own channel).
            if view.wants_host() {
                if let Some(host) = self.hosting.rewrite_host() {
                    view.set_host(host);
                }
            }
            // An extraction made a new repository: it opens in a new tab.
            if let Some(path) = view.take_open_repo() {
                self.submodule_open = Some(path);
            }
            // ✦ The Script editor's "Write with AI": what the dialog asked
            // of the window (describe the prompt, send, cancel).
            view.set_ai(self.ai_ui.enabled(), self.ai_ui.running());
            while let Some(wish) = view.take_ai_wish() {
                match wish {
                    crate::rewrite_view::AiWish::Describe => {
                        view.set_ai_system(
                            crate::ai_ui::rewrite_prompt_preview(&self.session, &self.repo).map(
                                |p| crate::rewrite_view::AiSystem {
                                    prompt: p.system,
                                    via: p.via,
                                },
                            ),
                        );
                    }
                    crate::rewrite_view::AiWish::Send {
                        instruction,
                        sample,
                    } => {
                        let mut line = None;
                        self.ai_ui.start_into(
                            &self.session,
                            &self.repo,
                            AiRequest::RewriteScript {
                                instruction,
                                sample,
                            },
                            &mut line,
                        );
                        if let Some(why) = line {
                            view.ai_stopped(why);
                        }
                    }
                    crate::rewrite_view::AiWish::Cancel => self.ai_ui.cancel(),
                    crate::rewrite_view::AiWish::DescribeRules => {
                        view.set_rules_system(
                            crate::ai_ui::rewrite_rules_prompt_preview(&self.session, &self.repo)
                                .map(|p| crate::rewrite_view::AiSystem {
                                    prompt: p.system,
                                    via: p.via,
                                }),
                        );
                    }
                    crate::rewrite_view::AiWish::SendRules {
                        description,
                        context,
                    } => {
                        let mut line = None;
                        self.ai_ui.start_into(
                            &self.session,
                            &self.repo,
                            AiRequest::RewriteRules {
                                description,
                                context,
                            },
                            &mut line,
                        );
                        if let Some(why) = line {
                            view.rules_ai_stopped(why);
                        }
                    }
                }
            }
        }

        // Hosting: the periodic timer and whatever the worker answered.
        // Preferences closing is a reason to ask again (a profile may
        // have been connected, reconnected or removed) and to re-read the
        // refresh interval.
        self.hosting.tick(Instant::now());
        self.hosting.poll();
        // AI: apply what the worker answered, and keep the status line's
        // progress text alive while a request runs.
        if let Some(view) = self.merge.as_mut() {
            view.ai_enabled = self.ai_ui.enabled();
            view.ai_busy = self.ai_ui.running_conflict();
        }
        // A prompt dialog's ✦ button follows AI on / off and the running state.
        if let InputMode::Prompt(dialog) = &mut self.input_mode {
            dialog.sync_ai(self.ai_ui.enabled(), self.ai_ui.running());
            // The Remotes dialog reads the branch's remotes from the daemon.
            dialog.poll();
        }
        // The Create worktree dialog's ✦ follows AI on / off and the running state.
        self.repo_dialogs
            .sync_worktree_ai(self.ai_ui.enabled(), self.ai_ui.running_worktree_name());
        self.repo_dialogs
            .sync_composer_ai(self.ai_ui.enabled(), &self.repo.root);
        if let Some(outcome) = self.ai_ui.poll() {
            // A failed job is a card (the status line is cleared below).
            if let AiOutcome::Done(_, Err(why)) = &outcome {
                crate::activity_log::report_problem(
                    crate::activity_log::ProblemSource::Ai,
                    &gitpoulpi_i18n::msg::window::ai_failed(why),
                );
            }
            match outcome {
                AiOutcome::Done(
                    AiRequest::ResolveConflict { path, conflict, .. },
                    Ok(AiAnswer::Resolution(lines)),
                ) => self.put_conflict_suggestion(&path, conflict, &lines),
                AiOutcome::Done(AiRequest::ResolveConflict { .. }, Err(why)) => {
                    let line = gitpoulpi_i18n::msg::merge::msg_ai_failed(&why);
                    if let Some(view) = self.merge.as_mut() {
                        view.note = Some(line);
                    }
                    self.last_message = Some(String::new());
                }
                AiOutcome::Done(
                    AiRequest::RewriteScript { .. },
                    Ok(AiAnswer::RewriteScript(done)),
                ) => match &mut self.input_mode {
                    InputMode::Rewrite(view) => {
                        view.put_ai_script(done.script, !done.problems.is_empty())
                    }
                    _ => {
                        self.last_message =
                            Some(gitpoulpi_i18n::msg::rewrite_ai::CLOSED.get().to_string());
                    }
                },
                AiOutcome::Done(AiRequest::RewriteScript { .. }, Err(why)) => {
                    if let InputMode::Rewrite(view) = &mut self.input_mode {
                        view.ai_stopped(gitpoulpi_i18n::msg::window::ai_failed(&why));
                    }
                    self.last_message = Some(String::new());
                }
                AiOutcome::Canceled(AiRequest::RewriteScript { .. }) => {
                    if let InputMode::Rewrite(view) = &mut self.input_mode {
                        view.ai_stopped(gitpoulpi_i18n::msg::window::AI_CANCELED.get().to_string());
                    }
                    self.last_message =
                        Some(gitpoulpi_i18n::msg::window::AI_CANCELED.get().to_string());
                }
                AiOutcome::Done(
                    AiRequest::RewriteRules { .. },
                    Ok(AiAnswer::RewriteRules(proposal)),
                ) => match &mut self.input_mode {
                    InputMode::Rewrite(view) => view.put_ai_rules(*proposal),
                    _ => {
                        self.last_message =
                            Some(gitpoulpi_i18n::msg::rewrite_ai::CLOSED.get().to_string());
                    }
                },
                AiOutcome::Done(AiRequest::RewriteRules { .. }, Err(why)) => {
                    if let InputMode::Rewrite(view) = &mut self.input_mode {
                        view.rules_ai_stopped(gitpoulpi_i18n::msg::window::ai_failed(&why));
                    }
                    self.last_message = Some(String::new());
                }
                AiOutcome::Canceled(AiRequest::RewriteRules { .. }) => {
                    if let InputMode::Rewrite(view) = &mut self.input_mode {
                        view.rules_ai_stopped(
                            gitpoulpi_i18n::msg::window::AI_CANCELED.get().to_string(),
                        );
                    }
                    self.last_message =
                        Some(gitpoulpi_i18n::msg::window::AI_CANCELED.get().to_string());
                }
                AiOutcome::Done(
                    _,
                    Ok(
                        AiAnswer::Resolution(_)
                        | AiAnswer::RewriteScript(_)
                        | AiAnswer::RewriteRules(_),
                    ),
                ) => {}
                AiOutcome::Done(_, Ok(AiAnswer::CommitMessage(message))) => {
                    self.commit_summary = message.summary;
                    self.commit_description = message.description;
                    self.field_editor.caret_mut(MainField::Summary).set_cursor(
                        &self.commit_summary,
                        self.commit_summary.len(),
                        false,
                    );
                    self.field_editor
                        .caret_mut(MainField::Description)
                        .set_cursor(
                            &self.commit_description,
                            self.commit_description.len(),
                            false,
                        );
                    self.last_message = Some(
                        gitpoulpi_i18n::msg::window::AI_WROTE_COMMIT
                            .get()
                            .to_string(),
                    );
                }
                AiOutcome::Done(_, Ok(AiAnswer::Explanation(text))) => {
                    self.ai_ui.fill_popup(Ok(text));
                    self.last_message = Some(
                        gitpoulpi_i18n::msg::window::AI_EXPLANATION_READY
                            .get()
                            .to_string(),
                    );
                }
                AiOutcome::Done(_, Ok(AiAnswer::StashMessage(text))) => {
                    if let InputMode::Prompt(dialog) = &mut self.input_mode {
                        if dialog.fill_from_ai(&text) {
                            self.last_message = Some(
                                gitpoulpi_i18n::msg::window::AI_WROTE_STASH
                                    .get()
                                    .to_string(),
                            );
                        }
                    }
                }
                AiOutcome::Done(_, Ok(AiAnswer::Split { source, plan })) => {
                    self.open_ai_split(*source, plan);
                }
                AiOutcome::Done(AiRequest::SplitCommit { oid, .. }, Err(_)) => {
                    // The changes could not be read for the AI: the plain
                    // editor asks the daemon again and says why if it fails.
                    self.link.request_split_source(oid);
                    self.last_message = Some(String::new());
                }
                AiOutcome::Done(_, Ok(AiAnswer::Plan(plan))) => {
                    self.input_mode =
                        InputMode::ComposeCommits(Box::new(ComposeForm::with_expanded(
                            plan,
                            self.settings.config().ui.composer_files
                                == gitpoulpi_config::ComposerFiles::Expanded,
                        )));
                    self.last_message = Some(
                        gitpoulpi_i18n::msg::window::AI_PROPOSED_PLAN
                            .get()
                            .to_string(),
                    );
                }
                AiOutcome::Done(_, Ok(AiAnswer::PullRequest(text))) => {
                    self.repo_dialogs
                        .pull_request_generated(Ok((text.title, text.description)));
                    self.last_message =
                        Some(gitpoulpi_i18n::msg::window::AI_WROTE_PR.get().to_string());
                }
                AiOutcome::Done(AiRequest::PullRequest { .. }, Err(why)) => {
                    self.repo_dialogs.pull_request_generated(Err(why.clone()));
                    self.last_message = Some(String::new());
                }
                AiOutcome::Done(AiRequest::Explain(_), Err(why)) => {
                    self.ai_ui.fill_popup(Err(why.clone()));
                    self.last_message = Some(String::new());
                }
                AiOutcome::Done(_, Ok(AiAnswer::WorktreeName(names))) => {
                    self.repo_dialogs
                        .worktree_named(Ok((names.name, names.branch)));
                    self.last_message = Some(String::new());
                }
                AiOutcome::Done(AiRequest::WorktreeName { .. }, Err(why)) => {
                    // Shown inside the dialog, not on the status line.
                    self.repo_dialogs.worktree_named(Err(why.clone()));
                    self.last_message = Some(String::new());
                }
                AiOutcome::Done(_, Err(_)) => {
                    self.last_message = Some(String::new());
                }
                AiOutcome::Canceled(AiRequest::ResolveConflict { .. }) => {
                    let line = gitpoulpi_i18n::msg::merge::MSG_AI_CANCELED
                        .get()
                        .to_string();
                    if let Some(view) = self.merge.as_mut() {
                        view.note = Some(line.clone());
                    }
                    self.last_message = Some(line);
                }
                AiOutcome::Canceled(request) => {
                    if matches!(request, AiRequest::Explain(_)) {
                        self.ai_ui.close_popup();
                    }
                    if matches!(request, AiRequest::PullRequest { .. }) {
                        self.repo_dialogs
                            .pull_request_generated(Err("canceled".into()));
                    }
                    if matches!(request, AiRequest::WorktreeName { .. }) {
                        self.repo_dialogs.worktree_named(Err("canceled".into()));
                    }
                    self.last_message =
                        Some(gitpoulpi_i18n::msg::window::AI_CANCELED.get().to_string());
                }
            }
        } else if let Some((conflict, via, secs)) = self.ai_ui.running_conflict_info() {
            let line = format!(
                "{} {}",
                crate::theme::AI_SPARK,
                gitpoulpi_i18n::msg::merge::msg_ai_working(conflict + 1, &via, secs)
            );
            if let Some(view) = self.merge.as_mut() {
                view.note = Some(line.clone());
            }
            self.last_message = Some(line);
        } else if let Some(line) = self.ai_ui.progress_line() {
            self.last_message = Some(line);
        }
        if let Some(notice) = self.hosting.take_notice() {
            self.last_message = Some(notice);
        }
        if self.hosting_prefs_was_open && self.preferences.is_none() {
            self.ai_ui.reload();
            self.hosting.set_interval(
                gitpoulpi_config::Settings::resolve_ambient()
                    .ok()
                    .and_then(|s| s.config().general.hosting_refresh_interval()),
            );
            self.hosting.demand();
        }
        self.hosting_prefs_was_open = self.preferences.is_some();
        {
            terminology::set_host(self.hosting.linked_is_gitlab());
            let (pull_rows, issue_rows) = self.hosting.row_counts();
            move_selection(&mut self.sidebar_states.pulls, pull_rows, 0);
            move_selection(&mut self.sidebar_states.issues, issue_rows, 0);
            let (pulls_shown, issues_shown) = self.hosting.sections_visible();
            let gone = match self.active_ref_section {
                SidebarSection::PullRequests => !pulls_shown,
                SidebarSection::Issues => !issues_shown,
                _ => false,
            };
            if gone {
                self.active_ref_section = SidebarSection::Local;
            }
        }

        // `gitpoulpi refs hide|show|solo` (or another TUI) changed the
        // filter: reload it. The graph cache keys on the sets, so the Log
        // relays out by itself.
        crate::push_group_ui::tick(
            &self.repo.root,
            self.refs_result
                .as_ref()
                .ok()
                .and_then(|refs| crate::remotes_ui::head_branch(refs))
                .as_ref()
                .map(|h| h.name.as_str()),
        );
        crate::push_group_ui::drain(&self.link);
        if self.link.take_pending_auto_stash_changed() {
            self.pending_auto_stash = crate::autostash_banner::load(&self.repo.root);
        }
        if self.link.take_pending_auto_stash_gone().is_some() {
            self.last_message = Some(gitpoulpi_i18n::msg::input::AUTOSTASH_REMOVED.get().into());
        }
        if self.link.take_ref_visibility_changed() {
            (self.hidden_from_graph, self.solo_refs) =
                refs_tree::load_ref_visibility(&self.repo.root);
            self.log_order = log_rows_now(
                &self.log_result,
                &self.status_result,
                &self.stashes_result,
                stash_rows_shown(self.graph_show_stashes, &self.solo_refs),
                &self.solo_refs,
                &self.refs_result,
            );
            self.log_len = log_len_now(
                &self.log_result,
                &self.solo_refs,
                &self.refs_result,
                &self.log_order,
            );
            move_table_selection(&mut self.log_state, self.log_len, 0);
        }

        // The one detector of "the walk changed": whatever mutated the hidden
        // set this frame (a click, a key, the palette, `refs hide` from a
        // shell) shows up as a different filter. The daemon walks from the
        // visible tips, so what the Log holds, what the lazy loader believes
        // and the status bar's total all describe the old walk: forget them
        // and ask for the new one. The old graph stays on screen until it
        // lands (no blank frame); answers still on their way for the old
        // filter are dropped by their generation (`apply.rs`).
        let walk = crate::graph::walk_filter(&self.hidden_from_graph, &self.solo_refs);
        if self.link.set_log_filter(walk) {
            self.lazy_log.restart();
            self.link.set_log_depth(0);
            self.commit_total.forget();
            // The new walk must replace the held Log even when it hashes
            // like it (a hidden ref whose commits another ref reaches).
            self.read_hashes.forget_log();
            self.link.request_refresh();
        }

        // The Log's display order for this frame. Cheap (no clone, one
        // scan per stash), so recomputed rather than kept in step by hand.
        // Outside solo the row count follows from it directly; under solo
        // there are no stash rows and the sites that change the solo set
        // keep `log_len` themselves.
        self.log_order = log_rows_now(
            &self.log_result,
            &self.status_result,
            &self.stashes_result,
            stash_rows_shown(self.graph_show_stashes, &self.solo_refs),
            &self.solo_refs,
            &self.refs_result,
        );
        if self.solo_refs.is_empty() {
            self.log_len = self
                .log_order
                .len(self.log_result.as_ref().map(|g| g.nodes.len()).unwrap_or(0));
        }

        self.refresh_log_hover();

        // Which tags no remote is known to have, computed once per
        // frame from the refs the daemon already handed us -- the same
        // set that draws the sidebar marker, grays out the palette's tag
        // commands and decides which row a tag's context menu offers.
        // One source, so those three can never tell different stories,
        // and no network: `Ref::tag_sync` is filled in from local refs
        // (see `REMOTE_TAG_MIRROR_PREFIX`).
        let unpushed_tags: std::collections::HashSet<String> = self
            .refs_result
            .as_ref()
            .map(|refs| {
                refs.iter()
                    .filter(|r| r.is_unpushed_tag())
                    .filter_map(|r| r.tag_name().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();

        // The same refs feed `tag_where_ui`: where each tag lives (marker,
        // tooltip, menus), one snapshot per frame.
        if let Ok(refs) = self.refs_result.as_ref() {
            crate::tag_where_ui::install(refs);
        }

        let wip_selected = self.normalize_right_panel();
        // The WIP row becoming selected restores a collapsed Status
        // pane -- Changed files and the composer live nowhere else. Same
        // "applied once, on top of every path that could select it"
        // shape as the block above; edge-triggered so a deliberate
        // re-collapse afterward isn't immediately fought.
        if wip_selection_should_expand_status(
            wip_selected,
            self.wip_row_was_selected,
            self.right_pane_collapsed,
        ) {
            self.right_pane_collapsed = false;
            persist_pane_layout(
                &self.session,
                self.divider1_pct,
                self.divider2_pct,
                self.left_auto_collapse.persisted(self.left_pane_collapsed),
                self.right_auto_collapse
                    .persisted(self.right_pane_collapsed),
            );
        }
        self.wip_row_was_selected = wip_selected;
        // A diff / file view in the center collapses the
        // Refs pane; closing it restores it (unless the user toggled it
        // meanwhile). Edge-triggered and never persisted -- see
        // `AutoCollapse`.
        // A clean editor gives way when another file is
        // opened in the central column under it; one with unsaved
        // changes stays until they are saved or discarded.
        self.follow_editor_selection();
        if self.editor.as_ref().is_some_and(|e| {
            editor_should_yield(e, self.file_insight.as_ref(), self.diff_view_state.as_ref())
        }) {
            self.editor = None;
        }
        sync_central_view(
            &mut self.central_view,
            self.merge.is_some(),
            self.editor.is_some(),
            self.file_insight.is_some(),
            self.diff_view_state.is_some(),
        );
        // The merge editor takes the whole width: the right panel goes too,
        // and comes back when the editor closes (unless the user opened it
        // meanwhile).
        let in_log = self.central_view == CentralView::Log;
        self.right_auto_collapse
            .enter_view(in_log, self.right_pane_collapsed);
        let merge_open = self.central_view == CentralView::Merge;
        if self
            .right_auto_collapse
            .sync(merge_open, &mut self.right_pane_collapsed)
            && self.right_pane_collapsed
        {
            self.status_focus_restore = self.focus == Focus::Status;
            if self.status_focus_restore {
                self.focus = Focus::Log;
            }
        }
        // Back in the Log: the right pane is as the Log left it, whatever
        // was done to it in the diff / file / editor views.
        if let Some(before) = self.right_auto_collapse.leave_view(in_log) {
            self.right_pane_collapsed = before;
            if before && self.focus == Focus::Status {
                self.focus = Focus::Log;
            }
        }
        // The editor closed with the panel back: the focus it took goes back.
        if !merge_open
            && !self.right_pane_collapsed
            && std::mem::take(&mut self.status_focus_restore)
            && self.focus == Focus::Log
        {
            self.focus = Focus::Status;
        }
        // The left pane follows the same rule as the right one: recorded
        // before the diff / file auto-collapse, restored after it.
        self.left_auto_collapse
            .enter_view(in_log, self.left_pane_collapsed);
        if self.left_auto_collapse.sync(
            matches!(
                self.central_view,
                CentralView::Diff | CentralView::File | CentralView::Editor | CentralView::Merge
            ),
            &mut self.left_pane_collapsed,
        ) && self.left_pane_collapsed
            && self.focus == Focus::Refs
        {
            self.focus = Focus::Log;
        }
        if let Some(before) = self.left_auto_collapse.leave_view(in_log) {
            self.left_pane_collapsed = before;
            if before && self.focus == Focus::Refs {
                self.focus = Focus::Log;
            }
        }
        // The WIP row's field only keeps the keyboard while the row it is
        // drawn in is the Log's selection and the Log is on screen with
        // focus -- otherwise the caret would be somewhere the user is not
        // looking, eating their keys.
        if self.composer_focus == Some(ComposerField::WipSummary)
            && !(wip_selected && self.focus == Focus::Log && self.central_view == CentralView::Log)
        {
            self.composer_focus = None;
        }

        // The palette is closed again: the composer review it was opened
        // from comes back (and opens its Move dropdown when asked).
        if self.compose_stash.is_some()
            && !matches!(self.input_mode, InputMode::CommandPalette { .. })
        {
            if let Some(mut form) = self.compose_stash.take() {
                if matches!(self.input_mode, InputMode::Normal) {
                    if std::mem::take(&mut self.compose_move_requested) {
                        form.start_move();
                    }
                    self.input_mode = InputMode::ComposeCommits(form);
                }
            }
        }

        // Built once per frame, from the state the user is about to see,
        // and reused by both the palette's renderer and its Enter handler
        // further down -- so what the popup grays out and what Enter
        // refuses can't disagree.
        // The undo journal, read once per frame: the toolbar, the palette
        // and the tooltip all draw from this one copy.
        let journal_state = self.link.journal_state();
        let undo_busy = self.link.undo_busy();
        let undo_avail = crate::undo_ui::avail(
            &journal_state,
            gitpoulpi_core::domain::Direction::Undo,
            undo_busy,
        );
        let redo_avail = crate::undo_ui::avail(
            &journal_state,
            gitpoulpi_core::domain::Direction::Redo,
            undo_busy,
        );
        // A risky undo the daemon asked about: put the question up when
        // nothing else is being asked.
        if matches!(self.input_mode, InputMode::Normal) {
            if let Some(c) = self.link.take_undo_confirm() {
                self.input_mode = InputMode::Confirm {
                    prompt: c.prompt,
                    action: ConfirmAction::UndoRedo {
                        direction: c.direction,
                        expect: c.expect,
                    },
                };
            }
        }
        let palette_ctx = {
            let selected_ref = selected_ref_leaf(
                &self.refs_result,
                &self.refs_filter,
                self.active_ref_section,
                self.sidebar_states
                    .state_mut(self.active_ref_section)
                    .selected(),
                self.hierarchical_refs,
                &self.collapsed_ref_folders,
                &self.remote_names,
            );
            let has_ref = selected_ref.is_some();
            let ref_kind = self.active_ref_section.ref_kind();
            PaletteContext {
                settings_open: self.preferences.is_some(),
                undo: undo_avail,
                redo: redo_avail,
                undo_offer: crate::safety_panel::offer_of(&journal_state).is_some(),
                safety_tab: self.repo_dialogs.safety_tab(),
                review: self.repo_dialogs.review_avail().or_else(|| {
                    // The review hosted over the Radar (not cloned).
                    self.management
                        .as_ref()
                        .and_then(|m| m.radar.review_avail())
                }),
                board: self.repo_dialogs.board_avail(),
                item: self.repo_dialogs.item_avail().or_else(|| {
                    // The conversation hosted over the Radar (not cloned).
                    self.management.as_ref().and_then(|m| m.radar.item_avail())
                }),
                agent_pending: self.ui_viewer.has_pending(),
                auto_stash_pending: self.pending_auto_stash.is_some(),
                agent_back: self.ui_viewer.can_go_back(),
                ai_enabled: self.ai_ui.enabled(),
                ai_running: self.ai_ui.running(),
                hosting: self.hosting.availability(),
                open_repository_tabs: self.repo_tabs.len(),
                current_tab_index: self.active_tab,
                in_repository_management: self.management.is_some(),
                workspace_active: crate::workspace_ui::chip::active_id().is_some(),
                workspaces_known: crate::workspace_ui::chip::visible(),
                workspace_selected: self.management.as_ref().is_some_and(|m| {
                    m.page == crate::repo_manager::HomePage::Workspaces
                        && m.ws.selected_workspace().is_some()
                }),
                management_row_selected: self.management.as_ref().is_some_and(|m| m.row_selected()),
                radar_open: self.management.as_ref().is_some_and(|m| m.radar.is_open()),
                notes_open: self.management.as_ref().is_some_and(|m| m.notes.is_open()),
                update_known: crate::updates::snapshot().available.is_some()
                    || crate::updates::snapshot().skipped.is_some(),
                update_whys: crate::updates::whys(),
                radar_row: self
                    .management
                    .as_ref()
                    .is_some_and(|m| m.radar.row_selected()),
                has_remotes: !self.remote_names.is_empty(),
                group: crate::push_group_ui::group_state(&self.repo.root),
                lfs_enabled: self.lfs.enabled,
                lfs_program_ok: self.lfs.unavailable_reason().is_none(),
                in_refs_pane: self.focus == Focus::Refs,
                in_diff_view: self.central_view == CentralView::Diff,
                in_file_view: self.central_view == CentralView::File && self.file_insight.is_some(),
                blame_shown: self.central_view == CentralView::File
                    && self
                        .file_insight
                        .as_ref()
                        .is_some_and(|f| f.blame_lines().is_some()),
                image_shown: (self.central_view == CentralView::File
                    && crate::image_view::pane::active())
                    || (self.central_view == CentralView::Diff
                        && crate::image_view::split::active()),
                in_history_view: self.history.is_some()
                    && match self.central_view {
                        CentralView::Diff => true,
                        CentralView::File => self.file_insight.as_ref().is_some_and(|f| {
                            self.history
                                .as_ref()
                                .is_some_and(|h| h.owns_insight(&f.request))
                        }),
                        _ => false,
                    },
                image_diff_shown: self.central_view == CentralView::Diff
                    && crate::image_view::split::active(),
                binary_shown: crate::open_with::view::shown().is_some(),
                has_text_selection: match self.central_view {
                    CentralView::Diff => self
                        .diff_view_state
                        .as_ref()
                        .is_some_and(|s| s.text_sel.is_some()),
                    CentralView::File => self
                        .file_insight
                        .as_ref()
                        .is_some_and(|i| i.text_sel.is_some()),
                    _ => false,
                },
                diff_stages: self
                    .diff_view_state
                    .as_ref()
                    .is_some_and(|s| s.source.staging() == Some(true)),
                in_stageable_diff: self
                    .diff_view_state
                    .as_ref()
                    .is_some_and(|s| s.source.staging().is_some()),
                selected_local_branch: has_ref && ref_kind == Some(RefRowKind::Local),
                selected_remote_branch: has_ref && ref_kind == Some(RefRowKind::Remote),
                selected_tag: has_ref && ref_kind == Some(RefRowKind::Tag),
                ref_marked: self.ref_drop_mark.is_some(),
                selected_tag_is_unpushed: ref_kind == Some(RefRowKind::Tag)
                    && selected_ref
                        .as_deref()
                        .is_some_and(|n| unpushed_tags.contains(n)),
                selected_tag_remote_only: ref_kind == Some(RefRowKind::Tag)
                    && selected_ref.as_deref().is_some_and(|n| {
                        crate::tag_where_ui::lookup(n)
                            .is_some_and(|i| i.place == gitpoulpi_core::domain::TagWhere::Remote)
                    }),
                soloing: !self.solo_refs.is_empty(),
                has_hidden_refs: !self.hidden_from_graph.is_empty(),
                selected_folder: selected_ref_folder(
                    &self.refs_result,
                    &self.refs_filter,
                    self.active_ref_section,
                    self.sidebar_states
                        .state(self.active_ref_section)
                        .selected(),
                    self.hierarchical_refs,
                    &self.collapsed_ref_folders,
                    &self.remote_names,
                )
                .is_some(),
                selected_stash: self.active_ref_section == SidebarSection::Stashes
                    && self
                        .sidebar_states
                        .state(SidebarSection::Stashes)
                        .selected()
                        .is_some_and(|i| i < self.stashes_result.as_deref().unwrap_or(&[]).len()),
                selected_worktree: self.active_ref_section == SidebarSection::Worktrees
                    && linked_worktree_count(self.worktrees_result.as_deref().unwrap_or(&[])) > 0
                    && self
                        .sidebar_states
                        .state(SidebarSection::Worktrees)
                        .selected()
                        .is_some_and(|i| i < self.worktrees_result.as_deref().unwrap_or(&[]).len()),
                selected_submodule: self.active_ref_section == SidebarSection::Submodules
                    && self
                        .sidebar_states
                        .state(SidebarSection::Submodules)
                        .selected()
                        .is_some_and(|i| i < self.submodules_result.len()),
                has_submodules: !self.submodules_result.is_empty(),
                has_refs_filter: !self.refs_filter.is_empty(),
                has_stash: self.stashes_result.as_ref().is_ok_and(|s| !s.is_empty()),
                unstaged_file_selected: selected_status_path_in(
                    &self.status_result,
                    &self.status_states,
                    StatusSection::Unstaged,
                    self.status_list_mode,
                    &self.collapsed_status_folders,
                )
                .is_some(),
                staged_file_selected: selected_status_path_in(
                    &self.status_result,
                    &self.status_states,
                    StatusSection::Staged,
                    self.status_list_mode,
                    &self.collapsed_status_folders,
                )
                .is_some(),
                commit_file_selected: self.commit_file_focus && self.commit_files_len > 0,
                selected_commit: selected_log_commit(
                    &self.log_result,
                    &self.log_state,
                    &self.log_order,
                )
                .is_some(),
                has_wip: has_wip_row(&self.status_result),
                compose_open: self.compose_stash.is_some(),
                compose_file: self
                    .compose_stash
                    .as_ref()
                    .is_some_and(|f| f.file_selected()),
                has_conflicts: !self.conflicts.is_empty(),
                merge_open: self.merge.is_some(),
                editor_open: self.central_view == CentralView::Editor && self.editor.is_some(),
                merge_picks_off: self.merge.as_ref().is_some_and(|m| m.model.is_outside()),
                merge_card: self.merge.as_ref().is_some_and(|m| m.card().is_some()),
                merge_no_blocks: self
                    .merge
                    .as_ref()
                    .is_some_and(|m| m.model.conflict_count() == 0),
                conflicted_files: self.conflicts.len(),
                merge_strip_edited: matches!(
                    self.merge.as_ref().and_then(|m| m.strip()),
                    Some(crate::merge_view::input::Strip::EditedOutside { .. })
                ),
                merge_strip_resolved: matches!(
                    self.merge.as_ref().and_then(|m| m.strip()),
                    Some(crate::merge_view::input::Strip::Resolved)
                ),
                merge_strip_changed: matches!(
                    self.merge.as_ref().and_then(|m| m.strip()),
                    Some(crate::merge_view::input::Strip::Changed)
                ),
                operation_abortable: matches!(
                    self.operation_state,
                    Some(
                        gitpoulpi_core::domain::OperationState::Merge { .. }
                            | gitpoulpi_core::domain::OperationState::CherryPick { .. }
                            | gitpoulpi_core::domain::OperationState::Revert { .. }
                            | gitpoulpi_core::domain::OperationState::Am { .. }
                    )
                ),
                operation_skippable: matches!(
                    self.operation_state,
                    Some(
                        gitpoulpi_core::domain::OperationState::CherryPick { .. }
                            | gitpoulpi_core::domain::OperationState::Revert { .. }
                    )
                ),
                merge_in_progress: matches!(
                    self.operation_state,
                    Some(gitpoulpi_core::domain::OperationState::Merge { .. })
                ),
                conflicted_file_selected: selected_conflicted_path(
                    &self.status_result,
                    &self.status_states,
                    self.status_list_mode,
                    &self.collapsed_status_folders,
                    &self.conflicts,
                )
                .is_some(),
            }
        };
        // Non-blocking: applies whatever the avatar worker has finished
        // fetching/decoding since the last frame.
        self.avatar_service.poll();
        // A terminal zoom (Ctrl +/-) changes the cell's pixel size and
        // arrives as a resize: measure again, so the Graph picture's
        // strokes and avatars are drawn for the real pixels, not for the
        // size seen at startup.
        if let Ok(size) = self.terminal.size() {
            if size != self.zoom_check_size {
                self.zoom_check_size = size;
                if self.graph_image.capability().draws_images() {
                    // Sixel on Windows: the only way to learn the new size.
                    terminal_image::measure_cell_px(self.graph_image.capability().protocol);
                    let cell = terminal_image::cell_size();
                    if cell != self.graph_image.capability().cell {
                        self.graph_image.set_cell(cell);
                        self.commit_graph_image.set_cell(cell);
                        self.avatar_service.set_cell_px(avatar_box_px(cell));
                    }
                }
            }
        }
        if self.avatar_live.dirty {
            self.avatar_live.apply(
                &mut self.avatar_service,
                self.remote_ui
                    .remotes
                    .iter()
                    .map(|r| (r.name.as_str(), r.fetch_url.as_str())),
            );
        }
        // The Preferences screen's background token-presence probe
        // (`spawn_token_presence_probe`): non-blocking, a no-op when Preferences
        // is closed or has no integrations to check.
        if let Some(prefs) = self.preferences.as_mut() {
            prefs.poll_token_presence();
        }
        FrameInputs {
            journal_state,
            undo_busy,
            undo_avail,
            redo_avail,
            unpushed_tags,
            palette_ctx,
        }
    }
}

impl WindowState {
    /// Keeps the Log's hover on the row under the pointer when the rows
    /// change under a resting pointer (a ref or a whole remote hidden or
    /// shown, a refresh, a page, a stash): `hovered_commit` holds a commit
    /// id and is otherwise re-resolved only by a pointer move, so the row
    /// highlight and the tooltip kept describing a commit that is no longer
    /// drawn there. Re-resolves the id at the same cell; an emptied row
    /// drops the hover. Only a hover inside the Log table is touched (the
    /// commit column and the File History list own their own rows).
    fn refresh_log_hover(&mut self) {
        let Some((id, column, x, y)) = self.hovered_commit else {
            return;
        };
        if self.central_view != CentralView::Log || !self.log_area.contains((x, y).into()) {
            return;
        }
        let now = log_row_to_index(self.log_area, self.log_state.offset(), y, self.log_len)
            .and_then(|i| {
                crate::log_table::log_commit_at(
                    &self.log_result,
                    &self.stashes_result,
                    &self.log_order,
                    i,
                )
            });
        match now {
            Some(now) if now == id => {}
            Some(now) => self.hovered_commit = Some((now, column, x, y)),
            None => self.hovered_commit = None,
        }
    }

    /// Opens the diff a Blame action asked for ("Show this commit's diff for
    /// the file") once the commit's changed files are loaded.
    pub(in crate::event_loop) fn tick_blame_diff(&mut self) {
        use crate::blame_view::Ready;
        match crate::blame_view::settle(self.commit_files.as_ref()) {
            Some(Ready::Open {
                commit,
                file_index,
                path,
            }) => open_commit_file(
                CommitFileOpen::Diff { file_index, path },
                commit,
                &self.link,
                &mut self.file_insight,
                &mut self.diff_view_state,
                &mut self.central_view,
                None,
            ),
            Some(Ready::Refused(why)) => self.last_message = Some(why.get().into()),
            None => {}
        }
    }
}
