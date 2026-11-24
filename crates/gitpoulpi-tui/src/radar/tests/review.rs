//! Review from the Radar: the question of a request of a repository that is
//! not cloned, the profile's `review_mode` that answers it, the API review
//! hosted over the Radar, and the palette's twins.

use super::*;
use crate::radar::account::{AccountAnswer, CloneIndex};
use crate::radar::worker::Setup;
use crate::review_view::ReviewTarget;
use gitpoulpi_hosting::account::{AccountItems, AccountPr};
use std::sync::Arc;

fn far(n: u64) -> AccountPr {
    let mut p = pr(n, "ann", "2026-10-12T10:00:00Z");
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

fn answer_of(rows: Vec<AccountPr>) -> AccountAnswer {
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

/// A Radar with one far request; `mode` is the profile's `review_mode` in
/// the config (`None`: the profile has no Radar table), `clone`: where its
/// repository is cloned, when it is.
fn with_far(dir: &std::path::Path, mode: Option<&str>, clone: Option<&std::path::Path>) -> RadarUi {
    let (mut ui, ..) = radar(dir);
    let radar_table = mode
        .map(|m| format!("\n[integrations.radar]\nreview_requests = true\nreview_mode = \"{m}\"\n"))
        .unwrap_or_default();
    std::fs::write(
        dir.join("cfg").join("config.toml"),
        format!(
            "[[integrations]]\nid = \"gh\"\nkind = \"github\"\nurl = \"https://github.com\"\ntoken_ref = \"integration:gh\"\n{radar_table}"
        ),
    )
    .unwrap();
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
    ui.apply_account(answer_of(vec![far(20)]));
    ui.rebuild();
    select(&mut ui, "Far change 20");
    ui
}

fn mode_in(dir: &std::path::Path) -> gitpoulpi_config::RemoteReviewMode {
    gitpoulpi_config::Config::load(&dir.join("cfg").join("config.toml"))
        .unwrap()
        .integrations[0]
        .radar_options()
        .review_mode
}

fn request(out: Outcome) -> ReviewRequest {
    match out {
        Outcome::ApiReview(r) | Outcome::CloneReview(r) => *r,
        other => panic!("not a review request: {other:?}"),
    }
}

#[test]
fn review_asks_and_a_letter_answers() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = with_far(dir.path(), None, None);
    assert_eq!(press(&mut ui, 'e'), Outcome::None);
    let text = screen(&mut ui, 120, 32);
    for want in [
        "Review request",
        "Far change 20",
        "acme/far",
        "Clone it, then review",
    ] {
        assert!(text.contains(want), "{want}:\n{text}");
    }
    assert!(!text.contains("Open the local clone"), "{text}");
    // Esc: nothing happens.
    assert_eq!(ui.on_key(key(KeyCode::Esc)), Outcome::None);
    assert!(ui.asking.is_none());

    // `a`: the API review of this very request.
    press(&mut ui, 'e');
    let api = request(press(&mut ui, 'a'));
    assert_eq!(
        (api.integration.as_str(), api.project.as_str(), api.number),
        ("gh", "acme/far", 20)
    );
    assert_eq!(api.label, "#20");
    assert_eq!(api.target_branch, "main");
    assert_eq!(api.source_branch, "feat-20");
    assert!(ui.asking.is_none());

    // `c`: the clone, from the address the host gave.
    press(&mut ui, 'e');
    let clone = request(press(&mut ui, 'c'));
    assert_eq!(clone.clone_url, "https://github.com/acme/far.git");
    assert_eq!(clone.number, 20);
}

#[test]
fn a_host_that_gave_no_clone_address_says_so_instead_of_cloning() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = with_far(dir.path(), None, None);
    for item in &mut ui.board.items {
        item.clone_url.clear();
    }
    press(&mut ui, 'e');
    let out = press(&mut ui, 'c');
    assert!(
        matches!(&out, Outcome::Message(m) if m.contains("no clone address") && m.contains("acme/far")),
        "{out:?}"
    );
}

#[test]
fn remember_writes_the_profiles_review_mode_and_the_next_review_skips_the_question() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = with_far(dir.path(), Some("ask"), None);
    press(&mut ui, 'e');
    press(&mut ui, 'r');
    request(press(&mut ui, 'a'));
    assert_eq!(mode_in(dir.path()), gitpoulpi_config::RemoteReviewMode::Api);
    // The profile's other Radar options are left alone.
    let saved = gitpoulpi_config::Config::load(&dir.path().join("cfg").join("config.toml"))
        .unwrap()
        .integrations[0]
        .radar_options();
    assert!(saved.review_requests);
    // No dialog any more: the API review starts at once.
    assert_eq!(request(press(&mut ui, 'e')).number, 20);
    assert!(ui.asking.is_none());

    // A clone pick remembered writes `clone`.
    let dir = tempfile::tempdir().unwrap();
    let mut ui = with_far(dir.path(), Some("ask"), None);
    press(&mut ui, 'e');
    press(&mut ui, 'r');
    request(press(&mut ui, 'c'));
    assert_eq!(
        mode_in(dir.path()),
        gitpoulpi_config::RemoteReviewMode::Clone
    );
    assert!(matches!(press(&mut ui, 'e'), Outcome::CloneReview(_)));
}

#[test]
fn a_remembered_pick_is_not_written_without_the_checkbox() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = with_far(dir.path(), Some("ask"), None);
    press(&mut ui, 'e');
    request(press(&mut ui, 'a'));
    assert_eq!(mode_in(dir.path()), gitpoulpi_config::RemoteReviewMode::Ask);
}

#[test]
fn the_mode_answers_for_a_known_clone_too() {
    let dir = tempfile::tempdir().unwrap();
    let clone = dir.path().join("app");
    // clone mode and a clone that is there: open it.
    let mut ui = with_far(dir.path(), Some("clone"), Some(&clone));
    assert_eq!(
        press(&mut ui, 'e'),
        Outcome::OpenRepo(
            clone.clone(),
            Some(Pending::Review {
                number: 20,
                repo: None,
                link: Some(("gh".into(), "acme/far".into()))
            })
        )
    );
    // ask and a clone that is there: the question offers it.
    let dir = tempfile::tempdir().unwrap();
    let clone = dir.path().join("app");
    let mut ui = with_far(dir.path(), None, Some(&clone));
    assert_eq!(press(&mut ui, 'e'), Outcome::None);
    let text = screen(&mut ui, 120, 32);
    assert!(text.contains("Open the local clone"), "{text}");
    assert!(!text.contains("Clone it, then review"), "{text}");
    assert_eq!(
        press(&mut ui, 'l'),
        Outcome::OpenRepo(
            clone,
            Some(Pending::Review {
                number: 20,
                repo: None,
                link: Some(("gh".into(), "acme/far".into()))
            })
        )
    );
    // api mode: the API even where a clone is.
    let dir = tempfile::tempdir().unwrap();
    let clone = dir.path().join("app");
    let mut ui = with_far(dir.path(), Some("api"), Some(&clone));
    assert!(matches!(press(&mut ui, 'e'), Outcome::ApiReview(_)));
}

#[test]
fn the_palette_twins_skip_the_question() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = with_far(dir.path(), None, None);
    assert!(matches!(
        ui.command(RadarCommand::Act(Act::ReviewApi)),
        Outcome::ApiReview(_)
    ));
    assert!(matches!(
        ui.command(RadarCommand::Act(Act::CloneReview)),
        Outcome::CloneReview(_)
    ));
    // Clone and Review on a request whose repository is cloned opens it.
    let dir = tempfile::tempdir().unwrap();
    let clone = dir.path().join("app");
    let mut ui = with_far(dir.path(), None, Some(&clone));
    assert!(matches!(
        ui.command(RadarCommand::Act(Act::CloneReview)),
        Outcome::OpenRepo(..)
    ));
    // A request of the scope's own repositories: Review opens it, the API
    // twin explains.
    let dir = tempfile::tempdir().unwrap();
    let (mut ui, ..) = radar(dir.path());
    select(&mut ui, "Change 3");
    assert!(matches!(
        ui.command(RadarCommand::Act(Act::Review)),
        Outcome::OpenRepo(..)
    ));
    assert!(matches!(
        ui.command(RadarCommand::Act(Act::ReviewApi)),
        Outcome::Message(_)
    ));
}

#[test]
fn the_question_owns_the_pointer_and_badges_its_own_controls() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = with_far(dir.path(), None, None);
    press(&mut ui, 'e');
    assert!(ui.popup_open(), "popups own the pointer");
    screen(&mut ui, 120, 32);
    let spots: Vec<Spot> = ui.nav_targets().into_iter().map(|(_, s)| s).collect();
    assert_eq!(spots.len(), 4, "{spots:?}");
    assert!(spots.iter().all(|s| matches!(s, Spot::Choice(_))));
    // A badge is a press: the second control is the API row.
    let out = ui.nav_focus(Spot::Choice(1));
    assert_eq!(request(out).number, 20);
    // The keys of the list are not reachable behind the dialog.
    press(&mut ui, 'e');
    ui.on_key(key(KeyCode::Char('x')));
    assert!(ui.asking.is_some(), "an unknown key keeps the question");
    assert_eq!(
        ui.selected_item().map(|i| ui.board.items[i].number),
        Some(20)
    );
}

struct NoHost;

impl crate::hosting_task::ClientFactory for NoHost {
    fn client(
        &self,
        _integration: &gitpoulpi_config::Integration,
        _all: &[gitpoulpi_config::Integration],
    ) -> gitpoulpi_hosting::Result<Box<dyn gitpoulpi_hosting::HostingClient>> {
        Err(gitpoulpi_hosting::Error::Unsupported("test".into()))
    }
}

fn hosted(ui: &mut RadarUi, config: &std::path::Path) {
    let integration = gitpoulpi_config::Integration::new(
        "gh",
        gitpoulpi_config::IntegrationKind::Github,
        "https://github.com",
    );
    let target = ReviewTarget {
        all: vec![integration.clone()],
        integration,
        project: "acme/far".into(),
        number: 20,
        iid_label: "#20".into(),
        title: "Far change 20".into(),
        remote: String::new(),
        web_url: "https://github.com/acme/far/pull/20".into(),
        source_branch: "feat-20".into(),
        target_branch: "main".into(),
        patch_set: None,
        compare_patch_set: None,
        repo_root: None,
    };
    let view = crate::review_view::ReviewView::open_api(
        target,
        Arc::new(NoHost),
        false,
        "s".into(),
        Some(config.to_path_buf()),
    );
    ui.host_review(Box::new(view));
}

#[test]
fn the_hosted_review_is_drawn_over_the_radar_and_owns_keys_and_commands() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = with_far(dir.path(), None, None);
    hosted(&mut ui, dir.path());
    assert!(ui.hosts_review());
    let text = screen(&mut ui, 140, 36);
    assert!(text.contains("Review #20"), "{text}");
    // No badges behind the review, and no popup: the pointer goes to it
    // through the body.
    assert!(ui.nav_targets().is_empty());
    assert!(!ui.popup_open());
    // The palette's availability comes from the view's own checks.
    assert!(ui.review_avail().is_some());
    // The Radar's own commands wait; Back closes the review.
    assert!(matches!(
        ui.command(RadarCommand::Refresh),
        Outcome::Message(_)
    ));
    assert!(ui.hosts_review());
    // A key goes to the view, not the list: `q` does not quit the app.
    assert_ne!(press(&mut ui, 'q'), Outcome::Quit);
    // Esc gives the Radar back.
    ui.on_key(key(KeyCode::Esc));
    assert!(!ui.hosts_review());
    let text = screen(&mut ui, 140, 36);
    assert!(!text.contains("Review #20"), "{text}");
    assert!(text.contains("Far change 20"), "{text}");

    hosted(&mut ui, dir.path());
    assert_eq!(ui.command(RadarCommand::Back), Outcome::None);
    assert!(!ui.hosts_review());
}

#[test]
fn a_request_of_a_host_without_a_review_view_opens_its_page_not_the_question() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = with_far(dir.path(), None, None);
    for item in &mut ui.board.items {
        item.review = false;
        item.url.clear();
    }
    // (Without a page the line says so; with one it opens the browser and
    // says that instead: `pty_radar_review.rs`.)
    assert!(matches!(press(&mut ui, 'e'), Outcome::Message(_)));
    assert!(ui.asking.is_none());
}
