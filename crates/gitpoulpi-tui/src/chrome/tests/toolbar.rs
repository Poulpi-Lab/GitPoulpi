use super::*;

/// Draws the toolbar exactly as the event loop does, one
/// row `width` wide, and returns the row's text plus what was
/// registered.
fn draw_toolbar(
    width: u16,
    hovered: Option<&HitTarget>,
) -> (String, ratatui::buffer::Buffer, ToolbarRender) {
    use ratatui::backend::TestBackend;
    let area = Rect::new(0, 0, width, 1);
    let mut terminal = Terminal::new(TestBackend::new(width, 1)).unwrap();
    let mut out = None;
    terminal
        .draw(|frame| {
            let toolbar = toolbar_render(area, hovered, ToolbarEnabled::ALL);
            frame.render_clipped(Paragraph::new(toolbar.line.clone()), area);
            out = Some(toolbar);
        })
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let text = (0..width)
        .map(|x| buf[(x, 0)].symbol().to_string())
        .collect();
    (text, buf, out.unwrap())
}

/// Golden, wide: every button, no `⋯` (the gear lives in the title bar).
/// (The second column of the two-cell `🔖` reads back as a blank,
/// hence the two spaces before "Tag"; three spaces between buttons are
/// each chip's own trailing margin, the gap, the next chip's margin.)
#[test]
fn the_toolbar_at_a_wide_width_shows_every_button_and_no_overflow() {
    let (text, _, toolbar) = draw_toolbar(110, None);
    assert_eq!(
        text,
        "     \u{21B6} Undo \u{25BE}   \u{21B7} Redo   \u{21E9} Pull \u{25BE}   \
         \u{21E7} Push \u{25BE}   \u{2387} Branch   \u{1F516}  Tag   \u{1F4E5}  Stash   \
         \u{1F4E4}  Pop   \u{25A3} LFS \u{25BE}   \u{2315} Search     "
    );
    assert!(!text.contains("Fetch"), "Fetch lives in the Pull dropdown");
    assert!(toolbar.overflow.is_none());
    assert!(!text.contains('\u{22EF}'));
}

fn button_span(
    hits: &[(Rect, HitTarget)],
    first: ToolbarButton,
    last: ToolbarButton,
) -> (u16, u16) {
    let find = |b: ToolbarButton| {
        hits.iter()
            .find(|(_, t)| *t == HitTarget::Toolbar(b))
            .map(|(r, _)| *r)
            .unwrap()
    };
    let (a, z) = (find(first), find(last));
    (a.x, z.x + z.width)
}

/// U3: with room to spare the Undo..Search group is centered in the bar
/// (within a column), at several widths, and never touches the gear.
#[test]
fn the_button_group_is_centered_at_several_widths() {
    for width in [110u16, 130, 160, 200] {
        let (_, _, tb) = draw_toolbar(width, None);
        assert!(tb.overflow.is_none(), "{width}");
        let (l, r) = button_span(&tb.hits, ToolbarButton::Undo, ToolbarButton::Search);
        let left = i32::from(l);
        let right = i32::from(width) - i32::from(r);
        assert!((left - right).abs() <= 1, "{width}: {left} vs {right}");
    }
}

/// A flowing chip (`ui.toolbar_pr_pinned` off) is the first item of the
/// group, which the alignment then places as a whole: centered here, the
/// chip never overlaps the buttons and nothing touches the profile field's
/// reserve.
#[test]
fn a_flowing_chip_leads_the_group_and_the_group_is_centered() {
    let chip = linked_chip();
    for width in [96u16, 100, 110, 120, 160] {
        let (_, _, tb) = draw_toolbar_with_chip(width, None, Some(&chip));
        if tb.overflow.is_some() {
            continue;
        }
        let chip_rect = tb.hits[0].0;
        assert_eq!(tb.hits[0].1, HitTarget::Hosting(HostingHit::LinkedPrChip));
        let (l, r) = button_span(&tb.hits, ToolbarButton::Undo, ToolbarButton::Search);
        assert_eq!(l, chip_rect.right() + 1, "{width}: adjacent to the chip");
        assert!(r + display_width(GEAR_LABEL) as u16 <= width, "{width}");
    }
    let (_, _, tb) = draw_toolbar_with_chip(160, None, Some(&chip));
    let first = tb.hits[0].0.x;
    let (_, r) = button_span(&tb.hits, ToolbarButton::Undo, ToolbarButton::Search);
    assert!((i32::from(first) - (160 - i32::from(r))).abs() <= 3);
}

/// U3: undo off hides Undo/Redo and the rest are still centered.
#[test]
fn centring_works_with_undo_hidden() {
    let area = Rect::new(0, 0, 120, 1);
    let tb = toolbar_render(
        area,
        None,
        ToolbarEnabled {
            undo_hidden: true,
            ..ToolbarEnabled::ALL
        },
    );
    assert!(!tb
        .hits
        .iter()
        .any(|(_, t)| *t == HitTarget::Toolbar(ToolbarButton::Undo)));
    let (l, r) = button_span(&tb.hits, ToolbarButton::Pull, ToolbarButton::Search);
    assert!((i32::from(l) - (120 - i32::from(r))).abs() <= 1);
}

/// U3: when buttons overflow, the bar stays packed left and the `⋯`
/// plus the gear still fit.
#[test]
fn an_overflowing_toolbar_stays_left_packed() {
    for width in [40u16, 60, 80] {
        let (_, _, tb) = draw_toolbar(width, None);
        if let Some((rect, _)) = &tb.overflow {
            assert!(
                rect.x + rect.width + display_width(GEAR_LABEL) as u16 <= width,
                "{width}"
            );
            let first = tb.hits.iter().map(|(r, _)| r.x).min().unwrap();
            assert_eq!(first, 0, "{width}");
        }
    }
}

/// Pop is grayed (icon and label) while there is no stash, and lit
/// like Stash once there is one; its hit rect is registered either way
/// so a click can explain why nothing happened.
#[test]
fn pop_is_grayed_without_a_stash_and_lit_with_one() {
    use ratatui::backend::TestBackend;
    let area = Rect::new(0, 0, 110, 1);
    let paint = |pop_enabled: bool| {
        let mut terminal = Terminal::new(TestBackend::new(110, 1)).unwrap();
        let mut hits = Vec::new();
        terminal
            .draw(|frame| {
                let toolbar = toolbar_render(
                    area,
                    None,
                    ToolbarEnabled {
                        pop: pop_enabled,
                        ..ToolbarEnabled::ALL
                    },
                );
                frame.render_clipped(Paragraph::new(toolbar.line.clone()), area);
                hits = toolbar.hits;
            })
            .unwrap();
        (terminal.backend().buffer().clone(), hits)
    };
    for (enabled, want) in [(false, th().ui.muted), (true, Color::White)] {
        let (buf, hits) = paint(enabled);
        let rect = hits
            .iter()
            .find(|(_, t)| *t == HitTarget::Toolbar(ToolbarButton::Pop))
            .expect("Pop is registered either way")
            .0;
        let word = (rect.x..rect.right())
            .find(|x| buf[(*x, 0)].symbol() == "P")
            .unwrap();
        assert_eq!(buf[(word, 0)].fg, want, "enabled={enabled}");
    }
    let stash_fg = |buf: &ratatui::buffer::Buffer, hits: &[(Rect, HitTarget)]| {
        let r = hits
            .iter()
            .find(|(_, t)| *t == HitTarget::Toolbar(ToolbarButton::Stash))
            .unwrap()
            .0;
        let x = (r.x..r.right())
            .find(|x| buf[(*x, 0)].symbol() == "S")
            .unwrap();
        buf[(x, 0)].fg
    };
    let (buf, hits) = paint(false);
    assert_eq!(stash_fg(&buf, &hits), Color::White);
}

fn draw_toolbar_with_chip(
    width: u16,
    hovered: Option<&HitTarget>,
    chip: Option<&ToolbarChip>,
) -> (String, ratatui::buffer::Buffer, ToolbarRender) {
    draw_toolbar_placed(width, hovered, chip, ToolbarPlacement::LEGACY)
}

fn draw_toolbar_placed(
    width: u16,
    hovered: Option<&HitTarget>,
    chip: Option<&ToolbarChip>,
    place: ToolbarPlacement,
) -> (String, ratatui::buffer::Buffer, ToolbarRender) {
    use ratatui::backend::TestBackend;
    let area = Rect::new(0, 0, width, 1);
    let mut terminal = Terminal::new(TestBackend::new(width, 1)).unwrap();
    let mut out = None;
    terminal
        .draw(|frame| {
            let toolbar = toolbar_render_with(area, hovered, chip, ToolbarEnabled::ALL, place);
            frame.render_clipped(Paragraph::new(toolbar.line.clone()), area);
            out = Some(toolbar);
        })
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let text = (0..width)
        .map(|x| buf[(x, 0)].symbol().to_string())
        .collect();
    (text, buf, out.unwrap())
}

fn linked_chip() -> ToolbarChip {
    ToolbarChip {
        text: "GL !53".to_string(),
        fg: Color::Rgb(20, 22, 30),
        bg: Some(th().refs.local),
        hit: HostingHit::LinkedPrChip,
    }
}

/// Pinned (the default), the chip ends the row left of the profile
/// field's reserve, its hit rect is the painted label, and the buttons stay
/// exactly where they are without a chip (one walk).
#[test]
fn the_pinned_pr_chip_is_at_the_far_end_and_every_rect_matches_what_was_painted() {
    let chip = linked_chip();
    let (with, _, toolbar) =
        draw_toolbar_placed(120, None, Some(&chip), ToolbarPlacement::default());
    let (without, _, plain) = draw_toolbar_placed(120, None, None, ToolbarPlacement::default());
    let chip_hit = toolbar
        .hits
        .iter()
        .find(|(_, t)| *t == HitTarget::Hosting(HostingHit::LinkedPrChip))
        .expect("chip registered");
    // 120 wide, 3 reserved for the gear and one gap: the chip ends at 116.
    assert_eq!(chip_hit.0, Rect::new(108, 0, 8, 1));
    let painted: String = with.chars().skip(108).take(8).collect();
    assert_eq!(painted, " GL !53 ");
    for (rect, target) in &toolbar.hits {
        if let HitTarget::Toolbar(b) = target {
            let old = plain
                .hits
                .iter()
                .find(|(_, t)| *t == HitTarget::Toolbar(*b))
                .unwrap()
                .0;
            assert_eq!(*rect, old, "{b:?}");
            assert!(rect.right() < chip_hit.0.x);
        }
    }
    // Start-aligned: the first button is at the left edge.
    assert!(without.starts_with(" \u{21B6} Undo"), "{without:?}");
    // The gear is not on the toolbar any more.
    assert!(!with.contains('\u{2699}'));
}

/// Start, center and end move the buttons, never a pinned chip.
#[test]
fn the_alignment_moves_the_buttons_and_never_the_pinned_chip() {
    use gitpoulpi_config::ToolbarAlign as A;
    let chip = linked_chip();
    let at = |align: A| {
        let place = ToolbarPlacement {
            align,
            ..ToolbarPlacement::default()
        };
        let (_, _, tb) = draw_toolbar_placed(160, None, Some(&chip), place);
        let (l, r) = button_span(&tb.hits, ToolbarButton::Undo, ToolbarButton::Search);
        let c = tb
            .hits
            .iter()
            .find(|(_, t)| matches!(t, HitTarget::Hosting(_)))
            .unwrap()
            .0;
        (l, r, c)
    };
    let (sl, _, sc) = at(A::Start);
    let (cl, _, cc) = at(A::Center);
    let (el, er, ec) = at(A::End);
    assert_eq!(sl, 0, "start packs against the left edge");
    assert!(sl < cl && cl < el, "{sl} {cl} {el}");
    assert!(sc == cc && cc == ec, "the pinned chip stays put");
    assert!(er < ec.x, "the end-aligned group stops before the chip");
    assert_eq!(er + 1, ec.x, "and ends one gap before it");
}

#[test]
fn hovering_the_chip_uses_the_strong_fill_and_resting_uses_its_state_color() {
    let chip = linked_chip();
    let place = ToolbarPlacement::default();
    let (_, rest, _) = draw_toolbar_placed(120, None, Some(&chip), place);
    // The pinned chip's label starts at x = 108; its text is at 109 onward.
    assert_eq!(rest[(110, 0)].bg, th().refs.local);
    let hovered = HitTarget::Hosting(HostingHit::LinkedPrChip);
    let (_, hot, _) = draw_toolbar_placed(120, Some(&hovered), Some(&chip), place);
    assert_eq!(Some(hot[(110, 0)].bg), plain_button_style(true).bg);

    // The dim `+ PR` chip has no fill at rest.
    let new_pr = ToolbarChip {
        text: "+ PR".to_string(),
        fg: th().ui.muted,
        bg: None,
        hit: HostingHit::NewPrChip,
    };
    let (text, rest, tb) = draw_toolbar_placed(120, None, Some(&new_pr), place);
    let x = tb
        .hits
        .iter()
        .find(|(_, t)| matches!(t, HitTarget::Hosting(_)))
        .unwrap()
        .0
        .x;
    let painted: String = text.chars().skip(x as usize).take(6).collect();
    assert_eq!(painted, " + PR ");
    assert_ne!(Some(rest[(x + 2, 0)].bg), plain_button_style(true).bg);
    let hovered = HitTarget::Hosting(HostingHit::NewPrChip);
    let (_, hot, _) = draw_toolbar_placed(120, Some(&hovered), Some(&new_pr), place);
    assert_eq!(Some(hot[(x + 2, 0)].bg), plain_button_style(true).bg);
}

/// Overflow keeps working with the chip present, the chip survives
/// the buttons collapsing, and a bar with no room drops the chip
/// rather than the buttons' `⋯`.
#[test]
fn the_chip_outlasts_the_buttons_and_gives_way_only_when_nothing_would_fit() {
    let chip = linked_chip();
    for width in 40..130u16 {
        let (text, _, toolbar) = draw_toolbar_with_chip(width, None, Some(&chip));
        let has_chip = toolbar
            .hits
            .iter()
            .any(|(_, t)| *t == HitTarget::Hosting(HostingHit::LinkedPrChip));
        if width >= 9 + TOOLBAR_MIN_BUTTON_ROOM {
            assert!(has_chip, "width {width}: {text:?}");
        }
        for (rect, _) in &toolbar.hits {
            assert!(rect.right() <= width, "width {width}: {rect:?}");
        }
    }
    let (text, _, toolbar) = draw_toolbar_with_chip(56, None, Some(&chip));
    assert!(text.contains('\u{22EF}'), "{text:?}");
    assert!(toolbar.overflow.is_some());
    let (_, _, toolbar) = draw_toolbar_with_chip(20, None, Some(&chip));
    assert!(!toolbar
        .hits
        .iter()
        .any(|(_, t)| matches!(t, HitTarget::Hosting(_))));
}

/// Golden, narrow: the least important buttons (Redo, Tag, Stash, Pop,
/// Search) collapse into one `⋯` at the end, the sync trio, Branch and
/// Undo stay.
#[test]
fn the_toolbar_at_a_narrow_width_collapses_the_least_important_buttons_into_one_overflow() {
    let (text, _, toolbar) = draw_toolbar(56, None);
    assert_eq!(
        text,
        " \u{21B6} Undo \u{25BE}   \u{21E9} Pull \u{25BE}   \u{21E7} Push \u{25BE}   \
         \u{2387} Branch   \u{22EF}          "
    );
    let (rect, hidden) = toolbar.overflow.expect("an overflow button");
    assert_eq!(
        hidden,
        vec![
            OverflowItem::Toolbar(ToolbarButton::Redo),
            OverflowItem::Toolbar(ToolbarButton::Tag),
            OverflowItem::Toolbar(ToolbarButton::Stash),
            OverflowItem::Toolbar(ToolbarButton::Pop),
            OverflowItem::Toolbar(ToolbarButton::Lfs),
            OverflowItem::Toolbar(ToolbarButton::Search),
        ]
    );
    // The `⋯` is registered exactly where it was painted.
    assert_eq!(
        find_hit_target(&toolbar.hits, rect.x + 1, 0),
        Some(HitTarget::BarOverflow(OverflowBar::Toolbar))
    );
    assert_eq!(text.chars().nth(rect.x as usize + 1), Some('\u{22EF}'));
}

/// A hidden split button takes its dropdown into the menu with it,
/// and the dropdown then hangs under the `⋯`.
#[test]
fn a_hidden_push_lists_its_dropdown_too_and_the_dropdown_hangs_under_the_overflow() {
    let (_, _, toolbar) = draw_toolbar(20, None);
    let (rect, hidden) = toolbar.overflow.expect("an overflow button");
    assert!(hidden.contains(&OverflowItem::Toolbar(ToolbarButton::Push)));
    assert!(hidden.contains(&OverflowItem::ToolbarDropdown(ToolbarButton::Push)));
    assert_eq!(toolbar.push_anchor, (rect.x, 1));
}

/// Every rect the toolbar registers covers the cells showing its own
/// label - the `🔖` is two cells wide, and counting it as one used to
/// shift every button after Tag one cell left of its paint.
#[test]
fn every_toolbar_hit_rect_sits_on_its_own_label() {
    let (text, buf, toolbar) = draw_toolbar(110, None);
    let _ = text;
    for (rect, target) in &toolbar.hits {
        let painted: String = (rect.x..rect.right())
            .map(|x| buf[(x, 0)].symbol().to_string())
            .collect();
        if let HitTarget::Toolbar(b) = target {
            let label = toolbar_text(*b);
            let word = label.split(' ').next_back().unwrap();
            assert!(painted.contains(word), "{b:?}: {painted:?}");
        }
    }
}

/// The `⋯` lights up under the pointer, and nothing else changes.
#[test]
fn the_toolbar_overflow_button_lights_up_on_hover() {
    let (_, idle, toolbar) = draw_toolbar(56, None);
    let (rect, _) = toolbar.overflow.unwrap();
    let (_, hovered, _) = draw_toolbar(56, Some(&HitTarget::BarOverflow(OverflowBar::Toolbar)));
    let cell = (rect.x + 1, 0);
    assert_eq!(idle[cell].style().bg, Some(th().ui.button));
    assert_eq!(hovered[cell].style().bg, Some(th().ui.button_hover));
}

/// The menu the `⋯` opens: one row per hidden button, and choosing a
/// row - by click or by Enter, both `Menu::activate` - yields the
/// very `HitTarget` the button itself registers, which the event loop
/// then dispatches through the button's own click code.
#[test]
fn choosing_an_overflow_menu_row_dispatches_the_hidden_buttons_own_click() {
    let (_, _, toolbar) = draw_toolbar(56, None);
    let (rect, hidden) = toolbar.overflow.unwrap();
    let mut menu = overflow_menu_for(&hidden, rect);
    assert_eq!(menu.row_count(), hidden.len());
    assert_eq!(menu.row_label(3), "\u{1F4E4} Pop");
    match menu.activate(3) {
        MenuOutcome::Run(item) => {
            assert_eq!(item.hit_target(), HitTarget::Toolbar(ToolbarButton::Pop))
        }
        other => panic!("expected Run, got {other:?}"),
    }
    // The keyboard path: Down to the last row, Enter.
    let mut menu = overflow_menu_for(&hidden, rect);
    assert_eq!(menu.on_key(KeyCode::End), MenuOutcome::Stay);
    assert_eq!(
        menu.on_key(KeyCode::Enter),
        MenuOutcome::Run(OverflowItem::Toolbar(ToolbarButton::Search))
    );
}

/// The toolbar is catalog text: French labels draw in French, the git-term
/// checkbox swaps `Stash` for `Remiser`, and English is what a test that
/// sets nothing gets.
#[test]
fn the_toolbar_follows_the_language_and_the_git_vocabulary() {
    use gitpoulpi_i18n::{override_git_terms, override_lang, Lang};
    let (en, _, _) = draw_toolbar(130, None);
    assert!(en.contains("Search") && en.contains("Stash"), "{en:?}");
    let _lang = override_lang(Lang::Fr);
    let (fr, _, _) = draw_toolbar(130, None);
    assert!(fr.contains("Chercher") && fr.contains("Annuler"), "{fr:?}");
    assert!(fr.contains("Stash") && !fr.contains("Remiser"), "{fr:?}");
    let _terms = override_git_terms(true);
    let (local, _, _) = draw_toolbar(130, None);
    assert!(
        local.contains("Remiser") && local.contains("Tirer") && local.contains("Pousser"),
        "{local:?}"
    );
    // Tooltips come from the same catalog.
    assert_eq!(
        crate::tooltip::hint_for(&HitTarget::Toolbar(ToolbarButton::Stash)),
        Some("Remiser les modifications en cours")
    );
}

/// The toolbar budget (plan 6.2): every label fits 9 cells, in every
/// language that ships, in both git vocabularies; the hit rects still sit
/// on the labels at the width the longest French set needs.
#[test]
fn toolbar_labels_fit_the_nine_cell_budget_in_every_shipped_language() {
    use gitpoulpi_i18n::{override_git_terms, override_lang, SHIPPED};
    for lang in SHIPPED {
        for translated in [false, true] {
            let _l = override_lang(*lang);
            let _t = override_git_terms(translated);
            for (_, label, button) in TOOLBAR_BUTTONS {
                let w = display_width(label.get());
                assert!(
                    w <= 9,
                    "{button:?} in {lang:?} (git terms translated: {translated}): {:?} is {w} cells",
                    label.get()
                );
            }
        }
    }
}

/// The `LFS ▾` button is one hit rect over its label and its painted
/// arrow (a click anywhere opens the dropdown), has no separate arrow
/// target, sits between Pop and Search, and leaves the bar (not just
/// grays) when LFS is turned off.
#[test]
fn the_lfs_button_is_one_hit_rect_and_hides_when_lfs_is_off() {
    let (text, _, tb) = draw_toolbar(110, None);
    let lfs: Vec<_> = tb
        .hits
        .iter()
        .filter(|(_, t)| *t == HitTarget::Toolbar(ToolbarButton::Lfs))
        .collect();
    assert_eq!(lfs.len(), 1);
    let rect = lfs[0].0;
    let painted: String = text
        .chars()
        .skip(rect.x as usize)
        .take(rect.width as usize)
        .collect();
    assert!(
        painted.contains("LFS") && painted.contains('\u{25BE}'),
        "{painted:?}"
    );
    assert!(!tb
        .hits
        .iter()
        .any(|(_, t)| *t == HitTarget::ToolbarDropdown(ToolbarButton::Lfs)));
    assert_eq!(tb.lfs_anchor, (rect.x, 1));
    let pop = tb
        .hits
        .iter()
        .find(|(_, t)| *t == HitTarget::Toolbar(ToolbarButton::Pop))
        .unwrap()
        .0;
    let search = tb
        .hits
        .iter()
        .find(|(_, t)| *t == HitTarget::Toolbar(ToolbarButton::Search))
        .unwrap()
        .0;
    assert!(pop.x + pop.width <= rect.x && rect.x + rect.width <= search.x);

    let hidden = toolbar_render(
        Rect::new(0, 0, 110, 1),
        None,
        ToolbarEnabled {
            lfs_hidden: true,
            ..ToolbarEnabled::ALL
        },
    );
    assert!(!hidden
        .hits
        .iter()
        .any(|(_, t)| *t == HitTarget::Toolbar(ToolbarButton::Lfs)));
}

// ----------------------------------------------------------------- column

/// Draws the toolbar as a column of `height` rows, as the event loop does.
fn draw_column(
    height: u16,
    chip: Option<&ToolbarChip>,
    enabled: ToolbarEnabled,
    place: ToolbarPlacement,
    hovered: Option<&HitTarget>,
) -> (ratatui::buffer::Buffer, ToolbarRender, Rect) {
    use ratatui::backend::TestBackend;
    // No profile field unless a test asks (`draw_column_with_profile`).
    let enabled = ToolbarEnabled {
        right_reserve: 0,
        ..enabled
    };
    let width = toolbar_column_width(enabled);
    let area = Rect::new(0, 0, width, height);
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    let mut out = None;
    terminal
        .draw(|frame| {
            let tb = toolbar_column(area, hovered, chip, enabled, place);
            tb.paint(frame, area);
            out = Some(tb);
        })
        .unwrap();
    (terminal.backend().buffer().clone(), out.unwrap(), area)
}

fn left_place() -> ToolbarPlacement {
    ToolbarPlacement {
        edge: gitpoulpi_config::ToolbarPosition::Left,
        ..ToolbarPlacement::default()
    }
}

fn row_text(buf: &ratatui::buffer::Buffer, y: u16) -> String {
    (0..buf.area.width)
        .map(|x| buf[(x, y)].symbol().to_string())
        .collect()
}

#[test]
fn the_column_is_as_wide_as_its_widest_button_within_bounds() {
    let w = toolbar_column_width(ToolbarEnabled::ALL);
    assert!((8..=COLUMN_MAX_WIDTH).contains(&w), "{w}");
    // " <icon> <label> " plus the arrow: Branch / Search are the widest labels.
    assert!(w >= 10, "{w}");
}

#[test]
fn every_button_is_one_row_of_the_column_with_a_hit_rect_where_it_was_painted() {
    let (buf, tb, area) = draw_column(30, None, ToolbarEnabled::ALL, left_place(), None);
    assert!(tb.overflow.is_none());
    assert!(tb.line.spans.is_empty(), "a column paints rows, not a line");
    let mut seen = Vec::new();
    for (rect, target) in &tb.hits {
        assert!(area.contains((rect.x, rect.y).into()), "{rect:?}");
        assert!(rect.right() <= area.right(), "{rect:?}");
        assert_eq!(rect.height, 1);
        if let HitTarget::Toolbar(b) = target {
            seen.push(*b);
            // The label is in the row the hit rect says.
            let (glyph, label) = toolbar_parts(*b);
            let text = row_text(&buf, rect.y);
            assert!(text.contains(label), "{b:?}: {text:?}");
            assert!(
                text.trim_start().starts_with(glyph.trim()),
                "{b:?}: {text:?}"
            );
        }
    }
    // Every button the toolbar has, once.
    let expected: Vec<_> = TOOLBAR_BUTTONS
        .iter()
        .map(|(_, _, b)| *b)
        .filter(|b| *b != ToolbarButton::Flow)
        .collect();
    for b in &expected {
        assert_eq!(
            seen.iter().filter(|s| *s == b).count(),
            1,
            "{b:?} in {seen:?}"
        );
    }
    // No two rects overlap (the arrow chips sit beside their button).
    for (i, (a, _)) in tb.hits.iter().enumerate() {
        for (b, _) in &tb.hits[i + 1..] {
            assert!(!a.intersects(*b), "{a:?} overlaps {b:?}");
        }
    }
}

#[test]
fn a_split_button_keeps_its_arrow_chip_at_the_end_of_its_row() {
    let (buf, tb, area) = draw_column(30, None, ToolbarEnabled::ALL, left_place(), None);
    for b in [
        ToolbarButton::Undo,
        ToolbarButton::Pull,
        ToolbarButton::Push,
    ] {
        let main = tb
            .hits
            .iter()
            .find(|(_, t)| *t == HitTarget::Toolbar(b))
            .unwrap()
            .0;
        let arrow = tb
            .hits
            .iter()
            .find(|(_, t)| *t == HitTarget::ToolbarDropdown(b))
            .unwrap_or_else(|| panic!("{b:?} has no arrow"))
            .0;
        assert_eq!(main.y, arrow.y);
        assert_eq!(main.right(), arrow.x, "{b:?}");
        assert_eq!(arrow.right(), area.right(), "{b:?}");
        assert_eq!(buf[(arrow.x, arrow.y)].symbol(), "\u{25BE}", "{b:?}");
    }
    // LFS is one rect over the whole row, with the arrow painted in it.
    let lfs = tb
        .hits
        .iter()
        .find(|(_, t)| *t == HitTarget::Toolbar(ToolbarButton::Lfs))
        .unwrap()
        .0;
    assert_eq!((lfs.x, lfs.width), (area.x, area.width));
}

#[test]
fn the_alignment_places_the_column_start_center_and_end() {
    use gitpoulpi_config::ToolbarAlign as A;
    let first_last = |align: A| {
        let place = ToolbarPlacement {
            align,
            ..left_place()
        };
        let (_, tb, _) = draw_column(40, None, ToolbarEnabled::ALL, place, None);
        let ys: Vec<u16> = tb
            .hits
            .iter()
            .filter(|(_, t)| matches!(t, HitTarget::Toolbar(_)))
            .map(|(r, _)| r.y)
            .collect();
        (*ys.iter().min().unwrap(), *ys.iter().max().unwrap())
    };
    let (s0, s1) = first_last(A::Start);
    let (c0, c1) = first_last(A::Center);
    let (e0, e1) = first_last(A::End);
    assert_eq!(s0, 0);
    assert_eq!(e1, 39, "end packs against the bottom row");
    assert_eq!(s1 - s0, c1 - c0);
    assert_eq!(s1 - s0, e1 - e0);
    assert!(s0 < c0 && c0 < e0, "{s0} {c0} {e0}");
    assert!(((c0 - s0) as i32 - (39 - c1) as i32).abs() <= 1, "centered");
}

#[test]
fn a_short_column_collapses_the_least_important_buttons_behind_one_more_row() {
    let (buf, tb, _) = draw_column(6, None, ToolbarEnabled::ALL, left_place(), None);
    let (rect, hidden) = tb.overflow.clone().expect("an overflow row");
    assert_eq!(rect.height, 1);
    assert!(row_text(&buf, rect.y).contains('\u{22EF}'));
    assert!(!hidden.is_empty());
    // Every button is drawn or listed, never both and never neither.
    let drawn: Vec<ToolbarButton> = tb
        .hits
        .iter()
        .filter_map(|(_, t)| match t {
            HitTarget::Toolbar(b) => Some(*b),
            _ => None,
        })
        .collect();
    for (_, _, b) in TOOLBAR_BUTTONS {
        if matches!(b, ToolbarButton::Flow) {
            continue;
        }
        let listed = hidden.contains(&OverflowItem::Toolbar(*b));
        assert_ne!(drawn.contains(b), listed, "{b:?}");
    }
    // Pull and Push outlast the rest, as in a row.
    assert!(drawn.contains(&ToolbarButton::Pull) && drawn.contains(&ToolbarButton::Push));
    // The rows used (buttons plus the overflow row) fit the height.
    let rows: std::collections::HashSet<u16> = tb.hits.iter().map(|(r, _)| r.y).collect();
    assert!(rows.iter().all(|y| *y < 6));
}

#[test]
fn the_pinned_chip_takes_the_bottom_row_and_a_flowing_one_leads_the_group() {
    let chip = linked_chip();
    let (buf, tb, area) = draw_column(30, Some(&chip), ToolbarEnabled::ALL, left_place(), None);
    let c = tb
        .hits
        .iter()
        .find(|(_, t)| matches!(t, HitTarget::Hosting(_)))
        .unwrap()
        .0;
    assert_eq!((c.x, c.y, c.width), (area.x, 29, area.width));
    assert!(row_text(&buf, 29).contains("GL !53"));
    let flows = ToolbarPlacement {
        pr_pinned: false,
        ..left_place()
    };
    let (buf, tb, _) = draw_column(30, Some(&chip), ToolbarEnabled::ALL, flows, None);
    let c = tb
        .hits
        .iter()
        .find(|(_, t)| matches!(t, HitTarget::Hosting(_)))
        .unwrap()
        .0;
    assert_eq!(c.y, 0, "the chip is the first of the group");
    assert!(row_text(&buf, 0).contains("GL !53"));
    let undo = tb
        .hits
        .iter()
        .find(|(_, t)| *t == HitTarget::Toolbar(ToolbarButton::Undo))
        .unwrap()
        .0;
    assert_eq!(undo.y, 1);
}

#[test]
fn a_long_chip_text_is_clipped_to_the_column() {
    let chip = ToolbarChip {
        text: "GL !53 a very long merge request title".to_string(),
        ..linked_chip()
    };
    let (buf, tb, area) = draw_column(30, Some(&chip), ToolbarEnabled::ALL, left_place(), None);
    let c = tb
        .hits
        .iter()
        .find(|(_, t)| matches!(t, HitTarget::Hosting(_)))
        .unwrap()
        .0;
    assert_eq!(c.width, area.width);
    assert_eq!(row_text(&buf, c.y).chars().count(), area.width as usize);
}

#[test]
fn dropdowns_open_beside_a_column_and_level_with_their_button() {
    let (_, tb, area) = draw_column(30, None, ToolbarEnabled::ALL, left_place(), None);
    let push = tb
        .hits
        .iter()
        .find(|(_, t)| *t == HitTarget::Toolbar(ToolbarButton::Push))
        .unwrap()
        .0;
    // `anchored_popup` hangs a popup one row below its anchor: the anchor is
    // the row above the button's.
    assert_eq!(tb.push_anchor, (area.right(), push.y - 1));
    let right = ToolbarPlacement {
        edge: gitpoulpi_config::ToolbarPosition::Right,
        ..ToolbarPlacement::default()
    };
    let (_, tb, area) = draw_column(30, None, ToolbarEnabled::ALL, right, None);
    assert_eq!(tb.push_anchor.0, area.x);
}

#[test]
fn a_bottom_row_hangs_its_dropdowns_from_its_own_row_and_a_top_row_from_the_one_below() {
    let at = |edge| {
        let place = ToolbarPlacement {
            edge,
            ..ToolbarPlacement::default()
        };
        let tb = toolbar_render_with(
            Rect::new(0, 7, 120, 1),
            None,
            None,
            ToolbarEnabled::ALL,
            place,
        );
        tb.push_anchor.1
    };
    assert_eq!(at(gitpoulpi_config::ToolbarPosition::Top), 8);
    assert_eq!(at(gitpoulpi_config::ToolbarPosition::Bottom), 7);
}

#[test]
fn the_profile_field_takes_the_last_row_of_a_tall_column_and_the_chip_the_one_above() {
    use ratatui::backend::TestBackend;
    let enabled = ToolbarEnabled::ALL; // right_reserve > 0: a profile field is shown
    let chip = linked_chip();
    let width = toolbar_column_width(enabled);
    let area = Rect::new(0, 0, width, 30);
    let mut terminal = Terminal::new(TestBackend::new(width, 30)).unwrap();
    let mut tb = None;
    terminal
        .draw(|frame| {
            let t = toolbar_column(area, None, Some(&chip), enabled, left_place());
            t.paint(frame, area);
            tb = Some(t);
        })
        .unwrap();
    let tb = tb.unwrap();
    let c = tb
        .hits
        .iter()
        .find(|(_, t)| matches!(t, HitTarget::Hosting(_)))
        .unwrap()
        .0;
    assert_eq!(c.y, 28, "the profile field owns row 29");
    assert!(tb.hits.iter().all(|(r, _)| r.y < 29));
}
