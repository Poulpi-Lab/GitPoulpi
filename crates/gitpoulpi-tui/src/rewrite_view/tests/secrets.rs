//! The Secrets tab (slice S14, TUI): the scan as a job, the findings list,
//! the rule "Remove from history" adds, and the promise that no secret value
//! ever reaches the screen or the daemon from here.

use super::render::{paint, previewed, rect_of, text};
use super::*;
use crate::daemon_link::Reply;
use gitpoulpi_core::domain::rewrite::secrets::{preview as redact, scan_text};
use gitpoulpi_core::domain::rewrite::{ContentReplace, RewritePhase, SecretFinding, SecretScan};

const W: u16 = 110;
const H: u16 = 34;

/// The planted token. Nothing in this file passes it to the view: the view
/// only ever gets what the scanner makes of it (a redacted preview).
const TOKEN: &str = "ghp_9aB3xK7mQ2vL5nR8tY1cE4wZ6dF0hJsPqUoV";

/// A finding the way the daemon would make it: the scanner's own preview.
fn finding(id: &str, text: &str, path: &str, more: u32) -> SecretFinding {
    let found = scan_text(text.as_bytes(), path);
    let m = found.first().expect("the scanner finds the planted value");
    SecretFinding {
        id: id.into(),
        rule: m.rule.to_string(),
        path: path.into(),
        more_paths: more,
        first_commit: Oid([0xa1; 20]),
        last_commit: Oid([0xb2; 20]),
        commits: 14,
        blobs: 3,
        preview: redact(m.rule, &m.value),
    }
}

fn scan() -> SecretScan {
    SecretScan {
        findings: vec![
            finding("f-0123456789ab", &format!("token = {TOKEN}\n"), ".env", 2),
            finding(
                "f-ba9876543210",
                "password = \"hunter2hunter2xx\"\n",
                "config/app.ini",
                0,
            ),
        ],
        commits: 412,
        blobs_scanned: 1_980,
        blobs_skipped_binary: 7,
        blobs_skipped_large: 1,
    }
}

fn scanned() -> (RewriteView, RawQueue, tempfile::TempDir) {
    let (mut v, q, t) = view();
    v.tab = PreviewTab::Secrets;
    v.start_scan();
    v.secrets.running.as_mut().unwrap().reply = Reply::ready(Ok(scan()));
    v.poll(None);
    (v, q, t)
}

#[test]
fn the_tab_says_what_it_does_before_anything_was_scanned() {
    let (mut v, _q, _t) = view();
    v.tab = PreviewTab::Secrets;
    let (s, hits, _) = paint(&v, W, H, None);
    assert!(s.contains("Secrets"), "{s}");
    assert!(s.contains("rotate it first"), "{s}");
    assert!(s.contains("not exhaustive"), "{s}");
    assert!(s.contains("Nothing scanned yet"), "{s}");
    rect_of(&hits, RewriteHit::SecretsScan);
    insta::assert_snapshot!("idle", s);
}

#[test]
fn the_scan_is_a_job_labelled_by_what_it_is_with_a_cancel() {
    let (mut v, _q, _t) = view();
    v.tab = PreviewTab::Secrets;
    v.start_scan();
    assert!(v.scanning() && v.busy());
    assert!(!v.can_preview(), "one job at a time");
    // The connection still holds the last report of an earlier job: it is
    // not this scan's. The first report of the scan is.
    let old = (
        3,
        RewriteProgress {
            phase: RewritePhase::Updating,
            done: 9,
            total: 9,
        },
    );
    v.poll(Some(old));
    assert_eq!(v.secrets.running.as_ref().unwrap().progress, None);
    let own = (
        8,
        RewriteProgress {
            phase: RewritePhase::Rewriting,
            done: 120,
            total: 500,
        },
    );
    v.poll(Some(own));
    let (s, hits, _) = paint(&v, W, H, None);
    assert!(s.contains("Scanning: 120 / 500"), "{s}");
    assert!(
        !s.contains("Rewriting:"),
        "the label is the job's, not the phase's\n{s}"
    );
    rect_of(&hits, RewriteHit::Cancel);
    assert_eq!(v.secrets.running.as_ref().unwrap().job, Some(8));
    insta::assert_snapshot!("running", s);
    // Esc stops a scan before it closes the workbench.
    assert_eq!(v.on_key(key(KeyCode::Esc)), RewriteKeyOutcome::Stay);
    assert_eq!(v.request_close(), RewriteKeyOutcome::Stay);
}

#[test]
fn a_canceled_scan_is_a_line_and_leaves_the_old_findings() {
    let (mut v, _q, _t) = scanned();
    v.start_scan();
    v.secrets.running.as_mut().unwrap().reply =
        Reply::ready(Err(gitpoulpi_core::Error::Canceled { job: 8 }));
    v.poll(None);
    assert!(!v.scanning());
    assert_eq!(v.findings_len(), 2);
    assert!(v
        .notice
        .as_ref()
        .is_some_and(|n| n.text == "Scan canceled." && !n.error));
}

#[test]
fn a_failed_scan_shows_the_reason() {
    let (mut v, _q, _t) = view();
    v.tab = PreviewTab::Secrets;
    v.start_scan();
    v.secrets.running.as_mut().unwrap().reply = Reply::ready(Err(gitpoulpi_core::Error::Backend(
        "repository is shallow".into(),
    )));
    v.poll(None);
    assert!(text(&v).contains("repository is shallow"));
}

#[test]
fn findings_are_a_master_detail_list_with_only_redacted_previews() {
    let (v, _q, _t) = scanned();
    let (s, hits, _) = paint(&v, W, H, None);
    assert!(s.contains("[x] GitHub token"), "{s}");
    assert!(s.contains("Password or secret"), "{s}");
    assert!(s.contains("ghp_****"), "{s}");
    assert!(s.contains(".env (+2 other files)"), "{s}");
    assert!(
        s.contains("412 commits walked, 1980 file versions read"),
        "{s}"
    );
    assert!(s.contains("7 binary files skipped"), "{s}");
    assert!(s.contains("1 file over 5 MiB skipped"), "{s}");
    assert!(
        s.contains("First seen in a1a1a1a, last seen in b2b2b2b"),
        "{s}"
    );
    assert!(s.contains("In 14 commits, 3 file versions"), "{s}");
    assert!(s.contains("Remove from history (2)"), "{s}");
    for hit in [
        RewriteHit::SecretRow(0),
        RewriteHit::SecretRow(1),
        RewriteHit::SecretsRemove,
        RewriteHit::SecretsAll,
        RewriteHit::SecretsNone,
    ] {
        rect_of(&hits, hit);
    }
    insta::assert_snapshot!("findings", s);
}

#[test]
fn no_secret_value_can_reach_the_screen_in_any_state() {
    let (mut v, _q, _t) = scanned();
    let value_body = &TOKEN[4..];
    let mut screens = Vec::new();
    for cursor in [0, 1] {
        v.secrets.cursor = cursor;
        screens.push(text(&v));
    }
    v.tab = PreviewTab::Secrets;
    v.remove_findings();
    screens.push(text(&v));
    // Even the rule's own pane and the Summary (which counts, never shows).
    v.tab = PreviewTab::Summary;
    screens.push(text(&v));
    for s in &screens {
        assert!(!s.contains(TOKEN), "{s}");
        assert!(!s.contains(value_body), "{s}");
        assert!(!s.contains("hunter2hunter2xx"), "{s}");
    }
    // Nor does the request: the rule names findings by id.
    let json = serde_json::to_string(&v.request_recipe()).unwrap();
    assert!(
        !json.contains(value_body) && !json.contains("hunter2"),
        "{json}"
    );
    assert!(json.contains("f-0123456789ab"), "{json}");
}

#[test]
fn keys_and_clicks_tick_and_remove_alike() {
    let (mut by_key, _q1, _t1) = scanned();
    let (mut by_mouse, _q2, _t2) = scanned();
    by_key.focus = Focus::Preview;
    by_key.on_key(key(KeyCode::Down));
    by_key.on_key(key(KeyCode::Char(' ')));
    let (_, hits, _) = paint(&by_mouse, W, H, None);
    let at = rect_of(&hits, RewriteHit::SecretRow(1));
    by_mouse.on_press(RewriteHit::SecretRow(1), at.x, at.y);
    let ticked = |v: &RewriteView| v.ticked_findings();
    assert_eq!(ticked(&by_key), 1);
    assert_eq!(ticked(&by_key), ticked(&by_mouse));

    by_key.on_key(key(KeyCode::Char('r')));
    by_mouse.on_press(RewriteHit::SecretsRemove, 0, 0);
    assert_eq!(by_key.recipe.rules, by_mouse.recipe.rules);
    assert_eq!(by_key.recipe.rules.len(), 1);

    // Check all and uncheck all: the chords and the buttons.
    let (mut a, _q3, _t3) = scanned();
    let (mut b, _q4, _t4) = scanned();
    a.on_key(alt('u'));
    b.on_press(RewriteHit::SecretsNone, 0, 0);
    assert_eq!((a.ticked_findings(), b.ticked_findings()), (0, 0));
    a.on_key(alt('a'));
    b.on_press(RewriteHit::SecretsAll, 0, 0);
    assert_eq!((a.ticked_findings(), b.ticked_findings()), (2, 2));
}

fn alt(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::ALT)
}

#[test]
fn remove_from_history_adds_one_scanning_rule_made_of_ids() {
    let (mut v, _q, _t) = scanned();
    v.uncheck_all_findings();
    v.toggle_finding(1);
    v.remove_findings();
    assert_eq!(v.recipe.rules.len(), 1);
    match &v.recipe.rules[0].kind {
        RuleKind::ContentReplace(c) => {
            assert!(c.paths.is_empty() && c.with.is_empty());
            match &c.find {
                ContentFind::Secrets { literals, scan } => {
                    assert!(*scan);
                    let ids: Vec<&str> = literals.iter().map(|l| l.0.as_str()).collect();
                    assert_eq!(ids, ["f-ba9876543210"]);
                }
                other => panic!("{other:?}"),
            }
        }
        other => panic!("{other:?}"),
    }
    assert!(v.validation().is_ok(), "{:?}", v.validation());
    assert!(v
        .notice
        .as_ref()
        .is_some_and(|n| !n.error && n.text.contains("Rule 1 will remove 1 finding")));
    // The request keeps the scanner on (it used to be switched off here).
    assert!(matches!(
        &v.request_recipe().rules[0].kind,
        RuleKind::ContentReplace(c) if matches!(c.find, ContentFind::Secrets { scan: true, .. })
    ));
    // The recipe on disk keeps ids, never a value.
    assert!(!v.saved_recipe().holds_secrets());

    // A second removal extends the same rule instead of piling up.
    v.check_all_findings();
    v.remove_findings();
    assert_eq!(v.recipe.rules.len(), 1);
    if let RuleKind::ContentReplace(ContentReplace {
        find: ContentFind::Secrets { literals, .. },
        ..
    }) = &v.recipe.rules[0].kind
    {
        assert_eq!(literals.len(), 2);
    }
    // The rule's row and pane never show the ids as values.
    let s = text(&v);
    assert!(!s.contains("f-0123456789ab"), "{s}");
}

#[test]
fn nothing_ticked_removes_nothing() {
    let (mut v, _q, _t) = scanned();
    v.uncheck_all_findings();
    v.remove_findings();
    assert!(v.recipe.rules.is_empty());
    assert!(v.notice.as_ref().is_some_and(|n| n.error));
    let (_, hits, _) = paint(&v, W, H, None);
    rect_of(&hits, RewriteHit::SecretsRemove);
}

#[test]
fn findings_go_stale_after_a_rewrite_a_scope_change_or_a_refused_id() {
    let (mut v, _q, _t) = scanned();
    assert!(v.secrets_stale_reason().is_none());
    v.set_scope_kind(ScopeKind::AllLocal);
    assert!(text(&v).contains("The scope changed since this scan"));
    v.set_scope_kind(ScopeKind::CurrentBranch);
    assert!(v.secrets_stale_reason().is_none());

    // The daemon refuses an id its history no longer holds.
    use gitpoulpi_core::TypedError;
    v.on_error(&gitpoulpi_core::Error::from_typed(
        TypedError::RewriteRefused(
            RewriteRefusal::InvalidRule,
            "finding f-0123456789ab was not found: scan again".into(),
        ),
    ));
    assert!(text(&v).contains("A rewrite ran since this scan"));
    assert!(v.notice.as_ref().is_some_and(|n| n.error));

    // And any apply makes them stale.
    let (mut w, _q, _t) = scanned();
    w.finish_apply(RewriteOutcome {
        run: RunId("run-1".into()),
        moved: Vec::new(),
        commits_changed: 1,
    });
    w.run = Run::Idle;
    assert!(w.secrets_stale_reason().is_some());
    // A new scan clears it.
    w.start_scan();
    w.secrets.running.as_mut().unwrap().reply = Reply::ready(Ok(scan()));
    w.poll(None);
    assert!(w.secrets_stale_reason().is_none());
}

#[test]
fn a_scan_that_finds_nothing_says_so_and_still_warns_it_is_not_exhaustive() {
    let (mut v, _q, _t) = view();
    v.tab = PreviewTab::Secrets;
    v.start_scan();
    v.secrets.running.as_mut().unwrap().reply = Reply::ready(Ok(SecretScan {
        commits: 3,
        blobs_scanned: 5,
        ..SecretScan::default()
    }));
    v.poll(None);
    let s = text(&v);
    assert!(s.contains("No secret found in this scope."), "{s}");
    assert!(s.contains("not exhaustive"), "{s}");
}

#[test]
fn every_rule_the_scanner_has_is_labelled() {
    for id in gitpoulpi_core::domain::rewrite::secrets::rule_ids() {
        assert!(
            crate::rewrite_view::secrets::rule_label(id).is_some(),
            "no label for {id}"
        );
    }
}

#[test]
fn the_tab_is_reachable_by_the_tab_keys_and_by_a_click() {
    let (mut v, _q, _t) = previewed();
    v.focus = Focus::Preview;
    for _ in 0..4 {
        v.on_key(key(KeyCode::Right));
    }
    assert_eq!(v.tab, PreviewTab::Secrets);
    v.tab = PreviewTab::Summary;
    let (_, hits, _) = paint(&v, W, H, None);
    let at = rect_of(&hits, RewriteHit::Tab(PreviewTab::Secrets));
    v.on_press(RewriteHit::Tab(PreviewTab::Secrets), at.x, at.y);
    assert_eq!(v.tab, PreviewTab::Secrets);
    // Left walks back out of it, the tab's own keys notwithstanding.
    v.on_key(key(KeyCode::Left));
    assert_eq!(v.tab, PreviewTab::Impact);
}
