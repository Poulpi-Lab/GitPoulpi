//! From `gitpoulpi-hosting`'s conversation shapes to what a drafted reply is
//! built from ([`ItemDigest`], [`Entry`]). The TUI, the CLI and MCP all map
//! through here, so the model sees the same text whichever frontend asks.
//!
//! System events (labels, state changes, pushes) are left out: they add
//! nothing to a reply.

use gitpoulpi_hosting::conversation::{
    Actor, ActorKind, CodeThread, Comment, ItemHeader, ItemKind, MarkupFlavor, TimelineEntry,
};

use super::prompt::{cut_end, MAX_HUNK_BYTES};
use super::{Entry, ItemDigest};

/// A name for a markup flavor, for the "write in" line of the request.
pub fn flavor_name(flavor: MarkupFlavor) -> &'static str {
    match flavor {
        MarkupFlavor::Gfm => "GitHub Flavored Markdown",
        MarkupFlavor::GitlabMd => "GitLab Flavored Markdown",
        MarkupFlavor::GiteaMd => "Gitea Markdown",
        MarkupFlavor::CommonMark => "CommonMark Markdown",
        MarkupFlavor::AzureMd => "Azure DevOps Markdown",
        MarkupFlavor::BitbucketMd => "Bitbucket Markdown",
        MarkupFlavor::Html => "HTML",
        MarkupFlavor::JiraWiki => "Jira wiki markup",
        MarkupFlavor::Textile => "Textile",
        MarkupFlavor::YoutrackMd => "YouTrack Markdown",
        MarkupFlavor::TrelloMd => "Trello Markdown",
        MarkupFlavor::PlainText => "plain text with no markup",
        MarkupFlavor::ObsidianMd => "Markdown",
    }
}

fn who(actor: &Actor) -> String {
    let bot = if actor.kind == ActorKind::Bot {
        " (bot)"
    } else {
        ""
    };
    format!("{}{bot}", actor.login)
}

fn comment_text(c: &Comment) -> String {
    let edited = if c.edited { ", edited" } else { "" };
    format!(
        "Comment by {} on {}{edited}:\n{}",
        who(&c.author),
        c.created_at,
        c.body.text.trim()
    )
}

fn thread_text(t: &CodeThread, with_hunk: bool) -> String {
    let mut out = format!("Code thread on {}", t.path);
    if let Some(line) = t.line.or(t.original_line) {
        out.push_str(&format!(":{line}"));
    }
    if t.resolved {
        out.push_str(" (resolved)");
    }
    if t.outdated {
        out.push_str(" (outdated)");
    }
    out.push('\n');
    if let (true, Some(hunk)) = (with_hunk, t.diff_hunk.as_deref()) {
        let (hunk, _) = cut_end(hunk, MAX_HUNK_BYTES);
        out.push_str(&format!("```diff\n{hunk}\n```\n"));
    }
    let comments: Vec<String> = t.comments.iter().map(comment_text).collect();
    out.push_str(&comments.join("\n\n"));
    out
}

/// The one-line header and the description of an item.
pub fn item_digest(h: &ItemHeader) -> ItemDigest {
    let kind = match h.key.kind {
        ItemKind::Issue => "Issue",
        ItemKind::PullRequest => "Pull request",
    };
    let mut summary = format!(
        "{kind} {} \"{}\" ({}) by {}",
        h.reference,
        h.title.trim(),
        h.state_label,
        who(&h.author)
    );
    if !h.labels.is_empty() {
        summary.push_str(&format!(", labels: {}", h.labels.join(", ")));
    }
    ItemDigest {
        summary,
        description: h.body.text.clone(),
    }
}

/// The entries of a conversation page, oldest first, without the system events.
pub fn entries(entries: &[TimelineEntry]) -> Vec<Entry> {
    entries
        .iter()
        .filter_map(|entry| match entry {
            TimelineEntry::Comment(c) => Some(Entry {
                thread_ids: [
                    Some(c.id.clone()),
                    c.discussion_id.clone(),
                    c.in_reply_to.clone(),
                ]
                .into_iter()
                .flatten()
                .collect(),
                text: comment_text(c),
                thread_text: None,
            }),
            TimelineEntry::Review(r) => Some(Entry {
                thread_ids: vec![r.id.clone()],
                text: format!(
                    "Review by {} on {} ({:?}):\n{}",
                    who(&r.author),
                    r.submitted_at,
                    r.state,
                    r.body.text.trim()
                ),
                thread_text: None,
            }),
            TimelineEntry::Thread(t) => Some(Entry {
                thread_ids: std::iter::once(t.id.clone())
                    .chain(t.comments.iter().map(|c| c.id.clone()))
                    .collect(),
                text: thread_text(t, false),
                thread_text: Some(thread_text(t, true)),
            }),
            TimelineEntry::Event(_) => None,
        })
        .collect()
}

#[cfg(test)]
mod tests;
