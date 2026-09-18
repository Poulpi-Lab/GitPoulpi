//! The Log's per-frame layout, built once per change of its inputs.
//!
//! The draw closure runs on every poll tick (the spinner animates, the
//! cursor blinks), so redoing the solo filter, lane walk, stash placement
//! and the whole-log search each time would cost milliseconds per frame on a
//! few thousand commits for the same answer. [`LogLayoutCache`] keys that
//! work on what it reads and hands the draw closure the previous result
//! until an input changes.
//!
//! Inputs are tracked by *generation counters* the event loop bumps where
//! it replaces (or appends to) `log_result` / `refs_result` /
//! `stashes_result`, never by hashing the 5000-node log each frame; the
//! small inputs (solo set, hidden set, settings) are compared by value.

use std::collections::{HashMap, HashSet};

use gitpoulpi_core::domain::{CommitGraph, CommitNode, Oid, Ref, Stash};
use ratatui::style::Color;
use ratatui::text::Line;

use super::{
    build_graph_lines_with, GraphRender, GraphRow, GraphSettings, GraphStash, LaneBackground,
    LogRows, WipAnchor,
};
use crate::log_pills::RefPill;
use crate::log_search::SearchIndex;

/// Everything the layout is a function of.
pub(crate) struct LayoutInputs<'a> {
    pub log: &'a gitpoulpi_core::Result<CommitGraph>,
    pub log_gen: u64,
    pub refs: &'a gitpoulpi_core::Result<Vec<Ref>>,
    pub refs_gen: u64,
    pub stashes: &'a gitpoulpi_core::Result<Vec<Stash>>,
    pub stashes_gen: u64,
    pub solo: &'a HashSet<String>,
    pub hidden: &'a HashSet<String>,
    pub has_wip: bool,
    /// `ui.graph_show_stashes` (the cache applies the "not while soloing"
    /// rule itself).
    pub show_stashes: bool,
    pub settings: GraphSettings,
}

#[derive(PartialEq, Eq)]
struct Key {
    log_gen: u64,
    log_len: usize,
    refs_gen: u64,
    stashes_gen: u64,
    solo: HashSet<String>,
    hidden: HashSet<String>,
    has_wip: bool,
    show_stashes: bool,
    render: GraphRender,
    lane_background: LaneBackground,
    tint_strength: gitpoulpi_config::GraphRowTintStrength,
    theme: crate::graph::LaneTheme,
    /// The UI theme: WIP/stash colors and the Follow-theme lanes are baked in.
    theme_gen: u64,
}

impl Key {
    fn of(i: &LayoutInputs<'_>) -> Key {
        Key {
            log_gen: i.log_gen,
            log_len: i.log.as_ref().map_or(0, |g| g.nodes.len()),
            refs_gen: i.refs_gen,
            stashes_gen: i.stashes_gen,
            solo: i.solo.clone(),
            hidden: i.hidden.clone(),
            has_wip: i.has_wip,
            show_stashes: i.show_stashes,
            render: i.settings.render,
            lane_background: i.settings.lane_background,
            tint_strength: i.settings.tint_strength,
            theme: i.settings.theme,
            theme_gen: crate::theme_generation(),
        }
    }
}

/// One built layout: the solo-filtered commits and everything laid out
/// over them.
pub(crate) struct LogLayout {
    /// The commits the Log shows (after Solo); `None` for an unloaded or
    /// empty log.
    pub nodes: Option<Vec<CommitNode>>,
    /// Refs after "Hide in graph" and Solo - what pills are drawn from.
    pub refs_for_pills: Vec<Ref>,
    /// `ref_pills_for_commit` for every commit that has a ref, computed
    /// once with the layout instead of a scan of all refs per drawn row.
    pub pills_by_oid: HashMap<Oid, Vec<RefPill>>,
    /// One Graph cell (and lane color) per commit.
    pub rows: Vec<(Line<'static>, Color)>,
    pub grid: Vec<GraphRow>,
    pub wip_line: Option<Line<'static>>,
    pub wip_row: Option<GraphRow>,
    pub stash_rows: Vec<GraphRow>,
    /// Stash Graph cells, in display order.
    pub stash_lines: Vec<Line<'static>>,
    pub stash_row_stashes: Vec<Option<Stash>>,
    pub order: LogRows,
    /// `ui.graph_ghost_ref`: the refs commits are deduced to belong to, and per
    /// node of `nodes` the index of its own (`ghost_ref::deduce`).
    ghost_refs: Vec<super::ghost_ref::GhostRef>,
    ghost_owner: Vec<Option<u32>>,
}

#[derive(Default)]
pub(crate) struct LogLayoutCache {
    key: Option<Key>,
    layout: Option<LogLayout>,
    /// Lazily built once per layout, only once a search is open.
    index: Option<SearchIndex>,
    search_for: Option<(u64, String)>,
    matches: Vec<usize>,
    match_set: HashSet<usize>,
    /// Bumped on every layout rebuild.
    build_id: u64,
    /// Rebuild counters, read by the tests that prove an unchanged redraw
    /// rebuilds nothing.
    pub layout_builds: u64,
    pub search_scans: u64,
}

impl LogLayoutCache {
    /// Brings the layout (and, with `query`, the search matches) up to
    /// date, rebuilding only what the inputs changed.
    pub(crate) fn update(&mut self, inputs: &LayoutInputs<'_>, query: Option<&str>) {
        let key = Key::of(inputs);
        if self.key.as_ref() != Some(&key) {
            self.layout = Some(build(inputs));
            self.key = Some(key);
            self.build_id += 1;
            self.layout_builds += 1;
            self.index = None;
            self.search_for = None;
        }
        let Some(query) = query else {
            // Search closed: drop the haystacks, keep nothing stale.
            self.index = None;
            self.search_for = None;
            self.matches.clear();
            self.match_set.clear();
            return;
        };
        if self
            .search_for
            .as_ref()
            .is_some_and(|(id, q)| *id == self.build_id && q == query)
        {
            return;
        }
        let layout = self.layout.as_ref().expect("layout was just built");
        self.matches = match &layout.nodes {
            Some(nodes) if !query.trim().is_empty() => {
                let index = self
                    .index
                    .get_or_insert_with(|| SearchIndex::new(nodes, &layout.refs_for_pills));
                index.find(query)
            }
            _ => Vec::new(),
        };
        self.match_set = self.matches.iter().copied().collect();
        self.search_for = Some((self.build_id, query.to_owned()));
        self.search_scans += 1;
    }

    pub(crate) fn layout(&self) -> &LogLayout {
        self.layout.as_ref().expect("update() runs before layout()")
    }

    pub(crate) fn matches(&self) -> &[usize] {
        &self.matches
    }

    pub(crate) fn match_set(&self) -> &HashSet<usize> {
        &self.match_set
    }
}

impl LogLayout {
    /// The pills of `commit`, sorted as `ref_pills_for_commit` sorts them;
    /// empty when the commit carries no ref.
    pub(crate) fn pills_for(&self, commit: Oid) -> &[RefPill] {
        self.pills_by_oid.get(&commit).map_or(&[], Vec::as_slice)
    }

    /// The ref the commit at `node` (an index of `nodes`) belongs to when it
    /// has none of its own; `None` when nothing reaches it.
    pub(crate) fn ghost_for(&self, node: usize) -> Option<&super::ghost_ref::GhostRef> {
        let g = (*self.ghost_owner.get(node)?)?;
        self.ghost_refs.get(g as usize)
    }

    /// [`Self::ghost_for`] by commit id (the tooltip's lookup).
    pub(crate) fn ghost_for_commit(&self, commit: Oid) -> Option<&super::ghost_ref::GhostRef> {
        let node = self.nodes.as_ref()?.iter().position(|n| n.id == commit)?;
        self.ghost_for(node)
    }
}

pub(super) fn pills_by_oid(refs: &[Ref]) -> HashMap<Oid, Vec<RefPill>> {
    let mut by_target: HashMap<Oid, Vec<Ref>> = HashMap::new();
    for r in refs {
        by_target.entry(r.target).or_default().push(r.clone());
    }
    by_target
        .into_iter()
        .map(|(oid, refs)| (oid, crate::log_pills::ref_pills_for_commit(oid, &refs)))
        .collect()
}

pub(super) fn build(i: &LayoutInputs<'_>) -> LogLayout {
    let refs_for_pills: Vec<Ref> = i
        .refs
        .as_ref()
        .map(|refs| {
            refs.iter()
                .filter(|r| !i.hidden.contains(&crate::refs_sidebar::ref_full_name(r)))
                .filter(|r| {
                    i.solo.is_empty() || i.solo.contains(&crate::refs_sidebar::ref_full_name(r))
                })
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    let nodes: Option<Vec<CommitNode>> = match i.log {
        Ok(g) if !g.nodes.is_empty() => Some(crate::log_table::apply_solo_filter(
            &g.nodes, i.solo, i.refs,
        )),
        _ => None,
    };
    let wip_anchor = i.has_wip.then(|| WipAnchor {
        head: i
            .refs
            .as_deref()
            .ok()
            .and_then(crate::refs_sidebar::head_oid),
    });
    let graph_stashes: Vec<GraphStash> =
        if crate::log_table::stash_rows_shown(i.show_stashes, i.solo) {
            GraphStash::from_stashes(i.stashes.as_deref().unwrap_or(&[]))
        } else {
            Vec::new()
        };
    let mut graph = nodes.as_ref().map(|n| {
        build_graph_lines_with(n, &refs_for_pills, wip_anchor, &graph_stashes, i.settings)
    });
    let order = graph
        .as_ref()
        .map(|g| g.grid.log_rows())
        .unwrap_or_else(|| LogRows::new(i.has_wip, Vec::new()));
    let (wip_line, wip_row) = match (graph.as_mut(), wip_anchor) {
        (Some(g), _) => (g.wip.take(), g.grid.wip.take()),
        (None, Some(anchor)) => {
            let mut g = build_graph_lines_with(&[], &[], Some(anchor), &[], i.settings);
            (g.wip.take(), g.grid.wip.take())
        }
        (None, None) => (None, None),
    };
    let grid = graph
        .as_mut()
        .map(|g| std::mem::take(&mut g.grid.rows))
        .unwrap_or_default();
    let stash_rows = graph
        .as_mut()
        .map(|g| std::mem::take(&mut g.grid.stashes))
        .unwrap_or_default();
    let stash_lines = graph
        .as_mut()
        .map(|g| std::mem::take(&mut g.stashes))
        .unwrap_or_default();
    let rows = graph.map(|g| g.rows).unwrap_or_default();
    let stash_row_stashes = (0..order.slots().len())
        .map(|k| {
            order
                .stash_display_index(k)
                .and_then(|d| crate::log_table::log_stash_at(i.stashes, &order, d))
        })
        .collect();
    let pills_by_oid = pills_by_oid(&refs_for_pills);
    let (ghost_refs, ghost_owner) = match nodes.as_deref() {
        Some(n) => super::ghost_ref::deduce(n, &refs_for_pills),
        None => (Vec::new(), Vec::new()),
    };
    LogLayout {
        nodes,
        refs_for_pills,
        pills_by_oid,
        rows,
        grid,
        wip_line,
        wip_row,
        stash_rows,
        stash_lines,
        stash_row_stashes,
        order,
        ghost_refs,
        ghost_owner,
    }
}
