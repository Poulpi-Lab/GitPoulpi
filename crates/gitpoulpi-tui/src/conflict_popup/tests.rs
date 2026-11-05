use super::*;
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

/// The popup is process-wide: tests that open it take turns.
static SERIAL: Mutex<()> = Mutex::new(());

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn click(x: u16, y: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: x,
        row: y,
        modifiers: KeyModifiers::NONE,
    }
}

fn files(n: usize) -> Vec<String> {
    (1..=n).map(|i| format!("src/file{i:02}.rs")).collect()
}

fn screen() -> String {
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    terminal.draw(render).unwrap();
    let buf = terminal.backend().buffer();
    let mut out = String::new();
    for y in 0..buf.area.height {
        for x in 0..buf.area.width {
            out.push_str(buf[(x, y)].symbol());
        }
        out.push('\n');
    }
    out
}

fn rect_of(button: Button) -> Rect {
    let slot = POPUP.lock().unwrap();
    slot.as_ref()
        .unwrap()
        .buttons
        .iter()
        .find(|(_, b)| *b == button)
        .map(|(r, _)| *r)
        .expect("button painted")
}

#[test]
fn it_lists_the_files_and_the_three_buttons() {
    let _turn = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    open("origin/main", &files(3));
    let screen = screen();
    assert!(screen.contains("Predicted conflicts"), "{screen}");
    assert!(screen.contains("origin/main"), "{screen}");
    assert!(screen.contains("src/file01.rs") && screen.contains("src/file03.rs"));
    assert!(screen.contains("Open merge preview"), "{screen}");
    assert!(screen.contains("Rebase onto base"), "{screen}");
    assert!(screen.contains("Close"), "{screen}");
    close();
}

#[test]
fn keys_move_between_buttons_and_enter_presses_one() {
    let _turn = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    open("origin/main", &files(2));
    screen();
    assert!(is_open());
    // Merge is focused first; Tab goes to Rebase.
    assert_eq!(on_key(key(KeyCode::Tab)), None);
    assert_eq!(
        on_key(key(KeyCode::Enter)),
        Some(PopupAction::Rebase("origin/main".into()))
    );
    assert!(!is_open(), "a pressed button closes the popup");

    open("origin/main", &files(2));
    screen();
    assert_eq!(
        on_key(key(KeyCode::Enter)),
        Some(PopupAction::Merge("origin/main".into()))
    );

    // Close does nothing; so does Esc.
    open("origin/main", &files(2));
    screen();
    on_key(key(KeyCode::Left));
    assert_eq!(on_key(key(KeyCode::Enter)), None);
    assert!(!is_open());
    open("origin/main", &files(2));
    assert_eq!(on_key(key(KeyCode::Esc)), None);
    assert!(!is_open());
}

#[test]
fn a_click_presses_a_button_and_a_click_away_closes() {
    let _turn = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    open("origin/main", &files(2));
    screen();
    let rebase = rect_of(Button::Rebase);
    assert_eq!(
        on_mouse(click(rebase.x + 1, rebase.y)),
        Some(PopupAction::Rebase("origin/main".into()))
    );
    assert!(!is_open());

    open("origin/main", &files(2));
    screen();
    // Inside the box but on no button: nothing happens.
    let outer = POPUP.lock().unwrap().as_ref().unwrap().outer;
    assert_eq!(on_mouse(click(outer.x + 2, outer.y + 1)), None);
    assert!(is_open());
    // Away from it: closed, and consumed.
    assert_eq!(on_mouse(click(0, 0)), None);
    assert!(!is_open());
}

#[test]
fn a_long_list_scrolls_and_says_more_follow() {
    let _turn = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    open("origin/main", &files(25));
    let first = screen();
    assert!(first.contains("src/file01.rs") && !first.contains("src/file25.rs"));
    assert!(first.contains("src/file10.rs  ..."), "{first}");
    on_key(key(KeyCode::End));
    let last = screen();
    assert!(
        last.contains("src/file25.rs") && !last.contains("src/file01.rs"),
        "{last}"
    );
    assert!(
        !last.contains("  ..."),
        "the end of the list: nothing follows"
    );
    on_key(key(KeyCode::Home));
    assert!(screen().contains("src/file01.rs"));
    close();
}

#[test]
fn a_long_list_has_a_bar_the_thumb_drags_and_a_short_one_has_none() {
    let _turn = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    open("origin/main", &files(40));
    let _ = screen();
    let track = POPUP.lock().unwrap().as_ref().unwrap().bar.expect("a bar");
    let ev = |kind, row| MouseEvent {
        kind,
        column: track.rect.x,
        row,
        modifiers: KeyModifiers::NONE,
    };
    on_mouse(ev(
        MouseEventKind::Down(MouseButton::Left),
        track.rect.y + 1,
    ));
    on_mouse(ev(
        MouseEventKind::Drag(MouseButton::Left),
        track.rect.bottom() + 9,
    ));
    on_mouse(ev(MouseEventKind::Up(MouseButton::Left), 0));
    let shown = screen();
    assert!(shown.contains("file40"), "dragged to the end\n{shown}");
    assert!(is_open(), "a press on the bar does not close the popup");
    open("origin/main", &files(3));
    let _ = screen();
    assert!(POPUP.lock().unwrap().as_ref().unwrap().bar.is_none());
    close();
}

#[test]
fn hover_lights_a_button_only_while_the_pointer_is_over_it() {
    let _turn = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    open("origin/main", &files(1));
    screen();
    let close_btn = rect_of(Button::Close);
    let moved = |x, y| MouseEvent {
        kind: MouseEventKind::Moved,
        column: x,
        row: y,
        modifiers: KeyModifiers::NONE,
    };
    on_mouse(moved(close_btn.x, close_btn.y));
    assert_eq!(
        POPUP.lock().unwrap().as_ref().unwrap().hovered,
        Some(Button::Close)
    );
    on_mouse(moved(0, 0));
    assert_eq!(POPUP.lock().unwrap().as_ref().unwrap().hovered, None);
    close();
}

#[test]
fn the_cross_closes_and_no_esc_hint_is_shown() {
    let _turn = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    open("origin/main", &files(2));
    let text = screen();
    assert!(text.contains('\u{2715}'), "{text}");
    assert!(!text.contains("Esc"), "{text}");
    let cross = POPUP
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .close
        .rect()
        .unwrap();
    assert_eq!(on_mouse(click(cross.x + 1, cross.y)), None);
    assert!(!is_open());
}
