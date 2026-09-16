//! A sequence of commits as a Mermaid `gitGraph` script (pure: no I/O).
//!
//! Mermaid's gitGraph is a *replay* language, not a graph description: every
//! `commit` lands on the checked-out branch, `branch` forks from the current
//! head and switches to the new branch, `merge` joins the *current head* of
//! another branch, and ids must be unique. So the generator plans lanes
//! first and then replays the commits oldest first:
//!
//! - A commit continues the lane of its first selected parent when nothing
//!   else has continued it yet; otherwise it opens a new lane.
//! - A new lane is created (`branch X`) right after its fork commit is
//!   emitted, so the fork point is the head at that moment even when the
//!   parent's lane moves on.
//! - A merge's other parent is merged through its lane when that lane's head
//!   still is that parent; otherwise a *pointer* branch (`<lane>-<sha>`) is
//!   created right after the parent and merged instead.
//! - Parents outside the selection are dropped: a commit left with no
//!   selected parent starts a lane of its own (declared up front, before the
//!   first commit), so a gap in the selection shows as disconnected lines.
//! - An octopus merge becomes sequential merges: the intermediate merge
//!   commits are suffixed ` (octopus k/n)`, the last carries the real id.
//!
//! Lane names: level `Structure` uses `main`, `b1`, `b2`...; the richer levels
//! use the local branch name found on the lane (remote branch otherwise).
//! The first lane's real name travels as `mainBranchName` in an `init`
//! header.

use std::cmp::Reverse;
use std::collections::{BTreeSet, BinaryHeap, HashMap, HashSet};

use serde::{Deserialize, Serialize};

use super::{CommitNode, Oid, Ref, RefKind};

/// How much of the history the script carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MermaidDetail {
    /// Commits, branches and merges; ids are short SHAs.
    Structure,
    /// Plus branch names, tags and the HEAD highlight.
    #[default]
    Refs,
    /// Plus the subject in the commit id.
    Subjects,
    /// Plus author and date in the commit id.
    Full,
}

impl MermaidDetail {
    pub const ALL: [MermaidDetail; 4] = [
        MermaidDetail::Structure,
        MermaidDetail::Refs,
        MermaidDetail::Subjects,
        MermaidDetail::Full,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Structure => "structure",
            Self::Refs => "refs",
            Self::Subjects => "subjects",
            Self::Full => "full",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|d| d.as_str() == s)
    }
}

/// The direction the graph grows in (`gitGraph LR:` / `gitGraph TB:`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MermaidOrientation {
    #[default]
    Lr,
    Tb,
}

impl MermaidOrientation {
    pub const ALL: [MermaidOrientation; 2] = [MermaidOrientation::Lr, MermaidOrientation::Tb];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lr => "lr",
            Self::Tb => "tb",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|o| o.as_str().eq_ignore_ascii_case(s))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MermaidOptions {
    pub detail: MermaidDetail,
    pub orientation: MermaidOrientation,
    /// Wrap the script in a ```mermaid fence.
    pub fence: bool,
    /// Subjects longer than this many characters are cut (`...`).
    pub subject_width: usize,
}

impl Default for MermaidOptions {
    fn default() -> Self {
        Self {
            detail: MermaidDetail::default(),
            orientation: MermaidOrientation::default(),
            fence: false,
            subject_width: 40,
        }
    }
}

/// Words the Mermaid lexer would read as syntax in a branch name position.
const KEYWORDS: [&str; 19] = [
    "gitGraph",
    "commit",
    "branch",
    "merge",
    "checkout",
    "switch",
    "cherry-pick",
    "id",
    "type",
    "tag",
    "order",
    "parent",
    "NORMAL",
    "REVERSE",
    "HIGHLIGHT",
    "LR",
    "TB",
    "BT",
    "accTitle",
];

/// One statement-level text with no quote, newline or control character.
fn clean(s: &str) -> String {
    let mapped: String = s
        .chars()
        .map(|c| match c {
            '"' => '\'',
            c if c.is_control() => ' ',
            c => c,
        })
        .collect();
    mapped.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn quote(s: &str) -> String {
    format!("\"{}\"", clean(s))
}

/// A branch name as it must be written after `branch` / `checkout` / `merge`.
fn branch_ref(name: &str) -> String {
    let plain = !name.is_empty()
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !KEYWORDS.contains(&name);
    if plain {
        name.to_string()
    } else {
        quote(name)
    }
}

fn truncate(s: &str, width: usize) -> String {
    if s.chars().count() <= width {
        return s.to_string();
    }
    let keep = width.saturating_sub(3);
    let cut: String = s.chars().take(keep).collect();
    format!("{}...", cut.trim_end())
}

/// `YYYY-MM-DD` (UTC) of a unix timestamp.
fn iso_date(ts: i64) -> String {
    let days = ts.div_euclid(86_400);
    // Howard Hinnant's civil_from_days.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

/// The shortest hex prefix length (at least 7) that tells every commit apart.
fn short_len(ids: &[Oid]) -> usize {
    let hex: Vec<String> = ids.iter().map(|o| o.to_hex()).collect();
    (7..=40)
        .find(|&n| {
            let set: HashSet<&str> = hex.iter().map(|h| &h[..n]).collect();
            set.len() == hex.len()
        })
        .unwrap_or(40)
}

/// Fills each node's `refs` from the repository's ref list (the Log walk
/// itself leaves them empty), skipping what the node already has.
pub fn attach_refs(nodes: &mut [CommitNode], refs: &[Ref]) {
    let mut at: HashMap<Oid, Vec<&RefKind>> = HashMap::new();
    for r in refs {
        at.entry(r.target).or_default().push(&r.kind);
    }
    for node in nodes {
        for kind in at.get(&node.id).into_iter().flatten() {
            if !node.refs.contains(kind) {
                node.refs.push((*kind).clone());
            }
        }
    }
}

/// The loaded commits from `from` up to `tip`: those reachable from `tip`
/// that have one of `from` as an ancestor (or are one), in `nodes` order. The
/// "from here to HEAD" range of the Log, answered from the loaded graph.
pub fn commits_to_tip(nodes: &[CommitNode], from: &[Oid], tip: Oid) -> Vec<CommitNode> {
    let by_id: HashMap<Oid, &CommitNode> = nodes.iter().map(|n| (n.id, n)).collect();
    let mut children: HashMap<Oid, Vec<Oid>> = HashMap::new();
    for n in nodes {
        for p in &n.parents {
            children.entry(*p).or_default().push(n.id);
        }
    }
    let mut below_tip: HashSet<Oid> = HashSet::new();
    let mut stack = vec![tip];
    while let Some(id) = stack.pop() {
        if by_id.contains_key(&id) && below_tip.insert(id) {
            stack.extend(by_id[&id].parents.iter().copied());
        }
    }
    let mut above_from: HashSet<Oid> = HashSet::new();
    let mut stack: Vec<Oid> = from
        .iter()
        .copied()
        .filter(|f| by_id.contains_key(f))
        .collect();
    while let Some(id) = stack.pop() {
        if above_from.insert(id) {
            stack.extend(children.get(&id).into_iter().flatten().copied());
        }
    }
    nodes
        .iter()
        .filter(|n| below_tip.contains(&n.id) && above_from.contains(&n.id))
        .cloned()
        .collect()
}

/// Where a commit is emitted.
struct Plan {
    /// Selected parents, deduplicated, first parent first.
    parents: Vec<usize>,
    lane: usize,
    /// Branches to create right after this commit is emitted, in order.
    pointers: Vec<Pointer>,
}

#[derive(Clone)]
enum Pointer {
    /// The lane's own branch, forked here.
    Lane(usize),
    /// A branch whose only job is to name this commit for a later merge.
    Stub(String),
}

/// The Mermaid `gitGraph` script of `commits` (any order; parents outside the
/// slice are ignored). `head` is the checked-out commit, if it is in the
/// slice.
pub fn mermaid_git_graph(
    commits: &[CommitNode],
    head: Option<Oid>,
    opts: &MermaidOptions,
) -> String {
    // Deduplicate, keep first occurrence.
    let mut seen = HashSet::new();
    let nodes: Vec<&CommitNode> = commits.iter().filter(|n| seen.insert(n.id)).collect();
    let n = nodes.len();
    let mut out = String::new();
    if n == 0 {
        finish(&mut out, opts, None);
        return out;
    }
    let index: HashMap<Oid, usize> = nodes.iter().enumerate().map(|(i, c)| (c.id, i)).collect();
    let parents: Vec<Vec<usize>> = nodes
        .iter()
        .map(|c| {
            let mut ps = Vec::new();
            for p in &c.parents {
                if let Some(&i) = index.get(p) {
                    if !ps.contains(&i) {
                        ps.push(i);
                    }
                }
            }
            ps
        })
        .collect();

    // Topological order, oldest first; ties by timestamp then input order
    // (the input is usually newest first).
    let mut indegree: Vec<usize> = parents.iter().map(Vec::len).collect();
    let mut children: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (c, ps) in parents.iter().enumerate() {
        for &p in ps {
            children[p].push(c);
        }
    }
    let key = |i: usize| Reverse((nodes[i].timestamp, n - 1 - i, i));
    let mut ready: BinaryHeap<Reverse<(i64, usize, usize)>> = BinaryHeap::new();
    for (i, d) in indegree.iter().enumerate() {
        if *d == 0 {
            ready.push(key(i));
        }
    }
    let mut order = Vec::with_capacity(n);
    while let Some(Reverse((_, _, i))) = ready.pop() {
        order.push(i);
        for &c in &children[i] {
            indegree[c] -= 1;
            if indegree[c] == 0 {
                ready.push(key(c));
            }
        }
    }
    // A cycle cannot happen in a git history; keep the function total anyway.
    if order.len() < n {
        let placed: HashSet<usize> = order.iter().copied().collect();
        order.extend((0..n).filter(|i| !placed.contains(i)));
    }
    let mut pos = vec![0usize; n];
    for (p, &i) in order.iter().enumerate() {
        pos[i] = p;
    }

    // Lanes.
    let mut plans: Vec<Plan> = (0..n)
        .map(|i| Plan {
            parents: parents[i]
                .iter()
                .copied()
                .filter(|&p| pos[p] < pos[i])
                .collect(),
            lane: 0,
            pointers: Vec::new(),
        })
        .collect();
    // Which child continues its first parent's lane: the one on HEAD's
    // first-parent chain, else the one with the longest first-parent run
    // above it, else the oldest.
    // `main` (lane 0) belongs to the same chain: the one from HEAD, else from
    // the newest commit nothing builds on, down to its root.
    let mut on_head = vec![false; n];
    let tip = head.and_then(|h| index.get(&h).copied()).or_else(|| {
        (0..n)
            .filter(|&i| children[i].is_empty())
            .max_by_key(|&i| (nodes[i].timestamp, std::cmp::Reverse(i)))
    });
    let mut walk = tip;
    let mut primary_root = tip;
    while let Some(i) = walk {
        on_head[i] = true;
        primary_root = Some(i);
        walk = plans[i].parents.first().copied();
    }
    let mut lane_count = 1usize;
    let mut height = vec![1usize; n];
    for &i in order.iter().rev() {
        if let Some(&p0) = plans[i].parents.first() {
            height[p0] = height[p0].max(height[i] + 1);
        }
    }
    let mut chosen: HashMap<usize, usize> = HashMap::new();
    for &i in &order {
        if let Some(&p0) = plans[i].parents.first() {
            let better = |a: usize, b: usize| (on_head[a], height[a]) > (on_head[b], height[b]);
            match chosen.get(&p0) {
                Some(&cur) if !better(i, cur) => {}
                _ => {
                    chosen.insert(p0, i);
                }
            }
        }
    }
    let mut root_lanes: Vec<usize> = Vec::new();
    for &i in &order {
        match plans[i].parents.first().copied() {
            Some(p0) if chosen[&p0] == i => {
                plans[i].lane = plans[p0].lane;
            }
            Some(p0) => {
                let lane = lane_count;
                lane_count += 1;
                plans[i].lane = lane;
                plans[p0].pointers.push(Pointer::Lane(lane));
            }
            None if Some(i) == primary_root => plans[i].lane = 0,
            None => {
                let lane = lane_count;
                lane_count += 1;
                plans[i].lane = lane;
                root_lanes.push(lane);
            }
        }
    }
    let mut lane_members: Vec<Vec<usize>> = vec![Vec::new(); lane_count];
    for &i in &order {
        lane_members[plans[i].lane].push(i);
    }

    // Names.
    let rich = opts.detail != MermaidDetail::Structure;
    let mut used: BTreeSet<String> = BTreeSet::new();
    let mut lane_names: Vec<String> = Vec::with_capacity(lane_count);
    let mut main_override: Option<String> = None;
    for (lane, members) in lane_members.iter().enumerate() {
        let wanted = if rich {
            members
                .iter()
                .rev()
                .find_map(|&i| {
                    nodes[i].refs.iter().find_map(|r| match r {
                        RefKind::LocalBranch { name } if !used.contains(name) => Some(name.clone()),
                        _ => None,
                    })
                })
                .or_else(|| {
                    members.iter().rev().find_map(|&i| {
                        nodes[i].refs.iter().find_map(|r| match r {
                            RefKind::RemoteBranch { remote, name } => {
                                let full = format!("{remote}/{name}");
                                (!used.contains(&full)).then_some(full)
                            }
                            _ => None,
                        })
                    })
                })
        } else {
            None
        };
        let name = match wanted {
            Some(w) if lane == 0 => {
                let safe = w
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '/'))
                    && !KEYWORDS.contains(&w.as_str());
                if safe {
                    main_override = Some(w.clone());
                    w
                } else {
                    "main".to_string()
                }
            }
            Some(w) => w,
            None if lane == 0 => "main".to_string(),
            None => {
                let mut k = lane;
                loop {
                    let cand = format!("b{k}");
                    if !used.contains(&cand) && !lane_names.contains(&cand) {
                        break cand;
                    }
                    k += 1;
                }
            }
        };
        let mut unique = clean(&name);
        let mut k = 2;
        while used.contains(&unique) {
            unique = format!("{} {k}", clean(&name));
            k += 1;
        }
        used.insert(unique.clone());
        lane_names.push(unique);
    }

    // Merge targets: a lane whose head is still the parent, or a pointer.
    // merge_target[(commit, parent_idx_in_plan)] = branch name.
    let mut merge_targets: HashMap<(usize, usize), String> = HashMap::new();
    for &c in &order {
        let ps = plans[c].parents.clone();
        for (k, &p) in ps.iter().enumerate().skip(1) {
            let lane = plans[p].lane;
            let members = &lane_members[lane];
            let at = members.iter().position(|&m| m == p).unwrap_or(0);
            let moved = members
                .get(at + 1)
                .is_some_and(|&next| pos[next] < pos[c] || next == c);
            // Mermaid refuses a merge whose current head was *created on*
            // the branch being merged (`Cannot merge ... into itself`).
            let from_that_branch = ps.first().is_some_and(|&p0| plans[p0].lane == lane);
            if moved || from_that_branch {
                let base = format!("{}-{}", lane_names[lane], &nodes[p].id.to_hex()[..7]);
                let mut name = clean(&base);
                let mut j = 2;
                while used.contains(&name) {
                    name = format!("{} {j}", clean(&base));
                    j += 1;
                }
                used.insert(name.clone());
                plans[p].pointers.push(Pointer::Stub(name.clone()));
                merge_targets.insert((c, k), name);
            } else {
                merge_targets.insert((c, k), lane_names[lane].clone());
            }
        }
    }

    // Ids.
    let len = short_len(&nodes.iter().map(|c| c.id).collect::<Vec<_>>());
    let mut taken: HashSet<String> = HashSet::new();
    let ids: Vec<String> = nodes
        .iter()
        .map(|c| {
            let sha = c.id.to_hex()[..len].to_string();
            let text = match opts.detail {
                MermaidDetail::Structure | MermaidDetail::Refs => sha,
                MermaidDetail::Subjects => {
                    let s = truncate(&clean(&c.summary), opts.subject_width);
                    if s.is_empty() {
                        sha
                    } else {
                        format!("{sha} {s}")
                    }
                }
                MermaidDetail::Full => {
                    let s = truncate(&clean(&c.summary), opts.subject_width);
                    let who = clean(&c.author_name);
                    let mut t = sha;
                    if !s.is_empty() {
                        t = format!("{t} {s}");
                    }
                    format!("{t} - {who}, {}", iso_date(c.timestamp))
                }
            };
            let mut unique = text.clone();
            let mut k = 2;
            while !taken.insert(unique.clone()) {
                unique = format!("{text} ({k})");
                k += 1;
            }
            unique
        })
        .collect();

    // Emit.
    let header_main = main_override.filter(|m| m != "main");
    let mut body = String::new();
    let mut cur_name = lane_names[0].clone();
    for &lane in &root_lanes {
        body.push_str(&format!("  branch {}\n", branch_ref(&lane_names[lane])));
        cur_name = lane_names[lane].clone();
    }
    for &c in &order {
        let lane_name = &lane_names[plans[c].lane];
        if cur_name != *lane_name {
            body.push_str(&format!("  checkout {}\n", branch_ref(lane_name)));
            cur_name = lane_name.clone();
        }
        let mut attrs = String::new();
        if rich {
            if head == Some(nodes[c].id) {
                attrs.push_str(" type: HIGHLIGHT");
            }
            let tags: Vec<String> = nodes[c]
                .refs
                .iter()
                .filter_map(|r| match r {
                    RefKind::Tag { name } => Some(clean(name)),
                    _ => None,
                })
                .collect();
            if !tags.is_empty() {
                attrs.push_str(&format!(" tag: {}", quote(&tags.join(", "))));
            }
        }
        let ps = &plans[c].parents;
        if ps.len() >= 2 {
            let extra = ps.len() - 1;
            for k in 1..=extra {
                let target = &merge_targets[&(c, k)];
                if k < extra {
                    body.push_str(&format!(
                        "  merge {} id: {}\n",
                        branch_ref(target),
                        quote(&format!("{} (octopus {k}/{extra})", ids[c]))
                    ));
                } else {
                    body.push_str(&format!(
                        "  merge {} id: {}{attrs}\n",
                        branch_ref(target),
                        quote(&ids[c])
                    ));
                }
            }
        } else {
            body.push_str(&format!("  commit id: {}{attrs}\n", quote(&ids[c])));
        }
        for ptr in plans[c].pointers.clone() {
            let name = match ptr {
                Pointer::Lane(l) => lane_names[l].clone(),
                Pointer::Stub(s) => s,
            };
            body.push_str(&format!("  branch {}\n", branch_ref(&name)));
            cur_name = name;
        }
    }
    out.push_str(&body);
    finish(&mut out, opts, header_main);
    out
}

/// Wraps `out` (the statements) with the header, the optional `init`
/// directive and the fence.
fn finish(out: &mut String, opts: &MermaidOptions, main_name: Option<String>) {
    let mut text = String::new();
    if let Some(name) = main_name {
        text.push_str(&format!(
            "%%{{init: {{'gitGraph': {{'mainBranchName': '{name}'}}}}}}%%\n"
        ));
    }
    text.push_str(match opts.orientation {
        MermaidOrientation::Lr => "gitGraph LR:\n",
        MermaidOrientation::Tb => "gitGraph TB:\n",
    });
    text.push_str(out);
    if opts.fence {
        text = format!("```mermaid\n{text}```\n");
    }
    *out = text;
}

#[cfg(test)]
mod tests;
