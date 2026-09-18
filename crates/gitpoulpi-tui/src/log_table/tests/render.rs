//! Rendering the Log through the real table: the graph column and row highlight.

use super::*;
use super::{changes_off, log_geometry, shown_order};
use crate::test_support::*;

/// The graph, through the **real Log table**, cell by cell out of a
/// `TestBackend` buffer. `crate::graph`'s own tests prove the parts and
/// their connections; this one proves the Graph *column* actually shows
/// them - `build_graph_lines` can be perfectly right while the column
/// is too narrow to contain lane 0, or while the table's own spacing
/// shifts what lands where. Both have happened.
///
/// Four commits on one lane, the topmost carrying a branch: the column
/// must read filled-then-hollow top to bottom, with no blank cell.
#[test]
fn the_rendered_graph_column_shows_a_marker_on_every_commit_row() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    let nodes = vec![
        commit(1, &[2]),
        commit(2, &[3]),
        commit(3, &[4]),
        commit(4, &[]),
    ];
    let refs = vec![local_branch_at("main", oid(1))];
    let graph = crate::graph::build_graph_lines(&nodes, &refs);

    let (log_area, offsets) = log_geometry();
    let widths = LogColumnWidths::for_area_width(
        log_area.width,
        &changes_off(),
        &std::collections::HashMap::new(),
    );
    let order = shown_order();
    let graph_x = offsets
        .iter()
        .find(|(c, _, _)| *c == LogColumn::Graph)
        .map(|&(_, x, _)| x)
        .expect("the Graph column is visible");

    let rows: Vec<Row> = graph
        .rows
        .iter()
        .map(|(line, _)| {
            Row::new(
                order
                    .iter()
                    .map(|&c| {
                        if c == LogColumn::Graph {
                            Cell::from(line.clone())
                        } else {
                            Cell::from("")
                        }
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect();

    let mut terminal = Terminal::new(TestBackend::new(
        log_area.width,
        log_area.y + log_area.height,
    ))
    .unwrap();
    let mut state = TableState::default();
    terminal
        .draw(|frame| {
            let header: Vec<Cell> = order.iter().map(|&c| Cell::from(c.title())).collect();
            frame.render_stateful_clipped(
                log_table(rows, header, &order, &widths, Style::default()),
                log_area,
                &mut state,
            );
        })
        .unwrap();

    let buf = terminal.backend().buffer().clone();
    let first_body_row = log_top_y(log_area);
    let column: Vec<String> = (0..nodes.len() as u16)
        .map(|i| buf[(graph_x, first_body_row + i)].symbol().to_string())
        .collect();
    assert_eq!(
        column,
        vec!["\u{25CF}", "\u{25CB}", "\u{25CB}", "\u{25CB}"],
        "the Graph column must carry a marker on every commit row - \
         filled where a ref points, hollow otherwise - and never a blank"
    );
}

/// A selected row's background runs unbroken from the Graph column to
/// the last column: text shorter than its column (a short message, an
/// author, a date) must not leave the rest of the column unhighlighted.
#[test]
fn a_selected_row_is_highlighted_across_every_column_without_holes() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    let nodes = [commit(1, &[])];
    let (log_area, offsets) = log_geometry();
    let widths = LogColumnWidths::for_area_width(
        log_area.width,
        &changes_off(),
        &std::collections::HashMap::new(),
    );
    let order = shown_order();
    let row = commit_row(
        &nodes[0],
        &[],
        Line::from("\u{25CB}"),
        Color::Blue,
        &widths,
        &order,
        true,
        false,
        None,
        false,
        avatar::LogAvatarMode::Off,
        gitpoulpi_config::AvatarFallback::default(),
        None,
    );
    let mut terminal = Terminal::new(TestBackend::new(
        log_area.width,
        log_area.y + log_area.height,
    ))
    .unwrap();
    let mut state = TableState::default();
    state.select(Some(0));
    terminal
        .draw(|frame| {
            let header: Vec<Cell> = order.iter().map(|&c| Cell::from(c.title())).collect();
            frame.render_stateful_clipped(
                log_table(vec![row], header, &order, &widths, Style::default()),
                log_area,
                &mut state,
            );
        })
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let y = log_top_y(log_area);
    let graph_x = offsets
        .iter()
        .find(|(c, _, _)| *c == LogColumn::Graph)
        .map(|&(_, x, _)| x)
        .expect("the Graph column is visible");
    let end = offsets
        .iter()
        .map(|&(_, x, w)| x + w)
        .max()
        .expect("columns");
    let holes: Vec<u16> = (graph_x..end)
        .filter(|&x| buf[(x, y)].bg != th().ui.selection)
        .collect();
    assert!(holes.is_empty(), "unhighlighted cells at x = {holes:?}");
}
