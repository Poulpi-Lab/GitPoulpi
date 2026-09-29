use super::*;

#[test]
fn reopening_a_connected_profile_shows_the_card_and_edit_prefills_the_token() {
    let mut h = H::new();
    h.connected(IntegrationKind::GitlabSelfManaged, "kai");
    let (mut page, _) = page_with(gitlab_ok());
    page.select_kind(GITLAB_SM, &h.config);
    assert_eq!(page.view(&h.config), View::Connected(0));
    let screen = text(&draw(&mut page, &h, 80, 26, None));
    for want in ["Kai G (@kai)", "Connected", "[ Edit ]", "[ Remove ]", "SSH"] {
        assert!(
            screen.contains(want),
            "missing {want:?}:
{screen}"
        );
    }
    for gone in ["Host domain", "[ Connect ]"] {
        assert!(
            !screen.contains(gone),
            "{gone:?} shown:
{screen}"
        );
    }
    page.activate(Item::Edit, &mut h.env());
    assert_eq!(page.view(&h.config), View::Form);
    let screen = text(&draw(&mut page, &h, 80, 30, None));
    assert!(screen.contains("Personal access token"), "{screen}");
    assert!(screen.contains("glpat-secret"), "{screen}");
    assert!(!screen.contains("[ Edit ]"), "{screen}");
}

#[test]
fn disconnect_keeps_the_profile_and_remove_confirms_then_deletes_the_token() {
    let mut h = H::new();
    let id = h.connected(IntegrationKind::GitlabSelfManaged, "kai");
    let token_ref = Integration::token_ref_for(&id);
    let mut page = IntegrationsPage::default();
    page.select_kind(GITLAB_SM, &h.config);
    assert_eq!(page.view(&h.config), View::Connected(0));

    page.activate(Item::Disconnect, &mut h.env());
    assert_eq!(h.config.integrations.len(), 1);
    assert!(h.config.integrations[0].user.is_none());
    assert_eq!(h.tokens.load(&token_ref).unwrap(), None);
    assert_eq!(h.presence.get(&token_ref), Some(&Some(false)));
    assert_eq!(page.view(&h.config), View::Form);
    assert_eq!(page.host, "https://gitlab.example.com");

    // Back to a connected card, then remove.
    h.config.integrations[0].user = Some(ConnectedUser::default());
    page.reset_view(&h.config);
    page.activate(Item::Remove, &mut h.env());
    assert!(matches!(page.view(&h.config), View::Confirm(0)));
    let screen = text(&draw(&mut page, &h, 80, 26, None));
    assert!(screen.contains("Remove profile"), "{screen}");
    page.activate(Item::CancelRemove, &mut h.env());
    assert_eq!(h.config.integrations.len(), 1);
    page.activate(Item::Remove, &mut h.env());
    page.activate(Item::ConfirmRemove, &mut h.env());
    assert!(h.config.integrations.is_empty());
    assert!(!h.presence.contains_key(&token_ref));
}

#[test]
fn an_unconnected_profile_offers_remove_and_the_removal_survives_a_reload() {
    let mut h = H::new();
    let gitlab = Integration::new("gitlab", IntegrationKind::Gitlab, "https://gitlab.com");
    h.config.integrations.push(gitlab);
    let mut page = IntegrationsPage::default();
    let gitlab_idx = IntegrationKind::ALL
        .iter()
        .position(|k| *k == IntegrationKind::Gitlab)
        .unwrap();
    page.select_kind(gitlab_idx, &h.config);
    // Never connected: opens in its form, which must offer Remove.
    assert_eq!(page.view(&h.config), View::Form);
    assert!(page.items(&h.config).contains(&Item::Remove));
    let screen = text(&draw(&mut page, &h, 140, 40, None));
    assert!(screen.contains("[ Remove ]"), "{screen}");

    // Keeping it returns to the form, not to an empty page.
    page.activate(Item::Remove, &mut h.env());
    assert!(matches!(page.view(&h.config), View::Confirm(0)));
    page.activate(Item::CancelRemove, &mut h.env());
    assert_eq!(page.view(&h.config), View::Form);

    page.activate(Item::Remove, &mut h.env());
    page.activate(Item::ConfirmRemove, &mut h.env());
    assert!(h.config.integrations.is_empty());
    assert!(
        Config::load(&h.path).unwrap().integrations.is_empty(),
        "the removal was persisted and nothing re-creates the profile"
    );
}

#[test]
fn two_profiles_of_one_kind_share_the_strip() {
    let mut h = H::new();
    h.connected(IntegrationKind::Github, "octo");
    h.connected(IntegrationKind::Github, "work-octo");
    let mut page = IntegrationsPage::default();
    let screen = text(&draw(&mut page, &h, 90, 26, None));
    assert!(screen.contains("@octo"), "{screen}");
    assert!(screen.contains("@work-octo"), "{screen}");
    assert!(screen.contains("+ Add profile"), "{screen}");
    assert!(screen.contains("   2"), "{screen}");
    let (rect, _, _) = *page
        .hits
        .iter()
        .find(|(_, hit, _)| *hit == Hit::Profile(1))
        .unwrap();
    page.on_mouse(
        MouseEventKind::Down(MouseButton::Left),
        rect.x,
        rect.y,
        &mut h.env(),
    );
    assert_eq!(page.sel, 1);
    assert_eq!(page.zone, Zone::Profiles);
}

#[test]
fn products_without_a_client_are_not_listed() {
    let h = H::new();
    let mut page = IntegrationsPage::default();
    let screen = text(&draw(&mut page, &h, 80, 26, None));
    assert!(
        !screen.contains("soon"),
        "a product is listed as soon:\n{screen}"
    );
}
