//! The Radar's account-wide rows: requests waiting for my review (and,
//! later, every open request and my assigned issues) of every repository a
//! host knows about, not only the cloned ones of the scope.
//!
//! The worker asks (`worker.rs`, one job per integration with a list turned
//! on) and answers with an [`AccountAnswer`]; this module turns the answers
//! into [`Item`]s, merged into the scope's:
//!
//! - a row the scope already lists (same host, project, kind and number) is
//!   dropped, the scope's row wins;
//! - a row whose project has a known local clone (the [`CloneIndex`], built on
//!   the worker from recents, workspaces and folder scans) carries that
//!   clone's path;
//! - any other row is "not cloned" (`repo` is `None`): dimmed chip, a word on
//!   the status line, and the actions that need a repository refuse.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use gitpoulpi_config::{ApiFamily, Integration, IntegrationKind};
use gitpoulpi_hosting::account::{AccountItems, AccountQuery};
use gitpoulpi_hosting::PullRequestState;
use gitpoulpi_i18n::msg;
use gitpoulpi_session::{ItemKey, KIND_ISSUE, KIND_PR};

use super::model::{Group, Item, ItemKind};
use super::vocab::Act;
use super::Note;
use crate::hosting_task::{age_label, Problem};

/// One integration to ask, with the question.
#[derive(Debug, Clone)]
pub(crate) struct AccountJob {
    pub(crate) integration: Integration,
    pub(crate) query: AccountQuery,
    /// How the integration is named in the Radar.
    pub(crate) label: String,
}

/// A connected code host whose account-wide lists are all off: the Radar
/// offers a `[Turn on]` for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Offer {
    pub(crate) id: String,
    /// `GitHub`, or `GitHub - host` when two profiles share a name.
    pub(crate) label: String,
}

/// What a round asks on top of the scope's repositories.
#[derive(Debug, Clone, Default)]
pub(crate) struct AccountPlan {
    /// Every configured integration (a client resolves its token from them).
    pub(crate) all: Vec<Integration>,
    pub(crate) jobs: Vec<AccountJob>,
    pub(crate) offers: Vec<Offer>,
    /// `<config>/radar-account.json`.
    pub(crate) cache: Option<PathBuf>,
}

/// What the worker says about one integration's account-wide lists.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AccountAnswer {
    pub(crate) round: u64,
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) kind: IntegrationKind,
    /// The integration's host (the first part of an item's key).
    pub(crate) host: String,
    /// The signed-in login on the host; empty when unknown.
    pub(crate) me: String,
    pub(crate) items: AccountItems,
    /// Unix seconds the rows were fetched at.
    pub(crate) fetched_at: u64,
    /// Read from the disk cache, not (yet) from the host.
    pub(crate) cached: bool,
    pub(crate) problem: Option<Problem>,
    /// The host was paused for this round: the reason, as the user reads it.
    pub(crate) paused: Option<String>,
}

/// Clones the machine knows about, by `(host, project)`, lowercase.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct CloneIndex(HashMap<(String, String), PathBuf>);

impl CloneIndex {
    /// Records the clone at `path` whose remote is `url`. The first clone of
    /// a project wins (the scope's repositories are offered first).
    pub(crate) fn add_remote(&mut self, url: &str, path: &Path) {
        if let Some((base, project)) = gitpoulpi_hosting::remote_web_base(url) {
            self.0
                .entry((host_key(&base), project_key(&project)))
                .or_insert_with(|| path.to_path_buf());
        }
    }

    pub(crate) fn find(&self, host: &str, project: &str) -> Option<&PathBuf> {
        self.0.get(&(host_key(host), project_key(project)))
    }

    #[cfg(test)]
    pub(crate) fn of(entries: &[(&str, &str, &Path)]) -> Self {
        let mut index = Self::default();
        for (host, project, path) in entries {
            index
                .0
                .insert((host_key(host), project_key(project)), path.to_path_buf());
        }
        index
    }
}

/// `https://Host:8443` / `host:8443` -> `host`.
fn host_key(host: &str) -> String {
    let rest = host.split_once("://").map_or(host, |(_, r)| r);
    let rest = rest.split('/').next().unwrap_or(rest);
    rest.split(':').next().unwrap_or(rest).to_ascii_lowercase()
}

fn project_key(project: &str) -> String {
    project
        .trim_matches('/')
        .trim_end_matches(".git")
        .to_lowercase()
}

/// An item's identity for deduplication: host and project case-insensitively
/// (a remote may spell them differently from the host's API).
fn identity(key: &ItemKey) -> (String, String, String, u64) {
    (
        host_key(&key.host),
        project_key(&key.repo),
        key.kind.clone(),
        key.number,
    )
}

/// Everything [`items`] reads.
pub(crate) struct AccountInput<'a> {
    pub(crate) answers: &'a [AccountAnswer],
    pub(crate) clones: &'a CloneIndex,
    pub(crate) show_not_cloned: bool,
    pub(crate) show_drafts: bool,
}

/// The account-wide rows to add to `scope_items`: none that the scope already
/// lists, none hidden by the two checkboxes.
pub(crate) fn items(input: &AccountInput, scope_items: &[Item]) -> Vec<Item> {
    let mut seen: HashSet<(String, String, String, u64)> = scope_items
        .iter()
        .filter(|i| i.kind != ItemKind::Wip)
        .map(|i| identity(&i.key))
        .collect();
    let mut out = Vec::new();
    for answer in input.answers {
        let me = answer.me.as_str();
        let is_me = |login: &str| !me.is_empty() && login.eq_ignore_ascii_case(me);
        let gitlab = answer.kind.family() == ApiFamily::Gitlab;
        let meta = crate::meta_picker::supports(answer.kind);
        let review = crate::review_view::supports(answer.kind);
        // The clone a row belongs to, else the "not cloned" marker.
        let place = |project: &str| -> Option<(Option<PathBuf>, String)> {
            match input.clones.find(&answer.host, project) {
                Some(path) => Some((Some(path.clone()), super::scope::name_of(path))),
                None if input.show_not_cloned => Some((None, project.to_string())),
                None => None,
            }
        };
        for row in &answer.items.prs {
            let pr = &row.pr;
            let draft = pr.state == PullRequestState::Draft;
            if !matches!(pr.state, PullRequestState::Open | PullRequestState::Draft)
                || (draft && !input.show_drafts)
            {
                continue;
            }
            let key = ItemKey::new(&answer.host, &row.project, KIND_PR, pr.number);
            if seen.contains(&identity(&key)) {
                continue;
            }
            let Some((repo, repo_name)) = place(&row.project) else {
                continue;
            };
            seen.insert(identity(&key));
            let asked = pr.review_requested || pr.reviewers.iter().any(|r| is_me(r));
            let group = if asked && !is_me(&pr.author) {
                Group::NeedsReview
            } else if is_me(&pr.author) {
                Group::Mine
            } else if pr.assignees.iter().any(|a| is_me(a)) {
                Group::Assigned
            } else {
                Group::OtherPrs
            };
            out.push(Item {
                key,
                kind: ItemKind::Pr,
                group,
                repo,
                repo_name,
                integration: answer.id.clone(),
                project: row.project.clone(),
                clone_url: row.clone_url.clone(),
                number: pr.number,
                label: pr.iid_label.clone(),
                title: pr.title.clone(),
                author: pr.author.clone(),
                url: pr.web_url.clone(),
                branch: Some(pr.source_branch.clone()),
                updated_at: pr.updated_at.clone(),
                labels: Vec::new(),
                draft,
                review_requested: asked,
                fork: pr.source_project.is_some(),
                gitlab,
                meta,
                review,
                wip: None,
                target: pr.target_branch.clone(),
                checks: pr.checks,
                has_conflicts: pr.has_conflicts,
                mentioned: false,
                host_id: answer.id.clone(),
                host_project: row.project.clone(),
                host_label: answer.kind.label().to_string(),
                acts: gitpoulpi_hosting::request_actions(answer.kind),
                fork_owner: pr
                    .source_project
                    .as_deref()
                    .and_then(|p| p.split('/').next())
                    .unwrap_or_default()
                    .to_string(),
            });
        }
        for row in &answer.items.issues {
            let issue = &row.issue;
            if issue.state != gitpoulpi_hosting::IssueState::Open {
                continue;
            }
            let key = ItemKey::new(&answer.host, &row.project, KIND_ISSUE, issue.number);
            if seen.contains(&identity(&key)) {
                continue;
            }
            let Some((repo, repo_name)) = place(&row.project) else {
                continue;
            };
            seen.insert(identity(&key));
            let label = if issue.reference.is_empty() {
                issue.iid_label.clone()
            } else {
                issue.reference.clone()
            };
            out.push(Item {
                key,
                kind: ItemKind::Issue,
                group: Group::IssueAssigned,
                repo,
                repo_name,
                integration: answer.id.clone(),
                project: row.project.clone(),
                clone_url: row.clone_url.clone(),
                number: issue.number,
                label,
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
                meta,
                review: false,
                wip: None,
                target: String::new(),
                checks: None,
                has_conflicts: None,
                mentioned: false,
                host_id: answer.id.clone(),
                host_project: row.project.clone(),
                host_label: answer.kind.label().to_string(),
                acts: gitpoulpi_hosting::request_actions(answer.kind),
                fork_owner: String::new(),
            });
        }
    }
    out
}

/// What each integration says about its account-wide lists, as note rows
/// (under REPOSITORIES): a failure, the host's pause, and the lines the
/// query itself left (rows cut by the limit, a group it could not read).
pub(crate) fn notes(answers: &[AccountAnswer], now: i64) -> Vec<Note> {
    let mut out = Vec::new();
    for answer in answers {
        let plain = |text: String, act| Note {
            path: PathBuf::new(),
            name: answer.label.clone(),
            text,
            act,
        };
        if let Some(why) = &answer.paused {
            out.push(plain(msg::radar::note_paused(why), None));
            continue;
        }
        if let Some(problem) = &answer.problem {
            let profile = Some(answer.id.clone());
            out.push(match problem {
                Problem::NoToken => plain(
                    msg::hosting_ui::reconnect(msg::hosting_ui::WHY_NO_TOKEN.get()),
                    Some((Act::Reconnect, profile)),
                ),
                Problem::Reconnect(_) => plain(
                    msg::hosting_ui::reconnect(msg::hosting_ui::WHY_TOKEN_REJECTED.get()),
                    Some((Act::Reconnect, profile)),
                ),
                Problem::Offline(_) => plain(
                    msg::radar::note_offline(age_label(now, answer.fetched_at as i64)),
                    None,
                ),
                Problem::RateLimited(why) | Problem::Other(why) => plain(why.clone(), None),
            });
            continue;
        }
        out.extend(answer.items.notes.iter().map(|n| plain(n.clone(), None)));
    }
    out
}

#[cfg(test)]
mod tests;
