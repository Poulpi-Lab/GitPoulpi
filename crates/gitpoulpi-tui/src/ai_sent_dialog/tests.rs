use super::*;
use gitpoulpi_core::privacy::Redaction;
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

fn click(x: u16, y: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: x,
        row: y,
        modifiers: KeyModifiers::NONE,
    }
}

fn sent() -> Prepared {
    Prepared {
        system: "You write commit messages.".into(),
        user: "diff -- src/a.rs (modified)\n+let token = \"[REDACTED:github-token]\";\n".into(),
        redactions: vec![Redaction {
            rule: "github-token".into(),
            preview: "ghp_****abcd".into(),
        }],
        excluded_files: vec![".env".into()],
    }
}

fn screen_of(width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(render).unwrap();
    crate::dialog_kit::buffer_text(terminal.backend().buffer())
}

async fn gate_confirm(gate: &Arc<dyn SendGate>, sent: &Prepared) -> bool {
    gate.confirm(sent).await
}

/// Asks the gate from a worker thread, like the AI worker does, and waits for
/// the dialog to appear.
fn ask_from_worker(sent: Prepared) -> std::thread::JoinHandle<bool> {
    reset();
    let handle = std::thread::spawn(move || {
        let gate = Gate::shared(
            AiAction::CommitMessage,
            "Claude".into(),
            "Commit message".into(),
        );
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async move { gate_confirm(&gate, &sent).await })
    });
    for _ in 0..300 {
        if is_open() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(is_open(), "the dialog opened");
    handle
}

/// A hit rect of the last paint.
fn rect_of(hit: Hit) -> Rect {
    POPUP
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .hits
        .iter()
        .find(|(_, h)| *h == hit)
        .map(|(r, _)| *r)
        .unwrap()
}

#[test]
fn it_shows_the_cleaned_request_what_was_removed_and_the_controls() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let worker = ask_from_worker(sent());
    let text = screen_of(110, 34);
    assert!(text.contains("Review what is sent to Claude"), "{text}");
    assert!(text.contains("1 secret replaced"), "{text}");
    assert!(text.contains("github-token: ghp_****abcd"), "{text}");
    assert!(text.contains("1 sensitive file left out"), "{text}");
    assert!(text.contains(".env"), "{text}");
    assert!(text.contains("What is sent"), "{text}");
    assert!(text.contains("You write commit messages."), "{text}");
    assert!(text.contains("[REDACTED:github-token]"), "{text}");
    assert!(
        text.contains("[ ] Don't show again for Commit message"),
        "{text}"
    );
    assert!(text.contains("Send") && text.contains("Cancel"), "{text}");
    // Nothing is sent until an answer: the worker is still waiting.
    assert!(!worker.is_finished());
    on_key(key(KeyCode::Esc));
    assert!(!worker.join().unwrap());
    assert!(!is_open());
}

#[test]
fn send_cancel_the_cross_and_a_click_away_answer_by_mouse() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let worker = ask_from_worker(sent());
    screen_of(110, 34);
    let send = rect_of(Hit::Send);
    on_mouse(click(send.x + 2, send.y));
    assert!(worker.join().unwrap(), "Send answers yes");

    let worker = ask_from_worker(sent());
    screen_of(110, 34);
    let cancel = rect_of(Hit::Cancel);
    on_mouse(click(cancel.x + 2, cancel.y));
    assert!(!worker.join().unwrap(), "Cancel answers no");

    let worker = ask_from_worker(sent());
    screen_of(110, 34);
    let cross = POPUP
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .close
        .rect()
        .unwrap();
    on_mouse(click(cross.x, cross.y));
    assert!(!worker.join().unwrap(), "the cross answers no");

    let worker = ask_from_worker(sent());
    screen_of(110, 34);
    on_mouse(click(0, 0));
    assert!(!worker.join().unwrap(), "a click away answers no");
}

#[test]
fn the_keyboard_twins_answer_and_toggle() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    // Send is focused first: Enter sends.
    let worker = ask_from_worker(sent());
    screen_of(110, 34);
    on_key(key(KeyCode::Enter));
    assert!(worker.join().unwrap());

    // Tab to Cancel and Enter.
    let worker = ask_from_worker(sent());
    screen_of(110, 34);
    on_key(key(KeyCode::Tab));
    on_key(key(KeyCode::Enter));
    assert!(!worker.join().unwrap());

    // The box: Shift+Tab from Send goes to it, Space ticks it, a click on its
    // row ticks it back, and the pane hides with Alt+W.
    let worker = ask_from_worker(sent());
    screen_of(110, 34);
    on_key(key(KeyCode::BackTab));
    on_key(key(KeyCode::Char(' ')));
    assert!(POPUP.lock().unwrap().as_ref().unwrap().dont_show);
    assert!(screen_of(110, 34).contains("[x] Don't show again"));
    let dont = rect_of(Hit::Dont);
    on_mouse(click(dont.x + 1, dont.y));
    assert!(!POPUP.lock().unwrap().as_ref().unwrap().dont_show);
    on_key(KeyEvent::new(KeyCode::Char('w'), KeyModifiers::ALT));
    assert!(!POPUP.lock().unwrap().as_ref().unwrap().pane.open);
    assert!(!screen_of(110, 34).contains("You write commit messages."));
    on_key(key(KeyCode::Esc));
    assert!(!worker.join().unwrap());
}

#[test]
fn a_long_request_scrolls_by_keys_the_wheel_and_the_bar() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let mut big = sent();
    big.user = (0..200)
        .map(|i| format!("line number {i}"))
        .collect::<Vec<_>>()
        .join("\n");
    let worker = ask_from_worker(big);
    let text = screen_of(110, 30);
    assert!(text.contains("line number 0"), "{text}");
    assert!(!text.contains("line number 199"), "{text}");
    on_key(key(KeyCode::End));
    let text = screen_of(110, 30);
    assert!(text.contains("line number 199"), "{text}");
    on_key(key(KeyCode::Home));
    let pane = rect_of(Hit::Pane);
    on_mouse(MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: pane.x + 4,
        row: pane.y + 1,
        modifiers: KeyModifiers::NONE,
    });
    assert_eq!(POPUP.lock().unwrap().as_ref().unwrap().pane.top, 3);
    // The bar is pressable: a press low on its track jumps there.
    screen_of(110, 30);
    let bar = rect_of(Hit::Bar);
    on_mouse(click(bar.x, bar.bottom() - 2));
    assert!(POPUP.lock().unwrap().as_ref().unwrap().pane.top > 3);
    on_key(key(KeyCode::Esc));
    assert!(!worker.join().unwrap());
}

#[test]
fn control_characters_in_the_request_never_reach_the_screen() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let mut bad = sent();
    bad.user = "evil \u{1b}[2J text".into();
    let worker = ask_from_worker(bad);
    let text = screen_of(110, 34);
    assert!(!text.contains('\u{1b}'));
    assert!(text.contains("evil \\x1b[2J text"), "{text}");
    on_key(key(KeyCode::Esc));
    assert!(!worker.join().unwrap());
}

#[test]
fn a_request_canceled_elsewhere_closes_its_dialog() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    let (tx, rx) = tokio::sync::oneshot::channel();
    ask(Open {
        action: AiAction::CommitMessage,
        via: "x".into(),
        action_label: "x".into(),
        sent: sent(),
        dont_show: false,
        focus: Focus::Send,
        hover: None,
        pane: SentPane::default(),
        hits: Vec::new(),
        frame: Rect::default(),
        close: Default::default(),
        reply: Some(tx),
    });
    assert!(is_open());
    drop(rx);
    assert!(
        !is_open(),
        "the receiver is gone: nothing is waiting for an answer"
    );
}

#[test]
fn dont_show_again_turns_the_action_off_in_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    remember_off_in(&path, AiAction::ExplainChanges);
    let config = gitpoulpi_config::Config::load(&path).unwrap();
    assert!(!config.ai.sent_preview.shows(AiAction::ExplainChanges));
    assert!(config.ai.sent_preview.shows(AiAction::CommitMessage));
}
