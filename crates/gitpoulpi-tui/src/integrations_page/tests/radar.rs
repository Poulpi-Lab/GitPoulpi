use super::*;
use gitpoulpi_config::RemoteReviewMode;

/// A page on a connected profile of `kind`, the keyboard on its body.
fn radar_page(h: &mut H, kind: IntegrationKind) -> Option<IntegrationsPage> {
    let mut page = IntegrationsPage::default();
    h.connected(kind, "kai");
    let index = page.kind_index(kind)?;
    page.select_kind(index, &h.config);
    page.zone = Zone::Body;
    Some(page)
}

fn saved(h: &H) -> Config {
    Config::load(&h.path).unwrap()
}

fn radar_of(h: &H) -> gitpoulpi_config::IntegrationRadar {
    h.config.integrations[0].radar_options()
}

#[test]
fn a_connected_code_host_shows_the_radar_block_all_off() {
    let mut h = H::new();
    let mut page = radar_page(&mut h, IntegrationKind::GitlabSelfManaged).unwrap();
    let screen = text(&draw(&mut page, &h, 100, 60, None));
    for want in [
        "Radar",
        "[ ] Requests waiting for my review",
        "[ ] Every open request",
        "[ ] Include drafts",
        "[ ] Issues assigned to me",
        "Namespaces",
        "Request of a repository that is not cloned",
        "Ask",
    ] {
        assert!(screen.contains(want), "missing {want:?}:\n{screen}");
    }
    assert!(h.config.integrations[0].radar.is_none());
}

#[test]
fn issues_are_hidden_where_the_host_has_none() {
    for kind in [
        IntegrationKind::Gerrit,
        IntegrationKind::BitbucketDataCenter,
    ] {
        let mut h = H::new();
        let Some(mut page) = radar_page(&mut h, kind) else {
            continue;
        };
        let screen = text(&draw(&mut page, &h, 100, 60, None));
        assert!(screen.contains("Every open request"), "{kind:?}:\n{screen}");
        assert!(!screen.contains("Issues assigned to me"), "{kind:?}");
        assert!(!page.items(&h.config).contains(&Item::RadarIssues));
    }
}

#[test]
fn bitbucket_cloud_says_why_there_are_no_controls() {
    let mut h = H::new();
    let Some(mut page) = radar_page(&mut h, IntegrationKind::Bitbucket) else {
        return;
    };
    let screen = text(&draw(&mut page, &h, 100, 60, None));
    assert!(screen.contains("no account-wide lists"), "{screen}");
    assert!(!screen.contains("Include drafts"));
    assert!(!page
        .items(&h.config)
        .iter()
        .any(|i| matches!(i, Item::RadarReview | Item::RadarMode)));
}

#[test]
fn the_keyboard_toggles_and_saves_at_once() {
    let mut h = H::new();
    let mut page = radar_page(&mut h, IntegrationKind::GitlabSelfManaged).unwrap();
    page.focus_radar(&h.config);
    assert_eq!(page.current_item(&h.config), Some(Item::RadarReview));
    key(&mut page, &mut h, KeyCode::Char(' '));
    assert!(radar_of(&h).review_requests);
    assert!(saved(&h).integrations[0].radar_options().review_requests);
    // Down, Enter: the next checkbox.
    key(&mut page, &mut h, KeyCode::Down);
    key(&mut page, &mut h, KeyCode::Enter);
    assert!(saved(&h).integrations[0].radar_options().open_requests);
    // Toggling it off again.
    key(&mut page, &mut h, KeyCode::Enter);
    assert!(!saved(&h).integrations[0].radar_options().open_requests);
    let screen = text(&draw(&mut page, &h, 100, 60, None));
    assert!(
        screen.contains("[x] Requests waiting for my review"),
        "{screen}"
    );
}

#[test]
fn a_click_toggles_the_checkbox_it_hits() {
    let mut h = H::new();
    let mut page = radar_page(&mut h, IntegrationKind::GitlabSelfManaged).unwrap();
    draw(&mut page, &h, 100, 60, None);
    let (rect, _, hoverable) = *page
        .hits
        .iter()
        .find(|(_, hit, _)| *hit == Hit::Item(Item::RadarDrafts))
        .unwrap();
    assert!(hoverable);
    page.on_mouse(
        MouseEventKind::Down(MouseButton::Left),
        rect.x + 1,
        rect.y,
        &mut h.env(),
    );
    assert!(saved(&h).integrations[0].radar_options().include_drafts);
    assert_eq!(page.current_item(&h.config), Some(Item::RadarDrafts));
}

#[test]
fn namespaces_are_typed_one_per_line_and_saved_as_a_list() {
    let mut h = H::new();
    let mut page = radar_page(&mut h, IntegrationKind::GitlabSelfManaged).unwrap();
    page.focus_radar(&h.config);
    for _ in 0..4 {
        key(&mut page, &mut h, KeyCode::Down);
    }
    assert_eq!(page.current_item(&h.config), Some(Item::RadarNamespaces));
    type_str(&mut page, &mut h, "ptm");
    key(&mut page, &mut h, KeyCode::Enter);
    type_str(&mut page, &mut h, "formation/docs");
    assert_eq!(
        saved(&h).integrations[0].radar_options().namespaces,
        vec!["ptm".to_string(), "formation/docs".to_string()]
    );
    // Both lines are on screen, one per row.
    let screen = text(&draw(&mut page, &h, 100, 60, None));
    assert!(screen.contains("ptm"), "{screen}");
    assert!(screen.contains("formation/docs"), "{screen}");
    // Up on the second line moves inside the area; on the first it leaves it.
    key(&mut page, &mut h, KeyCode::Up);
    assert_eq!(page.current_item(&h.config), Some(Item::RadarNamespaces));
    key(&mut page, &mut h, KeyCode::Up);
    assert_eq!(page.current_item(&h.config), Some(Item::RadarIssues));
}

#[test]
fn the_regular_expressions_checkbox_saves_and_an_invalid_line_is_shown_not_saved() {
    let mut h = H::new();
    let mut page = radar_page(&mut h, IntegrationKind::GitlabSelfManaged).unwrap();
    page.focus_radar(&h.config);
    for _ in 0..4 {
        key(&mut page, &mut h, KeyCode::Down);
    }
    type_str(&mut page, &mut h, "^ptm/");
    key(&mut page, &mut h, KeyCode::Tab);
    assert_eq!(page.current_item(&h.config), Some(Item::RadarRegex));
    key(&mut page, &mut h, KeyCode::Char(' '));
    let radar = saved(&h).integrations[0].radar_options();
    assert!(radar.namespaces_regex);
    assert_eq!(radar.namespaces, vec!["^ptm/".to_string()]);
    let screen = text(&draw(&mut page, &h, 100, 60, None));
    assert!(screen.contains("[x] Regular expressions"), "{screen}");

    // Back into the area: an unclosed group is reported and not saved.
    key(&mut page, &mut h, KeyCode::BackTab);
    assert_eq!(page.current_item(&h.config), Some(Item::RadarNamespaces));
    type_str(&mut page, &mut h, "(oops");
    let screen = text(&draw(&mut page, &h, 100, 60, None));
    assert!(
        screen.contains("Invalid expression: ^ptm/(oops"),
        "{screen}"
    );
    assert_eq!(
        saved(&h).integrations[0].radar_options().namespaces,
        vec!["^ptm/".to_string()],
        "the last good list stays"
    );
    // Fixing it saves again.
    type_str(&mut page, &mut h, ")");
    assert_eq!(
        saved(&h).integrations[0].radar_options().namespaces,
        vec!["^ptm/(oops)".to_string()]
    );
}

#[test]
fn turning_regular_expressions_on_over_an_invalid_line_is_refused() {
    let mut h = H::new();
    let mut page = radar_page(&mut h, IntegrationKind::GitlabSelfManaged).unwrap();
    page.focus_radar(&h.config);
    for _ in 0..4 {
        key(&mut page, &mut h, KeyCode::Down);
    }
    type_str(&mut page, &mut h, "a(b");
    key(&mut page, &mut h, KeyCode::Tab);
    key(&mut page, &mut h, KeyCode::Char(' '));
    assert!(!saved(&h).integrations[0].radar_options().namespaces_regex);
}

#[test]
fn an_old_comma_line_in_the_file_shows_as_lines_and_saves_as_a_list() {
    let mut h = H::new();
    let mut page = radar_page(&mut h, IntegrationKind::GitlabSelfManaged).unwrap();
    h.config.integrations[0].radar = Some(gitpoulpi_config::IntegrationRadar {
        namespaces: gitpoulpi_config::parse_namespace_text("ptm, formation", false),
        ..Default::default()
    });
    let screen = text(&draw(&mut page, &h, 100, 60, None));
    let rows: Vec<&str> = screen.lines().collect();
    let at = |needle: &str| rows.iter().position(|l| l.contains(needle));
    assert!(at("ptm").is_some() && at("formation").is_some(), "{screen}");
    assert_ne!(at("ptm"), at("formation"), "one entry per row:\n{screen}");
}

#[test]
fn the_namespaces_area_has_a_scrollbar_when_the_lines_overflow() {
    let mut h = H::new();
    let mut page = radar_page(&mut h, IntegrationKind::GitlabSelfManaged).unwrap();
    h.config.integrations[0].radar = Some(gitpoulpi_config::IntegrationRadar {
        namespaces: (1..=9).map(|n| format!("group{n}")).collect(),
        ..Default::default()
    });
    let screen = text(&draw(&mut page, &h, 100, 60, None));
    assert!(screen.contains('\u{2588}'), "a thumb:\n{screen}");
    assert!(
        screen.contains("group1") && !screen.contains("group9"),
        "{screen}"
    );
}

#[test]
fn namespaces_typed_comma_separated_still_save_as_a_list() {
    let mut h = H::new();
    let mut page = radar_page(&mut h, IntegrationKind::GitlabSelfManaged).unwrap();
    page.focus_radar(&h.config);
    for _ in 0..4 {
        key(&mut page, &mut h, KeyCode::Down);
    }
    assert_eq!(page.current_item(&h.config), Some(Item::RadarNamespaces));
    assert!(page.text_focused(&h.config));
    type_str(&mut page, &mut h, "acme, platform,, acme,");
    assert_eq!(
        saved(&h).integrations[0].radar_options().namespaces,
        vec!["acme".to_string(), "platform".to_string()]
    );
    // The typed text is kept as typed.
    let screen = text(&draw(&mut page, &h, 100, 60, None));
    assert!(screen.contains("acme, platform,, acme,"), "{screen}");
}

#[test]
fn the_combobox_picks_what_to_do_with_an_uncloned_request() {
    let mut h = H::new();
    let mut page = radar_page(&mut h, IntegrationKind::GitlabSelfManaged).unwrap();
    page.focus_radar(&h.config);
    for _ in 0..6 {
        key(&mut page, &mut h, KeyCode::Down);
    }
    assert_eq!(page.current_item(&h.config), Some(Item::RadarMode));
    key(&mut page, &mut h, KeyCode::Enter);
    assert!(page.combo_open());
    let screen = text(&draw(&mut page, &h, 100, 60, None));
    for want in ["Clone it", "Review through the API"] {
        assert!(screen.contains(want), "missing {want:?}:\n{screen}");
    }
    key(&mut page, &mut h, KeyCode::Down);
    key(&mut page, &mut h, KeyCode::Down);
    key(&mut page, &mut h, KeyCode::Enter);
    assert!(!page.combo_open());
    assert_eq!(radar_of(&h).review_mode, RemoteReviewMode::Api);
    assert_eq!(
        saved(&h).integrations[0].radar_options().review_mode,
        RemoteReviewMode::Api
    );
    // Esc closes without a change.
    key(&mut page, &mut h, KeyCode::Enter);
    key(&mut page, &mut h, KeyCode::Esc);
    assert!(!page.combo_open());
    assert_eq!(radar_of(&h).review_mode, RemoteReviewMode::Api);
}

#[test]
fn a_click_on_the_combobox_and_on_a_choice_picks_it() {
    let mut h = H::new();
    let mut page = radar_page(&mut h, IntegrationKind::GitlabSelfManaged).unwrap();
    draw(&mut page, &h, 100, 60, None);
    let (field, _, _) = *page
        .hits
        .iter()
        .find(|(_, hit, _)| *hit == Hit::Item(Item::RadarMode))
        .unwrap();
    page.on_mouse(
        MouseEventKind::Down(MouseButton::Left),
        field.x + 1,
        field.y,
        &mut h.env(),
    );
    assert!(page.combo_open());
    draw(&mut page, &h, 100, 60, None);
    let rows = page.radar_combo.item_rects();
    let (row, _) = rows[1];
    page.on_mouse(
        MouseEventKind::Down(MouseButton::Left),
        row.x + 1,
        row.y,
        &mut h.env(),
    );
    assert_eq!(radar_of(&h).review_mode, RemoteReviewMode::Clone);
}

#[test]
fn the_card_shows_the_saved_namespaces() {
    let mut h = H::new();
    let mut page = radar_page(&mut h, IntegrationKind::GitlabSelfManaged).unwrap();
    h.config.integrations[0].radar = Some(gitpoulpi_config::IntegrationRadar {
        namespaces: vec!["one".into(), "two".into()],
        ..Default::default()
    });
    let screen = text(&draw(&mut page, &h, 100, 60, None));
    assert!(screen.contains("one") && screen.contains("two"), "{screen}");
}
