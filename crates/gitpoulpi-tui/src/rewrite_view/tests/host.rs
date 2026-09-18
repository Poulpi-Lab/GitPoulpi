//! S23: the host's own protection in the workbench. A branch the host
//! protects is listed like any protected one (same checkbox, same all/none)
//! and says why ("protected on GitHub"); the "How to get through" overlay
//! explains the three ways with the exact settings page of the provider.

use super::render::{paint, rect_of};
use super::*;
use gitpoulpi_core::domain::rewrite::{
    HostKind, HostProtection, PushRefResult, PushRefStatus, PushRewrittenOutcome,
};

const WIDE_W: u16 = 150;
const WIDE_H: u16 = 44;

fn github(patterns: &[&str]) -> HostProtection {
    let mut host = HostProtection::new(HostKind::GitHub, "octo/thing", "https://github.com");
    host.asked = true;
    host.patterns = patterns.iter().map(|p| p.to_string()).collect();
    host
}

fn with_host(host: HostProtection) -> (RewriteView, RawQueue, tempfile::TempDir) {
    let (mut v, q, t) = super::render::previewed();
    v.tab = PreviewTab::Refs;
    v.set_host(Some(HostHandle {
        base: host,
        answer: None,
    }));
    (v, q, t)
}

fn never_open(_: &str) -> Result<(), String> {
    Err("no browser in tests".into())
}

#[test]
fn a_host_protected_branch_says_why_and_the_daemon_gets_the_hosts_words() {
    let (mut v, _q, _t) = with_host(github(&["main"]));
    let s = super::render::text(&v);
    assert!(s.contains("[ ] main"), "{s}");
    assert!(s.contains("protected on GitHub"), "{s}");
    assert_eq!(
        v.request_recipe()
            .scope_options
            .host_protection
            .map(|h| h.patterns),
        Some(vec!["main".to_string()]),
        "the preview request carries what the host protects"
    );
    assert!(
        v.saved_recipe().scope_options.host_protection.is_none(),
        "a saved recipe never holds the host's answer"
    );
    // The same checkbox as any protected branch.
    v.toggle_protected("refs/heads/main");
    assert!(super::render::text(&v).contains("[x] main"));
}

#[test]
fn a_branch_only_the_workflow_protects_has_no_host_label() {
    let (v, _q, _t) = with_host(github(&["release/*"]));
    let s = super::render::text(&v);
    assert!(s.contains("[ ] main"), "{s}");
    assert!(!s.contains("protected on GitHub"), "{s}");
}

#[test]
fn a_host_that_could_not_be_read_is_said_and_protects_nothing() {
    let mut host = github(&[]);
    host.unknown = Some("the token lacks the scope to read the host's branch protection".into());
    let (v, _q, _t) = with_host(host);
    let s = super::render::text(&v);
    assert!(s.contains("Host protection unknown"), "{s}");
    assert!(!v.howto_offered(), "nothing is known to be in the way");
}

#[test]
fn the_howto_shows_the_three_ways_and_the_exact_settings_page() {
    let (mut v, _q, _t) = with_host(github(&["main"]));
    assert!(v.howto_offered());
    let (s, hits, _) = paint(&v, WIDE_W, WIDE_H, None);
    assert!(s.contains("How to get through"), "the button\n{s}");
    let at = rect_of(&hits, RewriteHit::HostHowto);
    v.on_press(RewriteHit::HostHowto, at.x, at.y);
    assert!(v.host.howto.is_some());

    let (s, hits, _) = paint(&v, WIDE_W, WIDE_H, None);
    assert!(s.contains("1. Temporarily lift the protection"), "{s}");
    assert!(
        s.contains("2. Push the rewrite under a new branch name"),
        "{s}"
    );
    assert!(s.contains("3. Last resort: delete and recreate"), "{s}");
    assert!(
        s.contains("https://github.com/octo/thing/settings/branches"),
        "{s}"
    );
    assert!(s.contains("never changes the host's settings"), "{s}");
    for hit in [
        RewriteHit::HostCopy,
        RewriteHit::HostOpen,
        RewriteHit::HostClose,
        RewriteHit::HostLink(0),
    ] {
        rect_of(&hits, hit);
    }
    // Every way's page, in the order they are listed (way 2 switches the
    // default branch on the branches page too).
    assert_eq!(
        v.howto_links(),
        [
            "https://github.com/octo/thing/settings/branches",
            "https://github.com/octo/thing/settings/rules",
            "https://github.com/octo/thing/settings/branches",
            "https://github.com/octo/thing/settings",
        ]
    );
}

#[test]
fn the_howto_has_a_keyboard_way_in_and_out_and_opening_is_an_instruction_not_an_action() {
    let (mut v, _q, _t) = with_host(github(&["main"]));
    v.host.opener = never_open;
    v.on_key(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::ALT));
    assert!(v.host.howto.is_some(), "Alt+H opens it");
    v.on_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
    assert_eq!(v.host.howto.map(|h| h.link), Some(1));
    v.on_key(KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE));
    assert_eq!(
        v.notice.as_ref().map(|n| (n.error, n.text.as_str())),
        Some((true, "no browser in tests")),
        "a link that cannot be opened is said, nothing else happens"
    );
    v.on_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert!(v.host.howto.is_none());
}

#[test]
fn a_push_the_host_refused_offers_the_howto_with_the_branch_named() {
    let (mut v, _q, _t) = with_host(github(&[]));
    assert!(!v.howto_offered(), "nothing protected, nothing refused");
    v.push.sent = Some(PushRewrittenOutcome {
        run: RunId("run-1".into()),
        dry_run: false,
        results: vec![PushRefResult {
            name: "refs/heads/main".into(),
            remote: "origin".into(),
            expected_old: Oid([1; 20]),
            new: Some(Oid([2; 20])),
            status: PushRefStatus::RejectedProtectedByHost,
            detail: None,
        }],
        atomic_fallback: false,
    });
    assert!(v.howto_offered());
    v.open_howto();
    let (s, _, _) = paint(&v, WIDE_W, WIDE_H, None);
    assert!(s.contains("main"), "{s}");
    assert!(
        s.contains("The host refuses the rewritten history of main because GitHub protects it"),
        "{s}"
    );
}

#[test]
fn a_repository_that_is_not_linked_still_gets_the_links_and_says_why_nothing_was_read() {
    let mut host = HostProtection::new(HostKind::GitLab, "g/sub/p", "https://git.corp.example");
    host.patterns = Vec::new();
    let (mut v, _q, _t) = with_host(host);
    v.host.refused = true;
    v.open_howto();
    let lines = v.howto_links();
    assert_eq!(
        lines[0],
        "https://git.corp.example/g/sub/p/-/settings/repository#js-protected-branches-settings"
    );
    let (s, _, _) = paint(&v, WIDE_W, WIDE_H, None);
    assert!(s.contains("not linked to a host"), "{s}");
}

#[test]
fn the_french_howto_is_in_french_with_the_same_links() {
    let _fr = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::Fr);
    let (mut v, _q, _t) = with_host(github(&["main"]));
    v.open_howto();
    let (s, _, _) = paint(&v, WIDE_W, WIDE_H, None);
    assert!(s.contains("Lever temporairement la protection"), "{s}");
    assert!(
        s.contains("https://github.com/octo/thing/settings/branches"),
        "{s}"
    );
}
