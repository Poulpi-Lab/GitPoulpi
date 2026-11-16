use super::*;
use crate::reword_ui::AmendScope;
use crossterm::event::{KeyEventKind, KeyEventState, MouseButton, MouseEventKind};
use gitpoulpi_core::domain::Oid;

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

#[test]
fn a_cyrillic_key_letter_also_answers_to_the_latin_key_in_its_place() {
    assert_eq!(latin_twin('в'), Some('d'));
    assert_eq!(latin_twin('і'), Some('s'));
    assert_eq!(latin_twin('d'), None);
    let mut dialog = GuideDialog::conflicts_first(Blocked::Commit, &files(1));
    dialog.keys = vec![('р', Choice::Primary), ('о', Choice::Cancel)];
    // The Latin layout's key at the same place as 'р' is 'h'.
    assert_eq!(
        dialog.on_key(key(KeyCode::Char('h'))),
        Outcome::Chose(Choice::Primary)
    );
    assert_eq!(
        dialog.on_key(key(KeyCode::Char('Р'))),
        Outcome::Chose(Choice::Primary)
    );
    assert_eq!(
        dialog.on_key(key(KeyCode::Char('j'))),
        Outcome::Chose(Choice::Cancel)
    );
}

fn ctrl(c: char) -> KeyEvent {
    KeyEvent {
        modifiers: KeyModifiers::CONTROL,
        ..key(KeyCode::Char(c))
    }
}

fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

#[test]
fn the_off_branch_question_lists_a_button_per_branch_and_names_every_key() {
    let dialog = GuideDialog::off_branch(Oid([7; 20]), None, &names(&["feature/a", "topic"]), true);
    let (buf, text) = paint(&dialog, 100, 30);
    assert!(text.contains("Edit a commit of another branch"), "{text}");
    for want in [
        "Check out feature/a and edit (1)",
        "Check out topic and edit (2)",
        "Edit with advanced history rewrite (R)",
        "Cancel (C)",
    ] {
        assert!(text.contains(want), "{want}: {text}");
    }
    // One row each, left edges aligned: a list of buttons.
    let col_of = |needle: &str| {
        (0..30u16).find_map(|y| {
            let row: String = (0..100u16).map(|x| buf[(x, y)].symbol()).collect();
            row.find(needle).map(|at| (row[..at].chars().count(), y))
        })
    };
    let (a, ya) = col_of("Check out feature/a").unwrap();
    let (b, yb) = col_of("Check out topic").unwrap();
    assert_eq!(a, b);
    assert_eq!(yb, ya + 1);
}

#[test]
fn the_off_branch_question_without_a_local_branch_offers_to_create_one() {
    let dialog = GuideDialog::off_branch(Oid([7; 20]), None, &[], true);
    let (_, text) = paint(&dialog, 100, 30);
    assert!(text.contains("is on no local branch"), "{text}");
    assert!(text.contains("Create a branch here... (B)"), "{text}");
    assert!(!text.contains("Check out"), "{text}");
    // A build without the rewrite offers no rewrite.
    let dialog = GuideDialog::off_branch(Oid([7; 20]), None, &[], false);
    let (_, text) = paint(&dialog, 100, 30);
    assert!(!text.contains("advanced history rewrite"), "{text}");
}

#[test]
fn the_off_branch_keys_pick_move_and_cancel() {
    let mut dialog = GuideDialog::off_branch(
        Oid([7; 20]),
        Some(AmendScope::All),
        &names(&["a", "b"]),
        true,
    );
    assert_eq!(
        dialog.on_key(key(KeyCode::Char('2'))),
        Outcome::Chose(Choice::Pick(1))
    );
    assert_eq!(
        dialog.on_key(key(KeyCode::Char('r'))),
        Outcome::Chose(Choice::Rewrite)
    );
    assert_eq!(
        dialog.on_key(key(KeyCode::Char('c'))),
        Outcome::Chose(Choice::Cancel)
    );
    assert_eq!(
        dialog.on_key(key(KeyCode::Esc)),
        Outcome::Chose(Choice::Cancel)
    );
    // Arrows and Ctrl+J / Ctrl+K move the focus through the rows; Enter presses.
    assert_eq!(
        dialog.on_key(key(KeyCode::Enter)),
        Outcome::Chose(Choice::Pick(0))
    );
    for step in [key(KeyCode::Down), ctrl('j')] {
        let mut d = GuideDialog::off_branch(Oid([7; 20]), None, &names(&["a", "b"]), true);
        assert_eq!(d.on_key(step), Outcome::Stay);
        assert_eq!(
            d.on_key(key(KeyCode::Enter)),
            Outcome::Chose(Choice::Pick(1))
        );
    }
    for step in [key(KeyCode::Up), ctrl('k')] {
        let mut d = GuideDialog::off_branch(Oid([7; 20]), None, &names(&["a", "b"]), true);
        assert_eq!(d.on_key(step), Outcome::Stay);
        assert_eq!(
            d.on_key(key(KeyCode::Enter)),
            Outcome::Chose(Choice::Cancel)
        );
    }
}

#[test]
fn more_than_eight_branches_are_capped_and_say_so() {
    let many: Vec<String> = (1..=10).map(|i| format!("b{i}")).collect();
    let dialog = GuideDialog::off_branch(Oid([7; 20]), None, &many, true);
    let (_, text) = paint(&dialog, 100, 40);
    assert!(text.contains("Check out b8 and edit (8)"), "{text}");
    assert!(!text.contains("b9"), "{text}");
    assert!(text.contains("and 2 more"), "{text}");
}

#[test]
fn clicking_an_off_branch_row_answers_it() {
    let mut dialog = GuideDialog::off_branch(Oid([7; 20]), None, &names(&["a", "b"]), true);
    let (buf, _) = paint(&dialog, 100, 30);
    for (needle, want) in [
        ("Check out b and edit", Choice::Pick(1)),
        ("Edit with advanced", Choice::Rewrite),
        ("Cancel (C)", Choice::Cancel),
    ] {
        let (x, y) = (0..30u16)
            .find_map(|y| {
                let row: String = (0..100u16).map(|x| buf[(x, y)].symbol()).collect();
                row.find(needle)
                    .map(|at| (row[..at].chars().count() as u16 + 1, y))
            })
            .unwrap_or_else(|| panic!("no {needle}"));
        assert_eq!(dialog.on_mouse(click(x, y)), Outcome::Chose(want));
    }
}
