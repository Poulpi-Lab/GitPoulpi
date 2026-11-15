//! What a provider lets the conversation do.

use serde::{Deserialize, Serialize};

use super::model::{ItemKind, MarkupFlavor};

/// What a control may be offered for, per provider and item kind. Every
/// `false` is a real "Unsupported" and hides the matching control.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationCaps {
    pub read: bool,
    pub comment: bool,
    pub reply_in_thread: bool,
    pub start_thread: bool,
    pub edit_own: bool,
    pub delete_own: bool,
    pub resolve: bool,
    pub reactions: bool,
    pub events: bool,
    pub internal_notes: bool,
    pub read_flavor: MarkupFlavor,
    pub write_flavor: MarkupFlavor,
    pub max_body: Option<usize>,
}

impl ConversationCaps {
    /// Nothing: the answer for a provider or kind without a conversation.
    pub const NONE: Self = Self {
        read: false,
        comment: false,
        reply_in_thread: false,
        start_thread: false,
        edit_own: false,
        delete_own: false,
        resolve: false,
        reactions: false,
        events: false,
        internal_notes: false,
        read_flavor: MarkupFlavor::PlainText,
        write_flavor: MarkupFlavor::PlainText,
        max_body: None,
    };

    /// GitHub: issues have a flat comment list, pull requests also threads.
    pub fn github(kind: ItemKind) -> Self {
        Self {
            read: true,
            comment: true,
            reply_in_thread: kind == ItemKind::PullRequest,
            start_thread: false,
            edit_own: true,
            delete_own: true,
            resolve: kind == ItemKind::PullRequest,
            reactions: true,
            events: true,
            internal_notes: false,
            read_flavor: MarkupFlavor::Gfm,
            write_flavor: MarkupFlavor::Gfm,
            max_body: Some(65_536),
        }
    }

    /// Gitea and Forgejo: a request has code threads (resolution only on
    /// Gitea), an issue a flat list; no reactions are read, no internal notes.
    pub fn gitea(kind: ItemKind, can_resolve: bool) -> Self {
        let pr = kind == ItemKind::PullRequest;
        Self {
            read: true,
            comment: true,
            reply_in_thread: pr,
            start_thread: false,
            edit_own: true,
            delete_own: true,
            resolve: pr && can_resolve,
            reactions: false,
            events: true,
            internal_notes: false,
            read_flavor: MarkupFlavor::GiteaMd,
            write_flavor: MarkupFlavor::GiteaMd,
            max_body: None,
        }
    }

    /// Azure DevOps. A pull request has threads (a reply is a comment of the
    /// thread; the thread status is shown but not changed, so `resolve` is
    /// off). A work item has a flat
    /// comment list with no threads and no event history here. Azure DevOps
    /// Server (7.0) takes work item comments as HTML, so it is written as
    /// HTML and shown as the Markdown converted from it.
    pub fn azure_devops(kind: ItemKind, server: bool) -> Self {
        let pr = kind == ItemKind::PullRequest;
        Self {
            read: true,
            comment: true,
            reply_in_thread: pr,
            start_thread: false,
            edit_own: true,
            delete_own: true,
            resolve: false,
            reactions: false,
            events: pr,
            internal_notes: false,
            read_flavor: MarkupFlavor::AzureMd,
            write_flavor: if pr || !server {
                MarkupFlavor::AzureMd
            } else {
                MarkupFlavor::Html
            },
            max_body: Some(150_000),
        }
    }

    /// Bitbucket Cloud and Data Center: pull requests only (Cloud retired
    /// its issue tracker; Data Center has none). A resolved thread is shown
    /// but not changed (`resolve` off); no reactions, no internal notes.
    pub fn bitbucket(kind: ItemKind) -> Self {
        if kind == ItemKind::Issue {
            return Self::NONE;
        }
        Self {
            read: true,
            comment: true,
            reply_in_thread: true,
            start_thread: false,
            edit_own: true,
            delete_own: true,
            resolve: false,
            reactions: false,
            events: true,
            internal_notes: false,
            read_flavor: MarkupFlavor::BitbucketMd,
            write_flavor: MarkupFlavor::BitbucketMd,
            max_body: None,
        }
    }

    /// Gerrit: changes only. Messages and comments cannot be edited or
    /// deleted by their author (only an administrator can delete a change
    /// message), and a reply goes into the comment's thread as a new draft
    /// published with the next review.
    pub fn gerrit(kind: ItemKind) -> Self {
        if kind == ItemKind::Issue {
            return Self::NONE;
        }
        Self {
            read: true,
            comment: true,
            reply_in_thread: true,
            start_thread: false,
            edit_own: false,
            delete_own: false,
            resolve: true,
            reactions: false,
            events: true,
            internal_notes: false,
            read_flavor: MarkupFlavor::PlainText,
            write_flavor: MarkupFlavor::PlainText,
            max_body: None,
        }
    }

    /// GitLab: issues and merge requests both have discussions.
    pub fn gitlab(kind: ItemKind) -> Self {
        Self {
            read: true,
            comment: true,
            reply_in_thread: true,
            start_thread: false,
            edit_own: true,
            delete_own: true,
            resolve: kind == ItemKind::PullRequest,
            reactions: false,
            events: true,
            internal_notes: true,
            read_flavor: MarkupFlavor::GitlabMd,
            write_flavor: MarkupFlavor::GitlabMd,
            max_body: Some(1_000_000),
        }
    }
}
