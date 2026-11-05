//! The Log pane's commit table: building a commit row, the columns
//! (order, widths, hiding, header hit-testing, boundary drags), solo
//! filtering, search hits, and the Log's row arithmetic (WIP row, paging,
//! index <-> screen row).

use super::*;

mod columns;
mod fit;
mod geo;
mod geometry;
mod header_drag;
mod keyboard;
mod natural;
mod prefs;
mod row;
mod wip;

pub(crate) use columns::*;
pub(crate) use fit::*;
pub(crate) use geo::*;
pub(crate) use geometry::*;
pub(crate) use header_drag::*;
pub(crate) use keyboard::*;
pub(crate) use natural::*;
pub(crate) use prefs::*;
pub(crate) use row::*;
pub(crate) use wip::*;

/// The currently-selected commit in the Log table: `None` on the WIP row
/// and on a stash row, which are not commits to cherry-pick, reset onto
/// or branch from. Every display-index conversion goes through `rows`
/// ([`graph::LogRows`]), never a hand-rolled `- has_wip`: a stash row
/// between two commits shifts every commit below it.
pub(crate) fn selected_log_commit(
    log_result: &gitpoulpi_core::Result<gitpoulpi_core::domain::CommitGraph>,
    log_state: &TableState,
    rows: &graph::LogRows,
) -> Option<Oid> {
    let g = log_result.as_ref().ok()?;
    match rows.entry(log_state.selected()?) {
        graph::LogEntry::Commit(i) => g.nodes.get(i).map(|c| c.id),
        graph::LogEntry::Wip | graph::LogEntry::Stash(_) => None,
    }
}

/// The commits on the Log rows just below and just above the selection,
/// below first - what the commit-files prefetch warms.
/// The WIP row and stash rows are skipped: neither is a commit whose
/// files the next keypress would ask for by this path.
pub(crate) fn log_neighbor_commits(
    log_result: &gitpoulpi_core::Result<gitpoulpi_core::domain::CommitGraph>,
    log_state: &TableState,
    rows: &graph::LogRows,
) -> Vec<Oid> {
    let (Ok(g), Some(selected)) = (log_result.as_ref(), log_state.selected()) else {
        return Vec::new();
    };
    let len = rows.len(g.nodes.len());
    [selected.checked_add(1), selected.checked_sub(1)]
        .into_iter()
        .flatten()
        .filter(|&row| row < len)
        .filter_map(|row| match rows.entry(row) {
            graph::LogEntry::Commit(i) => g.nodes.get(i).map(|c| c.id),
            graph::LogEntry::Wip | graph::LogEntry::Stash(_) => None,
        })
        .collect()
}

/// Whether stash rows are drawn in the Log: `ui.graph_show_stashes`, and
/// never while soloing. Solo shows the soloed refs' ancestry and nothing
/// else, and the Log's selection arithmetic under solo reads the
/// unfiltered history - a stash row there would be one more thing for
/// the two to disagree about.
pub(crate) fn stash_rows_shown(
    setting: bool,
    solo_refs: &std::collections::HashSet<String>,
) -> bool {
    setting && solo_refs.is_empty()
}

/// The Log's display order right now: the WIP row, the commits, and the
/// stash rows placed among them ([`graph::place_stashes`]). Cheap - no
/// clone, one scan per stash - so it is recomputed wherever the inputs
/// change rather than cached.
pub(crate) fn log_rows_now(
    log_result: &gitpoulpi_core::Result<gitpoulpi_core::domain::CommitGraph>,
    status_result: &gitpoulpi_core::Result<WorkingTree>,
    stashes_result: &gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Stash>>,
    show_stashes: bool,
) -> graph::LogRows {
    let slots = match (show_stashes, log_result, stashes_result) {
        (true, Ok(g), Ok(stashes)) => {
            graph::place_stashes(&g.nodes, &graph::GraphStash::from_stashes(stashes))
        }
        _ => Vec::new(),
    };
    graph::LogRows::new(has_wip_row(status_result), slots)
}

/// How many rows the Log has: the (solo-filtered) commits plus the WIP
/// and stash rows of `rows`.
pub(crate) fn log_len_now(
    log_result: &gitpoulpi_core::Result<gitpoulpi_core::domain::CommitGraph>,
    solo_refs: &std::collections::HashSet<String>,
    refs_result: &gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Ref>>,
    rows: &graph::LogRows,
) -> usize {
    let commits = log_result
        .as_ref()
        .map(|g| apply_solo_filter(&g.nodes, solo_refs, refs_result).len())
        .unwrap_or(0);
    rows.len(commits)
}

/// The stash drawn at Log display index `index`, if a stash row is there.
pub(crate) fn log_stash_at(
    stashes_result: &gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Stash>>,
    rows: &graph::LogRows,
    index: usize,
) -> Option<gitpoulpi_core::domain::Stash> {
    let graph::LogEntry::Stash(k) = rows.entry(index) else {
        return None;
    };
    let stashes = stashes_result.as_ref().ok()?;
    // `slots[k].stash` indexes the drawable stashes (those with a commit
    // and a base), which is not necessarily the full list: find it by id.
    let drawable = graph::GraphStash::from_stashes(stashes);
    let id = drawable.get(rows.slots().get(k)?.stash)?.id;
    stashes.iter().find(|s| s.id.0 == id).cloned()
}

/// Whether the Log's selection is on the pinned WIP row - the keyboard
/// twin of hovering it, and what tells the menu key
/// (`.`/`Shift+F10`/`Menu`) to open the WIP row's own context menu
/// instead of the per-commit one.
pub(crate) fn selected_log_is_wip(log_state: &TableState, rows: &graph::LogRows) -> bool {
    log_state
        .selected()
        .is_some_and(|i| matches!(rows.entry(i), graph::LogEntry::Wip))
}

/// The stash row the Log's selection is on, if it is on one.
pub(crate) fn selected_log_stash(
    stashes_result: &gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Stash>>,
    log_state: &TableState,
    rows: &graph::LogRows,
) -> Option<gitpoulpi_core::domain::Stash> {
    log_stash_at(stashes_result, rows, log_state.selected()?)
}

/// `Name <email>`, or just the name when there is no address - a stash
/// detailed as a commit is "authored" by its `stash@{n}` handle and has
/// none, and `stash@{0} <>` reads as a bug.
#[cfg(test)]
pub(crate) fn author_label(name: &str, email: &str) -> String {
    if email.is_empty() {
        name.to_string()
    } else {
        format!("{name} <{email}>")
    }
}

/// Resolves one soloed ref (its full name, as stored in `solo_refs`) down
/// to the commit oid it points at right now - `None` if the ref no longer
/// exists (e.g. it was deleted or renamed while soloed).
pub(crate) fn solo_target_oid(solo_ref: &str, refs: &[gitpoulpi_core::domain::Ref]) -> Option<Oid> {
    refs.iter()
        .find(|r| ref_full_name(r) == solo_ref)
        .map(|r| r.target)
}

/// Every commit reachable from any oid in `targets` by walking `parents`
/// (both a first-parent mainline and every merge parent) - the ancestry
/// Solo mode shows the definition of "this branch's own
/// commit line", unioned across every currently-soloed ref: Solo
/// accumulates (soloing a second branch adds its ancestry to the view
/// instead of replacing the first one's), so the reachability walk seeds
/// its stack with every target at once rather than being called
/// separately per ref and merged after the fact - same result, one pass.
pub(crate) fn solo_filter_nodes(nodes: &[CommitNode], targets: &[Oid]) -> Vec<CommitNode> {
    let by_id: std::collections::HashMap<Oid, &CommitNode> =
        nodes.iter().map(|c| (c.id, c)).collect();
    let mut reachable: std::collections::HashSet<Oid> = std::collections::HashSet::new();
    let mut stack: Vec<Oid> = targets.to_vec();
    while let Some(id) = stack.pop() {
        if !reachable.insert(id) {
            continue;
        }
        if let Some(c) = by_id.get(&id) {
            stack.extend(c.parents.iter().copied());
        }
    }
    nodes
        .iter()
        .filter(|c| reachable.contains(&c.id))
        .cloned()
        .collect()
}

/// Match indices for `query`, over exactly the node list the Log renders
/// (i.e. after the solo filter).
///
/// The render pass computes the same thing inline from `solo_nodes`; this
/// exists so the key handler - which runs outside the render closure and
/// has no `solo_nodes` in scope - can't drift to a different definition
/// of "what matches". Both go through `log_search::find_matches`, so the
/// only thing duplicated is the cheap solo-filter call.
pub(crate) fn log_search_hits(
    log_result: &gitpoulpi_core::Result<gitpoulpi_core::domain::CommitGraph>,
    refs_result: &gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Ref>>,
    solo_refs: &std::collections::HashSet<String>,
    query: &str,
) -> Vec<usize> {
    let Ok(commit_graph) = log_result else {
        return Vec::new();
    };
    if commit_graph.nodes.is_empty() {
        return Vec::new();
    }
    let nodes = apply_solo_filter(&commit_graph.nodes, solo_refs, refs_result);
    let empty = Vec::new();
    let refs = refs_result.as_ref().unwrap_or(&empty);
    log_search::find_matches(&nodes, refs, query)
}

/// The node list actually shown in the Log/graph for this frame: the full
/// `nodes` list, unless one or more refs are currently soloed, in which
/// case only the union of their ancestries. Refs whose target can't be
/// resolved (deleted/renamed while soloed) are silently skipped rather
/// than blanking the whole view; if every soloed ref fails to resolve, or
/// the soloed set is empty, falls back to the full list - Solo silently
/// doing nothing beats the graph going blank.
pub(crate) fn apply_solo_filter(
    nodes: &[CommitNode],
    solo_refs: &std::collections::HashSet<String>,
    refs: &gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Ref>>,
) -> Vec<CommitNode> {
    let filtered = (!solo_refs.is_empty())
        .then(|| refs.as_ref().ok())
        .flatten()
        .and_then(|refs| {
            let targets: Vec<Oid> = solo_refs
                .iter()
                .filter_map(|name| solo_target_oid(name, refs))
                .collect();
            let filtered = solo_filter_nodes(nodes, &targets);
            (!filtered.is_empty()).then_some(filtered)
        });
    filtered.unwrap_or_else(|| nodes.to_vec())
}

#[cfg(test)]
mod tests;
