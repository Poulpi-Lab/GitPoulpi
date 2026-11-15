//! `gitpoulpi hosting issue comments|comment|comment-edit|comment-delete` and
//! `hosting review conversation|comment-edit|comment-delete`: the whole
//! conversation of an issue or a pull/merge request, and posting, editing and
//! deleting in it, through the conversation methods of `HostingClient` (the
//! same ones the TUI's conversation view calls), for every provider.
//!
//! Like every hosting call these run in the client process. After a post,
//! edit or delete in a repository the daemon is told
//! (`OperationKind::NotifyHostingChange`) so an open view refreshes; with
//! `--project` there is no repository, so nobody is told.
//!
//! A provider that cannot do something answers an honest error (exit 1, the
//! same as the other hosting commands) naming the provider and the action,
//! decided from its `ConversationCaps` before anything is sent.

use gitpoulpi_config::ApiFamily;
use gitpoulpi_core::domain::OperationKind;
use gitpoulpi_hosting::conversation::{
    Actor, Comment, ConversationCaps, Event, ItemHeader, ItemKey, ItemKind, Markup, MarkupFlavor,
    PageCursor, PostingIdentity, TimelineEntry, TimelineOpts,
};
use gitpoulpi_hosting::markup;
use gitpoulpi_hosting::xrefs::{self, XrefContext, XrefTarget};
use gitpoulpi_hosting::HostingClient;
use gitpoulpi_ipc::{HostingChange, HostingChangeArgs, HostingItemKind};
use serde_json::{json, Value};

use crate::client::Session;
use crate::repo_cmd::Emitted;

use super::link::Which;
use super::*;

/// What a conversation command does.
pub(super) enum Op {
    Read {
        limit: Option<usize>,
        older: Option<String>,
        events: bool,
    },
    Post {
        body: String,
        reply_to: Option<String>,
        internal: bool,
    },
    Edit {
        comment_id: String,
        body: String,
    },
    Delete {
        comment_id: String,
    },
}

/// The item a reference names, as the provider wants it: a number for the
/// forges, the tracker's own key in `ItemKey::id` for Jira, Linear, YouTrack
/// and Trello (whose number does not name the item).
pub(super) fn parse_item(
    kind: ItemKind,
    family: ApiFamily,
    reference: &str,
) -> Result<ItemKey, String> {
    let text = reference.trim();
    let bare = text.trim_start_matches(['#', '!']);
    let bare = match bare.get(..3) {
        Some(p) if p.eq_ignore_ascii_case("AB#") => &bare[3..],
        _ => bare,
    };
    if bare.is_empty() {
        return Err("the item is empty (a number like 12, or the tracker's own key)".to_string());
    }
    if let Ok(number) = bare.parse::<u64>() {
        return Ok(ItemKey {
            kind,
            number,
            id: String::new(),
        });
    }
    let own_spelling = kind == ItemKind::Issue
        && matches!(
            family,
            ApiFamily::Jira | ApiFamily::Trello | ApiFamily::Linear | ApiFamily::YouTrack
        );
    let allowed = |c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_';
    if own_spelling && bare.chars().all(allowed) {
        let number = bare
            .rsplit('-')
            .next()
            .and_then(|n| n.parse().ok())
            .unwrap_or(0);
        return Ok(ItemKey {
            kind,
            number,
            id: bare.to_string(),
        });
    }
    Err(match kind {
        ItemKind::Issue => format!("`{text}` is not an issue number (like 12, #12 or AB#12)"),
        ItemKind::PullRequest => {
            format!("`{text}` is not a pull request number (like 12, #12 or !12)")
        }
    })
}

/// The target of `--project P [--id I]` for an issue: no repository.
pub(super) fn issue_project_target(
    env: &HostingEnv,
    id: Option<&str>,
    project: &str,
) -> Result<RepoTarget, String> {
    let project = project.trim().trim_matches('/');
    if project.is_empty() {
        return Err("--project is empty (expected owner/name or group/project)".to_string());
    }
    let integrations = env.integrations()?;
    let integration = pick(&integrations, id)?.clone();
    if !integration.enabled {
        return Err(format!("`{}` is disabled", integration.id));
    }
    if let Some(why) = Which::Issues.unsupported_by(&integration) {
        return Err(why);
    }
    Ok(RepoTarget {
        integration,
        integrations,
        project: project.to_string(),
        remote: String::new(),
        ambiguous: false,
        linked: false,
        filter: None,
    })
}

fn kind_word(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::Issue => "issue",
        ItemKind::PullRequest => "pull request",
    }
}

fn kind_json(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::Issue => "issue",
        ItemKind::PullRequest => "pull_request",
    }
}

/// The honest refusal of something the provider cannot do.
fn refuse(target: &RepoTarget, kind: ItemKind, what: &str) -> String {
    format!(
        "{} does not support {what} on {}s (see `capabilities` of `hosting {} {}`)",
        target.integration.kind.label(),
        kind_word(kind),
        match kind {
            ItemKind::Issue => "issue",
            ItemKind::PullRequest => "review",
        },
        match kind {
            ItemKind::Issue => "comments",
            ItemKind::PullRequest => "conversation",
        },
    )
}

fn caps_json(caps: &ConversationCaps) -> Value {
    json!({
        "read": caps.read,
        "comment": caps.comment,
        "reply_in_thread": caps.reply_in_thread,
        "start_thread": caps.start_thread,
        "edit_own": caps.edit_own,
        "delete_own": caps.delete_own,
        "resolve": caps.resolve,
        "reactions": caps.reactions,
        "events": caps.events,
        "internal_notes": caps.internal_notes,
        "read_flavor": caps.read_flavor,
        "write_flavor": caps.write_flavor,
        "max_body": caps.max_body,
    })
}

fn identity_json(identity: &PostingIdentity) -> Value {
    let mut out = json!({
        "login": identity.login,
        "display": identity.display,
        "kind": identity.kind,
    });
    if let Ok(Value::Object(source)) = serde_json::to_value(&identity.source) {
        for (k, v) in source {
            out[k] = v;
        }
    }
    out
}

/// Who a post is made as, in one word: the identity's login (the configured
/// "Post as" name for an App bot), else who the host says wrote the comment,
/// else what the identity says about itself.
fn posted_as(identity: Option<&PostingIdentity>, comment: &Comment) -> String {
    let author = comment.author.login.trim();
    identity
        .and_then(|i| i.login.clone())
        .or_else(|| (!author.is_empty() && author != "ghost").then(|| author.to_string()))
        .or_else(|| {
            identity
                .map(|i| i.display.clone())
                .filter(|d| !d.is_empty())
        })
        .unwrap_or_default()
}

fn actor_json(actor: &Actor) -> Value {
    json!({
        "login": markup::sanitize(&actor.login),
        "display": markup::sanitize(&actor.display),
        "kind": actor.kind,
        "web_url": actor.web_url,
    })
}

/// The text as Markdown, when the host's own markup is another one.
fn body_markdown(body: &Markup) -> Option<String> {
    let text = markup::sanitize(&body.text);
    match body.flavor {
        MarkupFlavor::JiraWiki => Some(markup::jira_to_markdown(&text)),
        MarkupFlavor::Textile => Some(markup::textile_to_markdown(&text)),
        MarkupFlavor::Html => Some(markup::html_to_markdown(&text)),
        _ => None,
    }
}

fn refs_json(text: &str, ctx: &XrefContext) -> Value {
    let found = xrefs::scan(text, ctx);
    Value::Array(
        found
            .into_iter()
            .map(|x| {
                let (kind, in_app, url) = match &x.target {
                    XrefTarget::InApp(item) => (
                        match item.kind {
                            gitpoulpi_hosting::refs::RefKind::Issue => "issue",
                            gitpoulpi_hosting::refs::RefKind::PullRequest => "pull_request",
                        },
                        true,
                        None,
                    ),
                    XrefTarget::Web(url) => ("web", false, Some(url.clone())),
                    XrefTarget::Mention { url, .. } => ("mention", false, url.clone()),
                    XrefTarget::Commit { url, .. } => ("commit", false, url.clone()),
                    XrefTarget::Chip { .. } => ("chip", false, None),
                };
                json!({ "text": x.text, "kind": kind, "in_app": in_app, "url": url })
            })
            .collect(),
    )
}

fn comment_json(c: &Comment, ctx: &XrefContext) -> Value {
    let text = markup::sanitize(&c.body.text);
    json!({
        "type": "comment",
        "id": c.id,
        "author": actor_json(&c.author),
        "created_at": c.created_at,
        "updated_at": c.updated_at,
        "edited": c.edited,
        "body": text,
        "body_flavor": c.body.flavor,
        "body_markdown": body_markdown(&c.body),
        "refs": refs_json(body_markdown(&c.body).as_deref().unwrap_or(&text), ctx),
        "reactions": c.reactions.iter().map(|r| json!({"name": r.name, "count": r.count})).collect::<Vec<_>>(),
        "in_reply_to": c.in_reply_to,
        "discussion_id": c.discussion_id,
        "internal": c.internal,
        "can_edit": c.can_edit,
        "can_delete": c.can_delete,
        "web_url": c.web_url,
    })
}

fn event_json(e: &Event) -> Value {
    let mut out = json!({
        "type": "event",
        "id": e.id,
        "actor": e.actor.as_ref().map(actor_json),
        "at": e.at,
    });
    if let Ok(Value::Object(kind)) = serde_json::to_value(&e.kind) {
        for (k, v) in kind {
            out[k] = v;
        }
    }
    out
}

fn entry_json(entry: &TimelineEntry, ctx: &XrefContext) -> Value {
    match entry {
        TimelineEntry::Comment(c) => comment_json(c, ctx),
        TimelineEntry::Event(e) => event_json(e),
        TimelineEntry::Review(r) => json!({
            "type": "review",
            "id": r.id,
            "author": actor_json(&r.author),
            "state": r.state,
            "body": markup::sanitize(&r.body.text),
            "body_markdown": body_markdown(&r.body),
            "submitted_at": r.submitted_at,
        }),
        TimelineEntry::Thread(t) => json!({
            "type": "thread",
            "id": t.id,
            "path": t.path,
            "line": t.line,
            "original_line": t.original_line,
            "side": t.side.as_str(),
            "resolved": t.resolved,
            "outdated": t.outdated,
            "resolvable": t.resolvable,
            "diff_hunk": t.diff_hunk,
            "comments": t.comments.iter().map(|c| comment_json(c, ctx)).collect::<Vec<_>>(),
        }),
    }
}

fn header_json(h: &ItemHeader) -> Value {
    json!({
        "kind": kind_json(h.key.kind),
        "number": h.key.number,
        "id": h.key.id,
        "reference": h.reference,
        "title": markup::sanitize(&h.title),
        "state": h.state_label,
        "open": h.open,
        "author": actor_json(&h.author),
        "body": markup::sanitize(&h.body.text),
        "body_flavor": h.body.flavor,
        "body_markdown": body_markdown(&h.body),
        "created_at": h.created_at,
        "labels": h.labels,
        "web_url": h.web_url,
    })
}

fn xref_context(target: &RepoTarget) -> XrefContext {
    XrefContext {
        family: Some(target.integration.kind.family()),
        web_base: gitpoulpi_hosting::integration_web_base(&target.integration).unwrap_or_default(),
        project: target.project.clone(),
        ..XrefContext::default()
    }
}

fn first_line(text: &str) -> String {
    markup::sanitize(text.lines().next().unwrap_or("")).to_string()
}

fn human_of(entries: &[TimelineEntry]) -> Vec<String> {
    let mut out = Vec::new();
    for e in entries {
        match e {
            TimelineEntry::Comment(c) => out.push(format!(
                "{}  {}  {}: {}",
                c.id,
                c.created_at,
                markup::sanitize(&c.author.login),
                first_line(&c.body.text)
            )),
            TimelineEntry::Event(ev) => out.push(format!(
                "(event) {}  {}  {:?}",
                ev.at,
                ev.actor
                    .as_ref()
                    .map_or(String::new(), |a| markup::sanitize(&a.login)),
                ev.kind
            )),
            TimelineEntry::Review(r) => out.push(format!(
                "(review) {}: {:?} {}",
                markup::sanitize(&r.author.login),
                r.state,
                first_line(&r.body.text)
            )),
            TimelineEntry::Thread(t) => out.push(format!(
                "(thread) {} {}:{}  {} comments{}",
                t.id,
                t.path,
                t.line.unwrap_or(0),
                t.comments.len(),
                if t.resolved { " (resolved)" } else { "" }
            )),
        }
    }
    out
}

/// Whether the posted identity is read (best effort) next to the answer.
async fn identity_of(client: &dyn HostingClient, project: &str) -> Option<PostingIdentity> {
    client.posting_identity(project).await.ok()
}

fn change_args(target: &RepoTarget, key: &ItemKey, change: HostingChange) -> Result<Value, String> {
    serde_json::to_value(HostingChangeArgs {
        integration_id: target.integration.id.clone(),
        project: target.project.clone(),
        kind: match key.kind {
            ItemKind::Issue => HostingItemKind::Issue,
            ItemKind::PullRequest => HostingItemKind::PullRequest,
        },
        number: key.number,
        id: key.id.clone(),
        change,
    })
    .map_err(|e| e.to_string())
}

/// Tells the daemon (best effort: the host call already succeeded) so an
/// open view of the item refreshes. `false` when nobody could be told.
async fn notify(
    ipc: Option<&mut Session>,
    target: &RepoTarget,
    key: &ItemKey,
    change: HostingChange,
) -> bool {
    let (Some(ipc), Ok(args)) = (ipc, change_args(target, key, change)) else {
        return false;
    };
    ipc.call::<Value>(OperationKind::NotifyHostingChange, args)
        .await
        .is_ok()
}

fn base_json(target: &RepoTarget, key: &ItemKey) -> Value {
    let mut out = super::review::target_json(target);
    out["item"] = json!({ "kind": kind_json(key.kind), "number": key.number, "id": key.id });
    out
}

/// Runs one conversation command on `key` of `target`.
pub(super) async fn run(
    env: &HostingEnv,
    ipc: Option<&mut Session>,
    target: &RepoTarget,
    key: ItemKey,
    op: Op,
) -> Result<Emitted, String> {
    let client = env.client(&target.integration, &target.integrations)?;
    let caps = client.conversation_caps(key.kind);
    let kind = key.kind;
    match op {
        Op::Read {
            limit,
            older,
            events,
        } => {
            if !caps.read {
                return Err(refuse(target, kind, "reading the conversation"));
            }
            let cursor = older.map(PageCursor);
            let page = client
                .item_timeline(
                    &target.project,
                    &key,
                    cursor.as_ref(),
                    &TimelineOpts {
                        events: events && caps.events,
                    },
                )
                .await
                .map_err(|e| e.to_string())?;
            let identity = identity_of(client.as_ref(), &target.project).await;
            let ctx = xref_context(target);
            let mut entries = page.entries;
            if !events {
                entries.retain(|e| !matches!(e, TimelineEntry::Event(_)));
            }
            let mut trimmed = 0usize;
            if let Some(limit) = limit {
                if entries.len() > limit {
                    trimmed = entries.len() - limit;
                    entries.drain(..trimmed);
                }
            }
            let mut out = base_json(target, &key);
            out["item"] = header_json(&page.header);
            out["identity"] = identity.as_ref().map_or(Value::Null, identity_json);
            out["capabilities"] = caps_json(&caps);
            out["entries"] = Value::Array(entries.iter().map(|e| entry_json(e, &ctx)).collect());
            out["older"] = json!(page.older.as_ref().map(|c| c.0.clone()));
            out["trimmed_older"] = json!(trimmed);
            out["total_hint"] = json!(page.total_hint);
            let mut human = vec![format!(
                "{} {}  [{}]  {}",
                page.header.reference,
                markup::sanitize(&page.header.title),
                page.header.state_label,
                page.header.web_url
            )];
            human.extend(human_of(&entries));
            if let Some(c) = &page.older {
                human.push(format!("(older entries: --older {})", c.0));
            }
            Ok(Emitted {
                exit_code: 0,
                json: out,
                human: human.join("\n"),
            })
        }
        Op::Post {
            body,
            reply_to,
            internal,
        } => {
            if reply_to.is_some() && !caps.reply_in_thread {
                return Err(refuse(target, kind, "replying in a thread"));
            }
            if reply_to.is_none() && !caps.comment {
                return Err(refuse(target, kind, "commenting"));
            }
            if internal && !caps.internal_notes {
                return Err(refuse(target, kind, "internal comments"));
            }
            check_len(&caps, &body)?;
            let comment = match &reply_to {
                Some(discussion) => {
                    client
                        .reply_in_discussion(&target.project, &key, discussion, &body)
                        .await
                }
                None => {
                    client
                        .post_item_comment(&target.project, &key, &body, internal)
                        .await
                }
            }
            .map_err(|e| e.to_string())?;
            // After the post: a first post teaches an App bot's login.
            let identity = identity_of(client.as_ref(), &target.project).await;
            let notified = notify(ipc, target, &key, HostingChange::Posted).await;
            let ctx = xref_context(target);
            let who = posted_as(identity.as_ref(), &comment);
            let mut out = base_json(target, &key);
            out["comment"] = comment_json(&comment, &ctx);
            out["posted_as"] = json!(who);
            out["identity"] = identity.as_ref().map_or(Value::Null, identity_json);
            out["reply_to"] = json!(reply_to);
            out["notified"] = json!(notified);
            Ok(Emitted {
                exit_code: 0,
                json: out,
                human: format!(
                    "commented on {} as {who} (comment {})",
                    target_label(target, &key),
                    comment.id
                ),
            })
        }
        Op::Edit { comment_id, body } => {
            if !caps.edit_own {
                return Err(refuse(target, kind, "editing comments"));
            }
            check_len(&caps, &body)?;
            let comment = client
                .edit_item_comment(&target.project, &key, &comment_id, &body)
                .await
                .map_err(|e| e.to_string())?;
            let identity = identity_of(client.as_ref(), &target.project).await;
            let notified = notify(ipc, target, &key, HostingChange::Edited).await;
            let ctx = xref_context(target);
            let who = posted_as(identity.as_ref(), &comment);
            let mut out = base_json(target, &key);
            out["comment"] = comment_json(&comment, &ctx);
            out["posted_as"] = json!(who);
            out["identity"] = identity.as_ref().map_or(Value::Null, identity_json);
            out["notified"] = json!(notified);
            Ok(Emitted {
                exit_code: 0,
                json: out,
                human: format!(
                    "edited comment {comment_id} on {}",
                    target_label(target, &key)
                ),
            })
        }
        Op::Delete { comment_id } => {
            if !caps.delete_own {
                return Err(refuse(target, kind, "deleting comments"));
            }
            client
                .delete_item_comment(&target.project, &key, &comment_id)
                .await
                .map_err(|e| e.to_string())?;
            let notified = notify(ipc, target, &key, HostingChange::Deleted).await;
            let mut out = base_json(target, &key);
            out["deleted"] = json!(comment_id);
            out["notified"] = json!(notified);
            Ok(Emitted {
                exit_code: 0,
                json: out,
                human: format!(
                    "deleted comment {comment_id} on {}",
                    target_label(target, &key)
                ),
            })
        }
    }
}

fn check_len(caps: &ConversationCaps, body: &str) -> Result<(), String> {
    match caps.max_body {
        Some(max) if body.chars().count() > max => Err(format!(
            "the text is longer than the {max} characters the host takes"
        )),
        _ => Ok(()),
    }
}

fn target_label(target: &RepoTarget, key: &ItemKey) -> String {
    if key.id.is_empty() {
        format!("{}#{}", target.project, key.number)
    } else {
        key.id.clone()
    }
}

/// `hosting markdown`: the text as the app reads it.
pub(super) fn markdown(
    env: &HostingEnv,
    args: crate::cli::MarkdownArgs,
) -> Result<Emitted, String> {
    let text = read_issue_body(args.body, args.body_file)?;
    let clean = markup::sanitize(&text);
    let ctx = match args.id.as_deref() {
        Some(id) => {
            let integrations = env.integrations()?;
            let integration = pick(&integrations, Some(id))?;
            XrefContext {
                family: Some(integration.kind.family()),
                web_base: gitpoulpi_hosting::integration_web_base(integration).unwrap_or_default(),
                ..XrefContext::default()
            }
        }
        None => XrefContext::default(),
    };
    let width = args.width.unwrap_or(80).max(8);
    let lines = wrap(&clean, width);
    let kind = match args.for_item {
        crate::cli::MarkdownFor::Issue => "issue",
        crate::cli::MarkdownFor::Pr => "pull_request",
    };
    Ok(Emitted {
        exit_code: 0,
        json: json!({
            "for": kind,
            "width": width,
            "text": clean,
            "lines": lines,
            "refs": refs_json(&clean, &ctx),
        }),
        human: lines.join("\n"),
    })
}

/// Greedy word wrap to `width` characters; a blank line stays a blank line, a
/// word longer than the width is cut.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            out.push(String::new());
            continue;
        }
        let mut current = String::new();
        let mut used = 0usize;
        for word in line.split_whitespace() {
            let w = word.chars().count();
            if used > 0 && used + 1 + w > width {
                out.push(std::mem::take(&mut current));
                used = 0;
            }
            if w > width {
                for c in word.chars() {
                    let cw = 1;
                    if used + cw > width {
                        out.push(std::mem::take(&mut current));
                        used = 0;
                    }
                    current.push(c);
                    used += cw;
                }
                continue;
            }
            if used > 0 {
                current.push(' ');
                used += 1;
            }
            current.push_str(word);
            used += w;
        }
        out.push(current);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_come_with_any_prefix_and_keys_only_where_the_provider_uses_them() {
        let n = |family, text| parse_item(ItemKind::Issue, family, text).unwrap();
        assert_eq!(n(ApiFamily::Github, "#142").number, 142);
        assert_eq!(n(ApiFamily::AzureDevops, "AB#12").number, 12);
        assert_eq!(n(ApiFamily::Gitlab, "!7").number, 7);
        let jira = n(ApiFamily::Jira, "ABC-12");
        assert_eq!((jira.number, jira.id.as_str()), (12, "ABC-12"));
        let card = n(ApiFamily::Trello, "Xy9Zk3Ab");
        assert_eq!((card.number, card.id.as_str()), (0, "Xy9Zk3Ab"));
        assert!(parse_item(ItemKind::Issue, ApiFamily::Github, "ABC-12").is_err());
        assert!(parse_item(ItemKind::PullRequest, ApiFamily::Jira, "ABC-12").is_err());
        assert!(parse_item(ItemKind::Issue, ApiFamily::Github, "  ").is_err());
    }

    #[test]
    fn wrapping_keeps_blank_lines_and_cuts_a_word_longer_than_the_width() {
        assert_eq!(wrap("aa bb cc\n\ndd", 5), ["aa bb", "cc", "", "dd"]);
        assert_eq!(wrap("abcdefghij", 4), ["abcd", "efgh", "ij"]);
    }
}
