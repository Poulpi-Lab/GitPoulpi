//! The command alias and "Add GitPoulpi to PATH" rows of General's Launcher
//! group (`gitpoulpi_config::desktop::{alias, path}`). Same shape as the
//! shortcut rows (`launcher_page`): the machine is read on a thread when
//! Settings opens and again whenever `ui.command_alias` changes, and every
//! Create / Remove runs on a thread too (Windows spawns PowerShell for the
//! user PATH). A button pressed while a job runs queues behind it.

use super::*;
use gitpoulpi_config::desktop::alias::{self, Kind, NameError};
use gitpoulpi_config::desktop::path::{self as user_path, UserPath};
use gitpoulpi_config::open_in::Os;
use gitpoulpi_i18n::msg::launcher as t;
use std::sync::mpsc;
use std::sync::Arc;

/// The buttons of the alias and PATH rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ToolsButton {
    AliasCreate,
    AliasRemove,
    PathAdd,
    PathRemove,
}

/// What an alias or PATH job works on; tests give it temp directories.
#[derive(Clone)]
pub(crate) struct Machine {
    pub alias: alias::Env,
    pub path: user_path::Env,
    pub user: Arc<dyn UserPath + Send + Sync>,
    /// Windows: App Paths `<alias>.exe` (memory in tests).
    pub registry: gitpoulpi_config::desktop::registry::SharedRegistry,
}

impl Machine {
    /// This machine: no disk access, so cheap enough for the UI thread, but
    /// built on the job's thread like the rest.
    pub(crate) fn real() -> Machine {
        let os = Os::current();
        let home = gitpoulpi_config::home_dir().unwrap_or_default();
        let env = |k: &str| std::env::var(k).ok();
        let exe = std::env::current_exe().unwrap_or_default();
        // As the running installation: it touches only its own aliases.
        let cur = gitpoulpi_config::installation::current();
        let mut alias = alias::Env::discover(os, &env, &home, &exe).with_ctx(cur.ctx.clone());
        alias.newest = gitpoulpi_config::dev_mode::launcher_newest_on_disk(&exe);
        let keep = gitpoulpi_config::installation::path_keep(cur, &alias);
        let path = user_path::Env::discover(os, &env, &home, &alias).keeping(keep);
        Machine {
            alias,
            path,
            user: Arc::new(user_path::Registry),
            registry: gitpoulpi_config::desktop::registry::from_env(),
        }
    }
}

/// What a read found.
#[derive(Clone)]
struct Snapshot {
    /// The alias name the read was for.
    name: String,
    /// `None`: no (valid) name.
    alias: Option<alias::Status>,
    /// The same for the add variant (`ui.command_alias_add`).
    name_add: String,
    alias_add: Option<alias::Status>,
    /// Every alias of ours in the bin directory.
    existing: Vec<(String, alias::State)>,
    path: Result<user_path::Status, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Op {
    /// The names to create: the alias, the add variant, whichever are set.
    AliasCreate(Vec<(String, Kind)>),
    AliasRemove,
    PathAdd,
    PathRemove,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Done {
    AliasCreated(Vec<String>),
    AliasRemoved(Vec<String>),
    PathAdded,
    PathRemoved,
    PathNothing,
    Failed(String),
}

struct JobResult {
    snapshot: Snapshot,
    done: Option<Done>,
}

/// The rows' own state.
pub(super) struct ToolsState {
    snapshot: Option<Snapshot>,
    rx: Option<mpsc::Receiver<JobResult>>,
    pending: Option<Op>,
    /// What to select when the jobs are done (the palette's route).
    pub(super) focus_after: Vec<PrefAction>,
    /// The `ui.command_alias` and `ui.command_alias_add` the snapshot was
    /// read for.
    for_name: Option<(String, String)>,
    machine: Arc<dyn Fn() -> Machine + Send + Sync>,
}

impl Default for ToolsState {
    fn default() -> Self {
        Self {
            snapshot: None,
            rx: None,
            pending: None,
            focus_after: Vec::new(),
            for_name: None,
            machine: Arc::new(Machine::real),
        }
    }
}

impl ToolsState {
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
        self.rx.is_none() && self.pending.is_none()
    }
}

/// Windows: our shims written by an older version are rewritten by
/// themselves ("Shims follow by themselves"), the way the shortcuts are.
/// Best effort: a shim that cannot be rewritten stays `Outdated`.
pub(super) fn sync_shims(machine: &Machine) {
    if machine.alias.os != Os::Windows {
        return;
    }
    for (name, kind, state) in alias::list_kinds(&machine.alias) {
        if state == alias::State::Outdated {
            let _ = alias::create_with_app_path(&machine.alias, &*machine.registry, &name, kind);
        }
    }
}

fn read(machine: &Machine, name: &str, name_add: &str) -> Snapshot {
    sync_shims(machine);
    let valid = alias::validate_name(name).is_ok();
    let valid_add = alias::validate_name(name_add).is_ok();
    Snapshot {
        name: name.to_string(),
        alias: valid.then(|| alias::status(&machine.alias, name)),
        name_add: name_add.to_string(),
        alias_add: valid_add.then(|| alias::status_kind(&machine.alias, name_add, Kind::Add)),
        existing: alias::list(&machine.alias),
        path: user_path::status(&machine.path, &*machine.user),
    }
}

fn run(machine: &Machine, op: Op) -> Done {
    match op {
        Op::AliasCreate(names) => {
            let mut made = Vec::new();
            for (name, kind) in names {
                match alias::create_with_app_path(&machine.alias, &*machine.registry, &name, kind) {
                    Ok(_) => made.push(name),
                    Err(e) => return Done::Failed(e),
                }
            }
            Done::AliasCreated(made)
        }
        Op::AliasRemove => {
            let mut removed = Vec::new();
            for (name, _) in alias::list(&machine.alias) {
                match alias::remove_with_app_path(&machine.alias, &*machine.registry, &name) {
                    Ok(true) => removed.push(name),
                    Ok(false) => {}
                    Err(e) => return Done::Failed(e),
                }
            }
            Done::AliasRemoved(removed)
        }
        Op::PathAdd => match user_path::add(&machine.path, &*machine.user) {
            Ok(files) if files.is_empty() => Done::PathNothing,
            Ok(_) => Done::PathAdded,
            Err(e) => Done::Failed(e),
        },
        Op::PathRemove => match user_path::remove(&machine.path, &*machine.user) {
            Ok(files) if files.is_empty() => Done::PathNothing,
            Ok(_) => Done::PathRemoved,
            Err(e) => Done::Failed(e),
        },
    }
}

fn name_error_text(e: NameError) -> &'static str {
    match e {
        NameError::Empty => t::ALIAS_SET_NAME.get(),
        NameError::Space => t::ALIAS_BAD_SPACE.get(),
        NameError::Separator => t::ALIAS_BAD_SEPARATOR.get(),
        NameError::BadChar => t::ALIAS_BAD_CHAR.get(),
        NameError::BadStart => t::ALIAS_BAD_START.get(),
        NameError::TooLong => t::ALIAS_BAD_LONG.get(),
    }
}

/// Whether `reason` is one of these rows' own notes (shown as is, not behind
/// "coming").
pub(super) fn is_note(reason: &str) -> bool {
    [
        t::ALIAS_SET_NAME,
        t::ALIAS_BAD_SPACE,
        t::ALIAS_BAD_SEPARATOR,
        t::ALIAS_BAD_CHAR,
        t::ALIAS_BAD_START,
        t::ALIAS_BAD_LONG,
        t::ALIAS_CREATED,
        t::PATH_STATE_NONE,
        t::PATH_STATE_ADDED,
        t::PATH_STATE_ON,
    ]
    .iter()
    .any(|m| m.get() == reason)
}

impl PreferencesState {
    /// Test hook: where aliases and PATH entries are made (never the real ones).
    #[cfg(test)]
    pub(super) fn set_tools_machine(
        &mut self,
        machine: impl Fn() -> Machine + Send + Sync + 'static,
    ) {
        self.tools.machine = Arc::new(machine);
    }

    /// Starts the background job: run `op` if there is one, then read the
    /// machine. Queues `op` behind a job already running.
    pub(super) fn tools_start(&mut self, op: Option<Op>) {
        if self.tools.rx.is_some() {
            if op.is_some() {
                self.tools.pending = op;
            }
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.tools.rx = Some(rx);
        let make = Arc::clone(&self.tools.machine);
        let name = self.config.ui.command_alias.clone();
        let name_add = self.config.ui.command_alias_add.clone();
        self.tools.for_name = Some((name.clone(), name_add.clone()));
        std::thread::spawn(move || {
            let machine = make();
            let done = op.map(|op| run(&machine, op));
            let snapshot = read(&machine, &name, &name_add);
            let _ = tx.send_notify(JobResult { snapshot, done });
        });
    }

    /// Applies a finished job. Never blocks; called once a frame.
    pub(crate) fn poll_tools(&mut self) {
        let Some(rx) = &self.tools.rx else {
            if self.tools.snapshot.is_some()
                && self.tools.for_name.as_ref()
                    != Some(&(
                        self.config.ui.command_alias.clone(),
                        self.config.ui.command_alias_add.clone(),
                    ))
            {
                self.tools_start(None);
            }
            return;
        };
        let Ok(result) = rx.try_recv() else { return };
        self.tools.rx = None;
        self.tools.snapshot = Some(result.snapshot);
        if let Some(done) = result.done {
            if let Done::Failed(e) = &done {
                crate::activity_log::report_problem(
                    crate::activity_log::ProblemSource::Tool,
                    &t::tools_failed(e),
                );
            }
            self.message = Some(Self::tools_message(&done));
        }
        if let Some(op) = self.tools.pending.take() {
            self.tools_start(Some(op));
        }
        self.apply_pending_focus();
    }

    fn tools_message(done: &Done) -> String {
        match done {
            Done::AliasCreated(names) => t::done_alias_created(names.join(", ")),
            Done::AliasRemoved(names) if names.is_empty() => t::DONE_ALIAS_NOTHING.get().into(),
            Done::AliasRemoved(names) => t::done_alias_removed(names.join(", ")),
            Done::PathAdded => t::DONE_PATH_ADDED.get().into(),
            Done::PathRemoved => t::DONE_PATH_REMOVED.get().into(),
            Done::PathNothing => t::DONE_PATH_NOTHING.get().into(),
            Done::Failed(e) => t::tools_failed(e),
        }
    }

    pub(super) fn tools_button(&mut self, button: ToolsButton) {
        let op = match button {
            ToolsButton::AliasCreate => {
                let name = self.config.ui.command_alias.clone();
                let name_add = self.config.ui.command_alias_add.clone();
                let mut names = Vec::new();
                for (name, kind) in [(name, Kind::Open), (name_add, Kind::Add)] {
                    match alias::validate_name(&name) {
                        Ok(()) => names.push((name, kind)),
                        // An empty name is "none" for either: only both
                        // being empty is a mistake.
                        Err(NameError::Empty) => {}
                        Err(e) => {
                            self.message = Some(name_error_text(e).to_string());
                            return;
                        }
                    }
                }
                if names.is_empty() {
                    self.message = Some(t::ALIAS_NEED_NAME.get().to_string());
                    return;
                }
                Op::AliasCreate(names)
            }
            ToolsButton::AliasRemove => Op::AliasRemove,
            ToolsButton::PathAdd => Op::PathAdd,
            ToolsButton::PathRemove => Op::PathRemove,
        };
        self.message = Some(t::WORKING.get().to_string());
        self.tools_start(Some(op));
    }

    /// The palette's route: show the Launcher group, run `button` and
    /// leave the acted-on row selected (and so scrolled into view).
    pub(crate) fn tools_command(&mut self, button: ToolsButton) {
        let _ = self.show_page(None, Some("ui.command_alias"));
        self.tools_button(button);
        let counterpart = match button {
            ToolsButton::AliasCreate => ToolsButton::AliasRemove,
            ToolsButton::AliasRemove => ToolsButton::AliasCreate,
            ToolsButton::PathAdd => ToolsButton::PathRemove,
            ToolsButton::PathRemove => ToolsButton::PathAdd,
        };
        self.focus_after_job(&[PrefAction::Tools(button), PrefAction::Tools(counterpart)]);
    }

    /// The alias combobox: a suggestion, "none", or any typed name. A name
    /// that would run something else already gets a warning under it.
    pub(super) fn tools_alias_row(&self) -> Field {
        let name = self.config.ui.command_alias.clone();
        let options = vec![
            (t::ALIAS_NONE.get().to_string(), String::new()),
            ("gp".to_string(), "gp".to_string()),
        ];
        let mut row = Field::global(
            t::ALIAS.get(),
            "ui.command_alias",
            Control::Combo {
                options,
                value: name.clone(),
                custom: true,
            },
        )
        .with_help(t::ALIAS_HELP.get());
        let status = self
            .tools
            .snapshot
            .as_ref()
            .filter(|s| s.name == name)
            .and_then(|s| s.alias.as_ref());
        if let Some(status) = status {
            if let Some(path) = &status.shadowed_by {
                row = row.with_warning(&t::alias_shadowed(&name, path.display().to_string()));
            } else if status.powershell_builtin {
                row = row.with_warning(&t::alias_powershell(&name));
            }
        }
        row
    }

    /// The add variant's combobox (`ui.command_alias_add`): none, `gpa`, the
    /// alias name with an `a` after it, or any typed name.
    pub(super) fn tools_alias_add_row(&self) -> Field {
        let name = self.config.ui.command_alias_add.clone();
        let main = &self.config.ui.command_alias;
        let mut options = vec![
            (t::ALIAS_NONE.get().to_string(), String::new()),
            ("gpa".to_string(), "gpa".to_string()),
        ];
        let derived = format!("{main}a");
        if !main.is_empty() && derived != "gpa" && alias::validate_name(&derived).is_ok() {
            options.push((derived.clone(), derived));
        }
        let mut row = Field::global(
            t::ALIAS_ADD.get(),
            "ui.command_alias_add",
            Control::Combo {
                options,
                value: name.clone(),
                custom: true,
            },
        )
        .with_help(t::ALIAS_ADD_HELP.get());
        let status = self
            .tools
            .snapshot
            .as_ref()
            .filter(|s| s.name_add == name)
            .and_then(|s| s.alias_add.as_ref());
        if let Some(status) = status {
            if let Some(path) = &status.shadowed_by {
                row = row.with_warning(&t::alias_shadowed(&name, path.display().to_string()));
            } else if status.powershell_builtin {
                row = row.with_warning(&t::alias_powershell(&name));
            }
        }
        row
    }

    /// The state rows and the four buttons, below the shortcuts.
    pub(super) fn tools_action_rows(&self) -> Vec<Field> {
        let mut fields = Vec::new();
        let name = &self.config.ui.command_alias;
        let snap = self.tools.snapshot.as_ref();
        // ---- the command alias ----
        let checking = snap.is_none();
        let state = snap.and_then(|s| (s.name == *name).then_some(s.alias.as_ref()).flatten());
        let invalid = alias::validate_name(name).err();
        let alias_text = if checking {
            t::CHECKING.get()
        } else if let Some(e) = invalid {
            name_error_text(e)
        } else {
            match state.map(|s| s.state) {
                None => t::CHECKING.get(),
                Some(alias::State::NotInstalled) => t::STATE_NONE.get(),
                Some(alias::State::Installed) => t::ALIAS_CREATED.get(),
                Some(alias::State::Outdated) => t::ALIAS_OUTDATED.get(),
                Some(alias::State::Foreign | alias::State::Other) => t::STATE_FOREIGN.get(),
            }
        };
        fields.push(Field::plain(
            t::ALIAS_STATUS.get(),
            Control::Info(alias_text.to_string()),
            Target::None,
        ));
        // ---- the add variant ----
        let name_add = &self.config.ui.command_alias_add;
        let state_add = snap.and_then(|s| {
            (s.name_add == *name_add)
                .then_some(s.alias_add.as_ref())
                .flatten()
        });
        let invalid_add = alias::validate_name(name_add).err();
        let wants_add = invalid_add.is_none();
        if !name_add.is_empty() {
            let text = if checking {
                t::CHECKING.get()
            } else if let Some(e) = invalid_add {
                name_error_text(e)
            } else {
                match state_add.map(|s| s.state) {
                    None => t::CHECKING.get(),
                    Some(alias::State::NotInstalled) => t::STATE_NONE.get(),
                    Some(alias::State::Installed) => t::ALIAS_CREATED.get(),
                    Some(alias::State::Outdated) => t::ALIAS_OUTDATED.get(),
                    Some(alias::State::Foreign | alias::State::Other) => t::STATE_FOREIGN.get(),
                }
            };
            fields.push(Field::plain(
                t::ALIAS_ADD_STATUS.get(),
                Control::Info(text.to_string()),
                Target::None,
            ));
        }
        let others: Vec<String> = snap
            .map(|s| {
                s.existing
                    .iter()
                    .filter(|(n, _)| n != name && n != name_add)
                    .map(|(n, _)| n.clone())
                    .collect()
            })
            .unwrap_or_default();
        if !others.is_empty() {
            fields.push(Field::plain(
                t::ALIAS_ALSO.get(),
                Control::Info(others.join(", ")),
                Target::None,
            ));
        }
        let mut create = Field::button(
            t::ALIAS_CREATE.get(),
            PrefAction::Tools(ToolsButton::AliasCreate),
        )
        .with_help(t::ALIAS_CREATE_HELP.get());
        let mut remove = Field::button(
            t::ALIAS_REMOVE.get(),
            PrefAction::Tools(ToolsButton::AliasRemove),
        );
        let any = snap.is_some_and(|s| !s.existing.is_empty());
        if checking {
            create = create.coming(t::CHECKING.get());
            remove = remove.coming(t::CHECKING.get());
        } else if let Some(e) = invalid.filter(|e| *e != NameError::Empty || !wants_add) {
            create = create.coming(name_error_text(e));
        } else if let Some(e) = invalid_add.filter(|e| *e != NameError::Empty) {
            create = create.coming(name_error_text(e));
        } else if state.map(|s| s.state) == Some(alias::State::Foreign)
            || state_add.map(|s| s.state) == Some(alias::State::Foreign)
        {
            create = create.coming(t::STATE_FOREIGN.get());
        }
        // Only the buttons that apply: Create unless every named alias
        // exists, Remove when an alias is there to remove.
        let installed =
            |s: Option<&alias::Status>| s.map(|s| s.state) == Some(alias::State::Installed);
        let main_done = invalid.is_some() || installed(state);
        let add_done = invalid_add.is_some() || installed(state_add);
        let nothing_named = invalid.is_some() && invalid_add.is_some();
        if !(main_done && add_done) || nothing_named {
            fields.push(create);
        }
        if !checking && any {
            fields.push(remove);
        }
        // ---- PATH ----
        let path = snap.map(|s| &s.path);
        let path_state = match path {
            None => None,
            Some(Ok(p)) => Some(p.state),
            Some(Err(_)) => None,
        };
        let path_text = match (path, path_state) {
            (None, _) => t::CHECKING.get().to_string(),
            (Some(Err(e)), _) => e.clone(),
            (_, Some(user_path::State::NotAdded)) => t::PATH_STATE_NONE.get().into(),
            (_, Some(user_path::State::Added)) => t::PATH_STATE_ADDED.get().into(),
            (_, Some(user_path::State::Outdated)) => t::PATH_STATE_OUTDATED.get().into(),
            (_, Some(user_path::State::OnPath)) => t::PATH_STATE_ON.get().into(),
            (_, None) => t::CHECKING.get().into(),
        };
        fields.push(Field::plain(
            t::PATH_STATUS.get(),
            Control::Info(path_text),
            Target::None,
        ));
        let mut add = Field::button(t::PATH_ADD.get(), PrefAction::Tools(ToolsButton::PathAdd))
            .with_help(t::PATH_ADD_HELP.get());
        let rm = Field::button(
            t::PATH_REMOVE.get(),
            PrefAction::Tools(ToolsButton::PathRemove),
        );
        match path_state {
            None => add = add.coming(t::CHECKING.get()),
            Some(user_path::State::OnPath) => add = add.coming(t::PATH_STATE_ON.get()),
            _ => {}
        }
        // Only the buttons that apply: Add unless it is added, Remove once
        // it has been.
        if path_state != Some(user_path::State::Added) {
            fields.push(add);
        }
        if matches!(
            path_state,
            Some(user_path::State::Added | user_path::State::Outdated)
        ) {
            fields.push(rm);
        }
        fields
    }
}
