//! Fixtures shared by more than one module's `#[cfg(test)] mod tests`:
//! commits, oids, working trees, refs and similar builders.

use super::*;

pub(crate) fn oid(n: u8) -> Oid {
    Oid([n; 20])
}

pub(crate) fn commit(id: u8, parents: &[u8]) -> CommitNode {
    CommitNode {
        id: oid(id),
        parents: parents.iter().map(|&p| oid(p)).collect(),
        summary: format!("commit {id}"),
        message: format!("commit {id}"),
        author_name: "a".into(),
        author_email: "a@example.com".into(),
        timestamp: 0,
        refs: Vec::new(),
    }
}

pub(crate) fn local_branch_at(name: &str, target: Oid) -> gitpoulpi_core::domain::Ref {
    gitpoulpi_core::domain::Ref {
        kind: RefKind::LocalBranch { name: name.into() },
        target,
        visibility: gitpoulpi_core::domain::RefVisibility::Shown,
        is_head: false,
        upstream: None,
        tag_sync: None,
        multi_remote: None,
    }
}

pub(crate) fn remote_branch_at(
    remote: &str,
    name: &str,
    target: Oid,
) -> gitpoulpi_core::domain::Ref {
    gitpoulpi_core::domain::Ref {
        kind: RefKind::RemoteBranch {
            remote: remote.into(),
            name: name.into(),
        },
        target,
        visibility: gitpoulpi_core::domain::RefVisibility::Shown,
        is_head: false,
        upstream: None,
        tag_sync: None,
        multi_remote: None,
    }
}
