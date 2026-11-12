use super::*;

/// A page on a connected GitLab profile, the keyboard on its body.
fn page(h: &mut H) -> IntegrationsPage {
    let mut page = IntegrationsPage::default();
    h.connected(IntegrationKind::GitlabSelfManaged, "kai");
    let index = page.kind_index(IntegrationKind::GitlabSelfManaged).unwrap();
    page.select_kind(index, &h.config);
    page.zone = Zone::Body;
    page
}

/// Draws like Settings does: the page leaves a padding column on its right,
/// where the scrollbar goes.
fn draw(
    page: &mut IntegrationsPage,
    h: &H,
    w: u16,
    rows: u16,
    mouse: Option<(u16, u16)>,
) -> Buffer {
    let mut terminal = Terminal::new(TestBackend::new(w, rows)).unwrap();
    terminal
        .draw(|f| {
            let area = Rect::new(0, 0, w - 1, rows);
            page.render(f, area, &h.config, &h.presence, true, mouse)
        })
        .unwrap();
    terminal.backend().buffer().clone()
}

/// Rows of a terminal where only eight rows of the card show.
const SHORT: u16 = 11;

fn hit_of(page: &IntegrationsPage, item: Item) -> Option<Rect> {
    page.hits
        .iter()
        .find(|(_, h, _)| *h == Hit::Item(item))
        .map(|(r, _, _)| *r)
}

fn inside_window(page: &IntegrationsPage, r: Rect) -> bool {
    let w = page.scroll.window;
    r.y >= w.y && r.bottom() <= w.bottom()
}

fn wheel(page: &mut IntegrationsPage, h: &mut H, kind: MouseEventKind) -> bool {
    let w = page.scroll.window;
    page.on_mouse(kind, w.x + 2, w.y + 2, &mut h.env())
}

#[test]
fn a_tall_terminal_scrolls_nothing_and_draws_no_bar() {
    let mut h = H::new();
    let mut page = page(&mut h);
    draw(&mut page, &h, 100, 70, None);
    assert!(page.scroll.bar.is_none());
    assert_eq!(page.scroll.top, 0);
    assert!(!wheel(&mut page, &mut h, MouseEventKind::ScrollDown));
    assert_eq!(page.scroll.top, 0);
}

#[test]
fn a_short_terminal_scrolls_and_draws_the_bar() {
    let mut h = H::new();
    let mut page = page(&mut h);
    draw(&mut page, &h, 100, 24, None);
    assert!(page.scroll.content > page.scroll.window.height as usize);
    assert!(page.scroll.bar.is_some());
}

#[test]
fn tabbing_down_to_the_combobox_brings_it_into_view() {
    let mut h = H::new();
    let mut page = page(&mut h);
    draw(&mut page, &h, 100, SHORT, None);
    assert!(
        hit_of(&page, Item::RadarMode).is_none(),
        "the combobox starts below the fold"
    );
    page.focus_radar(&h.config);
    for _ in 0..6 {
        key(&mut page, &mut h, KeyCode::Tab);
        draw(&mut page, &h, 100, SHORT, None);
        let current = page.current_item(&h.config).unwrap();
        let r = hit_of(&page, current).expect("the focused control is on screen");
        assert!(inside_window(&page, r), "{current:?} at {r:?}");
    }
    assert_eq!(page.current_item(&h.config), Some(Item::RadarMode));
    let screen = text(&draw(&mut page, &h, 100, SHORT, None));
    assert!(
        screen.contains("Request of a repository that is not cloned"),
        "{screen}"
    );
    // Back up: shift+tab follows too.
    for _ in 0..6 {
        key(&mut page, &mut h, KeyCode::BackTab);
        draw(&mut page, &h, 100, SHORT, None);
        let current = page.current_item(&h.config).unwrap();
        let r = hit_of(&page, current).expect("the focused control is on screen");
        assert!(inside_window(&page, r));
    }
}

#[test]
fn the_palette_row_shows_the_radar_block() {
    let mut h = H::new();
    let mut page = page(&mut h);
    page.zone = Zone::Profiles;
    draw(&mut page, &h, 100, 18, None);
    assert!(hit_of(&page, Item::RadarReview).is_none());
    page.focus_radar(&h.config);
    let screen = text(&draw(&mut page, &h, 100, 18, None));
    assert!(screen.contains("Radar"), "{screen}");
    assert!(hit_of(&page, Item::RadarReview).is_some());
}

#[test]
fn the_wheel_scrolls_and_the_hit_rects_follow_the_drawn_controls() {
    let mut h = H::new();
    let mut page = page(&mut h);
    page.zone = Zone::Profiles;
    draw(&mut page, &h, 100, 24, None);
    // Scroll to the end: the whole Radar block is on screen.
    for _ in 0..40 {
        wheel(&mut page, &mut h, MouseEventKind::ScrollDown);
    }
    let buf = draw(&mut page, &h, 100, 24, None);
    assert!(page.scroll.top > 0);
    for (rect, hit, _) in &page.hits {
        if let Hit::Item(item) = hit {
            assert!(inside_window(&page, *rect), "{item:?} {rect:?}");
        }
    }
    let drafts = hit_of(&page, Item::RadarDrafts).expect("visible at the end");
    let row: String = (drafts.x..drafts.right())
        .map(|x| buf[(x, drafts.y)].symbol())
        .collect();
    assert!(row.contains("Include drafts"), "{row:?}");
    // A click on the drawn control hits that control.
    page.on_mouse(
        MouseEventKind::Down(MouseButton::Left),
        drafts.x + 1,
        drafts.y,
        &mut h.env(),
    );
    assert_eq!(page.current_item(&h.config), Some(Item::RadarDrafts));
    assert!(h.config.integrations[0].radar_options().include_drafts);
    // Nothing above the window answers: the strip row is not a control.
    for _ in 0..40 {
        wheel(&mut page, &mut h, MouseEventKind::ScrollUp);
    }
    assert_eq!(page.scroll.top, 0);
}

#[test]
fn the_wheel_stops_at_the_end() {
    let mut h = H::new();
    let mut page = page(&mut h);
    draw(&mut page, &h, 100, 24, None);
    for _ in 0..40 {
        wheel(&mut page, &mut h, MouseEventKind::ScrollDown);
        draw(&mut page, &h, 100, 24, None);
    }
    assert_eq!(
        page.scroll.top,
        page.scroll.content - page.scroll.window.height as usize
    );
}

#[test]
fn a_wheel_scroll_is_not_undone_by_the_next_frame() {
    let mut h = H::new();
    let mut page = page(&mut h);
    page.focus_radar(&h.config);
    draw(&mut page, &h, 100, 24, None);
    wheel(&mut page, &mut h, MouseEventKind::ScrollDown);
    let top = page.scroll.top;
    draw(&mut page, &h, 100, 24, None);
    draw(&mut page, &h, 100, 24, None);
    assert_eq!(page.scroll.top, top);
}

#[test]
fn the_scrollbar_thumb_drags_the_view() {
    let mut h = H::new();
    let mut page = page(&mut h);
    page.zone = Zone::Profiles;
    draw(&mut page, &h, 100, 24, None);
    let track = page.scroll.bar.unwrap();
    let col = track.rect.x;
    let bottom = track.rect.bottom() - 1;
    page.on_mouse(
        MouseEventKind::Down(MouseButton::Left),
        col,
        track.rect.y + track.thumb_start,
        &mut h.env(),
    );
    assert!(page.owns_pointer());
    page.on_mouse(
        MouseEventKind::Drag(MouseButton::Left),
        col,
        bottom,
        &mut h.env(),
    );
    page.on_mouse(
        MouseEventKind::Up(MouseButton::Left),
        col,
        bottom,
        &mut h.env(),
    );
    assert!(!page.owns_pointer());
    assert_eq!(
        page.scroll.top,
        page.scroll.content - page.scroll.window.height as usize
    );
}

#[test]
fn the_popup_opens_next_to_its_field_when_scrolled_and_takes_hover_and_wheel() {
    let mut h = H::new();
    let mut page = page(&mut h);
    page.focus_radar(&h.config);
    for _ in 0..6 {
        key(&mut page, &mut h, KeyCode::Tab);
        draw(&mut page, &h, 100, SHORT, None);
    }
    let field = hit_of(&page, Item::RadarMode).unwrap();
    let top = page.scroll.top;
    assert!(top > 0);
    key(&mut page, &mut h, KeyCode::Enter);
    assert!(page.combo_open());
    draw(&mut page, &h, 100, SHORT, None);
    let popup = page.radar_combo.popup_rect();
    assert!(
        popup.y == field.bottom() || popup.bottom() == field.y,
        "{popup:?} vs {field:?}"
    );
    let rects = page.radar_combo.item_rects();
    let (second, _) = rects[1];
    page.on_mouse(MouseEventKind::Moved, second.x + 1, second.y, &mut h.env());
    assert_eq!(page.radar_combo.hovered_value(), Some("clone"));
    // The wheel belongs to the popup, not to the page behind it.
    assert!(wheel(&mut page, &mut h, MouseEventKind::ScrollDown));
    assert_eq!(page.scroll.top, top);
}
