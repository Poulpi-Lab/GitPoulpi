//! Graph tests: the terminal's reading of the layout.
//!
//! The lane walk, the WIP and stash rows, the Log's row arithmetic and the
//! page-boundary stability are tested where they live, in
//! `gitpoulpi-app` (`crates/gitpoulpi-app/src/graph/tests.rs`). What is left
//! here is what only the terminal can answer:
//!
//! - **The glyph contract**: one cell, one code point, per part.
//! - **Rendering**, against a real `TestBackend` buffer -- what the
//!   terminal is actually handed. `where-we-are.md` warns that a test
//!   asserting on the strings you just generated is not evidence, so every
//!   claim about connection is checked against the buffer as well as the
//!   grid.
//! - **Colours**, resolved against the theme.
//!
//! Each test was checked red by reverting the corresponding line of
//! `layout.rs` or `part.rs`.

use super::layout::{build_graph_grid, build_graph_lines, build_graph_lines_with, WipAnchor};
use super::part::{NodeKind, NodeSet, TreeCell, TreePart, TreePartGlyph};
use super::{GraphSettings, GraphStash, LogLoadSettings};
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
        multi_remote: None,
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

fn node(kind: NodeKind) -> impl Fn(&TreePart) -> bool {
    move |p| matches!(p, TreePart::Node { kind: k, .. } if *k == kind)
}

fn wip_on(head: Option<u8>) -> Option<WipAnchor> {
    Some(WipAnchor {
        head: head.map(oid),
    })
}

/// History long enough to split, with a merge, a convergence and a
/// crossing in every block (the same shape the layout's own page-boundary
/// tests use in `gitpoulpi-app`).
fn pageable_history(blocks: u8) -> Vec<CommitNode> {
    let mut nodes = Vec::new();
    for b in 0..blocks {
        let base = b * 5;
        nodes.push(commit(base, &[base + 1, base + 2]));
        nodes.push(commit(base + 2, &[base + 3]));
        nodes.push(commit(base + 1, &[base + 3]));
        nodes.push(commit(base + 3, &[base + 4]));
        if b + 1 == blocks {
            nodes.push(commit(base + 4, &[]));
        } else {
            nodes.push(commit(base + 4, &[base + 5]));
        }
    }
    nodes
}

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

/// A commit with a date, for the placement tests.
fn dated(id: u8, parents: &[u8], timestamp: i64) -> CommitNode {
    CommitNode {
        timestamp,
        ..commit(id, parents)
    }
}

/// `stash@{n}`, made on `base` at `timestamp`.
fn stash_on(n: u32, base: u8, timestamp: i64) -> GraphStash {
    GraphStash {
        id: n,
        commit: oid(200 + n as u8),
        base: oid(base),
        timestamp,
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

// ---------------------------------------------------------------------------
// How much history is loaded
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Page boundaries (lazy loading)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Stashes: a box on a lane of its own beside the
// commit it was made on, at its date, joined to it by a dashed connector.
// Laid out as a pseudo-commit whose only parent is its base (see
// `build_graph_grid_full`); placed by `stash::place_stashes`.
// ---------------------------------------------------------------------------

use super::{LogEntry, LogRows, StashSlot};

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
