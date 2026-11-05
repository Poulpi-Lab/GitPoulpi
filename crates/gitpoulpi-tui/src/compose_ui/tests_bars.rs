//! The review list's scrollbar: drawn when the commits outgrow the popup,
//! pressed and dragged like the shared bar everywhere, absent when they fit.

use super::*;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn draw(f: &ComposeForm) -> ratatui::buffer::Buffer {
    let mut terminal = Terminal::new(TestBackend::new(110, 16)).unwrap();
    terminal
        .draw(|frame| {
            f.render(frame, frame.area(), None);
        })
        .unwrap();
    terminal.backend().buffer().clone()
}

#[test]
fn a_long_list_has_a_scrollbar_the_thumb_drags_and_a_short_one_has_none() {
    let mut f = ComposeForm::new(
        (0..30)
            .map(|i| (format!("feat: {i}"), vec![format!("f{i}.rs")]))
            .collect(),
    );
    let buf = draw(&f);
    let track = f.bar.get().expect("30 commits do not fit 16 rows");
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
    assert_eq!(f.selected, ComposeRow::Commit(0), "the selection stayed");

    let short = ComposeForm::new(vec![("feat: a".into(), vec!["a.rs".into()])]);
    draw(&short);
    assert!(short.bar.get().is_none());
}
