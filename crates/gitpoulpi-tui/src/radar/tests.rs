//! The Radar view without a terminal or a network: answers fed straight
//! in, keys sent, outcomes and the painted buffer read back.

use super::*;
use crate::hosting_task::{IssuesTarget, ProfileRef};
use crossterm::event::{KeyEvent, KeyModifiers};
use gitpoulpi_hosting::{IssueSummary, PullRequestState, PullRequestSummary};
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::cell::Cell;

thread_local! {
    static NOW: Cell<i64> = const { Cell::new(1_000_000) };
}

fn clock() -> i64 {
    NOW.with(Cell::get)
}

fn set_clock(now: i64) {
    NOW.with(|c| c.set(now));
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn press(ui: &mut RadarUi, c: char) -> Outcome {
    ui.on_key(key(KeyCode::Char(c)))
}

fn pr(n: u64, author: &str, updated: &str) -> PullRequestSummary {
    PullRequestSummary {
        number: n,
        iid_label: format!("#{n}"),
        title: format!("Change {n}"),
        state: PullRequestState::Open,
        source_branch: format!("feat-{n}"),
        target_branch: "main".into(),
        source_project: None,
        author: author.into(),
        web_url: format!("https://github.com/acme/app/pull/{n}"),
        updated_at: updated.into(),
        review_requested: false,
        assignees: vec![],
        reviewers: vec![],
        checks: None,
        has_conflicts: None,
    }
}

fn issue(n: u64, author: &str) -> IssueSummary {
    IssueSummary {
        number: n,
        iid_label: format!("#{n}"),
        reference: format!("#{n}"),
        title: format!("Bug {n}"),
        author: author.into(),
        updated_at: "2026-10-01T00:00:00Z".into(),
        web_url: format!("https://github.com/acme/app/issues/{n}"),
        ..IssueSummary::default()
    }
}

fn answer(
    path: &std::path::Path,
    project: &str,
    prs: Vec<PullRequestSummary>,
    issues: Vec<IssueSummary>,
) -> RepoAnswer {
    let mut snap = worker::empty_snapshot();
    snap.profile_id = Some("gh".into());
    snap.kind = Some(gitpoulpi_config::IntegrationKind::Github);
    snap.can_pulls = true;
    snap.project = project.into();
    snap.prs = prs;
    snap.issues = issues;
    snap.issues_target = Some(IssuesTarget {
        profile: ProfileRef {
            id: "gh".into(),
            label: "GitHub".into(),
            kind: gitpoulpi_config::IntegrationKind::Github,
        },
        project: project.into(),
        chosen: true,
    });
    RepoAnswer {
        path: path.to_path_buf(),
        name: path.file_name().unwrap().to_string_lossy().into_owned(),
        host: "github.com".into(),
        me: "kai".into(),
        snapshot: snap,
        cached: false,
        round: 0,
        paused: None,
    }
}

/// A Radar over two repositories (real folders, so "open" works) with
/// answers already in, config in `dir`.
fn radar(dir: &std::path::Path) -> (RadarUi, PathBuf, PathBuf) {
    let app = dir.join("app");
    let tool = dir.join("tool");
    std::fs::create_dir_all(&app).unwrap();
    std::fs::create_dir_all(&tool).unwrap();
    let cfg = dir.join("cfg");
    std::fs::create_dir_all(&cfg).unwrap();
    // The open tabs of the session are the scope.
    let mut session = gitpoulpi_session::GlobalSession::default();
    for p in [&app, &tool] {
        session.tabs.push(gitpoulpi_session::TabEntry {
            repo_path: p.clone(),
            active: false,
        });
    }
    std::fs::create_dir_all(cfg.join("sessions")).unwrap();
    session.save(&cfg.join("sessions").join("s.toml")).unwrap();
    let mut ui = RadarUi::with_parts("s", Some(cfg), None, clock);
    ui.apply_answer(answer(
        &app,
        "acme/app",
        vec![
            pr(1, "kai", "2026-10-09T10:00:00Z"),
            pr(2, "ann", "2026-10-08T10:00:00Z"),
        ],
        vec![issue(7, "kai")],
    ));
    ui.apply_answer(answer(
        &tool,
        "acme/tool",
        vec![pr(3, "bob", "2026-10-07T10:00:00Z")],
        vec![],
    ));
    ui.rebuild();
    (ui, app, tool)
}

/// Selects the item row whose title is `title`.
fn select(ui: &mut RadarUi, title: &str) {
    let at = ui
        .rows
        .iter()
        .position(|r| matches!(r, Row::Item(i) if ui.board.items[*i].title == title))
        .unwrap_or_else(|| panic!("no row {title}: {:?}", ui.rows));
    ui.selected = at;
}

fn screen(ui: &mut RadarUi, w: u16, h: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
    terminal
        .draw(|frame| {
            let area = frame.area();
            ui.render(frame, area);
        })
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    (0..h)
        .map(|y| {
            (0..w)
                .map(|x| buf[(x, y)].symbol().to_string())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_scope_is_the_open_tabs_and_both_repositories_are_listed() {
    let dir = tempfile::tempdir().unwrap();
    let (ui, ..) = radar(dir.path());
    assert_eq!(ui.repos.len(), 2);
    let titles: Vec<&str> = ui.board.items.iter().map(|i| i.title.as_str()).collect();
    assert!(titles.contains(&"Change 1") && titles.contains(&"Change 3"));
    assert!(titles.contains(&"Bug 7"));
}

#[test]
fn the_picked_scope_is_saved_and_restored_on_the_next_open() {
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, app, _) = radar(dir.path());
    let cfg = dir.path().join("cfg");
    let file = gitpoulpi_session::Workspaces::file_in(&cfg);
    let mut all = gitpoulpi_session::Workspaces::load(&file);
    let w = all.create("Acme");
    w.add_repo(&app, None);
    let id = w.id.clone();
    all.save(&file).unwrap();
    ui.picked(Hit::Scope, &id);
    let saved = RadarState::load(&RadarState::file_in(&cfg));
    assert_eq!(saved.scope.as_deref(), Some(id.as_str()));
    let reopened = RadarUi::with_parts("s", Some(cfg.clone()), None, clock);
    assert_eq!(reopened.scope, id);
    assert_eq!(reopened.repos.len(), 1);
    // Back to the open tabs is remembered too (an empty id, not "never picked").
    let mut ui = reopened;
    ui.picked(Hit::Scope, OPEN_TABS);
    let reopened = RadarUi::with_parts("s", Some(cfg), None, clock);
    assert_eq!(reopened.scope, OPEN_TABS);
    assert_eq!(reopened.repos.len(), 2);
}

#[test]
fn the_view_paints_headings_with_counts_and_rows_with_their_repository() {
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, ..) = radar(dir.path());
    let text = screen(&mut ui, 120, 30);
    assert!(text.contains("Radar"), "{text}");
    assert!(text.contains("MY PULL REQUESTS  1"), "{text}");
    assert!(text.contains("OTHER PULL REQUESTS  2"), "{text}");
    assert!(text.contains("ISSUES CREATED BY ME  1"), "{text}");
    assert!(
        text.contains("Change 1") && text.contains("Change 3"),
        "{text}"
    );
    // The repository chip and the number sit on the row.
    let row = text.lines().find(|l| l.contains("Change 3")).unwrap();
    assert!(
        row.contains("tool") && row.contains("#3") && row.contains("bob"),
        "{row}"
    );
    // The three comboboxes and the filter.
    assert!(
        text.contains("Open tabs") && text.contains("Everyone"),
        "{text}"
    );
}

#[test]
fn a_pinned_row_moves_to_the_top_and_survives_a_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, app, tool) = radar(dir.path());
    select(&mut ui, "Change 3");
    let out = press(&mut ui, 'p');
    assert!(
        matches!(&out, Outcome::Message(m) if m.contains("pinned")),
        "{out:?}"
    );
    let first_item = |ui: &RadarUi| {
        ui.rows
            .iter()
            .find_map(|r| match r {
                Row::Item(i) => Some(ui.board.items[*i].title.clone()),
                _ => None,
            })
            .unwrap()
    };
    assert_eq!(first_item(&ui), "Change 3");
    assert!(matches!(ui.rows[0], Row::Heading(Heading::Pinned, 1)));
    // A new Radar over the same config reads the pin back.
    let cfg = ui.config_dir.clone();
    let mut again = RadarUi::with_parts("s", cfg, None, clock);
    again.apply_answer(answer(
        &tool,
        "acme/tool",
        vec![pr(3, "bob", "2026-10-07T10:00:00Z")],
        vec![],
    ));
    again.apply_answer(answer(
        &app,
        "acme/app",
        vec![pr(1, "kai", "2026-10-09T10:00:00Z")],
        vec![],
    ));
    again.rebuild();
    assert_eq!(first_item(&again), "Change 3");
    // Unpinning puts it back in its group.
    select(&mut again, "Change 3");
    press(&mut again, 'p');
    assert!(!matches!(again.rows[0], Row::Heading(Heading::Pinned, _)));
}

#[test]
fn snoozing_hides_a_row_until_the_clock_passes_the_duration() {
    set_clock(1_000_000);
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, ..) = radar(dir.path());
    select(&mut ui, "Change 2");
    // `z` opens the duration combobox; Enter picks its first row, 1 hour.
    assert_eq!(press(&mut ui, 'z'), Outcome::None);
    assert!(ui.snooze_combo.is_open());
    let out = ui.on_key(key(KeyCode::Enter));
    assert!(
        matches!(&out, Outcome::Message(m) if m.contains("snoozed")),
        "{out:?}"
    );
    assert!(ui.board.items.iter().any(|i| i.title == "Change 2"));
    assert!(!ui
        .rows
        .iter()
        .any(|r| matches!(r, Row::Item(i) if ui.board.items[*i].title == "Change 2")));
    // The Snoozed view lists it.
    ui.picked(Hit::View, "snoozed");
    assert!(ui
        .rows
        .iter()
        .any(|r| matches!(r, Row::Item(i) if ui.board.items[*i].title == "Change 2")));
    // An hour later it is back (the poll notices the expiry by itself).
    ui.picked(Hit::View, "all");
    set_clock(1_000_000 + 3601);
    assert!(ui.poll());
    assert!(ui
        .rows
        .iter()
        .any(|r| matches!(r, Row::Item(i) if ui.board.items[*i].title == "Change 2")));
}

#[test]
fn enter_opens_the_repository_and_the_action_keys_carry_what_to_do_there() {
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, app, tool) = radar(dir.path());
    select(&mut ui, "Change 3");
    assert_eq!(
        ui.on_key(key(KeyCode::Enter)),
        Outcome::OpenRepo(tool.clone(), None)
    );
    assert_eq!(
        press(&mut ui, 'c'),
        Outcome::OpenRepo(tool.clone(), Some(Pending::CheckoutPr { number: 3 }))
    );
    assert_eq!(
        press(&mut ui, 't'),
        Outcome::OpenRepo(
            tool,
            Some(Pending::Worktree {
                branch: "feat-3".into()
            })
        )
    );
    assert_eq!(
        press(&mut ui, 's'),
        Outcome::OpenRepo(
            ui.board
                .items
                .iter()
                .find(|i| i.number == 3)
                .unwrap()
                .repo
                .clone()
                .unwrap(),
            Some(Pending::Show {
                pr: true,
                number: 3
            })
        )
    );
    // An issue starts work on a branch named from it; a PR does not.
    select(&mut ui, "Bug 7");
    assert_eq!(
        press(&mut ui, 'n'),
        Outcome::OpenRepo(
            app,
            Some(Pending::StartWork {
                branch: "7-bug-7".into()
            })
        )
    );
    assert!(
        matches!(press(&mut ui, 'c'), Outcome::Message(_)),
        "no branch to check out"
    );
    select(&mut ui, "Change 3");
    assert!(matches!(press(&mut ui, 'n'), Outcome::Message(_)));
}

#[test]
fn review_opens_the_repository_and_carries_the_request_where_the_host_takes_reviews() {
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, _app, tool) = radar(dir.path());
    // (No page either: a request page would open the real browser.)
    for item in &mut ui.board.items {
        item.review = false;
        item.url.clear();
    }
    select(&mut ui, "Change 3");
    assert!(
        matches!(press(&mut ui, 'e'), Outcome::Message(_)),
        "no review where the review view does not exist"
    );
    for item in &mut ui.board.items {
        item.review = true;
    }
    select(&mut ui, "Change 3");
    assert_eq!(
        press(&mut ui, 'e'),
        Outcome::OpenRepo(
            tool,
            Some(Pending::Review {
                number: 3,
                repo: None
            })
        )
    );
    select(&mut ui, "Bug 7");
    assert!(
        matches!(press(&mut ui, 'e'), Outcome::Message(_)),
        "an issue is not reviewed"
    );
}

#[test]
fn a_repository_gone_from_disk_is_not_opened() {
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, _, tool) = radar(dir.path());
    std::fs::remove_dir_all(&tool).unwrap();
    select(&mut ui, "Change 3");
    assert!(
        matches!(ui.on_key(key(KeyCode::Enter)), Outcome::Message(m) if m.contains("not a repository"))
    );
}

#[test]
fn the_row_menu_offers_the_actions_of_its_kind() {
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, ..) = radar(dir.path());
    select(&mut ui, "Change 3");
    ui.command(RadarCommand::Menu);
    let text = screen(&mut ui, 120, 30);
    for want in [
        "Open in browser",
        "Open repository",
        "Check out branch",
        "Create worktree...",
        "Show in repository",
        "Pin to top",
        "Snooze...",
    ] {
        assert!(text.contains(want), "{want}: {text}");
    }
    assert!(
        !text.contains("Start work on issue"),
        "a PR does not start work"
    );
    // Esc closes the menu, not the Radar.
    assert_eq!(ui.on_key(key(KeyCode::Esc)), Outcome::None);
    assert!(ui.menu.is_none());
    assert_eq!(ui.on_key(key(KeyCode::Esc)), Outcome::Close);
}

#[test]
fn the_filter_text_narrows_the_rows_and_escape_clears_it() {
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, ..) = radar(dir.path());
    press(&mut ui, '/');
    assert!(ui.typing());
    for c in "bob".chars() {
        press(&mut ui, c);
    }
    let items = |ui: &RadarUi| ui.rows.iter().filter(|r| matches!(r, Row::Item(_))).count();
    assert_eq!(items(&ui), 1);
    // `q` and `p` typed in the filter are letters, not commands.
    press(&mut ui, 'q');
    assert!(ui.typing());
    ui.on_key(key(KeyCode::Esc));
    assert!(!ui.typing());
    assert_eq!(items(&ui), 4, "Esc leaves the box and clears the text");
}

#[test]
fn the_view_and_filter_comboboxes_pick_by_keyboard() {
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, ..) = radar(dir.path());
    // `v` opens the view combobox; Down + Enter picks the second row.
    press(&mut ui, 'v');
    assert!(ui.view_combo.is_open());
    ui.on_key(key(KeyCode::Down));
    ui.on_key(key(KeyCode::Enter));
    assert_eq!(ui.view, View::Prs);
    assert!(ui
        .rows
        .iter()
        .all(|r| !matches!(r, Row::Item(i) if ui.board.items[*i].kind != ItemKind::Pr)));
    press(&mut ui, 'f');
    assert!(ui.who_combo.is_open());
    ui.on_key(key(KeyCode::Down));
    ui.on_key(key(KeyCode::Enter));
    assert_eq!(ui.who, Who::Mine);
}

#[test]
fn a_failed_answer_keeps_the_cached_rows_and_says_so_on_its_own_line() {
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, app, _) = radar(dir.path());
    let mut failed = answer(&app, "acme/app", vec![], vec![]);
    failed.snapshot.problem = Some(crate::hosting_task::Problem::Offline("no route".into()));
    failed.snapshot.fetched_at = (ui.now)();
    ui.apply_answer(failed);
    ui.rebuild();
    // Stale beats blank.
    assert!(ui.board.items.iter().any(|i| i.title == "Change 1"));
    assert_eq!(ui.notes.len(), 1);
    assert!(ui.notes[0].text.contains("offline"), "{:?}", ui.notes[0]);
    let text = screen(&mut ui, 120, 30);
    assert!(text.contains("REPOSITORIES"), "{text}");
}

#[test]
fn a_rejected_token_offers_reconnect_and_an_unlinked_repository_offers_link() {
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, app, tool) = radar(dir.path());
    let mut bad = answer(&app, "acme/app", vec![], vec![]);
    bad.snapshot.problem = Some(crate::hosting_task::Problem::Reconnect("403".into()));
    ui.apply_answer(bad);
    let mut unlinked = answer(&tool, "acme/tool", vec![], vec![]);
    unlinked.snapshot.profile_id = None;
    unlinked.snapshot.issues_target = None;
    unlinked.snapshot.suggestions = vec![crate::hosting_task::SuggestionRef {
        profile: ProfileRef {
            id: "gh".into(),
            label: "GitHub".into(),
            kind: gitpoulpi_config::IntegrationKind::Github,
        },
        remote: "origin".into(),
        project: "acme/tool".into(),
    }];
    ui.apply_answer(unlinked);
    ui.rebuild();
    assert_eq!(ui.notes.len(), 2);
    assert_eq!(ui.notes[0].act, Some((Act::Reconnect, Some("gh".into()))));
    assert_eq!(ui.notes[1].act, Some((Act::Link, None)));
    // Enter on the Link note routes to the repository's own link flow.
    let at = ui.rows.iter().position(|r| *r == Row::Note(1)).unwrap();
    ui.selected = at;
    assert_eq!(
        ui.on_key(key(KeyCode::Enter)),
        Outcome::OpenRepo(tool, Some(Pending::Link))
    );
    let at = ui.rows.iter().position(|r| *r == Row::Note(0)).unwrap();
    ui.selected = at;
    assert_eq!(
        ui.on_key(key(KeyCode::Enter)),
        Outcome::Integrations(Some("gh".into()))
    );
}

#[test]
fn only_github_and_gitlab_are_named() {
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, ..) = radar(dir.path());
    let text = screen(&mut ui, 160, 40);
    for word in ["Bitbucket", "Azure", "Jira", "Trello", "OAuth"] {
        assert!(!text.contains(word), "{word}");
    }
}

/// `app`'s answer with the lists the Radar asks on top: what mentions
/// the user.
fn app_with_mentions(app: &std::path::Path) -> RepoAnswer {
    let mut a = answer(
        app,
        "acme/app",
        vec![
            pr(1, "kai", "2026-10-09T10:00:00Z"),
            pr(2, "ann", "2026-10-08T10:00:00Z"),
            pr(4, "bob", "2026-10-06T10:00:00Z"),
        ],
        vec![issue(7, "kai"), issue(8, "ann"), issue(9, "ann")],
    );
    a.snapshot.prs_mentioned = vec![pr(4, "bob", "2026-10-06T10:00:00Z")];
    a.snapshot.issues_mentioned = vec![issue(8, "ann")];
    a
}

#[test]
fn what_mentions_the_user_is_its_own_group_and_has_its_own_filter() {
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, app, _) = radar(dir.path());
    ui.apply_answer(app_with_mentions(&app));
    ui.rebuild();
    let text = screen(&mut ui, 120, 40);
    // One request and one issue, neither the user's nor assigned to them.
    assert!(text.contains("MENTIONING ME  2"), "{text}");
    assert!(text.contains("OTHER PULL REQUESTS  2"), "{text}");
    // An issue that is not assigned, created or mentioning stays out.
    assert!(!text.contains("Bug 9"), "{text}");
    let group_of = |ui: &RadarUi, title: &str| {
        ui.board
            .items
            .iter()
            .find(|i| i.title == title)
            .map(|i| i.group)
    };
    assert_eq!(group_of(&ui, "Change 4"), Some(Group::Mentioned));
    assert_eq!(group_of(&ui, "Bug 8"), Some(Group::Mentioned));
    assert_eq!(group_of(&ui, "Change 2"), Some(Group::OtherPrs));
    // "Show: Mentioning me" keeps exactly those.
    ui.picked(Hit::Who, "mentioned");
    let shown: Vec<&str> = ui
        .rows
        .iter()
        .filter_map(|r| match r {
            Row::Item(i) => Some(ui.board.items[*i].title.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(shown, vec!["Change 4", "Bug 8"]);
}

fn app_with_checks(app: &std::path::Path) -> RepoAnswer {
    use gitpoulpi_hosting::ChecksState;
    let mut failing = pr(1, "kai", "2026-10-09T10:00:00Z");
    failing.checks = Some(ChecksState::Failing);
    failing.has_conflicts = Some(true);
    let mut passing = pr(2, "ann", "2026-10-08T10:00:00Z");
    passing.checks = Some(ChecksState::Passing);
    passing.has_conflicts = Some(false);
    let mut running = pr(4, "bob", "2026-10-06T10:00:00Z");
    running.checks = Some(ChecksState::Pending);
    answer(app, "acme/app", vec![failing, passing, running], vec![])
}

#[test]
fn a_request_row_wears_its_checks_and_conflict_glyphs() {
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, app, _) = radar(dir.path());
    ui.apply_answer(app_with_checks(&app));
    ui.rebuild();
    let text = screen(&mut ui, 120, 30);
    let row = |title: &str| {
        text.lines()
            .find(|l| l.contains(title))
            .unwrap_or_else(|| panic!("no {title}: {text}"))
            .to_string()
    };
    assert!(row("Change 1").contains("\u{2717}!"), "{}", row("Change 1"));
    assert!(row("Change 2").contains('\u{2713}'), "{}", row("Change 2"));
    assert!(!row("Change 2").contains('!'), "{}", row("Change 2"));
    assert!(row("Change 4").contains('\u{25CF}'), "{}", row("Change 4"));
    // A row of a repository that told nothing has no glyph.
    assert!(!row("Change 3").contains('\u{2713}') && !row("Change 3").contains('\u{2717}'));
}

#[test]
fn the_selected_requests_detail_line_says_it_in_words() {
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, app, _) = radar(dir.path());
    ui.apply_answer(app_with_checks(&app));
    ui.rebuild();
    select(&mut ui, "Change 1");
    let text = screen(&mut ui, 160, 30);
    let line = text.lines().nth(4).unwrap();
    assert!(
        line.contains("#1 Change 1: Checks failed, Conflicts with main"),
        "{line}"
    );
    select(&mut ui, "Change 2");
    let text = screen(&mut ui, 160, 30);
    assert!(
        text.lines()
            .nth(4)
            .unwrap()
            .contains("Checks passed, No conflicts"),
        "{text}"
    );
}

#[test]
fn hovering_the_glyphs_shows_a_tooltip_after_the_dwell_and_nothing_hides_behind_a_popup() {
    use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, app, _) = radar(dir.path());
    ui.apply_answer(app_with_checks(&app));
    ui.rebuild();
    screen(&mut ui, 120, 30);
    let (rect, _) = *ui
        .hits
        .iter()
        .find(|(_, h)| matches!(h, Hit::Status(_)))
        .expect("a glyph hit");
    let moved = |ui: &mut RadarUi, col, row| {
        ui.on_mouse(
            MouseEvent {
                kind: MouseEventKind::Moved,
                column: col,
                row,
                modifiers: KeyModifiers::NONE,
            },
            false,
        )
    };
    moved(&mut ui, rect.x, rect.y);
    assert!(matches!(ui.hover, Some(Hit::Status(_))));
    // Not yet: the dwell is running and the loop is told when it ends.
    assert!(!screen(&mut ui, 120, 30).contains('\u{256D}'));
    assert!(ui.tooltip_due().is_some());
    // After the dwell, a bordered box lists each phrase.
    ui.hover_since = Some(Instant::now() - crate::tooltip::DWELL * 2);
    let text = screen(&mut ui, 120, 30);
    assert!(text.contains('\u{256D}'), "{text}");
    assert!(ui.tooltip_due().is_none());
    // A key hides it, and the pointer must move again.
    ui.on_key(key(KeyCode::Down));
    assert!(!screen(&mut ui, 120, 30).contains('\u{256D}'));
    // A click on the glyphs is a click on the row.
    moved(&mut ui, rect.x, rect.y);
    ui.hover_since = Some(Instant::now() - crate::tooltip::DWELL * 2);
    ui.on_mouse(
        MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: rect.x,
            row: rect.y,
            modifiers: KeyModifiers::NONE,
        },
        false,
    );
    assert!(!screen(&mut ui, 120, 30).contains('\u{256D}'));
    // An open popup owns the pointer: no tooltip under it.
    moved(&mut ui, rect.x, rect.y);
    ui.hover_since = Some(Instant::now() - crate::tooltip::DWELL * 2);
    ui.open_combo(Hit::View);
    let text = screen(&mut ui, 120, 30);
    assert!(
        !text.lines().skip(5).any(|l| l.contains("Checks")),
        "no tooltip under a popup: {text}"
    );
}

/// The Radar of `radar()` plus one repository note (so the list ends
/// with a REPOSITORIES section under the request groups).
fn with_note(dir: &std::path::Path) -> RadarUi {
    let (mut ui, app, _) = radar(dir);
    let mut bad = answer(&app, "acme/app", vec![], vec![]);
    bad.snapshot.problem = Some(crate::hosting_task::Problem::Reconnect("403".into()));
    ui.apply_answer(bad);
    ui.rebuild();
    ui
}

fn blank_at(ui: &RadarUi) -> usize {
    ui.rows.iter().position(|r| *r == Row::Blank).unwrap()
}

#[test]
fn a_blank_line_separates_the_work_sections_from_repositories() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = with_note(dir.path());
    let blank = blank_at(&ui);
    assert!(matches!(
        ui.rows[blank + 1],
        Row::Heading(Heading::Repos, _)
    ));
    let text = screen(&mut ui, 120, 30);
    let lines: Vec<&str> = text.lines().collect();
    let at = lines
        .iter()
        .position(|l| l.contains("REPOSITORIES"))
        .expect("heading");
    assert!(lines[at - 1].trim().is_empty(), "{text}");
    assert!(!lines[at - 2].trim().is_empty(), "{text}");
}

#[test]
fn keys_step_over_the_blank_line_and_the_mouse_ignores_it() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = with_note(dir.path());
    let blank = blank_at(&ui);
    ui.selected = blank - 1;
    ui.on_key(key(KeyCode::Down));
    assert_eq!(ui.selected, blank + 1, "Down lands on REPOSITORIES");
    ui.on_key(key(KeyCode::Up));
    assert_eq!(ui.selected, blank - 1, "Up skips it too");
    // PageUp / PageDown / Home / End never stop on it.
    for code in [
        KeyCode::PageDown,
        KeyCode::PageUp,
        KeyCode::Home,
        KeyCode::End,
    ] {
        ui.on_key(key(code));
        assert_ne!(ui.rows[ui.selected], Row::Blank, "{code:?}");
    }
    // No hit rect covers the blank line.
    screen(&mut ui, 120, 30);
    assert!(!ui.hits.iter().any(|(_, h)| matches!(
        h,
        Hit::Row(i) | Hit::Heading(i) if *i == blank
    )));
}

mod account;
mod columns;
#[cfg(feature = "conversation")]
mod conversation;
mod resize;
#[cfg(feature = "review")]
mod review;
