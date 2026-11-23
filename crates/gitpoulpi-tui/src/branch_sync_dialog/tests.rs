use super::*;
use crate::daemon_link::RawQueue;
use crate::test_support::{local_branch_at, oid, remote_branch_at};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

/// The dialog is process-wide: tests that touch it take turns.
static SERIAL: Mutex<()> = Mutex::new(());

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn refs_with_behind_feature() -> Vec<Ref> {
    let mut main = local_branch_at("main", oid(1));
    main.is_head = true;
    main.upstream = Some("origin/main".into());
    let mut feature = local_branch_at("feature/x", oid(2));
    feature.upstream = Some("origin/feature/x".into());
    let mut same = local_branch_at("orbit", oid(3));
    same.upstream = Some("origin/orbit".into());
    vec![
        main,
        feature,
        same,
        local_branch_at("loose", oid(4)),
        remote_branch_at("origin", "main", oid(9)),
        remote_branch_at("origin", "feature/x", oid(7)),
        remote_branch_at("origin", "orbit", oid(3)),
    ]
}

fn worktree(path: &str, branch: &str) -> Worktree {
    Worktree {
        path: path.into(),
        branch: Some(branch.into()),
        is_main: false,
        is_locked: false,
    }
}

fn install(stage: Stage) {
    *state() = Some(Pending {
        root: "/repo".into(),
        spec: RunSpec {
            head_mode: None,
            fetch_opts: FetchOptions::default(),
        },
        stage,
    });
}

fn asking(list: Vec<(&str, &str)>) {
    install(Stage::Asking(Open {
        list: list
            .into_iter()
            .map(|(b, p)| Elsewhere {
                branch: b.into(),
                path: p.into(),
            })
            .collect(),
        selected: 0,
        rows: Vec::new(),
        close: Default::default(),
    }));
}

fn run(worktrees: WorktreePolicy) -> Action {
    Action::PullAllBranches(PullAllBranchesRequest {
        fetch: true,
        fetch_opts: FetchOptions::default(),
        head_mode: None,
        worktrees,
    })
}

fn screen() -> String {
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    terminal.draw(render).unwrap();
    dialog_kit::buffer_text(terminal.backend().buffer())
}

#[test]
fn only_branches_that_differ_from_their_upstream_are_candidates() {
    let found = candidates(&refs_with_behind_feature());
    assert_eq!(
        found,
        vec![Candidate {
            branch: "feature/x".into(),
            tip: oid(2),
            upstream_tip: oid(7),
        }],
        "the checked-out branch, an up to date one and one with no upstream are left out"
    );
}

#[test]
fn nothing_to_check_starts_the_run_at_once_skipping_worktrees() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    *state() = None;
    let queue = RawQueue::default();
    let refs = vec![local_branch_at("main", oid(1))];
    let action = start(
        &queue.requester(),
        Path::new("/repo"),
        &refs,
        Some(PullMode::Rebase),
        FetchOptions::default(),
    );
    assert_eq!(
        action,
        Some(Action::PullAllBranches(PullAllBranchesRequest {
            fetch: true,
            fetch_opts: FetchOptions::default(),
            head_mode: Some(PullMode::Rebase),
            worktrees: WorktreePolicy::Skip,
        }))
    );
    assert!(queue.queued().is_empty());
    assert!(!is_open() && !is_checking());
}

#[test]
fn a_candidate_asks_the_daemon_for_the_worktrees_first() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    *state() = None;
    let queue = RawQueue::default();
    let action = start(
        &queue.requester(),
        Path::new("/repo"),
        &refs_with_behind_feature(),
        None,
        FetchOptions::default(),
    );
    assert_eq!(action, None, "nothing is sent before the check ends");
    let asked = queue.queued();
    assert_eq!(asked.len(), 1);
    assert_eq!(asked[0].0, OperationKind::Worktrees);
    assert!(is_checking());
    *state() = None;
}

#[test]
fn a_behind_branch_in_another_worktree_opens_the_question() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let queue = RawQueue::default();
    install(Stage::Worktrees {
        reply: Reply::ready(Ok(vec![
            worktree("/work/feature", "feature/x"),
            worktree("/work/other", "unrelated"),
        ])),
        candidates: candidates(&refs_with_behind_feature()),
    });
    assert_eq!(poll(Path::new("/repo"), &queue.requester()), None);
    // The merge base of the candidate is asked for.
    let asked = queue.queued();
    assert_eq!(asked.len(), 1);
    assert_eq!(asked[0].0, OperationKind::MergeBase);
    // Its answer: the base is the branch's own tip, so it is only behind. The
    // probe was queued, not answered: stand in for the daemon.
    let mut slot = state();
    let Some(Pending {
        stage: Stage::Probing { probes, .. },
        ..
    }) = slot.as_mut()
    else {
        panic!("expected the merge base probe");
    };
    probes[0].2 = Reply::ready(Ok(Some(oid(2))));
    drop(slot);
    assert_eq!(poll(Path::new("/repo"), &queue.requester()), None);
    assert!(is_open());
    *state() = None;
}

#[test]
fn a_diverged_branch_in_another_worktree_does_not_open_the_question() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let queue = RawQueue::default();
    install(Stage::Probing {
        probes: vec![(
            Elsewhere {
                branch: "feature/x".into(),
                path: "/work/feature".into(),
            },
            oid(2),
            // Merge base is some older commit: diverged.
            Reply::ready(Ok(Some(oid(1)))),
        )],
        settled: Vec::new(),
    });
    assert_eq!(
        poll(Path::new("/repo"), &queue.requester()),
        Some(run(WorktreePolicy::Skip))
    );
    assert!(!is_open() && !is_checking());
}

#[test]
fn no_branch_in_another_worktree_runs_at_once() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let queue = RawQueue::default();
    install(Stage::Worktrees {
        reply: Reply::ready(Ok(vec![worktree("/work/other", "unrelated")])),
        candidates: candidates(&refs_with_behind_feature()),
    });
    assert_eq!(
        poll(Path::new("/repo"), &queue.requester()),
        Some(run(WorktreePolicy::Skip))
    );
}

#[test]
fn another_repository_does_not_advance_the_check() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let queue = RawQueue::default();
    install(Stage::Worktrees {
        reply: Reply::ready(Ok(vec![])),
        candidates: candidates(&refs_with_behind_feature()),
    });
    assert_eq!(poll(Path::new("/elsewhere"), &queue.requester()), None);
    assert!(is_checking());
    *state() = None;
}

#[test]
fn the_question_lists_the_branches_and_their_worktrees() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    asking(vec![
        ("feature/x", "/work/feature"),
        ("orbit", "/work/orbit"),
    ]);
    let text = screen();
    assert!(text.contains("Pull all branches"), "{text}");
    assert!(text.contains("feature/x  in  /work/feature"), "{text}");
    assert!(text.contains("orbit  in  /work/orbit"), "{text}");
    assert!(text.contains("1  Update those worktrees too"), "{text}");
    assert!(text.contains("2  Skip them"), "{text}");
    assert!(text.contains("3  Cancel"), "{text}");
    *state() = None;
}

#[test]
fn the_digits_answer_the_rows() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    asking(vec![("feature/x", "/work/feature")]);
    assert_eq!(
        on_key_action(key(KeyCode::Char('1'))),
        Some(run(WorktreePolicy::Update))
    );
    assert!(!is_open(), "an answer closes the question");
    asking(vec![("feature/x", "/work/feature")]);
    assert_eq!(
        on_key_action(key(KeyCode::Char('2'))),
        Some(run(WorktreePolicy::Skip))
    );
    asking(vec![("feature/x", "/work/feature")]);
    assert_eq!(on_key_action(key(KeyCode::Char('3'))), None);
    assert!(!is_open(), "Cancel closes the question and sends nothing");
}

#[test]
fn arrows_tab_and_enter_answer_the_highlighted_row() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    asking(vec![("feature/x", "/work/feature")]);
    assert_eq!(on_key_action(key(KeyCode::Down)), None);
    assert_eq!(
        on_key_action(key(KeyCode::Enter)),
        Some(run(WorktreePolicy::Skip)),
        "Down moves to Skip"
    );
    asking(vec![("feature/x", "/work/feature")]);
    assert_eq!(on_key_action(key(KeyCode::Tab)), None);
    assert_eq!(on_key_action(key(KeyCode::Tab)), None);
    assert_eq!(on_key_action(key(KeyCode::Tab)), None, "stops at Cancel");
    assert_eq!(on_key_action(key(KeyCode::Enter)), None);
    assert!(!is_open(), "Enter on Cancel closes without a run");
    asking(vec![("feature/x", "/work/feature")]);
    assert_eq!(
        on_key_action(key(KeyCode::Enter)),
        Some(run(WorktreePolicy::Update)),
        "Update is the highlighted row at first"
    );
}

#[test]
fn escape_cancels() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    asking(vec![("feature/x", "/work/feature")]);
    assert_eq!(on_key_action(key(KeyCode::Esc)), None);
    assert!(!is_open());
}

#[test]
fn a_click_on_a_row_answers_it_and_the_close_cross_cancels() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    asking(vec![("feature/x", "/work/feature")]);
    screen();
    let rows = match state().as_ref().map(|p| &p.stage) {
        Some(Stage::Asking(open)) => open.rows.clone(),
        _ => panic!("expected the question"),
    };
    let at = |r: Rect| MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: r.x + 3,
        row: r.y,
        modifiers: KeyModifiers::NONE,
    };
    assert_eq!(
        on_mouse_action(at(rows[1])),
        Some(run(WorktreePolicy::Skip))
    );
    assert!(!is_open());
    asking(vec![("feature/x", "/work/feature")]);
    screen();
    let cross = match state().as_ref().map(|p| &p.stage) {
        Some(Stage::Asking(open)) => open.close.rect().expect("a painted cross"),
        _ => panic!("expected the question"),
    };
    assert_eq!(
        on_mouse_action(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: cross.x,
            row: cross.y,
            modifiers: KeyModifiers::NONE,
        }),
        None
    );
    assert!(!is_open(), "the cross cancels");
}

#[test]
fn a_frame_too_small_for_the_question_still_offers_the_cross() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    asking(vec![("feature/x", "/work/feature")]);
    let mut terminal = Terminal::new(TestBackend::new(24, 8)).unwrap();
    terminal.draw(render).unwrap();
    assert!(is_open());
    *state() = None;
}
