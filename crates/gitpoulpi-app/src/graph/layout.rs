//! Lane assignment and row layout: history in, a grid of [`TreePart`]s out.
//!
//! The algorithm is the standard `git log --graph` lane tracker, simplified:
//! walking the commits in the order the backend gives them
//! (parents-after-children), a commit occupies the lane that was expecting
//! it -- or a fresh one if it is a branch tip -- continues that lane onto
//! its first parent, and any additional parents (a merge) claim a free or
//! new lane.
//!
//! Two invariants the layout keeps:
//!
//! - **Lane transitions render inside the transitioning commit's own row.**
//!   There is one row per commit and never a separate connector row. A
//!   merge's extra parents fan *out* from the merge node across to their
//!   new lanes on the merge's row; a lane about to collapse *into* another
//!   curves in on the row of the commit they meet at.
//! - **Convergence keys on `commit.id`, never on the first parent.** Two
//!   lanes can legitimately share a *future* target for many rows without
//!   having converged; keying on that target fires a convergence early,
//!   draws an elbow where nothing converges, and kills the other lane long
//!   before its own commit's row.
//!
//! Column arithmetic: lane `j` owns glyph column `2*j`, and column `2*j+1`
//! is the connector cell between lane `j` and lane `j+1`. A horizontal run
//! between lanes `lo` and `hi` therefore spans **two families of cells**:
//! the connectors `2*k+1` for `k in lo..hi`, *and* the glyph columns `2*k`
//! for `k in (lo+1)..hi`. Filling only one leaves the elbow floating a
//! blank cell away from its node.

use std::collections::{HashMap, HashSet};

use gitpoulpi_core::domain::{CommitNode, Oid, Ref};

use super::part::{ColorRef, NodeKind, Sides, TreeCell, TreePart};
use super::stash::{place_stashes, GraphStash, LogRows, StashSlot};

/// One laid-out commit row: its cells and the lane colour it carries.
#[derive(Debug, Clone)]
pub struct GraphRow {
    /// Column `2*j` is lane `j`'s glyph, `2*j+1` the connector to its
    /// right. Rows can differ in length - the lane vector only ever grows -
    /// so consumers that need a rectangle pad on the right with
    /// [`TreeCell::empty`].
    pub cells: Vec<TreeCell>,
    /// The lane this row's commit landed in.
    pub lane: usize,
    /// That lane's colour, used downstream to tint the commit's ref pills.
    pub color: ColorRef,
}

/// The whole Graph column as parts. A frontend paints it: the terminal UI
/// turns it into text or a bitmap, the desktop UI into vector strokes.
#[derive(Debug, Clone, Default)]
pub struct GraphGrid {
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

/// Where the WIP row hangs from: `head` is the commit HEAD points at, or
/// `None` when there is none to join (an unborn branch, or HEAD not among
/// the loaded, solo-filtered commits).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WipAnchor {
    pub head: Option<Oid>,
}

impl GraphGrid {
    /// The Log's rows in display order - WIP, then commits with the stash
    /// rows between them - as [`LogRows`] orders them.
    pub fn log_rows(&self) -> LogRows {
        LogRows::new(self.wip.is_some(), self.stash_slots.clone())
    }

    /// Every row in display order, for anything that walks the column as
    /// the Log shows it.
    pub fn display_rows(&self) -> Vec<&GraphRow> {
        let order = self.log_rows();
        let len = order.len(self.rows.len());
        (0..len)
            .filter_map(|d| match order.entry(d) {
                super::stash::LogEntry::Wip => self.wip.as_ref(),
                super::stash::LogEntry::Commit(i) => self.rows.get(i),
                super::stash::LogEntry::Stash(k) => self.stashes.get(k),
            })
            .collect()
    }

    /// The grid padded to a rectangle, for anything that walks neighbors.
    pub fn rectangular(&self) -> Vec<Vec<TreeCell>> {
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
}

/// The graph, with a WIP row on top when `wip` asks for one and a row per
/// stash whose base is among `nodes`.
///
/// **The WIP row is laid out as the newest "commit" of all**, before
/// `nodes[0]`, whose only parent is HEAD. The lane walk then does for it what
/// it does for any branch tip (the WIP takes lane 0, the lane runs down to
/// HEAD, HEAD continues it), so the connector is joined by the same code as
/// every other lane and can never overlap a lane it did not reserve. The
/// only difference is ink: that lane's cells are [`TreeCell::dashed`] until
/// HEAD's row, HEAD's `up` included.
///
/// When HEAD is `nodes[0]` (the usual case) every commit row is laid out
/// identically to the no-WIP graph, colors included: the WIP takes lane 0
/// and palette color 0, and HEAD inherits both. Lazy loading's prefix
/// stability holds too: the WIP is folded first, so appending a page cannot
/// change an earlier row.
///
/// **A stash is the same trick, folded at its place among the commits**
/// ([`place_stashes`]): a pseudo-commit whose only parent is its base.
/// Two differences, both so the stash sits *beside* its base the way
/// it is drawn rather than on top of the base's own lane:
///
/// - its lane is never the one the base lands in. The base keeps (or
///   takes) its own lane, and the stash's lane *converges* into it on the
///   base's row with the usual elbow. That is `stash_lanes`: a lane there
///   is never "the lane expecting this commit".
/// - it takes the first free lane to the **right** of the base's lane.
///   When no lane expects the base yet (the base is a branch tip below the
///   stash), a lane is *reserved* for it first, drawn as nothing until the
///   base arrives as an ordinary tip, so the stash has a lane to sit
///   right of.
///
/// The stash's box, its lane cells, the elbow and the run into the base
/// (and the base's arm on that side) are dashed wherever no history
/// shares them. A stash takes no palette color (`ColorRef::Stash`), so it
/// never recolors a branch.
///
/// The lane colours are neutral [`ColorRef`]s: `Lane(n)` is slot `n` of a
/// palette of `palette_len` colours (`ui.graph_color_theme`), which the
/// frontend owns. The fold never looks at a real colour.
pub fn build_graph_grid_themed(
    nodes: &[CommitNode],
    refs: &[Ref],
    wip: Option<WipAnchor>,
    stashes: &[GraphStash],
    palette_len: usize,
) -> GraphGrid {
    let mut lanes: Vec<Option<Oid>> = Vec::new();
    let mut lane_colors: Vec<ColorRef> = Vec::new();
    let mut next_color = 0usize;
    let mut rows: Vec<GraphRow> = Vec::with_capacity(nodes.len());
    // Lanes carrying a stash's connector down to its base.
    let mut stash_lanes: HashSet<usize> = HashSet::new();
    // Lanes held for a base no lane expects yet; drawn as nothing until
    // that commit arrives.
    let mut reserved: HashSet<usize> = HashSet::new();

    // HEAD, only when it is actually among the rows: a connector aimed at
    // a commit that is filtered out (solo) or not loaded yet would run to
    // the bottom of the column and never land.
    let wip_head: Option<Oid> = wip
        .and_then(|w| w.head)
        .filter(|head| nodes.iter().any(|n| n.id == *head));
    // The lane carrying the WIP connector, while it is still on its way
    // down to HEAD.
    let mut wip_lane: Option<usize> = None;
    let wip: Option<GraphRow> = wip.map(|_| match wip_head {
        Some(head) => {
            let color = ColorRef::Lane(0);
            lanes.push(Some(head));
            lane_colors.push(color);
            next_color = 1;
            wip_lane = Some(0);
            let down = Sides::new(false, true, false, false);
            let node = TreePart::Node {
                kind: NodeKind::Wip,
                sides: down,
            };
            GraphRow {
                cells: vec![
                    TreeCell::new(node, color).with_dashed(down),
                    TreeCell::empty(),
                ],
                lane: 0,
                color,
            }
        }
        // Nothing to hang from: a lone marker, which reserves no lane and
        // takes no palette color, so the commit rows below are laid out
        // exactly as if it were not there.
        None => GraphRow {
            cells: vec![
                TreeCell::new(
                    TreePart::Node {
                        kind: NodeKind::Wip,
                        sides: Sides::NONE,
                    },
                    ColorRef::Wip,
                ),
                TreeCell::empty(),
            ],
            lane: 0,
            color: ColorRef::Wip,
        },
    });

    let stash_slots = place_stashes(nodes, stashes);
    let mut stash_rows: Vec<GraphRow> = Vec::with_capacity(stash_slots.len());
    let mut next_slot = 0usize;
    // Where each commit is, for a stash's look-ahead down first-parent
    // chains (`stash_row`). Only built when there is a stash to place.
    let index_of: HashMap<Oid, usize> = if stash_slots.is_empty() {
        HashMap::new()
    } else {
        nodes.iter().enumerate().map(|(i, n)| (n.id, i)).collect()
    };

    for (index, commit) in nodes.iter().enumerate() {
        // The stash rows placed right above this commit, folded first.
        while let Some(slot) = stash_slots.get(next_slot).filter(|s| s.before == index) {
            next_slot += 1;
            stash_rows.push(stash_row(
                stashes[slot.stash].base,
                nodes,
                &index_of,
                &mut lanes,
                &mut lane_colors,
                &mut stash_lanes,
                &mut reserved,
                wip_lane,
            ));
        }

        // Was a lane already waiting for this commit? That is also exactly
        // the question "does this node's lane continue upwards" -- a fresh
        // branch tip has nothing above it, and a node that claimed `up`
        // with nothing above would fail the connectivity invariant. A
        // stash's lane is never that lane (it converges instead), and a
        // lane merely *reserved* for this commit has nothing above it.
        let expecting =
            (0..lanes.len()).find(|&j| lanes[j] == Some(commit.id) && !stash_lanes.contains(&j));
        let lane_continues_up = expecting.is_some_and(|j| !reserved.remove(&j));
        let my_lane = match expecting {
            Some(i) if lane_continues_up => i,
            found => {
                // A genuinely new branch tip always gets a fresh palette color, even
                // when the lane index it reuses still carries the color of whichever
                // unrelated branch last occupied it. A lane reserved for this tip by a
                // stash above it is taken as it is.
                let i = match found {
                    Some(i) => i,
                    None => free_lane(&mut lanes),
                };
                if lane_colors.len() <= i {
                    lane_colors.resize(i + 1, ColorRef::Lane(0));
                }
                lane_colors[i] = ColorRef::Lane(next_color % palette_len);
                next_color += 1;
                i
            }
        };
        // A freshly allocated tip's slot is still literally `None` here --
        // it is not written until the end of the iteration -- so reserve it
        // now with a placeholder, or this same commit's extra merge parents
        // (right below) can mistake it for free and steal it.
        if lanes[my_lane].is_none() {
            lanes[my_lane] = Some(commit.id);
        }

        // Extra (merge) parents claim their lane *before* this row is
        // rendered, so the row itself can show the fan-out instead of the
        // lane silently appearing on the next row with no connector. A
        // lane a stash reserved for that parent is the one it claims.
        let mut new_parent_lanes: Vec<usize> = Vec::new();
        for &parent in commit.parents.iter().skip(1) {
            let live = (0..lanes.len()).any(|j| {
                lanes[j] == Some(parent) && !stash_lanes.contains(&j) && !reserved.contains(&j)
            });
            if live {
                continue;
            }
            let held =
                (0..lanes.len()).find(|&j| lanes[j] == Some(parent) && reserved.contains(&j));
            let lane_idx = match held {
                Some(j) => {
                    reserved.remove(&j);
                    j
                }
                None => {
                    let j = free_lane(&mut lanes);
                    lanes[j] = Some(parent);
                    j
                }
            };
            new_parent_lanes.push(lane_idx);
            if lane_colors.len() <= lane_idx {
                lane_colors.resize(lane_idx + 1, ColorRef::Lane(0));
            }
            lane_colors[lane_idx] = ColorRef::Lane(next_color % palette_len);
            next_color += 1;
        }
        let is_merge = !new_parent_lanes.is_empty();
        let has_ref = refs.iter().any(|r| r.target == commit.id);
        let my_color = lane_colors[my_lane];

        // Fan-out elbow + fill, mirrored for a new parent lane landing on either
        // side of `my_lane`: a freed lower-index slot can be reused by a later
        // merge, so "the new lane is always to the right" does not hold.
        let mut fan_out: HashMap<usize, (TreePart, ColorRef)> = HashMap::new();
        // Connector cell `2*k+1`, between lane `k` and lane `k+1`.
        let mut h_connector: HashMap<usize, ColorRef> = HashMap::new();
        // Glyph cell `2*k` of an intermediate lane the run passes over.
        let mut h_crossing: HashMap<usize, ColorRef> = HashMap::new();
        // The same two families of cells, for runs that are *history* --
        // everything but a stash's connector. A cell only a stash's run
        // passes is dashed; one history shares stays solid.
        let mut solid_connector: HashSet<usize> = HashSet::new();
        let mut solid_crossing: HashSet<usize> = HashSet::new();
        for &j in &new_parent_lanes {
            // The new lane's *own* color, not a universal merge hue: forcing every
            // merge into one fixed hue would inject an extra color into a small
            // palette on each merge row.
            let color = lane_colors[j];
            let (lo, hi, corner) = if j < my_lane {
                (j, my_lane, TreePart::BranchOutLeft)
            } else {
                (my_lane, j, TreePart::BranchOutRight)
            };
            fan_out.insert(j, (corner, color));
            for k in lo..hi {
                h_connector.entry(k).or_insert(color);
                solid_connector.insert(k);
            }
            for k in (lo + 1)..hi {
                h_crossing.entry(k).or_insert(color);
                solid_crossing.insert(k);
            }
        }

        // The mirror of a fan-out: which *other* live lanes converge into
        // this row's commit. Not necessarily a real git merge -- two tips
        // reaching a shared ancestor without an explicit merge commit
        // converge the same way. Detected against the state as of
        // *entering* this row, and keyed on `commit.id`; see the module
        // header for why keying on the first parent is wrong. A stash's
        // connector converges here too, into its base; history first, so
        // a cell both share takes history's color.
        let next_target = commit.parents.first().copied();
        let mut converging_lanes: Vec<usize> = (0..lanes.len())
            .filter(|&j| j != my_lane && lanes[j] == Some(commit.id) && !reserved.contains(&j))
            .collect();
        converging_lanes.sort_by_key(|j| stash_lanes.contains(j));
        let mut converge: HashMap<usize, (TreePart, ColorRef)> = HashMap::new();
        for &j in &converging_lanes {
            let color = lane_colors[j];
            let history = !stash_lanes.contains(&j);
            let (lo, hi, corner) = if j < my_lane {
                (j, my_lane, TreePart::MergeInLeft)
            } else {
                (my_lane, j, TreePart::MergeInRight)
            };
            converge.insert(j, (corner, color));
            // Inserted second, so a fan-out and a convergence overlapping
            // on one row keep the fan-out's color.
            for k in lo..hi {
                h_connector.entry(k).or_insert(color);
                if history {
                    solid_connector.insert(k);
                }
            }
            for k in (lo + 1)..hi {
                h_crossing.entry(k).or_insert(color);
                if history {
                    solid_crossing.insert(k);
                }
            }
        }

        // Does this commit have a transition running out of it to a lower /
        // higher lane index? Both maps are keyed by the *other* lane, so a
        // key on either side of `my_lane` is an arm on that side. A run
        // never passes *over* `my_lane` -- the node is always an endpoint
        // -- so these two flags are exactly "is there a connector cell
        // beside me", which is what the connectivity invariant checks.
        let arm_left = fan_out.keys().chain(converge.keys()).any(|&j| j < my_lane);
        let arm_right = fan_out.keys().chain(converge.keys()).any(|&j| j > my_lane);

        let kind = if is_merge {
            NodeKind::Merge
        } else if has_ref {
            NodeKind::Ref
        } else {
            NodeKind::Commit
        };

        let mut cells: Vec<TreeCell> = Vec::with_capacity(lanes.len() * 2);
        for j in 0..lanes.len() {
            let glyph_cell = if j == my_lane {
                let node = TreePart::Node {
                    kind,
                    sides: Sides::new(
                        lane_continues_up,
                        next_target.is_some(),
                        arm_left,
                        arm_right,
                    ),
                };
                // HEAD, reached by the WIP connector: the half of its lane
                // arriving from above is that connector's last stretch, so
                // it is dashed too. An arm only a stash's connector uses is
                // that connector's last stretch as well. The marker and
                // every other stroke are history and stay solid.
                let dashed = Sides::new(
                    wip_lane == Some(my_lane) && wip_head == Some(commit.id),
                    false,
                    arm_left && my_lane > 0 && !solid_connector.contains(&(my_lane - 1)),
                    arm_right && !solid_connector.contains(&my_lane),
                );
                let cell = TreeCell::new(node, my_color).with_dashed(dashed);
                // The marker's *shape* already says what kind of commit this
                // is, so bold is reserved for the two kinds worth picking out
                // of a column at a glance.
                if matches!(kind, NodeKind::Merge | NodeKind::Ref) {
                    cell.bold()
                } else {
                    cell
                }
            } else if let Some((corner, color)) = fan_out.get(&j) {
                // An elbow inks only the half of its cell its own two arms reach. That
                // is wrong once another run on the same row passes straight over this
                // column: the far half is blank and that run comes out cut in two (three
                // lanes converging on one node put a `╯` in the middle of the outermost
                // lane's run).
                let part = if h_crossing.contains_key(&j) {
                    TreePart::BranchOutCrossed
                } else {
                    *corner
                };
                TreeCell::new(part, *color).bold()
            } else if let Some((corner, color)) = converge.get(&j) {
                let part = if h_crossing.contains_key(&j) {
                    TreePart::MergeInCrossed
                } else {
                    *corner
                };
                let cell = TreeCell::new(part, *color).bold();
                if stash_lanes.contains(&j) {
                    // A stash's connector landing: its own stroke from
                    // above is pending, and so is the run to the base
                    // unless history crosses this column too.
                    let sides = part.sides();
                    let run = !solid_crossing.contains(&j);
                    cell.with_dashed(Sides::new(
                        true,
                        false,
                        sides.left && run,
                        sides.right && run,
                    ))
                } else {
                    cell
                }
            } else if lanes[j].is_some() && !reserved.contains(&j) {
                // A live lane a run passes over keeps its own vertical --
                // a readable crossing -- but it has to keep the horizontal
                // too: a bare `│` punches a hole in the run on either side
                // of the crossed lane, because `│` inks the full height of
                // its cell and none of its width.
                let crossed = h_crossing.contains_key(&j);
                let part = if crossed {
                    TreePart::Crossing
                } else {
                    TreePart::Vertical
                };
                // The WIP connector or a stash's passing a commit on its
                // way down: only its vertical is pending, a run crossing
                // it is history -- unless that run is a stash's too.
                let pending_lane = wip_lane == Some(j) || stash_lanes.contains(&j);
                let pending_run = crossed && !solid_crossing.contains(&j);
                TreeCell::new(part, lane_colors[j]).with_dashed(Sides::new(
                    pending_lane,
                    pending_lane,
                    pending_run,
                    pending_run,
                ))
            } else if let Some(color) = h_crossing.get(&j) {
                // A *dead* intermediate column has nothing of its own to
                // draw, so the run continues straight through it instead of
                // leaving a hole in the middle.
                let cell = TreeCell::new(TreePart::Horizontal, *color).bold();
                if solid_crossing.contains(&j) {
                    cell
                } else {
                    cell.with_dashed(Sides::HORIZONTAL)
                }
            } else {
                TreeCell::empty()
            };
            cells.push(glyph_cell);

            // Fill the gap between a transitioning lane and its destination
            // column, so the transition reads as one thing joining rather
            // than as disconnected columns.
            cells.push(match h_connector.get(&j) {
                Some(color) => {
                    let cell = TreeCell::new(TreePart::Horizontal, *color).bold();
                    if solid_connector.contains(&j) {
                        cell
                    } else {
                        cell.with_dashed(Sides::HORIZONTAL)
                    }
                }
                None => TreeCell::empty(),
            });
        }

        rows.push(GraphRow {
            cells,
            lane: my_lane,
            color: my_color,
        });

        if wip_lane == Some(my_lane) && wip_head == Some(commit.id) {
            // The connector has arrived; from here down the lane is
            // HEAD's own history.
            wip_lane = None;
        }
        lanes[my_lane] = next_target;
        // This lane now expects the first parent. A lane a stash reserved
        // for that same commit is not needed any more: the stash's
        // connector will converge into this one.
        if let Some(target) = next_target {
            if let Some(r) = (0..lanes.len())
                .find(|&j| j != my_lane && reserved.contains(&j) && lanes[j] == Some(target))
            {
                reserved.remove(&r);
                lanes[r] = None;
            }
        }
        // `converging_lanes` are the other lanes that just visually joined
        // `my_lane` at this row's node; clear them now. Deliberately narrower than
        // a blanket "collapse any two lanes sharing a value": two lanes can share
        // a future target and must keep drawing their own `│` until the row where
        // that commit truly appears.
        for &j in &converging_lanes {
            lanes[j] = None;
            stash_lanes.remove(&j);
        }
    }

    GraphGrid {
        rows,
        wip,
        stashes: stash_rows,
        stash_slots,
    }
}

/// The first free lane, or a new one on the right.
fn free_lane(lanes: &mut Vec<Option<Oid>>) -> usize {
    match lanes.iter().position(Option::is_none) {
        Some(i) => i,
        None => {
            lanes.push(None);
            lanes.len() - 1
        }
    }
}

/// One stash row: the box on a lane of its own, right of the lane its
/// base is (or will be) on, with a dashed stroke down towards the base.
/// Every other live lane passes through the row, as it would a commit's.
///
/// The lane the base will land in is found by looking ahead: the first
/// lane already expecting the base, or else the first lane whose
/// first-parent chain reaches it before its row (the common case of a
/// stash made a few commits back on the current branch). Only when no
/// lane gets there is one reserved for the base.
#[allow(clippy::too_many_arguments)] // one call site, the fold's own state
fn stash_row(
    base: Oid,
    nodes: &[CommitNode],
    index_of: &HashMap<Oid, usize>,
    lanes: &mut Vec<Option<Oid>>,
    lane_colors: &mut Vec<ColorRef>,
    stash_lanes: &mut HashSet<usize>,
    reserved: &mut HashSet<usize>,
    wip_lane: Option<usize>,
) -> GraphRow {
    let base_index = index_of.get(&base).copied().unwrap_or(usize::MAX);
    // Does the first-parent chain from `from` reach `base` before (or at)
    // the base's own row?
    let reaches = |from: Oid| {
        let mut at = from;
        // Bounded by the history's length whatever order it came in.
        for _ in 0..=nodes.len() {
            if at == base {
                return true;
            }
            match index_of.get(&at) {
                Some(&i) if i < base_index => match nodes[i].parents.first() {
                    Some(&parent) => at = parent,
                    None => return false,
                },
                _ => return false,
            }
        }
        false
    };
    let expecting = |j: usize| lanes[j].is_some() && !stash_lanes.contains(&j);
    let anchor = (0..lanes.len())
        .find(|&j| expecting(j) && lanes[j] == Some(base))
        .or_else(|| (0..lanes.len()).find(|&j| expecting(j) && lanes[j].is_some_and(reaches)));
    let anchor = match anchor {
        Some(j) => j,
        None => {
            let j = free_lane(lanes);
            lanes[j] = Some(base);
            reserved.insert(j);
            j
        }
    };
    let lane = match ((anchor + 1)..lanes.len()).find(|&j| lanes[j].is_none()) {
        Some(j) => j,
        None => {
            lanes.push(None);
            lanes.len() - 1
        }
    };
    lanes[lane] = Some(base);
    stash_lanes.insert(lane);
    if lane_colors.len() < lanes.len() {
        lane_colors.resize(lanes.len(), ColorRef::Lane(0));
    }
    lane_colors[lane] = ColorRef::Stash;

    let down = Sides::new(false, true, false, false);
    let mut cells: Vec<TreeCell> = Vec::with_capacity(lanes.len() * 2);
    for j in 0..lanes.len() {
        cells.push(if j == lane {
            TreeCell::new(
                TreePart::Node {
                    kind: NodeKind::Stash,
                    sides: down,
                },
                ColorRef::Stash,
            )
            .with_dashed(down)
        } else if lanes[j].is_some() && !reserved.contains(&j) {
            let cell = TreeCell::new(TreePart::Vertical, lane_colors[j]);
            if wip_lane == Some(j) || stash_lanes.contains(&j) {
                cell.with_dashed(Sides::VERTICAL)
            } else {
                cell
            }
        } else {
            TreeCell::empty()
        });
        cells.push(TreeCell::empty());
    }
    GraphRow {
        cells,
        lane,
        color: ColorRef::Stash,
    }
}
