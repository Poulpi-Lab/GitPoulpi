//! Each page remembers its selected row and scroll: switching away and back,
//! or reopening Settings, lands where the user was.

use super::*;

fn draw_short(st: &mut PreferencesState) {
    use ratatui::backend::TestBackend;
    let mut terminal = ratatui::Terminal::new(TestBackend::new(100, 16)).unwrap();
    terminal
        .draw(|frame| st.render(frame, frame.area()))
        .unwrap();
}

fn at(section: PrefSection) -> usize {
    PrefSection::ALL.iter().position(|x| *x == section).unwrap()
}

fn walk_down(st: &mut PreferencesState) {
    go_tab(st, PrefSection::Appearance, SubTab::General);
    st.focus = PrefFocus::Fields;
    draw_short(st);
    for _ in 0..8 {
        st.on_key(KeyEvent::from(KeyCode::Down));
        draw_short(st);
    }
}

#[test]
fn a_page_comes_back_with_its_row_and_scroll() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    walk_down(&mut st);
    let (field, scroll) = (st.field, st.scroll);
    assert!(scroll > 0, "the short screen scrolled the page");

    st.focus = PrefFocus::Sections;
    st.switch_section(at(PrefSection::Sync));
    draw_short(&mut st);
    st.switch_section(at(PrefSection::Appearance));
    draw_short(&mut st);
    assert_eq!((st.field, st.scroll), (field, scroll));

    // A page never visited starts at the top.
    st.switch_section(at(PrefSection::LogGraph));
    assert_eq!((st.field, st.scroll), (st.first_selectable(), 0));
}

#[test]
fn a_reopened_screen_restores_the_views_and_clamps_a_gone_row() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    walk_down(&mut st);
    let (field, scroll) = (st.field, st.scroll);
    st.save_view();

    let mut again = state(dir.path());
    go_tab(&mut again, PrefSection::Appearance, SubTab::General);
    again.views = st.views.clone();
    again.restore_view();
    assert_eq!((again.field, again.scroll), (field, scroll));
    draw_short(&mut again);
    assert_eq!(again.scroll, scroll, "the paint keeps the restored scroll");

    // The page got shorter: the row is gone, the scroll is clamped by the paint.
    let slot = again.view_slot();
    again.views[slot] = Some((10_000, 10_000));
    again.restore_view();
    assert_eq!(again.field, again.first_selectable());
    draw_short(&mut again);
    assert!(again.scroll < 10_000);
}
