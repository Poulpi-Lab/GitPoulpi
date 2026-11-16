use super::*;
use crate::tag_where_ui::tests::tag;
use crossterm::event::KeyModifiers;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

/// The dialog is process-wide: tests that open it take turns.
static SERIAL: Mutex<()> = Mutex::new(());

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn screen() -> String {
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    terminal.draw(render).unwrap();
    crate::dialog_kit::buffer_text(terminal.backend().buffer())
}

fn click(x: u16, y: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: x,
        row: y,
        modifiers: crossterm::event::KeyModifiers::NONE,
    }
}

fn install_one(place_local: bool, pushed: &[&str]) {
    crate::tag_where_ui::install(&[tag("v1", 1, place_local, pushed, &[])]);
}

fn mode_prompt(mode: &InputMode) -> (&str, &PendingDestructive) {
    let InputMode::Confirm { prompt, action } = mode else {
        panic!("expected a confirm modal");
    };
    let ConfirmAction::Destructive(pending) = action else {
        panic!("expected a destructive confirm");
    };
    (prompt.as_str(), pending)
}

#[test]
fn a_tag_on_one_remote_offers_the_three_deletes_in_order() {
    assert_eq!(
        options(true, &["origin".to_string()]),
        vec![
            Scope::Local,
            Scope::Remote("origin".into()),
            Scope::Both("origin".into())
        ]
    );
    // A tag only on a remote has no local rows.
    assert_eq!(
        options(false, &["origin".to_string(), "up".to_string()]),
        vec![Scope::Remote("origin".into()), Scope::Remote("up".into())]
    );
}

#[test]
fn the_dialog_lists_numbered_rows_and_explains_the_highlighted_one() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    install_one(true, &["origin"]);
    let mut mode = InputMode::Normal;
    let mut message = None;
    begin("v1", false, &mut mode, &mut message);
    assert!(is_open());
    let text = screen();
    assert!(text.contains("Where should v1 be deleted?"), "{text}");
    assert!(text.contains("1  Delete locally"), "{text}");
    assert!(text.contains("2  Delete on origin"), "{text}");
    assert!(text.contains("3  Delete locally and on origin"), "{text}");
    // The first row is highlighted: its warning is the one on screen.
    assert!(text.contains("brings it back"), "{text}");

    // Down to row 2: the detail changes to the remote delete's.
    assert!(on_key(key(KeyCode::Down)).is_none());
    let text = screen();
    assert!(text.contains("for everyone who uses it"), "{text}");
    assert!(!text.contains("brings it back"), "{text}");
    on_key(key(KeyCode::Esc));
    assert!(!is_open());
}

#[test]
fn a_number_picks_its_row_and_esc_picks_nothing() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    install_one(true, &["origin"]);
    let mut mode = InputMode::Normal;
    let mut message = None;
    begin("v1", false, &mut mode, &mut message);
    let pick = on_key(key(KeyCode::Char('3'))).expect("3 picks the third row");
    assert_eq!(pick.scope, Scope::Both("origin".into()));
    assert!(!is_open());

    begin("v1", false, &mut mode, &mut message);
    assert_eq!(on_key(key(KeyCode::Char('9'))), None, "no ninth row");
    assert!(is_open());
    assert_eq!(on_key(key(KeyCode::Esc)), None);
    assert!(!is_open());

    // Enter picks the highlighted one.
    begin("v1", false, &mut mode, &mut message);
    on_key(key(KeyCode::Down));
    let pick = on_key(key(KeyCode::Enter)).unwrap();
    assert_eq!(pick.scope, Scope::Remote("origin".into()));
}

#[test]
fn the_palettes_remote_delete_starts_on_the_remote_row() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    install_one(true, &["origin"]);
    let mut mode = InputMode::Normal;
    let mut message = None;
    begin("v1", true, &mut mode, &mut message);
    let pick = on_key(key(KeyCode::Enter)).unwrap();
    assert_eq!(pick.scope, Scope::Remote("origin".into()));
}

#[test]
fn a_click_on_a_row_picks_it_and_a_click_away_closes() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    install_one(true, &["origin"]);
    let mut mode = InputMode::Normal;
    let mut message = None;
    begin("v1", false, &mut mode, &mut message);
    screen(); // the rows' rects come from the paint
    let second = POPUP.lock().unwrap().as_ref().unwrap().rows[1];
    let pick = on_mouse(click(second.x + 3, second.y)).expect("a click picks");
    assert_eq!(pick.scope, Scope::Remote("origin".into()));
    assert!(!is_open());

    begin("v1", false, &mut mode, &mut message);
    screen();
    assert_eq!(on_mouse(click(0, 0)), None);
    assert!(!is_open(), "a click away closes it");
}

#[test]
fn remote_and_both_ask_a_confirm_naming_the_remote_and_local_runs_at_once() {
    let remote = decide(Pick {
        name: "v1".into(),
        scope: Scope::Remote("origin".into()),
        on: vec!["origin".into()],
    });
    let Decision::Confirm { prompt, pending } = remote else {
        panic!("a remote delete is confirmed");
    };
    assert!(prompt.contains("origin"), "{prompt}");
    assert!(
        prompt.contains("The local"),
        "it says the local tag stays: {prompt}"
    );
    assert!(matches!(
        pending,
        PendingDestructive::DeleteRemoteTag { .. }
    ));

    let both = decide(Pick {
        name: "v1".into(),
        scope: Scope::Both("origin".into()),
        on: vec!["origin".into()],
    });
    let Decision::Confirm { prompt, pending } = both else {
        panic!("a combined delete is confirmed");
    };
    assert!(prompt.contains("here AND on origin"), "{prompt}");
    assert!(prompt.contains("Undo restores it only here"), "{prompt}");
    assert!(matches!(pending, PendingDestructive::DeleteTagBoth { .. }));

    let local = decide(Pick {
        name: "v1".into(),
        scope: Scope::Local,
        on: vec!["origin".into()],
    });
    assert!(matches!(local, Decision::DeleteLocal { .. }));
}

#[test]
fn a_tag_only_here_keeps_the_plain_confirm_and_a_remote_only_one_goes_to_its_remote() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    install_one(true, &[]);
    let mut mode = InputMode::Normal;
    let mut message = None;
    begin("v1", false, &mut mode, &mut message);
    assert!(!is_open(), "nothing to ask: only here");
    let (_, pending) = mode_prompt(&mode);
    assert!(matches!(pending, PendingDestructive::DeleteTag(n) if n == "v1"));

    install_one(false, &["origin"]);
    let mut mode = InputMode::Normal;
    begin("v1", false, &mut mode, &mut message);
    assert!(
        !is_open(),
        "one remote, no local tag: straight to its confirm"
    );
    let (prompt, pending) = mode_prompt(&mode);
    assert!(prompt.contains("origin"), "{prompt}");
    assert!(matches!(
        pending,
        PendingDestructive::DeleteRemoteTag { .. }
    ));
}

/// A window too small for the dialog: its frame says so and still closes with
/// Esc (it was an invisible modal that ate the keys).
#[test]
fn a_window_too_small_says_so_and_esc_still_closes() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    install_one(true, &["origin"]);
    let mut mode = InputMode::Normal;
    let mut message = None;
    begin("v1", false, &mut mode, &mut message);
    let mut terminal = Terminal::new(TestBackend::new(40, 10)).unwrap();
    terminal.draw(render).unwrap();
    let text = crate::dialog_kit::buffer_text(terminal.backend().buffer());
    assert!(text.contains("Window too small"), "{text}");
    assert!(text.contains('\u{2715}'), "{text}");
    on_key(key(KeyCode::Esc));
    assert!(!is_open());
}
