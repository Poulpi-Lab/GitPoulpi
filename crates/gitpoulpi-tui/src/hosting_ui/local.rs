//! The local Markdown issue source (slice L1) in the ISSUES section: the
//! row-menu and palette actions (close, reopen, open in Obsidian, create
//! the Base) and the folder watch that makes an edit made outside (Obsidian,
//! a shell) show up.
//!
//! Every write runs on its own thread and answers through a channel drained
//! in [`HostingUi::poll`]; the watch fingerprints the folder on a thread too,
//! so a slow disk (a vault on a network share) never holds up a frame.

use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use super::*;
use crate::redraw::NotifySend as _;

/// How often the folder is looked at.
const WATCH_EVERY: Duration = Duration::from_millis(750);

/// The folder watch of the linked local source.
#[derive(Default)]
pub(super) struct LocalWatch {
    dir: Option<PathBuf>,
    /// The last fingerprint seen; the first look only records it.
    seen: Option<u64>,
    last: Option<Instant>,
    rx: Option<mpsc::Receiver<u64>>,
}

impl LocalWatch {
    pub(super) fn deadline(&self) -> Option<Instant> {
        self.dir.as_ref()?;
        self.last.map(|at| at + WATCH_EVERY)
    }
}

/// A finished write: the message to say, or what went wrong.
pub(super) type LocalJob = mpsc::Receiver<Result<String, String>>;

#[cfg(feature = "local-issues")]
mod io {
    use std::path::{Path, PathBuf};

    pub(super) fn fingerprint(dir: &Path) -> u64 {
        gitpoulpi_hosting::local_markdown::Store::fingerprint(dir)
    }

    /// The `obsidian://` link of the note a `file://` URL names, when the
    /// note is inside a vault.
    pub(super) fn obsidian_url(file_url: &str) -> Option<(PathBuf, String)> {
        let path = gitpoulpi_hosting::local_markdown::path_from_file_url(file_url)?;
        let url = gitpoulpi_hosting::local_markdown::obsidian::open_url(&path)?;
        Some((path, url))
    }

    pub(super) fn create_base(folder: &Path, statuses: &[String]) -> Result<PathBuf, String> {
        gitpoulpi_hosting::local_markdown::obsidian::create_base(folder, statuses)
            .map_err(|e| e.to_string())
    }
}

#[cfg(not(feature = "local-issues"))]
mod io {
    use std::path::{Path, PathBuf};

    pub(super) fn fingerprint(_dir: &Path) -> u64 {
        0
    }

    pub(super) fn obsidian_url(_file_url: &str) -> Option<(PathBuf, String)> {
        None
    }

    pub(super) fn create_base(_folder: &Path, _statuses: &[String]) -> Result<PathBuf, String> {
        Err("this build was made without the local-issues feature".into()) // i18n-english
    }
}

impl HostingUi {
    /// The linked ISSUES source is a local Markdown folder.
    pub(super) fn issues_local(&self) -> bool {
        self.snapshot
            .as_ref()
            .and_then(|s| s.issues_target.as_ref())
            .is_some_and(|t| t.profile.kind.capabilities().local_folder)
    }

    /// The folder the local source reads, absolute.
    fn local_folder(&self) -> Option<PathBuf> {
        let target = self.snapshot.as_ref()?.issues_target.as_ref()?;
        (target.profile.kind.capabilities().local_folder && !target.project.is_empty())
            .then(|| gitpoulpi_hosting::resolve_local_folder(&self.repo_root, &target.project))
    }

    /// What a write needs: the profile, every profile (the token source reads
    /// them all) and the folder.
    fn local_job_inputs(&self) -> Option<(Integration, Vec<Integration>, String)> {
        let target = self.snapshot.as_ref()?.issues_target.as_ref()?;
        let all = (self.integrations)();
        let integration = all.iter().find(|i| i.id == target.profile.id)?.clone();
        let folder = self.local_folder()?.to_string_lossy().into_owned();
        Some((integration, all, folder))
    }

    /// The issue's `#3 Title` caption.
    fn issue_caption(&self, number: u64) -> String {
        self.snapshot
            .as_ref()
            .and_then(|s| s.issues.iter().find(|i| i.number == number))
            .map(|i| format!("{} {}", i.iid_label, i.title))
            .unwrap_or_else(|| format!("#{number}"))
    }

    fn start_local(&mut self, work: impl FnOnce() -> Result<String, String> + Send + 'static) {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send_notify(work());
        });
        self.local_jobs.push(rx);
    }

    /// Close (`closed`) or reopen issue `number`.
    pub(super) fn local_set_state(&mut self, number: u64, closed: bool) -> HostingEffect {
        let (Some((integration, all, folder)), Some(factory)) =
            (self.local_job_inputs(), self.factory.clone())
        else {
            return HostingEffect::Message(msg::local_issues::WHY_NOT_LOCAL.get().into());
        };
        let caption = self.issue_caption(number);
        let said = if closed {
            msg::local_issues::closed_issue(&caption)
        } else {
            msg::local_issues::reopened_issue(&caption)
        };
        self.start_local(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(crate::hosting_task::request_task_failed)?;
            let client = factory
                .client(&integration, &all)
                .map_err(|e| crate::hosting_task::error_text(&e))?;
            runtime
                .block_on(client.set_issue_state(&folder, number, closed))
                .map_err(|e| crate::hosting_task::error_text(&e))?;
            Ok(said)
        });
        HostingEffect::None
    }

    /// Writes `issues.base` next to the notes (once; never over a file).
    pub(super) fn local_create_base(&mut self) -> HostingEffect {
        let (Some((integration, all, folder)), Some(factory)) =
            (self.local_job_inputs(), self.factory.clone())
        else {
            return HostingEffect::Message(msg::local_issues::WHY_NOT_LOCAL.get().into());
        };
        self.start_local(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(crate::hosting_task::request_task_failed)?;
            let client = factory
                .client(&integration, &all)
                .map_err(|e| crate::hosting_task::error_text(&e))?;
            let statuses = runtime
                .block_on(client.issue_statuses(&folder))
                .map_err(|e| crate::hosting_task::error_text(&e))?;
            let path = io::create_base(std::path::Path::new(&folder), &statuses)?;
            Ok(msg::local_issues::base_created(path.display()))
        });
        HostingEffect::None
    }

    /// Opens the note behind `url` (a `file://` URL) in Obsidian.
    pub(super) fn local_open_obsidian(&mut self, url: &str) -> HostingEffect {
        let Some((path, link)) = io::obsidian_url(url) else {
            return HostingEffect::Message(msg::local_issues::WHY_NO_VAULT.get().into());
        };
        match crate::repo_menu::open_url(&link) {
            Ok(()) => HostingEffect::Message(msg::local_issues::opening_obsidian(path.display())),
            Err(err) => HostingEffect::Message(crate::activity_log::report_problem_clear(
                crate::activity_log::ProblemSource::Tool,
                &err,
            )),
        }
    }

    /// Whether the note behind `url` is inside an Obsidian vault.
    pub(super) fn in_vault(url: &str) -> bool {
        io::obsidian_url(url).is_some()
    }

    /// The row menu's extra entries for a local issue.
    pub(super) fn local_entries(issue: &IssueSummary) -> Vec<Entry<HostingAction>> {
        let obsidian = if Self::in_vault(&issue.web_url) {
            Entry::action(
                msg::local_issues::OPEN_IN_OBSIDIAN.get(),
                HostingAction::OpenObsidian(issue.web_url.clone()),
            )
        } else {
            Entry::disabled(
                msg::local_issues::OPEN_IN_OBSIDIAN.get(),
                msg::local_issues::WHY_NO_VAULT_SHORT.get(),
            )
        };
        let state = if issue.state == IssueState::Closed {
            Entry::action(
                msg::local_issues::REOPEN_ISSUE.get(),
                HostingAction::LocalState {
                    number: issue.number,
                    closed: false,
                },
            )
        } else {
            Entry::action(
                msg::local_issues::CLOSE_ISSUE.get(),
                HostingAction::LocalState {
                    number: issue.number,
                    closed: true,
                },
            )
        };
        vec![obsidian, state]
    }

    /// A menu of the listed issues for a palette command: `pick` chooses the
    /// action each one runs.
    pub(super) fn local_pick_menu(
        &mut self,
        title: &'static str,
        anchor: (u16, u16),
        wanted: impl Fn(&IssueSummary) -> bool,
        pick: impl Fn(&IssueSummary) -> HostingAction,
    ) -> HostingEffect {
        let entries: Vec<Entry<HostingAction>> = self
            .snapshot
            .as_ref()
            .map(|s| {
                s.issues
                    .iter()
                    .filter(|i| wanted(i))
                    .take(40)
                    .map(|i| Entry::action(format!("{}  {}", i.iid_label, i.title), pick(i)))
                    .collect()
            })
            .unwrap_or_default();
        if entries.is_empty() {
            return HostingEffect::Message(msg::hosting_ui::WHY_NO_ISSUES.get().into());
        }
        self.menu = Some(ContextMenu::new(title, entries, anchor.0, anchor.1));
        HostingEffect::None
    }

    /// Drains the finished writes. True when something on screen changed.
    pub(super) fn poll_local(&mut self) -> bool {
        let mut done: Vec<Result<String, String>> = Vec::new();
        self.local_jobs.retain(|rx| match rx.try_recv() {
            Ok(answer) => {
                done.push(answer);
                false
            }
            Err(mpsc::TryRecvError::Empty) => true,
            Err(mpsc::TryRecvError::Disconnected) => false,
        });
        let mut changed = !done.is_empty();
        for answer in done {
            match answer {
                Ok(said) => self.notice = Some(said),
                Err(why) => {
                    self.notice = Some(crate::activity_log::report_problem_clear(
                        crate::activity_log::ProblemSource::Hosting,
                        &msg::local_issues::failed(why),
                    ));
                }
            }
            // The list shows what the folder holds now.
            self.demand();
        }
        // The folder watch's fingerprint.
        if let Some(rx) = &self.local_watch.rx {
            if let Ok(print) = rx.try_recv() {
                self.local_watch.rx = None;
                if self.local_watch.seen.is_some_and(|before| before != print) {
                    self.demand();
                    changed = true;
                }
                self.local_watch.seen = Some(print);
            }
        }
        changed
    }

    /// Per frame: starts a fingerprint of the linked folder when one is due.
    pub(super) fn tick_local(&mut self, now: Instant) {
        let dir = self.local_folder();
        if dir != self.local_watch.dir {
            self.local_watch = LocalWatch {
                dir,
                ..LocalWatch::default()
            };
        }
        let Some(dir) = self.local_watch.dir.clone() else {
            return;
        };
        if self.local_watch.rx.is_some()
            || self
                .local_watch
                .last
                .is_some_and(|at| now.duration_since(at) < WATCH_EVERY)
        {
            return;
        }
        self.local_watch.last = Some(now);
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send_notify(io::fingerprint(&dir));
        });
        self.local_watch.rx = Some(rx);
    }
}
