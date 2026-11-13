//! The "File manager menu" rows of the Launcher page: "Open in GitPoulpi" in
//! the right-click menu of the file manager (`gitpoulpi_config::desktop::
//! folder_menu`). Same shape as the alias rows (`launcher_alias`): the machine
//! is read on a thread when Settings opens, every Install / Remove runs on a
//! thread too (Windows spawns PowerShell for the registry), and a button
//! pressed while a job runs queues behind it.
//!
//! The entry runs `gitpoulpi launch --folder`, which builds the terminal
//! command when it is clicked, so changing the terminal never makes an entry
//! outdated; only a moved exe or another icon does, and
//! `launcher::sync_shortcuts` rewrites those by itself.

use super::*;
use gitpoulpi_config::desktop::folder_menu::{self, Manager, Status, Unavailable};
use gitpoulpi_config::desktop::State;
use gitpoulpi_config::open_in::Os;
use gitpoulpi_i18n::msg::launcher as t;
use std::sync::mpsc;
use std::sync::Arc;

/// The buttons of the File manager menu row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FolderMenuButton {
    Install,
    Remove,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Done {
    Installed(Vec<Manager>),
    Removed(Vec<Manager>),
    Failed(String),
}

struct JobResult {
    snapshot: Status,
    done: Option<Done>,
}

/// Where the machine comes from; the argument is the logo colorway (the
/// icon). Tests give it temp directories and a memory registry.
pub(super) type MakeEnv = Arc<dyn Fn(&str) -> folder_menu::Env + Send + Sync>;

/// The rows' own state.
pub(super) struct FolderMenuState {
    snapshot: Option<Status>,
    rx: Option<mpsc::Receiver<JobResult>>,
    pending: Option<FolderMenuButton>,
    env: MakeEnv,
}

impl Default for FolderMenuState {
    fn default() -> Self {
        Self {
            snapshot: None,
            rx: None,
            pending: None,
            env: Arc::new(|logo| {
                crate::launcher::folder_menu_env(&crate::launcher::Env::real(), Some(logo))
            }),
        }
    }
}

impl FolderMenuState {
    /// A job runs or waits.
    pub(super) fn busy(&self) -> bool {
        self.rx.is_some() || self.pending.is_some()
    }

    /// The machine was read at least once.
    pub(super) fn has_read(&self) -> bool {
        self.snapshot.is_some()
    }

    #[cfg(test)]
    pub(super) fn is_idle(&self) -> bool {
        !self.busy()
    }
}

fn run(env: &folder_menu::Env, label: &str, button: FolderMenuButton) -> Done {
    match button {
        FolderMenuButton::Install => match folder_menu::install(env, label) {
            Ok(done) => Done::Installed(done),
            Err(e) => Done::Failed(e),
        },
        FolderMenuButton::Remove => match folder_menu::remove(env) {
            Ok(done) => Done::Removed(done),
            Err(e) => Done::Failed(e),
        },
    }
}

fn state_text(state: State) -> &'static str {
    match state {
        State::NotInstalled => t::STATE_NONE.get(),
        State::Installed => t::STATE_INSTALLED.get(),
        State::Outdated => t::STATE_OUTDATED.get(),
        State::Foreign => t::STATE_FOREIGN.get(),
    }
}

fn names(managers: &[Manager]) -> String {
    folder_menu::names(managers)
}

impl PreferencesState {
    /// Test hook: where the menu is written (never the real one).
    #[cfg(test)]
    pub(super) fn set_folder_menu_env(
        &mut self,
        env: impl Fn(&str) -> folder_menu::Env + Send + Sync + 'static,
    ) {
        self.folder_menu.env = Arc::new(env);
    }

    /// Starts the background job: run `button` if there is one, then read the
    /// machine. Queues the button behind a job already running.
    pub(super) fn folder_menu_start(&mut self, button: Option<FolderMenuButton>) {
        if self.folder_menu.rx.is_some() {
            if button.is_some() {
                self.folder_menu.pending = button;
            }
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.folder_menu.rx = Some(rx);
        let make = Arc::clone(&self.folder_menu.env);
        let logo = self.config.ui.logo_color.clone();
        let label = t::FOLDER_MENU_LABEL.get().to_string();
        std::thread::spawn(move || {
            let env = make(&logo);
            let done = button.map(|b| run(&env, &label, b));
            let snapshot = folder_menu::status(&env, &label);
            let _ = tx.send_notify(JobResult { snapshot, done });
        });
    }

    /// Applies a finished job. Never blocks; called once a frame.
    pub(crate) fn poll_folder_menu(&mut self) {
        let Some(rx) = &self.folder_menu.rx else {
            return;
        };
        let Ok(result) = rx.try_recv() else { return };
        self.folder_menu.rx = None;
        if let Some(e) = &result.snapshot.error {
            crate::activity_log::report_problem(
                crate::activity_log::ProblemSource::Tool,
                &t::failed(e),
            );
        }
        self.folder_menu.snapshot = Some(result.snapshot);
        if let Some(done) = result.done {
            if let Done::Failed(e) = &done {
                crate::activity_log::report_problem(
                    crate::activity_log::ProblemSource::Tool,
                    &t::failed(e),
                );
            }
            self.message = Some(Self::folder_menu_message(&done));
        }
        if let Some(button) = self.folder_menu.pending.take() {
            self.folder_menu_start(Some(button));
        }
        self.apply_pending_focus();
    }

    fn folder_menu_message(done: &Done) -> String {
        match done {
            Done::Installed(m) if m.is_empty() => t::FOLDER_MENU_NONE_DETECTED.get().into(),
            Done::Installed(m) => t::done_folder_menu_installed(names(m)),
            Done::Removed(m) if m.is_empty() => t::DONE_FOLDER_MENU_NOTHING.get().into(),
            Done::Removed(m) => t::done_folder_menu_removed(names(m)),
            Done::Failed(e) => t::failed(e),
        }
    }

    pub(super) fn folder_menu_button(&mut self, button: FolderMenuButton) {
        if let Some(s) = &self.folder_menu.snapshot {
            if s.unavailable == Some(Unavailable::NoHelper) {
                self.message = Some(t::FOLDER_MENU_NEEDS_HELPER.get().to_string());
                return;
            }
        }
        self.message = Some(t::WORKING.get().to_string());
        self.folder_menu_start(Some(button));
    }

    /// The palette's route: show the Launcher group, run the button and leave
    /// the buttons row selected.
    pub(crate) fn folder_menu_command(&mut self, remove: bool) {
        let _ = self.show_page(None, Some("ui.launch_terminal"));
        let button = if remove {
            FolderMenuButton::Remove
        } else {
            FolderMenuButton::Install
        };
        self.folder_menu_button(button);
        self.focus_after_job(&[
            PrefAction::FolderMenu(FolderMenuButton::Install),
            PrefAction::FolderMenu(FolderMenuButton::Remove),
        ]);
    }

    /// The heading, one state row per file manager, and the buttons.
    pub(super) fn folder_menu_rows(&self) -> Vec<Field> {
        let mut fields = vec![Field::heading(t::FOLDER_MENU_HEADING.get())];
        let os = Os::current();
        let snap = self.folder_menu.snapshot.as_ref();
        let info =
            |label: &str, text: String| Field::plain(label, Control::Info(text), Target::None);
        match snap {
            None => fields.push(info(t::FOLDER_MENU_STATUS.get(), t::CHECKING.get().into())),
            Some(s) if s.unavailable.is_some() => fields.push(info(
                t::FOLDER_MENU_STATUS.get(),
                t::FOLDER_MENU_NEEDS_HELPER.get().into(),
            )),
            Some(s) if os != Os::Linux => {
                let state = s.entries.first().map_or(State::NotInstalled, |e| e.state);
                fields.push(info(
                    t::FOLDER_MENU_STATUS.get(),
                    state_text(state).to_string(),
                ));
            }
            Some(s) => {
                // Linux: a row for every file manager found here, and for any
                // that still holds our entry.
                let shown: Vec<_> = s
                    .entries
                    .iter()
                    .filter(|e| e.detected || e.state != State::NotInstalled)
                    .collect();
                for e in &shown {
                    fields.push(info(e.manager.name(), state_text(e.state).to_string()));
                }
                let found: Vec<Manager> = s
                    .entries
                    .iter()
                    .filter(|e| e.detected)
                    .map(|e| e.manager)
                    .collect();
                let detected = if found.is_empty() {
                    t::FOLDER_MENU_NONE_DETECTED.get().to_string()
                } else {
                    names(&found)
                };
                fields.push(info(t::FOLDER_MENU_DETECTED.get(), detected));
            }
        }
        let ready = snap.is_some_and(|s| s.unavailable.is_none());
        let mut install = Btn::new(
            t::FOLDER_MENU_INSTALL.get(),
            PrefAction::FolderMenu(FolderMenuButton::Install),
        );
        let mut remove = Btn::new(
            t::FOLDER_MENU_REMOVE.get(),
            PrefAction::FolderMenu(FolderMenuButton::Remove),
        );
        install.enabled = ready
            && snap.is_some_and(|s| {
                !folder_menu::all_installed(s)
                    && s.entries
                        .iter()
                        .any(|e| e.detected && e.state != State::Foreign)
            });
        remove.enabled = snap.is_some_and(folder_menu::has_any_ours);
        let help = match os {
            Os::Windows => t::FOLDER_MENU_HELP_WINDOWS.get(),
            Os::Linux => t::FOLDER_MENU_HELP_LINUX.get(),
            Os::Macos => t::FOLDER_MENU_HELP_MACOS.get(),
        };
        fields.push(Field::buttons("", vec![install, remove]).with_help(help));
        fields
    }
}
