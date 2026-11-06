use super::*;
use crossterm::event::KeyModifiers;
use gitpoulpi_core::privacy::Reason;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

/// The dialog is process-wide: tests that open it take turns.
static SERIAL: Mutex<()> = Mutex::new(());

fn reset() {
    *POPUP.lock().unwrap() = None;
    WAITING.lock().unwrap().clear();
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn screen_of(width: u16, height: u16) -> (String, Terminal<TestBackend>) {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(render).unwrap();
    (
        crate::dialog_kit::buffer_text(terminal.backend().buffer()),
        terminal,
    )
}

fn screen() -> String {
    screen_of(100, 30).0
}

fn mouse(kind: MouseEventKind, x: u16, y: u16) -> MouseEvent {
    MouseEvent {
        kind,
        column: x,
        row: y,
        modifiers: KeyModifiers::NONE,
    }
}

fn click(x: u16, y: u16) -> MouseEvent {
    mouse(MouseEventKind::Down(MouseButton::Left), x, y)
}

fn files(n: usize) -> Vec<Sensitive> {
    let mut out = vec![
        Sensitive {
            path: ".env".into(),
            reason: Reason::Pattern(".env".into()),
        },
        Sensitive {
            path: "notes.txt".into(),
            reason: Reason::Secret {
                rule: "github-token".into(),
                preview: "ghp_****".into(),
            },
        },
    ];
    for i in 0..n {
        out.push(Sensitive {
            path: format!("keys/k{i}.pem"),
            reason: Reason::Pattern("*.pem".into()),
        });
    }
    out
}

fn open(n: usize) {
    reset();
    ask(PathBuf::from("/repo"), Action::StageAll, files(n));
}

/// Where the answer rows sit, read from the paint like a click would.
fn answer(i: usize) -> Rect {
    POPUP.lock().unwrap().as_ref().unwrap().answers[i]
}

#[test]
fn the_dialog_lists_every_file_with_its_reason_and_no_value() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    open(0);
    assert!(is_open());
    let text = screen();
    assert!(text.contains("Sensitive files"), "{text}");
    assert!(text.contains(".env"), "{text}");
    assert!(text.contains("matches .env"), "{text}");
    assert!(text.contains("notes.txt"), "{text}");
    assert!(text.contains("contains github-token (ghp_****)"), "{text}");
    assert!(text.contains("1. Stage anyway"), "{text}");
    assert!(text.contains("2. Cancel, stage nothing"), "{text}");
    reset();
}

#[test]
fn no_answer_is_chosen_until_an_arrow_or_a_digit() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    open(0);
    screen();
    // Enter with nothing highlighted answers nothing and keeps it open.
    assert!(on_key(key(KeyCode::Enter)).is_none());
    assert!(is_open());
    assert!(on_key(key(KeyCode::Down)).is_none());
    let text = screen();
    assert!(text.contains("> 1. Stage anyway"), "{text}");
    let staged = on_key(key(KeyCode::Enter)).expect("the first row is Stage anyway");
    assert_eq!(staged.action, Action::StageAll);
    assert_eq!(staged.root, PathBuf::from("/repo"));
    assert!(!is_open());
}

#[test]
fn escape_two_and_the_second_row_cancel_and_one_stages() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    open(0);
    assert!(on_key(key(KeyCode::Esc)).is_none());
    assert!(!is_open(), "Esc cancels");

    open(0);
    assert!(on_key(key(KeyCode::Char('2'))).is_none());
    assert!(!is_open(), "2 cancels");

    open(0);
    on_key(key(KeyCode::Down));
    on_key(key(KeyCode::Down));
    assert!(on_key(key(KeyCode::Enter)).is_none());
    assert!(!is_open(), "the highlighted Cancel row cancels");

    open(0);
    assert!(on_key(key(KeyCode::Char('1'))).is_some(), "1 stages");
    assert!(!is_open());
}

#[test]
fn the_cross_an_answer_and_a_click_away_work_by_mouse() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    open(0);
    screen();
    // The cross is read from the border row the paint drew.
    let cross = POPUP
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .close
        .rect()
        .unwrap();
    assert!(on_mouse(click(cross.x, cross.y)).is_none());
    assert!(!is_open(), "the cross cancels");

    open(0);
    screen();
    let a = answer(0);
    assert!(on_mouse(click(a.x + 3, a.y)).is_some(), "row 1 stages");
    assert!(!is_open());

    open(0);
    screen();
    let b = answer(1);
    assert!(on_mouse(click(b.x + 3, b.y)).is_none());
    assert!(!is_open(), "row 2 cancels");

    open(0);
    screen();
    assert!(on_mouse(click(0, 0)).is_none());
    assert!(!is_open(), "a click away cancels");

    // A click inside the dialog changes nothing.
    open(0);
    screen();
    let frame = POPUP.lock().unwrap().as_ref().unwrap().frame;
    assert!(on_mouse(click(frame.x + 3, frame.y + 1)).is_none());
    assert!(is_open());
    reset();
}

#[test]
fn a_long_list_scrolls_by_keys_the_wheel_and_the_bar() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    open(60);
    let (text, _) = screen_of(100, 24);
    assert!(text.contains(".env"), "{text}");
    assert!(
        !text.contains("k59.pem"),
        "the tail is below the fold: {text}"
    );
    {
        let p = POPUP.lock().unwrap();
        let open = p.as_ref().unwrap();
        assert!(open.track.is_some(), "a long list draws its bar");
        assert_eq!(open.top, 0);
    }
    on_key(key(KeyCode::PageDown));
    let paged = POPUP.lock().unwrap().as_ref().unwrap().top;
    assert!(paged > 0);
    on_key(key(KeyCode::End));
    let (text, _) = screen_of(100, 24);
    assert!(text.contains("k59.pem"), "{text}");
    on_key(key(KeyCode::Home));
    assert_eq!(POPUP.lock().unwrap().as_ref().unwrap().top, 0);

    // The wheel over the pane.
    let pane = POPUP.lock().unwrap().as_ref().unwrap().pane;
    on_mouse(mouse(MouseEventKind::ScrollDown, pane.x + 3, pane.y + 2));
    assert_eq!(POPUP.lock().unwrap().as_ref().unwrap().top, WHEEL_LINES);

    // The bar: a press and a drag on its thumb move the list.
    screen_of(100, 24);
    let track = POPUP.lock().unwrap().as_ref().unwrap().track.unwrap();
    on_mouse(click(track.rect.x, track.rect.bottom() - 2));
    let after_press = POPUP.lock().unwrap().as_ref().unwrap().top;
    assert!(
        after_press > WHEEL_LINES,
        "a press on the track jumps there"
    );
    on_mouse(mouse(
        MouseEventKind::Up(MouseButton::Left),
        track.rect.x,
        track.rect.bottom() - 2,
    ));
    reset();
}

#[test]
fn a_file_name_cannot_repaint_the_screen_and_refusals_queue() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    ask(
        PathBuf::from("/a"),
        Action::StageFile("x".into()),
        vec![Sensitive {
            path: "bad\u{1b}[2Jname.pem".into(),
            reason: Reason::Pattern("*.pem".into()),
        }],
    );
    ask(PathBuf::from("/b"), Action::StageAll, files(0));
    let text = screen();
    assert!(
        !text.contains('\u{1b}'),
        "an escape never reaches the screen"
    );
    assert!(text.contains("bad\\x1b[2Jname.pem"), "{text}");
    // Answering the first brings up the second.
    on_key(key(KeyCode::Esc));
    assert!(is_open());
    assert_eq!(
        POPUP.lock().unwrap().as_ref().unwrap().root,
        PathBuf::from("/b")
    );
    on_key(key(KeyCode::Esc));
    assert!(!is_open());
}
