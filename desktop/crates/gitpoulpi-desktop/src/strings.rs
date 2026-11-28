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

/// Shown when a toolbar button is clicked whose git operation the desktop does not run yet.
pub const NOT_AVAILABLE: &str = "This action is not available in the desktop frontend yet";

/// The link asked this window to come to the front (an agent's request); the desktop does not do that yet.
pub const NO_RAISE: &str = "this window cannot be raised on request yet";

/// Push with HEAD on no branch.
pub const PUSH_DETACHED: &str = "Nothing to push: HEAD is not on a branch";
/// Push in a repository without remotes.
pub const PUSH_NO_REMOTE: &str = "Nothing to push to: the repository has no remote";
/// Push of a branch without upstream when several remotes exist and none is `origin`.
pub const PUSH_CHOOSE_REMOTE: &str =
    "This branch has no upstream and the repository has several remotes: push it once from the terminal UI or the CLI to choose";

/// The staging guard found files that look like secrets and the desktop has no dialog for it yet.
pub fn sensitive_stage(files: usize) -> String {
    format!(
        "Not staged: {files} file(s) look like secrets. Stage them from the terminal UI or the CLI to confirm"
    )
}

/// Status line after an action finished: `title` is the button's label.
pub fn finished(title: &str) -> String {
    format!("{title}: done")
}

/// Status line while `n` daemon jobs run.
pub fn jobs_running(n: usize) -> String {
    format!("{n} job(s) running")
}

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
