//! The conversation of a YouTrack issue: its comments, and posting, editing
//! and deleting them. There are no replies, and the activity stream is not
//! read, so no events.
//!
//! The comment list comes oldest first, `$top` / `$skip` paged; the whole
//! list is read and the newest window cut from it (see
//! [`window_of`]). A comment says whether it is Markdown (`usesMarkdown`);
//! one in the old wiki syntax is shown as plain text rather than guessed at.

use serde_json::{json, Value};

use super::client::YouTrackClient;
use super::map::{issue_url, number_of};
use super::query::epoch_ms_to_iso;
use crate::client::encode_component;
use crate::conversation::time::is_edited;
use crate::conversation::tracker::{check_body, item_ref, opt_text, require_issue, text, window_of};
use crate::conversation::{
    Actor, ActorKind, Comment, IdentityKind, ItemHeader, ItemKey, ItemKind, Markup, MarkupFlavor,
    PageCursor, PostingIdentity, TimelineEntry, TimelineOpts, TimelinePage, TokenSource,
};
use crate::http::HttpRequest;
use crate::{Error, Result};

const COMMENT_FIELDS: &str = "id,text,usesMarkdown,created,updated,deleted,\
author(id,login,fullName)";
const ISSUE_FIELDS: &str = "id,idReadable,summary,description,created,resolved,\
reporter(id,login,fullName),tags(name),customFields(name,value(name))";
const ALL_COMMENTS_CAP: usize = 5000;

fn actor_of(site: &str, user: &Value) -> Actor {
    let Some(login) = opt_text(user, "login") else {
        return Actor::unknown();
    };
    let display = opt_text(user, "fullName").unwrap_or_else(|| login.clone());
    let mut actor = Actor::new(login, display, ActorKind::User);
    actor.web_url = opt_text(user, "id").map(|id| format!("{site}/users/{id}"));
    actor
}

fn iso(v: &Value, key: &str) -> Option<String> {
    v.get(key)
        .and_then(Value::as_u64)
        .map(epoch_ms_to_iso)
        .filter(|s| !s.is_empty())
}

fn comment_of(site: &str, reference: &str, c: &Value) -> Comment {
    let id = text(c, "id");
    let created_at = iso(c, "created").unwrap_or_default();
    let updated_at = iso(c, "updated");
    let flavor = if c["usesMarkdown"].as_bool() == Some(true) {
        MarkupFlavor::YoutrackMd
    } else {
        MarkupFlavor::PlainText
    };
    Comment {
        web_url: Some(format!(
            "{}#focus=Comments-{id}.0-0",
            issue_url(site, reference)
        )),
        author: actor_of(site, &c["author"]),
        body: Markup::new(text(c, "text"), flavor),
        edited: is_edited(&created_at, updated_at.as_deref()),
        created_at,
        updated_at,
        reactions: Vec::new(),
        in_reply_to: None,
        discussion_id: None,
        internal: false,
        can_edit: None,
        can_delete: None,
        id,
    }
}

impl YouTrackClient {
    pub(super) async fn conv_timeline(
        &self,
        project: &str,
        key: &ItemKey,
        older: Option<&PageCursor>,
        _opts: &TimelineOpts,
    ) -> Result<TimelinePage> {
        require_issue("YouTrack", key)?;
        let reference = item_ref(project, key);
        let site = self.site();
        let issue: Value = self
            .get_json(
                &format!("/issues/{}?fields={ISSUE_FIELDS}", encode_component(&reference)),
                "reading the YouTrack issue",
            )
            .await?;
        let rows: Vec<Value> = self
            .paged(
                &format!(
                    "/issues/{}/comments?fields={COMMENT_FIELDS}",
                    encode_component(&reference)
                ),
                "reading the YouTrack comments",
                ALL_COMMENTS_CAP,
            )
            .await?;
        let comments: Vec<Comment> = rows
            .iter()
            .filter(|c| c["deleted"].as_bool() != Some(true))
            .map(|c| comment_of(&site, &reference, c))
            .collect();
        let total = comments.len() as u32;
        let (window, older_cursor) = window_of(comments, older)?;

        let state = issue["customFields"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|f| text(f, "name").eq_ignore_ascii_case("State"))
            .and_then(|f| opt_text(&f["value"], "name"));
        let readable = opt_text(&issue, "idReadable").unwrap_or_else(|| reference.clone());
        let resolved = issue.get("resolved").is_some_and(|r| !r.is_null());
        let header = ItemHeader {
            key: ItemKey {
                kind: ItemKind::Issue,
                number: number_of(&readable),
                id: readable.clone(),
            },
            title: text(&issue, "summary"),
            state_label: state.unwrap_or_else(|| {
                if resolved { "resolved" } else { "open" }.to_string()
            }),
            open: !resolved,
            author: actor_of(&site, &issue["reporter"]),
            body: Markup::new(text(&issue, "description"), MarkupFlavor::YoutrackMd),
            created_at: iso(&issue, "created").unwrap_or_default(),
            web_url: issue_url(&site, &readable),
            labels: issue["tags"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|t| opt_text(t, "name"))
                .collect(),
            reference: readable,
        };
        Ok(TimelinePage {
            header,
            entries: window.into_iter().map(TimelineEntry::Comment).collect(),
            older: older_cursor,
            total_hint: Some(total),
        })
    }

    fn comments_url(&self, project: &str, key: &ItemKey) -> String {
        format!(
            "{}/issues/{}/comments",
            self.api(),
            encode_component(&item_ref(project, key))
        )
    }

    pub(super) async fn conv_post(
        &self,
        project: &str,
        key: &ItemKey,
        body: &str,
        internal: bool,
    ) -> Result<Comment> {
        require_issue("YouTrack", key)?;
        check_body("commenting on a YouTrack issue", body, None)?;
        if internal {
            return Err(Error::Unsupported(
                "internal comments: YouTrack comments are restricted by visibility groups, which the profile does not know".into(),
            ));
        }
        let url = format!("{}?fields={COMMENT_FIELDS}", self.comments_url(project, key));
        let response = self
            .send(
                HttpRequest::post_json(url, &json!({ "text": body, "usesMarkdown": true })),
                "commenting on a YouTrack issue",
            )
            .await?;
        Ok(comment_of(
            &self.site(),
            &item_ref(project, key),
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
        require_issue("YouTrack", key)?;
        check_body("editing a YouTrack comment", body, None)?;
        // YouTrack edits with POST on the comment itself.
        let url = format!(
            "{}/{}?fields={COMMENT_FIELDS}",
            self.comments_url(project, key),
            encode_component(comment_id)
        );
        let response = self
            .send(
                HttpRequest::post_json(url, &json!({ "text": body, "usesMarkdown": true })),
                "editing a YouTrack comment",
            )
            .await?;
        Ok(comment_of(
            &self.site(),
            &item_ref(project, key),
            &response.json::<Value>()?,
        ))
    }

    pub(super) async fn conv_delete(
        &self,
        project: &str,
        key: &ItemKey,
        comment_id: &str,
    ) -> Result<()> {
        require_issue("YouTrack", key)?;
        let url = format!(
            "{}/{}",
            self.comments_url(project, key),
            encode_component(comment_id)
        );
        self.send(HttpRequest::delete(url), "deleting a YouTrack comment")
            .await
            .map(|_| ())
    }

    /// The token's own user (`/users/me`).
    pub(super) async fn conv_identity(&self) -> Result<PostingIdentity> {
        let me: Value = self
            .get_json("/users/me?fields=id,login,fullName", "checking the YouTrack token")
            .await?;
        let actor = actor_of(&self.site(), &me);
        if actor.kind == ActorKind::Unknown {
            return Err(Error::Decode("no user in the YouTrack answer".to_string()));
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
