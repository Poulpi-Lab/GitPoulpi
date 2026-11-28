//! The commit form's rules, shared by every frontend: when the Commit button
//! is ready and what it says, and what a submit commits.
//!
//! Pure: a frontend passes what it knows (summary, description, staged count,
//! merge state, conflicts) and gets a decision back; it keeps the drawing, the
//! focus and the sending of the `Commit` action.

use gitpoulpi_i18n::msg::composer as msg;

/// Whether the composer is ready to commit, and if not, the reason to
/// show as the (disabled) button's own label - the same
/// register-a-hit-target-and-say-why convention as
/// `DiffBarButton::pending_reason`.
pub fn commit_readiness(
    summary: &str,
    amend: bool,
    staged_count: usize,
    merging: bool,
    conflicts: usize,
) -> Result<(), gitpoulpi_i18n::M> {
    // Git refuses to commit unmerged files, whatever else is staged.
    if conflicts > 0 {
        return Err(gitpoulpi_i18n::msg::conflict_guard::NEED_RESOLVE);
    }
    // `merging`: a merge is stopped, and its commit needs nothing staged
    // (the resolved result may equal HEAD, "take ours" everywhere).
    if !amend && !merging && staged_count == 0 {
        return Err(msg::NEED_STAGED);
    }
    if summary.trim().is_empty() {
        return Err(msg::NEED_SUMMARY);
    }
    Ok(())
}

/// The Commit button's own label: the reason it's disabled, "Amend
/// previous commit" in amend mode, or "Commit changes to N files"
/// otherwise.
pub fn commit_button_label(
    amend: bool,
    staged_count: usize,
    merging: bool,
    ready: Result<(), gitpoulpi_i18n::M>,
) -> String {
    match ready {
        Err(reason) => reason.get().to_string(),
        Ok(()) if amend => msg::AMEND.get().to_string(),
        Ok(()) if merging && staged_count == 0 => gitpoulpi_i18n::msg::merge::BANNER_COMMIT_MERGE
            .get()
            .to_string(),
        Ok(()) => msg::commit_files(staged_count as u64),
    }
}

/// What a submit of the composer decided.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitPlan {
    /// Nothing to commit: the message is empty, before or after the comment
    /// lines are stripped. The text is the status-line error.
    Rejected(String),
    /// Send `Action::Commit` with this message; `status` is the status-line
    /// text to show meanwhile ("Committing: ..." / "Amending: ...").
    Commit { message: String, status: String },
}

/// Decides a submit. Summary and description are joined git's own way -
/// summary, blank line, body - so `git log` renders them as it would any other
/// commit. `strip_comment_char` is `Some(c)` when "Drop comment lines when
/// making a commit" is on (`CommitSettings::strip_comments` with its
/// `comment_char`): the commit is made with `-m`, which git does not clean up,
/// so the comment lines go here.
pub fn plan_commit(
    summary: &str,
    description: &str,
    amend: bool,
    strip_comment_char: Option<char>,
) -> CommitPlan {
    if summary.trim().is_empty() {
        return CommitPlan::Rejected(msg::ERR_EMPTY_SUMMARY.get().into());
    }
    let message = if description.trim().is_empty() {
        summary.to_string()
    } else {
        format!("{summary}\n\n{description}")
    };
    let message = match strip_comment_char {
        Some(c) => gitpoulpi_core::repo_config::strip_comment_lines(&message, c),
        None => message,
    };
    if message.trim().is_empty() {
        return CommitPlan::Rejected(msg::ERR_EMPTY_SUMMARY.get().into());
    }
    let status = if amend {
        format!("Amending: {summary}")
    } else {
        format!("Committing: {summary}")
    };
    CommitPlan::Commit { message, status }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readiness_needs_staged_files_unless_amending_or_merging() {
        assert_eq!(
            commit_readiness("x", false, 0, false, 0),
            Err(msg::NEED_STAGED)
        );
        assert_eq!(commit_readiness("x", true, 0, false, 0), Ok(()));
        assert_eq!(commit_readiness("x", false, 0, true, 0), Ok(()));
    }

    #[test]
    fn readiness_needs_a_summary_and_no_conflicts() {
        assert_eq!(
            commit_readiness("  ", false, 2, false, 0),
            Err(msg::NEED_SUMMARY)
        );
        assert_eq!(
            commit_readiness("x", false, 2, false, 1),
            Err(gitpoulpi_i18n::msg::conflict_guard::NEED_RESOLVE)
        );
    }

    #[test]
    fn button_label_names_the_reason_or_the_action() {
        assert_eq!(
            commit_button_label(true, 0, false, Ok(())),
            msg::AMEND.get().to_string()
        );
        assert_eq!(
            commit_button_label(false, 3, false, Ok(())),
            msg::commit_files(3)
        );
        assert_eq!(
            commit_button_label(false, 0, false, Err(msg::NEED_STAGED)),
            msg::NEED_STAGED.get().to_string()
        );
    }

    #[test]
    fn the_message_is_summary_blank_line_body() {
        assert_eq!(
            plan_commit("fix: a", "why\nmore", false, None),
            CommitPlan::Commit {
                message: "fix: a\n\nwhy\nmore".into(),
                status: "Committing: fix: a".into()
            }
        );
        assert_eq!(
            plan_commit("fix: a", "  \n", true, None),
            CommitPlan::Commit {
                message: "fix: a".into(),
                status: "Amending: fix: a".into()
            }
        );
    }

    #[test]
    fn comment_lines_are_stripped_when_asked() {
        let CommitPlan::Commit { message, .. } =
            plan_commit("fix: a", "# note\nbody", false, Some('#'))
        else {
            panic!("expected a commit");
        };
        assert_eq!(message, "fix: a\n\nbody");
    }

    #[test]
    fn an_empty_message_is_rejected_before_or_after_stripping() {
        let empty = CommitPlan::Rejected(msg::ERR_EMPTY_SUMMARY.get().into());
        assert_eq!(plan_commit(" ", "body", false, None), empty);
        assert_eq!(plan_commit("# only", "", false, Some('#')), empty);
    }
}
