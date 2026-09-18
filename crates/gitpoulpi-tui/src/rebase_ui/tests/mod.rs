mod i18n;
mod input;
mod plan;
mod render;

use super::*;

use crate::test_support::*;

fn row(commit: u8, summary: &str, action: RebaseAction) -> RebaseRow {
    RebaseRow {
        action,
        ..RebaseRow::pick(oid(commit), summary, summary)
    }
}

/// Display order is newest first (row 0 = commit 3); `steps()` sends
/// oldest first.
fn form() -> RebaseForm {
    RebaseForm::new(
        "feature".into(),
        Some(oid(0)),
        None,
        vec![
            row(3, "third", RebaseAction::Pick),
            row(2, "second", RebaseAction::Pick),
            row(1, "first", RebaseAction::Pick),
        ],
    )
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn alt(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::ALT)
}

fn paint(f: &RebaseForm) -> (Vec<String>, Vec<(Rect, RebasePart)>) {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    let mut hits = Vec::new();
    terminal
        .draw(|frame| hits = f.render(frame, frame.area(), frame.area(), None, Default::default()))
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let rows = (0..30)
        .map(|y| (0..100).map(|x| buf[(x, y)].symbol().to_string()).collect())
        .collect();
    (rows, hits)
}

fn paint_hovered(
    f: &RebaseForm,
    hovered: Option<RebasePart>,
) -> (ratatui::buffer::Buffer, Vec<(Rect, RebasePart)>) {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    let mut hits = Vec::new();
    terminal
        .draw(|frame| {
            hits = f.render(
                frame,
                frame.area(),
                frame.area(),
                hovered,
                Default::default(),
            )
        })
        .unwrap();
    (terminal.backend().buffer().clone(), hits)
}

fn msg_row(commit: u8, message: &str) -> RebaseRow {
    RebaseRow::pick(oid(commit), message.lines().next().unwrap(), message)
}
