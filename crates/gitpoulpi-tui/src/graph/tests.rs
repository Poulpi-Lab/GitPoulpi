//! Graph tests.
//!
//! Two layers, deliberately:
//!
//! - **Topology**, against the [`GraphGrid`]: which part landed in which
//!   cell. Each regression test pins a defect first seen in a real terminal.
//! - **Rendering**, against a real `TestBackend` buffer -- what the
//!   terminal is actually handed. `where-we-are.md` warns that a test
//!   asserting on the strings you just generated is not evidence, so every
//!   claim about connection is checked against the buffer as well as the
//!   grid.
//!
//! Each test was checked red by reverting the corresponding line of
//! `layout.rs` or `part.rs`.

use super::layout::{
    build_graph_grid, build_graph_grid_with_wip, build_graph_lines, build_graph_lines_with,
    WipAnchor,
};
use super::part::{NodeKind, NodeSet, Sides, TreeCell, TreePart};
use super::{GraphSettings, LogLoadSettings};
use crate::display_width;
use gitpoulpi_core::domain::{CommitNode, Oid, Ref, RefKind};
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::Terminal;

fn oid(n: u8) -> Oid {
    Oid([n; 20])
}

fn commit(id: u8, parents: &[u8]) -> CommitNode {
    CommitNode {
        id: oid(id),
        parents: parents.iter().map(|&p| oid(p)).collect(),
        summary: format!("commit {id}"),
        message: format!("commit {id}"),
        author_name: "a".into(),
        author_email: "a@example.com".into(),
        timestamp: 0,
        refs: Vec::new(),
    }
}

fn local_branch_at(name: &str, target: Oid) -> Ref {
    Ref {
        kind: RefKind::LocalBranch { name: name.into() },
        target,
        visibility: gitpoulpi_core::domain::RefVisibility::Shown,
        is_head: false,
        upstream: None,
        tag_sync: None,
    }
}

/// The parts of one row, as the layout produced them.
fn parts(nodes: &[CommitNode], refs: &[Ref], row: usize) -> Vec<TreePart> {
    build_graph_grid(nodes, refs).rows[row]
        .cells
        .iter()
        .map(|c| c.part)
        .collect()
}

/// The glyphs of one row, as the renderer spells them under the default
/// set. Used only where a test is genuinely about the characters.
fn glyphs(nodes: &[CommitNode], refs: &[Ref], row: usize) -> Vec<String> {
    build_graph_lines(nodes, refs).rows[row]
        .0
        .spans
        .iter()
        .map(|s| s.content.to_string())
        .collect()
}

fn node(kind: NodeKind) -> impl Fn(&TreePart) -> bool {
    move |p| matches!(p, TreePart::Node { kind: k, .. } if *k == kind)
}

// ---------------------------------------------------------------------------
// The glyph contract
// ---------------------------------------------------------------------------

/// Glyph width is a portability question. Anything width-2 in some terminal
/// (an emoji-presentation code point, a Wide East-Asian one) pushes every
/// lane to its right half a column out of line on that machine only.
/// Asserted to be a single `char` too, so **no variation selector and no
/// combining mark can sneak back in** (U+20DD is absent from every monospace
/// font on Windows). See `part.rs`'s header for the font-coverage table.
#[test]
fn every_tree_part_is_one_cell_and_one_code_point() {
    for set in [NodeSet::Circles] {
        for part in TreePart::ALL {
            let glyph = part.glyph(set);
            assert_eq!(
                glyph.chars().count(),
                1,
                "{set:?} spells {part:?} as {glyph:?}, which is not a single \
                 code point. A combining mark or a variation selector renders \
                 differently per terminal - and per *font*: U+20DD is in none \
                 of the monospace fonts a Windows terminal uses."
            );
            assert_eq!(
                display_width(glyph),
                1,
                "{set:?} spells {part:?} as {glyph:?}, which is not one cell \
                 wide; it would shear the graph column"
            );
        }
    }
}

/// The three commit shapes, spelled out so a future "improvement" has to
/// delete an explicit assertion rather than quietly change a constant.
#[test]
fn the_default_set_is_hollow_filled_and_double_circles() {
    let set = NodeSet::Circles;
    let glyph = |kind| {
        TreePart::Node {
            kind,
            sides: super::part::Sides::VERTICAL,
        }
        .glyph(set)
    };
    assert_eq!(glyph(NodeKind::Commit), "\u{25CB}", "ordinary commit: ○");
    assert_eq!(glyph(NodeKind::Ref), "\u{25CF}", "ref-pointed commit: ●");
    assert_eq!(glyph(NodeKind::Merge), "\u{25CE}", "merge commit: ◎");
}

// ---------------------------------------------------------------------------
// Connectivity -- the property that kept failing
// ---------------------------------------------------------------------------

/// Walks the whole grid and asserts every declared connection is mutual:
/// a cell that reaches right has a right-hand neighbor that reaches left,
/// a cell that reaches down has a cell below it that reaches up. Returns
/// the first violation as a human-readable string.
fn first_break(grid: &[Vec<TreeCell>]) -> Option<String> {
    let rows = grid.len();
    let width = grid.first().map(Vec::len).unwrap_or(0);
    for y in 0..rows {
        for x in 0..width {
            let here = grid[y][x].part;
            let s = here.sides();
            if s.right {
                match grid[y].get(x + 1) {
                    Some(n) if n.part.sides().left => {}
                    other => {
                        return Some(format!(
                            "({x},{y}) {here:?} reaches right, but {:?} does not reach back",
                            other.map(|c| c.part)
                        ))
                    }
                }
            }
            if s.left {
                match x.checked_sub(1).and_then(|p| grid[y].get(p)) {
                    Some(n) if n.part.sides().right => {}
                    other => {
                        return Some(format!(
                            "({x},{y}) {here:?} reaches left, but {:?} does not reach back",
                            other.map(|c| c.part)
                        ))
                    }
                }
            }
            // Top and bottom of the grid are boundaries, not breaks: the
            // log is a window onto history, so the topmost row's lanes run
            // off the top and the bottom row's run off the bottom.
            if s.down && y + 1 < rows && !grid[y + 1][x].part.sides().up {
                return Some(format!(
                    "({x},{y}) {here:?} reaches down, but {:?} below it does not reach up",
                    grid[y + 1][x].part
                ));
            }
            if s.up && y > 0 && !grid[y - 1][x].part.sides().down {
                return Some(format!(
                    "({x},{y}) {here:?} reaches up, but {:?} above it does not reach down",
                    grid[y - 1][x].part
                ));
            }
        }
    }
    None
}

/// Every shape the layout can produce, checked for mutual connection cell by
/// cell. **This is the test the whole `TreePart` vocabulary exists for.**
///
/// It catches the defects that otherwise only a screenshot showed: a fan-out
/// to an adjacent lane with no horizontal fill, a run over a dead lane
/// leaving a hole, a run over another lane's elbow cut in two.
///
/// Red without the `for k in lo..hi` connector fill in `layout.rs`: the node
/// reaches right into blank space and the test names the cell.
#[test]
fn no_node_is_left_unconnected() {
    let shapes: Vec<(&str, Vec<CommitNode>)> = vec![
        (
            "a straight chain",
            vec![commit(1, &[2]), commit(2, &[3]), commit(3, &[])],
        ),
        (
            "a merge fanning out to a new lane on the right",
            vec![commit(1, &[2, 3]), commit(2, &[]), commit(3, &[])],
        ),
        (
            "a merge, then a convergence several rows later",
            vec![
                commit(1, &[2, 3]),
                commit(3, &[4]),
                commit(2, &[4]),
                commit(4, &[]),
            ],
        ),
        (
            "an octopus whose run crosses a dead lane",
            vec![
                commit(1, &[2, 3, 4]),
                commit(3, &[5]),
                commit(5, &[]),
                commit(2, &[4]),
                commit(4, &[]),
            ],
        ),
        (
            "an octopus whose run crosses a live lane",
            vec![
                commit(1, &[2, 3, 4]),
                commit(3, &[5]),
                commit(2, &[4]),
                commit(4, &[]),
                commit(5, &[]),
            ],
        ),
        (
            "three lanes converging on one node at once",
            vec![
                commit(1, &[2, 3, 4]),
                commit(2, &[5]),
                commit(3, &[5]),
                commit(4, &[5]),
                commit(5, &[]),
            ],
        ),
        (
            "two independent roots",
            vec![commit(1, &[2]), commit(2, &[]), commit(3, &[])],
        ),
        (
            "a merge on the very first row",
            vec![commit(3, &[1, 2]), commit(1, &[]), commit(2, &[])],
        ),
    ];
    for (what, nodes) in shapes {
        let grid = build_graph_grid(&nodes, &[]).rectangular();
        assert_eq!(
            first_break(&grid),
            None,
            "{what}: the graph has a cell that claims a connection nothing \
             joins.\n{}",
            render_ascii(&grid)
        );
    }
}

/// A grid dump for failure messages: a picture of the tree reads better
/// than a list of `TreePart` debug strings.
fn render_ascii(grid: &[Vec<TreeCell>]) -> String {
    grid.iter()
        .map(|row| {
            row.iter()
                .map(|c| c.part.glyph(NodeSet::Circles))
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The same claim, against the buffer a terminal is handed rather than the
/// structure the test built. Two properties:
///
/// 1. every cell's rendered symbol is exactly its part's glyph (nothing is
///    dropped or blanked by ratatui);
/// 2. for every **node**, each direction it claims a connection in has a
///    non-blank neighbor in the buffer.
#[test]
fn the_rendered_buffer_puts_something_on_every_side_a_node_claims() {
    let nodes = vec![
        commit(1, &[2, 3, 4]),
        commit(3, &[5]),
        commit(2, &[4]),
        commit(4, &[]),
        commit(5, &[]),
    ];
    let refs = vec![local_branch_at("main", oid(1))];
    let grid = build_graph_grid(&nodes, &refs).rectangular();
    let lines = build_graph_lines(&nodes, &refs).rows;
    let width = grid[0].len() as u16;
    let height = grid.len() as u16;

    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal
        .draw(|frame| {
            for (y, (line, _)) in lines.iter().enumerate() {
                frame.render_widget(
                    Paragraph::new(line.clone()),
                    Rect::new(0, y as u16, width, 1),
                );
            }
        })
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let at = |x: usize, y: usize| buf[(x as u16, y as u16)].symbol().to_string();

    for (y, row) in grid.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            assert_eq!(
                at(x, y),
                cell.part.glyph(NodeSet::Circles),
                "({x},{y}) rendered as {:?} but its part is {:?}",
                at(x, y),
                cell.part
            );
        }
    }

    let mut nodes_seen = 0;
    for (y, row) in grid.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            if !matches!(cell.part, TreePart::Node { .. }) {
                continue;
            }
            nodes_seen += 1;
            let s = cell.part.sides();
            let neighbors = [
                (s.left, x.checked_sub(1).map(|p| (p, y)), "left"),
                (s.right, Some((x + 1, y)), "right"),
                (s.up, y.checked_sub(1).map(|p| (x, p)), "up"),
                (s.down, Some((x, y + 1)), "down"),
            ];
            for (claimed, pos, which) in neighbors {
                if !claimed {
                    continue;
                }
                let Some((nx, ny)) = pos else { continue };
                if nx >= row.len() || ny >= grid.len() {
                    continue; // the edge of the window, not a break
                }
                assert_ne!(
                    at(nx, ny),
                    " ",
                    "the node at ({x},{y}) claims a connection {which}, but \
                     ({nx},{ny}) is blank in the rendered buffer - that is a \
                     node floating beside its own lane.\n{}",
                    render_ascii(&grid)
                );
            }
        }
    }
    assert!(nodes_seen >= 5, "the fixture must actually contain nodes");
}

/// Renders a history into a real `TestBackend` and hands back the buffer
/// plus the grid it was built from.
fn render_to_buffer(
    nodes: &[CommitNode],
    refs: &[Ref],
) -> (ratatui::buffer::Buffer, Vec<Vec<TreeCell>>) {
    render_to_buffer_with(nodes, refs, GraphSettings::default())
}

/// The same, at a chosen `GraphSettings`.
fn render_to_buffer_with(
    nodes: &[CommitNode],
    refs: &[Ref],
    settings: GraphSettings,
) -> (ratatui::buffer::Buffer, Vec<Vec<TreeCell>>) {
    let grid = build_graph_grid(nodes, refs).rectangular();
    let lines = build_graph_lines_with(nodes, refs, None, &[], settings).rows;
    let width = grid.iter().map(Vec::len).max().unwrap_or(0).max(1) as u16;
    let height = grid.len().max(1) as u16;
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal
        .draw(|frame| {
            for (y, (line, _)) in lines.iter().enumerate() {
                frame.render_widget(
                    Paragraph::new(line.clone()),
                    Rect::new(0, y as u16, width, 1),
                );
            }
        })
        .unwrap();
    (terminal.backend().buffer().clone(), grid)
}

/// Width discipline, measured on the **rendered buffer**: every cell the
/// graph paints is exactly one terminal column via the project's
/// `display_width`, never `str::len`. A width-2 symbol shears every lane to
/// its right on the machine whose font disagrees, and nowhere else.
#[test]
fn every_rendered_graph_cell_is_exactly_one_column_wide() {
    let nodes = vec![
        commit(1, &[2, 3]),
        commit(3, &[5]),
        commit(2, &[4]),
        commit(4, &[]),
        commit(5, &[]),
    ];
    let refs = vec![local_branch_at("main", oid(1))];
    let (buf, grid) = render_to_buffer(&nodes, &refs);
    let mut nodes_seen = 0;
    for (y, row) in grid.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            let symbol = buf[(x as u16, y as u16)].symbol();
            assert_eq!(
                display_width(symbol),
                1,
                "({x},{y}) rendered {symbol:?}, which is {} columns wide, \
                 not 1 - every lane to its right is now off by that much",
                display_width(symbol)
            );
            if matches!(cell.part, TreePart::Node { .. }) {
                nodes_seen += 1;
                assert_eq!(
                    symbol.chars().count(),
                    1,
                    "({x},{y}) is a node rendered as {symbol:?}: more than \
                     one char means a variation selector or a combining \
                     mark got back in, which sheared this column once"
                );
            }
        }
    }
    assert!(nodes_seen >= 5, "the fixture must actually contain nodes");
}

// ---------------------------------------------------------------------------
// Topology -- ported regressions
// ---------------------------------------------------------------------------

/// A merge commit that is also the very first rendered row must not let its
/// own second parent steal the lane slot `my_lane` itself is about to
/// occupy: `lanes[my_lane]` is still literally `None` at that point, so a
/// naive free-slot search could mistake it for free and silently destroy
/// that parent's lane tracking on the very next row.
#[test]
fn merge_at_the_very_first_row_does_not_let_its_own_parent_steal_its_lane() {
    let nodes = vec![commit(3, &[1, 2]), commit(1, &[]), commit(2, &[])];
    let row2 = parts(&nodes, &[], 2);
    assert!(
        row2.iter().any(|p| *p != TreePart::Empty),
        "commit 2's row should still show the fan-out connector, not render \
         as a disconnected fresh tip: {row2:?}"
    );
}

/// A merge whose base lane keeps going after the merge, while the merged-in
/// lane still has one more commit before the same ancestor. Convergence is
/// detected by the current row's own commit id, not by comparing lane
/// *targets*, which would collapse the base lane rows early.
#[test]
fn base_lane_keeps_rendering_through_rows_before_the_other_lane_converges() {
    let nodes = vec![
        commit(1, &[2, 3]),
        commit(3, &[4]),
        commit(2, &[4]),
        commit(4, &[]),
    ];
    let row1 = parts(&nodes, &[], 1);
    assert_eq!(
        row1[0],
        TreePart::Vertical,
        "lane 0 must keep rendering its own pipe through commit 3's row \
         instead of going blank or drawing an early convergence corner: {row1:?}"
    );
    let row3 = parts(&nodes, &[], 3);
    assert!(
        node(NodeKind::Commit)(&row3[0]),
        "commit 4 - the real convergence point - should render as a plain \
         commit node in lane 0: {row3:?}"
    );
    assert!(
        row3[0].sides().right && !row3[0].sides().left,
        "…with an arm to the right only: {row3:?}"
    );
}

/// A fresh second parent appended as a new lane to the *right* needs an
/// elbow that connects left (where the line arrives from) and down. Always
/// drawing `╭` would put the corner's open side away from the incoming line.
#[test]
fn fan_out_to_a_new_lane_on_the_right_curves_toward_the_incoming_line() {
    let nodes = vec![commit(1, &[2, 3]), commit(2, &[]), commit(3, &[])];
    let row0 = parts(&nodes, &[], 0);
    assert_eq!(
        row0[2],
        TreePart::BranchOutRight,
        "expected ╮ (left+down) at the new right-hand lane: {row0:?}"
    );
}

/// Merges and their fan-out lines continue in whichever lane's own color
/// they already are; a fixed merge hue would add an extra color to a small
/// palette on every merge row.
#[test]
fn merge_node_and_fan_out_use_their_own_lane_colors_not_a_universal_merge_color() {
    let nodes = vec![commit(1, &[2, 3]), commit(2, &[]), commit(3, &[])];
    let grid = build_graph_grid(&nodes, &[]);
    let row0 = &grid.rows[0];
    assert_eq!(row0.cells[0].color, row0.color);
    assert_ne!(
        row0.cells[2].color, row0.color,
        "the new lane's elbow must take its own lane color"
    );
}

/// The commonest fan-out, a merge whose second parent lands in the
/// immediately adjacent lane, still gets a horizontal fill: the range
/// `(lo + 1)..hi` is empty when `hi == lo + 1`, which would leave the corner
/// a blank cell away from the merge node.
#[test]
fn fan_out_to_an_adjacent_lane_connects_all_the_way_to_the_node() {
    let nodes = vec![commit(1, &[2, 3]), commit(2, &[]), commit(3, &[])];
    let row0 = parts(&nodes, &[], 0);
    assert!(node(NodeKind::Merge)(&row0[0]), "{row0:?}");
    assert_eq!(
        row0[1],
        TreePart::Horizontal,
        "the connector touching the merge node must be drawn, not left blank: {row0:?}"
    );
    assert_eq!(row0[2], TreePart::BranchOutRight, "{row0:?}");
}

/// Mirror of the fan-out case: a lane converging in from the adjacent column
/// must not leave its `╯` detached from the node it converges into.
#[test]
fn convergence_from_an_adjacent_lane_connects_all_the_way_to_the_node() {
    let nodes = vec![
        commit(1, &[2, 3]),
        commit(3, &[4]),
        commit(2, &[4]),
        commit(4, &[]),
    ];
    let row3 = parts(&nodes, &[], 3);
    assert!(node(NodeKind::Commit)(&row3[0]), "{row3:?}");
    assert_eq!(row3[1], TreePart::Horizontal, "{row3:?}");
    assert_eq!(row3[2], TreePart::MergeInRight, "{row3:?}");
}

/// A horizontal run passing over a *dead* intermediate lane must not leave
/// that lane's glyph cell blank and punch a hole in the line.
#[test]
fn a_run_crossing_a_dead_intermediate_lane_is_not_broken_by_it() {
    let nodes = vec![
        commit(1, &[2, 3, 4]),
        commit(3, &[5]),
        commit(5, &[]),
        commit(2, &[4]),
        commit(4, &[]),
    ];
    let row4 = parts(&nodes, &[], 4);
    assert!(node(NodeKind::Commit)(&row4[0]), "{row4:?}");
    assert_eq!(row4[1], TreePart::Horizontal, "{row4:?}");
    assert_eq!(
        row4[2],
        TreePart::Horizontal,
        "the run must continue through the dead intermediate lane instead of \
         leaving a hole: {row4:?}"
    );
    assert_eq!(row4[3], TreePart::Horizontal, "{row4:?}");
    assert_eq!(row4[4], TreePart::MergeInRight, "{row4:?}");
}

/// The same hole on the other axis: a *live* intermediate lane keeps its
/// own vertical, but a bare `│` inks the full cell height and none of its
/// width, so the run would lose pixels on each side. `┼` draws both lines
/// whole and is deliberately the one knot no node kind uses: a crossing
/// that looked like a node would claim a commit sits on the crossed lane.
#[test]
fn a_run_crossing_a_live_intermediate_lane_keeps_both_lines_whole() {
    let nodes = vec![
        commit(1, &[2, 3, 4]),
        commit(3, &[5]),
        commit(2, &[4]),
        commit(4, &[]),
        commit(5, &[]),
    ];
    let row3 = parts(&nodes, &[], 3);
    assert!(node(NodeKind::Commit)(&row3[0]), "{row3:?}");
    assert_eq!(row3[1], TreePart::Horizontal, "{row3:?}");
    assert_eq!(
        row3[2],
        TreePart::Crossing,
        "lane 1 is still live here, so its vertical must survive - but so \
         must the run passing over it: {row3:?}"
    );
    assert_eq!(row3[4], TreePart::MergeInRight, "{row3:?}");
    // ...and a live lane on a row with no run over it stays a plain pipe,
    // or "draw the crossing" quietly becomes "draw a knot on every row".
    let row1 = parts(&nodes, &[], 1);
    assert_eq!(row1[0], TreePart::Vertical, "{row1:?}");
}

/// When three lanes converge on one node, the outermost one's run passes
/// straight over the middle one's elbow cell. An elbow inks only the half
/// of its cell its own two arms reach, so the far half went blank and that
/// outer run came out cut in two. The elbow has to grow its third arm.
#[test]
fn a_corner_another_run_passes_over_grows_the_arm_that_run_needs() {
    let nodes = vec![
        commit(1, &[2, 3, 4]),
        commit(2, &[5]),
        commit(3, &[5]),
        commit(4, &[5]),
        commit(5, &[]),
    ];
    let row4 = parts(&nodes, &[], 4);
    assert_eq!(
        row4[2],
        TreePart::MergeInCrossed,
        "lane 1 converges here AND lane 2's run passes over it, so its elbow \
         needs the third arm or that run is cut in two: {row4:?}"
    );
    assert_eq!(
        row4[4],
        TreePart::MergeInRight,
        "lane 2 is the outermost: nothing passes over it, so it keeps the \
         rounded elbow: {row4:?}"
    );
}

/// A node grows the arms its own transitions need and no others: a merge
/// fanning out to the right must not also claim a left arm pointing into
/// blank space (`no_node_is_left_unconnected` catches it directly).
#[test]
fn a_node_grows_only_the_arms_its_own_transitions_need() {
    let nodes = vec![commit(1, &[2, 3]), commit(2, &[]), commit(3, &[])];
    let row0 = parts(&nodes, &[], 0);
    let s = row0[0].sides();
    assert!(
        s.right && !s.left,
        "the merge fans out right only: {row0:?}"
    );
    let row1 = parts(&nodes, &[], 1);
    let s = row1[0].sides();
    assert!(
        !s.right && !s.left,
        "a commit with no transition needs no arms: {row1:?}"
    );
}

/// A ref on a commit changes the marker to the filled circle, and nothing
/// else about the row.
#[test]
fn a_ref_pointed_commit_gets_the_filled_circle() {
    let nodes = vec![commit(1, &[2]), commit(2, &[])];
    let refs = vec![local_branch_at("main", oid(1))];
    assert!(node(NodeKind::Ref)(&parts(&nodes, &refs, 0)[0]));
    assert!(node(NodeKind::Commit)(&parts(&nodes, &refs, 1)[0]));
    assert_eq!(glyphs(&nodes, &refs, 0)[0], "\u{25CF}");
    assert_eq!(glyphs(&nodes, &refs, 1)[0], "\u{25CB}");
}

// ---- The WIP row -------------------------------------------------------
//
// The shape: one row like a commit, a dashed-outline circle on
// HEAD's lane, joined to HEAD by a dashed line. Laid out as the newest
// "commit" of all whose parent is HEAD (see `build_graph_grid_with_wip`).

const DOWN: Sides = Sides::new(false, true, false, false);
const UP: Sides = Sides::new(true, false, false, false);

fn wip_on(head: Option<u8>) -> Option<WipAnchor> {
    Some(WipAnchor {
        head: head.map(oid),
    })
}

/// The usual case: HEAD is the newest commit. The WIP marker sits right
/// above it on its lane and in its color, the connector is dashed on both
/// sides of the row boundary, and **nothing else about the graph moves** -
/// not a part, not a color -- compared with a clean tree.
#[test]
fn with_head_on_top_the_wip_sits_on_heads_lane_and_leaves_the_rest_alone() {
    let nodes = vec![commit(1, &[2, 3]), commit(2, &[]), commit(3, &[])];
    let clean = build_graph_grid(&nodes, &[]);
    let dirty = build_graph_grid_with_wip(&nodes, &[], wip_on(Some(1)));

    let wip = dirty.wip.as_ref().expect("a WIP row was asked for");
    assert_eq!(
        wip.cells[0].part,
        TreePart::Node {
            kind: NodeKind::Wip,
            sides: DOWN,
        }
    );
    assert_eq!(wip.cells[0].dashed, DOWN, "the connector leaves dashed");
    assert_eq!(wip.color, dirty.rows[0].color, "HEAD's own lane color");

    // HEAD takes the connector in on a dashed `up`, and keeps every other
    // stroke -- here the merge fan-out -- solid.
    let head = &dirty.rows[0].cells[0];
    assert!(head.part.sides().up, "HEAD must connect upwards: {head:?}");
    assert_eq!(head.dashed, UP);

    for (i, (c, d)) in clean.rows.iter().zip(&dirty.rows).enumerate() {
        let parts =
            |r: &super::layout::GraphRow| r.cells.iter().map(|c| c.part).collect::<Vec<_>>();
        let colors =
            |r: &super::layout::GraphRow| r.cells.iter().map(|c| c.color).collect::<Vec<_>>();
        if i > 0 {
            assert_eq!(parts(c), parts(d), "row {i} changed shape");
        }
        assert_eq!(colors(c), colors(d), "row {i} changed color");
        assert_eq!(c.lane, d.lane, "row {i} changed lane");
    }
    assert!(
        dirty.rows[1..]
            .iter()
            .all(|r| r.cells.iter().all(|c| c.dashed == Sides::NONE)),
        "nothing below HEAD is pending"
    );
}

/// HEAD is *not* the newest commit - another branch has commits above it.
/// The WIP still sits in the top row, and its dashed lane runs down
/// through the rows between to land on HEAD, without touching the other
/// branch's lane.
#[test]
fn with_head_further_down_the_dashed_connector_runs_to_it_on_its_own_lane() {
    // 5 (feature, parent 6) is newer than HEAD = 1; 6 and 2 are older.
    let nodes = vec![
        commit(5, &[6]),
        commit(1, &[2]),
        commit(6, &[]),
        commit(2, &[]),
    ];
    let grid = build_graph_grid_with_wip(&nodes, &[], wip_on(Some(1)));
    let wip = grid.wip.as_ref().unwrap();
    assert_eq!(wip.lane, 0);

    // Row 0 is commit 5, on a lane of its own; the WIP lane passes it.
    assert_ne!(
        grid.rows[0].lane, 0,
        "commit 5 must not sit on the WIP lane"
    );
    let passing = grid.rows[0].cells[0];
    assert_eq!(passing.part, TreePart::Vertical);
    assert_eq!(passing.dashed, Sides::VERTICAL);
    assert!(
        grid.rows[0].cells[2..]
            .iter()
            .all(|c| c.dashed == Sides::NONE),
        "commit 5's own lane is history: {:?}",
        grid.rows[0].cells
    );

    // Row 1 is HEAD, landing on the WIP lane with a dashed `up`.
    assert_eq!(grid.rows[1].lane, 0);
    assert_eq!(grid.rows[1].cells[0].dashed, UP);
    assert!(grid.rows[2..]
        .iter()
        .all(|r| r.cells.iter().all(|c| c.dashed == Sides::NONE)));
}

/// A branch whose commits sit above HEAD *and* descend from it: that
/// branch's lane converges into HEAD beside the connector. The elbow is
/// history and stays solid; only the WIP lane is dashed.
#[test]
fn a_lane_converging_on_head_beside_the_connector_stays_solid() {
    let nodes = vec![commit(3, &[1]), commit(1, &[2]), commit(2, &[])];
    let grid = build_graph_grid_with_wip(&nodes, &[], wip_on(Some(1)));
    assert_eq!(grid.rows[0].cells[0].dashed, Sides::VERTICAL);
    let head_row = &grid.rows[1];
    assert_eq!(head_row.lane, 0);
    assert_eq!(head_row.cells[0].dashed, UP);
    assert!(
        head_row.cells[1..].iter().all(|c| c.dashed == Sides::NONE),
        "the convergence elbow is history: {:?}",
        head_row.cells
    );
}

/// Nothing to hang from - an unborn branch, or HEAD filtered out by solo
/// or not loaded yet: a lone marker that claims no lane and no color, so
/// the commit rows are exactly those of a clean tree.
#[test]
fn with_no_head_to_join_the_wip_marker_stands_alone_and_reserves_nothing() {
    let nodes = vec![commit(1, &[2]), commit(2, &[])];
    let clean = build_graph_grid(&nodes, &[]);
    for anchor in [wip_on(None), wip_on(Some(9))] {
        let grid = build_graph_grid_with_wip(&nodes, &[], anchor);
        let wip = grid.wip.as_ref().unwrap();
        assert_eq!(
            wip.cells[0].part,
            TreePart::Node {
                kind: NodeKind::Wip,
                sides: Sides::NONE,
            },
            "{anchor:?}"
        );
        for (c, d) in clean.rows.iter().zip(&grid.rows) {
            assert_eq!(c.cells, d.cells, "{anchor:?}");
        }
    }
    // An empty log (a fresh repository with only untracked files).
    let empty = build_graph_grid_with_wip(&[], &[], wip_on(None));
    assert!(empty.rows.is_empty());
    assert!(empty.wip.is_some());
}

/// The text rendering of the same: `◌` for the marker, dimmed, and `┊`
/// for a lane cell that only carries the connector.
#[test]
fn the_text_wip_is_a_dotted_circle_and_its_connector_a_dashed_lane() {
    let nodes = vec![
        commit(5, &[6]),
        commit(1, &[2]),
        commit(6, &[]),
        commit(2, &[]),
    ];
    let lines = build_graph_lines_with(&nodes, &[], wip_on(Some(1)), &[], GraphSettings::default());
    let wip = lines.wip.expect("a WIP line");
    assert_eq!(wip.spans[0].content, "\u{25CC}");
    assert!(wip.spans[0]
        .style
        .add_modifier
        .contains(ratatui::style::Modifier::DIM));
    assert_eq!(lines.rows[0].0.spans[0].content, "\u{250A}");
    // HEAD's own marker is an ordinary, undimmed circle.
    let head = &lines.rows[1].0.spans[0];
    assert_eq!(head.content, "\u{25CB}");
    assert!(!head
        .style
        .add_modifier
        .contains(ratatui::style::Modifier::DIM));
}

/// Lazy loading's invariant survives the WIP row: it is folded first, so
/// appending a page cannot change any row already laid out.
#[test]
fn the_wip_row_keeps_the_layout_prefix_stable() {
    let whole = vec![
        commit(5, &[6]),
        commit(1, &[2]),
        commit(6, &[2]),
        commit(2, &[]),
    ];
    let full = build_graph_grid_with_wip(&whole, &[], wip_on(Some(1)));
    let page = build_graph_grid_with_wip(&whole[..2], &[], wip_on(Some(1)));
    assert_eq!(
        full.wip.as_ref().unwrap().cells,
        page.wip.as_ref().unwrap().cells
    );
    for (i, row) in page.rows.iter().enumerate() {
        let mut expected = full.rows[i].cells.clone();
        expected.truncate(row.cells.len());
        assert_eq!(row.cells[..expected.len()], expected[..], "row {i}");
    }
}

// ---------------------------------------------------------------------------
// How much history is loaded
// ---------------------------------------------------------------------------

/// The Log's initial page has to be big enough that no window can exhaust
/// it, or the pane shows blank space under the last row and scrolling cannot
/// reveal more.
#[test]
fn the_default_commit_count_cannot_regress_to_a_screenful() {
    let s = LogLoadSettings::default();
    assert_eq!(s.initial_commits, 5000);
    assert!(
        s.initial_commits >= LogLoadSettings::MIN_INITIAL_COMMITS,
        "the default must satisfy its own floor"
    );
    // A 200-row terminal is a real thing on a large display; the loaded
    // count has to dwarf it with room for scrolling, not merely match it.
    assert!(s.initial_commits > 200 * 4);
}

#[test]
fn a_configured_commit_count_is_clamped_to_the_floor() {
    assert_eq!(
        LogLoadSettings::with_initial_commits(50).initial_commits,
        LogLoadSettings::MIN_INITIAL_COMMITS
    );
    assert_eq!(
        LogLoadSettings::with_initial_commits(20_000).initial_commits,
        20_000
    );
}

/// Loading thousands of commits has to stay linear: the graph walk runs on
/// every frame the Log is drawn. Not a benchmark but a ceiling, so a
/// quadratic pass fails here rather than in a user's terminal.
#[test]
fn a_full_page_of_history_lays_out_quickly() {
    let nodes: Vec<CommitNode> = (0..LogLoadSettings::DEFAULT_INITIAL_COMMITS)
        .map(|i| CommitNode {
            id: Oid([(i % 251) as u8; 20]),
            parents: Vec::new(),
            summary: String::new(),
            message: String::new(),
            author_name: String::new(),
            author_email: String::new(),
            timestamp: 0,
            refs: Vec::new(),
        })
        .collect();
    let start = std::time::Instant::now();
    let grid = build_graph_grid(&nodes, &[]);
    assert_eq!(grid.rows.len(), nodes.len());
    assert!(
        start.elapsed() < std::time::Duration::from_secs(5),
        "laying out {} commits took {:?}",
        nodes.len(),
        start.elapsed()
    );
}

// ---------------------------------------------------------------------------
// Page boundaries (lazy loading)
// ---------------------------------------------------------------------------

/// History long enough to split, shaped so that a merge, a convergence and
/// a crossing straddle *some* boundary wherever the split is made - a
/// straight chain would pass every test below while proving nothing, since
/// a chain has only one lane to renumber.
///
/// One block is five commits: a merge fanning out, its two sides running
/// in parallel, and their convergence, with the chain continuing into the
/// next block. Ids are the index, so a commit's identity is stable across
/// the two layouts being compared.
fn pageable_history(blocks: u8) -> Vec<CommitNode> {
    let mut nodes = Vec::new();
    for b in 0..blocks {
        let base = b * 5;
        nodes.push(commit(base, &[base + 1, base + 2]));
        nodes.push(commit(base + 2, &[base + 3]));
        nodes.push(commit(base + 1, &[base + 3]));
        nodes.push(commit(base + 3, &[base + 4]));
        // The last block's tail is a root: history has to end somewhere,
        // and that is the state the lazy loader stops asking at.
        if b + 1 == blocks {
            nodes.push(commit(base + 4, &[]));
        } else {
            nodes.push(commit(base + 4, &[base + 5]));
        }
    }
    nodes
}

/// **The invariant lazy loading stands on.** Appending a page of history
/// under the rows already on screen must not move, renumber or recolor a
/// single cell of them: a commit drawn in the blue lane stays in the blue
/// lane when page 2 arrives.
///
/// It holds because `build_graph_grid` is a *fold* (row `i` depends on
/// `nodes[0..=i]` alone), so the layout is prefix-stable by construction.
/// This test stops that being an accident: a two-pass allocator that packs
/// lanes optimally would renumber the whole graph on every page and fails
/// here.
///
/// Checked at **every** split, because a boundary on a quiet row proves
/// nothing; the interesting ones are rows where a lane opens or closes.
///
/// Red if the lane search in `layout.rs` starts from `nodes.len() % 3`
/// instead of 0: the prefix's lanes then depend on later history, and this
/// names the first row that moved.
#[test]
fn appending_a_page_does_not_disturb_the_rows_already_laid_out() {
    let whole = pageable_history(12);
    let full = build_graph_grid(&whole, &[]);

    for boundary in 1..whole.len() {
        let page_one = build_graph_grid(&whole[..boundary], &[]);
        assert_eq!(
            page_one.rows.len(),
            boundary,
            "page 1 must lay out one row per commit it holds"
        );
        for (row, before) in page_one.rows.iter().enumerate() {
            let after = &full.rows[row];
            assert_eq!(
                before.lane, after.lane,
                "boundary {boundary}: row {row} changed lane when the next page was appended"
            );
            assert_eq!(
                before.color, after.color,
                "boundary {boundary}: row {row} changed color when the next page was appended"
            );
            assert_eq!(
                before.cells, after.cells,
                "boundary {boundary}: row {row}'s cells were redrawn when the next page was appended"
            );
        }
    }
}

/// The other half of the same boundary: an edge that ran off the bottom of
/// page 1 has to *land* on something in page 2, and nothing in page 2 may
/// claim an upward connection page 1 did not offer.
///
/// `no_node_is_left_unconnected` exempts the top and bottom rows (the log is
/// a window onto history), which would hide an orphaned edge at a page
/// join, so this checks the joined rows directly.
#[test]
fn no_edge_is_orphaned_across_a_page_boundary() {
    let whole = pageable_history(12);
    let joined = build_graph_grid(&whole, &[]).rectangular();

    for boundary in 1..whole.len() {
        let last_of_page_one = joined[boundary - 1].clone();
        let first_of_page_two = joined[boundary].clone();
        let mut edges_crossing = 0;
        for x in 0..last_of_page_one.len() {
            let above = last_of_page_one[x].part;
            let below = first_of_page_two[x].part;
            if above.sides().down {
                edges_crossing += 1;
                assert!(
                    below.sides().up,
                    "boundary {boundary}: column {x} -- {above:?} runs off the bottom of page 1 \
                     but {below:?} at the top of page 2 does not join it\n{}",
                    render_ascii(&joined)
                );
            }
            assert_eq!(
                below.sides().up,
                above.sides().down,
                "boundary {boundary}: column {x} -- {below:?} and {above:?} disagree about \
                 whether an edge crosses the join\n{}",
                render_ascii(&joined)
            );
        }
        assert!(
            edges_crossing > 0,
            "boundary {boundary} has no edge crossing it at all, so it proves nothing"
        );
    }
}

/// Prefix stability, asserted against the **buffer a terminal is handed**
/// rather than the grid the test built. Every cell of every row page 1
/// drew must still hold the same symbol and foreground color once page 2
/// is appended underneath.
#[test]
fn the_rendered_rows_are_untouched_when_a_page_is_appended_underneath() {
    let whole = pageable_history(8);
    let boundary = 23; // mid-block, so lanes are open across it.

    let render = |nodes: &[CommitNode]| {
        let lines: Vec<_> = build_graph_lines(nodes, &[])
            .rows
            .into_iter()
            .map(|(line, _)| line)
            .collect();
        let height = lines.len() as u16;
        let mut terminal = Terminal::new(TestBackend::new(24, height)).unwrap();
        terminal
            .draw(|f| {
                f.render_widget(Paragraph::new(lines), Rect::new(0, 0, 24, height));
            })
            .unwrap();
        terminal.backend().buffer().clone()
    };

    let before = render(&whole[..boundary]);
    let after = render(&whole);

    for y in 0..boundary as u16 {
        for x in 0..24u16 {
            let b = &before[(x, y)];
            let a = &after[(x, y)];
            assert_eq!(
                (b.symbol(), b.fg),
                (a.symbol(), a.fg),
                "cell ({x},{y}) of the already-drawn page changed when the next page arrived"
            );
        }
    }
}

/// The cost of a page, measured rather than assumed. The graph is re-laid
/// out from the whole node list on every frame, so the question is whether
/// a longer list makes the per-frame walk superlinear. This lays out the
/// default initial page, then the same plus one page, and requires the
/// second to cost no more than three times the first (about
/// `(5000 + 1000) / 5000` expected; a quadratic walk blows through it).
/// Loose on purpose: a shared CI runner is noisy and the failure guarded
/// against is an order of magnitude.
#[test]
fn appending_a_page_keeps_the_layout_linear() {
    let chain = |len: usize| -> Vec<CommitNode> {
        (0..len)
            .map(|i| CommitNode {
                id: Oid([(i % 251) as u8; 20]),
                parents: Vec::new(),
                summary: String::new(),
                message: String::new(),
                author_name: String::new(),
                author_email: String::new(),
                timestamp: 0,
                refs: Vec::new(),
            })
            .collect()
    };
    let initial = chain(LogLoadSettings::DEFAULT_INITIAL_COMMITS);
    let plus_one_page =
        chain(LogLoadSettings::DEFAULT_INITIAL_COMMITS + LogLoadSettings::DEFAULT_PAGE_COMMITS);

    // The best of several runs: a regression (a quadratic walk) raises every
    // run, while a busy machine only adds time to some of them, so the
    // minimum is what the code costs and a single wall-clock sample is what
    // the neighbours' CPU use costs (it failed 48 ms against 13 ms under a
    // full workspace run). The first run also warms the allocator.
    let time = |nodes: &[CommitNode]| {
        (0..7)
            .map(|_| {
                let start = std::time::Instant::now();
                let grid = build_graph_grid(nodes, &[]);
                assert_eq!(grid.rows.len(), nodes.len());
                start.elapsed()
            })
            .min()
            .expect("seven runs")
    };
    let base = time(&initial);
    let grown = time(&plus_one_page);

    assert!(
        grown <= base * 3 + std::time::Duration::from_millis(5),
        "laying out one extra page cost {grown:?} against {base:?} for the initial page \
         - that is not linear growth"
    );
}

/// What one frame costs at the configured **ceiling**, printed rather than
/// only bounded, because a maintainer considering a higher
/// `general.max_commits_in_graph` needs the number.
///
/// The Log re-lays out its whole node list on every frame, so lazy loading
/// makes that walk twenty times longer than the initial page at the default
/// ceiling. Run with `--release -- --ignored --nocapture` for the real
/// figure; a debug build is about an order of magnitude slower.
#[test]
#[ignore = "a measurement, printed for a human; the bound that gates CI is a_full_page_of_history_lays_out_quickly"]
fn what_a_frame_costs_at_each_depth() {
    for depth in [
        LogLoadSettings::DEFAULT_INITIAL_COMMITS,
        20_000,
        LogLoadSettings::DEFAULT_MAX_COMMITS,
    ] {
        let nodes = pageable_history_of(depth);
        let start = std::time::Instant::now();
        let grid = build_graph_grid(&nodes, &[]);
        let elapsed = start.elapsed();
        assert_eq!(grid.rows.len(), nodes.len());
        println!("{depth:>7} commits: {elapsed:?} per frame");
    }
}

/// `pageable_history` by commit count rather than by block, for the
/// measurement above.
fn pageable_history_of(len: usize) -> Vec<CommitNode> {
    let mut nodes = pageable_history(u8::MAX);
    while nodes.len() < len {
        let base = nodes.len();
        for k in 0..5 {
            let id = ((base + k) % 251) as u8;
            nodes.push(commit(id, &[]));
        }
    }
    nodes.truncate(len);
    nodes
}

// ---------------------------------------------------------------------------
// Stashes: a box on a lane of its own beside the
// commit it was made on, at its date, joined to it by a dashed connector.
// Laid out as a pseudo-commit whose only parent is its base (see
// `build_graph_grid_full`); placed by `stash::place_stashes`.
// ---------------------------------------------------------------------------

use super::layout::{build_graph_grid_full, stash_color};
use super::stash::{place_stashes, GraphStash, LogEntry, LogRows, StashSlot};

/// A commit with a date, for the placement tests.
fn dated(id: u8, parents: &[u8], timestamp: i64) -> CommitNode {
    CommitNode {
        timestamp,
        ..commit(id, parents)
    }
}

/// `stash@{n}`, made on `base` at `timestamp`. Its own commit id is
/// `200 + n`, which no test history uses.
fn stash_on(n: u32, base: u8, timestamp: i64) -> GraphStash {
    GraphStash {
        id: n,
        commit: oid(200 + n as u8),
        base: oid(base),
        timestamp,
    }
}

const RIGHT: Sides = Sides::new(false, false, false, true);

fn stash_node(sides: Sides) -> TreePart {
    TreePart::Node {
        kind: NodeKind::Stash,
        sides,
    }
}

/// The commonest case, right after `git stash` on a clean tree: the stash
/// is newer than HEAD, which is the newest commit and a branch tip. The
/// stash row goes on top, its box on the lane **right of** HEAD's, and a
/// dashed connector runs down and curves into HEAD. HEAD keeps the lane
/// and color it has on a tree with no stash - a stash never recolors
/// history.
#[test]
fn a_stash_on_the_tip_sits_right_of_it_and_curves_into_it_dashed() {
    let nodes = vec![dated(1, &[2], 100), dated(2, &[], 90)];
    let stashes = [stash_on(0, 1, 150)];
    let clean = build_graph_grid(&nodes, &[]);
    let grid = build_graph_grid_full(&nodes, &[], None, &stashes);

    assert_eq!(
        grid.stash_slots,
        vec![StashSlot {
            before: 0,
            stash: 0
        }]
    );
    let stash = &grid.stashes[0];
    assert_eq!(stash.lane, 1, "right of HEAD's lane 0");
    assert_eq!(stash.color, stash_color());
    assert_eq!(stash.cells[2].part, stash_node(DOWN));
    assert_eq!(stash.cells[2].dashed, DOWN, "the connector leaves dashed");
    assert_eq!(
        stash.cells[0].part,
        TreePart::Empty,
        "HEAD's lane is only reserved on the stash row: nothing is drawn above a tip"
    );

    let head = &grid.rows[0];
    assert_eq!(head.lane, 0);
    assert_eq!(
        head.cells[0].part,
        TreePart::Node {
            kind: NodeKind::Commit,
            sides: Sides::new(false, true, false, true),
        },
        "a tip (nothing above it) with the stash arriving from the right"
    );
    assert_eq!(
        head.cells[0].dashed, RIGHT,
        "only the stash's arm is pending"
    );
    assert_eq!(head.cells[1].part, TreePart::Horizontal);
    assert_eq!(head.cells[1].dashed, Sides::HORIZONTAL);
    assert_eq!(head.cells[2].part, TreePart::MergeInRight);
    assert_eq!(head.cells[2].dashed, Sides::new(true, false, true, false));

    for (i, (c, s)) in clean.rows.iter().zip(&grid.rows).enumerate() {
        assert_eq!(c.lane, s.lane, "row {i} changed lane");
        assert_eq!(c.color, s.color, "row {i} changed color");
    }
    assert!(
        grid.rows[1].cells.iter().all(|c| c.dashed == Sides::NONE),
        "nothing below the base is pending"
    );
}

/// **Placement is by date**: right above the first commit, from the top
/// down to the base, that is not newer than the stash. Here the stash is
/// older than commit 5 and newer than 4, so it sits between them, and its
/// dashed lane runs down past 4 to its base 3.
#[test]
fn a_stash_is_placed_at_its_date_and_its_lane_runs_down_to_its_base() {
    let nodes = vec![dated(5, &[4], 50), dated(4, &[3], 40), dated(3, &[], 30)];
    let stashes = [stash_on(0, 3, 45)];
    assert_eq!(
        place_stashes(&nodes, &stashes),
        vec![StashSlot {
            before: 1,
            stash: 0
        }]
    );
    let grid = build_graph_grid_full(&nodes, &[], None, &stashes);
    let order = grid.log_rows();
    assert_eq!(
        (0..order.len(nodes.len()))
            .map(|d| order.entry(d))
            .collect::<Vec<_>>(),
        vec![
            LogEntry::Commit(0),
            LogEntry::Stash(0),
            LogEntry::Commit(1),
            LogEntry::Commit(2),
        ]
    );
    // Commit 4's row: the stash's lane passes it, dashed.
    let passing = grid.rows[1].cells[2];
    assert_eq!(passing.part, TreePart::Vertical);
    assert_eq!(passing.dashed, Sides::VERTICAL);
    assert_eq!(passing.color, stash_color());
    // Commit 3's row: the connector lands.
    assert_eq!(grid.rows[2].cells[2].part, TreePart::MergeInRight);
    assert_eq!(grid.rows[0].lane, grid.rows[2].lane, "one branch, one lane");
}

/// Several stashes above one commit keep the stash list's order
/// (`stash@{0}` first, the newest), each on a lane of its own; a stash
/// older than its own base (clock skew) sits right above the base, never
/// below it.
#[test]
fn stashes_on_one_commit_keep_list_order_and_never_go_below_their_base() {
    let nodes = vec![dated(1, &[2], 100), dated(2, &[], 90)];
    let stashes = [stash_on(0, 2, 200), stash_on(1, 2, 190), stash_on(2, 1, 10)];
    let slots = place_stashes(&nodes, &stashes);
    assert_eq!(
        slots,
        vec![
            StashSlot {
                before: 0,
                stash: 0
            },
            StashSlot {
                before: 0,
                stash: 1
            },
            StashSlot {
                before: 0,
                stash: 2
            },
        ],
        "all three above commit 1: two by date, the skewed one capped at its base"
    );
    let grid = build_graph_grid_full(&nodes, &[], None, &stashes);
    let lanes: Vec<usize> = grid.stashes.iter().map(|r| r.lane).collect();
    let mut distinct = lanes.clone();
    distinct.sort_unstable();
    distinct.dedup();
    assert_eq!(distinct.len(), 3, "each stash has its own lane: {lanes:?}");
}

/// **The WIP and a stash on the same commit (HEAD).** The WIP keeps lane 0
/// and HEAD's color, the stash row sits between the WIP row and HEAD
/// with its box on lane 1, and HEAD's row takes both connectors: the
/// WIP's dashed from above, the stash's dashed from the right. Below HEAD
/// the graph is exactly the WIP-only one.
#[test]
fn the_wip_and_a_stash_on_head_coexist() {
    let nodes = vec![dated(1, &[2], 100), dated(2, &[], 90)];
    let stashes = [stash_on(0, 1, 150)];
    let wip_only = build_graph_grid_with_wip(&nodes, &[], wip_on(Some(1)));
    let grid = build_graph_grid_full(&nodes, &[], wip_on(Some(1)), &stashes);

    let order = grid.log_rows();
    assert_eq!(order.entry(0), LogEntry::Wip);
    assert_eq!(order.entry(1), LogEntry::Stash(0));
    assert_eq!(order.entry(2), LogEntry::Commit(0));

    assert_eq!(grid.wip.as_ref().unwrap().lane, 0);
    let stash = &grid.stashes[0];
    assert_eq!(stash.lane, 1);
    assert_eq!(
        stash.cells[0].part,
        TreePart::Vertical,
        "the WIP connector passes"
    );
    assert_eq!(stash.cells[0].dashed, Sides::VERTICAL);

    let head = &grid.rows[0].cells[0];
    assert_eq!(head.dashed, Sides::new(true, false, false, true));
    assert_eq!(grid.rows[0].color, wip_only.rows[0].color);
    assert_eq!(grid.rows[1].cells[..2], wip_only.rows[1].cells[..2]);
}

/// A stash whose base is not loaded (lazy loading has not reached it, or
/// solo filtered it out) is not drawn at all - and appending the page
/// that holds its base cannot move anything already laid out above the
/// stash's slot, nor a stash that was already placed.
#[test]
fn a_stash_waits_for_its_base_and_placed_stashes_stay_put() {
    let whole = vec![
        dated(5, &[4], 50),
        dated(4, &[3], 40),
        dated(3, &[2], 30),
        dated(2, &[], 20),
    ];
    let stashes = [stash_on(0, 4, 45), stash_on(1, 2, 25)];
    let page = build_graph_grid_full(&whole[..2], &[], None, &stashes);
    let full = build_graph_grid_full(&whole, &[], None, &stashes);
    assert_eq!(
        page.stash_slots,
        vec![StashSlot {
            before: 1,
            stash: 0
        }],
        "stash@{{1}}'s base is not loaded yet"
    );
    assert_eq!(full.stash_slots[0], page.stash_slots[0]);
    assert_eq!(full.stash_slots.len(), 2);
    assert_eq!(
        page.stashes[0].cells,
        full.stashes[0].cells[..page.stashes[0].cells.len()]
    );
    for (i, row) in page.rows.iter().enumerate() {
        let n = row.cells.len();
        assert_eq!(row.cells[..], full.rows[i].cells[..n], "row {i}");
    }
}

/// A named history, its stashes and its WIP anchor.
type StashShape = (
    &'static str,
    Vec<CommitNode>,
    Vec<GraphStash>,
    Option<WipAnchor>,
);

/// Every connection the stash shapes claim is mutual, over the rows in
/// the order the Log draws them - the same walk as
/// `no_node_is_left_unconnected`, with the stash rows in their places.
#[test]
fn no_stash_shape_leaves_a_connection_dangling() {
    let shapes: Vec<StashShape> = vec![
        (
            "on the tip",
            vec![dated(1, &[2], 100), dated(2, &[], 90)],
            vec![stash_on(0, 1, 150)],
            None,
        ),
        (
            "at its date, above a later commit",
            vec![dated(5, &[4], 50), dated(4, &[3], 40), dated(3, &[], 30)],
            vec![stash_on(0, 3, 45)],
            None,
        ),
        (
            "with the WIP on the same commit",
            vec![dated(1, &[2], 100), dated(2, &[], 90)],
            vec![stash_on(0, 1, 150)],
            wip_on(Some(1)),
        ),
        (
            "two on one commit, one on another",
            vec![dated(1, &[2], 100), dated(2, &[3], 90), dated(3, &[], 80)],
            vec![stash_on(0, 2, 200), stash_on(1, 2, 95), stash_on(2, 3, 85)],
            None,
        ),
        (
            "on a merge parent that sits on another branch",
            vec![
                dated(9, &[1, 2], 100),
                dated(1, &[3], 90),
                dated(2, &[3], 80),
                dated(3, &[], 70),
            ],
            vec![stash_on(0, 2, 110)],
            None,
        ),
        (
            "on a branch whose tip is below the stash, beside another branch",
            vec![dated(7, &[8], 100), dated(6, &[8], 90), dated(8, &[], 80)],
            vec![stash_on(0, 6, 120)],
            wip_on(Some(7)),
        ),
    ];
    for (what, nodes, stashes, wip) in shapes {
        let grid = build_graph_grid_full(&nodes, &[], wip, &stashes);
        let rows = grid.display_rows();
        let width = rows.iter().map(|r| r.cells.len()).max().unwrap_or(0);
        let rect: Vec<Vec<TreeCell>> = rows
            .iter()
            .map(|r| {
                let mut cells = r.cells.clone();
                cells.resize(width, TreeCell::empty());
                cells
            })
            .collect();
        assert_eq!(
            rows.len(),
            nodes.len() + stashes.len() + usize::from(wip.is_some())
        );
        assert_eq!(
            first_break(&rect),
            None,
            "{what}: a stash shape claims a connection nothing joins.\n{}",
            render_ascii(&rect)
        );
        // Every dashed side is one the part really draws.
        for row in &rect {
            for c in row {
                let (d, s) = (c.dashed, c.part.sides());
                assert!(
                    (!d.up || s.up)
                        && (!d.down || s.down)
                        && (!d.left || s.left)
                        && (!d.right || s.right),
                    "{what}: {c:?} dashes a side it does not draw"
                );
            }
        }
    }
}

/// The text rendering: `□` for the box, undimmed (a stash is recorded
/// work); `┊` for its lane passing a commit, `┄` for its run into the
/// base, both dimmed; the elbow a dimmed solid `╯` (no dashed corner
/// exists in the box-drawing block).
#[test]
fn the_text_stash_is_a_box_and_its_connector_dashed_and_dimmed() {
    use ratatui::style::Modifier;
    let nodes = vec![dated(5, &[4], 50), dated(4, &[3], 40), dated(3, &[], 30)];
    let stashes = [stash_on(0, 3, 45)];
    let lines = build_graph_lines_with(&nodes, &[], None, &stashes, GraphSettings::default());
    let dim = |s: &ratatui::text::Span| s.style.add_modifier.contains(Modifier::DIM);
    let stash = &lines.stashes[0].spans[2];
    assert_eq!(stash.content, "\u{25A1}");
    assert!(!dim(stash), "the box is not pending");
    let passing = &lines.rows[1].0.spans[2];
    assert_eq!(passing.content, "\u{250A}");
    assert!(dim(passing));
    let base = &lines.rows[2].0.spans;
    assert_eq!(base[1].content, "\u{2504}");
    assert!(dim(&base[1]));
    assert_eq!(base[2].content, "\u{256F}");
    assert!(dim(&base[2]));
    assert!(!dim(&base[0]), "the base commit itself is history");
}

/// `LogRows` round-trips: every commit's display index maps back to that
/// commit, every stash row's to that stash row, and the WIP row is 0.
#[test]
fn the_log_row_order_round_trips() {
    for wip in [false, true] {
        let order = LogRows::new(
            wip,
            vec![
                StashSlot {
                    before: 0,
                    stash: 1,
                },
                StashSlot {
                    before: 0,
                    stash: 0,
                },
                StashSlot {
                    before: 2,
                    stash: 2,
                },
            ],
        );
        let commits = 4;
        let len = order.len(commits);
        assert_eq!(len, usize::from(wip) + 3 + commits);
        for i in 0..commits {
            assert_eq!(
                order.entry(order.commit_display_index(i)),
                LogEntry::Commit(i)
            );
        }
        for k in 0..3 {
            let d = order.stash_display_index(k).unwrap();
            assert_eq!(order.entry(d), LogEntry::Stash(k));
        }
        if wip {
            assert_eq!(order.entry(0), LogEntry::Wip);
        }
        let seen: std::collections::HashSet<_> = (0..len).map(|d| order.entry(d)).collect();
        assert_eq!(seen.len(), len, "every display index shows a different row");
    }
}

/// The Log's author avatars sit on exactly the commit rows the table
/// drew: through `LogRows`, with the WIP row and the stash rows taking
/// their screen line and getting no avatar, and the scroll offset
/// applied the same way `visible_raster_rows` applies it.
///
/// Red if `visible_commit_rows` counts only commit rows toward the screen
/// line: every commit after the stash row lands one line too high.
#[test]
fn the_avatar_rows_are_the_commit_rows_the_table_drew() {
    use super::{visible_commit_rows, ColumnWindow, VisibleCommit};
    let order = LogRows::new(
        true,
        vec![StashSlot {
            before: 1,
            stash: 0,
        }],
    );
    // Display: 0 WIP, 1 commit 0, 2 stash, 3 commit 1, 4 commit 2.
    let window = ColumnWindow {
        rows: 10,
        offset: 0,
        selected: None,
    };
    let rows = visible_commit_rows(&order, 3, &window);
    let at = |row, display, commit| VisibleCommit {
        row,
        display,
        commit,
    };
    assert_eq!(rows, vec![at(1, 1, 0), at(3, 3, 1), at(4, 4, 2)]);
    for v in &rows {
        assert_eq!(order.entry(v.display), LogEntry::Commit(v.commit));
    }

    // Scrolled by two: the stash row is the top line now.
    let scrolled = ColumnWindow {
        rows: 2,
        offset: 2,
        selected: None,
    };
    assert_eq!(
        visible_commit_rows(&order, 3, &scrolled),
        vec![at(1, 3, 1)],
        "the window is two rows tall: the stash, then commit 1"
    );
}

/// `ui.graph_row_tint_strength`: strong is the most visible blend, the two
/// other levels are lighter (closer to the pane) but still not the pane.
#[test]
fn the_row_tint_blend_per_level() {
    use gitpoulpi_config::GraphRowTintStrength as S;
    use ratatui::style::Color;
    let lane = Color::Rgb(64, 156, 255);
    let Color::Rgb(sr, sg, sb) = super::part::row_tint_bg(lane, S::Strong) else {
        panic!()
    };
    // The 0.22 mix over PANEL_BG (29, 32, 34).
    assert_eq!((sr, sg, sb), (37, 59, 83));
    let dist = |c: Color| {
        let Color::Rgb(r, g, b) = c else { panic!() };
        i32::from(b) - 34 + i32::from(g) - 32 + i32::from(r) - 29
    };
    let medium = dist(super::part::row_tint_bg(lane, S::Medium));
    let subtle = dist(super::part::row_tint_bg(lane, S::Subtle));
    let strong = dist(super::part::row_tint_bg(lane, S::Strong));
    assert!(
        strong > medium && medium > subtle && subtle >= 10,
        "{strong} {medium} {subtle}"
    );
}

/// The maintainer wanted row tints to stand out more on Fire Light: a light
/// theme's bands take more of the (dark) lane color than a dark theme's.
#[test]
fn a_light_theme_takes_more_of_the_lane_in_a_row_tint() {
    use gitpoulpi_config::GraphRowTintStrength as S;
    let dark = super::part::row_tint_mix(S::Medium);
    let spec = gitpoulpi_config::theme::preset("fire-light").expect("built in");
    let t = crate::theme::Theme::from(&spec);
    let _g = crate::theme::override_theme(&t);
    let light = super::part::row_tint_mix(S::Medium);
    assert!(light > dark * 1.5, "{light} {dark}");
}
