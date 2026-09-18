use super::*;

#[test]
fn render_default_shows_every_row_and_its_buttons() {
    let f = form();
    let (rows, hits) = paint(&f);
    let all = rows.join("\n");
    assert!(all.contains("Interactive rebase"), "{all}");
    assert!(all.contains("third"), "{all}");
    assert!(all.contains("Start rebase"), "{all}");
    for i in 0..3 {
        assert!(hits.iter().any(|(_, p)| *p == RebasePart::Row(i)));
    }
    assert!(hits.iter().any(|(_, p)| *p == RebasePart::Start));
    assert!(hits.iter().any(|(_, p)| *p == RebasePart::Cancel));
}

/// Kept-commit dots use the active `ui.graph_color_theme`'s first lane
/// color, not the GitPoulpi palette unconditionally.
#[test]
fn kept_commit_dots_follow_the_graph_color_theme() {
    use crate::graph::LaneTheme;
    let t = LaneTheme::from_slug;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let f = form();
    let dot_colors = |theme: LaneTheme| {
        let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
        terminal
            .draw(|frame| {
                f.render(frame, frame.area(), frame.area(), None, theme);
            })
            .unwrap();
        let buf = terminal.backend().buffer().clone();
        let mut colors = Vec::new();
        for y in 0..30 {
            for x in 0..100 {
                if buf[(x, y)].symbol() == "\u{25CF}" {
                    colors.push(buf[(x, y)].fg);
                }
            }
        }
        colors
    };
    let inspired = dot_colors(t("material"));
    let poulpi = dot_colors(t("gitpoulpi"));
    assert!(!inspired.is_empty());
    assert!(inspired.contains(&crate::graph::lane_palette(t("material"))[0]));
    assert!(poulpi.contains(&crate::graph::lane_palette(t("gitpoulpi"))[0]));
    assert_ne!(
        crate::graph::lane_palette(t("material"))[0],
        crate::graph::lane_palette(t("gitpoulpi"))[0]
    );
    assert!(!inspired.contains(&crate::graph::lane_palette(t("gitpoulpi"))[0]));
}

/// Every control lights with `plain_button_style(true)`'s fill when
/// hovered: the three footer buttons, the autosquash checkbox and the
/// action pills.
#[test]
fn every_control_takes_the_strong_hover_fill_when_hovered() {
    let f = form();
    let (_, hits) = paint_hovered(&f, None);
    for part in [
        RebasePart::Reset,
        RebasePart::Cancel,
        RebasePart::Start,
        RebasePart::Autosquash,
        RebasePart::Dropdown(1),
    ] {
        let rect = hits.iter().find(|(_, p)| *p == part).unwrap().0;
        let (rest, _) = paint_hovered(&f, None);
        let (lit, _) = paint_hovered(&f, Some(part));
        assert_ne!(rest[(rect.x, rect.y)].bg, th().ui.button_hover);
        assert_eq!(lit[(rect.x, rect.y)].bg, th().ui.button_hover, "{part:?}");
    }
}

#[test]
fn the_open_list_is_drawn_and_its_choices_win_over_the_rows_they_cover() {
    let mut f = RebaseForm::new_cherry_pick(
        "main".into(),
        vec![
            row(3, "third", RebaseAction::Pick),
            row(2, "second", RebaseAction::Pick),
            row(1, "first", RebaseAction::Pick),
        ],
    );
    f.head_sha = Some("ba80bc7".into());
    f.on_click(RebasePart::Dropdown(0));
    let (rows, hits) = paint(&f);
    let all = rows.join("\n");
    assert!(all.contains("Reword") && all.contains("Squash"), "{all}");
    assert!(all.contains("Cancel cherry pick"), "{all}");
    assert!(all.contains("onto main ba80bc7"), "{all}");
    let squash = hits
        .iter()
        .position(|(_, p)| *p == RebasePart::Action(0, RebaseAction::Squash))
        .unwrap();
    let row1 = hits
        .iter()
        .position(|(_, p)| *p == RebasePart::Row(1))
        .unwrap();
    assert!(squash < row1, "choices are registered before the rows");
    // Nothing but Pick/Reword/Squash/Drop.
    assert!(!hits
        .iter()
        .any(|(_, p)| *p == RebasePart::Action(0, RebaseAction::Fixup)));
}

#[test]
fn the_footer_lists_the_mnemonics_and_has_three_buttons_for_both_views() {
    for f in [
        form(),
        RebaseForm::new_cherry_pick("main".into(), vec![row(1, "a", RebaseAction::Pick)]),
    ] {
        let (buf, hits) = paint_hovered(&f, None);
        let text = crate::dialog_kit::buffer_text(&buf);
        for want in [
            "shortcuts:",
            "Pick",
            "Squash",
            "Reword",
            "Drop",
            "Ctrl+\u{2191} Move up",
            "Ctrl+\u{2193} Move down",
            " Reset ",
        ] {
            assert!(text.contains(want), "{want:?} missing:\n{text}");
        }
        let cancel = if f.cherry_pick {
            "Cancel cherry pick"
        } else {
            "Cancel rebase"
        };
        assert!(
            text.contains(cancel)
                && text.contains(if f.cherry_pick {
                    "Start cherry pick"
                } else {
                    "Start rebase"
                })
        );
        // The first letter of each mnemonic is the bright one.
        let (x, y) = (0..buf.area.height)
            .flat_map(|y| (0..buf.area.width).map(move |x| (x, y)))
            .find(|&(x, y)| {
                buf[(x, y)].symbol() == "P"
                    && buf[(x + 1, y)].symbol() == "i"
                    && buf[(x + 2, y)].symbol() == "c"
                    && y > 20
                    && x < 8
            })
            .expect("the Pick mnemonic");
        assert_ne!(buf[(x, y)].fg, buf[(x + 1, y)].fg);
        // Bottom and top Cancel, Reset and Start are all clickable.
        assert_eq!(
            hits.iter()
                .filter(|(_, p)| *p == RebasePart::Cancel)
                .count(),
            2
        );
        assert!(hits.iter().any(|(_, p)| *p == RebasePart::Reset));
        assert!(hits.iter().any(|(_, p)| *p == RebasePart::Start));
    }
}

#[test]
fn the_rebase_header_names_the_branch_and_what_it_goes_onto() {
    let mut f = form();
    f.onto_label = Some("feature/login".into());
    let (buf, _) = paint_hovered(&f, None);
    let text = crate::dialog_kit::buffer_text(&buf);
    assert!(
        text.contains("Rebasing feature onto feature/login"),
        "{text}"
    );
}

#[test]
fn a_squashed_row_shows_the_arrow_into_the_row_below_and_is_dimmed() {
    let mut f = form();
    f.on_click(RebasePart::Action(0, RebaseAction::Squash));
    let (rows, _) = paint(&f);
    let line = rows.iter().find(|l| l.contains("third")).unwrap();
    assert!(line.contains("Squash"), "{line}");
    assert!(line.contains('\u{2193}'), "{line}");
}
