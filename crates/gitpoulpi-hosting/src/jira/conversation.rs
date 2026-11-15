//! The conversation of a Jira issue (Cloud and Data Center): the comment
//! list, the changelog as events, and posting, editing and deleting.
//!
//! Everything goes over API v2 on both editions, where a comment body is
//! wiki markup (v3 wants an Atlassian Document Format tree). The user writes
//! Markdown: it is converted to wiki markup on the way out
//! ([`markup::markdown_to_jira`]) and the wiki markup read back is converted
//! to Markdown ([`markup::jira_to_markdown`]). Jira has no replies and no
//! way to post an internal comment without naming a project role, so those
//! are unsupported.
//!
//! The comment list is read newest first (`orderBy=-created`), the newest
//! window of [`WINDOW`] entries from `startAt` 0; the changelog comes with
//! the issue (`expand=changelog`, whatever Jira keeps in it) and is cut to
//! the time span of the window.

use serde_json::{json, Value};

use super::client::JiraClient;
use super::map::{browse_url, number_of};
use crate::client::encode_component;
use crate::conversation::time::is_edited;
use crate::conversation::tracker::{
    check_body, decode_cursor, encode_cursor, item_ref, oldest_secs, opt_text, require_issue,
    span_of, text,
};
use crate::conversation::{
    Actor, ActorKind, Comment, Event, EventKind, IdentityKind, ItemHeader, ItemKey, ItemKind,
    Markup, MarkupFlavor, PageCursor, PostingIdentity, TimelineEntry, TimelineOpts, TimelinePage,
    TokenSource, WINDOW,
};
use crate::http::HttpRequest;
use crate::markup::{jira_to_markdown, markdown_to_jira};
use crate::review::invalid;
use crate::{Error, Result};

const PAGE: usize = 100;
const MAX_BODY: usize = 32_767;
const ISSUE_FIELDS: &str = "summary,status,reporter,creator,labels,description,created";

/// The actor of a Jira user object. Cloud identifies by `accountId` (an
/// `app` account is a bot), Data Center by `name`.
fn actor_of(site: &str, user: Option<&Value>) -> Actor {
    let Some(user) = user.filter(|u| u.is_object()) else {
        return Actor::unknown();
    };
    let account = text(user, "accountId");
    let login = if account.is_empty() {
        opt_text(user, "name")
            .or_else(|| opt_text(user, "key"))
            .unwrap_or_default()
    } else {
        account.clone()
    };
    if login.is_empty() {
        return Actor::unknown();
    }
    let display = opt_text(user, "displayName").unwrap_or_else(|| login.clone());
    let kind = if text(user, "accountType") == "app" {
        ActorKind::Bot
    } else {
        ActorKind::User
    };
    let site = site.trim_end_matches('/');
    let mut actor = Actor::new(login.clone(), display, kind);
    actor.web_url = Some(if account.is_empty() {
        format!(
            "{site}/secure/ViewProfile.jspa?name={}",
            encode_component(&login)
        )
    } else {
        format!("{site}/jira/people/{account}")
    });
    actor
}

fn markdown_of(wiki: &str) -> Markup {
    Markup::new(jira_to_markdown(wiki), MarkupFlavor::Gfm)
}

fn comment_of(site: &str, issue_key: &str, c: &Value) -> Comment {
    let created_at = text(c, "created");
    let updated_at = opt_text(c, "updated");
    let id = match &c["id"] {
        Value::String(s) => s.clone(),
        other => other.to_string().trim_matches('"').to_string(),
    };
    Comment {
        web_url: Some(format!(
            "{}?focusedCommentId={id}",
            browse_url(site, issue_key)
        )),
        id,
        author: actor_of(site, c.get("author")),
        body: markdown_of(&text(c, "body")),
        edited: is_edited(&created_at, updated_at.as_deref()),
        created_at,
        updated_at,
        reactions: Vec::new(),
        in_reply_to: None,
        discussion_id: None,
        internal: c.get("visibility").is_some_and(|v| v.is_object()),
        can_edit: None,
        can_delete: None,
    }
}

/// Labels a changelog item adds and removes (`"a b"` to `"a c"`).
fn label_diff(from: &str, to: &str) -> (Vec<String>, Vec<String>) {
    let old: Vec<&str> = from.split_whitespace().collect();
    let new: Vec<&str> = to.split_whitespace().collect();
    let added = new
        .iter()
        .filter(|l| !old.contains(l))
        .map(|l| l.to_string())
        .collect();
    let removed = old
        .iter()
        .filter(|l| !new.contains(l))
        .map(|l| l.to_string())
        .collect();
    (added, removed)
}

fn event_kind(item: &Value) -> EventKind {
    let field = text(item, "field");
    let from = text(item, "fromString");
    let to = text(item, "toString");
    let one = |s: &str| {
        if s.is_empty() {
            Vec::new()
        } else {
            vec![s.to_string()]
        }
    };
    match field.to_ascii_lowercase().as_str() {
        "status" => EventKind::StatusChanged { from, to },
        "summary" => EventKind::Renamed { from, to },
        "assignee" => EventKind::Assigned {
            added: one(&to),
            removed: one(&from),
        },
        "labels" => {
            let (added, removed) = label_diff(&from, &to);
            EventKind::Labeled { added, removed }
        }
        _ => EventKind::Other {
            text: match (from.is_empty(), to.is_empty()) {
                (_, false) if from.is_empty() => format!("set {field} to {to}"),
                (false, true) => format!("cleared {field}"),
                _ => format!("changed {field} from {from} to {to}"),
            },
        },
    }
}

impl JiraClient {
    fn issue_url(&self, issue_key: &str) -> String {
        format!("{}/issue/{}", self.api_v2(), encode_component(issue_key))
    }

    fn comment_body(&self, body: &str) -> Value {
        json!({ "body": markdown_to_jira(body) })
    }

    pub(super) async fn conv_timeline(
        &self,
        project: &str,
        key: &ItemKey,
        older: Option<&PageCursor>,
        opts: &TimelineOpts,
    ) -> Result<TimelinePage> {
        require_issue("Jira", key)?;
        let (start_at, upper) = match older {
            None => (0usize, None),
            Some(c) => {
                let (p, u) = decode_cursor(c)
                    .ok_or_else(|| invalid("reading a conversation", "bad page cursor"))?;
                let n = p
                    .parse::<usize>()
                    .map_err(|_| invalid("reading a conversation", "bad page cursor"))?;
                (n, u)
            }
        };
        let issue_key = item_ref(project, key);
        let site = self.site();
        let mut url = format!("{}?fields={ISSUE_FIELDS}", self.issue_url(&issue_key));
        if opts.events {
            url.push_str("&expand=changelog");
        }
        let issue: Value = self.get_json(url, "reading the Jira issue").await?;

        let mut comments: Vec<Comment> = Vec::new();
        let mut next = start_at;
        let mut total = 0usize;
        for _ in 0..(WINDOW / PAGE) {
            let url = format!(
                "{}/comment?startAt={next}&maxResults={PAGE}&orderBy=-created",
                self.issue_url(&issue_key)
            );
            let page: Value = self.get_json(url, "reading the Jira comments").await?;
            total = page["total"].as_u64().unwrap_or(0) as usize;
            let rows = page["comments"].as_array().cloned().unwrap_or_default();
            next += rows.len();
            comments.extend(rows.iter().map(|c| comment_of(&site, &issue_key, c)));
            if rows.is_empty() || next >= total {
                break;
            }
        }
        comments.reverse();
        let has_older = next < total;
        let span = span_of(
            comments.iter().map(|c| c.created_at.as_str()),
            has_older,
            upper,
        );
        let older_cursor = has_older.then(|| {
            encode_cursor(
                &next.to_string(),
                oldest_secs(comments.iter().map(|c| c.created_at.as_str())),
            )
        });

        let mut entries: Vec<TimelineEntry> =
            comments.into_iter().map(TimelineEntry::Comment).collect();
        if opts.events {
            let histories = issue["changelog"]["histories"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            for h in histories {
                let at = text(&h, "created");
                if !span.holds(&at) {
                    continue;
                }
                let actor = actor_of(&site, h.get("author"));
                let hid = match &h["id"] {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                for (i, item) in h["items"].as_array().into_iter().flatten().enumerate() {
                    entries.push(TimelineEntry::Event(Event {
                        id: format!("{hid}.{i}"),
                        actor: Some(actor.clone()),
                        at: at.clone(),
                        kind: event_kind(item),
                    }));
                }
            }
            // Stable: a comment and an event of the same second keep the
            // order they were read in.
            entries.sort_by_key(|e| crate::conversation::time::iso_secs(e.at()).unwrap_or(0));
        }

        let fields = &issue["fields"];
        let done = fields["status"]["statusCategory"]["key"].as_str() == Some("done");
        let author = fields
            .get("reporter")
            .filter(|r| r.is_object())
            .or_else(|| fields.get("creator"));
        let header = ItemHeader {
            key: ItemKey {
                kind: ItemKind::Issue,
                number: number_of(&issue_key),
                id: issue_key.clone(),
            },
            reference: issue_key.clone(),
            title: text(fields, "summary"),
            state_label: text(&fields["status"], "name"),
            open: !done,
            author: actor_of(&site, author),
            body: markdown_of(&text(fields, "description")),
            created_at: text(fields, "created"),
            web_url: browse_url(&site, &issue_key),
            labels: fields["labels"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|l| l.as_str().map(str::to_string))
                .collect(),
        };
        Ok(TimelinePage {
            header,
            entries,
            older: older_cursor,
            total_hint: Some(total as u32),
        })
    }

    pub(super) async fn conv_post(
        &self,
        project: &str,
        key: &ItemKey,
        body: &str,
        internal: bool,
    ) -> Result<Comment> {
        require_issue("Jira", key)?;
        check_body("commenting on a Jira issue", body, Some(MAX_BODY))?;
        if internal {
            return Err(Error::Unsupported(
                "internal comments: Jira needs a project role to restrict a comment to".into(),
            ));
        }
        let issue_key = item_ref(project, key);
        let url = format!("{}/comment", self.issue_url(&issue_key));
        let request = self.authed(HttpRequest::post_json(url, &self.comment_body(body)));
        let response = self.transport.send(request).await?;
        response.error_for_status("commenting on a Jira issue")?;
        Ok(comment_of(
            &self.site(),
            &issue_key,
            &response.json::<Value>()?,
        ))
    }

    pub(super) async fn conv_edit(
        &self,
        project: &str,
        key: &ItemKey,
        comment_id: &str,
        body: &str,
    ) -> Result<Comment> {
        require_issue("Jira", key)?;
        check_body("editing a Jira comment", body, Some(MAX_BODY))?;
        let issue_key = item_ref(project, key);
        let url = format!(
            "{}/comment/{}",
            self.issue_url(&issue_key),
            encode_component(comment_id)
        );
        let request = self.authed(HttpRequest::put_json(url, &self.comment_body(body)));
        let response = self.transport.send(request).await?;
        response.error_for_status("editing a Jira comment")?;
        Ok(comment_of(
            &self.site(),
            &issue_key,
            &response.json::<Value>()?,
        ))
    }

    pub(super) async fn conv_delete(
        &self,
        project: &str,
        key: &ItemKey,
        comment_id: &str,
    ) -> Result<()> {
        require_issue("Jira", key)?;
        let url = format!(
            "{}/comment/{}",
            self.issue_url(&item_ref(project, key)),
            encode_component(comment_id)
        );
        let response = self
            .transport
            .send(self.authed(HttpRequest::delete(url)))
            .await?;
        response.error_for_status("deleting a Jira comment")
    }

    /// The token's own user (`GET /myself`). Cloud identifies by account id,
    /// shown by display name; Data Center by user name.
    pub(super) async fn conv_identity(&self) -> Result<PostingIdentity> {
        let me: Value = self
            .get_json(format!("{}/myself", self.api()), "checking the Jira token")
            .await?;
        let actor = actor_of(&self.site(), Some(&me));
        if actor.kind == ActorKind::Unknown {
            return Err(Error::Decode("no user in the Jira answer".to_string()));
        }
        let mut identity = PostingIdentity::new(
            Some(actor.login),
            IdentityKind::PersonalToken,
            TokenSource::Profile {
                id: self.integration.id.clone(),
            },
        );
        identity.display = actor.display;
        Ok(identity)
    }
}
