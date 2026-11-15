//! Reading and writing the conversation of an issue or a pull / merge
//! request: its header, comments, system events and code threads, and
//! posting, replying, editing and deleting as the identity the token has.
//!
//! The shapes live in [`model`] (provider-neutral: every host maps into
//! them), what a provider can do in [`caps`], and who a post is made as in
//! [`identity`]. The operations are `HostingClient` methods with
//! `Unsupported` defaults; GitHub and GitLab implement them in
//! `github/conversation.rs` and `gitlab/conversation.rs`.
//!
//! **Pagination contract.** The first read answers the newest window of at
//! most [`WINDOW`] entries (oldest to newest inside it); `older` is the
//! cursor for the window before it, `None` when the window starts the
//! conversation.

pub mod caps;
#[cfg(any(
    feature = "gitea",
    feature = "azure-devops",
    feature = "bitbucket",
    feature = "gerrit"
))]
pub(crate) mod compat;
pub mod identity;
pub mod model;
#[cfg(any(
    feature = "github",
    feature = "gitlab",
    feature = "gitea",
    feature = "azure-devops",
    feature = "bitbucket",
    feature = "gerrit"
))]
pub(crate) mod time;

pub use caps::ConversationCaps;
pub use identity::{IdentityKind, PostingIdentity, TokenSource};
pub use model::{
    Actor, ActorKind, CodeThread, Comment, Event, EventKind, ItemHeader, ItemKey, ItemKind, Markup,
    MarkupFlavor, PageCursor, Reaction, ReviewEntry, TimelineEntry, TimelineOpts, TimelinePage,
};

/// The most entries one timeline page holds.
pub const WINDOW: usize = 300;
