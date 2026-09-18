use super::super::list_editor::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

const W: u16 = 34;
const H: u16 = 16;

fn spec(n: usize) -> ListSpec {
    ListSpec {
        title: "Profiles".into(),
        items: (0..n)
            .map(|i| ListItem {
                label: format!("Item {i}"),
                sub: Some(format!("sub{i}")),
                ..ListItem::default()
            })
            .collect(),
        add: AddKind::Plain,
        duplicate: true,
        reorder: true,
        min_len: 0,
    }
}

fn editor(n: usize) -> ListEditor {
    let mut e = ListEditor::new(spec(n));
    paint(&mut e);
    e
}

fn paint(e: &mut ListEditor) -> Buffer {
    let area = Rect::new(0, 0, W, H);
    let mut buf = Buffer::empty(area);
    e.render(&mut buf, area, true, area);
    buf
}

fn text(buf: &Buffer) -> String {
    (0..buf.area.height)
        .map(|y| {
            (0..buf.area.width)
                .map(|x| buf[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn row_text(buf: &Buffer, r: Rect) -> String {
    (r.x..r.right()).map(|x| buf[(x, r.y)].symbol()).collect()
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn press(e: &mut ListEditor, code: KeyCode, mods: KeyModifiers) -> ListEvent {
    e.on_key(KeyEvent::new(code, mods))
}

fn op(o: ListOp) -> ListEvent {
    ListEvent::Op(o)
}

fn mouse(kind: MouseEventKind, x: u16, y: u16) -> MouseEvent {
    MouseEvent {
        kind,
        column: x,
        row: y,
        modifiers: KeyModifiers::NONE,
    }
}

fn rect_of(e: &ListEditor, hit: ListHit) -> Rect {
    e.hits()
        .iter()
        .find(|(_, h)| *h == hit)
        .unwrap_or_else(|| panic!("no hit rect for {hit:?}"))
        .0
}

fn click(e: &mut ListEditor, hit: ListHit) -> ListEvent {
    let r = rect_of(e, hit);
    let down = e.on_mouse(mouse(MouseEventKind::Down(MouseButton::Left), r.x, r.y));
    e.on_mouse(mouse(MouseEventKind::Up(MouseButton::Left), r.x, r.y));
    down
}

#[test]
fn navigation_keys_select_and_report_only_changes() {
    let mut e = editor(5);
    assert_eq!(e.on_key(key(KeyCode::Down)), op(ListOp::Select(1)));
    assert_eq!(e.on_key(key(KeyCode::End)), op(ListOp::Select(4)));
    assert_eq!(
        e.on_key(key(KeyCode::Down)),
        ListEvent::Handled,
        "already last"
    );
    assert_eq!(e.on_key(key(KeyCode::Up)), op(ListOp::Select(3)));
    assert_eq!(e.on_key(key(KeyCode::Home)), op(ListOp::Select(0)));
    assert_eq!(e.on_key(key(KeyCode::PageDown)), op(ListOp::Select(4)));
    assert_eq!(e.on_key(key(KeyCode::PageUp)), op(ListOp::Select(0)));
    assert_eq!(e.selected, 0);
}

#[test]
fn enter_and_right_go_to_the_detail_left_goes_back() {
    let mut e = editor(3);
    assert_eq!(e.on_key(key(KeyCode::Enter)), op(ListOp::FocusDetail));
    assert_eq!(e.on_key(key(KeyCode::Right)), op(ListOp::FocusDetail));
    assert_eq!(e.on_key(key(KeyCode::Left)), op(ListOp::Back));
    assert_eq!(
        e.on_key(key(KeyCode::Esc)),
        ListEvent::Ignored,
        "Esc is the page's"
    );
    let mut empty = editor(0);
    assert_eq!(empty.on_key(key(KeyCode::Enter)), ListEvent::Ignored);
}

#[test]
fn add_duplicate_and_move_keys_emit_their_ops() {
    let mut e = editor(4);
    e.select(2);
    assert_eq!(
        press(&mut e, KeyCode::Insert, KeyModifiers::NONE),
        op(ListOp::Add(None))
    );
    // `Ctrl+N` opens the navigation badges everywhere: it no longer adds.
    assert_ne!(
        press(&mut e, KeyCode::Char('n'), KeyModifiers::CONTROL),
        op(ListOp::Add(None)),
        "Ctrl+N is the badges' prefix, not Add"
    );
    e.select(2);
    assert_eq!(
        press(&mut e, KeyCode::Char('d'), KeyModifiers::CONTROL),
        op(ListOp::Duplicate(2))
    );
    assert_eq!(
        press(&mut e, KeyCode::Up, KeyModifiers::ALT),
        op(ListOp::Move { from: 2, to: 1 })
    );
    assert_eq!(
        press(&mut e, KeyCode::Down, KeyModifiers::ALT),
        op(ListOp::Move { from: 2, to: 3 })
    );
}

#[test]
fn moving_past_either_end_and_unavailable_features_do_nothing() {
    let mut e = editor(3);
    assert!(!e.can_move_up());
    assert_eq!(
        press(&mut e, KeyCode::Up, KeyModifiers::ALT),
        ListEvent::Handled
    );
    e.select(2);
    assert_eq!(
        press(&mut e, KeyCode::Down, KeyModifiers::ALT),
        ListEvent::Handled
    );

    let mut plain = spec(3);
    plain.duplicate = false;
    plain.reorder = false;
    let mut e = ListEditor::new(plain);
    paint(&mut e);
    assert_eq!(
        press(&mut e, KeyCode::Char('d'), KeyModifiers::CONTROL),
        ListEvent::Handled
    );
    assert_eq!(
        press(&mut e, KeyCode::Down, KeyModifiers::ALT),
        ListEvent::Handled
    );
    let screen = text(&paint(&mut e));
    assert!(!screen.contains("Duplicate"), "{screen}");
    assert!(!screen.contains('\u{2191}'), "{screen}");
}

#[test]
fn the_up_button_is_drawn_disabled_on_the_first_row_not_hidden() {
    let mut e = editor(3);
    let buf = paint(&mut e);
    let up = rect_of(&e, ListHit::Up);
    assert_eq!(row_text(&buf, up).trim(), "\u{2191}");
    let disabled = crate::th().chrome.disabled_button_bg;
    assert_eq!(buf[(up.x, up.y)].bg, disabled);
    // Its click does nothing.
    assert_eq!(click(&mut e, ListHit::Up), ListEvent::Handled);
    // Down is enabled.
    let down = rect_of(&e, ListHit::Down);
    assert_ne!(buf[(down.x, down.y)].bg, disabled);
    e.select(1);
    assert_eq!(
        click(&mut e, ListHit::Up),
        op(ListOp::Move { from: 1, to: 0 })
    );
}

#[test]
fn delete_asks_first_cancel_keeps_and_remove_confirms() {
    let mut e = editor(3);
    e.select(1);
    assert_eq!(e.on_key(key(KeyCode::Delete)), ListEvent::Handled);
    assert_eq!(e.confirm_remove, Some(1));
    assert!(e.owns_pointer());
    let screen = text(&paint(&mut e));
    assert!(screen.contains("Remove \"Item 1\"?"), "{screen}");
    // Cancel has the focus: Enter keeps the item.
    assert_eq!(e.on_key(key(KeyCode::Enter)), ListEvent::Handled);
    assert_eq!(e.confirm_remove, None);
    // Ask again, move to Remove, confirm.
    e.on_key(key(KeyCode::Delete));
    e.on_key(key(KeyCode::Right));
    assert_eq!(e.on_key(key(KeyCode::Enter)), op(ListOp::Remove(1)));
    assert_eq!(e.confirm_remove, None);
    // Esc closes it, and nothing else gets the key meanwhile.
    e.on_key(key(KeyCode::Delete));
    assert_eq!(e.on_key(key(KeyCode::Down)), ListEvent::Handled);
    assert_eq!(e.selected, 1);
    assert_eq!(e.on_key(key(KeyCode::Esc)), ListEvent::Handled);
    assert_eq!(e.confirm_remove, None);
}

#[test]
fn a_pristine_item_is_removed_without_asking() {
    let mut s = spec(3);
    s.items[2].pristine = true;
    let mut e = ListEditor::new(s);
    paint(&mut e);
    e.select(2);
    assert_eq!(e.on_key(key(KeyCode::Delete)), op(ListOp::Remove(2)));
    assert_eq!(e.confirm_remove, None);
}

#[test]
fn remove_is_disabled_at_min_len() {
    let mut s = spec(1);
    s.min_len = 1;
    let mut e = ListEditor::new(s);
    paint(&mut e);
    assert!(!e.can_remove());
    assert_eq!(e.on_key(key(KeyCode::Delete)), ListEvent::Handled);
    assert_eq!(e.confirm_remove, None);
    assert_eq!(click(&mut e, ListHit::Remove), ListEvent::Handled);
    assert_eq!(e.confirm_remove, None);
}

#[test]
fn the_remove_confirmation_owns_the_pointer() {
    let mut e = editor(3);
    e.select(1);
    click(&mut e, ListHit::Remove);
    assert_eq!(e.confirm_remove, Some(1));
    paint(&mut e);
    // A click on a row behind the panel selects nothing and closes it.
    let row0 = rect_of(&e, ListHit::Row(0));
    e.on_mouse(mouse(
        MouseEventKind::Down(MouseButton::Left),
        row0.x + 2,
        row0.y,
    ));
    assert_eq!(e.selected, 1);
    assert_eq!(
        e.confirm_remove, None,
        "a click outside the panel closes it"
    );
    // Clicking the confirm button removes.
    click(&mut e, ListHit::Remove);
    paint(&mut e);
    let buttons = e.popup_hits();
    assert_eq!(buttons.len(), 2);
    let confirm = buttons[1];
    let down = e.on_mouse(mouse(
        MouseEventKind::Down(MouseButton::Left),
        confirm.x,
        confirm.y,
    ));
    assert_eq!(down, op(ListOp::Remove(1)));
}

#[test]
fn add_menu_opens_a_dropdown_and_reports_the_pick() {
    let mut s = spec(2);
    s.add = AddKind::Menu(vec!["OpenAI".into(), "Anthropic".into(), "Local".into()]);
    let mut e = ListEditor::new(s);
    let buf = paint(&mut e);
    assert!(text(&buf).contains("+ Add \u{25BE}"), "{}", text(&buf));
    assert_eq!(
        press(&mut e, KeyCode::Insert, KeyModifiers::NONE),
        ListEvent::Handled
    );
    assert!(e.owns_pointer());
    let screen = text(&paint(&mut e));
    assert!(screen.contains("Anthropic"), "{screen}");
    assert!(!e.popup_hits().is_empty());
    e.on_key(key(KeyCode::Down));
    assert_eq!(e.on_key(key(KeyCode::Enter)), op(ListOp::Add(Some(1))));
    assert!(!e.owns_pointer());
    // Click an entry.
    click(&mut e, ListHit::Add);
    paint(&mut e);
    let third = e.popup_hits()[2];
    let ev = e.on_mouse(mouse(
        MouseEventKind::Down(MouseButton::Left),
        third.x,
        third.y,
    ));
    assert_eq!(ev, op(ListOp::Add(Some(2))));
    // Esc closes without a pick.
    press(&mut e, KeyCode::Insert, KeyModifiers::NONE);
    assert_eq!(e.on_key(key(KeyCode::Esc)), ListEvent::Handled);
    assert!(!e.owns_pointer());
}

#[test]
fn rows_marks_toolbar_and_scrollbar_are_painted_at_34x16() {
    let mut s = spec(30);
    s.items[0].mark = Mark::Default;
    s.items[1].mark = Mark::Active;
    s.items[2].mark = Mark::Problem("no token".into());
    let mut e = ListEditor::new(s);
    let buf = paint(&mut e);
    let screen = text(&buf);
    let lines: Vec<&str> = screen.lines().collect();
    assert!(lines[0].contains("Profiles"), "{screen}");
    assert!(screen.contains("\u{2605} Item 0"), "{screen}");
    assert!(screen.contains("\u{25CF} Item 1"), "{screen}");
    assert!(screen.contains("\u{26A0} Item 2"), "{screen}");
    assert!(
        screen.contains("no token"),
        "the reason replaces the sub text:\n{screen}"
    );
    assert!(!screen.contains("sub2"), "{screen}");
    assert!(screen.contains("sub0"), "{screen}");
    for label in ["+ Add", "Duplicate", "Remove", "\u{2191}", "\u{2193}"] {
        assert!(screen.contains(label), "{label}:\n{screen}");
    }
    // The toolbar wraps on 34 columns: not all on one line.
    assert!(
        lines[14].contains("+ Add") && lines[14].contains("Remove"),
        "{screen}"
    );
    assert!(
        lines[15].contains('\u{2191}') && lines[15].contains('\u{2193}'),
        "the arrows wrap together:\n{screen}"
    );
    // 30 items overflow the rows: the thumb replaces plain border glyphs.
    let right: String = (2..9)
        .map(|y| buf[(W - 1, y)].symbol().to_string())
        .collect();
    assert!(
        right.chars().any(|c| c != '\u{2502}'),
        "scrollbar missing: {right:?}"
    );
    let few = text(&paint(&mut editor(3)));
    let few_right: String = few
        .lines()
        .skip(1)
        .take(4)
        .map(|l| l.chars().last().unwrap())
        .collect();
    assert_eq!(
        few_right, "\u{2502}\u{2502}\u{2502}\u{2502}",
        "no bar when it fits"
    );
}

#[test]
fn every_hit_rect_is_what_was_painted() {
    let mut s = spec(4);
    s.add = AddKind::Menu(vec!["A".into()]);
    let mut e = ListEditor::new(s);
    let buf = paint(&mut e);
    for (rect, hit) in e.hits().to_vec() {
        let painted = row_text(&buf, rect);
        match hit {
            ListHit::Row(i) => assert!(painted.contains(&format!("Item {i}")), "{painted:?}"),
            ListHit::Add => assert_eq!(painted, " + Add \u{25BE} "),
            ListHit::Duplicate => assert_eq!(painted, " Duplicate "),
            ListHit::Remove => assert_eq!(painted, " Remove "),
            ListHit::Up => assert_eq!(painted, " \u{2191} "),
            ListHit::Down => assert_eq!(painted, " \u{2193} "),
        }
    }
    assert_eq!(e.hits().len(), 4 + 5);
}

#[test]
fn click_selects_and_a_double_click_focuses_the_detail() {
    let mut e = editor(5);
    assert_eq!(click(&mut e, ListHit::Row(3)), op(ListOp::Select(3)));
    assert_eq!(e.selected, 3);
    assert_eq!(click(&mut e, ListHit::Row(3)), op(ListOp::FocusDetail));
    // The pair is consumed: a third click is a plain click again.
    assert_eq!(click(&mut e, ListHit::Row(3)), ListEvent::Handled);
}

#[test]
fn toolbar_clicks_do_what_their_keys_do() {
    let mut e = editor(3);
    e.select(1);
    assert_eq!(click(&mut e, ListHit::Add), op(ListOp::Add(None)));
    assert_eq!(click(&mut e, ListHit::Duplicate), op(ListOp::Duplicate(1)));
    assert_eq!(
        click(&mut e, ListHit::Down),
        op(ListOp::Move { from: 1, to: 2 })
    );
    click(&mut e, ListHit::Remove);
    assert_eq!(e.confirm_remove, Some(1));
}

#[test]
fn the_wheel_scrolls_without_selecting() {
    let mut e = editor(30);
    let r = rect_of(&e, ListHit::Row(2));
    let ev = e.on_mouse(mouse(MouseEventKind::ScrollDown, r.x, r.y));
    assert_eq!(ev, ListEvent::Handled);
    assert_eq!(e.scroll, 3);
    assert_eq!(e.selected, 0);
    let screen = text(&paint(&mut e));
    assert!(
        screen.contains("Item 3") && !screen.contains("Item 1 "),
        "{screen}"
    );
    e.on_mouse(mouse(MouseEventKind::ScrollUp, r.x, r.y));
    assert_eq!(e.scroll, 0);
    // Outside the editor the wheel is not ours.
    assert_eq!(
        e.on_mouse(mouse(MouseEventKind::ScrollDown, W + 5, 3)),
        ListEvent::Ignored
    );
}

#[test]
fn keyboard_selection_scrolls_the_row_into_view() {
    let mut e = editor(30);
    e.on_key(key(KeyCode::End));
    assert_eq!(e.selected, 29);
    let screen = text(&paint(&mut e));
    assert!(screen.contains("Item 29"), "{screen}");
}

fn drag_to(e: &mut ListEditor, hit: ListHit) -> ListEvent {
    let r = rect_of(e, hit);
    e.on_mouse(mouse(MouseEventKind::Drag(MouseButton::Left), r.x + 3, r.y))
}

#[test]
fn dragging_a_row_reorders_on_release_and_shows_an_insertion_bar() {
    let mut e = editor(5);
    let from = rect_of(&e, ListHit::Row(1));
    e.on_mouse(mouse(
        MouseEventKind::Down(MouseButton::Left),
        from.x + 3,
        from.y,
    ));
    assert_eq!(
        e.drag,
        Some(Drag {
            from: 1,
            over: 1,
            started: false
        })
    );
    // No move yet: nothing painted.
    assert_eq!(drag_to(&mut e, ListHit::Row(1)), ListEvent::Handled);
    assert!(!e.drag.unwrap().started);
    drag_to(&mut e, ListHit::Row(3));
    assert_eq!(
        e.drag,
        Some(Drag {
            from: 1,
            over: 3,
            started: true
        })
    );
    let screen = text(&paint(&mut e));
    assert!(
        screen.contains('\u{2581}'),
        "insertion bar below the target row:\n{screen}"
    );
    let r = rect_of(&e, ListHit::Row(3));
    let up = e.on_mouse(mouse(MouseEventKind::Up(MouseButton::Left), r.x + 3, r.y));
    assert_eq!(up, op(ListOp::Move { from: 1, to: 3 }));
    assert_eq!(e.drag, None);
}

#[test]
fn dragging_up_draws_the_bar_above_and_releasing_in_place_moves_nothing() {
    let mut e = editor(5);
    let from = rect_of(&e, ListHit::Row(3));
    e.on_mouse(mouse(
        MouseEventKind::Down(MouseButton::Left),
        from.x + 3,
        from.y,
    ));
    drag_to(&mut e, ListHit::Row(0));
    assert!(text(&paint(&mut e)).contains('\u{2594}'));
    drag_to(&mut e, ListHit::Row(3));
    let r = rect_of(&e, ListHit::Row(3));
    let up = e.on_mouse(mouse(MouseEventKind::Up(MouseButton::Left), r.x, r.y));
    assert_eq!(up, ListEvent::Handled, "back on its own row: no move");
}

#[test]
fn esc_cancels_a_drag() {
    let mut e = editor(5);
    let from = rect_of(&e, ListHit::Row(0));
    e.on_mouse(mouse(
        MouseEventKind::Down(MouseButton::Left),
        from.x + 3,
        from.y,
    ));
    drag_to(&mut e, ListHit::Row(2));
    assert!(e.is_dragging());
    assert_eq!(e.on_key(key(KeyCode::Esc)), ListEvent::Handled);
    assert!(!e.is_dragging());
    let r = rect_of(&e, ListHit::Row(2));
    let up = e.on_mouse(mouse(MouseEventKind::Up(MouseButton::Left), r.x, r.y));
    assert_ne!(
        up,
        op(ListOp::Move { from: 0, to: 2 }),
        "the release must not move it"
    );
    assert!(!text(&paint(&mut e)).contains('\u{2581}'));
}

#[test]
fn no_drag_without_reorder() {
    let mut s = spec(4);
    s.reorder = false;
    let mut e = ListEditor::new(s);
    paint(&mut e);
    let from = rect_of(&e, ListHit::Row(0));
    e.on_mouse(mouse(
        MouseEventKind::Down(MouseButton::Left),
        from.x + 3,
        from.y,
    ));
    assert_eq!(e.drag, None);
}

#[test]
fn the_scrollbar_thumb_drags_the_list() {
    let mut e = editor(30);
    // The thumb sits at the top of the right border, below the arrow.
    let (x, y) = (W - 1, 2);
    e.on_mouse(mouse(MouseEventKind::Down(MouseButton::Left), x, y));
    e.on_mouse(mouse(MouseEventKind::Drag(MouseButton::Left), x, 8));
    assert!(e.scroll > 0, "the drag scrolled");
    assert_eq!(e.selected, 0);
    e.on_mouse(mouse(MouseEventKind::Up(MouseButton::Left), x, 8));
}

#[test]
fn a_click_outside_is_not_ours_and_hover_tracks_buttons() {
    let mut e = editor(3);
    assert_eq!(
        e.on_mouse(mouse(MouseEventKind::Down(MouseButton::Left), W + 2, 2)),
        ListEvent::Ignored
    );
    let r = rect_of(&e, ListHit::Duplicate);
    e.on_mouse(mouse(MouseEventKind::Moved, r.x, r.y));
    assert_eq!(e.hover, Some(ListHit::Duplicate));
}

#[test]
fn set_spec_keeps_the_selection_in_range() {
    let mut e = editor(5);
    e.select(4);
    e.set_spec(spec(2));
    assert_eq!(e.selected, 1);
    e.set_spec(spec(0));
    assert_eq!(e.selected, 0);
    assert_eq!(e.on_key(key(KeyCode::Delete)), ListEvent::Handled);
}

fn grouped() -> ListEditor {
    let item = |label: &str, group: &str, fixed: bool| ListItem {
        label: label.into(),
        group: Some(group.into()),
        fixed,
        ..ListItem::default()
    };
    let mut e = ListEditor::new(ListSpec {
        title: "Workflow".into(),
        items: vec![
            item("General", "General", true),
            item("main", "Bases", false),
            item("develop", "Bases", false),
            item("feature", "Topics", false),
        ],
        add: AddKind::Plain,
        duplicate: false,
        reorder: true,
        min_len: 0,
    });
    paint(&mut e);
    e
}

#[test]
fn group_headers_are_painted_where_the_group_changes_and_are_not_rows() {
    let mut e = grouped();
    let buf = paint(&mut e);
    let screen = text(&buf);
    for header in ["General", "Bases", "Topics"] {
        assert!(screen.contains(header), "{header}:\n{screen}");
    }
    // Four items, three headers: only the items have hit rects.
    let rows = e
        .hits()
        .iter()
        .filter(|(_, h)| matches!(h, ListHit::Row(_)))
        .count();
    assert_eq!(rows, 4);
}

#[test]
fn moves_never_cross_a_group_and_a_fixed_item_cannot_be_removed() {
    let mut e = grouped();
    e.select(0);
    assert!(!e.can_remove(), "fixed");
    e.select(1);
    assert!(!e.can_move_up(), "General is another group");
    assert!(e.can_move_down());
    e.select(2);
    assert!(!e.can_move_down(), "Topics is another group");
    assert!(e.can_remove());
}

#[test]
fn a_click_on_a_row_under_a_header_selects_that_item() {
    let mut e = grouped();
    paint(&mut e);
    let (rect, _) = *e
        .hits()
        .iter()
        .find(|(_, h)| *h == ListHit::Row(3))
        .unwrap();
    let ev = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: rect.x + 2,
        row: rect.y,
        modifiers: KeyModifiers::NONE,
    };
    assert_eq!(e.on_mouse(ev), ListEvent::Op(ListOp::Select(3)));
}
