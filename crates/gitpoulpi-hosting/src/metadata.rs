//! Assignees, reviewers, labels and milestones: the host-agnostic shapes
//! the pickers (TUI), `pr edit` / `issue edit` (CLI/MCP) and the create
//! forms speak. GitHub and GitLab fill them (`github_meta`, `gitlab_meta`);
//! every other provider keeps the `Unsupported` defaults of the trait and
//! nothing about it shows in a UI.

use serde::{Deserialize, Serialize};

/// A label of a project.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedLabel {
    pub name: String,
    /// Hex colour without the `#` (`d73a4a`); empty when the host sent none.
    #[serde(default)]
    pub color: String,
    #[serde(default)]
    pub description: String,
}

/// A milestone of a project (open ones are offered).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedMilestone {
    /// What an edit sends: the milestone *number* on GitHub, GitLab's
    /// project-wide milestone `id`.
    pub id: u64,
    pub title: String,
    /// The due date as the host sent it (`2026-11-01`), when it has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due_on: Option<String>,
    /// `open` / `closed` / `active`, as the host spells it.
    #[serde(default)]
    pub state: String,
}

/// Whether a metadata call is about an issue or a pull / merge request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetaKind {
    Issue,
    PullRequest,
}

/// What an issue or request carries right now. Logins for people, names
/// for labels. `reviewers` stays empty for an issue.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemMeta {
    #[serde(default)]
    pub assignees: Vec<String>,
    #[serde(default)]
    pub reviewers: Vec<String>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub milestone: Option<HostedMilestone>,
}

/// Replacements for an issue's metadata. `None` leaves a field alone;
/// `Some(vec![])` clears it. `milestone: Some(None)` clears the milestone.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IssueEdit {
    pub assignees: Option<Vec<String>>,
    pub labels: Option<Vec<String>>,
    pub milestone: Option<Option<u64>>,
}

impl IssueEdit {
    pub fn is_empty(&self) -> bool {
        self.assignees.is_none() && self.labels.is_none() && self.milestone.is_none()
    }
}

/// Replacements for a pull / merge request's metadata (see [`IssueEdit`]).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrEdit {
    pub assignees: Option<Vec<String>>,
    pub reviewers: Option<Vec<String>>,
    pub labels: Option<Vec<String>>,
    pub milestone: Option<Option<u64>>,
}

impl PrEdit {
    pub fn is_empty(&self) -> bool {
        self.assignees.is_none()
            && self.reviewers.is_none()
            && self.labels.is_none()
            && self.milestone.is_none()
    }
}

/// What to add and what to remove to turn the list `have` into `wanted`
/// (logins compare case-insensitively).
pub fn list_delta(have: &[String], wanted: &[String]) -> (Vec<String>, Vec<String>) {
    let has = |list: &[String], x: &str| list.iter().any(|y| y.eq_ignore_ascii_case(x));
    let add = wanted
        .iter()
        .filter(|w| !has(have, w))
        .cloned()
        .collect::<Vec<_>>();
    let remove = have
        .iter()
        .filter(|h| !has(wanted, h))
        .cloned()
        .collect::<Vec<_>>();
    (add, remove)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_delta_of_two_lists_is_what_to_add_and_remove() {
        let have = vec!["a".to_string(), "B".to_string()];
        let wanted = vec!["b".to_string(), "c".to_string()];
        let (add, remove) = list_delta(&have, &wanted);
        assert_eq!(add, vec!["c"]);
        assert_eq!(remove, vec!["a"]);
    }

    #[test]
    fn an_empty_edit_says_so() {
        assert!(IssueEdit::default().is_empty());
        assert!(PrEdit::default().is_empty());
        assert!(!PrEdit {
            milestone: Some(None),
            ..PrEdit::default()
        }
        .is_empty());
    }
}
