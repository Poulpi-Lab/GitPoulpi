//! The desktop's side of [`gitpoulpi_link::LinkHost`].
//!
//! The link runs on its own thread and calls the host from there, so everything here is plain shared
//! state behind a mutex. `wake` bumps an epoch and wakes the futures waiting for news: the window's
//! pump (see `app::live`) and the query futures of [`super::Backend::call`]. The jobs, the "said" lines
//! and the last success are what the status line shows; hooks with no desktop surface yet answer
//! neutrally.
//!
//! The link keeps its host in a process-wide slot, so one host serves every link of the process; it is a
//! singleton ([`DesktopHost::global`]) and `wake` is a broadcast. A second window or a test with two
//! repositories therefore shares the jobs and the lines; a link-scoped host is a request for `main`.

use std::collections::BTreeMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::task::{Context, Poll, Waker};

use gitpoulpi_core::domain::RefFilter;
use gitpoulpi_core::privacy::Sensitive;
use gitpoulpi_core::NotCancellableReason;
use gitpoulpi_ipc::{HostingChangeArgs, JobInfo};
use gitpoulpi_link::error_text::{error_text, typed_text};
use gitpoulpi_link::{
    Action, JobEvent, LinkHost, PushGroupEvent, Report, Requester, StepOutcome, Ticket, TimingKind,
    UndoKey,
};

use crate::strings;

#[derive(Default)]
struct Shared {
    epoch: u64,
    wakers: Vec<Waker>,
    jobs: BTreeMap<u64, JobInfo>,
    /// Lines for the status line; `None` for those that belong to no repository.
    said: Vec<(Option<PathBuf>, String)>,
    /// The last success per repository.
    done: BTreeMap<PathBuf, String>,
}

/// What the link told the window since the last [`DesktopHost::take_news`].
#[derive(Debug, Default, PartialEq, Eq)]
pub struct News {
    /// Lines for the status line (a job joined, a grouped push failed, a request this frontend cannot
    /// answer yet).
    pub said: Vec<String>,
    /// The title of the last action that succeeded.
    pub done: Option<String>,
}

pub struct DesktopHost {
    shared: Mutex<Shared>,
}

impl DesktopHost {
    /// The one host of the process.
    pub fn global() -> Arc<DesktopHost> {
        static HOST: OnceLock<Arc<DesktopHost>> = OnceLock::new();
        HOST.get_or_init(|| {
            Arc::new(DesktopHost {
                shared: Mutex::new(Shared::default()),
            })
        })
        .clone()
    }

    fn lock(&self) -> MutexGuard<'_, Shared> {
        self.shared.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn bump(&self) {
        let wakers = {
            let mut s = self.lock();
            s.epoch += 1;
            std::mem::take(&mut s.wakers)
        };
        for w in wakers {
            w.wake();
        }
    }

    /// A future that resolves once the link woke the host after `seen`; `seen` is updated to the epoch
    /// it resolved at.
    pub fn changed<'a>(&'a self, seen: &'a mut u64) -> Changed<'a> {
        Changed { host: self, seen }
    }

    /// The epoch now, for a pump to start from.
    pub fn epoch(&self) -> u64 {
        self.lock().epoch
    }

    /// Registers `waker` for the next wake (a query future that found its answer missing).
    pub(super) fn register(&self, waker: &Waker) {
        let mut s = self.lock();
        if !s.wakers.iter().any(|w| w.will_wake(waker)) {
            s.wakers.push(waker.clone());
        }
    }

    /// The jobs that have not ended, oldest first: what the status line counts.
    pub fn live_jobs(&self) -> Vec<JobInfo> {
        self.lock()
            .jobs
            .values()
            .filter(|j| !j.state.is_finished())
            .cloned()
            .collect()
    }

    /// What was reported for the repository at `root` (and what belongs to no repository).
    pub fn take_news(&self, root: &Path) -> News {
        let mut s = self.lock();
        let (mine, others): (Vec<_>, Vec<_>) = std::mem::take(&mut s.said)
            .into_iter()
            .partition(|(repo, _)| repo.as_deref().is_none_or(|r| r == root));
        s.said = others;
        News {
            said: mine.into_iter().map(|(_, text)| text).collect(),
            done: s.done.remove(root),
        }
    }

    fn say(&self, repo: Option<&Path>, text: String) {
        self.lock().said.push((repo.map(Path::to_path_buf), text));
        self.bump();
    }
}

/// See [`DesktopHost::changed`].
pub struct Changed<'a> {
    host: &'a DesktopHost,
    seen: &'a mut u64,
}

impl Future for Changed<'_> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        // The epoch check and the registration share one lock, so a wake cannot slip in between.
        let mut s = self.host.lock();
        if s.epoch != *self.seen {
            let epoch = s.epoch;
            drop(s);
            *self.seen = epoch;
            return Poll::Ready(());
        }
        if !s.wakers.iter().any(|w| w.will_wake(cx.waker())) {
            s.wakers.push(cx.waker().clone());
        }
        Poll::Pending
    }
}

/// The title the status line gives a finished action, for the ones the desktop starts.
fn title_of(action: &Action) -> Option<String> {
    use gitpoulpi_i18n::msg;
    Some(match action {
        Action::AutoStashed(inner) | Action::AllowSensitive(inner) => return title_of(inner),
        Action::Fetch(_) | Action::FetchAll => msg::menu_ref::FETCH.get().to_string(),
        Action::Pull(_) | Action::PullThenPush { .. } => msg::toolbar::PULL.get().to_string(),
        Action::Push { .. } | Action::FetchThenForcePush { .. } => {
            msg::toolbar::PUSH.get().to_string()
        }
        Action::Undo { .. } | Action::UndoSequence { .. } => msg::toolbar::UNDO.get().to_string(),
        Action::Redo { .. } => msg::toolbar::REDO.get().to_string(),
        _ => return None,
    })
}

impl LinkHost for DesktopHost {
    fn wake(&self) {
        self.bump();
    }

    fn before_exit(&self) {}

    fn raise_window(&self) -> Result<String, String> {
        Err(strings::NO_RAISE.to_string())
    }

    fn ask_sensitive_stage(
        &self,
        _root: std::path::PathBuf,
        _action: Action,
        files: Vec<Sensitive>,
        ticket: Option<Ticket>,
    ) {
        // No dialog yet: the stage is not done, and the person is told where to confirm it.
        gitpoulpi_link::report_ticket(ticket, StepOutcome::Canceled);
        self.say(None, strings::sensitive_stage(files.len()));
    }

    fn on_connected(&self, _requester: Requester) {
        // Each link's own requester is reached through its activity handle (see `Backend::requester`).
        self.bump();
    }

    fn jobs(&self, event: JobEvent) {
        {
            let mut s = self.lock();
            match event {
                JobEvent::Seed(list) => s.jobs = list.into_iter().map(|j| (j.id, j)).collect(),
                JobEvent::Changed(info) => {
                    s.jobs.insert(info.id, info);
                }
                JobEvent::Removed(id) => {
                    s.jobs.remove(&id);
                }
                JobEvent::Say(text) => s.said.push((None, text)),
            }
        }
        self.bump();
    }

    fn push_group(&self, event: PushGroupEvent<'_>) {
        if let PushGroupEvent::Failed { root, why } = event {
            self.say(Some(root), why.to_string());
        }
    }

    fn report(&self, report: Report<'_>) {
        match report {
            Report::Success { action, repo, .. } => {
                if let Some(title) = title_of(action) {
                    self.lock().done.insert(repo.to_path_buf(), title);
                    self.bump();
                }
            }
            // The failures reach the window typed on the link's own channels.
            Report::Failure { .. } | Report::Rejection { .. } => {}
            Report::Message { text, repo } => self.say(Some(repo), text.to_string()),
            Report::BackgroundFetchSkipped { .. } => {}
        }
    }

    fn timing(&self, _kind: TimingKind, _repo: &Path, _label: String, _ms: u64) {}

    fn expect_background_fetch(&self) {}

    fn background_fetch_pending(&self) -> bool {
        false
    }

    fn job_label(&self, id: u64) -> String {
        self.lock()
            .jobs
            .get(&id)
            .map_or_else(|| format!("#{id}"), |j| j.label.clone())
    }

    fn not_cancellable(&self, _reason: NotCancellableReason) -> &'static str {
        ""
    }

    fn initial_log_filter(&self, _root: &Path) -> RefFilter {
        RefFilter::default()
    }

    fn undo_key(&self, which: UndoKey) -> String {
        match which {
            UndoKey::Undo => "Ctrl+Z".to_string(),
            UndoKey::UndoOffer => String::new(),
        }
    }

    fn daemon_job_live(&self) -> bool {
        !self.live_jobs().is_empty()
    }

    fn is_own_hosting_change(&self, _change: &HostingChangeArgs) -> bool {
        false
    }
}

/// The text of a failed request, as the status line words it.
pub fn failure_text(error: &gitpoulpi_core::Error) -> String {
    error_text(error)
}

/// The text of a refused push.
pub fn rejection_text(rejection: &gitpoulpi_core::domain::PushRejection) -> String {
    typed_text(&gitpoulpi_core::TypedError::PushRejected(rejection.clone()))
}
