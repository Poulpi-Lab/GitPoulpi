use super::*;
use crossterm::event::{KeyCode, KeyEventState, KeyModifiers, MouseButton, MouseEventKind};
use ratatui::buffer::Buffer;

/// The dialog's state is process-wide: tests that touch it take turns.
static TURN: Mutex<()> = Mutex::new(());

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

fn click(col: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: col,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

fn paint(dialog: &GuideDialog, w: u16, h: u16) -> String {
    let area = Rect::new(0, 0, w, h);
    let mut buf = Buffer::empty(area);
    dialog.render(&mut buf, area);
    crate::dialog_kit::buffer_text(&buf)
}

fn cuts() -> Vec<Cut> {
    vec![
        Cut::new(Part::Diff, "", 91_234, 60_000),
        Cut::new(Part::ReviewFile, "src/big.rs", 30_000, 12_000),
        Cut::new(Part::ContextFile, "CLAUDE.md", 9_000, 8_192),
    ]
}

#[test]
fn it_says_which_part_is_over_by_how_much_and_which_setting_raises_it() {
    let text = paint(&dialog(&cuts()), 120, 40);
    assert!(text.contains("Over the AI limit"), "{text}");
    assert!(text.contains("The diff is 91.2 KB (91234"), "{text}");
    assert!(text.contains("limit 60.0 KB (60000"), "{text}");
    assert!(text.contains("ai.limits.diff_bytes"), "{text}");
    assert!(text.contains("The diff of src/big.rs is 30.0 KB"), "{text}");
    assert!(text.contains("ai.limits.review_file_bytes"), "{text}");
    assert!(
        text.contains("The context file CLAUDE.md is 9.0 KB"),
        "{text}"
    );
    assert!(
        text.contains("ai.limits.review_context_file_bytes"),
        "{text}"
    );
    // What each button does, and that privacy still applies.
    assert!(text.contains("Truncated, its end is left out"), "{text}");
    assert!(text.contains("secrets are still"), "{text}");
    for button in [
        "Send all (A)",
        "Send truncated (T)",
        "Open setting (O)",
        "Cancel (C)",
    ] {
        assert!(text.contains(button), "{button}: {text}");
    }
    assert!(text.contains('\u{2715}'), "the close cross: {text}");
}

#[test]
fn a_long_list_is_capped_and_says_how_many_more() {
    let many: Vec<Cut> = (0..9)
        .map(|i| Cut::new(Part::ReviewFile, format!("src/f{i}.rs"), 20_000, 12_000))
        .collect();
    let text = paint(&dialog(&many), 120, 40);
    assert!(text.contains("src/f5.rs"), "{text}");
    assert!(!text.contains("src/f6.rs"), "{text}");
    assert!(text.contains("and 3 more"), "{text}");
}

#[test]
fn every_button_has_a_key_escape_cancels_and_focus_starts_on_open_setting() {
    let mut d = dialog(&cuts());
    paint(&d, 120, 40);
    for (c, want) in [
        ('a', Choice::Primary),
        ('t', Choice::Truncated),
        ('o', Choice::Secondary),
        ('c', Choice::Cancel),
        ('A', Choice::Primary),
    ] {
        assert_eq!(d.on_key(key(KeyCode::Char(c))), Outcome::Chose(want), "{c}");
    }
    assert_eq!(d.on_key(key(KeyCode::Esc)), Outcome::Chose(Choice::Cancel));
    // Sending is never one Enter away.
    let mut d = dialog(&cuts());
    assert_eq!(
        d.on_key(key(KeyCode::Enter)),
        Outcome::Chose(Choice::Secondary)
    );
}

#[test]
fn the_cross_and_a_click_outside_cancel_and_buttons_are_clickable() {
    let mut d = dialog(&cuts());
    let text = paint(&d, 120, 40);
    // Find the cross and the buttons by their painted text.
    let (cross_row, cross_col) = text
        .lines()
        .enumerate()
        .find_map(|(r, l)| {
            l.chars()
                .position(|c| c == '\u{2715}')
                .map(|c| (r as u16, c as u16))
        })
        .expect("a cross");
    assert_eq!(
        d.on_mouse(click(cross_col, cross_row)),
        Outcome::Chose(Choice::Cancel)
    );
    assert_eq!(d.on_mouse(click(0, 0)), Outcome::Chose(Choice::Cancel));
    let (row, col) = text
        .lines()
        .enumerate()
        .find_map(|(r, l)| l.find("Send truncated (T)").map(|c| (r as u16, c as u16)))
        .expect("the button");
    // `find` is a byte offset and the line is ASCII up to the button: the
    // frame's box characters are multi-byte, so count chars.
    let col = text.lines().nth(row as usize).unwrap()[..col as usize]
        .chars()
        .count() as u16;
    assert_eq!(
        d.on_mouse(click(col + 1, row)),
        Outcome::Chose(Choice::Truncated)
    );
}

fn ask_in_thread(cuts: Vec<Cut>) -> std::thread::JoinHandle<LimitAnswer> {
    std::thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(Gate::shared().ask(&cuts))
    })
}

fn wait_open() {
    for _ in 0..500 {
        if is_open() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("the dialog never opened");
}

/// The worker waits for the dialog and gets exactly what was chosen.
#[test]
fn the_worker_gets_the_answer_of_the_button_pressed() {
    let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
    for (press, want) in [
        ('a', LimitAnswer::SendAll),
        ('t', LimitAnswer::SendTruncated),
        ('c', LimitAnswer::Cancel),
    ] {
        let worker = ask_in_thread(cuts());
        wait_open();
        on_key(key(KeyCode::Char(press)));
        assert_eq!(worker.join().unwrap(), want, "{press}");
        assert!(!is_open(), "the dialog closed");
        assert_eq!(take_open_setting(), None);
    }
    let worker = ask_in_thread(cuts());
    wait_open();
    on_key(key(KeyCode::Esc));
    assert_eq!(worker.join().unwrap(), LimitAnswer::Cancel);
}

/// Open setting cancels the request and names the first limit listed.
#[test]
fn open_setting_cancels_the_request_and_names_the_first_limit() {
    let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
    let worker = ask_in_thread(vec![
        Cut::new(Part::ReviewText, "", 5_000, 4_000),
        Cut::new(Part::Diff, "", 70_000, 60_000),
    ]);
    wait_open();
    on_key(key(KeyCode::Char('o')));
    assert_eq!(worker.join().unwrap(), LimitAnswer::Cancel);
    assert_eq!(take_open_setting(), Some("ai.limits.review_text_bytes"));
    assert_eq!(take_open_setting(), None, "taken once");
}

/// A request canceled elsewhere (the jobs list) drops its question.
#[test]
fn a_request_canceled_elsewhere_closes_its_question() {
    let _turn = TURN.lock().unwrap_or_else(|e| e.into_inner());
    let (tx, rx) = tokio::sync::oneshot::channel();
    ask(Open {
        dialog: dialog(&cuts()),
        setting: "ai.limits.diff_bytes",
        reply: Some(tx),
    });
    assert!(is_open());
    drop(rx);
    assert!(!is_open());
}
