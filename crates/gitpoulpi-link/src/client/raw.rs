//! One generic request path for views that talk to the daemon with their
//! own typed answers (the Undo History panel, the Reflog and Lost work
//! views): a request goes to the link thread, which sends it on the one
//! connection, off the render thread, and the answer lands in a [`Reply`]
//! the view polls every frame. Nothing here ever waits.

use super::*;
use std::sync::Mutex;

type Slot<T> = Arc<Mutex<Option<GitResult<T>>>>;

/// A request for the link thread.
pub(crate) struct RawRequest {
    /// The repository to ask about; `None` = the window's own.
    repo: Option<RepoRef>,
    command: OperationKind,
    args: serde_json::Value,
    reply: Box<dyn FnOnce(GitResult<serde_json::Value>) + Send>,
}

/// The queue's two ends. The sender is cloned into [`Requester`]s; the
/// link thread takes the receiver once.
pub struct RawQueue {
    tx: tokio::sync::mpsc::UnboundedSender<RawRequest>,
    rx: Mutex<Option<tokio::sync::mpsc::UnboundedReceiver<RawRequest>>>,
}

impl Default for RawQueue {
    fn default() -> Self {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        Self {
            tx,
            rx: Mutex::new(Some(rx)),
        }
    }
}

impl RawQueue {
    pub fn requester(&self) -> Requester {
        Requester {
            tx: self.tx.clone(),
        }
    }

    /// The requests queued so far, as (operation, arguments), for tests that
    /// check what a view asked of the daemon. Nothing answers them.
    #[cfg(any(test, feature = "test-support"))]
    pub fn queued(&self) -> Vec<(OperationKind, serde_json::Value)> {
        let Ok(mut held) = self.rx.lock() else {
            return Vec::new();
        };
        let Some(rx) = held.as_mut() else {
            return Vec::new();
        };
        let mut out = Vec::new();
        while let Ok(request) = rx.try_recv() {
            out.push((request.command, request.args));
        }
        out
    }

    pub(super) fn take_receiver(&self) -> Option<tokio::sync::mpsc::UnboundedReceiver<RawRequest>> {
        self.rx.lock().ok().and_then(|mut rx| rx.take())
    }
}

/// Runs one queued request on its own task and delivers the answer.
pub(super) fn spawn_raw(conn: &LinkConn, request: RawRequest) {
    let conn = conn.clone();
    tokio::spawn(async move {
        let RawRequest {
            repo,
            command,
            args,
            reply,
        } = request;
        let result = call_raw(
            &conn.write_half,
            &conn.pending,
            &conn.next_id,
            &conn.session,
            repo.as_ref().unwrap_or(&conn.repo),
            command,
            args,
        )
        .await;
        reply(result);
    });
}

/// A clonable handle to send requests with.
#[derive(Clone)]
pub struct Requester {
    tx: tokio::sync::mpsc::UnboundedSender<RawRequest>,
}

impl Requester {
    /// Sends `command` and decodes the answer as `T`.
    pub fn call<T: serde::de::DeserializeOwned + Send + 'static>(
        &self,
        command: OperationKind,
        args: serde_json::Value,
    ) -> Reply<T> {
        self.call_in(None, command, args)
    }

    /// [`Self::call`] about another repository of the session (the daemon
    /// holds many, ADR-0006); `None` = the window's own.
    pub fn call_in<T: serde::de::DeserializeOwned + Send + 'static>(
        &self,
        repo: Option<RepoRef>,
        command: OperationKind,
        args: serde_json::Value,
    ) -> Reply<T> {
        let slot: Slot<T> = Arc::new(Mutex::new(None));
        let fill = Arc::clone(&slot);
        let request = RawRequest {
            repo,
            command,
            args,
            reply: Box::new(move |result| {
                let decoded = result.and_then(|value| {
                    serde_json::from_value::<T>(value).map_err(|e| {
                        CoreError::Backend(gitpoulpi_i18n::msg::link_error::malformed_response(e))
                    })
                });
                if let Ok(mut held) = fill.lock() {
                    *held = Some(decoded);
                }
                super::host::wake();
            }),
        };
        if self.tx.send(request).is_err() {
            if let Ok(mut held) = slot.lock() {
                *held = Some(Err(CoreError::Backend(
                    gitpoulpi_i18n::msg::link_error::CLOSED.get().to_string(),
                )));
            }
        }
        Reply { slot }
    }
}

/// The answer to one [`Requester::call`], once it has come.
pub struct Reply<T> {
    slot: Slot<T>,
}

#[cfg(any(test, feature = "test-support"))]
impl<T> Reply<T> {
    /// An answer that is already there.
    pub fn ready(result: GitResult<T>) -> Self {
        Self {
            slot: Arc::new(Mutex::new(Some(result))),
        }
    }

    /// A request that never answers.
    pub fn pending() -> Self {
        Self::channel().0
    }

    /// A request and the function that answers it later.
    pub fn channel() -> (Self, impl FnOnce(GitResult<T>)) {
        let slot: Slot<T> = Arc::new(Mutex::new(None));
        let fill = Arc::clone(&slot);
        (Self { slot }, move |result| {
            *fill.lock().unwrap() = Some(result);
        })
    }
}

impl<T> Reply<T> {
    /// The answer, once; `None` while the request is still running.
    pub fn take(&self) -> Option<GitResult<T>> {
        self.slot.lock().ok().and_then(|mut held| held.take())
    }
}
