//! The conversation view's command vocabulary, kept apart from the view so
//! the command palette (and a build without a hosting provider) can name the
//! commands without the view's types.
#![cfg_attr(
    not(all(feature = "hosting", feature = "conversation")),
    allow(dead_code)
)]

use gitpoulpi_i18n::msg::item_view as msg;

/// What the palette needs to know to gray an item-view command out. Plain
/// data (`Copy`): the reasons are resolved by the view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ItemAvail {
    /// The conversation is in.
    pub(crate) loaded: bool,
    pub(crate) has_back: bool,
    pub(crate) has_older: bool,
    pub(crate) has_unread: bool,
    /// The selected entry holds links.
    pub(crate) has_links: bool,
    /// The selected entry holds a Mermaid diagram.
    pub(crate) has_diagram: bool,
    /// The selected entry has a page on the host.
    pub(crate) has_url: bool,
    /// Why a reply to the selected entry is not offered (`None`: it is).
    pub(crate) reply: Option<&'static str>,
    /// Why the selected comment cannot be edited / deleted.
    pub(crate) edit: Option<&'static str>,
    pub(crate) delete: Option<&'static str>,
    /// Why the text cannot be sent now.
    pub(crate) send: Option<&'static str>,
    /// The host lists system events.
    pub(crate) has_events: bool,
    /// The window is wide enough for the composer's Split.
    pub(crate) split_fits: bool,
    /// The composer is there (the host takes comments).
    pub(crate) composer: bool,
}

/// What the view can be asked to do. The palette lists one entry per value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ItemCmd {
    Back,
    Refresh,
    LoadOlder,
    PrevEntry,
    NextEntry,
    NextUnread,
    Reply,
    NewComment,
    Links,
    /// Open the Mermaid diagram of the selected entry.
    OpenDiagram,
    CopyLink,
    OpenBrowser,
    Edit,
    Delete,
    ToggleEvents,
    TogglePreview,
    ToggleSplit,
    Send,
    Close,
}

impl ItemCmd {
    /// Why the command does nothing right now (`None`: it can run).
    pub(crate) fn unavailable(self, avail: Option<ItemAvail>) -> Option<&'static str> {
        let Some(a) = avail else {
            return Some(msg::WHY_NO_VIEW.get());
        };
        match self {
            Self::Refresh | Self::OpenBrowser | Self::Close => None,
            Self::Back => (!a.has_back).then_some(msg::WHY_NO_BACK.get()),
            Self::PrevEntry | Self::NextEntry => (!a.loaded).then_some(msg::WHY_NOT_LOADED.get()),
            Self::LoadOlder => (!a.has_older).then_some(msg::WHY_NO_OLDER.get()),
            Self::NextUnread => (!a.has_unread).then_some(msg::WHY_NO_UNREAD.get()),
            Self::Reply => a.reply,
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
            Self::CopyLink => (!a.has_url).then_some(msg::WHY_NO_URL.get()),
            Self::Edit => a.edit,
            Self::Delete => a.delete,
            Self::ToggleEvents => (!a.has_events).then_some(msg::WHY_NO_EVENTS.get()),
            Self::Send => a.send,
        }
    }
}
