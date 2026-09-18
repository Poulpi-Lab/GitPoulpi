//! The Launcher group at the end of Settings > General: the terminal shortcuts
//! start GitPoulpi in, and one state / Create / Remove trio per place
//! the OS offers (`gitpoulpi_config::desktop`). There is no Update button: a
//! shortcut we made follows the settings by itself (`launcher::sync_shortcuts`,
//! run before every read here and by `launcher_sync` while a TUI runs).
//!
//! The machine (installed terminals, shortcut states) is read on a thread when
//! Settings opens (`launcher_start`), and so is every Create /
//! Remove: Windows spawns PowerShell for a `.lnk`. Rows say "checking..."
//! until the first answer; a button pressed while a job runs queues behind it.

use super::*;
use crate::launcher::{self, Done, Op, Snapshot};
use gitpoulpi_config::desktop::{Place, State};
use gitpoulpi_config::open_in::Os;
use gitpoulpi_i18n::msg::launcher as t;
use std::sync::mpsc;
use std::sync::Arc;

/// A shortcut button of the page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LauncherButton {
    Install(Place),
    Remove(Place),
}

struct JobResult {
    snapshot: Snapshot,
    done: Option<Done>,
}

/// The page's own state.
pub(super) struct LauncherState {
    snapshot: Option<Snapshot>,
    rx: Option<mpsc::Receiver<JobResult>>,
    /// A button pressed while a job ran: started when it ends.
    pending: Option<Op>,
    /// The `ui.launch_terminal` (and launcher build) the snapshot was read
    /// for: a different choice in the page re-reads (and syncs) at once.
    for_terminal: Option<String>,
    /// The running exe, for dev-mode detection.
    exe: std::path::PathBuf,
    /// Where the machine comes from; tests give it temp directories.
    env: Arc<dyn Fn() -> launcher::Env + Send + Sync>,
}

impl Default for LauncherState {
    fn default() -> Self {
        Self {
            snapshot: None,
            rx: None,
            pending: None,
            for_terminal: None,
            exe: std::env::current_exe().unwrap_or_default(),
            env: Arc::new(launcher::Env::real),
        }
    }
}

/// Whether `reason` is one of this page's own notes (shown as the row's value
/// as is, not behind "coming").
pub(super) fn is_note(reason: &str) -> bool {
    [
        t::CHECKING,
        t::STATE_NONE,
        t::STATE_FOREIGN,
        t::WINDOWS_ONLY,
    ]
    .iter()
    .any(|m| m.get() == reason)
}

fn place_label(place: Place) -> &'static str {
    match (place, Os::current()) {
        (Place::Launcher, Os::Macos) => t::PLACE_APPLICATIONS.get(),
        (Place::Launcher, _) => t::PLACE_LAUNCHER.get(),
        (Place::Desktop, _) => t::PLACE_DESKTOP.get(),
        (Place::StartMenu, _) => t::PLACE_START_MENU.get(),
    }
}

impl LauncherState {
    #[cfg(test)]
    pub(super) fn is_idle(&self) -> bool {
        self.rx.is_none() && self.pending.is_none()
    }
}

impl PreferencesState {
    /// Test hook: where shortcuts are made (never the real home).
    #[cfg(test)]
    pub(super) fn set_launcher_env(
        &mut self,
        env: impl Fn() -> launcher::Env + Send + Sync + 'static,
    ) {
        self.launcher.env = Arc::new(env);
    }

    /// Starts the background job: read the machine, after running `op` if
    /// there is one. Queues `op` behind a job already running.
    pub(super) fn launcher_start(&mut self, op: Option<Op>) {
        if self.launcher.rx.is_some() {
            if op.is_some() {
                self.launcher.pending = op;
            }
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.launcher.rx = Some(rx);
        let make_env = Arc::clone(&self.launcher.env);
        let setting = self.config.ui.launch_terminal.clone();
        let logo = self.config.ui.logo_color.clone();
        let config = self.config.clone();
        self.launcher.for_terminal = Some(self.launcher_key());
        std::thread::spawn(move || {
            let mut env = make_env();
            env.newest = gitpoulpi_config::dev_mode::launcher_newest(&config, &env.exe);
            let done = op.map(|op| launcher::run(&env, &setting, &logo, op));
            // A shortcut of ours never reads "outdated": it is rewritten first.
            launcher::sync_shortcuts(&env, &setting, &logo);
            let snapshot = launcher::snapshot(&env, &setting, &logo);
            let _ = tx.send_notify(JobResult { snapshot, done });
        });
    }

    /// What the snapshot depends on besides the machine: the terminal and
    /// whether shortcuts follow the newest build.
    fn launcher_key(&self) -> String {
        format!(
            "{} {}",
            self.config.ui.launch_terminal,
            self.launcher_newest()
        )
    }

    /// Dev mode with `ui.launcher_build = "newest"`.
    pub(super) fn launcher_newest(&self) -> bool {
        gitpoulpi_config::dev_mode::launcher_newest(&self.config, &self.launcher.exe)
    }

    /// Developer mode (`general.dev_mode`, or detected from the exe's place
    /// in a checkout): the only time the build choice is offered.
    pub(super) fn dev_mode(&self) -> bool {
        gitpoulpi_config::dev_mode::dev_mode(&self.config, &self.launcher.exe)
    }

    /// Applies a finished job. Never blocks; called once a frame.
    pub(crate) fn poll_launcher(&mut self) {
        let Some(rx) = &self.launcher.rx else {
            if self.launcher.snapshot.is_some()
                && self.launcher.for_terminal.as_deref() != Some(self.launcher_key().as_str())
            {
                self.launcher_start(None);
            }
            return;
        };
        let Ok(result) = rx.try_recv() else { return };
        self.launcher.rx = None;
        self.launcher.snapshot = Some(result.snapshot);
        if let Some(done) = result.done {
            if matches!(done, Done::NoTerminal | Done::Failed(_)) {
                crate::activity_log::report_problem(
                    crate::activity_log::ProblemSource::Tool,
                    &Self::launcher_message(&done),
                );
            }
            self.message = Some(Self::launcher_message(&done));
        }
        if let Some(op) = self.launcher.pending.take() {
            self.launcher_start(Some(op));
        }
        self.apply_pending_focus();
    }

    pub(super) fn launcher_message(done: &Done) -> String {
        match done {
            Done::Installed(p) => t::done_created(place_label(*p)),
            Done::Removed(p) => t::done_removed(place_label(*p)),
            Done::NothingToRemove(p) => t::done_nothing(place_label(*p)),
            Done::NoTerminal => t::NO_TERMINAL.get().to_string(),
            Done::Failed(e) => t::failed(e),
        }
    }

    pub(super) fn launcher_button(&mut self, button: LauncherButton) {
        let op = match button {
            LauncherButton::Install(p) => Op::Install(p),
            LauncherButton::Remove(p) => Op::Remove(p),
        };
        self.message = Some(t::WORKING.get().to_string());
        self.launcher_start(Some(op));
    }

    /// The palette's route: show the Launcher rows and run `button` for the
    /// OS's first place (`desktop`: the Desktop folder).
    pub(crate) fn launcher_command(&mut self, desktop: bool, remove: bool) {
        let _ = self.show_page(None, Some("ui.launch_terminal"));
        let places = Place::for_os(Os::current());
        if remove {
            for place in places {
                self.launcher_button(LauncherButton::Remove(*place));
            }
            self.focus_after_job(&[
                PrefAction::Launcher(LauncherButton::Remove(places[0])),
                PrefAction::Launcher(LauncherButton::Install(places[0])),
            ]);
            return;
        }
        let place = if desktop { Place::Desktop } else { places[0] };
        self.launcher_button(LauncherButton::Install(place));
        self.focus_after_job(&[
            PrefAction::Launcher(LauncherButton::Install(place)),
            PrefAction::Launcher(LauncherButton::Remove(place)),
        ]);
    }

    /// Asks to select the first selectable of `actions` once the running
    /// jobs ended: until the machine was read the buttons are disabled, and
    /// a finished Create leaves its Remove button on the same row.
    pub(super) fn focus_after_job(&mut self, actions: &[PrefAction]) {
        self.tools.focus_after = actions.to_vec();
        self.apply_pending_focus();
    }

    /// Runs the request of [`Self::focus_after_job`] when no job is left.
    pub(super) fn apply_pending_focus(&mut self) {
        if self.tools.focus_after.is_empty()
            || self.launcher.rx.is_some()
            || self.launcher.pending.is_some()
            || self.tools.busy()
            || (self.launcher.snapshot.is_none() && !self.tools.has_read())
        {
            return;
        }
        for action in std::mem::take(&mut self.tools.focus_after) {
            if self.focus_action(action) {
                return;
            }
        }
    }

    /// Selects the button row of `action` on the open page, so the page
    /// scrolls to it (the Create / Remove rows sit below the fold of a tall
    /// terminal otherwise) and flashes it. `false`: no such enabled row.
    fn focus_action(&mut self, action: PrefAction) -> bool {
        // A disabled button cannot hold the selection: the row the page
        // opened on stays (the alias box, whose status row says why).
        let Some(at) = self
            .fields()
            .iter()
            .position(|f| f.target == Target::Action(action) && f.selectable())
        else {
            return false;
        };
        self.field = at;
        self.focus = PrefFocus::Fields;
        self.kept_in_view = None;
        self.flash = Some((self.section, at));
        true
    }

    /// The terminal combobox and the relaunch switch (pool rows).
    pub(super) fn launcher_rows(&self) -> Vec<Field> {
        let ui = &self.config.ui;
        let snap = self.launcher.snapshot.as_ref();
        let auto = match snap.and_then(|s| s.resolved) {
            Some(id) => {
                let name = snap
                    .and_then(|s| s.terminals.iter().find(|(i, _)| *i == id))
                    .map_or(id, |(_, label)| *label);
                t::terminal_auto_is(name)
            }
            None => t::TERMINAL_AUTO.get().to_string(),
        };
        let mut options = vec![
            (auto, "auto".to_string()),
            (
                t::TERMINAL_OS_DEFAULT.get().to_string(),
                "default".to_string(),
            ),
        ];
        options.extend(
            snap.map(|s| s.terminals.as_slice())
                .unwrap_or_default()
                .iter()
                .map(|(id, label)| ((*label).to_string(), (*id).to_string())),
        );
        let mut relaunch = Field::global(
            t::RELAUNCH.get(),
            "ui.relaunch_in_terminal",
            Control::Toggle(ui.relaunch_in_terminal),
        )
        .with_help(t::RELAUNCH_HELP.get());
        if !cfg!(windows) {
            relaunch = relaunch.coming(t::WINDOWS_ONLY.get());
        }
        // Dev mode only; never a row (nor a palette / search hit) otherwise.
        let build = self.dev_mode().then(|| {
            Field::global(
                t::BUILD.get(),
                "ui.launcher_build",
                Control::Combo {
                    options: vec![
                        (t::BUILD_CURRENT.get().to_string(), "current".to_string()),
                        (t::BUILD_NEWEST.get().to_string(), "newest".to_string()),
                    ],
                    value: ui.launcher_build.as_str().to_string(),
                    custom: false,
                },
            )
            .with_help(t::BUILD_HELP.get())
        });
        let mut rows = vec![
            Field::global(
                t::TERMINAL.get(),
                "ui.launch_terminal",
                Control::Combo {
                    options,
                    value: ui.launch_terminal.clone(),
                    custom: false,
                },
            )
            .with_help(t::TERMINAL_HELP.get()),
            relaunch,
            Field::global(
                t::REUSE.get(),
                "ui.reuse_window",
                Control::Toggle(ui.reuse_window),
            )
            .with_help(t::REUSE_HELP.get()),
            self.tools_alias_row(),
            self.tools_alias_add_row(),
        ];
        rows.extend(build);
        rows
    }

    /// The per-place rows below the page's keys.
    pub(super) fn launcher_action_rows(&self) -> Vec<Field> {
        let mut fields = Vec::new();
        let places: Vec<Place> = match &self.launcher.snapshot {
            Some(s) => s.rows.iter().map(|r| r.place).collect(),
            None => Place::for_os(Os::current()).to_vec(),
        };
        for place in places {
            let state = self
                .launcher
                .snapshot
                .as_ref()
                .and_then(|s| s.rows.iter().find(|r| r.place == place))
                .map(|r| r.status.state);
            let label = place_label(place);
            let state_text = match state {
                None => t::CHECKING.get(),
                Some(State::NotInstalled) => t::STATE_NONE.get(),
                Some(State::Installed) => t::STATE_INSTALLED.get(),
                Some(State::Outdated) => t::STATE_OUTDATED.get(),
                Some(State::Foreign) => t::STATE_FOREIGN.get(),
            };
            fields.push(Field::plain(
                label,
                Control::Info(state_text.to_string()),
                Target::None,
            ));
            // Created shortcuts follow the settings by themselves: no button.
            let mut install = Field::button(
                &t::create(label),
                PrefAction::Launcher(LauncherButton::Install(place)),
            )
            .with_help(t::CREATE_HELP.get());
            let remove = Field::button(
                &t::remove(label),
                PrefAction::Launcher(LauncherButton::Remove(place)),
            );
            match state {
                None => {
                    install = install.coming(t::CHECKING.get());
                }
                Some(State::Foreign) => install = install.coming(t::STATE_FOREIGN.get()),
                _ => {}
            }
            // Only the button that applies: Remove once it exists, else Create.
            fields.push(
                if matches!(state, Some(State::Installed | State::Outdated)) {
                    remove
                } else {
                    install
                },
            );
        }
        fields
    }
}
