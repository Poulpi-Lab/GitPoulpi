//! Buttons are real buttons (padded, filled, the hand pointer), never
//! bracketed text; the Radar block's Namespaces area has a pressable
//! scrollbar and every control is badged for `Ctrl+N`.

use super::*;
use crate::pointer_map::{self, Kind};
use crate::th;

/// The screen text with the checkbox marks (`[x]`, `[ ]`) taken out: what is
/// left must have no bracket at all.
fn without_marks(screen: &str) -> String {
    screen.replace("[x]", "   ").replace("[ ]", "   ")
}

/// Draws with a fresh pointer map, as a frame does.
fn paint(page: &mut IntegrationsPage, h: &H, w: u16, rows: u16) -> Buffer {
    pointer_map::begin_frame();
    draw(page, h, w, rows, None)
}

/// Whatever is clickable: a button, checkbox or radio is a hand, a text
/// field the I-beam, in every view of the page.
#[test]
fn every_button_is_real_with_the_hand_pointer_and_no_bracket() {
    let mut h = H::new();
    h.connected(IntegrationKind::GitlabSelfManaged, "kai");
    let kinds = IntegrationKind::IMPLEMENTED.len();
    let mut buttons = 0;
    for kind in 0..kinds {
        for state in 0..3 {
            let mut page = IntegrationsPage::default();
            page.select_kind(kind, &h.config);
            match state {
                1 => page.add_profile(&h.config),
                2 => page.zone = Zone::Body,
                _ => {}
            }
            let buf = paint(&mut page, &h, 100, 70);
            let screen = text(&buf);
            let bare = without_marks(&screen);
            assert!(
                !bare.contains('[') && !bare.contains(']'),
                "a bracketed label (kind {kind}, view {state}):\n{screen}"
            );
            for (rect, hit, _) in &page.hits {
                let want = match hit {
                    Hit::Item(item) if item.is_text() => Kind::Field,
                    Hit::Item(_) | Hit::AddProfile => Kind::Hand,
                    Hit::Type(_) | Hit::Profile(_) => continue,
                };
                assert_eq!(
                    pointer_map::at(rect.x, rect.y),
                    Some(want),
                    "{hit:?} at {rect:?} (kind {kind}, view {state}):\n{screen}"
                );
                buttons += 1;
            }
        }
    }
    assert!(buttons > 20, "the sweep saw only {buttons} controls");
}

/// A button is padded on the button fill: ` Connect `, not `[ Connect ]`.
#[test]
fn a_button_is_drawn_on_the_button_fill() {
    let h = H::new();
    let mut page = IntegrationsPage::default();
    let buf = paint(&mut page, &h, 70, 26);
    let (rect, _, _) = *page
        .hits
        .iter()
        .find(|(_, hit, _)| *hit == Hit::Item(Item::StartConnect))
        .expect("the Connect button");
    let rest = buf[(rect.x, rect.y)].bg;
    assert_ne!(rest, th().ui.panel, "a flat label, not a button");
    assert_eq!(
        text(&buf).lines().nth(rect.y as usize).unwrap()[..]
            .chars()
            .skip(rect.x as usize)
            .take(rect.width as usize)
            .collect::<String>(),
        " Connect "
    );
}

/// `Ctrl+N` badges the Radar block's checkboxes, its Namespaces area and the
/// Regular expressions checkbox, each at a cell that clicks it.
#[test]
fn the_radar_block_controls_are_badged_for_ctrl_n() {
    let mut h = H::new();
    h.connected(IntegrationKind::GitlabSelfManaged, "kai");
    let mut page = IntegrationsPage::default();
    page.select_kind(
        page.kind_index(IntegrationKind::GitlabSelfManaged).unwrap(),
        &h.config,
    );
    page.zone = Zone::Body;
    paint(&mut page, &h, 100, 70);
    let badges = page.nav_hits();
    for item in [
        Item::RadarReview,
        Item::RadarNamespaces,
        Item::RadarRegex,
        Item::RadarMode,
    ] {
        let (rect, _, _) = *page
            .hits
            .iter()
            .find(|(_, hit, _)| *hit == Hit::Item(item))
            .unwrap_or_else(|| panic!("{item:?} not painted"));
        assert!(
            badges
                .iter()
                .any(|(badge, at)| badge.y == rect.y && rect.contains((at.0, at.1).into())),
            "{item:?} has no badge: {badges:?}"
        );
    }
    // Pressing the Regular expressions badge toggles it.
    let (rect, _, _) = *page
        .hits
        .iter()
        .find(|(_, hit, _)| *hit == Hit::Item(Item::RadarRegex))
        .unwrap();
    let (_, at) = *badges.iter().find(|(b, _)| b.y == rect.y).unwrap();
    page.on_mouse(
        MouseEventKind::Down(MouseButton::Left),
        at.0,
        at.1,
        &mut h.env(),
    );
    assert!(h.config.integrations[0].radar_options().namespaces_regex);
}

fn namespaces_page(h: &mut H, lines: usize) -> IntegrationsPage {
    h.connected(IntegrationKind::GitlabSelfManaged, "kai");
    let mut page = IntegrationsPage::default();
    page.select_kind(
        page.kind_index(IntegrationKind::GitlabSelfManaged).unwrap(),
        &h.config,
    );
    page.zone = Zone::Body;
    page.sync_radar(&h.config);
    page.radar_ns = (1..=lines)
        .map(|n| format!("group-{n}"))
        .collect::<Vec<_>>()
        .join("\n");
    page
}

/// The Namespaces scrollbar is pressable and draggable: an end arrow steps a
/// line, the track jumps, the thumb drags, and it shows the hand.
#[test]
fn the_namespaces_scrollbar_can_be_pressed_and_dragged() {
    let mut h = H::new();
    let mut page = namespaces_page(&mut h, 20);
    paint(&mut page, &h, 100, 70);
    let bar = page.ns_bar.expect("20 lines overflow the 4-row area");
    let top = |p: &IntegrationsPage| {
        p.editor
            .caret(Item::RadarNamespaces)
            .map_or(0, |c| c.top.get())
    };
    let screen_y = |p: &IntegrationsPage, pos: u16| bar.rect.y - p.scroll.top as u16 + pos;
    assert_eq!(top(&page), 0);
    // The hand shows over the bar, not the I-beam of the area.
    assert_eq!(
        pointer_map::at(bar.rect.x, screen_y(&page, 1)),
        Some(Kind::Hand)
    );
    // The end arrow steps one line down.
    let down = bar.track_len - 1;
    page.on_mouse(
        MouseEventKind::Down(MouseButton::Left),
        bar.rect.x,
        screen_y(&page, down),
        &mut h.env(),
    );
    page.on_mouse(
        MouseEventKind::Up(MouseButton::Left),
        bar.rect.x,
        screen_y(&page, down),
        &mut h.env(),
    );
    assert_eq!(top(&page), 1, "the arrow steps a line");
    // Drag the thumb to the bottom: the view reaches the last lines.
    paint(&mut page, &h, 100, 70);
    let bar = page.ns_bar.unwrap();
    let grab_row = screen_y(&page, bar.thumb_start);
    page.on_mouse(
        MouseEventKind::Down(MouseButton::Left),
        bar.rect.x,
        grab_row,
        &mut h.env(),
    );
    assert!(page.owns_pointer(), "the held thumb owns the pointer");
    page.on_mouse(
        MouseEventKind::Drag(MouseButton::Left),
        bar.rect.x,
        screen_y(&page, bar.track_len - 1),
        &mut h.env(),
    );
    page.on_mouse(
        MouseEventKind::Up(MouseButton::Left),
        bar.rect.x,
        screen_y(&page, bar.track_len - 1),
        &mut h.env(),
    );
    assert!(!page.owns_pointer());
    assert_eq!(top(&page), 20 - 4, "the thumb reached the end");
    // A press on the bare track jumps there from the other end.
    paint(&mut page, &h, 100, 70);
    let bar = page.ns_bar.unwrap();
    page.on_mouse(
        MouseEventKind::Down(MouseButton::Left),
        bar.rect.x,
        screen_y(&page, bar.arrow),
        &mut h.env(),
    );
    page.on_mouse(
        MouseEventKind::Up(MouseButton::Left),
        bar.rect.x,
        screen_y(&page, bar.arrow),
        &mut h.env(),
    );
    assert!(top(&page) < 20 - 4, "the track press moved the view up");
}
