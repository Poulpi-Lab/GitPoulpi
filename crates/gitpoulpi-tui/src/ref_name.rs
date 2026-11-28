//! A ref's lookup name (pure; re-exported by `refs_sidebar`).

use gitpoulpi_core::domain::{Ref, RefKind};

/// A ref's display/lookup name exactly as `group_refs` formats it (a local
/// branch or tag's bare name, a remote branch's `remote/name`), shared so
/// `hidden_from_graph`'s keys (built from `RefTreeRow::Leaf::full_name`)
/// match what this computes from a raw `Ref` when filtering
/// `refs_for_pills`.
pub(crate) fn ref_full_name(r: &Ref) -> String {
    match &r.kind {
        RefKind::LocalBranch { name } => name.clone(),
        RefKind::RemoteBranch { remote, name } => format!("{remote}/{name}"),
        RefKind::Tag { name } => name.clone(),
    }
}
