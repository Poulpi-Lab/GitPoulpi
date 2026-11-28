//! The Refs sidebar's data: the sections, their rows and counts, the filter and the hide / solo
//! bookkeeping. The rows come from the shared `gitpoulpi_app::refs_tree` (the terminal UI draws the
//! same ones); what is desktop-specific is only how a row is drawn.

use std::collections::{BTreeSet, HashMap, HashSet};

use gitpoulpi_app::palette::vocab::refs_sidebar::SidebarSection;
use gitpoulpi_app::refs_sections::{linked_worktree_count, visible_sections_with};
use gitpoulpi_app::refs_tree::{
    folder_fully_in, group_refs, hidden_in_graph, hidden_through_remote, ref_display_rows,
    ref_full_name, refs_under_folder, remote_hidden, CollapsedFolders, RefRowKind, RefTreeRow,
};
use gitpoulpi_app::tag_where::{snapshot_of, TagInfo};
use gitpoulpi_core::domain::{
    Oid, Ref, RefKind, Stash, Submodule, SubmoduleState, TagWhere, Worktree,
};
use gitpoulpi_i18n::msg;

use crate::strings;

/// The sections of the pane the desktop shows. Their order and titles are the shared
/// [`SidebarSection`]'s; PULL REQUESTS and ISSUES are not here until the hosting state is.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Kind {
    Local,
    Remote,
    Worktrees,
    Stashes,
    Tags,
    Submodules,
}

impl Kind {
    /// The shared order (`SidebarSection::ORDER`) without the sections the desktop has no data for.
    pub const ALL: [Kind; 6] = [
        Kind::Local,
        Kind::Remote,
        Kind::Worktrees,
        Kind::Stashes,
        Kind::Tags,
        Kind::Submodules,
    ];

    /// The shared section this one stands for.
    pub fn section(self) -> SidebarSection {
        match self {
            Kind::Local => SidebarSection::Local,
            Kind::Remote => SidebarSection::Remote,
            Kind::Worktrees => SidebarSection::Worktrees,
            Kind::Stashes => SidebarSection::Stashes,
            Kind::Tags => SidebarSection::Tags,
            Kind::Submodules => SidebarSection::Submodules,
        }
    }

    /// The desktop's section for a shared one, `None` for those it does not show.
    pub fn of_section(section: SidebarSection) -> Option<Kind> {
        Kind::ALL.into_iter().find(|k| k.section() == section)
    }

    pub fn title(self) -> &'static str {
        self.section().display_title()
    }

    /// The ref kind a section lists; `None` for the sections that hold no refs.
    pub fn ref_kind(self) -> Option<RefRowKind> {
        self.section().ref_kind()
    }
}

/// What a row stands for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RowKind {
    /// A `/`-folder, or a remote's own node (`is_remote_root`).
    Folder {
        path: String,
        is_remote_root: bool,
        expanded: bool,
    },
    Ref {
        full_name: String,
        target: Option<Oid>,
        is_head: bool,
        /// For a tag: where it lives (here, on a remote, both, at different commits).
        tag_where: Option<TagWhere>,
    },
    Stash {
        target: Option<Oid>,
    },
    Worktree,
    Submodule,
}

/// The glyph the terminal UI puts after a tag, by where the tag lives.
pub fn tag_glyph(w: TagWhere) -> &'static str {
    match w {
        TagWhere::Local => "\u{2B06}",
        TagWhere::Remote => "\u{2B07}",
        TagWhere::Both => "\u{2194}",
        TagWhere::Differs => "\u{21C5}",
    }
}

/// One drawn row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    /// Identity that survives a rebuild (the selection is kept by it).
    pub key: String,
    pub label: String,
    pub depth: u8,
    pub kind: RowKind,
}

impl Row {
    /// The commit this row points the Log at.
    pub fn target(&self) -> Option<Oid> {
        match &self.kind {
            RowKind::Ref { target, .. } | RowKind::Stash { target } => *target,
            _ => None,
        }
    }
}

/// One section: its rows as drawn (folds applied) and its header count.
#[derive(Clone, Debug)]
pub struct Section {
    pub kind: Kind,
    /// The header's count: the section's refs after the filter, its stashes or worktrees.
    pub count: usize,
    /// Full names of the section's refs after the filter, sorted: what a folder's bulk hide / solo covers.
    pub names: Vec<String>,
    pub rows: Vec<Row>,
}

pub struct Inputs<'a> {
    pub refs: &'a [Ref],
    pub stashes: &'a [Stash],
    pub worktrees: &'a [Worktree],
    pub submodules: &'a [Submodule],
    /// The sections the user hid (`ui.sidebar_sections_hidden`).
    pub hidden_sections: &'a HashSet<SidebarSection>,
    /// The sidebar filter, matched case-insensitively against full names; empty keeps everything.
    pub filter: &'a str,
    /// `ui.ref_tree`: split names on `/` into folders.
    pub hierarchical: bool,
    pub collapsed: &'a CollapsedFolders,
}

fn row_kind_of(r: &Ref) -> RefRowKind {
    match &r.kind {
        RefKind::LocalBranch { .. } => RefRowKind::Local,
        RefKind::RemoteBranch { .. } => RefRowKind::Remote,
        RefKind::Tag { .. } => RefRowKind::Tag,
    }
}

/// The remotes the refs belong to, for splitting a remote branch's name.
pub fn remote_names(refs: &[Ref]) -> Vec<String> {
    let set: BTreeSet<&str> = refs
        .iter()
        .filter_map(|r| match &r.kind {
            RefKind::RemoteBranch { remote, .. } => Some(remote.as_str()),
            _ => None,
        })
        .collect();
    set.into_iter().map(str::to_string).collect()
}

/// The sections shown, in the shared order and by the shared rule (`refs_sections`): LOCAL, REMOTE and
/// TAGS always; WORKTREES once a linked worktree exists; STASHES when there is a stash; SUBMODULES when
/// there is one; minus what the user hid.
pub fn build_sections(input: &Inputs) -> Vec<Section> {
    let remotes = remote_names(input.refs);
    let groups = group_refs(input.refs, input.filter);
    let tags = snapshot_of(input.refs);
    let info: HashMap<(RefRowKind, String), &Ref> = input
        .refs
        .iter()
        .map(|r| ((row_kind_of(r), ref_full_name(r)), r))
        .collect();
    let shown = visible_sections_with(
        linked_worktree_count(input.worktrees),
        input.stashes.len(),
        (false, false),
        input.submodules.len(),
        input.hidden_sections,
    );
    let mut out = Vec::new();
    for kind in shown.into_iter().filter_map(Kind::of_section) {
        let (rows, names): (Vec<Row>, Vec<String>) = match kind {
            Kind::Worktrees => (
                input.worktrees.iter().map(worktree_row).collect(),
                Vec::new(),
            ),
            Kind::Stashes => (input.stashes.iter().map(stash_row).collect(), Vec::new()),
            Kind::Submodules => (
                input.submodules.iter().map(submodule_row).collect(),
                Vec::new(),
            ),
            _ => {
                let rk = kind.ref_kind().expect("a ref section");
                let list = groups.names(rk).to_vec();
                let rows =
                    ref_display_rows(&list, rk, input.hierarchical, input.collapsed, &remotes)
                        .into_iter()
                        .map(|row| ref_row(row, rk, &info, &tags, input.collapsed))
                        .collect();
                (rows, list)
            }
        };
        let count = if kind.ref_kind().is_some() {
            names.len()
        } else {
            rows.len()
        };
        out.push(Section {
            kind,
            count,
            names,
            rows,
        });
    }
    out
}

fn ref_row(
    row: RefTreeRow,
    kind: RefRowKind,
    info: &HashMap<(RefRowKind, String), &Ref>,
    tags: &HashMap<String, TagInfo>,
    collapsed: &CollapsedFolders,
) -> Row {
    match row {
        RefTreeRow::Folder {
            path,
            label,
            depth,
            is_remote_root,
        } => {
            let expanded = !collapsed.contains(&(kind, path.clone()));
            Row {
                key: format!("dir:{path}"),
                label,
                depth,
                kind: RowKind::Folder {
                    path,
                    is_remote_root,
                    expanded,
                },
            }
        }
        RefTreeRow::Leaf {
            full_name,
            display_name,
            depth,
        } => {
            let r = info.get(&(kind, full_name.clone())).copied();
            Row {
                key: full_name.clone(),
                label: display_name,
                depth,
                kind: RowKind::Ref {
                    target: r.map(|r| r.target),
                    is_head: r.is_some_and(|r| r.is_head),
                    tag_where: tags.get(&full_name).map(|t| t.place),
                    full_name,
                },
            }
        }
    }
}

fn stash_row(s: &Stash) -> Row {
    Row {
        key: format!("S:{}", s.id.0),
        label: format!("{}  {}", strings::stash_name(s.id.0), s.message),
        depth: 0,
        kind: RowKind::Stash { target: s.commit },
    }
}

fn worktree_row(w: &Worktree) -> Row {
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
    Row {
        key: format!("W:{}", w.path.display()),
        label,
        depth: 0,
        kind: RowKind::Worktree,
    }
}

fn submodule_row(m: &Submodule) -> Row {
    let state = match m.state {
        SubmoduleState::Uninitialized => " (-)",
        SubmoduleState::UpToDate => "",
        SubmoduleState::Modified => " (+)",
        SubmoduleState::Conflicted => " (U)",
    };
    Row {
        key: format!("M:{}", m.path),
        label: format!("{}{state}", m.name),
        depth: 0,
        kind: RowKind::Submodule,
    }
}

/// Every folder path of every ref section, unfolded: what "collapse all" puts in the collapsed set.
pub fn all_folders(refs: &[Ref], hierarchical: bool) -> CollapsedFolders {
    let none = CollapsedFolders::new();
    let sections = build_sections(&Inputs {
        refs,
        stashes: &[],
        worktrees: &[],
        submodules: &[],
        hidden_sections: &HashSet::new(),
        filter: "",
        hierarchical,
        collapsed: &none,
    });
    let mut out = CollapsedFolders::new();
    for s in sections {
        let Some(kind) = s.kind.ref_kind() else {
            continue;
        };
        for row in s.rows {
            if let RowKind::Folder { path, .. } = row.kind {
                out.insert((kind, path));
            }
        }
    }
    out
}

// -- hide and solo ---------------------------------------------------------

/// Whether a row is hidden from the graph (a leaf named in the set or under a hidden remote, a folder
/// whose every ref is hidden, a remote hidden as a whole).
pub fn row_hidden(section: &Section, row: &Row, hidden: &HashSet<String>) -> bool {
    let remote_section = section.kind == Kind::Remote;
    match &row.kind {
        RowKind::Ref { full_name, .. } => hidden_in_graph(hidden, full_name, remote_section),
        RowKind::Folder {
            path,
            is_remote_root,
            ..
        } if section.kind.ref_kind().is_some() => {
            if remote_section && (*is_remote_root && remote_hidden(hidden, path)) {
                return true;
            }
            if remote_section && hidden_through_remote(hidden, path) {
                return true;
            }
            let under = refs_under_folder(None, &section.names, path);
            !under.is_empty()
                && under
                    .iter()
                    .all(|n| hidden_in_graph(hidden, n, remote_section))
        }
        _ => false,
    }
}

/// Whether a row is soloed (a leaf in the set, a folder whose every ref is).
pub fn row_soloed(section: &Section, row: &Row, solo: &HashSet<String>) -> bool {
    match &row.kind {
        RowKind::Ref { full_name, .. } => solo.contains(full_name),
        RowKind::Folder { path, .. } if section.kind.ref_kind().is_some() => {
            folder_fully_in(solo, &refs_under_folder(None, &section.names, path))
        }
        _ => false,
    }
}

/// How many refs are hidden: the set's own entries plus every remote branch a hidden remote covers.
pub fn hidden_count(hidden: &HashSet<String>, remote_names: &[String]) -> usize {
    hidden.iter().filter(|k| !k.ends_with('/')).count()
        + remote_names
            .iter()
            .filter(|n| !hidden.contains(*n) && hidden_through_remote(hidden, n))
            .count()
}

/// The text each section's rows read as, one per line, for the golden tests: a fold marker, the
/// hide / solo state and the label.
#[cfg(test)]
pub fn dump(sections: &[Section], hidden: &HashSet<String>, solo: &HashSet<String>) -> String {
    let mut out = String::new();
    for s in sections {
        out.push_str(&format!("{} ({})\n", s.kind.title(), s.count));
        for row in &s.rows {
            let indent = "  ".repeat(row.depth as usize + 1);
            let mark = match &row.kind {
                RowKind::Folder {
                    expanded,
                    is_remote_root,
                    ..
                } => format!(
                    "{}{}",
                    if *expanded { "v " } else { "> " },
                    if *is_remote_root { "@" } else { "" }
                ),
                RowKind::Ref { is_head: true, .. } => "* ".to_string(),
                _ => "  ".to_string(),
            };
            let state = match (row_hidden(s, row, hidden), row_soloed(s, row, solo)) {
                (_, true) => " [solo]",
                (true, _) => " [hidden]",
                _ => "",
            };
            let tag = match &row.kind {
                RowKind::Ref {
                    tag_where: Some(w), ..
                } => format!(" {}", tag_glyph(*w)),
                _ => String::new(),
            };
            out.push_str(&format!("{indent}{mark}{}{tag}{state}\n", row.label));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::{RefVisibility, StashId, TagSync};

    fn oid(n: u8) -> Oid {
        Oid([n; 20])
    }

    fn mk(kind: RefKind, n: u8, is_head: bool) -> Ref {
        Ref {
            kind,
            target: oid(n),
            visibility: RefVisibility::Shown,
            is_head,
            upstream: None,
            tag_sync: None,
            multi_remote: None,
        }
    }

    fn local(name: &str, n: u8, head: bool) -> Ref {
        mk(RefKind::LocalBranch { name: name.into() }, n, head)
    }

    fn remote(remote: &str, name: &str, n: u8) -> Ref {
        mk(
            RefKind::RemoteBranch {
                remote: remote.into(),
                name: name.into(),
            },
            n,
            false,
        )
    }

    fn tag(name: &str, n: u8) -> Ref {
        mk(RefKind::Tag { name: name.into() }, n, false)
    }

    /// The fixture of the golden tests.
    fn refs() -> Vec<Ref> {
        vec![
            local("main", 1, true),
            local("feature/login", 2, false),
            local("feature/signup", 3, false),
            local("zeta", 4, false),
            remote("origin", "main", 1),
            remote("origin", "feature/login", 2),
            remote("grp/sub", "dev", 5),
            tag("releases/v1.0", 1),
            tag("v0.9", 6),
        ]
    }

    fn stash(id: u32, message: &str) -> Stash {
        Stash {
            id: StashId(id),
            message: message.into(),
            branch_at_creation: "main".into(),
            timestamp: 0,
            commit: Some(oid(7)),
            base: Some(oid(1)),
        }
    }

    fn build(
        refs: &[Ref],
        stashes: &[Stash],
        filter: &str,
        collapsed: &CollapsedFolders,
    ) -> Vec<Section> {
        build_sections(&Inputs {
            refs,
            stashes,
            worktrees: &[],
            submodules: &[],
            hidden_sections: &HashSet::new(),
            filter,
            hierarchical: true,
            collapsed,
        })
    }

    const GOLDEN_FULL: &str = "\
LOCAL (4)
  v feature
      login
      signup
  * main
    zeta
REMOTE (3)
  v @grp/sub
      dev
  v @origin
    v feature
        login
      main
STASHES (1)
    stash@{0}  wip
TAGS (2)
  v releases
      v1.0
    v0.9
";

    fn none() -> HashSet<String> {
        HashSet::new()
    }

    #[test]
    fn the_sections_rows_and_counts_match_the_terminal_tree() {
        let sections = build(&refs(), &[stash(0, "wip")], "", &CollapsedFolders::new());
        let kinds: Vec<Kind> = sections.iter().map(|s| s.kind).collect();
        assert_eq!(
            kinds,
            vec![Kind::Local, Kind::Remote, Kind::Stashes, Kind::Tags],
            "worktrees only once a linked one exists, stashes only when there is one"
        );
        assert_eq!(dump(&sections, &none(), &none()), GOLDEN_FULL);
    }

    #[test]
    fn a_collapsed_folder_hides_its_children_and_keeps_its_count() {
        let mut collapsed = CollapsedFolders::new();
        collapsed.insert((RefRowKind::Local, "feature".into()));
        let sections = build(&refs(), &[], "", &collapsed);
        let local = &sections[0];
        assert_eq!(
            local.count, 4,
            "the header counts refs, folds do not change it"
        );
        let labels: Vec<&str> = local.rows.iter().map(|r| r.label.as_str()).collect();
        assert_eq!(labels, vec!["feature", "main", "zeta"]);
        assert!(matches!(
            local.rows[0].kind,
            RowKind::Folder {
                expanded: false,
                ..
            }
        ));
    }

    #[test]
    fn the_filter_matches_the_full_name_case_insensitively_and_keeps_the_way_to_a_match() {
        let sections = build(
            &refs(),
            &[stash(0, "wip")],
            "login",
            &CollapsedFolders::new(),
        );
        assert_eq!(
            dump(&sections, &none(), &none()),
            "\
LOCAL (1)
  v feature
      login
REMOTE (1)
  v @origin
    v feature
        login
STASHES (1)
    stash@{0}  wip
TAGS (0)
"
        );
        let sections = build(&refs(), &[], "ORIGIN/m", &CollapsedFolders::new());
        assert_eq!(sections[1].names, vec!["origin/main"]);
    }

    #[test]
    fn hidden_and_soloed_marks_follow_the_shared_rules() {
        let sections = build(&refs(), &[], "", &CollapsedFolders::new());
        let hidden: HashSet<String> = ["feature/login", "feature/signup", "origin/", "zeta"]
            .into_iter()
            .map(str::to_string)
            .collect();
        let solo: HashSet<String> = ["main".to_string()].into_iter().collect();
        let text = dump(&sections, &hidden, &solo);
        assert!(text.contains("  v feature [hidden]\n"), "{text}");
        assert!(text.contains("  * main [solo]\n"), "{text}");
        assert!(text.contains("  zeta [hidden]\n"), "{text}");
        assert!(
            text.contains("  v @origin [hidden]\n"),
            "a hidden remote hides its node: {text}"
        );
        assert!(
            text.contains("      login [hidden]\n"),
            "and its branches: {text}"
        );
        assert!(!text.contains("@grp/sub [hidden]"));
    }

    #[test]
    fn a_folders_refs_are_the_names_beneath_it() {
        let sections = build(&refs(), &[], "", &CollapsedFolders::new());
        let under = refs_under_folder(None, &sections[0].names, "feature");
        assert_eq!(under, vec!["feature/login", "feature/signup"]);
    }

    #[test]
    fn a_remote_named_with_a_slash_is_one_node() {
        let sections = build(&refs(), &[], "", &CollapsedFolders::new());
        let remotes: Vec<&Row> = sections[1]
            .rows
            .iter()
            .filter(|r| {
                matches!(
                    r.kind,
                    RowKind::Folder {
                        is_remote_root: true,
                        ..
                    }
                )
            })
            .collect();
        let labels: Vec<&str> = remotes.iter().map(|r| r.label.as_str()).collect();
        assert_eq!(labels, vec!["grp/sub", "origin"]);
    }

    #[test]
    fn tags_carry_the_terminal_uis_glyph_for_where_they_live() {
        let sync = |pushed: &[&str], differs: &[&str], local: bool| {
            Some(TagSync {
                pushed_to: pushed.iter().map(|s| s.to_string()).collect(),
                differs_on: differs.iter().map(|s| s.to_string()).collect(),
                local,
            })
        };
        let mut local_only = tag("a-local", 1);
        local_only.tag_sync = sync(&[], &[], true);
        let mut both = tag("b-both", 2);
        both.tag_sync = sync(&["origin"], &[], true);
        let mut remote_only = tag("c-remote", 3);
        remote_only.tag_sync = sync(&["origin"], &[], false);
        let mut differs = tag("d-differs", 4);
        differs.tag_sync = sync(&[], &["origin"], true);
        let unknown = tag("e-unknown", 5);
        let sections = build(
            &[local_only, both, remote_only, differs, unknown],
            &[],
            "",
            &CollapsedFolders::new(),
        );
        assert_eq!(
            dump(&sections, &none(), &none()),
            "LOCAL (0)\nREMOTE (0)\nTAGS (5)\n    a-local \u{2B06}\n    b-both \u{2194}\n    c-remote \u{2B07}\n    d-differs \u{21C5}\n    e-unknown\n"
        );
    }

    #[test]
    fn a_hidden_remote_counts_every_branch_it_covers() {
        let h: HashSet<String> = ["origin/".to_string(), "zeta".to_string()]
            .into_iter()
            .collect();
        let remotes = vec![
            "origin/main".to_string(),
            "origin/dev".to_string(),
            "up/x".to_string(),
        ];
        assert_eq!(hidden_count(&h, &remotes), 3);
    }

    fn submodule(name: &str, state: SubmoduleState) -> Submodule {
        Submodule {
            name: name.into(),
            path: name.into(),
            url: String::new(),
            branch: None,
            state,
            expected: None,
            head: None,
        }
    }

    #[test]
    fn worktrees_and_submodules_follow_the_shared_rule_and_order() {
        let none = HashSet::new();
        let subs = [
            submodule("vendor/lib", SubmoduleState::UpToDate),
            submodule("tools", SubmoduleState::Modified),
        ];
        let sections = build_sections(&Inputs {
            refs: &refs(),
            stashes: &[stash(0, "wip")],
            worktrees: &[],
            submodules: &subs,
            hidden_sections: &none,
            filter: "",
            hierarchical: true,
            collapsed: &CollapsedFolders::new(),
        });
        let kinds: Vec<Kind> = sections.iter().map(|s| s.kind).collect();
        assert_eq!(
            kinds,
            vec![
                Kind::Local,
                Kind::Remote,
                Kind::Stashes,
                Kind::Tags,
                Kind::Submodules
            ],
            "SUBMODULES comes last, as in the terminal pane"
        );
        let last = sections.last().unwrap();
        assert_eq!(last.count, 2);
        let labels: Vec<&str> = last.rows.iter().map(|r| r.label.as_str()).collect();
        assert_eq!(labels, vec!["vendor/lib", "tools (+)"]);
        // What the user hid stays out.
        let hidden: HashSet<SidebarSection> = [SidebarSection::Stashes].into_iter().collect();
        let sections = build_sections(&Inputs {
            refs: &refs(),
            stashes: &[stash(0, "wip")],
            worktrees: &[],
            submodules: &subs,
            hidden_sections: &hidden,
            filter: "",
            hierarchical: true,
            collapsed: &CollapsedFolders::new(),
        });
        assert!(sections.iter().all(|s| s.kind != Kind::Stashes));
    }

    #[test]
    fn collapse_all_names_every_folder() {
        let all = all_folders(&refs(), true);
        assert!(all.contains(&(RefRowKind::Local, "feature".into())));
        assert!(all.contains(&(RefRowKind::Remote, "origin".into())));
        assert!(all.contains(&(RefRowKind::Remote, "origin/feature".into())));
        assert!(all.contains(&(RefRowKind::Tag, "releases".into())));
        assert_eq!(all.len(), 5, "{all:?}");
    }
}
