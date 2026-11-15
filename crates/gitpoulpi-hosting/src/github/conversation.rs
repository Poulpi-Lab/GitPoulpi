//! The conversation of a GitHub issue or pull request: the header, the
//! timeline (comments, reviews, events), code threads, and posting, replying,
//! editing and deleting.
//!
//! Issue comments, reviews and events come from REST
//! (`issues/{n}/timeline`, which serves pull requests too); code threads
//! come from GraphQL, the only API that gives a thread's id and resolved
//! state. A comment's id is the REST comment id; a *code* comment's id is
//! prefixed `review-` so edit and delete address `pulls/comments/{id}`
//! instead of `issues/comments/{id}`.

use serde_json::{json, Value};

use super::GitHubClient;
use crate::client::encode_component;
use crate::conversation::time::is_edited;
use crate::conversation::{
    Actor, ActorKind, CodeThread, Comment, Event, EventKind, IdentityKind, ItemHeader, ItemKey,
    ItemKind, Markup, MarkupFlavor, PageCursor, PostingIdentity, Reaction, ReviewEntry,
    TimelineEntry, TimelineOpts, TimelinePage, TokenSource, WINDOW,
};
use crate::http::{HttpRequest, HttpResponse};
use crate::review::invalid;
use crate::review::model::{DiffSide, ReviewState};
use crate::{Error, Result};

/// Pages of 100 rows that make one window.
const PAGES_PER_WINDOW: u32 = (WINDOW / 100) as u32;
const REVIEW_PREFIX: &str = "review-";

const THREADS_QUERY: &str = "query($owner:String!,$name:String!,$number:Int!,$after:String){\
repository(owner:$owner,name:$name){pullRequest(number:$number){\
reviewThreads(first:100,after:$after){pageInfo{hasNextPage endCursor}\
nodes{id isResolved isOutdated path line originalLine startLine originalStartLine diffSide \
comments(first:100){nodes{databaseId author{login __typename url} body createdAt updatedAt \
lastEditedAt url diffHunk replyTo{databaseId} viewerCanUpdate viewerCanDelete \
reactionGroups{content reactors{totalCount}}}}}}}}}";

const REPLY_MUTATION: &str = "mutation($thread:ID!,$body:String!){\
addPullRequestReviewThreadReply(input:{pullRequestReviewThreadId:$thread,body:$body}){\
comment{databaseId author{login __typename url} body createdAt updatedAt lastEditedAt url \
diffHunk replyTo{databaseId} viewerCanUpdate viewerCanDelete \
reactionGroups{content reactors{totalCount}}}}}";

const VIEWER_QUERY: &str = "query{viewer{login}}";

/// Timeline events that carry nothing worth a line in a conversation.
const SILENT_EVENTS: &[&str] = &[
    "subscribed",
    "unsubscribed",
    "mentioned",
    "line-commented",
    "commit-commented",
    "head_ref_deleted",
    "head_ref_restored",
    "base_ref_changed",
    "referenced",
];

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

/// A REST `user` / `actor` object; `via_app` is `performed_via_github_app`.
fn actor_of(user: Option<&Value>, via_app: bool) -> Actor {
    let Some(user) = user.filter(|u| u.is_object()) else {
        return Actor::unknown();
    };
    let login = text(user, "login");
    let kind = if user.get("type").and_then(Value::as_str) == Some("Bot") || via_app {
        ActorKind::Bot
    } else {
        ActorKind::User
    };
    let mut actor = Actor::new(login.clone(), opt_text(user, "name").unwrap_or(login), kind);
    actor.web_url = opt_text(user, "html_url");
    actor
}

/// REST `reactions`: `{"+1":2,"heart":1,...}` without the zeros.
fn reactions_of(row: &Value) -> Vec<Reaction> {
    const ORDER: [&str; 8] = [
        "+1", "-1", "laugh", "hooray", "confused", "heart", "rocket", "eyes",
    ];
    let Some(r) = row.get("reactions") else {
        return Vec::new();
    };
    ORDER
        .iter()
        .filter_map(|name| {
            let count = r.get(*name).and_then(Value::as_u64).unwrap_or(0) as u32;
            (count > 0).then(|| Reaction {
                name: (*name).to_string(),
                count,
            })
        })
        .collect()
}

/// A REST issue comment, a timeline `commented` event or a review comment.
/// `review`: a code comment (its id gets the `review-` prefix).
fn comment_of(row: &Value, review: bool) -> Comment {
    let user = row.get("user").or_else(|| row.get("actor"));
    let via_app = row
        .get("performed_via_github_app")
        .is_some_and(|a| !a.is_null());
    let id = row.get("id").and_then(Value::as_u64).unwrap_or_default();
    let created_at = text(row, "created_at");
    let updated_at = opt_text(row, "updated_at");
    Comment {
        id: if review {
            format!("{REVIEW_PREFIX}{id}")
        } else {
            id.to_string()
        },
        author: actor_of(user, via_app),
        body: Markup::new(text(row, "body"), MarkupFlavor::Gfm),
        edited: is_edited(&created_at, updated_at.as_deref()),
        created_at,
        updated_at,
        reactions: reactions_of(row),
        in_reply_to: row
            .get("in_reply_to_id")
            .and_then(Value::as_u64)
            .map(|i| format!("{REVIEW_PREFIX}{i}")),
        discussion_id: None,
        internal: false,
        can_edit: None,
        can_delete: None,
        web_url: opt_text(row, "html_url"),
    }
}

fn graphql_reactions(groups: &Value) -> Vec<Reaction> {
    groups
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|g| {
            let count = g["reactors"]["totalCount"].as_u64().unwrap_or(0) as u32;
            let name = match g["content"].as_str()? {
                "THUMBS_UP" => "+1",
                "THUMBS_DOWN" => "-1",
                "LAUGH" => "laugh",
                "HOORAY" => "hooray",
                "CONFUSED" => "confused",
                "HEART" => "heart",
                "ROCKET" => "rocket",
                "EYES" => "eyes",
                _ => return None,
            };
            (count > 0).then(|| Reaction {
                name: name.to_string(),
                count,
            })
        })
        .collect()
}

/// A GraphQL `PullRequestReviewComment`.
fn gql_comment_of(c: &Value, thread_id: Option<&str>) -> Comment {
    let author = &c["author"];
    let actor = if author.is_object() {
        let login = text(author, "login");
        let kind = if author["__typename"].as_str() == Some("Bot") {
            ActorKind::Bot
        } else {
            ActorKind::User
        };
        let mut a = Actor::new(login.clone(), login, kind);
        a.web_url = opt_text(author, "url");
        a
    } else {
        Actor::unknown()
    };
    let created_at = text(c, "createdAt");
    let updated_at = opt_text(c, "updatedAt");
    let edited = c["lastEditedAt"].is_string() || is_edited(&created_at, updated_at.as_deref());
    Comment {
        id: format!(
            "{REVIEW_PREFIX}{}",
            c["databaseId"].as_u64().unwrap_or_default()
        ),
        author: actor,
        body: Markup::new(text(c, "body"), MarkupFlavor::Gfm),
        created_at,
        updated_at,
        edited,
        reactions: graphql_reactions(&c["reactionGroups"]),
        in_reply_to: c["replyTo"]["databaseId"]
            .as_u64()
            .map(|i| format!("{REVIEW_PREFIX}{i}")),
        discussion_id: thread_id.map(str::to_string),
        internal: false,
        can_edit: c["viewerCanUpdate"].as_bool(),
        can_delete: c["viewerCanDelete"].as_bool(),
        web_url: opt_text(c, "url"),
    }
}

fn thread_of(t: &Value) -> CodeThread {
    let id = text(t, "id");
    let outdated = t["isOutdated"].as_bool().unwrap_or(false);
    let num = |k: &str| t[k].as_u64().map(|n| n as u32);
    let line = if outdated { None } else { num("line") };
    let start_line = if outdated {
        num("originalStartLine")
    } else {
        num("startLine").or_else(|| num("originalStartLine"))
    };
    let nodes = t["comments"]["nodes"].as_array();
    let comments: Vec<Comment> = nodes
        .into_iter()
        .flatten()
        .map(|c| gql_comment_of(c, Some(&id)))
        .collect();
    let diff_hunk = nodes
        .and_then(|n| n.first())
        .and_then(|c| opt_text(c, "diffHunk"));
    CodeThread {
        id,
        path: text(t, "path"),
        line,
        original_line: num("originalLine").or_else(|| num("line")),
        side: if t["diffSide"].as_str() == Some("LEFT") {
            DiffSide::Old
        } else {
            DiffSide::New
        },
        start_line,
        resolved: t["isResolved"].as_bool().unwrap_or(false),
        outdated,
        resolvable: true,
        diff_hunk,
        comments,
    }
}

fn label_name(v: &Value) -> Vec<String> {
    v.get("label")
        .and_then(|l| l.get("name"))
        .and_then(Value::as_str)
        .map(|n| vec![n.to_string()])
        .unwrap_or_default()
}

fn login_of(v: &Value, key: &str) -> Vec<String> {
    v.get(key)
        .and_then(|u| u.get("login"))
        .and_then(Value::as_str)
        .map(|l| vec![l.to_string()])
        .unwrap_or_default()
}

/// One timeline row that is not a comment or a review, as an event.
fn event_of(row: &Value) -> Option<Event> {
    let name = row.get("event")?.as_str()?;
    let actor = row
        .get("actor")
        .filter(|a| a.is_object())
        .map(|a| actor_of(Some(a), false));
    let at = opt_text(row, "created_at").unwrap_or_default();
    let id = row
        .get("id")
        .map(|i| i.to_string().trim_matches('"').to_string())
        .unwrap_or_default();
    let kind = match name {
        "closed" => EventKind::Closed,
        "reopened" => EventKind::Reopened,
        "merged" => EventKind::Merged,
        "labeled" => EventKind::Labeled {
            added: label_name(row),
            removed: Vec::new(),
        },
        "unlabeled" => EventKind::Labeled {
            added: Vec::new(),
            removed: label_name(row),
        },
        "assigned" => EventKind::Assigned {
            added: login_of(row, "assignee"),
            removed: Vec::new(),
        },
        "unassigned" => EventKind::Assigned {
            added: Vec::new(),
            removed: login_of(row, "assignee"),
        },
        "renamed" => EventKind::Renamed {
            from: text(&row["rename"], "from"),
            to: text(&row["rename"], "to"),
        },
        // A force push: no commit count to give.
        "head_ref_force_pushed" => EventKind::CommitsPushed {
            count: 0,
            head: opt_text(row, "commit_id"),
        },
        "review_requested" => EventKind::ReviewRequested {
            who: row["requested_reviewer"]["login"]
                .as_str()
                .or_else(|| row["requested_team"]["name"].as_str())
                .unwrap_or_default()
                .to_string(),
        },
        "cross-referenced" => {
            let issue = &row["source"]["issue"];
            let repo = issue["repository"]["full_name"].as_str().unwrap_or("");
            let number = issue["number"].as_u64().unwrap_or_default();
            EventKind::CrossReferenced {
                from: if repo.is_empty() {
                    format!("#{number}")
                } else {
                    format!("{repo}#{number}")
                },
                url: opt_text(issue, "html_url"),
            }
        }
        other if SILENT_EVENTS.contains(&other) => return None,
        other => EventKind::Other {
            text: other.replace(['_', '-'], " "),
        },
    };
    Some(Event {
        id,
        actor,
        at,
        kind,
    })
}

fn review_state(text: &str) -> Option<ReviewState> {
    match text.to_ascii_lowercase().as_str() {
        "approved" => Some(ReviewState::Approved),
        "changes_requested" => Some(ReviewState::ChangesRequested),
        "commented" => Some(ReviewState::Commented),
        "dismissed" => Some(ReviewState::Dismissed),
        // PENDING is somebody's unsent draft.
        _ => None,
    }
}

/// The timeline rows of one window, in order, as entries.
fn entries_of(rows: &[Value], opts: &TimelineOpts) -> Vec<TimelineEntry> {
    let mut out: Vec<TimelineEntry> = Vec::new();
    let mut last_was_commit = false;
    for row in rows {
        let name = row["event"].as_str().unwrap_or_default();
        let mut is_commit = false;
        match name {
            "commented" => out.push(TimelineEntry::Comment(comment_of(row, false))),
            "reviewed" => {
                let body = text(row, "body");
                let Some(state) = review_state(&text(row, "state")) else {
                    continue;
                };
                // A summary-less "commented" review only carries code
                // comments, which are threads already.
                if state == ReviewState::Commented && body.trim().is_empty() {
                    continue;
                }
                out.push(TimelineEntry::Review(ReviewEntry {
                    id: row["id"].as_u64().unwrap_or_default().to_string(),
                    author: actor_of(row.get("user"), false),
                    state,
                    body: Markup::new(body, MarkupFlavor::Gfm),
                    submitted_at: text(row, "submitted_at"),
                }));
            }
            "committed" => {
                if !opts.events {
                    continue;
                }
                is_commit = true;
                let sha = opt_text(row, "sha");
                let merged_into_last = last_was_commit
                    && match out.last_mut() {
                        Some(TimelineEntry::Event(Event {
                            kind: EventKind::CommitsPushed { count, head },
                            ..
                        })) => {
                            *count += 1;
                            head.clone_from(&sha);
                            true
                        }
                        _ => false,
                    };
                if !merged_into_last {
                    out.push(TimelineEntry::Event(Event {
                        id: sha.clone().unwrap_or_default(),
                        actor: None,
                        at: row["author"]["date"]
                            .as_str()
                            .or_else(|| row["committer"]["date"].as_str())
                            .unwrap_or_default()
                            .to_string(),
                        kind: EventKind::CommitsPushed {
                            count: 1,
                            head: sha,
                        },
                    }));
                }
            }
            _ if opts.events => {
                if let Some(event) = event_of(row) {
                    out.push(TimelineEntry::Event(event));
                }
            }
            _ => {}
        }
        last_was_commit = is_commit;
    }
    out
}

/// `rel="last"` page number of a `Link` header.
fn last_page(response: &HttpResponse) -> Option<u32> {
    let link = response
        .headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("link"))
        .map(|(_, v)| v.as_str())?;
    link.split(',').find_map(|part| {
        if !part.contains("rel=\"last\"") {
            return None;
        }
        let url = part.split('>').next()?.trim().trim_start_matches('<');
        let query = url.split_once('?')?.1;
        query
            .split('&')
            .find_map(|kv| kv.strip_prefix("page=")?.parse().ok())
    })
}

/// `<page>@<time>`: the next page to load, and the time the newer window
/// starts at (code threads opened from then on belong to the newer one).
fn parse_cursor(cursor: &PageCursor) -> Option<(u32, String)> {
    let (page, before) = cursor.0.split_once('@')?;
    Some((page.parse().ok().filter(|p| *p >= 1)?, before.to_string()))
}

impl GitHubClient {
    fn issue_api(&self, project: &str, rest: &str) -> String {
        self.repo_url(project, &format!("issues/{rest}"))
    }

    async fn conv_header(&self, project: &str, key: &ItemKey) -> Result<ItemHeader> {
        let url = self.issue_api(project, &key.number.to_string());
        let row: Value = self
            .get_json(url, "reading a GitHub issue or pull request")
            .await?;
        let merged = row["pull_request"]["merged_at"].is_string();
        let open = row["state"].as_str() == Some("open");
        let state_label = if merged {
            "merged"
        } else if open {
            "open"
        } else {
            "closed"
        };
        Ok(ItemHeader {
            key: key.clone(),
            reference: format!("#{}", key.number),
            title: text(&row, "title"),
            state_label: state_label.to_string(),
            open,
            author: actor_of(row.get("user"), false),
            body: Markup::new(text(&row, "body"), MarkupFlavor::Gfm),
            created_at: text(&row, "created_at"),
            web_url: text(&row, "html_url"),
            labels: row["labels"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|l| l["name"].as_str().or_else(|| l.as_str()))
                .map(str::to_string)
                .collect(),
        })
    }

    async fn timeline_page(
        &self,
        project: &str,
        key: &ItemKey,
        page: u32,
    ) -> Result<(Vec<Value>, HttpResponse)> {
        let url = self.issue_api(
            project,
            &format!("{}/timeline?per_page=100&page={page}", key.number),
        );
        let response = self.transport.send(self.get(url)).await?;
        response.error_for_status("reading the timeline of a GitHub issue or pull request")?;
        let rows: Vec<Value> = response.json()?;
        Ok((rows, response))
    }

    /// The raw rows of the newest window (pages walked back from the last)
    /// or of the window ending at page `cursor`, plus the next older page and
    /// a count hint.
    async fn timeline_window(
        &self,
        project: &str,
        key: &ItemKey,
        cursor: Option<u32>,
    ) -> Result<(Vec<Value>, Option<u32>, Option<u32>)> {
        let mut first: Option<(Vec<Value>, HttpResponse)> = None;
        let (high, last_known) = match cursor {
            Some(c) => (c, None),
            None => {
                let page_one = self.timeline_page(project, key, 1).await?;
                let last = last_page(&page_one.1).unwrap_or(1);
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
                    self.timeline_page(project, key, page).await?.0
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

    /// Every code thread of a pull request (GraphQL, paged by 100 threads).
    async fn conv_threads(&self, project: &str, number: u64) -> Result<Vec<CodeThread>> {
        let (owner, name) = project
            .split_once('/')
            .ok_or_else(|| invalid("reading review threads", "expected owner/name"))?;
        let mut out = Vec::new();
        let mut after = Value::Null;
        for _ in 0..crate::client::MAX_PAGES {
            let data = self
                .graphql(
                    "reading the review threads of a GitHub pull request",
                    THREADS_QUERY,
                    json!({ "owner": owner, "name": name, "number": number, "after": after }),
                )
                .await?;
            let threads = &data["repository"]["pullRequest"]["reviewThreads"];
            if threads.is_null() {
                return Err(Error::Decode("no reviewThreads in the answer".to_string()));
            }
            out.extend(
                threads["nodes"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(thread_of),
            );
            if threads["pageInfo"]["hasNextPage"].as_bool() == Some(true) {
                after = threads["pageInfo"]["endCursor"].clone();
            } else {
                break;
            }
        }
        Ok(out)
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
        let header = self.conv_header(project, key).await?;
        let (rows, next, total_hint) = self
            .timeline_window(project, key, cursor.as_ref().map(|c| c.0))
            .await?;
        let mut entries = entries_of(&rows, opts);
        // The window's own start: threads opened before it belong to an older
        // window, unless this one reaches the start of the conversation.
        let window_start = rows
            .iter()
            .find_map(|r| {
                opt_text(r, "created_at")
                    .or_else(|| opt_text(r, "submitted_at"))
                    .or_else(|| r["author"]["date"].as_str().map(str::to_string))
            })
            .unwrap_or_default();
        if key.kind == ItemKind::PullRequest {
            let lower = if next.is_some() {
                window_start.clone()
            } else {
                String::new()
            };
            let upper = cursor.as_ref().map(|c| c.1.clone());
            for t in self.conv_threads(project, key.number).await? {
                let opened = t.comments.first().map_or("", |c| c.created_at.as_str());
                if opened >= lower.as_str() && upper.as_deref().is_none_or(|u| opened < u) {
                    entries.push(TimelineEntry::Thread(t));
                }
            }
            entries.sort_by(|a, b| a.at().cmp(b.at()));
        }
        Ok(TimelinePage {
            header,
            entries,
            older: next.map(|p| PageCursor(format!("{p}@{window_start}"))),
            total_hint,
        })
    }

    /// Remembers who an App bot posted as, from the author of its first post.
    fn learn_from_post(&self, project: &str, comment: &Comment) {
        let id = &self.integration.id;
        if comment.author.kind == ActorKind::Bot
            && !comment.author.login.is_empty()
            && (crate::scope::is_app_installation_token(&self.token)
                || self.scopes.acts_as_bot(id, project))
            && self.scopes.post_as(id, project).is_none()
            && self.scopes.learned_login(id, project).is_none()
        {
            self.scopes.learn_login(id, project, &comment.author.login);
        }
    }

    pub(super) async fn conv_post(
        &self,
        project: &str,
        key: &ItemKey,
        body: &str,
        internal: bool,
    ) -> Result<Comment> {
        let (what, label) = match key.kind {
            ItemKind::PullRequest => (
                "commenting on a GitHub pull request",
                "commenting on a pull request",
            ),
            ItemKind::Issue => ("commenting on a GitHub issue", "commenting on an issue"),
        };
        if internal {
            return Err(invalid(label, "GitHub has no internal comments"));
        }
        check_body(label, body)?;
        let url = self.issue_api(project, &format!("{}/comments", key.number));
        let response = self
            .transport
            .send(self.post(url, &json!({ "body": body })))
            .await?;
        response.error_for_status(what)?;
        let comment = comment_of(&response.json::<Value>()?, false);
        self.learn_from_post(project, &comment);
        Ok(comment)
    }

    pub(super) async fn conv_reply(
        &self,
        _project: &str,
        key: &ItemKey,
        discussion_id: &str,
        body: &str,
    ) -> Result<Comment> {
        if key.kind != ItemKind::PullRequest {
            return Err(Error::Unsupported(
                "replying in a thread of a GitHub issue".to_string(),
            ));
        }
        check_body("replying to a thread", body)?;
        let data = self
            .graphql(
                "replying to a GitHub review thread",
                REPLY_MUTATION,
                json!({ "thread": discussion_id, "body": body }),
            )
            .await?;
        Ok(gql_comment_of(
            &data["addPullRequestReviewThreadReply"]["comment"],
            Some(discussion_id),
        ))
    }

    /// `issues/comments/{id}` for a conversation comment, `pulls/comments/{id}`
    /// for a code comment (`review-` prefix).
    fn comment_url(&self, project: &str, id: &str) -> String {
        match id.strip_prefix(REVIEW_PREFIX) {
            Some(raw) => self.repo_url(
                project,
                &format!("pulls/comments/{}", encode_component(raw)),
            ),
            None => self.repo_url(
                project,
                &format!("issues/comments/{}", encode_component(id)),
            ),
        }
    }

    pub(super) async fn conv_edit(
        &self,
        project: &str,
        _key: &ItemKey,
        comment_id: &str,
        body: &str,
    ) -> Result<Comment> {
        check_body("editing a comment", body)?;
        let url = self.comment_url(project, comment_id);
        let response = self
            .transport
            .send(self.authed(HttpRequest::patch_json(url, &json!({ "body": body }))))
            .await?;
        response.error_for_status("editing a GitHub comment")?;
        Ok(comment_of(
            &response.json::<Value>()?,
            comment_id.starts_with(REVIEW_PREFIX),
        ))
    }

    pub(super) async fn conv_delete(
        &self,
        project: &str,
        _key: &ItemKey,
        comment_id: &str,
    ) -> Result<()> {
        let response = self
            .transport
            .send(self.delete(self.comment_url(project, comment_id)))
            .await?;
        response.error_for_status("deleting a GitHub comment")
    }

    /// Who a post is made as. A personal token is its `/user` login; an App
    /// bot (an installation token, or `/user` answering 403 / 404) is the
    /// configured "Post as" name, else the login learned from an earlier
    /// post, else GraphQL `viewer`, else unknown (shown as "the GitHub App
    /// bot" until the first post teaches it).
    pub(super) async fn conv_identity(&self, project: &str) -> Result<PostingIdentity> {
        let id = &self.integration.id;
        let by_command = self.scopes.acts_as_bot(id, project);
        let source = if by_command {
            TokenSource::RepoTokenCommand
        } else {
            TokenSource::Profile { id: id.clone() }
        };
        // The cached login is the profile's own token's; a token command's
        // token has to be asked.
        let user = if by_command {
            self.fetch_user().await.map(|u| u.0.login)
        } else {
            self.login().await
        };
        match user {
            Ok(login) => Ok(PostingIdentity::new(
                Some(login),
                IdentityKind::PersonalToken,
                source,
            )),
            Err(Error::BotToken) => {
                let login = match self
                    .scopes
                    .post_as(id, project)
                    .or_else(|| self.scopes.learned_login(id, project))
                {
                    Some(login) => Some(login),
                    None => self.viewer_login().await,
                };
                Ok(PostingIdentity::new(login, IdentityKind::AppBot, source))
            }
            Err(e) => Err(e),
        }
    }

    /// GraphQL `viewer { login }`: what an installation token may or may not
    /// answer; any failure is "unknown".
    async fn viewer_login(&self) -> Option<String> {
        let data = self
            .graphql("reading the GitHub viewer", VIEWER_QUERY, json!({}))
            .await
            .ok()?;
        let login = data["viewer"]["login"].as_str()?.trim().to_string();
        (!login.is_empty()).then_some(login)
    }
}

fn check_body(what: &str, body: &str) -> Result<()> {
    if body.trim().is_empty() {
        return Err(invalid(what, "the comment is empty"));
    }
    Ok(())
}
