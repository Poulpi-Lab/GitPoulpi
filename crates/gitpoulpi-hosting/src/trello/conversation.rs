//! The conversation of a Trello card: its comments, the moves between lists
//! and its creation as events, and posting, editing and deleting comments.
//!
//! A card is addressed by its id (`ItemKey::id`; the card number alone does
//! not name it across boards). Comments and history are one `actions`
//! listing, newest first, up to Trello's cap of 1000 per call; it is
//! reversed and the newest window cut from it locally. Trello has no
//! replies and no internal comments. Post and edit send the text as a JSON
//! body; the key and token stay in the header like every other call.

use serde_json::{json, Value};

use super::client::TrelloClient;
use crate::client::encode_component;
use crate::conversation::tracker::{check_body, opt_text, require_issue, text, window_of};
use crate::conversation::{
    Actor, ActorKind, Comment, Event, EventKind, IdentityKind, ItemHeader, ItemKey, ItemKind,
    Markup, MarkupFlavor, PageCursor, PostingIdentity, TimelineEntry, TimelineOpts, TimelinePage,
    TokenSource,
};
use crate::http::HttpRequest;
use crate::review::invalid;
use crate::{Error, Result};

const MAX_BODY: usize = 16_384;
const CARD_FIELDS: &str = "name,desc,url,shortUrl,idShort,closed,idList,labels";
const ACTIONS_LIMIT: usize = 1000;

fn actor_of(member: &Value) -> Actor {
    let Some(login) = opt_text(member, "username") else {
        return Actor::unknown();
    };
    let display = opt_text(member, "fullName").unwrap_or_else(|| login.clone());
    let mut actor = Actor::new(login.clone(), display, ActorKind::User);
    actor.web_url = Some(format!("https://trello.com/{login}"));
    actor
}

fn comment_of(card_url: &str, a: &Value) -> Comment {
    let id = text(a, "id");
    let created_at = text(a, "date");
    let updated_at = opt_text(&a["data"], "dateLastEdited");
    Comment {
        web_url: (!card_url.is_empty()).then(|| format!("{card_url}#comment-{id}")),
        author: actor_of(&a["memberCreator"]),
        body: Markup::new(text(&a["data"], "text"), MarkupFlavor::TrelloMd),
        edited: updated_at.is_some(),
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

fn event_of(a: &Value) -> Option<Event> {
    let data = &a["data"];
    let kind = match text(a, "type").as_str() {
        "createCard" | "copyCard" => EventKind::Opened,
        "updateCard" => {
            if let (Some(from), Some(to)) = (
                opt_text(&data["listBefore"], "name"),
                opt_text(&data["listAfter"], "name"),
            ) {
                EventKind::StatusChanged { from, to }
            } else if let Some(old) = data["old"]["closed"].as_bool() {
                // `old.closed` is what it was: archived before means reopened.
                if old {
                    EventKind::Reopened
                } else {
                    EventKind::Closed
                }
            } else {
                return None;
            }
        }
        _ => return None,
    };
    Some(Event {
        id: text(a, "id"),
        actor: Some(actor_of(&a["memberCreator"])),
        at: text(a, "date"),
        kind,
    })
}

fn card_id(key: &ItemKey) -> Result<String> {
    let id = key.id.trim();
    if id.is_empty() {
        return Err(invalid(
            "reading a Trello card",
            "the card id is needed (the card number does not name a card)",
        ));
    }
    Ok(id.to_string())
}

impl TrelloClient {
    pub(super) async fn conv_timeline(
        &self,
        key: &ItemKey,
        older: Option<&PageCursor>,
        opts: &TimelineOpts,
    ) -> Result<TimelinePage> {
        require_issue("Trello", key)?;
        let id = card_id(key)?;
        let card: Value = self
            .get_json(
                &format!(
                    "/cards/{}?fields={CARD_FIELDS}&list=true&list_fields=name",
                    encode_component(&id)
                ),
                "reading the Trello card",
            )
            .await?;
        let mut actions: Vec<Value> = self
            .get_json(
                &format!(
                    "/cards/{}/actions?filter=commentCard,updateCard:idList,updateCard:closed,createCard&limit={ACTIONS_LIMIT}&memberCreator_fields=username,fullName",
                    encode_component(&id)
                ),
                "reading the Trello card's activity",
            )
            .await?;
        // Newest first from Trello; the timeline is oldest first.
        actions.reverse();
        let card_url = opt_text(&card, "url").unwrap_or_default();
        let author = actions
            .iter()
            .find(|a| text(a, "type") == "createCard")
            .map(|a| actor_of(&a["memberCreator"]))
            .unwrap_or_else(Actor::unknown);
        let created_at = actions
            .iter()
            .find(|a| text(a, "type") == "createCard")
            .map(|a| text(a, "date"))
            .unwrap_or_default();

        let mut entries: Vec<TimelineEntry> = Vec::new();
        for a in &actions {
            if text(a, "type") == "commentCard" {
                entries.push(TimelineEntry::Comment(comment_of(&card_url, a)));
            } else if opts.events {
                if let Some(e) = event_of(a) {
                    entries.push(TimelineEntry::Event(e));
                }
            }
        }
        let total = entries.len() as u32;
        let (window, older_cursor) = window_of(entries, older)?;

        let closed = card["closed"].as_bool() == Some(true);
        let labels = card["labels"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|l| opt_text(l, "name").or_else(|| opt_text(l, "color")))
            .collect();
        let number = card["idShort"].as_u64().unwrap_or(key.number);
        let header = ItemHeader {
            key: ItemKey {
                kind: ItemKind::Issue,
                number,
                id,
            },
            reference: format!("#{number}"),
            title: text(&card, "name"),
            state_label: if closed {
                "archived".to_string()
            } else {
                opt_text(&card["list"], "name").unwrap_or_else(|| "open".to_string())
            },
            open: !closed,
            author,
            body: Markup::new(text(&card, "desc"), MarkupFlavor::TrelloMd),
            created_at,
            web_url: opt_text(&card, "url")
                .or_else(|| opt_text(&card, "shortUrl"))
                .unwrap_or_default(),
            labels,
        };
        Ok(TimelinePage {
            header,
            entries: window,
            older: older_cursor,
            total_hint: Some(total),
        })
    }

    pub(super) async fn conv_post(
        &self,
        key: &ItemKey,
        body: &str,
        internal: bool,
    ) -> Result<Comment> {
        require_issue("Trello", key)?;
        check_body("commenting on a Trello card", body, Some(MAX_BODY))?;
        if internal {
            return Err(Error::Unsupported(
                "internal comments: Trello has none".into(),
            ));
        }
        let url = format!(
            "{}/cards/{}/actions/comments",
            self.integration.api_base(),
            encode_component(&card_id(key)?)
        );
        let response = self
            .send(
                HttpRequest::post_json(url, &json!({ "text": body })),
                "commenting on a Trello card",
            )
            .await?;
        Ok(comment_of("", &response.json::<Value>()?))
    }

    pub(super) async fn conv_edit(
        &self,
        key: &ItemKey,
        comment_id: &str,
        body: &str,
    ) -> Result<Comment> {
        require_issue("Trello", key)?;
        check_body("editing a Trello comment", body, Some(MAX_BODY))?;
        let url = format!(
            "{}/actions/{}/text",
            self.integration.api_base(),
            encode_component(comment_id)
        );
        let response = self
            .send(
                HttpRequest::put_json(url, &json!({ "value": body })),
                "editing a Trello comment",
            )
            .await?;
        Ok(comment_of("", &response.json::<Value>()?))
    }

    pub(super) async fn conv_delete(&self, key: &ItemKey, comment_id: &str) -> Result<()> {
        require_issue("Trello", key)?;
        let url = format!(
            "{}/actions/{}",
            self.integration.api_base(),
            encode_component(comment_id)
        );
        self.send(HttpRequest::delete(url), "deleting a Trello comment")
            .await
            .map(|_| ())
    }

    /// The token's own member: `username` is the login of its comments.
    pub(super) async fn conv_identity(&self) -> Result<PostingIdentity> {
        let me: Value = self
            .get_json(
                "/members/me?fields=username,fullName",
                "checking the Trello token",
            )
            .await?;
        let actor = actor_of(&me);
        if actor.kind == ActorKind::Unknown {
            return Err(Error::Decode("no member in the Trello answer".to_string()));
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
