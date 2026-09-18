//! The repository's commit and merge behavior (Settings > Repository > Commit,
//! Merge options, Message template; plan `repo-settings.md`, slice 1).
//!
//! The switches live in the repo data directory (`RepoPreferences`); the
//! template is a file there that git's `commit.template` points at. This module
//! is the read side the window and the daemon link share: what the composer
//! starts with, what is stripped from a message, and the squash message git
//! leaves behind.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use gitpoulpi_session::RepoPreferences;

/// This repository's preferences as saved (defaults when there is no file).
/// A small file read: callers are action tasks and the window's reload on
/// Settings close, never a per-frame path.
pub(crate) fn repo_preferences(repo_root: &Path) -> RepoPreferences {
    gitpoulpi_session::RepoState::load(&crate::remotes_ui::repo_state_path(repo_root))
        .map(|s| s.preferences)
        .unwrap_or_default()
}

/// The commit-time switches, resolved: what the composer and the commit
/// submit read.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CommitSettings {
    pub(crate) push_after_commit: bool,
    /// The composer's Skip hooks option starts checked.
    pub(crate) skip_hooks_default: bool,
    /// An empty composer starts with the template.
    pub(crate) template_apply: bool,
    /// Comment lines are removed from the message when it is committed.
    pub(crate) strip_comments: bool,
    /// `core.commentChar`, `#` by default.
    pub(crate) comment_char: char,
    /// The template file (`commit.template`, else the repo data dir's own).
    pub(crate) template_path: PathBuf,
}

impl Default for CommitSettings {
    fn default() -> Self {
        Self {
            push_after_commit: false,
            skip_hooks_default: false,
            template_apply: false,
            strip_comments: false,
            comment_char: '#',
            template_path: PathBuf::new(),
        }
    }
}

impl CommitSettings {
    /// Reads the saved preferences and resolves the template file from the
    /// repository-local `commit.template` and `core.commentChar` the daemon
    /// last reported (`repo_config`).
    pub(crate) fn load(repo_root: &Path, repo_config: &BTreeMap<String, Option<String>>) -> Self {
        let prefs = repo_preferences(repo_root);
        let state_path = crate::remotes_ui::repo_state_path(repo_root);
        let state_dir = state_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default();
        let configured = repo_config
            .get(gitpoulpi_core::repo_config::COMMIT_TEMPLATE)
            .and_then(|v| v.as_deref());
        let home = std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from);
        let template_path = match configured.map(str::trim).filter(|v| !v.is_empty()) {
            Some(_) => gitpoulpi_session::template_path(repo_root, "", configured, home.as_deref()),
            None => state_dir.join(gitpoulpi_session::TEMPLATE_FILE),
        };
        Self {
            push_after_commit: prefs.push_after_commit.unwrap_or(false),
            skip_hooks_default: prefs.skip_hooks_by_default.unwrap_or(false),
            template_apply: prefs.template_apply.unwrap_or(false),
            strip_comments: prefs.template_strip_comments.unwrap_or(false),
            comment_char: gitpoulpi_core::repo_config::comment_char(
                repo_config
                    .get(gitpoulpi_core::repo_config::CORE_COMMENT_CHAR)
                    .and_then(|v| v.as_deref()),
            ),
            template_path,
        }
    }

    /// The template as the composer's `(summary, description)`; empty when
    /// there is none.
    pub(crate) fn template_fields(&self) -> (String, String) {
        gitpoulpi_session::split_template(&gitpoulpi_session::read_template(&self.template_path))
    }

    /// `message` as it is committed: without its comment lines when the
    /// setting is on.
    pub(crate) fn prepared(&self, message: &str) -> String {
        if self.strip_comments {
            gitpoulpi_core::repo_config::strip_comment_lines(message, self.comment_char)
        } else {
            message.to_string()
        }
    }
}

/// The message git left for the next commit after `merge --squash`
/// (`SQUASH_MSG`), with its modification time (to tell one squash from the
/// next), or `None`. Comment lines are dropped: a conflicted squash lists its
/// conflicts there.
pub(crate) fn squash_message(repo_root: &Path) -> Option<(std::time::SystemTime, String)> {
    let git_dir = gitpoulpi_core::paths::git_dir(repo_root)?;
    let path = git_dir.join("SQUASH_MSG");
    let modified = std::fs::metadata(&path).ok()?.modified().ok()?;
    let text = std::fs::read_to_string(&path).ok()?;
    let message = gitpoulpi_core::repo_config::strip_comment_lines(&text, '#');
    (!message.is_empty()).then_some((modified, message))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(strip: bool, comment: char) -> CommitSettings {
        CommitSettings {
            strip_comments: strip,
            comment_char: comment,
            ..CommitSettings::default()
        }
    }

    #[test]
    fn comments_are_stripped_only_when_the_setting_is_on() {
        let text = "fix: x\n\n# why?\nbody\n";
        assert_eq!(settings(false, '#').prepared(text), text);
        assert_eq!(settings(true, '#').prepared(text), "fix: x\n\nbody");
        assert_eq!(settings(true, ';').prepared(text), text.trim());
    }

    #[test]
    fn the_defaults_are_all_off() {
        let s = CommitSettings::default();
        assert!(!s.push_after_commit && !s.skip_hooks_default);
        assert!(!s.template_apply && !s.strip_comments);
        assert_eq!(s.comment_char, '#');
    }
}
