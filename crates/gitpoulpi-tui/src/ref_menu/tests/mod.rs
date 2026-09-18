use super::*;
use crate::test_support::*;

#[cfg(feature = "worktree")]
mod create_actions;
mod delete;
mod entries;
mod fast_forward;
mod i18n;
mod stash;

pub(super) fn row_state(
    entries: &[context_menu::Entry<RefContextAction>],
    prefix: &str,
) -> Option<Result<RefContextAction, String>> {
    entries.iter().find_map(|e| match e {
        context_menu::Entry::Action { label, action, .. } if label.contains(prefix) => {
            Some(Ok(*action))
        }
        context_menu::Entry::Disabled { label } if label.contains(prefix) => {
            Some(Err(label.clone()))
        }
        _ => None,
    })
}
