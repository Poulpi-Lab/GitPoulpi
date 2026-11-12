//! The Integrations page's scrolling through Settings' own mouse routing:
//! the wheel (and the pointer, for an open popup) reach the page.

use super::*;
use crossterm::event::{MouseButton, MouseEventKind};
use gitpoulpi_config::ConnectedUser;

const COLS: u16 = 100;
const ROWS: u16 = 24;

fn screen(state: &mut PreferencesState) -> String {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let mut terminal = Terminal::new(TestBackend::new(COLS, ROWS)).unwrap();
    terminal
        .draw(|frame| state.render(frame, frame.area()))
        .unwrap();
    let buffer = terminal.backend().buffer();
    (0..ROWS)
        .map(|y| {
            (0..COLS)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn connected_state(dir: &Path) -> PreferencesState {
    let mut state = state(dir);
    let mut profile = Integration::new("work", IntegrationKind::Github, "https://github.com");
    profile.user = Some(ConnectedUser {
        login: "kai".into(),
        name: "Kai".into(),
        id: "7".into(),
    });
    state.config.integrations.push(profile);
    state.show_radar_options();
    state
}

#[test]
fn the_wheel_scrolls_the_integrations_card_through_settings() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = connected_state(dir.path());
    let before = screen(&mut state);
    assert!(
        before.contains("[ ] Requests waiting for my review"),
        "{before}"
    );
    assert!(!before.contains("Personal access token"), "{before}");
    let (row, line) = before
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains("[ ] Requests waiting for my review"))
        .unwrap();
    let col = line.find("[ ]").unwrap() as u16;
    for _ in 0..8 {
        assert!(state.on_mouse(MouseEventKind::ScrollUp, col, row as u16));
    }
    let after = screen(&mut state);
    assert!(after.contains("Personal access token"), "{after}");
}

/// `Ctrl+N` through Settings badges the Radar block's checkboxes, the
/// Namespaces area and the Regular expressions checkbox, and a press at the
/// badge's point toggles the checkbox.
#[test]
fn ctrl_n_badges_the_radar_block_including_the_regular_expressions_checkbox() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = connected_state(dir.path());
    // A tall screen: the whole Radar block is on it.
    let text = {
        use ratatui::backend::TestBackend;
        let mut terminal = ratatui::Terminal::new(TestBackend::new(100, 90)).unwrap();
        terminal
            .draw(|frame| state.render(frame, frame.area()))
            .unwrap();
        let buffer = terminal.backend().buffer();
        (0..90)
            .map(|y| {
                (0..100)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let badged_rows: Vec<u16> = state
        .nav_targets()
        .iter()
        .map(|(rect, _)| rect.y)
        .collect();
    for label in [
        "Requests waiting for my review",
        "e.g. acme, platform",
        "Regular expressions",
    ] {
        let row = text
            .lines()
            .position(|l| l.contains(label))
            .unwrap_or_else(|| panic!("no {label}:\n{text}")) as u16;
        assert!(badged_rows.contains(&row), "{label} has no badge:\n{text}");
    }
    let row = text
        .lines()
        .position(|l| l.contains("Regular expressions"))
        .unwrap() as u16;
    let (_, point) = *state
        .nav_targets()
        .iter()
        .find(|(rect, _)| rect.y == row)
        .unwrap();
    assert!(!state.config.integrations[0]
        .radar_options()
        .namespaces_regex);
    state.nav_press(point);
    assert!(state.config.integrations[0].radar_options().namespaces_regex);
}

#[test]
fn an_open_popup_takes_the_pointer_even_outside_the_page() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = connected_state(dir.path());
    screen(&mut state);
    // Tab to the combobox and open it.
    for _ in 0..6 {
        state.on_key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Tab,
            crossterm::event::KeyModifiers::NONE,
        ));
        screen(&mut state);
    }
    state.on_key(crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Enter,
        crossterm::event::KeyModifiers::NONE,
    ));
    assert!(state.integrations.combo_open());
    let open = screen(&mut state);
    assert!(open.contains("Review through the API"), "{open}");
    // The wheel and a move far from the page are the popup's: it stays open.
    assert!(state.on_mouse(MouseEventKind::ScrollDown, 1, 1));
    assert!(state.on_mouse(MouseEventKind::Moved, 1, 1));
    assert!(state.integrations.combo_open());
    // A press on the sidebar only closes the popup: it is modal.
    let section = state.section;
    assert!(state.on_mouse(MouseEventKind::Down(MouseButton::Left), 3, 8));
    assert!(!state.integrations.combo_open());
    assert_eq!(state.section, section);
}
