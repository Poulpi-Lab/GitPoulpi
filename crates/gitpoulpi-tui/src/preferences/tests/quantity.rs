//! The "Keep log files for" row: a number and a unit on one line
//! (`Control::Quantity`), stored as days plus the unit it is read in.

use super::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEventKind};

fn press(st: &mut PreferencesState, code: KeyCode) {
    st.on_key(KeyEvent::new(code, KeyModifiers::NONE));
}

fn type_text(st: &mut PreferencesState, text: &str) {
    for c in text.chars() {
        press(st, KeyCode::Char(c));
    }
}

/// The General page with the retention row selected.
fn on_retention(st: &mut PreferencesState) {
    go(st, PrefSection::General);
    st.focus = PrefFocus::Fields;
    st.field = st
        .fields()
        .iter()
        .position(|f| f.target == Target::Global("log.retention_days"))
        .expect("the retention row");
}

fn stored(dir: &Path) -> String {
    std::fs::read_to_string(dir.join("config.toml")).unwrap_or_default()
}

fn row(screen: &str) -> String {
    screen
        .lines()
        .find(|l| l.contains("Keep log files for"))
        .unwrap_or_else(|| panic!("no retention row:\n{screen}"))
        .to_string()
}

/// Left-click one part (0 number, 1 unit) of the selected row.
fn click_part(st: &mut PreferencesState, part: usize) {
    let index = st.field;
    let rect = st
        .btn_rects
        .iter()
        .find(|(_, f, p)| *f == index && *p == part)
        .map(|(r, _, _)| *r)
        .unwrap_or_else(|| panic!("no rect for part {part}: {:?}", st.btn_rects));
    st.on_mouse(MouseEventKind::Down(MouseButton::Left), rect.x + 1, rect.y);
}

/// Replaces what is typed in the open number with `text`.
fn retype(st: &mut PreferencesState, text: &str) {
    for _ in 0..8 {
        press(st, KeyCode::Backspace);
    }
    type_text(st, text);
}

#[test]
fn the_row_draws_the_number_and_the_unit_on_one_line() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    on_retention(&mut st);
    let line = row(&draw_screen(&mut st));
    // `[ 1 ] [week v]`: the default 7 days read back as one week.
    let at = |needle: &str| {
        line.find(needle)
            .unwrap_or_else(|| panic!("{needle}: {line}"))
    };
    assert!(at("1") < at("week"), "{line}");
    assert!(line.contains('\u{25BE}'), "{line}");
    assert!(!line.contains("1 week"), "two bands, not one text: {line}");
    assert_eq!(st.btn_rects.len(), 2, "{:?}", st.btn_rects);
}

#[test]
fn an_old_config_value_reads_back_in_the_largest_even_unit() {
    let dir = tempfile::tempdir().unwrap();
    for (days, shown) in [
        (14, "2 weeks"),
        (90, "3 months"),
        (365, "1 year"),
        (45, "45 days"),
        (1, "1 day"),
        (2, "2 days"),
    ] {
        let mut st = state(dir.path());
        st.config.log.retention_days = days;
        on_retention(&mut st);
        assert_eq!(st.fields()[st.field].value_text(), shown, "{days} days");
    }
}

#[test]
fn typing_two_with_the_unit_on_weeks_writes_fourteen_days_and_the_unit() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    on_retention(&mut st);
    press(&mut st, KeyCode::Enter);
    assert!(st.edit.is_some(), "Enter types in the number");
    retype(&mut st, "2");
    press(&mut st, KeyCode::Enter);
    assert!(st.edit.is_none());
    let file = stored(dir.path());
    assert!(file.contains("retention_days = 14"), "{file}");
    assert!(file.contains("retention_unit = \"weeks\""), "{file}");
    assert_eq!(st.config.log.retention_quantity().0, 2);
    assert!(row(&draw_screen(&mut st)).contains("weeks"));
}

#[test]
fn tab_walks_number_then_unit_and_the_unit_opens_a_dropdown_that_does_not_cycle() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    on_retention(&mut st);
    press(&mut st, KeyCode::Tab);
    assert_eq!(st.ai.chip_focus(2), 1);
    press(&mut st, KeyCode::Enter);
    assert!(st.choice.is_some(), "Enter on the unit opens the list");
    // Down once from "weeks" is "months": 1 month = 30 days.
    press(&mut st, KeyCode::Down);
    press(&mut st, KeyCode::Enter);
    assert!(st.choice.is_none());
    let file = stored(dir.path());
    assert!(file.contains("retention_days = 30"), "{file}");
    assert!(file.contains("retention_unit = \"months\""), "{file}");
    // BackTab goes back to the number.
    press(&mut st, KeyCode::BackTab);
    assert_eq!(st.ai.chip_focus(2), 0);
}

#[test]
fn up_and_down_step_the_number_while_typing_and_tab_moves_to_the_unit() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    on_retention(&mut st);
    press(&mut st, KeyCode::Enter);
    press(&mut st, KeyCode::Up);
    press(&mut st, KeyCode::Up);
    assert_eq!(st.edit.as_ref().unwrap().1, "3");
    press(&mut st, KeyCode::Down);
    assert_eq!(st.edit.as_ref().unwrap().1, "2");
    // Never below one.
    press(&mut st, KeyCode::Down);
    press(&mut st, KeyCode::Down);
    assert_eq!(st.edit.as_ref().unwrap().1, "1");
    press(&mut st, KeyCode::Up);
    press(&mut st, KeyCode::Tab);
    assert!(st.edit.is_none(), "Tab commits");
    assert_eq!(st.ai.chip_focus(2), 1, "and lands on the unit");
    assert!(stored(dir.path()).contains("retention_days = 14"));
}

#[test]
fn zero_and_too_many_are_refused_with_the_reason_under_the_row() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    on_retention(&mut st);
    press(&mut st, KeyCode::Enter);
    retype(&mut st, "0");
    press(&mut st, KeyCode::Enter);
    assert!(st.edit.is_some(), "a refused number keeps the field open");
    assert!(st.edit_invalid.is_some());
    assert!(stored(dir.path()).is_empty(), "nothing was written");
    let screen = draw_screen(&mut st);
    assert!(screen.contains("is outside 1"), "{screen}");
    // 600 weeks = 4200 days, above ten years (521 weeks).
    retype(&mut st, "600");
    press(&mut st, KeyCode::Enter);
    assert!(st.edit.is_some());
    assert!(draw_screen(&mut st).contains("600 is outside 1"));
    retype(&mut st, "521");
    press(&mut st, KeyCode::Enter);
    assert!(st.edit.is_none());
    assert!(stored(dir.path()).contains("retention_days = 3647"));
    // Not a number at all.
    press(&mut st, KeyCode::Enter);
    retype(&mut st, "x");
    press(&mut st, KeyCode::Enter);
    assert!(st.edit.is_some() && st.edit_invalid.is_some());
    let (_, why) = st.inline_error.clone().expect("the reason is kept");
    assert!(why.contains("not a whole number"), "{why}");
}

#[test]
fn ten_years_is_the_most_and_a_longer_unit_cuts_the_number_to_fit() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    on_retention(&mut st);
    press(&mut st, KeyCode::Enter);
    retype(&mut st, "60");
    press(&mut st, KeyCode::Enter);
    // 60 weeks, then the unit years: 60 is cut to 10 (3650 days).
    press(&mut st, KeyCode::Tab);
    press(&mut st, KeyCode::Enter);
    press(&mut st, KeyCode::Down);
    press(&mut st, KeyCode::Down);
    press(&mut st, KeyCode::Enter);
    let file = stored(dir.path());
    assert!(file.contains("retention_days = 3650"), "{file}");
    assert!(file.contains("retention_unit = \"years\""), "{file}");
    assert_eq!(st.fields()[st.field].value_text(), "10 years");
}

#[test]
fn a_click_on_the_number_types_and_a_click_on_the_unit_opens_the_list() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    on_retention(&mut st);
    draw_screen(&mut st);
    click_part(&mut st, 1);
    assert!(st.choice.is_some(), "one click opens the unit list");
    press(&mut st, KeyCode::Esc);
    assert!(st.choice.is_none());
    draw_screen(&mut st);
    click_part(&mut st, 0);
    assert!(st.edit.is_some(), "one click opens the number");
    retype(&mut st, "3");
    press(&mut st, KeyCode::Enter);
    assert!(stored(dir.path()).contains("retention_days = 21"));
}
