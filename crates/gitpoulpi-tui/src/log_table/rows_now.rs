//! The Log's display order and the Solo filter: which rows exist (WIP,
//! commits, stashes) as plain functions over the daemon's results. Pure;
//! re-exported by `log_table`.

use gitpoulpi_core::domain::{CommitNode, Oid, WorkingTree};

use crate::graph;
use crate::refs_sidebar::ref_full_name;

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
    solo_refs: &std::collections::HashSet<String>,
    refs_result: &gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Ref>>,
) -> graph::LogRows {
    let slots = match (show_stashes, log_result, stashes_result) {
        (true, Ok(g), Ok(stashes)) => {
            graph::place_stashes(&g.nodes, &graph::GraphStash::from_stashes(stashes))
        }
        _ => Vec::new(),
    };
    let shown = log_result
        .as_ref()
        .ok()
        .and_then(|g| solo_shown_indices(&g.nodes, solo_refs, refs_result));
    graph::LogRows::new(has_wip_row(status_result), slots).with_shown(shown)
}

/// Resolves one soloed ref (its full name, as stored in `solo_refs`) down
/// to the commit oid it points at right now - `None` if the ref no longer
/// exists (e.g. it was deleted or renamed while soloed).
pub(crate) fn solo_target_oid(solo_ref: &str, refs: &[gitpoulpi_core::domain::Ref]) -> Option<Oid> {
    refs.iter()
        .find(|r| ref_full_name(r) == solo_ref)
        .map(|r| r.target)
}

/// What the last [`solo_shown_indices`] answer was computed from.
#[derive(PartialEq, Eq)]
pub(crate) struct ShownKey {
    /// The held list's buffer and shape: a refresh or a page replaces or
    /// grows it, a new walk changes its ends.
    nodes_at: usize,
    len: usize,
    first: Option<Oid>,
    last: Option<Oid>,
    /// Where the soloed refs point now (the refs version, as far as Solo
    /// can tell).
    targets: std::collections::HashSet<Oid>,
}

pub(crate) type Shown = Option<std::sync::Arc<Vec<usize>>>;

thread_local! {
    pub(crate) static SHOWN_MEMO: std::cell::RefCell<Option<(ShownKey, Shown)>> =
        const { std::cell::RefCell::new(None) };
    /// How many times the reachability pass ran on this thread (tests).
    pub(crate) static SHOWN_COMPUTES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// The held indices of the commits `apply_solo_filter` keeps, or `None`
/// when it keeps them all (no Solo, or nothing to resolve).
///
/// The reachability pass over every held commit runs only when what it
/// depends on changed (the held list, where the soloed refs point): this is
/// asked every frame, and a repository of 100k commits must not pay for it
/// on each one.
pub(crate) fn solo_shown_indices(
    nodes: &[CommitNode],
    solo_refs: &std::collections::HashSet<String>,
    refs: &gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Ref>>,
) -> Shown {
    if solo_refs.is_empty() {
        return None;
    }
    let refs = refs.as_ref().ok()?;
    let targets: std::collections::HashSet<Oid> = solo_refs
        .iter()
        .filter_map(|name| solo_target_oid(name, refs))
        .collect();
    let key = ShownKey {
        nodes_at: nodes.as_ptr() as usize,
        len: nodes.len(),
        first: nodes.first().map(|c| c.id),
        last: nodes.last().map(|c| c.id),
        targets,
    };
    SHOWN_MEMO.with(|memo| {
        let mut memo = memo.borrow_mut();
        if let Some((held, value)) = memo.as_ref() {
            if *held == key {
                return value.clone();
            }
        }
        SHOWN_COMPUTES.with(|n| n.set(n.get() + 1));
        let targets: Vec<Oid> = key.targets.iter().copied().collect();
        let kept = solo_filter_nodes(nodes, &targets);
        let value = (!kept.is_empty()).then(|| {
            let ids: std::collections::HashSet<Oid> = kept.iter().map(|c| c.id).collect();
            std::sync::Arc::new(
                nodes
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| ids.contains(&c.id))
                    .map(|(i, _)| i)
                    .collect::<Vec<usize>>(),
            )
        });
        *memo = Some((key, value.clone()));
        value
    })
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

/// Whether the working tree has uncommitted changes, i.e. whether the Log
/// shows its WIP row as display index 0. The one spelling of that
/// question, so the render side and every event handler agree on it.
pub(crate) fn has_wip_row(status_result: &gitpoulpi_core::Result<WorkingTree>) -> bool {
    matches!(status_result, Ok(tree) if !tree.files.is_empty())
}
