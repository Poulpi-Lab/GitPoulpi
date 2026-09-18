//! A selected (or hovered) Log row's highlight band starts
//! at the commit's node. The lane cells left of it keep the plain background
//! (the other lanes' lines are drawn on it); the node, the rest of the Graph
//! column and every column to the right carry the selection color.

use super::*;
use crate::test_support::*;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

const ORDER: [LogColumn; 2] = [LogColumn::Graph, LogColumn::Message];

/// Row 2's commit sits in lane 2, so two live lanes lie left of its node.
fn nodes() -> Vec<CommitNode> {
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

/// Background of every cell of display row 2 (the lane-2 commit), the
/// Graph cells first, plus the node's index within the Graph column.
fn bgs(selected: bool, hovered: bool) -> (Vec<ratatui::style::Color>, usize, u16) {
    let nodes = nodes();
    let graph = crate::graph::build_graph_lines(&nodes, &[]);
    let node = crate::graph::node_column(&graph.grid.rows[2].cells).expect("a node");
    let widths = LogColumnWidths {
        graph_w: 10,
        message_w: 6,
        ..Default::default()
    };
    let rows: Vec<Row> = nodes
        .iter()
        .zip(graph.rows)
        .enumerate()
        .map(|(i, (c, (line, color)))| {
            commit_row_with_changes(RowCtx {
                commit: c,
                pills: &[],
                graph: line,
                graph_node: crate::graph::node_column(&graph.grid.rows[i].cells),
                lane_color: color,
                widths: &widths,
                visible_order: &ORDER,
                is_selected: selected && i == 2,
                is_hovered: hovered && i == 2,
                popup_fill_width: None,
                is_dimmed: false,
                avatar_mode: avatar::LogAvatarMode::Off,
                avatar_fallback: gitpoulpi_config::AvatarFallback::Initials,
                ref_connector: None,
                changes: None,
                ghost: None,
            })
        })
        .collect();
    let area = Rect::new(0, 0, 18, 10);
    let mut terminal = Terminal::new(TestBackend::new(area.width, area.height)).unwrap();
    let mut state = TableState::default();
    terminal
        .draw(|frame| {
            let header: Vec<Cell> = ORDER.iter().map(|&c| Cell::from(c.title())).collect();
            frame.render_stateful_clipped(
                log_table(rows, header, &ORDER, &widths, Style::default()),
                area,
                &mut state,
            );
        })
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let y = log_top_y(area) + 2;
    let out = (1..17).map(|x| buf[(x, y)].bg).collect();
    (out, node, widths.graph_w)
}

#[test]
fn the_selection_band_starts_at_the_node_in_the_text_graph() {
    let sel = th().ui.selection;
    let (bg, node, gw) = bgs(true, false);
    assert!(node >= 2, "lane 2 has two lanes to its left (node {node})");
    for (x, c) in bg.iter().enumerate().take(node) {
        assert_ne!(
            *c, sel,
            "cell {x} left of the node keeps the plain background"
        );
    }
    for (x, c) in bg.iter().enumerate().skip(node) {
        assert_eq!(*c, sel, "cell {x} from the node on carries the selection");
    }
    assert_eq!(bg.len(), usize::from(gw) + 6);
}

#[test]
fn the_hover_band_starts_at_the_node_too() {
    let hover = th().ui.hover;
    let (bg, node, _) = bgs(false, true);
    assert!(bg[..node].iter().all(|c| *c != hover));
    assert!(bg[node..].iter().all(|c| *c == hover));
}

#[test]
fn an_unselected_row_has_no_band() {
    let sel = th().ui.selection;
    let (bg, ..) = bgs(false, false);
    assert!(bg.iter().all(|c| *c != sel));
}
