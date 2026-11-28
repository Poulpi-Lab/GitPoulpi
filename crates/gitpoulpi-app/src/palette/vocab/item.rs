//! The conversation view's command vocabulary, kept apart from the view so
//! the command palette (and a build without a hosting provider) can name the
//! commands without the view's types.
#![cfg_attr(
    not(all(feature = "hosting", feature = "conversation")),
    allow(dead_code)
)]

use gitpoulpi_i18n::msg::item_view as msg;

/// The code a thread is about: where `Open Code` goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeRef {
    pub thread_id: String,
    pub path: String,
    /// The line it sits on now (`None` once outdated).
    pub line: Option<u32>,
    /// The line it was written on.
    pub original_line: Option<u32>,
    /// On the old (deleted) side of the diff.
    pub old_side: bool,
    pub outdated: bool,
}

/// What the palette needs to know to gray an item-view command out. Plain
/// data (`Copy`): the reasons are resolved by the view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemAvail {
    /// The conversation is in.
    pub loaded: bool,
    pub has_back: bool,
    pub has_older: bool,
    pub has_unread: bool,
    /// The selected entry holds links.
    pub has_links: bool,
    /// The selected entry holds a Mermaid diagram.
    pub has_diagram: bool,
    /// Why Open Code is not offered (`None`: the selected entry is a code
    /// thread and the view can open code).
    pub code: Option<&'static str>,
    /// The selected entry has a page on the host.
    pub has_url: bool,
    /// Why a reply to the selected entry is not offered (`None`: it is).
    pub reply: Option<&'static str>,
    /// Why a quote reply is not offered.
    pub quote: Option<&'static str>,
    /// Why the selected comment cannot be edited / deleted.
    pub edit: Option<&'static str>,
    pub delete: Option<&'static str>,
    /// Why the text cannot be sent now.
    pub send: Option<&'static str>,
    /// Why the internal note checkbox does nothing.
    pub internal: Option<&'static str>,
    /// The host lists system events.
    pub has_events: bool,
    /// The window is wide enough for the composer's Split.
    pub split_fits: bool,
    /// The composer is there (the host takes comments).
    pub composer: bool,
}

/// What the view can be asked to do. The palette lists one entry per value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemCmd {
    Back,
    Refresh,
    LoadOlder,
    PrevEntry,
    NextEntry,
    NextUnread,
    Reply,
    Quote,
    NewComment,
    Links,
    /// Open the Mermaid diagram of the selected entry.
    OpenDiagram,
    /// Open the code of the selected thread (the review's Files page).
    OpenCode,
    CopyLink,
    /// The dragged text, else the selected entry's text.
    CopyText,
    OpenBrowser,
    Edit,
    Delete,
    ToggleEvents,
    TogglePreview,
    ToggleSplit,
    Send,
    ToggleInternal,
    Close,
}

impl ItemCmd {
    /// Why the command does nothing right now (`None`: it can run).
    pub fn unavailable(self, avail: Option<ItemAvail>) -> Option<&'static str> {
        let Some(a) = avail else {
            return Some(msg::WHY_NO_VIEW.get());
        };
        match self {
            Self::Refresh | Self::OpenBrowser | Self::Close | Self::CopyText => None,
            Self::Back => (!a.has_back).then_some(msg::WHY_NO_BACK.get()),
            Self::PrevEntry | Self::NextEntry => (!a.loaded).then_some(msg::WHY_NOT_LOADED.get()),
            Self::LoadOlder => (!a.has_older).then_some(msg::WHY_NO_OLDER.get()),
            Self::NextUnread => (!a.has_unread).then_some(msg::WHY_NO_UNREAD.get()),
            Self::Reply => a.reply,
            Self::Quote => a.quote,
            Self::ToggleInternal => {
                if !a.composer {
                    Some(msg::WHY_NO_COMPOSER.get())
                } else {
                    a.internal
                }
            }
            Self::NewComment | Self::TogglePreview => {
                (!a.composer).then_some(msg::WHY_NO_COMPOSER.get())
            }
            Self::ToggleSplit => {
                if !a.composer {
                    Some(msg::WHY_NO_COMPOSER.get())
                } else if !a.split_fits {
                    Some(gitpoulpi_i18n::msg::md_editor::SPLIT_NEEDS_WIDTH.get())
                } else {
                    None
                }
            }
            Self::Links => (!a.has_links).then_some(msg::WHY_NO_LINKS.get()),
            Self::OpenDiagram => (!a.has_diagram).then_some(msg::WHY_NO_DIAGRAM.get()),
            Self::OpenCode => a.code,
            Self::CopyLink => (!a.has_url).then_some(msg::WHY_NO_URL.get()),
            Self::Edit => a.edit,
            Self::Delete => a.delete,
            Self::ToggleEvents => (!a.has_events).then_some(msg::WHY_NO_EVENTS.get()),
            Self::Send => a.send,
        }
    }
}
