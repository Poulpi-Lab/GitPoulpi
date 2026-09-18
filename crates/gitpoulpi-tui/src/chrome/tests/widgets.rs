use super::*;

#[test]
fn a_diff_bar_button_is_reachable_through_the_shared_hit_target_mechanism() {
    let state =
        DiffViewState::for_commit(gitpoulpi_core::domain::Oid([3u8; 20]), 0, PathBuf::new());
    let hits = diff_view::diff_bar(&state, None, false, 200).buttons;
    let bar = Rect::new(4, 2, 200, 1);
    let targets: Vec<(Rect, HitTarget)> = hits
        .iter()
        .map(|&(dx, w, button)| {
            (
                Rect::new(bar.x + dx, bar.y, w, 1),
                HitTarget::DiffBarButton(button),
            )
        })
        .collect();
    // The very first button is File View. It registers a target
    // whether or not it is backed yet -- that is what lets a click
    // either run it or explain itself, instead of silently doing
    // nothing.
    let (first_rect, _) = &targets[0];
    assert_eq!(
        find_hit_target(&targets, first_rect.x, bar.y),
        Some(HitTarget::DiffBarButton(DiffBarButton::FileView))
    );
    // A row above/below the bar hits nothing.
    assert_eq!(find_hit_target(&targets, first_rect.x, bar.y + 1), None);
}

/// The search bar's three buttons must be hit-testable at the exact
/// cells they were painted at. The bar is built once and both painted
/// and registered from the same `(dx, w, button)` list, so this pins
/// that contract rather than restating the layout arithmetic.
#[test]
fn the_search_bar_registers_its_buttons_where_it_paints_them() {
    let search = LogSearch {
        query: "merge".into(),
        current: 1,
    };
    let bar = Rect::new(20, 3, 46, 1);
    let (_, hits) = log_search::search_bar(&search, 4, 40, bar.width);
    let targets: Vec<(Rect, HitTarget)> = hits
        .iter()
        .map(|&(dx, w, button)| {
            (
                Rect::new(bar.x + dx, bar.y, w, 1),
                HitTarget::SearchBarButton(button),
            )
        })
        .collect();

    for &(dx, w, button) in &hits {
        // Every cell of a button, not just its first, must resolve --
        // an off-by-one in the width would make the last column dead.
        for probe in 0..w {
            assert_eq!(
                find_hit_target(&targets, bar.x + dx + probe, bar.y),
                Some(HitTarget::SearchBarButton(button)),
                "cell {probe} of {button:?} didn't hit"
            );
        }
    }
    // Rows above/below the one-line bar hit nothing.
    assert_eq!(
        find_hit_target(&targets, bar.x + hits[0].0, bar.y + 1),
        None
    );
}

/// Draws the quit button on a one-row title bar of `width` cells, then the
/// gear left of it; returns the buffer row, and both rects when drawn.
fn title_row(
    width: u16,
    title_end_x: u16,
    hovered: bool,
) -> (
    ratatui::buffer::Buffer,
    Option<Rect>,
    Vec<(Rect, HitTarget)>,
) {
    use ratatui::backend::TestBackend;

    let mut terminal = Terminal::new(TestBackend::new(width, 1)).unwrap();
    let mut hits: Vec<(Rect, HitTarget)> = Vec::new();
    let mut close = None;
    terminal
        .draw(|frame| {
            close = render_close_app_button(
                frame,
                Rect::new(0, 0, width, 1),
                title_end_x,
                false,
                &mut hits,
            );
            if let Some(close) = close {
                render_gear(frame, close, title_end_x, hovered, &mut hits);
            }
        })
        .unwrap();
    (terminal.backend().buffer().clone(), close, hits)
}

/// The gear sits immediately left of the quit button with a gap of blank
/// cells between them (so a click aimed at the gear never quits), and each
/// registered rect is the one its glyph landed in. Falsified by painting the
/// gear anywhere else (the old tab-row spot) or by dropping the gap.
#[test]
fn the_gear_sits_left_of_the_quit_button_with_a_gap_and_is_clickable_there() {
    let (buffer, close, hits) = title_row(60, 20, false);
    let close = close.expect("quit button drawn");
    let row: String = (0..60).map(|x| buffer[(x, 0)].symbol()).collect();
    let gear_x = (0..60u16)
        .find(|&x| buffer[(x, 0)].symbol() == "\u{2699}")
        .expect("gear drawn");
    assert!(gear_x < close.x, "{row:?}");
    assert!(
        close.x - (gear_x + 1) >= 2,
        "at least one blank cell between the gear's label and the quit button: {row:?}"
    );
    assert!(
        (gear_x + 1..close.x).all(|x| buffer[(x, 0)].symbol() == " "),
        "{row:?}"
    );
    assert_eq!(find_hit_target(&hits, gear_x, 0), Some(HitTarget::Gear));
    assert_eq!(
        find_hit_target(&hits, close.x + 1, 0),
        Some(HitTarget::CloseApp)
    );
    // The cells between the two buttons hit neither: no mis-click quits.
    for x in (gear_x + 2)..close.x {
        assert_eq!(find_hit_target(&hits, x, 0), None, "col {x}");
    }
    assert!(
        (0..gear_x.saturating_sub(1)).all(|x| find_hit_target(&hits, x, 0).is_none()),
        "no gear anywhere else"
    );
}

/// A gear that would run into the title gives way, and registers nothing.
#[test]
fn a_title_bar_without_room_for_the_gear_registers_no_gear() {
    // 14 wide: the quit button fits, the gear (3 cells + gap) does not
    // clear a title that ends at column 8.
    let (buffer, close, hits) = title_row(14, 8, false);
    assert!(close.is_some());
    let row: String = (0..14).map(|x| buffer[(x, 0)].symbol()).collect();
    assert!(!row.contains('\u{2699}'), "{row:?}");
    assert!(hits.iter().all(|(_, t)| *t != HitTarget::Gear), "{hits:?}");
}

/// Hover feedback: the maintainer's report
/// named the REMOTE section's `+` and the top-right `\u{D7}` as
/// having "a filled background even when NOT hovered" - plain glyph
/// buttons must be plain (no background fill) at rest and only gain
/// one on hover. The gear is the same design (`plain_button_style`),
/// so it is pinned here too rather than only the two named ones.
/// Falsified by reverting `render_gear`/`render_close_app_button` to
/// unconditionally fill their background.
#[test]
fn the_gear_and_close_app_button_are_plain_glyphs_at_rest_and_filled_on_hover() {
    use ratatui::backend::TestBackend;

    let gear_cell = |hovered: bool| {
        let (buffer, close, _) = title_row(60, 20, hovered);
        let close = close.unwrap();
        let x = (0..close.x)
            .find(|&x| buffer[(x, 0)].symbol() == "\u{2699}")
            .expect("gear drawn");
        buffer[(x, 0)].clone()
    };
    let idle = gear_cell(false);
    let hovered = gear_cell(true);
    assert_eq!(idle.style().bg, Some(Color::Reset), "plain at rest");
    assert_eq!(hovered.style().bg, Some(th().ui.button_hover));
    assert_ne!(
        idle.style().bg,
        hovered.style().bg,
        "hovering the gear must change its rendered style"
    );

    let render_close = |hovered: bool| {
        let mut terminal = Terminal::new(TestBackend::new(40, 6)).unwrap();
        let mut hits: Vec<(Rect, HitTarget)> = Vec::new();
        terminal
            .draw(|frame| {
                render_close_app_button(frame, Rect::new(0, 0, 40, 6), 4, hovered, &mut hits);
            })
            .unwrap();
        terminal.backend().buffer().clone()
    };
    let idle = render_close(false);
    let hovered = render_close(true);
    // The close glyph's own cell, read back the same way the "close
    // control is registered where the cross is drawn" test does.
    let cross_x = (0..40)
        .find(|&x| idle[(x, 0)].symbol() == "\u{D7}")
        .expect("the cross must be drawn");
    assert_eq!(
        idle[(cross_x, 0)].style().bg,
        Some(Color::Reset),
        "plain at rest: close button"
    );
    assert_eq!(hovered[(cross_x, 0)].style().bg, Some(th().ui.button_hover));
    assert_ne!(
        idle[(cross_x, 0)].style().bg,
        hovered[(cross_x, 0)].style().bg
    );
}

/// Every "plain glyph" button's hovered style must keep the glyph
/// legible: foreground and background must never collide. Coordinator
/// report (observed in a release build): hovering the REMOTE
/// `+` made it disappear. It turned out to be a width-gating bug (see
/// `render_add_remote_button`'s doc comment), not a color collision,
/// but this pins the color half of that guarantee for every plain
/// button so a future change can't reintroduce a fg==bg regression
/// either. Falsified by setting `plain_button_style`'s hovered arm to
/// `.fg(PLAIN_BUTTON_HOVER_BG)`.
#[test]
fn plain_button_style_never_collides_foreground_and_background() {
    for hovered in [false, true] {
        let style = plain_button_style(hovered);
        let bg = style.bg.unwrap_or(Color::Reset);
        let fg = style.fg.unwrap_or(Color::Reset);
        assert_ne!(fg, bg, "hovered={hovered}: {style:?}");
    }
}

/// The reported bug itself: on a sidebar too narrow for the
/// hovered tooltip label ("+ Add remote") but wide enough for the
/// resting glyph ("+"), hovering must NOT make the button disappear.
/// Before the fix, the whole button (glyph included) was gated on
/// room for whichever label was about to be drawn, so the gate that
/// passed at rest failed the instant the pointer arrived.
#[test]
fn the_add_remote_button_never_disappears_on_hover_even_when_narrow() {
    use ratatui::backend::TestBackend;

    // Wide enough for " + " (3 cols) plus a title, nowhere near wide
    // enough for " + Add remote " (15 cols).
    let area = Rect::new(0, 0, 20, 1);
    let section_title_width = 8;

    let render = |hovered: bool| {
        let mut terminal = Terminal::new(TestBackend::new(20, 1)).unwrap();
        let mut hits: Vec<(Rect, HitTarget)> = Vec::new();
        terminal
            .draw(|frame| {
                render_add_remote_button(
                    frame,
                    area,
                    section_title_width,
                    hovered,
                    false,
                    &mut hits,
                );
            })
            .unwrap();
        (terminal.backend().buffer().clone(), hits)
    };

    let (idle_buf, idle_hits) = render(false);
    let (hovered_buf, hovered_hits) = render(true);

    let has_plus = |buf: &ratatui::buffer::Buffer| (0..20).any(|x| buf[(x, 0)].symbol() == "+");
    assert!(has_plus(&idle_buf), "the + must be drawn at rest");
    assert!(
        has_plus(&hovered_buf),
        "the + must still be drawn while hovered, even though the full \
         tooltip label doesn't fit"
    );
    assert_eq!(idle_hits.len(), 1, "the button must stay clickable at rest");
    assert_eq!(
        hovered_hits.len(),
        1,
        "the button must stay clickable while hovered - this is exactly \
         the maintainer's report: it went unclickable because it went unrendered"
    );

    // And the hover fill must actually be visible under the glyph
    // (not the same as the resting/default background).
    let plus_x = (0..20)
        .find(|&x| hovered_buf[(x, 0)].symbol() == "+")
        .unwrap();
    let cell = &hovered_buf[(plus_x, 0)];
    assert_ne!(
        cell.style().fg,
        cell.style().bg,
        "hovered + must stay legible: {cell:?}"
    );
}

/// The wide case: plenty of room, so hovering should still grow into
/// the "+ Add remote" tooltip label the way it always has.
#[test]
fn the_add_remote_button_shows_the_tooltip_label_when_there_is_room() {
    use ratatui::backend::TestBackend;

    let area = Rect::new(0, 0, 60, 1);
    let mut terminal = Terminal::new(TestBackend::new(60, 1)).unwrap();
    let mut hits: Vec<(Rect, HitTarget)> = Vec::new();
    terminal
        .draw(|frame| render_add_remote_button(frame, area, 8, true, false, &mut hits))
        .unwrap();
    let buffer = terminal.backend().buffer();
    let row: String = (0..60).map(|x| buffer[(x, 0)].symbol()).collect();
    assert!(row.contains("Add remote"), "{row:?}");
}

/// The command palette's rows resolve to the command the *scrolled*
/// list is showing, on both axes, and nothing outside the list block
/// resolves at all.
///
/// The scrolled case is the point: the popup shows a window into the
/// matches, so row 0 is `window_start`, not command 0. A hit test
/// that forgot that would run whatever command happened to share a
/// screen position with the one the user pointed at - the worst
/// possible failure for a surface whose entries include
/// `reset --hard`.
#[test]
fn a_palette_row_resolves_to_the_command_the_scrolled_window_shows() {
    let popup = PalettePopup {
        popup: Rect::new(10, 4, 40, 20),
        list: Rect::new(10, 7, 40, 10),
        window_start: 12,
        count: 30,
        segments: [Rect::ZERO; 4],
    };

    assert_eq!(palette_row_at(&popup, 20, 7), Some(12), "the top row");
    assert_eq!(palette_row_at(&popup, 20, 16), Some(21), "the last row");
    // Above and below the list block.
    assert_eq!(palette_row_at(&popup, 20, 6), None);
    assert_eq!(palette_row_at(&popup, 20, 17), None);
    // The block's own border columns are not rows.
    assert_eq!(palette_row_at(&popup, 10, 7), None);
    assert_eq!(palette_row_at(&popup, 49, 7), None);
    // ...and neither is anything outside it.
    assert_eq!(palette_row_at(&popup, 5, 7), None);
    assert_eq!(palette_row_at(&popup, 60, 7), None);

    // A window that runs past the end of the matches: the rows with
    // no command behind them are not clickable, however much list
    // block is drawn there.
    let short = PalettePopup {
        window_start: 0,
        count: 3,
        ..popup
    };
    assert_eq!(palette_row_at(&short, 20, 9), Some(2));
    assert_eq!(palette_row_at(&short, 20, 10), None);

    // Nothing matched: every row is empty.
    let empty = PalettePopup { count: 0, ..short };
    assert_eq!(palette_row_at(&empty, 20, 7), None);
}

/// Mirrors `the_gear_lands_in_the_toolbars_right_hand_corner_and_is_clickable_there`:
/// the close button lands at the title bar's right-hand corner and
/// resolves to `HitTarget::CloseApp` there, nowhere else.
#[test]
fn the_close_button_lands_in_the_title_bars_right_hand_corner_and_is_clickable_there() {
    use ratatui::backend::TestBackend;

    let mut terminal = Terminal::new(TestBackend::new(60, 1)).unwrap();
    let mut hits: Vec<(Rect, HitTarget)> = Vec::new();
    terminal
        .draw(|frame| {
            render_close_app_button(frame, Rect::new(0, 0, 60, 1), 20, false, &mut hits);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let row: String = (0..60).map(|x| buffer[(x, 0)].symbol()).collect();
    assert!(row.contains('\u{D7}'), "{row:?}");
    assert!(
        row[..40].chars().all(|c| c == ' '),
        "the close button must not be drawn anywhere but the corner: {row:?}"
    );

    let (rect, _) = hits.first().expect("button registered");
    assert_eq!(
        find_hit_target(&hits, rect.x, rect.y),
        Some(HitTarget::CloseApp)
    );
    assert_eq!(find_hit_target(&hits, 20, 0), None);
}

/// Same falsifiable "no room, no button" rule the gear already has --
/// a title so long it reaches the corner leaves nothing to click.
#[test]
fn a_title_bar_too_narrow_for_the_close_button_registers_no_button_at_all() {
    use ratatui::backend::TestBackend;

    let mut terminal = Terminal::new(TestBackend::new(10, 1)).unwrap();
    let mut hits: Vec<(Rect, HitTarget)> = Vec::new();
    terminal
        .draw(|frame| {
            render_close_app_button(frame, Rect::new(0, 0, 10, 1), 9, false, &mut hits);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let row: String = (0..10).map(|x| buffer[(x, 0)].symbol()).collect();
    assert!(!row.contains('\u{D7}'), "{row:?}");
    assert!(hits.is_empty(), "{hits:?}");
}

/// A very long title (`title_end_x` reaching past where the button
/// would land) also suppresses the button - overlapping the title
/// text is exactly the case called out ("must not
/// overlap existing title content at narrow widths").
#[test]
fn a_close_button_overlapping_the_title_text_is_suppressed() {
    use ratatui::backend::TestBackend;

    let mut terminal = Terminal::new(TestBackend::new(20, 1)).unwrap();
    let mut hits: Vec<(Rect, HitTarget)> = Vec::new();
    // title_end_x = 18, right_edge (title_area.right() - 1) = 19,
    // width(" \u{D7} ") = 3 -- 18 + 3 > 19, so nothing is drawn.
    terminal
        .draw(|frame| {
            render_close_app_button(frame, Rect::new(0, 0, 20, 1), 18, false, &mut hits);
        })
        .unwrap();

    assert!(hits.is_empty(), "{hits:?}");
}

/// The badge for each pane lands inside that pane's own area, and a
/// pane too narrow for the 3-cell badge (a collapsed strip, say) is
/// simply skipped rather than drawn truncated.
#[test]
fn navigation_badges_draw_in_each_panes_area_and_skip_narrow_ones() {
    use ratatui::backend::TestBackend;

    let mut terminal = Terminal::new(TestBackend::new(20, 3)).unwrap();
    terminal
        .draw(|frame| {
            render_navigation_badges(
                frame,
                &[
                    (Rect::new(0, 0, 8, 1), '1'),
                    (Rect::new(8, 0, 8, 1), '2'),
                    // Too narrow for the badge -- must be skipped.
                    (Rect::new(16, 0, 2, 1), '3'),
                ],
            );
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let row: String = (0..20).map(|x| buffer[(x, 0)].symbol()).collect();
    assert!(row.contains('1'), "{row:?}");
    assert!(row.contains('2'), "{row:?}");
    assert!(
        !row.contains('3'),
        "the narrow pane must not get a badge: {row:?}"
    );
}

/// kitty draws U+2699 as an empty cell (a text-default emoji), so under kitty
/// and Ghostty the gear is the non-emoji GEAR WITHOUT HUB; every other
/// terminal keeps the standard one. Same width either way.
#[test]
fn the_gear_glyph_follows_the_terminal() {
    use crate::chrome::{gear_label_for, GEAR_LABEL, GEAR_LABEL_KITTY};
    let with = |vars: &'static [(&'static str, &'static str)]| {
        gear_label_for(&move |name: &str| {
            vars.iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| (*v).to_string())
        })
    };
    assert_eq!(with(&[("KITTY_WINDOW_ID", "3")]), GEAR_LABEL_KITTY);
    assert_eq!(with(&[("TERM", "xterm-kitty")]), GEAR_LABEL_KITTY);
    assert_eq!(with(&[("TERM_PROGRAM", "ghostty")]), GEAR_LABEL_KITTY);
    assert_eq!(with(&[("TERM", "xterm-256color")]), GEAR_LABEL);
    assert_eq!(with(&[]), GEAR_LABEL);
    assert_eq!(display_width(GEAR_LABEL), display_width(GEAR_LABEL_KITTY));
}

/// Draws quit, gear and search on a one-row title of `width` cells whose
/// title ends at `title_end_x`.
fn title_row_with_search(
    width: u16,
    title_end_x: u16,
) -> (ratatui::buffer::Buffer, Vec<(Rect, HitTarget)>) {
    use ratatui::backend::TestBackend;

    let mut terminal = Terminal::new(TestBackend::new(width, 1)).unwrap();
    let mut hits: Vec<(Rect, HitTarget)> = Vec::new();
    terminal
        .draw(|frame| {
            let close = render_close_app_button(
                frame,
                Rect::new(0, 0, width, 1),
                title_end_x,
                false,
                &mut hits,
            );
            if let Some(gear) =
                close.and_then(|c| render_gear(frame, c, title_end_x, false, &mut hits))
            {
                render_search_button(frame, gear, title_end_x, false, &mut hits);
            }
        })
        .unwrap();
    (terminal.backend().buffer().clone(), hits)
}

/// The Search button is the magnifier and its label, flush left of the gear
/// and clickable there. Falsified by drawing it anywhere else, dropping the
/// label or registering a different rect.
#[test]
fn the_search_button_sits_flush_left_of_the_gear() {
    let (buffer, hits) = title_row_with_search(60, 10);
    let row: String = (0..60).map(|x| buffer[(x, 0)].symbol()).collect();
    let label = gitpoulpi_i18n::msg::tab_bar::SEARCH.get();
    assert!(row.contains(&format!("\u{2315} {label} ")), "{row:?}");
    let col = |sym: &str| (0..60u16).find(|&x| buffer[(x, 0)].symbol() == sym);
    let gear = col("\u{2699}").expect("gear");
    let mag = col("\u{2315}").expect("magnifier");
    let after = (display_width(label) + 4) as u16;
    assert_eq!(
        gear,
        mag + after,
        "the label, one padding cell each side, rects adjacent"
    );
    assert_eq!(
        find_hit_target(&hits, mag, 0),
        Some(HitTarget::GlobalSearchButton)
    );
    assert_eq!(
        find_hit_target(&hits, gear - 2, 0),
        Some(HitTarget::GlobalSearchButton),
        "the label is clickable"
    );
    assert_eq!(find_hit_target(&hits, gear, 0), Some(HitTarget::Gear));
}

/// Narrow title bar: the label goes first (the magnifier stays), then the
/// button; the gear stays while it fits, and a button with no room is neither
/// drawn nor registered.
#[test]
fn the_search_button_drops_its_label_then_gives_way_before_the_gear() {
    let label = gitpoulpi_i18n::msg::tab_bar::SEARCH.get();
    let (buffer, _) = title_row_with_search(60, 10);
    let gear = (0..60u16)
        .find(|&x| buffer[(x, 0)].symbol() == "\u{2699}")
        .unwrap();
    // The gear's rect starts one cell before its glyph. Room for the icon
    // (3 cells) but not for the label.
    let limit = gear - 1 - 4;
    let (buffer, hits) = title_row_with_search(60, limit);
    let row: String = (0..60).map(|x| buffer[(x, 0)].symbol()).collect();
    assert!(
        row.contains('\u{2699}') && row.contains('\u{2315}'),
        "{row:?}"
    );
    assert!(!row.contains(label), "label dropped: {row:?}");
    assert!(hits
        .iter()
        .any(|(_, t)| *t == HitTarget::GlobalSearchButton));
    // No room for even the icon.
    let (buffer, hits) = title_row_with_search(60, gear - 1 - 1);
    let row: String = (0..60).map(|x| buffer[(x, 0)].symbol()).collect();
    assert!(row.contains('\u{2699}'), "gear stays: {row:?}");
    assert!(!row.contains('\u{2315}'), "search gave way: {row:?}");
    assert!(hits
        .iter()
        .all(|(_, t)| *t != HitTarget::GlobalSearchButton));
}
