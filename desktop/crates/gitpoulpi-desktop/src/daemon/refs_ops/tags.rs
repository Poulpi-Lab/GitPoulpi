//! Tags: where a tag lives, the delete question, pushing, and the qualified spelling.

use super::*;

// -- tags -------------------------------------------------------------------------------------------

/// The remote tags go to: `origin`, else the only remote.
pub fn default_remote(snap: &Snapshot) -> Option<String> {
    let remotes = snap.remote_names();
    if remotes.iter().any(|r| r == "origin") {
        return Some("origin".into());
    }
    (remotes.len() == 1).then(|| remotes[0].clone())
}

/// Push one tag to the default remote.
pub fn push_tag_plan(snap: &Snapshot, name: &str) -> Plan {
    match default_remote(snap) {
        Some(remote) => {
            let said = msg::menu_ref::pushing(name, &remote);
            Plan::submit(
                Action::TagPush {
                    remote,
                    selection: TagSelection::Named(vec![name.to_string()]),
                },
                said,
            )
        }
        None => Plan::Say(msg::input::NO_DEFAULT_REMOTE.get().to_string()),
    }
}

/// Push every tag no remote has.
pub fn push_all_tags_plan(snap: &Snapshot) -> Plan {
    match default_remote(snap) {
        Some(remote) => Plan::submit(
            Action::TagPush {
                remote: remote.clone(),
                selection: TagSelection::AllUnpushed,
            },
            msg::menu_ref::pushing(msg::menu_ref::PUSH_ALL_TAGS.get(), &remote),
        ),
        None => Plan::Say(msg::input::NO_DEFAULT_REMOTE.get().to_string()),
    }
}

/// `refs/heads/main` from a target.
pub fn qualified_ref_name(target: &RefTarget) -> Option<String> {
    let name = target.name();
    match target {
        RefTarget::Local(_) => Some(format!("refs/heads/{name}")),
        RefTarget::Remote(_) => Some(format!("refs/remotes/{name}")),
        RefTarget::Tag(_) => Some(format!("refs/tags/{name}")),
        _ => None,
    }
}
