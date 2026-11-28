//! Drag and drop of refs: what dropping one ref on another offers, and what each offer does.

use super::*;

// -- drag and drop ----------------------------------------------------------------------------------

/// One choice of the menu a drop opens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropOp {
    Merge,
    Rebase,
    FastForward,
}

/// Whether `source` dropped on `target` offers anything: local on local, remote on local.
pub fn drop_is_valid(source: &RefTarget, target: &RefTarget) -> bool {
    if source == target {
        return false;
    }
    matches!(
        (source, target),
        (RefTarget::Local(_), RefTarget::Local(_)) | (RefTarget::Remote(_), RefTarget::Local(_))
    )
}

/// The rows a drop offers, with their labels: `(op, label, why it is refused)`.
pub fn drop_rows(
    snap: &Snapshot,
    source: &RefTarget,
    target: &RefTarget,
) -> Vec<(DropOp, String, Option<String>)> {
    if !drop_is_valid(source, target) {
        return Vec::new();
    }
    let (s, t) = (source.name(), target.name());
    let same = snap.oid_of(source).is_some() && snap.oid_of(source) == snap.oid_of(target);
    let ff_why = same.then(|| msg::menu_ref::FF_ALREADY_THERE.get().to_string());
    let rebase = match source {
        RefTarget::Remote(_) => msg::menu_drop::rebase_target_onto(&t, &s),
        _ => msg::menu_drop::rebase_onto(&s, &t),
    };
    vec![
        (DropOp::Merge, msg::menu_drop::merge_into(&s, &t), None),
        (DropOp::Rebase, rebase, None),
        (
            DropOp::FastForward,
            msg::menu_ref::fast_forward(&t, &s),
            ff_why,
        ),
    ]
}

/// Anything that changes a branch other than the checked-out one first checks it out (the checkout
/// stops the rest when it is refused).
fn after_checkout(snap: &Snapshot, branch: &str, then: Action) -> Action {
    if snap.head().map(|h| h.0) == Some(branch) {
        then
    } else {
        Action::CheckoutThen {
            branch: branch.to_string(),
            then: Box::new(then),
        }
    }
}

/// What a chosen drop row does.
pub fn drop_plan(snap: &Snapshot, op: DropOp, source: &RefTarget, target: &RefTarget) -> Plan {
    use msg::menu_drop as m;
    let (s, t) = (source.name(), target.name());
    let (Some(src), Some(tgt)) = (snap.find(source), snap.find(target)) else {
        return Plan::Say(msg::git_actions::gone(&s, ""));
    };
    match (op, source, target) {
        (DropOp::Merge, _, RefTarget::Local(_)) => Plan::submit(
            after_checkout(snap, &t, Action::Merge(src.clone())),
            m::merging(&s, &t),
        ),
        (DropOp::Rebase, RefTarget::Local(_), RefTarget::Local(_)) => Plan::submit(
            after_checkout(snap, &s, Action::Rebase(tgt.clone())),
            m::rebasing_source(&s, &t),
        ),
        (DropOp::Rebase, RefTarget::Remote(_), RefTarget::Local(_)) => Plan::submit(
            after_checkout(snap, &t, Action::Rebase(src.clone())),
            m::rebasing_target(&t, &s),
        ),
        (DropOp::FastForward, _, RefTarget::Local(_)) => Plan::submit(
            Action::FastForward {
                branch: t.clone(),
                target: src.target,
                target_label: s.clone(),
            },
            msg::menu_ref::fast_forwarding(&t, &s),
        ),
        _ => Plan::Say(m::cannot_drop(&s, &t)),
    }
}
