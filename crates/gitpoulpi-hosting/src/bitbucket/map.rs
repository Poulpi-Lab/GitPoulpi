//! Wire shapes of the Bitbucket Cloud API and their mapping to the
//! host-agnostic model.

use crate::model::{HostedUser, PullRequestState, PullRequestSummary};
use crate::project::{HostedProject, HostedSshKey};

#[derive(serde::Deserialize)]
pub(super) struct Page<T> {
    #[serde(default = "Vec::new")]
    pub values: Vec<T>,
    #[serde(default)]
    pub next: Option<String>,
}

#[derive(serde::Deserialize, Default)]
pub(super) struct Link {
    #[serde(default)]
    pub href: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(serde::Deserialize, Default)]
pub(super) struct Account {
    #[serde(default)]
    pub uuid: String,
    #[serde(default)]
    pub nickname: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub links: AccountLinks,
}

#[derive(serde::Deserialize, Default)]
pub(super) struct AccountLinks {
    #[serde(default)]
    pub html: Link,
}

impl Account {
    /// The name a person recognises: display name, else nickname.
    pub fn shown(&self) -> String {
        self.display_name
            .clone()
            .filter(|n| !n.is_empty())
            .or_else(|| self.nickname.clone())
            .unwrap_or_default()
    }

    pub fn into_user(self) -> HostedUser {
        HostedUser {
            login: self
                .nickname
                .clone()
                .filter(|n| !n.is_empty())
                .or_else(|| self.display_name.clone())
                .unwrap_or_default(),
            name: self.display_name.unwrap_or_default(),
            id: self.uuid,
            web_url: self.links.html.href,
        }
    }
}

#[derive(serde::Deserialize)]
pub(super) struct WorkspaceAccess {
    pub workspace: WorkspaceBase,
}

#[derive(serde::Deserialize)]
pub(super) struct WorkspaceBase {
    pub slug: String,
}

#[derive(serde::Deserialize)]
pub(super) struct Repo {
    pub full_name: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "yes")]
    pub is_private: bool,
    #[serde(default)]
    pub mainbranch: Option<Branch>,
    #[serde(default)]
    pub links: RepoLinks,
}

fn yes() -> bool {
    true
}

#[derive(serde::Deserialize)]
pub(super) struct Branch {
    pub name: String,
}

#[derive(serde::Deserialize, Default)]
pub(super) struct RepoLinks {
    #[serde(default)]
    pub html: Link,
    #[serde(default)]
    pub clone: Vec<Link>,
}

/// `https://user@bitbucket.org/ws/repo.git` -> without the `user@`, which the
/// API embeds and which would otherwise leak into the clone remote.
fn without_user(url: &str) -> String {
    match url.split_once("://") {
        Some((scheme, rest)) => match rest.split_once('@') {
            Some((user, tail)) if !user.contains('/') => format!("{scheme}://{tail}"),
            _ => url.to_string(),
        },
        None => url.to_string(),
    }
}

impl Repo {
    pub fn into_project(self, integration_id: &str) -> HostedProject {
        let clone = |kind: &str| {
            self.links
                .clone
                .iter()
                .find(|l| l.name.as_deref() == Some(kind))
                .and_then(|l| l.href.clone())
        };
        let https_url = clone("https").map(|u| without_user(&u));
        let ssh_url = clone("ssh");
        let name = if self.name.is_empty() {
            self.full_name
                .rsplit('/')
                .next()
                .unwrap_or_default()
                .to_string()
        } else {
            self.name.clone()
        };
        HostedProject {
            integration_id: integration_id.to_string(),
            id: self.full_name.clone(),
            name,
            full_name: self.full_name,
            description: self.description.filter(|d| !d.is_empty()),
            default_branch: self.mainbranch.map(|b| b.name),
            ssh_url,
            https_url,
            web_url: self.links.html.href,
            private: self.is_private,
        }
    }
}

#[derive(serde::Deserialize)]
pub(super) struct SshKey {
    #[serde(default)]
    pub uuid: String,
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub comment: Option<String>,
}

impl SshKey {
    pub fn into_key(self) -> HostedSshKey {
        HostedSshKey {
            id: self.uuid,
            title: self
                .label
                .filter(|l| !l.is_empty())
                .or(self.comment)
                .unwrap_or_default(),
            key: self.key,
        }
    }
}

#[derive(serde::Deserialize)]
pub(super) struct PullRequest {
    pub id: u64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub draft: bool,
    pub source: Endpoint,
    pub destination: Endpoint,
    #[serde(default)]
    pub author: Option<Account>,
    #[serde(default)]
    pub updated_on: String,
    #[serde(default)]
    pub links: PrLinks,
    #[serde(default)]
    pub reviewers: Vec<Account>,
    #[serde(default)]
    pub participants: Vec<Participant>,
}

#[derive(serde::Deserialize, Default)]
pub(super) struct PrLinks {
    #[serde(default)]
    pub html: Link,
}

#[derive(serde::Deserialize)]
pub(super) struct Endpoint {
    #[serde(default)]
    pub branch: Option<Branch>,
    /// `null` once the fork is deleted.
    #[serde(default)]
    pub repository: Option<RepoRef>,
}

#[derive(serde::Deserialize)]
pub(super) struct RepoRef {
    #[serde(default)]
    pub full_name: String,
}

#[derive(serde::Deserialize)]
pub(super) struct Participant {
    #[serde(default)]
    pub user: Option<Account>,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub approved: bool,
}

impl PullRequest {
    /// `me` is the current user's uuid when known (for `review_requested`).
    pub fn into_summary(self, project: &str, me: Option<&str>) -> PullRequestSummary {
        let state = match self.state.as_str() {
            "MERGED" => PullRequestState::Merged,
            "DECLINED" | "SUPERSEDED" => PullRequestState::Closed,
            _ if self.draft => PullRequestState::Draft,
            _ => PullRequestState::Open,
        };
        let source_project = self
            .source
            .repository
            .as_ref()
            .map(|r| r.full_name.clone())
            .filter(|f| !f.is_empty() && !f.eq_ignore_ascii_case(project));
        let mut reviewers: Vec<String> = self
            .participants
            .iter()
            .filter(|p| p.role == "REVIEWER")
            .filter_map(|p| p.user.as_ref().map(Account::shown))
            .collect();
        for r in &self.reviewers {
            let name = r.shown();
            if !name.is_empty() && !reviewers.contains(&name) {
                reviewers.push(name);
            }
        }
        let review_requested = me.is_some_and(|me| {
            self.participants.iter().any(|p| {
                p.role == "REVIEWER" && !p.approved && p.user.as_ref().is_some_and(|u| u.uuid == me)
            }) || self.reviewers.iter().any(|r| r.uuid == me)
        }) && state != PullRequestState::Merged
            && state != PullRequestState::Closed;
        let number = self.id;
        PullRequestSummary {
            number,
            iid_label: format!("#{number}"),
            title: self.title,
            state,
            source_branch: self.source.branch.map(|b| b.name).unwrap_or_default(),
            target_branch: self.destination.branch.map(|b| b.name).unwrap_or_default(),
            source_project,
            author: self.author.map(|a| a.shown()).unwrap_or_default(),
            web_url: self.links.html.href.unwrap_or_default(),
            updated_at: self.updated_on,
            review_requested,
            assignees: Vec::new(),
            reviewers,
            checks: None,
            has_conflicts: None,
        }
    }
}
