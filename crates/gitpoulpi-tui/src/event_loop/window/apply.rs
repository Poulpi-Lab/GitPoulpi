//! Applying what the daemon link delivered: appended Log pages, a stopped
//! link, optimistic staging and the refresh outcome that replaces the
//! results the panes draw from.

use crate::event_loop::*;
use init::WindowState;

/// How long a status message is immune to the refresh that clears it.
pub(super) const MESSAGE_MIN_LIFETIME: Duration = Duration::from_secs(3);

impl WindowState {
    /// Starts the age of `last_message` when it differs from the one seen
    /// last (called once per frame by `tick`, and before a refresh is applied
    /// so a message set this iteration is already counted as new).
    pub(in crate::event_loop) fn note_message(&mut self) {
        if self.last_message != self.message_seen {
            self.message_seen = self.last_message.clone();
            self.message_set_at = Instant::now();
        }
    }

    /// The management tab's dialogs and rows, then the quit check. Returns
    /// `true` when the window should end.
    pub(in crate::event_loop) fn after_input(&mut self) -> bool {
        // The management tab's dialogs: folder listings for the picker,
        // and a finished `git init` / clone (which opens its repository as
        // a tab and moves the window to it). Drained before the quit
        // check so a final report is never lost to a keystroke in the
        // same iteration. `try_recv` only -- nothing here waits.
        for effect in self.repo_dialogs.poll() {
            input::menu_actions::apply_effect(&mut self.command_ctx(), effect);
        }
        if self.switch_to.is_some() {
            self.should_quit = true;
        }
        // "Restart to update" (or a revert) swapped the files: leave, and
        // `run_with` starts the new binary once the terminal is restored.
        if crate::updates::auto::take_quit_request() {
            self.should_quit = true;
            self.quit_how = Some("restart to update");
        }

        // The management tab's rows, filled in as its probe answers.
        if let (Some(probe), Some(manager)) = (&self.management_probe, self.management.as_mut()) {
            for (path, facts) in probe.drain() {
                manager.learn(&path, facts);
            }
        }
        // A workspace changed in this process: the Home page reloads.
        crate::workspace_ui::apply::sync_management(&self.session, self.management.as_mut());
        // The Radar's workers (hosting answers, WIP facts).
        if let Some(manager) = self.management.as_mut() {
            // Turned off in the settings while showing: the view and its
            // worker go away at once (turning it on again reopens nothing).
            if !crate::build_features::radar_enabled() {
                manager.radar.close();
            }
            manager.radar.poll();
            if let Some(said) = manager.radar.take_said() {
                self.last_message = Some(said);
            }
        }
        // Leaving for the management tab ends this window.
        if self.show_management.is_some() {
            self.should_quit = true;
        }

        // Leaving this window -- quitting, another tab,
        // the Repository Management tab -- would drop the editor with
        // it. With unsaved changes, stay and ask Save / Discard / Cancel
        // instead; the user leaves again once that is answered.
        if self.should_quit {
            if let Some(ed) = self.editor.as_mut().filter(|e| e.doc.is_dirty()) {
                self.should_quit = false;
                self.switch_to = None;
                self.close_window = false;
                self.show_management = None;
                self.quit_how = None;
                ed.prompt = true;
                self.central_view = CentralView::Editor;
                self.last_message = Some(gitpoulpi_i18n::msg::window::unsaved_changes(
                    ed.path.display(),
                ));
            }
        }
        // The same for the merge editor's unsaved picks and typing.
        if self.should_quit {
            if let Some(view) = self.merge.as_mut().filter(|v| v.model.is_dirty()) {
                self.should_quit = false;
                self.switch_to = None;
                self.close_window = false;
                self.show_management = None;
                self.quit_how = None;
                view.ask_close();
                self.central_view = CentralView::Merge;
                self.last_message = Some(gitpoulpi_i18n::msg::merge::msg_unsaved_leave(
                    view.path().display(),
                ));
            }
        }
        if self.should_quit {
            return true;
        }
        false
    }

    /// Applies every outcome the link has queued.
    pub(in crate::event_loop) fn apply_outcomes(&mut self) {
        self.apply_log_pages_and_link_state();
        let refreshed = self.apply_refresh_outcome();
        self.flow_tick();
        self.apply_notices();
        self.apply_commit_settings(refreshed);
        self.apply_merge_replies();
        self.apply_detail_reads(refreshed);
        self.apply_pending_split_select();
    }

    /// The AI's answer to "Split commit with AI...": the editor, prefilled
    /// with its plan, or (the AI failed) the plain editor with the reason.
    pub(in crate::event_loop) fn open_ai_split(
        &mut self,
        source: gitpoulpi_core::domain::SplitSource,
        plan: Result<Vec<gitpoulpi_core::domain::SplitPart>, String>,
    ) {
        use gitpoulpi_i18n::msg::split_ui as m;
        if !matches!(self.input_mode, InputMode::Normal) {
            // Never replace what the user is typing into.
            self.last_message = Some(m::AI_DIALOG_OPEN.get().into());
            return;
        }
        let form = match plan {
            Ok(parts) => {
                self.last_message = Some(m::AI_PLAN_READY.get().into());
                SplitForm::with_parts(source, parts)
            }
            Err(why) => {
                crate::activity_log::report_problem(
                    crate::activity_log::ProblemSource::Ai,
                    &gitpoulpi_i18n::msg::window::ai_failed(&why),
                );
                let note = m::ai_failed(&why);
                self.last_message = Some(note.clone());
                let mut form = SplitForm::new(source);
                form.note(note);
                form
            }
        };
        self.input_mode = InputMode::SplitCommit(Box::new(form));
    }

    /// After a split: selects its newest commit once a refresh lists it.
    fn apply_pending_split_select(&mut self) {
        let Some((oid, since, back)) = self.pending_split_select else {
            return;
        };
        if select_log_commit(&self.log_result, &self.log_order, &mut self.log_state, oid)
            || since.elapsed() > Duration::from_secs(5)
        {
            self.pending_split_select = None;
            // Split from the rebase popup: back to it, on the new history.
            if let Some(upstream) = back {
                if matches!(self.input_mode, InputMode::Normal) {
                    if let Ok(form) = rebase_form_for(&self.log_result, &self.refs_result, upstream)
                    {
                        self.input_mode = InputMode::InteractiveRebase(Box::new(form));
                    }
                }
            }
        }
    }

    fn apply_log_pages_and_link_state(&mut self) {
        // A lazily-loaded page of older history. **Appended, never replaced**:
        // the rows already on screen keep their indices (and their lanes and
        // colors: the lane assignment is a fold over the node list), so the
        // selection and scroll offset survive by construction. Pinned by
        // `appending_a_page_does_not_disturb_the_rows_already_laid_out` in
        // `graph/tests.rs`.
        // The Log's visible rows, and the commit column's when it shows the
        // Changes part.
        let mut visible = self.visible_commit_ids.clone();
        visible.extend(self.commit_col.visible_ids.iter().copied());
        request_visible_changes(
            &self.link,
            &mut self.commit_changes,
            &mut self.changes_asked,
            &visible,
        );
        // The numbered changes of a commit asked for by "Split commit...":
        // the editor opens on them.
        while let Ok(answer) = self.link.split_source_rx.try_recv() {
            match answer.result {
                Ok(source) => {
                    let back = crate::split_ui::take_return(answer.commit);
                    let proposed = match self.pending_split_proposal.take() {
                        Some((oid, parts)) if oid == source.commit => Some(parts),
                        other => {
                            self.pending_split_proposal = other;
                            None
                        }
                    };
                    let mut form = match proposed {
                        Some(parts) => {
                            let form = SplitForm::with_parts(source, parts);
                            crate::ui_intent::proposal::set_proposed(
                                &crate::ui_intent::proposal::split_text(&form.build_parts()),
                            );
                            form
                        }
                        None => SplitForm::new(source),
                    };
                    if let Some(upstream) = back {
                        form = form.returning_to_rebase(upstream);
                    }
                    self.input_mode = InputMode::SplitCommit(Box::new(form));
                }
                Err(e) => {
                    let text = crate::daemon_link::error_text::error_text(&e);
                    if matches!(&self.pending_split_proposal, Some((oid, _)) if *oid == answer.commit)
                    {
                        // The proposal could not open: it failed.
                        self.pending_split_proposal = None;
                        if let Some(active) = crate::ui_intent::proposal::close() {
                            crate::ui_intent::proposal::settle(
                                &active.id,
                                gitpoulpi_ipc::ui::ProposalStatus::Failed,
                            );
                        }
                    }
                    self.last_message = Some(gitpoulpi_i18n::msg::window::error_line(&text));
                    // Asked from the rebase popup: back to it.
                    if let Some(upstream) = crate::split_ui::take_return(answer.commit) {
                        if let Ok(form) =
                            rebase_form_for(&self.log_result, &self.refs_result, upstream)
                        {
                            self.input_mode = InputMode::InteractiveRebase(Box::new(form));
                        }
                    }
                }
            }
        }
        while let Ok(page) = self.link.log_page_rx.try_recv() {
            // A page of another walk (the hidden-refs filter moved since it
            // was asked for) must not reach `on_page`, which would append it
            // at an offset that means something else now.
            if page.log_filter_gen != self.link.log_filter_generation() {
                continue;
            }
            let Ok(graph) = page.result else { continue };
            // A page can bring in the base of a stash that was waiting for
            // it, and that stash's row is placed at its date -- possibly
            // above the selection. Keep the selection on its commit.
            let kept = selected_log_commit(&self.log_result, &self.log_state, &self.log_order);
            if let Ok(held) = self.log_result.as_mut() {
                if self
                    .lazy_log
                    .on_page(page.request, graph.nodes, &mut held.nodes)
                {
                    self.log_gen += 1;
                    self.read_hashes.forget_log();
                    // Tell the periodic refresh to read this deep from
                    // now on, or the next two-second poll would truncate
                    // the log back to the initial page under a user who
                    // has just scrolled into what they waited for.
                    self.link.set_log_depth(self.lazy_log.loaded());
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
                    // The list only got longer; a stash row that just found
                    // its base is the one thing that can move a commit.
                    if let Some(commit) = kept {
                        select_log_commit(
                            &self.log_result,
                            &self.log_order,
                            &mut self.log_state,
                            commit,
                        );
                    }
                    move_table_selection(&mut self.log_state, self.log_len, 0);
                    // `log_follow` is deliberately left alone: forcing it
                    // `true` made ratatui snap the offset back to the
                    // (off-screen, wheel-scrolled-away-from) selection,
                    // i.e. the view jumped to the top when more commits
                    // arrived.
                }
            }
        }

        // The link stopped -- the daemon could not be reached, the
        // connection dropped, or the link thread died. Said once in the
        // footer, and by the panes for as long as nothing real is loaded
        // (`pane_notice`); `r` then starts a fresh link, which starts a
        // fresh embedded daemon too if the old one is gone.
        while let Ok(reason) = self.link.failure_rx.try_recv() {
            self.last_message = Some(gitpoulpi_i18n::msg::window::link_lost(&reason));
            self.link_failure = Some(reason);
        }
        if std::mem::take(&mut self.retry_link_requested) {
            self.link = daemon_link::start(self.session.to_string(), self.repo.clone());
            self.read_hashes = daemon_link::ReadHashes::default();
            self.changes_asked.clear();
            self.link.set_log_depth(self.lazy_log.loaded());
            self.link_failure = None;
            self.still_loading = self.log_result.is_err();
            self.repo_open_started_at = Instant::now();
            self.last_message = Some(gitpoulpi_i18n::msg::window::RECONNECTING.get().to_string());
        }

        // Optimistic staging: move the rows now, before the daemon has
        // answered; the scoped refresh that follows its answer replaces
        // this copy, and an error puts `optimistic_backup` back.
        let optimistic = self.link.take_optimistic();
        if !optimistic.is_empty() {
            if let Ok(tree) = self.status_result.as_mut() {
                let before = tree.clone();
                let mut moved = false;
                for action in &optimistic {
                    moved |= status_optimistic::apply(tree, action);
                }
                if moved {
                    self.optimistic_backup.get_or_insert(before);
                    for section in StatusSection::ORDER {
                        let len = build_status_section_lines(
                            tree,
                            section,
                            self.status_list_mode,
                            &self.collapsed_status_folders,
                            "",
                        )
                        .len();
                        move_selection(self.status_states.state_mut(section), len, 0);
                    }
                }
            }
        }
    }

    /// Returns whether a refresh outcome landed.
    fn apply_refresh_outcome(&mut self) -> bool {
        self.note_message();
        let mut refreshed = false;
        while let Ok(outcome) = self.link.outcome_rx.try_recv() {
            refreshed = true;
            // The daemon round trip landed -- whatever it carries (even
            // an `Err`) is a real answer now, not the client-side
            // placeholder. Every later `Err` in these five is shown as
            // the error it is.
            self.still_loading = false;
            // The commit the Log selection is on, taken before anything
            // is replaced: rows above it can come and go (a stash row) and
            // the selection is a row index, so it is put back on its commit
            // below instead of sliding to a neighbour.
            let kept_commit =
                selected_log_commit(&self.log_result, &self.log_state, &self.log_order);
            let wip_before = self.log_order.has_wip();
            // A scoped refresh (a stage, say) re-read only what its
            // `ChangeScope` names; the other results are placeholders and
            // the previous ones stay. Status is always re-read.
            let scope = outcome.scope;
            // ...except that a status read from before a mutation the user
            // has since submitted would undo the optimistic rows: it is
            // kept out, and the refresh that follows the mutation brings
            // the real one (a status we do not have yet is still taken).
            if outcome.generation == self.link.generation() || self.status_result.is_err() {
                self.status_result = outcome.status_result;
                self.optimistic_backup = None;
            }
            // Who this repository signs in as (the push menu and the
            // Activity Log read it; nothing is spawned for it).
            if let Ok(tree) = &self.status_result {
                crate::activity_log::identity::note(&self.repo.root, tree.repo_auth.as_ref());
            }
            // A read whose content is what the window already holds is not
            // replaced and its generation is not bumped, so an idle poll
            // leaves the Log layout cache (keyed on the generations) alone.
            // A Log read under another hidden-refs filter than the one in
            // use is a walk of other tips: not taken, nor its hash recorded
            // (the refresh the filter change asked for brings the right one).
            let changed =
                self.read_hashes
                    .apply(outcome.read_hashes.for_filter_generation(
                        outcome.log_filter_gen,
                        self.link.log_filter_generation(),
                    ));
            if changed.log {
                self.log_result = outcome.log_result;
                self.log_gen += 1;
            }
            if changed.refs {
                match super::refs_update::refs_update(&self.refs_result, outcome.refs_result) {
                    super::refs_update::RefsUpdate::Keep { why } => {
                        self.last_message =
                            Some(gitpoulpi_i18n::msg::window::refs_refresh_failed(&why));
                    }
                    super::refs_update::RefsUpdate::Replace(result) => {
                        self.refs_result = result;
                        self.refs_gen += 1;
                    }
                }
            }
            if changed.stashes {
                self.stashes_result = outcome.stashes_result;
                self.stashes_gen += 1;
            }
            if scope.intersects(gitpoulpi_ipc::ChangeScope::WORKTREES) {
                self.worktrees_result = outcome.worktrees_result;
            }
            if scope.intersects(crate::daemon_link::SUBMODULES_SCOPE) {
                if let Ok(list) = outcome.submodules_result {
                    self.submodules_result = list;
                }
            }
            if scope.intersects(gitpoulpi_ipc::ChangeScope::CONFLICTS) {
                let had_conflicts = !self.conflicts.is_empty();
                self.conflicts = outcome.conflicts_result.unwrap_or_default();
                self.rebase_state = outcome.rebase_state_result.unwrap_or_default();
                self.operation_state = outcome.operation_state_result.unwrap_or_default();
                self.index_lock = outcome.index_lock_result.unwrap_or_default();
                self.after_conflicts_changed();
                if !had_conflicts && !self.conflicts.is_empty() && self.operation_state.is_none() {
                    self.reveal_stray_conflicts();
                }
            }
            self.lfs.apply(outcome.lfs);
            self.lfs.enabled = lfs_ui::resolved_enabled(&self.repo.root);
            // A failed `remotes` read leaves the previous list in place
            // rather than emptying it: an empty list is not "no remotes",
            // it is "we could not ask", and acting on it would re-split
            // every remote branch on its first `/` for one frame.
            if let Ok(remotes) = outcome.remotes_result {
                self.remote_names = remotes.iter().map(|r| r.name.clone()).collect();
                // A removed remote takes its hide with it.
                if !crate::refs_tree::drop_gone_remotes(
                    &mut self.hidden_from_graph,
                    &self.remote_names,
                )
                .is_empty()
                {
                    refs_tree::submit_ref_visibility(
                        &self.link,
                        &self.hidden_from_graph,
                        &self.solo_refs,
                    );
                }
                self.remote_ui.remotes = remotes;
                self.remote_ui.loaded = true;
                // Where the avatar worker may ask GitHub which account an
                // author is: `origin` when it is on github.com, else the
                // first remote that is (`avatar::github`).
                self.avatar_service
                    .set_github_repo(avatar::pick_github_repo(
                        self.remote_ui
                            .remotes
                            .iter()
                            .map(|r| (r.name.as_str(), r.fetch_url.as_str())),
                    ));
                // ...and the connected GitLab host, if the remote is one.
                self.avatar_live.dirty = true;
            }
            if let Ok(pin) = outcome.ssh_command_result {
                self.remote_ui.ssh_pin = pin;
            }
            if let Ok(state) = outcome.sparse_result {
                self.repo_dialogs.set_sparse(&state);
                self.sparse = state;
            }
            if let Ok(values) = outcome.repo_config_result {
                self.repo_config = values;
                self.repo_config_version += 1;
            }
            if scope.intersects(gitpoulpi_ipc::ChangeScope::REFS) {
                if let Ok(refs) = &self.refs_result {
                    self.hosting
                        .on_refresh(Instant::now(), &self.remote_ui.remotes, refs);
                }
            }
            // A refresh replaces the whole node list, so it also resets
            // what the lazy loader believes is loaded -- and a refresh
            // that came back shorter than it asked for is how "history is
            // fully walked" is discovered in the first place.
            if changed.log {
                self.lazy_log.on_refresh_asked(
                    self.log_result.as_ref().map(|g| g.nodes.len()).unwrap_or(0),
                    outcome.log_limit,
                );
            }
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
            // (The WIP row appearing or going is left to the index: the
            // selection then lands on it, or on the top commit, as before.)
            if let Some(commit) = kept_commit.filter(|_| self.log_order.has_wip() == wip_before) {
                select_log_commit(
                    &self.log_result,
                    &self.log_order,
                    &mut self.log_state,
                    commit,
                );
            }
            move_table_selection(&mut self.log_state, self.log_len, 0);
            // Not `log_follow = true`: a refresh must not yank a
            // wheel-scrolled view back to the selection either.
            // Re-clamp the worktree/stash selections against the list
            // that just arrived: popping or dropping a stash shortens the
            // list under a selection that may now point past its end
            // (`move_selection(.., 0)` clamps rather than moving), and an
            // emptied section must not keep a phantom selection that
            // would reappear the next time it becomes non-empty.
            move_selection(
                &mut self.sidebar_states.worktrees,
                self.worktrees_result.as_deref().unwrap_or(&[]).len(),
                0,
            );
            move_selection(
                &mut self.sidebar_states.stashes,
                self.stashes_result.as_deref().unwrap_or(&[]).len(),
                0,
            );
            move_selection(
                &mut self.sidebar_states.submodules,
                self.submodules_result.len(),
                0,
            );
            // A section that just disappeared (last stash dropped) can't
            // stay active, or the sidebar would have focus on nothing.
            let still_visible = visible_sections_with(
                linked_worktree_count(self.worktrees_result.as_deref().unwrap_or(&[])),
                self.stashes_result.as_deref().unwrap_or(&[]).len(),
                self.hosting.sections_visible(),
                self.submodules_result.len(),
                &self.hidden_sidebar_sections,
            );
            if !still_visible.contains(&self.active_ref_section) {
                self.active_ref_section = still_visible
                    .first()
                    .copied()
                    .unwrap_or(SidebarSection::Local);
            }
            // Each sub-pane's selection is clamped against ITS OWN row
            // count, recomputed exactly the way the draw pass builds it.
            // Otherwise staging a file -- which shortens one half and
            // lengthens the other -- leaves a selection pointing past the
            // end of the list it belongs to.
            if let Ok(tree) = &self.status_result {
                for section in StatusSection::ORDER {
                    let len = build_status_section_lines(
                        tree,
                        section,
                        self.status_list_mode,
                        &self.collapsed_status_folders,
                        "",
                    )
                    .len();
                    move_selection(self.status_states.state_mut(section), len, 0);
                }
            }
            if self.refs_result.is_ok() {
                // Clamp against the rows as displayed (a tree has folder and
                // remote rows besides the branches), not the branch count: a
                // selection on a deeper row of the REMOTE tree was pulled up
                // to the branch count by every refresh, the first load's
                // follow-up outcome included.
                for (section, state) in [
                    (SidebarSection::Local, &mut self.sidebar_states.local),
                    (SidebarSection::Remote, &mut self.sidebar_states.remote),
                    (SidebarSection::Tags, &mut self.sidebar_states.tags),
                ] {
                    let len = ref_section_row_count(
                        &self.refs_result,
                        &self.refs_filter,
                        section,
                        self.hierarchical_refs,
                        &self.collapsed_ref_folders,
                        &self.remote_names,
                        0,
                        0,
                        (0, 0),
                        0,
                    );
                    move_selection(state, len, 0);
                }
            }
            // The follow-up half of the first load (worktrees, LFS) arrives
            // on its own a moment after the first paint; it must not wipe a
            // message the user's first keystrokes just produced. The very
            // first outcome keeps it too: it is the open's own load, and
            // anything on the status bar then is the user's, typed or
            // clicked while the repository was still loading.
            // A refresh ends a message that has been showing a while (an
            // "auto-fetching..." progress line, a stale error), but never one
            // that has only just appeared: a refresh already in flight when
            // the message was set carries nothing newer to say.
            if self.open_first_outcome_logged
                && scope != daemon_link::FIRST_LOAD_LATER
                && self.message_set_at.elapsed() >= MESSAGE_MIN_LIFETIME
            {
                self.last_message = None;
            }
            if !self.open_first_outcome_logged {
                self.open_first_outcome_logged = true;
                open_trace("first refresh applied", self.open_t0);
            }
            // (Copied: the tab context borrows the whole window.)
            let worktrees = self.worktrees_result.as_deref().unwrap_or(&[]).to_vec();
            // Create worktree's "open it as a tab": once the refresh
            // lists it, which is the proof the daemon made it. The
            // `switch_to` this sets ends the window at the next
            // iteration's quit check, like every other tab switch.
            if let Some(message) = open_pending_worktree_tab(&worktrees, &mut self.tab_ctx()) {
                self.last_message = Some(message);
            }
            // A worktree that was removed (or deleted from disk) takes
            // its tab with it.
            if let Some(message) = close_stale_worktree_tabs(&worktrees, &mut self.tab_ctx()) {
                self.last_message = Some(message);
            }
        }

        refreshed
    }
}
