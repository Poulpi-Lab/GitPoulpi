//! Text that is not a translatable catalog string.
//!
//! Most of these are literals in the terminal UI as well: git syntax (`stash@{n}`), the WIP row's
//! placeholder, the sort glyphs. [`NOT_AVAILABLE`] is the one real gap: no catalog key says that an
//! action is not wired up yet in this frontend.

/// The WIP row's message cell (the terminal UI draws the same literal).
pub const WIP_PLACEHOLDER: &str = "// WIP";

/// How git names a stash: the label of a stash row and of a stash in the Refs sidebar.
pub fn stash_name(id: u32) -> String {
    format!("stash@{{{id}}}")
}

/// Sort toggle of the file list (the terminal UI's `SORT_LABEL`, without its padding).
pub fn sort_label(ascending: bool) -> &'static str {
    if ascending {
        "A\u{2193}Z"
    } else {
        "Z\u{2193}A"
    }
}

/// Shown when a toolbar action is clicked: those buttons are not connected to git operations yet.
pub const NOT_AVAILABLE: &str =
    "Read-only for now: this action is not available in the desktop frontend yet";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stash_name_uses_git_syntax() {
        assert_eq!(stash_name(2), "stash@{2}");
    }

    #[test]
    fn sort_label_flips() {
        assert_eq!(sort_label(true), "A\u{2193}Z");
        assert_eq!(sort_label(false), "Z\u{2193}A");
    }
}
