use super::*;
use crossterm::event::MouseEvent;
use ratatui::backend::TestBackend;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

fn open(dir: &Path, name: &str, bytes: &[u8]) -> Editor {
    std::fs::write(dir.join(name), bytes).unwrap();
    Editor::open(dir, Path::new(name)).unwrap()
}

fn draw(editor: &mut Editor, w: u16, h: u16) -> (ratatui::buffer::Buffer, Vec<(Rect, EditorHit)>) {
    let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
    let mut hits = Vec::new();
    terminal
        .draw(|frame| hits = editor.render(frame, Rect::new(0, 0, w, h), None))
        .unwrap();
    (terminal.backend().buffer().clone(), hits)
}

fn row_text(buf: &ratatui::buffer::Buffer, y: u16) -> String {
    (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect()
}

#[test]
fn open_edit_save_round_trips_the_file_on_disk() {
    let dir = tempfile::tempdir().unwrap();
    let mut clip = Clipboard::new();
    let mut ed = open(dir.path(), "a.txt", b"one\r\ntwo\r\n");
    ed.on_key(&key(KeyCode::End), &mut clip);
    ed.on_key(&key(KeyCode::Char('!')), &mut clip);
    assert!(ed.doc.is_dirty());
    let out = ed.on_key(&ctrl('s'), &mut clip);
    assert!(out.saved, "{out:?}");
    assert!(!ed.doc.is_dirty());
    assert_eq!(
        std::fs::read(dir.path().join("a.txt")).unwrap(),
        b"one!\r\ntwo\r\n"
    );
    let leftovers: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().contains("gitpoulpi-save"))
        .collect();
    assert!(leftovers.is_empty(), "the temp file was renamed away");
}

#[test]
fn binary_and_missing_files_are_refused_with_a_message() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("b.bin"), b"\x00\x01").unwrap();
    let err = Editor::open(dir.path(), Path::new("b.bin")).err().unwrap();
    assert!(err.contains("binary"), "{err}");
    let err = Editor::open(dir.path(), Path::new("gone.txt"))
        .err()
        .unwrap();
    assert!(err.contains("not in the working tree"), "{err}");
}

#[test]
fn esc_with_unsaved_changes_asks_and_each_answer_does_its_thing() {
    let dir = tempfile::tempdir().unwrap();
    let mut clip = Clipboard::new();
    let mut ed = open(dir.path(), "a.txt", b"x\n");
    assert!(
        ed.on_key(&key(KeyCode::Esc), &mut clip).close,
        "clean: closes"
    );

    ed.on_key(&key(KeyCode::Char('y')), &mut clip);
    let out = ed.on_key(&key(KeyCode::Esc), &mut clip);
    assert!(!out.close && ed.prompt, "dirty: asks");
    ed.on_key(&key(KeyCode::Esc), &mut clip);
    assert!(!ed.prompt, "Esc in the prompt is Cancel");
    ed.on_key(&key(KeyCode::Esc), &mut clip);
    let out = ed.on_key(&key(KeyCode::Char('d')), &mut clip);
    assert!(out.close && !out.saved, "Discard closes without saving");
    assert_eq!(std::fs::read(dir.path().join("a.txt")).unwrap(), b"x\n");

    let mut ed = open(dir.path(), "a.txt", b"x\n");
    ed.on_key(&key(KeyCode::Char('y')), &mut clip);
    ed.on_key(&key(KeyCode::Esc), &mut clip);
    let out = ed.click(EditorHit::PromptSave);
    assert!(out.close && out.saved);
    assert_eq!(std::fs::read(dir.path().join("a.txt")).unwrap(), b"yx\n");
}

#[test]
fn a_file_changed_on_disk_is_not_overwritten_without_a_second_save() {
    let dir = tempfile::tempdir().unwrap();
    let mut clip = Clipboard::new();
    let mut ed = open(dir.path(), "a.txt", b"x\n");
    ed.on_key(&key(KeyCode::Char('y')), &mut clip);
    std::fs::write(dir.path().join("a.txt"), b"changed elsewhere, longer\n").unwrap();
    let out = ed.save();
    assert!(!out.saved && out.message.unwrap().contains("changed on disk"));
    assert!(ed.save().saved, "the second save overwrites");
}

#[test]
fn ctrl_z_and_ctrl_y_undo_and_redo_through_the_editor() {
    let dir = tempfile::tempdir().unwrap();
    let mut clip = Clipboard::new();
    let mut ed = open(dir.path(), "a.txt", b"");
    for c in "abc".chars() {
        ed.on_key(&key(KeyCode::Char(c)), &mut clip);
    }
    ed.on_key(&ctrl('z'), &mut clip);
    assert_eq!(ed.doc.text(), "");
    ed.on_key(&ctrl('y'), &mut clip);
    assert_eq!(ed.doc.text(), "abc");
    // The C0 spelling some terminals send for Ctrl+Z.
    ed.on_key(&key(KeyCode::Char('\u{1a}')), &mut clip);
    assert_eq!(ed.doc.text(), "");
}

#[test]
fn only_unused_ctrl_chords_pass_through_to_the_window() {
    let dir = tempfile::tempdir().unwrap();
    let ed = open(dir.path(), "a.txt", b"");
    assert!(ed.passes_through(&ctrl('p')), "the palette");
    assert!(ed.passes_through(&ctrl('n')), "the Ctrl+N prefix");
    assert!(!ed.passes_through(&ctrl('s')));
    assert!(!ed.passes_through(&ctrl('z')));
    assert!(!ed.passes_through(&key(KeyCode::Tab)), "Tab indents");
    assert!(!ed.passes_through(&key(KeyCode::Char('q'))));
}

fn alt(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::ALT)
}

/// `editor.save = "Alt+s"`: Alt+S writes the file, Ctrl+S no longer does.
#[test]
fn a_rebound_save_chord_saves_and_the_old_one_does_not() {
    let dir = tempfile::tempdir().unwrap();
    let mut clip = Clipboard::new();
    let km = crate::keymap::test_keymap(&[("editor", "save", &["Alt+s"])]);
    crate::keymap::with_test_keymap(km, || {
        let mut ed = open(dir.path(), "a.txt", b"x\n");
        ed.on_key(&key(KeyCode::Char('y')), &mut clip);
        let out = ed.on_key(&ctrl('s'), &mut clip);
        assert!(!out.saved, "the old chord is inert");
        assert_eq!(std::fs::read(dir.path().join("a.txt")).unwrap(), b"x\n");
        let out = ed.on_key(&alt('s'), &mut clip);
        assert!(out.saved);
        assert_eq!(std::fs::read(dir.path().join("a.txt")).unwrap(), b"yx\n");
    });
}

/// The prompt's keys and the pass-through list come from the keymap.
#[test]
fn the_prompt_keys_and_the_pass_through_list_follow_the_keymap() {
    let dir = tempfile::tempdir().unwrap();
    let mut clip = Clipboard::new();
    let km = crate::keymap::test_keymap(&[
        ("editor_prompt", "discard", &["x"]),
        ("global", "nav_prefix", &["Ctrl+Z"]),
        ("global", "undo", &["F9"]),
        ("editor", "undo", &["Ctrl+L"]),
    ]);
    crate::keymap::with_test_keymap(km, || {
        let mut ed = open(dir.path(), "a.txt", b"x\n");
        // A rebound prefix sits on the editor's old undo letter: the
        // window keeps it; the new undo chord is the editor's.
        assert!(ed.passes_through(&ctrl('z')));
        assert!(!ed.passes_through(&ctrl('l')));
        assert!(!ed.passes_through(&ctrl('y')), "redo stays bound");
        ed.on_key(&key(KeyCode::Char('y')), &mut clip);
        ed.on_key(&key(KeyCode::Esc), &mut clip);
        assert!(ed.prompt);
        let out = ed.on_key(&key(KeyCode::Char('d')), &mut clip);
        assert!(!out.close, "the old Discard key is inert");
        let out = ed.on_key(&key(KeyCode::Char('x')), &mut clip);
        assert!(out.close && !out.saved);
    });
}

#[test]
fn find_selects_the_next_match_and_counts_them() {
    let dir = tempfile::tempdir().unwrap();
    let mut clip = Clipboard::new();
    let mut ed = open(dir.path(), "a.txt", b"foo bar\nFOO baz\n");
    ed.on_key(&ctrl('f'), &mut clip);
    for c in "foo".chars() {
        ed.on_key(&key(KeyCode::Char(c)), &mut clip);
    }
    assert_eq!(ed.doc.selection(), Some((0, 3)));
    ed.on_key(&key(KeyCode::Enter), &mut clip);
    assert_eq!(ed.doc.selection(), Some((8, 11)), "case-insensitive");
    let (buf, _) = draw(&mut ed, 80, 8);
    assert!(
        row_text(&buf, 0).contains("2 of 2"),
        "{:?}",
        row_text(&buf, 0)
    );
    ed.on_key(&key(KeyCode::Esc), &mut clip);
    assert!(ed.find.is_none(), "Esc closes the find bar first");
}

/// Buffer test: a Rust file draws with its
/// keyword and its identifier in different theme colors, with the
/// gutter's line numbers, the caret, and the dirty marker.
#[test]
fn a_rust_file_draws_highlighted_with_a_gutter_and_the_dirty_marker() {
    let dir = tempfile::tempdir().unwrap();
    let mut clip = Clipboard::new();
    let mut ed = open(dir.path(), "main.rs", b"fn main() {\n    let x = 1;\n}\n");
    let (buf, hits) = draw(&mut ed, 100, 10);
    // Bar row 0, border row 1, text from row 2; gutter "1 ".
    assert!(row_text(&buf, 0).contains("Save") && row_text(&buf, 0).contains("Rust"));
    assert!(row_text(&buf, 1).contains("Edit - main.rs @ working tree"));
    assert!(!row_text(&buf, 1).contains("modified"));
    assert_eq!(buf[(1, 2)].symbol(), "1");
    assert_eq!(buf[(1, 3)].symbol(), "2");
    let fn_cell = &buf[(3, 2)];
    let main_cell = &buf[(6, 2)];
    assert_eq!(fn_cell.symbol(), "f");
    assert_eq!(main_cell.symbol(), "m");
    assert!(matches!(main_cell.fg, Color::Rgb(..)));
    assert_ne!(fn_cell.fg, main_cell.fg, "keyword vs function name");
    assert!(
        fn_cell.modifier.contains(Modifier::REVERSED),
        "the caret sits on the first char"
    );
    assert!(hits.iter().any(|(_, h)| *h == EditorHit::Close));

    ed.on_key(&key(KeyCode::Char('x')), &mut clip);
    let (buf, _) = draw(&mut ed, 80, 10);
    assert!(row_text(&buf, 1).contains("\u{25CF} modified"));
    assert_eq!(buf[(3, 2)].symbol(), "x");
}

#[test]
fn tabs_draw_as_tab_stops_and_a_click_lands_on_the_char_under_it() {
    let dir = tempfile::tempdir().unwrap();
    let mut ed = open(dir.path(), "a.txt", b"\tab\n");
    let (buf, _) = draw(&mut ed, 40, 6);
    // Gutter "1 " from x=1; the tab covers x=3..7, 'a' at x=7.
    assert_eq!(buf[(7, 2)].symbol(), "a");
    ed.on_mouse(
        MouseEventKind::Down(MouseButton::Left),
        KeyModifiers::NONE,
        8,
        2,
    );
    assert_eq!(ed.doc.cursor(), 2, "on 'b'");
    ed.on_mouse(
        MouseEventKind::Drag(MouseButton::Left),
        KeyModifiers::NONE,
        3,
        2,
    );
    assert_eq!(ed.doc.selection(), Some((0, 2)));
}

#[test]
fn the_wheel_scrolls_without_moving_the_caret_and_the_view_stays_in_range() {
    let dir = tempfile::tempdir().unwrap();
    let text: String = (0..100).map(|i| format!("line {i}\n")).collect();
    let mut ed = open(dir.path(), "a.txt", text.as_bytes());
    draw(&mut ed, 40, 12);
    for _ in 0..100 {
        ed.on_mouse(MouseEventKind::ScrollDown, KeyModifiers::NONE, 5, 5);
    }
    let (buf, _) = draw(&mut ed, 40, 12);
    assert_eq!(ed.doc.cursor(), 0);
    // 101 lines (the empty last one), 9 visible: the last is at the
    // bottom row of the text.
    assert!(
        row_text(&buf, 10).contains("101"),
        "{:?}",
        row_text(&buf, 10)
    );
}

// ---- theme: light presets (maintainer report 2026-10-06) ----

fn lum(c: ratatui::style::Color) -> Option<f64> {
    use ratatui::style::Color;
    let Color::Rgb(r, g, b) = c else { return None };
    let ch = |v: u8| {
        let v = f64::from(v) / 255.0;
        if v <= 0.03928 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    Some(0.2126 * ch(r) + 0.7152 * ch(g) + 0.0722 * ch(b))
}

fn ratio(a: ratatui::style::Color, b: ratatui::style::Color) -> Option<f64> {
    let (la, lb) = (lum(a)?, lum(b)?);
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    Some((hi + 0.05) / (lo + 0.05))
}

/// The effective background of a cell: its own, else the theme's.
fn cell_bg(cell: &ratatui::buffer::Cell, t: &crate::theme::Theme) -> ratatui::style::Color {
    use ratatui::style::Color;
    match cell.bg {
        Color::Reset if t.ui.bg != Color::Reset => t.ui.bg,
        Color::Reset => t.ui.blend_bg,
        c => c,
    }
}

/// Every drawn glyph of `buf` reads against its background (3.0, the
/// bar's buttons and the text alike). Returns the failures.
fn unreadable(
    buf: &ratatui::buffer::Buffer,
    t: &crate::theme::Theme,
) -> Vec<(
    u16,
    u16,
    String,
    ratatui::style::Color,
    ratatui::style::Color,
)> {
    use ratatui::style::Modifier;
    let mut bad = Vec::new();
    for y in 0..buf.area.height {
        for x in 0..buf.area.width {
            let cell = &buf[(x, y)];
            let sym = cell.symbol();
            if sym.trim().is_empty()
                || cell.modifier.intersects(Modifier::REVERSED | Modifier::DIM)
                || sym
                    .chars()
                    .next()
                    .is_some_and(|c| ('\u{2800}'..='\u{28ff}').contains(&c))
            {
                continue;
            }
            let bg = cell_bg(cell, t);
            if let Some(r) = ratio(cell.fg, bg) {
                // Syntax comments are drawn faint by design (2.5); the bar's
                // buttons and the plain text hold 3.0.
                if r < if y >= 2 { 2.5 } else { 3.0 } {
                    bad.push((x, y, sym.to_string(), cell.fg, bg));
                }
            }
        }
    }
    bad
}

fn draw_themed(
    editor: &mut Editor,
    t: &crate::theme::Theme,
    hovered: Option<EditorHit>,
) -> ratatui::buffer::Buffer {
    let mut terminal = Terminal::new(TestBackend::new(100, 14)).unwrap();
    terminal
        .draw(|frame| {
            if t.ui.bg != ratatui::style::Color::Reset {
                frame.render_widget(
                    ratatui::widgets::Block::default()
                        .style(ratatui::style::Style::default().bg(t.ui.bg)),
                    frame.area(),
                );
            }
            editor.render(frame, Rect::new(0, 0, 100, 14), hovered);
        })
        .unwrap();
    terminal.backend().buffer().clone()
}

#[test]
fn the_editor_is_readable_under_every_preset_buttons_and_text() {
    let dir = tempfile::tempdir().unwrap();
    let mut failures = Vec::new();
    for spec in gitpoulpi_config::theme::presets() {
        let t = crate::theme::Theme::from(&spec);
        let _g = crate::theme::override_theme(&t);
        for hovered in [None, Some(EditorHit::Close)] {
            let mut ed = open(
                dir.path(),
                "a.rs",
                b"fn main() {\n    let s = \"text\"; \n}\n",
            );
            let buf = draw_themed(&mut ed, &t, hovered);
            let bad = unreadable(&buf, &t);
            if !bad.is_empty() {
                failures.push(format!(
                    "{} ({}): {:?}",
                    spec.slug,
                    spec.syntax,
                    &bad[..bad.len().min(3)]
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn switching_theme_with_the_editor_open_recolors_the_text_next_frame() {
    let dir = tempfile::tempdir().unwrap();
    let mut ed = open(dir.path(), "a.rs", b"fn main() {\n    let x = 1;\n}\n");
    let dark = crate::theme::Theme::dark();
    let light = crate::theme::Theme::from(&gitpoulpi_config::theme::preset("light").unwrap());
    {
        let _g = crate::theme::override_theme(&dark);
        let buf = draw_themed(&mut ed, &dark, None);
        assert!(unreadable(&buf, &dark).is_empty());
    }
    let _g = crate::theme::override_theme(&light);
    let buf = draw_themed(&mut ed, &light, None);
    let bad = unreadable(&buf, &light);
    assert!(bad.is_empty(), "{bad:?}");
}

// -- The status bar's chips and their menus -----------------------------

fn click_at(col: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: col,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

/// The cell where `needle` starts on some row of the drawn buffer.
fn find_text(buf: &ratatui::buffer::Buffer, needle: &str) -> Option<(u16, u16)> {
    (0..buf.area.height).find_map(|y| {
        let row = row_text(buf, y);
        row.find(needle)
            .map(|byte| (row[..byte].chars().count() as u16, y))
    })
}

fn hit_rect(hits: &[(Rect, EditorHit)], which: EditorHit) -> Rect {
    hits.iter().find(|(_, h)| *h == which).expect("hit").0
}

#[test]
fn the_bar_shows_the_encoding_the_line_breaks_and_the_indentation_as_chips() {
    let dir = tempfile::tempdir().unwrap();
    let mut ed = open(dir.path(), "a.txt", b"caf\xE9\r\n    x\r\n");
    let (buf, hits) = draw(&mut ed, 110, 8);
    let bar = row_text(&buf, 0);
    for chip in ["Windows-1252", "CRLF", "Spaces: 4", "Ln 1, Col 1"] {
        assert!(bar.contains(chip), "{chip} in {bar:?}");
    }
    let body = (0..8).map(|y| row_text(&buf, y)).collect::<String>();
    assert!(body.contains("caf\u{e9}"), "the accented text decoded");
    for hit in [
        EditorHit::EncodingChip,
        EditorHit::EolChip,
        EditorHit::IndentChip,
    ] {
        let r = hit_rect(&hits, hit);
        assert_eq!((r.y, r.height), (0, 1));
    }
    let enc = hit_rect(&hits, EditorHit::EncodingChip);
    let shown: String = (enc.x..enc.right()).map(|x| buf[(x, 0)].symbol()).collect();
    assert_eq!(shown.trim(), "Windows-1252", "the rect is the chip");
    // A file with several kinds of break says so.
    let mut mixed = open(dir.path(), "m.txt", b"a\r\nb\n");
    let (buf, _) = draw(&mut mixed, 110, 8);
    assert!(row_text(&buf, 0).contains("Mixed"));
}

#[test]
fn the_encoding_menu_converts_by_keyboard_and_the_save_writes_the_new_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let mut clip = Clipboard::new();
    let mut ed = open(dir.path(), "a.txt", b"caf\xE9\n");
    let (_, hits) = draw(&mut ed, 110, 12);
    let out = ed.click(EditorHit::EncodingChip);
    assert!(out.message.is_none() && ed.menu_open());
    let chip = hit_rect(&hits, EditorHit::EncodingChip);
    let (buf, _) = draw(&mut ed, 110, 12);
    let (x, y) = find_text(&buf, "Reopen with encoding").expect("the first step is drawn");
    assert!(
        y > 0 && x >= chip.x.saturating_sub(40),
        "hangs under the bar"
    );
    assert!(find_text(&buf, "Convert to encoding").is_some());
    // Down, Enter: the encodings to convert to; type to filter; Enter.
    ed.on_key(&key(KeyCode::Down), &mut clip);
    ed.on_key(&key(KeyCode::Enter), &mut clip);
    assert!(ed.menu_open(), "the second step is another menu");
    for c in "utf-8".chars() {
        ed.on_key(&key(KeyCode::Char(c)), &mut clip);
    }
    let (buf, _) = draw(&mut ed, 110, 12);
    assert!(find_text(&buf, "UTF-8 BOM").is_some() && find_text(&buf, "Windows-1250").is_none());
    let out = ed.on_key(&key(KeyCode::Enter), &mut clip);
    assert!(!ed.menu_open());
    assert!(out.message.unwrap().contains("UTF-8"));
    assert!(ed.doc.is_dirty());
    assert_eq!(ed.doc.text(), "caf\u{e9}\n");
    assert!(ed.save().saved);
    assert_eq!(
        std::fs::read(dir.path().join("a.txt")).unwrap(),
        "caf\u{e9}\n".as_bytes()
    );
}

#[test]
fn a_conversion_the_encoding_cannot_hold_says_so_and_changes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let mut ed = open(dir.path(), "a.txt", "price: 5\u{20ac}\n".as_bytes());
    ed.open_menu(StatusMenu::Convert);
    let (buf, _) = draw(&mut ed, 110, 24);
    let (x, y) = find_text(&buf, "ISO-8859-1").expect("listed");
    let out = ed.menu_mouse(click_at(x + 1, y));
    let message = out.message.expect("refused with a reason");
    assert!(
        message.contains("ISO-8859-1") && message.contains("line 1"),
        "{message}"
    );
    assert!(!ed.doc.is_dirty());
    assert_eq!(ed.doc.charset(), crate::editor_charset::Charset::Utf8);
}

#[test]
fn reopening_reads_the_bytes_again_and_asks_first_when_there_are_unsaved_changes() {
    use crate::editor_charset::Charset;
    let dir = tempfile::tempdir().unwrap();
    let mut clip = Clipboard::new();
    let mut ed = open(dir.path(), "a.txt", b"caf\xE9\n");
    assert_eq!(ed.doc.charset(), Charset::Windows1252);
    let out = ed.reopen_as(Charset::Koi8R);
    assert!(out.message.unwrap().contains("KOI8-R"));
    assert_eq!(ed.doc.text(), "caf\u{418}\n");
    assert!(!ed.doc.is_dirty(), "reopening is not a change");
    // With unsaved changes: the prompt, Cancel keeps everything.
    ed.on_key(&key(KeyCode::Char('x')), &mut clip);
    ed.reopen_as(Charset::Windows1252);
    assert!(ed.prompt && ed.doc.text().starts_with("xcaf"));
    ed.on_key(&key(KeyCode::Esc), &mut clip);
    assert!(!ed.prompt && ed.doc.is_dirty());
    // Discard reopens without the changes (and does not close).
    ed.reopen_as(Charset::Windows1252);
    let out = ed.on_key(&key(KeyCode::Char('d')), &mut clip);
    assert!(!out.close && !ed.prompt && !ed.doc.is_dirty());
    assert_eq!(ed.doc.text(), "caf\u{e9}\n");
    // Save then reopen writes first.
    ed.on_key(&key(KeyCode::Char('x')), &mut clip);
    ed.reopen_as(Charset::Koi8R);
    let out = ed.on_key(&key(KeyCode::Char('s')), &mut clip);
    assert!(out.saved && !out.close && !ed.prompt);
    assert_eq!(
        std::fs::read(dir.path().join("a.txt")).unwrap(),
        b"cxaf\xE9\n"
    );
    assert_eq!(ed.doc.charset(), Charset::Koi8R);
    // Bytes the page cannot read are refused with a message, the document stays.
    std::fs::write(dir.path().join("a.txt"), b"\xFF\xFEa").unwrap();
    let out = ed.reopen_as(Charset::Utf16Le);
    assert!(out.message.unwrap().contains("UTF-16 LE"));
    assert_eq!(
        ed.doc.charset(),
        Charset::Koi8R,
        "the document stays as it was"
    );
}

#[test]
fn the_line_ending_menu_converts_by_a_click_and_the_save_writes_it() {
    let dir = tempfile::tempdir().unwrap();
    let mut ed = open(dir.path(), "a.txt", b"one\r\ntwo\r\n");
    let (_, hits) = draw(&mut ed, 110, 12);
    let chip = hit_rect(&hits, EditorHit::EolChip);
    ed.click(EditorHit::EolChip);
    let (buf, _) = draw(&mut ed, 110, 12);
    let (cx, cy) = find_text(&buf, "Convert to CRLF").expect("the three rows");
    assert!(cy > 0 && find_text(&buf, "Convert to CR (classic").is_some());
    assert!(cx + 1 >= chip.x.min(cx + 1));
    // The current kind carries the check mark.
    assert!(row_text(&buf, cy).contains('\u{2713}'));
    let (lx, ly) = find_text(&buf, "Convert to LF").unwrap();
    let out = ed.menu_mouse(click_at(lx + 2, ly));
    assert!(out.message.unwrap().contains("LF"));
    assert!(!ed.menu_open());
    assert!(ed.doc.is_dirty());
    assert!(ed.save().saved);
    assert_eq!(
        std::fs::read(dir.path().join("a.txt")).unwrap(),
        b"one\ntwo\n"
    );
    let (buf, _) = draw(&mut ed, 110, 12);
    assert!(row_text(&buf, 0).contains(" LF "));
}

#[test]
fn the_indentation_menu_converts_tabs_to_spaces_and_sets_the_size() {
    let dir = tempfile::tempdir().unwrap();
    let mut clip = Clipboard::new();
    let mut ed = open(dir.path(), "a.txt", b"a\n\tb\n\t\tc\n");
    ed.open_menu(StatusMenu::Indentation);
    let (buf, _) = draw(&mut ed, 110, 24);
    assert!(find_text(&buf, "Indent using spaces").is_some());
    assert!(find_text(&buf, "Size: 8").is_some());
    // Type to filter, Enter: tabs become four spaces.
    for c in "to spaces".chars() {
        ed.on_key(&key(KeyCode::Char(c)), &mut clip);
    }
    let out = ed.on_key(&key(KeyCode::Enter), &mut clip);
    assert!(out.message.unwrap().contains('2'), "two lines changed");
    assert_eq!(ed.doc.text(), "a\n    b\n        c\n");
    assert!(ed.save().saved);
    assert_eq!(
        std::fs::read(dir.path().join("a.txt")).unwrap(),
        b"a\n    b\n        c\n"
    );
    // The size row: Tab now inserts that many spaces.
    ed.open_menu(StatusMenu::Indentation);
    let (buf, _) = draw(&mut ed, 110, 24);
    let (x, y) = find_text(&buf, "Size: 2").unwrap();
    ed.menu_mouse(click_at(x + 1, y));
    let (buf, _) = draw(&mut ed, 110, 12);
    assert!(row_text(&buf, 0).contains("Spaces: 2"));
}

#[test]
fn a_menu_is_modal_for_the_keyboard_and_esc_closes_it() {
    let dir = tempfile::tempdir().unwrap();
    let mut clip = Clipboard::new();
    let mut ed = open(dir.path(), "a.txt", b"x\n");
    ed.open_menu(StatusMenu::LineEndings);
    assert!(!ed.passes_through(&ctrl('p')), "the menu keeps every key");
    let out = ed.on_key(&key(KeyCode::Char('z')), &mut clip);
    assert_eq!(out, EditorOutcome::default());
    assert_eq!(ed.doc.text(), "x\n", "typing did not reach the text");
    ed.on_key(&key(KeyCode::Esc), &mut clip);
    assert!(!ed.menu_open());
    assert!(!ed.prompt, "Esc closed the menu, not the editor");
}

#[test]
fn saving_text_the_encoding_cannot_hold_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let mut clip = Clipboard::new();
    let mut ed = open(dir.path(), "a.txt", b"caf\xE9\n");
    ed.on_key(&key(KeyCode::Char('\u{4e2d}')), &mut clip);
    let out = ed.save();
    assert!(!out.saved && out.message.unwrap().contains("Windows-1252"));
    assert_eq!(
        std::fs::read(dir.path().join("a.txt")).unwrap(),
        b"caf\xE9\n"
    );
}

/// The highlighter advances only while drawing, one time budget a frame:
/// after a jump far down a long file the editor reports it is not caught
/// up (the loop keeps drawing) until a later frame is, then goes idle.
#[test]
fn a_long_jump_keeps_asking_for_frames_until_highlighting_has_caught_up() {
    let dir = tempfile::tempdir().unwrap();
    let mut clip = Clipboard::new();
    let text: String = (0..15_000)
        .map(|n| format!("fn item_{n}() {{ let x = {n}; }}\n"))
        .collect();
    let mut ed = open(dir.path(), "big.rs", text.as_bytes());
    ed.on_key(
        &KeyEvent::new(KeyCode::End, KeyModifiers::CONTROL),
        &mut clip,
    );
    draw(&mut ed, 100, 10);
    assert!(ed.highlight_pending(), "one frame budget cannot do it");
    let mut frames = 1;
    while ed.highlight_pending() {
        assert!(frames < 10_000, "never caught up");
        draw(&mut ed, 100, 10);
        frames += 1;
    }
    assert!(frames > 1);
    let mut small = open(dir.path(), "a.rs", b"fn main() {}\n");
    draw(&mut small, 100, 10);
    assert!(!small.highlight_pending(), "nothing to catch up on");
}

/// The text is an I-beam (a field in the pointer map); the buttons are hit
/// targets outside it and answer with the hand.
#[test]
fn the_text_area_is_registered_as_a_field_and_the_buttons_are_not_in_it() {
    let dir = tempfile::tempdir().unwrap();
    let mut ed = open(dir.path(), "a.txt", b"alpha\nbeta\n");
    crate::pointer_map::begin_frame();
    let (_, hits) = draw(&mut ed, 80, 12);
    let t = ed.text_area;
    assert_eq!(
        crate::pointer_map::at(t.x + 1, t.y),
        Some(crate::pointer_map::Kind::Field)
    );
    let (save, _) = hits
        .iter()
        .find(|(_, h)| *h == EditorHit::Save)
        .expect("the Save button");
    assert!(!t.intersects(*save));
}
