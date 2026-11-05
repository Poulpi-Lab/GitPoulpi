//! The plan list's scrollbar: drawn when the commits outgrow the popup,
//! pressed and dragged like the shared bar everywhere, scrolled by the wheel
//! without moving the selection, absent when they fit.

use super::*;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn long() -> RebaseForm {
    RebaseForm::new(
        "feature".into(),
        Some(oid(0)),
        None,
        (1..=40u8)
            .rev()
            .map(|i| row(i, &format!("commit {i}"), RebaseAction::Pick))
            .collect(),
    )
}

fn draw(f: &RebaseForm) -> ratatui::buffer::Buffer {
    let mut terminal = Terminal::new(TestBackend::new(100, 20)).unwrap();
    terminal
        .draw(|frame| {
            f.render(frame, frame.area(), frame.area(), None, Default::default());
        })
        .unwrap();
    terminal.backend().buffer().clone()
}

#[test]
fn a_long_plan_has_a_scrollbar_the_thumb_drags_and_the_wheel_scrolls() {
    let mut f = long();
    let buf = draw(&f);
    let track = f.bar.get().expect("40 commits do not fit 20 rows");
    assert_eq!(buf[(track.rect.x, track.rect.y)].symbol(), "\u{2191}");
    assert_eq!(
        buf[(track.rect.x, track.rect.bottom() - 1)].symbol(),
        "\u{2193}"
    );
    let ev = |kind, row| MouseEvent {
        kind,
        column: track.rect.x,
        row,
        modifiers: KeyModifiers::NONE,
    };
    assert!(f.on_bar_mouse(ev(
        MouseEventKind::Down(MouseButton::Left),
        track.rect.y + 1
    )));
    assert!(f.on_bar_mouse(ev(
        MouseEventKind::Drag(MouseButton::Left),
        track.rect.bottom() + 9
    )));
    f.on_bar_mouse(ev(MouseEventKind::Up(MouseButton::Left), 0));
    draw(&f);
    let (offset, rows, count) = f.window.get();
    assert_eq!(offset, count - rows, "dragged to the end");
    assert_eq!(f.cursor, 0, "the selection stayed");
    f.on_wheel(-3);
    draw(&f);
    assert_eq!(f.window.get().0, count - rows - 3, "the wheel scrolled it");
    // A short plan has no bar.
    let short = form();
    draw(&short);
    assert!(short.bar.get().is_none());
}
