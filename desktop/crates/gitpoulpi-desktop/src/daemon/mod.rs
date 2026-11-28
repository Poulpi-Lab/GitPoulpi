//! The link to GitPoulpi's daemon.
//!
//! The window is a client of its session's daemon exactly like the terminal UI: [`Backend`] starts a
//! [`gitpoulpi_link::DaemonLink`] (which finds the session's running daemon through its discovery file
//! or starts one inside this process), so a terminal UI, the CLI and this window on the same session
//! see each other's changes live. Reads go through the link's own connection ([`Backend::call`]);
//! refreshes the daemon pushes arrive on the link's receivers, which the window drains when
//! [`host::DesktopHost`] wakes it (`app::live`); writes are link actions (`transfer`).

mod history;
pub mod host;
mod queries;
pub mod refs_ops;
pub mod staging;
pub mod transfer;

use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::{Arc, Mutex, MutexGuard};
use std::task::{Context, Poll};

use gitpoulpi_core::domain::{OperationKind, RepoRef};
use gitpoulpi_link::{DaemonLink, Reply, Requester};
use serde::de::DeserializeOwned;

pub use history::{is_detached, merge_head_history, Refresh, Walk};
pub use host::DesktopHost;
pub use queries::LogSource;

struct Open {
    link: DaemonLink,
    repo: RepoRef,
    requester: Requester,
}

pub struct Backend {
    session: String,
    host: Arc<DesktopHost>,
    open: Mutex<Open>,
}

pub type Shared = Arc<Backend>;

impl Backend {
    /// Opens the repository enclosing `repo_path` in the daemon of its default session (the session the
    /// terminal UI and the CLI use for it).
    pub fn start(repo_path: &Path) -> anyhow::Result<Shared> {
        let root = find_root(repo_path).ok_or_else(|| anyhow::anyhow!("not a git repository"))?;
        let session = gitpoulpi_config::default_session_name_from_root(&root);
        Self::start_in(session, &root)
    }

    /// [`Backend::start`] in a named session (`--session`).
    pub fn start_in(session: String, repo_path: &Path) -> anyhow::Result<Shared> {
        let root = find_root(repo_path).ok_or_else(|| anyhow::anyhow!("not a git repository"))?;
        let host = DesktopHost::global();
        let open = open_link(&session, &root, &host);
        Ok(Arc::new(Self {
            session,
            host,
            open: Mutex::new(open),
        }))
    }

    fn open(&self) -> MutexGuard<'_, Open> {
        self.open.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Switches to the repository enclosing `root`; `false` when there is none. The old link ends.
    pub fn set_repo(&self, root: &Path) -> bool {
        match find_root(root) {
            Some(r) => {
                *self.open() = open_link(&self.session, &r, &self.host);
                true
            }
            None => false,
        }
    }

    pub fn repo_root(&self) -> PathBuf {
        self.open().repo.root.clone()
    }

    pub fn session(&self) -> &str {
        &self.session
    }

    pub fn host(&self) -> &Arc<DesktopHost> {
        &self.host
    }

    /// Runs `f` on the repository's link: the receivers to drain and the request methods.
    pub fn with_link<R>(&self, f: impl FnOnce(&DaemonLink) -> R) -> R {
        f(&self.open().link)
    }

    /// One request on the link's connection. The answer is a future any executor can poll: the link
    /// thread fills it and wakes the host, which wakes whoever waits.
    pub fn call<T: DeserializeOwned + Send + 'static>(
        &self,
        command: OperationKind,
        args: serde_json::Value,
    ) -> impl Future<Output = Result<T, String>> + Send + 'static {
        let reply = self.open().requester.call::<T>(command, args);
        Answer {
            reply,
            host: self.host.clone(),
        }
    }
}

/// The pending answer of one request.
struct Answer<T> {
    reply: Reply<T>,
    host: Arc<DesktopHost>,
}

impl<T> Future for Answer<T> {
    type Output = Result<T, String>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Registered before the look, so an answer that lands in between wakes this poll again.
        self.host.register(cx.waker());
        match self.reply.take() {
            Some(result) => Poll::Ready(result.map_err(|e| host::failure_text(&e))),
            None => Poll::Pending,
        }
    }
}

fn open_link(session: &str, root: &Path, host: &Arc<DesktopHost>) -> Open {
    let repo = RepoRef::from_root(root);
    let link = gitpoulpi_link::start(session.to_string(), repo.clone(), host.clone());
    let requester = link.activity_handle().raw.requester();
    Open {
        link,
        repo,
        requester,
    }
}

/// The nearest ancestor of `start` (itself included) holding a `.git` entry.
pub fn find_root(start: &Path) -> Option<PathBuf> {
    let mut dir = start;
    loop {
        if dir.join(".git").exists() {
            return Some(dir.to_path_buf());
        }
        dir = dir.parent()?;
    }
}
