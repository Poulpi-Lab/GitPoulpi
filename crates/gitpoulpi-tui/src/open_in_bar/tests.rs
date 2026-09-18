use super::*;
use crossterm::event::{KeyCode, KeyModifiers};
use gitpoulpi_config::open_in::OpenInEntry;
use gitpoulpi_config::UiConfig;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn install(labels: &[&str]) {
    let ui = UiConfig {
        open_in: Some(
            labels
                .iter()
                .map(|l| OpenInEntry {
                    label: (*l).into(),
                    command: "true".into(),
                })
                .collect(),
        ),
        open_in_default: labels.get(1).copied().unwrap_or_default().into(),
        ..UiConfig::default()
    };
    crate::tab_dir::install(&ui);
    close();
    LAST_USED.store(NONE, Ordering::Relaxed);
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn paint(width: u16, hits: &mut Vec<(Rect, HitTarget)>) -> Terminal<TestBackend> {
    let mut terminal = Terminal::new(TestBackend::new(width, 20)).unwrap();
    terminal
        .draw(|frame| {
            set_bar(Rect::new(0, 0, width, 1));
            render_popup(frame, None, hits);
        })
        .unwrap();
    terminal
}

fn row_text(terminal: &Terminal<TestBackend>, rect: Rect) -> String {
    let buf = terminal.backend().buffer();
    (rect.x..rect.right())
        .map(|x| buf[(x, rect.y)].symbol().to_string())
        .collect()
}

#[test]
fn popup_rows_lead_the_hit_list_and_match_what_was_painted() {
    let _serial = crate::tab_dir::TEST_SERIAL
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    install(&["Files", "Terminal", "Editor"]);
    open();
    let tab = (Rect::new(0, 0, 20, 1), HitTarget::OpenRepositoryButton);
    let mut hits = vec![tab.clone()];
    let terminal = paint(80, &mut hits);

    let items: Vec<_> = hits
        .iter()
        .take_while(|(_, t)| !matches!(t, HitTarget::OpenRepositoryButton))
        .collect();
    // Three entries, "Configure...", and the popup shield; then the old hits.
    assert_eq!(items.len(), 5, "{items:?}");
    assert_eq!(hits.last(), Some(&tab));
    let texts: Vec<String> = items[..4]
        .iter()
        .map(|(rect, t)| {
            assert!(matches!(t, HitTarget::OpenInItem(_)), "{t:?}");
            row_text(&terminal, *rect)
        })
        .collect();
    assert!(texts[0].contains("Files"), "{texts:?}");
    assert!(
        texts[1].contains("\u{2022} Terminal"),
        "the default is marked: {texts:?}"
    );
    assert!(texts[2].contains("Editor"), "{texts:?}");
    assert!(texts[3].contains("Configure"), "{texts:?}");
    assert!(matches!(items[4].1, HitTarget::OpenInPopup));
    close();
}

#[test]
fn the_keyboard_path_picks_with_up_down_enter_and_esc_closes() {
    let _serial = crate::tab_dir::TEST_SERIAL
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    install(&["Files", "Terminal", "Editor"]);
    open();
    assert!(is_open());
    // Starts on the default (Terminal); Down moves to Editor.
    assert_eq!(on_key(key(KeyCode::Down)), None);
    assert_eq!(on_key(key(KeyCode::Enter)), Some(Pick::Entry(2)));
    assert!(!is_open());

    open();
    assert_eq!(on_key(key(KeyCode::Esc)), None);
    assert!(!is_open());

    // The last-used entry is where the next popup starts.
    LAST_USED.store(0, Ordering::Relaxed);
    open();
    assert_eq!(on_key(key(KeyCode::Enter)), Some(Pick::Entry(0)));

    open();
    on_key(key(KeyCode::End));
    assert_eq!(on_key(key(KeyCode::Enter)), Some(Pick::Configure));
}

#[test]
fn a_click_away_closes_without_a_pick_and_a_click_on_a_row_picks_it() {
    use crossterm::event::{MouseButton, MouseEventKind};
    let _serial = crate::tab_dir::TEST_SERIAL
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    install(&["Files", "Terminal"]);
    let click = |x, y| MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: x,
        row: y,
        modifiers: KeyModifiers::NONE,
    };
    open();
    let mut hits = Vec::new();
    paint(80, &mut hits);
    assert_eq!(on_mouse(click(0, 15)), None);
    assert!(!is_open(), "a click away closes it");

    open();
    let mut hits = Vec::new();
    paint(80, &mut hits);
    let (rect, _) = hits[1];
    assert_eq!(on_mouse(click(rect.x + 1, rect.y)), Some(Pick::Entry(1)));
    assert!(!is_open());
}

#[test]
fn the_button_is_a_combobox_field_that_fits_its_width() {
    let mut terminal = Terminal::new(TestBackend::new(40, 1)).unwrap();
    let w = button_width();
    terminal
        .draw(|frame| render_button(frame.buffer_mut(), Rect::new(0, 0, w, 1), false))
        .unwrap();
    let text = row_text(&terminal, Rect::new(0, 0, w, 1));
    assert_eq!(text, format!(" {} \u{25BE} ", msg::BUTTON.get()));
}

#[test]
fn the_tooltip_names_the_default_entry() {
    let _serial = crate::tab_dir::TEST_SERIAL
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    install(&["Files", "Terminal"]);
    assert_eq!(tooltip(), "Open in Terminal...");
    assert_eq!(tooltip(), tooltip());
}
