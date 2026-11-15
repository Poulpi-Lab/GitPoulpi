//! The conversation of an Azure DevOps pull request (threads) or work item
//! (a flat comment list): the header, the timeline and posting, replying,
//! editing and deleting comments.
//!
//! **Pull requests.** `.../pullRequests/{id}/threads` answers every thread
//! at once, so the window is cut from the whole and `older` is the index
//! where the window starts. A thread without a file context is a general
//! conversation; with `threadContext.filePath` it is a code thread; a thread
//! made of system comments (`commentType: system`, or a
//! `CodeReviewThreadType` property) is an event, and a `VoteUpdate` is a
//! verdict. A comment id is `"{thread}.{comment}"` because an edit or a
//! delete needs both; the discussion id of a comment is the thread id. A
//! reply answers the thread's first comment (comment ids start at 1).
//!
//! **Work items.** `.../wit/workItems/{id}/comments` (preview API), newest
//! first with `continuationToken` as the `older` cursor. Services answers and
//! takes Markdown (`format=markdown`); Server (7.0) answers HTML bodies,
//! shown through the HTML importer, and takes HTML, generated from the
//! Markdown. The description is always an HTML field. There is no thread and
//! no event history.

use serde_json::{json, Value};

use super::AzureDevopsClient;
use crate::azure_devops::map::is_closed_state;
use crate::azure_devops::spec::split_project;
use crate::client::encode_component as enc;
use crate::conversation::compat::{check_body, sort_by_time, token_source};
use crate::conversation::time::is_edited;
use crate::conversation::{
    Actor, ActorKind, CodeThread, Comment, Event, EventKind, IdentityKind, ItemHeader, ItemKey,
    ItemKind, Markup, MarkupFlavor, PageCursor, PostingIdentity, ReviewEntry, TimelineEntry,
    TimelineOpts, TimelinePage, WINDOW,
};
use crate::http::HttpRequest;
use crate::markup::{html_to_markdown, markdown_to_html};
use crate::review::invalid;
use crate::review::model::{DiffSide, ReviewState};
use crate::{Error, Result};
use gitpoulpi_config::IntegrationKind;

const FLAVOR: MarkupFlavor = MarkupFlavor::AzureMd;
const PAGE: usize = 100;
const PAGES_PER_WINDOW: usize = WINDOW / PAGE;

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

/// An identity (`displayName`, `uniqueName`) or a `"Name <email>"` string.
fn actor_of(v: Option<&Value>) -> Actor {
    match v {
        Some(Value::Object(o)) => {
            let unique = o
                .get("uniqueName")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let display = o
                .get("displayName")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let login = if unique.is_empty() { display } else { unique };
            if login.is_empty() {
                return Actor::unknown();
            }
            let mut actor = Actor::new(
                login,
                if display.is_empty() { login } else { display },
                ActorKind::User,
            );
            actor.web_url = o
                .get("url")
                .and_then(Value::as_str)
                .filter(|u| u.starts_with("http"))
                .map(str::to_string);
            actor
        }
        Some(Value::String(s)) => {
            let name = s.split('<').next().unwrap_or(s).trim();
            let email = s
                .split_once('<')
                .and_then(|(_, r)| r.strip_suffix('>'))
                .unwrap_or(name);
            if name.is_empty() {
                Actor::unknown()
            } else {
                Actor::new(email, name, ActorKind::User)
            }
        }
        _ => Actor::unknown(),
    }
}

/// HTML written by Azure DevOps Server or an older Services comment.
fn looks_like_html(body: &str) -> bool {
    let b = body.trim_start();
    b.starts_with('<')
        || ["<div", "<p>", "<p ", "<br", "<a ", "<span", "<ul", "<ol"]
            .iter()
            .any(|t| body.contains(t))
}

fn pr_comment_of(thread: u64, c: &Value) -> Comment {
    let created_at = text(c, "publishedDate");
    let updated = opt_text(c, "lastContentUpdatedDate").or_else(|| opt_text(c, "lastUpdatedDate"));
    let id = c["id"].as_u64().unwrap_or_default();
    let parent = c["parentCommentId"].as_u64().filter(|p| *p > 0);
    Comment {
        id: format!("{thread}.{id}"),
        author: actor_of(c.get("author")),
        body: Markup::new(text(c, "content"), FLAVOR),
        edited: is_edited(&created_at, updated.as_deref()),
        created_at,
        updated_at: updated,
        reactions: Vec::new(),
        in_reply_to: parent.map(|p| format!("{thread}.{p}")),
        discussion_id: Some(thread.to_string()),
        internal: false,
        can_edit: None,
        can_delete: None,
        web_url: None,
    }
}

fn thread_property(thread: &Value, name: &str) -> Option<String> {
    let p = &thread["properties"][name];
    p["$value"]
        .as_str()
        .map(str::to_string)
        .or_else(|| p["$value"].as_i64().map(|n| n.to_string()))
}

/// A system thread as an event or a verdict.
fn system_entry(thread: &Value, comments: &[&Value]) -> Option<TimelineEntry> {
    let first = comments.first()?;
    let content = text(first, "content");
    let at = text(first, "publishedDate");
    let id = thread["id"].as_u64().unwrap_or_default().to_string();
    let actor = actor_of(first.get("author"));
    let kind = thread_property(thread, "CodeReviewThreadType").unwrap_or_default();
    if kind == "VoteUpdate" {
        let vote: i32 = thread_property(thread, "CodeReviewVoteResult")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let state = match vote {
            v if v > 0 => ReviewState::Approved,
            v if v < 0 => ReviewState::ChangesRequested,
            _ => {
                return Some(TimelineEntry::Event(Event {
                    id,
                    actor: Some(actor),
                    at,
                    kind: EventKind::Other { text: content },
                }))
            }
        };
        let voter = thread_property(thread, "CodeReviewVotedByIdentity")
            .map(|_| actor.clone())
            .unwrap_or(actor);
        return Some(TimelineEntry::Review(ReviewEntry {
            id,
            author: voter,
            state,
            body: Markup::new(content, FLAVOR),
            submitted_at: at,
        }));
    }
    let lower = content.to_ascii_lowercase();
    let event = if kind == "StatusUpdate" || kind.is_empty() {
        if lower.contains("completed") {
            EventKind::Merged
        } else if lower.contains("abandoned") {
            EventKind::Closed
        } else if lower.contains("reactivated") {
            EventKind::Reopened
        } else {
            EventKind::Other { text: content }
        }
    } else if kind == "ReviewerAdded" {
        EventKind::ReviewRequested {
            who: content
                .trim_end_matches(" as a reviewer")
                .trim_end_matches(" as a required reviewer")
                .rsplit(" added ")
                .next()
                .unwrap_or_default()
                .to_string(),
        }
    } else {
        EventKind::Other { text: content }
    };
    Some(TimelineEntry::Event(Event {
        id,
        actor: Some(actor),
        at,
        kind: event,
    }))
}

fn resolved_status(status: &str) -> bool {
    matches!(
        status.to_ascii_lowercase().as_str(),
        "fixed" | "closed" | "wontfix" | "bydesign"
    )
}

fn code_thread(thread: &Value, comments: Vec<Comment>) -> Option<CodeThread> {
    let ctx = &thread["threadContext"];
    let path = ctx["filePath"].as_str().filter(|p| !p.is_empty())?;
    let num = |v: &Value| v["line"].as_u64().map(|n| n as u32);
    let (side, start, end) = if ctx["rightFileStart"].is_object() {
        (
            DiffSide::New,
            num(&ctx["rightFileStart"]),
            num(&ctx["rightFileEnd"]),
        )
    } else {
        (
            DiffSide::Old,
            num(&ctx["leftFileStart"]),
            num(&ctx["leftFileEnd"]),
        )
    };
    let line = end.or(start);
    Some(CodeThread {
        id: thread["id"].as_u64().unwrap_or_default().to_string(),
        path: path.trim_start_matches('/').to_string(),
        line,
        original_line: line,
        side,
        start_line: start.filter(|s| Some(*s) != line),
        resolved: resolved_status(thread["status"].as_str().unwrap_or_default()),
        outdated: false,
        resolvable: true,
        diff_hunk: None,
        comments,
    })
}

fn pr_entries(thread: &Value, out: &mut Vec<TimelineEntry>, events: bool) {
    if thread["isDeleted"].as_bool() == Some(true) {
        return;
    }
    let live: Vec<&Value> = thread["comments"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|c| c["isDeleted"].as_bool() != Some(true))
        .collect();
    if live.is_empty() {
        return;
    }
    let is_system = live
        .iter()
        .all(|c| c["commentType"].as_str() == Some("system"))
        || thread["properties"]["CodeReviewThreadType"].is_object();
    if is_system {
        let entry = system_entry(thread, &live);
        if let Some(e) = entry {
            if events || matches!(e, TimelineEntry::Review(_)) {
                out.push(e);
            }
        }
        return;
    }
    let id = thread["id"].as_u64().unwrap_or_default();
    let comments: Vec<Comment> = live
        .iter()
        .filter(|c| c["commentType"].as_str() != Some("system"))
        .map(|c| pr_comment_of(id, c))
        .collect();
    if comments.is_empty() {
        return;
    }
    match code_thread(thread, comments.clone()) {
        Some(t) => out.push(TimelineEntry::Thread(t)),
        None => out.extend(comments.into_iter().map(TimelineEntry::Comment)),
    }
}

impl AzureDevopsClient {
    pub(super) fn is_server(&self) -> bool {
        self.integration.kind == IntegrationKind::AzureDevopsServer
    }

    /// The version the work item comment routes are in (still a preview).
    fn comments_version(&self) -> &'static str {
        if self.is_server() {
            "7.0-preview.3"
        } else {
            "7.1-preview.4"
        }
    }

    /// `{root}/{project}/_apis/{path}?{query&}api-version={version}`.
    fn conv_url(&self, project: &str, path: &str, query: &str, version: &str) -> Result<String> {
        let mut url = self.url(Some(project), path, query)?;
        let cut = url.rfind("api-version=").unwrap_or(url.len());
        url.truncate(cut);
        url.push_str("api-version=");
        url.push_str(version);
        Ok(url)
    }

    async fn pr_threads_url(
        &self,
        project: &str,
        id: u64,
        rest: &str,
        query: &str,
    ) -> Result<String> {
        let (p, _) = split_project(project);
        let repo = self.repo_id(project).await?;
        self.url(
            Some(p),
            &format!(
                "git/repositories/{}/pullRequests/{id}/threads{rest}",
                enc(&repo)
            ),
            query,
        )
    }

    async fn conv_json(&self, request: HttpRequest, what: &str) -> Result<Value> {
        self.send(request, what).await?.json()
    }

    pub(super) async fn conv_timeline(
        &self,
        project: &str,
        key: &ItemKey,
        older: Option<&PageCursor>,
        opts: &TimelineOpts,
    ) -> Result<TimelinePage> {
        match key.kind {
            ItemKind::PullRequest => self.pr_timeline(project, key, older, opts).await,
            ItemKind::Issue => self.item_timeline_wi(project, key, older).await,
        }
    }

    async fn pr_timeline(
        &self,
        project: &str,
        key: &ItemKey,
        older: Option<&PageCursor>,
        opts: &TimelineOpts,
    ) -> Result<TimelinePage> {
        let end: Option<usize> = match older {
            Some(c) => Some(
                c.0.parse()
                    .map_err(|_| invalid("reading a conversation", "bad page cursor"))?,
            ),
            None => None,
        };
        let (p, r) = split_project(project);
        let repo = self.repo_id(project).await?;
        let pull_url = self.url(
            Some(p),
            &format!(
                "git/repositories/{}/pullRequests/{}",
                enc(&repo),
                key.number
            ),
            "",
        )?;
        let row = self
            .conv_json(self.get(pull_url), "reading an Azure DevOps pull request")
            .await?;
        let (state_label, open) = match row["status"].as_str().unwrap_or_default() {
            "completed" => ("merged", false),
            "abandoned" => ("abandoned", false),
            _ if row["isDraft"].as_bool() == Some(true) => ("draft", true),
            _ => ("open", true),
        };
        let spec = crate::spec::spec_for(gitpoulpi_config::ApiFamily::AzureDevops);
        let header = ItemHeader {
            key: key.clone(),
            reference: format!("!{}", key.number),
            title: text(&row, "title"),
            state_label: state_label.to_string(),
            open,
            author: actor_of(row.get("createdBy")),
            body: Markup::new(text(&row, "description"), FLAVOR),
            created_at: text(&row, "creationDate"),
            web_url: (spec.pr_web_url)(&self.root()?, &format!("{p}/{r}"), key.number)
                .unwrap_or_default(),
            labels: row["labels"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|l| l["name"].as_str())
                .map(str::to_string)
                .collect(),
        };
        let threads_url = self.pr_threads_url(project, key.number, "", "").await?;
        let list = self
            .conv_json(
                self.get(threads_url),
                "reading the threads of a pull request",
            )
            .await?;
        let mut entries = Vec::new();
        for thread in list["value"].as_array().into_iter().flatten() {
            pr_entries(thread, &mut entries, opts.events);
        }
        sort_by_time(&mut entries);
        let end = end.unwrap_or(entries.len()).min(entries.len());
        let start = end.saturating_sub(WINDOW);
        let total = entries.len() as u32;
        let window: Vec<TimelineEntry> = entries.drain(start..end).collect();
        Ok(TimelinePage {
            header,
            entries: window,
            older: (start > 0).then(|| PageCursor(start.to_string())),
            total_hint: older.is_none().then_some(total),
        })
    }

    fn wi_comment_of(c: &Value) -> Comment {
        let body = text(c, "text");
        let format = c["format"].as_str().unwrap_or_default();
        let markdown = if format == "html" || (format != "markdown" && looks_like_html(&body)) {
            html_to_markdown(&body)
        } else {
            body
        };
        let created_at = text(c, "createdDate");
        let updated = opt_text(c, "modifiedDate");
        let id = c["id"].as_u64().unwrap_or_default().to_string();
        Comment {
            id,
            author: actor_of(c.get("createdBy")),
            body: Markup::new(markdown, FLAVOR),
            edited: is_edited(&created_at, updated.as_deref()),
            created_at,
            updated_at: updated,
            reactions: Vec::new(),
            in_reply_to: None,
            discussion_id: None,
            internal: false,
            can_edit: None,
            can_delete: None,
            web_url: None,
        }
    }

    async fn item_timeline_wi(
        &self,
        project: &str,
        key: &ItemKey,
        older: Option<&PageCursor>,
    ) -> Result<TimelinePage> {
        let (p, _) = split_project(project);
        let item = self
            .conv_json(
                self.get(self.url(
                    Some(p),
                    &format!("wit/workItems/{}", key.number),
                    "fields=System.Title,System.State,System.CreatedBy,System.CreatedDate,\
                     System.Description,System.Tags,System.TeamProject",
                )?),
                "reading an Azure DevOps work item",
            )
            .await?;
        let fields = &item["fields"];
        let state = text(fields, "System.State");
        let spec = crate::spec::spec_for(gitpoulpi_config::ApiFamily::AzureDevops);
        let header = ItemHeader {
            key: key.clone(),
            reference: format!("#{}", key.number),
            title: text(fields, "System.Title"),
            open: !is_closed_state(&state),
            state_label: state.to_ascii_lowercase(),
            author: actor_of(fields.get("System.CreatedBy")),
            body: Markup::new(
                html_to_markdown(&text(fields, "System.Description")),
                FLAVOR,
            ),
            created_at: text(fields, "System.CreatedDate"),
            web_url: (spec.issue_web_url)(&self.root()?, p, &key.number.to_string())
                .unwrap_or_default(),
            labels: text(fields, "System.Tags")
                .split(';')
                .map(str::trim)
                .filter(|t| !t.is_empty())
                .map(str::to_string)
                .collect(),
        };
        let mut token: Option<String> = older.map(|c| c.0.clone());
        let mut next: Option<String> = None;
        let mut entries: Vec<TimelineEntry> = Vec::new();
        let mut hint: Option<u32> = None;
        for page in 0..PAGES_PER_WINDOW {
            let mut query = format!("$top={PAGE}&order=desc");
            if let Some(t) = &token {
                query.push_str(&format!("&continuationToken={}", enc(t)));
            }
            let url = self.conv_url(
                p,
                &format!("wit/workItems/{}/comments", key.number),
                &query,
                self.comments_version(),
            )?;
            let body = self
                .conv_json(self.get(url), "reading the comments of a work item")
                .await?;
            if page == 0 && older.is_none() {
                hint = body["totalCount"].as_u64().map(|n| n as u32);
            }
            for c in body["comments"].as_array().into_iter().flatten() {
                if c["isDeleted"].as_bool() != Some(true) {
                    entries.push(TimelineEntry::Comment(Self::wi_comment_of(c)));
                }
            }
            next = opt_text(&body, "continuationToken");
            match &next {
                Some(t) => token = Some(t.clone()),
                None => break,
            }
        }
        sort_by_time(&mut entries);
        Ok(TimelinePage {
            header,
            entries,
            older: next.map(PageCursor),
            total_hint: hint,
        })
    }

    /// The body as the host takes it: Markdown, or HTML on Server where
    /// work item comments have no Markdown format.
    fn wi_payload(&self, body: &str) -> (Value, &'static str) {
        if self.is_server() {
            (json!({ "text": markdown_to_html(body) }), "")
        } else {
            (json!({ "text": body }), "format=markdown")
        }
    }

    pub(super) async fn conv_post(
        &self,
        project: &str,
        key: &ItemKey,
        body: &str,
        internal: bool,
    ) -> Result<Comment> {
        if internal {
            return Err(invalid(
                "commenting",
                "Azure DevOps has no internal comments",
            ));
        }
        check_body("commenting", body)?;
        match key.kind {
            ItemKind::PullRequest => {
                let url = self.pr_threads_url(project, key.number, "", "").await?;
                let payload = json!({
                    "comments": [{ "parentCommentId": 0, "content": body, "commentType": 1 }],
                    "status": 1,
                });
                let thread = self
                    .conv_json(
                        self.post(url, &payload, None),
                        "commenting on an Azure DevOps pull request",
                    )
                    .await?;
                let id = thread["id"].as_u64().unwrap_or_default();
                let first = thread["comments"]
                    .get(0)
                    .ok_or_else(|| Error::Decode("the thread came back empty".to_string()))?;
                Ok(pr_comment_of(id, first))
            }
            ItemKind::Issue => {
                let (p, _) = split_project(project);
                let (payload, query) = self.wi_payload(body);
                let url = self.conv_url(
                    p,
                    &format!("wit/workItems/{}/comments", key.number),
                    query,
                    self.comments_version(),
                )?;
                let row = self
                    .conv_json(
                        self.post(url, &payload, None),
                        "commenting on an Azure DevOps work item",
                    )
                    .await?;
                Ok(Self::wi_comment_of(&row))
            }
        }
    }

    pub(super) async fn conv_reply(
        &self,
        project: &str,
        key: &ItemKey,
        discussion_id: &str,
        body: &str,
    ) -> Result<Comment> {
        if key.kind != ItemKind::PullRequest {
            return Err(Error::Unsupported(
                "replying in a thread of an Azure DevOps work item".to_string(),
            ));
        }
        check_body("replying", body)?;
        let thread: u64 = discussion_id
            .parse()
            .map_err(|_| invalid("replying", "not a thread of this host"))?;
        let url = self
            .pr_threads_url(project, key.number, &format!("/{thread}/comments"), "")
            .await?;
        // The first comment of a thread is comment 1.
        let payload = json!({ "parentCommentId": 1, "content": body, "commentType": 1 });
        let row = self
            .conv_json(
                self.post(url, &payload, None),
                "replying to an Azure DevOps thread",
            )
            .await?;
        Ok(pr_comment_of(thread, &row))
    }

    /// `"{thread}.{comment}"` as the two numbers.
    fn pr_comment_ids(id: &str) -> Result<(u64, u64)> {
        id.split_once('.')
            .and_then(|(t, c)| Some((t.parse().ok()?, c.parse().ok()?)))
            .ok_or_else(|| invalid("editing a comment", "not a comment of this host"))
    }

    pub(super) async fn conv_edit(
        &self,
        project: &str,
        key: &ItemKey,
        comment_id: &str,
        body: &str,
    ) -> Result<Comment> {
        check_body("editing a comment", body)?;
        match key.kind {
            ItemKind::PullRequest => {
                let (thread, comment) = Self::pr_comment_ids(comment_id)?;
                let url = self
                    .pr_threads_url(
                        project,
                        key.number,
                        &format!("/{thread}/comments/{comment}"),
                        "",
                    )
                    .await?;
                let row = self
                    .conv_json(
                        self.authed(HttpRequest::patch_json(url, &json!({ "content": body }))),
                        "editing an Azure DevOps comment",
                    )
                    .await?;
                Ok(pr_comment_of(thread, &row))
            }
            ItemKind::Issue => {
                let (p, _) = split_project(project);
                let (payload, query) = self.wi_payload(body);
                let url = self.conv_url(
                    p,
                    &format!("wit/workItems/{}/comments/{}", key.number, enc(comment_id)),
                    query,
                    self.comments_version(),
                )?;
                let row = self
                    .conv_json(
                        self.authed(HttpRequest::patch_json(url, &payload)),
                        "editing an Azure DevOps work item comment",
                    )
                    .await?;
                Ok(Self::wi_comment_of(&row))
            }
        }
    }

    pub(super) async fn conv_delete(
        &self,
        project: &str,
        key: &ItemKey,
        comment_id: &str,
    ) -> Result<()> {
        let (url, what) = match key.kind {
            ItemKind::PullRequest => {
                let (thread, comment) = Self::pr_comment_ids(comment_id)?;
                (
                    self.pr_threads_url(
                        project,
                        key.number,
                        &format!("/{thread}/comments/{comment}"),
                        "",
                    )
                    .await?,
                    "deleting an Azure DevOps comment",
                )
            }
            ItemKind::Issue => {
                let (p, _) = split_project(project);
                (
                    self.conv_url(
                        p,
                        &format!("wit/workItems/{}/comments/{}", key.number, enc(comment_id)),
                        "",
                        self.comments_version(),
                    )?,
                    "deleting an Azure DevOps work item comment",
                )
            }
        };
        self.send(self.authed(HttpRequest::delete(url)), what)
            .await?;
        Ok(())
    }

    /// The token's own account, from `connectionData`; unknown when the
    /// server does not say (then nothing is matched to "mine").
    pub(super) async fn conv_identity(&self, project: &str) -> Result<PostingIdentity> {
        let (source, _) = token_source(&self.scopes, &self.integration.id, project);
        let user = self.connection_user().await?;
        let mut identity = PostingIdentity::new(
            user.map(|u| u.login).filter(|l| !l.is_empty()),
            IdentityKind::PersonalToken,
            source,
        );
        if identity.login.is_none() {
            identity.kind = IdentityKind::Unknown;
            identity.display = "your Azure DevOps account".to_string();
        }
        Ok(identity)
    }
}
