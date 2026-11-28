//! The reasons a row gives in place of its value that are the row's own words
//! (shown as is), not an unfinished setting's "coming" note.

use gitpoulpi_i18n::msg::{launcher as t, settings_general, settings_repo as repo};

fn is_template_match(reason: &str, template: &str) -> bool {
    // `template` has `\0` where the name goes.
    match template.split_once('\0') {
        Some((pre, post)) => {
            reason.len() >= pre.len() + post.len()
                && reason.starts_with(pre)
                && reason.ends_with(post)
        }
        None => reason == template,
    }
}

/// Whether `reason` is one of the Settings pages' own notes: the undo-off
/// note, the launcher page's, the installation rows', the alias and PATH
/// rows' and the sparse checkout page's.
pub fn is_note(reason: &str) -> bool {
    reason == settings_general::UNDO_OFF_NOTE.get()
        || [
            t::CHECKING,
            t::STATE_NONE,
            t::STATE_FOREIGN,
            t::WINDOWS_ONLY,
            t::NAME_PRIMARY,
            t::SUBMENU_UNAVAILABLE,
            t::ALIAS_SET_NAME,
            t::ALIAS_BAD_SPACE,
            t::ALIAS_BAD_SEPARATOR,
            t::ALIAS_BAD_CHAR,
            t::ALIAS_BAD_START,
            t::ALIAS_BAD_LONG,
            t::ALIAS_CREATED,
            t::PATH_STATE_NONE,
            t::PATH_STATE_ADDED,
            t::PATH_STATE_ON,
            repo::SPARSE_NEEDS_ENABLED,
            repo::SPARSE_NEEDS_PATHS,
        ]
        .iter()
        .any(|m| m.get() == reason)
        || is_template_match(reason, &t::alias_used_by("\0"))
}
