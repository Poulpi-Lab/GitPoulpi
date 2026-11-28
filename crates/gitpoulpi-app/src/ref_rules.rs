//! The rules about refs a frontend shares: what a new branch or tag may be
//! called, the twins of a branch (its local and remote halves), what a Delete
//! row acts on, whether moving a branch would fast-forward and what a Merge
//! row says, and the scopes of a tag delete.
//!
//! All of it reads plain `Ref` lists; the texts come from `gitpoulpi-i18n`.
//! A frontend calls these where it builds a menu row, a form validation or a
//! confirmation, so the terminal UI and the desktop never disagree.

use crate::refs_tree::RefRowKind;
use gitpoulpi_core::domain::{
    CommitGraph, CommitNode, MergeFastForward, Oid, Ref, RefKind, Stash, StashId,
};
use gitpoulpi_i18n::msg::{menu_inline as inline, menu_ref};

/// What a name box is naming.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewRef {
    Branch,
    Tag,
}

/// `git check-ref-format`'s rules, as far as a short human message goes.
pub fn check_ref_format(name: &str) -> Result<(), String> {
    if name == "@" {
        return Err(inline::NAME_AT.get().into());
    }
    if name.starts_with('/') || name.ends_with('/') || name.contains("//") {
        return Err(inline::NAME_SLASH.get().into());
    }
    if name.ends_with('.') {
        return Err(inline::NAME_DOT_END.get().into());
    }
    if name.contains("..") {
        return Err(inline::NAME_DOTDOT.get().into());
    }
    if name.contains("@{") {
        return Err(inline::NAME_AT_BRACE.get().into());
    }
    if name.starts_with('-') {
        return Err(inline::NAME_DASH_START.get().into());
    }
    if let Some(bad) = name
        .chars()
        .find(|c| c.is_whitespace() || c.is_control() || "~^:?*[\\".contains(*c))
    {
        return Err(if bad.is_whitespace() {
            inline::NAME_SPACES.get().to_string()
        } else {
            inline::name_bad_char(bad.escape_default())
        });
    }
    for part in name.split('/') {
        if part.starts_with('.') {
            return Err(inline::NAME_PART_DOT.get().into());
        }
        if part.ends_with(".lock") {
            return Err(inline::NAME_PART_LOCK.get().into());
        }
    }
    Ok(())
}

/// The trimmed name if it can be created, otherwise the message to show next
/// to the field. Checks git's format rules, an existing ref of the same kind,
/// and a directory/file clash (`a` next to `a/b`), which git also refuses.
pub fn validate_new_ref(kind: NewRef, raw: &str, refs: &[Ref]) -> Result<String, String> {
    let name = raw.trim();
    if name.is_empty() {
        return Err(match kind {
            NewRef::Branch => inline::ENTER_BRANCH.get().to_string(),
            NewRef::Tag => inline::ENTER_TAG.get().to_string(),
        });
    }
    check_ref_format(name)?;
    let existing = refs.iter().filter_map(|r| match (&r.kind, kind) {
        (RefKind::LocalBranch { name }, NewRef::Branch) => Some(name.as_str()),
        (RefKind::Tag { name }, NewRef::Tag) => Some(name.as_str()),
        _ => None,
    });
    for other in existing {
        if other == name {
            return Err(match kind {
                NewRef::Branch => inline::exists_branch(name),
                NewRef::Tag => inline::exists_tag(name),
            });
        }
        if other.starts_with(&format!("{name}/")) || name.starts_with(&format!("{other}/")) {
            return Err(inline::clashes(name, other));
        }
    }
    Ok(name.to_string())
}

/// The refs a rename must not collide with: every ref but the branch being
/// renamed (so `a` may become `a2`, and `a/b` may take `a`'s place). Pass the
/// result to [`validate_new_ref`].
pub fn refs_without_branch(refs: &[Ref], old: &str) -> Vec<Ref> {
    refs.iter()
        .filter(|r| !matches!(&r.kind, RefKind::LocalBranch { name } if name == old))
        .cloned()
        .collect()
}

/// The `Ref` a sidebar row names, matched on the displayed name (`origin/x`
/// for a remote branch).
pub fn find_ref<'a>(refs: &'a [Ref], kind: RefRowKind, name: &str) -> Option<&'a Ref> {
    refs.iter().find(|r| match (&r.kind, kind) {
        (RefKind::LocalBranch { name: n }, RefRowKind::Local) => n == name,
        (RefKind::RemoteBranch { remote, name: n }, RefRowKind::Remote) => {
            format!("{remote}/{n}") == name
        }
        (RefKind::Tag { name: n }, RefRowKind::Tag) => n == name,
        _ => false,
    })
}

/// `(remote, branch)` of the remote-tracking ref shown as `display`
/// (`origin/feature/x`), or `None` when no such ref is loaded.
pub fn remote_branch_parts(refs: &[Ref], display: &str) -> Option<(String, String)> {
    refs.iter().find_map(|r| match &r.kind {
        RefKind::RemoteBranch { remote, name } if format!("{remote}/{name}") == display => {
            Some((remote.clone(), name.clone()))
        }
        _ => None,
    })
}

/// A branch's counterpart on the other side, as sidebar names:
/// `(local_twin, remote_twin)`. For a local branch the remote twin is its
/// upstream when that ref is loaded, else a remote branch of the same
/// name; for a remote branch the local twin is the local branch tracking
/// it, else one of the same name. Tags have neither.
pub fn branch_twins(
    refs: &[Ref],
    kind: RefRowKind,
    full_name: &str,
) -> (Option<String>, Option<String>) {
    match kind {
        RefRowKind::Local => {
            let upstream = refs.iter().find_map(|r| match &r.kind {
                RefKind::LocalBranch { name } if name == full_name => r.upstream.clone(),
                _ => None,
            });
            let remote = upstream
                .filter(|u| remote_branch_parts(refs, u).is_some())
                .or_else(|| {
                    refs.iter().find_map(|r| match &r.kind {
                        RefKind::RemoteBranch { remote, name }
                            if name == full_name && name != "HEAD" =>
                        {
                            Some(format!("{remote}/{name}"))
                        }
                        _ => None,
                    })
                });
            (None, remote)
        }
        RefRowKind::Remote => {
            let Some((_, branch)) = remote_branch_parts(refs, full_name) else {
                return (None, None);
            };
            let tracking = refs.iter().find_map(|r| match &r.kind {
                RefKind::LocalBranch { name } if r.upstream.as_deref() == Some(full_name) => {
                    Some(name.clone())
                }
                _ => None,
            });
            let local = tracking.or_else(|| {
                refs.iter().find_map(|r| match &r.kind {
                    RefKind::LocalBranch { name } if *name == branch => Some(name.clone()),
                    _ => None,
                })
            });
            (local, None)
        }
        RefRowKind::Tag => (None, None),
    }
}

/// What a Delete row acts on: the local branch and/or the remote one.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DeleteTargets {
    pub local: Option<String>,
    /// `(remote, branch)`.
    pub remote: Option<(String, String)>,
}

/// The delete targets of the branch row `full_name`: itself plus its twin
/// on the other side (a tag has none).
pub fn delete_targets(refs: &[Ref], kind: RefRowKind, full_name: &str) -> DeleteTargets {
    let (local_twin, remote_twin) = branch_twins(refs, kind, full_name);
    match kind {
        RefRowKind::Local => DeleteTargets {
            local: Some(full_name.to_string()),
            remote: remote_twin.and_then(|t| remote_branch_parts(refs, &t)),
        },
        RefRowKind::Remote => DeleteTargets {
            local: local_twin,
            remote: remote_branch_parts(refs, full_name),
        },
        RefRowKind::Tag => DeleteTargets::default(),
    }
}

/// The refusal for deleting the checked-out branch, or `None`.
pub fn checked_out_refusal(refs: &[Ref], branch: &str) -> Option<String> {
    refs.iter()
        .any(|r| r.is_head && matches!(&r.kind, RefKind::LocalBranch { name } if name == branch))
        .then(|| menu_ref::cannot_delete_checked_out(branch))
}

/// The remotes that have `tag`: moving or deleting it leaves them at the
/// old position.
pub fn tag_remotes(tag: &Ref) -> Vec<String> {
    tag.tag_sync
        .as_ref()
        .map(|s| {
            s.pushed_to
                .iter()
                .chain(s.differs_on.iter())
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}

/// Which delete a row of the "delete where?" choice stands for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagDeleteScope {
    Local,
    Remote(String),
    Both(String),
}

/// The rows of the tag-delete choice, in order: local first, then per remote
/// its remote-only and its both-places delete. A tag with no local ref has no
/// local rows.
pub fn tag_delete_options(has_local: bool, remotes: &[String]) -> Vec<TagDeleteScope> {
    let mut out = Vec::new();
    if has_local {
        out.push(TagDeleteScope::Local);
    }
    for r in remotes {
        out.push(TagDeleteScope::Remote(r.clone()));
        if has_local {
            out.push(TagDeleteScope::Both(r.clone()));
        }
    }
    out
}

/// Whether moving one branch onto another commit would be a
/// fast-forward, as far as the loaded graph can tell.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FfVerdict {
    /// Not answerable from the loaded pages (or not asked): the row is
    /// offered and `GitBackend::fast_forward` has the last word.
    #[default]
    Unknown,
    Possible,
    /// The two are the same commit.
    AlreadyThere,
    /// The branch to move already contains the target.
    Ahead,
    Diverged,
}

impl FfVerdict {
    /// Why the row is grayed out, or `None` when it is offered.
    pub fn refusal(self, moving: &str) -> Option<String> {
        match self {
            FfVerdict::Unknown | FfVerdict::Possible => None,
            FfVerdict::AlreadyThere => Some(menu_ref::FF_ALREADY_THERE.get().into()),
            FfVerdict::Ahead => Some(menu_ref::ff_already_ahead(moving)),
            FfVerdict::Diverged => Some(menu_ref::FF_DIVERGED.get().into()),
        }
    }
}

/// What a Merge row says it will do, from the fast-forward policy and
/// whether the merge could fast-forward at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeLabel {
    /// Nothing to add: the result is a merge commit (or unknown).
    Plain,
    /// A fast-forward is possible but the policy makes a merge commit.
    NewCommit,
    /// A fast-forward is possible and the policy takes it.
    FastForwards,
    /// The policy only allows a fast-forward.
    FastForwardOnly,
}

impl MergeLabel {
    /// The wording class of a Merge row under `policy`, given the verdict of
    /// the fast-forward between the same two refs.
    pub fn of(policy: MergeFastForward, verdict: FfVerdict) -> Self {
        use MergeFastForward as P;
        let possible = verdict == FfVerdict::Possible;
        match (policy, possible) {
            (P::Never, true) => Self::NewCommit,
            (P::Allowed, true) => Self::FastForwards,
            (P::Only, _) => Self::FastForwardOnly,
            _ => Self::Plain,
        }
    }
}

/// Is moving a branch at `moving` to `target` a fast-forward? Walks
/// parents from `target` through the loaded graph only; a walk that
/// leaves the loaded pages answers `Unknown` rather than guessing.
pub fn ff_verdict(graph: &CommitGraph, moving: Oid, target: Oid) -> FfVerdict {
    if moving == target {
        return FfVerdict::AlreadyThere;
    }
    let by_id: std::collections::HashMap<Oid, &CommitNode> =
        graph.nodes.iter().map(|n| (n.id, n)).collect();
    // `Some(true)` reached, `Some(false)` exhausted inside the graph,
    // `None` walked off the loaded pages.
    let reaches = |from: Oid, wanted: Oid| -> Option<bool> {
        let mut seen = std::collections::HashSet::new();
        let mut stack = vec![from];
        let mut complete = true;
        while let Some(id) = stack.pop() {
            if id == wanted {
                return Some(true);
            }
            if !seen.insert(id) {
                continue;
            }
            match by_id.get(&id) {
                Some(node) => stack.extend(node.parents.iter().copied()),
                None => complete = false,
            }
        }
        complete.then_some(false)
    };
    match (reaches(target, moving), reaches(moving, target)) {
        (Some(true), _) => FfVerdict::Possible,
        (_, Some(true)) => FfVerdict::Ahead,
        (Some(false), Some(false)) => FfVerdict::Diverged,
        _ => FfVerdict::Unknown,
    }
}

/// The commit a ref created "here" points at: the checked-out branch's tip.
/// `None` on a detached or unborn HEAD.
pub fn head_oid(refs: &[Ref]) -> Option<Oid> {
    refs.iter()
        .find(|r| r.is_head && matches!(r.kind, RefKind::LocalBranch { .. }))
        .map(|r| r.target)
}

/// Why `name` is not a valid branch name, if it is not - the rules of
/// `git check-ref-format --branch` a user can actually break by typing.
pub fn branch_name_error(name: &str) -> Option<String> {
    let bad = name.is_empty()
        || name == "@"
        || name.starts_with('-')
        || name.starts_with('/')
        || name.ends_with('/')
        || name.ends_with('.')
        || name.ends_with(".lock")
        || name.contains("..")
        || name.contains("//")
        || name.contains("@{")
        || name.split('/').any(|part| part.starts_with('.'))
        || name
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || "~^:?*[\\".contains(c));
    bad.then(|| gitpoulpi_i18n::msg::init::bad_branch(format!("{name:?}")))
}

/// Which stash a "stash pop/apply/drop" command acts on: the one selected
/// in the stash list (`selected` is its index) when the list has the focus,
/// otherwise the most recent (`stash@{0}`).
pub fn default_stash(stashes: &[Stash], selected: Option<usize>) -> Option<&Stash> {
    selected
        .and_then(|i| stashes.get(i))
        .or_else(|| stashes.first())
}

/// The stash rows carry their stash id as the context menu's row identity
/// (refs carry their name there instead), so acting on one means parsing it
/// back.
pub fn parse_stash_id(raw: &str) -> Option<StashId> {
    raw.parse().ok().map(StashId)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::RefVisibility;

    fn oid(n: u8) -> Oid {
        Oid([n; 20])
    }

    fn commit(id: u8, parents: &[u8]) -> CommitNode {
        CommitNode {
            id: oid(id),
            parents: parents.iter().map(|&p| oid(p)).collect(),
            summary: String::new(),
            message: String::new(),
            author_name: "a".into(),
            author_email: "a@example.com".into(),
            timestamp: 0,
            refs: Vec::new(),
        }
    }

    fn mk(kind: RefKind, upstream: Option<&str>, is_head: bool) -> Ref {
        Ref {
            kind,
            target: oid(1),
            visibility: RefVisibility::Shown,
            is_head,
            upstream: upstream.map(str::to_string),
            tag_sync: None,
            multi_remote: None,
        }
    }

    fn local(name: &str, upstream: Option<&str>, is_head: bool) -> Ref {
        mk(
            RefKind::LocalBranch { name: name.into() },
            upstream,
            is_head,
        )
    }

    fn remote(remote: &str, name: &str) -> Ref {
        mk(
            RefKind::RemoteBranch {
                remote: remote.into(),
                name: name.into(),
            },
            None,
            false,
        )
    }

    #[test]
    fn names_are_checked_for_format_duplicates_and_clashes() {
        let refs = vec![
            local("main", None, true),
            mk(RefKind::Tag { name: "v1".into() }, None, false),
        ];
        assert_eq!(
            validate_new_ref(NewRef::Branch, " feat/x ", &refs).as_deref(),
            Ok("feat/x")
        );
        for bad in ["", "a b", "a..b", "-x", "x.lock", "a//b", "@"] {
            assert!(
                validate_new_ref(NewRef::Branch, bad, &refs).is_err(),
                "{bad}"
            );
        }
        assert!(validate_new_ref(NewRef::Branch, "main", &refs).is_err());
        assert!(validate_new_ref(NewRef::Branch, "main/x", &refs).is_err());
        assert!(validate_new_ref(NewRef::Tag, "main", &refs).is_ok());
        assert!(validate_new_ref(NewRef::Tag, "v1", &refs).is_err());
        let others = refs_without_branch(&refs, "main");
        assert!(validate_new_ref(NewRef::Branch, "main", &others).is_ok());
    }

    #[test]
    fn twins_and_delete_targets_pair_the_local_and_remote_halves() {
        let refs = vec![
            local("feat", Some("origin/feat"), false),
            remote("origin", "feat"),
            remote("origin", "other"),
        ];
        assert_eq!(
            branch_twins(&refs, RefRowKind::Local, "feat"),
            (None, Some("origin/feat".into()))
        );
        assert_eq!(
            branch_twins(&refs, RefRowKind::Remote, "origin/feat"),
            (Some("feat".into()), None)
        );
        let t = delete_targets(&refs, RefRowKind::Remote, "origin/other");
        assert_eq!(t.local, None);
        assert_eq!(t.remote, Some(("origin".into(), "other".into())));
        assert_eq!(
            delete_targets(&refs, RefRowKind::Tag, "v1"),
            DeleteTargets::default()
        );
        let head = vec![local("main", None, true)];
        assert!(checked_out_refusal(&head, "main").is_some());
        assert!(checked_out_refusal(&head, "other").is_none());
    }

    #[test]
    fn tag_delete_rows_follow_where_the_tag_lives() {
        let on = vec!["origin".to_string()];
        assert_eq!(
            tag_delete_options(true, &on),
            vec![
                TagDeleteScope::Local,
                TagDeleteScope::Remote("origin".into()),
                TagDeleteScope::Both("origin".into())
            ]
        );
        assert_eq!(
            tag_delete_options(false, &on),
            vec![TagDeleteScope::Remote("origin".into())]
        );
    }

    #[test]
    fn the_fast_forward_verdict_reads_the_loaded_graph() {
        // 1 <- 2 <- 3, and 4 off 1.
        let graph = CommitGraph {
            nodes: vec![
                commit(4, &[1]),
                commit(3, &[2]),
                commit(2, &[1]),
                commit(1, &[]),
            ],
        };
        assert_eq!(ff_verdict(&graph, oid(2), oid(3)), FfVerdict::Possible);
        assert_eq!(ff_verdict(&graph, oid(3), oid(2)), FfVerdict::Ahead);
        assert_eq!(ff_verdict(&graph, oid(4), oid(3)), FfVerdict::Diverged);
        assert_eq!(ff_verdict(&graph, oid(2), oid(2)), FfVerdict::AlreadyThere);
        assert_eq!(ff_verdict(&graph, oid(2), oid(9)), FfVerdict::Unknown);
        assert!(FfVerdict::Diverged.refusal("x").is_some());
        assert!(FfVerdict::Possible.refusal("x").is_none());
        assert_eq!(
            MergeLabel::of(MergeFastForward::Never, FfVerdict::Possible),
            MergeLabel::NewCommit
        );
        assert_eq!(
            MergeLabel::of(MergeFastForward::Only, FfVerdict::Unknown),
            MergeLabel::FastForwardOnly
        );
    }
}
