//! The conversation of a GitLab issue or merge request: the header, its
//! discussions (system notes become events, positional discussions become
//! code threads) and posting, replying, editing and deleting notes.
//!
//! Issues and merge requests share one shape on GitLab: a list of
//! *discussions*, each a list of *notes*. The listing is oldest first with no
//! way to reverse it, so the newest window is found from `X-Total-Pages` and
//! read from the last page backwards.

use serde_json::{json, Value};

use super::GitLabClient;
use crate::client::encode_component;
use crate::conversation::time::is_edited;
use crate::conversation::{
    Actor, ActorKind, CodeThread, Comment, Event, EventKind, IdentityKind, ItemHeader, ItemKey,
    ItemKind, Markup, MarkupFlavor, PageCursor, PostingIdentity, TimelineEntry, TimelineOpts,
    TimelinePage, TokenSource, WINDOW,
};
use crate::gitlab::encode_gitlab_project_path;
use crate::http::HttpResponse;
use crate::review::gitlab::{thread_of, Discussion};
use crate::review::invalid;
use crate::{Error, Result};

const PAGES_PER_WINDOW: u32 = (WINDOW / 100) as u32;

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

/// Project and group access tokens act as users named `project_<id>_bot...`
/// / `group_<id>_bot...`; newer GitLab also says so in a `bot` flag.
fn is_bot_login(login: &str) -> bool {
    let rest = login
        .strip_prefix("project_")
        .or_else(|| login.strip_prefix("group_"));
    rest.is_some_and(|r| {
        let digits = r.chars().take_while(char::is_ascii_digit).count();
        digits > 0 && r[digits..].starts_with("_bot")
    })
}

fn actor_of(user: Option<&Value>) -> Actor {
    let Some(user) = user.filter(|u| u.is_object()) else {
        return Actor::unknown();
    };
    let login = text(user, "username");
    let kind = if user["bot"].as_bool() == Some(true) || is_bot_login(&login) {
        ActorKind::Bot
    } else {
        ActorKind::User
    };
    let mut actor = Actor::new(login.clone(), opt_text(user, "name").unwrap_or(login), kind);
    actor.web_url = opt_text(user, "web_url");
    actor
}

fn note_comment(
    note: &Value,
    discussion_id: Option<&str>,
    in_reply_to: Option<String>,
    item_url: &str,
) -> Comment {
    let id = note["id"].as_u64().unwrap_or_default();
    let created_at = text(note, "created_at");
    let updated_at = opt_text(note, "updated_at");
    Comment {
        id: id.to_string(),
        author: actor_of(note.get("author")),
        body: Markup::new(text(note, "body"), MarkupFlavor::GitlabMd),
        edited: is_edited(&created_at, updated_at.as_deref()),
        created_at,
        updated_at,
        reactions: Vec::new(),
        in_reply_to,
        discussion_id: discussion_id.map(str::to_string),
        internal: note["internal"].as_bool().unwrap_or(false)
            || note["confidential"].as_bool().unwrap_or(false),
        can_edit: None,
        can_delete: None,
        web_url: (!item_url.is_empty()).then(|| format!("{item_url}#note_{id}")),
    }
}

/// Splits a system note's `~123 label` / `~"bug" label` list.
fn labels_in(body: &str, verb: &str) -> Vec<String> {
    let Some(rest) = body.strip_prefix(verb) else {
        return Vec::new();
    };
    let rest = rest
        .trim()
        .trim_end_matches(" labels")
        .trim_end_matches(" label");
    rest.split(" ~")
        .map(|l| {
            l.trim()
                .trim_start_matches('~')
                .trim_matches('"')
                .to_string()
        })
        .filter(|l| !l.is_empty())
        .collect()
}

fn mentions_in(body: &str, verb: &str) -> Vec<String> {
    body.strip_prefix(verb)
        .map(|r| {
            r.split([',', ' '])
                .filter(|w| w.starts_with('@'))
                .map(|w| w.trim_start_matches('@').to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// What a system note says, as an event kind.
fn system_kind(body: &str) -> EventKind {
    let b = body.trim();
    match b {
        "closed" => return EventKind::Closed,
        "reopened" => return EventKind::Reopened,
        _ => {}
    }
    if b.starts_with("merged") {
        return EventKind::Merged;
    }
    if b.starts_with("added ") && (b.ends_with(" label") || b.ends_with(" labels")) {
        return EventKind::Labeled {
            added: labels_in(b, "added "),
            removed: Vec::new(),
        };
    }
    if b.starts_with("removed ") && (b.ends_with(" label") || b.ends_with(" labels")) {
        return EventKind::Labeled {
            added: Vec::new(),
            removed: labels_in(b, "removed "),
        };
    }
    if let Some(rest) = b.strip_prefix("assigned to ") {
        return EventKind::Assigned {
            added: mentions_in(&format!("assigned to {rest}"), "assigned to "),
            removed: Vec::new(),
        };
    }
    if b.starts_with("unassigned ") {
        return EventKind::Assigned {
            added: Vec::new(),
            removed: mentions_in(b, "unassigned "),
        };
    }
    if let Some(rest) = b.strip_prefix("changed title from ") {
        let strip = |s: &str| s.trim().trim_matches('*').to_string();
        if let Some((from, to)) = rest.split_once(" to ") {
            return EventKind::Renamed {
                from: strip(from),
                to: strip(to),
            };
        }
    }
    if let Some(rest) = b.strip_prefix("added ") {
        if let Some((n, tail)) = rest.split_once(' ') {
            if let (Ok(count), true) = (n.parse::<u32>(), tail.starts_with("commit")) {
                return EventKind::CommitsPushed { count, head: None };
            }
        }
    }
    if let Some(from) = b.strip_prefix("mentioned in ") {
        return EventKind::CrossReferenced {
            from: from.to_string(),
            url: None,
        };
    }
    if b.starts_with("requested review from ") {
        return EventKind::ReviewRequested {
            who: mentions_in(b, "requested review from ").join(", "),
        };
    }
    EventKind::Other {
        text: b.to_string(),
    }
}

/// `last` page from `X-Total-Pages`; 1 when the host does not say.
fn total_pages(response: &HttpResponse) -> u32 {
    response
        .headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("x-total-pages"))
        .and_then(|(_, v)| v.trim().parse().ok())
        .filter(|n| *n >= 1)
        .unwrap_or(1)
}

fn parse_cursor(cursor: &PageCursor) -> Option<u32> {
    cursor.0.parse().ok().filter(|p| *p >= 1)
}

impl GitLabClient {
    /// `<api>/projects/<id>/{issues|merge_requests}/<iid><rest>`.
    fn item_url(&self, project: &str, key: &ItemKey, rest: &str) -> String {
        let route = match key.kind {
            ItemKind::Issue => "issues",
            ItemKind::PullRequest => "merge_requests",
        };
        format!(
            "{}/projects/{}/{route}/{}{rest}",
            self.integration.api_base(),
            encode_gitlab_project_path(project),
            key.number
        )
    }

    async fn conv_item_row(&self, project: &str, key: &ItemKey) -> Result<Value> {
        let response = self
            .transport
            .send(self.get(self.item_url(project, key, "")))
            .await?;
        response.error_for_status("reading a GitLab issue or merge request")?;
        response.json()
    }

    fn header_of(&self, row: &Value, key: &ItemKey) -> ItemHeader {
        let state = row["state"].as_str().unwrap_or_default();
        let (state_label, open) = match state {
            "opened" => ("open", true),
            "merged" => ("merged", false),
            _ => ("closed", false),
        };
        ItemHeader {
            key: key.clone(),
            reference: match key.kind {
                ItemKind::Issue => format!("#{}", key.number),
                ItemKind::PullRequest => format!("!{}", key.number),
            },
            title: text(row, "title"),
            state_label: state_label.to_string(),
            open,
            author: actor_of(row.get("author")),
            body: Markup::new(text(row, "description"), MarkupFlavor::GitlabMd),
            created_at: text(row, "created_at"),
            web_url: text(row, "web_url"),
            labels: row["labels"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|l| l.as_str().or_else(|| l["name"].as_str()))
                .map(str::to_string)
                .collect(),
        }
    }

    async fn discussions_page(
        &self,
        project: &str,
        key: &ItemKey,
        page: u32,
    ) -> Result<(Vec<Value>, HttpResponse)> {
        let url = self.item_url(
            project,
            key,
            &format!("/discussions?per_page=100&page={page}"),
        );
        let response = self.transport.send(self.get(url)).await?;
        response.error_for_status("reading the discussions of a GitLab issue or merge request")?;
        let rows: Vec<Value> = response.json()?;
        Ok((rows, response))
    }

    /// Newest window of discussions (the last pages first) or the window
    /// ending at page `cursor`, the next older page and a count hint.
    async fn discussions_window(
        &self,
        project: &str,
        key: &ItemKey,
        cursor: Option<u32>,
    ) -> Result<(Vec<Value>, Option<u32>, Option<u32>)> {
        let mut first: Option<(Vec<Value>, HttpResponse)> = None;
        let (high, last_known) = match cursor {
            Some(c) => (c, None),
            None => {
                let page_one = self.discussions_page(project, key, 1).await?;
                let last = total_pages(&page_one.1);
                first = Some(page_one);
                (last, Some(last))
            }
        };
        let low = high.saturating_sub(PAGES_PER_WINDOW - 1).max(1);
        let mut rows: Vec<Value> = Vec::new();
        let mut last_len = 0usize;
        for page in low..=high {
            let page_rows = match first.take() {
                Some((r, _)) if page == 1 => r,
                kept => {
                    first = kept;
                    self.discussions_page(project, key, page).await?.0
                }
            };
            if page == high {
                last_len = page_rows.len();
            }
            rows.extend(page_rows);
        }
        let hint = last_known.map(|last| (last - 1) * 100 + last_len as u32);
        Ok((rows, (low > 1).then(|| low - 1), hint))
    }

    pub(super) async fn conv_timeline(
        &self,
        project: &str,
        key: &ItemKey,
        older: Option<&PageCursor>,
        opts: &TimelineOpts,
    ) -> Result<TimelinePage> {
        let cursor = match older {
            Some(c) => Some(
                parse_cursor(c)
                    .ok_or_else(|| invalid("reading a conversation", "bad page cursor"))?,
            ),
            None => None,
        };
        let row = self.conv_item_row(project, key).await?;
        let header = self.header_of(&row, key);
        let head_sha = text(&row, "sha");
        let (discussions, next, total_hint) = self.discussions_window(project, key, cursor).await?;
        let mut entries = Vec::new();
        for d in discussions {
            self.entries_of_discussion(&d, &header.web_url, &head_sha, opts, &mut entries);
        }
        Ok(TimelinePage {
            header,
            entries,
            older: next.map(|p| PageCursor(p.to_string())),
            total_hint,
        })
    }

    fn entries_of_discussion(
        &self,
        d: &Value,
        item_url: &str,
        head_sha: &str,
        opts: &TimelineOpts,
        out: &mut Vec<TimelineEntry>,
    ) {
        let id = text(d, "id");
        let notes = d["notes"].as_array().cloned().unwrap_or_default();
        let Some(first) = notes.first() else {
            return;
        };
        if first["system"].as_bool() == Some(true) {
            if opts.events {
                for n in notes.iter().filter(|n| n["system"].as_bool() == Some(true)) {
                    out.push(TimelineEntry::Event(Event {
                        id: n["id"].as_u64().unwrap_or_default().to_string(),
                        actor: Some(actor_of(n.get("author"))),
                        at: text(n, "created_at"),
                        kind: system_kind(&text(n, "body")),
                    }));
                }
            }
            return;
        }
        let first_id = first["id"].as_u64().unwrap_or_default().to_string();
        let comments: Vec<Comment> = notes
            .iter()
            .filter(|n| n["system"].as_bool() != Some(true))
            .enumerate()
            .map(|(i, n)| note_comment(n, Some(&id), (i > 0).then(|| first_id.clone()), item_url))
            .collect();
        if first.get("position").is_some_and(|p| !p.is_null()) {
            let parsed = serde_json::from_value::<Discussion>(d.clone())
                .ok()
                .and_then(|disc| thread_of(disc, head_sha));
            if let Some(t) = parsed {
                out.push(TimelineEntry::Thread(CodeThread {
                    id: t.id,
                    path: t.path,
                    line: t.line,
                    original_line: t.original_line,
                    side: t.side,
                    start_line: t.start_line,
                    resolved: t.resolved,
                    outdated: t.outdated,
                    resolvable: t.resolvable,
                    diff_hunk: None,
                    comments,
                }));
                return;
            }
        }
        out.extend(comments.into_iter().map(TimelineEntry::Comment));
    }

    pub(super) async fn conv_post(
        &self,
        project: &str,
        key: &ItemKey,
        body: &str,
        internal: bool,
    ) -> Result<Comment> {
        let label = match key.kind {
            ItemKind::PullRequest => "commenting on a merge request",
            ItemKind::Issue => "commenting on an issue",
        };
        check_body(label, body)?;
        let mut payload = json!({ "body": body });
        if internal {
            payload["internal"] = json!(true);
        }
        let response = self
            .transport
            .send(self.post(self.item_url(project, key, "/notes"), &payload))
            .await?;
        response.error_for_status("commenting on a GitLab issue or merge request")?;
        Ok(note_comment(&response.json::<Value>()?, None, None, ""))
    }

    pub(super) async fn conv_reply(
        &self,
        project: &str,
        key: &ItemKey,
        discussion_id: &str,
        body: &str,
    ) -> Result<Comment> {
        check_body("replying to a discussion", body)?;
        let url = self.item_url(
            project,
            key,
            &format!("/discussions/{}/notes", encode_component(discussion_id)),
        );
        let response = self
            .transport
            .send(self.post(url, &json!({ "body": body })))
            .await?;
        response.error_for_status("replying to a GitLab discussion")?;
        Ok(note_comment(
            &response.json::<Value>()?,
            Some(discussion_id),
            None,
            "",
        ))
    }

    pub(super) async fn conv_edit(
        &self,
        project: &str,
        key: &ItemKey,
        comment_id: &str,
        body: &str,
    ) -> Result<Comment> {
        check_body("editing a comment", body)?;
        let url = self.item_url(
            project,
            key,
            &format!("/notes/{}", encode_component(comment_id)),
        );
        let response = self
            .transport
            .send(self.put(url, &json!({ "body": body })))
            .await?;
        response.error_for_status("editing a GitLab note")?;
        Ok(note_comment(&response.json::<Value>()?, None, None, ""))
    }

    pub(super) async fn conv_delete(
        &self,
        project: &str,
        key: &ItemKey,
        comment_id: &str,
    ) -> Result<()> {
        let url = self.item_url(
            project,
            key,
            &format!("/notes/{}", encode_component(comment_id)),
        );
        let response = self.transport.send(self.delete(url)).await?;
        response.error_for_status("deleting a GitLab note")
    }

    /// Who a post is made as: the token's own user (`GET /user`); a project
    /// or group access token answers `bot: true` (or is named like one).
    pub(super) async fn conv_identity(&self, project: &str) -> Result<PostingIdentity> {
        let id = &self.integration.id;
        let source = if self.scopes.acts_as_bot(id, project) {
            TokenSource::RepoTokenCommand
        } else {
            TokenSource::Profile { id: id.clone() }
        };
        let url = format!("{}/user", self.integration.api_base());
        let response = self.transport.send(self.get(url)).await?;
        response.error_for_status("reading the GitLab token's user")?;
        let user: Value = response.json()?;
        let login = text(&user, "username");
        if login.is_empty() {
            return Err(Error::Decode("no username in the GitLab user".to_string()));
        }
        let kind = if user["bot"].as_bool() == Some(true) || is_bot_login(&login) {
            IdentityKind::ProjectOrGroupBot
        } else {
            IdentityKind::PersonalToken
        };
        Ok(PostingIdentity::new(Some(login), kind, source))
    }
}

fn check_body(what: &str, body: &str) -> Result<()> {
    if body.trim().is_empty() {
        return Err(invalid(what, "the comment is empty"));
    }
    Ok(())
}
