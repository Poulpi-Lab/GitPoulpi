//! Every list of the workbench that can outgrow its room draws the shared
//! scrollbar beside its rows, registers it as a hit target, and lets the bar
//! and the wheel move the list without moving the selection - and shows
//! nothing when everything fits (`bars.rs`).

use super::render::{paint, previewed, rect_of};
use super::*;
use gitpoulpi_core::domain::rewrite::{
    PushState, RefChange, RewriteWarning, SecretFinding, SecretScan,
};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

const W: u16 = 110;
const H: u16 = 30;

fn findings(n: usize) -> SecretScan {
    SecretScan {
        findings: (0..n)
            .map(|i| SecretFinding {
                id: format!("f-{i:012x}"),
                rule: "github-token".into(),
                path: format!("secrets/cfg{i:02}.env"),
                more_paths: 0,
                first_commit: Oid([0xa1; 20]),
                last_commit: Oid([0xb2; 20]),
                commits: 1,
                blobs: 1,
                preview: "ghp_****abcd".into(),
            })
            .collect(),
        commits: 40,
        blobs_scanned: 40,
        blobs_skipped_binary: 0,
        blobs_skipped_large: 0,
    }
}

fn scanned(n: usize) -> (RewriteView, crate::daemon_link::RawQueue, tempfile::TempDir) {
    let (mut v, q, t) = view();
    v.tab = PreviewTab::Secrets;
    v.start_scan();
    v.secrets.running.as_mut().unwrap().reply = crate::daemon_link::Reply::ready(Ok(findings(n)));
    v.poll(None);
    (v, q, t)
}

/// Whether the bar's end arrows are drawn at both ends of `track`.
fn arrows(buf: &Buffer, track: Rect) -> bool {
    buf[(track.x, track.y)].symbol() == "\u{2191}"
        && buf[(track.x, track.bottom() - 1)].symbol() == "\u{2193}"
}

fn bar_of(hits: &[(Rect, RewriteHit)]) -> Option<(Rect, BarId)> {
    hits.iter().find_map(|(r, h)| match h {
        RewriteHit::Bar(id) => Some((*r, *id)),
        _ => None,
    })
}

#[test]
fn the_secrets_list_draws_a_bar_when_it_overflows_and_none_when_it_fits() {
    let (v, _q, _t) = scanned(40);
    let (s, hits, buf) = paint(&v, W, H, None);
    let (track, id) = bar_of(&hits).unwrap_or_else(|| panic!("no bar\n{s}"));
    assert_eq!(id, BarId::Secrets);
    assert!(arrows(&buf, track), "{s}");

    let (v, _q, _t) = scanned(3);
    let (s, hits, _) = paint(&v, W, H, None);
    assert!(bar_of(&hits).is_none(), "{s}");
}

#[test]
fn the_thumb_drags_the_list_and_the_selection_stays_until_a_key_moves_it() {
    let (mut v, _q, _t) = scanned(40);
    let (_, hits, _) = paint(&v, W, H, None);
    let (track, id) = bar_of(&hits).expect("bar");
    // Hold the thumb (right under the up arrow), drag to the bottom.
    v.on_press(RewriteHit::Bar(id), track.x, track.y + 1);
    assert!(v.bar_drag(track.bottom() + 5));
    let (s, _, _) = paint(&v, W, H, None);
    let top = v.paint.secrets_top.get();
    assert!(top > 20, "dragged to the end: top {top}\n{s}");
    assert_eq!(v.secrets.cursor, 0, "the selection did not move");
    assert!(s.contains("cfg39"), "{s}");
    assert!(v.bar_release());
    // A key that moves the selection brings the list back to it.
    v.move_secret_cursor(1);
    let _ = paint(&v, W, H, None);
    assert_eq!(v.paint.secrets_top.get(), 1);
}

#[test]
fn the_wheel_scrolls_the_list_under_the_pointer_and_not_the_selection() {
    let (mut v, _q, _t) = scanned(40);
    let _ = paint(&v, W, H, None);
    v.on_wheel(Focus::Preview, 12, 3);
    let _ = paint(&v, W, H, None);
    assert_eq!(v.paint.secrets_top.get(), 3);
    assert_eq!(v.secrets.cursor, 0);
}

#[test]
fn the_rules_list_has_its_bar() {
    let (mut v, _q, _t) = view();
    for _ in 0..25 {
        v.add_rule(RuleTemplate::DateShift);
    }
    let (s, hits, buf) = paint(&v, W, 20, None);
    let (track, id) = bar_of(&hits).unwrap_or_else(|| panic!("no bar\n{s}"));
    assert_eq!(id, BarId::Rules);
    assert!(arrows(&buf, track), "{s}");
    // The newest rule is selected, so the list ends on it: the up arrow
    // steps one row back and the selection stays.
    let end = v.paint.rules_top.get();
    assert!(end > 0, "{s}");
    v.on_press(RewriteHit::Bar(id), track.x, track.y);
    let _ = paint(&v, W, 20, None);
    assert_eq!(
        v.paint.rules_top.get(),
        end - 1,
        "the up arrow steps one row"
    );
    assert_eq!(v.rules.cursor, 24);
}

#[test]
fn a_long_summary_scrolls_with_its_bar_and_with_the_keys() {
    let (mut v, _q, _t) = previewed();
    v.tab = PreviewTab::Impact;
    let plan = v.plan.as_mut().unwrap();
    plan.warnings = (0..60)
        .map(|i| RewriteWarning::OtherBranchesKeepOldHistory {
            refs: vec![format!("refs/heads/branch-{i}")],
        })
        .collect();
    let (s, hits, buf) = paint(&v, W, 24, None);
    let (track, id) = bar_of(&hits).unwrap_or_else(|| panic!("no bar\n{s}"));
    assert_eq!(id, BarId::Text);
    assert!(arrows(&buf, track), "{s}");
    let first = s.clone();
    v.scroll_text(5);
    let (s, _, _) = paint(&v, W, 24, None);
    assert_ne!(first, s, "the keys scroll the text");
    v.on_wheel(Focus::Preview, track.y, -50);
    let (again, _, _) = paint(&v, W, 24, None);
    assert_eq!(first, again, "the wheel scrolls it back to the top");
}

#[test]
fn the_refs_tab_scrolls_its_list_of_moving_refs() {
    let (mut v, _q, _t) = previewed();
    v.tab = PreviewTab::Refs;
    let plan = v.plan.as_mut().unwrap();
    plan.refs = (0..50)
        .map(|i| RefChange {
            name: format!("refs/heads/topic-{i}"),
            old: Oid([1; 20]),
            new: Some(Oid([2; 20])),
            pushed: PushState::LocalOnly,
            protected: false,
            included: true,
            checked_out_in: None,
        })
        .collect();
    let (s, hits, buf) = paint(&v, W, 24, None);
    let (track, id) = bar_of(&hits).unwrap_or_else(|| panic!("no bar\n{s}"));
    assert_eq!(id, BarId::RefsMoving);
    assert!(arrows(&buf, track), "{s}");
    assert!(!s.contains("topic-49"), "{s}");
    // The keys' twin of the bar: a page down.
    assert!(v.page_bar(BarId::RefsMoving, 1));
    let (s, _, _) = paint(&v, W, 24, None);
    assert!(s.contains("topic-9 ") && !s.contains("topic-0 "), "{s}");
}

#[test]
fn every_bar_has_a_one_column_rect_in_the_hit_list_as_painted() {
    let (v, _q, _t) = scanned(40);
    let (_, hits, _) = paint(&v, W, H, None);
    let rect = rect_of(&hits, RewriteHit::Bar(BarId::Secrets));
    assert_eq!(rect.width, 1);
}
