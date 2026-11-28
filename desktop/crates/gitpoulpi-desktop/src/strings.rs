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

// Branch, tag and stash forms.
/// The check box of the New branch form.
pub const CHECKOUT_AFTER_CREATE: &str = "Check out the new branch";
/// The message field of the New tag form (a message makes the tag annotated).
pub const TAG_MESSAGE_OPTIONAL: &str = "Message (optional: a message makes it an annotated tag)";
/// The message field of an annotated tag.
pub const TAG_MESSAGE: &str = "Message";
/// The upstream field of the Set upstream form.
pub const UPSTREAM_LABEL: &str = "Upstream (a remote branch, for example origin/main)";
/// The line under the upstream field.
pub const UPSTREAM_EMPTY_UNSETS: &str = "Leave the field empty to remove the upstream.";
/// The message field of the Stash form.
pub const STASH_MESSAGE_LABEL: &str = "Message (optional)";
/// Show changes of a stash that holds none.
pub const STASH_EMPTY: &str = "This stash has no changes to show";
/// The name field of the forms.
pub const NAME_LABEL: &str = "Name";

// Worktrees, submodules, remotes, LFS and file commands.
/// The button that closes a read-only list.
pub const CLOSE: &str = "Close";
/// The pattern field of the LFS track and untrack forms.
pub const LFS_PATTERN_LABEL: &str = "Pattern (for example *.psd)";
/// A pattern typed in an LFS form that is empty.
pub const LFS_PATTERN_NEEDED: &str = "type a pattern, for example *.psd";
/// The folder field of the Create worktree form.
pub const FOLDER_LABEL: &str = "Folder";
/// The branch field of the Create worktree form.
pub const WORKTREE_BRANCH_LABEL: &str =
    "Worktree branch (a new name, an existing branch to check out, or empty)";
/// The check box of the Create worktree form.
pub const OPEN_WORKTREE_AFTER: &str = "Open the new worktree after creating it";
/// Why a remote command has no remote.
pub const NO_REMOTE_TO_ACT_ON: &str = "this repository has no remote";
/// The name field of the Add and Edit remote forms.
pub const REMOTE_NAME_LABEL: &str = "Name";
/// The fetch URL field.
pub const REMOTE_FETCH_URL_LABEL: &str = "Fetch URL";
/// The push URL field.
pub const REMOTE_PUSH_URL_LABEL: &str = "Push URL (empty: same as the fetch URL)";
/// A submodule command run on a row that is not a submodule.
pub const SELECT_SUBMODULE: &str = "select a submodule in the Refs pane first";
/// A worktree command run with no worktree selected.
pub const SELECT_WORKTREE: &str = "select a worktree in the Refs pane first";
/// A file command run with no file selected.
pub const SELECT_FILE: &str = "select a file in the working changes or in a commit's files first";
/// Why the file command needs a file on disk.
pub const FILE_NOT_ON_DISK: &str = "the file is not on disk";
/// Running an external program failed.
pub fn launch_failed(what: &str, why: &str) -> String {
    format!("could not start {what}: {why}")
}
/// Status line after an external program started.
pub fn launched(what: &str) -> String {
    format!("opened {what}")
}
/// The LFS menu's heading.
pub const LFS_MENU_TITLE: &str = "Git LFS";
/// The status line when LFS is not initialised in the repository.
pub const LFS_NOT_INITIALIZED: &str = "Git LFS is not initialized in this repository";
/// The first line of the LFS status dialog.
pub fn lfs_files_line(n: usize) -> String {
    format!("{n} file(s) stored with LFS")
}
/// One tracked pattern line of the LFS status.
pub fn lfs_pattern_line(pattern: &str, lockable: bool) -> String {
    if lockable {
        format!("{pattern} (lockable)")
    } else {
        pattern.to_string()
    }
}
/// The line of the file history view that names the file.
pub fn history_title(path: &str) -> String {
    format!("History of {path}")
}
/// The title of the file blame view.
pub fn blame_title(path: &str) -> String {
    format!("Blame of {path}")
}
/// Why the file has no history.
pub const NO_FILE_HISTORY: &str = "this file has no history yet";
/// The Open in submenu's heading on a worktree.
pub const OPEN_IN: &str = "Open in";
/// A worktree menu without a configured Open in entry.
pub const OPEN_IN_NONE: &str = "Open in (no entry yet)";

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
