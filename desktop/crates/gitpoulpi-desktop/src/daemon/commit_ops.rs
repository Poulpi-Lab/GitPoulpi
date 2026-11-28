//! The Log's commit operations as link actions: cherry-pick, revert, reset, detached checkout, the
//! operation banner's Abort and Skip, and the comparison reads.
//!
//! One submitted action is one journal entry, so each of these is one Undo. A cherry-pick of several
//! commits goes out as ONE daemon request (the commits oldest first), so it is one entry too; the link's
//! `Action::CherryPick` takes a single commit and would make one entry per commit.

use gitpoulpi_app::palette::vocab::log_menu::CommitField;
use gitpoulpi_core::domain::{
    CherryPickOptions, CherryPickOutcome, CommitNode, DiffTarget, FileDiff, Oid, OperationKind,
    ResetMode,
};
use gitpoulpi_i18n::msg::menu_log as msg;
use gitpoulpi_link::Action;

use super::Backend;

/// The length of a short SHA everywhere in the interface (the Log column, messages, the clipboard).
pub const SHORT_SHA_LEN: usize = 7;

/// A commit id as the Log's SHA column shows it.
pub fn short(oid: Oid) -> String {
    oid.to_hex()[..SHORT_SHA_LEN].to_string()
}

/// One commit cherry-picked onto HEAD.
pub fn cherry_pick_action(commit: Oid) -> Action {
    Action::CherryPick(commit)
}

/// The inverse of `commit` committed on HEAD; `mainline` (1-based) for a merge commit.
pub fn revert_action(commit: Oid, mainline: Option<u32>) -> Action {
    Action::Revert { commit, mainline }
}

/// The current branch moved to `target`.
pub fn reset_action(target: Oid, mode: ResetMode) -> Action {
    Action::Reset { target, mode }
}

/// HEAD detached onto `commit`.
pub fn checkout_action(commit: Oid) -> Action {
    Action::CheckoutCommit(commit)
}

/// The banner's Abort (merge, cherry-pick, revert).
pub fn abort_action() -> Action {
    Action::OperationAbort
}

/// The banner's Skip (cherry-pick, revert).
pub fn skip_action() -> Action {
    Action::OperationSkip
}

/// The question a revert asks first (a revert rewrites the working tree from the commit).
pub fn revert_prompt(commit: Oid, mainline: Option<u32>) -> String {
    match mainline {
        Some(n) => msg::confirm_revert_merge(short(commit), n),
        None => msg::confirm_revert(short(commit)),
    }
}

/// The question before `reset --hard`; soft and mixed only move the branch pointer and never ask.
pub fn reset_prompt(target: Oid, mode: ResetMode) -> Option<String> {
    (mode == ResetMode::Hard).then(|| msg::confirm_reset_hard(short(target)))
}

/// The status-line word of a reset mode while it runs.
pub fn mode_word(mode: ResetMode) -> String {
    format!("{mode:?}")
}

/// The noun of a copied field, as the Copy submenu and the status line spell it.
pub fn field_label(field: CommitField) -> &'static str {
    match field {
        CommitField::Message => msg::FIELD_MESSAGE.get(),
        CommitField::AuthorName => msg::FIELD_AUTHOR_NAME.get(),
        CommitField::AuthorEmail => msg::FIELD_AUTHOR_EMAIL.get(),
        CommitField::Date => msg::FIELD_DATE.get(),
        CommitField::ShortSha => msg::FIELD_SHORT_SHA.get(),
        CommitField::LongSha => msg::FIELD_FULL_SHA.get(),
    }
}

/// Every field, in the Copy submenu's order.
pub const FIELDS: [CommitField; 6] = [
    CommitField::Message,
    CommitField::AuthorName,
    CommitField::AuthorEmail,
    CommitField::Date,
    CommitField::ShortSha,
    CommitField::LongSha,
];

/// What lands on the clipboard: the whole message (not the one-line summary), the date as the details
/// pane writes it.
pub fn field_value(
    field: CommitField,
    node: &CommitNode,
    dates: &gitpoulpi_app::dates::DateSettings,
) -> String {
    match field {
        CommitField::Message => node.message.clone(),
        CommitField::AuthorName => node.author_name.clone(),
        CommitField::AuthorEmail => node.author_email.clone(),
        CommitField::Date => dates.format_verbose(node.timestamp),
        CommitField::ShortSha => short(node.id),
        CommitField::LongSha => node.id.to_hex(),
    }
}

type Fut<T> =
    std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, String>> + Send + 'static>>;

impl Backend {
    /// Cherry-picks `commits` (oldest first) as one operation: one journal entry, so one Undo.
    pub fn cherry_pick_many(&self, commits: &[Oid]) -> Fut<CherryPickOutcome> {
        Box::pin(self.call(
            OperationKind::CherryPick,
            serde_json::json!({ "commits": commits, "opts": CherryPickOptions::default() }),
        ))
    }

    /// The files of a comparison (see `panels::log::compare`).
    pub fn target_diff(&self, target: DiffTarget) -> Fut<Vec<FileDiff>> {
        Box::pin(self.call(OperationKind::Diff, serde_json::json!({ "target": target })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_hard_reset_asks() {
        let t = Oid([1; 20]);
        assert!(reset_prompt(t, ResetMode::Soft).is_none());
        assert!(reset_prompt(t, ResetMode::Mixed).is_none());
        let hard = reset_prompt(t, ResetMode::Hard).unwrap();
        assert!(hard.contains("0101010"), "{hard}");
    }

    #[test]
    fn a_revert_names_the_mainline_of_a_merge() {
        let c = Oid([2; 20]);
        assert!(!revert_prompt(c, None).contains("mainline"));
        assert!(revert_prompt(c, Some(2)).contains("mainline"));
    }

    #[test]
    fn the_copied_values_are_the_whole_fields() {
        let node = CommitNode {
            id: Oid([0xab; 20]),
            parents: vec![],
            summary: "one".into(),
            message: "one\n\ntwo".into(),
            author_name: "Ada".into(),
            author_email: "ada@example.com".into(),
            timestamp: 0,
            refs: vec![],
        };
        assert_eq!(
            field_value(CommitField::Message, &node, &Default::default()),
            "one\n\ntwo"
        );
        assert_eq!(
            field_value(CommitField::ShortSha, &node, &Default::default()),
            "abababa"
        );
        assert_eq!(
            field_value(CommitField::LongSha, &node, &Default::default()).len(),
            40
        );
        assert_eq!(
            field_value(CommitField::AuthorEmail, &node, &Default::default()),
            "ada@example.com"
        );
        assert_eq!(FIELDS.len(), 6);
    }
}
