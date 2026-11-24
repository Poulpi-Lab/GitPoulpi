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
    *PEEK.lock().unwrap() = Peek::Off;
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
            line: None,
        },
        Sensitive {
            path: "notes.txt".into(),
            reason: Reason::Secret {
                rule: "github-token".into(),
                preview: "ghp_****".into(),
            },
            line: Some(7),
        },
    ];
    for i in 0..n {
        out.push(Sensitive {
            path: format!("keys/k{i}.pem"),
            reason: Reason::Pattern("*.pem".into()),
            line: None,
        });
    }
    out
}

fn open(n: usize) {
    reset();
    ask(PathBuf::from("/repo"), Action::StageAll, files(n), None);
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
            line: None,
        }],
        None,
    );
    ask(PathBuf::from("/b"), Action::StageAll, files(0), None);
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

#[test]
fn a_secret_names_its_line_as_path_colon_line() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    open(0);
    let text = screen();
    assert!(text.contains("notes.txt:7"), "{text}");
    // A path-pattern finding has no line to name.
    assert!(!text.contains(".env:"), "{text}");
    assert!(text.contains("Show in file"), "{text}");
    reset();
}

/// What the window does with the ask: opens the file, scrolls, comes back.
fn walk_the_round_trip(root: &str, expect_path: &str, expect_line: Option<u32>) {
    assert!(!is_open(), "the question steps aside");
    assert!(peeking());
    assert_eq!(peek_request(std::path::Path::new("/other")), None);
    let (path, line) = peek_request(std::path::Path::new(root)).expect("the file is asked for");
    assert_eq!((path.as_str(), line), (expect_path, expect_line));
    assert_eq!(peek_line(), expect_line);
    peek_applied();
    assert_eq!(peek_line(), None);
    assert!(!is_open(), "still stepped aside while the File View is up");
    peek_closed();
    assert!(is_open(), "the File View closed: the question is back");
}

#[test]
fn show_in_file_from_the_key_opens_the_focused_finding_and_comes_back() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    open(0);
    screen();
    // The first finding is the pattern one: a file with no line.
    on_key(key(KeyCode::Char('s')));
    walk_the_round_trip("/repo", ".env", None);
    // Tab focuses the next finding: the secret on line 7.
    screen();
    on_key(key(KeyCode::Tab));
    on_key(key(KeyCode::Char('S')));
    walk_the_round_trip("/repo", "notes.txt", Some(7));
    // Shift+Tab and Ctrl+K go back, wrapping.
    on_key(key(KeyCode::BackTab));
    on_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::CONTROL));
    on_key(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL));
    on_key(key(KeyCode::Char('s')));
    walk_the_round_trip("/repo", ".env", None);
    reset();
}

#[test]
fn show_in_file_from_the_button_and_a_row_click_pick_the_file() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    open(0);
    screen();
    let (button, hit) = {
        let slot = POPUP.lock().unwrap();
        let open = slot.as_ref().unwrap();
        let hit = open.row_hits.iter().find(|(_, f)| *f == 1).unwrap().0;
        (open.show_button, hit)
    };
    assert!(button.width > 0, "the button is painted");
    // A click on the secret's row focuses it, the button opens it.
    on_mouse(click(hit.x + 2, hit.y));
    assert!(is_open());
    on_mouse(click(button.x + 1, button.y));
    walk_the_round_trip("/repo", "notes.txt", Some(7));
    reset();
}

#[test]
fn the_palette_entry_shows_the_focused_file_only_while_the_question_is_open() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    show_in_file();
    assert!(!peeking(), "nothing is asked of a closed question");
    open(0);
    screen();
    show_in_file();
    walk_the_round_trip("/repo", ".env", None);
    reset();
}

#[test]
fn a_waiting_refusal_does_not_lose_the_peek_round_trip() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    open(0);
    screen();
    on_key(key(KeyCode::Char('s')));
    // A second refusal arrives while the File View is up.
    ask(PathBuf::from("/b"), Action::StageAll, files(0), None);
    assert_eq!(WAITING.lock().unwrap().len(), 1);
    peek_request(std::path::Path::new("/repo"));
    peek_closed();
    assert!(is_open());
    reset();
}

/// A tracked stage the guard refuses reports "waiting for you", and the
/// dialog's answer resubmits it under the same ticket.
fn guarded_stage(
    name: &str,
) -> (
    crate::daemon_link::ticket_tests::Harness,
    gitpoulpi_testkit::FixtureRepo,
    crate::daemon_link::Ticket,
) {
    use crate::daemon_link::{StepOutcome, UserWait};
    let fixture = gitpoulpi_testkit::FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .write_worktree(".env", b"TOKEN=1\n")
        .unwrap()
        .build();
    let h = crate::daemon_link::ticket_tests::harness(name, &fixture);
    let ticket = h.link.submit_tracked(Action::StageAll);
    assert_eq!(
        crate::daemon_link::ticket_tests::wait_needs_user(ticket),
        vec![StepOutcome::NeedsUser(UserWait::SensitiveStage)]
    );
    for _ in 0..200 {
        if POPUP.lock().unwrap().is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert!(
        POPUP.lock().unwrap().is_some(),
        "the guard's question is open"
    );
    (h, fixture, ticket)
}

#[test]
fn stage_anyway_reports_ok_under_the_same_ticket() {
    use crate::daemon_link::StepOutcome;
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    let (h, fixture, ticket) = guarded_stage("ticket-sensitive-ok");
    let staged = on_key(key(KeyCode::Char('1'))).expect("answer 1 is Stage anyway");
    assert_eq!(staged.ticket, Some(ticket));
    apply(staged, &h.link, fixture.root());
    assert_eq!(
        crate::daemon_link::ticket_tests::wait_final(ticket),
        vec![StepOutcome::Ok(None)]
    );
    assert!(fixture.git(&["status", "--porcelain"]).contains("A  .env"));
    reset();
}

#[test]
fn every_way_of_closing_the_question_cancels_the_ticket() {
    use crate::daemon_link::{take_for, StepOutcome};
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    // Answer 2 ("Cancel, stage nothing").
    let (_h, fixture, ticket) = guarded_stage("ticket-sensitive-cancel");
    assert!(on_key(key(KeyCode::Char('2'))).is_none());
    assert_eq!(take_for(ticket), vec![StepOutcome::Canceled]);
    assert!(!fixture.git(&["status", "--porcelain"]).contains("A  .env"));
    // Esc.
    let (_h, _fixture, ticket) = guarded_stage("ticket-sensitive-esc");
    assert!(on_key(key(KeyCode::Esc)).is_none());
    assert_eq!(take_for(ticket), vec![StepOutcome::Canceled]);
    reset();
}
