//! Text that is not a translatable catalog string.
//!
//! Most of these are literals in the terminal UI as well: git syntax (`stash@{n}`), the WIP row's
//! placeholder, the sort glyphs, and a few sentences no catalog key words yet (an action not wired up
//! in this frontend, why a push has nowhere to go).

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

/// The link asked this window to come to the front (an agent's request); the desktop does not do that yet.
pub const NO_RAISE: &str = "this window cannot be raised on request yet";

/// Push with HEAD on no branch.
pub const PUSH_DETACHED: &str = "Nothing to push: HEAD is not on a branch";
/// Push in a repository without remotes.
pub const PUSH_NO_REMOTE: &str = "Nothing to push to: the repository has no remote";
/// Push of a branch without upstream when several remotes exist and none is `origin`.
pub const PUSH_CHOOSE_REMOTE: &str =
    "This branch has no upstream and the repository has several remotes: push it once from the terminal UI or the CLI to choose";

/// Status line after an action finished: `title` is the button's label.
pub fn finished(title: &str) -> String {
    format!("{title}: done")
}

/// Status line while `n` daemon jobs run.
pub fn jobs_running(n: usize) -> String {
    format!("{n} job(s) running")
}

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
