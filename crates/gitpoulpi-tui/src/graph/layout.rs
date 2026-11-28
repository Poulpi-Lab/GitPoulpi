//! The terminal's reading of the commit-graph layout: the lane walk itself
//! is `gitpoulpi_app::graph::layout`, frontend-neutral and colour-free; this
//! file resolves its [`ColorRef`]s against the lane palette and the theme,
//! and turns rows into `Line`s.

use ratatui::style::{Color, Style};
use ratatui::text::Line;

use gitpoulpi_app::graph::layout as neutral;
use gitpoulpi_core::domain::{CommitNode, Ref};

pub(crate) use gitpoulpi_app::graph::WipAnchor;

use super::part::{ColorRef, TreeCell};
use super::GraphSettings;
use super::{GraphStash, LogRows, StashSlot};

/// One rendered commit row, before it becomes a `Line`.
#[derive(Debug, Clone)]
pub(crate) struct GraphRow {
    /// Column `2*j` is lane `j`'s glyph, `2*j+1` the connector to its
    /// right. Rows can differ in length - the lane vector only ever grows -
    /// so consumers that need a rectangle pad on the right with
    /// [`TreeCell::empty`].
    pub cells: Vec<TreeCell>,
    /// The lane this row's commit landed in. Kept for the row builders of
    /// the renderer's tests; no terminal code reads it.
    #[allow(dead_code)]
    pub lane: usize,
    /// That lane's color, used downstream to tint the commit's ref pills.
    pub color: Color,
}

/// The whole Graph column as parts. Tests assert on this; the renderer
/// turns it into `Line`s.
#[derive(Debug, Clone, Default)]
pub(crate) struct GraphGrid {
    /// One per commit, in the order of the `nodes` handed in - the WIP
    /// row is **not** among them, so `rows[i]` is always `nodes[i]`.
    pub rows: Vec<GraphRow>,
    /// The WIP row, laid out above `rows[0]` when one was asked for. Kept
    /// apart from `rows` so a commit's index in the grid stays its index
    /// in `nodes`; the Log's display index is this plus one.
    pub wip: Option<GraphRow>,
    /// One per placed stash, in display order: `stashes[k]` is drawn where
    /// `stash_slots[k]` says. Kept apart from `rows` for the same reason
    /// the WIP row is.
    pub stashes: Vec<GraphRow>,
    pub stash_slots: Vec<StashSlot>,
}

/// A stash's box and its connector. One neutral color for every stash
/// (`th().graph.stash`), rather than a palette color: a stash is not a
/// branch, and taking a palette slot would recolor every branch tip below it.
pub(crate) fn stash_color() -> Color {
    crate::th().graph.stash
}

/// The WIP marker's color when it has no commit to hang from, and so no
/// lane color to inherit: `th().graph.wip`.
fn wip_color() -> Color {
    crate::th().graph.wip
}

impl GraphGrid {
    /// The Log's rows in display order - WIP, then commits with the stash
    /// rows between them - as [`LogRows`] orders them.
    pub(crate) fn log_rows(&self) -> LogRows {
        LogRows::new(self.wip.is_some(), self.stash_slots.clone())
    }

    /// The grid padded to a rectangle, for anything that walks neighbors.
    #[cfg(test)]
    pub(crate) fn rectangular(&self) -> Vec<Vec<TreeCell>> {
        let width = self.rows.iter().map(|r| r.cells.len()).max().unwrap_or(0);
        self.rows
            .iter()
            .map(|r| {
                let mut row = r.cells.clone();
                row.resize(width, TreeCell::empty());
                row
            })
            .collect()
    }

    /// The neutral grid with every [`ColorRef`] resolved against `palette`
    /// and the theme's WIP and stash colors.
    fn resolve(grid: neutral::GraphGrid, palette: &[Color]) -> GraphGrid {
        let color_of = |c: ColorRef| match c {
            ColorRef::None => Color::Reset,
            ColorRef::Lane(n) => palette[n],
            ColorRef::Wip => wip_color(),
            ColorRef::Stash => stash_color(),
        };
        let row_of = |r: neutral::GraphRow| GraphRow {
            cells: r
                .cells
                .into_iter()
                .map(|c| TreeCell::resolve(c, color_of))
                .collect(),
            lane: r.lane,
            color: color_of(r.color),
        };
        GraphGrid {
            rows: grid.rows.into_iter().map(row_of).collect(),
            wip: grid.wip.map(row_of),
            stashes: grid.stashes.into_iter().map(row_of).collect(),
            stash_slots: grid.stash_slots,
        }
    }
}

/// What [`build_graph_lines`] hands back: one rendered row per commit (plus
/// that commit's lane color), and the WIP row's own Graph cell when one
/// was asked for.
pub(crate) struct GraphLines {
    pub rows: Vec<(Line<'static>, Color)>,
    /// The WIP row's Graph cell: the dotted marker on its own lane. The
    /// connector down to HEAD is in `rows`, on the cells it passes
    /// through, because that is where the table draws them.
    pub wip: Option<Line<'static>>,
    /// Each placed stash row's Graph cell, in display order
    /// (`grid.stash_slots` says where each goes).
    pub stashes: Vec<Line<'static>>,
    /// The parts behind `rows`, kept so the image renderer rasterises exactly
    /// what the text renderer would have drawn. Handed back rather than
    /// recomputed: the lane walk is the only place lane assignment happens.
    pub grid: GraphGrid,
}

/// The renderer's entry point. Uses [`GraphSettings::default`], i.e. the
/// text rendering; the Log passes a real one through when the terminal
/// can show images.
#[cfg(test)]
pub(crate) fn build_graph_lines(nodes: &[CommitNode], refs: &[Ref]) -> GraphLines {
    build_graph_lines_with(nodes, refs, None, &[], GraphSettings::default())
}

pub(crate) fn build_graph_lines_with(
    nodes: &[CommitNode],
    refs: &[Ref],
    wip: Option<WipAnchor>,
    stashes: &[GraphStash],
    settings: GraphSettings,
) -> GraphLines {
    let grid = build_graph_grid_themed(
        nodes,
        refs,
        wip,
        stashes,
        super::lane_palette(settings.theme),
    );
    let line_of = |row: &GraphRow| -> Line<'static> {
        // `row_tint`: from the commit's node to the column's right edge,
        // in a faint version of its lane color (the message, author and
        // date columns stay untinted). In the image renderer the picture
        // carries it instead (`raster`).
        let tint = (settings.lane_background == super::LaneBackground::RowTint)
            .then(|| {
                let node = super::connector::node_column(&row.cells)?;
                Some((
                    node,
                    super::part::row_tint_bg(row.cells[node].color, settings.tint_strength),
                ))
            })
            .flatten();
        let spans: Vec<_> = row
            .cells
            .iter()
            .enumerate()
            .map(|(i, c)| match settings.render {
                // Blanks, not an empty line: the Graph cell has to
                // occupy the same columns either way, or the image
                // would sit over a column the table has silently
                // re-flowed.
                super::GraphRender::Image => ratatui::text::Span::raw(" "),
                super::GraphRender::Text => {
                    let span = c.span(settings.node_set);
                    match tint {
                        Some((node, bg)) if i >= node => span.patch_style(Style::default().bg(bg)),
                        _ => span,
                    }
                }
            })
            .collect();
        Line::from(spans)
    };
    let rows = grid
        .rows
        .iter()
        .map(|row| (line_of(row), row.color))
        .collect();
    let wip = grid.wip.as_ref().map(line_of);
    let stashes = grid.stashes.iter().map(line_of).collect();
    GraphLines {
        rows,
        wip,
        stashes,
        grid,
    }
}

/// One commit row's Graph cell as characters, whatever the Log renders: the
/// Log's own cells are blanks under the image renderer, so a compact commit
/// list (`commit_column`) draws its lanes from the grid with this, through
/// the same `TreeCell::span` the Log's text mode uses.
pub(crate) fn text_line(row: &GraphRow, set: super::NodeSet) -> Line<'static> {
    Line::from(
        row.cells
            .iter()
            .map(|c| c.span(set))
            .collect::<Vec<ratatui::text::Span<'static>>>(),
    )
}

/// The graph of `nodes` alone, with no WIP row.
#[cfg(test)]
pub(crate) fn build_graph_grid(nodes: &[CommitNode], refs: &[Ref]) -> GraphGrid {
    build_graph_grid_with_wip(nodes, refs, None)
}

/// The graph, with a WIP row on top when `wip` asks for one, and no
/// stashes.
#[cfg(test)]
pub(crate) fn build_graph_grid_with_wip(
    nodes: &[CommitNode],
    refs: &[Ref],
    wip: Option<WipAnchor>,
) -> GraphGrid {
    build_graph_grid_full(nodes, refs, wip, &[])
}

/// The graph, with a WIP row on top when `wip` asks for one and a row per
/// stash whose base is among `nodes`. See
/// [`neutral::build_graph_grid_themed`] for how the WIP and stash rows are laid out.
#[cfg(test)]
pub(crate) fn build_graph_grid_full(
    nodes: &[CommitNode],
    refs: &[Ref],
    wip: Option<WipAnchor>,
    stashes: &[GraphStash],
) -> GraphGrid {
    build_graph_grid_themed(nodes, refs, wip, stashes, &super::LANE_PALETTE[..])
}

/// [`neutral::build_graph_grid_themed`] with the lane colors taken from
/// `palette` (`ui.graph_color_theme`) instead of the GitPoulpi default.
pub(crate) fn build_graph_grid_themed(
    nodes: &[CommitNode],
    refs: &[Ref],
    wip: Option<WipAnchor>,
    stashes: &[GraphStash],
    palette: &[Color],
) -> GraphGrid {
    let grid = neutral::build_graph_grid_themed(nodes, refs, wip, stashes, palette.len());
    GraphGrid::resolve(grid, palette)
}
