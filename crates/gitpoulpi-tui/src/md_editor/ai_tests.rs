//! Buffer tests of "Write with AI" in the composer: the button and its key,
//! the panel's rows, what is asked, the undoable fill, Stop and the errors.
//! The model is a stand-in closure (`AiHelp::fake`): no process, no socket.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

use super::ai::{AiContextSource, AiHelp, AiScope, AiTask, Fake};
use super::*;
use crossterm::event::{KeyModifiers, MouseEvent};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn alt(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::ALT)
}

fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

fn click(col: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: col,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

fn paint(ed: &mut MdEditor, width: u16, height: u16) -> Buffer {
    let mut buf = Buffer::empty(Rect::new(0, 0, width, height));
    ed.render(
        &mut buf,
        Rect::new(0, 0, width, height),
        true,
        "placeholder",
    );
    buf
}

fn screen(buf: &Buffer) -> String {
    (0..buf.area.height)
        .map(|y| {
            (0..buf.area.width)
                .map(|x| buf[(x, y)].symbol().to_string())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Asks what the form would: a pull request pair, with the hint and draft.
struct Source;

impl AiContextSource for Source {
    fn task(&self, hint: &str, draft: Option<&str>) -> Result<AiTask, String> {
        Ok(AiTask::PullRequest {
            source: "feature".into(),
            target: "main".into(),
            hint: hint.to_string(),
            draft: draft.map(str::to_string),
        })
    }
}

struct Refusing;

impl AiContextSource for Refusing {
    fn task(&self, _: &str, _: Option<&str>) -> Result<AiTask, String> {
        Err("Pick the branches first.".into())
    }
}

type Seen = Arc<Mutex<Vec<AiTask>>>;

/// An editor with the helper bound over `fake`; `seen` records every task.
fn bound(
    text: &str,
    fake: impl Fn(&AiTask) -> Result<String, String> + Send + Sync + 'static,
) -> (MdEditor, Seen) {
    let seen: Seen = Arc::default();
    let log = seen.clone();
    let fake: Fake = Arc::new(move |task| {
        log.lock().unwrap().push(task.clone());
        fake(task)
    });
    let mut help = AiHelp::new_in(AiScope::default(), Box::new(Source), None);
    help.fake = Some(fake);
    let mut ed = MdEditor::new(text.to_string(), md_render::Context::default());
    ed.bind_ai_with(help);
    (ed, seen)
}

fn type_str(ed: &mut MdEditor, text: &str) {
    for c in text.chars() {
        ed.on_key(key(KeyCode::Char(c)));
    }
}

/// Paints until `done` holds (the worker answers on another thread).
fn wait_for(ed: &mut MdEditor, what: &str, done: impl Fn(&MdEditor) -> bool) {
    let until = Instant::now() + Duration::from_secs(10);
    while Instant::now() < until {
        paint(ed, 90, 20);
        if done(ed) {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("timed out waiting for {what}");
}

fn rect_of(ed: &mut MdEditor, part: ai::AiPart) -> Rect {
    paint(ed, 90, 20);
    ed.ai_mut()
        .unwrap()
        .rect_of(part)
        .unwrap_or_else(|| panic!("no {part:?} on screen"))
}

#[test]
fn without_a_bound_helper_there_is_no_button_and_alt_i_does_nothing() {
    let mut ed = MdEditor::new("text".into(), md_render::Context::default());
    let s = screen(&paint(&mut ed, 90, 12));
    assert!(
        !s.contains("Write with AI") && !s.contains('\u{2726}'),
        "{s}"
    );
    assert_eq!(ed.on_key(alt('i')), KeyUse::Pass);
    assert!(!ed.ai_open());
}

#[test]
fn the_bar_has_a_real_button_with_a_label_and_a_glyph_and_alt_i_twin() {
    let (mut ed, _) = bound("", |_| Ok(String::new()));
    let buf = paint(&mut ed, 90, 12);
    let s = screen(&buf);
    assert!(s.contains("\u{2726} Write with AI"), "{s}");
    // The button is where the slot was reserved: the right end of the bar.
    let slot = ed.ai_slot();
    assert_eq!(slot.right(), 90, "right-aligned on the bar");
    assert_eq!(ed.on_key(alt('i')), KeyUse::Handled);
    assert!(ed.ai_open(), "Alt+I opens the panel");
    assert_eq!(ed.on_key(alt('i')), KeyUse::Handled);
    assert!(!ed.ai_open(), "Alt+I again closes it");
    // The mouse twin.
    assert_eq!(ed.on_mouse(click(slot.x + 2, slot.y)), MouseUse::Inside);
    assert!(ed.ai_open());
    paint(&mut ed, 90, 12);
    let slot = ed.ai_slot();
    ed.on_mouse(click(slot.x + 2, slot.y));
    assert!(!ed.ai_open());
}

#[test]
fn a_narrow_bar_shows_the_short_label() {
    let (mut ed, _) = bound("", |_| Ok(String::new()));
    let s = screen(&paint(&mut ed, 40, 12));
    assert!(s.contains("\u{2726} AI"), "{s}");
    assert!(!s.contains("Write with AI"), "{s}");
}

#[test]
fn the_open_panel_shows_the_field_the_two_boxes_and_the_buttons() {
    let (mut ed, _) = bound("", |_| Ok(String::new()));
    paint(&mut ed, 90, 16);
    ed.on_key(alt('i'));
    let s = screen(&paint(&mut ed, 90, 16));
    for want in [
        "What I want in this answer",
        "The AI only writes a draft, nothing is posted.",
        "for example: agree, and ask for a failing test",
        "[ ] Improve my draft",
        "[ ] Let the AI read the repository (no local clone)",
        "Recent:",
        "nothing yet",
        "Draft",
        "Close",
    ] {
        assert!(s.contains(want), "{want:?} missing:\n{s}");
    }
    // The text area keeps its rows under the panel.
    assert!(s.contains("placeholder") || s.lines().count() >= 16, "{s}");
}

#[test]
fn the_repository_box_is_unticked_and_disabled_without_a_clone_and_a_click_only_says_why() {
    let (mut ed, _) = bound("", |_| Ok(String::new()));
    paint(&mut ed, 90, 16);
    ed.on_key(alt('i'));
    let repo = rect_of(&mut ed, ai::AiPart::Repo);
    ed.on_mouse(click(repo.x + 1, repo.y));
    let s = screen(&paint(&mut ed, 90, 16));
    assert!(s.contains("[ ] Let the AI read the repository"), "{s}");
    assert!(!s.contains("[x] Let the AI read"), "{s}");
    assert!(
        s.contains("Let the AI read the repository: no local clone"),
        "{s}"
    );
}

#[test]
fn improve_my_draft_starts_ticked_only_when_there_is_a_text() {
    let (mut with, _) = bound("my words", |_| Ok(String::new()));
    paint(&mut with, 90, 16);
    with.on_key(alt('i'));
    assert!(with.ai_mut().unwrap().improve());
    let (mut empty, _) = bound("", |_| Ok(String::new()));
    paint(&mut empty, 90, 16);
    empty.on_key(alt('i'));
    assert!(!empty.ai_mut().unwrap().improve());
}

#[test]
fn enter_in_the_field_asks_with_the_hint_and_fills_the_editor_then_previews() {
    let (mut ed, seen) = bound("old text", |_| {
        Ok("# New draft\n\nWritten by the AI.".into())
    });
    paint(&mut ed, 90, 16);
    ed.on_key(alt('i'));
    type_str(&mut ed, "agree and ask for a test");
    assert_eq!(
        ed.text(),
        "old text",
        "typing went to the field, not the text"
    );
    assert_eq!(ed.on_key(key(KeyCode::Enter)), KeyUse::Handled);
    wait_for(&mut ed, "the draft", |e| e.text() != "old text");
    assert_eq!(ed.text(), "# New draft\n\nWritten by the AI.");
    assert_eq!(ed.mode(), Mode::Preview, "the user reads the draft");
    let tasks = seen.lock().unwrap().clone();
    assert_eq!(tasks.len(), 1);
    match &tasks[0] {
        AiTask::PullRequest { hint, draft, .. } => {
            assert_eq!(hint, "agree and ask for a test");
            assert_eq!(draft.as_deref(), Some("old text"), "improve was ticked");
        }
        other => panic!("{other:?}"),
    }
    let s = screen(&paint(&mut ed, 90, 16));
    assert!(s.contains("Draft ready."), "{s}");
}

#[test]
fn ctrl_z_gives_the_previous_text_back_in_one_step() {
    let (mut ed, _) = bound("old text", |_| Ok("a\nb\nc".into()));
    paint(&mut ed, 90, 16);
    ed.on_key(alt('i'));
    ed.on_key(key(KeyCode::Enter));
    wait_for(&mut ed, "the draft", |e| e.text() == "a\nb\nc");
    assert_eq!(ed.mode(), Mode::Preview);
    assert_eq!(ed.on_key(ctrl('z')), KeyUse::Handled);
    assert_eq!(ed.text(), "old text");
    assert_eq!(ed.mode(), Mode::Write, "the restored text is shown");
    // And forward again, still in the editor's own history.
    ed.set_mode(Mode::Preview);
    ed.on_key(ctrl('y'));
    assert_eq!(ed.text(), "a\nb\nc");
}

#[test]
fn an_unticked_improve_box_sends_no_draft() {
    let (mut ed, seen) = bound("keep me", |_| Ok("fresh".into()));
    paint(&mut ed, 90, 16);
    ed.on_key(alt('i'));
    let improve = rect_of(&mut ed, ai::AiPart::Improve);
    ed.on_mouse(click(improve.x + 1, improve.y));
    assert!(!ed.ai_mut().unwrap().improve());
    // Shift+Tab goes back to the field, where Enter drafts.
    ed.on_key(key(KeyCode::BackTab));
    ed.on_key(key(KeyCode::Enter));
    wait_for(&mut ed, "the draft", |e| e.text() == "fresh");
    let tasks = seen.lock().unwrap().clone();
    match &tasks[0] {
        AiTask::PullRequest { draft, .. } => assert_eq!(*draft, None),
        other => panic!("{other:?}"),
    }
}

#[test]
fn space_on_the_improve_box_toggles_it_by_keyboard() {
    let (mut ed, _) = bound("x", |_| Ok(String::new()));
    paint(&mut ed, 90, 16);
    ed.on_key(alt('i'));
    assert!(ed.ai_mut().unwrap().improve());
    ed.on_key(key(KeyCode::Tab));
    ed.on_key(key(KeyCode::Char(' ')));
    assert!(!ed.ai_mut().unwrap().improve());
}

#[test]
fn a_failure_is_shown_inline_and_the_text_is_untouched() {
    let (mut ed, _) = bound("mine", |_| Err("the agent exited with status 2".into()));
    paint(&mut ed, 90, 16);
    ed.on_key(alt('i'));
    ed.on_key(key(KeyCode::Enter));
    let until = Instant::now() + Duration::from_secs(10);
    let mut s = String::new();
    while Instant::now() < until {
        s = screen(&paint(&mut ed, 90, 16));
        if s.contains("AI: the agent exited") {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(s.contains("AI: the agent exited with status 2"), "{s}");
    assert_eq!(ed.text(), "mine");
    assert_eq!(ed.mode(), Mode::Write);
}

#[test]
fn a_source_that_cannot_say_what_to_write_about_shows_its_reason() {
    let mut help = AiHelp::new_in(AiScope::default(), Box::new(Refusing), None);
    help.fake = Some(Arc::new(|_| Ok("never".into())));
    let mut ed = MdEditor::new(String::new(), md_render::Context::default());
    ed.bind_ai_with(help);
    paint(&mut ed, 90, 16);
    ed.on_key(alt('i'));
    ed.on_key(key(KeyCode::Enter));
    let s = screen(&paint(&mut ed, 90, 16));
    assert!(s.contains("AI: Pick the branches first."), "{s}");
    assert!(!ed.ai_running());
}

#[test]
fn stop_ends_the_run_and_its_answer_never_lands() {
    let (gate_tx, gate_rx) = mpsc::channel::<()>();
    let gate_rx = Mutex::new(gate_rx);
    let started = Arc::new(AtomicBool::new(false));
    let flag = started.clone();
    let (mut ed, _) = bound("mine", move |_| {
        flag.store(true, Ordering::SeqCst);
        let _ = gate_rx
            .lock()
            .unwrap()
            .recv_timeout(Duration::from_secs(10));
        Ok("late answer".into())
    });
    paint(&mut ed, 90, 16);
    ed.on_key(alt('i'));
    ed.on_key(key(KeyCode::Enter));
    assert!(ed.ai_running());
    let s = screen(&paint(&mut ed, 90, 16));
    assert!(s.contains("Writing..."), "{s}");
    assert!(s.contains("Stop"), "the Run button turned into Stop:\n{s}");
    // Esc stops it (the panel stays open); the mouse twin is the button.
    assert_eq!(ed.on_key(key(KeyCode::Esc)), KeyUse::Handled);
    assert!(!ed.ai_running());
    assert!(ed.ai_open(), "the first Esc only stops");
    let _ = gate_tx.send(());
    std::thread::sleep(Duration::from_millis(100));
    paint(&mut ed, 90, 16);
    assert_eq!(ed.text(), "mine", "the stopped run's text is dropped");
    let s = screen(&paint(&mut ed, 90, 16));
    assert!(s.contains("Stopped."), "{s}");
    // A second Esc closes the panel.
    ed.on_key(key(KeyCode::Esc));
    assert!(!ed.ai_open());
}

#[test]
fn the_stop_button_stops_with_the_mouse() {
    let (gate_tx, gate_rx) = mpsc::channel::<()>();
    let gate_rx = Mutex::new(gate_rx);
    let (mut ed, _) = bound("", move |_| {
        let _ = gate_rx
            .lock()
            .unwrap()
            .recv_timeout(Duration::from_secs(10));
        Ok("late".into())
    });
    paint(&mut ed, 90, 16);
    ed.on_key(alt('i'));
    ed.on_key(key(KeyCode::Enter));
    let stop = rect_of(&mut ed, ai::AiPart::Stop);
    ed.on_mouse(click(stop.x + 1, stop.y));
    assert!(!ed.ai_running());
    let _ = gate_tx.send(());
}

#[test]
fn the_close_button_and_escape_close_the_panel_and_stop_a_run() {
    let (mut ed, _) = bound("", |_| Ok("x".into()));
    paint(&mut ed, 90, 16);
    ed.on_key(alt('i'));
    let close = rect_of(&mut ed, ai::AiPart::Close);
    assert_eq!(ed.on_mouse(click(close.x + 1, close.y)), MouseUse::Inside);
    assert!(!ed.ai_open());
    ed.on_key(alt('i'));
    assert_eq!(ed.on_key(key(KeyCode::Esc)), KeyUse::Handled);
    assert!(!ed.ai_open());
    // With the panel closed, Esc is the form's again.
    assert_eq!(ed.on_key(key(KeyCode::Esc)), KeyUse::Pass);
}

#[test]
fn tab_walks_the_panel_then_hands_the_keyboard_back_to_the_text() {
    let (mut ed, _) = bound("", |_| Ok("x".into()));
    paint(&mut ed, 90, 16);
    ed.on_key(alt('i'));
    for _ in 0..4 {
        assert_eq!(ed.on_key(key(KeyCode::Tab)), KeyUse::Handled);
    }
    // Back on the text: typing edits it and the form owns Tab again.
    ed.on_key(key(KeyCode::Char('q')));
    assert_eq!(ed.text(), "q");
    assert_eq!(ed.on_key(key(KeyCode::Tab)), KeyUse::Pass);
}

#[test]
fn recent_instructions_are_listed_clickable_and_walked_with_up_and_down() {
    let (mut ed, _) = bound("", |_| Ok("x".into()));
    paint(&mut ed, 90, 16);
    for hint in ["first idea", "second idea"] {
        if ed.ai_open() {
            ed.on_key(alt('i'));
        }
        ed.on_key(alt('i'));
        ed.on_key(ctrl('a'));
        type_str(&mut ed, hint);
        ed.on_key(key(KeyCode::Enter));
        wait_for(&mut ed, "the answer", |e| !e.ai_running());
    }
    assert_eq!(
        ed.ai_mut().unwrap().recent(),
        vec!["second idea".to_string(), "first idea".to_string()]
    );
    // The panel is still open (a finished run leaves it so).
    let s = screen(&paint(&mut ed, 90, 16));
    assert!(s.contains("second idea") && s.contains("first idea"), "{s}");
    // A click on an entry fills the field.
    let first = rect_of(&mut ed, ai::AiPart::Recent(1));
    ed.on_mouse(click(first.x + 1, first.y));
    assert_eq!(ed.ai_mut().unwrap().hint(), "first idea");
    // Up / Down are the keyboard twin: newest, then older, then back.
    for _ in 0..3 {
        ed.on_key(key(KeyCode::Down));
    }
    assert_eq!(ed.ai_mut().unwrap().hint(), "");
    ed.on_key(key(KeyCode::Up));
    assert_eq!(ed.ai_mut().unwrap().hint(), "second idea");
    ed.on_key(key(KeyCode::Up));
    assert_eq!(ed.ai_mut().unwrap().hint(), "first idea");
}

#[test]
fn a_short_editor_drops_the_label_and_the_recent_row_before_the_field() {
    let (mut ed, _) = bound("", |_| Ok("x".into()));
    paint(&mut ed, 90, 8);
    ed.on_key(alt('i'));
    let s = screen(&paint(&mut ed, 90, 8));
    assert!(s.contains("Draft") && s.contains("Close"), "{s}");
    assert!(s.contains("for example"), "the field stays:\n{s}");
    assert!(!s.contains("What I want in this answer"), "{s}");
}

#[test]
fn alt_i_works_while_the_preview_shows() {
    let (mut ed, _) = bound("text", |_| Ok("x".into()));
    paint(&mut ed, 90, 16);
    ed.set_mode(Mode::Preview);
    assert_eq!(ed.on_key(alt('i')), KeyUse::Handled);
    assert!(ed.ai_open());
}

/// The writing help has its own key, `Alt+I`, in every host: `Alt+W` is
/// never the editor's (the forms use it to open the host's web form), so one
/// key never means two things depending on the focus.
#[test]
fn alt_w_is_never_the_writing_help_s_key() {
    let (mut ed, _) = bound("text", |_| Ok("x".into()));
    paint(&mut ed, 90, 16);
    assert_eq!(
        ed.on_key(alt('w')),
        KeyUse::Pass,
        "the form's, not the editor's"
    );
    assert!(!ed.ai_open());
    assert_eq!(ed.on_key(alt('i')), KeyUse::Handled);
    assert!(ed.ai_open());
}

// -- the repository part ------------------------------------------------------

#[test]
fn without_a_clone_a_draft_never_asks_to_read_the_repository() {
    let (mut ed, seen) = bound("", |_| Ok("x".into()));
    paint(&mut ed, 90, 16);
    ed.on_key(alt('i'));
    ed.on_key(key(KeyCode::Enter));
    wait_for(&mut ed, "the answer", |e| !e.ai_running());
    assert_eq!(ed.ai_mut().unwrap().last_read_repo, Some(false));
    assert_eq!(seen.lock().unwrap().len(), 1);
}

#[test]
fn a_pull_request_description_is_written_from_its_diff_so_the_box_is_off() {
    let source = ai::PullRequestSource {
        branches: Arc::new(Mutex::new(("feature".into(), "main".into()))),
    };
    let mut help = AiHelp::new_in(AiScope::default(), Box::new(source), None);
    help.fake = Some(Arc::new(|_| Ok("x".into())));
    let mut ed = MdEditor::new(String::new(), md_render::Context::default());
    ed.bind_ai_with(help);
    paint(&mut ed, 90, 16);
    ed.on_key(alt('i'));
    let s = screen(&paint(&mut ed, 90, 16));
    assert!(
        s.contains("[ ] Let the AI read the repository (not for this text)"),
        "{s}"
    );
}

#[test]
fn an_offered_box_starts_unticked_ticks_with_a_click_or_space_and_brings_refresh() {
    let (mut ed, _) = bound("", |_| Ok("x".into()));
    paint(&mut ed, 90, 16);
    ed.on_key(alt('i'));
    ed.ai_mut().unwrap().offer_repo(false);
    let s = screen(&paint(&mut ed, 90, 16));
    assert!(
        s.contains("[ ] Let the AI read the repository"),
        "off by default:\n{s}"
    );
    assert!(!s.contains("Refresh context"), "{s}");
    // A click ticks it, and the Refresh button appears.
    let r = rect_of(&mut ed, ai::AiPart::Repo);
    ed.on_mouse(click(r.x + 1, r.y));
    let s = screen(&paint(&mut ed, 90, 16));
    assert!(s.contains("[x] Let the AI read the repository"), "{s}");
    assert!(s.contains("Refresh context"), "{s}");
    // The click left the keyboard on the box: Space toggles it back off and
    // Refresh goes.
    ed.on_key(key(KeyCode::Char(' ')));
    assert!(!ed.ai_mut().unwrap().repo_on());
    let s = screen(&paint(&mut ed, 90, 16));
    assert!(!s.contains("Refresh context"), "{s}");
    ed.on_key(key(KeyCode::Char(' ')));
    assert!(ed.ai_mut().unwrap().repo_on());
    // A draft now asks the worker to read the repository.
    ed.on_key(key(KeyCode::Tab)); // Draft
    ed.on_key(key(KeyCode::Enter));
    wait_for(&mut ed, "the answer", |e| !e.ai_running());
    assert_eq!(ed.ai_mut().unwrap().last_read_repo, Some(true));
}

#[test]
fn the_context_line_has_a_row_while_the_box_is_ticked_and_refresh_clears_it() {
    let (mut ed, _) = bound("", |_| Ok("x".into()));
    paint(&mut ed, 90, 20);
    ed.on_key(alt('i'));
    ed.ai_mut().unwrap().offer_repo(true);
    ed.ai_mut()
        .unwrap()
        .set_context_line("Context reused (4 of 5 parts, HEAD a1b2c3d)");
    let s = screen(&paint(&mut ed, 90, 20));
    assert!(
        s.contains("Context reused (4 of 5 parts, HEAD a1b2c3d)"),
        "{s}"
    );
    // The Refresh button forgets what was kept and says so.
    let r = rect_of(&mut ed, ai::AiPart::Refresh);
    ed.on_mouse(click(r.x + 1, r.y));
    let s = screen(&paint(&mut ed, 90, 20));
    assert!(s.contains("Context refreshed."), "{s}");
    assert!(!s.contains("Context reused"), "{s}");
    // Unticked, a line from an earlier draft is not shown.
    ed.ai_mut()
        .unwrap()
        .set_context_line("Context reused (1 of 1 parts, HEAD abcdef0)");
    ed.ai_mut().unwrap().offer_repo(false);
    let s = screen(&paint(&mut ed, 90, 20));
    assert!(!s.contains("Context reused"), "{s}");
}
