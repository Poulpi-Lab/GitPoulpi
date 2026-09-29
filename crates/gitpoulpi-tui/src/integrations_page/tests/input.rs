use super::*;

#[test]
fn clicking_a_type_and_the_add_chip_drives_the_page() {
    let mut h = H::new();
    let mut page = IntegrationsPage::default();
    draw(&mut page, &h, 70, 26, None);
    let (rect, _, _) = *page
        .hits
        .iter()
        .find(|(_, hit, _)| *hit == Hit::Type(3))
        .unwrap();
    assert!(page.on_mouse(
        MouseEventKind::Down(MouseButton::Left),
        rect.x + 2,
        rect.y,
        &mut h.env()
    ));
    assert_eq!(page.kind, 3);
    draw(&mut page, &h, 70, 26, None);
    let (add, _, _) = *page
        .hits
        .iter()
        .find(|(_, hit, _)| *hit == Hit::AddProfile)
        .unwrap();
    page.on_mouse(
        MouseEventKind::Down(MouseButton::Left),
        add.x + 1,
        add.y,
        &mut h.env(),
    );
    assert!(matches!(page.mode, Mode::Form(None)));
    assert_eq!(page.zone, Zone::Body);
    // A click that hits nothing is not the page's.
    assert!(!page.on_mouse(
        MouseEventKind::Down(MouseButton::Left),
        69,
        25,
        &mut h.env()
    ));
}

#[test]
fn keyboard_walks_types_profiles_and_form_and_backs_out_with_esc() {
    let mut h = H::new();
    let mut page = IntegrationsPage::default();
    key(&mut page, &mut h, KeyCode::Char('j'));
    assert_eq!(page.kind, 1);
    key(&mut page, &mut h, KeyCode::Tab);
    assert_eq!(page.zone, Zone::Profiles);
    key(&mut page, &mut h, KeyCode::Tab);
    assert_eq!(page.zone, Zone::Body);
    key(&mut page, &mut h, KeyCode::BackTab);
    assert_eq!(page.zone, Zone::Profiles);
    // `a` opens the form; typing letters goes into the field.
    key(&mut page, &mut h, KeyCode::Char('a'));
    assert!(matches!(page.mode, Mode::Form(None)));
    assert!(page.text_focused(&h.config));
    type_str(&mut page, &mut h, "ghe.corp.io");
    assert_eq!(page.host, "ghe.corp.io");
    // Esc leaves the field's zone, then the strip, then the page.
    key(&mut page, &mut h, KeyCode::Esc);
    assert_eq!(page.zone, Zone::Profiles);
    key(&mut page, &mut h, KeyCode::Esc);
    assert_eq!(page.zone, Zone::Types);
    assert_eq!(key(&mut page, &mut h, KeyCode::Esc), Nav::Sections);
}

#[test]
fn a_token_is_never_copied_and_a_paste_lands_on_one_line() {
    let mut h = H::new();
    let mut page = IntegrationsPage::default();
    page.select_kind(GITLAB_SM, &h.config);
    page.add_profile(&h.config);
    type_str(&mut page, &mut h, "gitlab.example.com");
    key(&mut page, &mut h, KeyCode::Tab);
    assert!(page.paste("glpat-PASTED\n", &h.config));
    assert_eq!(page.token, "glpat-PASTED");
    page.on_key(
        &KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL),
        &mut h.env(),
    );
    page.on_key(
        &KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
        &mut h.env(),
    );
    assert!(h.message.is_none(), "{:?}", h.message);
}
