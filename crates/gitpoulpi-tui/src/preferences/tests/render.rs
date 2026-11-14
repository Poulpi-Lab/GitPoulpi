use super::*;
use crate::th;

/// No stale cells from a previous frame (a Log tooltip, a context menu, or a
/// row that shrank between two Preferences frames): `Paragraph` re-styles
/// cells it has no content for but never resets their symbol, so `render` and
/// `render_sidebar`/`render_body` each `Clear` their area.
///
/// Draws something dirty first, then Preferences alone, and asserts the result
/// is byte-for-byte what drawing Preferences over a blank screen gives.
#[test]
fn preferences_render_leaves_no_stale_cells_from_a_previous_frame() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    let dir = tempfile::tempdir().unwrap();
    let area = Rect::new(0, 0, 80, 24);

    let render = |dirty: bool| {
        let mut terminal = Terminal::new(TestBackend::new(area.width, area.height)).unwrap();
        if dirty {
            // A tooltip-shaped mess: border fragments and a tinted
            // background, covering the whole screen so no row --
            // sidebar or body, short section or long one -- escapes
            // the check.
            terminal
                .draw(|frame| {
                    let buf = frame.buffer_mut();
                    for y in area.top()..area.bottom() {
                        for x in area.left()..area.right() {
                            buf[(x, y)].set_symbol("\u{2502}").set_style(
                                Style::default().bg(th().ui.selection).fg(th().ui.accent),
                            );
                        }
                    }
                })
                .unwrap();
        }
        let mut prefs = state(dir.path());
        terminal
            .draw(|frame| {
                prefs.render(frame, area);
            })
            .unwrap();
        // A cell the two-wide octopus covers is never drawn (the terminal
        // skips it), and the backend only holds what it was told to draw:
        // blank those, they are not "stale cells".
        let mut buffer = terminal.backend().buffer().clone();
        for y in 0..area.height {
            let mut x = 0;
            while x < area.width {
                if display_width(buffer[(x, y)].symbol()) > 1 {
                    buffer[(x + 1, y)].reset();
                    x += 1;
                }
                x += 1;
            }
        }
        buffer
    };

    assert_eq!(
        render(false),
        render(true),
        "cells from before Preferences opened were not fully repainted"
    );
}

/// The close control hovers with the same strong fill as the gear and
/// the app's own close cross, not a faint lift (owner, 2026-09-28).
#[test]
fn the_close_control_hovers_with_the_plain_button_fill() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    terminal
        .draw(|frame| state.render(frame, frame.area()))
        .unwrap();
    let rect = state.close_rect.expect("the close control is drawn");
    let idle = terminal.backend().buffer()[(rect.x + 1, rect.y)].style();
    assert_ne!(idle.bg, crate::theme::plain_button_style(true).bg);

    state.on_mouse(MouseEventKind::Moved, rect.x + 1, rect.y);
    terminal
        .draw(|frame| state.render(frame, frame.area()))
        .unwrap();
    let hovered = terminal.backend().buffer()[(rect.x + 1, rect.y)].style();
    let want = crate::theme::plain_button_style(true);
    assert_eq!((hovered.fg, hovered.bg), (want.fg, want.bg));
}

/// Renders into a `TestBackend` and reads the *buffer* back, so it asserts on
/// cells the widget painted, never on a string the test built
/// (`where-we-are.md`: falsifying buffer tests).
#[test]
fn the_screen_draws_both_scopes_in_the_sidebar_and_the_general_page_beside_them() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
    terminal
        .draw(|frame| state.render(frame, frame.area()))
        .unwrap();

    let buffer = terminal.backend().buffer();
    let rendered: Vec<String> = (0..40)
        .map(|y| (0..100).map(|x| buffer[(x, y)].symbol()).collect())
        .collect();
    let screen = rendered.join("\n");

    // Both scopes are named.
    assert!(screen.contains("Current profile"), "{screen}");
    assert!(screen.contains("Settings - Tab"), "{screen}");
    assert!(screen.contains("Repo-specific settings"), "{screen}");
    // Every page is reachable from the list.
    for section in PrefSection::ALL.iter().filter(|s| s.is_built()) {
        assert!(
            screen.contains(section.title()),
            "{}\n{screen}",
            section.title()
        );
    }
    // The General page is beside it.
    assert!(screen.contains("Remember tabs"), "{screen}");
    // The Log & graph page, with its fields.
    go(&mut state, PrefSection::LogGraph);
    state.focus = PrefFocus::Fields;
    state.field = state
        .fields()
        .iter()
        .position(|f| f.label == "Initial commits in graph")
        .unwrap();
    terminal
        .draw(|frame| state.render(frame, frame.area()))
        .unwrap();
    let buffer = terminal.backend().buffer();
    let screen = (0..40)
        .map(|y| {
            (0..100)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(screen.contains("Initial commits in graph"), "{screen}");
    // ... and the number, not just the label.
    assert!(screen.contains("5000"), "{screen}");
    // The warning glyph under that field.
    assert!(screen.contains('\u{26A0}'), "{screen}");
}

/// A build with no provider has no Integrations page: it is not in the
/// sidebar and the keyboard never lands on it.
#[cfg(not(feature = "hosting"))]
#[test]
fn a_build_with_no_provider_has_no_integrations_page() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    let mut terminal = Terminal::new(TestBackend::new(100, 34)).unwrap();
    terminal
        .draw(|frame| state.render(frame, frame.area()))
        .unwrap();
    let buffer = terminal.backend().buffer();
    let screen: String = (0..34)
        .map(|y| {
            (0..100)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !screen.contains(PrefSection::Integrations.title()),
        "{screen}"
    );
    assert!(screen.contains(PrefSection::General.title()), "{screen}");
    assert!(!PrefSection::Integrations.is_built());
}

/// Selecting a section has to change what the right-hand pane
/// *renders*, not merely what an enum holds.
#[test]
#[cfg(all(feature = "github", feature = "gitlab"))]
fn choosing_a_section_changes_the_page_that_is_drawn() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    state.section = PrefSection::ALL
        .iter()
        .position(|s| *s == PrefSection::Integrations)
        .unwrap();

    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    terminal
        .draw(|frame| state.render(frame, frame.area()))
        .unwrap();
    let buffer = terminal.backend().buffer();
    let screen: String = (0..30)
        .map(|y| {
            (0..100)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(screen.contains("No GitHub profile yet"), "{screen}");
    assert!(screen.contains("+ Add profile"), "{screen}");
    assert!(
        !screen.contains("Initial commits in graph"),
        "the General page is still drawn:\n{screen}"
    );
}

/// A stored token must not be renderable, ever. This asserts on the
/// painted cells, because "the struct does not hold it" is a weaker
/// claim than "the terminal never showed it".
#[test]
#[cfg(all(feature = "github", feature = "gitlab"))]
fn a_stored_token_is_never_painted_on_the_screen() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    let id = add_profile(&mut state, IntegrationKind::Gitlab);
    state.config.integrations[0].user = Some(gitpoulpi_config::ConnectedUser {
        login: "kai".into(),
        ..Default::default()
    });
    let token_ref = state.config.integrations[0].token_ref.clone();
    state
        .tokens
        .store(&token_ref, "glpat-NEVER-SHOW-ME")
        .unwrap();
    state.token_presence.insert(token_ref, Some(true));
    assert_eq!(id, "gitlab");
    state.integrations.select_kind(2, &state.config);
    state.section = PrefSection::ALL
        .iter()
        .position(|s| *s == PrefSection::Integrations)
        .unwrap();

    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    terminal
        .draw(|frame| state.render(frame, frame.area()))
        .unwrap();
    let buffer = terminal.backend().buffer();
    let screen: String = (0..30)
        .map(|y| {
            (0..100)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(!screen.contains("glpat"), "{screen}");
    assert!(!screen.contains("NEVER-SHOW-ME"), "{screen}");
    // But it does say a token is there, and what it is filed under.
    assert!(screen.contains("stored"), "{screen}");
    assert!(screen.contains("@kai"), "{screen}");
}

/// A setting nothing reads has to say so on screen, or the user's
/// only way to find out is to change it and watch nothing happen --
/// the exact failure `settings.rs` exists to end.
#[cfg(feature = "mcp")]
#[test]
fn an_unwired_setting_is_marked_as_such_in_the_rendered_page() {
    // No shipped row is unwired any more, so take a real one and mark it so
    // (see `every_editable_row_is_live_or_explicitly_coming`). Checked
    // through `field_lines` directly.
    let dir = tempfile::tempdir().unwrap();
    let state = state(dir.path());
    let mut fields = state.agents_fields();
    let index = fields
        .iter()
        .position(|f| f.label == "MCP server port")
        .expect("the row exists");
    fields[index].live = false;
    let field = &fields[index];
    assert!(!field.live && field.coming.is_none());
    let lines: String = state
        .field_lines(field, index, false, 60)
        .iter()
        .map(|l| {
            l.spans
                .iter()
                .map(|s| s.content.as_ref())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(lines.contains("nothing uses this setting yet"), "{lines}");
}

#[test]
fn wrapping_breaks_on_words_and_never_loses_one() {
    let wrapped = wrap_words("the quick brown fox jumps", 10);
    assert!(
        wrapped.iter().all(|l| display_width(l) <= 10),
        "{wrapped:?}"
    );
    assert_eq!(wrapped.join(" "), "the quick brown fox jumps");
}

fn row_text(line: &Line) -> String {
    line.spans.iter().map(|s| s.content.as_ref()).collect()
}

/// The body of the General page drawn at 100 columns: no dot leader anywhere
/// between a label and its value.
#[test]
fn the_body_has_no_dot_leader() {
    let dir = tempfile::tempdir().unwrap();
    let mut prefs = state(dir.path());
    go(&mut prefs, PrefSection::General);
    let screen = draw_screen(&mut prefs);
    // The sidebar's search results use U+00B7 as a separator; the body does not.
    let body: String = screen
        .lines()
        .map(|l| l.chars().skip(30).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!body.contains('\u{00B7}'), "{screen}");
}

/// One value column per page: every labelled row of General starts its
/// control at the same cell.
#[test]
fn every_value_on_a_page_starts_at_the_same_column() {
    let dir = tempfile::tempdir().unwrap();
    let mut prefs = state(dir.path());
    go(&mut prefs, PrefSection::General);
    let fields = prefs.fields();
    prefs.set_label_col(&fields, 68);
    let mut starts = std::collections::BTreeSet::new();
    let mut rows = 0;
    for (index, field) in fields.iter().enumerate() {
        let is_control = matches!(
            field.control,
            Control::Toggle(_) | Control::Text(_) | Control::Number { .. } | Control::Choice { .. }
        );
        if !is_control || field.coming.is_some() {
            continue;
        }
        let lines = prefs.field_lines(field, index, false, 68);
        let first = &lines[0];
        let control = display_width(first.spans.last().unwrap().content.as_ref());
        starts.insert(display_width(&row_text(first)) - control);
        rows += 1;
    }
    assert!(rows >= 4, "General has several controls: {rows}");
    assert_eq!(starts.len(), 1, "value columns: {starts:?}");
}

#[test]
fn a_long_label_wraps_in_the_label_column_and_never_gets_an_ellipsis() {
    let dir = tempfile::tempdir().unwrap();
    let prefs = state(dir.path());
    let fields = vec![Field::plain(
        "Beim Start automatisch nach neuen Versionen suchen",
        Control::Toggle(true),
        Target::None,
    )];
    prefs.set_label_col(&fields, 68);
    let rows: Vec<String> = prefs
        .field_lines(&fields[0], 0, false, 68)
        .iter()
        .map(row_text)
        .collect();
    assert!(rows[0].contains("[x]"), "{rows:?}");
    assert!(
        rows[1].trim_end().len() > 3 && !rows[1].contains("[x]"),
        "{rows:?}"
    );
    assert!(rows.iter().all(|r| !r.contains('\u{2026}')), "{rows:?}");
}

/// Below `label_col + 24` columns the control drops under its label.
#[test]
fn at_body_50_the_row_stacks_the_control_under_the_label() {
    let dir = tempfile::tempdir().unwrap();
    let prefs = state(dir.path());
    let fields = vec![Field::plain(
        "Default branch name for new repositories",
        Control::Text("main".into()),
        Target::None,
    )];
    prefs.set_label_col(&fields, 50);
    let rows: Vec<String> = prefs
        .field_lines(&fields[0], 0, false, 50)
        .iter()
        .map(row_text)
        .collect();
    assert_eq!(
        rows[0].trim_end(),
        "Default branch name for new repositories",
        "{rows:?}"
    );
    assert!(rows[1].starts_with("   main"), "{rows:?}");
    prefs.set_label_col(&fields, 68);
    let wide = prefs.field_lines(&fields[0], 0, false, 68);
    assert!(row_text(&wide[0]).contains("main"), "side by side at 68");
}

/// The title line wears the main window's octopus and the app's name, at the
/// narrow and the wide layout; the octopus is the About click target.
#[test]
fn the_title_line_wears_the_octopus_and_the_app_name_at_80_and_160_columns() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    let dir = tempfile::tempdir().unwrap();
    for width in [80u16, 160] {
        let mut state = state(dir.path());
        let mut terminal = Terminal::new(TestBackend::new(width, 24)).unwrap();
        terminal
            .draw(|frame| state.render(frame, frame.area()))
            .unwrap();
        let buffer = terminal.backend().buffer();
        let top: String = (0..width).map(|x| buffer[(x, 0)].symbol()).collect();
        let logo = state.logo_rect.expect("the octopus is drawn");
        assert_eq!((logo.x, logo.y), (1, 0), "{top}");
        assert_eq!(buffer[(2, 0)].symbol(), "\u{1F419}", "{top}");
        insta::assert_snapshot!(
            format!("settings_title_line_{width}"),
            top.trim_end_matches(['\u{2500}', ' '])
        );
        // The close cross keeps its place at the right edge.
        let close = state.close_rect.expect("the cross is drawn");
        assert!(close.right() <= width, "{close:?}");
        assert!(logo.right() < close.x, "{logo:?} {close:?}");
    }
}

/// A text field and an editable combo fill the room right of the value
/// column, side by side (wide) and stacked (narrow); a pick-only choice fits
/// its widest option (at least 20 cells) and never stretches.
#[test]
fn text_and_choice_bands_fill_the_room_wide_and_stacked() {
    let dir = tempfile::tempdir().unwrap();
    let state = state(dir.path());
    let text = Field::global("Name", "x.name", Control::Text("v".into()));
    let editable = Field::global(
        "Model",
        "x.model",
        Control::Combo {
            options: vec![("one".into(), "one".into())],
            value: "one".into(),
            custom: true,
        },
    );
    let choice = Field::global(
        "Mode",
        "x.mode",
        Control::Choice {
            options: vec!["off".into(), "ask".into(), "auto".into()],
            index: 0,
        },
    );
    let pick_combo = Field::global(
        "Preset",
        "x.preset",
        Control::Combo {
            options: vec![("a".into(), "a".into()), ("b".into(), "b".into())],
            value: "a".into(),
            custom: false,
        },
    );
    let long_pick = Field::global(
        "Mode",
        "x.mode",
        Control::Choice {
            options: vec![
                "short".into(),
                "a considerably longer option label here".into(),
            ],
            index: 0,
        },
    );
    let first_row = |field: &Field, width: usize| {
        state.label_col.set((0, false));
        row_text(&state.field_lines(field, 0, false, width)[0])
    };
    for field in [&text, &editable] {
        // Wide: the row runs to the body width, with no content cap.
        let row = first_row(field, 140);
        assert_eq!(display_width(&row), 140, "wide: {row:?}");
        // Mid width: the row ends at the body width.
        let row = first_row(field, 90);
        assert_eq!(display_width(&row), 90, "mid: {row:?}");
        // Stacked (narrow): label above, the band fills the line under it.
        state.label_col.set((0, false));
        let lines = state.field_lines(field, 0, false, 30);
        assert_eq!(
            display_width(&row_text(&lines[1])),
            30,
            "stacked: {lines:?}"
        );
    }
    // A pick-only choice is its content wide: label column + 20 minimum.
    for field in [&choice, &pick_combo] {
        let (label_col, _) = PreferencesState::layout_for(display_width(&field.label), 140);
        let row = first_row(field, 140);
        assert_eq!(
            display_width(&row),
            label_col + 20,
            "short options: {row:?}"
        );
        state.label_col.set((0, false));
        let lines = state.field_lines(field, 0, false, 30);
        assert_eq!(
            display_width(&row_text(&lines[1])),
            2 + 20,
            "stacked: {lines:?}"
        );
    }
    // Longer options widen it: widest label + 4, capped by the room.
    let (label_col, _) = PreferencesState::layout_for(display_width(&long_pick.label), 140);
    let widest = "a considerably longer option label here".len();
    let row = first_row(&long_pick, 140);
    assert_eq!(display_width(&row), label_col + widest + 4, "{row:?}");
    let row = first_row(&long_pick, 40);
    assert!(display_width(&row) <= 40, "{row:?}");
}

/// The pointing hand over what a click acts on, the I-beam over a text field
/// (`pointer_text_at`), on cells the paint put on screen.
#[test]
fn settings_show_the_hand_on_a_toggle_and_the_cross_and_the_i_beam_on_a_text_field() {
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;
    use ratatui::Terminal;

    let dir = tempfile::tempdir().unwrap();
    let mut state = state(dir.path());
    let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
    let paint = |state: &mut PreferencesState, terminal: &mut Terminal<TestBackend>| {
        crate::pointer_map::begin_frame();
        terminal
            .draw(|frame| state.render(frame, frame.area()))
            .unwrap();
        terminal.backend().buffer().clone()
    };
    let row_in = |buffer: &Buffer, needle: &str| -> Option<u16> {
        (0..40).find(|y| {
            let line: String = (0..100).map(|x| buffer[(x, *y)].symbol()).collect();
            line.contains(needle)
        })
    };
    // Scrolls the page until `needle` is on screen; its row.
    let show =
        |state: &mut PreferencesState, terminal: &mut Terminal<TestBackend>, needle: &str| -> u16 {
            for scroll in (0..400).step_by(2) {
                state.scroll = scroll;
                let buffer = paint(state, terminal);
                if let Some(y) = row_in(&buffer, needle) {
                    return y;
                }
            }
            panic!("{needle:?} never came on screen");
        };

    let cell_of = |buffer: &Buffer, y: u16, needle: &str| -> u16 {
        let line: String = (0..100).map(|x| buffer[(x, y)].symbol()).collect();
        // The first match right of the sidebar (28 cells).
        let from: String = line.chars().skip(30).collect();
        let at = from.find(needle).expect(needle);
        (30 + from[..at].chars().count()) as u16
    };
    // A checkbox row: the label is the arrow, the box is the hand, and the
    // whole row lights (the click on the label still acts).
    let toggle = show(&mut state, &mut terminal, "Remember tabs");
    let buffer = paint(&mut state, &mut terminal);
    let toggle = row_in(&buffer, "Remember tabs").unwrap_or(toggle);
    let label_x = cell_of(&buffer, toggle, "Remember tabs");
    let box_x = cell_of(&buffer, toggle, "[");
    assert!(
        !state.pointer_hand_at(label_x, toggle),
        "the label is not a hand"
    );
    assert!(!state.pointer_text_at(label_x, toggle), "nor an I-beam");
    assert!(
        state.pointer_hand_at(box_x + 1, toggle),
        "the box is a hand"
    );
    // The pointer on the label lights the whole row: label, box and the
    // cells between them, the same wash for every kind of row.
    state.mouse_pos = Some((label_x, toggle));
    let lit = paint(&mut state, &mut terminal);
    for x in [label_x, box_x, box_x + 1] {
        assert_eq!(lit[(x, toggle)].bg, th().ui.hover, "col {x} of the row");
    }
    state.mouse_pos = None;
    let close = state.close_rect.expect("the cross");
    assert!(state.pointer_hand_at(close.x + 1, close.y), "the cross");

    let buffer = paint(&mut state, &mut terminal);
    let section = row_in(&buffer, "Synchronization").expect("a section");
    assert!(
        !state.pointer_hand_at(4, section),
        "a section of the list is a row: the arrow"
    );
    let search = row_in(&buffer, "Search settings").expect("the search box");
    assert!(
        !state.pointer_hand_at(4, search),
        "the search box is a field"
    );
    assert!(
        state.pointer_text_at(4, search),
        "the I-beam in the search box"
    );

    // A text row: the label is the arrow, only the input box is the I-beam.
    let text = show(&mut state, &mut terminal, "Default branch name");
    let buffer = paint(&mut state, &mut terminal);
    let text = row_in(&buffer, "Default branch name").unwrap_or(text);
    let label_x = cell_of(&buffer, text, "Default branch name");
    let input_x = (30..100)
        .find(|x| state.pointer_text_at(*x, text))
        .expect("an input box on the row");
    assert!(input_x > label_x + 10, "the box starts right of the label");
    assert!(
        !state.pointer_hand_at(input_x, text),
        "an input is not a hand"
    );
    assert!(
        !state.pointer_text_at(label_x, text),
        "the label is the arrow"
    );

    // Editing the text field: the open field keeps the arrow too.
    state.focus = PrefFocus::Fields;
    state.field = state
        .fields()
        .iter()
        .position(|f| f.label == "Default branch name")
        .unwrap();
    state.activate();
    assert!(state.edit.is_some());
    let text = show(&mut state, &mut terminal, "Default branch name");
    assert!(
        !state.pointer_hand_at(input_x, text),
        "the field being edited is not a hand"
    );
    assert!(
        state.pointer_text_at(input_x, text),
        "the edited field is the I-beam"
    );
}

/// The pointer kind the Settings page registers over the cell of the first
/// combobox setting it lists (its `v`), for the cross-surface consistency
/// test of the issues pane (`hosting_ui::tests::rows`).
pub(crate) fn first_combobox_pointer_kind(
    dir: &std::path::Path,
) -> Option<crate::pointer_map::Kind> {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    let mut state = state(dir);
    let label = state
        .fields()
        .iter()
        .find(|f| matches!(f.control, Control::Choice { .. }))?
        .label
        .clone();
    let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
    for scroll in (0..400).step_by(2) {
        state.scroll = scroll;
        crate::pointer_map::begin_frame();
        terminal
            .draw(|frame| state.render(frame, frame.area()))
            .unwrap();
        let buffer = terminal.backend().buffer().clone();
        for y in 0..40 {
            let line: String = (0..100).map(|x| buffer[(x, y)].symbol()).collect();
            if line.contains(&label) {
                if let Some(x) = (30..100).find(|x| buffer[(*x, y)].symbol() == "\u{25BE}") {
                    return crate::pointer_map::at(x, y);
                }
            }
        }
    }
    None
}

#[test]
fn a_settings_combobox_registers_the_hand() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(
        first_combobox_pointer_kind(dir.path()),
        Some(crate::pointer_map::Kind::Hand)
    );
}
