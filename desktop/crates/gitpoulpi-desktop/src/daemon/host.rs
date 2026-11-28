//! The desktop's side of [`gitpoulpi_link::LinkHost`].
//!
//! Every link has its own [`DesktopHost`] (the link keeps the host it was started with and hands each
//! hook the [`LinkContext`] it comes from), so a second window or a second repository never shares
//! jobs, lines or prompts with this one. The hooks run on the link's thread, so the state is plain data
//! behind a mutex. What the window waits for is the [`Notifier`] the host bumps on `wake`: it outlives a
//! repository switch (the window's [`super::Backend`] owns it and hands it to each new host), so the
//! window's pump (see `app::live`) and the query futures of [`super::Backend::call`] keep one place to
//! wait on. The jobs, the "said" lines and the last success are what the status line shows; hooks with
//! no desktop surface yet answer neutrally. The undo journal moving also wakes the window
//! ([`LinkHost::wake_on_journal`]), which is how the toolbar's Undo and Redo follow it.

use std::collections::BTreeMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::{Arc, Mutex, MutexGuard};
use std::task::{Context, Poll, Waker};

use gitpoulpi_core::domain::RefFilter;
use gitpoulpi_core::privacy::Sensitive;
use gitpoulpi_core::NotCancellableReason;
use gitpoulpi_ipc::{HostingChangeArgs, JobInfo};
use gitpoulpi_link::error_text::{error_text, typed_text};
use gitpoulpi_link::{
    Action, JobEvent, LinkContext, LinkHost, PushGroupEvent, Report, Requester, StepOutcome,
    Ticket, TimingKind, UndoKey,
};

use super::activity::{Feed, FeedEntry, GroupRun};
use crate::strings;

#[derive(Default)]
struct Wakes {
    epoch: u64,
    wakers: Vec<Waker>,
}

/// What a window waits on: an epoch the link's hooks bump and the wakers waiting for the next bump.
#[derive(Default)]
pub struct Notifier {
    wakes: Mutex<Wakes>,
}

impl Notifier {
    fn lock(&self) -> MutexGuard<'_, Wakes> {
        self.wakes.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Something happened: wakes everything that waits.
    pub fn bump(&self) {
        let wakers = {
            let mut s = self.lock();
            s.epoch += 1;
            std::mem::take(&mut s.wakers)
        };
        for w in wakers {
            w.wake();
        }
    }

    /// A future that resolves once the notifier was bumped after `seen`; `seen` is updated to the
    /// epoch it resolved at.
    pub fn changed<'a>(&'a self, seen: &'a mut u64) -> Changed<'a> {
        Changed {
            notifier: self,
            seen,
        }
    }

    /// The epoch now, for a pump to start from.
    pub fn epoch(&self) -> u64 {
        self.lock().epoch
    }

    /// Registers `waker` for the next bump (a query future that found its answer missing).
    pub(super) fn register(&self, waker: &Waker) {
        let mut s = self.lock();
        if !s.wakers.iter().any(|w| w.will_wake(waker)) {
            s.wakers.push(waker.clone());
        }
    }
}

/// See [`Notifier::changed`].
pub struct Changed<'a> {
    notifier: &'a Notifier,
    seen: &'a mut u64,
}

impl Future for Changed<'_> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        // The epoch check and the registration share one lock, so a bump cannot slip in between.
        let mut s = self.notifier.lock();
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

#[derive(Default)]
struct State {
    jobs: BTreeMap<u64, JobInfo>,
    /// Lines for the status line.
    said: Vec<String>,
    /// The title of the last action that succeeded.
    done: Option<String>,
    /// Stages the staging guard refused, waiting for the window to ask the person.
    prompts: Vec<SensitivePrompt>,
    /// The Git operations tab of the Activity Log.
    feed: Feed,
    /// The grouped push (or pull) the link last reported.
    group: Option<GroupRun>,
}

/// A stage the guard refused because some files look like secrets: nothing was staged. The answer is
/// "Stage anyway" (the action again, wrapped in `Action::AllowSensitive`) or cancel (the ticket ends).
#[derive(Clone, Debug)]
pub struct SensitivePrompt {
    pub action: Action,
    pub files: Vec<Sensitive>,
    pub ticket: Option<Ticket>,
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
    notifier: Arc<Notifier>,
    state: Mutex<State>,
}

impl DesktopHost {
    /// The host of one link; `notifier` is what its wakes bump.
    pub fn new(notifier: Arc<Notifier>) -> Arc<DesktopHost> {
        Arc::new(DesktopHost {
            notifier,
            state: Mutex::new(State::default()),
        })
    }

    /// A host that wakes nobody in particular: for a second client of a session (a test, a helper).
    pub fn detached() -> Arc<DesktopHost> {
        Self::new(Arc::new(Notifier::default()))
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
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

    /// Every job the daemon still lists, the running ones first (then by id): the jobs panel.
    pub fn all_jobs(&self) -> Vec<JobInfo> {
        let mut jobs: Vec<JobInfo> = self.lock().jobs.values().cloned().collect();
        jobs.sort_by_key(|j| (j.state.is_finished(), j.id));
        jobs
    }

    /// The job with this id, finished or not.
    pub fn job(&self, id: u64) -> Option<JobInfo> {
        self.lock().jobs.get(&id).cloned()
    }

    /// The Git operations feed, oldest first.
    pub fn feed(&self) -> Vec<FeedEntry> {
        self.lock().feed.snapshot()
    }

    /// Empties the Git operations feed.
    pub fn clear_feed(&self) {
        self.lock().feed.clear();
        super::activity::set_error_present(false);
        self.notifier.bump();
    }

    /// The newest failed operation, for "copy / open the last error".
    pub fn last_error(&self) -> Option<FeedEntry> {
        self.lock().feed.last_error()
    }

    /// The grouped push the link last reported, if any.
    pub fn group_run(&self) -> Option<GroupRun> {
        self.lock().group.clone()
    }

    /// What was reported since the last call.
    pub fn take_news(&self) -> News {
        let mut s = self.lock();
        News {
            said: std::mem::take(&mut s.said),
            done: s.done.take(),
        }
    }

    /// The refused stages that no dialog asked about yet.
    pub fn take_prompts(&self) -> Vec<SensitivePrompt> {
        std::mem::take(&mut self.lock().prompts)
    }

    fn say(&self, text: String) {
        self.lock().said.push(text);
        self.notifier.bump();
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
    fn wake(&self, _link: &LinkContext) {
        self.notifier.bump();
    }

    fn wake_on_journal(&self) -> bool {
        true
    }

    fn before_exit(&self, _link: &LinkContext) {}

    fn raise_window(&self, _link: &LinkContext) -> Result<String, String> {
        Err(strings::NO_RAISE.to_string())
    }

    fn ask_sensitive_stage(
        &self,
        _link: &LinkContext,
        _root: PathBuf,
        action: Action,
        files: Vec<Sensitive>,
        ticket: Option<Ticket>,
    ) {
        // The window of this link asks (`app::live` takes the prompt on its next wake); the ticket
        // stays waiting.
        gitpoulpi_link::report_ticket(
            ticket,
            StepOutcome::NeedsUser(gitpoulpi_link::UserWait::SensitiveStage),
        );
        self.lock().prompts.push(SensitivePrompt {
            action,
            files,
            ticket,
        });
        self.notifier.bump();
    }

    fn on_connected(&self, _link: &LinkContext, _requester: Requester) {
        // The link's requester is reached through its activity handle (see `Backend::open_link`).
        self.notifier.bump();
    }

    fn jobs(&self, _link: &LinkContext, event: JobEvent) {
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
                JobEvent::Say(text) => s.said.push(text),
            }
        }
        super::activity::publish_jobs(self.all_jobs());
        self.notifier.bump();
    }

    fn push_group(&self, _link: &LinkContext, event: PushGroupEvent<'_>) {
        {
            let mut s = self.lock();
            match &event {
                PushGroupEvent::Started { request, .. } => {
                    GroupRun::on_started(&mut s.group, request)
                }
                PushGroupEvent::Outcome { outcome, .. } => {
                    GroupRun::on_outcome(&mut s.group, outcome)
                }
                PushGroupEvent::Failed { why, .. } => GroupRun::on_failed(&mut s.group, why),
                PushGroupEvent::Progress {
                    run_id,
                    remote,
                    status,
                } => GroupRun::on_progress(&mut s.group, run_id, remote, status.clone()),
            }
        }
        if let PushGroupEvent::Failed { why, .. } = event {
            self.say(why.to_string());
        } else {
            self.notifier.bump();
        }
    }

    fn report(&self, _link: &LinkContext, report: Report<'_>) {
        use gitpoulpi_i18n::msg::activity_log as m;
        use gitpoulpi_log::Level;
        let ms = |d: std::time::Duration| Some(d.as_millis() as u64);
        let name = |action: &Action| title_of(action).unwrap_or_else(|| action.activity_label());
        match report {
            Report::Success {
                action,
                repo,
                elapsed,
            } => {
                let mut s = self.lock();
                s.feed.push(
                    Level::Info,
                    name(action),
                    Vec::new(),
                    Some(repo.to_path_buf()),
                    ms(elapsed),
                );
                if let Some(title) = title_of(action) {
                    s.done = Some(title);
                }
                drop(s);
                self.notifier.bump();
            }
            // The failures also reach the window typed, as `LinkEvent`s of the link's own drain.
            Report::Failure {
                action,
                error,
                repo,
                elapsed,
                context,
            } => {
                let details = vec![
                    gitpoulpi_core::redact(&error_text(error)),
                    m::ctx_operation(context.describe()),
                    m::ctx_repository(repo.display().to_string()),
                ];
                super::activity::set_error_present(true);
                self.lock().feed.push(
                    Level::Error,
                    m::failed_title(name(action)),
                    details,
                    Some(repo.to_path_buf()),
                    ms(elapsed),
                );
                self.notifier.bump();
            }
            Report::Rejection {
                action,
                rejection,
                repo,
                elapsed,
            } => {
                self.lock().feed.push(
                    Level::Warn,
                    m::failed_title(name(action)),
                    vec![rejection_text(rejection)],
                    Some(repo.to_path_buf()),
                    ms(elapsed),
                );
                self.notifier.bump();
            }
            Report::Message { text, repo } => {
                self.lock().feed.push(
                    Level::Info,
                    text.to_string(),
                    Vec::new(),
                    Some(repo.to_path_buf()),
                    None,
                );
                self.say(text.to_string())
            }
            Report::BackgroundFetchSkipped { .. } => {}
        }
    }

    fn timing(&self, _link: &LinkContext, kind: TimingKind, repo: &Path, label: String, ms: u64) {
        // What the person waited for is already a line of its own (`report`); the background polls
        // are kept at the Debug level, which the Activity Log hides by default.
        if kind == TimingKind::Background {
            self.lock().feed.push(
                gitpoulpi_log::Level::Debug,
                label,
                Vec::new(),
                Some(repo.to_path_buf()),
                Some(ms),
            );
            self.notifier.bump();
        }
    }

    fn expect_background_fetch(&self, _link: &LinkContext) {}

    fn background_fetch_pending(&self, _link: &LinkContext) -> bool {
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
