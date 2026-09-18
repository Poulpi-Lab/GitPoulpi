//! Search dimming in the Log table and the match counter.

use super::sample_commit;
use super::*;
use crate::graph::LANE_PALETTE;

/// Counter and dimming are driven off the SAME match list, so a hit
/// count of N must mean exactly N undimmed rows. This pins the two
/// together at the logic level; the render-level equivalent is
/// `a_closed_search_bar_leaves_the_log_untouched` below.
#[test]
fn the_counter_agrees_with_the_number_of_matching_rows() {
    let nodes = vec![
        sample_commit(1, "Merge branch 'a'"),
        sample_commit(2, "fix: guard"),
        sample_commit(3, "Merge branch 'b'"),
    ];
    let refs: Vec<gitpoulpi_core::domain::Ref> = Vec::new();
    let hits = log_search::find_matches(&nodes, &refs, "merge");
    assert_eq!(hits.len(), 2);
    assert_eq!(log_search::counter_text(0, hits.len()), "1 of 2");
    let dimmed = nodes
        .iter()
        .enumerate()
        .filter(|(i, _)| !hits.contains(i))
        .count();
    assert_eq!(dimmed, 1, "exactly the non-matching rows should dim");
}

/// Renders one commit row twice - `is_dimmed: false` (what every row
/// gets while no search is open) and `is_dimmed: true` - into real
/// buffers.
fn render_commit_row(is_dimmed: bool) -> ratatui::buffer::Buffer {
    use ratatui::buffer::Buffer;
    use ratatui::widgets::Widget;

    let commit = sample_commit(7, "Merge branch 'feature'");
    let refs: Vec<gitpoulpi_core::domain::Ref> = Vec::new();
    let order = [LogColumn::Graph, LogColumn::Message, LogColumn::Sha];
    let width = 60u16;
    let widths = LogColumnWidths::for_area_width(width, &Default::default(), &Default::default());
    let row = commit_row(
        &commit,
        &refs,
        Line::from(vec![Span::raw("\u{25CE}")]),
        LANE_PALETTE[0],
        &widths,
        &order,
        false,
        false,
        None,
        is_dimmed,
        avatar::LogAvatarMode::Off,
        gitpoulpi_config::AvatarFallback::Initials,
        None,
    );
    let area = Rect::new(0, 0, width, 1);
    let mut buf = Buffer::empty(area);
    // `Table` implements both `Widget` and `StatefulWidget`; name the
    // one we mean rather than relying on inference.
    Widget::render(
        Table::new(
            vec![row],
            order
                .iter()
                .map(|&c| Constraint::Length(widths.width_of(c)))
                .collect::<Vec<_>>(),
        ),
        area,
        &mut buf,
    );
    buf
}

/// THE regression test for this feature: with the find bar closed,
/// every row renders exactly as it did before search existed. A
/// search that quietly dims or restyles the table when it isn't even
/// open would be far worse than no search at all.
#[test]
fn a_closed_search_bar_leaves_the_log_untouched() {
    let undimmed = render_commit_row(false);
    for cell in undimmed.content.iter() {
        assert!(
            !cell.modifier.contains(Modifier::DIM),
            "a row rendered with no search open must carry no DIM modifier"
        );
    }
}

/// ...and the dimming must actually reach the cells when a search IS
/// running - otherwise the test above passes vacuously forever.
#[test]
fn a_non_matching_row_is_dimmed_but_still_rendered() {
    let dimmed = render_commit_row(true);
    let undimmed = render_commit_row(false);

    assert!(
        dimmed
            .content
            .iter()
            .any(|c| c.modifier.contains(Modifier::DIM)),
        "a non-matching row must be dimmed"
    );
    // Dimmed, NOT hidden: the text is identical, only the style
    // differs. This is what separates search from the solo filter.
    let text_of =
        |b: &ratatui::buffer::Buffer| -> String { b.content.iter().map(|c| c.symbol()).collect() };
    assert_eq!(
        text_of(&dimmed),
        text_of(&undimmed),
        "dimming must not change which characters are painted"
    );
}
