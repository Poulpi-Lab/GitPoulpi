use super::*;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn wrapped(text: &str, width: u16, max_lines: u16) -> (Vec<String>, u16) {
    let mut terminal = Terminal::new(TestBackend::new(width, 6)).unwrap();
    let mut drawn = 0;
    terminal
        .draw(|f| {
            let area = f.area();
            drawn = put_wrapped(
                &mut Canvas::new(f.buffer_mut()),
                area,
                0,
                0,
                text,
                Style::default(),
                max_lines,
            );
        })
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let rows = (0..buf.area.height)
        .map(|y| {
            (0..buf.area.width)
                .map(|x| buf[(x, y)].symbol())
                .collect::<String>()
                .replace(' ', "")
                .to_string()
        })
        .collect();
    (rows, drawn)
}

#[test]
fn put_wrapped_pins_wrapping_clipping_and_empty_text() {
    let (rows, drawn) = wrapped("aaa bbb ccc", 7, 4);
    assert_eq!((&rows[0][..], &rows[1][..], drawn), ("aaabbb", "ccc", 2));
    let (rows, drawn) = wrapped("aaa bbb ccc", 7, 1);
    assert_eq!((&rows[0][..], &rows[1][..], drawn), ("aaabbb", "", 1));
    let (rows, drawn) = wrapped("abcdefghij", 4, 4);
    assert_eq!((&rows[0][..], drawn), ("abc\u{2026}", 1));
    let (_, drawn) = wrapped("", 7, 4);
    assert_eq!(drawn, 1);
    let (rows, drawn) = wrapped("日本 語テ キ", 5, 4);
    assert_eq!(
        (&rows[0][..], &rows[1][..], &rows[2][..], drawn),
        ("日本", "語テ", "キ", 3)
    );
}
