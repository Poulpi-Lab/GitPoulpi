//! Messages and requests the link sends back outside a refresh: LFS state,
//! action errors and messages, dialog seeds, push rejections.

use crate::event_loop::*;
use init::WindowState;

impl WindowState {
    pub(in crate::event_loop) fn apply_notices(&mut self) {
        // The push menu's identity row or the tab menu asked for the SSH &
        // credentials page.
        if std::mem::take(&mut self.remote_ui.want_repo_auth) {
            let mut state = crate::preferences::PreferencesState::open(&self.repo.root);
            state.open_repo_auth_page();
            self.preferences = Some(state);
        }
        // The Git & Encoding page's LFS rows: what the program is, and the
        // "Initialize LFS" button's request.
        if let Some(prefs) = self.preferences.as_mut() {
            prefs.set_lfs_program(self.lfs.status.as_ref().map(|s| {
                match &s.binary {
                    gitpoulpi_core::domain::LfsBinary::Installed { version } => {
                        gitpoulpi_i18n::msg::window::lfs_program_installed(version)
                    }
                    gitpoulpi_core::domain::LfsBinary::TooOld { version } => {
                        gitpoulpi_i18n::msg::window::lfs_program_too_old(version)
                    }
                    gitpoulpi_core::domain::LfsBinary::Missing => {
                        gitpoulpi_i18n::msg::window::LFS_PROGRAM_MISSING
                            .get()
                            .to_string()
                    }
                }
            }));
            if prefs.take_lfs_init_request() {
                self.link.submit_action(Action::LfsInstall);
            }
            // The Sparse checkout and LFS pages: the state of the last
            // refresh in, a button press out (as the same tracked actions the
            // Sparse checkout dialog runs).
            prefs.set_sparse(&self.sparse);
            prefs.set_lfs_status(self.lfs.status.as_ref());
            prefs.set_in_flight(self.link.in_flight_text());
            // The SSH & credentials page talks to the daemon through the
            // link's request queue, never on this thread.
            prefs.set_repo_auth_requester(self.link.activity_handle().raw.requester());
            prefs.poll_repo_auth();
            match prefs.take_sparse_request() {
                Some(crate::preferences::SparseRequest::Set(dirs)) => {
                    self.link.submit_action(Action::SparseSet { dirs });
                }
                Some(crate::preferences::SparseRequest::Disable) => {
                    self.link.submit_action(Action::SparseDisable);
                }
                None => {}
            }
            // The page shows git's own values; a pick goes to the daemon
            // (journaled, so Undo restores the old one).
            prefs.set_repo_config(&self.repo_config, self.repo_config_version);
            prefs.set_remote_names(&self.remote_names);
            for (key, value) in prefs.take_repo_config_requests() {
                self.link
                    .submit_action(Action::SetRepoConfig { key, value });
            }
        }
        // A running LFS transfer's progress on the status line (the
        // action's own "fetched ..." message replaces it when it ends).
        if let Some((operation, progress)) = self.link.lfs_progress() {
            if !progress.done {
                self.last_message = Some(lfs_ui::progress_text(operation, &progress));
            }
        }
        // The daemon posed the repository's LFS filters before a pull or
        // push (`git.lfs_autofix_filters`): say so now; the completion
        // message below carries the note too.
        if self.link.take_lfs_filters_posed() {
            self.last_message = Some(gitpoulpi_i18n::msg::lfs_ui::FILTERS_POSED.get().to_string());
        }
        if let Some(said) = crate::open_with::take_notice() {
            self.last_message = Some(said);
        }
        // A Cancel the daemon refused (or a job it no longer knows).
        if let Some(said) = crate::jobs_ui::take_notice() {
            self.last_message = Some(said);
        }
        for late in external_tools::take_late_errors() {
            crate::activity_log::report_problem(crate::activity_log::ProblemSource::Tool, &late);
        }
        while let Ok(err) = self.link.action_error_rx.try_recv() {
            // An error that names the git-lfs install page is the "not
            // installed" one (`LfsUnavailableReason::message`): offer to
            // open the page instead of leaving the link in a status line.
            if err.contains(gitpoulpi_core::domain::LFS_INSTALL_URL) {
                self.input_mode = InputMode::Confirm {
                    prompt: err.clone(),
                    action: ConfirmAction::OpenLfsInstallPage,
                };
            }
            let line = gitpoulpi_i18n::msg::window::error_line(&err);
            // A reported failure: this line is clickable (its entry).
            self.last_message = Some(crate::activity_log::status_error::claim(line));
            // The split editor waits for the answer: a refusal keeps it open
            // on the same plan with the reason.
            if let InputMode::SplitCommit(form) = &mut self.input_mode {
                if form.is_applying() {
                    form.refused(err.clone());
                }
            }
            // The daemon refused: show the status as it was (the link has
            // already asked for the real one).
            if let Some(before) = self.optimistic_backup.take() {
                self.status_result = Ok(before);
            }
            // A failed worktree add never appears in a refresh; stop
            // waiting for it (a failure of another action only means the
            // tab is not opened automatically -- never a wrong one).
            self.pending_worktree_tab = None;
        }
        // A failure that was reported (a clone, a hosting call, an external
        // program, an AI request, a save, an action of another tab...) and
        // that the link's error channel did not carry: the status line says
        // it too, whichever tab or view is in front.
        if let Some(line) = crate::activity_log::status_error::take_pending() {
            self.last_message = Some(line);
        }
        // A Create worktree menu row or palette command asked for the
        // dialog (`WorktreeSeed`); it is `repo_dialogs`' to show.
        if let Some(seed) = self.worktree_seed.take() {
            let worktrees = self.worktrees_result.as_deref().unwrap_or(&[]);
            let refs = self.refs_result.as_deref().unwrap_or(&[]);
            let branches: Vec<crate::worktree_dialog::BranchChoice> = refs
                .iter()
                .filter_map(|r| match &r.kind {
                    gitpoulpi_core::domain::RefKind::LocalBranch { name } => {
                        Some(crate::worktree_dialog::BranchChoice {
                            name: name.clone(),
                            busy: r.is_head
                                || worktrees.iter().any(|w| w.branch.as_deref() == Some(name)),
                        })
                    }
                    _ => None,
                })
                .collect();
            self.repo_dialogs.open_worktree_with(
                seed,
                &self.repo.root,
                branches,
                crate::worktree_dialog::WorktreeDialog::take_pick_ref(),
            );
        }
        // A worktree row's Remove / Merge into...: the dialog needs this
        // window's refs, worktrees and daemon link.
        #[cfg(feature = "worktree")]
        if let Some(request) = crate::worktree_remove_dialog::take_request() {
            use crate::worktree_remove_dialog::{Mode, Request, Seed};
            let (path, mode) = match request {
                Request::Remove(p) => (p, Mode::Remove),
                Request::Merge(p) => (p, Mode::Merge),
            };
            let worktrees = self.worktrees_result.as_deref().unwrap_or(&[]).to_vec();
            let canonical =
                |p: &std::path::Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
            match worktrees
                .iter()
                .find(|w| canonical(&w.path) == canonical(&path))
                .cloned()
            {
                Some(worktree) => {
                    let refs = self.refs_result.as_deref().unwrap_or(&[]).to_vec();
                    let requester = self.link.activity_handle().raw.requester();
                    self.repo_dialogs.open_worktree_remove(
                        Seed {
                            worktree,
                            worktrees,
                            refs,
                            mode,
                        },
                        &requester,
                    );
                }
                None => {
                    self.last_message = Some(gitpoulpi_i18n::msg::worktree_manage::not_listed(
                        path.display(),
                    ));
                }
            }
        }
        // Start a pull request asked for the form dialog (a matched
        // integration had a usable token); with no token the action
        // already opened a browser URL directly and never sets this.
        #[cfg(feature = "hosting")]
        if let Some(seed) = self.pull_request_seed.take() {
            let tip_subject = self
                .log_result
                .as_ref()
                .ok()
                .and_then(|g| g.nodes.first())
                .map(|n| n.summary.clone())
                .unwrap_or_default();
            self.repo_dialogs
                .open_pull_request(seed, &tip_subject, self.ai_ui.enabled());
        }
        // A ref menu's "Reflog..." asked for the Undo history panel.
        if let Some((tab, refname)) = crate::safety_panel::take_open_request() {
            self.repo_dialogs.open_safety(&self.link, tab, refname);
        }
        // A Create patch menu row or palette command asked for the
        // dialog (`PatchSeed`); it is `repo_dialogs`' to show.
        if let Some(seed) = self.patch_seed.take() {
            self.repo_dialogs.open_save_patch(seed, &self.repo.root);
        }
        // A submodule's "Open in new tab".
        if let Some(path) = self.submodule_open.take() {
            if let Some(message) = self
                .tab_ctx()
                .switcher_action(repo_switcher::SwitcherAction::Open(path))
            {
                self.last_message = Some(message);
            }
        }
        while let Ok(msg) = self.link.action_message_rx.try_recv() {
            crate::activity_log::note_message(&msg, &self.repo.root);
            // The split went through: the editor has nothing left to show.
            if let InputMode::SplitCommit(form) = &self.input_mode {
                if form.is_applying() {
                    let back = form.return_to();
                    self.input_mode = InputMode::Normal;
                    // The Log selects the newest commit the split made (and
                    // the rebase popup it came from reopens).
                    if let Some(newest) = crate::daemon_link::take_split_created(&self.repo.root) {
                        self.pending_split_select = Some((newest, Instant::now(), back));
                    }
                }
            }
            self.last_message = Some(if self.link.take_lfs_filters_posed_note() {
                lfs_ui::with_posed_note(msg)
            } else {
                msg
            });
        }
        // A push the remote refused because it moved on: a
        // dialog (pull first / force push with lease / cancel), hung
        // under the Push button that started it.
        // Something outside GitPoulpi (a shell, an AI agent) changed the
        // repository and it is now the newest entry of the undo history:
        // one line, only while the window is the one in front.
        if self.link.take_outside_news() && !self.ui_viewer.unfocused() {
            use gitpoulpi_i18n::msg::undo_safety as s;
            let key = crate::keymap::current().shown(crate::keymap::Action::GlobalUndo);
            self.last_message = Some(if key.is_empty() {
                s::OUTSIDE_RECORDED_NO_KEY.get().to_string()
            } else {
                s::outside_recorded(key)
            });
        }
        while let Ok(crate::daemon_link::PushRejectedNotice { rejection, ticket }) =
            self.link.push_rejected_rx.try_recv()
        {
            // A branch with extra targets answers a refused push in the push
            // group dialog (every remote at once, one decision each); the
            // single-remote menu stays for a branch without targets.
            let grouped = matches!(
                rejection.reason,
                gitpoulpi_core::domain::PushRejectReason::NonFastForward
                    | gitpoulpi_core::domain::PushRejectReason::StaleLease
            ) && self.refs_result.as_ref().ok().is_some_and(|refs| {
                crate::remotes_ui::branch_group(refs, &rejection.branch)
                    .is_some_and(|g| g.extra_push)
            });
            if grouped {
                crate::push_group_ui::PushGroupUi::open_plan(
                    &self.repo.root,
                    &rejection.branch,
                    None,
                );
                let line = gitpoulpi_i18n::msg::remotes::diverged_title(
                    &rejection.remote,
                    &rejection.branch,
                );
                // The group dialog decides per remote, outside any one
                // ticket: a tracked push ends here as refused.
                crate::daemon_link::report_ticket(
                    ticket,
                    crate::daemon_link::StepOutcome::Failed { text: line.clone() },
                );
                self.last_message = Some(line);
                continue;
            }
            let (col, row) = self.remote_ui.push_anchor();
            self.last_message = Some(
                self.remote_ui
                    .on_push_rejected(&rejection, ticket, col, row),
            );
        }

        // A checkout/merge/rebase/pull blocked by local changes while
        // `general.auto_stash` is `ask`: its dialog (stash and continue).
        while let Ok(needed) = self.link.autostash_rx.try_recv() {
            let (col, row) = self.remote_ui.push_anchor();
            self.last_message = Some(self.remote_ui.on_autostash_needed(needed, col, row));
        }
        // A held ticket whose dialog was closed without an answer ends here.
        let confirming = match &self.input_mode {
            InputMode::Confirm {
                action:
                    crate::modes::ConfirmAction::Destructive(
                        crate::modes::PendingDestructive::ForcePush { ticket, .. },
                    ),
                ..
            } => *ticket,
            _ => None,
        };
        self.remote_ui.reap_ticket(confirming);
    }
}
