//! The composer's "Insert issue reference" picker (plan D8).
//!
//! The list is a combobox (`HostingUi` owns it as `PickerTarget::IssueRef`,
//! so it is modal, its rows lead the hit list and it has the keyboard like
//! every other dropdown). This module is the pure part: which rows it
//! offers, in what order, and what text lands in the commit message.
//!
//! - The reference the checked-out branch names (`refs::from_branch`:
//!   `ABC-7-login`, `42-fix`) comes first, marked "from branch".
//! - Then the linked ISSUES source's rows, as the provider spells them
//!   (`#12`, `ABC-123`, `AB#123`, a Trello short link); the combobox's
//!   search line filters them.
//! - Nothing is ever put in the message on its own (plan default for open
//!   question 4): the text is inserted only when a row is picked.

use crate::combo::ComboOption;
use gitpoulpi_hosting::refs::{self, RefContext};
use gitpoulpi_hosting::IssueSummary;
use gitpoulpi_i18n::msg;

/// What an issue row inserts: the provider's own spelling, else its label.
fn reference_of(issue: &IssueSummary) -> &str {
    if issue.reference.is_empty() {
        &issue.iid_label
    } else {
        &issue.reference
    }
}

/// `(reference, title)` of every issue, in the list's order: what the Flow
/// Start form's picker offers.
#[cfg_attr(not(feature = "hosting"), allow(dead_code))]
pub(crate) fn choices(issues: &[IssueSummary]) -> Vec<(String, String)> {
    issues
        .iter()
        .filter(|i| !reference_of(i).is_empty())
        .map(|i| (reference_of(i).to_string(), i.title.clone()))
        .collect()
}

/// The rows of the picker, branch-detected first. Each option's `value`
/// is the reference to insert; its label is what is listed and searched.
pub(crate) fn options(
    branch: Option<&str>,
    ctx: &RefContext,
    issues: &[IssueSummary],
) -> Vec<ComboOption> {
    let mut out = Vec::new();
    let from_branch = branch.and_then(|b| refs::from_branch(b, ctx));
    if let Some(found) = &from_branch {
        let title = issues
            .iter()
            .find(|i| reference_of(i) == found.reference)
            .map(|i| i.title.as_str())
            .unwrap_or("");
        let label = if title.is_empty() {
            format!(
                "{}  {}",
                found.reference,
                msg::composer::REF_FROM_BRANCH.get()
            )
        } else {
            format!(
                "{}  {}  {}",
                found.reference,
                title,
                msg::composer::REF_FROM_BRANCH.get()
            )
        };
        out.push(ComboOption::new(label, found.reference.clone()));
    }
    for issue in issues {
        let reference = reference_of(issue);
        if reference.is_empty()
            || from_branch
                .as_ref()
                .is_some_and(|f| f.reference == reference)
        {
            continue;
        }
        out.push(ComboOption::new(
            format!("{}  {}", short(issue, reference), issue.title),
            reference.to_string(),
        ));
    }
    out
}

/// The short label listed in front of the title: the reference unless it
/// is a long URL (a Trello short link), where the card number reads better.
fn short<'a>(issue: &'a IssueSummary, reference: &'a str) -> &'a str {
    if reference.contains("://") && !issue.iid_label.is_empty() {
        &issue.iid_label
    } else {
        reference
    }
}

/// What to type at the caret of `text` (byte offset `at`) to insert
/// `reference` as a word of its own: a space before it unless the text
/// starts there or already has whitespace just before, and one after it.
pub(crate) fn insertion(text: &str, at: usize, reference: &str) -> String {
    let before = text[..at.min(text.len())].chars().next_back();
    let lead = before.is_some_and(|c| !c.is_whitespace());
    format!("{}{reference} ", if lead { " " } else { "" })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_config::ApiFamily;
    use gitpoulpi_hosting::refs::IssueLinker;

    fn issue(reference: &str, iid: &str, title: &str) -> IssueSummary {
        IssueSummary {
            number: 1,
            reference: reference.into(),
            iid_label: iid.into(),
            title: title.into(),
            ..Default::default()
        }
    }

    fn jira() -> RefContext {
        RefContext {
            issues: Some(IssueLinker {
                family: ApiFamily::Jira,
                web_base: "https://j".into(),
                project: "ABC".into(),
                project_keys: Vec::new(),
                cards: Vec::new(),
            }),
            pulls: None,
        }
    }

    #[test]
    fn the_branch_issue_comes_first_and_is_not_repeated() {
        let issues = [
            issue("ABC-1", "ABC-1", "First"),
            issue("ABC-7", "ABC-7", "Login"),
        ];
        let o = options(Some("ABC-7-login"), &jira(), &issues);
        assert_eq!(o.len(), 2);
        assert_eq!(o[0].value, "ABC-7");
        assert!(o[0].label.starts_with("ABC-7  Login"));
        assert!(o[0].label.contains(msg::composer::REF_FROM_BRANCH.get()));
        assert_eq!(o[1].value, "ABC-1");
    }

    #[test]
    fn a_branch_issue_missing_from_the_list_is_still_offered() {
        let o = options(Some("feature/ABC-9"), &jira(), &[]);
        assert_eq!(o.len(), 1);
        assert_eq!(o[0].value, "ABC-9");
    }

    #[test]
    fn no_branch_reference_lists_the_rows_in_order() {
        let issues = [issue("#2", "#2", "B"), issue("#1", "#1", "A")];
        let o = options(Some("main"), &RefContext::default(), &issues);
        let values: Vec<&str> = o.iter().map(|o| o.value.as_str()).collect();
        assert_eq!(values, ["#2", "#1"]);
    }

    #[test]
    fn a_trello_short_link_is_inserted_whole_but_listed_by_number() {
        let card = issue("https://trello.com/c/AbC123", "#12", "Card");
        let o = options(None, &RefContext::default(), &[card]);
        assert_eq!(o[0].value, "https://trello.com/c/AbC123");
        assert!(o[0].label.starts_with("#12  Card"));
    }

    #[test]
    fn insertion_separates_the_word_from_its_neighbors() {
        assert_eq!(insertion("", 0, "ABC-1"), "ABC-1 ");
        assert_eq!(insertion("fix", 3, "ABC-1"), " ABC-1 ");
        assert_eq!(insertion("fix ", 4, "#2"), "#2 ");
    }
}
