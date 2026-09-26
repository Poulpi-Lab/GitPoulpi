//! The Kanban board's column logic (slice L2): which columns an issue source
//! has, which column each issue sits in, what a move changes, and the board
//! filters. Pure data in, pure data out: no IO and no UI, so the TUI, a CLI
//! and a provider added later (Linear, YouTrack) share it.
//!
//! A provider declares how its issues form columns with
//! [`HostingClient::board_source`](crate::HostingClient::board_source) and, for
//! the status-like sources, the ordered names with
//! [`HostingClient::issue_statuses`](crate::HostingClient::issue_statuses).
//! Moving a card is [`HostingClient::move_issue`](crate::HostingClient::move_issue),
//! whose default executes the [`MovePlan`] computed here.

use std::collections::HashSet;

use crate::model::{IssueState, IssueSummary};
use crate::{Error, Result};

/// How an issue source forms its columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BoardSource {
    /// Open / Closed only: a provider with no workflow, nothing configured.
    #[default]
    OpenClosed,
    /// A status the tool owns and can change (local Markdown notes).
    LocalStatus,
    /// A workflow status of the provider that cannot be changed from here
    /// (`IssueSummary::status_label`; Jira and Azure now use `LocalStatus`:
    /// their clients override `move_issue` with a transition / state patch;
    /// nothing uses this today). Read-only until the client overrides
    /// `move_issue`.
    ProviderStatus,
    /// A list of the provider's board (`IssueSummary::kind_label`: Trello).
    Lists,
    /// A configurable set of labels, one per column, plus Open and Closed
    /// (GitHub, GitLab, Gitea, Forgejo).
    Labels,
}

impl BoardSource {
    /// Whether a card can be moved at all with the default `move_issue`.
    pub fn movable(self) -> bool {
        !matches!(self, BoardSource::ProviderStatus)
    }
}

/// What a source needs to lay its board out.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BoardSpec {
    pub source: BoardSource,
    /// The ordered column names the provider declares
    /// (`HostingClient::issue_statuses`); the status-like sources only.
    pub declared: Vec<String>,
    /// The labels that form the columns of a [`BoardSource::Labels`] board,
    /// in board order (configured per repository).
    pub labels: Vec<String>,
}

/// What a column is, so the frontend can translate the two fixed ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnRole {
    /// A status, a list or a label: `name` is shown as it is.
    Named,
    /// The column of open issues with no status label.
    Open,
    /// The column of closed issues.
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardColumn {
    pub name: String,
    pub role: ColumnRole,
}

impl BoardColumn {
    fn named(name: &str) -> Self {
        Self {
            name: name.to_string(),
            role: ColumnRole::Named,
        }
    }

    fn open() -> Self {
        Self {
            name: "Open".into(), // i18n-english
            role: ColumnRole::Open,
        }
    }

    fn closed() -> Self {
        Self {
            name: "Closed".into(), // i18n-english
            role: ColumnRole::Closed,
        }
    }
}

/// The status-like value of an issue for `source` (`None` for the sources
/// that do not use one, or an issue with no value).
fn status_of(source: BoardSource, issue: &IssueSummary) -> Option<&str> {
    let value = match source {
        BoardSource::LocalStatus | BoardSource::ProviderStatus => issue.status_label.as_deref(),
        BoardSource::Lists => issue.kind_label.as_deref(),
        BoardSource::OpenClosed | BoardSource::Labels => None,
    };
    value.map(str::trim).filter(|v| !v.is_empty())
}

fn same(a: &str, b: &str) -> bool {
    a.trim().eq_ignore_ascii_case(b.trim())
}

/// The columns of the board, left to right: the declared names first, then
/// any other value the issues carry (in order of first appearance).
pub fn columns(spec: &BoardSpec, issues: &[IssueSummary]) -> Vec<BoardColumn> {
    match spec.source {
        BoardSource::OpenClosed => vec![BoardColumn::open(), BoardColumn::closed()],
        BoardSource::Labels => {
            let mut out = vec![BoardColumn::open()];
            let mut seen: Vec<&str> = Vec::new();
            for label in &spec.labels {
                if !seen.iter().any(|s| same(s, label)) && !label.trim().is_empty() {
                    seen.push(label);
                    out.push(BoardColumn::named(label.trim()));
                }
            }
            out.push(BoardColumn::closed());
            out
        }
        BoardSource::LocalStatus | BoardSource::ProviderStatus | BoardSource::Lists => {
            let mut names: Vec<String> = Vec::new();
            let declared = spec.declared.iter().map(String::as_str);
            let seen = issues.iter().filter_map(|i| status_of(spec.source, i));
            for name in declared.chain(seen) {
                let name = name.trim();
                if !name.is_empty() && !names.iter().any(|n| same(n, name)) {
                    names.push(name.to_string());
                }
            }
            if names.is_empty() {
                // Nothing declared and nothing seen: a board still has columns.
                return vec![BoardColumn::open(), BoardColumn::closed()];
            }
            names.iter().map(|n| BoardColumn::named(n)).collect()
        }
    }
}

/// The index in `cols` of the column `issue` sits in. An issue with no
/// status goes to the first column (open) or the last (closed).
pub fn column_index(spec: &BoardSpec, cols: &[BoardColumn], issue: &IssueSummary) -> usize {
    let last = cols.len().saturating_sub(1);
    let closed_slot = || {
        cols.iter()
            .position(|c| c.role == ColumnRole::Closed)
            .unwrap_or(last)
    };
    match spec.source {
        BoardSource::OpenClosed => usize::from(issue.state == IssueState::Closed),
        BoardSource::Labels => {
            if issue.state == IssueState::Closed {
                return closed_slot();
            }
            spec.labels
                .iter()
                .find(|l| issue.labels.iter().any(|have| same(have, l)))
                .and_then(|l| cols.iter().position(|c| same(&c.name, l)))
                .unwrap_or(0)
        }
        _ => match status_of(spec.source, issue) {
            Some(value) => cols.iter().position(|c| same(&c.name, value)).unwrap_or(
                if issue.state == IssueState::Closed {
                    last
                } else {
                    0
                },
            ),
            None if issue.state == IssueState::Closed => closed_slot(),
            None => 0,
        },
    }
}

/// What moving an issue to a column changes. Only the parts that differ are
/// set.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MovePlan {
    /// The new status (local Markdown).
    pub status: Option<String>,
    /// The new list (Trello).
    pub list: Option<String>,
    /// The complete new label set.
    pub labels: Option<Vec<String>>,
    /// Close (`true`) or reopen (`false`).
    pub closed: Option<bool>,
}

impl MovePlan {
    pub fn is_empty(&self) -> bool {
        self.status.is_none()
            && self.list.is_none()
            && self.labels.is_none()
            && self.closed.is_none()
    }
}

/// What it takes to move `issue` to `to`. `Err(Unsupported)` for a read-only
/// source.
pub fn plan_move(spec: &BoardSpec, issue: &IssueSummary, to: &BoardColumn) -> Result<MovePlan> {
    let mut plan = MovePlan::default();
    match spec.source {
        BoardSource::ProviderStatus => {
            return Err(Error::Unsupported("moving an issue to a column".into()));
        }
        BoardSource::LocalStatus => {
            if !status_of(spec.source, issue).is_some_and(|s| same(s, &to.name)) {
                plan.status = Some(to.name.clone());
            }
        }
        BoardSource::Lists => {
            if !status_of(spec.source, issue).is_some_and(|s| same(s, &to.name)) {
                plan.list = Some(to.name.clone());
            }
        }
        BoardSource::OpenClosed | BoardSource::Labels => {
            let is_closed = issue.state == IssueState::Closed;
            let want_closed = to.role == ColumnRole::Closed;
            if is_closed != want_closed {
                plan.closed = Some(want_closed);
            }
            if spec.source == BoardSource::Labels && !want_closed {
                let mut labels: Vec<String> = issue
                    .labels
                    .iter()
                    .filter(|have| !spec.labels.iter().any(|l| same(have, l)))
                    .cloned()
                    .collect();
                if to.role == ColumnRole::Named {
                    labels.push(to.name.clone());
                }
                if labels != issue.labels {
                    plan.labels = Some(labels);
                }
            }
        }
    }
    Ok(plan)
}

/// Applies `plan` to the cached copy of the issue, so the board shows the
/// result without waiting for the next listing.
pub fn apply_plan(issue: &mut IssueSummary, plan: &MovePlan) {
    if let Some(status) = &plan.status {
        issue.status_label = Some(status.clone());
    }
    if let Some(list) = &plan.list {
        issue.kind_label = Some(list.clone());
    }
    if let Some(labels) = &plan.labels {
        issue.labels = labels.clone();
    }
    if let Some(closed) = plan.closed {
        issue.state = if closed {
            IssueState::Closed
        } else {
            IssueState::Open
        };
    }
}

/// The board filters: an empty field matches everything.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BoardFilter {
    pub assignee: Option<String>,
    pub label: Option<String>,
    pub milestone: Option<String>,
    pub text: String,
}

impl BoardFilter {
    pub fn is_empty(&self) -> bool {
        self.assignee.is_none()
            && self.label.is_none()
            && self.milestone.is_none()
            && self.text.trim().is_empty()
    }

    pub fn matches(&self, issue: &IssueSummary) -> bool {
        if let Some(a) = &self.assignee {
            if !issue.assignees.iter().any(|have| same(have, a)) {
                return false;
            }
        }
        if let Some(l) = &self.label {
            if !issue.labels.iter().any(|have| same(have, l)) {
                return false;
            }
        }
        if let Some(m) = &self.milestone {
            if !issue.milestone.as_deref().is_some_and(|have| same(have, m)) {
                return false;
            }
        }
        let q = self.text.trim().to_lowercase();
        q.is_empty()
            || [&issue.title, &issue.iid_label, &issue.reference]
                .iter()
                .any(|f| f.to_lowercase().contains(&q))
    }
}

/// The distinct values the filter comboboxes offer, sorted.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FilterChoices {
    pub assignees: Vec<String>,
    pub labels: Vec<String>,
    pub milestones: Vec<String>,
}

pub fn filter_choices(issues: &[IssueSummary]) -> FilterChoices {
    fn distinct<'a>(values: impl Iterator<Item = &'a str>) -> Vec<String> {
        let mut seen: HashSet<String> = HashSet::new();
        let mut out: Vec<String> = values
            .filter(|v| !v.trim().is_empty() && seen.insert(v.to_lowercase()))
            .map(str::to_string)
            .collect();
        out.sort_by_key(|v| v.to_lowercase());
        out
    }
    FilterChoices {
        assignees: distinct(
            issues
                .iter()
                .flat_map(|i| i.assignees.iter().map(String::as_str)),
        ),
        labels: distinct(
            issues
                .iter()
                .flat_map(|i| i.labels.iter().map(String::as_str)),
        ),
        milestones: distinct(issues.iter().filter_map(|i| i.milestone.as_deref())),
    }
}

/// The labels that look like statuses (`status::todo`, `status:doing`,
/// `status/done`, `Status: Review`), for the "Set status labels" prompt to
/// propose. Order is the order given.
pub fn status_like_labels(all: &[String]) -> Vec<String> {
    all.iter()
        .filter(|l| {
            let lower = l.trim().to_lowercase();
            ["status::", "status:", "status/", "status-"]
                .iter()
                .any(|p| lower.starts_with(p))
        })
        .cloned()
        .collect()
}

/// Splits a typed `a, b, c` into the label list.
pub fn parse_label_list(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for part in text.split(',') {
        let part = part.trim();
        if !part.is_empty() && !out.iter().any(|o| same(o, part)) {
            out.push(part.to_string());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn issue(n: u64, status: Option<&str>, closed: bool, labels: &[&str]) -> IssueSummary {
        IssueSummary {
            number: n,
            id: n.to_string(),
            iid_label: format!("#{n}"),
            title: format!("Issue {n}"),
            status_label: status.map(str::to_string),
            labels: labels.iter().map(|s| s.to_string()).collect(),
            state: if closed {
                IssueState::Closed
            } else {
                IssueState::Open
            },
            ..IssueSummary::default()
        }
    }

    fn names(cols: &[BoardColumn]) -> Vec<&str> {
        cols.iter().map(|c| c.name.as_str()).collect()
    }

    fn local(declared: &[&str]) -> BoardSpec {
        BoardSpec {
            source: BoardSource::LocalStatus,
            declared: declared.iter().map(|s| s.to_string()).collect(),
            labels: Vec::new(),
        }
    }

    fn labels_spec(labels: &[&str]) -> BoardSpec {
        BoardSpec {
            source: BoardSource::Labels,
            declared: Vec::new(),
            labels: labels.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn status_columns_are_the_declared_ones_then_the_values_seen() {
        let spec = local(&["todo", "doing", "done"]);
        let issues = [
            issue(1, Some("review"), false, &[]),
            issue(2, Some("DOING"), false, &[]),
            issue(3, Some("blocked"), false, &[]),
            issue(4, Some("review"), false, &[]),
        ];
        let cols = columns(&spec, &issues);
        assert_eq!(
            names(&cols),
            ["todo", "doing", "done", "review", "blocked"],
            "declared order first, the rest as first seen, no case duplicates"
        );
        assert_eq!(
            column_index(&spec, &cols, &issues[1]),
            1,
            "case-insensitive"
        );
        assert_eq!(column_index(&spec, &cols, &issues[2]), 4);
    }

    #[test]
    fn a_status_board_with_nothing_still_has_open_and_closed() {
        let cols = columns(&local(&[]), &[]);
        assert_eq!(names(&cols), ["Open", "Closed"]);
        assert_eq!(cols[1].role, ColumnRole::Closed);
    }

    #[test]
    fn an_issue_without_a_status_goes_first_or_last_by_state() {
        let spec = BoardSpec {
            source: BoardSource::ProviderStatus,
            declared: vec!["To Do".into(), "In Progress".into(), "Done".into()],
            labels: Vec::new(),
        };
        let cols = columns(&spec, &[]);
        assert_eq!(column_index(&spec, &cols, &issue(1, None, false, &[])), 0);
        assert_eq!(column_index(&spec, &cols, &issue(2, None, true, &[])), 2);
        // A closed issue whose status is not a column (renamed workflow).
        assert_eq!(
            column_index(&spec, &cols, &issue(3, Some("Gone"), true, &[])),
            2
        );
    }

    #[test]
    fn trello_lists_come_from_kind_label() {
        let spec = BoardSpec {
            source: BoardSource::Lists,
            declared: vec!["Backlog".into(), "Doing".into()],
            labels: Vec::new(),
        };
        let mut card = issue(1, None, false, &[]);
        card.kind_label = Some("Doing".into());
        let cols = columns(&spec, std::slice::from_ref(&card));
        assert_eq!(names(&cols), ["Backlog", "Doing"]);
        assert_eq!(column_index(&spec, &cols, &card), 1);
        let plan = plan_move(&spec, &card, &cols[0]).unwrap();
        assert_eq!(plan.list.as_deref(), Some("Backlog"));
        assert!(plan_move(&spec, &card, &cols[1]).unwrap().is_empty());
    }

    #[test]
    fn a_label_board_is_open_the_labels_then_closed() {
        let spec = labels_spec(&["status::todo", "status::doing"]);
        let issues = [
            issue(1, None, false, &["bug"]),
            issue(2, None, false, &["bug", "status::doing"]),
            issue(3, None, true, &["status::todo"]),
        ];
        let cols = columns(&spec, &issues);
        assert_eq!(
            names(&cols),
            ["Open", "status::todo", "status::doing", "Closed"]
        );
        assert_eq!(column_index(&spec, &cols, &issues[0]), 0);
        assert_eq!(column_index(&spec, &cols, &issues[1]), 2);
        assert_eq!(
            column_index(&spec, &cols, &issues[2]),
            3,
            "closed wins over a status label"
        );
    }

    #[test]
    fn without_a_label_set_a_host_board_is_open_and_closed() {
        let spec = labels_spec(&[]);
        let cols = columns(&spec, &[]);
        assert_eq!(names(&cols), ["Open", "Closed"]);
        assert_eq!(
            column_index(&spec, &cols, &issue(1, None, true, &[])),
            1,
            "closed"
        );
    }

    #[test]
    fn moving_on_a_label_board_swaps_the_status_label_and_keeps_the_others() {
        let spec = labels_spec(&["status::todo", "status::doing"]);
        let cols = columns(&spec, &[]);
        let card = issue(1, None, false, &["bug", "status::todo"]);
        let plan = plan_move(&spec, &card, &cols[2]).unwrap();
        assert_eq!(
            plan.labels,
            Some(vec!["bug".to_string(), "status::doing".to_string()])
        );
        assert_eq!(plan.closed, None);
        // Back to Open drops the status label.
        let plan = plan_move(&spec, &card, &cols[0]).unwrap();
        assert_eq!(plan.labels, Some(vec!["bug".to_string()]));
    }

    #[test]
    fn closing_and_reopening_on_a_label_board() {
        let spec = labels_spec(&["status::todo"]);
        let cols = columns(&spec, &[]);
        let open = issue(1, None, false, &["status::todo"]);
        let plan = plan_move(&spec, &open, &cols[2]).unwrap();
        assert_eq!(plan.closed, Some(true));
        assert_eq!(plan.labels, None, "closing leaves the labels alone");
        let closed = issue(2, None, true, &[]);
        let plan = plan_move(&spec, &closed, &cols[1]).unwrap();
        assert_eq!(plan.closed, Some(false));
        assert_eq!(plan.labels, Some(vec!["status::todo".to_string()]));
    }

    #[test]
    fn moving_on_an_open_closed_board_only_changes_the_state() {
        let spec = BoardSpec::default();
        let cols = columns(&spec, &[]);
        let card = issue(1, None, false, &["bug"]);
        let plan = plan_move(&spec, &card, &cols[1]).unwrap();
        assert_eq!(
            plan,
            MovePlan {
                closed: Some(true),
                ..MovePlan::default()
            }
        );
    }

    #[test]
    fn local_status_move_sets_the_status_and_a_no_op_is_empty() {
        let spec = local(&["todo", "doing", "done"]);
        let cols = columns(&spec, &[]);
        let card = issue(1, Some("todo"), false, &[]);
        assert_eq!(
            plan_move(&spec, &card, &cols[1]).unwrap().status.as_deref(),
            Some("doing")
        );
        assert!(plan_move(&spec, &card, &cols[0]).unwrap().is_empty());
    }

    #[test]
    fn a_provider_status_board_is_read_only() {
        let spec = BoardSpec {
            source: BoardSource::ProviderStatus,
            declared: vec!["To Do".into(), "Done".into()],
            labels: Vec::new(),
        };
        assert!(!spec.source.movable());
        let cols = columns(&spec, &[]);
        let err = plan_move(&spec, &issue(1, Some("To Do"), false, &[]), &cols[1]);
        assert!(matches!(err, Err(Error::Unsupported(_))));
    }

    #[test]
    fn apply_plan_updates_the_cached_issue() {
        let mut card = issue(1, Some("todo"), false, &["bug"]);
        apply_plan(
            &mut card,
            &MovePlan {
                status: Some("done".into()),
                labels: Some(vec![]),
                closed: Some(true),
                ..MovePlan::default()
            },
        );
        assert_eq!(card.status_label.as_deref(), Some("done"));
        assert!(card.labels.is_empty());
        assert_eq!(card.state, IssueState::Closed);
    }

    #[test]
    fn filters_combine_and_an_empty_filter_matches_everything() {
        let mut a = issue(1, Some("todo"), false, &["bug"]);
        a.assignees = vec!["Kai".into()];
        a.milestone = Some("v0.1".into());
        let b = issue(2, Some("todo"), false, &["ui"]);
        assert!(BoardFilter::default().matches(&a));
        assert!(BoardFilter::default().is_empty());
        let f = BoardFilter {
            assignee: Some("kai".into()),
            label: Some("BUG".into()),
            milestone: Some("v0.1".into()),
            text: "issue 1".into(),
        };
        assert!(f.matches(&a));
        assert!(!f.matches(&b));
        let only_text = BoardFilter {
            text: "#2".into(),
            ..BoardFilter::default()
        };
        assert!(!only_text.matches(&a));
        assert!(only_text.matches(&b));
    }

    #[test]
    fn filter_choices_are_distinct_and_sorted() {
        let mut a = issue(1, None, false, &["ui", "bug"]);
        a.assignees = vec!["kai".into(), "Anna".into()];
        a.milestone = Some("v2".into());
        let mut b = issue(2, None, false, &["BUG"]);
        b.assignees = vec!["anna".into()];
        b.milestone = Some("v1".into());
        let c = filter_choices(&[a, b]);
        assert_eq!(c.assignees, ["Anna", "kai"]);
        assert_eq!(c.labels, ["bug", "ui"]);
        assert_eq!(c.milestones, ["v1", "v2"]);
    }

    #[test]
    fn label_lists_are_parsed_and_proposed() {
        assert_eq!(
            parse_label_list(" status::todo , status::doing,,STATUS::todo"),
            ["status::todo", "status::doing"]
        );
        let all: Vec<String> = ["bug", "status::todo", "Status: Review", "status/done"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(
            status_like_labels(&all),
            ["status::todo", "Status: Review", "status/done"]
        );
    }
}
