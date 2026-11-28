//! The Log pane's commit context menu: its actions, the copyable commit
//! fields, which entries a commit gets, their confirmations and applying a
//! chosen action.

use super::*;
use context_menu::glyph_label;
use gitpoulpi_i18n::msg::{menu_log as msg, menu_ref};

mod apply;

pub(crate) use apply::{apply_log_context_action, apply_revert_mainline, apply_wip_context_action};

/// Where the "Rewrite history" submenu starts the workbench.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RewriteFrom {
    /// The workbench as the chord opens it.
    Workbench,
    /// The Log's selected commits (or this one).
    Selected,
    /// This commit and everything after it.
    FromHere,
}

/// One row of a commit's right-click context menu (`log_context_menu`).
///
/// Everything git does *with* an existing commit rather than to a ref,
/// which is why these hang off a Log row and not off the sidebar:
/// cherry-pick copies it here, revert un-does it here, reset moves the
/// branch to it, and branch/tag create a new ref pointing at it.
///
/// `Copy` is the odd one out - it never reaches the daemon, because
/// nothing about the repository changes. It reads a field of the commit
/// the Log already has and puts it on the system clipboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LogContextAction {
    CherryPick,
    Revert,
    Reset(gitpoulpi_core::domain::ResetMode),
    CreateBranchHere,
    CreateTagHere,
    /// Detach HEAD onto this commit (`CheckoutTarget::Commit`), which is
    /// `Checkout > this commit`. Distinct from
    /// `RefContextAction::Checkout`, which moves HEAD to a *ref*.
    CheckoutCommit,
    /// Give this commit a new message (`GitBackend::reword`), in the
    /// Edit commit message popup (`reword_ui.rs`).
    EditMessage,
    /// Amend this commit's properties (`GitBackend::amend_commit`) in the
    /// complete Amend popup (`reword_ui.rs`); the scope picks the field
    /// that starts focused.
    Amend(AmendScope),
    /// Open the split editor on this commit (`split_ui/`): it asks the
    /// daemon for the commit's numbered changes first.
    SplitCommit,
    /// Open the interactive rebase popup (`rebase_ui.rs`) with the range
    /// from just after this commit up to HEAD. Only offered when this
    /// commit is an ancestor of HEAD on its first-parent chain (see
    /// `rebase_form_for`); a merge commit disables the row entirely.
    InteractiveRebase,
    /// Open Create worktree on this commit: a linked worktree, detached
    /// here or on a new branch started here.
    CreateWorktree,
    /// Open Create patch, seeded with this one commit
    /// (`PatchSource::Commits`).
    CreatePatch,
    /// Open Create patch, seeded with the range from this commit to HEAD
    /// (`PatchSource::Range`).
    CreatePatchToHead,
    Copy(CommitField),
    /// A row only the commit column's menu has (the views of a file at a
    /// commit). Run by the event loop, which owns the open view.
    Column(ColumnOp),
    /// AI explains this commit in a read-only popup. Offered only while
    /// `ai.enabled`; run by the event loop (it owns the AI worker), so
    /// `apply_log_context_action` does nothing for it.
    AiExplain,
    /// AI proposes how to split this commit; the split editor opens
    /// prefilled with it. Offered only while `ai.enabled`; run by the
    /// event loop like [`Self::AiExplain`].
    AiSplitCommit,
    /// Copy the URL of this commit on the remote's host.
    CopyCommitLink,
    /// Open the Copy as Mermaid gitGraph popup (`mermaid_popup.rs`) on the
    /// Log's selected commits, or on this one and its run up to HEAD.
    CopyAsMermaid,
    /// Name, then describe, an annotated tag at this commit.
    CreateAnnotatedTagHere,
    /// A row of the "Rewrite history" submenu: opens the workbench with the
    /// scope this commit (or the selection) gives it. Run by the event loop,
    /// which owns the workbench's context.
    RewriteHistory(RewriteFrom),
    /// Open Apply patch's file picker (the WIP menu's row, reachable from
    /// a ref's merged menu too). Run by the event loop, which owns the
    /// dialogs.
    ApplyPatch,
    /// Open the Lost work view: commits no branch reaches.
    RecoverLost,
    /// One of a ref's own rows inside a menu that also carries a commit's
    /// (a ref pill's merged menu, a commit's per-ref submenus). The index
    /// points into the table the menu was opened with (`pill_menu`), which
    /// keeps this `Copy` while the ref it names is a `String`. Run by the
    /// event loop through `apply_ref_context_action`.
    Ref(RefContextAction, u8),
}

mod vocab;
pub(crate) use vocab::{ColumnOp, CommitField};

/// The rows the commit column's menu puts before the Log's own: the views of
/// the open file at the commit under the pointer.
pub(crate) fn column_menu_entries() -> Vec<context_menu::Entry<LogContextAction>> {
    ColumnOp::ALL
        .into_iter()
        .map(|op| context_menu::Entry::action(op.label(), LogContextAction::Column(op)))
        .collect()
}

/// The length of a "short SHA" everywhere in this UI - the Log column,
/// the context-menu caption, every status message and the clipboard.
pub(crate) const SHORT_SHA_LEN: usize = 7;

/// What the two context menus need in order to spell their rows the way
/// the menu says "Reset main to this commit", not "Reset branch
/// here".
///
/// Built once per menu *opening*, not per frame: the labels are a
/// snapshot of the moment the user right-clicked, and a branch that moves
/// under an open menu must not silently relabel its rows.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct MenuContext {
    /// The checked-out local branch, or `None` on a detached HEAD.
    pub(crate) current_branch: Option<String>,
    /// Whether the menu's ref and the checked-out branch can be
    /// fast-forwarded onto each other - what grays out the two
    /// Fast-forward rows. `Default` (unknown: both offered, the backend
    /// decides) for every menu that is not about a branch.
    pub(crate) fast_forward: FastForwardCheck,
    /// Why "Start a pull request" is disabled for this branch (no
    /// upstream, or a remote host `gitpoulpi-hosting` cannot address),
    /// or `None` to offer it. Computed by the caller from live
    /// remote/upstream/integration data (`pull_request::pull_request_reason`) -
    /// not by `for_ref`, the way `fast_forward` is, because unlike a
    /// fast-forward verdict it needs the configured integrations, which
    /// `MenuContext`'s other constructors have no reason to take.
    pub(crate) pr_reason: Option<String>,
    /// How many commits are multi-selected in the Log (Ctrl/Shift+click,
    /// Space). Above one, a commit's menu offers only the multi-commit
    /// cherry-pick: every other row acts on a single commit.
    pub(crate) selected_count: usize,
    /// `ai.enabled`: whether the AI rows are offered at all.
    pub(crate) ai_enabled: bool,
    /// For a remote branch's menu: the local branch tracking it (or of the
    /// same name), which the Delete rows can take along.
    pub(crate) local_twin: Option<String>,
    /// For a local branch's menu: its remote-tracking twin (`origin/x`).
    pub(crate) remote_twin: Option<String>,
    /// The merge fast-forward policy in force (labels the Merge row of a
    /// branch's menu).
    pub(crate) merge_ff: gitpoulpi_core::domain::MergeFastForward,
    /// For a local branch's menu: its extra push targets and pull sources
    /// (`None` for a branch with only its upstream: today's rows).
    pub(crate) group: Option<crate::remotes_ui::BranchGroup>,
}

/// Whether moving one branch onto another commit would be a
/// fast-forward, as far as the loaded graph can tell.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum FfVerdict {
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
    pub(crate) fn refusal(self, moving: &str) -> Option<String> {
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
pub(crate) enum MergeLabel {
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
    pub(crate) fn of(policy: gitpoulpi_core::domain::MergeFastForward, verdict: FfVerdict) -> Self {
        use gitpoulpi_core::domain::MergeFastForward as P;
        let possible = verdict == FfVerdict::Possible;
        match (policy, possible) {
            (P::Never, true) => Self::NewCommit,
            (P::Allowed, true) => Self::FastForwards,
            (P::Only, _) => Self::FastForwardOnly,
            _ => Self::Plain,
        }
    }
}

/// The two Fast-forward rows' verdicts for one ref menu.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct FastForwardCheck {
    /// `Fast-forward <this> to <current>`.
    pub(crate) this_to_head: FfVerdict,
    /// `Fast-forward <current> to <this>`.
    pub(crate) head_to_this: FfVerdict,
}

/// Is moving a branch at `moving` to `target` a fast-forward? Walks
/// parents from `target` through the loaded graph only; a walk that
/// leaves the loaded pages answers `Unknown` rather than guessing.
pub(crate) fn ff_verdict(
    graph: &gitpoulpi_core::domain::CommitGraph,
    moving: Oid,
    target: Oid,
) -> FfVerdict {
    if moving == target {
        return FfVerdict::AlreadyThere;
    }
    let by_id: std::collections::HashMap<Oid, &gitpoulpi_core::domain::CommitNode> =
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

impl MenuContext {
    /// The context for one ref's menu: `from_refs`, plus the two
    /// fast-forward verdicts between that ref and the checked-out branch.
    pub(crate) fn for_ref(
        refs: &gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Ref>>,
        log: &gitpoulpi_core::Result<gitpoulpi_core::domain::CommitGraph>,
        section: SidebarSection,
        name: &str,
    ) -> Self {
        let mut ctx = Self::from_refs(refs);
        if let (Ok(all), Some(kind)) = (refs.as_ref(), section.ref_kind()) {
            (ctx.local_twin, ctx.remote_twin) = branch_twins(all, kind, name);
            if kind == RefRowKind::Local {
                ctx.group = crate::remotes_ui::branch_group(all, name);
            }
        }
        let (Ok(all), Ok(graph), Some(kind)) = (refs.as_ref(), log.as_ref(), section.ref_kind())
        else {
            return ctx;
        };
        let head = all
            .iter()
            .find(|r| r.is_head && matches!(r.kind, RefKind::LocalBranch { .. }));
        let this = ref_by_display_name(refs, kind, name);
        if let (Some(head), Some(this)) = (head, this) {
            ctx.fast_forward = FastForwardCheck {
                this_to_head: ff_verdict(graph, this.target, head.target),
                head_to_this: ff_verdict(graph, head.target, this.target),
            };
        }
        ctx
    }

    pub(crate) fn from_refs(
        refs: &gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Ref>>,
    ) -> Self {
        Self {
            current_branch: refs.as_ref().ok().and_then(|refs| {
                refs.iter().find_map(|r| match (&r.kind, r.is_head) {
                    (RefKind::LocalBranch { name }, true) => Some(name.clone()),
                    _ => None,
                })
            }),
            fast_forward: FastForwardCheck::default(),
            pr_reason: None,
            selected_count: 0,
            ai_enabled: false,
            local_twin: None,
            remote_twin: None,
            merge_ff: gitpoulpi_core::domain::MergeFastForward::default(),
            group: None,
        }
    }

    pub(crate) fn with_selected(mut self, count: usize) -> Self {
        self.selected_count = count;
        self
    }

    /// Offers the AI rows (`ai.enabled`).
    pub(crate) fn with_ai(mut self, enabled: bool) -> Self {
        self.ai_enabled = enabled;
        self
    }

    /// What to call HEAD in a label. `HEAD` rather than inventing a name
    /// when it is detached - git's own word, and the state the user is
    /// actually in.
    pub(crate) fn head_label(&self) -> &str {
        self.current_branch.as_deref().unwrap_or("HEAD")
    }
}

/// The menu offered for a commit, grouped into sections.
///
/// The groups, top to bottom: what this commit can do to the current
/// branch (cherry-pick / revert / reset), what can be created at it
/// (branch / tag / a detached checkout), and Copy. Rules between them
/// come from `Entry::Separator`; `Menu::new` drops any that end up
/// leading, trailing or doubled, so a group that is empty in some future
/// context cannot leave its rule behind.
///
/// The three resets are a submenu rather than three top-level rows: they
/// were half the menu, they all read "Reset ... to this commit", and the
/// rows a user actually reaches for were buried under them. See
/// `context_menu`'s header for why a submenu here is a drill-in.
///
/// Glyphs are plain BMP arrows and geometric shapes with no emoji
/// presentation and no variation selector, per the convention in
/// `where-we-are.md`: a VS16-dependent glyph renders as a hole under a
/// different terminal style.
pub(crate) fn commit_menu_entries(
    ctx: &MenuContext,
    is_merge: bool,
) -> Vec<context_menu::Entry<LogContextAction>> {
    use context_menu::Entry;
    use gitpoulpi_core::domain::ResetMode;
    let head = ctx.head_label();
    let merge_reason = msg::MERGE_NO_PATCH.get();
    if ctx.selected_count > 1 {
        let mut many = vec![Entry::action(
            glyph_label("\u{2295}", msg::cherry_pick_many(ctx.selected_count, head)),
            LogContextAction::CherryPick,
        )];
        if crate::build_features::HISTORY_REWRITE {
            many.push(Entry::submenu(
                glyph_label("\u{21BB}", msg::REWRITE_HISTORY.get()),
                vec![
                    Entry::action(
                        msg::REWRITE_SELECTED.get(),
                        LogContextAction::RewriteHistory(RewriteFrom::Selected),
                    ),
                    Entry::action(
                        msg::REWRITE_WORKBENCH.get(),
                        LogContextAction::RewriteHistory(RewriteFrom::Workbench),
                    ),
                ],
            ));
        }
        if crate::build_features::MERMAID {
            many.push(
                Entry::action(
                    glyph_label("\u{2398}", gitpoulpi_i18n::msg::mermaid::MENU_COPY.get()),
                    LogContextAction::CopyAsMermaid,
                )
                .with_key(crate::keymap::Action::LogCopyMermaid),
            );
        }
        return many;
    }
    let mut entries = vec![
        Entry::action(
            glyph_label("\u{2295}", msg::cherry_pick_onto(head)),
            LogContextAction::CherryPick,
        )
        .with_key(crate::keymap::Action::GitCherryPick),
        Entry::danger(
            glyph_label("\u{21BA}", msg::REVERT.get()),
            LogContextAction::Revert,
        )
        .with_key(crate::keymap::Action::GitRevert),
        Entry::submenu(
            glyph_label("\u{2190}", msg::reset_to(head)),
            vec![
                Entry::action(
                    msg::RESET_SOFT.get(),
                    LogContextAction::Reset(ResetMode::Soft),
                ),
                Entry::action(
                    msg::RESET_MIXED.get(),
                    LogContextAction::Reset(ResetMode::Mixed),
                )
                .with_key(crate::keymap::Action::GitResetMixed),
                Entry::danger(
                    msg::RESET_HARD.get(),
                    LogContextAction::Reset(ResetMode::Hard),
                ),
            ],
        ),
        Entry::action(
            glyph_label("\u{270E}", msg::EDIT_MESSAGE.get()),
            LogContextAction::EditMessage,
        )
        .with_key(crate::keymap::Action::GitEditMessage),
        Entry::action(
            glyph_label("\u{2710}", msg::AMEND.get()),
            LogContextAction::Amend(AmendScope::All),
        )
        .with_key(crate::keymap::Action::GitAmend),
        if is_merge {
            Entry::disabled(
                glyph_label("\u{25D1}", msg::SPLIT_COMMIT.get()),
                gitpoulpi_i18n::msg::split_ui::REFUSE_MERGE_COMMIT.get(),
            )
        } else {
            Entry::action(
                glyph_label("\u{25D1}", msg::SPLIT_COMMIT.get()),
                LogContextAction::SplitCommit,
            )
        },
        if is_merge {
            Entry::disabled(
                glyph_label("\u{25C8}", msg::REBASE_HERE.get()),
                merge_reason,
            )
        } else {
            Entry::action(
                glyph_label("\u{25C8}", msg::REBASE_HERE.get()),
                LogContextAction::InteractiveRebase,
            )
        },
        Entry::Separator,
        Entry::action(
            glyph_label("\u{21B3}", msg::CREATE_BRANCH_HERE.get()),
            LogContextAction::CreateBranchHere,
        ),
        Entry::action(
            glyph_label("\u{25C7}", msg::CREATE_TAG_HERE.get()),
            LogContextAction::CreateTagHere,
        ),
        Entry::action(
            glyph_label("\u{25C7}", msg::CREATE_ANNOTATED_TAG_HERE.get()),
            LogContextAction::CreateAnnotatedTagHere,
        ),
        Entry::action(
            glyph_label("\u{2713}", msg::CHECKOUT_COMMIT.get()),
            LogContextAction::CheckoutCommit,
        ),
        Entry::action(
            glyph_label("\u{229E}", msg::CREATE_WORKTREE_HERE.get()),
            LogContextAction::CreateWorktree,
        ),
        if is_merge {
            Entry::disabled(msg::PATCH_FROM_COMMIT.get(), merge_reason)
        } else {
            Entry::action(msg::PATCH_FROM_COMMIT.get(), LogContextAction::CreatePatch)
        },
        if is_merge {
            Entry::disabled(msg::patch_to_head(head), merge_reason)
        } else if ctx.current_branch.is_none() {
            Entry::disabled(msg::patch_to_head(head), msg::HEAD_DETACHED.get())
        } else {
            Entry::action(
                msg::patch_to_head(head),
                LogContextAction::CreatePatchToHead,
            )
        },
        Entry::Separator,
        Entry::action(
            glyph_label(
                "\u{21BA}",
                gitpoulpi_i18n::msg::undo_safety::LOST_MENU_ROW.get(),
            ),
            LogContextAction::RecoverLost,
        ),
        Entry::action(
            glyph_label("\u{2398}", msg::COPY_COMMIT_LINK.get()),
            LogContextAction::CopyCommitLink,
        ),
        Entry::submenu(
            glyph_label("\u{2398}", menu_ref::COPY.get()),
            CommitField::all()
                .into_iter()
                .map(|f| Entry::action(copy_row_label(f.label()), LogContextAction::Copy(f)))
                .chain(crate::build_features::MERMAID.then_some(Entry::Separator))
                .chain(crate::build_features::MERMAID.then(|| {
                    Entry::action(
                        gitpoulpi_i18n::msg::mermaid::MENU_COPY.get(),
                        LogContextAction::CopyAsMermaid,
                    )
                    .with_key(crate::keymap::Action::LogCopyMermaid)
                }))
                .collect(),
        ),
    ];
    if !crate::build_features::WORKTREE {
        entries.retain(|e| {
            !matches!(
                e,
                Entry::Action {
                    action: LogContextAction::CreateWorktree,
                    ..
                }
            )
        });
    }
    if crate::build_features::HISTORY_REWRITE {
        // Right below Split commit and the rebase row, before the
        // create-at-this-commit group.
        let at = entries
            .iter()
            .position(|e| matches!(e, Entry::Separator))
            .unwrap_or(entries.len());
        entries.insert(
            at,
            Entry::submenu(
                glyph_label("\u{21BB}", msg::REWRITE_HISTORY.get()),
                vec![
                    Entry::action(
                        msg::REWRITE_FROM_HERE.get(),
                        LogContextAction::RewriteHistory(RewriteFrom::FromHere),
                    ),
                    Entry::action(
                        msg::REWRITE_WORKBENCH.get(),
                        LogContextAction::RewriteHistory(RewriteFrom::Workbench),
                    ),
                ],
            ),
        );
    }
    if ctx.ai_enabled {
        // After Edit commit message: both are about the commit's story.
        entries.insert(
            4,
            Entry::action(
                glyph_label("\u{2726}", msg::EXPLAIN_CHANGES_AI.get()),
                LogContextAction::AiExplain,
            )
            .with_key(crate::keymap::Action::GitAiExplain),
        );
        // Next to the other ✦ row; grayed with the reason where a plain
        // split is.
        entries.insert(
            5,
            if is_merge {
                Entry::disabled(
                    glyph_label("\u{2726}", msg::SPLIT_COMMIT_AI.get()),
                    gitpoulpi_i18n::msg::split_ui::REFUSE_MERGE_COMMIT.get(),
                )
            } else {
                Entry::action(
                    glyph_label("\u{2726}", msg::SPLIT_COMMIT_AI.get()),
                    LogContextAction::AiSplitCommit,
                )
            },
        );
    }
    entries
}

/// A Copy submenu row: the field name with its first letter capitalized,
/// because `CommitField::label` is written for the middle of a sentence
/// ("copied author email") and a menu row is not.
pub(crate) fn copy_row_label(label: &str) -> String {
    let mut chars = label.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// The confirm prompt a commit action needs before it runs, or `None`
/// when it runs straight away.
///
/// Split out from `apply_log_context_action` so the *policy* - which
/// rows stop at the modal - is a pure function with a test, rather than
/// something only reachable by driving a live `DaemonLink`.
///
/// Revert and `reset --hard` are the two that confirm, for the same reason
/// `DiscardFile` does: both rewrite the working tree from a commit, so an
/// uncommitted edit to a file they touch is gone. Soft and mixed resets
/// only move the branch pointer (and are in the reflog); a prompt there
/// would teach the user to press `y` without reading. Copy, branch and
/// tag create nothing and destroy nothing.
pub(crate) fn log_action_confirmation(
    action: LogContextAction,
    commit: Oid,
) -> Option<(String, PendingDestructive)> {
    use gitpoulpi_core::domain::ResetMode;
    let short = &commit.to_hex()[..SHORT_SHA_LEN];
    match action {
        LogContextAction::Revert => Some((
            msg::confirm_revert(short),
            PendingDestructive::Revert {
                commit,
                mainline: None,
            },
        )),
        LogContextAction::Reset(ResetMode::Hard) => Some((
            msg::confirm_reset_hard(short),
            PendingDestructive::ResetHard { target: commit },
        )),
        LogContextAction::CherryPick
        | LogContextAction::Reset(_)
        | LogContextAction::CreateBranchHere
        | LogContextAction::CreateTagHere
        | LogContextAction::CheckoutCommit
        | LogContextAction::EditMessage
        | LogContextAction::Amend(_)
        | LogContextAction::SplitCommit
        | LogContextAction::InteractiveRebase
        | LogContextAction::CreateWorktree
        | LogContextAction::CreatePatch
        | LogContextAction::CreatePatchToHead
        | LogContextAction::AiExplain
        | LogContextAction::AiSplitCommit
        | LogContextAction::RewriteHistory(_)
        | LogContextAction::CopyCommitLink
        | LogContextAction::CopyAsMermaid
        | LogContextAction::CreateAnnotatedTagHere
        | LogContextAction::ApplyPatch
        | LogContextAction::RecoverLost
        | LogContextAction::Ref(..)
        | LogContextAction::Column(_)
        | LogContextAction::Copy(_) => None,
    }
}

/// A patch filename's base: lowercase, non-alphanumeric runs collapsed to
/// one `-`, trimmed of leading/trailing `-`, capped so the whole name
/// stays reasonable on every filesystem.
pub(crate) fn slugify(text: &str) -> String {
    let mut out = String::new();
    let mut last_was_dash = false;
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash && !out.is_empty() {
            out.push('-');
            last_was_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out.chars().take(40).collect()
}

/// The loaded commit an `Oid` names, or `None` if it has scrolled out of
/// the loaded page (or the log request failed).
///
/// Only the Copy rows need this: everything else sends the `Oid` to the
/// daemon, which resolves it against the real repository. A copy has to
/// read a field the client already holds, and "the client already holds
/// it" stops being true at the edge of `LogLoadSettings`' window.
pub(crate) fn loaded_commit(
    log_result: &gitpoulpi_core::Result<gitpoulpi_core::domain::CommitGraph>,
    commit: Oid,
) -> Option<&gitpoulpi_core::domain::CommitNode> {
    log_result
        .as_ref()
        .ok()?
        .nodes
        .iter()
        .find(|n| n.id == commit)
}

/// One row of the WIP row's own right-click context menu
/// (`wip_context_menu`) - everything the Log's pinned "uncommitted
/// changes" row can do to the *whole* working tree, as opposed to the
/// per-file rows the Status pane's own menus already cover.
///
/// `StashAll`/`StageAll`/`DiscardAll` are the whole-tree actions that
/// already have a palette command, offered here so a right-click on the
/// row reaches them without a trip to `Ctrl+P`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WipContextAction {
    /// Open Create patch, seeded with the working tree's current changes
    /// (`PatchSource::WorkingTree`) - same seed `PaletteCommand::
    /// CreatePatchFromWip` builds.
    CreatePatch,
    /// Open Apply patch's file picker.
    ApplyPatch,
    StashAll,
    StageAll,
    DiscardAll,
    /// AI explains the working changes in a read-only popup (only
    /// offered while `ai.enabled`; run by the event loop).
    AiExplain,
    /// AI proposes how to split the working changes into logical commits
    /// (only while `ai.enabled`; run by the event loop).
    AiCompose,
}

/// Why a whole-tree WIP action is disabled, or `None` to offer it.
///
/// Reachable only in principle: the WIP row this menu is anchored on is
/// drawn exactly when `has_wip_row` is true (`graph::LogRows`), so the
/// menu never opens over a clean tree. Kept as a tested branch in case a
/// palette-driven opening ever can. `ApplyPatch` is exempt: applying a
/// patch file needs no existing changes.
fn wip_action_disabled_reason(action: WipContextAction, has_wip: bool) -> Option<&'static str> {
    if has_wip || action == WipContextAction::ApplyPatch {
        return None;
    }
    Some(msg::NO_UNCOMMITTED.get())
}

/// The WIP row's context menu: create/apply a patch, plus the existing
/// whole-tree actions a right-click on this row should reach without a
/// trip to `Ctrl+P`.
pub(crate) fn wip_menu_entries(
    has_wip: bool,
    ai_enabled: bool,
) -> Vec<context_menu::Entry<WipContextAction>> {
    use context_menu::Entry;
    let row =
        |label: &str, action: WipContextAction| match wip_action_disabled_reason(action, has_wip) {
            Some(reason) => Entry::disabled(label, reason),
            None => Entry::action(label, action),
        };
    let mut entries = vec![
        row(msg::WIP_PATCH.get(), WipContextAction::CreatePatch),
        row(msg::APPLY_PATCH.get(), WipContextAction::ApplyPatch),
        Entry::Separator,
        row(msg::STASH_ALL.get(), WipContextAction::StashAll),
        row(msg::STAGE_ALL.get(), WipContextAction::StageAll),
        match wip_action_disabled_reason(WipContextAction::DiscardAll, has_wip) {
            Some(reason) => Entry::disabled(msg::DISCARD_ALL.get(), reason),
            None => Entry::danger(msg::DISCARD_ALL.get(), WipContextAction::DiscardAll),
        },
    ];
    if ai_enabled {
        entries.insert(
            0,
            row(
                &glyph_label("\u{2726}", msg::EXPLAIN_CHANGES_AI.get()),
                WipContextAction::AiExplain,
            ),
        );
        entries.insert(
            1,
            row(
                &glyph_label("\u{2726}", msg::COMPOSE_COMMITS_AI.get()),
                WipContextAction::AiCompose,
            ),
        );
    }
    entries
}

/// The stash context menu, opened over a stash row in the Log: exactly the
/// STASHES section's own (Pop / Apply / Drop), so both run through
/// `apply_ref_context_action` with the stash's id as the row identity.
pub(crate) fn stash_context_menu(
    stash: &gitpoulpi_core::domain::Stash,
    col: u16,
    row: u16,
) -> (SidebarSection, String, ContextMenu<RefContextAction>) {
    let id = stash.id.0.to_string();
    let mut entries = ref_menu_entries(
        SidebarSection::Stashes,
        &id,
        &MenuContext::default(),
        false,
        false,
        false,
    );
    drop_reveal_row(&mut entries);
    (
        SidebarSection::Stashes,
        id.clone(),
        ContextMenu::new(format!("stash@{{{id}}}"), entries, col, row),
    )
}

/// The Edit commit message popup for `commit`, prefilled with its full
/// message and knowing whether it is already pushed - or why it cannot
/// open: the commit is not on the checked-out branch (the backend would
/// refuse, so the Log says so first), or it has left the loaded page.
pub(crate) fn reword_form_for(
    log_result: &gitpoulpi_core::Result<gitpoulpi_core::domain::CommitGraph>,
    refs: &gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Ref>>,
    commit: Oid,
) -> Result<RewordForm, String> {
    let short = &commit.to_hex()[..SHORT_SHA_LEN];
    let refs = refs.as_deref().unwrap_or(&[]);
    let (Ok(graph), Some(node)) = (log_result.as_ref(), loaded_commit(log_result, commit)) else {
        return Err(msg::not_loaded(short));
    };
    if on_current_branch(graph, refs, commit) == Some(false) {
        return Err(msg::not_on_current_branch(short));
    }
    Ok(RewordForm::new(
        commit,
        &node.message,
        pushed_upstream(graph, refs, commit),
    ))
}

/// The menu a right click on a property of the commit-detail pane opens:
/// that property's own Amend row (and, for the message, the whole-message
/// editor) above the full Amend popup.
pub(crate) fn detail_part_menu_entries(
    scope: AmendScope,
) -> Vec<context_menu::Entry<LogContextAction>> {
    use context_menu::Entry;
    let amend = |label: &str, scope: AmendScope| {
        Entry::action(
            glyph_label("\u{2710}", label),
            LogContextAction::Amend(scope),
        )
    };
    let mut entries = Vec::new();
    match scope {
        AmendScope::Summary => entries.push(amend(msg::AMEND_SUMMARY.get(), AmendScope::Summary)),
        AmendScope::Description => {
            entries.push(amend(msg::AMEND_DESCRIPTION.get(), AmendScope::Description))
        }
        AmendScope::Author => entries.push(amend(msg::AMEND_AUTHOR.get(), AmendScope::Author)),
        AmendScope::All | AmendScope::Message | AmendScope::Date => {}
    }
    if matches!(scope, AmendScope::Summary | AmendScope::Description) {
        entries.push(
            Entry::action(
                glyph_label("\u{270E}", msg::EDIT_MESSAGE.get()),
                LogContextAction::EditMessage,
            )
            .with_key(crate::keymap::Action::GitEditMessage),
        );
    }
    entries
        .push(amend(msg::AMEND.get(), AmendScope::All).with_key(crate::keymap::Action::GitAmend));
    entries
}

/// The Amend popup over `scope`'s fields for `commit`, prefilled with the
/// commit's message and author; refuses as [`reword_form_for`] does.
pub(crate) fn amend_form_for(
    log_result: &gitpoulpi_core::Result<gitpoulpi_core::domain::CommitGraph>,
    refs: &gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Ref>>,
    commit: Oid,
    scope: AmendScope,
) -> Result<RewordForm, String> {
    let form = reword_form_for(log_result, refs, commit)?;
    let short = &commit.to_hex()[..SHORT_SHA_LEN];
    let node = loaded_commit(log_result, commit).ok_or_else(|| msg::not_loaded(short))?;
    Ok(form.amend(scope, &node.author_name, &node.author_email, node.timestamp))
}

/// Whether the split editor can open on `commit`, or why not: it has left
/// the loaded page, it is a merge, or it is not on the checked-out branch
/// (the backend would refuse, so the Log says so first).
pub(crate) fn split_precheck(
    log_result: &gitpoulpi_core::Result<gitpoulpi_core::domain::CommitGraph>,
    refs: &gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Ref>>,
    commit: Oid,
) -> Result<(), String> {
    let short = &commit.to_hex()[..SHORT_SHA_LEN];
    let refs = refs.as_deref().unwrap_or(&[]);
    let (Ok(graph), Some(node)) = (log_result.as_ref(), loaded_commit(log_result, commit)) else {
        return Err(msg::not_loaded(short));
    };
    if node.parents.len() > 1 {
        return Err(gitpoulpi_i18n::msg::split_ui::REFUSE_MERGE_COMMIT
            .get()
            .to_string());
    }
    if on_current_branch(graph, refs, commit) == Some(false) {
        return Err(msg::split_not_on_current_branch(short));
    }
    Ok(())
}

/// The interactive-rebase popup for the range just after `commit` up to
/// HEAD, built entirely from the loaded `CommitGraph` - no daemon round
/// trip, same reason `reword_form_for` skips one.
///
/// Walks HEAD's *first-parent* chain only (the same line an interactive
/// rebase is limited to: it refuses any merge in the range), refusing
/// with a reason the moment that walk: hits a merge commit before
/// reaching `commit`, runs off the edge of the loaded page, or reaches
/// the root without ever finding `commit` (not an ancestor of HEAD --
/// "rebase HEAD onto here" for a divergent commit isn't offered). Also
/// refuses on a detached HEAD: there is no branch to rebase.
pub(crate) fn rebase_form_for(
    log_result: &gitpoulpi_core::Result<gitpoulpi_core::domain::CommitGraph>,
    refs: &gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Ref>>,
    commit: Oid,
) -> Result<RebaseForm, String> {
    use gitpoulpi_core::domain::RefKind;

    let short = &commit.to_hex()[..SHORT_SHA_LEN];
    let refs = refs.as_deref().unwrap_or(&[]);
    let Ok(graph) = log_result.as_ref() else {
        return Err(msg::not_loaded(short));
    };
    let head_branch = refs
        .iter()
        .find(|r| r.is_head && matches!(r.kind, RefKind::LocalBranch { .. }));
    let Some(head_branch) = head_branch else {
        return Err(msg::REBASE_DETACHED.get().into());
    };
    let branch_name = match &head_branch.kind {
        RefKind::LocalBranch { name } => name.clone(),
        _ => unreachable!("filtered to LocalBranch above"),
    };

    let by_id: std::collections::HashMap<Oid, &gitpoulpi_core::domain::CommitNode> =
        graph.nodes.iter().map(|n| (n.id, n)).collect();

    let mut chain: Vec<&gitpoulpi_core::domain::CommitNode> = Vec::new();
    let mut cur = head_branch.target;
    loop {
        let Some(node) = by_id.get(&cur) else {
            return Err(msg::HISTORY_SCROLLED_HEAD.get().into());
        };
        if node.parents.len() > 1 {
            return Err(msg::MERGE_IN_WAY.get().into());
        }
        let found = node.id == commit;
        chain.push(node);
        if found {
            break;
        }
        match node.parents.first() {
            Some(parent) => cur = *parent,
            None => {
                return Err(msg::not_ancestor(short));
            }
        }
    }

    // The clicked commit is the base the rows are replayed onto, so it
    // is the upstream itself (its parent would make the backend's range
    // one commit longer than the plan and refuse every Start).
    let upstream = Some(commit);
    let rows: Vec<RebaseRow> = chain[..chain.len() - 1]
        .iter()
        .map(|n| RebaseRow::pick(n.id, &n.summary, &n.message))
        .collect();
    let mut form = RebaseForm::new(branch_name, upstream, None, rows);
    form.onto_label = Some(short.to_string());
    Ok(form)
}

/// The interactive view for cherry-picking `selected` (newest first)
/// onto HEAD. Refuses a commit that is not in the loaded page or is a
/// merge, with the reason to show.
pub(crate) fn cherry_pick_form_for(
    log_result: &gitpoulpi_core::Result<gitpoulpi_core::domain::CommitGraph>,
    refs: &gitpoulpi_core::Result<Vec<gitpoulpi_core::domain::Ref>>,
    selected: &[Oid],
) -> Result<RebaseForm, String> {
    let mut rows = Vec::with_capacity(selected.len());
    for &id in selected {
        let node =
            loaded_commit(log_result, id).ok_or_else(|| msg::SELECTED_GONE.get().to_string())?;
        if node.parents.len() > 1 {
            return Err(msg::cherry_merge(&id.to_hex()[..SHORT_SHA_LEN]));
        }
        rows.push(RebaseRow::pick(id, &node.summary, &node.message));
    }
    let head = MenuContext::from_refs(refs).head_label().to_string();
    let mut form = RebaseForm::new_cherry_pick(head, rows);
    form.head_sha = refs
        .as_ref()
        .ok()
        .and_then(|rs| refs_sidebar::head_oid(rs))
        .map(|oid| oid.to_hex()[..SHORT_SHA_LEN].to_string());
    Ok(form)
}

/// The commit a Log screen row points at.
///
/// `None` for the border and header rows, for anything past the end of
/// the graph, and - deliberately - for the pinned WIP row, which
/// renders the working tree rather than a commit and so has no `Oid` to
/// cherry-pick, revert or reset onto. All of that arithmetic goes through
/// `log_row_to_index`, never a hand-rolled `log_area.y + n`: every
/// open-coded version of this has landed one row out at some point.
pub(crate) fn log_commit_at_row(
    log_result: &gitpoulpi_core::Result<gitpoulpi_core::domain::CommitGraph>,
    rows: &graph::LogRows,
    area: Rect,
    offset: usize,
    log_len: usize,
    row: u16,
) -> Option<Oid> {
    let index = log_row_to_index(area, offset, row, log_len)?;
    match rows.entry(index) {
        graph::LogEntry::Commit(i) => log_result.as_ref().ok()?.nodes.get(i).map(|c| c.id),
        graph::LogEntry::Wip | graph::LogEntry::Stash(_) => None,
    }
}

pub(crate) trait CommitFieldExt: Sized {
    /// Every field, in menu order. `all()` rather than a literal at the
    /// call site so the menu and `label`/`value` cannot drift apart.
    fn all() -> [CommitField; 6];
    /// The menu row, which is also the noun the status line uses.
    fn label(self) -> &'static str;
    /// What lands on the clipboard.
    ///
    /// `message` is the *full* `summary\n\nbody`, not the one-line
    /// `summary` the Log column shows: a user copying a commit message
    /// wants the message, and the truncated version is already on screen.
    /// `Date` goes through `format_timestamp_full`, so what is copied is
    /// the same rendering the hover tooltip shows - seconds included,
    /// and honouring `ui.date_time_format` rather than inventing a
    /// second date format nobody configured.
    fn value(self, node: &gitpoulpi_core::domain::CommitNode) -> String;
}

impl CommitFieldExt for CommitField {
    /// Every field, in menu order. `all()` rather than a literal at the
    /// call site so the menu and `label`/`value` cannot drift apart.
    fn all() -> [CommitField; 6] {
        [
            CommitField::Message,
            CommitField::AuthorName,
            CommitField::AuthorEmail,
            CommitField::Date,
            CommitField::ShortSha,
            CommitField::LongSha,
        ]
    }
    /// The menu row, which is also the noun the status line uses.
    fn label(self) -> &'static str {
        match self {
            CommitField::Message => msg::FIELD_MESSAGE.get(),
            CommitField::AuthorName => msg::FIELD_AUTHOR_NAME.get(),
            CommitField::AuthorEmail => msg::FIELD_AUTHOR_EMAIL.get(),
            CommitField::Date => msg::FIELD_DATE.get(),
            CommitField::ShortSha => msg::FIELD_SHORT_SHA.get(),
            CommitField::LongSha => msg::FIELD_FULL_SHA.get(),
        }
    }
    /// What lands on the clipboard.
    ///
    /// `message` is the *full* `summary\n\nbody`, not the one-line
    /// `summary` the Log column shows: a user copying a commit message
    /// wants the message, and the truncated version is already on screen.
    /// `Date` goes through `format_timestamp_full`, so what is copied is
    /// the same rendering the hover tooltip shows - seconds included,
    /// and honouring `ui.date_time_format` rather than inventing a
    /// second date format nobody configured.
    fn value(self, node: &gitpoulpi_core::domain::CommitNode) -> String {
        match self {
            CommitField::Message => node.message.clone(),
            CommitField::AuthorName => node.author_name.clone(),
            CommitField::AuthorEmail => node.author_email.clone(),
            CommitField::Date => format_timestamp_full(node.timestamp),
            CommitField::ShortSha => node.id.to_hex()[..SHORT_SHA_LEN].to_string(),
            CommitField::LongSha => node.id.to_hex(),
        }
    }
}

#[cfg(test)]
mod tests;
