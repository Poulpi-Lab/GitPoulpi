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
