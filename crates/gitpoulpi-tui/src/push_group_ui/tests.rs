use super::*;
use crate::daemon_link::{RawQueue, Reply};
use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use gitpoulpi_core::domain::{
    BranchRemotes, GroupAction as A, Oid, PlannedTarget, Remote, RemoteGroupPlan,
    RemoteGroupStatus, RemoteRelation, TargetRole,
};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn serial() -> std::sync::MutexGuard<'static, ()> {
    let guard = TEST_SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset_for_tests();
    guard
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn ctrl(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::CONTROL)
}

fn planned(remote: &str, relation: RemoteRelation, suggested: A, allowed: &[A]) -> PlannedTarget {
    let (ahead, behind) = match relation {
        RemoteRelation::LocalAhead(n) => (n, 0),
        RemoteRelation::RemoteAhead(n) => (0, n),
        RemoteRelation::Diverged { ahead, behind } => (ahead, behind),
        _ => (0, 0),
    };
    PlannedTarget {
        remote: remote.into(),
        branch: "feature".into(),
        role: TargetRole::PushTo,
        relation,
        rewritten: false,
        ahead,
        behind,
        local_unmatched: None,
        remote_unmatched: None,
        suggested,
        allowed: allowed.to_vec(),
        lease: Some(Oid([3; 20])),
        conflicts: Vec::new(),
        same_url_as: None,
        checked: true,
        fetched: false,
        error: None,
        missing: false,
    }
}

fn plan_of(targets: Vec<PlannedTarget>) -> RemoteGroupPlan {
    RemoteGroupPlan {
        branch: "feature".into(),
        mode: GroupMode::Push,
        local: Oid([1; 20]),
        preflight: true,
        targets,
    }
}

fn three_ready() -> RemoteGroupPlan {
    plan_of(
        ["origin", "gitlab", "backup"]
            .into_iter()
            .map(|r| {
                planned(
                    r,
                    RemoteRelation::LocalAhead(2),
                    A::Push,
                    &[A::Push, A::Skip],
                )
            })
            .collect(),
    )
}

fn open_plan_with(plan: RemoteGroupPlan) -> RawQueue {
    let queue = RawQueue::default();
    set_requester(queue.requester());
    with(|c| {
        c.open(
            Path::new("/repo"),
            "feature",
            Step::Plan(PlanView::from_plan(plan, false)),
        );
    });
    queue
}

fn screen_of(width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(render).unwrap();
    dialog_kit::buffer_text(terminal.backend().buffer())
}

fn screen() -> String {
    screen_of(110, 34)
}

fn submitted(effect: Effect) -> Vec<Action> {
    match effect {
        Effect::Submit(actions) => actions,
        Effect::None => panic!("nothing submitted"),
        Effect::Confirm { prompt, .. } => panic!("asked to confirm: {prompt}"),
    }
}

#[test]
fn every_remote_ready_runs_with_the_suggested_push() {
    let _s = serial();
    open_plan_with(three_ready());
    let text = screen();
    for remote in ["origin", "gitlab", "backup"] {
        assert!(text.contains(remote), "{text}");
    }
    assert!(text.contains("ready"), "{text}");
    assert!(text.contains("Then: push to 3 remotes."), "{text}");
    let actions = submitted(on_key(ctrl(KeyCode::Enter)));
    let [Action::RemoteGroupRun(req)] = actions.as_slice() else {
        panic!("one run expected: {actions:?}");
    };
    assert_eq!(req.mode, GroupMode::Push);
    assert_eq!(req.targets.len(), 3);
    assert!(req.targets.iter().all(|t| t.action == A::Push));
    // The same dialog is now the progress view.
    assert!(screen().contains("Closing this window does not stop the run."));
}

#[test]
fn a_diverged_remote_defaults_to_merge_and_shows_the_conflict_line() {
    let _s = serial();
    let mut gitlab = planned(
        "gitlab",
        RemoteRelation::Diverged {
            ahead: 2,
            behind: 1,
        },
        A::MergeFrom,
        &[A::MergeFrom, A::RebaseOnto, A::ForceWithLease, A::Skip],
    );
    gitlab.conflicts = vec!["src/a.rs".into()];
    open_plan_with(plan_of(vec![
        planned(
            "origin",
            RemoteRelation::LocalAhead(2),
            A::Push,
            &[A::Push, A::Skip],
        ),
        gitlab,
    ]));
    let text = screen();
    assert!(text.contains("diverged"), "{text}");
    assert!(text.contains("Merge from gitlab"), "{text}");
    assert!(
        text.contains("merge would conflict in 1 file: src/a.rs"),
        "{text}"
    );
    assert!(
        text.contains("Then: merge gitlab, push to 2 remotes."),
        "{text}"
    );
}

#[test]
fn left_and_right_step_the_action_and_s_skips_the_row() {
    let _s = serial();
    open_plan_with(plan_of(vec![planned(
        "gitlab",
        RemoteRelation::Diverged {
            ahead: 1,
            behind: 1,
        },
        A::MergeFrom,
        &[A::MergeFrom, A::RebaseOnto, A::ForceWithLease, A::Skip],
    )]));
    on_key(key(KeyCode::Right));
    assert!(screen().contains("Rebase onto gitlab"));
    on_key(key(KeyCode::Right));
    assert!(screen().contains("Force push (lease)"));
    on_key(key(KeyCode::Char('s')));
    assert!(screen().contains("Then: nothing to do."));
}

#[test]
fn a_force_push_asks_naming_the_remote_and_the_commit_count_and_runs_nothing() {
    let _s = serial();
    let mut mirror = planned(
        "mirror",
        RemoteRelation::Diverged {
            ahead: 4,
            behind: 4,
        },
        A::ForceWithLease,
        &[A::ForceWithLease, A::MergeFrom, A::Skip],
    );
    mirror.rewritten = true;
    mirror.remote_unmatched = Some(0);
    mirror.behind = 4;
    open_plan_with(plan_of(vec![mirror]));
    let text = screen();
    assert!(
        text.contains("you rebased") || text.contains("rewritten"),
        "{text}"
    );
    match on_key(ctrl(KeyCode::Enter)) {
        Effect::Confirm { prompt, pending } => {
            assert!(prompt.contains("mirror"), "{prompt}");
            assert!(prompt.contains('4'), "{prompt}");
            let PendingDestructive::GroupForcePush { request } = pending else {
                panic!("wrong pending");
            };
            assert_eq!(request.targets[0].action, A::ForceWithLease);
            assert_eq!(request.targets[0].lease, Some(Oid([3; 20])));
        }
        _ => panic!("a force push must be confirmed"),
    }
    // Nothing started: the dialog is still the plan.
    assert!(with(|c| c.run.is_none()));
    assert!(screen().contains("Force push replaces 4 commits on mirror."));
}

#[test]
fn skip_problems_sets_aside_diverged_and_unreachable_rows() {
    let _s = serial();
    let mut down = planned("down", RemoteRelation::Absent, A::Skip, &[A::Skip]);
    down.error = Some("no route".into());
    open_plan_with(plan_of(vec![
        planned(
            "origin",
            RemoteRelation::LocalAhead(1),
            A::Push,
            &[A::Push, A::Skip],
        ),
        planned(
            "gitlab",
            RemoteRelation::Diverged {
                ahead: 1,
                behind: 1,
            },
            A::MergeFrom,
            &[A::MergeFrom, A::Skip],
        ),
        down,
    ]));
    with(|c| {
        if let Some(Dlg {
            step: Step::Plan(p),
            ..
        }) = c.dlg.as_mut()
        {
            p.skip_problems();
            let chosen: Vec<_> = p.rows.iter().map(|r| r.chosen).collect();
            assert_eq!(chosen, vec![A::Push, A::Skip, A::Skip]);
        }
    });
}

#[test]
fn rebasing_while_another_remote_holds_the_commits_warns_of_a_force_push() {
    let _s = serial();
    let mut up = planned(
        "origin",
        RemoteRelation::UpToDate,
        A::Push,
        &[A::Push, A::Skip],
    );
    up.local_unmatched = Some(0);
    open_plan_with(plan_of(vec![
        up,
        planned(
            "gitlab",
            RemoteRelation::Diverged {
                ahead: 1,
                behind: 1,
            },
            A::RebaseOnto,
            &[A::RebaseOnto, A::MergeFrom, A::Skip],
        ),
    ]));
    let text = screen();
    assert!(
        text.contains("origin will then need a force push"),
        "{text}"
    );
}

fn select_with(rows: &[(&str, bool)], primary: Option<&str>) -> Select {
    let remotes: Vec<Remote> = rows
        .iter()
        .map(|(n, _)| Remote {
            name: (*n).into(),
            fetch_url: format!("/srv/{n}.git"),
            push_url: None,
        })
        .collect();
    let configured: Vec<TargetRef> = rows
        .iter()
        .filter(|(_, on)| *on)
        .map(|(n, _)| TargetRef {
            remote: (*n).into(),
            branch: "feature".into(),
        })
        .collect();
    let status = RemoteGroupStatus {
        remotes: BranchRemotes {
            branch: "feature".into(),
            primary: primary.map(|p| TargetRef {
                remote: p.into(),
                branch: "feature".into(),
            }),
            push_to: configured
                .iter()
                .filter(|t| Some(t.remote.as_str()) != primary)
                .cloned()
                .collect(),
            pull_from: Vec::new(),
            missing: Vec::new(),
        },
        targets: Vec::new(),
        pending: None,
    };
    let mut s = Select::new(Reply::ready(Ok(remotes)), Reply::ready(Ok(status)));
    assert!(s.poll("feature"));
    s
}

#[test]
fn the_selection_pre_ticks_the_upstream_and_the_push_targets() {
    let _s = serial();
    let queue = RawQueue::default();
    set_requester(queue.requester());
    let s = select_with(
        &[("origin", true), ("gitlab", true), ("backup", false)],
        Some("origin"),
    );
    with(|c| c.open(Path::new("/repo"), "feature", Step::Select(s)));
    let text = screen();
    assert!(text.contains("[x] origin"), "{text}");
    assert!(text.contains("[x] gitlab"), "{text}");
    assert!(text.contains("[ ] backup"), "{text}");
    assert!(text.contains("Remember for this branch"), "{text}");
    // Uncheck gitlab (row 2): Space, then Next asks for the plan of the
    // others, and without Remember nothing is written.
    on_key(key(KeyCode::Down));
    on_key(key(KeyCode::Char(' ')));
    assert!(screen().contains("[ ] gitlab"));
    match on_key(key(KeyCode::Enter)) {
        Effect::None => {}
        _ => panic!("no config write without Remember"),
    }
    let asked = queue.queued();
    let (kind, args) = asked
        .iter()
        .find(|(k, _)| *k == OperationKind::RemoteGroupPlan)
        .expect("the plan was asked");
    assert_eq!(*kind, OperationKind::RemoteGroupPlan);
    let targets = args["targets"].as_array().unwrap();
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0]["remote"], "origin");
}

#[test]
fn remember_writes_the_ticked_remotes_as_the_branchs_push_targets() {
    let _s = serial();
    let queue = RawQueue::default();
    set_requester(queue.requester());
    let s = select_with(
        &[("origin", true), ("gitlab", true), ("backup", false)],
        Some("origin"),
    );
    with(|c| c.open(Path::new("/repo"), "feature", Step::Select(s)));
    on_key(key(KeyCode::Down));
    on_key(key(KeyCode::Down));
    on_key(key(KeyCode::Char(' ')));
    on_key(key(KeyCode::Char('m')));
    let actions = submitted(on_key(key(KeyCode::Enter)));
    let [Action::BranchSetRemotes {
        branch,
        upstream,
        push_to,
        ..
    }] = actions.as_slice()
    else {
        panic!("{actions:?}");
    };
    assert_eq!(branch, "feature");
    assert_eq!(*upstream, None);
    assert_eq!(push_to, &vec!["gitlab".to_string(), "backup".to_string()]);
}

#[test]
fn progress_follows_the_events_and_a_conflict_pauses_behind_the_strip() {
    let _s = serial();
    open_plan_with(three_ready());
    let actions = submitted(on_key(ctrl(KeyCode::Enter)));
    assert_eq!(actions.len(), 1);
    on_progress("run-1", "origin", TargetStatus::Pushed { created: false });
    on_progress("run-1", "gitlab", TargetStatus::Integrating);
    let text = screen();
    assert!(
        text.contains("\u{2713} origin") || text.contains("origin"),
        "{text}"
    );
    assert!(text.contains("sent"), "{text}");
    assert!(text.contains("combining"), "{text}");
    assert!(text.contains("waiting"), "{text}");
    // Another run's events are not ours.
    on_progress("other", "backup", TargetStatus::Pushed { created: false });
    assert!(!with(|c| c
        .run
        .as_ref()
        .unwrap()
        .rows
        .iter()
        .any(|r| r.remote == "backup" && r.status.is_some())));
    // The run stops on conflicts: the dialog closes, the strip stays.
    let outcome = RemoteGroupOutcome {
        run_id: "run-1".into(),
        branch: "feature".into(),
        mode: GroupMode::Push,
        status: GroupRunStatus::Conflicted,
        local_tip: None,
        targets: Vec::new(),
        pending: true,
        stopped: Some(gitpoulpi_core::domain::StoppedAt {
            remote: "gitlab".into(),
            branch: "feature".into(),
            op: gitpoulpi_core::domain::StoppedOp::Merge,
            pre_tip: Oid([1; 20]),
            remote_tip: Oid([2; 20]),
        }),
        conflicts: vec!["a.txt".into()],
        plan: None,
        message: "conflicts".into(),
    };
    on_outcome(Path::new("/repo"), &outcome);
    assert!(!is_open());
    let (text, ready) = strip::text(Path::new("/repo")).expect("the strip shows");
    assert!(text.contains("paused (gitlab)"), "{text}");
    assert!(!ready);
    // Continue is not offered until the daemon says the conclusion is in.
    assert!(matches!(
        strip::click(Path::new("/repo"), StripButton::Continue),
        Effect::None
    ));
    with(|c| c.paused.as_mut().unwrap().ready = true);
    let actions = submitted(strip::click(Path::new("/repo"), StripButton::Continue));
    assert!(matches!(
        actions.as_slice(),
        [Action::RemoteGroupContinue { expect_run_id: Some(id), .. }] if id == "run-1"
    ));
    // Abort group is a confirmation, never immediate.
    assert!(matches!(
        strip::click(Path::new("/repo"), StripButton::Abort),
        Effect::Confirm { .. }
    ));
}

#[test]
fn a_closed_progress_view_comes_back_and_the_run_is_untouched() {
    let _s = serial();
    open_plan_with(three_ready());
    submitted(on_key(ctrl(KeyCode::Enter)));
    on_key(key(KeyCode::Esc));
    assert!(!is_open());
    on_progress("run-9", "origin", TargetStatus::Pushed { created: true });
    assert!(PushGroupUi::open_progress());
    assert!(is_open());
    assert!(screen().contains("created"));
}

#[test]
fn the_strip_paints_one_row_with_its_buttons() {
    let _s = serial();
    with(|c| {
        c.paused = Some(Paused {
            root: PathBuf::from("/repo"),
            branch: "feature".into(),
            run_id: "r".into(),
            remote: "gitlab".into(),
            ready: true,
            prefer_rebase: false,
            mode: GroupMode::Push,
        });
    });
    let mut terminal = Terminal::new(TestBackend::new(120, 3)).unwrap();
    let mut hits = Vec::new();
    terminal
        .draw(|f| {
            hits = render_strip(f, Rect::new(0, 0, 120, 1), Path::new("/repo"), None);
        })
        .unwrap();
    let text = dialog_kit::buffer_text(terminal.backend().buffer());
    assert!(text.contains("Grouped push paused (gitlab)"), "{text}");
    assert!(
        text.contains("Continue") && text.contains("Abort group"),
        "{text}"
    );
    assert_eq!(hits.len(), 2);
    // Not ready: Continue stays (owner decision) but greyed, and a press does
    // nothing.
    with(|c| c.paused.as_mut().unwrap().ready = false);
    terminal
        .draw(|f| {
            hits = render_strip(f, Rect::new(0, 0, 120, 1), Path::new("/repo"), None);
        })
        .unwrap();
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].1, StripButton::Continue);
    assert_eq!(hits[1].1, StripButton::Abort);
    let cell = |hit: Rect| terminal.backend().buffer()[(hit.x + 1, hit.y)].style();
    let disabled = dialog_kit::button_style(dialog_kit::ButtonState::Disabled);
    assert_eq!(cell(hits[0].0).bg, disabled.bg, "Continue is greyed");
    assert_ne!(cell(hits[1].0).bg, disabled.bg, "Abort stays live");
    assert!(matches!(
        strip_click(Path::new("/repo"), StripButton::Continue),
        Effect::None
    ));
    with(|c| c.paused.as_mut().unwrap().ready = true);
    assert!(matches!(
        strip_click(Path::new("/repo"), StripButton::Continue),
        Effect::Submit(_)
    ));
}

#[test]
fn a_click_on_the_run_button_and_on_a_combo_row_work_from_the_paint() {
    let _s = serial();
    open_plan_with(three_ready());
    screen();
    let (run_rect, combo_rect) = with(|c| {
        let d = c.dlg.as_ref().unwrap();
        let run = d
            .hits
            .iter()
            .find(|(_, p)| *p == Part::Button(model::Btn::Run))
            .unwrap()
            .0;
        let combo = d.hits.iter().find(|(_, p)| *p == Part::Combo(1)).unwrap().0;
        (run, combo)
    });
    let click = |r: Rect| MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: r.x + 1,
        row: r.y,
        modifiers: KeyModifiers::NONE,
    };
    // The combo opens and a click on its Skip row picks it.
    on_mouse(click(combo_rect));
    assert!(with(|c| match &c.dlg.as_ref().unwrap().step {
        Step::Plan(p) => p.rows[1].combo.is_open(),
        _ => false,
    }));
    screen();
    on_key(key(KeyCode::Down));
    on_key(key(KeyCode::Enter));
    assert!(screen().contains("Then: push to 2 remotes."));
    let actions = submitted(on_mouse(click(run_rect)));
    let [Action::RemoteGroupRun(req)] = actions.as_slice() else {
        panic!("{actions:?}");
    };
    assert_eq!(req.targets[1].action, A::Skip);
}

/// Opens one state of the dialog for the layout guard (`i18n_layout_tests`):
/// 0 the selection, 1 a plan with every kind of row, 2 the loading plan,
/// 3 a progress view with a row in each state.
pub(crate) fn open_for_layout(which: usize) {
    reset_for_tests();
    let queue = RawQueue::default();
    set_requester(queue.requester());
    std::mem::forget(queue);
    match which {
        0 => {
            let s = select_with(
                &[
                    ("origin", true),
                    ("gitlab", true),
                    ("a-rather-long-remote-name", false),
                ],
                Some("origin"),
            );
            with(|c| c.open(Path::new("/repo"), "feature", Step::Select(s)));
        }
        1 => {
            let mut clash = planned(
                "gitlab",
                RemoteRelation::Diverged {
                    ahead: 12,
                    behind: 3,
                },
                A::MergeFrom,
                &[A::MergeFrom, A::RebaseOnto, A::ForceWithLease, A::Skip],
            );
            clash.conflicts = vec!["src/a-long-file-name.rs".into(), "docs/b.md".into()];
            let mut mirror = planned(
                "a-rather-long-remote-name",
                RemoteRelation::Diverged {
                    ahead: 4,
                    behind: 4,
                },
                A::ForceWithLease,
                &[A::ForceWithLease, A::Skip],
            );
            mirror.rewritten = true;
            let mut down = planned("down", RemoteRelation::Absent, A::Skip, &[A::Skip]);
            down.error = Some("no route".into());
            open_plan_with(plan_of(vec![
                planned(
                    "origin",
                    RemoteRelation::LocalAhead(2),
                    A::Push,
                    &[A::Push, A::Skip],
                ),
                clash,
                planned(
                    "backup",
                    RemoteRelation::RemoteAhead(3),
                    A::FastForwardFrom,
                    &[A::FastForwardFrom, A::Skip],
                ),
                mirror,
                down,
            ]));
        }
        2 => {
            with(|c| {
                c.open(
                    Path::new("/repo"),
                    "feature",
                    Step::Plan(PlanView::loading(Reply::pending(), None, false)),
                );
            });
        }
        _ => {
            open_plan_with(three_ready());
            let _ = on_key(ctrl(KeyCode::Enter));
            on_progress("r", "origin", TargetStatus::Pushed { created: false });
            on_progress("r", "gitlab", TargetStatus::Integrating);
        }
    }
}

/// Only a plain push (or a skipped twin) runs without a decision; a pull
/// takes fast-forwards and, unless the mode is fast-forward only, merges.
#[test]
fn only_plain_rows_run_unattended() {
    let _s = serial();
    let view = PlanView::from_plan(three_ready(), false);
    assert!(view.unattended());
    let diverged = plan_of(vec![
        planned(
            "origin",
            RemoteRelation::LocalAhead(1),
            A::Push,
            &[A::Push, A::Skip],
        ),
        planned(
            "gitlab",
            RemoteRelation::Diverged { ahead: 1, behind: 1 },
            A::MergeFrom,
            &[A::MergeFrom, A::Skip],
        ),
    ]);
    assert!(!PlanView::from_plan(diverged, false).unattended());
    let mut twin = planned("gitlab", RemoteRelation::UpToDate, A::Skip, &[A::Skip]);
    twin.same_url_as = Some("origin".into());
    let with_twin = plan_of(vec![
        planned(
            "origin",
            RemoteRelation::LocalAhead(1),
            A::Push,
            &[A::Push, A::Skip],
        ),
        twin,
    ]);
    assert!(PlanView::from_plan(with_twin, false).unattended());
    let mut unreachable = planned("origin", RemoteRelation::LocalAhead(1), A::Skip, &[A::Skip]);
    unreachable.error = Some("no route".into());
    assert!(!PlanView::from_plan(plan_of(vec![unreachable]), false).unattended());

    let mut pull = plan_of(vec![
        planned(
            "origin",
            RemoteRelation::RemoteAhead(1),
            A::FastForwardFrom,
            &[A::FastForwardFrom, A::Skip],
        ),
        planned(
            "gitlab",
            RemoteRelation::Diverged { ahead: 1, behind: 1 },
            A::MergeFrom,
            &[A::MergeFrom, A::RebaseOnto, A::Skip],
        ),
    ]);
    pull.mode = GroupMode::Pull;
    let mut view = PlanView::from_plan(pull, false);
    assert!(view.unattended(), "merge and fast-forward are plain pulls");
    view.ff_only = true;
    assert!(!view.unattended(), "ff-only never merges by itself");
}
