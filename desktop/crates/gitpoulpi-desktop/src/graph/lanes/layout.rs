//! Lane assignment (the terminal UI's `layout.rs`).

use std::collections::{HashMap, HashSet};

use gitpoulpi_core::domain::{CommitNode, Oid};

use super::part::*;
use super::stash::*;

#[derive(Debug, Clone)]
pub struct GraphRow {
    pub cells: Vec<TreeCell>,
    pub lane: usize,
    pub color: ColorRef,
}

#[derive(Debug, Clone, Default)]
pub struct GraphGrid {
    pub rows: Vec<GraphRow>,
    pub wip: Option<GraphRow>,
    pub stashes: Vec<GraphRow>,
    pub stash_slots: Vec<StashSlot>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WipAnchor {
    pub head: Option<Oid>,
}

pub fn build_graph_grid(
    nodes: &[CommitNode],
    ref_targets: &HashSet<Oid>,
    wip: Option<WipAnchor>,
    stashes: &[GraphStash],
    palette_len: usize,
) -> GraphGrid {
    let lane0 = ColorRef::Lane(0);
    let pick = |n: usize| ColorRef::Lane(n % palette_len.max(1));
    let mut lanes: Vec<Option<Oid>> = Vec::new();
    let mut lane_colors: Vec<ColorRef> = Vec::new();
    let mut next_color = 0usize;
    let mut rows: Vec<GraphRow> = Vec::with_capacity(nodes.len());
    let mut stash_lanes: HashSet<usize> = HashSet::new();
    let mut reserved: HashSet<usize> = HashSet::new();

    let wip_head: Option<Oid> = wip
        .and_then(|w| w.head)
        .filter(|head| nodes.iter().any(|n| n.id == *head));
    let mut wip_lane: Option<usize> = None;
    let wip: Option<GraphRow> = wip.map(|_| match wip_head {
        Some(head) => {
            let color = lane0;
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
    let index_of: HashMap<Oid, usize> = if stash_slots.is_empty() {
        HashMap::new()
    } else {
        nodes.iter().enumerate().map(|(i, n)| (n.id, i)).collect()
    };

    for (index, commit) in nodes.iter().enumerate() {
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

        let expecting =
            (0..lanes.len()).find(|&j| lanes[j] == Some(commit.id) && !stash_lanes.contains(&j));
        let lane_continues_up = expecting.is_some_and(|j| !reserved.remove(&j));
        let my_lane = match expecting {
            Some(i) if lane_continues_up => i,
            found => {
                let i = match found {
                    Some(i) => i,
                    None => free_lane(&mut lanes),
                };
                if lane_colors.len() <= i {
                    lane_colors.resize(i + 1, lane0);
                }
                lane_colors[i] = pick(next_color);
                next_color += 1;
                i
            }
        };
        if lanes[my_lane].is_none() {
            lanes[my_lane] = Some(commit.id);
        }

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
                lane_colors.resize(lane_idx + 1, lane0);
            }
            lane_colors[lane_idx] = pick(next_color);
            next_color += 1;
        }
        let is_merge = !new_parent_lanes.is_empty();
        let has_ref = ref_targets.contains(&commit.id);
        let my_color = lane_colors[my_lane];

        let mut fan_out: HashMap<usize, (TreePart, ColorRef)> = HashMap::new();
        let mut h_connector: HashMap<usize, ColorRef> = HashMap::new();
        let mut h_crossing: HashMap<usize, ColorRef> = HashMap::new();
        let mut solid_connector: HashSet<usize> = HashSet::new();
        let mut solid_crossing: HashSet<usize> = HashSet::new();
        for &j in &new_parent_lanes {
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
                let dashed = Sides::new(
                    wip_lane == Some(my_lane) && wip_head == Some(commit.id),
                    false,
                    arm_left && my_lane > 0 && !solid_connector.contains(&(my_lane - 1)),
                    arm_right && !solid_connector.contains(&my_lane),
                );
                TreeCell::new(node, my_color).with_dashed(dashed)
            } else if let Some((corner, color)) = fan_out.get(&j) {
                let part = if h_crossing.contains_key(&j) {
                    TreePart::BranchOutCrossed
                } else {
                    *corner
                };
                TreeCell::new(part, *color)
            } else if let Some((corner, color)) = converge.get(&j) {
                let part = if h_crossing.contains_key(&j) {
                    TreePart::MergeInCrossed
                } else {
                    *corner
                };
                let cell = TreeCell::new(part, *color);
                if stash_lanes.contains(&j) {
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
                let crossed = h_crossing.contains_key(&j);
                let part = if crossed {
                    TreePart::Crossing
                } else {
                    TreePart::Vertical
                };
                let pending_lane = wip_lane == Some(j) || stash_lanes.contains(&j);
                let pending_run = crossed && !solid_crossing.contains(&j);
                TreeCell::new(part, lane_colors[j]).with_dashed(Sides::new(
                    pending_lane,
                    pending_lane,
                    pending_run,
                    pending_run,
                ))
            } else if let Some(color) = h_crossing.get(&j) {
                let cell = TreeCell::new(TreePart::Horizontal, *color);
                if solid_crossing.contains(&j) {
                    cell
                } else {
                    cell.with_dashed(Sides::HORIZONTAL)
                }
            } else {
                TreeCell::empty()
            };
            cells.push(glyph_cell);
            cells.push(match h_connector.get(&j) {
                Some(color) => {
                    let cell = TreeCell::new(TreePart::Horizontal, *color);
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
            wip_lane = None;
        }
        lanes[my_lane] = next_target;
        if let Some(target) = next_target {
            if let Some(r) = (0..lanes.len())
                .find(|&j| j != my_lane && reserved.contains(&j) && lanes[j] == Some(target))
            {
                reserved.remove(&r);
                lanes[r] = None;
            }
        }
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

fn free_lane(lanes: &mut Vec<Option<Oid>>) -> usize {
    match lanes.iter().position(Option::is_none) {
        Some(i) => i,
        None => {
            lanes.push(None);
            lanes.len() - 1
        }
    }
}

#[allow(clippy::too_many_arguments)]
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
    let reaches = |from: Oid| {
        let mut at = from;
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
