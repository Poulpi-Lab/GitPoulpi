//! The conversation of a Redmine issue. A *journal* is a comment (its
//! `notes`), a set of changes (its `details`), or both: the notes become a
//! comment and each change an event.
//!
//! Text is Textile or Markdown as the server is configured (the profile's
//! `markup`): Textile is read through [`textile_to_markdown`] and written
//! with [`markdown_to_textile`], so the user always works in Markdown.
//! Redmine has no replies and cannot delete a journal's notes, so those are
//! unsupported; an edit is `PUT /journals/{id}.json` (Redmine 5.0 and later;
//! an older server answers with an error that is shown as is).
//!
//! The whole issue with its journals is one answer, so the newest window is
//! cut from it locally.

use serde_json::{json, Value};

use super::client::RedmineClient;
use super::map::issue_url;
use super::textile_out::markdown_to_textile;
use crate::conversation::time::is_edited;
use crate::conversation::tracker::{check_body, opt_text, require_issue, text, window_of};
use crate::conversation::{
    Actor, ActorKind, Comment, Event, EventKind, IdentityKind, ItemHeader, ItemKey, ItemKind,
    Markup, MarkupFlavor, PageCursor, PostingIdentity, TimelineEntry, TimelineOpts, TimelinePage,
    TokenSource,
};
use crate::http::HttpRequest;
use crate::markup::{sanitize, textile_to_markdown};
use crate::{Error, Result};
use gitpoulpi_config::RedmineMarkup;

/// The user Redmine names in an issue or a journal: `{id, name}`. The
/// numeric id is the login (a journal carries no login name).
fn actor_of(site: &str, user: &Value) -> Actor {
    let Some(id) = user["id"].as_u64() else {
        return Actor::unknown();
    };
    let display = opt_text(user, "name").unwrap_or_else(|| id.to_string());
    let mut actor = Actor::new(id.to_string(), display, ActorKind::User);
    actor.web_url = Some(format!("{}/users/{id}", site.trim_end_matches('/')));
    actor
}

/// A status id as a name, when the statuses are known.
fn status_name(statuses: &[(u64, String, bool)], id: &str) -> String {
    id.parse::<u64>()
        .ok()
        .and_then(|n| statuses.iter().find(|(i, _, _)| *i == n))
        .map(|(_, name, _)| name.clone())
        .unwrap_or_else(|| format!("#{id}"))
}

fn is_closed(statuses: &[(u64, String, bool)], id: &str) -> Option<bool> {
    let n = id.parse::<u64>().ok()?;
    statuses
        .iter()
        .find(|(i, _, _)| *i == n)
        .map(|(_, _, c)| *c)
}

fn detail_kind(detail: &Value, statuses: &[(u64, String, bool)]) -> EventKind {
    let property = text(detail, "property");
    let name = text(detail, "name");
    let old = text(detail, "old_value");
    let new = text(detail, "new_value");
    match (property.as_str(), name.as_str()) {
        ("attr", "status_id") => match (is_closed(statuses, &old), is_closed(statuses, &new)) {
            (Some(false), Some(true)) => EventKind::Closed,
            (Some(true), Some(false)) => EventKind::Reopened,
            _ => EventKind::StatusChanged {
                from: status_name(statuses, &old),
                to: status_name(statuses, &new),
            },
        },
        ("attr", "subject") => EventKind::Renamed { from: old, to: new },
        // The journal gives user ids, not names.
        ("attr", "assigned_to_id") => EventKind::Other {
            text: if new.is_empty() {
                "removed the assignee".into()
            } else {
                "changed the assignee".into()
            },
        },
        ("attr", "description") => EventKind::Other {
            text: "edited the description".into(),
        },
        ("attachment", _) => EventKind::Other {
            text: format!("added the attachment {new}"),
        },
        ("relation", _) => EventKind::Other {
            text: "changed a relation".into(),
        },
        (_, field) => {
            let field = field.trim_end_matches("_id").replace('_', " ");
            EventKind::Other {
                text: match (old.is_empty(), new.is_empty()) {
                    (true, false) => format!("set {field} to {new}"),
                    (false, true) => format!("cleared {field}"),
                    _ => format!("changed {field} from {old} to {new}"),
                },
            }
        }
    }
}

impl RedmineClient {
    fn textile(&self) -> bool {
        self.integration.markup == RedmineMarkup::Textile
    }

    /// What the server stored, as Markdown for display.
    fn read_markup(&self, raw: &str) -> Markup {
        let text = if self.textile() {
            textile_to_markdown(raw)
        } else {
            sanitize(raw)
        };
        Markup::new(text, MarkupFlavor::Gfm)
    }

    /// What the user wrote (Markdown), in the server's format.
    fn write_markup(&self, markdown: &str) -> String {
        if self.textile() {
            markdown_to_textile(markdown)
        } else {
            markdown.to_string()
        }
    }

    /// `index`: the journal's 1-based position, what `#note-N` counts.
    fn journal_comment(&self, site: &str, number: u64, index: usize, j: &Value) -> Comment {
        let id = j["id"].as_u64().unwrap_or_default();
        let created_at = text(j, "created_on");
        let updated_at = opt_text(j, "updated_on");
        Comment {
            id: id.to_string(),
            author: actor_of(site, &j["user"]),
            body: self.read_markup(&text(j, "notes")),
            edited: is_edited(&created_at, updated_at.as_deref()),
            created_at,
            updated_at,
            reactions: Vec::new(),
            in_reply_to: None,
            discussion_id: None,
            internal: j["private_notes"].as_bool() == Some(true),
            can_edit: None,
            can_delete: None,
            web_url: Some(format!("{}#note-{index}", issue_url(site, number))),
        }
    }

    async fn read_issue(&self, number: u64) -> Result<Value> {
        let found: Value = self
            .get_json(
                &format!("/issues/{number}.json?include=journals"),
                "reading the Redmine issue",
            )
            .await?;
        if found["issue"].is_object() {
            Ok(found["issue"].clone())
        } else {
            Err(Error::Decode("no issue in the Redmine answer".to_string()))
        }
    }

    /// `(id, name, is_closed)` of every status; empty when they cannot be
    /// read (the events then show ids).
    async fn status_table(&self) -> Vec<(u64, String, bool)> {
        self.get_json::<Value>("/issue_statuses.json", "listing the Redmine statuses")
            .await
            .ok()
            .and_then(|v| v["issue_statuses"].as_array().cloned())
            .unwrap_or_default()
            .iter()
            .filter_map(|s| {
                Some((
                    s["id"].as_u64()?,
                    text(s, "name"),
                    s["is_closed"].as_bool() == Some(true),
                ))
            })
            .collect()
    }

    pub(super) async fn conv_timeline(
        &self,
        key: &ItemKey,
        older: Option<&PageCursor>,
        opts: &TimelineOpts,
    ) -> Result<TimelinePage> {
        require_issue("Redmine", key)?;
        let issue = self.read_issue(key.number).await?;
        let site = self.site();
        let journals: Vec<Value> = issue["journals"].as_array().cloned().unwrap_or_default();
        let wants_statuses = opts.events
            && journals.iter().any(|j| {
                j["details"]
                    .as_array()
                    .is_some_and(|d| d.iter().any(|d| text(d, "name") == "status_id"))
            });
        let statuses = if wants_statuses {
            self.status_table().await
        } else {
            Vec::new()
        };

        let mut entries: Vec<TimelineEntry> = Vec::new();
        for (n, j) in journals.iter().enumerate() {
            if !text(j, "notes").trim().is_empty() {
                entries.push(TimelineEntry::Comment(self.journal_comment(
                    &site,
                    key.number,
                    n + 1,
                    j,
                )));
            }
            if opts.events {
                let id = j["id"].as_u64().unwrap_or_default();
                let actor = actor_of(&site, &j["user"]);
                for (i, d) in j["details"].as_array().into_iter().flatten().enumerate() {
                    entries.push(TimelineEntry::Event(Event {
                        id: format!("{id}.{i}"),
                        actor: Some(actor.clone()),
                        at: text(j, "created_on"),
                        kind: detail_kind(d, &statuses),
                    }));
                }
            }
        }
        let total = entries.len() as u32;
        let (window, older_cursor) = window_of(entries, older)?;

        let closed = issue["status"]["is_closed"].as_bool() == Some(true);
        let header = ItemHeader {
            key: ItemKey {
                kind: ItemKind::Issue,
                number: key.number,
                id: String::new(),
            },
            reference: format!("#{}", key.number),
            title: text(&issue, "subject"),
            state_label: text(&issue["status"], "name"),
            open: !closed,
            author: actor_of(&site, &issue["author"]),
            body: self.read_markup(&text(&issue, "description")),
            created_at: text(&issue, "created_on"),
            web_url: issue_url(&site, key.number),
            labels: Vec::new(),
        };
        Ok(TimelinePage {
            header,
            entries: window,
            older: older_cursor,
            total_hint: Some(total),
        })
    }

    /// The comment now standing for `journal_id`, or the newest one whose
    /// notes are `notes` when the id is not known yet (a fresh post).
    async fn find_comment(
        &self,
        number: u64,
        journal_id: Option<u64>,
        notes: &str,
    ) -> Result<Comment> {
        let issue = self.read_issue(number).await?;
        let site = self.site();
        let journals = issue["journals"].as_array().cloned().unwrap_or_default();
        let hit = journals
            .iter()
            .enumerate()
            .rev()
            .find(|(_, j)| match journal_id {
                Some(id) => j["id"].as_u64() == Some(id),
                None => text(j, "notes").trim() == notes.trim(),
            });
        hit.map(|(n, j)| self.journal_comment(&site, number, n + 1, j))
            .ok_or_else(|| Error::Decode("the Redmine comment is not in the issue".to_string()))
    }

    pub(super) async fn conv_post(
        &self,
        key: &ItemKey,
        body: &str,
        internal: bool,
    ) -> Result<Comment> {
        require_issue("Redmine", key)?;
        check_body("commenting on a Redmine issue", body, None)?;
        let notes = self.write_markup(body);
        let mut issue = json!({ "notes": notes });
        if internal {
            issue["private_notes"] = json!(true);
        }
        // 204 and no body: the comment is read back.
        self.edit("commenting on a Redmine issue", key.number, issue)
            .await?;
        self.find_comment(key.number, None, &notes).await
    }

    pub(super) async fn conv_edit(
        &self,
        key: &ItemKey,
        comment_id: &str,
        body: &str,
    ) -> Result<Comment> {
        require_issue("Redmine", key)?;
        check_body("editing a Redmine comment", body, None)?;
        let id: u64 = comment_id.trim().parse().map_err(|_| {
            crate::conversation::tracker::invalid(
                "editing a Redmine comment",
                "the comment id is not a number",
            )
        })?;
        let url = format!("{}/journals/{id}.json", self.api());
        let payload = json!({ "journal": { "notes": self.write_markup(body) } });
        self.send(
            HttpRequest::put_json(url, &payload),
            "editing a Redmine comment (needs Redmine 5.0 or later)",
        )
        .await?;
        self.find_comment(key.number, Some(id), "").await
    }

    /// The API key's own user: the numeric id is the login (see
    /// [`actor_of`]), the account's name the display.
    pub(super) async fn conv_identity(&self) -> Result<PostingIdentity> {
        let me: Value = self
            .get_json("/users/current.json", "checking the Redmine API key")
            .await?;
        let user = &me["user"];
        let id = user["id"]
            .as_u64()
            .ok_or_else(|| Error::Decode("no user in the Redmine answer".to_string()))?;
        let name = format!("{} {}", text(user, "firstname"), text(user, "lastname"));
        let mut identity = PostingIdentity::new(
            Some(id.to_string()),
            IdentityKind::PersonalToken,
            TokenSource::Profile {
                id: self.integration.id.clone(),
            },
        );
        identity.display = [name.trim().to_string(), text(user, "login")]
            .into_iter()
            .find(|s| !s.is_empty())
            .unwrap_or_else(|| id.to_string());
        Ok(identity)
    }
}
