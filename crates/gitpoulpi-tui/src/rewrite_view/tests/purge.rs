//! The purge screen (slice S15, TUI): the checklist with what blocks, the
//! confirmation, the run and its outcome. Keys and clicks reach the same
//! methods; the daemon's answers are put in by hand.

use super::render::{paint, rect_of, text};
use super::*;
use crate::daemon_link::Reply;
use gitpoulpi_core::domain::rewrite::{
    HostChecklist, HostKind, HostStep, PurgeBlocker, PurgeBlockerKind, PurgeCheck, PurgeEntryInfo,
    PurgeOutcome, PurgeStatus,
};

use super::super::purge::{fix_of, Fix, PurgeConfirm};

const W: u16 = 110;
const H: u16 = 40;

fn blocker(kind: PurgeBlockerKind, name: &str) -> PurgeBlocker {
    PurgeBlocker {
        kind,
        name: name.into(),
        commits: 3,
    }
}

fn outcome(blockers: Vec<PurgeBlocker>) -> PurgeOutcome {
    PurgeOutcome {
        status: if blockers.is_empty() {
            PurgeStatus::Ready
        } else {
            PurgeStatus::Blocked
        },
        blockers,
        old_commits: 12,
        old_blobs: 30,
        old_objects: 55,
        reflogs: vec!["HEAD".into(), "refs/heads/main".into()],
        reflog_entries: 4,
        pins_dropped: 0,
        runs_removed: 0,
        remaining: 0,
    }
}

fn check(blockers: Vec<PurgeBlocker>) -> PurgeCheck {
    PurgeCheck {
        entries: vec![
            PurgeEntryInfo {
                id: 7,
                label: "Rewrite history".into(),
                rewrite: true,
            },
            PurgeEntryInfo {
                id: 5,
                label: "Stage file".into(),
                rewrite: false,
            },
        ],
        already_purged: Vec::new(),
        outcome: outcome(blockers),
        hosts: vec![HostChecklist::new("origin", "git@github.com:acme/app.git")],
        prompt: String::new(),
    }
}

fn blocked() -> Vec<PurgeBlocker> {
    vec![
        blocker(PurgeBlockerKind::Stash, "stash@{0}"),
        blocker(PurgeBlockerKind::RemoteTracking, "refs/remotes/origin/main"),
        blocker(PurgeBlockerKind::Worktree, "/tmp/other"),
    ]
}

/// A purge screen whose first check came back with `check`.
fn opened(check: PurgeCheck) -> (RewriteView, RawQueue, tempfile::TempDir) {
    let (mut v, q, t) = view();
    v.open_purge(false);
    answer_check(&mut v, check);
    (v, q, t)
}

fn answer_check(v: &mut RewriteView, check: PurgeCheck) {
    v.purge.as_mut().unwrap().check_wait = Some(Reply::ready(Ok(check)));
    v.poll(None);
}

fn screen(v: &RewriteView) -> &super::super::purge::PurgeScreen {
    v.purge.as_ref().unwrap()
}

fn click(v: &mut RewriteView, hit: RewriteHit) -> RewriteKeyOutcome {
    let (_, hits, _) = paint(v, W, H, None);
    let at = rect_of(&hits, hit);
    v.on_press(hit, at.x, at.y)
}

fn applied() -> (RewriteView, RawQueue, tempfile::TempDir) {
    let (mut v, q, t) = view();
    with_identity_rule(&mut v);
    v.finish_apply(RewriteOutcome {
        run: RunId("run-1".into()),
        moved: Vec::new(),
        commits_changed: 12,
    });
    (v, q, t)
}

#[test]
fn a_blocked_checklist_lists_what_holds_the_old_history() {
    let (v, _q, _t) = opened(check(blocked()));
    let s = text(&v);
    assert!(s.contains("Purge old history"), "{s}");
    assert!(s.contains("#7 Rewrite history"), "{s}");
    assert!(s.contains("#5 Stage file (also loses its undo)"), "{s}");
    assert!(s.contains("12 old"), "{s}");
    assert!(s.contains("Still holding the old history"), "{s}");
    assert!(s.contains("stash@{0}"), "{s}");
    assert!(s.contains("Worktree /tmp/other"), "{s}");
    assert!(s.contains("On origin (GitHub)"), "{s}");
    assert!(!s.contains("Esc  "), "no Esc label beside the cross: {s}");
    insta::assert_snapshot!("blocked", s);
}

#[test]
fn a_ready_checklist_says_so_and_enables_purge() {
    let (v, _q, _t) = opened(check(Vec::new()));
    let s = text(&v);
    assert!(s.contains("the purge can run"), "{s}");
    assert!(!s.contains("Still holding"), "{s}");
    assert!(screen(&v).can_purge());
    insta::assert_snapshot!("ready", s);
}

#[test]
fn a_blocked_checklist_keeps_purge_disabled_for_keys_and_clicks() {
    let (mut v, _q, _t) = opened(check(blocked()));
    assert!(!screen(&v).can_purge());
    v.on_key(key(KeyCode::Char('p')));
    assert!(screen(&v).confirm.is_none(), "the key does nothing");
    click(&mut v, RewriteHit::PurgeRun);
    assert!(screen(&v).confirm.is_none(), "the click does nothing");
}

#[test]
fn the_confirmation_states_what_is_lost_and_has_no_default() {
    let (mut v, _q, _t) = opened(check(Vec::new()));
    v.on_key(key(KeyCode::Char('p')));
    assert_eq!(
        screen(&v).confirm,
        Some(PurgeConfirm::Purge { cursor: None })
    );
    let s = paint(&v, W, H, None).0;
    assert!(s.contains("Purge for good?"), "{s}");
    assert!(s.contains("Undo is lost for 2 operations."), "{s}");
    assert!(s.contains("1  Purge for good"), "{s}");
    assert!(s.contains("2  Do not purge"), "{s}");
    assert!(s.contains("cannot be undone"), "{s}");
    insta::assert_snapshot!("confirm", s);
    // Enter with no choice made does nothing.
    v.on_key(key(KeyCode::Enter));
    assert!(screen(&v).confirm.is_some() && !screen(&v).running());
}

#[test]
fn declining_the_confirmation_changes_nothing() {
    for decline in [key(KeyCode::Char('2')), key(KeyCode::Esc)] {
        let (mut v, _q, _t) = opened(check(Vec::new()));
        v.on_key(key(KeyCode::Char('p')));
        v.on_key(decline);
        assert!(screen(&v).confirm.is_none() && !screen(&v).running());
    }
    // The mouse: "Do not purge", the cross and the rest of the workbench.
    for hit in [
        RewriteHit::PurgeAnswer(1),
        RewriteHit::PurgeCancel,
        RewriteHit::PurgeShield,
    ] {
        let (mut v, _q, _t) = opened(check(Vec::new()));
        v.on_key(key(KeyCode::Char('p')));
        click(&mut v, hit);
        assert!(
            screen(&v).confirm.is_none() && !screen(&v).running(),
            "{hit:?}"
        );
    }
}

#[test]
fn confirming_by_key_or_click_starts_the_run_once() {
    let (mut by_key, _q1, _t1) = opened(check(Vec::new()));
    by_key.on_key(key(KeyCode::Char('p')));
    by_key.on_key(key(KeyCode::Char('1')));
    assert!(screen(&by_key).running());

    let (mut by_arrows, _q2, _t2) = opened(check(Vec::new()));
    by_arrows.on_key(key(KeyCode::Char('p')));
    by_arrows.on_key(key(KeyCode::Down));
    by_arrows.on_key(key(KeyCode::Up));
    assert!(matches!(
        screen(&by_arrows).confirm,
        Some(PurgeConfirm::Purge { cursor: Some(0) })
    ));
    by_arrows.on_key(key(KeyCode::Enter));
    assert!(screen(&by_arrows).running());

    let (mut by_mouse, _q3, _t3) = opened(check(Vec::new()));
    click(&mut by_mouse, RewriteHit::PurgeRun);
    click(&mut by_mouse, RewriteHit::PurgeAnswer(0));
    assert!(screen(&by_mouse).running());
}

#[test]
fn the_run_shows_its_step_and_says_it_cannot_be_stopped() {
    let (mut v, _q, _t) = opened(check(Vec::new()));
    v.on_key(key(KeyCode::Char('p')));
    v.on_key(key(KeyCode::Char('1')));
    v.purge.as_mut().unwrap().progress = Some((Some("Expiring reflogs".into()), Some(40)));
    let s = paint(&v, W, H, None).0;
    assert!(s.contains("Purging: Expiring reflogs (40%)"), "{s}");
    assert!(s.contains("cannot be stopped"), "{s}");
    assert!(!s.contains("Check again"), "no buttons while it runs: {s}");
    // Nothing closes it, by key or mouse, and the window is busy.
    assert!(v.busy() && v.purging());
    assert_eq!(v.on_key(key(KeyCode::Esc)), RewriteKeyOutcome::Stay);
    assert_eq!(v.request_close(), RewriteKeyOutcome::Stay);
    assert_eq!(v.outside_press(), RewriteKeyOutcome::Stay);
    assert_eq!(
        click(&mut v, RewriteHit::PurgeClose),
        RewriteKeyOutcome::Stay
    );
    assert!(v.purge_open());
    insta::assert_snapshot!("running", s);
}

fn finished(status: PurgeStatus, remaining: u32) -> PurgeOutcome {
    PurgeOutcome {
        status,
        remaining,
        ..outcome(Vec::new())
    }
}

#[test]
fn a_purged_outcome_says_the_old_history_is_gone() {
    let (mut v, _q, _t) = opened(check(Vec::new()));
    v.on_key(key(KeyCode::Char('p')));
    v.on_key(key(KeyCode::Char('1')));
    v.purge.as_mut().unwrap().run_wait = Some(Reply::ready(Ok(finished(PurgeStatus::Purged, 0))));
    v.poll(None);
    assert!(!screen(&v).running());
    let s = paint(&v, W, H, None).0;
    assert!(
        s.contains("The old history is gone from this clone: 55 objects"),
        "{s}"
    );
    assert!(!s.contains("Still holding"), "{s}");
    assert!(s.contains("On origin (GitHub)"), "the host steps stay: {s}");
    assert!(!s.contains("[Purge]") && !s.contains("Check again"), "{s}");
    insta::assert_snapshot!("outcome_done", s);
}

#[test]
fn an_incomplete_outcome_counts_what_is_left() {
    let (mut v, _q, _t) = opened(check(Vec::new()));
    v.on_key(key(KeyCode::Char('p')));
    v.on_key(key(KeyCode::Char('1')));
    v.purge.as_mut().unwrap().run_wait =
        Some(Reply::ready(Ok(finished(PurgeStatus::Incomplete, 8))));
    v.poll(None);
    let s = text(&v);
    assert!(s.contains("8 old objects are still in this clone."), "{s}");
}

#[test]
fn a_blocked_answer_goes_back_to_the_list_with_a_new_check() {
    let (mut v, _q, _t) = opened(check(Vec::new()));
    v.on_key(key(KeyCode::Char('p')));
    v.on_key(key(KeyCode::Char('1')));
    let mut blocked_answer = finished(PurgeStatus::Blocked, 0);
    blocked_answer.blockers = blocked();
    v.purge.as_mut().unwrap().run_wait = Some(Reply::ready(Ok(blocked_answer)));
    v.poll(None);
    let s = screen(&v);
    assert!(s.outcome.is_none());
    assert!(s.check_wait.is_some(), "the list is checked again");
    assert!(s
        .error
        .as_deref()
        .is_some_and(|e| e.contains("nothing was changed")));
}

#[test]
fn a_failed_check_is_shown_and_the_screen_stays_usable() {
    let (mut v, _q, _t) = view();
    v.open_purge(false);
    v.purge.as_mut().unwrap().check_wait =
        Some(Reply::ready(Err(CoreError::Backend("undo is off".into()))));
    v.poll(None);
    let s = text(&v);
    assert!(s.contains("undo is off"), "{s}");
    assert!(!screen(&v).can_purge());
}

#[test]
fn every_fix_has_a_key_and_a_click_that_do_the_same() {
    // Fetch: Enter on the remote-tracking row, or its button.
    let rt = vec![blocker(
        PurgeBlockerKind::RemoteTracking,
        "refs/remotes/origin/main",
    )];
    let (mut by_key, _q1, _t1) = opened(check(rt.clone()));
    by_key.on_key(key(KeyCode::Enter));
    assert!(screen(&by_key).fix_wait.is_some());
    let (mut by_mouse, _q2, _t2) = opened(check(rt));
    click(&mut by_mouse, RewriteHit::PurgeFix(0));
    assert!(screen(&by_mouse).fix_wait.is_some());

    // Drop: a confirmation first (it is work), then the call.
    let st = vec![blocker(PurgeBlockerKind::Stash, "stash@{2}")];
    let (mut by_key, _q3, _t3) = opened(check(st.clone()));
    by_key.on_key(key(KeyCode::Enter));
    assert!(matches!(
        &screen(&by_key).confirm,
        Some(PurgeConfirm::Drop {
            id: 2,
            cursor: None,
            ..
        })
    ));
    assert!(screen(&by_key).fix_wait.is_none(), "nothing dropped yet");
    let s = paint(&by_key, W, H, None).0;
    assert!(s.contains("Drop stash@{2}?"), "{s}");
    assert!(s.contains("1  Drop it") && s.contains("2  Keep it"), "{s}");
    insta::assert_snapshot!("drop_confirm", s);
    by_key.on_key(key(KeyCode::Char('1')));
    assert!(screen(&by_key).fix_wait.is_some());

    let (mut by_mouse, _q4, _t4) = opened(check(st));
    click(&mut by_mouse, RewriteHit::PurgeFix(0));
    assert!(screen(&by_mouse).confirm.is_some());
    click(&mut by_mouse, RewriteHit::PurgeAnswer(0));
    assert!(screen(&by_mouse).fix_wait.is_some());

    // Keeping the stash drops nothing.
    let (mut keep, _q5, _t5) = opened(check(vec![blocker(PurgeBlockerKind::Stash, "stash@{0}")]));
    keep.on_key(key(KeyCode::Enter));
    keep.on_key(key(KeyCode::Char('2')));
    assert!(screen(&keep).confirm.is_none() && screen(&keep).fix_wait.is_none());
}

#[test]
fn a_fix_is_followed_by_a_new_check() {
    let (mut v, _q, _t) = opened(check(blocked()));
    v.on_key(key(KeyCode::Down));
    v.on_key(key(KeyCode::Enter));
    assert!(screen(&v).fix_wait.is_some());
    v.purge.as_mut().unwrap().fix_wait = Some(Reply::ready(Ok(serde_json::Value::Null)));
    v.poll(None);
    assert!(screen(&v).fix_wait.is_none());
    assert!(screen(&v).check_wait.is_some(), "re-checked after the fix");
    answer_check(
        &mut v,
        check(vec![blocker(PurgeBlockerKind::Worktree, "/tmp/other")]),
    );
    assert_eq!(screen(&v).blockers().len(), 1);
    assert_eq!(screen(&v).cursor, 0, "the cursor stays on a row");
}

#[test]
fn a_failed_fix_says_why_and_checks_again() {
    let (mut v, _q, _t) = opened(check(vec![blocker(
        PurgeBlockerKind::RemoteTracking,
        "refs/remotes/origin/main",
    )]));
    v.on_key(key(KeyCode::Enter));
    v.purge.as_mut().unwrap().fix_wait =
        Some(Reply::ready(Err(CoreError::Backend("no network".into()))));
    v.poll(None);
    let s = screen(&v);
    assert!(s.error.as_deref().is_some_and(|e| e.contains("no network")));
    assert!(s.check_wait.is_some());
}

#[test]
fn check_again_is_a_key_and_a_button() {
    let (mut by_key, _q1, _t1) = opened(check(blocked()));
    by_key.on_key(key(KeyCode::Char('r')));
    assert!(screen(&by_key).check_wait.is_some());
    let (mut by_mouse, _q2, _t2) = opened(check(blocked()));
    click(&mut by_mouse, RewriteHit::PurgeCheck);
    assert!(screen(&by_mouse).check_wait.is_some());
}

#[test]
fn rows_without_a_fix_copy_their_name_by_key_and_click() {
    let wt = vec![blocker(PurgeBlockerKind::Worktree, "/tmp/other")];
    assert_eq!(fix_of(&wt[0]), Fix::None);
    let (mut by_key, _q1, _t1) = opened(check(wt.clone()));
    by_key.on_key(key(KeyCode::Char('c')));
    assert!(by_key.notice.is_some());
    let (mut by_mouse, _q2, _t2) = opened(check(wt));
    click(&mut by_mouse, RewriteHit::PurgeCopy(0));
    assert!(by_mouse.notice.is_some());
    assert_eq!(by_key.notice.is_some(), by_mouse.notice.is_some());
}

#[test]
fn the_host_link_is_copyable() {
    let (mut v, _q, _t) = opened(check(Vec::new()));
    let tall = |v: &RewriteView| paint(v, W, 80, None);
    let (s, hits, _) = tall(&v);
    assert!(s.contains("Copy link"), "GitHub has a support link: {s}");
    let at = rect_of(&hits, RewriteHit::PurgeLink(0));
    v.on_press(RewriteHit::PurgeLink(0), at.x, at.y);
    assert!(v.notice.is_some());
}

#[test]
fn fixes_and_hosts_are_mapped_per_kind() {
    assert_eq!(
        fix_of(&blocker(PurgeBlockerKind::RemoteTracking, "x")),
        Fix::Fetch
    );
    assert_eq!(
        fix_of(&blocker(PurgeBlockerKind::Stash, "stash@{4}")),
        Fix::Drop(4)
    );
    assert_eq!(
        fix_of(&blocker(PurgeBlockerKind::Stash, "weird")),
        Fix::None
    );
    for kind in [
        PurgeBlockerKind::Branch,
        PurgeBlockerKind::Tag,
        PurgeBlockerKind::Worktree,
        PurgeBlockerKind::UndoPin,
        PurgeBlockerKind::Notes,
        PurgeBlockerKind::Replace,
        PurgeBlockerKind::Other,
    ] {
        assert_eq!(fix_of(&blocker(kind, "x")), Fix::None);
    }
}

#[test]
fn every_blocker_kind_and_host_step_has_wording() {
    let kinds = [
        PurgeBlockerKind::Branch,
        PurgeBlockerKind::Tag,
        PurgeBlockerKind::RemoteTracking,
        PurgeBlockerKind::Stash,
        PurgeBlockerKind::Worktree,
        PurgeBlockerKind::UndoPin,
        PurgeBlockerKind::Notes,
        PurgeBlockerKind::Replace,
        PurgeBlockerKind::Other,
    ];
    let blockers = kinds.iter().map(|k| blocker(*k, "NAME")).collect();
    let (v, _q, _t) = opened(check(blockers));
    let s = paint(&v, 120, 80, None).0;
    assert_eq!(s.matches("NAME").count(), kinds.len(), "{s}");
    // Every host family renders its title.
    for (url, title) in [
        ("https://gitlab.example.org/a/b", "GitLab"),
        ("https://dev.azure.com/o/p/_git/r", "Azure DevOps"),
        ("https://bitbucket.org/a/b.git", "Bitbucket"),
        ("/srv/git/x.git", "other host"),
    ] {
        let mut c = check(Vec::new());
        c.hosts = vec![HostChecklist::new("up", url)];
        assert_ne!(c.hosts[0].host, HostKind::GitHub);
        let (v, _q, _t) = opened(c);
        let s = paint(&v, 120, 80, None).0;
        assert!(s.contains(&format!("On up ({title})")), "{title}: {s}");
        assert!(s.contains("Revoke the secret"), "{s}");
        let _ = HostStep::RotateSecret;
    }
}

#[test]
fn the_cross_esc_and_an_outside_click_close_the_screen_back_to_the_workbench() {
    let (mut v, _q, _t) = opened(check(blocked()));
    assert_eq!(v.on_key(key(KeyCode::Esc)), RewriteKeyOutcome::Stay);
    assert!(!v.purge_open(), "Esc");
    for how in 0..3 {
        let (mut v, _q, _t) = opened(check(blocked()));
        let out = match how {
            0 => click(&mut v, RewriteHit::PurgeClose),
            1 => click(&mut v, RewriteHit::PurgeShield),
            _ => v.outside_press(),
        };
        assert_eq!(out, RewriteKeyOutcome::Stay);
        assert!(!v.purge_open(), "way {how}");
    }
}

#[test]
fn a_purge_opened_from_the_palette_closes_the_workbench_too() {
    let (v, _q, _t) = view();
    let mut v = v.with_purge();
    assert!(v.purge_open());
    assert_eq!(v.on_key(key(KeyCode::Esc)), RewriteKeyOutcome::Close);
    let mut v = view().0.with_purge();
    assert_eq!(
        click(&mut v, RewriteHit::PurgeClose),
        RewriteKeyOutcome::Close
    );
}

#[test]
fn the_cross_is_drawn_without_an_esc_label() {
    let (v, _q, _t) = opened(check(blocked()));
    let (s, hits, _) = paint(&v, W, H, None);
    let cross = rect_of(&hits, RewriteHit::PurgeClose);
    assert!(cross.width > 0);
    assert!(!s.lines().next().unwrap_or("").contains("Esc"));
}

#[test]
fn the_result_panel_offers_the_purge_by_button_and_key() {
    let (v, _q, _t) = applied();
    let s = text(&v);
    assert!(s.contains("Purge old history..."), "{s}");
    insta::assert_snapshot!("result_panel_button", s);
    let (mut by_key, _q1, _t1) = applied();
    by_key.on_key(key(KeyCode::Char('e')));
    assert!(by_key.purge_open() && !screen(&by_key).standalone);
    let (mut by_mouse, _q2, _t2) = applied();
    click(&mut by_mouse, RewriteHit::PurgeOpen);
    assert!(by_mouse.purge_open());
    // The screen asked its check on opening.
    assert!(screen(&by_key).check_wait.is_some());
}

#[test]
fn the_wheel_and_the_keys_scroll_a_long_checklist() {
    let many: Vec<PurgeBlocker> = (0..30)
        .map(|i| blocker(PurgeBlockerKind::Branch, &format!("refs/heads/b{i}")))
        .collect();
    let (mut v, _q, _t) = opened(check(many));
    let (first, _, _) = paint(&v, W, 30, None);
    assert!(first.contains("b0") && !first.contains("b29"), "{first}");
    for _ in 0..29 {
        v.on_key(key(KeyCode::Down));
    }
    let (last, _, _) = paint(&v, W, 30, None);
    assert!(last.contains("b29"), "the cursor is followed: {last}");
    v.purge_wheel(-1000);
    let (top, _, _) = paint(&v, W, 30, None);
    assert!(top.contains("b0"), "the wheel scrolls: {top}");
}
