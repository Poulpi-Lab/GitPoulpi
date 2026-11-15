//! The conversation of a Bitbucket Data Center pull request: the header and
//! the activity list (comments with their nested replies, approvals, the
//! lifecycle, pushed commits), and posting, replying, editing and deleting
//! comments.
//!
//! Activities are read most recent first (the order the host answers in), so
//! the newest window is the first pages and the `older` cursor is the next
//! `start`. A comment's `version` is required to edit or delete it, so those
//! two read the comment first. Verify the activity order against a real
//! server (hand check in `docs/developer/where-we-are.md`).
//!
//! There is no issue tracker: every issue call is `Unsupported`.

use serde_json::{json, Value};

use super::{epoch_ms_to_iso, BitbucketDcClient, NO_ISSUES};
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
const LIMIT: usize = 100;
const PAGES_PER_WINDOW: usize = WINDOW / LIMIT;

fn text(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn actor_of(user: Option<&Value>) -> Actor {
    let Some(user) = user.filter(|u| u.is_object()) else {
        return Actor::unknown();
    };
    let login = text(user, "name");
    let login = if login.is_empty() {
        text(user, "slug")
    } else {
        login
    };
    if login.is_empty() {
        return Actor::unknown();
    }
    let display = text(user, "displayName");
    let display = if display.is_empty() {
        login.clone()
    } else {
        display
    };
    let mut actor = Actor::new(login, display, ActorKind::User);
    actor.web_url = user["links"]["self"][0]["href"]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    actor
}

fn iso(v: &Value, key: &str) -> String {
    epoch_ms_to_iso(v[key].as_u64().unwrap_or_default())
}

fn comment_of(c: &Value, parent: Option<&str>, discussion: &str) -> Comment {
    let created_at = iso(c, "createdDate");
    let updated = iso(c, "updatedDate");
    let updated_at = (!updated.is_empty()).then_some(updated);
    let ops = &c["permittedOperations"];
    Comment {
        id: c["id"].as_u64().unwrap_or_default().to_string(),
        author: actor_of(c.get("author")),
        body: Markup::new(text(c, "text"), FLAVOR),
        edited: is_edited(&created_at, updated_at.as_deref()),
        created_at,
        updated_at,
        reactions: Vec::new(),
        in_reply_to: parent.map(str::to_string),
        discussion_id: Some(discussion.to_string()),
        internal: false,
        can_edit: ops["editable"].as_bool(),
        can_delete: ops["deletable"].as_bool(),
        web_url: None,
    }
}

/// A comment and its nested replies, depth first, oldest first.
fn flatten(c: &Value, root: &str, parent: Option<&str>, out: &mut Vec<Comment>) {
    let me = comment_of(c, parent, root);
    let id = me.id.clone();
    out.push(me);
    for reply in c["comments"].as_array().into_iter().flatten() {
        flatten(reply, root, Some(&id), out);
    }
}

fn thread_of(c: &Value, anchor: &Value, comments: Vec<Comment>) -> CodeThread {
    let line = anchor["line"].as_u64().map(|l| l as u32);
    // `fileType` says which file the line belongs to: FROM is the old one.
    let side = if anchor["fileType"].as_str() == Some("FROM") {
        DiffSide::Old
    } else {
        DiffSide::New
    };
    let orphaned = anchor["orphaned"].as_bool() == Some(true);
    CodeThread {
        id: c["id"].as_u64().unwrap_or_default().to_string(),
        path: text(anchor, "path"),
        line: (!orphaned).then_some(line).flatten(),
        original_line: line,
        side,
        start_line: None,
        resolved: c["threadResolved"].as_bool() == Some(true)
            || c["state"].as_str() == Some("RESOLVED"),
        outdated: orphaned,
        resolvable: true,
        diff_hunk: None,
        comments,
    }
}

/// An activity as timeline entries.
fn entries_of(a: &Value, opts: &TimelineOpts, out: &mut Vec<TimelineEntry>) {
    let at = iso(a, "createdDate");
    let id = a["id"].as_u64().unwrap_or_default().to_string();
    let actor = Some(actor_of(a.get("user")));
    let action = a["action"].as_str().unwrap_or_default();
    if !opts.events && !matches!(action, "COMMENTED" | "APPROVED") {
        return;
    }
    let event = |kind: EventKind| {
        TimelineEntry::Event(Event {
            id: id.clone(),
            actor: actor.clone(),
            at: at.clone(),
            kind,
        })
    };
    match action {
        "COMMENTED" => {
            let c = &a["comment"];
            if !c.is_object() {
                return;
            }
            let root = c["id"].as_u64().unwrap_or_default().to_string();
            let mut comments = Vec::new();
            flatten(c, &root, None, &mut comments);
            let anchor = if a["commentAnchor"].is_object() {
                &a["commentAnchor"]
            } else {
                &c["anchor"]
            };
            if anchor.is_object() && !text(anchor, "path").is_empty() {
                out.push(TimelineEntry::Thread(thread_of(c, anchor, comments)));
            } else {
                out.extend(comments.into_iter().map(TimelineEntry::Comment));
            }
        }
        "APPROVED" => out.push(TimelineEntry::Review(ReviewEntry {
            id,
            author: actor_of(a.get("user")),
            state: ReviewState::Approved,
            body: Markup::new("", FLAVOR),
            submitted_at: at,
        })),
        "OPENED" => out.push(event(EventKind::Opened)),
        "MERGED" => out.push(event(EventKind::Merged)),
        "DECLINED" => out.push(event(EventKind::Closed)),
        "REOPENED" => out.push(event(EventKind::Reopened)),
        "RESCOPED" => {
            let added = a["added"]["total"].as_u64().unwrap_or_default() as u32;
            if added > 0 {
                out.push(event(EventKind::CommitsPushed {
                    count: added,
                    head: a["toHash"].as_str().map(str::to_string),
                }));
            } else {
                out.push(event(EventKind::Other {
                    text: "changed the commits of the pull request".to_string(),
                }));
            }
        }
        "UPDATED" => {
            let (from, to) = (text(a, "previousTitle"), text(a, "title"));
            if !from.is_empty() && from != to {
                out.push(event(EventKind::Renamed { from, to }));
            } else {
                out.push(event(EventKind::Other {
                    text: "updated the pull request".to_string(),
                }));
            }
        }
        "UNAPPROVED" => out.push(event(EventKind::Other {
            text: "removed their approval".to_string(),
        })),
        other => out.push(event(EventKind::Other {
            text: other.to_ascii_lowercase().replace('_', " "),
        })),
    }
}

impl BitbucketDcClient {
    fn conv_pr_url(&self, project: &str, key: &ItemKey, rest: &str) -> Result<String> {
        if key.kind != ItemKind::PullRequest {
            return Err(Error::Unsupported(NO_ISSUES.to_string()));
        }
        Ok(format!("{}/{}{rest}", self.pr_url(project)?, key.number))
    }

    async fn conv_get(&self, url: String, what: &str) -> Result<Value> {
        let response = self
            .transport
            .send(self.authed(HttpRequest::get(url)))
            .await?;
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
                self.conv_pr_url(project, key, "")?,
                "reading a Bitbucket Data Center pull request",
            )
            .await?;
        let state = row["state"].as_str().unwrap_or_default();
        let (state_label, open) = match state {
            "OPEN" => ("open", true),
            "MERGED" => ("merged", false),
            "DECLINED" => ("declined", false),
            _ => ("closed", false),
        };
        let header = ItemHeader {
            key: key.clone(),
            reference: format!("#{}", key.number),
            title: text(&row, "title"),
            state_label: state_label.to_string(),
            open,
            author: actor_of(row["author"].get("user")),
            body: Markup::new(text(&row, "description"), FLAVOR),
            created_at: iso(&row, "createdDate"),
            web_url: row["links"]["self"][0]["href"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            labels: Vec::new(),
        };
        let mut start: u64 = match older {
            Some(c) => c
                .0
                .parse()
                .map_err(|_| invalid("reading a conversation", "bad page cursor"))?,
            None => 0,
        };
        let mut entries = Vec::new();
        let mut next: Option<u64> = None;
        for _ in 0..PAGES_PER_WINDOW {
            let url = self.conv_pr_url(
                project,
                key,
                &format!("/activities?limit={LIMIT}&start={start}"),
            )?;
            let page = self
                .conv_get(url, "reading the activity of a Bitbucket pull request")
                .await?;
            for a in page["values"].as_array().into_iter().flatten() {
                entries_of(a, opts, &mut entries);
            }
            next = match (page["isLastPage"].as_bool(), page["nextPageStart"].as_u64()) {
                (Some(false), Some(n)) | (None, Some(n)) => Some(n),
                _ => None,
            };
            match next {
                Some(n) => start = n,
                None => break,
            }
        }
        sort_by_time(&mut entries);
        Ok(TimelinePage {
            header,
            entries,
            older: next.map(|n| PageCursor(n.to_string())),
            total_hint: None,
        })
    }

    pub(crate) async fn conv_post(
        &self,
        project: &str,
        key: &ItemKey,
        body: &str,
        internal: bool,
    ) -> Result<Comment> {
        let url = self.conv_pr_url(project, key, "/comments")?;
        if internal {
            return Err(invalid(
                "commenting on a pull request",
                "Bitbucket has no internal comments",
            ));
        }
        check_body("commenting on a pull request", body)?;
        self.conv_send(url, json!({ "text": body }), None).await
    }

    pub(crate) async fn conv_reply(
        &self,
        project: &str,
        key: &ItemKey,
        discussion_id: &str,
        body: &str,
    ) -> Result<Comment> {
        let url = self.conv_pr_url(project, key, "/comments")?;
        check_body("replying to a comment", body)?;
        let parent: u64 = discussion_id
            .parse()
            .map_err(|_| invalid("replying to a comment", "not a comment of this host"))?;
        self.conv_send(
            url,
            json!({ "text": body, "parent": { "id": parent } }),
            Some(discussion_id),
        )
        .await
    }

    async fn conv_send(
        &self,
        url: String,
        payload: Value,
        discussion: Option<&str>,
    ) -> Result<Comment> {
        let response = self
            .transport
            .send(self.authed(HttpRequest::post_json(url, &payload)))
            .await?;
        response.error_for_status("commenting on a Bitbucket pull request")?;
        let row: Value = response.json()?;
        let id = row["id"].as_u64().unwrap_or_default().to_string();
        Ok(comment_of(
            &row,
            discussion,
            discussion.unwrap_or(id.as_str()),
        ))
    }

    /// The comment as the host holds it now: its `version` is what an edit
    /// or a delete must quote.
    async fn conv_version(&self, project: &str, key: &ItemKey, comment_id: &str) -> Result<Value> {
        let url = self.conv_pr_url(
            project,
            key,
            &format!("/comments/{}", encode_component(comment_id)),
        )?;
        self.conv_get(url, "reading a Bitbucket comment").await
    }

    pub(crate) async fn conv_edit(
        &self,
        project: &str,
        key: &ItemKey,
        comment_id: &str,
        body: &str,
    ) -> Result<Comment> {
        check_body("editing a comment", body)?;
        let current = self.conv_version(project, key, comment_id).await?;
        let version = current["version"].as_u64().unwrap_or_default();
        let url = self.conv_pr_url(
            project,
            key,
            &format!("/comments/{}", encode_component(comment_id)),
        )?;
        let response = self
            .transport
            .send(self.authed(HttpRequest::put_json(
                url,
                &json!({ "text": body, "version": version }),
            )))
            .await?;
        response.error_for_status("editing a Bitbucket comment")?;
        let row: Value = response.json()?;
        Ok(comment_of(&row, None, comment_id))
    }

    pub(crate) async fn conv_delete(
        &self,
        project: &str,
        key: &ItemKey,
        comment_id: &str,
    ) -> Result<()> {
        let current = self.conv_version(project, key, comment_id).await?;
        let version = current["version"].as_u64().unwrap_or_default();
        let url = self.conv_pr_url(
            project,
            key,
            &format!("/comments/{}?version={version}", encode_component(comment_id)),
        )?;
        let response = self
            .transport
            .send(self.authed(HttpRequest::delete(url)))
            .await?;
        response.error_for_status("deleting a Bitbucket comment")
    }

    /// The token's own user name (`X-AUSERNAME`).
    pub(crate) async fn conv_identity(&self, project: &str) -> Result<PostingIdentity> {
        let (source, _) = token_source(&self.scopes, &self.integration.id, project);
        let login = self.my_name().await?;
        Ok(PostingIdentity::new(
            Some(login),
            IdentityKind::PersonalToken,
            source,
        ))
    }
}
