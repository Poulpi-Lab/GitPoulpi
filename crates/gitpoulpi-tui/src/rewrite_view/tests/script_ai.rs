//! ✦ Write a script with AI (slice S12, TUI): the button, the dialog that
//! shows what is sent, the sample that is off until ticked, and the answer
//! that lands in the editor marked and unapplied. The window's side (the
//! worker, the model) is put in by hand: here the dialog's wishes are read
//! and answered.

use super::render::{paint, rect_of};
use super::script::{script_text, script_view};
use super::*;
use crate::rewrite_view::{AiSystem, AiWish, SampleRow};
use crate::th;

const W: u16 = 140;
const H: u16 = 44;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn type_text(v: &mut RewriteView, text: &str) {
    for c in text.chars() {
        v.on_key(key(KeyCode::Char(c)));
    }
}

fn rows() -> Vec<SampleRow> {
    vec![
        SampleRow {
            author_name: "Ada Lovelace".into(),
            author_email: "ada@secret.example".into(),
            timestamp: 1_700_000_000,
            subject: "fix: the parser".into(),
        },
        SampleRow {
            author_name: "Bob Builder".into(),
            author_email: "bob@secret.example".into(),
            timestamp: 1_700_086_400,
            subject: "feat: login".into(),
        },
    ]
}

/// A Script editor with AI on, a sample to offer, and the dialog open and
/// described by the window.
fn open_dialog() -> (RewriteView, RawQueue, tempfile::TempDir) {
    let (mut v, q, t) = script_view();
    v.scope_ctx.sample = rows();
    v.set_ai(true, false);
    v.open_ai_dialog();
    assert_eq!(v.take_ai_wish(), Some(AiWish::Describe));
    v.set_ai_system(Ok(AiSystem {
        prompt: "You write one Lua script.\nCONTRACT-LINE-TWO".into(),
        via: "Fake agent".into(),
    }));
    (v, q, t)
}

fn screen(v: &RewriteView) -> String {
    paint(v, W, H, None).0
}

#[test]
fn with_ai_off_the_editor_has_no_ai_button_and_the_dialog_will_not_open() {
    let (mut v, _q, _t) = script_view();
    assert!(!screen(&v).contains("Write with AI"), "{}", screen(&v));
    v.open_ai_dialog();
    assert!(v.script.ai.dialog.is_none());
    assert_eq!(v.take_ai_wish(), None, "nothing is asked of the window");
    assert!(
        v.script
            .editor
            .as_ref()
            .and_then(|e| e.note.as_ref())
            .is_some_and(|n| n.text.contains("turned off")),
        "it says why"
    );
}

#[test]
fn the_button_carries_the_sparkle_in_the_ai_purple_and_opens_the_dialog() {
    let (mut v, _q, _t) = script_view();
    v.set_ai(true, false);
    let (text, hits, buf) = paint(&v, W, H, None);
    assert!(text.contains("\u{2726} Write with AI..."), "{text}");
    let r = rect_of(&hits, RewriteHit::ScriptAi);
    let spark = (r.x..r.right())
        .find(|x| buf[(*x, r.y)].symbol() == crate::theme::AI_SPARK)
        .expect("the sparkle is on the button");
    assert_eq!(buf[(spark, r.y)].fg, th().status.ai);
    // Nothing was asked of a model by showing it.
    assert_eq!(v.take_ai_wish(), None);
    v.on_press(RewriteHit::ScriptAi, r.x + 2, r.y);
    assert!(v.script.ai.dialog.is_some());
    assert_eq!(v.take_ai_wish(), Some(AiWish::Describe), "only describes");
}

#[test]
fn a_running_request_greys_the_button() {
    let (mut v, _q, _t) = script_view();
    v.set_ai(true, true);
    v.open_ai_dialog();
    assert!(v.script.ai.dialog.is_none());
    assert_eq!(v.take_ai_wish(), None);
}

#[test]
#[cfg(feature = "ai")]
fn the_dialog_shows_exactly_what_is_sent_and_the_sample_is_off() {
    let (mut v, _q, _t) = open_dialog();
    type_text(&mut v, "move night commits");
    v.on_key(KeyEvent::new(KeyCode::Char('w'), KeyModifiers::ALT));
    let s = screen(&v);
    assert!(s.contains("What is sent"), "{s}");
    assert!(s.contains("CONTRACT-LINE-TWO"), "the system prompt\n{s}");
    assert!(s.contains("What the script must do:"), "{s}");
    assert!(s.contains("move night commits"), "the request\n{s}");
    assert!(s.contains("Sent to: Fake agent"), "{s}");
    assert!(
        s.contains("[ ] Send a sample of 2 commits"),
        "off by default\n{s}"
    );
    for secret in ["Ada", "secret.example", "fix: the parser"] {
        assert!(!s.contains(secret), "{secret:?} is shown as sent\n{s}");
    }
    // Ticking it shows the headers that would go, and only those.
    let hits = paint(&v, W, H, None).1;
    let r = rect_of(&hits, RewriteHit::AiSample);
    v.on_press(RewriteHit::AiSample, r.x, r.y);
    let s = screen(&v);
    assert!(s.contains("[x] Send a sample of 2 commits"), "{s}");
    assert!(s.contains("Ada Lovelace <ada@secret.example>"), "{s}");
    assert!(s.contains("2023-11-14 22:13:20 UTC"), "{s}");
    assert!(s.contains("feat: login"), "{s}");
}

#[test]
fn send_asks_the_window_with_no_commit_data_unless_the_box_was_ticked() {
    let (mut v, _q, _t) = open_dialog();
    // Nothing typed: Send is refused, nothing is asked.
    let hits = paint(&v, W, H, None).1;
    let send = rect_of(&hits, RewriteHit::AiSend);
    v.on_press(RewriteHit::AiSend, send.x + 1, send.y);
    assert_eq!(v.take_ai_wish(), None);
    assert!(screen(&v).contains("Say what the script must do."));

    let field = rect_of(&hits, RewriteHit::AiText);
    v.on_press(RewriteHit::AiText, field.x + 1, field.y);
    type_text(&mut v, "  replace my old email  ");
    let hits = paint(&v, W, H, None).1;
    let send = rect_of(&hits, RewriteHit::AiSend);
    v.on_press(RewriteHit::AiSend, send.x + 1, send.y);
    assert_eq!(
        v.take_ai_wish(),
        Some(AiWish::Send {
            instruction: "replace my old email".into(),
            sample: Vec::new(),
        }),
        "the sample is empty while the box is not ticked"
    );
    assert!(v.script.ai.dialog.as_ref().unwrap().working);
}

#[test]
fn with_the_box_ticked_the_request_carries_the_sample() {
    let (mut v, _q, _t) = open_dialog();
    type_text(&mut v, "x");
    // Keyboard: Tab to the box, Space ticks it, Tab to the pane and Send.
    v.on_key(key(KeyCode::Tab));
    v.on_key(key(KeyCode::Char(' ')));
    v.on_key(key(KeyCode::Tab));
    v.on_key(key(KeyCode::Tab));
    v.on_key(key(KeyCode::Enter));
    match v.take_ai_wish() {
        Some(AiWish::Send { sample, .. }) => assert_eq!(sample, rows()),
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_answer_lands_in_the_editor_marked_and_nothing_is_applied() {
    let (mut v, _q, _t) = open_dialog();
    type_text(&mut v, "x");
    v.on_key(key(KeyCode::Enter));
    v.on_key(key(KeyCode::Enter));
    assert!(matches!(v.take_ai_wish(), Some(AiWish::Send { .. })));
    let script = "--! name: AI\n--! kinds: commit\nfunction rewrite_commit(c, ctx)\nend\n";
    v.put_ai_script(script.to_string(), false);
    assert!(v.script.ai.dialog.is_none(), "the dialog closes");
    assert_eq!(script_text(&v), script);
    let s = screen(&v);
    assert!(
        s.contains("\u{2726} written by AI: read it before Preview"),
        "{s}"
    );
    assert!(s.contains("The AI wrote a script."), "{s}");
    // No preview, no apply, no test: the view asked the daemon for nothing
    // but the live check.
    assert!(!matches!(v.run, Run::Previewing { .. } | Run::Done(_)));
    assert!(matches!(
        v.script.test,
        crate::rewrite_view::script::TestState::Idle
    ));
    // The first edit clears the mark: the text is the user's now.
    v.script_focus(ScriptFocus::Editor);
    v.on_key(key(KeyCode::Char('x')));
    assert!(!screen(&v).contains("written by AI"), "{}", screen(&v));
}

#[test]
fn a_failed_or_canceled_request_comes_back_to_the_dialog_with_why() {
    let (mut v, _q, _t) = open_dialog();
    type_text(&mut v, "x");
    v.on_key(key(KeyCode::Enter));
    v.on_key(key(KeyCode::Enter));
    assert!(matches!(v.take_ai_wish(), Some(AiWish::Send { .. })));
    // Working: Esc cancels the request, it does not close the workbench.
    assert_eq!(v.on_key(key(KeyCode::Esc)), RewriteKeyOutcome::Stay);
    assert_eq!(v.take_ai_wish(), Some(AiWish::Cancel));
    v.ai_stopped("AI: it did not answer".into());
    let s = screen(&v);
    assert!(s.contains("AI: it did not answer"), "{s}");
    assert!(!v.script.ai.dialog.as_ref().unwrap().working, "can retry");
    // Esc again closes the dialog, the editor stays.
    v.on_key(key(KeyCode::Esc));
    assert!(v.script.ai.dialog.is_none());
    assert!(v.script.editor.is_some());
}

#[test]
fn the_rule_is_not_replaced_by_closing_the_dialog() {
    let (mut v, _q, _t) = open_dialog();
    let before = script_text(&v);
    v.on_key(key(KeyCode::Esc));
    assert!(v.script.ai.dialog.is_none());
    assert_eq!(script_text(&v), before);
    assert_eq!(v.take_ai_wish(), None, "nothing was sent");
}

// ----------------------------------------------- the "What is sent" pane

fn alt(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::ALT)
}

/// A contract of `n` numbered lines, so the open pane must scroll.
fn long_contract(v: &mut RewriteView, n: usize) {
    let prompt = (0..n)
        .map(|i| format!("CONTRACT-LINE-{i:03}"))
        .collect::<Vec<_>>()
        .join("\n");
    v.set_ai_system(Ok(AiSystem {
        prompt,
        via: "Fake agent".into(),
    }));
}

#[test]
#[cfg(feature = "ai")]
fn what_is_sent_is_collapsed_by_default_and_the_dialog_is_shorter_for_it() {
    let (mut v, _q, _t) = open_dialog();
    long_contract(&mut v, 60);
    let (text, hits, _) = paint(&v, W, H, None);
    assert!(text.contains("\u{25B8} What is sent"), "{text}");
    assert!(text.contains("Alt+W: show or hide"), "{text}");
    assert!(!text.contains("CONTRACT-LINE-000"), "{text}");
    assert!(!hits.iter().any(|(_, h)| *h == RewriteHit::AiSent));
    assert!(!hits.iter().any(|(_, h)| *h == RewriteHit::AiSentBar));
    let collapsed = rect_of(&hits, RewriteHit::AiPanel).height;
    v.on_key(alt('w'));
    let (text, hits, _) = paint(&v, W, H, None);
    assert!(text.contains("\u{25BE} What is sent"), "{text}");
    assert!(text.contains("CONTRACT-LINE-000"), "{text}");
    let open = rect_of(&hits, RewriteHit::AiPanel).height;
    assert!(collapsed < open, "collapsed {collapsed} < open {open}");
    assert!(open <= H - 4, "the dialog stays on the screen");
    v.on_key(alt('w'));
    assert!(!screen(&v).contains("CONTRACT-LINE-000"));
}

#[cfg(feature = "ai")]
fn check_pane(w: u16, h: u16) {
    let (mut v, _q, _t) = open_dialog();
    long_contract(&mut v, 80);
    v.on_key(alt('w'));
    let (text, hits, buf) = paint(&v, w, h, None);
    let pane = rect_of(&hits, RewriteHit::AiSent);
    let bar = rect_of(&hits, RewriteHit::AiSentBar);
    let panel = rect_of(&hits, RewriteHit::AiPanel);
    assert!(pane.height >= 4, "{w}x{h}\n{text}");
    assert_eq!(bar.x, pane.right() - 1, "{w}x{h}: the pane's right edge");
    assert_eq!((bar.y, bar.height), (pane.y, pane.height), "{w}x{h}");
    assert!(bar.x < panel.right() - 1, "{w}x{h}: inside the border");
    assert_eq!(buf[(bar.x, bar.y)].symbol(), "\u{2191}", "{w}x{h}");
    assert_eq!(
        buf[(bar.x, bar.bottom() - 1)].symbol(),
        "\u{2193}",
        "{w}x{h}"
    );
    let thumb = (bar.y..bar.bottom())
        .filter(|y| buf[(bar.x, *y)].bg == th().scrollbar.thumb)
        .count();
    assert!(thumb >= 1, "{w}x{h}: a thumb is drawn\n{text}");
    for y in pane.y..pane.bottom() {
        assert_eq!(buf[(bar.x - 1, y)].symbol(), " ", "{w}x{h}: text under bar");
    }
    let tint = buf[(pane.x + 1, pane.y)].bg;
    assert_ne!(tint, th().ui.panel, "{w}x{h}");
    assert_ne!(tint, crate::input_bg(false, false), "{w}x{h}");
    assert_ne!(tint, crate::input_bg(true, false), "{w}x{h}");
    for y in pane.y..pane.bottom() {
        assert_eq!(buf[(pane.x, y)].bg, tint, "{w}x{h}: the whole pane");
    }
}

#[test]
#[cfg(feature = "ai")]
fn the_open_pane_has_its_own_scrollbar_at_a_small_and_a_large_size() {
    check_pane(60, 22);
    check_pane(90, 30);
    check_pane(200, 70);
}

#[test]
#[cfg(feature = "ai")]
fn the_keys_toggle_the_pane_and_scroll_it() {
    let (mut v, _q, _t) = open_dialog();
    long_contract(&mut v, 80);
    // Tab: the sample box, then the pane; Enter opens it.
    for _ in 0..2 {
        v.on_key(key(KeyCode::Tab));
    }
    assert!(!screen(&v).contains("CONTRACT-LINE-000"));
    v.on_key(key(KeyCode::Enter));
    assert!(screen(&v).contains("CONTRACT-LINE-000"));
    for _ in 0..3 {
        v.on_key(key(KeyCode::Down));
    }
    let s = screen(&v);
    assert!(
        !s.contains("CONTRACT-LINE-000") && s.contains("CONTRACT-LINE-002"),
        "{s}"
    );
    v.on_key(key(KeyCode::End));
    assert!(screen(&v).contains("CONTRACT-LINE-079"));
    v.on_key(key(KeyCode::Home));
    assert!(screen(&v).contains("CONTRACT-LINE-000"));
    v.on_key(key(KeyCode::Enter));
    assert!(!screen(&v).contains("CONTRACT-LINE-000"));
}

#[test]
#[cfg(feature = "ai")]
fn the_mouse_toggles_wheels_and_drags_the_pane_scrollbar() {
    use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
    let (mut v, _q, _t) = open_dialog();
    long_contract(&mut v, 80);
    let (_, hits, _) = paint(&v, W, H, None);
    let head = rect_of(&hits, RewriteHit::AiSentHead);
    v.on_press(RewriteHit::AiSentHead, head.x + 3, head.y);
    assert!(screen(&v).contains("CONTRACT-LINE-000"));
    v.on_ai_wheel(3);
    let s = screen(&v);
    assert!(
        !s.contains("CONTRACT-LINE-000") && s.contains("CONTRACT-LINE-003"),
        "{s}"
    );
    let (_, hits, buf) = paint(&v, W, H, None);
    let bar = rect_of(&hits, RewriteHit::AiSentBar);
    let thumb_y = (bar.y..bar.bottom())
        .find(|y| buf[(bar.x, *y)].bg == th().scrollbar.thumb)
        .expect("a thumb");
    v.on_press(RewriteHit::AiSentBar, bar.x, thumb_y);
    let mouse = |kind, row| MouseEvent {
        kind,
        column: bar.x,
        row,
        modifiers: KeyModifiers::NONE,
    };
    let below = bar.bottom() + 3;
    v.on_ai_bar_mouse(mouse(MouseEventKind::Drag(MouseButton::Left), below));
    v.on_ai_bar_mouse(mouse(MouseEventKind::Up(MouseButton::Left), below));
    assert!(screen(&v).contains("CONTRACT-LINE-079"));
    v.on_press(RewriteHit::AiSentBar, bar.x, bar.y + 1);
    v.on_ai_bar_mouse(mouse(MouseEventKind::Up(MouseButton::Left), bar.y + 1));
    assert!(!screen(&v).contains("CONTRACT-LINE-079"));
}
