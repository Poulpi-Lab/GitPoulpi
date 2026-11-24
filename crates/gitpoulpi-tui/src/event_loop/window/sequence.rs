//! The running sequence, one frame at a time: the window's half of the
//! runner (`sequence_run`).
//!
//! Each frame the window takes the run out of its process-wide slot, turns
//! what it sees (the cancel and continue requests, the ticket outcomes, the
//! AI's end, a closed dialog, the clock) into [`RunEvent`]s, and carries out
//! the [`RunEffect`]s the machine answers with. Executing a step is the one
//! place that needs the command layer: the command runs on this window's
//! [`CommandCtx`] with the step's ticket *armed*, so the action it submits
//! reports under that ticket (`daemon_link::tickets::arm`).

use super::*;
use crate::daemon_link as tickets;
use crate::daemon_link::Action;
use crate::sequence_run::exec::{self, Plan};
use crate::sequence_run::machine::{
    DialogKind, ExecResult, PauseKind, Run, RunEffect, RunEvent, StepOptions, StepState,
};
use gitpoulpi_i18n::msg::sequence_run as m;
use std::collections::VecDeque;

impl WindowState {
    /// The local branches, each flagged when it is checked out somewhere (the
    /// Create worktree dialog's Ref and branch lists).
    pub(in crate::event_loop) fn worktree_branch_choices(
        &self,
    ) -> Vec<crate::worktree_dialog::BranchChoice> {
        let worktrees = self.worktrees_result.as_deref().unwrap_or(&[]);
        let refs = self.refs_result.as_deref().unwrap_or(&[]);
        refs.iter()
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
            .collect()
    }

    /// Once a frame: moves the running sequence along.
    pub(in crate::event_loop) fn sequence_tick(&mut self, palette_ctx: PaletteContext) {
        if !crate::sequence_run::visible() {
            return;
        }
        let requests = crate::sequence_run::take_requests();
        let Some(mut run) = crate::sequence_run::take() else {
            return;
        };
        let mut queue: VecDeque<RunEvent> = VecDeque::new();
        if requests.cancel {
            queue.push_back(RunEvent::Cancel);
        }
        if requests.cont {
            queue.push_back(RunEvent::Continue);
        }
        if requests.go_there {
            let target = run.ctx.target_root.clone();
            if let Some(message) = self
                .tab_ctx()
                .switcher_action(repo_switcher::SwitcherAction::Open(target))
            {
                self.last_message = Some(message);
            }
        }
        if !run.is_over() {
            self.gather_events(&run, &mut queue, palette_ctx);
        }
        while let Some(event) = queue.pop_front() {
            let before = run.state.clone();
            let effects = run.on(event);
            for effect in effects {
                self.carry_out(&mut run, effect, &mut queue, palette_ctx);
            }
            // A review the compose AI opened is answered through the form:
            // its Apply submits under the step's ticket.
            if run.state != before && run.state == StepState::Paused(PauseKind::ComposeReview) {
                if let Some(ticket) = run.ticket {
                    tickets::arm(ticket, exec::accepts(Plan::Compose));
                }
            }
        }
        if let Some(job) = run.job_id {
            crate::jobs_ui::local_relabel(job, run.job_label());
        }
        crate::sequence_run::put_back(run);
    }

    /// What this frame has to tell the machine, besides the requests.
    fn gather_events(
        &mut self,
        run: &Run,
        queue: &mut VecDeque<RunEvent>,
        _palette_ctx: PaletteContext,
    ) {
        if let Some(ticket) = run.ticket {
            for outcome in tickets::take_for(ticket) {
                queue.push_back(RunEvent::StepDone { ticket, outcome });
            }
        }
        if run.state == StepState::AwaitingAi {
            let kind = match run.current().map(|s| exec::plan(&s.id)) {
                Some(Plan::Ai(kind)) => Some(kind),
                Some(Plan::Compose) => Some(gitpoulpi_config::AiAction::CommitComposer),
                _ => None,
            };
            if let Some(end) = kind.and_then(crate::ai_ui::take_finished) {
                queue.push_back(RunEvent::AiDone(end));
            }
        }
        // A dialog nobody holds a ticket for closed without an answer.
        if matches!(
            run.state,
            StepState::Paused(
                PauseKind::ComposeReview | PauseKind::Dialog(DialogKind::Other)
            )
        ) {
            let waiting = run.ticket.is_some_and(|t| tickets::armed() == Some(t));
            if waiting
                && matches!(self.input_mode, InputMode::Normal)
                && self.compose_stash.is_none()
            {
                queue.push_back(RunEvent::DialogClosed);
            }
        }
        let target = run.ctx.target_root.clone();
        queue.push_back(RunEvent::Tick {
            now: Instant::now(),
            active_root: self.repo.root.clone(),
            input_idle: matches!(self.input_mode, InputMode::Normal)
                && self.preferences.is_none(),
            refreshes: crate::sequence_run::refreshes(),
            target_open: self.repo_tabs.iter().any(|t| {
                crate::sequence_run::machine::same_dir(&t.path, &target)
            }) || crate::sequence_run::machine::same_dir(&self.repo.root, &target),
        });
    }

    fn carry_out(
        &mut self,
        run: &mut Run,
        effect: RunEffect,
        queue: &mut VecDeque<RunEvent>,
        palette_ctx: PaletteContext,
    ) {
        match effect {
            RunEffect::ShowStrip => crate::redraw::poke(),
            RunEffect::Say(text) => self.last_message = Some(text),
            RunEffect::LogFailure(text) => {
                crate::activity_log::report_problem(
                    crate::activity_log::ProblemSource::Sequence,
                    &text,
                );
            }
            RunEffect::CancelJob => {
                self.cancel_in_flight(run);
            }
            RunEffect::ApplyCompose => {
                if let InputMode::ComposeCommits(form) = &mut self.input_mode {
                    if let crate::compose_ui::ComposeOutcome::Submit(action) = form.try_apply() {
                        self.last_message =
                            Some(gitpoulpi_i18n::msg::input::CREATING_COMMITS.get().into());
                        self.link.submit_action(action);
                        self.input_mode = InputMode::Normal;
                    }
                }
            }
            RunEffect::Finish => {
                tickets::disarm();
            }
            RunEffect::Execute {
                step,
                ticket,
                command,
                options,
            } => {
                let result = self.execute(run, step, ticket, command, &options, palette_ctx);
                queue.push_front(RunEvent::Executed(result));
            }
        }
    }

    /// The request in flight stops: the AI's, and the daemon jobs the step
    /// started.
    fn cancel_in_flight(&mut self, run: &Run) {
        if self.ai_ui.running() {
            self.ai_ui.cancel();
        }
        if let Some(since) = run.step_started {
            if crate::jobs_ui::cancel_daemon_since(since)
                == crate::jobs_ui::DaemonCancel::NotCancellable
            {
                self.last_message = Some(m::SAY_STOPPING.get().to_string());
            }
        }
    }

    fn head_name(&self) -> String {
        self.refs_result
            .as_deref()
            .ok()
            .and_then(crate::remotes_ui::head_branch)
            .map(|h| h.name)
            .unwrap_or_default()
    }

    /// The name template of the step, expanded; a name already used by a
    /// branch or a worktree folder gets the next `${n}`.
    fn expanded_name(&self, template: &str) -> String {
        let branches: Vec<String> = self
            .worktree_branch_choices()
            .into_iter()
            .map(|b| b.name)
            .collect();
        let taken = |candidate: &str| {
            branches.iter().any(|b| b == candidate)
                || crate::worktree_dialog::default_worktree_path(&self.repo.root, candidate)
                    .exists()
        };
        exec::expand(template, &self.head_name(), &taken)
    }

    /// Runs `command` with `ticket` armed for the action it submits.
    /// `true`: the ticket was used.
    fn run_armed(&mut self, command: PaletteCommand, ticket: crate::daemon_link::Ticket, plan: Plan) -> bool {
        tickets::arm(ticket, exec::accepts(plan));
        self.last_message = None;
        let cx = &mut self.command_ctx();
        commands::run_command(cx, command);
        tickets::armed() != Some(ticket)
    }

    fn refusal(&self, step_label: &str) -> ExecResult {
        ExecResult::Refused(
            self.last_message
                .clone()
                .unwrap_or_else(|| m::step_failed_start(step_label)),
        )
    }

    /// Executes step `index` of the run.
    fn execute(
        &mut self,
        run: &Run,
        index: usize,
        ticket: crate::daemon_link::Ticket,
        command: Option<PaletteCommand>,
        options: &StepOptions,
        palette_ctx: PaletteContext,
    ) -> ExecResult {
        let step = &run.steps[index];
        let label = step.label.clone();
        let Some(command) = command else {
            return ExecResult::Refused(m::step_unknown(&label));
        };
        if let Some(reason) = palette::unavailable_reason(command, palette_ctx) {
            return ExecResult::Refused(m::step_unavailable(&label, reason));
        }
        let never = step.pause == gitpoulpi_config::sequences::PauseMode::Never;
        tickets::disarm();
        match exec::plan(&step.id) {
            Plan::Stage | Plan::Action => {
                if self.run_armed(command, ticket, exec::plan(&step.id)) {
                    ExecResult::Submitted
                } else if !matches!(self.input_mode, InputMode::Normal) {
                    // A form or prompt of the command: its submit takes the
                    // armed ticket.
                    ExecResult::Dialog(DialogKind::Other)
                } else {
                    tickets::disarm();
                    self.refusal(&label)
                }
            }
            Plan::Instant => {
                self.run_armed(command, ticket, Plan::Instant);
                tickets::disarm();
                ExecResult::Instant
            }
            Plan::Commit => self.execute_commit(command, ticket, options, &label),
            Plan::Ai(kind) => {
                while crate::ai_ui::take_finished(kind).is_some() {}
                self.run_armed(command, ticket, Plan::Ai(kind));
                tickets::disarm();
                if self.ai_ui.running() {
                    ExecResult::AiStarted
                } else {
                    self.refusal(&label)
                }
            }
            Plan::Compose => {
                let kind = gitpoulpi_config::AiAction::CommitComposer;
                while crate::ai_ui::take_finished(kind).is_some() {}
                self.run_armed(command, ticket, Plan::Compose);
                tickets::disarm();
                if self.ai_ui.running() {
                    ExecResult::AiStarted
                } else {
                    self.refusal(&label)
                }
            }
            Plan::StashMessage => {
                self.run_armed(command, ticket, Plan::StashMessage);
                if matches!(self.input_mode, InputMode::Prompt(_)) {
                    ExecResult::Dialog(DialogKind::Other)
                } else {
                    tickets::disarm();
                    self.refusal(&label)
                }
            }
            Plan::Push => self.execute_push(command, ticket, options, never, &label),
            Plan::ForcePush => {
                self.run_armed(command, ticket, Plan::ForcePush);
                if self.remote_ui.holds_ticket(ticket) {
                    ExecResult::Dialog(DialogKind::Confirm)
                } else {
                    tickets::disarm();
                    self.refusal(&label)
                }
            }
            Plan::Worktree { choose } => {
                self.execute_worktree(command, ticket, options, never, choose, run, &label)
            }
            Plan::OpenWorktreeInTab => {
                if let Some(path) = run.ctx.last_worktree.clone() {
                    self.pending_worktree_tab = Some(path);
                } else {
                    self.run_armed(command, ticket, Plan::Instant);
                    tickets::disarm();
                }
                ExecResult::Instant
            }
            Plan::AgentHere => {
                let profile = options.text("profile").unwrap_or("");
                match crate::agent_launch::open_here(profile, &run.ctx.target_root) {
                    Ok(said) => {
                        self.last_message = Some(said);
                        ExecResult::Instant
                    }
                    Err(why) => ExecResult::Refused(why),
                }
            }
            Plan::OpenInLabel => {
                let label_option = options.text("label").unwrap_or("");
                match crate::tab_dir::open_in_by_label(label_option, &run.ctx.target_root) {
                    Ok(said) => {
                        self.last_message = Some(said);
                        ExecResult::Instant
                    }
                    Err(why) => ExecResult::Refused(why),
                }
            }
        }
    }

    fn execute_commit(
        &mut self,
        command: PaletteCommand,
        ticket: crate::daemon_link::Ticket,
        options: &StepOptions,
        label: &str,
    ) -> ExecResult {
        if self.commit_summary.trim().is_empty() {
            if let Some(template) = options.text("message") {
                self.commit_summary = self.expanded_name(template);
            }
        }
        if self.run_armed(command, ticket, Plan::Commit) {
            return ExecResult::Submitted;
        }
        // Not committed: no message is the usual reason, and the run waits
        // for the person to give one (the commit then uses the armed ticket).
        if self.commit_summary.trim().is_empty() && matches!(self.input_mode, InputMode::Normal) {
            return ExecResult::EmptyCommit;
        }
        tickets::disarm();
        self.refusal(label)
    }

    fn execute_push(
        &mut self,
        command: PaletteCommand,
        ticket: crate::daemon_link::Ticket,
        options: &StepOptions,
        never: bool,
        label: &str,
    ) -> ExecResult {
        if never {
            let Some(remote) = options.text("remote") else {
                return ExecResult::Refused(m::step_needs_option(label, "remote"));
            };
            let head = self
                .refs_result
                .as_deref()
                .ok()
                .and_then(crate::remotes_ui::head_branch);
            return match head {
                Some(head) => {
                    self.link
                        .submit_as(Some(ticket), crate::remotes_ui::push_action(remote, &head, false));
                    ExecResult::Submitted
                }
                None => ExecResult::Refused(gitpoulpi_i18n::msg::remotes::DETACHED_PUSH.get().into()),
            };
        }
        let used = self.run_armed(command, ticket, Plan::Push);
        if self.remote_ui.holds_ticket(ticket) {
            ExecResult::Dialog(DialogKind::PushChooser)
        } else if used {
            ExecResult::Submitted
        } else {
            tickets::disarm();
            self.refusal(label)
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn execute_worktree(
        &mut self,
        command: PaletteCommand,
        ticket: crate::daemon_link::Ticket,
        options: &StepOptions,
        never: bool,
        choose: bool,
        _run: &Run,
        label: &str,
    ) -> ExecResult {
        let open_tab = options.flag("open_tab");
        let name = options.text("name").map(|t| self.expanded_name(t));
        if never {
            let Some(name) = name else {
                return ExecResult::Refused(m::step_needs_option(label, "name"));
            };
            let refs = self.refs_result.as_deref().unwrap_or(&[]);
            let worktrees = self.worktrees_result.as_deref().unwrap_or(&[]);
            let Some(seed) = (if choose {
                crate::worktree_dialog::seed_to_choose(refs, worktrees)
            } else {
                crate::worktree_dialog::seed_from_head(refs, worktrees)
            }) else {
                return ExecResult::Refused(gitpoulpi_i18n::msg::worktree_manage::NO_HEAD.get().into());
            };
            let mut dialog = crate::worktree_dialog::WorktreeDialog::with_choices(
                seed,
                &self.repo.root,
                self.worktree_branch_choices(),
                false,
            );
            dialog.apply_run_seed(&crate::repo_dialogs::WorktreeRunSeed {
                name,
                reference: options.text("branch").map(str::to_string),
                open_tab,
            });
            return match dialog.plan() {
                Ok((path, opts)) => {
                    self.pending_worktree_tab = open_tab.then(|| path.clone());
                    self.link
                        .submit_as(Some(ticket), Action::WorktreeAdd { path, opts });
                    ExecResult::Submitted
                }
                Err(why) => ExecResult::Refused(why),
            };
        }
        crate::repo_dialogs::arm_worktree_ticket(ticket);
        if let Some(name) = name {
            crate::repo_dialogs::arm_worktree_run(crate::repo_dialogs::WorktreeRunSeed {
                name,
                reference: options.text("branch").map(str::to_string),
                open_tab,
            });
        }
        self.last_message = None;
        commands::run_command(&mut self.command_ctx(), command);
        if self.worktree_seed.is_some() {
            ExecResult::Dialog(DialogKind::Worktree)
        } else {
            crate::repo_dialogs::clear_worktree_arming();
            self.refusal(label)
        }
    }
}
