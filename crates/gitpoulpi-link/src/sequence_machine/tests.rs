use super::*;
use gitpoulpi_app::ai_state::AiEnd;
use gitpoulpi_config::sequences::StepSpec;

fn def(steps: Vec<StepSpec>) -> SequenceDef {
    SequenceDef {
        id: "backup".into(),
        name: "Backup".into(),
        steps,
        ..SequenceDef::default()
    }
}

fn run_of(steps: Vec<StepSpec>) -> Run {
    Run::new(&def(steps), Path::new("/repo"), Instant::now(), |id| {
        (None, id.to_string())
    })
}

fn tick_at(run: &mut Run, root: &str, refreshes: u64) -> Vec<RunEffect> {
    run.on(RunEvent::Tick {
        now: Instant::now(),
        active_root: PathBuf::from(root),
        input_idle: true,
        refreshes,
        target_open: true,
    })
}

fn tick(run: &mut Run) -> Vec<RunEffect> {
    // Every frame a refresh has landed, unless a test says otherwise.
    let next = run.last_refreshes + 1;
    tick_at(run, "/repo", next)
}

/// Ticks until the machine executes the step; returns its ticket.
fn start_step(run: &mut Run) -> Ticket {
    let effects = tick(run);
    let ticket = effects
        .iter()
        .find_map(|e| match e {
            RunEffect::Execute { ticket, .. } => Some(*ticket),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no Execute in {effects:?}, state {:?}", run.state));
    assert_eq!(run.state, StepState::Submitted(ticket));
    ticket
}

fn ok(run: &mut Run, ticket: Ticket) -> Vec<RunEffect> {
    run.on(RunEvent::StepDone {
        ticket,
        outcome: StepOutcome::Ok(None),
    })
}

fn says(effects: &[RunEffect]) -> String {
    effects
        .iter()
        .filter_map(|e| match e {
            RunEffect::Say(s) => Some(s.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

#[test]
fn a_chain_of_ok_steps_ends_done() {
    let mut run = run_of(vec![StepSpec::new("stage_all"), StepSpec::new("fetch")]);
    let t1 = start_step(&mut run);
    assert_eq!(run.position(), (1, 2));
    let effects = ok(&mut run, t1);
    assert_eq!(run.state, StepState::Ready);
    assert_eq!(run.position(), (2, 2));
    assert!(!effects.contains(&RunEffect::Finish));
    let t2 = start_step(&mut run);
    let effects = ok(&mut run, t2);
    assert_eq!(run.state, StepState::Done);
    assert!(effects.contains(&RunEffect::Finish));
    assert!(says(&effects).contains("Backup finished"), "{effects:?}");
}

#[test]
fn a_failure_names_the_step_and_stops() {
    let mut run = run_of(vec![
        StepSpec::new("stage_all"),
        StepSpec::new("push").with_option("remote", "origin"),
    ]);
    let t1 = start_step(&mut run);
    ok(&mut run, t1);
    let t2 = start_step(&mut run);
    let effects = run.on(RunEvent::StepDone {
        ticket: t2,
        outcome: StepOutcome::Failed {
            text: "no such remote".into(),
        },
    });
    assert!(matches!(run.state, StepState::Failed { step: 1, .. }));
    let said = says(&effects);
    assert!(said.contains("Backup") && said.contains("2/2"), "{said}");
    assert!(
        said.contains("push") && said.contains("no such remote"),
        "{said}"
    );
    let logged = effects.iter().find_map(|e| match e {
        RunEffect::LogFailure(t) => Some(t.clone()),
        _ => None,
    });
    let logged = logged.expect("an Activity Log entry");
    assert!(
        logged.contains("push") && logged.contains("remote"),
        "{logged}"
    );
    assert!(effects.contains(&RunEffect::Finish));
    // A failed run is dismissed with Cancel and runs nothing more.
    assert!(tick(&mut run).is_empty());
    run.on(RunEvent::Cancel);
    assert_eq!(run.state, StepState::Canceled);
}

#[test]
fn cancel_in_flight_stops_the_job_and_keeps_earlier_steps() {
    let mut run = run_of(vec![StepSpec::new("stage_all"), StepSpec::new("push")]);
    let t1 = start_step(&mut run);
    ok(&mut run, t1);
    let _t2 = start_step(&mut run);
    let effects = run.on(RunEvent::Cancel);
    assert!(effects.contains(&RunEffect::CancelJob));
    assert!(effects.contains(&RunEffect::Finish));
    assert_eq!(run.state, StepState::Canceled);
    let said = says(&effects);
    assert!(said.contains("canceled after step 2/2 (push)"), "{said}");
    assert!(said.contains("earlier steps are kept"), "{said}");
    // The old ticket's late answer does nothing.
    assert!(ok(&mut run, _t2).is_empty());
}

#[test]
fn cancel_while_paused_does_not_stop_a_job() {
    let mut run = run_of(vec![StepSpec::new("commit")]);
    let _ = start_step(&mut run);
    run.on(RunEvent::Executed(ExecResult::EmptyCommit));
    assert_eq!(run.state, StepState::Paused(PauseKind::EmptyCommit));
    let effects = run.on(RunEvent::Cancel);
    assert!(!effects.contains(&RunEffect::CancelJob));
    assert_eq!(run.state, StepState::Canceled);
}

#[test]
fn the_ai_message_pauses_by_default_and_continue_goes_on() {
    let mut run = run_of(vec![
        StepSpec::new("ai_generate_commit_message"),
        StepSpec::new("commit"),
    ]);
    let _ = start_step(&mut run);
    run.on(RunEvent::Executed(ExecResult::AiStarted));
    assert_eq!(run.state, StepState::AwaitingAi);
    run.on(RunEvent::AiDone(AiEnd::Ok));
    assert_eq!(run.state, StepState::Paused(PauseKind::ReviewMessage));
    assert!(run.can_continue());
    // Nothing runs while it waits.
    assert!(tick(&mut run).is_empty());
    run.on(RunEvent::Continue);
    assert_eq!(run.state, StepState::Ready);
    assert_eq!(run.position(), (2, 2));
}

#[test]
fn pause_never_on_the_ai_message_goes_straight_on() {
    let mut run = run_of(vec![
        StepSpec::new("ai_generate_commit_message").with_pause(PauseMode::Never),
        StepSpec::new("commit"),
    ]);
    let _ = start_step(&mut run);
    run.on(RunEvent::Executed(ExecResult::AiStarted));
    run.on(RunEvent::AiDone(AiEnd::Ok));
    assert_eq!(run.state, StepState::Ready);
    assert_eq!(run.position(), (2, 2));
}

#[test]
fn an_ai_failure_stops_at_its_step() {
    let mut run = run_of(vec![StepSpec::new("ai_generate_commit_message")]);
    let _ = start_step(&mut run);
    run.on(RunEvent::Executed(ExecResult::AiStarted));
    let effects = run.on(RunEvent::AiDone(AiEnd::Failed("rate limited".into())));
    assert!(matches!(run.state, StepState::Failed { .. }));
    assert!(says(&effects).contains("rate limited"));
}

#[test]
fn compose_review_never_applies_as_proposed() {
    let mut run = run_of(vec![
        StepSpec::new("ai_compose_commits").with_pause(PauseMode::Never)
    ]);
    let _ = start_step(&mut run);
    run.on(RunEvent::Executed(ExecResult::AiStarted));
    let effects = run.on(RunEvent::AiDone(AiEnd::Ok));
    assert!(effects.contains(&RunEffect::ApplyCompose));
    assert_eq!(run.state, StepState::Paused(PauseKind::ComposeReview));
    // The review closing without an answer cancels.
    let mut run = run_of(vec![StepSpec::new("ai_compose_commits")]);
    let _ = start_step(&mut run);
    run.on(RunEvent::Executed(ExecResult::AiStarted));
    run.on(RunEvent::AiDone(AiEnd::Ok));
    run.on(RunEvent::DialogClosed);
    assert_eq!(run.state, StepState::Canceled);
}

#[test]
fn an_empty_commit_pauses_and_the_commit_finishes_the_step() {
    let mut run = run_of(vec![StepSpec::new("commit"), StepSpec::new("push")]);
    let ticket = start_step(&mut run);
    run.on(RunEvent::Executed(ExecResult::EmptyCommit));
    assert_eq!(run.state, StepState::Paused(PauseKind::EmptyCommit));
    // The person commits: the armed ticket reports the commit.
    ok(&mut run, ticket);
    assert_eq!(run.state, StepState::Ready);
    assert_eq!(run.position(), (2, 2));
}

#[test]
fn continue_on_an_empty_commit_runs_the_step_again() {
    let mut run = run_of(vec![StepSpec::new("commit")]);
    let _ = start_step(&mut run);
    run.on(RunEvent::Executed(ExecResult::EmptyCommit));
    run.on(RunEvent::Continue);
    assert_eq!(run.state, StepState::Ready);
    let _ = start_step(&mut run);
}

#[test]
fn always_shows_the_next_step_before_running_it() {
    let mut run = run_of(vec![
        StepSpec::new("stage_all").with_pause(PauseMode::Always)
    ]);
    let effects = tick(&mut run);
    assert_eq!(run.state, StepState::Paused(PauseKind::BeforeStep));
    assert!(!effects
        .iter()
        .any(|e| matches!(e, RunEffect::Execute { .. })));
    run.on(RunEvent::Continue);
    let _ = start_step(&mut run);
}

#[test]
fn a_force_push_always_asks_even_with_default_pause() {
    let mut run = run_of(vec![StepSpec::new("force_push_with_lease")]);
    tick(&mut run);
    assert_eq!(run.state, StepState::Paused(PauseKind::BeforeStep));
}

#[test]
fn a_rejected_push_waits_for_the_user_and_the_resolved_push_continues() {
    let mut run = run_of(vec![StepSpec::new("push"), StepSpec::new("fetch")]);
    let ticket = start_step(&mut run);
    run.on(RunEvent::StepDone {
        ticket,
        outcome: StepOutcome::NeedsUser(UserWait::PushRejected),
    });
    assert_eq!(run.state, StepState::WaitingForUser(UserWait::PushRejected));
    // Nothing runs, nothing is canceled, while the dialog is up.
    assert!(tick(&mut run).is_empty());
    // Pull then push (or force with lease) worked: the same ticket ends Ok.
    ok(&mut run, ticket);
    assert_eq!(run.state, StepState::Ready);
    assert_eq!(run.position(), (2, 2));
}

#[test]
fn a_rejected_push_whose_dialog_is_closed_cancels_the_run() {
    let mut run = run_of(vec![StepSpec::new("push"), StepSpec::new("fetch")]);
    let ticket = start_step(&mut run);
    run.on(RunEvent::StepDone {
        ticket,
        outcome: StepOutcome::NeedsUser(UserWait::PushRejected),
    });
    let effects = run.on(RunEvent::StepDone {
        ticket,
        outcome: StepOutcome::Canceled,
    });
    assert_eq!(run.state, StepState::Canceled);
    assert!(effects.contains(&RunEffect::Finish));
    assert!(says(&effects).contains("canceled after step 1/2 (push)"));
}

#[test]
fn a_window_that_is_not_the_target_waits_and_runs_nothing() {
    let mut run = run_of(vec![StepSpec::new("fetch")]);
    let effects = tick_at(&mut run, "/elsewhere", 0);
    assert_eq!(run.state, StepState::WaitingForWindow);
    assert!(!effects
        .iter()
        .any(|e| matches!(e, RunEffect::Execute { .. })));
    // Still elsewhere: still nothing.
    assert!(tick_at(&mut run, "/elsewhere", 0).is_empty());
    // The target window comes up: back to Ready, the next tick runs it.
    tick_at(&mut run, "/repo", 1);
    assert_eq!(run.state, StepState::Ready);
    let _ = start_step(&mut run);
}

#[test]
fn a_worktree_step_with_open_tab_moves_the_target_and_waits_for_its_tab() {
    let mut run = run_of(vec![
        StepSpec::new("create_worktree_from_current_branch")
            .with_option("name", "wip")
            .with_option("open_tab", true),
        StepSpec::new("tab_dir_terminal"),
    ]);
    let ticket = start_step(&mut run);
    run.on(RunEvent::StepDone {
        ticket,
        outcome: StepOutcome::Ok(Some(StepPayload::Worktree(PathBuf::from("/wt/wip")))),
    });
    assert_eq!(run.ctx.target_root, PathBuf::from("/wt/wip"));
    assert_eq!(run.ctx.last_worktree, Some(PathBuf::from("/wt/wip")));
    // The old tab is on screen: the terminal step waits for the new one.
    tick_at(&mut run, "/repo", 100);
    assert_eq!(run.state, StepState::WaitingForWindow);
    tick_at(&mut run, "/wt/wip", 101);
    assert_eq!(run.state, StepState::Ready);
    let _ = start_step_at(&mut run, "/wt/wip");
}

fn start_step_at(run: &mut Run, root: &str) -> Ticket {
    let next = run.last_refreshes + 1;
    let effects = tick_at(run, root, next);
    effects
        .iter()
        .find_map(|e| match e {
            RunEffect::Execute { ticket, .. } => Some(*ticket),
            _ => None,
        })
        .expect("Execute")
}

#[test]
fn a_worktree_without_open_tab_keeps_the_target() {
    let mut run = run_of(vec![
        StepSpec::new("create_worktree_from_current_branch").with_option("name", "wip"),
        StepSpec::new("tab_dir_terminal"),
    ]);
    let ticket = start_step(&mut run);
    run.on(RunEvent::StepDone {
        ticket,
        outcome: StepOutcome::Ok(Some(StepPayload::Worktree(PathBuf::from("/wt/wip")))),
    });
    assert_eq!(run.ctx.target_root, PathBuf::from("/repo"));
    assert_eq!(run.ctx.last_worktree, Some(PathBuf::from("/wt/wip")));
}

#[test]
fn a_target_tab_that_was_closed_cancels_with_the_reason() {
    let mut run = run_of(vec![StepSpec::new("fetch"), StepSpec::new("fetch")]);
    let t = start_step(&mut run);
    ok(&mut run, t);
    // The window of the target was seen; now its tab is gone.
    let effects = run.on(RunEvent::Tick {
        now: Instant::now(),
        active_root: PathBuf::from("/other"),
        input_idle: true,
        refreshes: 0,
        target_open: false,
    });
    assert_eq!(run.state, StepState::Canceled);
    assert!(says(&effects).contains("was closed"), "{effects:?}");
}

#[test]
fn a_tab_that_never_opens_fails_the_step() {
    let mut run = run_of(vec![StepSpec::new("fetch")]);
    run.ctx.target_root = PathBuf::from("/wt/never");
    run.ctx.target_seen = false;
    let t0 = Instant::now();
    let tick_now = |run: &mut Run, now| {
        run.on(RunEvent::Tick {
            now,
            active_root: PathBuf::from("/repo"),
            input_idle: true,
            refreshes: 0,
            target_open: false,
        })
    };
    tick_now(&mut run, t0);
    assert_eq!(run.state, StepState::WaitingForWindow);
    tick_now(&mut run, t0 + WINDOW_WAIT + Duration::from_secs(1));
    assert!(matches!(run.state, StepState::Failed { .. }));
}

#[test]
fn the_step_after_a_mutation_waits_for_a_later_status_refresh() {
    let mut run = run_of(vec![
        StepSpec::new("stage_all"),
        StepSpec::new("ai_generate_commit_message"),
    ]);
    let t = start_step_with(&mut run, 5);
    ok(&mut run, t);
    // The refresh counter has not moved: the optimistic status is not enough.
    assert!(tick_at(&mut run, "/repo", 5).is_empty());
    assert_eq!(run.state, StepState::Ready);
    // A refresh landed: now the AI step runs.
    let effects = tick_at(&mut run, "/repo", 6);
    assert!(effects
        .iter()
        .any(|e| matches!(e, RunEffect::Execute { .. })));
}

fn start_step_with(run: &mut Run, refreshes: u64) -> Ticket {
    tick_at(run, "/repo", refreshes)
        .iter()
        .find_map(|e| match e {
            RunEffect::Execute { ticket, .. } => Some(*ticket),
            _ => None,
        })
        .expect("Execute")
}

#[test]
fn a_step_that_cannot_start_fails_with_its_reason() {
    let mut run = run_of(vec![StepSpec::new("push")]);
    let _ = start_step(&mut run);
    let effects = run.on(RunEvent::Executed(ExecResult::Refused(
        "detached HEAD".into(),
    )));
    assert!(matches!(run.state, StepState::Failed { .. }));
    assert!(says(&effects).contains("detached HEAD"));
}

#[test]
fn a_dialog_of_the_step_pauses_and_closing_it_cancels() {
    let mut run = run_of(vec![StepSpec::new("new_branch")]);
    let _ = start_step(&mut run);
    run.on(RunEvent::Executed(ExecResult::Dialog(DialogKind::Other)));
    assert_eq!(
        run.state,
        StepState::Paused(PauseKind::Dialog(DialogKind::Other))
    );
    run.on(RunEvent::DialogClosed);
    assert_eq!(run.state, StepState::Canceled);
}

#[test]
fn nothing_runs_while_the_user_is_typing() {
    let mut run = run_of(vec![StepSpec::new("fetch")]);
    let effects = run.on(RunEvent::Tick {
        now: Instant::now(),
        active_root: PathBuf::from("/repo"),
        input_idle: false,
        refreshes: 0,
        target_open: true,
    });
    assert!(effects.is_empty());
    assert_eq!(run.state, StepState::Ready);
}
