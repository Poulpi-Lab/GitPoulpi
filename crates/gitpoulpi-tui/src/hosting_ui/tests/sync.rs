//! How a snapshot, a problem or the worker round trip reaches the sections.

use super::*;

#[test]
fn a_rejected_token_shows_a_reconnect_row_that_opens_preferences_on_the_profile() {
    let mut snap = snapshot(vec![], vec![]);
    snap.problem = Some(Problem::Reconnect("401".into()));
    let (mut ui, _d) = ui_with(snap);
    assert_eq!(ui.rows(HostingList::Pulls)[2], HostingRow::Reconnect);
    // The issues carry their own problem, from their own profile.
    assert!(!ui
        .rows(HostingList::Issues)
        .contains(&HostingRow::Reconnect));
    let (screen, _, _) = draw_section(&ui, HostingList::Pulls, 40, 8, None);
    assert!(screen.contains("Reconnect (token rejected)"), "{screen}");
    let mut clip = crate::clipboard::Clipboard::new();
    assert_eq!(
        ui.activate_row(HostingList::Pulls, 2, &mut clip),
        HostingEffect::Preferences(Some("corp".into()))
    );
}

#[test]
fn offline_keeps_the_stale_list_and_says_how_old() {
    let (mut ui, _d) = ui_with(snapshot(vec![open_pr(53)], vec![]));
    let mut failed = snapshot(vec![], vec![]);
    failed.problem = Some(Problem::Offline("no route".into()));
    failed.fetched_at = 5_000;
    ui.set_clock(|| 1_000 + 600);
    ui.apply(failed);
    assert_eq!(ui.data_len(HostingList::Pulls), 1, "stale data kept");
    assert_eq!(ui.snapshot().unwrap().fetched_at, 1_000);
    let (screen, _, _) = draw_section(&ui, HostingList::Pulls, 50, 8, None);
    assert!(screen.contains("updated 10m ago"), "{screen}");
    assert!(screen.contains("!53"), "{screen}");
}

#[test]
fn a_rate_limited_refresh_keeps_the_stale_list() {
    let (mut ui, _d) = ui_with(snapshot(vec![open_pr(53)], vec![]));
    let mut limited = snapshot(vec![], vec![]);
    limited.problem = Some(Problem::from_error(
        &gitpoulpi_hosting::Error::RateLimited {
            host: "api.github.com".into(),
            retry_in_secs: 120,
        },
    ));
    ui.apply(limited);
    assert_eq!(ui.data_len(HostingList::Pulls), 1, "stale data kept");
    assert!(matches!(
        ui.snapshot().unwrap().problem,
        Some(Problem::RateLimited(_))
    ));
    assert!(!ui
        .snapshot()
        .unwrap()
        .problem
        .as_ref()
        .unwrap()
        .wants_reconnect());
}

#[test]
fn a_full_round_trip_through_the_worker_fills_the_sections() {
    let mut factory = FakeFactory::new();
    factory.prs = vec![open_pr(53)];
    factory.issues = vec![issue(12)];
    factory.head = Some(open_pr(53));
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("hosting-cache.json");
    let pick = gitpoulpi_session::IssuesSource {
        integration: "corp".into(),
        project: "g/p".into(),
        filter: None,
    };
    gitpoulpi_session::RepoState {
        pulls_source: Some(pick.clone()),
        issues_source: Some(pick),
        ..Default::default()
    }
    .save(&crate::remotes_ui::repo_state_path(dir.path()))
    .unwrap();
    let mut ui = HostingUi::with_parts(
        dir.path(),
        None,
        Some(HostingUi::spawn_worker(Arc::new(factory))),
        profiles,
        Some(cache.clone()),
    );
    let mut head = local_branch_at("feature", oid(1));
    head.is_head = true;
    head.upstream = Some("origin/feature".into());
    let refs = vec![head, remote_branch_at("origin", "feature", oid(1))];
    let remotes = vec![Remote {
        name: "origin".into(),
        fetch_url: "git@gitlab.example.com:g/p.git".into(),
        push_url: None,
    }];
    ui.on_refresh(Instant::now(), &remotes, &refs);
    let deadline = Instant::now() + Duration::from_secs(10);
    while !ui.poll() {
        assert!(Instant::now() < deadline, "the worker never answered");
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(ui.sections_visible(), (true, true));
    assert_eq!(ui.chip().unwrap().text, "GL !53");
    // And it was cached, so the next launch paints before the network.
    let mut next = HostingUi::with_parts(dir.path(), None, None, profiles, Some(cache));
    next.load_cache();
    assert_eq!(next.sections_visible(), (true, true));
    assert!(!next.fresh);
}
