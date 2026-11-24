//! The branches of a request row, the host buttons of the Actions column and
//! the row actions Merge, Close, Mark as ready and Clone (their refusals, their
//! confirmation and what they hand the event loop). The host calls themselves
//! are the hosting crate's tests and the PTY test's.

use super::*;
use crate::radar::account::{AccountAnswer, CloneIndex};
use crate::radar::host_acts::why_not;
use crate::radar::worker::Setup;
use crate::radar::Cell;
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

/// The column of `needle` in `line`, in cells (the screen has no wide glyphs here).
fn col(line: &str, needle: &str) -> usize {
    let at = line
        .find(needle)
        .unwrap_or_else(|| panic!("{needle} in {line}"));
    line[..at].chars().count()
}

#[test]
fn the_branches_column_follows_the_repository_and_holds_the_branches_at_every_width() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = ui_with(dir.path(), vec![far(20, "Add login")], None, github());
    for width in 70u16..=200 {
        let text = screen(&mut ui, width, 30);
        let header = text.lines().find(|l| l.contains("Repository")).unwrap();
        assert!(
            col(header, "Repository") < col(header, "Branches")
                && col(header, "Branches") < col(header, "Title"),
            "{width}: {header}"
        );
        let row = text.lines().find(|l| l.contains("Add")).unwrap();
        // Under its header, before the title, never after it.
        let cell = col(header, "Branches");
        let under: String = row.chars().skip(cell).take(8).collect();
        assert!(
            under.starts_with("feat"),
            "{width}: {under:?}\n{header}\n{row}"
        );
        assert!(cell < col(row, "Add"), "{width}: {row}");
        // The issue's cell stays empty.
        let issue = text.lines().find(|l| l.contains("Crash on")).unwrap();
        let blank: String = issue.chars().skip(cell).take(8).collect();
        assert!(blank.trim().is_empty(), "{width}: {blank:?}");
    }
}

#[test]
fn the_branches_are_whole_when_wide_and_cut_with_the_target_kept_when_narrower() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = ui_with(dir.path(), vec![far(20, "Add login")], None, github());
    let wide = screen(&mut ui, 200, 30);
    assert!(wide.contains("feature/login \u{2192} main"), "{wide}");
    ui.state.columns.branches = Some(12);
    let text = screen(&mut ui, 200, 30);
    let row = text.lines().find(|l| l.contains("Add login")).unwrap();
    assert!(
        row.contains("\u{2026} \u{2192} main") || row.contains("\u{2192} main"),
        "{row}"
    );
    assert!(!row.contains("feature/login"), "{row}");
    assert_eq!(ui.col_eff[4], 12);
    assert!(ui
        .hits
        .iter()
        .any(|(_, h)| matches!(h, Hit::Cut(_, Cell::Branches))));
}

#[test]
fn a_window_too_narrow_for_the_column_keeps_the_branches_in_the_title_tooltip() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = ui_with(dir.path(), vec![far(20, "Add login")], None, github());
    let mut hidden_seen = false;
    for width in 40u16..=120 {
        let text = screen(&mut ui, width, 30);
        let header = text.lines().find(|l| l.contains("Repository")).unwrap();
        if !header.contains("Branches") && !text.contains("feature") {
            hidden_seen = true;
            assert!(
                ui.hits
                    .iter()
                    .any(|(_, h)| matches!(h, Hit::Cut(_, Cell::Title))),
                "{width}: no tooltip for the hidden branches"
            );
        }
    }
    assert!(hidden_seen, "a very narrow window drops the column");
    // The tooltip text carries them.
    let item = ui.board.items.iter().find(|i| i.number == 20).unwrap();
    assert!(RadarUi::cut_text(item, Cell::Title).contains("feature/login \u{2192} main"));
}

#[test]
fn the_branches_column_is_picked_after_the_repository_and_sized_by_keys() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = ui_with(dir.path(), vec![far(20, "Add login")], None, github());
    screen(&mut ui, 160, 30);
    ui.column_command(ColumnCmd::Pick);
    assert_eq!(ui.col_pick, Some(Cell::Repo));
    ui.column_command(ColumnCmd::Pick);
    assert_eq!(ui.col_pick, Some(Cell::Branches));
    screen(&mut ui, 160, 30);
    let before = ui.col_eff[4];
    ui.column_command(ColumnCmd::Narrower);
    assert!(ui.state.columns.branches.is_some());
    assert_eq!(ui.col_eff[4], before - 2);
    screen(&mut ui, 160, 30);
    assert!(ui
        .hits
        .iter()
        .any(|(_, h)| *h == Hit::Divider(Cell::Branches)));
    ui.column_command(ColumnCmd::Reset);
    assert_eq!(ui.state.columns.branches, None);
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
        assert!(
            row.contains("\u{2192} main"),
            "branches gone at {width}: {row}"
        );
        let header = text.lines().find(|l| l.contains("Repository")).unwrap();
        assert!(
            header.contains("Branches") && header.contains("Title"),
            "{header}"
        );
        if width >= 140 {
            assert!(
                row.contains("feature/login \u{2192} main"),
                "{width}: {row}"
            );
        }
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
