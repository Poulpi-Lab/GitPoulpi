//! Which link an event belongs to.
//!
//! A frontend with several repositories open has one [`super::DaemonLink`]
//! per repository, all raising hooks on a [`super::LinkHost`]. Every event
//! hook therefore receives the [`LinkContext`] of the link it comes from
//! (its [`LinkId`] and repository root), so the frontend routes it without
//! filtering by path.
//!
//! The free functions the client raises hooks from (the notified channel
//! sends, the ticket queue, the raw request replies) have no link at hand, so
//! the context rides on the thread: it is installed on every thread of a
//! link's own runtime (worker and blocking threads), on the link thread
//! itself, and entered by the few [`super::DaemonLink`] methods that raise a
//! hook from the caller's thread. A hook raised from a thread that carries no
//! context (a frontend thread calling an [`crate::activity::Activity`] method,
//! a test) sees [`LinkContext::detached`] and the process-wide host.

use std::cell::RefCell;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::LinkHost;

/// A small identity for one link, unique within the process and never reused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LinkId(u64);

impl LinkId {
    /// The id of "no link": what a hook raised off every link's threads sees.
    pub const DETACHED: LinkId = LinkId(0);

    pub(super) fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// The raw number (the same one tickets carry for their link).
    pub fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for LinkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "link-{}", self.0)
    }
}

/// The link a hook was raised for: its id and the repository it shows.
#[derive(Clone)]
pub struct LinkContext {
    id: LinkId,
    root: PathBuf,
    /// The host this link was started with, so two links with two hosts each
    /// reach their own. `None` for a detached context (the installed host).
    host: Option<Arc<dyn LinkHost>>,
}

impl fmt::Debug for LinkContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LinkContext")
            .field("id", &self.id)
            .field("root", &self.root)
            .finish()
    }
}

impl LinkContext {
    pub(super) fn new(id: LinkId, root: PathBuf, host: Arc<dyn LinkHost>) -> Self {
        Self {
            id,
            root,
            host: Some(host),
        }
    }

    /// The context of no link: id [`LinkId::DETACHED`], an empty root.
    pub fn detached() -> Self {
        Self {
            id: LinkId::DETACHED,
            root: PathBuf::new(),
            host: None,
        }
    }

    pub fn id(&self) -> LinkId {
        self.id
    }

    /// The repository root the link was started for.
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn is_detached(&self) -> bool {
        self.id == LinkId::DETACHED
    }

    pub(super) fn host(&self) -> Option<Arc<dyn LinkHost>> {
        self.host.clone()
    }

    /// Makes this the context of the current thread for good (a runtime
    /// thread's start).
    pub(super) fn install_on_thread(&self) {
        CURRENT.with(|slot| *slot.borrow_mut() = Some(self.clone()));
    }

    /// Makes this the context of the current thread until the guard drops.
    pub(super) fn enter(&self) -> Entered {
        let previous = CURRENT.with(|slot| slot.borrow_mut().replace(self.clone()));
        Entered { previous }
    }
}

thread_local! {
    static CURRENT: RefCell<Option<LinkContext>> = const { RefCell::new(None) };
}

/// The context of the current thread, if it carries one.
pub(super) fn current() -> Option<LinkContext> {
    CURRENT.with(|slot| slot.borrow().clone())
}

/// Restores the thread's previous context on drop.
pub(super) struct Entered {
    previous: Option<LinkContext>,
}

impl Drop for Entered {
    fn drop(&mut self) {
        let previous = self.previous.take();
        CURRENT.with(|slot| *slot.borrow_mut() = previous);
    }
}
