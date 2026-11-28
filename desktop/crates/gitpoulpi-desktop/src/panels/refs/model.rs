//! The Refs sidebar's data: sections, the folder tree built from `a/b/c` names, and the filter.

use std::collections::{BTreeMap, HashMap};

use gitpoulpi_core::domain::{Oid, Ref, RefKind, Stash, Worktree};
use gitpoulpi_i18n::msg;
use gpui_kit::component::tree::TreeItem;
use gpui_kit::SharedString;

use crate::strings;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Local,
    Remote,
    Worktrees,
    Stashes,
    Tags,
}

impl Kind {
    pub const ALL: [Kind; 5] = [
        Kind::Local,
        Kind::Remote,
        Kind::Worktrees,
        Kind::Stashes,
        Kind::Tags,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Kind::Local => msg::sidebar::SECTION_LOCAL.get(),
            Kind::Remote => msg::sidebar::SECTION_REMOTE.get(),
            Kind::Worktrees => msg::sidebar::SECTION_WORKTREES.get(),
            Kind::Stashes => msg::sidebar::SECTION_STASHES.get(),
            Kind::Tags => msg::sidebar::SECTION_TAGS.get(),
        }
    }
}

/// What a tree item stands for, looked up by its id.
#[derive(Clone)]
pub struct Meta {
    pub is_head: bool,
    pub tag_remote: bool,
    pub target: Option<Oid>,
    pub folder: bool,
}

impl Meta {
    fn leaf(is_head: bool, tag_remote: bool, target: Option<Oid>) -> Self {
        Self {
            is_head,
            tag_remote,
            target,
            folder: false,
        }
    }

    fn folder() -> Self {
        Self {
            is_head: false,
            tag_remote: false,
            target: None,
            folder: true,
        }
    }
}

#[derive(Default)]
struct Node {
    children: BTreeMap<String, Node>,
    leaf: Option<SharedString>,
}

fn insert(node: &mut Node, path: &str, id: SharedString) {
    let mut cur = node;
    for part in path.split('/') {
        cur = cur.children.entry(part.to_string()).or_default();
    }
    cur.leaf = Some(id);
}

fn to_items(prefix: &str, node: &Node, meta: &mut HashMap<SharedString, Meta>) -> Vec<TreeItem> {
    let mut folders = Vec::new();
    let mut leaves = Vec::new();
    for (name, child) in &node.children {
        if child.children.is_empty() {
            if let Some(id) = &child.leaf {
                leaves.push(TreeItem::new(id.clone(), name.clone()));
            }
        } else {
            let id = format!("{prefix}/{name}");
            let fid: SharedString = format!("dir:{id}").into();
            meta.insert(fid.clone(), Meta::folder());
            let mut kids = to_items(&id, child, meta);
            if let Some(id) = &child.leaf {
                kids.insert(0, TreeItem::new(id.clone(), name.clone()));
            }
            folders.push(
                TreeItem::new(fid, name.clone())
                    .expanded(true)
                    .children(kids),
            );
        }
    }
    folders.extend(leaves);
    folders
}

/// The tree items of each section and how many entries each holds, in [`Kind::ALL`] order.
pub struct SectionItems {
    pub items: [Vec<TreeItem>; 5],
    pub counts: [usize; 5],
    pub meta: HashMap<SharedString, Meta>,
}

pub fn build(refs: &[Ref], stashes: &[Stash], worktrees: &[Worktree]) -> SectionItems {
    let mut meta: HashMap<SharedString, Meta> = HashMap::new();
    let mut local = Node::default();
    let mut remote: BTreeMap<String, Node> = BTreeMap::new();
    let mut tags = Node::default();
    let (mut n_local, mut n_remote, mut n_tags) = (0, 0, 0);
    for r in refs {
        match &r.kind {
            RefKind::LocalBranch { name } => {
                let id: SharedString = format!("L:{name}").into();
                meta.insert(id.clone(), Meta::leaf(r.is_head, false, Some(r.target)));
                insert(&mut local, name, id);
                n_local += 1;
            }
            RefKind::RemoteBranch { remote: rm, name } => {
                let id: SharedString = format!("R:{rm}/{name}").into();
                meta.insert(id.clone(), Meta::leaf(false, false, Some(r.target)));
                insert(remote.entry(rm.clone()).or_default(), name, id);
                n_remote += 1;
            }
            RefKind::Tag { name } => {
                let id: SharedString = format!("T:{name}").into();
                meta.insert(
                    id.clone(),
                    Meta::leaf(false, r.tag_where().is_some(), Some(r.target)),
                );
                insert(&mut tags, name, id);
                n_tags += 1;
            }
        }
    }
    let local_items = to_items("L", &local, &mut meta);
    let remote_items: Vec<TreeItem> = remote
        .iter()
        .map(|(rm, node)| {
            let rid: SharedString = format!("remote:{rm}").into();
            meta.insert(rid.clone(), Meta::folder());
            TreeItem::new(rid, rm.clone())
                .expanded(true)
                .children(to_items(&format!("R/{rm}"), node, &mut meta))
        })
        .collect();
    let tag_items = to_items("T", &tags, &mut meta);
    let stash_items: Vec<TreeItem> = stashes
        .iter()
        .map(|s| {
            let id: SharedString = format!("S:{}", s.id.0).into();
            meta.insert(id.clone(), Meta::leaf(false, false, s.commit));
            TreeItem::new(
                id,
                format!("{}  {}", strings::stash_name(s.id.0), s.message),
            )
        })
        .collect();
    let worktree_items: Vec<TreeItem> = worktrees
        .iter()
        .map(|w| {
            let id: SharedString = format!("W:{}", w.path.display()).into();
            meta.insert(id.clone(), Meta::leaf(false, false, None));
            let name = w
                .branch
                .clone()
                .unwrap_or_else(|| msg::sidebar::DETACHED.get().to_string());
            let label = if w.is_main {
                format!("{name}  {}", msg::sidebar::WORKTREE_MAIN.get())
            } else {
                w.path
                    .file_name()
                    .map(|f| f.to_string_lossy().into_owned())
                    .unwrap_or(name)
            };
            TreeItem::new(id, label)
        })
        .collect();
    SectionItems {
        items: [
            local_items,
            remote_items,
            worktree_items,
            stash_items,
            tag_items,
        ],
        counts: [n_local, n_remote, worktrees.len(), stashes.len(), n_tags],
        meta,
    }
}

/// Keeps the items whose label contains `q` (lowercase), with the folders on the way to them.
pub fn filter_items(items: &[TreeItem], q: &str) -> Vec<TreeItem> {
    items
        .iter()
        .filter_map(|it| {
            if it.children.is_empty() {
                it.label.to_lowercase().contains(q).then(|| it.clone())
            } else {
                let kids = filter_items(&it.children, q);
                (!kids.is_empty()).then(|| {
                    // `TreeItem::children` appends, so the matches replace the list by assignment.
                    let mut folder = it.clone().expanded(true);
                    folder.children = kids;
                    folder
                })
            }
        })
        .collect()
}

/// Rows the tree shows: every item plus the children of the expanded ones.
pub fn count_rows(items: &[TreeItem]) -> usize {
    items
        .iter()
        .map(|i| {
            1 + if i.is_expanded() {
                count_rows(&i.children)
            } else {
                0
            }
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leaf_node(paths: &[&str]) -> Vec<TreeItem> {
        let mut root = Node::default();
        let mut meta = HashMap::new();
        for p in paths {
            insert(&mut root, p, format!("L:{p}").into());
        }
        to_items("L", &root, &mut meta)
    }

    #[test]
    fn slashes_become_folders_and_folders_come_first() {
        let items = leaf_node(&["main", "feature/a", "feature/b", "zeta"]);
        let labels: Vec<_> = items.iter().map(|i| i.label.to_string()).collect();
        assert_eq!(labels, vec!["feature", "main", "zeta"]);
        assert_eq!(items[0].children.len(), 2);
    }

    #[test]
    fn filter_keeps_the_path_to_a_match_only() {
        let items = leaf_node(&["main", "feature/alpha", "feature/beta"]);
        let kept = filter_items(&items, "alp");
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].children.len(), 1);
        assert_eq!(kept[0].children[0].label.to_string(), "alpha");
        assert!(filter_items(&items, "nothing").is_empty());
    }

    #[test]
    fn rows_count_expanded_folders() {
        let items = leaf_node(&["main", "feature/a", "feature/b"]);
        // folder + its two children + main
        assert_eq!(count_rows(&items), 4);
    }
}
