//! Buffer tests of the help popup's tabs (`help_view::render` on a `TestBackend`).

use super::*;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::Terminal;

const W: u16 = 100;
const H: u16 = 30;
/// The tab row of the popup at 100x30 (popup starts on row 3, the filter box takes 3).
const TAB_ROW: u16 = 6;

fn draw_at(height: u16, filter: &str) -> Buffer {
    let mut term = Terminal::new(TestBackend::new(W, height)).unwrap();
    let (mut scroll, mut inner) = (0u16, 0u16);
    term.draw(|f| render(f, filter, &mut scroll, &mut inner))
        .unwrap();
    term.backend().buffer().clone()
}

fn draw(filter: &str) -> Buffer {
    draw_at(H, filter)
}

fn row(buf: &Buffer, y: u16) -> String {
    (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect()
}

fn screen(buf: &Buffer) -> String {
    (0..buf.area.height)
        .map(|y| row(buf, y))
        .collect::<Vec<_>>()
        .join("\n")
}

fn fresh() {
    forget();
    set_tab(0);
}

fn header_x(i: usize) -> u16 {
    (0..W)
        .find(|&x| tab_at(x, TAB_ROW) == Some(i))
        .expect("tab painted")
}

#[test]
fn the_tab_row_lists_every_tab_in_order() {
    fresh();
    let text = row(&draw(""), TAB_ROW);
    let mut at = 0;
    for t in TABS.iter() {
        let label = t.label.get();
        let found = text[at..].find(label);
        assert!(found.is_some(), "{label:?} missing or out of order: {text}");
        at += found.unwrap() + label.len();
    }
}

#[test]
fn the_active_tab_alone_is_highlighted() {
    fresh();
    set_tab(2);
    let buf = draw("");
    let lit: Vec<usize> = (0..TABS.len())
        .filter(|&i| buf[(header_x(i), TAB_ROW)].bg == th().ui.accent)
        .collect();
    assert_eq!(lit, vec![2]);
    fresh();
}

#[test]
fn the_hover_lights_one_header_and_the_body_is_no_header() {
    fresh();
    let _ = draw("");
    let x = header_x(4);
    assert_eq!(tab_at(x, TAB_ROW + 1), None);
    assert_ne!(draw("")[(x, TAB_ROW)].bg, th().ui.hover);
    hover(x, TAB_ROW);
    assert_eq!(draw("")[(x, TAB_ROW)].bg, th().ui.hover);
    hover(0, 0);
    assert_ne!(draw("")[(x, TAB_ROW)].bg, th().ui.hover);
    fresh();
}

#[test]
fn each_guide_tab_shows_its_own_entries() {
    fresh();
    for (i, tab) in TABS.iter().enumerate().skip(1) {
        set_tab(i);
        let shown = screen(&draw(""));
        let (title, _) = tab.entries[0].get().split_once('|').unwrap();
        assert!(
            shown.contains(title),
            "tab {i}: {title:?} not drawn\n{shown}"
        );
    }
    set_tab(0);
    assert!(screen(&draw("")).contains("Global"));
    fresh();
}

#[test]
fn the_filter_narrows_the_current_tab() {
    fresh();
    set_tab(5);
    let all = screen(&draw(""));
    let some = screen(&draw("ssh"));
    assert!(
        all.contains("Connect") && all.contains("Link a repository"),
        "{all}"
    );
    assert!(
        some.contains("SSH keys") && !some.contains("Connect"),
        "{some}"
    );
    fresh();
}

#[test]
fn the_scrollbar_shows_only_when_a_guide_tab_overflows() {
    fresh();
    set_tab(3);
    // Short window: the tab overflows, the right edge of the body is the bar.
    let _ = draw_at(14, "");
    let edge = W * 85 / 100 - 1;
    assert!(
        crate::popup_list::View::bar_press(edge, 8, 0),
        "no scrollbar on an overflowing tab"
    );
    crate::popup_list::View::bar_release();
    // One entry in a tall window: nothing to scroll.
    set_tab(5);
    let _ = draw_at(H, "toolbar chip");
    assert!(
        !crate::popup_list::View::bar_press(edge, 8, 0),
        "scrollbar on a tab that fits"
    );
    fresh();
}

#[test]
fn stepping_wraps_round_both_ways() {
    fresh();
    step_tab(-1);
    assert_eq!(tab(), TABS.len() - 1);
    step_tab(1);
    assert_eq!(tab(), 0);
    set_tab(99);
    assert_eq!(tab(), TABS.len() - 1);
    fresh();
}

#[test]
fn the_docs_link_follows_the_active_tab() {
    fresh();
    set_tab(5);
    assert!(docs_url(tab()).ends_with("/docs/user/hosting-integration.md"));
    assert!(screen(&draw("")).contains("hosting-integration.md"));
    set_tab(0);
    assert!(screen(&draw("")).contains("keybindings.md"));
    fresh();
}

#[test]
fn narrow_popups_shorten_labels_instead_of_overflowing() {
    let n = TABS.len();
    for width in [30usize, 40, 66] {
        let end = tab_cells(width)
            .last()
            .map(|(at, c)| at + display_width(c))
            .unwrap_or(0);
        // Never wider than the room, unless even one-cell labels do not fit.
        assert!(end <= width.max(3 * n + n - 1), "{width}: {end}");
    }
}

#[test]
fn the_popup_has_a_close_cross_and_no_esc_text_on_its_frame() {
    fresh();
    let buf = draw("");
    let top = row(&buf, 3);
    assert!(top.contains('\u{2715}'), "{top}");
    let cross_x = (0..W)
        .find(|&x| on_cross(x, 3))
        .expect("the cross is a hit rect from the paint walk");
    assert_eq!(buf[(cross_x + 1, 3)].symbol(), "\u{2715}");
    assert!(!on_cross(0, 3));
}
