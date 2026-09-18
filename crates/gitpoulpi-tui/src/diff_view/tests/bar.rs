use super::*;
use crate::th;

#[test]
fn disabled_bar_buttons_all_explain_themselves() {
    // The invariant, which outlives any particular button: a button
    // that is off must say why. `enabled()` is defined as
    // `pending_reason().is_none()`, so the two can never disagree --
    // this pins that they are also both *consulted*.
    for button in DIFF_BAR_BUTTONS {
        assert_eq!(
            button.enabled(),
            button.pending_reason().is_none(),
            "{}",
            button.bar_label()
        );
    }
    // File View / Blame / History went live with `GitBackend`'s
    // read_blob / blame / `LogOptions::paths`; nothing on the bar is
    // inert today.
    assert!(DIFF_BAR_BUTTONS.iter().all(|b| b.enabled()));
}

#[test]
fn the_bar_registers_a_hit_rect_per_drawn_button_and_clips_to_width() {
    let state = DiffViewState::for_commit(test_oid(), 0, PathBuf::new());
    let hits = diff_bar(&state, None, false, 200).buttons;
    // Every button, Edit included -- extended 2026-09-25 to offer
    // Edit on a commit's diff too (it opens the working-tree copy of
    // the path, refused at click time when that copy is missing).
    // (The column selector is one item with two hits.)
    assert_eq!(hits.len(), DIFF_BAR_BUTTONS.len() + 1);
    assert!(hits
        .iter()
        .any(|(_, _, b)| *b == DiffBarButton::EditWorkingFile));
    let working = DiffViewState::for_working(DiffSource::Unstaged, PathBuf::from("a"));
    let working_hits = diff_bar(&working, None, false, 200).buttons;
    assert_eq!(working_hits.len(), DIFF_BAR_BUTTONS.len() + 1);
    assert!(working_hits
        .iter()
        .any(|(_, _, b)| *b == DiffBarButton::EditWorkingFile));
    // Rects are laid out left to right and never overlap.
    for pair in hits.windows(2) {
        assert!(pair[0].0 + pair[0].1 <= pair[1].0);
    }
    // A narrow bar drops trailing buttons rather than drawing them
    // outside the pane (where they'd be invisible but still clickable).
    let narrow_bar = diff_bar(&state, None, false, 20);
    let (narrow_line, narrow) = (narrow_bar.line, narrow_bar.buttons);
    assert!(narrow.len() < DIFF_BAR_BUTTONS.len());
    for (x, w, _) in narrow {
        assert!(x + w <= 20);
    }
    // The encoding dropdown is a button like the others: it names the choice
    // when the bar is wide and goes behind the `⋯` when it is not.
    let narrow_text: String = narrow_line
        .spans
        .iter()
        .map(|s| s.content.as_ref())
        .collect();
    assert!(!narrow_text.contains("Auto"), "got: {narrow_text:?}");
    let wide_line = diff_bar(&state, None, false, 200).line;
    let wide_text: String = wide_line.spans.iter().map(|s| s.content.as_ref()).collect();
    assert!(wide_text.contains("Auto \u{25BE}"));
    let mut chosen = state.clone();
    chosen.encoding = Some(crate::editor_charset::Charset::Windows1252);
    let line = diff_bar(&chosen, None, false, 200).line;
    let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
    assert!(text.contains("Windows-1252 \u{25BE}"), "got: {text:?}");
}

// ---- the responsive Diff bar ----------------------

/// Renders the bar into a `TestBackend` row, as the event loop does.
fn draw_diff_bar(width: u16, overflow_hovered: bool) -> (String, Buffer, DiffBarRender) {
    use ratatui::backend::TestBackend;
    let state = DiffViewState::for_commit(test_oid(), 0, PathBuf::new());
    let mut terminal = Terminal::new(TestBackend::new(width, 1)).unwrap();
    let mut out = None;
    terminal
        .draw(|frame| {
            let bar = diff_bar(&state, None, overflow_hovered, width);
            frame.render_widget(
                ratatui::widgets::Paragraph::new(bar.line.clone()),
                frame.area(),
            );
            out = Some(bar);
        })
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let text = (0..width)
        .map(|x| buf[(x, 0)].symbol().to_string())
        .collect();
    (text, buf, out.unwrap())
}

/// Golden, wide: every button, the encoding dropdown last.
#[test]
fn the_diff_bar_at_a_wide_width_shows_every_button_and_the_encoding() {
    let (text, _, bar) = draw_diff_bar(124, false);
    assert_eq!(
        text.trim_end(),
        " File View   Diff View   Blame   File \u{2502} All   Edit   \u{2191}   \u{2193}   \
         \u{25A5} Inline   \u{25A6} Split   \u{25A4} Hunks   \u{00B6} WS   \
         \u{21B5} Wrap   \u{25A7} Map   Auto \u{25BE}"
    );
    assert!(bar.overflow.is_none());
    assert!(bar.hidden.is_empty());
}

/// Golden, narrow: the hunk arrows stay, then the layouts; the rest
/// goes behind one `⋯` at the end, the encoding dropdown with it. There is no
/// Close button on this bar: the pane title's `\u{2715} Esc` is the only
/// close affordance.
#[test]
fn the_diff_bar_at_a_narrow_width_collapses_the_least_important_into_one_overflow() {
    let (text, _, bar) = draw_diff_bar(40, false);
    assert_eq!(
        text.trim_end(),
        " File \u{2502} All   \u{2191}   \u{2193}   \u{25A5} Inline   \u{22EF}"
    );
    assert_eq!(
        bar.hidden,
        vec![
            DiffBarButton::FileView,
            DiffBarButton::DiffView,
            DiffBarButton::Blame,
            DiffBarButton::EditWorkingFile,
            DiffBarButton::Mode(DiffViewMode::Split),
            DiffBarButton::ToggleHunksOnly,
            DiffBarButton::ToggleWhitespace,
            DiffBarButton::ToggleWordWrap,
            DiffBarButton::ToggleMinimap,
            DiffBarButton::Encoding,
        ]
    );
    let (x, w) = bar.overflow.expect("an overflow button");
    assert_eq!(text.chars().nth(x as usize + 1), Some('\u{22EF}'));
    assert!(x + w <= 40);
    assert!(!text.contains("Auto"));
    // Every hidden button plus every drawn one is the whole bar --
    // Edit included, extended 2026-09-25.
    // (The column selector is one item, drawn as two hits.)
    assert_eq!(
        bar.hidden.len() + bar.buttons.len(),
        DIFF_BAR_BUTTONS.len() + 1
    );
}

/// The Diff bar's `⋯` takes the same chip colors as its buttons and
/// lights up the same way under the pointer.
#[test]
fn the_diff_bar_overflow_button_lights_up_on_hover() {
    let (_, idle, bar) = draw_diff_bar(40, false);
    let (_, hovered, _) = draw_diff_bar(40, true);
    let (x, _) = bar.overflow.unwrap();
    assert_eq!(idle[(x + 1, 0)].style().bg, Some(th().ui.button));
    assert_eq!(hovered[(x + 1, 0)].style().bg, Some(th().ui.button_hover));
}

#[test]
fn the_active_mode_button_is_the_one_matching_the_state() {
    let mut state = DiffViewState::for_commit(test_oid(), 0, PathBuf::new());
    state.mode = DiffViewMode::Split;
    assert!(DiffBarButton::Mode(DiffViewMode::Split).is_active(&state));
    assert!(!DiffBarButton::Mode(DiffViewMode::Inline).is_active(&state));
    assert!(!DiffBarButton::ToggleWordWrap.is_active(&state));
    state.word_wrap = true;
    assert!(DiffBarButton::ToggleWordWrap.is_active(&state));
    // The Hunks toggle is independent of the layout radio pair.
    assert!(DiffBarButton::ToggleHunksOnly.is_active(&state));
    state.hunks_only = false;
    assert!(!DiffBarButton::ToggleHunksOnly.is_active(&state));
    assert!(DiffBarButton::Mode(DiffViewMode::Split).is_active(&state));
}

#[test]
fn the_layout_switches_between_the_two_and_back() {
    let m = DiffViewMode::Inline;
    assert_eq!(m.next(), DiffViewMode::Split);
    assert_eq!(m.next().next(), m);
}

/// The bar in the whole-file state: Hunks drawn un-lit, the layout
/// still lit - the two axes are independent on screen too.
#[test]
fn the_hunks_button_lights_only_while_hunks_only_is_on() {
    let mut state = DiffViewState::for_commit(test_oid(), 0, PathBuf::new());
    let lit = |state: &DiffViewState| {
        let bar = diff_bar(state, None, false, 200);
        let (x, _, _) = *bar
            .buttons
            .iter()
            .find(|(_, _, b)| *b == DiffBarButton::ToggleHunksOnly)
            .unwrap();
        let mut at = 0u16;
        bar.line
            .spans
            .iter()
            .find(|s| {
                let here = at;
                at += display_width(&s.content) as u16;
                here == x
            })
            .unwrap()
            .style
            .bg
            == Some(th().ui.accent)
    };
    assert!(lit(&state));
    state.hunks_only = false;
    assert!(!lit(&state));
}

/// A ratatui color as a measurable theme color (`None` for `Reset`).
fn measurable(c: ratatui::style::Color) -> Option<gitpoulpi_config::theme::ThemeColor> {
    use gitpoulpi_config::theme::ThemeColor;
    use ratatui::style::Color;
    match c {
        Color::Rgb(r, g, b) => Some(ThemeColor::Rgb(r, g, b)),
        Color::White => Some(ThemeColor::Ansi(7)),
        _ => None,
    }
}

/// Maintainer report 2026-10-06: on light themes the bar's buttons kept dark
/// fills under dark text. Every drawn button (rest, hovered, the `⋯`) of
/// every preset reads, 3.0 at least.
#[test]
fn every_bar_button_reads_on_every_preset() {
    let state = DiffViewState::for_working(DiffSource::Unstaged, PathBuf::from("a"));
    let mut bad = Vec::new();
    for spec in gitpoulpi_config::theme::presets() {
        let theme = crate::theme::Theme::from(&spec);
        let _g = crate::theme::override_theme(&theme);
        let mut cases: Vec<Option<DiffBarButton>> = vec![None];
        cases.extend(DIFF_BAR_BUTTONS.iter().map(|b| Some(*b)));
        for hovered in cases {
            for (width, overflow_hovered) in [(200, false), (30, true)] {
                let bar = diff_bar(&state, hovered, overflow_hovered, width);
                for span in &bar.line.spans {
                    let (Some(fg), Some(bg)) = (
                        span.style.fg.and_then(measurable),
                        span.style.bg.and_then(measurable),
                    ) else {
                        continue;
                    };
                    let ratio = gitpoulpi_config::theme::contrast_ratio(fg, bg).unwrap();
                    if ratio < 3.0 {
                        bad.push(format!(
                            "{} {:?}: {:?} {ratio:.2}",
                            spec.slug, hovered, span.content
                        ));
                    }
                }
            }
        }
    }
    assert!(bad.is_empty(), "{bad:#?}");
}
