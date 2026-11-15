//! The conversation's reads: a plain thread over a channel, drained by the
//! render loop with `try_recv`; nothing here is awaited on the input thread.
//!
//! One run answers, in order: the saved copy (an open only), the provider's
//! capabilities, the timeline (the newest window, or the one before a cursor)
//! and, on an open, who a post would be made as. A failed run says whether
//! the host was unreachable (the saved copy stays, offline banner), said to
//! slow down, or refused.

use std::path::PathBuf;
use std::sync::{mpsc, Arc};

use gitpoulpi_config::Integration;
use gitpoulpi_hosting::conversation::{
    ConversationCaps, ItemKey, PageCursor, PostingIdentity, TimelineOpts, TimelinePage,
};
use gitpoulpi_hosting::Error;

use super::cache;
use crate::hosting_task::{error_text, request_task_failed, ClientFactory};
use crate::redraw::NotifySend as _;

/// Which item and on which profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Source {
    pub(crate) integration: Integration,
    /// Every profile (the token source reads them all).
    pub(crate) all: Vec<Integration>,
    pub(crate) project: String,
    pub(crate) key: ItemKey,
}

/// What one run does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Mode {
    /// Saved copy first, then the host, then the identity.
    Open,
    /// The host only (F5, the timer, after a post).
    Refresh,
    /// The window before this cursor.
    Older(PageCursor),
}

/// Why a run failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Failure {
    /// The host could not be reached.
    Offline(String),
    /// The host said to slow down.
    RateLimited(String),
    /// The provider has no conversation for this item.
    Unsupported(String),
    Other(String),
}

impl Failure {
    pub(crate) fn text(&self) -> &str {
        match self {
            Failure::Offline(t)
            | Failure::RateLimited(t)
            | Failure::Unsupported(t)
            | Failure::Other(t) => t,
        }
    }
}

pub(crate) enum Msg {
    /// The saved copy, with when it was saved (unix seconds).
    Cached(Box<TimelinePage>, i64),
    Caps(ConversationCaps),
    Fresh(Box<TimelinePage>),
    Older(Box<TimelinePage>),
    Identity(Result<PostingIdentity, String>),
    Failed(Failure),
    /// The run is over (the channel holds nothing more).
    Done,
}

fn classify(err: &Error) -> Failure {
    let text = error_text(err);
    match err {
        Error::Http(_) => Failure::Offline(text),
        Error::RateLimited { .. } => Failure::RateLimited(text),
        Error::Unsupported(_) | Error::NotImplemented(_) => Failure::Unsupported(text),
        _ => Failure::Other(text),
    }
}

/// Starts a run. `cache_dir` is the repository's state directory (`None`:
/// nothing is saved or read).
pub(crate) fn start(
    factory: Arc<dyn ClientFactory>,
    source: Source,
    cache_dir: Option<PathBuf>,
    mode: Mode,
) -> mpsc::Receiver<Msg> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let file = cache_dir
            .as_deref()
            .map(|d| cache::file_for(d, &source.integration.id, &source.project, &source.key));
        if mode == Mode::Open {
            if let Some((page, at)) = file.as_deref().and_then(cache::read) {
                let _ = tx.send_notify(Msg::Cached(Box::new(page), at));
            }
        }
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        let rt = match rt {
            Ok(rt) => rt,
            Err(e) => {
                let _ = tx.send_notify(Msg::Failed(Failure::Other(request_task_failed(e))));
                let _ = tx.send_notify(Msg::Done);
                return;
            }
        };
        rt.block_on(async {
            let client = match factory.client(&source.integration, &source.all) {
                Ok(c) => c,
                Err(e) => {
                    let _ = tx.send_notify(Msg::Failed(classify(&e)));
                    return;
                }
            };
            let caps = client.conversation_caps(source.key.kind);
            let readable = caps.read;
            let _ = tx.send_notify(Msg::Caps(caps));
            if !readable {
                let _ = tx.send_notify(Msg::Failed(Failure::Unsupported(
                    gitpoulpi_i18n::msg::item_view::NOT_READABLE
                        .get()
                        .to_string(),
                )));
                return;
            }
            let older = match &mode {
                Mode::Older(c) => Some(c.clone()),
                _ => None,
            };
            let page = client
                .item_timeline(
                    &source.project,
                    &source.key,
                    older.as_ref(),
                    &TimelineOpts { events: true },
                )
                .await;
            match page {
                Ok(page) => {
                    if older.is_some() {
                        let _ = tx.send_notify(Msg::Older(Box::new(page)));
                    } else {
                        if let Some(file) = &file {
                            cache::write(file, &page);
                        }
                        let _ = tx.send_notify(Msg::Fresh(Box::new(page)));
                    }
                }
                Err(e) => {
                    let _ = tx.send_notify(Msg::Failed(classify(&e)));
                    return;
                }
            }
            if mode == Mode::Open {
                let identity = client
                    .posting_identity(&source.project)
                    .await
                    .map_err(|e| error_text(&e));
                let _ = tx.send_notify(Msg::Identity(identity));
            }
        });
        let _ = tx.send_notify(Msg::Done);
    });
    rx
}
