//! The conversation of a Gitea / Forgejo issue or pull request: the header,
//! the timeline (comments and system events), for a pull request also its
//! code threads and submitted reviews (from [`super::review`]), and posting,
//! replying, editing and deleting comments.
//!
//! The timeline route (`issues/{n}/timeline`, Gitea 1.18+ and Forgejo) is
//! oldest first with a total in `X-Total-Count`, so the newest window is read
//! from the last pages backwards; a server without it answers 404 and the
//! plain `issues/{n}/comments` list is read the same way (comments only, no
//! events). The `older` cursor is `t:<page>` or `c:<page>` so a later read
//! goes back to the route that worked.
//!
//! A pull request's code threads and reviews come from the review module and
//! belong to the newest window whatever their age: Gitea does not page them.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use super::{header, GiteaClient, PAGE};
use crate::client::encode_component;
use crate::conversation::compat::{
    check_body, comment_from_review, review_entry, sort_by_time, thread_from_review, token_source,
};
use crate::conversation::time::is_edited;
use crate::conversation::{
    Actor, ActorKind, Comment, Event, EventKind, IdentityKind, ItemHeader, ItemKey, ItemKind,
    Markup, MarkupFlavor, PageCursor, PostingIdentity, TimelineEntry, TimelineOpts, TimelinePage,
    WINDOW,
};
use crate::http::{HttpRequest, HttpResponse};
use crate::review::invalid;
use crate::{Error, Result};

const FLAVOR: MarkupFlavor = MarkupFlavor::GiteaMd;
const PAGES_PER_WINDOW: u32 = (WINDOW / PAGE as usize) as u32;

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

fn actor_of(user: Option<&Value>, original_author: Option<String>) -> Actor {
    let Some(user) = user.filter(|u| u.is_object()) else {
        return match original_author {
            Some(name) => Actor::new("ghost", name, ActorKind::Unknown),
            None => Actor::unknown(),
        };
    };
    let login = text(user, "login");
    let display = original_author
        .or_else(|| opt_text(user, "full_name"))
        .unwrap_or_else(|| login.clone());
    let kind = if user["is_bot"].as_bool() == Some(true) {
        ActorKind::Bot
    } else {
        ActorKind::User
    };
    let mut actor = Actor::new(login, display, kind);
    actor.web_url = opt_text(user, "html_url");
    actor
}

fn comment_of(row: &Value, discussion_id: Option<&str>) -> Comment {
    let created_at = text(row, "created_at");
    let updated_at = opt_text(row, "updated_at");
    Comment {
        id: row["id"].as_u64().unwrap_or_default().to_string(),
        author: actor_of(row.get("user"), opt_text(row, "original_author")),
        body: Markup::new(text(row, "body"), FLAVOR),
        edited: is_edited(&created_at, updated_at.as_deref()),
        created_at,
        updated_at,
        reactions: Vec::new(),
        in_reply_to: None,
        discussion_id: discussion_id.map(str::to_string),
        internal: false,
        can_edit: None,
        can_delete: None,
        web_url: opt_text(row, "html_url"),
    }
}

fn names(items: Option<&Value>) -> Vec<String> {
    items
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|l| l["name"].as_str().or_else(|| l["login"].as_str()))
        .map(str::to_string)
        .collect()
}

/// A timeline row as an event, `None` for the kinds that are shown some
/// other way (comments, and a pull request's reviews and code comments).
fn event_kind(row: &Value) -> Option<EventKind> {
    let kind = text(row, "type");
    let body = text(row, "body");
    Some(match kind.as_str() {
        "close" => EventKind::Closed,
        "reopen" => EventKind::Reopened,
        "merge_pull" => EventKind::Merged,
        "label" => {
            let label = row["label"]["name"].as_str().unwrap_or_default().to_string();
            if body == "1" {
                EventKind::Labeled {
                    added: vec![label],
                    removed: vec![],
                }
            } else {
                EventKind::Labeled {
                    added: vec![],
                    removed: vec![label],
                }
            }
        }
        "assignees" => {
            let who = row["assignee"]["login"]
                .as_str()
                .or_else(|| row["assignee_team"]["name"].as_str())
                .unwrap_or_default()
                .to_string();
            if row["removed_assignee"].as_bool() == Some(true) {
                EventKind::Assigned {
                    added: vec![],
                    removed: vec![who],
                }
            } else {
                EventKind::Assigned {
                    added: vec![who],
                    removed: vec![],
                }
            }
        }
        "change_title" => EventKind::Renamed {
            from: text(row, "old_title"),
            to: text(row, "new_title"),
        },
        "pull_push" => {
            let pushed: Value = serde_json::from_str(&body).unwrap_or(Value::Null);
            let ids: Vec<&str> = pushed["commit_ids"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .collect();
            EventKind::CommitsPushed {
                count: ids.len() as u32,
                head: ids.last().map(|s| s.to_string()),
            }
        }
        "issue_ref" | "pull_ref" | "comment_ref" => {
            let issue = &row["ref_issue"];
            let repo = issue["repository"]["full_name"].as_str().unwrap_or_default();
            let number = issue["number"].as_u64().unwrap_or_default();
            EventKind::CrossReferenced {
                from: format!("{repo}#{number}"),
                url: opt_text(issue, "html_url").or_else(|| opt_text(row, "html_url")),
            }
        }
        "commit_ref" => EventKind::CrossReferenced {
            from: "a commit".to_string(),
            url: opt_text(row, "html_url"),
        },
        "review_request" => EventKind::ReviewRequested {
            who: row["assignee"]["login"]
                .as_str()
                .or_else(|| row["assignee_team"]["name"].as_str())
                .unwrap_or_default()
                .to_string(),
        },
        // Shown as threads and reviews, or not at all.
        "comment" | "review" | "code" => return None,
        other => EventKind::Other {
            text: other.replace('_', " "),
        },
    })
}

/// One window of raw rows.
struct Window {
    rows: Vec<Value>,
    /// The route that answered: the plain comment list (no events).
    legacy: bool,
    older: Option<PageCursor>,
    hint: Option<u32>,
}

fn parse_cursor(cursor: &PageCursor) -> Option<(bool, u32)> {
    let (route, page) = cursor.0.split_once(':')?;
    let legacy = match route {
        "t" => false,
        "c" => true,
        _ => return None,
    };
    page.parse().ok().filter(|p| *p >= 1).map(|p| (legacy, p))
}

impl GiteaClient {
    async fn conv_page(&self, url: &str, page: u32) -> Result<HttpResponse> {
        let sep = if url.contains('?') { '&' } else { '?' };
        self.transport
            .send(self.get(format!("{url}{sep}limit={PAGE}&page={page}")))
            .await
    }

    /// The newest window of one list route, or the window ending at page
    /// `cursor`. `None`: the route does not exist (404 on its first page).
    async fn conv_window(
        &self,
        url: &str,
        legacy: bool,
        cursor: Option<u32>,
        allow_missing: bool,
    ) -> Result<Option<Window>> {
        let what = "reading the conversation of a Gitea or Forgejo item";
        let route = if legacy { 'c' } else { 't' };
        let mut pages: BTreeMap<u32, Vec<Value>> = BTreeMap::new();
        let mut total: Option<u32> = None;
        let high = match cursor {
            Some(c) => c,
            None => {
                let first = self.conv_page(url, 1).await?;
                if allow_missing && first.status == 404 {
                    return Ok(None);
                }
                first.error_for_status(what)?;
                total = header(&first, "x-total-count").and_then(|t| t.parse().ok());
                pages.insert(1, first.json()?);
                match total {
                    Some(t) => t.div_ceil(PAGE).max(1),
                    None => {
                        // No total: walk forward to the last page.
                        let mut last = 1;
                        while pages[&last].len() >= PAGE as usize && last < crate::client::MAX_PAGES
                        {
                            last += 1;
                            let r = self.conv_page(url, last).await?;
                            r.error_for_status(what)?;
                            pages.insert(last, r.json()?);
                        }
                        last
                    }
                }
            }
        };
        let low = high.saturating_sub(PAGES_PER_WINDOW - 1).max(1);
        let mut rows: Vec<Value> = Vec::new();
        for page in low..=high {
            let page_rows = match pages.remove(&page) {
                Some(r) => r,
                None => {
                    let r = self.conv_page(url, page).await?;
                    r.error_for_status(what)?;
                    r.json()?
                }
            };
            if page == high && cursor.is_none() && total.is_none() {
                total = Some((high - 1) * PAGE + page_rows.len() as u32);
            }
            rows.extend(page_rows);
        }
        Ok(Some(Window {
            rows,
            legacy,
            older: (low > 1).then(|| PageCursor(format!("{route}:{}", low - 1))),
            hint: if cursor.is_none() { total } else { None },
        }))
    }

    async fn conv_rows(
        &self,
        project: &str,
        number: u64,
        older: Option<&PageCursor>,
    ) -> Result<Window> {
        let timeline = self.repo_url(project, &format!("issues/{number}/timeline"));
        let comments = self.repo_url(project, &format!("issues/{number}/comments"));
        if let Some(cursor) = older {
            let (legacy, page) = parse_cursor(cursor)
                .ok_or_else(|| invalid("reading a conversation", "bad page cursor"))?;
            let url = if legacy { &comments } else { &timeline };
            return self
                .conv_window(url, legacy, Some(page), false)
                .await?
                .ok_or_else(|| invalid("reading a conversation", "no such page"));
        }
        if let Some(window) = self.conv_window(&timeline, false, None, true).await? {
            return Ok(window);
        }
        self.conv_window(&comments, true, None, false)
            .await?
            .ok_or_else(|| invalid("reading a conversation", "no such item"))
    }

    fn conv_header(&self, row: &Value, key: &ItemKey) -> ItemHeader {
        let merged = row["pull_request"]["merged"].as_bool() == Some(true);
        let open = row["state"].as_str() == Some("open");
        let state_label = if merged {
            "merged"
        } else if open {
            "open"
        } else {
            "closed"
        };
        ItemHeader {
            key: key.clone(),
            reference: format!("#{}", key.number),
            title: text(row, "title"),
            state_label: state_label.to_string(),
            open,
            author: actor_of(row.get("user"), opt_text(row, "original_author")),
            body: Markup::new(text(row, "body"), FLAVOR),
            created_at: text(row, "created_at"),
            web_url: text(row, "html_url"),
            labels: names(row.get("labels")),
        }
    }

    pub(super) async fn conv_timeline(
        &self,
        project: &str,
        key: &ItemKey,
        older: Option<&PageCursor>,
        opts: &TimelineOpts,
    ) -> Result<TimelinePage> {
        let row: Value = self
            .get_json(
                self.repo_url(project, &format!("issues/{}", key.number)),
                "reading a Gitea or Forgejo issue or pull request",
            )
            .await?;
        let header = self.conv_header(&row, key);
        let window = self.conv_rows(project, key.number, older).await?;
        let mut entries: Vec<TimelineEntry> = Vec::new();
        for r in &window.rows {
            if window.legacy || r["type"].as_str() == Some("comment") {
                entries.push(TimelineEntry::Comment(comment_of(r, None)));
            } else if opts.events {
                if let Some(kind) = event_kind(r) {
                    entries.push(TimelineEntry::Event(Event {
                        id: r["id"].as_u64().unwrap_or_default().to_string(),
                        actor: Some(actor_of(r.get("user"), None)),
                        at: text(r, "created_at"),
                        kind,
                    }));
                }
            }
        }
        if key.kind == ItemKind::PullRequest && older.is_none() {
            for t in self.review_threads(project, key.number).await? {
                entries.push(TimelineEntry::Thread(thread_from_review(t, FLAVOR)));
            }
            for r in self.review_list_reviews(project, key.number).await? {
                entries.push(TimelineEntry::Review(review_entry(r, FLAVOR)));
            }
        }
        sort_by_time(&mut entries);
        Ok(TimelinePage {
            header,
            entries,
            older: window.older,
            total_hint: window.hint,
        })
    }

    pub(super) async fn conv_post(
        &self,
        project: &str,
        key: &ItemKey,
        body: &str,
        internal: bool,
    ) -> Result<Comment> {
        let label = match key.kind {
            ItemKind::PullRequest => "commenting on a pull request",
            ItemKind::Issue => "commenting on an issue",
        };
        if internal {
            return Err(invalid(
                label,
                format!("{} has no internal comments", self.label()),
            ));
        }
        check_body(label, body)?;
        let url = self.repo_url(project, &format!("issues/{}/comments", key.number));
        let response = self
            .transport
            .send(self.post(url, &json!({ "body": body })))
            .await?;
        response.error_for_status(label)?;
        Ok(comment_of(&response.json::<Value>()?, None))
    }

    pub(super) async fn conv_reply(
        &self,
        project: &str,
        key: &ItemKey,
        discussion_id: &str,
        body: &str,
    ) -> Result<Comment> {
        if key.kind != ItemKind::PullRequest {
            return Err(Error::Unsupported(format!(
                "replying in a thread of a {} issue",
                self.label()
            )));
        }
        let reply = self
            .review_reply(project, key.number, discussion_id, body)
            .await?;
        Ok(comment_from_review(reply, Some(discussion_id), FLAVOR))
    }

    fn conv_comment_url(&self, project: &str, id: &str) -> String {
        self.repo_url(
            project,
            &format!("issues/comments/{}", encode_component(id)),
        )
    }

    pub(super) async fn conv_edit(
        &self,
        project: &str,
        comment_id: &str,
        body: &str,
    ) -> Result<Comment> {
        check_body("editing a comment", body)?;
        let url = self.conv_comment_url(project, comment_id);
        let response = self
            .transport
            .send(self.authed(HttpRequest::patch_json(url, &json!({ "body": body }))))
            .await?;
        response.error_for_status("editing a comment")?;
        Ok(comment_of(&response.json::<Value>()?, None))
    }

    pub(super) async fn conv_delete(&self, project: &str, comment_id: &str) -> Result<()> {
        let url = self.conv_comment_url(project, comment_id);
        let response = self
            .transport
            .send(self.authed(HttpRequest::delete(url)))
            .await?;
        response.error_for_status("deleting a comment")
    }

    /// The token's own user. The profile's cached login is its own token's;
    /// the repository's token command may be a different user, so it is
    /// asked.
    pub(super) async fn conv_identity(&self, project: &str) -> Result<PostingIdentity> {
        let (source, by_command) = token_source(&self.scopes, &self.integration.id, project);
        let login = if by_command {
            self.fetch_user().await?.login
        } else {
            self.login().await?
        };
        if login.is_empty() {
            return Err(Error::Decode("no login in the user".to_string()));
        }
        Ok(PostingIdentity::new(
            Some(login),
            IdentityKind::PersonalToken,
            source,
        ))
    }
}
