use super::*;
use crate::th;

#[test]
fn empty_page_lists_types_and_offers_connect() {
    let h = H::new();
    let mut page = IntegrationsPage::default();
    let screen = text(&draw(&mut page, &h, 70, 26, None));
    for want in [
        "Hosting",
        "GitHub",
        "GitLab Self-Managed",
        "No GitHub profile yet.",
        "+ Add profile",
        "[ Connect ]",
    ] {
        assert!(screen.contains(want), "missing {want:?}:\n{screen}");
    }
}

#[test]
fn self_managed_form_is_disabled_until_host_and_token_are_set() {
    let mut h = H::new();
    let mut page = IntegrationsPage::default();
    page.select_kind(GITLAB_SM, &h.config);
    page.add_profile(&h.config);
    let screen = text(&draw(&mut page, &h, 80, 30, None));
    for want in [
        "Host domain",
        "Personal access token",
        "Scopes: api, read_user",
        "Generate a token (enter the host first)",
        "[ Connect ]",
    ] {
        assert!(screen.contains(want), "missing {want:?}:\n{screen}");
    }
    assert!(!page.can_connect(&h.config, &h.presence));
    // Activating the disabled Connect explains itself instead of running.
    page.activate(Item::Connect, &mut h.env());
    assert!(page.pending.is_none());
    assert!(page.error.as_deref().unwrap().contains("host domain"));

    type_str(&mut page, &mut h, "gitlab.example.com");
    key(&mut page, &mut h, KeyCode::Tab);
    type_str(&mut page, &mut h, "glpat-TYPED-SECRET");
    assert!(page.can_connect(&h.config, &h.presence));
    let screen = text(&draw(&mut page, &h, 80, 30, None));
    assert!(
        screen.contains("Generate a token on gitlab.example.com"),
        "{screen}"
    );
    assert!(
        screen.contains("glpat-TYPED-SECRET"),
        "the token must be visible in clear:\n{screen}"
    );
    assert!(!screen.contains("\u{2022}\u{2022}\u{2022}"), "{screen}");
}

#[test]
fn it_survives_every_narrow_size_and_every_view() {
    let mut h = H::new();
    h.connected(IntegrationKind::GitlabSelfManaged, "kai");
    for (w, rows) in [(20, 8), (30, 12), (40, 20), (55, 24), (64, 30), (120, 40)] {
        for kind in 0..IntegrationKind::IMPLEMENTED.len() {
            let mut page = IntegrationsPage::default();
            page.select_kind(kind, &h.config);
            draw(&mut page, &h, w, rows, Some((3, 3)));
            page.add_profile(&h.config);
            draw(&mut page, &h, w, rows, Some((10, 5)));
        }
    }
}

#[test]
fn every_hit_rect_is_inside_the_area_and_hover_uses_the_strong_fill() {
    let h = H::new();
    let mut page = IntegrationsPage::default();
    let buf = draw(&mut page, &h, 70, 26, None);
    assert!(!page.hits.is_empty());
    for (rect, _, _) in &page.hits {
        assert!(rect.right() <= 70 && rect.bottom() <= 26, "{rect:?}");
    }
    let (rect, _, _) = *page
        .hits
        .iter()
        .find(|(_, hit, _)| *hit == Hit::Type(2))
        .unwrap();
    assert_ne!(buf[(rect.x + 3, rect.y)].bg, th().ui.button_hover);
    let hovered = draw(&mut page, &h, 70, 26, Some((rect.x + 3, rect.y)));
    assert_eq!(hovered[(rect.x + 3, rect.y)].bg, th().ui.button_hover);
    for target in [Hit::Item(Item::StartConnect), Hit::AddProfile] {
        let (r, _, _) = *page.hits.iter().find(|(_, hit, _)| *hit == target).unwrap();
        let b = draw(&mut page, &h, 70, 26, Some((r.x + 1, r.y)));
        assert_eq!(b[(r.x + 1, r.y)].bg, th().ui.button_hover, "{target:?}");
    }
}

/// The selected type row is shown by its highlight alone: no arrow glyph
/// before it (it clashed with the expand / collapse arrows).
#[test]
fn the_selected_type_row_has_no_arrow_marker() {
    let h = H::new();
    let mut page = IntegrationsPage::default();
    let screen = text(&draw(&mut page, &h, 70, 26, None));
    assert!(!screen.contains('\u{25B8}'), "{screen}");
}
