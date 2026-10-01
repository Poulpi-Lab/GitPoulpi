//! Trello JSON to the host-agnostic model.

use std::collections::HashMap;

use serde::Deserialize;

use crate::model::{Choice, HostedUser, IssueCreateOptions, IssueState, IssueSummary};
use crate::project::HostedProject;

/// The card fields the list asks for (the default is every field).
pub(super) const CARD_FIELDS: &str =
    "name,idShort,shortUrl,url,closed,idList,idMembers,labels,dateLastActivity";

#[derive(Deserialize, Default)]
pub(super) struct TrelloMember {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub username: String,
    #[serde(rename = "fullName", default)]
    pub full_name: String,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct TrelloBoard {
    #[serde(default)]
    id: String,
    #[serde(default)]
    name: String,
    #[serde(rename = "shortLink", default)]
    short_link: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    desc: Option<String>,
    #[serde(rename = "prefs", default)]
    prefs: Option<Prefs>,
}

#[derive(Deserialize)]
struct Prefs {
    #[serde(rename = "permissionLevel", default)]
    permission_level: String,
}

#[derive(Deserialize)]
pub(super) struct TrelloList {
    pub id: String,
    #[serde(default)]
    pub name: String,
}

#[derive(Deserialize)]
struct Label {
    #[serde(default)]
    name: String,
    #[serde(default)]
    color: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct TrelloCard {
    #[serde(default)]
    id: String,
    #[serde(default)]
    name: String,
    #[serde(rename = "idShort", default)]
    id_short: u64,
    #[serde(rename = "shortUrl", default)]
    short_url: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    closed: bool,
    #[serde(rename = "idList", default)]
    pub id_list: String,
    #[serde(rename = "idMembers", default)]
    pub id_members: Vec<String>,
    #[serde(default)]
    members: Vec<TrelloMember>,
    #[serde(default)]
    labels: Vec<Label>,
    #[serde(rename = "dateLastActivity", default)]
    date_last_activity: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct CreatedCard {
    #[serde(rename = "idShort", default)]
    id_short: u64,
    #[serde(rename = "shortUrl", default)]
    short_url: String,
    #[serde(default)]
    url: String,
}

impl CreatedCard {
    pub fn number(&self) -> u64 {
        self.id_short
    }
    pub fn web_url(&self) -> String {
        if self.short_url.is_empty() {
            self.url.clone()
        } else {
            self.short_url.clone()
        }
    }
}

pub(super) fn map_board(integration_id: &str, row: TrelloBoard) -> HostedProject {
    let key = if row.short_link.is_empty() {
        row.id.clone()
    } else {
        row.short_link.clone()
    };
    HostedProject {
        integration_id: integration_id.to_string(),
        id: row.id,
        name: if row.name.is_empty() {
            key.clone()
        } else {
            row.name
        },
        full_name: key,
        description: row.desc.filter(|d| !d.trim().is_empty()),
        default_branch: None,
        ssh_url: None,
        https_url: None,
        web_url: Some(row.url).filter(|u| !u.is_empty()),
        private: row
            .prefs
            .is_some_and(|p| p.permission_level == "private" || p.permission_level == "org"),
    }
}

pub(super) fn map_user(u: TrelloMember) -> HostedUser {
    HostedUser {
        login: u.username,
        name: u.full_name,
        id: u.id,
        web_url: u.url.filter(|u| !u.is_empty()),
    }
}

/// `names` are the card's list names by id.
pub(super) fn map_card(lists: &HashMap<String, String>, row: TrelloCard) -> IssueSummary {
    let assignees = row
        .members
        .iter()
        .map(|m| {
            if m.full_name.is_empty() {
                m.username.clone()
            } else {
                m.full_name.clone()
            }
        })
        .filter(|n| !n.is_empty())
        .collect();
    let web_url = if row.short_url.is_empty() {
        row.url
    } else {
        row.short_url.clone()
    };
    IssueSummary {
        branch_name: None,
        number: row.id_short,
        id: row.id,
        reference: if row.short_url.is_empty() {
            web_url.clone()
        } else {
            row.short_url
        },
        kind_label: lists.get(&row.id_list).cloned().filter(|n| !n.is_empty()),
        status_label: None,
        iid_label: format!("#{}", row.id_short),
        title: row.name,
        state: if row.closed {
            IssueState::Closed
        } else {
            IssueState::Open
        },
        // A card has no author; the creator is only in the action log.
        author: String::new(),
        assignees,
        labels: row
            .labels
            .into_iter()
            .map(|l| {
                if l.name.is_empty() {
                    l.color.unwrap_or_default()
                } else {
                    l.name
                }
            })
            .filter(|n| !n.is_empty())
            .collect(),
        web_url,
        updated_at: row.date_last_activity.unwrap_or_default(),
        milestone: None,
    }
}

/// The create-form choices: the open lists, the first one the default.
pub(super) fn map_options(lists: Vec<TrelloList>) -> IssueCreateOptions {
    let lists: Vec<Choice> = lists
        .into_iter()
        .map(|l| Choice {
            id: l.id,
            label: l.name,
        })
        .collect();
    IssueCreateOptions {
        default_list: lists.first().map(|l| l.id.clone()),
        types: Vec::new(),
        lists,
        default_type: None,
    }
}
