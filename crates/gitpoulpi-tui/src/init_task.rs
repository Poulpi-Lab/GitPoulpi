//! `git init`, on its own connection to the session's daemon.
//!
//! The same shape as [`crate::clone_task`] and for the same reasons:
//! `daemon_link` is bound to one `RepoRef` for its whole life, and this
//! operation has no repository -- it is what *produces* one. It is also
//! the one place the overlay's Create mode could block the input loop,
//! because `git init` is a process spawn, and on Windows a process
//! spawn is not free. So it runs on a thread and reports on an
//! `std::sync::mpsc` channel the render loop drains with `try_recv`,
//! exactly like every other channel this window reads.
//!
//! No progress: an init either lands or fails, and it does so in
//! milliseconds. The one message it sends is the answer.

use crate::redraw::NotifySend as _;
use std::path::PathBuf;
use std::sync::mpsc as std_mpsc;

use gitpoulpi_core::domain::{InitOptions, OperationKind, RepoRef};
use gitpoulpi_ipc::transport::{read_message, write_message};
use gitpoulpi_ipc::{Request, ServerMessage};

/// The one thing a running init says: where it landed, or why it did
/// not. Always sent exactly once, including when the connection dies --
/// a Create form that can wait forever is worse than one that fails.
#[derive(Debug, Clone)]
pub struct InitFinished {
    pub path: PathBuf,
    pub result: Result<RepoRef, String>,
}

pub fn start(session: String, path: PathBuf, opts: InitOptions) -> Receiver {
    let (tx, rx) = std_mpsc::channel();
    std::thread::spawn(move || {
        let runtime = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(runtime) => runtime,
            Err(e) => {
                let _ = tx.send_notify(InitFinished {
                    path,
                    result: Err(gitpoulpi_i18n::msg::task_error::init_runtime(e)),
                });
                return;
            }
        };
        runtime.block_on(run(session, path, opts, tx));
    });
    Receiver { rx }
}

pub struct Receiver {
    rx: std_mpsc::Receiver<InitFinished>,
}

impl Receiver {
    pub fn try_recv(&self) -> Option<InitFinished> {
        self.rx.try_recv().ok()
    }
}

async fn run(
    session: String,
    path: PathBuf,
    opts: InitOptions,
    tx: std_mpsc::Sender<InitFinished>,
) {
    let finish = |result: Result<RepoRef, String>| {
        let _ = tx.send_notify(InitFinished {
            path: path.clone(),
            result,
        });
    };

    let port = match crate::daemon_link::ensure_daemon(&session).await {
        Ok(port) => port,
        Err(e) => return finish(Err(e)),
    };
    let stream = match gitpoulpi_ipc::discovery::connect(port).await {
        Ok(stream) => stream,
        Err(e) => return finish(Err(gitpoulpi_i18n::msg::task_error::daemon_unreachable(e))),
    };
    let (mut read_half, mut write_half) = stream.into_split();

    let request = Request {
        id: 1,
        session,
        // No repo target: `dispatch` answers `Init` before `repo_ref`
        // would reject the request for having none, exactly as it does
        // for `Discover`.
        repo: None,
        command: OperationKind::Init,
        args: serde_json::json!({ "path": path, "opts": opts }),
    };
    if write_message(&mut write_half, &request).await.is_err() {
        return finish(Err(gitpoulpi_i18n::msg::task_error::INIT_SEND.get().into()));
    }

    loop {
        match read_message::<_, ServerMessage>(&mut read_half).await {
            Ok(Some(ServerMessage::Response(response))) if response.request_id == request.id => {
                return finish(match response.result {
                    Ok(value) => serde_json::from_value::<RepoRef>(value)
                        .map_err(gitpoulpi_i18n::msg::task_error::init_malformed),
                    Err(e) => Err(e.message),
                });
            }
            // Broadcast events for other repositories share this
            // connection; they are not ours.
            Ok(Some(_)) => continue,
            Ok(None) | Err(_) => {
                return finish(Err(gitpoulpi_i18n::msg::task_error::INIT_CLOSED
                    .get()
                    .into()))
            }
        }
    }
}
