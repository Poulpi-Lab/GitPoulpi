use super::*;
use crossterm::event::{KeyEventKind, KeyEventState, MouseButton, MouseEventKind};

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

fn paint(dialog: &GuideDialog, w: u16, h: u16) -> (Buffer, String) {
    let area = Rect::new(0, 0, w, h);
    let mut buf = Buffer::empty(area);
    dialog.render(&mut buf, area);
    let text = crate::dialog_kit::buffer_text(&buf);
    (buf, text)
}

fn files(n: usize) -> Vec<String> {
    (1..=n).map(|i| format!("src/file{i}.rs")).collect()
}

/// Any request does: the dialog only carries it back to the window.
fn conflict_request() -> AiRequest {
    AiRequest::Compose
}

#[test]
fn conflicts_first_lists_the_files_and_names_every_button_key() {
    let dialog = GuideDialog::conflicts_first(Blocked::Commit, &files(2));
    let (_, text) = paint(&dialog, 90, 24);
    assert!(text.contains("Resolve the conflicts first"), "{text}");
    assert!(text.contains("2 files still have conflicts"), "{text}");
    assert!(
        text.contains("src/file1.rs") && text.contains("src/file2.rs"),
        "{text}"
    );
    assert!(text.contains("Resolve conflicts (R)"), "{text}");
    assert!(text.contains("Cancel (C)"), "{text}");
    assert!(text.contains('\u{2715}'), "the close cross: {text}");
}

#[test]
fn the_list_is_capped_and_says_how_many_more() {
    let dialog = GuideDialog::conflicts_first(Blocked::Compose, &files(11));
    let (_, text) = paint(&dialog, 90, 30);
    assert!(text.contains("11 files still have conflicts"), "{text}");
    assert!(text.contains("src/file8.rs"), "{text}");
    assert!(!text.contains("src/file9.rs"), "{text}");
    assert!(text.contains("and 3 more"), "{text}");
}

#[test]
fn one_file_uses_the_singular() {
    let dialog = GuideDialog::conflicts_first(Blocked::Commit, &files(1));
    let (_, text) = paint(&dialog, 90, 24);
    assert!(text.contains("1 file still has conflicts"), "{text}");
}

#[test]
fn the_conflicts_dialog_keys_buttons_cross_and_escape() {
    let mut dialog = GuideDialog::conflicts_first(Blocked::Commit, &files(1));
    paint(&dialog, 90, 24);
    assert_eq!(
        dialog.on_key(key(KeyCode::Char('r'))),
        Outcome::Chose(Choice::Primary)
    );
    assert_eq!(
        dialog.on_key(key(KeyCode::Char('R'))),
        Outcome::Chose(Choice::Primary)
    );
    assert_eq!(
        dialog.on_key(key(KeyCode::Char('c'))),
        Outcome::Chose(Choice::Cancel)
    );
    assert_eq!(
        dialog.on_key(key(KeyCode::Esc)),
        Outcome::Chose(Choice::Cancel)
    );
    assert_eq!(dialog.on_key(key(KeyCode::Char('x'))), Outcome::Stay);
    // The focused button is the first: Enter resolves.
    assert_eq!(
        dialog.on_key(key(KeyCode::Enter)),
        Outcome::Chose(Choice::Primary)
    );
}

#[test]
fn clicking_the_buttons_the_cross_or_outside_answers() {
    let mut dialog = GuideDialog::conflicts_first(Blocked::Commit, &files(1));
    let (buf, _) = paint(&dialog, 90, 24);
    // Find the buttons by their text in the painted rows.
    let find = |needle: &str| -> (u16, u16) {
        for y in 0..24u16 {
            let row: String = (0..90u16).map(|x| buf[(x, y)].symbol()).collect();
            if let Some(at) = row.find(needle) {
                let x = row[..at].chars().count() as u16;
                return (x + 1, y);
            }
        }
        panic!("no {needle:?} painted");
    };
    let (x, y) = find("Resolve conflicts (R)");
    assert_eq!(
        dialog.on_mouse(click(x, y)),
        Outcome::Chose(Choice::Primary)
    );
    let (x, y) = find("Cancel (C)");
    assert_eq!(dialog.on_mouse(click(x, y)), Outcome::Chose(Choice::Cancel));
    let (x, y) = find("\u{2715}");
    assert_eq!(dialog.on_mouse(click(x, y)), Outcome::Chose(Choice::Cancel));
    assert_eq!(dialog.on_mouse(click(0, 0)), Outcome::Chose(Choice::Cancel));
}

#[test]
fn too_large_says_what_is_too_large_the_limit_and_the_setting() {
    let dialog = GuideDialog::too_large(conflict_request(), "src/big.rs", 49_206, 40_000);
    let (_, text) = paint(&dialog, 100, 24);
    assert!(text.contains("Conflict too large to send"), "{text}");
    assert!(text.contains("src/big.rs"), "{text}");
    // The box wraps the sentence: check the numbers, not the exact breaks.
    assert!(text.contains("49.2 KB (49206"), "{text}");
    assert!(text.contains("40.0 KB (40000"), "{text}");
    assert!(text.contains("ai.limits.conflict_bytes"), "{text}");
    assert!(text.contains("Send anyway (S)"), "{text}");
    assert!(text.contains("Open setting (O)"), "{text}");
    assert!(text.contains("Cancel (C)"), "{text}");
    assert!(text.contains('\u{2715}'), "the close cross: {text}");
}

#[test]
fn too_large_keys_answer_each_button_and_escape_cancels() {
    let mut dialog = GuideDialog::too_large(conflict_request(), "a", 50_000, 40_000);
    paint(&dialog, 100, 24);
    for (c, want) in [
        ('s', Choice::Primary),
        ('o', Choice::Secondary),
        ('c', Choice::Cancel),
    ] {
        assert_eq!(dialog.on_key(key(KeyCode::Char(c))), Outcome::Chose(want));
    }
    assert_eq!(
        dialog.on_key(key(KeyCode::Esc)),
        Outcome::Chose(Choice::Cancel)
    );
    // Open setting is the focused default: sending is never one Enter away.
    let mut dialog = GuideDialog::too_large(conflict_request(), "a", 50_000, 40_000);
    assert_eq!(
        dialog.on_key(key(KeyCode::Enter)),
        Outcome::Chose(Choice::Secondary)
    );
}

#[test]
fn sizes_read_as_a_person_would() {
    assert_eq!(size_text(900), "900 bytes");
    assert_eq!(size_text(49_206), "49.2 KB (49206 bytes)");
    assert_eq!(size_text(2_500_000), "2.5 MB (2500000 bytes)");
}
