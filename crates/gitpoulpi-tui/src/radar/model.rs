//! The Radar's data, with no terminal and no network: what the repositories
//! of the scope say (their hosting snapshots, their WIP facts) folded into
//! items, groups, pins and snoozes.
//!
//! Pure on purpose: the clock is a parameter, so "a snooze ends" is a unit
//! test, not a sleep.

use std::collections::HashSet;
use std::path::PathBuf;

use gitpoulpi_core::fuzzy::{self, Anchor};
use gitpoulpi_hosting::{ChecksState, IssueState, PullRequestState};
use gitpoulpi_session::{ItemKey, RadarState, KIND_ISSUE, KIND_PR, KIND_WIP};

use super::account::{self, AccountAnswer, AccountInput, CloneIndex};
use super::worker::RepoAnswer;
use crate::repo_switcher::RepoFacts;

/// What an item is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ItemKind {
    Pr,
    Issue,
    Wip,
}

/// The groups of the Radar, in the order they are drawn. A pull request
/// is in exactly one of the PR groups (the first that applies), an issue in
/// one of the issue groups; [`Group::Mentioned`] takes both kinds, what mentions
/// the user and is neither theirs, asked of them nor assigned to them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum Group {
    NeedsReview,
    Mine,
    Assigned,
    Mentioned,
    OtherPrs,
    IssueAssigned,
    IssueCreated,
    Wip,
}

impl Group {
    pub(crate) const ALL: [Group; 8] = [
        Group::NeedsReview,
        Group::Mine,
        Group::Assigned,
        Group::Mentioned,
        Group::OtherPrs,
        Group::IssueAssigned,
        Group::IssueCreated,
        Group::Wip,
    ];
}

/// What a WIP row says about its repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WipFacts {
    pub(crate) branch: Option<String>,
    pub(crate) modified: u32,
    pub(crate) added: u32,
    pub(crate) ahead: u32,
    pub(crate) behind: u32,
}

impl WipFacts {
    /// A repository belongs in the WIP group when it has uncommitted changes
    /// or commits not pushed yet.
    pub(crate) fn of(facts: &RepoFacts) -> Option<Self> {
        (facts.dirty || facts.ahead > 0 || facts.modified > 0 || facts.added > 0).then(|| Self {
            branch: facts.branch.clone(),
            modified: facts.modified,
            added: facts.added,
            ahead: facts.ahead,
            behind: facts.behind,
        })
    }

    /// What "until it changes" watches for a WIP row.
    fn stamp(&self) -> String {
        format!("{}/{}/{}", self.modified, self.added, self.ahead)
    }
}

/// One row's worth of data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Item {
    pub(crate) key: ItemKey,
    pub(crate) kind: ItemKind,
    pub(crate) group: Group,
    /// The local clone the item belongs to; `None` for an account-wide row
    /// of a repository with no known clone ("not cloned").
    pub(crate) repo: Option<PathBuf>,
    pub(crate) repo_name: String,
    /// The integration an account-wide row came from; empty for the scope's.
    pub(crate) integration: String,
    /// `owner/name` on the host (account-wide rows).
    pub(crate) project: String,
    /// Where to clone the project from (account-wide rows; may be empty).
    pub(crate) clone_url: String,
    pub(crate) number: u64,
    /// `#12`, `!53`; empty for a WIP row.
    pub(crate) label: String,
    pub(crate) title: String,
    pub(crate) author: String,
    pub(crate) url: String,
    /// A request's source branch, a WIP row's current branch.
    pub(crate) branch: Option<String>,
    pub(crate) updated_at: String,
    pub(crate) labels: Vec<String>,
    pub(crate) draft: bool,
    pub(crate) review_requested: bool,
    /// The request comes from a fork (its branch is not in the repository).
    pub(crate) fork: bool,
    pub(crate) gitlab: bool,
    /// The provider takes assignees, reviewers, labels and milestones
    /// (GitHub, GitLab).
    pub(crate) meta: bool,
    /// The in-app review exists for the provider (GitHub, GitLab, Gitea,
    /// Forgejo, Gerrit).
    pub(crate) review: bool,
    pub(crate) wip: Option<WipFacts>,
    /// A request's target branch (the detail line names it for a conflict).
    pub(crate) target: String,
    /// The head's checks, when the host said.
    pub(crate) checks: Option<ChecksState>,
    /// The request conflicts with its target, when the host said.
    pub(crate) has_conflicts: Option<bool>,
    /// The item mentions the user (whatever group it is in).
    pub(crate) mentioned: bool,
    /// The integration the row's host calls go through (scope rows too);
    /// empty for a WIP row.
    pub(crate) host_id: String,
    /// `owner/name` on the host, for every row that has a host.
    pub(crate) host_project: String,
    /// The provider's name (`GitLab`), for "not supported by ...".
    pub(crate) host_label: String,
    /// What the host lets a request or issue do (merge, close, ready).
    pub(crate) acts: gitpoulpi_hosting::RequestActions,
    /// A fork request's source owner (`alice` of `alice/app`); empty when
    /// the source branch is in the repository itself.
    pub(crate) fork_owner: String,
}

impl Item {
    /// `source → target` for a request, `owner:source` when the source is a
    /// fork's; empty when the host did not say. A missing target leaves the
    /// source alone, a missing source leaves nothing.
    pub(crate) fn branches_text(&self) -> String {
        match self.branches_parts() {
            Some((source, Some(target))) => format!("{source} \u{2192} {target}"),
            Some((source, None)) => source,
            None => String::new(),
        }
    }

    /// The branches as `(source, target)` (the source is `owner:branch` for a
    /// fork's); `None` for anything but a request that names its source.
    pub(crate) fn branches_parts(&self) -> Option<(String, Option<String>)> {
        if self.kind != ItemKind::Pr {
            return None;
        }
        let source = self.branch.as_deref().filter(|b| !b.is_empty())?;
        let source = if self.fork_owner.is_empty() {
            source.to_string()
        } else {
            format!("{}:{source}", self.fork_owner)
        };
        Some((source, Some(self.target.clone()).filter(|t| !t.is_empty())))
    }

    /// What "until it changes" watches: the host's `updated_at`, or the WIP
    /// counters.
    pub(crate) fn stamp(&self) -> String {
        match &self.wip {
            Some(w) => w.stamp(),
            None => self.updated_at.clone(),
        }
    }

    /// The text the filter box searches.
    fn haystack(&self) -> String {
        let mut text = format!(
            "{} {} {} {}",
            self.repo_name, self.label, self.title, self.author
        );
        if let Some(branch) = &self.branch {
            text.push(' ');
            text.push_str(branch);
        }
        for label in &self.labels {
            text.push(' ');
            text.push_str(label);
        }
        text.to_lowercase()
    }
}

/// Seconds since the epoch of an ISO-8601 stamp as the hosts send it
/// (`2026-10-14T09:30:00Z`, with or without fractions or an offset).
pub(crate) fn iso_to_unix(text: &str) -> Option<i64> {
    let bytes = text.as_bytes();
    if bytes.len() < 19 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    let num = |a: usize, b: usize| text.get(a..b)?.parse::<i64>().ok();
    let (y, m, d) = (num(0, 4)?, num(5, 7)?, num(8, 10)?);
    let (hh, mm, ss) = (num(11, 13)?, num(14, 16)?, num(17, 19)?);
    // Days since 1970-01-01 (Howard Hinnant's civil-from-days, inverted).
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    let mut secs = days * 86_400 + hh * 3600 + mm * 60 + ss;
    // A numeric offset (`+02:00`) is subtracted; `Z` and fractions are not.
    let rest = text[19..].trim_start_matches(|c: char| c == '.' || c.is_ascii_digit());
    if let Some(sign @ ('+' | '-')) = rest.chars().next() {
        let off = &rest[1..];
        let oh = off.get(0..2).and_then(|s| s.parse::<i64>().ok());
        let om = off
            .get(3..5)
            .or_else(|| off.get(2..4))
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);
        if let Some(oh) = oh {
            let off = oh * 3600 + om * 60;
            secs -= if sign == '+' { off } else { -off };
        }
    }
    Some(secs)
}

/// Everything [`Board::build`] reads.
pub(crate) struct BoardInput<'a> {
    pub(crate) answers: &'a [RepoAnswer],
    /// The account-wide rows, merged into the scope's (see `account`).
    pub(crate) accounts: &'a [AccountAnswer],
    pub(crate) clones: &'a CloneIndex,
    pub(crate) wips: &'a [(PathBuf, String, WipFacts)],
    pub(crate) state: &'a RadarState,
    pub(crate) now: i64,
}

/// The items of the scope, split into the visible ones, the pinned ones and
/// the snoozed ones.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Board {
    pub(crate) items: Vec<Item>,
    /// Indices into `items`, pin order.
    pub(crate) pinned: Vec<usize>,
    /// Indices into `items`, per group, newest first. Pinned and snoozed
    /// items are in neither.
    pub(crate) groups: Vec<(Group, Vec<usize>)>,
    pub(crate) snoozed: Vec<usize>,
}

impl Board {
    pub(crate) fn build(input: &BoardInput) -> Board {
        let mut items: Vec<Item> = Vec::new();
        for answer in input.answers {
            let snap = &answer.snapshot;
            let me = answer.me.as_str();
            let is_me = |login: &str| !me.is_empty() && login.eq_ignore_ascii_case(me);
            let gitlab = snap
                .kind
                .is_some_and(|k| k.family() == gitpoulpi_config::ApiFamily::Gitlab);
            if snap.can_pulls {
                let mention_numbers: HashSet<u64> =
                    snap.prs_mentioned.iter().map(|p| p.number).collect();
                // The mentioned ones the plain list did not carry (the lists
                // are asked separately and may differ by a moment).
                let extra = snap
                    .prs_mentioned
                    .iter()
                    .filter(|m| !snap.prs.iter().any(|p| p.number == m.number));
                for pr in snap.prs.iter().chain(extra) {
                    if !matches!(pr.state, PullRequestState::Open | PullRequestState::Draft) {
                        continue;
                    }
                    let mentioned = mention_numbers.contains(&pr.number);
                    let asked = pr.review_requested || pr.reviewers.iter().any(|r| is_me(r));
                    let group = if asked && !is_me(&pr.author) {
                        Group::NeedsReview
                    } else if is_me(&pr.author) {
                        Group::Mine
                    } else if pr.assignees.iter().any(|a| is_me(a)) {
                        Group::Assigned
                    } else if mentioned {
                        Group::Mentioned
                    } else {
                        Group::OtherPrs
                    };
                    items.push(Item {
                        key: ItemKey::new(&answer.host, &snap.project, KIND_PR, pr.number),
                        kind: ItemKind::Pr,
                        group,
                        repo: Some(answer.path.clone()),
                        repo_name: answer.name.clone(),
                        integration: String::new(),
                        project: String::new(),
                        clone_url: String::new(),
                        number: pr.number,
                        label: pr.iid_label.clone(),
                        title: pr.title.clone(),
                        author: pr.author.clone(),
                        url: pr.web_url.clone(),
                        branch: Some(pr.source_branch.clone()),
                        updated_at: pr.updated_at.clone(),
                        labels: Vec::new(),
                        draft: pr.state == PullRequestState::Draft,
                        review_requested: asked,
                        fork: pr.source_project.is_some(),
                        gitlab,
                        meta: snap.kind.is_some_and(crate::meta_picker::supports),
                        review: snap.kind.is_some_and(crate::review_view::supports),
                        wip: None,
                        target: pr.target_branch.clone(),
                        checks: pr.checks,
                        has_conflicts: pr.has_conflicts,
                        mentioned,
                        host_id: snap.profile_id.clone().unwrap_or_default(),
                        host_project: snap.project.clone(),
                        host_label: snap.kind.map(|k| k.label().to_string()).unwrap_or_default(),
                        acts: snap
                            .kind
                            .map(gitpoulpi_hosting::request_actions)
                            .unwrap_or_default(),
                        fork_owner: pr
                            .source_project
                            .as_deref()
                            .and_then(|p| p.split('/').next())
                            .unwrap_or_default()
                            .to_string(),
                    });
                }
            }
            if let Some(target) = snap.issues_target.as_ref() {
                let mention_numbers: HashSet<u64> =
                    snap.issues_mentioned.iter().map(|i| i.number).collect();
                let extra = snap
                    .issues_mentioned
                    .iter()
                    .filter(|m| !snap.issues.iter().any(|i| i.number == m.number));
                for issue in snap.issues.iter().chain(extra) {
                    if issue.state != IssueState::Open {
                        continue;
                    }
                    let mentioned = mention_numbers.contains(&issue.number);
                    let group = if issue.assignees.iter().any(|a| is_me(a)) {
                        Group::IssueAssigned
                    } else if is_me(&issue.author) {
                        Group::IssueCreated
                    } else if mentioned {
                        Group::Mentioned
                    } else {
                        // Neither assigned to, created by nor mentioning the
                        // user: not the Radar's.
                        continue;
                    };
                    let reference = if issue.reference.is_empty() {
                        issue.iid_label.clone()
                    } else {
                        issue.reference.clone()
                    };
                    items.push(Item {
                        key: ItemKey::new(&answer.host, &target.project, KIND_ISSUE, issue.number),
                        kind: ItemKind::Issue,
                        group,
                        repo: Some(answer.path.clone()),
                        repo_name: answer.name.clone(),
                        integration: String::new(),
                        project: String::new(),
                        clone_url: String::new(),
                        number: issue.number,
                        label: reference,
                        title: issue.title.clone(),
                        author: issue.author.clone(),
                        url: issue.web_url.clone(),
                        branch: None,
                        updated_at: issue.updated_at.clone(),
                        labels: issue.labels.clone(),
                        draft: false,
                        review_requested: false,
                        fork: false,
                        gitlab,
                        meta: crate::meta_picker::supports(target.profile.kind),
                        review: false,
                        wip: None,
                        target: String::new(),
                        checks: None,
                        has_conflicts: None,
                        mentioned,
                        host_id: target.profile.id.clone(),
                        host_project: target.project.clone(),
                        host_label: target.profile.kind.label().to_string(),
                        acts: gitpoulpi_hosting::request_actions(target.profile.kind),
                        fork_owner: String::new(),
                    });
                }
            }
        }
        let more = account::items(
            &AccountInput {
                answers: input.accounts,
                clones: input.clones,
                show_not_cloned: input.state.show_not_cloned,
                show_drafts: input.state.show_drafts,
            },
            &items,
        );
        items.extend(more);
        for (path, name, facts) in input.wips {
            items.push(Item {
                key: ItemKey::new("", &path.to_string_lossy(), KIND_WIP, 0),
                kind: ItemKind::Wip,
                group: Group::Wip,
                repo: Some(path.clone()),
                repo_name: name.clone(),
                integration: String::new(),
                project: String::new(),
                clone_url: String::new(),
                number: 0,
                label: String::new(),
                title: facts.branch.clone().unwrap_or_default(),
                author: String::new(),
                url: String::new(),
                branch: facts.branch.clone(),
                updated_at: String::new(),
                labels: Vec::new(),
                draft: false,
                review_requested: false,
                fork: false,
                gitlab: false,
                meta: false,
                review: false,
                wip: Some(facts.clone()),
                target: String::new(),
                checks: None,
                has_conflicts: None,
                mentioned: false,
                host_id: String::new(),
                host_project: String::new(),
                host_label: String::new(),
                acts: Default::default(),
                fork_owner: String::new(),
            });
        }
        // Newest first inside a group (ISO stamps sort chronologically); a
        // stable order for the rest: the repository, then the number.
        items.sort_by(|a, b| {
            b.updated_at
                .cmp(&a.updated_at)
                .then_with(|| a.repo_name.cmp(&b.repo_name))
                .then_with(|| a.number.cmp(&b.number))
        });
        let mut board = Board {
            items,
            ..Board::default()
        };
        let mut pinned: Vec<(usize, usize)> = Vec::new();
        let mut by_group: Vec<(Group, Vec<usize>)> =
            Group::ALL.iter().map(|g| (*g, Vec::new())).collect();
        for (i, item) in board.items.iter().enumerate() {
            if input
                .state
                .snoozed(&item.key, input.now, &item.stamp())
                .is_some()
            {
                board.snoozed.push(i);
            } else if let Some(pos) = input.state.pins.iter().position(|p| *p == item.key) {
                pinned.push((pos, i));
            } else if let Some((_, list)) = by_group.iter_mut().find(|(g, _)| *g == item.group) {
                list.push(i);
            }
        }
        pinned.sort();
        board.pinned = pinned.into_iter().map(|(_, i)| i).collect();
        board.groups = by_group;
        board
    }
}

/// Which kinds of item the view shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum View {
    #[default]
    All,
    Prs,
    Issues,
    Wips,
    Snoozed,
}

impl View {
    pub(crate) const ALL: [View; 5] = [
        View::All,
        View::Prs,
        View::Issues,
        View::Wips,
        View::Snoozed,
    ];

    pub(crate) fn key(self) -> &'static str {
        match self {
            View::All => "all",
            View::Prs => "prs",
            View::Issues => "issues",
            View::Wips => "wips",
            View::Snoozed => "snoozed",
        }
    }

    pub(crate) fn from_key(key: &str) -> Option<View> {
        View::ALL.into_iter().find(|v| v.key() == key)
    }

    fn shows(self, kind: ItemKind) -> bool {
        match self {
            View::All | View::Snoozed => true,
            View::Prs => kind == ItemKind::Pr,
            View::Issues => kind == ItemKind::Issue,
            View::Wips => kind == ItemKind::Wip,
        }
    }
}

/// The second filter: whose items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Who {
    #[default]
    Everyone,
    /// The user's own: requests they wrote, issues they created.
    Mine,
    Assigned,
    ReviewRequested,
    /// What mentions the user, in any group.
    Mentioned,
}

impl Who {
    pub(crate) const ALL: [Who; 5] = [
        Who::Everyone,
        Who::Mine,
        Who::Assigned,
        Who::ReviewRequested,
        Who::Mentioned,
    ];

    pub(crate) fn key(self) -> &'static str {
        match self {
            Who::Everyone => "everyone",
            Who::Mine => "mine",
            Who::Assigned => "assigned",
            Who::ReviewRequested => "review",
            Who::Mentioned => "mentioned",
        }
    }

    pub(crate) fn from_key(key: &str) -> Option<Who> {
        Who::ALL.into_iter().find(|w| w.key() == key)
    }

    fn admits(self, item: &Item) -> bool {
        match self {
            Who::Everyone => true,
            Who::Mine => matches!(item.group, Group::Mine | Group::IssueCreated),
            Who::Assigned => matches!(item.group, Group::Assigned | Group::IssueAssigned),
            Who::ReviewRequested => item.group == Group::NeedsReview,
            Who::Mentioned => item.mentioned,
        }
    }
}

/// A section heading of the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Heading {
    Pinned,
    Group(Group),
    Snoozed,
    /// Repositories with something to say (offline, Reconnect, Link...).
    Repos,
}

/// One line of the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Row {
    Heading(Heading, usize),
    /// An item, as an index into the board's items.
    Item(usize),
    /// A repository's note: index into the view's `answers`.
    Note(usize),
    /// A code host whose account-wide lists are off, with a `[Turn on]`:
    /// index into the Radar's offers.
    Hint(usize),
    /// Nothing to show (and why).
    Empty,
    /// A breathing line before REPOSITORIES when something is listed above
    /// it. Never selectable, never hit (see `RadarUi::step`).
    Blank,
}

/// The two item filters besides the view: whose items, and which integration
/// profile's (`""` = every profile).
#[derive(Debug, Clone, Copy)]
pub(crate) struct Narrow<'a> {
    pub(crate) who: Who,
    pub(crate) profile: &'a str,
}

impl Narrow<'_> {
    /// Everyone's items, whatever the profile.
    #[cfg(test)]
    pub(crate) fn of(who: Who) -> Narrow<'static> {
        Narrow { who, profile: "" }
    }
}

/// The rows of `board` for the view, the filter, the integration profile
/// (`""` = every profile; a WIP row has no profile, so a chosen one hides it),
/// the folds and the text.
/// `notes` says how many repository notes there are (they end the list);
/// `hints` how many hosts offer their account-wide lists (they open it).
pub(crate) fn rows(
    board: &Board,
    view: View,
    narrow: Narrow,
    text: &str,
    folded: &HashSet<Heading>,
    notes: usize,
    hints: usize,
) -> Vec<Row> {
    let query = text.trim().to_lowercase();
    let admit = |i: usize| {
        let item = &board.items[i];
        view.shows(item.kind)
            && narrow.who.admits(item)
            && (narrow.profile.is_empty() || item.host_id == narrow.profile)
            && (query.is_empty() || fuzzy::score(&item.haystack(), &query, Anchor::Whole).is_some())
    };
    let mut out: Vec<Row> = Vec::new();
    let section = |heading: Heading, list: Vec<usize>, out: &mut Vec<Row>| {
        if list.is_empty() {
            return;
        }
        out.push(Row::Heading(heading, list.len()));
        if !folded.contains(&heading) {
            out.extend(list.into_iter().map(Row::Item));
        }
    };
    if view == View::Snoozed {
        let list: Vec<usize> = board
            .snoozed
            .iter()
            .copied()
            .filter(|&i| admit(i))
            .collect();
        if !list.is_empty() {
            out.push(Row::Heading(Heading::Snoozed, list.len()));
            out.extend(list.into_iter().map(Row::Item));
        }
    } else {
        if view == View::All {
            out.extend((0..hints).map(Row::Hint));
        }
        // Pinned rows on top, whatever their group (but still the view's
        // kinds and the filters).
        let pinned: Vec<usize> = board.pinned.iter().copied().filter(|&i| admit(i)).collect();
        section(Heading::Pinned, pinned, &mut out);
        for (group, list) in &board.groups {
            let list: Vec<usize> = list.iter().copied().filter(|&i| admit(i)).collect();
            section(Heading::Group(*group), list, &mut out);
        }
        // The snoozed group stays folded away until asked for: a heading
        // with its count (the Snoozed view lists it).
        let snoozed = board.snoozed.iter().filter(|&&i| admit(i)).count();
        if snoozed > 0 && view == View::All {
            out.push(Row::Heading(Heading::Snoozed, snoozed));
            if folded.contains(&Heading::Snoozed) {
                // Folded is the default (see `RadarUi::new`): no rows.
            } else {
                out.extend(
                    board
                        .snoozed
                        .iter()
                        .copied()
                        .filter(|&i| admit(i))
                        .map(Row::Item),
                );
            }
        }
        if view == View::All && notes > 0 {
            // Owner's ask: air between the work sections and REPOSITORIES
            // (only that heading, not every one).
            if !out.is_empty() {
                out.push(Row::Blank);
            }
            out.push(Row::Heading(Heading::Repos, notes));
            if !folded.contains(&Heading::Repos) {
                out.extend((0..notes).map(Row::Note));
            }
        }
    }
    if out.is_empty() {
        out.push(Row::Empty);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hosting_task::{HostingSnapshot, IssuesTarget, ProfileRef};
    use gitpoulpi_hosting::{IssueSummary, PullRequestSummary};

    fn pr(n: u64, author: &str, updated: &str) -> PullRequestSummary {
        PullRequestSummary {
            number: n,
            iid_label: format!("#{n}"),
            title: format!("Change {n}"),
            state: PullRequestState::Open,
            source_branch: format!("feat-{n}"),
            target_branch: "main".into(),
            source_project: None,
            author: author.into(),
            web_url: format!("https://github.com/acme/app/pull/{n}"),
            updated_at: updated.into(),
            review_requested: false,
            assignees: vec![],
            reviewers: vec![],
            checks: None,
            has_conflicts: None,
        }
    }

    fn issue(n: u64, author: &str, assignee: Option<&str>) -> IssueSummary {
        IssueSummary {
            number: n,
            iid_label: format!("#{n}"),
            reference: format!("#{n}"),
            title: format!("Issue {n}"),
            author: author.into(),
            assignees: assignee.map(|a| vec![a.to_string()]).unwrap_or_default(),
            updated_at: "2026-10-01T00:00:00Z".into(),
            ..IssueSummary::default()
        }
    }

    fn answer(name: &str, prs: Vec<PullRequestSummary>, issues: Vec<IssueSummary>) -> RepoAnswer {
        let mut snap = crate::radar::worker::empty_snapshot();
        snap.can_pulls = true;
        snap.project = "acme/app".into();
        snap.prs = prs;
        snap.issues = issues;
        snap.issues_target = Some(IssuesTarget {
            profile: ProfileRef {
                id: "gh".into(),
                label: "GitHub".into(),
                kind: gitpoulpi_config::IntegrationKind::Github,
            },
            project: "acme/app".into(),
            chosen: true,
        });
        let snap: HostingSnapshot = snap;
        RepoAnswer {
            path: PathBuf::from(format!("/repos/{name}")),
            name: name.into(),
            host: "github.com".into(),
            me: "kai".into(),
            snapshot: snap,
            cached: false,
            round: 1,
            paused: None,
        }
    }

    fn build(answers: &[RepoAnswer], state: &RadarState, now: i64) -> Board {
        Board::build(&BoardInput {
            answers,
            accounts: &[],
            clones: &CloneIndex::default(),
            wips: &[],
            state,
            now,
        })
    }

    #[test]
    fn a_request_lands_in_the_first_group_that_applies() {
        let mut asked = pr(1, "ann", "2026-10-05T00:00:00Z");
        asked.review_requested = true;
        let mut asked_mine = pr(2, "kai", "2026-10-05T00:00:00Z");
        asked_mine.reviewers = vec!["kai".into()];
        let mut assigned = pr(3, "ann", "2026-10-05T00:00:00Z");
        assigned.assignees = vec!["KAI".into()];
        let other = pr(4, "bob", "2026-10-05T00:00:00Z");
        let mut merged = pr(5, "kai", "2026-10-05T00:00:00Z");
        merged.state = PullRequestState::Merged;
        let a = answer(
            "app",
            vec![asked, asked_mine, assigned, other, merged],
            vec![],
        );
        let board = build(&[a], &RadarState::default(), 0);
        let group_of = |n: u64| board.items.iter().find(|i| i.number == n).map(|i| i.group);
        assert_eq!(group_of(1), Some(Group::NeedsReview));
        // Your own request asking for your review is yours, not "needs review".
        assert_eq!(group_of(2), Some(Group::Mine));
        assert_eq!(group_of(3), Some(Group::Assigned));
        assert_eq!(group_of(4), Some(Group::OtherPrs));
        assert_eq!(group_of(5), None, "a merged request is not on the Radar");
    }

    #[test]
    fn issues_are_the_users_or_not_listed() {
        let a = answer(
            "app",
            vec![],
            vec![
                issue(1, "ann", Some("kai")),
                issue(2, "kai", None),
                issue(3, "ann", None),
            ],
        );
        let board = build(&[a], &RadarState::default(), 0);
        let group_of = |n: u64| board.items.iter().find(|i| i.number == n).map(|i| i.group);
        assert_eq!(group_of(1), Some(Group::IssueAssigned));
        assert_eq!(group_of(2), Some(Group::IssueCreated));
        assert_eq!(group_of(3), None);
    }

    #[test]
    fn newest_first_inside_a_group() {
        let a = answer(
            "app",
            vec![
                pr(1, "bob", "2026-10-01T00:00:00Z"),
                pr(2, "bob", "2026-10-09T00:00:00Z"),
                pr(3, "bob", "2026-10-05T00:00:00Z"),
            ],
            vec![],
        );
        let board = build(&[a], &RadarState::default(), 0);
        let (_, others) = board
            .groups
            .iter()
            .find(|(g, _)| *g == Group::OtherPrs)
            .unwrap();
        let numbers: Vec<u64> = others.iter().map(|&i| board.items[i].number).collect();
        assert_eq!(numbers, vec![2, 3, 1]);
    }

    #[test]
    fn pins_come_first_in_pin_order_and_leave_their_group() {
        let a = answer(
            "app",
            vec![
                pr(1, "bob", "2026-10-09T00:00:00Z"),
                pr(2, "bob", "2026-10-05T00:00:00Z"),
                pr(3, "kai", "2026-10-01T00:00:00Z"),
            ],
            vec![],
        );
        let mut state = RadarState::default();
        state.toggle_pin(&ItemKey::new("github.com", "acme/app", KIND_PR, 3));
        state.toggle_pin(&ItemKey::new("github.com", "acme/app", KIND_PR, 2));
        let board = build(&[a], &state, 0);
        let numbers: Vec<u64> = board
            .pinned
            .iter()
            .map(|&i| board.items[i].number)
            .collect();
        assert_eq!(numbers, vec![3, 2]);
        let rows = rows(
            &board,
            View::All,
            Narrow::of(Who::Everyone),
            "",
            &HashSet::new(),
            0,
            0,
        );
        assert!(matches!(rows[0], Row::Heading(Heading::Pinned, 2)));
        let in_groups: usize = board.groups.iter().map(|(_, l)| l.len()).sum();
        assert_eq!(in_groups, 1);
    }

    #[test]
    fn a_snooze_hides_an_item_until_the_clock_passes_it() {
        let a = answer("app", vec![pr(1, "bob", "2026-10-09T00:00:00Z")], vec![]);
        let key = ItemKey::new("github.com", "acme/app", KIND_PR, 1);
        let mut state = RadarState::default();
        state.snooze(&key, gitpoulpi_session::SnoozeFor::Hour, 1000, "");
        let hidden = build(std::slice::from_ref(&a), &state, 1500);
        assert_eq!(hidden.snoozed.len(), 1);
        assert!(hidden.groups.iter().all(|(_, l)| l.is_empty()));
        let back = build(std::slice::from_ref(&a), &state, 4600);
        assert!(back.snoozed.is_empty());
        assert_eq!(back.groups.iter().map(|(_, l)| l.len()).sum::<usize>(), 1);
    }

    #[test]
    fn until_changed_comes_back_when_the_host_updates_the_item() {
        let key = ItemKey::new("github.com", "acme/app", KIND_PR, 1);
        let mut state = RadarState::default();
        state.snooze(
            &key,
            gitpoulpi_session::SnoozeFor::UntilChanged,
            0,
            "2026-10-09T00:00:00Z",
        );
        let same = answer("app", vec![pr(1, "bob", "2026-10-09T00:00:00Z")], vec![]);
        assert_eq!(build(&[same], &state, 99).snoozed.len(), 1);
        let moved = answer("app", vec![pr(1, "bob", "2026-10-10T08:00:00Z")], vec![]);
        assert!(build(&[moved], &state, 99).snoozed.is_empty());
    }

    #[test]
    fn views_and_filters_narrow_the_rows() {
        let mut asked = pr(1, "ann", "2026-10-05T00:00:00Z");
        asked.review_requested = true;
        let a = answer(
            "app",
            vec![asked, pr(2, "kai", "2026-10-04T00:00:00Z")],
            vec![issue(3, "kai", None)],
        );
        let board = build(&[a], &RadarState::default(), 0);
        let none = HashSet::new();
        let count = |view, who, text: &str| {
            rows(&board, view, Narrow::of(who), text, &none, 0, 0)
                .iter()
                .filter(|r| matches!(r, Row::Item(_)))
                .count()
        };
        assert_eq!(count(View::All, Who::Everyone, ""), 3);
        assert_eq!(count(View::Prs, Who::Everyone, ""), 2);
        assert_eq!(count(View::Issues, Who::Everyone, ""), 1);
        assert_eq!(count(View::All, Who::Mine, ""), 2);
        assert_eq!(count(View::All, Who::ReviewRequested, ""), 1);
        assert_eq!(count(View::All, Who::Assigned, ""), 0);
        assert_eq!(count(View::All, Who::Everyone, "change 2"), 1);
        assert_eq!(count(View::All, Who::Everyone, "zzzz"), 0);
    }

    #[test]
    fn a_folded_group_keeps_its_heading_and_count_only() {
        let a = answer("app", vec![pr(1, "bob", "2026-10-05T00:00:00Z")], vec![]);
        let board = build(&[a], &RadarState::default(), 0);
        let folded = HashSet::from([Heading::Group(Group::OtherPrs)]);
        let r = rows(
            &board,
            View::All,
            Narrow::of(Who::Everyone),
            "",
            &folded,
            0,
            0,
        );
        assert_eq!(r, vec![Row::Heading(Heading::Group(Group::OtherPrs), 1)]);
    }

    #[test]
    fn a_wip_row_is_keyed_by_its_path() {
        let facts = RepoFacts {
            branch: Some("main".into()),
            dirty: true,
            modified: 2,
            ..RepoFacts::default()
        };
        let wip = WipFacts::of(&facts).unwrap();
        assert!(WipFacts::of(&RepoFacts::default()).is_none());
        let wips = vec![(PathBuf::from("/repos/app"), "app".to_string(), wip)];
        let board = Board::build(&BoardInput {
            answers: &[],
            accounts: &[],
            clones: &CloneIndex::default(),
            wips: &wips,
            state: &RadarState::default(),
            now: 0,
        });
        assert_eq!(board.items[0].key.kind, KIND_WIP);
        assert_eq!(board.items[0].key.repo, "/repos/app");
        assert_eq!(board.items[0].stamp(), "2/0/0");
    }

    #[test]
    fn a_blank_row_precedes_repositories_only_when_something_is_above_it() {
        let facts = RepoFacts {
            branch: Some("main".into()),
            dirty: true,
            modified: 1,
            ..RepoFacts::default()
        };
        let wips = vec![(
            PathBuf::from("/repos/app"),
            "app".to_string(),
            WipFacts::of(&facts).unwrap(),
        )];
        let with_wip = Board::build(&BoardInput {
            answers: &[],
            accounts: &[],
            clones: &CloneIndex::default(),
            wips: &wips,
            state: &RadarState::default(),
            now: 0,
        });
        let none = HashSet::new();
        let r = rows(
            &with_wip,
            View::All,
            Narrow::of(Who::Everyone),
            "",
            &none,
            1,
            0,
        );
        assert_eq!(
            r,
            vec![
                Row::Heading(Heading::Group(Group::Wip), 1),
                Row::Item(0),
                Row::Blank,
                Row::Heading(Heading::Repos, 1),
                Row::Note(0),
            ]
        );
        // Nothing above REPOSITORIES (here: the filter hides the WIP row).
        let r = rows(
            &with_wip,
            View::All,
            Narrow::of(Who::Everyone),
            "zzzz",
            &none,
            1,
            0,
        );
        assert_eq!(
            r,
            vec![Row::Heading(Heading::Repos, 1), Row::Note(0)],
            "no blank first"
        );
        // Folded REPOSITORIES keeps the gap too.
        let folded = HashSet::from([Heading::Repos]);
        let r = rows(
            &with_wip,
            View::All,
            Narrow::of(Who::Everyone),
            "",
            &folded,
            1,
            0,
        );
        assert_eq!(r[2], Row::Blank);
    }

    #[test]
    fn iso_stamps_become_unix_seconds() {
        assert_eq!(iso_to_unix("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(iso_to_unix("2026-10-14T09:30:00Z"), Some(1_791_970_200));
        assert_eq!(iso_to_unix("2026-10-14T09:30:00.123Z"), Some(1_791_970_200));
        assert_eq!(
            iso_to_unix("2026-10-14T11:30:00+02:00"),
            Some(1_791_970_200)
        );
        assert_eq!(iso_to_unix("nonsense"), None);
    }
}
