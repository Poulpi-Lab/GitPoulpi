use super::*;
use gitpoulpi_config::RedmineMarkup;

/// A page on a connected Redmine profile, the keyboard on its text format row.
fn redmine_page(h: &mut H) -> IntegrationsPage {
    let mut page = IntegrationsPage::default();
    h.connected(IntegrationKind::Redmine, "kai");
    let index = page.kind_index(IntegrationKind::Redmine).unwrap();
    page.select_kind(index, &h.config);
    page.zone = Zone::Body;
    let at = page
        .items(&h.config)
        .iter()
        .position(|i| *i == Item::RedmineMarkup)
        .expect("the row is offered");
    page.item = at;
    page
}

fn saved(h: &H) -> Config {
    Config::load(&h.path).unwrap()
}

#[test]
fn a_redmine_profile_shows_its_text_format_and_textile_is_the_default() {
    let mut h = H::new();
    let mut page = redmine_page(&mut h);
    let screen = text(&draw(&mut page, &h, 100, 60, None));
    assert!(
        screen.contains("Text format of the server: Textile"),
        "{screen}"
    );
    assert!(screen.contains("Administration > Settings"), "{screen}");
    assert_eq!(h.config.integrations[0].markup, RedmineMarkup::Textile);
}

#[test]
fn enter_switches_the_format_and_saves_at_once() {
    let mut h = H::new();
    let mut page = redmine_page(&mut h);
    assert_eq!(page.current_item(&h.config), Some(Item::RedmineMarkup));
    key(&mut page, &mut h, KeyCode::Enter);
    assert_eq!(saved(&h).integrations[0].markup, RedmineMarkup::Markdown);
    let screen = text(&draw(&mut page, &h, 100, 60, None));
    assert!(
        screen.contains("Text format of the server: Markdown"),
        "{screen}"
    );
    key(&mut page, &mut h, KeyCode::Enter);
    assert_eq!(saved(&h).integrations[0].markup, RedmineMarkup::Textile);
}

#[test]
fn a_click_switches_it_too_and_other_kinds_have_no_such_row() {
    let mut h = H::new();
    let mut page = redmine_page(&mut h);
    draw(&mut page, &h, 100, 60, None);
    let (rect, _, hoverable) = *page
        .hits
        .iter()
        .find(|(_, hit, _)| *hit == Hit::Item(Item::RedmineMarkup))
        .unwrap();
    assert!(hoverable);
    page.on_mouse(
        MouseEventKind::Down(MouseButton::Left),
        rect.x + 1,
        rect.y,
        &mut h.env(),
    );
    assert_eq!(saved(&h).integrations[0].markup, RedmineMarkup::Markdown);

    let mut other = H::new();
    other.connected(IntegrationKind::GitlabSelfManaged, "kai");
    let mut page = IntegrationsPage::default();
    let index = page.kind_index(IntegrationKind::GitlabSelfManaged).unwrap();
    page.select_kind(index, &other.config);
    assert!(!page.items(&other.config).contains(&Item::RedmineMarkup));
    assert!(!text(&draw(&mut page, &other, 100, 60, None)).contains("Text format"));
}

#[test]
fn a_default_format_is_not_written_to_the_file() {
    let mut i = gitpoulpi_config::Integration::new("rm", IntegrationKind::Redmine, "https://r.io");
    assert!(!toml::to_string(&i).unwrap().contains("markup"));
    i.markup = RedmineMarkup::Markdown;
    let text = toml::to_string(&i).unwrap();
    assert!(text.contains("markup = \"markdown\""), "{text}");
}
