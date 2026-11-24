//! The branches of a request row, the host buttons of the Actions column and
//! the row actions Merge, Close, Mark as ready and Clone (their refusals, their
//! confirmation and what they hand the event loop). The host calls themselves
//! are the hosting crate's tests and the PTY test's.

use super::*;
use crate::radar::account::{AccountAnswer, CloneIndex};
use crate::radar::host_acts::why_not;
use crate::radar::render::plan_title;
use crate::radar::worker::Setup;
use gitpoulpi_hosting::account::{AccountIssue, AccountItems, AccountPr};

fn far(n: u64, title: &str) -> AccountPr {
    let mut p = pr(n, "ann", "2026-10-12T10:00:00Z");
    p.review_requested = true;
    p.source_branch = "feature/login".into();
    AccountPr {
        project: "acme/far".into(),
        clone_url: "https://github.com/acme/far.git".into(),
        pr: PullRequestSummary {
            title: title.into(),
            ..p
        },
    }
}

fn far_issue(n: u64) -> AccountIssue {
    AccountIssue {
        project: "acme/far".into(),
        clone_url: "https://github.com/acme/far.git".into(),
        issue: IssueSummary {
            number: n,
            iid_label: format!("#{n}"),
            reference: format!("#{n}"),
            title: "Crash on empty list".into(),
            ..IssueSummary::default()
        },
    }
}

/// A Radar over one account-wide request and one issue of `acme/far`, GitHub
/// profile `gh`; `clone` says where the project is cloned, when it is.
fn ui_with(
    dir: &std::path::Path,
    prs: Vec<AccountPr>,
    clone: Option<&std::path::Path>,
    kind: gitpoulpi_config::IntegrationKind,
) -> RadarUi {
    let (mut ui, ..) = radar(dir);
    let clones = match clone {
        Some(path) => CloneIndex::of(&[("github.com", "acme/far", path)]),
        None => CloneIndex::default(),
    };
    ui.apply_setup(Setup {
        round: ui.round,
        active: vec!["gh".into()],
        offers: vec![],
        clones,
    });
    ui.apply_account(AccountAnswer {
        round: 0,
        id: "gh".into(),
        label: "GitHub".into(),
        kind,
        host: "github.com".into(),
        me: "kai".into(),
        items: AccountItems {
            prs,
            issues: vec![far_issue(30)],
            ..AccountItems::default()
        },
        fetched_at: 0,
        cached: false,
        problem: None,
        paused: None,
    });
    ui.rebuild();
    ui
}

fn item(ui: &RadarUi, title: &str) -> Item {
    ui.board
        .items
        .iter()
        .find(|i| i.title == title)
        .unwrap_or_else(|| panic!("no item {title}"))
        .clone()
}

fn github() -> gitpoulpi_config::IntegrationKind {
    gitpoulpi_config::IntegrationKind::Github
}

#[test]
fn a_request_names_its_branches_and_a_fork_its_owner() {
    let dir = tempfile::tempdir().unwrap();
    let mut forked = far(21, "From a fork");
    forked.pr.source_project = Some("alice/far".into());
    let ui = ui_with(
        dir.path(),
        vec![far(20, "Add login"), forked],
        None,
        github(),
    );
    assert_eq!(
        item(&ui, "Add login").branches_text(),
        "feature/login \u{2192} main"
    );
    assert_eq!(
        item(&ui, "From a fork").branches_text(),
        "alice:feature/login \u{2192} main"
    );
    // An issue has none; a host that left the target out shows the source.
    assert_eq!(item(&ui, "Crash on empty list").branches_text(), "");
    let mut bare = item(&ui, "Add login");
    bare.target.clear();
    assert_eq!(bare.branches_text(), "feature/login");
    bare.branch = Some(String::new());
    assert_eq!(bare.branches_text(), "", "missing data shows nothing");
}

#[test]
fn the_branches_share_the_title_room_and_leave_a_narrow_row_alone() {
    // Everything fits.
    assert_eq!(plan_title(60, 20, 18), (20, 18));
    // The branches are cut before the title is: a third of the room, at
    // least twelve cells.
    let (title, branches) = plan_title(60, 50, 40);
    assert_eq!(branches, 20.min(60 - title - 2));
    assert!(
        title + 2 + branches <= 60 && branches >= 12,
        "{title} {branches}"
    );
    // A short title leaves the branches what it does not use.
    let (title, branches) = plan_title(60, 10, 70);
    assert_eq!(title, 10);
    assert_eq!(branches, 48);
    // Narrow: the branches are cut, never dropped.
    let (title, branches) = plan_title(39, 20, 18);
    assert!(
        branches >= 8 && title + 2 + branches <= 39,
        "{title} {branches}"
    );
    let (title, branches) = plan_title(24, 30, 18);
    assert!(branches >= 8 && title >= 4, "{title} {branches}");
    // Only a room too small for both is the title's alone. None: the title only.
    assert_eq!(plan_title(12, 20, 18), (12, 0));
    assert_eq!(plan_title(80, 20, 0), (80, 0));
}

#[test]
fn a_request_row_paints_its_branches_and_an_issue_row_its_repository() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = ui_with(dir.path(), vec![far(20, "Add login")], None, github());
    let text = screen(&mut ui, 160, 30);
    let request = text.lines().find(|l| l.contains("Add login")).unwrap();
    assert!(request.contains("feature/login \u{2192} main"), "{request}");
    assert!(request.contains("acme/far"), "{request}");
    let issue = text
        .lines()
        .find(|l| l.contains("Crash on empty list"))
        .unwrap();
    assert!(issue.contains("acme/far"), "{issue}");
}

#[test]
fn the_host_buttons_leave_a_narrow_column_but_not_the_row_menu() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = ui_with(dir.path(), vec![far(20, "Add login")], None, github());
    let wide = screen(&mut ui, 160, 30);
    let wide_row = wide.lines().find(|l| l.contains("Add login")).unwrap();
    for glyph in ["\u{2295}", "\u{00D7}", "\u{21E3}"] {
        assert!(wide_row.contains(glyph), "{glyph} in {wide_row}");
    }
    let narrow = screen(&mut ui, 80, 30);
    let narrow_row = narrow.lines().find(|l| l.contains("Add login")).unwrap();
    assert!(!narrow_row.contains('\u{2295}'), "{narrow_row}");
    assert!(
        narrow_row.contains('\u{22EE}'),
        "the menu button stays: {narrow_row}"
    );
    // The row menu lists them all.
    select(&mut ui, "Add login");
    ui.open_menu(2, 8);
    let menu = screen(&mut ui, 160, 30);
    for word in ["Merge", "Close...", "Clone repository"] {
        assert!(menu.contains(word), "{word}:\n{menu}");
    }
}

#[test]
fn merge_is_refused_with_its_reason_and_close_and_ready_follow_the_row() {
    let dir = tempfile::tempdir().unwrap();
    let ui = ui_with(dir.path(), vec![far(20, "Add login")], None, github());
    let mut it = item(&ui, "Add login");
    assert_eq!(why_not(&it, Act::Merge), None);
    assert_eq!(why_not(&it, Act::CloseItem), None);
    assert!(why_not(&it, Act::Ready).unwrap().contains("not a draft"));
    it.draft = true;
    assert!(why_not(&it, Act::Merge).unwrap().contains("draft"));
    assert_eq!(why_not(&it, Act::Ready), None);
    it.draft = false;
    it.has_conflicts = Some(true);
    assert!(why_not(&it, Act::Merge).unwrap().contains("main"));
    it.has_conflicts = None;
    it.checks = Some(gitpoulpi_hosting::ChecksState::Failing);
    assert!(why_not(&it, Act::Merge).unwrap().contains("checks"));
    // An issue cannot be merged; a host without the calls says so.
    let issue = item(&ui, "Crash on empty list");
    assert!(why_not(&issue, Act::Merge).is_some());
    assert_eq!(why_not(&issue, Act::CloseItem), None);
    it.checks = None;
    it.acts = Default::default();
    it.host_label = "Bitbucket".into();
    for act in [Act::Merge, Act::CloseItem] {
        assert!(why_not(&it, act).unwrap().contains("Bitbucket"), "{act:?}");
    }
}

#[test]
fn merge_asks_first_and_escape_sends_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = ui_with(dir.path(), vec![far(20, "Add login")], None, github());
    select(&mut ui, "Add login");
    assert_eq!(press(&mut ui, 'm'), Outcome::None);
    assert!(ui.confirm.is_some(), "a confirmation, not a call");
    assert!(ui.acting.is_none());
    let text = screen(&mut ui, 120, 36);
    for word in [
        "Merge #20",
        "feature/login \u{2192} main",
        "acme/far",
        "Squash and",
        "cannot be undone",
    ] {
        assert!(text.contains(word), "{word}:\n{text}");
    }
    ui.on_key(key(KeyCode::Esc));
    assert!(ui.confirm.is_none());
    assert!(ui.acting.is_none(), "nothing was sent");
    // A draft is refused before any box opens.
    ui.board.items.iter_mut().for_each(|i| i.draft = true);
    select(&mut ui, "Add login");
    assert!(matches!(press(&mut ui, 'm'), Outcome::Message(_)));
    assert!(ui.confirm.is_none());
}

#[test]
fn close_asks_first_for_a_request_and_for_an_issue() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = ui_with(dir.path(), vec![far(20, "Add login")], None, github());
    select(&mut ui, "Add login");
    press(&mut ui, 'x');
    let text = screen(&mut ui, 120, 36);
    assert!(
        text.contains("Close #20") && text.contains("Close it"),
        "{text}"
    );
    ui.on_key(key(KeyCode::Esc));
    select(&mut ui, "Crash on empty list");
    press(&mut ui, 'x');
    let text = screen(&mut ui, 120, 36);
    assert!(text.contains("closes the issue"), "{text}");
    ui.on_key(key(KeyCode::Esc));
    assert!(ui.confirm.is_none() && ui.acting.is_none());
}

#[test]
fn clone_hands_the_event_loop_the_project_and_refuses_for_a_clone_that_is_there() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = ui_with(dir.path(), vec![far(20, "Add login")], None, github());
    select(&mut ui, "Add login");
    assert_eq!(
        press(&mut ui, 'g'),
        Outcome::CloneRepo(Box::new(CloneRequest {
            integration: "gh".into(),
            project: "acme/far".into(),
            clone_url: "https://github.com/acme/far.git".into(),
        }))
    );
    let there = dir.path().join("far");
    let mut ui = ui_with(
        dir.path(),
        vec![far(20, "Add login")],
        Some(&there),
        github(),
    );
    select(&mut ui, "Add login");
    match press(&mut ui, 'g') {
        Outcome::Message(text) => assert!(text.contains("Already cloned at"), "{text}"),
        other => panic!("{other:?}"),
    }
}

/// The sizes of the owner's `radar.toml` (`repo = 34, number = 4, author = 60,
/// age = 9`) leave the title little room; the branches must still show.
#[test]
fn wide_sized_columns_do_not_push_the_branches_off_the_row() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = ui_with(dir.path(), vec![far(20, "Add login")], None, github());
    ui.state.columns.repo = Some(34);
    ui.state.columns.number = Some(4);
    ui.state.columns.author = Some(60);
    ui.state.columns.age = Some(9);
    for width in [100u16, 120, 140, 160, 200] {
        let text = screen(&mut ui, width, 30);
        let row = text
            .lines()
            .find(|l| l.contains("Add login"))
            .unwrap_or_else(|| panic!("title gone at {width}\n{text}"));
        assert!(row.contains("feature/"), "branches gone at {width}: {row}");
    }
}

#[test]
fn a_request_row_keeps_its_branches_at_every_width_from_seventy() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = ui_with(dir.path(), vec![far(20, "Add login")], None, github());
    for width in 70u16..=140 {
        let text = screen(&mut ui, width, 30);
        let row = text.lines().find(|l| l.contains("Add")).unwrap();
        assert!(
            row.contains("feat") || row.contains("\u{2192}"),
            "{width}: {row}"
        );
    }
}
