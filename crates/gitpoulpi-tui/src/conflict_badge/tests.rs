use super::*;
use gitpoulpi_core::domain::conflict_prevention::BaseSource;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::cell::Cell;

fn oid(n: u8) -> Oid {
    Oid([n; 20])
}

fn conflicts(head: Oid, files: &[&str]) -> ConflictPrediction {
    ConflictPrediction {
        status: PredictionStatus::Conflicts,
        head: Some(head.to_hex()),
        base_ref: Some("origin/main".into()),
        base: Some(oid(9).to_hex()),
        base_source: Some(BaseSource::Globs),
        files: files.iter().map(|f| f.to_string()).collect(),
        reason: None,
    }
}

fn answered(p: ConflictPrediction) -> Reply<ConflictPrediction> {
    Reply::ready(Ok(p))
}

#[test]
fn it_asks_once_per_key_and_shows_the_answer_for_the_same_head() {
    let asked = Cell::new(0);
    let mut badge = ConflictBadge::default();
    let now = Instant::now();
    assert!(badge.shown(Some(oid(1))).is_none());
    badge.tick(Some(7), now, || {
        asked.set(asked.get() + 1);
        answered(conflicts(oid(1), &["a.txt"]))
    });
    // The answer is read on the next frame; nothing is asked again.
    badge.tick(Some(7), now, || unreachable!("one in flight"));
    for _ in 0..5 {
        badge.tick(Some(7), now, || unreachable!("same key never asks again"));
    }
    assert_eq!(asked.get(), 1);
    let shown = badge.shown(Some(oid(1))).expect("badge");
    assert_eq!(text(shown), "Conflicts with origin/main in 1 file");
}

#[test]
fn a_stale_badge_never_lingers_after_head_moves() {
    let mut badge = ConflictBadge::default();
    let now = Instant::now();
    badge.tick(Some(7), now, || answered(conflicts(oid(1), &["a.txt"])));
    badge.tick(Some(7), now, || unreachable!());
    assert!(badge.shown(Some(oid(1))).is_some());
    // HEAD moved: hidden in the same frame, before any new answer exists.
    assert!(badge.shown(Some(oid(2))).is_none());
    assert!(badge.shown(None).is_none());
    // And the new key is asked at once.
    let asked = Cell::new(false);
    badge.tick(Some(8), now, || {
        asked.set(true);
        Reply::pending()
    });
    assert!(asked.get());
    assert!(
        badge.shown(Some(oid(2))).is_none(),
        "nothing for the new HEAD yet"
    );
}

#[test]
fn an_answer_to_an_older_key_is_dropped() {
    let mut badge = ConflictBadge::default();
    let now = Instant::now();
    let (slow, fill) = Reply::channel();
    badge.tick(Some(1), now, || slow);
    // The key moved while it waited: a second request goes out.
    let second = Cell::new(0);
    badge.tick(Some(2), now, || {
        second.set(second.get() + 1);
        Reply::pending()
    });
    assert_eq!(second.get(), 1);
    // The first request's answer was dropped with it (its reply is no longer
    // polled), and nothing shows for it.
    fill(Ok(conflicts(oid(1), &["a.txt"])));
    badge.tick(Some(2), now, || unreachable!("the second is in flight"));
    assert!(badge.shown(Some(oid(1))).is_none());
}

#[test]
fn a_superseded_answer_is_asked_again_after_a_pause() {
    let mut badge = ConflictBadge::default();
    let now = Instant::now();
    let superseded = ConflictPrediction {
        status: PredictionStatus::Superseded,
        ..Default::default()
    };
    badge.tick(Some(7), now, || answered(superseded));
    badge.tick(Some(7), now, || unreachable!("paused"));
    let later = now + RETRY_AFTER + Duration::from_millis(1);
    let asked = Cell::new(false);
    badge.tick(Some(7), later, || {
        asked.set(true);
        Reply::pending()
    });
    assert!(asked.get());
}

#[test]
fn closing_settings_forgets_the_answer_and_asks_again() {
    let mut badge = ConflictBadge::default();
    let now = Instant::now();
    badge.tick(Some(7), now, || answered(conflicts(oid(1), &["a.txt"])));
    badge.tick(Some(7), now, || unreachable!());
    assert!(!badge.settings_closed(true), "opening is not closing");
    assert!(badge.shown(Some(oid(1))).is_some());
    assert!(badge.settings_closed(false));
    assert!(badge.shown(Some(oid(1))).is_none());
    assert!(!badge.settings_closed(false), "only once");
}

#[test]
fn off_clean_and_failed_answers_show_nothing() {
    let mut badge = ConflictBadge::default();
    let now = Instant::now();
    let off = ConflictPrediction {
        status: PredictionStatus::Disabled,
        ..Default::default()
    };
    badge.tick(Some(1), now, || answered(off));
    badge.tick(Some(1), now, || unreachable!());
    assert!(badge.is_off());
    assert!(badge.shown(Some(oid(1))).is_none());

    let mut badge = ConflictBadge::default();
    badge.tick(Some(1), now, || {
        Reply::ready(Err(gitpoulpi_core::Error::Backend("boom".into())))
    });
    badge.tick(Some(1), now, || unreachable!("a failed key is not retried"));
    assert!(badge.shown(Some(oid(1))).is_none());
}

#[test]
fn no_key_drops_the_answer() {
    let mut badge = ConflictBadge::default();
    let now = Instant::now();
    badge.tick(Some(1), now, || answered(conflicts(oid(1), &["a.txt"])));
    badge.tick(Some(1), now, || unreachable!());
    badge.tick(None, now, || unreachable!("a detached HEAD asks nothing"));
    assert!(badge.shown(Some(oid(1))).is_none());
}

#[test]
fn the_key_follows_branch_moves_and_ignores_tags() {
    let local = |name: &str, target: Oid, head: bool| Ref {
        kind: RefKind::LocalBranch { name: name.into() },
        target,
        visibility: Default::default(),
        is_head: head,
        upstream: None,
        tag_sync: None,
        multi_remote: None,
    };
    let tag = |target: Oid| Ref {
        kind: RefKind::Tag { name: "v1".into() },
        target,
        visibility: Default::default(),
        is_head: false,
        upstream: None,
        tag_sync: None,
        multi_remote: None,
    };
    let a = [local("feature", oid(1), true)];
    let moved = [local("feature", oid(2), true)];
    assert_ne!(key_of(&a, 0), key_of(&moved, 0), "a commit changes the key");
    assert_ne!(key_of(&a, 0), key_of(&a, 1), "so does closing Settings");
    let with_tag = [local("feature", oid(1), true), tag(oid(5))];
    assert_eq!(
        key_of(&a, 0),
        key_of(&with_tag, 0),
        "a tag is nobody's base"
    );
}

#[test]
fn the_strip_says_where_and_how_many_and_its_button_is_a_hit_rect() {
    let prediction = conflicts(oid(1), &["a.txt", "b.txt"]);
    let mut terminal = Terminal::new(TestBackend::new(90, 3)).unwrap();
    let mut rects = Vec::new();
    terminal
        .draw(|frame| {
            let area = Rect::new(0, 0, 90, 1);
            rects = render(frame, area, &prediction, false);
        })
        .unwrap();
    let buf = terminal.backend().buffer();
    let row: String = (0..90).map(|x| buf[(x, 0)].symbol().to_string()).collect();
    assert!(
        row.contains("Conflicts with origin/main in 2 files"),
        "{row:?}"
    );
    assert!(row.contains("Show files"), "{row:?}");
    assert_eq!(rects.len(), 1);
    let rect = rects[0];
    let label: String = (rect.x..rect.right())
        .map(|x| buf[(x, 0)].symbol().to_string())
        .collect();
    assert_eq!(label.trim(), "Show files", "the rect is the painted button");
    // The row under the strip is untouched.
    let below: String = (0..90).map(|x| buf[(x, 1)].symbol().to_string()).collect();
    assert!(below.trim().is_empty());
}
