//! The conversation `ai hosting-reply` reads. Two JSON shapes are accepted:
//!
//! - what `hosting issue comments --json` and `hosting review conversation
//!   --json` print: `item` (the header, bodies as text plus `body_flavor`)
//!   and `entries` typed by `type`;
//! - the model's own page (`header` and `entries` typed by `entry`, bodies as
//!   `{text, flavor}`), the shape the command first took.
//!
//! The printed shape is read leniently: the AI needs the item, the comments,
//! the reviews and the code threads; a field a host does not give is empty.

use gitpoulpi_hosting::conversation::{
    Actor, ActorKind, CodeThread, Comment, ItemHeader, ItemKey, ItemKind, Markup, MarkupFlavor,
    ReviewEntry, TimelineEntry, TimelinePage,
};
use gitpoulpi_hosting::review::model::{DiffSide, ReviewState};
use serde_json::Value;

/// Parses the conversation text in either shape.
pub(crate) fn parse(text: &str) -> Result<TimelinePage, String> {
    let value: Value = serde_json::from_str(text).map_err(|e| {
        format!("the conversation is not JSON (what `hosting issue comments --json` prints): {e}")
    })?;
    if value.get("header").is_some() {
        return serde_json::from_value(value).map_err(|e| {
            format!("the conversation is not a page of entries (`header` and `entries`): {e}")
        });
    }
    if value.get("item").is_some_and(Value::is_object) {
        return from_printed(&value);
    }
    Err(
        "the conversation has neither `item` (what `hosting issue comments --json` or \
         `hosting review conversation --json` prints) nor `header`"
            .to_string(),
    )
}

fn text(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn opt_text(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(Value::as_str).map(str::to_string)
}

fn flag(v: &Value, key: &str) -> bool {
    v.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn number(v: &Value, key: &str) -> Option<u32> {
    v.get(key)
        .and_then(Value::as_u64)
        .and_then(|n| u32::try_from(n).ok())
}

fn actor(v: Option<&Value>) -> Actor {
    let Some(v) = v.filter(|v| v.is_object()) else {
        return Actor::unknown();
    };
    let login = text(v, "login");
    let display = match text(v, "display") {
        d if d.is_empty() => login.clone(),
        d => d,
    };
    let kind: ActorKind = v
        .get("kind")
        .and_then(|k| serde_json::from_value(k.clone()).ok())
        .unwrap_or_default();
    Actor {
        login,
        display,
        kind,
        web_url: opt_text(v, "web_url"),
    }
}

/// `body` (text) with its `body_flavor`.
fn body(v: &Value) -> Markup {
    let flavor: MarkupFlavor = v
        .get("body_flavor")
        .and_then(|f| serde_json::from_value(f.clone()).ok())
        .unwrap_or_default();
    Markup::new(text(v, "body"), flavor)
}

fn comment(v: &Value) -> Comment {
    Comment {
        id: text(v, "id"),
        author: actor(v.get("author")),
        body: body(v),
        created_at: text(v, "created_at"),
        updated_at: opt_text(v, "updated_at"),
        edited: flag(v, "edited"),
        reactions: Vec::new(),
        in_reply_to: opt_text(v, "in_reply_to"),
        discussion_id: opt_text(v, "discussion_id"),
        internal: flag(v, "internal"),
        can_edit: v.get("can_edit").and_then(Value::as_bool),
        can_delete: v.get("can_delete").and_then(Value::as_bool),
        web_url: opt_text(v, "web_url"),
    }
}

fn entry(v: &Value) -> Option<TimelineEntry> {
    match v.get("type").and_then(Value::as_str)? {
        "comment" => Some(TimelineEntry::Comment(comment(v))),
        "review" => Some(TimelineEntry::Review(ReviewEntry {
            id: text(v, "id"),
            author: actor(v.get("author")),
            state: v
                .get("state")
                .and_then(|s| serde_json::from_value::<ReviewState>(s.clone()).ok())
                .unwrap_or(ReviewState::Commented),
            body: body(v),
            submitted_at: text(v, "submitted_at"),
        })),
        "thread" => Some(TimelineEntry::Thread(CodeThread {
            id: text(v, "id"),
            path: text(v, "path"),
            line: number(v, "line"),
            original_line: number(v, "original_line"),
            side: v
                .get("side")
                .and_then(|s| serde_json::from_value::<DiffSide>(s.clone()).ok())
                .unwrap_or_default(),
            start_line: None,
            resolved: flag(v, "resolved"),
            outdated: flag(v, "outdated"),
            resolvable: flag(v, "resolvable"),
            diff_hunk: opt_text(v, "diff_hunk"),
            comments: v
                .get("comments")
                .and_then(Value::as_array)
                .map(|all| all.iter().map(comment).collect())
                .unwrap_or_default(),
        })),
        // System events add nothing to a reply.
        _ => None,
    }
}

fn from_printed(v: &Value) -> Result<TimelinePage, String> {
    let item = &v["item"];
    let kind = match text(item, "kind").as_str() {
        "pull_request" => ItemKind::PullRequest,
        _ => ItemKind::Issue,
    };
    let header = ItemHeader {
        key: ItemKey {
            kind,
            number: item.get("number").and_then(Value::as_u64).unwrap_or(0),
            id: text(item, "id"),
        },
        reference: text(item, "reference"),
        title: text(item, "title"),
        state_label: text(item, "state"),
        open: flag(item, "open"),
        author: actor(item.get("author")),
        body: body(item),
        created_at: text(item, "created_at"),
        web_url: text(item, "web_url"),
        labels: item
            .get("labels")
            .and_then(Value::as_array)
            .map(|l| {
                l.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default(),
    };
    let entries = v
        .get("entries")
        .and_then(Value::as_array)
        .ok_or_else(|| "the conversation has no `entries` list".to_string())?
        .iter()
        .filter_map(entry)
        .collect();
    Ok(TimelinePage {
        header,
        entries,
        older: None,
        total_hint: None,
    })
}
