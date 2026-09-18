//! The result panel's push list, the answer of a push, the note for
//! collaborators and the maps (slice S13, TUI). Keys and clicks reach the
//! same methods; the daemon's answers are put in by hand.

use super::render::{paint, rect_of, text};
use super::*;
use crate::daemon_link::Reply;
use gitpoulpi_core::domain::rewrite::{
    note_text as core_note_text, NoteRef, NoteRefKind, PushRefResult, PushRefStatus,
    PushRewrittenOutcome, RewriteMaps, RewriteNote,
};

const W: u16 = 110;
const H: u16 = 34;

fn on_origin(name: &str, protected: bool) -> RefChange {
    RefChange {
        name: name.into(),
        old: Oid([1; 20]),
        new: Some(Oid([2; 20])),
        pushed: PushState::OnRemote {
            remote: "origin".into(),
            remote_oid: Oid([1; 20]),
            rewritten_on_remote: 3,
        },
        protected,
        included: true,
        checked_out_in: None,
    }
}

fn local_only(name: &str) -> RefChange {
    RefChange {
        pushed: PushState::LocalOnly,
        ..on_origin(name, false)
    }
}

/// An apply that moved a protected `main`, a `feature` and a tag that are on
/// origin, and a branch that never left this machine.
fn finished() -> (RewriteView, RawQueue, tempfile::TempDir) {
    let (mut v, q, t) = view();
    with_identity_rule(&mut v);
    v.finish_apply(RewriteOutcome {
        run: RunId("run-1".into()),
        moved: vec![
            on_origin("refs/heads/main", true),
            on_origin("refs/heads/feature", false),
            on_origin("refs/tags/v1", false),
            local_only("refs/heads/scratch"),
        ],
        commits_changed: 412,
    });
    (v, q, t)
}

fn result(name: &str, status: PushRefStatus, detail: Option<&str>) -> PushRefResult {
    PushRefResult {
        name: name.into(),
        remote: "origin".into(),
        expected_old: Oid([1; 20]),
        new: Some(Oid([2; 20])),
        status,
        detail: detail.map(str::to_string),
    }
}

fn answered(v: &mut RewriteView, out: PushRewrittenOutcome) {
    v.push.sending = Some(Reply::ready(Ok(out)));
    v.poll(None);
}

fn alt(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::ALT)
}

#[test]
fn the_list_offers_the_pushed_refs_with_the_protected_one_unticked() {
    let (v, _q, _t) = finished();
    let picks = &v.push.picks;
    assert_eq!(picks.len(), 3, "the local-only branch is not offered");
    assert!(
        picks[0].protected && !picks[0].ticked,
        "main starts unticked"
    );
    assert!(picks[1].ticked && picks[2].ticked);
    let s = text(&v);
    assert!(s.contains("[ ] main"), "{s}");
    assert!(s.contains("[x] feature"), "{s}");
    assert!(s.contains("protected"), "{s}");
    assert!(
        s.contains("The remote keeps the new history even if you undo"),
        "{s}"
    );
    assert!(!s.contains("Esc"), "{s}");
    insta::assert_snapshot!("list", s);
}

#[test]
fn the_request_names_the_ticked_refs_and_the_protected_ones_among_them() {
    let (mut v, _q, _t) = finished();
    let run = RunId("run-1".into());
    let req = v.push.request(&run);
    assert_eq!(
        req.refs.as_deref(),
        Some(&["refs/heads/feature".to_string(), "refs/tags/v1".to_string()][..])
    );
    assert!(req.include_protected.is_empty(), "main stays out: {req:?}");
    assert!(!req.include_all_protected && !req.dry_run);
    v.check_all_picks();
    let req = v.push.request(&run);
    assert_eq!(req.refs.as_ref().map(Vec::len), Some(3));
    assert_eq!(req.include_protected, vec!["refs/heads/main".to_string()]);
    v.uncheck_all_picks();
    assert!(v.push.request(&run).include_protected.is_empty());
}

#[test]
fn keys_and_clicks_tick_the_same_boxes() {
    let (mut by_key, _q1, _t1) = finished();
    let (mut by_mouse, _q2, _t2) = finished();
    // Space on the first row, then Down and Space on the second.
    by_key.on_key(key(KeyCode::Char(' ')));
    by_key.on_key(key(KeyCode::Down));
    by_key.on_key(key(KeyCode::Char(' ')));
    let (_, hits, _) = paint(&by_mouse, W, H, None);
    for i in [0, 1] {
        let at = rect_of(&hits, RewriteHit::PushRow(i));
        by_mouse.on_press(RewriteHit::PushRow(i), at.x, at.y);
    }
    let ticks = |v: &RewriteView| v.push.picks.iter().map(|p| p.ticked).collect::<Vec<_>>();
    assert_eq!(ticks(&by_key), ticks(&by_mouse));
    assert_eq!(ticks(&by_key), [true, false, true]);

    // Check all / uncheck all: the chord and the buttons.
    let (mut a, _q3, _t3) = finished();
    let (mut b, _q4, _t4) = finished();
    a.on_key(alt('a'));
    b.on_press(RewriteHit::PushAll, 0, 0);
    assert_eq!(ticks(&a), ticks(&b));
    assert!(a.push.picks[0].ticked);
    a.on_key(alt('u'));
    b.on_press(RewriteHit::PushNone, 0, 0);
    assert_eq!(ticks(&a), ticks(&b));
    assert!(!a.push.picks[0].ticked);
    let (_, hits, _) = paint(&a, W, H, None);
    rect_of(&hits, RewriteHit::PushAll);
    rect_of(&hits, RewriteHit::PushNone);
}

#[test]
fn nothing_ticked_is_said_and_nothing_is_sent() {
    let (mut v, _q, _t) = finished();
    v.push.picks.iter_mut().for_each(|p| p.ticked = false);
    v.push_send();
    assert!(!v.pushing());
    assert!(v.notice.as_ref().is_some_and(|n| n.error));
    assert!(text(&v).contains("Tick at least one ref"));
}

#[test]
fn a_push_in_flight_blocks_closing_and_shows_it() {
    let (mut v, _q, _t) = finished();
    v.push_send();
    assert!(v.pushing() && v.busy());
    assert_eq!(v.request_close(), RewriteKeyOutcome::Stay);
    assert_eq!(v.on_key(key(KeyCode::Esc)), RewriteKeyOutcome::Stay);
    let (s, hits, _) = paint(&v, W, H, None);
    assert!(s.contains("Sending the rewritten history"), "{s}");
    // The button is there but cannot send a second push.
    rect_of(&hits, RewriteHit::PushSend);
    v.push_send();
    assert!(v.pushing());
}

#[test]
fn the_answer_is_one_row_per_ref_and_a_rejected_one_offers_fetch_and_review() {
    let (mut v, _q, _t) = finished();
    v.check_all_picks();
    answered(
        &mut v,
        PushRewrittenOutcome {
            run: RunId("run-1".into()),
            dry_run: false,
            results: vec![
                result(
                    "refs/heads/main",
                    PushRefStatus::RejectedProtectedByHost,
                    None,
                ),
                result("refs/heads/feature", PushRefStatus::Pushed, None),
                result("refs/tags/v1", PushRefStatus::RejectedRemoteMoved, None),
                result(
                    "refs/heads/odd",
                    PushRefStatus::RejectedOther,
                    Some("hook declined\nmore"),
                ),
                result("refs/heads/held", PushRefStatus::NotPushed, None),
                result("refs/heads/skipped", PushRefStatus::SkippedProtected, None),
            ],
            atomic_fallback: true,
        },
    );
    assert!(!v.pushing());
    let (s, hits, _) = paint(&v, W, 40, None);
    for want in [
        "rejected: protected by the host",
        "feature  origin  pushed",
        "rejected: the remote moved since the preview, fetch and review",
        "origin  rejected: hook declined",
        "not pushed",
        "skipped: protected branch",
        "refused an atomic push",
    ] {
        assert!(s.contains(want), "{want}\n{s}");
    }
    assert!(!s.contains("more"), "only git's first line: {s}");
    let fetches: Vec<usize> = hits
        .iter()
        .filter_map(|(_, h)| match h {
            RewriteHit::FetchReview(i) => Some(*i),
            _ => None,
        })
        .collect();
    assert_eq!(fetches, [0, 2, 3], "one action per rejected row");
    insta::assert_snapshot!("answer", paint(&v, W, 40, None).0);

    // The click and the key reach the same outcome.
    let at = rect_of(&hits, RewriteHit::FetchReview(2));
    assert_eq!(
        v.on_press(RewriteHit::FetchReview(2), at.x, at.y),
        RewriteKeyOutcome::FetchReview
    );
    v.push.cursor = 1;
    assert_eq!(v.on_key(key(KeyCode::Char('f'))), RewriteKeyOutcome::Stay);
    v.push.cursor = 2;
    assert_eq!(
        v.on_key(key(KeyCode::Char('f'))),
        RewriteKeyOutcome::FetchReview
    );
}

#[test]
fn a_push_that_found_nothing_to_send_says_so() {
    let (mut v, _q, _t) = finished();
    answered(
        &mut v,
        PushRewrittenOutcome {
            run: RunId("run-1".into()),
            dry_run: false,
            results: Vec::new(),
            atomic_fallback: false,
        },
    );
    assert!(text(&v).contains("No rewritten ref was on a remote"));
}

#[test]
fn a_failed_push_is_a_line_not_a_crash() {
    let (mut v, _q, _t) = finished();
    v.push_send();
    v.push.sending = Some(Reply::ready(Err(gitpoulpi_core::Error::Backend(
        "remote unreachable".into(),
    ))));
    v.poll(None);
    assert!(!v.pushing());
    let n = v.notice.as_ref().expect("a notice");
    assert!(n.error && n.text.contains("remote unreachable"), "{n:?}");
    assert!(v.push.sent.is_none(), "the ticks stay for another try");
}

#[test]
fn the_maps_are_two_copyable_paths() {
    let (mut v, _q, _t) = finished();
    v.open_maps();
    v.push.maps_wait = Some(Reply::ready(Ok(RewriteMaps {
        run: RunId("run-1".into()),
        dir: "/repo/.git/gitpoulpi/rewrite/run-1".into(),
        commit_map: "/repo/.git/gitpoulpi/rewrite/run-1/commit-map".into(),
        ref_map: "/repo/.git/gitpoulpi/rewrite/run-1/ref-map".into(),
        copied: false,
    })));
    v.poll(None);
    let (s, hits, _) = paint(&v, W, H, None);
    assert!(
        s.contains("Commit map  /repo/.git/gitpoulpi/rewrite/run-1/commit-map"),
        "{s}"
    );
    assert!(
        s.contains("Ref map  /repo/.git/gitpoulpi/rewrite/run-1/ref-map"),
        "{s}"
    );
    assert!(s.contains("Click a path to copy it."), "{s}");
    rect_of(&hits, RewriteHit::MapCopy(0));
    rect_of(&hits, RewriteHit::MapCopy(1));
    insta::assert_snapshot!("maps", s);
}

fn note() -> RewriteNote {
    let refs = vec![
        NoteRef {
            name: "refs/heads/main".into(),
            short: "main".into(),
            kind: NoteRefKind::Branch,
            remote: "origin".into(),
            old: Oid([0xab; 20]),
            new: Oid([0xcd; 20]),
        },
        NoteRef {
            name: "refs/tags/v1.2".into(),
            short: "v1.2".into(),
            kind: NoteRefKind::Tag,
            remote: "origin".into(),
            old: Oid([0x12; 20]),
            new: Oid([0x34; 20]),
        },
    ];
    RewriteNote {
        run: RunId("run-1".into()),
        text: core_note_text(&refs),
        refs,
    }
}

#[test]
fn the_note_is_in_the_interface_language_around_untranslated_commands() {
    let n = note();
    let text = crate::rewrite_view::push::note_text(&n);
    insta::assert_snapshot!("note", text);
    // Every git command of the English note (the CLI's) is in this one too.
    for line in n
        .text
        .lines()
        .filter(|l| l.trim_start().starts_with("git "))
    {
        assert!(text.contains(line.trim()), "{line}\n{text}");
    }
    // Sentences come from the catalog: French reads French around the same commands.
    let fr = gitpoulpi_i18n::with_lang(gitpoulpi_i18n::Lang::Fr, || {
        crate::rewrite_view::push::note_text(&n)
    });
    assert!(fr.contains("git reset --hard origin/main"), "{fr}");
    assert!(!fr.contains("No local work on it"), "{fr}");
    assert!(!fr.contains('\u{2014}'), "no em dash");
}

#[test]
fn a_note_with_nothing_to_tell_says_nobody_needs_to_act() {
    let empty = RewriteNote {
        run: RunId("run-1".into()),
        refs: Vec::new(),
        text: String::new(),
    };
    assert!(
        crate::rewrite_view::push::note_text(&empty).contains("nobody else needs to do anything")
    );
}

#[test]
fn without_a_pushed_ref_the_panel_offers_no_push() {
    let (mut v, _q, _t) = view();
    with_identity_rule(&mut v);
    v.finish_apply(RewriteOutcome {
        run: RunId("run-1".into()),
        moved: vec![local_only("refs/heads/scratch")],
        commits_changed: 3,
    });
    let (s, hits, _) = paint(&v, W, H, None);
    assert!(s.contains("nothing to push"), "{s}");
    assert!(!hits.iter().any(|(_, h)| *h == RewriteHit::PushSend));
    rect_of(&hits, RewriteHit::Undo);
}
