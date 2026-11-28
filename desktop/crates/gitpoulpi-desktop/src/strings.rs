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

/// The reason shown for a command the desktop app does not implement yet (palette row, disabled menu
/// item, status line).
pub const NOT_AVAILABLE_YET: &str = "Not available yet in the desktop app";

/// The status line's answer when `what` is run but not implemented yet.
pub fn not_available_yet(what: &str) -> String {
    if what.is_empty() {
        NOT_AVAILABLE_YET.to_string()
    } else {
        format!("{what} - {}", NOT_AVAILABLE_YET.to_lowercase())
    }
}

// Menu bar headings that have no catalog string.
pub const MENU_FILE: &str = "File";
pub const MENU_EDIT: &str = "Edit";
pub const MENU_VIEW: &str = "View";
pub const MENU_REPOSITORY: &str = "Repository";
pub const MENU_GIT: &str = "Git";
pub const MENU_TOOLS: &str = "Tools";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stash_name_uses_git_syntax() {
        assert_eq!(stash_name(2), "stash@{2}");
    }

    #[test]
    fn the_status_line_names_the_command() {
        assert_eq!(
            not_available_yet("Git: Fetch"),
            "Git: Fetch - not available yet in the desktop app"
        );
        assert_eq!(not_available_yet(""), NOT_AVAILABLE_YET);
    }

    #[test]
    fn sort_label_flips() {
        assert_eq!(sort_label(true), "A\u{2193}Z");
        assert_eq!(sort_label(false), "Z\u{2193}A");
    }
}
