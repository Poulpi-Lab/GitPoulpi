//! The rows of the Refs pane's PULL REQUESTS and ISSUES sections, as plain
//! data: which rows exist, in which order, with which counts and titles. A
//! frontend lists exactly these rows; glyphs, colours, widths and hit
//! rectangles stay with the frontend.
//!
//! How a frontend uses it: keep the user's choices (folded groups, search
//! texts) next to the latest [`HostingSnapshot`], build a [`RowsInput`] from
//! them each frame, then call [`RowsInput::rows`] per [`HostingList`]. A row
//! that names an item carries an index into the snapshot's `prs` / `issues`
//! (see [`HostingRow::Pull`] and [`HostingRow::Issue`]); the state of such an
//! item is its own `PullRequestState` / `IssueState` (open, draft, merged,
//! closed), which the frontend maps to a glyph or colour, with
//! [`state_word`] for the words.

use std::collections::HashSet;

use gitpoulpi_config::{ApiFamily, IntegrationKind};
use gitpoulpi_hosting::{IssueSummary, PullRequestState, PullRequestSummary};
use gitpoulpi_i18n::msg;

use crate::hosting_task::{age_label, HostingSnapshot, Problem};

/// Which of the two lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostingList {
    Pulls,
    Issues,
}

/// The four collapsible groups of the PULL REQUESTS section.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrGroup {
    Mine,
    Assigned,
    Awaiting,
    All,
}

impl PrGroup {
    /// The key a folded group is remembered under (`fold_state`).
    pub fn key(self) -> &'static str {
        match self {
            PrGroup::Mine => "mine",
            PrGroup::Assigned => "assigned",
            PrGroup::Awaiting => "awaiting",
            PrGroup::All => "all",
        }
    }

    /// The groups in the order the section lists them.
    pub const ALL: [PrGroup; 4] = [
        PrGroup::Mine,
        PrGroup::Assigned,
        PrGroup::Awaiting,
        PrGroup::All,
    ];

    /// The group's header text; the pull / merge request word follows the host.
    pub fn label(self) -> &'static str {
        match self {
            PrGroup::Mine => msg::hosting_ui::GROUP_MINE.get(),
            PrGroup::Assigned => msg::hosting_ui::GROUP_ASSIGNED.get(),
            PrGroup::Awaiting => msg::hosting_ui::GROUP_AWAITING.get(),
            PrGroup::All => msg::hosting_ui::GROUP_ALL.get(),
        }
    }

    /// Whether `pr` belongs to the group, for the signed-in user `me`.
    pub fn holds(self, pr: &PullRequestSummary, me: &str) -> bool {
        let is_me = |login: &String| !me.is_empty() && login.eq_ignore_ascii_case(me);
        match self {
            PrGroup::All => true,
            PrGroup::Mine => !me.is_empty() && pr.author.eq_ignore_ascii_case(me),
            PrGroup::Assigned => pr.assignees.iter().any(is_me),
            PrGroup::Awaiting => pr.review_requested || pr.reviewers.iter().any(is_me),
        }
    }
}

/// One row of a sidebar list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostingRow {
    /// "Reconnect" - the token is missing, rejected or short of scope.
    Reconnect,
    /// An inert line (an error, or "offline").
    Notice(String),
    /// The search box.
    Search,
    /// The state-filter combobox.
    Filter,
    /// A group's header (chevron, name, count).
    Group(PrGroup),
    /// A pull request under a group: `(group, index into the snapshot)`.
    Pull(PrGroup, usize),
    /// The issue provider combobox (GitHub / GitLab profiles).
    IssueProvider,
    /// The provider's repository combobox.
    IssueRepo,
    Issue(usize),
    /// The pull-request provider combobox (GitHub / GitLab profiles).
    PullProvider,
    /// The pull-request provider's repository combobox.
    PullRepo,
    /// The optional extra query of the issue source: a JQL / WIQL text field
    /// (Jira, Azure) or the Trello lists dropdown.
    IssueQuery,
}

impl HostingRow {
    /// A row that is a control, not an item: one click uses it.
    pub fn is_control(&self) -> bool {
        !matches!(
            self,
            HostingRow::Notice(_) | HostingRow::Pull(..) | HostingRow::Issue(_)
        )
    }
}

/// The state the rows are computed from. The frontend owns it (folds, search
/// texts, the clock) and rebuilds the value cheaply each frame.
#[derive(Debug, Clone, Copy)]
pub struct RowsInput<'a> {
    /// The latest snapshot, `None` before the first answer.
    pub snapshot: Option<&'a HostingSnapshot>,
    /// The groups the user folded.
    pub collapsed: &'a HashSet<PrGroup>,
    /// What was typed in the pull request search box.
    pub pr_query: &'a str,
    /// What was typed in the issue search box.
    pub issue_query: &'a str,
    /// The spinner frame while the pull requests are loading for a while, else `None`.
    pub loading_pulls: Option<char>,
    /// The spinner frame while the issues are loading for a while, else `None`.
    pub loading_issues: Option<char>,
    /// Whether the snapshot is up to date (not stale from the cache).
    pub fresh: bool,
    /// The current time, in unix seconds.
    pub now: i64,
}

impl<'a> RowsInput<'a> {
    /// The spinner frame of one list, when it is loading.
    pub fn loading(&self, list: HostingList) -> Option<char> {
        match list {
            HostingList::Pulls => self.loading_pulls,
            HostingList::Issues => self.loading_issues,
        }
    }

    /// The loading line of a list, when it is loading.
    pub fn loading_row(&self, list: HostingList) -> Option<HostingRow> {
        let frame = self.loading(list)?;
        Some(HostingRow::Notice(match list {
            HostingList::Pulls => msg::hosting_ui::loading_prs(frame),
            HostingList::Issues => msg::hosting_ui::loading_issues(frame),
        }))
    }

    /// The snapshot, when this repository is linked to a provider.
    pub fn linked_snapshot(&self) -> Option<&'a HostingSnapshot> {
        self.snapshot.filter(|s| s.linked())
    }

    /// The search text of a list.
    pub fn query(&self, list: HostingList) -> &'a str {
        match list {
            HostingList::Pulls => self.pr_query,
            HostingList::Issues => self.issue_query,
        }
    }

    /// The API family of the chosen issue source.
    pub fn issues_family(&self) -> Option<ApiFamily> {
        self.snapshot
            .and_then(|s| s.issues_target.as_ref())
            .map(|t| t.profile.kind.family())
    }

    /// `(pull requests, issues)` sections to show. The pull requests need
    /// a profile that answers for the remote; the issues only need a
    /// GitHub / GitLab profile to pick a tracker from.
    pub fn sections_visible(&self) -> (bool, bool) {
        (
            self.snapshot
                .is_some_and(|s| !s.pull_profiles.is_empty() || s.linked())
                || (self.snapshot.is_none() && self.loading(HostingList::Pulls).is_some()),
            self.snapshot.is_some_and(|s| !s.issue_profiles.is_empty())
                || (self.snapshot.is_none() && self.loading(HostingList::Issues).is_some()),
        )
    }

    /// Indexes into the snapshot's pull requests that `group` shows,
    /// after the search box.
    pub fn group_members(&self, group: PrGroup) -> Vec<usize> {
        let Some(snap) = self.linked_snapshot() else {
            return Vec::new();
        };
        snap.prs
            .iter()
            .enumerate()
            .filter(|(_, pr)| group.holds(pr, &snap.me) && pr_matches(pr, self.pr_query))
            .map(|(i, _)| i)
            .collect()
    }

    /// The rows of one list, top to bottom.
    pub fn rows(&self, list: HostingList) -> Vec<HostingRow> {
        match list {
            HostingList::Pulls => self.pull_rows(),
            HostingList::Issues => self.issue_rows(),
        }
    }

    fn problem_rows(problem: Option<&Problem>, has_data: bool) -> Option<HostingRow> {
        let problem = problem?;
        if problem.wants_reconnect() {
            Some(HostingRow::Reconnect)
        } else if !has_data {
            Some(HostingRow::Notice(match problem {
                Problem::Offline(_) => msg::hosting_ui::OFFLINE.get().to_string(),
                Problem::Other(why) | Problem::RateLimited(why) => why.clone(),
                _ => String::new(),
            }))
        } else {
            None
        }
    }

    fn pull_rows(&self) -> Vec<HostingRow> {
        let Some(shown) = self
            .snapshot
            .filter(|s| !s.pull_profiles.is_empty() || s.linked())
        else {
            return self
                .loading_row(HostingList::Pulls)
                .into_iter()
                .collect::<Vec<_>>();
        };
        let mut rows = vec![HostingRow::PullProvider, HostingRow::PullRepo];
        let Some(snap) = Some(shown).filter(|s| s.linked() && s.can_pulls) else {
            return rows;
        };
        rows.extend(Self::problem_rows(
            snap.problem.as_ref(),
            !snap.prs.is_empty(),
        ));
        rows.push(HostingRow::Search);
        rows.push(HostingRow::Filter);
        for group in PrGroup::ALL {
            rows.push(HostingRow::Group(group));
            if !self.collapsed.contains(&group) {
                rows.extend(
                    self.group_members(group)
                        .into_iter()
                        .map(|i| HostingRow::Pull(group, i)),
                );
            }
        }
        rows
    }

    fn issue_rows(&self) -> Vec<HostingRow> {
        let Some(snap) = self.snapshot.filter(|s| !s.issue_profiles.is_empty()) else {
            return self
                .loading_row(HostingList::Issues)
                .into_iter()
                .collect::<Vec<_>>();
        };
        let mut rows = vec![HostingRow::IssueProvider, HostingRow::IssueRepo];
        if snap
            .issues_target
            .as_ref()
            .is_some_and(|t| !t.project.is_empty())
        {
            // The provider-native extra query (Jira JQL, Azure WIQL, Trello
            // lists): one optional step after the project.
            if self.issues_family().is_some_and(takes_issue_query) {
                rows.push(HostingRow::IssueQuery);
            }
            rows.extend(Self::problem_rows(
                snap.issues_problem.as_ref(),
                !snap.issues.is_empty(),
            ));
            rows.push(HostingRow::Search);
            rows.push(HostingRow::Filter);
            rows.extend(
                snap.issues
                    .iter()
                    .enumerate()
                    .filter(|(_, i)| issue_matches(i, self.issue_query))
                    .map(|(i, _)| HostingRow::Issue(i)),
            );
        }
        rows
    }

    /// How many items (not rows) the list's snapshot holds.
    pub fn data_len(&self, list: HostingList) -> usize {
        match list {
            HostingList::Pulls => self.linked_snapshot().map_or(0, |s| s.prs.len()),
            HostingList::Issues => self.snapshot.map_or(0, |s| s.issues.len()),
        }
    }

    /// `(pull request rows, issue rows)`.
    pub fn row_counts(&self) -> (usize, usize) {
        (
            self.rows(HostingList::Pulls).len(),
            self.rows(HostingList::Issues).len(),
        )
    }

    /// The count in a section's title: real items, not status rows.
    pub fn item_count(&self, list: HostingList) -> usize {
        self.data_len(list)
    }

    /// The section's title: `PULL REQUESTS`, and `GITHUB ISSUES` /
    /// `GITLAB ISSUES` once a provider is chosen.
    pub fn section_title(&self, list: HostingList) -> String {
        if list == HostingList::Issues {
            if let Some(target) = self.snapshot.and_then(|s| s.issues_target.as_ref()) {
                let name = provider_name(target.profile.kind).to_uppercase();
                // Azure calls them work items, Trello cards; the rest issues.
                return match target.profile.kind.family() {
                    ApiFamily::AzureDevops => msg::hosting_ui::issues_title_work_items(name),
                    ApiFamily::Trello => msg::hosting_ui::issues_title_cards(name),
                    _ => msg::hosting_ui::issues_title_of(name),
                };
            }
        }
        match list {
            HostingList::Pulls => msg::hosting_ui::PULLS_TITLE.get().to_string(),
            HostingList::Issues => msg::hosting_ui::ISSUES_TITLE.get().to_string(),
        }
    }

    /// Text appended to a section's title: how old the data is, when it
    /// is not fresh.
    pub fn title_suffix(&self, list: HostingList) -> String {
        // In flight for a while: a spinner after the (stale) data's age.
        let spinner = self
            .loading(list)
            .map(|f| format!(" {f}"))
            .unwrap_or_default();
        let Some(snap) = self.snapshot else {
            return spinner;
        };
        let stale = !self.fresh || matches!(snap.problem, Some(Problem::Offline(_)));
        if stale && snap.fetched_at > 0 {
            let age = if self.now - snap.fetched_at < 60 {
                msg::reltime::JUST_NOW.get().to_string()
            } else {
                msg::hosting_ui::age_suffix(age_label(self.now, snap.fetched_at))
            };
            format!(" \u{00B7} {age}{spinner}")
        } else {
            spinner
        }
    }
}

/// How a provider is named in the UI ("GitHub", "Azure DevOps"...).
pub fn provider_name(kind: IntegrationKind) -> &'static str {
    match kind.family() {
        ApiFamily::Github => "GitHub",
        ApiFamily::Gitlab => "GitLab",
        ApiFamily::AzureDevops => "Azure DevOps",
        ApiFamily::Bitbucket => "Bitbucket",
        ApiFamily::Jira => "Jira",
        ApiFamily::Trello => "Trello",
        ApiFamily::Gitea => kind.label(),
        ApiFamily::Linear => "Linear",
        ApiFamily::YouTrack => "YouTrack",
        ApiFamily::Redmine => "Redmine",
        ApiFamily::Gerrit => "Gerrit",
        ApiFamily::LocalMarkdown => "Local Markdown", // i18n-english
    }
}

/// `GH` / `GL` / `AZ` / `BB` for the profile's provider family (the chip's
/// tag; the same letters `gitpoulpi_hosting::FamilySpec::tag` carries).
pub fn provider_tag(kind: Option<IntegrationKind>) -> &'static str {
    match kind.map(|k| k.family()) {
        Some(ApiFamily::Github) => "GH",
        Some(ApiFamily::Gitlab) => "GL",
        Some(ApiFamily::AzureDevops) => "AZ",
        Some(ApiFamily::Bitbucket) => "BB",
        Some(ApiFamily::Jira) => "JI",
        Some(ApiFamily::Trello) => "TR",
        Some(ApiFamily::Gitea) => "GT",
        Some(ApiFamily::Linear) => "LN",
        Some(ApiFamily::YouTrack) => "YT",
        Some(ApiFamily::Redmine) => "RM",
        Some(ApiFamily::Gerrit) => "GR",
        Some(ApiFamily::LocalMarkdown) => "MD",
        None => msg::hosting_ui::PR_ABBR.get(),
    }
}

/// The word for a request's state, in the display language.
pub fn state_word(state: PullRequestState) -> &'static str {
    match state {
        PullRequestState::Open => msg::hosting_ui::STATE_OPEN.get(),
        PullRequestState::Draft => msg::hosting_ui::STATE_DRAFT.get(),
        PullRequestState::Merged => msg::hosting_ui::STATE_MERGED.get(),
        PullRequestState::Closed => msg::hosting_ui::STATE_CLOSED.get(),
    }
}

/// Whether the ISSUES section of a provider of `family` takes an extra
/// provider-native query (`IssuesSource::filter`): a JQL fragment (Jira), a
/// WIQL condition (Azure), Trello list names, a YouTrack query.
pub fn takes_issue_query(family: ApiFamily) -> bool {
    matches!(
        family,
        ApiFamily::Jira | ApiFamily::AzureDevops | ApiFamily::Trello | ApiFamily::YouTrack
    )
}

/// Whether `pr` matches what was typed in the search box.
pub fn pr_matches(pr: &PullRequestSummary, query: &str) -> bool {
    let q = query.trim().to_lowercase();
    q.is_empty()
        || [&pr.title, &pr.iid_label, &pr.author, &pr.source_branch]
            .iter()
            .any(|f| f.to_lowercase().contains(&q))
}

/// Whether `issue` matches what was typed in the search box.
pub fn issue_matches(issue: &IssueSummary, query: &str) -> bool {
    let q = query.trim().to_lowercase();
    q.is_empty()
        || [&issue.title, &issue.iid_label, &issue.author]
            .iter()
            .any(|f| f.to_lowercase().contains(&q))
        || issue.labels.iter().any(|l| l.to_lowercase().contains(&q))
}

/// Whether the chip of a branch's latest request is drawn: a closed or
/// merged one is hidden when `ui.hide_finished_pr_chip` is on.
pub fn chip_visible(state: PullRequestState, hide_finished: bool) -> bool {
    !(hide_finished && matches!(state, PullRequestState::Closed | PullRequestState::Merged))
}

#[cfg(all(test, feature = "github", feature = "gitlab"))]
mod tests {
    use super::*;
    use crate::hosting_task::fake::{issue, pr};

    fn input<'a>(
        snapshot: Option<&'a HostingSnapshot>,
        collapsed: &'a HashSet<PrGroup>,
    ) -> RowsInput<'a> {
        RowsInput {
            snapshot,
            collapsed,
            pr_query: "",
            issue_query: "",
            loading_pulls: None,
            loading_issues: None,
            fresh: true,
            now: 0,
        }
    }

    #[test]
    fn nothing_is_listed_before_the_first_answer() {
        let none = HashSet::new();
        let view = input(None, &none);
        assert!(view.rows(HostingList::Pulls).is_empty());
        assert_eq!(view.sections_visible(), (false, false));
        let loading = RowsInput {
            loading_pulls: Some('|'),
            ..view
        };
        assert_eq!(loading.sections_visible(), (true, false));
        assert!(matches!(
            loading.rows(HostingList::Pulls).as_slice(),
            [HostingRow::Notice(_)]
        ));
    }

    #[test]
    fn groups_filter_by_the_signed_in_user_and_the_search_text() {
        let mut mine = pr(1, "a", PullRequestState::Open);
        mine.author = "me".into();
        let mut theirs = pr(2, "b", PullRequestState::Open);
        theirs.reviewers = vec!["ME".into()];
        assert!(PrGroup::Mine.holds(&mine, "me"));
        assert!(!PrGroup::Mine.holds(&theirs, "me"));
        assert!(PrGroup::Awaiting.holds(&theirs, "me"));
        assert!(!PrGroup::Mine.holds(&mine, ""), "nobody is signed in");
        assert!(PrGroup::All.holds(&theirs, ""));
        assert!(pr_matches(&mine, " CHANGE number 1 "));
        assert!(!pr_matches(&mine, "zzz"));
        assert!(issue_matches(&issue(7), "#7"));
    }

    #[test]
    fn controls_are_the_rows_a_click_uses() {
        assert!(HostingRow::Search.is_control());
        assert!(HostingRow::Group(PrGroup::All).is_control());
        assert!(!HostingRow::Pull(PrGroup::All, 0).is_control());
        assert!(!HostingRow::Notice(String::new()).is_control());
    }

    #[test]
    fn a_finished_request_hides_its_chip_only_when_asked() {
        assert!(!chip_visible(PullRequestState::Merged, true));
        assert!(chip_visible(PullRequestState::Merged, false));
        assert!(chip_visible(PullRequestState::Open, true));
        assert!(takes_issue_query(ApiFamily::Jira));
        assert!(!takes_issue_query(ApiFamily::Github));
    }
}
