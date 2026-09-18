use super::*;
use crate::modes::{ConfirmAction, InputMode, PendingDestructive};
use crossterm::event::{KeyCode, KeyModifiers};
use gitpoulpi_core::domain::CommitNode;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

/// The popup is process-wide: tests that open it take turns.
static SERIAL: Mutex<()> = Mutex::new(());

fn oid(n: u8) -> Oid {
    Oid([n; 20])
}

fn node(id: u8, parents: &[u8], summary: &str) -> CommitNode {
    CommitNode {
        id: oid(id),
        parents: parents.iter().map(|p| oid(*p)).collect(),
        summary: summary.into(),
        message: summary.into(),
        author_name: "A".into(),
        author_email: "a@example.com".into(),
        timestamp: 0,
        refs: Vec::new(),
    }
}

/// 3 is a merge of 1 (the branch it was made on) and 2; 4 is ordinary.
fn graph() -> CommitGraph {
    CommitGraph {
        nodes: vec![
            node(4, &[3], "after"),
            node(3, &[1, 2], "merge feature"),
            node(2, &[1], "feature work"),
            node(1, &[], "main work"),
        ],
    }
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn screen() -> String {
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    terminal.draw(render).unwrap();
    let buf = terminal.backend().buffer();
    let mut out = String::new();
    for y in 0..buf.area.height {
        for x in 0..buf.area.width {
            out.push_str(buf[(x, y)].symbol());
        }
        out.push('\n');
    }
    out
}

#[test]
fn only_a_merge_commit_has_parents_to_choose_from() {
    let g = graph();
    assert_eq!(merge_parents(&g, oid(3)), Some(vec![oid(1), oid(2)]));
    assert_eq!(merge_parents(&g, oid(4)), None);
    assert_eq!(merge_parents(&g, oid(9)), None);
}

#[test]
fn the_rows_name_each_parent_and_the_pick_carries_its_number() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let g = graph();
    let rows = parent_options(&g, &[oid(1), oid(2)]);
    assert_eq!(rows.len(), 2);
    assert!(rows[0].label.starts_with("Parent 1: 0101010"), "{rows:?}");
    assert!(rows[0].label.ends_with("main work"), "{rows:?}");
    assert!(rows[1].label.contains("feature work"), "{rows:?}");

    open(&g, oid(3), &[oid(1), oid(2)]);
    assert!(is_open());
    let text = screen();
    assert!(text.contains("Revert a merge commit"), "{text}");
    assert!(text.contains("Which parent is the mainline"), "{text}");
    assert!(text.contains("main work"), "{text}");
    assert!(text.contains("feature work"), "{text}");

    // The keyboard: Down moves to parent 2, Enter picks it.
    assert_eq!(on_key(key(KeyCode::Down)), None);
    assert_eq!(
        on_key(key(KeyCode::Enter)),
        Some(Pick {
            commit: oid(3),
            parent: 2
        })
    );
    assert!(!is_open());

    // Esc closes with nothing picked.
    open(&g, oid(3), &[oid(1), oid(2)]);
    assert_eq!(on_key(key(KeyCode::Esc)), None);
    assert!(!is_open());
}

#[test]
fn the_pick_goes_on_to_the_confirm_naming_the_mainline() {
    let mut mode = InputMode::Normal;
    crate::log_menu::apply_revert_mainline(
        Pick {
            commit: oid(3),
            parent: 2,
        },
        &mut mode,
    );
    let InputMode::Confirm { prompt, action } = mode else {
        panic!("expected the confirm modal");
    };
    assert!(prompt.contains("parent 2"), "{prompt}");
    let ConfirmAction::Destructive(pending) = action else {
        panic!("expected a destructive confirm");
    };
    assert!(matches!(
        pending,
        PendingDestructive::Revert {
            commit,
            mainline: Some(2)
        } if commit == oid(3)
    ));
}

#[test]
fn the_cross_closes_the_popup_and_an_outside_click_too() {
    let _serial = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let click = |x: u16, y: u16| MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: x,
        row: y,
        modifiers: KeyModifiers::NONE,
    };
    let g = graph();
    open(&g, oid(3), &[oid(1), oid(2)]);
    let text = screen();
    assert!(text.contains('\u{2715}'), "{text}");
    assert!(!text.contains("Esc"), "{text}");
    let cross = POPUP
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .close
        .rect()
        .expect("a cross");
    assert_eq!(on_mouse(click(cross.x + 1, cross.y)), None);
    assert!(!is_open());
}
