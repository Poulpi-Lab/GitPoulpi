//! The conversation of a Bitbucket Cloud pull request: the header, its
//! comments (an `inline` comment is a code thread, `parent` makes a reply,
//! a `deleted` one is hidden), the activity as events and approvals, and
//! posting, replying, editing and deleting comments.
//!
//! Comments are read newest first (`sort=-created_on`) so the newest window
//! is the first pages; the `older` cursor is the `next` URL the host gave,
//! kept only when it points at the API's own host. Activity is not windowed:
//! its first page belongs to the newest window, and it is left out silently
//! when it cannot be read (it only adds events and approvals).
//!
//! Issues were retired on Cloud: every issue call is `Unsupported`.

use std::collections::HashMap;

use serde_json::{json, Value};

use super::BitbucketClient;
use crate::client::encode_component;
use crate::conversation::compat::{check_body, sort_by_time, token_source};
use crate::conversation::time::is_edited;
use crate::conversation::{
    Actor, ActorKind, CodeThread, Comment, Event, EventKind, IdentityKind, ItemHeader, ItemKey,
    ItemKind, Markup, MarkupFlavor, PageCursor, PostingIdentity, ReviewEntry, TimelineEntry,
    TimelineOpts, TimelinePage, WINDOW,
};
use crate::http::HttpRequest;
use crate::review::invalid;
use crate::review::model::{DiffSide, ReviewState};
use crate::{Error, Result};

const FLAVOR: MarkupFlavor = MarkupFlavor::BitbucketMd;
const PAGELEN: usize = 100;
const PAGES_PER_WINDOW: usize = WINDOW / PAGELEN;

fn text(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn opt_text(v: &Value, key: &str) -> Option<String> {
    v.get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// The name a comment is attributed to: the nickname, else the account id.
fn login_of(user: &Value) -> String {
    opt_text(user, "nickname")
        .or_else(|| opt_text(user, "account_id"))
        .or_else(|| opt_text(user, "display_name"))
        .unwrap_or_default()
}

fn actor_of(user: Option<&Value>) -> Actor {
    let Some(user) = user.filter(|u| u.is_object()) else {
        return Actor::unknown();
    };
    let login = login_of(user);
    if login.is_empty() {
        return Actor::unknown();
    }
    let display = opt_text(user, "display_name").unwrap_or_else(|| login.clone());
    let mut actor = Actor::new(login, display, ActorKind::User);
    actor.web_url = user["links"]["html"]["href"]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    actor
}

/// One comment of the host. `discussion` is the id a reply goes under.
fn comment_of(c: &Value, discussion: Option<String>) -> Comment {
    let created_at = text(c, "created_on");
    let updated_at = opt_text(c, "updated_on");
    let parent = c["parent"]["id"].as_u64().map(|p| p.to_string());
    let id = c["id"].as_u64().unwrap_or_default().to_string();
    Comment {
        discussion_id: Some(discussion.or_else(|| parent.clone()).unwrap_or_else(|| id.clone())),
        in_reply_to: parent,
        id,
        author: actor_of(c.get("user")),
        body: Markup::new(text(&c["content"], "raw"), FLAVOR),
        edited: is_edited(&created_at, updated_at.as_deref()),
        created_at,
        updated_at,
        reactions: Vec::new(),
        internal: false,
        can_edit: None,
        can_delete: None,
        web_url: c["links"]["html"]["href"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(str::to_string),
    }
}

/// What a code comment sits on: `to` is the line of the new file, `from`
/// the one of the old file.
fn anchor_of(inline: &Value) -> (String, Option<u32>, DiffSide, Option<u32>) {
    let num = |k: &str| inline[k].as_u64().map(|n| n as u32);
    let (line, side, start) = match (num("to"), num("from")) {
        (Some(to), _) => (Some(to), DiffSide::New, num("start_to")),
        (None, from) => (from, DiffSide::Old, num("start_from")),
    };
    (
        text(inline, "path"),
        line,
        side,
        start.filter(|s| Some(*s) != line),
    )
}

fn state_of(row: &Value) -> (&'static str, bool) {
    match row["state"].as_str().unwrap_or_default() {
        "OPEN" => ("open", true),
        "MERGED" => ("merged", false),
        "DECLINED" => ("declined", false),
        "SUPERSEDED" => ("superseded", false),
        _ => ("closed", false),
    }
}

/// The activity entry as an event or an approval (comments are read from
/// their own list).
fn activity_entry(a: &Value) -> Option<TimelineEntry> {
    if let Some(u) = a.get("update").filter(|u| u.is_object()) {
        let at = text(u, "date");
        let state = u["state"].as_str().unwrap_or_default();
        let kind = if u["changes"]["title"].is_object() {
            EventKind::Renamed {
                from: text(&u["changes"]["title"], "old"),
                to: text(&u["changes"]["title"], "new"),
            }
        } else {
            match state {
                "MERGED" => EventKind::Merged,
                "DECLINED" | "SUPERSEDED" => EventKind::Closed,
                _ if u["source"]["commit"]["hash"].is_string() => EventKind::Other {
                    text: "updated the source branch".to_string(),
                },
                _ => return None,
            }
        };
        return Some(TimelineEntry::Event(Event {
            id: format!("update-{at}"),
            actor: Some(actor_of(u.get("author"))),
            at,
            kind,
        }));
    }
    if let Some(ap) = a.get("approval").filter(|a| a.is_object()) {
        let at = text(ap, "date");
        return Some(TimelineEntry::Review(ReviewEntry {
            id: format!("approval-{at}"),
            author: actor_of(ap.get("user")),
            state: ReviewState::Approved,
            body: Markup::new("", FLAVOR),
            submitted_at: at,
        }));
    }
    None
}

impl BitbucketClient {
    fn conv_url(&self, project: &str, key: &ItemKey, rest: &str) -> Result<String> {
        if key.kind != ItemKind::PullRequest {
            return Err(Error::Unsupported(super::ISSUES_RETIRED.to_string()));
        }
        Ok(format!("{}/{}{rest}", self.pr_url(project), key.number))
    }

    /// A cursor is a `next` URL the host gave: only the API's own host.
    fn conv_cursor(&self, cursor: &PageCursor) -> Result<String> {
        if cursor.0.starts_with(&self.origin()) {
            Ok(cursor.0.clone())
        } else {
            Err(invalid("reading a conversation", "bad page cursor"))
        }
    }

    async fn conv_get(&self, url: String, what: &str) -> Result<Value> {
        let response = self.transport.send(self.get(url)).await?;
        response.error_for_status(what)?;
        response.json()
    }

    pub(crate) async fn conv_timeline(
        &self,
        project: &str,
        key: &ItemKey,
        older: Option<&PageCursor>,
        opts: &TimelineOpts,
    ) -> Result<TimelinePage> {
        let row = self
            .conv_get(
                self.conv_url(project, key, "")?,
                "reading a Bitbucket pull request",
            )
            .await?;
        let (state_label, open) = state_of(&row);
        let header = ItemHeader {
            key: key.clone(),
            reference: format!("#{}", key.number),
            title: text(&row, "title"),
            state_label: state_label.to_string(),
            open,
            author: actor_of(row.get("author")),
            body: Markup::new(text(&row, "description"), FLAVOR),
            created_at: text(&row, "created_on"),
            web_url: row["links"]["html"]["href"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            labels: Vec::new(),
        };

        let mut url = match older {
            Some(c) => self.conv_cursor(c)?,
            None => self.conv_url(
                project,
                key,
                &format!("/comments?pagelen={PAGELEN}&sort=-created_on"),
            )?,
        };
        let mut raw: Vec<Value> = Vec::new();
        let mut next: Option<String> = None;
        let mut hint: Option<u32> = None;
        for page in 0..PAGES_PER_WINDOW {
            let body = self
                .conv_get(url.clone(), "reading the comments of a Bitbucket pull request")
                .await?;
            if page == 0 && older.is_none() {
                hint = body["size"].as_u64().map(|s| s as u32);
            }
            raw.extend(body["values"].as_array().cloned().unwrap_or_default());
            next = opt_text(&body, "next");
            match &next {
                Some(n) if page + 1 < PAGES_PER_WINDOW => url = self.conv_cursor(&PageCursor(n.clone()))?,
                _ => break,
            }
        }
        let older_cursor = match next {
            Some(n) => {
                self.conv_cursor(&PageCursor(n.clone()))?;
                Some(PageCursor(n))
            }
            None => None,
        };

        let mut entries = comment_entries(raw);
        if older.is_none() && opts.events {
            let url = self.conv_url(project, key, "/activity?pagelen=50")?;
            if let Ok(activity) = self.conv_get(url, "reading the Bitbucket activity").await {
                entries.extend(
                    activity["values"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(activity_entry),
                );
            }
        }
        sort_by_time(&mut entries);
        Ok(TimelinePage {
            header,
            entries,
            older: older_cursor,
            total_hint: hint,
        })
    }

    pub(crate) async fn conv_post(
        &self,
        project: &str,
        key: &ItemKey,
        body: &str,
        internal: bool,
    ) -> Result<Comment> {
        let url = self.conv_url(project, key, "/comments")?;
        if internal {
            return Err(invalid(
                "commenting on a pull request",
                "Bitbucket has no internal comments",
            ));
        }
        check_body("commenting on a pull request", body)?;
        self.conv_send_comment(url, json!({ "content": { "raw": body } }), None)
            .await
    }

    pub(crate) async fn conv_reply(
        &self,
        project: &str,
        key: &ItemKey,
        discussion_id: &str,
        body: &str,
    ) -> Result<Comment> {
        let url = self.conv_url(project, key, "/comments")?;
        check_body("replying to a comment", body)?;
        let parent: u64 = discussion_id
            .parse()
            .map_err(|_| invalid("replying to a comment", "not a comment of this host"))?;
        self.conv_send_comment(
            url,
            json!({ "content": { "raw": body }, "parent": { "id": parent } }),
            Some(discussion_id.to_string()),
        )
        .await
    }

    async fn conv_send_comment(
        &self,
        url: String,
        payload: Value,
        discussion: Option<String>,
    ) -> Result<Comment> {
        let response = self
            .transport
            .send(self.authed(HttpRequest::post_json(url, &payload)))
            .await?;
        response.error_for_status("commenting on a Bitbucket pull request")?;
        Ok(comment_of(&response.json::<Value>()?, discussion))
    }

    pub(crate) async fn conv_edit(
        &self,
        project: &str,
        key: &ItemKey,
        comment_id: &str,
        body: &str,
    ) -> Result<Comment> {
        let url = self.conv_url(
            project,
            key,
            &format!("/comments/{}", encode_component(comment_id)),
        )?;
        check_body("editing a comment", body)?;
        let response = self
            .transport
            .send(self.authed(HttpRequest::put_json(
                url,
                &json!({ "content": { "raw": body } }),
            )))
            .await?;
        response.error_for_status("editing a Bitbucket comment")?;
        Ok(comment_of(&response.json::<Value>()?, None))
    }

    pub(crate) async fn conv_delete(
        &self,
        project: &str,
        key: &ItemKey,
        comment_id: &str,
    ) -> Result<()> {
        let url = self.conv_url(
            project,
            key,
            &format!("/comments/{}", encode_component(comment_id)),
        )?;
        let response = self.transport.send(self.authed(HttpRequest::delete(url))).await?;
        response.error_for_status("deleting a Bitbucket comment")
    }

    /// The token's own account (`GET /user`).
    pub(crate) async fn conv_identity(&self, project: &str) -> Result<PostingIdentity> {
        let (source, _) = token_source(&self.scopes, &self.integration.id, project);
        let account = self
            .conv_get(
                format!("{}/user", self.api()),
                "reading the Bitbucket account",
            )
            .await?;
        let login = login_of(&account);
        if login.is_empty() {
            return Err(Error::Decode("no nickname in the Bitbucket user".to_string()));
        }
        Ok(PostingIdentity::new(
            Some(login),
            IdentityKind::PersonalToken,
            source,
        ))
    }
}

/// The window's comments as entries: a thread on code becomes a
/// [`CodeThread`], any other comment (a reply keeps its `in_reply_to`) a
/// plain comment. Deleted comments are dropped.
fn comment_entries(raw: Vec<Value>) -> Vec<TimelineEntry> {
    let rows: Vec<&Value> = raw
        .iter()
        .filter(|c| c["deleted"].as_bool() != Some(true))
        .collect();
    let parents: HashMap<u64, Option<u64>> = rows
        .iter()
        .filter_map(|c| Some((c["id"].as_u64()?, c["parent"]["id"].as_u64())))
        .collect();
    // The topmost ancestor present in the window, else the parent itself.
    let root_of = |id: u64| -> u64 {
        let mut cur = id;
        for _ in 0..64 {
            match parents.get(&cur).copied().flatten() {
                Some(p) if parents.contains_key(&p) => cur = p,
                Some(p) => return p,
                None => return cur,
            }
        }
        cur
    };
    let mut groups: Vec<(u64, Vec<&Value>)> = Vec::new();
    for c in rows {
        let id = c["id"].as_u64().unwrap_or_default();
        let root = root_of(id);
        match groups.iter_mut().find(|(r, _)| *r == root) {
            Some((_, v)) => v.push(c),
            None => groups.push((root, vec![c])),
        }
    }
    let mut out = Vec::new();
    for (root, mut members) in groups {
        members.sort_by_key(|c| c["id"].as_u64().unwrap_or_default());
        let anchor = members.iter().find(|c| c["inline"].is_object());
        let comments: Vec<Comment> = members
            .iter()
            .map(|c| comment_of(c, Some(root.to_string())))
            .collect();
        match anchor {
            Some(a) => {
                let inline = &a["inline"];
                let (path, line, side, start_line) = anchor_of(inline);
                let head = members[0];
                out.push(TimelineEntry::Thread(CodeThread {
                    id: root.to_string(),
                    path,
                    line,
                    original_line: line,
                    side,
                    start_line,
                    resolved: head["resolution"].is_object(),
                    outdated: inline["outdated"].as_bool() == Some(true),
                    resolvable: true,
                    diff_hunk: None,
                    comments,
                }));
            }
            None => out.extend(comments.into_iter().map(TimelineEntry::Comment)),
        }
    }
    out
}
