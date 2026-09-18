//! The Log's two combined context menus.
//!
//! - **A ref pill's menu** (right-click a pill in the Ref column, or one
//!   of the rows of the `+N` reveal popup): that ref's entries *and* the
//!   pointed commit's entries in one list, grouped by kind.
//!   When the commit carries other refs, each one gets a
//!   submenu holding its own entries, so a ref hidden behind `+N` is one
//!   drill-in away.
//! - **A commit row's menu**: the commit's entries, plus one submenu per
//!   ref on the commit.
//!
//! Both are `ContextMenu<LogContextAction>`: a ref's row travels as
//! `LogContextAction::Ref(action, i)`, `i` indexing the [`RefTable`] the
//! menu was opened with, which the event loop keeps beside the menu. That
//! keeps the action `Copy` and gives the mouse, the keyboard and the
//! palette one place to run a ref's row (`apply_ref_context_action`).

use super::*;

use context_menu::{glyph_label, Entry};
use gitpoulpi_core::domain::{CommitGraph, Ref, Remote};
use gitpoulpi_i18n::msg::{menu_log, menu_pill as msg};

/// The refs a menu's `LogContextAction::Ref(_, i)` rows point into:
/// `(section, displayed name)`, the chosen ref first.
pub(crate) type RefTable = Vec<(SidebarSection, String)>;

/// A menu and the ref table its rows refer to.
pub(crate) struct PillMenu {
    pub(crate) entries: Vec<Entry<LogContextAction>>,
    pub(crate) table: RefTable,
}

/// Everything a combined menu is built from.
pub(crate) struct MenuSources<'a> {
    pub(crate) refs_result: &'a gitpoulpi_core::Result<Vec<Ref>>,
    pub(crate) log_result: &'a gitpoulpi_core::Result<CommitGraph>,
    pub(crate) remotes: &'a [Remote],
    pub(crate) hidden: &'a std::collections::HashSet<String>,
    pub(crate) solo: &'a std::collections::HashSet<String>,
    pub(crate) unpushed_tags: &'a std::collections::HashSet<String>,
    /// How many commits are multi-selected (see `MenuContext`).
    pub(crate) selected_count: usize,
    /// `ai.enabled`: whether the AI rows are offered.
    pub(crate) ai_enabled: bool,
}

/// A ref's own entries, its rows tagged with table index `idx`.
pub(crate) fn ref_rows(
    src: &MenuSources<'_>,
    section: SidebarSection,
    name: &str,
    idx: u8,
) -> Vec<Entry<LogContextAction>> {
    let ctx = menu_context_for_ref(src.refs_result, src.log_result, section, name, src.remotes);
    let mut entries = ref_menu_entries(
        section,
        name,
        &ctx,
        src.hidden.contains(name),
        src.solo.contains(name),
        src.unpushed_tags.contains(name),
    );
    drop_reveal_row(&mut entries);
    map_entries(entries, idx)
}

/// `entries` with every ref action wrapped as `LogContextAction::Ref(_, idx)`.
pub(crate) fn map_entries(
    entries: Vec<Entry<RefContextAction>>,
    idx: u8,
) -> Vec<Entry<LogContextAction>> {
    entries
        .into_iter()
        .map(|e| match e {
            Entry::Action {
                label,
                action,
                destructive,
                key,
            } => Entry::Action {
                label,
                action: LogContextAction::Ref(action, idx),
                destructive,
                key,
            },
            Entry::Submenu { label, items } => Entry::Submenu {
                label,
                items: map_entries(items, idx),
            },
            Entry::Separator => Entry::Separator,
            Entry::Disabled { label } => Entry::Disabled { label },
        })
        .collect()
}

/// The refs on `commit`, in pill order, as sidebar identities. A remote
/// pill carries only the branch name, so the remote is looked up among the
/// refs on the same commit.
pub(crate) fn refs_at_commit(refs: &[Ref], commit: Oid) -> RefTable {
    ref_pills_for_commit(commit, refs)
        .into_iter()
        .filter_map(|pill| pill_identity(refs, commit, &pill.name, pill.kind))
        .collect()
}

/// The sidebar identity of a pill named `name` of `kind` on `commit`.
pub(crate) fn pill_identity(
    refs: &[Ref],
    commit: Oid,
    name: &str,
    kind: RefRowKind,
) -> Option<(SidebarSection, String)> {
    let section = SidebarSection::from_ref_kind(kind);
    match kind {
        RefRowKind::Local | RefRowKind::Tag => Some((section, name.to_string())),
        RefRowKind::Remote => refs.iter().find_map(|r| match &r.kind {
            RefKind::RemoteBranch { remote, name: n } if r.target == commit && n == name => {
                Some((section, format!("{remote}/{n}")))
            }
            // Already a full `remote/name` (a caller that resolved it).
            RefKind::RemoteBranch { remote, name: n } if format!("{remote}/{n}") == name => {
                Some((section, name.to_string()))
            }
            _ => None,
        }),
    }
}

/// Removes and returns the first row that is `action` (or, for a grayed
/// row, whose label contains `disabled_hint`).
fn take(
    entries: &mut Vec<Entry<LogContextAction>>,
    action: LogContextAction,
    disabled_hint: Option<&str>,
) -> Option<Entry<LogContextAction>> {
    let pos = entries.iter().position(|e| match e {
        Entry::Action { action: a, .. } => *a == action,
        Entry::Disabled { label } => disabled_hint.is_some_and(|h| label.contains(h)),
        _ => false,
    })?;
    Some(entries.remove(pos))
}

/// Removes the submenu whose label contains `hint`, returning its rows.
fn take_submenu(
    entries: &mut Vec<Entry<LogContextAction>>,
    hint: &str,
) -> Option<Vec<Entry<LogContextAction>>> {
    let pos = entries
        .iter()
        .position(|e| matches!(e, Entry::Submenu { label, .. } if label.contains(hint)))?;
    match entries.remove(pos) {
        Entry::Submenu { items, .. } => Some(items),
        _ => None,
    }
}

/// Pushes `rows` (dropping `None`s) followed by a separator.
fn group(out: &mut Vec<Entry<LogContextAction>>, rows: Vec<Option<Entry<LogContextAction>>>) {
    let rows: Vec<_> = rows.into_iter().flatten().collect();
    if !rows.is_empty() {
        out.extend(rows);
        out.push(Entry::Separator);
    }
}

/// One submenu per ref in `table[from..]`, holding that ref's own rows.
fn ref_submenus(
    src: &MenuSources<'_>,
    table: &RefTable,
    from: usize,
) -> Vec<Entry<LogContextAction>> {
    table
        .iter()
        .enumerate()
        .skip(from)
        .map(|(i, (section, name))| {
            let rows: Vec<_> = ref_rows(src, *section, name, i as u8).into_iter().collect();
            Entry::submenu(name.clone(), rows)
        })
        .collect()
}

fn commit_is_merge(src: &MenuSources<'_>, commit: Oid) -> bool {
    loaded_commit(src.log_result, commit).is_some_and(|n| n.parents.len() > 1)
}

/// A commit row's menu: the commit's rows, then one submenu per ref on it.
pub(crate) fn commit_menu(src: &MenuSources<'_>, commit: Oid) -> PillMenu {
    let ctx = MenuContext::from_refs(src.refs_result)
        .with_selected(src.selected_count)
        .with_ai(src.ai_enabled);
    let mut entries = commit_menu_entries(&ctx, commit_is_merge(src, commit));
    let table = if src.selected_count > 1 {
        RefTable::new()
    } else {
        src.refs_result
            .as_deref()
            .map(|refs| refs_at_commit(refs, commit))
            .unwrap_or_default()
    };
    if !table.is_empty() {
        entries.push(Entry::Separator);
        entries.extend(ref_submenus(src, &table, 0));
    }
    PillMenu { entries, table }
}

/// A ref pill's menu: `chosen`'s rows and `commit`'s rows in one list, and
/// a submenu per *other* ref on the commit.
///
/// Groups, top to bottom (an empty one leaves no rule behind): sync
/// (Pull / Push / Set upstream / Fetch); Checkout; merge, rebase and
/// fast-forward; Create worktree from; Create branch here, Reset, Edit
/// message, Revert and the rest of the history rewrites; Start a pull
/// request; Apply patch; Rename / Delete; view (hide / solo); Copy rows,
/// links and patches; the tag rows. Left out because the feature does not
/// exist: Drop commit, Move commit down, Pin to Left, Compare against the
/// working directory (and the AI rows, owned elsewhere).
pub(crate) fn pill_menu(
    src: &MenuSources<'_>,
    commit: Oid,
    chosen: (SidebarSection, String),
) -> PillMenu {
    use LogContextAction as L;
    use RefContextAction as R;
    let (section, name) = &chosen;
    let mut table: RefTable = vec![chosen.clone()];
    if let Ok(refs) = src.refs_result.as_ref() {
        table.extend(
            refs_at_commit(refs, commit)
                .into_iter()
                .filter(|other| other != &chosen),
        );
    }
    let ctx = MenuContext::from_refs(src.refs_result)
        .with_selected(0)
        .with_ai(src.ai_enabled);
    let is_merge = commit_is_merge(src, commit);
    let mut r: Vec<Entry<L>> = ref_rows(src, *section, name, 0)
        .into_iter()
        .filter(|e| !matches!(e, Entry::Separator))
        .collect();
    let mut c: Vec<Entry<L>> = commit_menu_entries(&ctx, is_merge)
        .into_iter()
        .filter(|e| !matches!(e, Entry::Separator))
        .collect();
    let rr = |a: R| L::Ref(a, 0);

    let mut out: Vec<Entry<L>> = Vec::new();
    // Sync.
    group(
        &mut out,
        vec![
            take(&mut r, rr(R::PullBranch), None),
            take(&mut r, rr(R::PushBranch), None),
            take(&mut r, rr(R::SetUpstream), None),
            take(&mut r, rr(R::FetchBranchRemote), None),
        ],
    );
    // Checkout: the ref, and the commit itself.
    let mut checkout: Vec<Entry<L>> = Vec::new();
    checkout.extend(take(&mut r, rr(R::Checkout), None));
    checkout.extend(take(&mut c, L::CheckoutCommit, None));
    if !checkout.is_empty() {
        out.push(Entry::submenu(
            glyph_label("\u{2713}", msg::CHECKOUT.get()),
            checkout,
        ));
        out.push(Entry::Separator);
    }
    group(
        &mut out,
        vec![
            take(&mut r, rr(R::MergeIntoCurrent), None),
            take(&mut r, rr(R::RebaseOntoThis), None),
            take(
                &mut r,
                rr(R::FastForwardToCurrent),
                Some(msg::HINT_FAST_FORWARD.get()),
            ),
            take(
                &mut r,
                rr(R::FastForwardCurrentToThis),
                Some(msg::HINT_FAST_FORWARD.get()),
            ),
        ],
    );
    // Create worktree from: this ref / this commit.
    let mut worktree: Vec<Entry<L>> = Vec::new();
    worktree.extend(take(&mut r, rr(R::CreateWorktree), None));
    worktree.extend(take(&mut c, L::CreateWorktree, None));
    if !worktree.is_empty() {
        out.push(Entry::submenu(
            glyph_label("\u{229E}", msg::WORKTREE_FROM.get()),
            worktree,
        ));
        out.push(Entry::Separator);
    }
    // The commit's history rows.
    group(
        &mut out,
        vec![
            take(&mut c, L::CreateBranchHere, None),
            take_submenu(&mut c, msg::HINT_RESET.get()).map(|items| {
                Entry::submenu(
                    glyph_label("\u{2190}", menu_log::reset_to(ctx.head_label())),
                    items,
                )
            }),
            take(&mut c, L::EditMessage, None),
            take(&mut c, L::Amend(crate::reword_ui::AmendScope::All), None),
            take(&mut c, L::Revert, None),
            take(&mut c, L::CherryPick, None),
            take(
                &mut c,
                L::InteractiveRebase,
                Some(msg::HINT_REBASE_HERE.get()),
            ),
        ],
    );
    // AI (hidden while AI is off: the row only exists with `ai_enabled`).
    group(
        &mut out,
        vec![
            take(&mut c, L::AiExplain, None),
            take(&mut c, L::AiSplitCommit, None),
        ],
    );
    // Start a pull request (grayed with its reason when it cannot run).
    group(
        &mut out,
        vec![take(
            &mut r,
            rr(R::StartPullRequest),
            Some(msg::HINT_START_PR.get()),
        )],
    );
    group(
        &mut out,
        vec![Some(Entry::action(
            menu_log::APPLY_PATCH.get(),
            L::ApplyPatch,
        ))],
    );
    // Rename / Delete, and whatever else is specific to this kind of ref
    // (a tag's push / annotate / delete rows).
    let mut edits: Vec<Option<Entry<L>>> = vec![
        take(&mut r, rr(R::RenameBranch), None),
        take(&mut r, rr(R::DeleteBranch), None),
        take(&mut r, rr(R::DeleteRemoteBranch), None),
        take(&mut r, rr(R::DeleteBranchAndRemote), None),
    ];
    let hide = take(&mut r, rr(R::ToggleHideInGraph), None);
    let solo = take(&mut r, rr(R::ToggleSolo), None);
    let copy_name = take_submenu(&mut r, msg::HINT_COPY.get());
    let branch_link = take(&mut r, rr(R::CopyBranchLink), None);
    edits.extend(std::mem::take(&mut r).into_iter().map(Some));
    group(&mut out, edits);
    // Copy, links, patches.
    let copy_sha = Entry::action(
        glyph_label("\u{2398}", msg::COPY_COMMIT_SHA.get()),
        L::Copy(CommitField::LongSha),
    );
    let mut copies = copy_name.unwrap_or_default();
    copies.push(Entry::Separator);
    copies.extend(take_submenu(&mut c, msg::HINT_COPY.get()).unwrap_or_default());
    group(
        &mut out,
        vec![
            Some(Entry::action(
                glyph_label("\u{2398}", msg::COPY_BRANCH_NAME.get()),
                rr(R::Copy(RefField::Name)),
            )),
            Some(copy_sha),
            branch_link,
            take(&mut c, L::CopyCommitLink, None),
            take(&mut c, L::CreatePatch, Some(msg::HINT_PATCH_COMMIT.get())),
            take(
                &mut c,
                L::CreatePatchToHead,
                Some(msg::HINT_PATCH_RANGE.get()),
            ),
            Some(Entry::submenu(
                glyph_label("\u{2398}", msg::COPY_MORE.get()),
                copies,
            )),
        ],
    );
    group(&mut out, vec![hide, solo]);
    group(
        &mut out,
        vec![
            take(&mut c, L::CreateTagHere, None),
            take(&mut c, L::CreateAnnotatedTagHere, None),
        ],
    );
    // Other refs on the commit, one submenu each.
    let others = ref_submenus(src, &table, 1);
    out.extend(others);
    PillMenu {
        entries: out,
        table,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::*;
    use std::collections::HashSet;

    fn head_at(name: &str, target: Oid) -> Ref {
        let mut r = local_branch_at(name, target);
        r.is_head = true;
        r
    }

    fn commit_node(id: u8) -> gitpoulpi_core::domain::CommitNode {
        crate::test_support::commit(id, &[])
    }

    fn labels(entries: &[Entry<LogContextAction>]) -> Vec<String> {
        entries
            .iter()
            .map(|e| match e {
                Entry::Action { label, .. }
                | Entry::Disabled { label }
                | Entry::Submenu { label, .. } => label.clone(),
                Entry::Separator => "-".into(),
            })
            .collect()
    }

    fn actions(entries: &[Entry<LogContextAction>], out: &mut Vec<LogContextAction>) {
        for e in entries {
            match e {
                Entry::Action { action, .. } => out.push(*action),
                Entry::Submenu { items, .. } => actions(items, out),
                _ => {}
            }
        }
    }

    #[test]
    fn a_remote_pill_resolves_to_its_remote_slash_name() {
        let commit = oid(1);
        let refs = vec![remote_branch_at("origin", "hello", commit)];
        assert_eq!(
            pill_identity(&refs, commit, "hello", RefRowKind::Remote),
            Some((SidebarSection::Remote, "origin/hello".to_string()))
        );
    }

    #[test]
    fn the_merged_menu_carries_both_the_refs_and_the_commits_rows() {
        let commit = oid(1);
        let refs = vec![
            head_at("main", commit),
            local_branch_at("hello", commit),
            remote_branch_at("origin", "main", commit),
        ];
        let refs_result = Ok(refs);
        let log_result = Ok(CommitGraph {
            nodes: vec![commit_node(1)],
        });
        let (hidden, solo, unpushed) = (HashSet::new(), HashSet::new(), HashSet::new());
        let src = MenuSources {
            refs_result: &refs_result,
            log_result: &log_result,
            remotes: &[],
            hidden: &hidden,
            solo: &solo,
            unpushed_tags: &unpushed,
            selected_count: 0,
            ai_enabled: false,
        };
        let menu = pill_menu(&src, commit, (SidebarSection::Local, "main".into()));
        let mut all = Vec::new();
        actions(&menu.entries, &mut all);
        use LogContextAction as L;
        use RefContextAction as R;
        for wanted in [
            L::Ref(R::PullBranch, 0),
            L::Ref(R::PushBranch, 0),
            L::Ref(R::SetUpstream, 0),
            L::CheckoutCommit,
            L::Ref(R::CreateWorktree, 0),
            L::CreateWorktree,
            L::CreateBranchHere,
            L::Revert,
            L::EditMessage,
            L::ApplyPatch,
            L::Ref(R::RenameBranch, 0),
            L::Ref(R::DeleteBranch, 0),
            L::Ref(R::DeleteRemoteBranch, 0),
            L::Ref(R::DeleteBranchAndRemote, 0),
            L::Ref(R::Copy(RefField::Name), 0),
            L::Copy(CommitField::LongSha),
            L::CopyCommitLink,
            L::Ref(R::CopyBranchLink, 0),
            L::CreatePatch,
            L::CreateTagHere,
            L::CreateAnnotatedTagHere,
        ] {
            // `Checkout` is offered only for a non-checked-out branch.
            if wanted == L::Ref(R::Checkout, 0) {
                continue;
            }
            if matches!(wanted, L::Ref(R::CreateWorktree, 0) | L::CreateWorktree)
                && !crate::build_features::WORKTREE
            {
                continue;
            }
            assert!(
                all.contains(&wanted),
                "{wanted:?} missing from {:?}",
                labels(&menu.entries)
            );
        }
        // The other local branch has its own submenu; the table lists it.
        assert!(menu.table.len() >= 2);
        assert!(labels(&menu.entries).iter().any(|l| l.contains("hello")));
    }

    #[test]
    fn a_commit_menu_has_one_submenu_per_ref() {
        let commit = oid(1);
        let refs = vec![head_at("master", commit), local_branch_at("hello", commit)];
        let refs_result = Ok(refs);
        let log_result = Ok(CommitGraph {
            nodes: vec![commit_node(1)],
        });
        let (hidden, solo, unpushed) = (HashSet::new(), HashSet::new(), HashSet::new());
        let src = MenuSources {
            refs_result: &refs_result,
            log_result: &log_result,
            remotes: &[],
            hidden: &hidden,
            solo: &solo,
            unpushed_tags: &unpushed,
            selected_count: 0,
            ai_enabled: false,
        };
        let menu = commit_menu(&src, commit);
        let subs: Vec<String> = menu
            .entries
            .iter()
            .filter_map(|e| match e {
                Entry::Submenu { label, .. } => Some(label.clone()),
                _ => None,
            })
            .collect();
        assert!(subs.contains(&"hello".to_string()), "{subs:?}");
        assert!(subs.contains(&"master".to_string()), "{subs:?}");
        assert_eq!(menu.table.len(), 2);
    }
}
