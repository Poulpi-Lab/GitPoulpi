//! Buffer and golden tests for the ref-to-commit connector
//! (`ui.ref_connector`), text renderer: the Log table is
//! drawn through the same `commit_row` / `log_table` the event loop uses,
//! and the Ref + Graph columns are read back cell by cell.

use super::*;
use crate::test_support::*;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::Terminal;

const REF_W: u16 = 16;
const GRAPH_W: u16 = 8;
const MESSAGE_W: u16 = 12;
const ORDER: [LogColumn; 3] = [LogColumn::Ref, LogColumn::Graph, LogColumn::Message];

/// Three branch tips over one root, so the third tip's commit sits in
/// lane 2 with lanes 0 and 1 still live beside it:
///
/// ```text
/// row 0  ●          commit 1 (lane 0)
/// row 1  │ ●        commit 2 (lane 1)
/// row 2  │ │ ●      commit 3 (lane 2)   <- lanes to cross
/// ...
/// ```
fn three_lanes() -> Vec<CommitNode> {
    vec![
        commit(1, &[4]),
        commit(2, &[5]),
        commit(3, &[6]),
        commit(4, &[7]),
        commit(5, &[7]),
        commit(6, &[7]),
        commit(7, &[]),
    ]
}

fn widths(ref_w: u16) -> LogColumnWidths {
    LogColumnWidths {
        ref_w,
        graph_w: GRAPH_W,
        message_w: MESSAGE_W,
        ..Default::default()
    }
}

/// The table, through the real row builder and table. `connector` is
/// `ui.ref_connector` as the event loop resolves it for this order.
fn render(
    nodes: &[CommitNode],
    refs: &[gitpoulpi_core::domain::Ref],
    widths: &LogColumnWidths,
    order: &[LogColumn],
    connector: bool,
) -> (Buffer, Rect) {
    let graph = crate::graph::build_graph_lines(nodes, refs);
    let on = ref_connector_applies(connector, order);
    let rows: Vec<Row> = nodes
        .iter()
        .zip(graph.rows)
        .enumerate()
        .map(|(i, (c, (line, color)))| {
            commit_row(
                c,
                refs,
                line,
                color,
                widths,
                order,
                false,
                false,
                None,
                false,
                avatar::LogAvatarMode::Off,
                gitpoulpi_config::AvatarFallback::Initials,
                on.then(|| RefConnector {
                    graph_row: graph.grid.rows.get(i),
                }),
            )
        })
        .collect();
    let width: u16 = order.iter().map(|&c| widths.width_of(c)).sum::<u16>() + 2;
    let area = Rect::new(0, 0, width, nodes.len() as u16 + 3);
    let mut terminal = Terminal::new(TestBackend::new(area.width, area.height)).unwrap();
    let mut state = TableState::default();
    terminal
        .draw(|frame| {
            let header: Vec<Cell> = order.iter().map(|&c| Cell::from(c.title())).collect();
            frame.render_stateful_clipped(
                log_table(rows, header, order, widths, Style::default()),
                area,
                &mut state,
            );
        })
        .unwrap();
    (terminal.backend().buffer().clone(), area)
}

/// Display row `index`'s cells from `x` for `w` columns, as text. The
/// cell a wide glyph (the pill's `💻`) spills into is skipped, so the
/// string reads the way the terminal paints it.
fn cells(buf: &Buffer, area: Rect, index: usize, x: u16, w: u16) -> String {
    let y = log_top_y(area) + index as u16;
    let mut out = String::new();
    let mut skip = 0;
    for x in x..x + w {
        if skip > 0 {
            skip -= 1;
            continue;
        }
        let symbol = buf[(x, y)].symbol();
        skip = display_width(symbol).saturating_sub(1);
        out.push_str(symbol);
    }
    out
}

/// The Ref column then the Graph column of display row `index`, split.
fn ref_and_graph(buf: &Buffer, area: Rect, w: &LogColumnWidths, index: usize) -> (String, String) {
    let x = area.x + 1;
    (
        cells(buf, area, index, x, w.ref_w),
        cells(buf, area, index, x + w.ref_w, w.graph_w),
    )
}

/// The pill's own text for a local branch `name`, as `ref_pill_spans`
/// lays it out, so the goldens below spell out only the connector.
fn pill(name: &str, width: usize) -> String {
    let pills = ref_pills_for_commit(oid(0xEE), &[local_branch_at(name, oid(0xEE))]);
    ref_pill_spans(&pills, width, Color::Reset, None)
        .iter()
        .map(|s| s.content.to_string())
        .collect()
}

const H: &str = "\u{2500}"; // ─, the connector

/// **Lane 0.** The node is the Graph column's first cell, so the whole
/// connector is the Ref column's padding: `─` from the end of the pill to
/// the column's edge, right against the node. The Graph column is
/// untouched, and a row without a pill has no connector at all.
#[test]
fn a_ref_in_lane_0_is_joined_by_the_ref_columns_padding_alone() {
    let nodes = three_lanes();
    let refs = vec![local_branch_at("main", oid(1))];
    let w = widths(REF_W);
    let (on, area) = render(&nodes, &refs, &w, &ORDER, true);
    let (off, _) = render(&nodes, &refs, &w, &ORDER, false);

    let label = pill("main", REF_W as usize);
    let (ref_on, graph_on) = ref_and_graph(&on, area, &w, 0);
    let (ref_off, graph_off) = ref_and_graph(&off, area, &w, 0);
    // Golden: the pill, then `─` to the column's edge.
    let pad = REF_W as usize - display_width(&label);
    assert!(pad > 0, "the pill leaves room: {label:?}");
    assert_eq!(ref_on, format!("{label}{}", H.repeat(pad)));
    assert_eq!(ref_off, format!("{label}{}", " ".repeat(pad)));
    // The Graph column is exactly what it was: the node, first cell.
    assert_eq!(graph_on, graph_off);
    assert!(graph_on.starts_with('\u{25CF}'), "{graph_on:?}");

    // Rows with no pill: nothing anywhere.
    for index in 1..nodes.len() {
        assert_eq!(
            ref_and_graph(&on, area, &w, index),
            ref_and_graph(&off, area, &w, index),
            "row {index} has no pill and must be untouched"
        );
    }
}

/// **Lane 2, with lanes 0 and 1 in between.** The connector runs from the
/// Ref column's edge across the Graph column's blank cells to the node --
/// and **skips** the two live lanes: their `│` stay exactly where and what
/// they were. Nothing shifts: every non-blank cell is identical with the
/// connector off.
#[test]
fn a_ref_in_lane_2_crosses_the_lanes_between_without_touching_them() {
    let nodes = three_lanes();
    let refs = vec![local_branch_at("feature", oid(3))];
    let w = widths(REF_W);
    let (on, area) = render(&nodes, &refs, &w, &ORDER, true);
    let (off, _) = render(&nodes, &refs, &w, &ORDER, false);

    let (ref_off, graph_off) = ref_and_graph(&off, area, &w, 2);
    assert_eq!(
        graph_off, "\u{2502} \u{2502} \u{25CF}   ",
        "the layout under test"
    );

    let (ref_on, graph_on) = ref_and_graph(&on, area, &w, 2);
    let label = pill("feature", REF_W as usize);
    let pad = REF_W as usize - display_width(&label);
    // Golden, both halves.
    assert_eq!(ref_on, format!("{label}{}", H.repeat(pad)));
    assert_eq!(graph_on, format!("\u{2502}{H}\u{2502}{H}\u{25CF}   "));
    assert_eq!(ref_off, format!("{label}{}", " ".repeat(pad)));

    // Cell by cell: only blanks left of the node changed, and to `─`.
    let (on_cells, off_cells): (Vec<char>, Vec<char>) =
        (graph_on.chars().collect(), graph_off.chars().collect());
    let node = off_cells.iter().position(|&c| c == '\u{25CF}').unwrap();
    for (j, (a, b)) in on_cells.iter().zip(&off_cells).enumerate() {
        if j < node && *b == ' ' {
            assert_eq!(*a, '\u{2500}', "blank cell {j} left of the node");
        } else {
            assert_eq!(a, b, "cell {j} must not change");
        }
    }
    // The connector is dimmed, in the node's lane color.
    let y = log_top_y(area) + 2;
    let gx = area.x + 1 + w.ref_w;
    let node_fg = on[(gx + node as u16, y)].fg;
    let conn = &on[(gx + 1, y)];
    assert_eq!(conn.fg, node_fg);
    assert!(conn.modifier.contains(Modifier::DIM));
}

/// **The pill column truncated.** A branch name too long for the Ref
/// column fills it to the edge (truncated with its ellipsis): there is no
/// room for padding and none is forced in - the Ref column is exactly
/// the connector-off rendering. The Graph column's half is still drawn,
/// so the line still reaches the node.
#[test]
fn a_truncated_pill_leaves_no_padding_but_the_graph_half_still_reaches_the_node() {
    let nodes = three_lanes();
    let refs = vec![local_branch_at("a-very-long-feature-branch-name", oid(3))];
    let narrow = 12;
    let w = widths(narrow);
    let (on, area) = render(&nodes, &refs, &w, &ORDER, true);
    let (off, _) = render(&nodes, &refs, &w, &ORDER, false);
    let (ref_on, graph_on) = ref_and_graph(&on, area, &w, 2);
    let (ref_off, _) = ref_and_graph(&off, area, &w, 2);
    assert!(ref_on.contains('\u{2026}'), "truncated: {ref_on:?}");
    assert!(
        !ref_on.contains('\u{2500}'),
        "no room, no padding: {ref_on:?}"
    );
    assert_eq!(ref_on, ref_off);
    assert_eq!(graph_on, format!("\u{2502}{H}\u{2502}{H}\u{25CF}   "));
    // And the Message column starts where it always did.
    let mx = area.x + 1 + w.ref_w + w.graph_w;
    assert_eq!(
        cells(&on, area, 2, mx, w.message_w),
        cells(&off, area, 2, mx, w.message_w)
    );
}

/// With the Ref column moved away from the Graph column (columns are
/// reorderable) or the setting off, no connector is drawn at all - a
/// line would otherwise have to run through the Message column.
#[test]
fn no_connector_unless_the_ref_column_sits_right_before_the_graph() {
    assert!(ref_connector_applies(true, &ORDER));
    assert!(!ref_connector_applies(false, &ORDER));
    assert!(!ref_connector_applies(
        true,
        &[LogColumn::Ref, LogColumn::Message, LogColumn::Graph]
    ));
    assert!(!ref_connector_applies(
        true,
        &[LogColumn::Graph, LogColumn::Ref, LogColumn::Message]
    ));
    assert!(!ref_connector_applies(
        true,
        &[LogColumn::Graph, LogColumn::Message]
    ));
    assert!(ref_connector_applies(true, &LogColumn::ALL));

    let nodes = three_lanes();
    let refs = vec![local_branch_at("feature", oid(3))];
    let w = widths(REF_W);
    let apart = [LogColumn::Ref, LogColumn::Message, LogColumn::Graph];
    let (on, area) = render(&nodes, &refs, &w, &apart, true);
    let (off, _) = render(&nodes, &refs, &w, &apart, false);
    assert_eq!(on, off);
    let _ = area;
}
