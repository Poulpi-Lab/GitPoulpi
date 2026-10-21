//! The Radar with account-wide rows: the checkboxes, the hint row, the
//! "not cloned" rows and what their actions say.

use super::*;
use crate::radar::account::{AccountAnswer, CloneIndex, Offer};
use crate::radar::worker::Setup;
use gitpoulpi_hosting::account::{AccountIssue, AccountItems, AccountPr};
use ratatui::style::Color;

fn far(n: u64, state: PullRequestState) -> AccountPr {
    let mut p = pr(n, "ann", "2026-10-12T10:00:00Z");
    p.state = state;
    p.review_requested = true;
    AccountPr {
        project: "acme/far".into(),
        clone_url: "https://github.com/acme/far.git".into(),
        pr: PullRequestSummary {
            title: format!("Far change {n}"),
            ..p
        },
    }
}

fn account(rows: Vec<AccountPr>) -> AccountAnswer {
    AccountAnswer {
        round: 0,
        id: "gh".into(),
        label: "GitHub".into(),
        kind: gitpoulpi_config::IntegrationKind::Github,
        host: "github.com".into(),
        me: "kai".into(),
        items: AccountItems {
            prs: rows,
            ..AccountItems::default()
        },
        fetched_at: 0,
        cached: false,
        problem: None,
        paused: None,
    }
}

fn setup(ui: &mut RadarUi, offers: Vec<Offer>, active: &[&str]) {
    ui.apply_setup(Setup {
        round: ui.round,
        active: active.iter().map(|s| s.to_string()).collect(),
        offers,
        clones: CloneIndex::default(),
    });
}

/// A Radar with two cloned repositories and two far requests (one draft).
fn with_far(dir: &std::path::Path) -> RadarUi {
    let (mut ui, ..) = radar(dir);
    setup(&mut ui, vec![], &["gh"]);
    ui.apply_account(account(vec![
        far(20, PullRequestState::Open),
        far(21, PullRequestState::Draft),
    ]));
    ui.rebuild();
    ui
}

fn titles(ui: &RadarUi) -> Vec<String> {
    ui.board.items.iter().map(|i| i.title.clone()).collect()
}

#[test]
fn far_rows_are_listed_dimmed_with_a_word_and_the_two_checkboxes() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = with_far(dir.path());
    assert!(titles(&ui).contains(&"Far change 20".to_string()));
    assert!(
        !titles(&ui).contains(&"Far change 21".to_string()),
        "drafts are hidden by default"
    );
    select(&mut ui, "Far change 20");
    let text = screen(&mut ui, 140, 30);
    assert!(text.contains("[x] Not cloned"), "{text}");
    assert!(text.contains("[ ] Drafts"), "{text}");
    let row = text
        .lines()
        .skip(5)
        .find(|l| l.contains("Far change 20"))
        .expect("the row");
    assert!(row.contains("acme/far"), "{row}");
    // The status line names the selected row's state.
    let status = text.lines().nth(4).unwrap();
    assert!(status.contains("Not cloned"), "{status}");
    // The chip is dimmed: the muted color, not a repository color.
    let (x, y) = {
        let mut terminal = Terminal::new(TestBackend::new(140, 30)).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                ui.render(f, area);
            })
            .unwrap();
        let buf = terminal.backend().buffer().clone();
        let y = (0..30)
            .find(|&y| {
                (0..140)
                    .map(|x| buf[(x, y)].symbol().to_string())
                    .collect::<String>()
                    .contains("Far change 20")
                    && y > 4
            })
            .unwrap();
        let x = (0..140)
            .find(|&x| buf[(x, y)].symbol() == "a" && buf[(x + 1, y)].symbol() == "c")
            .unwrap();
        assert_eq!(buf[(x, y)].fg, crate::th().ui.muted, "dimmed chip");
        (x, y)
    };
    assert!(x > 0 && y > 0);
}

#[test]
fn the_checkboxes_are_not_drawn_while_no_list_is_asked() {
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, ..) = radar(dir.path());
    let text = screen(&mut ui, 140, 30);
    assert!(
        !text.contains("Not cloned") && !text.contains("Drafts"),
        "{text}"
    );
}

#[test]
fn a_and_d_flip_the_checkboxes_and_radar_toml_remembers() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = with_far(dir.path());
    let cfg = dir.path().join("cfg");
    // Drafts on: the draft row comes.
    press(&mut ui, 'd');
    assert!(titles(&ui).contains(&"Far change 21".to_string()));
    assert!(screen(&mut ui, 140, 30).contains("[x] Drafts"));
    // Not cloned off: both far rows go, the cloned ones stay.
    press(&mut ui, 'a');
    assert!(!titles(&ui).iter().any(|t| t.starts_with("Far change")));
    assert!(titles(&ui).contains(&"Change 1".to_string()));
    assert!(screen(&mut ui, 140, 30).contains("[ ] Not cloned"));
    let saved = RadarState::load(&RadarState::file_in(&cfg));
    assert!(saved.show_drafts && !saved.show_not_cloned);
    // A new Radar starts the way this one was left.
    let reopened = RadarUi::with_parts("s", Some(cfg), None, clock);
    assert!(reopened.state.show_drafts && !reopened.state.show_not_cloned);
}

#[test]
fn clicking_a_checkbox_does_what_its_key_does() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = with_far(dir.path());
    screen(&mut ui, 140, 30);
    let (rect, _) = *ui
        .hits
        .iter()
        .find(|(_, h)| *h == Hit::NotCloned)
        .expect("a hit rect");
    ui.on_mouse(
        MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: rect.x,
            row: rect.y,
            modifiers: KeyModifiers::NONE,
        },
        false,
    );
    assert!(!ui.state.show_not_cloned);
}

#[test]
fn the_palette_commands_reach_the_checkboxes() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = with_far(dir.path());
    ui.command(RadarCommand::Drafts);
    assert!(ui.state.show_drafts);
    ui.command(RadarCommand::NotCloned);
    assert!(!ui.state.show_not_cloned);
}

#[test]
fn ctrl_n_badges_reach_the_checkboxes() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = with_far(dir.path());
    screen(&mut ui, 140, 30);
    let spots: Vec<Spot> = ui.nav_targets().into_iter().map(|(_, s)| s).collect();
    assert!(spots.contains(&Spot::NotCloned) && spots.contains(&Spot::Drafts));
    ui.nav_focus(Spot::Drafts);
    assert!(ui.state.show_drafts);
}

#[test]
fn a_far_row_says_clone_it_first_and_review_waits_for_the_choice() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = with_far(dir.path());
    select(&mut ui, "Far change 20");
    for c in ['c', 't', 's'] {
        assert_eq!(
            press(&mut ui, c),
            Outcome::Message("clone the repository first".into()),
            "key {c}"
        );
    }
    // An issue of a repository that is not cloned says the same.
    ui.account_answers[0].items.issues = vec![AccountIssue {
        project: "acme/far".into(),
        clone_url: String::new(),
        issue: IssueSummary {
            number: 30,
            iid_label: "#30".into(),
            reference: "#30".into(),
            title: "Far bug".into(),
            ..IssueSummary::default()
        },
    }];
    ui.rebuild();
    select(&mut ui, "Far bug");
    assert_eq!(
        press(&mut ui, 'n'),
        Outcome::Message("clone the repository first".into())
    );
    select(&mut ui, "Far change 20");
    // Review asks how (tests/review.rs follows the answers).
    assert_eq!(press(&mut ui, 'e'), Outcome::None);
    assert!(ui.asking.is_some());
    ui.asking = None;
    assert_eq!(
        ui.run(Act::OpenRepo),
        Outcome::Message("clone the repository first".into())
    );
    // The menu offers what needs no repository.
    let index = ui.selected_item().unwrap();
    let entries = ui.item_entries(index);
    let labels: Vec<&str> = entries
        .iter()
        .filter_map(|e| match e {
            Entry::Action { label, .. } => Some(label.as_str()),
            _ => None,
        })
        .collect();
    assert!(labels.contains(&"Open in browser"), "{labels:?}");
    assert!(!labels.contains(&"Check out branch"), "{labels:?}");
}

#[test]
fn a_known_clone_opens_its_repository() {
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, app, _) = radar(dir.path());
    ui.apply_setup(Setup {
        round: ui.round,
        active: vec!["gh".into()],
        offers: vec![],
        clones: CloneIndex::of(&[("github.com", "acme/far", &app)]),
    });
    ui.apply_account(account(vec![far(20, PullRequestState::Open)]));
    ui.rebuild();
    select(&mut ui, "Far change 20");
    assert_eq!(
        press(&mut ui, 'c'),
        Outcome::OpenRepo(app, Some(Pending::CheckoutPr { number: 20 }))
    );
}

#[test]
fn review_is_offered_where_the_review_view_exists() {
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, app, _) = radar(dir.path());
    select(&mut ui, "Change 2");
    assert!(ui
        .board
        .items
        .iter()
        .all(|i| i.kind != ItemKind::Pr || i.review));
    assert_eq!(
        press(&mut ui, 'e'),
        Outcome::OpenRepo(
            app,
            Some(Pending::Review {
                number: 2,
                repo: None
            })
        )
    );
}

fn with_config(dir: &std::path::Path) -> (RadarUi, PathBuf) {
    let (mut ui, ..) = radar(dir);
    let cfg = dir.join("cfg");
    let file = cfg.join("config.toml");
    std::fs::write(
        &file,
        "[[integrations]]\nid = \"gh\"\nkind = \"github\"\nurl = \"https://github.com\"\ntoken_ref = \"integration:gh\"\n\n[integrations.radar]\nassigned_issues = false\ninclude_drafts = true\nnamespaces = [\"acme\"]\n",
    )
    .unwrap();
    setup(
        &mut ui,
        vec![Offer {
            id: "gh".into(),
            label: "GitHub".into(),
        }],
        &[],
    );
    ui.rebuild();
    (ui, file)
}

fn radar_of(file: &std::path::Path) -> gitpoulpi_config::IntegrationRadar {
    gitpoulpi_config::Config::load(file).unwrap().integrations[0].radar_options()
}

#[test]
fn the_hint_row_opens_the_list_and_enter_turns_the_lists_on() {
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, file) = with_config(dir.path());
    assert_eq!(ui.rows[0], Row::Hint(0));
    let text = screen(&mut ui, 140, 30);
    let line = text.lines().find(|l| l.contains("Turn on")).expect("hint");
    assert!(line.contains("GitHub"), "{line}");
    ui.selected = 0;
    let out = ui.on_key(key(KeyCode::Enter));
    assert!(matches!(out, Outcome::Message(m) if m.contains("GitHub") && m.contains("turned on")));
    let radar = radar_of(&file);
    assert!(radar.review_requests);
    // The profile's other Radar options are left alone.
    assert!(radar.include_drafts && radar.namespaces == vec!["acme".to_string()]);
    assert!(ui.offers.is_empty(), "the hint is gone");
    assert!(ui.force_account || ui.worker.is_none());
}

#[test]
fn the_turn_on_key_the_button_and_the_palette_do_the_same() {
    // The key, from anywhere in the list.
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, file) = with_config(dir.path());
    ui.selected = ui.rows.len() - 1;
    press(&mut ui, 'T');
    assert!(radar_of(&file).review_requests);
    // The button.
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, file) = with_config(dir.path());
    screen(&mut ui, 140, 30);
    let (rect, _) = *ui
        .hits
        .iter()
        .find(|(_, h)| matches!(h, Hit::TurnOn(_)))
        .expect("the button");
    ui.on_mouse(
        MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: rect.x,
            row: rect.y,
            modifiers: KeyModifiers::NONE,
        },
        false,
    );
    assert!(radar_of(&file).review_requests);
    // The palette command.
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, file) = with_config(dir.path());
    ui.command(RadarCommand::Act(Act::TurnOn));
    assert!(radar_of(&file).review_requests);
}

#[test]
fn turning_on_with_nothing_offered_says_so_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, ..) = radar(dir.path());
    assert_eq!(
        press(&mut ui, 'T'),
        Outcome::Message("nothing to turn on".into())
    );
}

#[test]
fn the_hint_stays_out_of_the_other_views() {
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, _) = with_config(dir.path());
    ui.picked(Hit::View, "issues");
    assert!(!ui.rows.iter().any(|r| matches!(r, Row::Hint(_))));
}

#[test]
fn a_failed_profile_is_a_note_row_with_reconnect() {
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, ..) = radar(dir.path());
    setup(&mut ui, vec![], &["gh"]);
    let mut rejected = account(vec![]);
    rejected.problem = Some(crate::hosting_task::Problem::Reconnect("401".into()));
    ui.apply_account(rejected);
    ui.rebuild();
    let at = ui
        .rows
        .iter()
        .position(|r| matches!(r, Row::Note(i) if ui.notes[*i].name == "GitHub"))
        .expect("a note for the profile");
    ui.selected = at;
    assert_eq!(
        ui.run(Act::Reconnect),
        Outcome::Integrations(Some("gh".into()))
    );
    // A truncated list says how many rows were cut.
    let mut cut = account(vec![]);
    cut.items.notes = vec!["12 more pull/merge requests are not listed (limit 300)".into()];
    ui.apply_account(cut);
    ui.rebuild();
    assert!(screen(&mut ui, 140, 30).contains("12 more"));
}

#[test]
fn turning_a_profile_off_drops_its_rows_at_the_next_setup() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = with_far(dir.path());
    assert!(titles(&ui).contains(&"Far change 20".to_string()));
    setup(&mut ui, vec![], &[]);
    ui.rebuild();
    assert!(!titles(&ui).contains(&"Far change 20".to_string()));
    let _ = Color::Reset;
}

#[test]
fn unticking_not_cloned_with_a_far_row_selected_never_indexes_the_old_board() {
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, ..) = radar(dir.path());
    setup(&mut ui, vec![], &["gh"]);
    let mut last = far(30, PullRequestState::Open);
    last.project = "zeta/far".into();
    // The oldest item sorts last: the highest index of the board.
    last.pr.updated_at = "2020-01-01T10:00:00Z".into();
    last.clone_url = "https://github.com/zeta/far.git".into();
    ui.apply_account(account(vec![far(20, PullRequestState::Open), last]));
    ui.rebuild();
    select(&mut ui, "Far change 30");
    // The rebuild swaps the board before the rows; the selection lookup must
    // not read the new board through the old rows.
    ui.toggle_not_cloned();
    assert!(!titles(&ui).contains(&"Far change 30".to_string()));
    for row in &ui.rows {
        if let Row::Item(i) = row {
            assert!(*i < ui.board.items.len(), "rows point inside the board");
        }
    }
    ui.toggle_not_cloned();
    assert!(titles(&ui).contains(&"Far change 30".to_string()));
}
