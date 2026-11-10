//! Drag and drop of refs: what dropping one ref on
//! another *offers* (`drop_menu_entries`) and what each offer *does*
//! (`plan_drop`).
//!
//! The gesture itself (arming, the open-hand pointer, the lit target, the
//! footer banner) lives in `refs_sidebar` (`RefDrag`,
//! `ref_drop_is_valid`); a release on a valid target opens this module's
//! menu at the pointer. Its rows are ordinary `RefContextAction::Drop(..)`
//! rows so the existing menu machinery (keyboard, mouse, hover) serves
//! them unchanged, and every one is also reachable without a mouse: the
//! palette's `Refs: Drop selected ref on the checked-out branch...` opens
//! the same menu, and the ref menu's own rows cover the HEAD-relative
//! subset.
//!
//! | dragged -> target | rows |
//! |---|---|
//! | local X -> local Y | Merge X into Y, Rebase X onto Y, Interactive rebase X onto Y, Fast-forward Y to X |
//! | local X -> remote R | Push X to R's remote |
//! | remote R -> local Y | Merge R into Y, Rebase Y onto R, Fast-forward Y to R, Pull (when R is Y's upstream) |
//!
//! Anything that changes a branch other than the checked-out one first
//! checks it out (`Action::CheckoutThen`, which stops if the checkout is
//! refused), the same as a click on the menu entry.

use gitpoulpi_config::PullButtonMode;
use gitpoulpi_core::domain::{CommitGraph, PullOptions, Ref};

use super::*;
use context_menu::glyph_label;
use gitpoulpi_i18n::msg::{menu_drop as msg, menu_log, menu_ref};

/// The glyph that tags the ref marked for a keyboard drop (`Refs: Mark as
/// Drop Source`): the black diamond, text presentation in every monospace
/// font measured for this project, so it never flips under a bold style.
pub(crate) const MARK_GLYPH: char = '\u{25C6}';

/// One choice of the drop menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DropOp {
    /// Merge the dragged ref into the target branch.
    Merge,
    /// Local X onto Y: replay X on Y. Remote R onto Y: replay Y on R.
    Rebase,
    /// Open the interactive rebase popup for X's commits onto Y.
    InteractiveRebase,
    /// Move the target branch forward to the dragged ref.
    FastForward,
    /// Push the dragged local branch to the target's remote.
    Push,
    /// Pull the dragged remote branch into the (local) target.
    Pull,
}

/// What a chosen `DropOp` turns into.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum DropPlan {
    /// Submit this action and say this on the status line.
    Submit(Action, String),
    /// Open the interactive rebase popup.
    Rebase(Box<RebaseForm>),
}

fn find_ref<'a>(refs: &'a [Ref], kind: RefRowKind, name: &str) -> Option<&'a Ref> {
    refs.iter().find(|r| match (&r.kind, kind) {
        (RefKind::LocalBranch { name: n }, RefRowKind::Local) => n == name,
        (RefKind::RemoteBranch { remote, name: n }, RefRowKind::Remote) => {
            format!("{remote}/{n}") == name
        }
        (RefKind::Tag { name: n }, RefRowKind::Tag) => n == name,
        _ => false,
    })
}

/// `then`, preceded by checking `branch` out unless it already is.
fn after_checkout(branch: &str, head: Option<&HeadBranch>, then: Action) -> Action {
    if head.is_some_and(|h| h.name == branch) {
        then
    } else {
        Action::CheckoutThen {
            branch: branch.to_string(),
            then: Box::new(then),
        }
    }
}

/// The rows a drop of `source` on `target` offers. Empty when the pair
/// is not a valid drop (`ref_drop_is_valid` is the gate the gesture
/// itself uses; this repeats no rule but reads the same table).
pub(crate) fn drop_menu_entries(
    source: &str,
    source_kind: RefRowKind,
    target: &str,
    target_kind: RefRowKind,
    refs: &[Ref],
    graph: Option<&CommitGraph>,
    merge_ff: gitpoulpi_core::domain::MergeFastForward,
) -> Vec<context_menu::Entry<RefContextAction>> {
    use context_menu::Entry;
    if !ref_drop_is_valid(source, source_kind, target, target_kind) {
        return Vec::new();
    }
    let src = find_ref(refs, source_kind, source);
    let tgt = find_ref(refs, target_kind, target);
    let row = |label: String, op: DropOp| Entry::action(label, RefContextAction::Drop(op));
    // Fast-forward grays out with the reason, like the ref menu's rows.
    let verdict = match (graph, tgt, src) {
        (Some(g), Some(t), Some(s)) => ff_verdict(g, t.target, s.target),
        _ => FfVerdict::Unknown,
    };
    // The Merge row says what the fast-forward policy will make of this pair.
    let merge_row = || {
        let text = match crate::log_menu::MergeLabel::of(merge_ff, verdict) {
            crate::log_menu::MergeLabel::Plain => msg::merge_into(source, target),
            crate::log_menu::MergeLabel::NewCommit => msg::merge_into_commit(source, target),
            crate::log_menu::MergeLabel::FastForwards => msg::merge_into_ff(source, target),
            crate::log_menu::MergeLabel::FastForwardOnly => msg::merge_into_ff_only(source, target),
        };
        let label = glyph_label("\u{21C6}", text);
        match verdict {
            FfVerdict::Diverged if merge_ff == gitpoulpi_core::domain::MergeFastForward::Only => {
                Entry::disabled(label, &verdict.refusal(target).unwrap_or_default())
            }
            _ => row(label, DropOp::Merge),
        }
    };
    let ff = |label: String| match verdict.refusal(target) {
        Some(why) => Entry::disabled(label, &why),
        None => row(label, DropOp::FastForward),
    };
    match (source_kind, target_kind) {
        (RefRowKind::Local, RefRowKind::Local) => vec![
            merge_row(),
            row(
                glyph_label("\u{27F3}", msg::rebase_onto(source, target)),
                DropOp::Rebase,
            ),
            row(
                glyph_label("\u{2261}", msg::rebase_interactive(source, target)),
                DropOp::InteractiveRebase,
            ),
            ff(glyph_label(
                "\u{21A0}",
                menu_ref::fast_forward(target, source),
            )),
        ],
        (RefRowKind::Local, RefRowKind::Remote) => vec![row(
            glyph_label("\u{21E7}", msg::push_to(source, target)),
            DropOp::Push,
        )],
        (RefRowKind::Remote, RefRowKind::Local) => {
            let mut rows = vec![
                merge_row(),
                row(
                    glyph_label("\u{27F3}", msg::rebase_target_onto(target, source)),
                    DropOp::Rebase,
                ),
                ff(glyph_label(
                    "\u{21A0}",
                    menu_ref::fast_forward(target, source),
                )),
            ];
            // A pull only means something from the branch's own upstream.
            if tgt.and_then(|t| t.upstream.as_deref()) == Some(source) {
                rows.push(row(
                    glyph_label("\u{21E9}", msg::pull_into(source, target)),
                    DropOp::Pull,
                ));
            }
            rows
        }
        _ => Vec::new(),
    }
}

/// The menu's title.
pub(crate) fn drop_menu_title(source: &str, target: &str) -> String {
    format!("{source} \u{25B8} {target}")
}

/// What `op` does for this drop. `Err` is a status line saying why not.
#[allow(clippy::too_many_arguments)]
pub(crate) fn plan_drop(
    op: DropOp,
    source: &str,
    source_kind: RefRowKind,
    target: &str,
    target_kind: RefRowKind,
    refs: &[Ref],
    graph: Option<&CommitGraph>,
    remotes: &[String],
    pull_mode: PullButtonMode,
) -> Result<DropPlan, String> {
    let gone = |name: &str| crate::git_actions::gone(name);
    let src = find_ref(refs, source_kind, source).ok_or_else(|| gone(source))?;
    let tgt = find_ref(refs, target_kind, target).ok_or_else(|| gone(target))?;
    let head = head_branch(refs);
    match (op, source_kind, target_kind) {
        // Both directions merge `src` into the local branch `target`.
        (DropOp::Merge, _, RefRowKind::Local) => Ok(DropPlan::Submit(
            after_checkout(target, head.as_ref(), Action::Merge(src.clone())),
            msg::merging(source, target),
        )),
        (DropOp::Rebase, RefRowKind::Local, RefRowKind::Local) => Ok(DropPlan::Submit(
            after_checkout(source, head.as_ref(), Action::Rebase(tgt.clone())),
            msg::rebasing_source(source, target),
        )),
        (DropOp::Rebase, RefRowKind::Remote, RefRowKind::Local) => Ok(DropPlan::Submit(
            after_checkout(target, head.as_ref(), Action::Rebase(src.clone())),
            msg::rebasing_target(target, source),
        )),
        (DropOp::InteractiveRebase, RefRowKind::Local, RefRowKind::Local) => {
            let graph = graph.ok_or(msg::NO_HISTORY.get())?;
            let checkout = (head.as_ref().map(|h| h.name.as_str()) != Some(source))
                .then(|| source.to_string());
            rebase_form_onto(graph, source, src.target, tgt.target, checkout).map(|mut form| {
                form.onto_label = Some(target.to_string());
                DropPlan::Rebase(Box::new(form))
            })
        }
        (DropOp::FastForward, _, RefRowKind::Local) => Ok(DropPlan::Submit(
            Action::FastForward {
                branch: target.to_string(),
                target: src.target,
                target_label: source.to_string(),
            },
            menu_ref::fast_forwarding(target, source),
        )),
        (DropOp::Push, RefRowKind::Local, RefRowKind::Remote) => {
            let (remote, _) = gitpoulpi_core::domain::split_remote_ref(target, remotes)
                .ok_or_else(|| msg::not_on_remote(target))?;
            let branch = HeadBranch {
                name: source.to_string(),
                upstream: src.upstream.clone(),
            };
            Ok(DropPlan::Submit(
                push_action(&remote, &branch, false),
                menu_ref::pushing(source, &remote),
            ))
        }
        (DropOp::Pull, RefRowKind::Remote, RefRowKind::Local) => {
            let branch = HeadBranch {
                name: target.to_string(),
                upstream: tgt.upstream.clone(),
            };
            let (pull, said) = pull_action(pull_mode, Some(&branch), None)?;
            debug_assert!(matches!(&pull, Action::Pull(PullOptions { .. })));
            Ok(DropPlan::Submit(
                after_checkout(target, head.as_ref(), pull),
                said,
            ))
        }
        _ => Err(msg::cannot_drop(source, target)),
    }
}

/// The interactive-rebase popup for `source`'s commits that `target`
/// lacks, replayed onto `target` (`git rebase -i --onto target
/// <merge-base>`), built from the loaded graph.
///
/// Refuses what the popup cannot model: a merge commit in the range
/// (interactive rebase would flatten it) or history that has scrolled out
/// of the loaded page. `checkout` is the branch to check out before
/// starting (the popup rebases the checked-out branch), `None` when
/// `source` already is.
pub(crate) fn rebase_form_onto(
    graph: &CommitGraph,
    source: &str,
    source_tip: Oid,
    target_tip: Oid,
    checkout: Option<String>,
) -> Result<RebaseForm, String> {
    let by_id: std::collections::HashMap<Oid, &gitpoulpi_core::domain::CommitNode> =
        graph.nodes.iter().map(|n| (n.id, n)).collect();
    // Everything `target` already contains.
    let mut known = std::collections::HashSet::new();
    let mut stack = vec![target_tip];
    while let Some(id) = stack.pop() {
        if !known.insert(id) {
            continue;
        }
        if let Some(node) = by_id.get(&id) {
            stack.extend(node.parents.iter().copied());
        }
    }
    let mut rows = Vec::new();
    let mut cur = source_tip;
    let base = loop {
        if known.contains(&cur) {
            break cur;
        }
        let Some(node) = by_id.get(&cur) else {
            return Err(msg::HISTORY_SCROLLED.get().into());
        };
        if node.parents.len() > 1 {
            return Err(menu_log::MERGE_IN_WAY.get().into());
        }
        rows.push(RebaseRow::pick(node.id, &node.summary, &node.message));
        match node.parents.first() {
            Some(parent) => cur = *parent,
            None => return Err(msg::shares_no_history(source)),
        }
    };
    if rows.is_empty() {
        return Err(msg::nothing_to_replay(source));
    }
    let onto = (target_tip != base).then_some(target_tip);
    let mut form = RebaseForm::new(source.to_string(), Some(base), onto, rows);
    form.checkout = checkout;
    Ok(form)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::*;

    fn remote_at(remote: &str, name: &str, target: Oid) -> Ref {
        Ref {
            kind: RefKind::RemoteBranch {
                remote: remote.into(),
                name: name.into(),
            },
            target,
            visibility: gitpoulpi_core::domain::RefVisibility::Shown,
            is_head: false,
            upstream: None,
            tag_sync: None,
        }
    }

    /// 1 <- 2 <- 3 (main at 2, feature at 3), and 4 branching from 1.
    fn fixture() -> (Vec<Ref>, CommitGraph) {
        let mut main = local_branch_at("main", oid(2));
        main.is_head = true;
        main.upstream = Some("origin/main".into());
        let refs = vec![
            main,
            local_branch_at("feature", oid(3)),
            local_branch_at("side", oid(4)),
            remote_at("origin", "main", oid(1)),
        ];
        let graph = CommitGraph {
            nodes: vec![
                commit(4, &[1]),
                commit(3, &[2]),
                commit(2, &[1]),
                commit(1, &[]),
            ],
        };
        (refs, graph)
    }

    fn labels(entries: &[context_menu::Entry<RefContextAction>]) -> Vec<String> {
        entries
            .iter()
            .filter_map(|e| match e {
                context_menu::Entry::Action { label, .. } => Some(label.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn a_local_on_a_local_offers_merge_rebase_interactive_and_a_fast_forward() {
        let (refs, graph) = fixture();
        let entries = drop_menu_entries(
            "feature",
            RefRowKind::Local,
            "main",
            RefRowKind::Local,
            &refs,
            Some(&graph),
            Default::default(),
        );
        let text = entries_text(&entries);
        assert!(text.contains("Merge feature into main"), "{text}");
        assert!(text.contains("Rebase feature onto main"), "{text}");
        assert!(
            text.contains("Interactive rebase feature onto main"),
            "{text}"
        );
        // main (2) is an ancestor of feature (3): a real fast-forward.
        assert!(text.contains("Fast-forward main to feature"), "{text}");
        assert!(
            labels(&entries).iter().any(|l| l.contains("Fast-forward")),
            "offered, not grayed: {text}"
        );
    }

    fn entries_text(entries: &[context_menu::Entry<RefContextAction>]) -> String {
        format!("{entries:?}")
    }

    #[test]
    fn a_diverged_fast_forward_is_grayed_out_not_offered() {
        let (refs, graph) = fixture();
        let entries = drop_menu_entries(
            "side",
            RefRowKind::Local,
            "main",
            RefRowKind::Local,
            &refs,
            Some(&graph),
            Default::default(),
        );
        assert!(
            !labels(&entries).iter().any(|l| l.contains("Fast-forward")),
            "diverged branches must not offer a live fast-forward: {}",
            entries_text(&entries)
        );
        assert!(entries_text(&entries).contains("diverged"));
    }

    #[test]
    fn a_local_on_a_remote_offers_only_push_and_an_invalid_pair_nothing() {
        let (refs, graph) = fixture();
        let push = drop_menu_entries(
            "main",
            RefRowKind::Local,
            "origin/main",
            RefRowKind::Remote,
            &refs,
            Some(&graph),
            Default::default(),
        );
        assert_eq!(labels(&push), vec!["\u{21E7} Push main to origin/main"]);
        assert!(drop_menu_entries(
            "main",
            RefRowKind::Local,
            "main",
            RefRowKind::Local,
            &refs,
            Some(&graph),
            Default::default(),
        )
        .is_empty());
    }

    #[test]
    fn a_remote_on_its_local_also_offers_pull_but_not_on_an_unrelated_one() {
        let (refs, graph) = fixture();
        let onto_main = labels(&drop_menu_entries(
            "origin/main",
            RefRowKind::Remote,
            "main",
            RefRowKind::Local,
            &refs,
            Some(&graph),
            Default::default(),
        ));
        assert!(onto_main.iter().any(|l| l.contains("Pull origin/main")));
        assert!(onto_main.iter().any(|l| l.contains("Merge origin/main")));
        let onto_feature = labels(&drop_menu_entries(
            "origin/main",
            RefRowKind::Remote,
            "feature",
            RefRowKind::Local,
            &refs,
            Some(&graph),
            Default::default(),
        ));
        assert!(!onto_feature.iter().any(|l| l.contains("Pull")));
    }

    fn plan(
        op: DropOp,
        s: &str,
        sk: RefRowKind,
        t: &str,
        tk: RefRowKind,
    ) -> Result<DropPlan, String> {
        let (refs, graph) = fixture();
        plan_drop(
            op,
            s,
            sk,
            t,
            tk,
            &refs,
            Some(&graph),
            &["origin".to_string()],
            PullButtonMode::Merge,
        )
    }

    #[test]
    fn merging_into_a_branch_that_is_not_checked_out_checks_it_out_first() {
        use RefRowKind::Local;
        // Into main (HEAD): straight merge.
        match plan(DropOp::Merge, "feature", Local, "main", Local) {
            Ok(DropPlan::Submit(Action::Merge(from), _)) => {
                assert_eq!(
                    from.kind,
                    RefKind::LocalBranch {
                        name: "feature".into()
                    }
                )
            }
            other => panic!("{other:?}"),
        }
        // Into feature (not HEAD): checkout feature, *then* merge.
        match plan(DropOp::Merge, "main", Local, "feature", Local) {
            Ok(DropPlan::Submit(Action::CheckoutThen { branch, then }, _)) => {
                assert_eq!(branch, "feature");
                assert!(matches!(*then, Action::Merge(_)));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn rebasing_x_onto_y_checks_x_out_and_replays_it_on_y() {
        use RefRowKind::Local;
        match plan(DropOp::Rebase, "feature", Local, "main", Local) {
            Ok(DropPlan::Submit(Action::CheckoutThen { branch, then }, said)) => {
                assert_eq!(branch, "feature");
                match *then {
                    Action::Rebase(onto) => {
                        assert_eq!(
                            onto.kind,
                            RefKind::LocalBranch {
                                name: "main".into()
                            }
                        )
                    }
                    other => panic!("{other:?}"),
                }
                assert!(said.contains("feature") && said.contains("main"), "{said}");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn fast_forward_moves_the_target_to_the_dragged_ref() {
        use RefRowKind::Local;
        match plan(DropOp::FastForward, "feature", Local, "main", Local) {
            Ok(DropPlan::Submit(
                Action::FastForward {
                    branch,
                    target,
                    target_label,
                },
                _,
            )) => {
                assert_eq!(
                    (branch.as_str(), target, target_label.as_str()),
                    ("main", oid(3), "feature")
                );
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn pushing_names_the_remote_split_from_the_dropped_ref() {
        use RefRowKind::{Local, Remote};
        match plan(DropOp::Push, "feature", Local, "origin/main", Remote) {
            Ok(DropPlan::Submit(Action::Push { remote, opts }, _)) => {
                assert_eq!(remote, "origin");
                assert_eq!(opts.branch.as_deref(), Some("feature"));
                // No upstream yet: the first push sets it.
                assert!(opts.set_upstream);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_pull_drop_on_the_checked_out_branch_pulls_without_a_checkout() {
        use RefRowKind::{Local, Remote};
        match plan(DropOp::Pull, "origin/main", Remote, "main", Local) {
            Ok(DropPlan::Submit(Action::Pull(_), said)) => {
                assert!(said.contains("main"), "{said}")
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn interactive_rebase_replays_only_the_commits_the_target_lacks_onto_it() {
        use RefRowKind::Local;
        // side (4) vs main (2): 4 is the only commit main lacks; base 1.
        match plan(DropOp::InteractiveRebase, "side", Local, "main", Local) {
            Ok(DropPlan::Rebase(form)) => {
                assert_eq!(form.upstream, Some(oid(1)));
                assert_eq!(form.onto, Some(oid(2)));
                assert_eq!(form.steps().len(), 1);
                assert_eq!(form.steps()[0].commit, oid(4));
                // side is not checked out: the popup must check it out.
                assert_eq!(form.checkout.as_deref(), Some("side"));
            }
            other => panic!("{other:?}"),
        }
        // feature (3) vs main (2): one commit, no `--onto` needed.
        match plan(DropOp::InteractiveRebase, "feature", Local, "main", Local) {
            Ok(DropPlan::Rebase(form)) => {
                assert_eq!(form.upstream, Some(oid(2)));
                assert_eq!(form.onto, None);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn interactive_rebase_refuses_an_empty_range_and_a_merge_in_the_way() {
        use RefRowKind::Local;
        // main is contained in feature: nothing to replay.
        let err = plan(DropOp::InteractiveRebase, "main", Local, "feature", Local)
            .expect_err("nothing to replay");
        assert!(err.contains("nothing"), "{err}");
        // A merge commit in the range.
        let (refs, _) = fixture();
        let graph = CommitGraph {
            nodes: vec![
                commit(3, &[2, 4]),
                commit(2, &[1]),
                commit(4, &[1]),
                commit(1, &[]),
            ],
        };
        let err = plan_drop(
            DropOp::InteractiveRebase,
            "feature",
            Local,
            "side",
            Local,
            &refs,
            Some(&graph),
            &[],
            PullButtonMode::Merge,
        );
        assert!(
            matches!(&err, Err(e) if e.contains("merge commit")),
            "{err:?}"
        );
    }

    #[test]
    fn a_vanished_ref_is_reported_not_acted_on() {
        use RefRowKind::Local;
        let err = plan(DropOp::Merge, "gone", Local, "main", Local).expect_err("gone");
        assert!(err.contains("gone"), "{err}");
    }
}
