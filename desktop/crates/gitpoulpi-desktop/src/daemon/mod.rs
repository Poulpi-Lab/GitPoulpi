//! The link to GitPoulpi's daemon.
//!
//! The daemon is *embedded*: this process starts a `gitpoulpi_daemon::Daemon` on a dedicated tokio
//! runtime, then talks to it through a `gitpoulpi_ipc::Client` over loopback TCP, so the data path is
//! exactly the one the terminal UI and the CLI use. GPUI has its own executor; the two are bridged
//! with `tokio::task::JoinHandle`s, which are plain futures and can be awaited from `cx.spawn`.

mod history;
mod queries;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use gitpoulpi_config::DefaultBackend;
use gitpoulpi_core::domain::{OperationKind, RepoRef};
use gitpoulpi_ipc::{Client, Endpoint, Request};
use serde::de::DeserializeOwned;

pub use history::{is_detached, merge_head_history, Refresh, Walk};
pub use queries::LogSource;

/// Name of the daemon session this process hosts.
pub const SESSION: &str = "desktop";

pub struct Backend {
    rt: tokio::runtime::Runtime,
    endpoint: Endpoint,
    session: String,
    repo: Mutex<RepoRef>,
    next_id: AtomicU64,
}

pub type Shared = Arc<Backend>;

impl Backend {
    /// Starts the embedded daemon and points it at the repository enclosing `repo_path`.
    pub fn start(repo_path: &Path) -> anyhow::Result<Shared> {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .thread_name("gp-daemon")
            .build()?;
        let session = SESSION.to_string();
        let endpoint = rt.block_on(async {
            let handle = gitpoulpi_daemon::Daemon::new(session.clone())
                .spawn(gitpoulpi_daemon::backend_for(DefaultBackend::Git2))
                .await
                .map_err(|e| anyhow::anyhow!("daemon: {e}"))?;
            let endpoint = handle.endpoint();
            // The accept loop must outlive this function: the process exiting stops it.
            std::mem::forget(handle);
            Ok::<_, anyhow::Error>(endpoint)
        })?;
        let root = find_root(repo_path).ok_or_else(|| anyhow::anyhow!("not a git repository"))?;
        Ok(Arc::new(Self {
            rt,
            endpoint,
            session,
            repo: Mutex::new(RepoRef::from_root(&root)),
            next_id: AtomicU64::new(1),
        }))
    }

    /// Switches to the repository enclosing `root`; `false` when there is none.
    pub fn set_repo(&self, root: &Path) -> bool {
        match find_root(root) {
            Some(r) => {
                *self.repo.lock().unwrap() = RepoRef::from_root(&r);
                true
            }
            None => false,
        }
    }

    pub fn repo_root(&self) -> PathBuf {
        self.repo.lock().unwrap().root.clone()
    }

    /// One IPC round trip on the daemon runtime. A fresh connection per call keeps a slow diff from
    /// queueing behind a log page (the terminal UI multiplexes one socket instead).
    pub fn call<T: DeserializeOwned + Send + 'static>(
        &self,
        command: OperationKind,
        args: serde_json::Value,
    ) -> impl std::future::Future<Output = Result<T, String>> + Send + 'static {
        let endpoint = self.endpoint.clone();
        let session = self.session.clone();
        let repo = self.repo.lock().unwrap().id.clone();
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let handle = self.rt.spawn(async move {
            let mut client = Client::connect(&endpoint)
                .await
                .map_err(|e| e.to_string())?;
            let request = Request {
                id,
                session,
                repo: Some(repo),
                command,
                args,
            };
            let response = client.call(&request).await.map_err(|e| e.to_string())?;
            match response.result {
                Ok(v) => serde_json::from_value::<T>(v).map_err(|e| e.to_string()),
                Err(e) => Err(e.message),
            }
        });
        async move { handle.await.map_err(|e| e.to_string())? }
    }

    /// Runs a future on the daemon runtime and returns a handle that can be awaited anywhere.
    pub fn run<T: Send + 'static>(
        &self,
        fut: impl std::future::Future<Output = T> + Send + 'static,
    ) -> impl std::future::Future<Output = Result<T, String>> + Send + 'static {
        let handle = self.rt.spawn(fut);
        async move { handle.await.map_err(|e| e.to_string()) }
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
