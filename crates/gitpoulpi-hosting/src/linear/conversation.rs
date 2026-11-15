//! The conversation of a Linear issue: its comments (with one level of
//! replies through `parentId`), the issue history as events, and posting,
//! replying, editing and deleting.
//!
//! Comments are Markdown. `comments(last: 100, before: $cursor)` answers the
//! newest comments oldest first, so a window is up to three of those
//! queries, each continued from the previous one's `startCursor`; the
//! history comes with the first query and is cut to the window's time span.

use serde_json::{json, Value};

use super::client::LinearClient;
use super::map::CLOSED_TYPES;
use crate::conversation::time::is_edited;
use crate::conversation::time::iso_secs;
use crate::conversation::tracker::{
    check_body, decode_cursor, encode_cursor, item_ref, oldest_secs, opt_text, require_issue,
    span_of, text,
};
use crate::conversation::{
    Actor, ActorKind, Comment, Event, EventKind, IdentityKind, ItemHeader, ItemKey, ItemKind,
    Markup, MarkupFlavor, PageCursor, PostingIdentity, TimelineEntry, TimelineOpts, TimelinePage,
    TokenSource, WINDOW,
};
use crate::{Error, Result};

const PAGE: usize = 100;

const USER: &str = "id name displayName url";
const COMMENT: &str = "id body createdAt updatedAt editedAt url \
    user { id name displayName url } botActor { id name } parent { id }";

fn first_query() -> String {
    format!(
        "query IssueConversation($id: String!, $before: String) {{ issue(id: $id) {{ \
            id identifier number title url description createdAt \
            state {{ name type }} creator {{ {USER} }} labels {{ nodes {{ name }} }} \
            comments(last: {PAGE}, before: $before) {{ nodes {{ {COMMENT} }} \
                pageInfo {{ hasPreviousPage startCursor }} }} \
            history(last: {PAGE}) {{ nodes {{ id createdAt actor {{ {USER} }} \
                fromState {{ name }} toState {{ name }} \
                addedLabels {{ name }} removedLabels {{ name }} \
                fromAssignee {{ {USER} }} toAssignee {{ {USER} }} \
                fromTitle toTitle }} }} }} }}"
    )
}

fn more_query() -> String {
    format!(
        "query IssueComments($id: String!, $before: String) {{ issue(id: $id) {{ \
            comments(last: {PAGE}, before: $before) {{ nodes {{ {COMMENT} }} \
                pageInfo {{ hasPreviousPage startCursor }} }} }} }}"
    )
}

const COMMENT_CREATE: &str = "mutation CommentCreate($input: CommentCreateInput!) { \
    commentCreate(input: $input) { success comment { id body createdAt updatedAt editedAt url \
        user { id name displayName url } botActor { id name } parent { id } } } }";

const COMMENT_UPDATE: &str = "mutation CommentUpdate($id: String!, $input: CommentUpdateInput!) { \
    commentUpdate(id: $id, input: $input) { success comment { id body createdAt updatedAt \
        editedAt url user { id name displayName url } botActor { id name } parent { id } } } }";

const COMMENT_DELETE: &str = "mutation CommentDelete($id: String!) { \
    commentDelete(id: $id) { success } }";

const VIEWER: &str = "query Viewer { viewer { id name displayName } }";

/// A Linear user: the unique `displayName` is the login.
fn user_actor(u: &Value) -> Option<Actor> {
    if !u.is_object() {
        return None;
    }
    let handle = opt_text(u, "displayName").or_else(|| opt_text(u, "name"))?;
    let mut actor = Actor::new(
        handle.clone(),
        opt_text(u, "name").unwrap_or(handle),
        ActorKind::User,
    );
    actor.web_url = opt_text(u, "url");
    Some(actor)
}

/// The author of a comment: a user, else the app (`botActor`).
fn comment_actor(c: &Value) -> Actor {
    if let Some(a) = user_actor(&c["user"]) {
        return a;
    }
    match opt_text(&c["botActor"], "name") {
        Some(name) => Actor::new(name.clone(), name, ActorKind::Bot),
        None => Actor::unknown(),
    }
}

fn comment_of(c: &Value) -> Comment {
    let id = text(c, "id");
    let parent = opt_text(&c["parent"], "id");
    let created_at = text(c, "createdAt");
    let updated_at = opt_text(c, "updatedAt");
    Comment {
        // Replies go under a top-level comment: its id is the discussion.
        discussion_id: Some(parent.clone().unwrap_or_else(|| id.clone())),
        in_reply_to: parent,
        author: comment_actor(c),
        body: Markup::new(text(c, "body"), MarkupFlavor::Gfm),
        edited: c["editedAt"].as_str().is_some_and(|s| !s.is_empty())
            || is_edited(&created_at, updated_at.as_deref()),
        created_at,
        updated_at,
        reactions: Vec::new(),
        internal: false,
        can_edit: None,
        can_delete: None,
        web_url: opt_text(c, "url"),
        id,
    }
}

fn names(v: &Value) -> Vec<String> {
    v.as_array()
        .into_iter()
        .flatten()
        .filter_map(|n| opt_text(n, "name"))
        .collect()
}

/// The events one history node stands for (a node can change several things).
pub(super) fn history_events(node: &Value) -> Vec<Event> {
    let id = text(node, "id");
    let at = text(node, "createdAt");
    let actor = user_actor(&node["actor"]);
    let mut kinds = Vec::new();
    if let (Some(from), Some(to)) = (
        opt_text(&node["fromState"], "name"),
        opt_text(&node["toState"], "name"),
    ) {
        kinds.push(EventKind::StatusChanged { from, to });
    }
    let (added, removed) = (names(&node["addedLabels"]), names(&node["removedLabels"]));
    if !added.is_empty() || !removed.is_empty() {
        kinds.push(EventKind::Labeled { added, removed });
    }
    let who = |v: &Value| {
        user_actor(v)
            .map(|a| a.login)
            .into_iter()
            .collect::<Vec<_>>()
    };
    let (added, removed) = (who(&node["toAssignee"]), who(&node["fromAssignee"]));
    if !added.is_empty() || !removed.is_empty() {
        kinds.push(EventKind::Assigned { added, removed });
    }
    if let (Some(from), Some(to)) = (opt_text(node, "fromTitle"), opt_text(node, "toTitle")) {
        kinds.push(EventKind::Renamed { from, to });
    }
    kinds
        .into_iter()
        .enumerate()
        .map(|(i, kind)| Event {
            id: format!("{id}.{i}"),
            actor: actor.clone(),
            at: at.clone(),
            kind,
        })
        .collect()
}

impl LinearClient {
    pub(super) async fn conv_timeline(
        &self,
        project: &str,
        key: &ItemKey,
        older: Option<&PageCursor>,
        opts: &TimelineOpts,
    ) -> Result<TimelinePage> {
        require_issue("Linear", key)?;
        let (mut before, upper) = match older {
            None => (None, None),
            Some(c) => {
                let (p, u) = decode_cursor(c).ok_or_else(|| Error::Api {
                    what: "reading a conversation".into(),
                    status: 422,
                    detail: "bad page cursor".into(),
                })?;
                (Some(p), u)
            }
        };
        let id = item_ref(project, key);
        let mut comments: Vec<Comment> = Vec::new();
        let mut issue = Value::Null;
        let mut has_older = false;
        for round in 0..(WINDOW / PAGE) {
            let query = if round == 0 {
                first_query()
            } else {
                more_query()
            };
            let data = self
                .graphql(
                    "reading the Linear issue",
                    &query,
                    json!({ "id": id, "before": before }),
                )
                .await?;
            let node = data["issue"].clone();
            if node.is_null() {
                return Err(Error::Api {
                    what: "reading the Linear issue".into(),
                    status: 404,
                    detail: format!("no issue {id}"),
                });
            }
            let page = node["comments"].clone();
            let mut rows: Vec<Comment> = page["nodes"]
                .as_array()
                .into_iter()
                .flatten()
                .map(comment_of)
                .collect();
            // Older rows go in front of the ones already read.
            rows.append(&mut comments);
            comments = rows;
            if round == 0 {
                issue = node;
            }
            has_older = page["pageInfo"]["hasPreviousPage"].as_bool() == Some(true);
            before = opt_text(&page["pageInfo"], "startCursor");
            if !has_older || before.is_none() {
                has_older = false;
                break;
            }
        }
        let created =
            |c: &Vec<Comment>| -> Vec<String> { c.iter().map(|c| c.created_at.clone()).collect() };
        let times = created(&comments);
        let span = span_of(times.iter().map(String::as_str), has_older, upper);
        let older_cursor = has_older.then(|| {
            encode_cursor(
                before.as_deref().unwrap_or_default(),
                oldest_secs(times.iter().map(String::as_str)),
            )
        });
        let mut entries: Vec<TimelineEntry> =
            comments.into_iter().map(TimelineEntry::Comment).collect();
        if opts.events {
            for node in issue["history"]["nodes"].as_array().into_iter().flatten() {
                for event in history_events(node) {
                    if span.holds(&event.at) {
                        entries.push(TimelineEntry::Event(event));
                    }
                }
            }
            entries.sort_by_key(|e| iso_secs(e.at()).unwrap_or(0));
        }

        let state_type = text(&issue["state"], "type");
        let identifier = opt_text(&issue, "identifier").unwrap_or_else(|| id.clone());
        let header = ItemHeader {
            key: ItemKey {
                kind: ItemKind::Issue,
                number: issue["number"].as_u64().unwrap_or(key.number),
                id: identifier.clone(),
            },
            reference: identifier,
            title: text(&issue, "title"),
            state_label: text(&issue["state"], "name"),
            open: !CLOSED_TYPES.contains(&state_type.as_str()),
            author: user_actor(&issue["creator"]).unwrap_or_else(Actor::unknown),
            body: Markup::new(text(&issue, "description"), MarkupFlavor::Gfm),
            created_at: text(&issue, "createdAt"),
            web_url: text(&issue, "url"),
            labels: names(&issue["labels"]["nodes"]),
        };
        Ok(TimelinePage {
            header,
            entries,
            older: older_cursor,
            total_hint: None,
        })
    }

    async fn conv_mutation(
        &self,
        what: &str,
        query: &str,
        vars: Value,
        field: &str,
    ) -> Result<Value> {
        let data = self.graphql(what, query, vars).await?;
        let out = data[field].clone();
        if out["success"].as_bool() != Some(true) {
            return Err(Error::Api {
                what: what.to_string(),
                status: 422,
                detail: "Linear did not accept the change".into(),
            });
        }
        Ok(out)
    }

    pub(super) async fn conv_post(
        &self,
        project: &str,
        key: &ItemKey,
        body: &str,
        parent: Option<&str>,
    ) -> Result<Comment> {
        require_issue("Linear", key)?;
        check_body("commenting on a Linear issue", body, None)?;
        let mut input = json!({ "issueId": item_ref(project, key), "body": body });
        if let Some(p) = parent {
            input["parentId"] = json!(p);
        }
        let out = self
            .conv_mutation(
                "commenting on a Linear issue",
                COMMENT_CREATE,
                json!({ "input": input }),
                "commentCreate",
            )
            .await?;
        Ok(comment_of(&out["comment"]))
    }

    pub(super) async fn conv_edit(
        &self,
        key: &ItemKey,
        comment_id: &str,
        body: &str,
    ) -> Result<Comment> {
        require_issue("Linear", key)?;
        check_body("editing a Linear comment", body, None)?;
        let out = self
            .conv_mutation(
                "editing a Linear comment",
                COMMENT_UPDATE,
                json!({ "id": comment_id, "input": { "body": body } }),
                "commentUpdate",
            )
            .await?;
        Ok(comment_of(&out["comment"]))
    }

    pub(super) async fn conv_delete(&self, key: &ItemKey, comment_id: &str) -> Result<()> {
        require_issue("Linear", key)?;
        self.conv_mutation(
            "deleting a Linear comment",
            COMMENT_DELETE,
            json!({ "id": comment_id }),
            "commentDelete",
        )
        .await
        .map(|_| ())
    }

    /// The API key's own user (`viewer`); Linear has no bot tokens that
    /// answer differently.
    pub(super) async fn conv_identity(&self) -> Result<PostingIdentity> {
        let data = self
            .graphql("checking the Linear token", VIEWER, json!({}))
            .await?;
        let actor = user_actor(&data["viewer"])
            .ok_or_else(|| Error::Decode("no viewer in the Linear answer".to_string()))?;
        Ok(PostingIdentity::new(
            Some(actor.login),
            IdentityKind::PersonalToken,
            TokenSource::Profile {
                id: self.integration.id.clone(),
            },
        ))
    }
}
