//! Staging and committing as link actions: which [`Action`] a selection, a hunk or the commit form turns
//! into, the confirmation texts and the commit form's rules.
//!
//! The rules are the terminal UI's (its `composer.rs` and `status_pane.rs` keep them private, so they
//! are restated here): a commit needs a staged file (except while amending) and a summary, a conflicted
//! file blocks it, the summary and description are joined the way the history shows them, and the
//! repository's comment-stripping setting applies. One submitted action is one journal entry, so a
//! selection goes out as one `StagePaths` / `UnstagePaths` / `DiscardPaths`, never as one action per file.

use std::path::PathBuf;

use gitpoulpi_core::domain::{FileStatus, HunkId, LineSelector, StatusEntry};
use gitpoulpi_i18n::msg;
use gitpoulpi_link::commit_settings::CommitSettings;
use gitpoulpi_link::Action;

use super::Backend;

/// The longest summary line before the counter turns red (hosting sites wrap at 72 columns).
pub const SUMMARY_LIMIT: usize = 72;
/// From this many characters left the counter turns amber.
pub const SUMMARY_WARN_REMAINING: i64 = 10;

/// Characters left of the summary budget; negative once over it.
pub fn summary_remaining(summary_chars: usize) -> i64 {
    SUMMARY_LIMIT as i64 - summary_chars as i64
}

/// Stage (`stage`) or unstage `paths` as one action. A renamed file's partner path is added by the link
/// (`DaemonLink::submit_action`), so the row acts on both names.
pub fn stage_action(paths: &[PathBuf], stage: bool) -> Option<Action> {
    match (paths, stage) {
        ([], _) => None,
        ([one], true) => Some(Action::StageFile(one.clone())),
        ([one], false) => Some(Action::UnstageFile(one.clone())),
        (many, true) => Some(Action::StagePaths {
            paths: many.to_vec(),
            rename: false,
        }),
        (many, false) => Some(Action::UnstagePaths {
            paths: many.to_vec(),
            rename: false,
        }),
    }
}

/// Throws the uncommitted changes of `paths` away: one action, one journal entry.
pub fn discard_action(paths: &[PathBuf]) -> Option<Action> {
    (!paths.is_empty()).then(|| Action::DiscardPaths(paths.to_vec()))
}

/// What a hunk or line action does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PartOp {
    Stage,
    Unstage,
    Discard,
}

/// A whole hunk, or `lines` of it (indices into the hunk's own lines, as the diff rows number them).
pub fn part_action(path: PathBuf, hunk: HunkId, lines: Option<Vec<u32>>, op: PartOp) -> Action {
    match (lines, op) {
        (None, PartOp::Stage) => Action::StageHunk { path, hunk },
        (None, PartOp::Unstage) => Action::UnstageHunk { path, hunk },
        (None, PartOp::Discard) => Action::DiscardHunk { path, hunk },
        (Some(line_indices), op) => {
            let selector = LineSelector { hunk, line_indices };
            match op {
                PartOp::Stage => Action::StageLines { path, selector },
                PartOp::Unstage => Action::UnstageLines { path, selector },
                PartOp::Discard => Action::DiscardLines { path, selector },
            }
        }
    }
}

/// The question before a discard: the count and, when some are untracked, how many.
pub fn discard_prompt(entries: &[StatusEntry], paths: &[PathBuf]) -> String {
    let untracked = paths
        .iter()
        .filter(|p| {
            entries
                .iter()
                .any(|e| &e.path == *p && e.worktree == Some(FileStatus::Untracked))
        })
        .count();
    let n = paths.len() as u64;
    if untracked == 0 {
        msg::status_pane::discard_prompt(n)
    } else {
        msg::status_pane::discard_prompt_untracked(n, untracked)
    }
}

/// Why the commit button cannot commit yet, in the terminal UI's order: a conflict first, then nothing
/// staged (unless amending), then an empty summary. The text is the button's own label.
pub fn commit_readiness(
    summary: &str,
    amend: bool,
    staged: usize,
    conflicts: usize,
) -> Result<(), &'static str> {
    if conflicts > 0 {
        return Err(msg::conflict_guard::NEED_RESOLVE.get());
    }
    if !amend && staged == 0 {
        return Err(msg::composer::NEED_STAGED.get());
    }
    if summary.trim().is_empty() {
        return Err(msg::composer::NEED_SUMMARY.get());
    }
    Ok(())
}

/// The commit message: summary, a blank line, then the description; the repository's comment lines
/// removed when that setting is on. `None` when nothing is left.
pub fn commit_message(
    summary: &str,
    description: &str,
    settings: &CommitSettings,
) -> Option<String> {
    let summary = summary.trim_end();
    let message = if description.trim().is_empty() {
        summary.to_string()
    } else {
        format!("{summary}\n\n{}", description.trim_end())
    };
    let message = settings.prepared(&message);
    (!message.trim().is_empty()).then_some(message)
}

/// The `Commit` action of the form.
pub fn commit_action(
    message: String,
    amend: bool,
    no_verify: bool,
    settings: &CommitSettings,
) -> Action {
    Action::Commit {
        message,
        amend,
        no_verify,
        push: settings.push_after_commit,
    }
}

/// This repository's commit settings (template, comment stripping, skip hooks by default). A small file
/// read: call it when the working tree is shown and after a commit, never per frame.
pub fn commit_settings(root: &std::path::Path, config: &RepoConfig) -> CommitSettings {
    CommitSettings::load(root, config)
}

/// The repository-local settings the commit form reads (`user.name`, `user.email`, `commit.template`,
/// `core.commentChar`), by key.
pub type RepoConfig = std::collections::BTreeMap<String, Option<String>>;

/// "Name <email>" from the configured identity, when there is one.
pub fn author_line(config: &RepoConfig) -> Option<String> {
    let get = |key: &str| {
        config
            .get(key)
            .and_then(|v| v.as_deref())
            .map(str::trim)
            .filter(|v| !v.is_empty())
    };
    match (
        get(gitpoulpi_core::repo_config::USER_NAME),
        get(gitpoulpi_core::repo_config::USER_EMAIL),
    ) {
        (Some(name), Some(email)) => Some(format!("{name} <{email}>")),
        (Some(name), None) => Some(name.to_string()),
        (None, Some(email)) => Some(email.to_string()),
        (None, None) => None,
    }
}

impl Backend {
    /// The repository's commit-related settings.
    pub fn repo_config(
        &self,
    ) -> impl std::future::Future<Output = Result<RepoConfig, String>> + Send + 'static {
        self.call(
            gitpoulpi_core::domain::OperationKind::RepoConfig,
            serde_json::json!({}),
        )
    }

    /// Submits `action` and returns the ticket its outcome is reported under (see `RightPanel::settle`).
    pub fn submit_tracked(&self, action: Action) -> gitpoulpi_link::Ticket {
        self.with_link(|link| link.submit_tracked(action))
    }

    /// Submits `action` without waiting for it: failures reach the status line through the link.
    pub fn submit(&self, action: Action) {
        self.with_link(|link| link.submit_action(action));
    }

    /// Tells the link which unstaged and staged files are one rename, so a row acts on both names.
    pub fn set_rename_pairs(&self, files: &[StatusEntry]) {
        let tree = gitpoulpi_core::domain::WorkingTree {
            files: files.to_vec(),
            repo_auth: None,
        };
        self.with_link(|link| link.set_rename_pairs(&tree));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> PathBuf {
        PathBuf::from(s)
    }

    fn entry(path: &str, worktree: Option<FileStatus>) -> StatusEntry {
        StatusEntry {
            path: p(path),
            index: None,
            worktree,
            renamed_from: None,
        }
    }

    #[test]
    fn a_selection_is_one_action() {
        assert_eq!(stage_action(&[], true), None);
        assert_eq!(
            stage_action(&[p("a")], true),
            Some(Action::StageFile(p("a")))
        );
        assert_eq!(
            stage_action(&[p("a")], false),
            Some(Action::UnstageFile(p("a")))
        );
        assert_eq!(
            stage_action(&[p("a"), p("b")], true),
            Some(Action::StagePaths {
                paths: vec![p("a"), p("b")],
                rename: false
            })
        );
        assert!(matches!(
            stage_action(&[p("a"), p("b")], false),
            Some(Action::UnstagePaths { .. })
        ));
        assert_eq!(discard_action(&[]), None);
        assert_eq!(
            discard_action(&[p("a"), p("b")]),
            Some(Action::DiscardPaths(vec![p("a"), p("b")]))
        );
    }

    #[test]
    fn hunks_and_lines_map_to_their_actions() {
        let h = HunkId(2);
        assert_eq!(
            part_action(p("f"), h, None, PartOp::Stage),
            Action::StageHunk {
                path: p("f"),
                hunk: h
            }
        );
        assert!(matches!(
            part_action(p("f"), h, None, PartOp::Discard),
            Action::DiscardHunk { .. }
        ));
        match part_action(p("f"), h, Some(vec![1, 3]), PartOp::Unstage) {
            Action::UnstageLines { selector, .. } => {
                assert_eq!(selector.line_indices, vec![1, 3]);
                assert_eq!(selector.hunk, h);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn the_discard_question_counts_untracked_files() {
        let entries = [
            entry("a", Some(FileStatus::Modified)),
            entry("b", Some(FileStatus::Untracked)),
        ];
        let plain = discard_prompt(&entries, &[p("a")]);
        assert!(
            plain.contains("1 file") && !plain.contains("untracked"),
            "{plain}"
        );
        let both = discard_prompt(&entries, &[p("a"), p("b")]);
        assert!(
            both.contains("2 files") && both.contains("1 untracked"),
            "{both}"
        );
    }

    #[test]
    fn readiness_follows_the_terminal_uis_order() {
        assert!(commit_readiness("x", false, 1, 0).is_ok());
        assert_eq!(
            commit_readiness("x", false, 0, 0),
            Err(msg::composer::NEED_STAGED.get())
        );
        assert!(
            commit_readiness("x", true, 0, 0).is_ok(),
            "amend needs nothing staged"
        );
        assert_eq!(
            commit_readiness("  ", false, 1, 0),
            Err(msg::composer::NEED_SUMMARY.get())
        );
        assert_eq!(
            commit_readiness("x", false, 1, 2),
            Err(msg::conflict_guard::NEED_RESOLVE.get()),
            "a conflict blocks everything"
        );
    }

    #[test]
    fn the_message_joins_summary_and_body_and_strips_comments_when_asked() {
        let mut settings = CommitSettings::default();
        assert_eq!(
            commit_message("Fix it", "", &settings).as_deref(),
            Some("Fix it")
        );
        assert_eq!(
            commit_message("Fix it", "because\n", &settings).as_deref(),
            Some("Fix it\n\nbecause")
        );
        settings.strip_comments = true;
        let stripped = commit_message("Fix it", "# note\nbody", &settings).unwrap();
        assert!(
            !stripped.contains("# note") && stripped.contains("body"),
            "{stripped}"
        );
        assert_eq!(commit_message("# only", "", &settings), None);
    }

    #[test]
    fn the_author_reads_name_and_email() {
        let mut c = RepoConfig::new();
        assert_eq!(author_line(&c), None);
        c.insert("user.name".into(), Some("Ada".into()));
        assert_eq!(author_line(&c).as_deref(), Some("Ada"));
        c.insert("user.email".into(), Some("ada@example.com".into()));
        assert_eq!(author_line(&c).as_deref(), Some("Ada <ada@example.com>"));
    }

    #[test]
    fn the_counter_counts_down_from_the_limit() {
        assert_eq!(summary_remaining(0), 72);
        assert_eq!(summary_remaining(80), -8);
    }
}
