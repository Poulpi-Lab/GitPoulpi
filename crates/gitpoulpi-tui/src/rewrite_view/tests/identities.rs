//! The Identities tab (slice S16, TUI): the list with counts and what the
//! `.mailmap` already maps, "Merge into..." (keys and clicks reach the same
//! methods), the Undo it leaves, and the Summary's "Use .mailmap" offer. The
//! daemon's answers are put in by hand.

use super::render::{paint, previewed, rect_of, text};
use super::*;
use crate::daemon_link::Reply;
use gitpoulpi_core::domain::rewrite::{Ident, IdentityCount, IdentityList, MailmapAdded};

const W: u16 = 110;
const H: u16 = 34;

fn row(name: &str, email: &str, a: u64, c: u64, to: Option<(&str, &str)>) -> IdentityCount {
    IdentityCount {
        name: name.into(),
        email: email.into(),
        as_author: a,
        as_committer: c,
        mapped_to: to.map(|(n, e)| Ident::new(n, e)),
    }
}

fn list() -> IdentityList {
    IdentityList {
        identities: vec![
            row("Alex Doe", "alex@example.org", 310, 12, None),
            row("alex", "alex@old.example.com", 60, 0, None),
            row("Alexander", "alex@laptop.example.com", 22, 0, None),
            row(
                "Robert",
                "bob@example.com",
                9,
                0,
                Some(("Robert Roe", "robert@example.org")),
            ),
            row("Casey Example", "casey@example.com", 0, 400, None),
        ],
        commits: 401,
        has_mailmap: true,
    }
}

/// The tab opened and its list answered.
fn opened() -> (RewriteView, RawQueue, tempfile::TempDir) {
    let (mut v, q, t) = view();
    v.tab = PreviewTab::Identities;
    v.focus = Focus::Preview;
    v.load_identities();
    v.identities.load.as_mut().unwrap().0 = Reply::ready(Ok(list()));
    v.poll(None);
    (v, q, t)
}

fn click(v: &mut RewriteView, hit: RewriteHit) -> RewriteKeyOutcome {
    let (_, hits, _) = paint(v, W, H, None);
    let at = rect_of(&hits, hit);
    v.on_press(hit, at.x, at.y)
}

fn key(v: &mut RewriteView, code: KeyCode) -> RewriteKeyOutcome {
    v.on_key(KeyEvent::new(code, KeyModifiers::NONE))
}

#[test]
fn the_tab_lists_every_pair_with_counts_and_what_is_already_mapped() {
    let (v, _q, _t) = opened();
    let s = text(&v);
    assert!(s.contains("Identities"), "{s}");
    assert!(s.contains("Alex Doe <alex@example.org>"), "{s}");
    assert!(s.contains("310 as author, 12 as committer"), "{s}");
    assert!(
        s.contains("Robert <bob@example.com>  mapped to Robert Roe <robert@example.org>"),
        "{s}"
    );
    assert!(s.contains("0 of 5 selected"), "{s}");
    assert!(!s.contains("There is no .mailmap yet"), "{s}");
    insta::assert_snapshot!("list", s);
}

#[test]
fn a_repository_without_a_mailmap_says_merge_creates_it() {
    let (mut v, _q, _t) = opened();
    v.identities.loaded.as_mut().unwrap().list.has_mailmap = false;
    assert!(text(&v).contains("There is no .mailmap yet: \"Merge into...\" creates it."));
}

#[test]
fn opening_the_tab_reads_the_list_and_a_moved_scope_reads_it_again() {
    let (mut v, _q, _t) = view();
    v.tab = PreviewTab::Identities;
    assert!(v.identities.load.is_none());
    v.poll(None);
    assert!(
        v.identities_loading(),
        "the tab on screen asks, off the render thread"
    );
    assert!(text(&v).contains("Reading the identities..."));
    v.identities.load.as_mut().unwrap().0 = Reply::ready(Ok(list()));
    v.poll(None);
    assert!(!v.identities_loading());
    v.poll(None);
    assert!(
        !v.identities_loading(),
        "an answered list is not asked again"
    );
    v.recipe.scope = gitpoulpi_core::domain::rewrite::Scope::AllLocal;
    v.poll(None);
    assert!(v.identities_loading(), "another scope, another list");
}

#[test]
fn ticking_with_keys_and_clicks_counts_and_check_all_and_none_work() {
    let (mut v, _q, _t) = opened();
    key(&mut v, KeyCode::Char(' '));
    key(&mut v, KeyCode::Down);
    key(&mut v, KeyCode::Char(' '));
    assert_eq!(v.ticked_identities(), 2);
    assert!(text(&v).contains("2 of 5 selected"));
    click(&mut v, RewriteHit::IdentRow(2));
    assert_eq!(v.ticked_identities(), 3);
    click(&mut v, RewriteHit::IdentNone);
    assert_eq!(v.ticked_identities(), 0);
    click(&mut v, RewriteHit::IdentAll);
    assert_eq!(v.ticked_identities(), 5);
}

#[test]
fn merge_into_needs_two_and_asks_which_one_stays() {
    let (mut v, _q, _t) = opened();
    assert!(!v.can_merge());
    key(&mut v, KeyCode::Char(' '));
    key(&mut v, KeyCode::Char('m'));
    assert!(
        v.identities.pick.is_none(),
        "one identity is nothing to merge"
    );
    assert!(text(&v).contains("Tick at least two identities to merge."));

    key(&mut v, KeyCode::Down);
    key(&mut v, KeyCode::Char(' '));
    key(&mut v, KeyCode::Down);
    key(&mut v, KeyCode::Char(' '));
    assert!(v.can_merge());
    click(&mut v, RewriteHit::IdentMerge);
    let pick = v.identities.pick.as_ref().expect("the dialog is open");
    // Most commits first.
    assert_eq!(
        pick.candidates[0].ident,
        Ident::new("Alex Doe", "alex@example.org")
    );
    assert_eq!(pick.candidates.len(), 3);
    let s = text(&v);
    assert!(s.contains("Which of the 3 selected identities"), "{s}");
    assert!(
        s.contains("1  Alex Doe <alex@example.org>  (310 commits)"),
        "{s}"
    );
    insta::assert_snapshot!("pick", s);
}

#[test]
fn choosing_one_sends_the_others_as_the_lines_to_write_and_esc_leaves() {
    let (mut v, _q, _t) = opened();
    for i in 0..3 {
        v.toggle_identity(i);
    }
    v.open_merge_pick();
    key(&mut v, KeyCode::Esc);
    assert!(v.identities.pick.is_none());
    assert!(v.identities.saving.is_none(), "Esc writes nothing");

    v.open_merge_pick();
    let args = v.merge_args(0).unwrap();
    assert_eq!(args.canonical, Ident::new("Alex Doe", "alex@example.org"));
    assert_eq!(
        args.from,
        vec![
            Ident::new("alex", "alex@old.example.com"),
            Ident::new("Alexander", "alex@laptop.example.com")
        ]
    );
    assert_eq!(args.file, None);

    // The key and the click do it too.
    v.open_merge_pick();
    key(&mut v, KeyCode::Char('1'));
    assert!(v.identities.saving.is_some() && v.identities.pick.is_none());
    let (mut v, _q, _t) = opened();
    for i in 0..3 {
        v.toggle_identity(i);
    }
    v.open_merge_pick();
    click(&mut v, RewriteHit::IdentPick(1));
    assert!(v.identities.saving.is_some());
}

#[test]
fn a_merge_that_wrote_lines_offers_undo_and_reads_the_list_again() {
    let (mut v, _q, _t) = opened();
    for i in 0..3 {
        v.toggle_identity(i);
    }
    v.open_merge_pick();
    v.merge_into(0);
    assert!(text(&v).contains("Reading the identities..."), "working");
    v.identities.saving = Some(Reply::ready(Ok(MailmapAdded {
        added: 2,
        file: ".mailmap".into(),
    })));
    v.poll(None);
    assert_eq!(v.ticked_identities(), 0);
    assert!(
        v.identities.load.is_some(),
        "what is mapped changed: read again"
    );
    v.identities.load.as_mut().unwrap().0 = Reply::ready(Ok(list()));
    v.poll(None);
    let s = text(&v);
    assert!(
        s.contains("Added 2 lines to .mailmap. Commit the file to share it."),
        "{s}"
    );
    assert!(s.contains("Undo"), "{s}");
    insta::assert_snapshot!("merged", s);
    // Undo: the button and its key hand the journal's undo to the caller.
    assert_eq!(
        click(&mut v, RewriteHit::IdentUndo),
        RewriteKeyOutcome::Undo
    );
    assert_eq!(key(&mut v, KeyCode::Char('u')), RewriteKeyOutcome::Undo);
}

#[test]
fn nothing_new_to_write_says_so_and_offers_no_undo() {
    let (mut v, _q, _t) = opened();
    v.identities.saving = Some(Reply::ready(Ok(MailmapAdded {
        added: 0,
        file: ".mailmap".into(),
    })));
    v.poll(None);
    assert!(v
        .notice
        .as_ref()
        .unwrap()
        .text
        .contains("Already in .mailmap"));
    assert!(v.identities.done.is_none());
    assert_eq!(key(&mut v, KeyCode::Char('u')), RewriteKeyOutcome::Stay);
}

#[test]
fn a_failed_merge_shows_the_reason_and_keeps_the_ticks() {
    let (mut v, _q, _t) = opened();
    v.toggle_identity(0);
    v.toggle_identity(1);
    v.identities.saving = Some(Reply::ready(Err(gitpoulpi_core::Error::Backend(
        "cannot write .mailmap".into(),
    ))));
    v.poll(None);
    assert!(text(&v).contains("cannot write .mailmap"));
    assert_eq!(v.ticked_identities(), 2);
}

#[test]
fn the_dialog_owns_the_pointer_and_the_workbench_behind_it_does_not_react() {
    let (mut v, _q, _t) = opened();
    for i in 0..2 {
        v.toggle_identity(i);
    }
    v.open_merge_pick();
    let (_, hits, _) = paint(&v, W, H, None);
    // The dialog's targets come first, so a click over its rows cannot reach
    // a list row behind it.
    let first = hits.first().unwrap().1;
    assert!(
        matches!(
            first,
            RewriteHit::IdentPickCancel | RewriteHit::IdentPick(_)
        ),
        "{first:?}"
    );
    let shield = rect_of(&hits, RewriteHit::IdentPickShield);
    assert_eq!((shield.x, shield.y), (0, 0));
    // A click on the shield leaves the dialog and writes nothing.
    click(&mut v, RewriteHit::IdentPickShield);
    assert!(v.identities.pick.is_none() && v.identities.saving.is_none());
}

#[test]
fn pushed_commits_and_an_author_rule_offer_the_mailmap_and_the_button_ticks_the_matches() {
    let (mut v, _q, _t) = previewed();
    // `previewed` has an identity rule for old@example.com on a pushed branch;
    // it also gives the new identity a name.
    v.move_rule_cursor(-1);
    v.edit_field(3, FieldInput::Text("Newbie".into()));
    assert!(v.mailmap_offer());
    let s = text(&v);
    assert!(
        s.contains("A .mailmap fixes the display without rewriting. Use .mailmap instead?"),
        "{s}"
    );
    insta::assert_snapshot!("offer", s);

    click(&mut v, RewriteHit::IdentOffer);
    assert_eq!(v.tab, PreviewTab::Identities);
    assert!(v.identities_loading());
    let mut answer = list();
    answer
        .identities
        .push(row("Old", "old@example.com", 40, 0, None));
    answer
        .identities
        .push(row("Elsewhere", "else@example.com", 1, 0, None));
    v.identities.load.as_mut().unwrap().0 = Reply::ready(Ok(answer));
    v.poll(None);
    assert_eq!(v.ticked_identities(), 1, "the identity the rule matches");
    assert!(v
        .identities
        .ticked
        .contains(&Ident::new("Old", "old@example.com")));
    // The identity the rule would set can be the one to keep, though no
    // commit has it yet.
    assert!(v.can_merge());
    v.open_merge_pick();
    let pick = v.identities.pick.as_ref().unwrap();
    assert!(pick
        .candidates
        .iter()
        .any(|c| c.ident == Ident::new("Newbie", "new@example.org") && c.commits == 0));
}

#[test]
fn no_offer_for_local_commits_or_when_the_rule_changed_nothing() {
    let (mut v, _q, _t) = previewed();
    for r in &mut v.plan.as_mut().unwrap().refs {
        r.pushed = PushState::LocalOnly;
    }
    assert!(!v.mailmap_offer());
    assert!(!text(&v).contains("Use .mailmap instead"));
    let (mut v, _q, _t) = previewed();
    v.plan.as_mut().unwrap().per_rule.clear();
    assert!(!v.mailmap_offer());
    let (mut v, _q, _t) = previewed();
    v.recipe.rules[0].enabled = false;
    assert!(!v.mailmap_offer());
}

#[test]
fn the_palette_opens_the_workbench_on_the_tab() {
    let (v, _q, _t) = view();
    let v = v.with_identities();
    assert_eq!(v.tab, PreviewTab::Identities);
    assert_eq!(v.focus, Focus::Preview);
    assert!(v.identities_loading());
}
