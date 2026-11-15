//! Buffer tests of the Markdown editor: the three modes, what the Preview
//! shows, the keys, the worker for long texts and the drafts.

use super::*;
use crossterm::event::{KeyModifiers, MouseEvent};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn alt(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::ALT)
}

fn editor(text: &str) -> MdEditor {
    MdEditor::new(text.to_string(), md_render::Context::default())
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

fn rows(buf: &Buffer) -> Vec<String> {
    (0..buf.area.height)
        .map(|y| {
            (0..buf.area.width)
                .map(|x| buf[(x, y)].symbol().to_string())
                .collect::<String>()
        })
        .collect()
}

fn screen(buf: &Buffer) -> String {
    rows(buf).join("\n")
}

fn click(col: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: col,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

const SAMPLE: &str = "# Plan\n\nSome **bold** and `code`.\n\n- first\n- second\n";

#[test]
fn write_mode_shows_the_raw_text_and_the_three_mode_buttons() {
    let mut ed = editor(SAMPLE);
    let s = screen(&paint(&mut ed, 60, 12));
    for want in ["Write", "Preview", "Split", "# Plan", "**bold**"] {
        assert!(s.contains(want), "{want:?} missing:\n{s}");
    }
}

#[test]
fn preview_mode_shows_the_markdown_rendered() {
    let mut ed = editor(SAMPLE);
    ed.set_mode(Mode::Preview);
    let s = screen(&paint(&mut ed, 60, 12));
    assert!(s.contains("Plan"), "{s}");
    assert!(!s.contains("# Plan"), "the heading marker is gone:\n{s}");
    assert!(
        s.contains("Some bold and code."),
        "inline marks are gone:\n{s}"
    );
    assert!(!s.contains("**"), "{s}");
    assert!(s.contains("\u{2022} first"), "a list becomes bullets:\n{s}");
}

#[test]
fn preview_of_an_empty_text_says_so() {
    let mut ed = editor("");
    ed.set_mode(Mode::Preview);
    let s = screen(&paint(&mut ed, 60, 10));
    assert!(s.contains(msg::PREVIEW_EMPTY.get()), "{s}");
}

#[test]
fn alt_p_flips_between_write_and_preview() {
    let mut ed = editor(SAMPLE);
    paint(&mut ed, 60, 12);
    assert_eq!(ed.on_key(alt('p')), KeyUse::Handled);
    assert_eq!(ed.mode(), Mode::Preview);
    assert_eq!(ed.on_key(alt('p')), KeyUse::Handled);
    assert_eq!(ed.mode(), Mode::Write);
}

#[test]
fn split_is_refused_in_a_narrow_editor_and_says_why() {
    let mut ed = editor(SAMPLE);
    paint(&mut ed, SPLIT_MIN_WIDTH - 1, 12);
    ed.on_key(alt('l'));
    assert_eq!(ed.mode(), Mode::Write);
    let s = screen(&paint(&mut ed, SPLIT_MIN_WIDTH - 1, 12));
    assert!(s.contains(msg::SPLIT_NEEDS_WIDTH.get()), "{s}");
}

#[test]
fn split_shows_the_text_and_its_preview_side_by_side() {
    let mut ed = editor(SAMPLE);
    paint(&mut ed, SPLIT_MIN_WIDTH, 12);
    ed.on_key(alt('l'));
    assert_eq!(ed.mode(), Mode::Split);
    let buf = paint(&mut ed, SPLIT_MIN_WIDTH, 12);
    let heading_row = rows(&buf)
        .into_iter()
        .find(|r| r.contains("# Plan"))
        .expect("the raw heading on the left");
    // The same row carries the rendered heading on the right.
    let after = heading_row.split("# Plan").nth(1).unwrap();
    assert!(after.contains("Plan"), "{heading_row}");
    // Alt+L again leaves Split.
    ed.on_key(alt('l'));
    assert_eq!(ed.mode(), Mode::Write);
}

#[test]
fn a_split_editor_that_shrinks_falls_back_to_write() {
    let mut ed = editor(SAMPLE);
    paint(&mut ed, SPLIT_MIN_WIDTH, 12);
    ed.set_mode(Mode::Split);
    paint(&mut ed, 70, 12);
    assert_eq!(ed.mode(), Mode::Write);
}

#[test]
fn enter_is_a_newline_and_the_send_keys_submit() {
    let mut ed = editor("");
    paint(&mut ed, 60, 10);
    ed.on_key(key(KeyCode::Char('a')));
    ed.on_key(key(KeyCode::Enter));
    ed.on_key(key(KeyCode::Char('b')));
    assert_eq!(ed.text(), "a\nb");
    for k in [
        KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL),
        KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL),
        KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT),
    ] {
        assert_eq!(ed.on_key(k), KeyUse::Submit, "{k:?}");
    }
    assert_eq!(ed.text(), "a\nb", "sending does not touch the text");
}

#[test]
fn tab_and_escape_are_left_to_the_form() {
    let mut ed = editor("x");
    paint(&mut ed, 60, 10);
    assert_eq!(ed.on_key(key(KeyCode::Tab)), KeyUse::Pass);
    assert_eq!(ed.on_key(key(KeyCode::Esc)), KeyUse::Pass);
    ed.set_mode(Mode::Preview);
    assert_eq!(ed.on_key(key(KeyCode::Tab)), KeyUse::Pass);
    // Typing in the Preview changes nothing.
    assert_eq!(ed.on_key(key(KeyCode::Char('z'))), KeyUse::Pass);
    assert_eq!(ed.text(), "x");
}

#[test]
fn the_mode_buttons_are_real_buttons() {
    let mut ed = editor(SAMPLE);
    paint(&mut ed, 90, 12);
    let rect_of = |ed: &MdEditor, part: Part| {
        ed.hits
            .iter()
            .find(|(_, p)| *p == part)
            .map(|(r, _)| *r)
            .unwrap()
    };
    let preview = rect_of(&ed, Part::Preview);
    assert_eq!(
        ed.on_mouse(click(preview.x + 1, preview.y)),
        MouseUse::Inside
    );
    assert_eq!(ed.mode(), Mode::Preview);
    paint(&mut ed, 90, 12);
    let write = rect_of(&ed, Part::Write);
    ed.on_mouse(click(write.x + 1, write.y));
    assert_eq!(ed.mode(), Mode::Write);
    // A click on the disabled Split says why instead of switching.
    paint(&mut ed, 90, 12);
    let split = rect_of(&ed, Part::Split);
    ed.on_mouse(click(split.x + 1, split.y));
    assert_eq!(ed.mode(), Mode::Write);
    assert!(paint_text(&mut ed).contains(msg::SPLIT_NEEDS_WIDTH.get()));
}

fn paint_text(ed: &mut MdEditor) -> String {
    screen(&paint(ed, 90, 12))
}

#[test]
fn a_click_outside_is_not_the_editors() {
    let mut ed = editor(SAMPLE);
    let mut buf = Buffer::empty(Rect::new(0, 0, 80, 20));
    ed.render(&mut buf, Rect::new(0, 5, 60, 12), true, "");
    assert_eq!(ed.on_mouse(click(70, 1)), MouseUse::Outside);
    assert_eq!(ed.on_mouse(click(5, 8)), MouseUse::Inside);
}

#[test]
fn the_ai_slot_is_reserved_and_empty() {
    let mut ed = editor(SAMPLE);
    let buf = paint(&mut ed, 80, 12);
    let slot = ed.ai_slot();
    assert!(slot.width > 0 && slot.right() == 80);
    for x in slot.x..slot.right() {
        assert_eq!(buf[(x, slot.y)].symbol(), " ", "x={x}");
    }
}

#[test]
fn the_preview_scrolls_with_the_keys() {
    let long: String = (1..=60).map(|n| format!("line {n}\n\n")).collect();
    let mut ed = editor(&long);
    ed.set_mode(Mode::Preview);
    let first = screen(&paint(&mut ed, 60, 12));
    assert!(first.contains("line 1"));
    assert!(!first.contains("line 40"));
    for _ in 0..4 {
        ed.on_key(key(KeyCode::PageDown));
    }
    let later = screen(&paint(&mut ed, 60, 12));
    assert!(!later.contains("line 1\n"), "{later}");
    ed.on_key(key(KeyCode::Home));
    assert!(screen(&paint(&mut ed, 60, 12)).contains("line 1"));
    ed.on_key(key(KeyCode::End));
    assert!(screen(&paint(&mut ed, 60, 12)).contains("line 60"));
}

#[test]
fn a_long_text_is_laid_out_on_a_worker_and_never_blocks_the_paint() {
    let long = format!(
        "# Big heading\n\n{}",
        "word ".repeat(preview::SYNC_LIMIT / 4)
    );
    assert!(long.len() > preview::SYNC_LIMIT);
    let mut ed = editor(&long);
    ed.set_mode(Mode::Preview);
    let started = Instant::now();
    let first = screen(&paint(&mut ed, 60, 12));
    // The first paint did not wait for the layout (the debounce alone is
    // longer than any paint).
    assert!(
        started.elapsed() < preview::DEBOUNCE,
        "{:?}",
        started.elapsed()
    );
    assert!(!first.contains("Big heading"), "{first}");
    assert!(first.contains(msg::PREVIEW_UPDATING.get()), "{first}");
    let deadline = Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let s = screen(&paint(&mut ed, 60, 12));
        if s.contains("Big heading") {
            assert!(!s.contains(msg::PREVIEW_UPDATING.get()), "{s}");
            assert!(!s.contains("# Big"), "{s}");
            break;
        }
        assert!(Instant::now() < deadline, "the worker never answered");
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

#[test]
fn the_preview_resolves_the_hosts_references() {
    let ctx = md_render::Context {
        xrefs: gitpoulpi_hosting::xrefs::XrefContext {
            family: Some(gitpoulpi_config::ApiFamily::Github),
            web_base: "https://github.com".into(),
            project: "o/r".into(),
            ..Default::default()
        },
    };
    let doc = md_render::render("see #7 and @mara", 60, &ctx);
    let urls: Vec<&str> = doc.links.iter().map(|l| l.url.as_str()).collect();
    assert!(
        urls.contains(&"https://github.com/o/r/issues/7"),
        "{urls:?}"
    );
    assert!(urls.iter().any(|u| u.ends_with("/mara")), "{urls:?}");
}

#[test]
fn a_converted_text_says_how_it_is_sent() {
    let mut ed = editor("hello");
    ed.set_sent_as(Some("Jira wiki markup".into()));
    ed.set_mode(Mode::Preview);
    let s = screen(&paint(&mut ed, 60, 10));
    assert!(s.contains("Sent as Jira wiki markup"), "{s}");
}

#[test]
fn drafts_are_kept_per_target_and_removed_once_sent() {
    let dir = tempfile::tempdir().unwrap();
    let key = draft::key(&["h", "o/p", "issue#3", "reply"]);
    {
        let mut ed = editor("").with_draft_in(key.clone(), Some(dir.path().to_path_buf()));
        ed.paste("half a thought");
    } // dropped: written
    let mut again = editor("").with_draft_in(key.clone(), Some(dir.path().to_path_buf()));
    assert_eq!(again.text(), "half a thought");
    // Another target has none.
    let other = editor("").with_draft_in(
        draft::key(&["h", "o/p", "issue#4", "reply"]),
        Some(dir.path().to_path_buf()),
    );
    assert_eq!(other.text(), "");
    // Sent: the draft is gone and dropping does not bring it back.
    again.sent();
    drop(again);
    let third = editor("").with_draft_in(key, Some(dir.path().to_path_buf()));
    assert_eq!(third.text(), "");
}

#[test]
fn a_draft_is_written_after_the_typing_pauses() {
    let dir = tempfile::tempdir().unwrap();
    let key = draft::key(&["h", "p", "n"]);
    let mut ed = editor("").with_draft_in(key.clone(), Some(dir.path().to_path_buf()));
    ed.paste("typing");
    paint(&mut ed, 60, 10);
    assert_eq!(draft::load(dir.path(), &key), None, "not yet: still typing");
    std::thread::sleep(draft::IDLE + std::time::Duration::from_millis(50));
    paint(&mut ed, 60, 10);
    assert_eq!(draft::load(dir.path(), &key).as_deref(), Some("typing"));
    // Throwing the text away removes it.
    ed.discard();
    assert_eq!(draft::load(dir.path(), &key), None);
}

#[test]
fn an_editor_with_text_does_not_take_a_draft_over_it() {
    let dir = tempfile::tempdir().unwrap();
    let key = draft::key(&["k"]);
    draft::save(dir.path(), &key, "old draft");
    let ed = editor("edit me").with_draft_in(key, Some(dir.path().to_path_buf()));
    assert_eq!(ed.text(), "edit me");
}
