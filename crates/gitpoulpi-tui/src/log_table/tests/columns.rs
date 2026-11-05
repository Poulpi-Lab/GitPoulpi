//! Column widths, sacrifice order, offsets and hover.

use super::*;
use super::{changes_off, sample_commit, shown_order};

#[test]
fn log_column_widths_sum_to_the_available_inner_width() {
    let no_hidden = changes_off();
    let no_overrides = std::collections::HashMap::new();
    for area_width in [20u16, 40, 84, 120, 193, 300] {
        let w = LogColumnWidths::for_area_width(area_width, &no_hidden, &no_overrides);
        let sum = w.ref_w + w.graph_w + w.message_w + w.author_w + w.date_w + w.changes_w + w.sha_w;
        assert_eq!(
            sum,
            area_width.saturating_sub(2),
            "widths must fill the inner area exactly at width {area_width} (no overlap, no gap)"
        );
    }
}

/// `Message` is the one column `hideable()` refuses to hide, because a
/// Log pane with no commit summaries shows nothing. It used to be
/// allocated *last*, out of whatever the five protected columns had
/// left, so at a narrow pane it was the only column that actually
/// vanished - exactly backwards.
#[test]
fn the_message_column_never_vanishes_however_narrow_the_pane() {
    let no_hidden = changes_off();
    let no_overrides = std::collections::HashMap::new();
    for area_width in 0..=300u16 {
        let w = LogColumnWidths::for_area_width(area_width, &no_hidden, &no_overrides);
        let inner = area_width.saturating_sub(2);
        let sum = w.ref_w + w.graph_w + w.message_w + w.author_w + w.date_w + w.changes_w + w.sha_w;
        assert_eq!(
            sum, inner,
            "widths must still tile the pane exactly at width {area_width}"
        );
        // One cell of every text column is a gutter (`commit_row`), so
        // a Message column narrower than 2 prints nothing at all.
        assert!(
            w.message_w >= 2 || inner < 2,
            "the commit summary vanished at width {area_width}: {w:?}"
        );
    }
}

/// What gets given up as the pane shrinks, in order. Pinned here
/// because "which column disappears first" is a product decision, not
/// an accident of the allocation loop: see `SACRIFICE_ORDER`.
#[test]
fn a_narrow_pane_sacrifices_columns_in_the_documented_order() {
    let no_hidden = changes_off();
    let no_overrides = std::collections::HashMap::new();
    let dropped = |area_width: u16| -> Vec<LogColumn> {
        let w = LogColumnWidths::for_area_width(area_width, &no_hidden, &no_overrides);
        LogColumn::ALL
            .into_iter()
            .filter(|c| w.width_of(*c) == 0 && *c != LogColumn::Changes)
            .collect()
    };
    assert_eq!(dropped(120), Vec::new(), "a wide pane keeps every column");
    assert_eq!(
        dropped(40),
        vec![LogColumn::Ref],
        "the ref pills are the first thing worth ten cells to give up"
    );
    assert_eq!(dropped(30), vec![LogColumn::Ref, LogColumn::Date]);

    // The whole list, at every width: whatever has been given up is
    // always a *prefix* of `SACRIFICE_ORDER` -- never a column chosen
    // by whichever loop happened to run out of budget first -- it only
    // ever grows as the pane narrows, and `Message` is never in it.
    let mut previous: Vec<LogColumn> = Vec::new();
    // From width 3 down: at 2 and below the pane has no inner width at
    // all, so every column is zero and "given up" stops meaning
    // anything.
    for area_width in (3..=300u16).rev() {
        let gone = dropped(area_width);
        assert!(
            !gone.contains(&LogColumn::Message),
            "Message was sacrificed at width {area_width}"
        );
        let expected: Vec<LogColumn> = LogColumn::SACRIFICE_ORDER
            .into_iter()
            .filter(|c| *c != LogColumn::Changes)
            .take(gone.len())
            .collect();
        let mut sorted_expected = expected.clone();
        sorted_expected.sort_by_key(|c| LogColumn::ALL.iter().position(|a| a == c));
        assert_eq!(
            gone, sorted_expected,
            "at width {area_width} the columns given up are not a prefix \
             of SACRIFICE_ORDER"
        );
        assert!(
            gone.len() >= previous.len(),
            "width {area_width} gives up fewer columns than the wider \
             pane before it"
        );
        previous = gone;
    }
    assert_eq!(
        previous.len(),
        LogColumn::SACRIFICE_ORDER.len() - 1,
        "at width 3 everything but Message has been given up"
    );
}

/// `Changes` is an optional extra: when it is shown and the pane gets
/// tight it is the first column given up, ahead of `Ref`.
#[test]
fn a_shown_changes_column_is_the_first_one_sacrificed() {
    let none = std::collections::HashSet::new();
    let no_overrides = std::collections::HashMap::new();
    let wide = LogColumnWidths::for_area_width(140, &none, &no_overrides);
    assert!(wide.changes_w >= 17, "room for `12 files +345 -67`");
    let tight = |w: u16| LogColumnWidths::for_area_width(w, &none, &no_overrides);
    let dropped_at = (3..=140u16)
        .rev()
        .find(|w| tight(*w).changes_w == 0)
        .expect("a narrow pane gives it up");
    let narrower = tight(dropped_at);
    assert_eq!(narrower.changes_w, 0);
    assert!(
        narrower.ref_w > 0,
        "Ref outlives Changes at width {dropped_at}"
    );
}

#[test]
fn the_changes_cell_shortens_with_its_column_and_shows_a_placeholder_first() {
    use gitpoulpi_core::domain::CommitStat;
    let text = |stat: Option<&CommitStat>, w: usize| -> String {
        changes_spans(stat, w, Style::default())
            .iter()
            .map(|s| s.content.as_ref())
            .collect()
    };
    let stat = CommitStat {
        id: Oid([0; 20]),
        files: 3,
        insertions: 12,
        deletions: 4,
    };
    assert_eq!(text(None, 18), "\u{2026}");
    assert_eq!(text(Some(&stat), 18), "3 files +12 -4");
    assert_eq!(text(Some(&stat), 9), "3f +12 -4");
    assert_eq!(text(Some(&stat), 4), "3f");
    let one = CommitStat { files: 1, ..stat };
    assert_eq!(text(Some(&one), 18), "1 file +12 -4");
    let binary = CommitStat {
        insertions: 0,
        deletions: 0,
        ..stat
    };
    assert_eq!(text(Some(&binary), 18), "3 files");
}

/// The rendered half of the same defect: a real `log_table` over a
/// sweep of narrow panes must still print the commit summary. The
/// widths tiling to `inner` is necessary but not sufficient - the
/// gutter `commit_row` keeps can eat a 1-cell column whole.
#[test]
fn a_narrow_log_pane_still_renders_the_commit_summary() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    let order = shown_order();
    for width in [24u16, 30, 40, 50, 64, 80, 120] {
        let log_area = Rect::new(0, 0, width, 6);
        let widths = LogColumnWidths::for_area_width(
            log_area.width,
            &changes_off(),
            &std::collections::HashMap::new(),
        );
        let offsets = column_x_offsets(log_area.x, &order, &widths);
        let commit = sample_commit(1, "Fix the crash on startup");
        let mut terminal = Terminal::new(TestBackend::new(width, log_area.height)).unwrap();
        terminal
            .draw(|frame| {
                let header: Vec<Cell> = order.iter().map(|&c| Cell::from(c.title())).collect();
                let row = commit_row(
                    &commit,
                    &[],
                    Line::from(""),
                    Color::Reset,
                    &widths,
                    &order,
                    false,
                    false,
                    None,
                    false,
                    avatar::LogAvatarMode::Off,
                    gitpoulpi_config::AvatarFallback::Initials,
                    None,
                );
                frame.render_stateful_clipped(
                    log_table(vec![row], header, &order, &widths, Style::default()),
                    log_area,
                    &mut TableState::default(),
                );
            })
            .unwrap();
        let buf = terminal.backend().buffer().clone();
        let (_, mx, mw) = offsets
            .iter()
            .find(|(c, _, _)| *c == LogColumn::Message)
            .copied()
            .expect("Message is always visible");
        let printed: String = (mx..mx + mw)
            .map(|x| buf[(x, log_area.y + 2)].symbol().to_string())
            .collect::<Vec<_>>()
            .concat();
        assert!(
            !printed.trim().is_empty(),
            "the commit summary is blank at pane width {width} \
             (Message column x={mx} w={mw})"
        );
        assert!(
            commit
                .summary
                .starts_with(printed.trim().trim_end_matches('\u{2026}')),
            "what is printed at width {width} must be the start of the \
             summary, got [{printed}]"
        );
    }
}

#[test]
fn log_column_widths_keep_sha_and_date_intact_at_reasonable_widths() {
    let no_hidden = changes_off();
    let no_overrides = std::collections::HashMap::new();
    // this project's own Herdr fixture pane width
    let w = LogColumnWidths::for_area_width(84, &no_hidden, &no_overrides);
    assert_eq!(w.sha_w, 8); // 7 hex chars plus the last column's grip cell
    assert!(w.date_w >= 16, "\"YYYY-MM-DD HH:MM\" is 16 chars");
    assert!(
        w.ref_w >= 16,
        "must fit icon + checkmark + a real branch name"
    );
}

#[test]
fn log_column_widths_hidden_column_contributes_nothing() {
    let mut hidden = std::collections::HashSet::new();
    hidden.insert(LogColumn::Sha);
    hidden.insert(LogColumn::Author);
    let no_overrides = std::collections::HashMap::new();
    let w = LogColumnWidths::for_area_width(84, &hidden, &no_overrides);
    assert_eq!(w.sha_w, 0);
    assert_eq!(w.author_w, 0);
    let sum = w.ref_w + w.graph_w + w.message_w + w.author_w + w.date_w + w.changes_w + w.sha_w;
    assert_eq!(sum, 82, "hidden columns free their budget for the rest");
}

#[test]
fn log_column_widths_honors_an_override_within_its_min_and_the_budget() {
    let no_hidden = changes_off();
    let mut overrides = std::collections::HashMap::new();
    overrides.insert(LogColumn::Ref, 20);
    let w = LogColumnWidths::for_area_width(84, &no_hidden, &overrides);
    assert_eq!(w.ref_w, 20);
    // A drag past a column's floor is clamped up to it, not honored verbatim.
    let mut too_small = std::collections::HashMap::new();
    too_small.insert(LogColumn::Sha, 0);
    let w = LogColumnWidths::for_area_width(84, &no_hidden, &too_small);
    assert_eq!(w.sha_w, LogColumn::Sha.min_width());
}

fn cells(w: &LogColumnWidths) -> [u16; 7] {
    [
        w.ref_w,
        w.graph_w,
        w.message_w,
        w.author_w,
        w.date_w,
        w.changes_w,
        w.sha_w,
    ]
}

/// The core of the window-resize fix: Ref, Graph (and every other fixed
/// column) keep their cells whatever the pane width; `Message` alone
/// absorbs the difference. Growing back restores the user's widths
/// exactly, because nothing about a resize is stored.
///
/// Falsified: derived Graph from a share of the pane again, watched it
/// change between 120 and 200, restored.
#[test]
fn a_resize_only_moves_the_message_column() {
    let no_hidden = changes_off();
    let mut overrides = std::collections::HashMap::new();
    overrides.insert(LogColumn::Graph, 13);
    overrides.insert(LogColumn::Ref, 25);
    for set in [std::collections::HashMap::new(), overrides] {
        let at = |w: u16| LogColumnWidths::for_area_width(w, &no_hidden, &set);
        let base = at(120);
        for wider in [140u16, 200, 300] {
            let w = at(wider);
            assert_eq!(w.ref_w, base.ref_w, "Ref moved at {wider}");
            assert_eq!(w.graph_w, base.graph_w, "Graph moved at {wider}");
            assert_eq!(w.author_w, base.author_w);
            assert_eq!(w.message_w, base.message_w + (wider - 120));
        }
        for narrower in [110u16, 105] {
            let w = at(narrower);
            assert_eq!(w.ref_w, base.ref_w, "Ref moved at {narrower}");
            assert_eq!(w.graph_w, base.graph_w, "Graph moved at {narrower}");
            assert_eq!(w.message_w, base.message_w - (120 - narrower));
        }
        // Too narrow for the fixed columns: they give way, Message first,
        // then Graph before Ref ...
        let tight = at(70);
        assert!(tight.graph_w < base.graph_w || tight.ref_w < base.ref_w);
        assert!(tight.message_w >= LogColumn::Message.min_width());
        // ... and nothing was kept: the same inputs at 120 again.
        assert_eq!(cells(&at(120)), cells(&base));
    }
}

#[test]
fn the_users_graph_and_ref_widths_are_cells_not_shares() {
    let no_hidden = changes_off();
    let mut o = std::collections::HashMap::new();
    o.insert(LogColumn::Graph, 12);
    o.insert(LogColumn::Ref, 18);
    for width in [110u16, 160, 240] {
        let w = LogColumnWidths::for_area_width(width, &no_hidden, &o);
        assert_eq!((w.graph_w, w.ref_w), (12, 18), "at {width}");
    }
}

/// `Message` is flexible: an override for it (an older version stored
/// one) is ignored rather than pinning it.
#[test]
fn a_message_override_does_not_pin_the_message_column() {
    let no_hidden = changes_off();
    let natural = LogColumnWidths::for_area_width(120, &no_hidden, &Default::default());
    let mut pinned = std::collections::HashMap::new();
    pinned.insert(LogColumn::Message, 20);
    let w = LogColumnWidths::for_area_width(120, &no_hidden, &pinned);
    assert_eq!(cells(&w), cells(&natural));
}

#[test]
fn column_x_offsets_are_left_to_right_and_contiguous() {
    let no_hidden = changes_off();
    let no_overrides = std::collections::HashMap::new();
    let widths = LogColumnWidths::for_area_width(84, &no_hidden, &no_overrides);
    let order = shown_order();
    let offsets = column_x_offsets(0, &order, &widths);
    assert_eq!(offsets.len(), 6);
    for pair in offsets.windows(2) {
        let (_, x0, w0) = pair[0];
        let (_, x1, _) = pair[1];
        assert_eq!(x0 + w0, x1, "columns must be contiguous, no gap/overlap");
    }
}

#[test]
fn log_hover_column_finds_the_containing_column() {
    let no_hidden = changes_off();
    let no_overrides = std::collections::HashMap::new();
    let widths = LogColumnWidths::for_area_width(84, &no_hidden, &no_overrides);
    let order = shown_order();
    let offsets = column_x_offsets(0, &order, &widths);
    let (_, ref_x, _) = offsets[0];
    assert_eq!(log_hover_column(&offsets, ref_x), Some(LogColumn::Ref));
    assert_eq!(log_hover_column(&offsets, 1000), None);
}
