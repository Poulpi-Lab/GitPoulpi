//! The column's Graph part: how wide it is, and the picture drawn over it on
//! a terminal with an image protocol.
//!
//! **Same path as the Log.** The picture is a [`graph::GraphImage`] fed with
//! rows from [`graph::visible_raster_rows`] (the Log's own function) and the
//! avatars from [`graph::attach_node_avatars`], under the same settings
//! (`ui.avatars`, lane background, tint strength). Only the window differs:
//! the column lists commits alone (no WIP row, no stash rows), so the walk is
//! over a [`graph::LogRows`] with neither, and `grid[i]` is the lane row of
//! `nodes[i]` whatever the Log put between the commits. Lanes stay connected
//! because each commit's row is its own grid row, drawn on consecutive
//! screen lines (the Log's WIP and stash rows carry their own lanes and do
//! not alter the commits' rows).

use gitpoulpi_core::domain::CommitNode;

use crate::avatar::AvatarService;
use crate::graph::{self, GraphImage, LaneBackground, LogLayout, RasterRow};

/// The least the title keeps, in cells, whatever the lanes ask for.
pub(super) const MIN_TITLE: usize = 12;
/// The author's share (the first word of the name) and the date's, when the
/// user has not dragged them.
pub(super) const AUTHOR_W: usize = 8;
pub(super) const DATE_W: usize = 5;
/// The Refs part's share when the user has not dragged it.
pub(super) const REFS_W: usize = 14;
/// The Changes part's share (`3f +12 -4`) and the short hash's.
pub(super) const CHANGES_W: usize = 12;
pub(super) const SHA_W: usize = 7;
/// The least a part the user resized keeps.
pub(super) fn min_lanes() -> usize {
    graph::min_width()
}
pub(super) const MIN_AUTHOR: usize = 4;
pub(super) const MIN_DATE: usize = 4;
pub(super) const MIN_REFS: usize = 6;
pub(super) const MIN_MESSAGE: usize = 4;
pub(super) const MIN_CHANGES: usize = 4;
pub(super) const MIN_SHA: usize = 4;
/// The inner width from which each optional part appears.
const SHOW_AUTHOR_AT: usize = 34;
const SHOW_DATE_AT: usize = 28;

/// How an inner width of `width` cells is shared: the cells of each part
/// (`0` = not shown), the order they run in, and how wide the whole row is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Widths {
    /// The Refs part's cells (`0` = not shown); as the Log's Ref column.
    pub(super) refs: usize,
    /// The lanes' cells (the picture's width, or the text lanes').
    pub(super) graph: usize,
    /// The title's cells: what the others leave, never under [`MIN_TITLE`].
    pub(super) title: usize,
    pub(super) author: usize,
    pub(super) date: usize,
    pub(super) changes: usize,
    pub(super) sha: usize,
    /// The parts left to right (the user's order; a part with no cells is
    /// skipped).
    pub(super) order: [super::Part; super::PARTS],
    /// The whole row. Equal to the inner width unless the user's widths
    /// ask for more, which is when the row scrolls sideways.
    pub(super) content: usize,
}

impl Default for Widths {
    fn default() -> Self {
        Widths {
            refs: 0,
            graph: 0,
            title: 0,
            author: 0,
            date: 0,
            changes: 0,
            sha: 0,
            order: super::Part::ORDER,
            content: 0,
        }
    }
}

impl Widths {
    /// The cells a part takes with the cell of air before it.
    fn cost(cells: usize) -> usize {
        if cells > 0 {
            cells + 1
        } else {
            0
        }
    }

    /// The cells of `part` (`0` = not shown).
    pub(super) fn of(&self, part: super::Part) -> usize {
        use super::Part;
        match part {
            Part::Refs => self.refs,
            Part::Graph => self.graph,
            Part::Title => self.title,
            Part::Author => self.author,
            Part::Date => self.date,
            Part::Changes => self.changes,
            Part::Sha => self.sha,
        }
    }

    /// Where each part starts in the row, and its width, in the user's order
    /// (a part that is not shown is left out). One cell of air between two
    /// parts is the separator.
    pub(super) fn spans(&self) -> Vec<(super::Part, usize, usize)> {
        let mut out = Vec::new();
        let mut x = 0usize;
        for part in self.order {
            let w = self.of(part);
            if w > 0 {
                out.push((part, x, w));
                x += w + 1;
            }
        }
        out
    }
}

/// The rule: **the lanes take every cell they need**, a cut graph being a
/// graph with missing links. The title is guaranteed [`MIN_TITLE`] cells;
/// what gives way for the lanes is, in this order, the author, the date, and
/// only then the lanes themselves (cut at the right, as the Log cuts a graph
/// wider than its column). `needed` is the widest row of the window.
#[cfg(test)]
pub(super) fn plan_widths(width: usize, needed: usize) -> Widths {
    plan_shown(width, needed, super::DEFAULT_SHOWN, super::Part::ORDER)
}

/// [`plan_widths`] for the parts the user chose to show (`shown` by slot,
/// `Part::slot`) in `order`. The parts that are an explicit option (Refs,
/// Changes, SHA) are never given up for room: they take their cells first
/// ([`REFS_W`], [`CHANGES_W`], [`SHA_W`]) and the rest is shared as above;
/// when the row is then wider than the column it scrolls sideways.
pub(super) fn plan_shown(
    width: usize,
    needed: usize,
    shown: [bool; super::PARTS],
    order: [super::Part; super::PARTS],
) -> Widths {
    let refs = if shown[3] { REFS_W } else { 0 };
    let changes = if shown[5] { CHANGES_W } else { 0 };
    let sha = if shown[6] { SHA_W } else { 0 };
    let reserved = Widths::cost(refs) + Widths::cost(changes) + Widths::cost(sha);
    let rest = if reserved > 0 {
        width.saturating_sub(reserved).max(MIN_TITLE)
    } else {
        width
    };
    let needed = if shown[0] { needed } else { 0 };
    let mut plan = plan_rest(rest, needed, shown[1], shown[2]);
    plan.refs = refs;
    plan.changes = changes;
    plan.sha = sha;
    plan.order = order;
    plan.content += reserved;
    plan
}

fn plan_rest(width: usize, needed: usize, author_ok: bool, date_ok: bool) -> Widths {
    let wants = [
        (
            author_ok && width >= SHOW_AUTHOR_AT,
            date_ok && width >= SHOW_DATE_AT,
        ),
        (false, date_ok && width >= SHOW_DATE_AT),
        (false, false),
    ];
    let shared = |graph: usize, author: bool, date: bool| {
        let author = if author { AUTHOR_W } else { 0 };
        let date = if date { DATE_W } else { 0 };
        let used = Widths::cost(graph) + Widths::cost(author) + Widths::cost(date);
        Widths {
            graph,
            title: width.saturating_sub(used).max(1),
            author,
            date,
            content: width,
            ..Widths::default()
        }
    };
    for (author, date) in wants {
        let tail = (if date { DATE_W + 1 } else { 0 }) + (if author { AUTHOR_W + 1 } else { 0 });
        if Widths::cost(needed) + MIN_TITLE + tail <= width {
            return shared(needed, author, date);
        }
    }
    // Nothing fits whole: no author, no date, the lanes cut.
    let room = width.saturating_sub(MIN_TITLE + 1);
    shared(needed.min(room), false, false)
}

/// [`plan_shown`] when the user dragged a part (`custom` by slot, see
/// `Part::slot`: lanes, author, date, refs, message, changes, sha; `0` = not
/// dragged). **Their widths are kept, not given up**: a part is never hidden
/// to make room, the message keeps what is left (at least [`MIN_TITLE`])
/// unless it has a width of its own, and when the row does not fit the column
/// it is wider than it and scrolls sideways. Nothing dragged: [`plan_shown`]'s
/// rule.
#[cfg(test)]
pub(super) fn plan_widths_with(width: usize, needed: usize, custom: [u16; super::PARTS]) -> Widths {
    plan_custom(
        width,
        needed,
        custom,
        super::DEFAULT_SHOWN,
        super::Part::ORDER,
    )
}

pub(super) fn plan_custom(
    width: usize,
    needed: usize,
    custom: [u16; super::PARTS],
    shown: [bool; super::PARTS],
    order: [super::Part; super::PARTS],
) -> Widths {
    if custom == [0; super::PARTS] {
        return plan_shown(width, needed, shown, order);
    }
    let pick = |wanted: u16, auto: usize, min: usize| {
        if wanted > 0 {
            usize::from(wanted).max(min)
        } else {
            auto
        }
    };
    let part = |slot: usize, auto: usize, min: usize| {
        if shown[slot] {
            pick(custom[slot], auto, min)
        } else {
            0
        }
    };
    let refs = part(3, REFS_W, MIN_REFS);
    let graph = if !shown[0] || (needed == 0 && custom[0] == 0) {
        0
    } else {
        pick(custom[0], needed, min_lanes())
    };
    let author = part(1, AUTHOR_W, MIN_AUTHOR);
    let date = part(2, DATE_W, MIN_DATE);
    let changes = part(5, CHANGES_W, MIN_CHANGES);
    let sha = part(6, SHA_W, MIN_SHA);
    let fixed = [refs, graph, author, date, changes, sha]
        .into_iter()
        .map(Widths::cost)
        .sum::<usize>();
    let title = if custom[4] > 0 {
        usize::from(custom[4]).max(MIN_MESSAGE)
    } else {
        width.saturating_sub(fixed).max(MIN_TITLE)
    };
    Widths {
        refs,
        graph,
        title,
        author,
        date,
        changes,
        sha,
        order,
        content: fixed + title,
    }
}

/// The widest lane row among the `shown` commits from `offset`.
pub(super) fn lanes_needed(layout: &LogLayout, offset: usize, shown: usize) -> usize {
    let widest = layout
        .grid
        .iter()
        .skip(offset)
        .take(shown)
        .map(|r| r.cells.len())
        .max()
        .unwrap_or(0);
    // The automatic width is never the compact one (`graph::COMPACT_W`):
    // that is the user's to pick, by sizing the part down to its least.
    if widest > 0 {
        widest.max(graph::min_width() + 1)
    } else {
        0
    }
}

/// The raster rows of the `rows` commits shown from `offset`, `selected`
/// being the index (into the commits) of the selected one. Through
/// [`graph::visible_raster_rows`] over a commits-only order: for the same
/// commits the cells are the Log's.
pub(crate) fn raster_rows(
    layout: &LogLayout,
    offset: usize,
    rows: u16,
    selected: Option<usize>,
) -> Vec<RasterRow> {
    let window = graph::ColumnWindow {
        rows,
        offset,
        selected,
    };
    graph::visible_raster_rows(
        &layout.grid,
        None,
        &[],
        &graph::LogRows::new(false, Vec::new()),
        &window,
    )
}

/// What the picture needs besides the rows: the image owned by the window
/// (a second one beside the Log's, which is never on screen together with
/// the column), and the settings the Log's picture reads.
pub(crate) struct ColumnPicture<'a> {
    pub(crate) image: &'a mut GraphImage,
    /// `ui.avatars = "graph"`: author pictures inside the circles.
    pub(crate) avatars: bool,
    pub(crate) service: &'a mut AvatarService,
    pub(crate) lane_background: LaneBackground,
    pub(crate) tint_strength: gitpoulpi_config::GraphRowTintStrength,
}

impl ColumnPicture<'_> {
    /// Whether this terminal draws the Graph as pixels.
    pub(crate) fn draws(&self) -> bool {
        self.image.capability().draws_graph()
    }

    /// The rows for the window, with the avatars attached.
    pub(super) fn rows_for(
        &mut self,
        layout: &LogLayout,
        nodes: &[CommitNode],
        offset: usize,
        rows: u16,
        selected: Option<usize>,
    ) -> Vec<RasterRow> {
        self.image.set_avatars(self.avatars);
        self.image.set_lane_background(self.lane_background);
        self.image.set_tint_strength(self.tint_strength);
        let mut raster = raster_rows(layout, offset, rows, selected);
        if self.avatars {
            let window = graph::ColumnWindow {
                rows,
                offset,
                selected,
            };
            let order = graph::LogRows::new(false, Vec::new());
            let visible = graph::visible_commit_rows(&order, nodes.len(), &window);
            graph::attach_node_avatars(&mut raster, nodes, &visible, self.service);
        }
        raster
    }
}
