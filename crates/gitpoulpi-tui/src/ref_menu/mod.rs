//! The Refs sidebar's context menu: its actions, the copyable ref fields,
//! which entries a ref row gets and applying a chosen action (checkout,
//! rename, delete, hide/solo, stash operations...).

use super::*;
use crate::tree_fold::{FoldCtx, TreeFold};
use context_menu::glyph_label;
use gitpoulpi_i18n::msg::menu_ref as msg;

mod apply;

pub(crate) use apply::{apply_ref_context_action, RefActionCtx};

/// One row of a ref's right-click context menu (`ref_context_menu`) --
/// which rows appear depends on the ref's `RefRowKind`, see
/// `ref_menu_entries`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RefContextAction {
    Checkout,
    /// Scroll the Log to this ref's commit and flash its row
    /// (`ref_reveal`); in every menu of a ref that has a commit.
    RevealInGraph,
    ToggleHideInGraph,
    ToggleSolo,
    /// Hide/show every ref beneath a folder or remote-root row at once --
    /// the folder's own context menu. `full_name` (the row's identity, per
    /// `apply_ref_context_action`'s doc comment) is the folder's
    /// collapse-key path here, not a ref name.
    ToggleHideFolder,
    /// Solo/un-solo every ref beneath a folder or remote-root row at once.
    ToggleSoloFolder,
    MergeIntoCurrent,
    RebaseOntoThis,
    RenameBranch,
    /// Delete the local branch. Refused with a message on the checked-out
    /// one, at run time rather than by hiding the row.
    DeleteBranch,
    /// Delete the branch from its remote (`origin/x`), keeping any local
    /// branch of that name. Confirmed: other people can see the change.
    DeleteRemoteBranch,
    /// Delete the local branch and its remote twin together.
    DeleteBranchAndRemote,
    /// Copy the URL of this branch on its remote's host.
    CopyBranchLink,
    DeleteTag,
    /// Send this tag to `origin`. Offered only for a tag no remote is
    /// known to have - `Push <tag> to origin`.
    PushTag,
    /// Delete this tag from `origin`, keeping the local one. The mirror
    /// of `DeleteTag`, and offered only when the remote actually has it.
    DeleteTagFromRemote,
    /// Fetch this tag from the remote that has it: the row of a tag that
    /// lives only on a remote (`tag_where_ui`).
    FetchTag,
    /// Re-make this tag as an annotated one, where it already is.
    AnnotateTag,
    /// The bulk twin of `PushTag`: every unpushed tag, not just this
    /// one. Offered from any tag's menu (not only an unpushed tag's) --
    /// same shape as the command palette's `PushAllUnpushedTags`.
    PushAllUnpushedTags,
    StashPop,
    StashApply,
    StashDrop,
    /// SUBMODULES rows (`full_name` is the submodule's path): check it out
    /// at the recorded commit, cloning it first when it is not cloned.
    SubmoduleUpdate,
    /// Register it in `.git/config` without cloning.
    SubmoduleInit,
    /// Open it as a repository tab of its own.
    SubmoduleOpen,
    Copy(RefField),
    /// Choose (or type, or unset) this local branch's upstream.
    SetUpstream,
    /// Push this local branch - to its upstream's remote, else the
    /// default remote, setting the upstream on a first push.
    PushBranch,
    /// Pull into this branch. Offered only for the checked-out one:
    /// a pull integrates into HEAD.
    PullBranch,
    /// Fetch this branch's remote (its upstream's, else the default).
    FetchBranchRemote,
    /// Move this (local, not checked-out) branch forward to the
    /// checked-out branch's commit - `Fast-forward <this> to <current>`.
    FastForwardToCurrent,
    /// Move the checked-out branch forward to this branch (local or
    /// remote) - `Fast-forward <current> to <this>`.
    FastForwardCurrentToThis,
    /// Open Create worktree on this branch or tag.
    CreateWorktree,
    /// Open the Reflog view on this local branch: where it was before.
    Reflog,
    /// Start a pull request from this local branch:
    /// opens the host's compare/new-request page directly with no
    /// token, or a small form + API call with one. Local branches only -
    /// a remote branch already *is* the source a PR would name, and
    /// a tag is not a branch a host can open a request from.
    StartPullRequest,
    /// One row of the menu a ref drag-and-drop opens (`ref_drop`). The
    /// dragged ref travels beside the menu (`RefActionCtx::drop_source`),
    /// the row's own `full_name` is the target it was dropped on.
    Drop(DropOp),
    /// A row of the left pane's section checklist (`sections_menu`): check
    /// or uncheck one section. Runs without closing the menu.
    ToggleSidebarSection(SidebarSection),
    /// The checklist's "Reset to default": every section shown again.
    ResetSidebarSections,
    /// The checklist's "Use for all repositories": this repository's
    /// choice becomes the global default.
    UseSidebarSectionsForAll,
    /// Expand/collapse the folders of a refs section (its own menu) or of a
    /// folder row and everything below it (`crate::tree_fold`).
    Fold(crate::tree_fold::TreeFold),
    /// The section menu's way on to the show/hide checklist.
    OpenSectionsChecklist,
}

/// One cell of a ref, as its Copy submenu offers it.
///
/// Three, not six: a ref has a name, a spelling of that name git itself
/// uses, and a commit it points at. Anything else on the row (ahead/
/// behind, the checked-out marker) is computed, not a value to paste.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RefField {
    /// What the sidebar shows: `main`, `origin/feature`, `v1.2.0`.
    Name,
    /// The fully-qualified spelling: `refs/heads/main`. What a script,
    /// a `git push` refspec or a CI config wants.
    FullRef,
    /// The commit the ref points at, in full.
    TargetSha,
}

impl RefField {
    pub(crate) const fn all() -> [RefField; 3] {
        [RefField::Name, RefField::FullRef, RefField::TargetSha]
    }

    fn label(self) -> &'static str {
        match self {
            RefField::Name => msg::NOUN_NAME.get(),
            RefField::FullRef => msg::NOUN_FULL_REF.get(),
            RefField::TargetSha => msg::NOUN_TARGET_SHA.get(),
        }
    }

    /// The Copy submenu's own wording, which is shorter than `label`
    /// because the parent row already says "Copy".
    fn row_label(self) -> &'static str {
        match self {
            RefField::Name => msg::COPY_NAME.get(),
            RefField::FullRef => msg::COPY_FULL_REF.get(),
            RefField::TargetSha => msg::COPY_TARGET_SHA.get(),
        }
    }
}

/// `refs/heads/main` from (`Local`, `main`).
///
/// Derived from the row's kind rather than read from the backend
/// because the sidebar's `full_name` is already the unambiguous short
/// spelling within its section, and git's prefix for a section is fixed.
/// A remote row's `full_name` is `origin/feature` (see `group_refs`), so
/// `refs/remotes/` simply prefixes it.
pub(crate) fn qualified_ref_name(kind: RefRowKind, full_name: &str) -> String {
    match kind {
        RefRowKind::Local => format!("refs/heads/{full_name}"),
        RefRowKind::Remote => format!("refs/remotes/{full_name}"),
        RefRowKind::Tag => format!("refs/tags/{full_name}"),
    }
}

/// The `Ref` a sidebar row names, so its target `Oid` can be copied.
///
/// Matched on the *displayed* name for all three kinds - the same
/// `{remote}/{name}` join `group_refs` uses for remote rows, so the
/// lookup and the row it came from cannot spell the ref differently.
pub(crate) fn ref_by_display_name<'a>(
    refs: &'a gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Ref>>,
    kind: RefRowKind,
    full_name: &str,
) -> Option<&'a gitpoulpi_core::domain::Ref> {
    refs.as_ref().ok()?.iter().find(|r| match (&r.kind, kind) {
        (RefKind::LocalBranch { name }, RefRowKind::Local) => name == full_name,
        (RefKind::RemoteBranch { remote, name }, RefRowKind::Remote) => {
            format!("{remote}/{name}") == full_name
        }
        (RefKind::Tag { name }, RefRowKind::Tag) => name == full_name,
        _ => false,
    })
}

/// `(remote, branch)` of the remote-tracking ref shown as `display`
/// (`origin/feature/x`), or `None` when no such ref is loaded.
pub(crate) fn remote_branch_parts(
    refs: &[gitpoulpi_core::domain::Ref],
    display: &str,
) -> Option<(String, String)> {
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
pub(crate) fn branch_twins(
    refs: &[gitpoulpi_core::domain::Ref],
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
pub(crate) struct DeleteTargets {
    pub(crate) local: Option<String>,
    /// `(remote, branch)`.
    pub(crate) remote: Option<(String, String)>,
}

pub(crate) fn delete_targets(
    refs: &[gitpoulpi_core::domain::Ref],
    kind: RefRowKind,
    full_name: &str,
) -> DeleteTargets {
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
pub(crate) fn checked_out_refusal(
    refs: &[gitpoulpi_core::domain::Ref],
    branch: &str,
) -> Option<String> {
    refs.iter()
        .any(|r| r.is_head && matches!(&r.kind, RefKind::LocalBranch { name } if name == branch))
        .then(|| msg::cannot_delete_checked_out(branch))
}

/// A folder or remote-root row's own context menu: the bulk twin of a
/// leaf's hide/solo entries, and nothing else - a folder can't be
/// checked out, renamed, merged or deleted, so there is nothing else for
/// it to offer. A remote's own root row gets this *in addition to* the
/// remote-name menu (`remotes_ui::remote_menu_entries`, opened the same
/// right-click resolves to) - the two are separate surfaces by design,
/// mirroring how a leaf's ref menu and a Log ref-pill's are the same menu
/// while a remote's fetch/edit/remove menu is deliberately a different
/// one.
pub(crate) fn folder_menu_entries(
    all_hidden: bool,
    all_soloed: bool,
) -> Vec<context_menu::Entry<RefContextAction>> {
    use context_menu::Entry;
    let hide_label = if all_hidden {
        glyph_label(hide_in_graph_icon!(), msg::SHOW_ALL_IN_GRAPH.get())
    } else {
        glyph_label(hide_in_graph_icon!(), msg::HIDE_ALL_IN_GRAPH.get())
    };
    let solo_label = if all_soloed {
        glyph_label("\u{25C9}", msg::CLEAR_SOLO_ALL.get())
    } else {
        glyph_label("\u{25C9}", msg::SOLO_ALL.get())
    };
    let mut out = vec![
        Entry::action(hide_label, RefContextAction::ToggleHideFolder),
        Entry::action(solo_label, RefContextAction::ToggleSoloFolder),
        Entry::Separator,
    ];
    out.extend(crate::tree_fold::entries(
        &[TreeFold::ExpandBelow, TreeFold::CollapseBelow],
        FoldCtx::Refs,
        RefContextAction::Fold,
    ));
    out
}

/// The menu of a refs section's header or empty area: fold every folder of
/// the section, then the way on to the show/hide checklist of sections.
pub(crate) fn section_menu_entries() -> Vec<context_menu::Entry<RefContextAction>> {
    let mut out = crate::tree_fold::entries(
        &[TreeFold::ExpandAll, TreeFold::CollapseAll],
        FoldCtx::Refs,
        RefContextAction::Fold,
    );
    out.push(context_menu::Entry::Separator);
    out.push(context_menu::Entry::action(
        gitpoulpi_i18n::msg::tree_fold::SECTIONS_MENU.get(),
        RefContextAction::OpenSectionsChecklist,
    ));
    out
}

/// The `RefMenu` row identity of a section's own menu (see
/// `section_menu_entries`); the menu's section says which one.
pub(crate) const SECTION_MENU_KEY: &str = "\u{1}section-folds";

/// The menu rows offered for a ref, grouped into sections
/// and labeled with the names actually involved - "Merge feature into
/// main", not "Merge into current".
///
/// There is exactly one ref context menu in this codebase and this is
/// it: the sidebar's rows and the Log's ref chips both open it, so a row
/// added here appears in both.
///
/// Which rows appear depends on the kind, and on whether this *is* the
/// checked-out branch:
///
/// - **Local branches** get the full set. Checkout, Merge and Rebase are
///   omitted for the branch already checked out (all three are no-ops or
///   errors against yourself), and so is Delete - git refuses to delete
///   the branch HEAD is on, and a row whose only outcome is that error
///   is a dead affordance.
/// - **Remote branches** get the graph toggles and Copy but no checkout:
///   checking one out directly is refused (it detaches HEAD; create a
///   local tracking branch first), and a row that only prints that
///   refusal is a dead affordance.
/// - **Tags** get checkout (detaching onto a tag is a normal thing to
///   want), hide/solo, delete and Copy, but no rename: git tags aren't
///   renamable, only delete+recreate.
/// - **Stashes** aren't refs at all: no hide/solo/merge/copy, just the
///   three things you can do with a stash entry. "Pop" is the
///   "reprendre" action.
///
/// Copy is a submenu for the same reason the commit menu's Reset is: it
/// is three rows that all begin with the same word, and they would push
/// the rows a user actually reaches for off the top of the list.
pub(crate) fn ref_menu_entries(
    section: SidebarSection,
    full_name: &str,
    ctx: &MenuContext,
    is_hidden: bool,
    is_solo: bool,
    is_unpushed_tag: bool,
) -> Vec<context_menu::Entry<RefContextAction>> {
    use context_menu::Entry;
    let Some(kind) = section.ref_kind() else {
        return match section {
            SidebarSection::Stashes => vec![
                Entry::action(
                    glyph_label("\u{2192}", msg::REVEAL_IN_GRAPH.get()),
                    RefContextAction::RevealInGraph,
                )
                .with_key(crate::keymap::Action::RefsReveal),
                Entry::Separator,
                Entry::action(
                    glyph_label("\u{21A9}", msg::STASH_POP.get()),
                    RefContextAction::StashPop,
                )
                .with_key(crate::keymap::Action::GitStashPop),
                Entry::action(
                    glyph_label("\u{2398}", msg::STASH_APPLY.get()),
                    RefContextAction::StashApply,
                ),
                Entry::Separator,
                Entry::danger(
                    glyph_label("\u{2717}", msg::STASH_DROP.get()),
                    RefContextAction::StashDrop,
                )
                .with_key(crate::keymap::Action::GitDeleteRef),
            ],
            SidebarSection::Submodules if crate::build_features::SUBMODULE => vec![
                Entry::action(
                    glyph_label("\u{21E9}", msg::SUBMODULE_UPDATE.get()),
                    RefContextAction::SubmoduleUpdate,
                ),
                Entry::action(
                    glyph_label("\u{2022}", msg::SUBMODULE_INIT.get()),
                    RefContextAction::SubmoduleInit,
                ),
                Entry::Separator,
                Entry::action(
                    glyph_label("\u{2197}", msg::SUBMODULE_OPEN.get()),
                    RefContextAction::SubmoduleOpen,
                ),
            ],
            // The sidebar has no entry point for worktrees: no menu rather
            // than a menu of dead entries.
            _ => Vec::new(),
        };
    };
    let hide_label = if is_hidden {
        glyph_label(hide_in_graph_icon!(), msg::SHOW_IN_GRAPH.get())
    } else {
        glyph_label(hide_in_graph_icon!(), msg::HIDE_IN_GRAPH.get())
    };
    // The complement of hide-in-graph: soloing one or more branches shows
    // only their ancestries (unioned) and hides every other line. Solo
    // accumulates across refs -- selecting "Solo" on a ref that's already
    // soloed clears just that one (the label flips to "Clear solo" for
    // it), selecting it on a different ref adds that ref alongside
    // whatever's already soloed.
    let solo_label = if is_solo {
        glyph_label("\u{25C9}", msg::CLEAR_SOLO.get())
    } else {
        glyph_label("\u{25C9}", msg::SOLO.get())
    };
    let copy = Entry::submenu(
        glyph_label("\u{2398}", msg::COPY.get()),
        RefField::all()
            .into_iter()
            .map(|f| Entry::action(f.row_label(), RefContextAction::Copy(f)))
            .collect(),
    );
    let view = vec![
        Entry::action(hide_label, RefContextAction::ToggleHideInGraph),
        Entry::action(solo_label, RefContextAction::ToggleSolo)
            .with_key(crate::keymap::Action::RefsSolo),
    ];
    let is_current = kind == RefRowKind::Local && ctx.current_branch.as_deref() == Some(full_name);
    let head = ctx.head_label();
    // The two Fast-forward rows. Only with a checked-out branch
    // (a detached HEAD has no branch to move, and moving another branch
    // "to HEAD" there is a reset in disguise). A row that is not a
    // fast-forward stays, grayed out with the reason (`FfVerdict`).
    let ff_row =
        |label: String, verdict: FfVerdict, moving: &str, action| match verdict.refusal(moving) {
            Some(why) => Entry::disabled(label, &why),
            None => Entry::action(label, action),
        };
    let worktree_row = crate::build_features::WORKTREE.then(|| {
        Entry::action(
            glyph_label("\u{229E}", msg::create_worktree(full_name)),
            RefContextAction::CreateWorktree,
        )
    });
    // Every ref has a commit to show: first in the menu, then a rule.
    let mut entries: Vec<Entry<RefContextAction>> = vec![
        Entry::action(
            glyph_label("\u{2192}", msg::REVEAL_IN_GRAPH.get()),
            RefContextAction::RevealInGraph,
        )
        .with_key(crate::keymap::Action::RefsReveal),
        Entry::Separator,
    ];
    match kind {
        RefRowKind::Local => {
            if !is_current {
                entries.push(Entry::action(
                    glyph_label("\u{2713}", msg::checkout(full_name)),
                    RefContextAction::Checkout,
                ));
                entries.push(Entry::Separator);
                entries.push(Entry::action(
                    glyph_label("\u{21C6}", msg::merge_into(full_name, head)),
                    RefContextAction::MergeIntoCurrent,
                ));
                entries.push(
                    Entry::action(
                        glyph_label("\u{27F3}", msg::rebase_onto(head, full_name)),
                        RefContextAction::RebaseOntoThis,
                    )
                    .with_key(crate::keymap::Action::GitRebaseOnto),
                );
                if ctx.current_branch.is_some() {
                    entries.push(ff_row(
                        glyph_label("\u{21A0}", msg::fast_forward(full_name, head)),
                        ctx.fast_forward.this_to_head,
                        full_name,
                        RefContextAction::FastForwardToCurrent,
                    ));
                    entries.push(ff_row(
                        glyph_label("\u{21A0}", msg::fast_forward(head, full_name)),
                        ctx.fast_forward.head_to_this,
                        head,
                        RefContextAction::FastForwardCurrentToThis,
                    ));
                }
            }
            entries.extend(worktree_row);
            entries.push(Entry::action(
                glyph_label(
                    "\u{21BA}",
                    gitpoulpi_i18n::msg::undo_safety::REFLOG_MENU_ROW.get(),
                ),
                RefContextAction::Reflog,
            ));
            // The sync group. Pull only on the checked-out
            // branch: a pull integrates into HEAD, so offering it on
            // another branch would pull into the wrong one.
            entries.push(Entry::Separator);
            if is_current {
                entries.push(
                    Entry::action(
                        glyph_label("\u{21E9}", msg::PULL.get()),
                        RefContextAction::PullBranch,
                    )
                    .with_key(crate::keymap::Action::GitPull),
                );
            }
            entries.push(
                Entry::action(
                    glyph_label("\u{21E7}", msg::push_branch(full_name)),
                    RefContextAction::PushBranch,
                )
                .with_key(crate::keymap::Action::GitPush),
            );
            let pr_label = glyph_label("\u{21F1}", msg::start_pr(full_name));
            if crate::build_features::HOSTING {
                entries.push(match &ctx.pr_reason {
                    None => Entry::action(pr_label, RefContextAction::StartPullRequest),
                    Some(reason) => Entry::disabled(pr_label, reason),
                });
            }
            entries.push(Entry::action(
                glyph_label("\u{27F2}", msg::FETCH.get()),
                RefContextAction::FetchBranchRemote,
            ));
            entries.push(Entry::action(
                glyph_label("\u{21C5}", msg::set_upstream(full_name)),
                RefContextAction::SetUpstream,
            ));
            entries.push(Entry::Separator);
            entries.extend(view);
            entries.push(Entry::Separator);
            entries.push(Entry::action(
                glyph_label("\u{270E}", msg::rename(full_name)),
                RefContextAction::RenameBranch,
            ));
            entries.extend(delete_entries(kind, full_name, ctx));
        }
        RefRowKind::Remote => {
            // Same behavior as double-clicking the row: `checkout_ref_now`
            // resolves this to the local branch that tracks it (or offers
            // to create one), never a raw `git checkout` of the remote ref
            // -- that would detach HEAD.
            entries.push(Entry::action(
                glyph_label("\u{2713}", msg::checkout(full_name)),
                RefContextAction::Checkout,
            ));
            // The common one: bring the checked-out branch up to what
            // was just fetched.
            if ctx.current_branch.is_some() {
                entries.push(ff_row(
                    glyph_label("\u{21A0}", msg::fast_forward(head, full_name)),
                    ctx.fast_forward.head_to_this,
                    head,
                    RefContextAction::FastForwardCurrentToThis,
                ));
            }
            entries.extend(worktree_row);
            entries.push(Entry::Separator);
            entries.extend(view);
            let deletes = delete_entries(kind, full_name, ctx);
            if !deletes.is_empty() {
                entries.push(Entry::Separator);
                entries.extend(deletes);
            }
        }
        RefRowKind::Tag => {
            use gitpoulpi_core::domain::TagWhere;
            // Where the tag lives decides its rows (`tag_where_ui`): a tag
            // only on a remote is not in this repository, so it gets Fetch
            // and a remote delete but no checkout or annotate; a tag only
            // here gets Push; one on a remote gets a Delete that asks which
            // side (`tag_delete_dialog`).
            let info = crate::tag_where_ui::lookup(full_name);
            let place = info
                .as_ref()
                .map(|i| i.place)
                .or(is_unpushed_tag.then_some(TagWhere::Local));
            let remote_only = place == Some(TagWhere::Remote);
            if !remote_only {
                entries.push(Entry::action(
                    glyph_label("\u{2713}", msg::checkout(full_name)),
                    RefContextAction::Checkout,
                ));
                entries.extend(worktree_row);
            }
            entries.push(Entry::Separator);
            entries.extend(view);
            entries.push(Entry::Separator);
            match place {
                Some(TagWhere::Local) => entries.push(Entry::action(
                    glyph_label("\u{2B06}", msg::push_tag(full_name)),
                    RefContextAction::PushTag,
                )),
                Some(TagWhere::Remote) => {
                    let remote = info
                        .as_ref()
                        .and_then(|i| i.all_remotes().into_iter().next())
                        .unwrap_or_default();
                    entries.push(Entry::action(
                        glyph_label(
                            "\u{2B07}",
                            gitpoulpi_i18n::msg::tag_where::fetch_tag(full_name, &remote),
                        ),
                        RefContextAction::FetchTag,
                    ));
                }
                // Both / differs: nothing to push or fetch; the Delete row
                // below carries the remote side.
                _ => {}
            }
            // Not tied to this tag specifically -- reachable from any
            // tag's menu, the way the palette's `PushAllUnpushedTags`
            // needs no ref selected at all.
            entries.push(
                Entry::action(
                    glyph_label("\u{2B06}", msg::PUSH_ALL_TAGS.get()),
                    RefContextAction::PushAllUnpushedTags,
                )
                .with_key(crate::keymap::Action::GitPushTags),
            );
            if !remote_only {
                entries.push(Entry::action(
                    glyph_label("\u{270E}", msg::annotate(full_name)),
                    RefContextAction::AnnotateTag,
                ));
            }
            // The red Delete says what it will ask: a tag on a remote opens
            // the scope dialog ("Delete x..."), a remote-only tag names the
            // remote, and a tag only here deletes at once after its confirm.
            match (place, &info) {
                (Some(TagWhere::Remote), Some(i)) => entries.push(Entry::danger(
                    glyph_label(
                        "\u{2717}",
                        gitpoulpi_i18n::msg::tag_where::delete_tag_on_remote(
                            full_name,
                            i.all_remotes().join(", "),
                        ),
                    ),
                    RefContextAction::DeleteTagFromRemote,
                )),
                (Some(TagWhere::Both | TagWhere::Differs), _) => entries.push(Entry::danger(
                    glyph_label(
                        "\u{2717}",
                        gitpoulpi_i18n::msg::tag_where::delete_tag_scope(full_name),
                    ),
                    RefContextAction::DeleteTag,
                )),
                _ => entries.push(Entry::danger(
                    glyph_label("\u{2717}", msg::delete(full_name)),
                    RefContextAction::DeleteTag,
                )),
            }
        }
    }
    entries.push(Entry::Separator);
    if has_branch_link(kind, ctx) {
        entries.push(Entry::action(
            glyph_label("\u{2398}", msg::COPY_BRANCH_LINK.get()),
            RefContextAction::CopyBranchLink,
        ));
    }
    entries.push(copy);
    entries
}

/// Takes the leading Reveal in graph row (and its rule) off `entries`: a
/// menu opened in the Log is already there.
pub(crate) fn drop_reveal_row(entries: &mut Vec<context_menu::Entry<RefContextAction>>) {
    if matches!(
        entries.first(),
        Some(context_menu::Entry::Action {
            action: RefContextAction::RevealInGraph,
            ..
        })
    ) {
        entries.drain(..2.min(entries.len()));
    }
}

/// Whether a "Copy link to branch on remote" row makes sense: a remote
/// branch, or a local one that has a remote twin.
pub(crate) fn has_branch_link(kind: RefRowKind, ctx: &MenuContext) -> bool {
    match kind {
        RefRowKind::Remote => true,
        RefRowKind::Local => ctx.remote_twin.is_some(),
        RefRowKind::Tag => false,
    }
}

/// The delete rows for a branch: the row's own ref, and the
/// combinations with its counterpart when there is one. Every row is
/// confirmed when run; the checked-out branch is refused at run time
/// (with a message) rather than hidden.
pub(crate) fn delete_entries(
    kind: RefRowKind,
    full_name: &str,
    ctx: &MenuContext,
) -> Vec<context_menu::Entry<RefContextAction>> {
    use context_menu::Entry;
    let mut out = Vec::new();
    match kind {
        RefRowKind::Local => {
            out.push(
                Entry::danger(
                    glyph_label("\u{2717}", msg::delete(full_name)),
                    RefContextAction::DeleteBranch,
                )
                .with_key(crate::keymap::Action::GitDeleteRef),
            );
            if let Some(twin) = &ctx.remote_twin {
                out.push(Entry::danger(
                    glyph_label("\u{2717}", msg::delete(twin)),
                    RefContextAction::DeleteRemoteBranch,
                ));
                out.push(Entry::danger(
                    glyph_label("\u{2717}", msg::delete_both(full_name, twin)),
                    RefContextAction::DeleteBranchAndRemote,
                ));
            }
        }
        RefRowKind::Remote => {
            out.push(Entry::danger(
                glyph_label("\u{2717}", msg::delete(full_name)),
                RefContextAction::DeleteRemoteBranch,
            ));
            if let Some(local) = &ctx.local_twin {
                out.push(Entry::danger(
                    glyph_label("\u{2717}", msg::delete_both(local, full_name)),
                    RefContextAction::DeleteBranchAndRemote,
                ));
            }
        }
        RefRowKind::Tag => {}
    }
    out
}

/// Which stash a "stash pop/apply/drop" command acts on: the one
/// selected in the STASHES pane when that pane has focus, otherwise the
/// most recent (`stash@{0}`).
pub(crate) fn target_stash(
    stashes: &gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Stash>>,
    focus: Focus,
    active: SidebarSection,
    stash_state: &ListState,
) -> Option<gitpoulpi_core::domain::Stash> {
    let stashes = stashes.as_ref().ok()?;
    if focus == Focus::Refs && active == SidebarSection::Stashes {
        if let Some(selected) = stash_state.selected().and_then(|i| stashes.get(i)) {
            return Some(selected.clone());
        }
    }
    stashes.first().cloned()
}

/// The STASHES sidebar rows carry their stash id as the context menu's
/// row identity (refs carry their name there instead), so acting on one
/// means parsing it back.
pub(crate) fn parse_stash_id(raw: &str) -> Option<gitpoulpi_core::domain::StashId> {
    raw.parse().ok().map(gitpoulpi_core::domain::StashId)
}

#[cfg(test)]
mod tests;
