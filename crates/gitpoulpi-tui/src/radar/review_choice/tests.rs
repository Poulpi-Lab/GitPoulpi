use super::*;
use ratatui::buffer::Buffer;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn press(c: char) -> KeyEvent {
    key(KeyCode::Char(c))
}

fn click(col: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: col,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

const BODY: Rect = Rect {
    x: 0,
    y: 1,
    width: 100,
    height: 30,
};

fn dialog(cloned: bool) -> (ReviewChoice, Buffer) {
    let mut d = ReviewChoice::new(
        "!58 Round invoice totals".into(),
        "northwind/billing".into(),
        "GitLab".into(),
        cloned,
    );
    let mut buf = Buffer::empty(Rect::new(0, 0, 100, 32));
    d.render(&mut buf, BODY);
    (d, buf)
}

fn text(buf: &Buffer) -> String {
    crate::dialog_kit::buffer_text(buf)
}

fn rect_of(d: &ReviewChoice, slot: Slot) -> Rect {
    d.hits
        .iter()
        .find(|(_, s)| *s == slot)
        .map(|(r, _)| *r)
        .expect("a rect for the control")
}

#[test]
fn a_request_without_a_clone_offers_clone_and_api_with_cancel_and_remember() {
    let (_, buf) = dialog(false);
    let t = text(&buf);
    for want in [
        "Review request",
        "!58 Round invoice totals",
        "Repository northwind/billing",
        "c  Clone it, then review",
        "a  Review through the API",
        "Cancel",
        "[ ] Remember for GitLab",
    ] {
        assert!(t.contains(want), "{want}:\n{t}");
    }
    assert!(!t.contains("Open the local clone"), "{t}");
}

#[test]
fn a_known_clone_offers_the_local_one_instead_of_cloning() {
    let (_, buf) = dialog(true);
    let t = text(&buf);
    assert!(t.contains("l  Open the local clone"), "{t}");
    assert!(t.contains("a  Review through the API"), "{t}");
    assert!(!t.contains("Clone it, then review"), "{t}");
}

#[test]
fn the_letters_pick_and_only_the_offered_ones() {
    let (mut d, _) = dialog(false);
    assert_eq!(
        d.on_key(press('l')),
        ChoiceOutcome::Stay,
        "no clone to open"
    );
    assert_eq!(
        d.on_key(press('c')),
        ChoiceOutcome::Pick {
            pick: Pick::Clone,
            remember: false
        }
    );
    assert_eq!(
        d.on_key(press('a')),
        ChoiceOutcome::Pick {
            pick: Pick::Api,
            remember: false
        }
    );
    let (mut d, _) = dialog(true);
    assert_eq!(
        d.on_key(press('l')),
        ChoiceOutcome::Pick {
            pick: Pick::Local,
            remember: false
        }
    );
    assert_eq!(d.on_key(press('c')), ChoiceOutcome::Stay);
}

#[test]
fn enter_picks_the_highlighted_row_and_tab_walks_to_cancel_and_remember() {
    let (mut d, _) = dialog(false);
    d.on_key(key(KeyCode::Down));
    assert_eq!(
        d.on_key(key(KeyCode::Enter)),
        ChoiceOutcome::Pick {
            pick: Pick::Api,
            remember: false
        }
    );
    let (mut d, _) = dialog(false);
    d.on_key(key(KeyCode::Tab));
    d.on_key(key(KeyCode::Tab));
    assert_eq!(d.on_key(key(KeyCode::Enter)), ChoiceOutcome::Cancel);
    d.on_key(key(KeyCode::Tab));
    assert_eq!(d.focus, Slot::Remember);
    assert_eq!(d.on_key(key(KeyCode::Enter)), ChoiceOutcome::Stay);
    assert!(d.remember, "Enter on the checkbox toggles it");
}

#[test]
fn remember_travels_with_the_pick() {
    let (mut d, _) = dialog(false);
    assert_eq!(d.on_key(press('r')), ChoiceOutcome::Stay);
    assert_eq!(
        d.on_key(press('a')),
        ChoiceOutcome::Pick {
            pick: Pick::Api,
            remember: true
        }
    );
}

#[test]
fn escape_cancels() {
    let (mut d, _) = dialog(false);
    assert_eq!(d.on_key(key(KeyCode::Esc)), ChoiceOutcome::Cancel);
    // A Ctrl chord is not a letter pick.
    let ctrl = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    assert_eq!(d.on_key(ctrl), ChoiceOutcome::Stay);
}

#[test]
fn the_mouse_picks_toggles_and_closes() {
    let (mut d, _) = dialog(false);
    let api = rect_of(&d, Slot::Row(1));
    assert_eq!(
        d.on_mouse(click(api.x + 3, api.y)),
        ChoiceOutcome::Pick {
            pick: Pick::Api,
            remember: false
        }
    );
    let check = rect_of(&d, Slot::Remember);
    assert_eq!(d.on_mouse(click(check.x + 1, check.y)), ChoiceOutcome::Stay);
    assert!(d.remember);
    let cancel = rect_of(&d, Slot::Cancel);
    assert_eq!(
        d.on_mouse(click(cancel.x + 1, cancel.y)),
        ChoiceOutcome::Cancel
    );
    // The cross of the title.
    let cross = d.close.rect().expect("a close cross");
    assert_eq!(d.on_mouse(click(cross.x, cross.y)), ChoiceOutcome::Cancel);
    // Away from the dialog closes it; inside it, on nothing, does not.
    assert_eq!(d.on_mouse(click(0, 31)), ChoiceOutcome::Cancel);
    assert_eq!(
        d.on_mouse(click(d.frame.x + 2, d.frame.y + 2)),
        ChoiceOutcome::Stay
    );
}

#[test]
fn every_control_has_a_badge_that_acts_like_its_click() {
    let (mut d, _) = dialog(false);
    let targets = d.nav_targets();
    assert_eq!(targets.len(), 4, "two rows, Cancel and the checkbox");
    assert_eq!(
        d.nav_press(1),
        ChoiceOutcome::Pick {
            pick: Pick::Api,
            remember: false
        }
    );
    assert_eq!(d.nav_press(2), ChoiceOutcome::Cancel);
    assert_eq!(d.nav_press(3), ChoiceOutcome::Stay);
    assert!(d.remember);
}
