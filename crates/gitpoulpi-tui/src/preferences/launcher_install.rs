//! The "This installation" rows of Settings > Launcher: which GitPoulpi is
//! running (kind, path, environment), its launcher name, colour, primary role,
//! environment and folder-menu submenu, the other installations on this machine
//! (Forget for the ones that are gone) and the legacy launchers an older build
//! left under the plain name.
//!
//! The rows are pool rows with virtual keys `installation.*`, routed by
//! `write_global` into this installation's own entry of `[installation.settings]`
//! (and into the `ui.*` keys older builds read while it is the primary), so the
//! `Settings:` palette entries and the search come for free. Writing a plain
//! value is a quick file write; the primary handover, Forget and Remove legacy
//! rewrite launchers, so they run on a thread like the shortcut buttons. Nothing
//! here is journaled: it is system integration, and Remove is the way back.

use super::*;
use crate::launcher::{self, Env};
use gitpoulpi_config::desktop::{self, folder_menu, InstallCtx, Place};
use gitpoulpi_config::installation::{self, Current, Effective, PRIMARY_NAME};
use gitpoulpi_config::open_in::Os;
use gitpoulpi_i18n::msg::launcher as t;
use std::sync::mpsc;
use std::sync::Arc;

/// The version the launcher name carries when the setting asks for it.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// A button of the installation rows. Copy, so it can ride in a [`PrefAction`]:
/// a Forget names the other installation by its place in [`PreferencesState::others`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InstallButton {
    BecomePrimary,
    RemoveLegacy,
    Forget(u8),
}

/// What a job runs.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Job {
    /// Only read what the machine holds.
    Read,
    BecomePrimary,
    Resign,
    RemoveLegacy,
    Forget(Gone),
    /// The launcher name changed (this is the one it had, with the version
    /// when asked): the launchers move to the new names.
    Rename(String),
}

/// An installation whose folder is gone, as Forget needs it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Gone {
    id: String,
    name: String,
    aliases: Vec<String>,
    submenu: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Done {
    Primary(String),
    Resigned(String),
    Forgot(String),
    LegacyRemoved(usize),
    /// The launchers followed a new name; the row's own message stays.
    Renamed,
    Failed(String),
}

struct Reply {
    legacy: Vec<Place>,
    done: Option<Done>,
    /// The file as the job left it, read on the job's thread (a read that
    /// fails for a moment, an antivirus or a sync tool holding the file just
    /// written, is tried again there and never blocks a frame).
    loaded: Option<Config>,
}

/// Reads `file`, trying again a few times when it cannot be read for a moment.
fn load_settled(file: &Path) -> Option<Config> {
    for attempt in 0..5 {
        if let Ok(config) = Config::load(file) {
            return Some(config);
        }
        if attempt < 4 {
            std::thread::sleep(std::time::Duration::from_millis(40));
        }
    }
    None
}

/// Where this installation comes from; tests give it a fake.
type MakeThis = Arc<dyn Fn() -> Current + Send + Sync>;

/// The rows' own state.
pub(super) struct InstallState {
    this: std::cell::OnceCell<Arc<Current>>,
    make: MakeThis,
    rx: Option<mpsc::Receiver<Reply>>,
    pending: Option<Job>,
    /// Where legacy launchers sit (`None`: not read yet).
    legacy: Option<Vec<Place>>,
}

impl Default for InstallState {
    fn default() -> Self {
        #[cfg(not(test))]
        let make: MakeThis = Arc::new(|| installation::current().clone());
        #[cfg(test)]
        let make: MakeThis = Arc::new(|| fake_current(false));
        Self {
            this: std::cell::OnceCell::new(),
            make,
            rx: None,
            pending: None,
            legacy: None,
        }
    }
}

impl InstallState {
    /// A job runs or waits.
    pub(super) fn busy(&self) -> bool {
        self.rx.is_some() || self.pending.is_some()
    }
}

/// A stand-in installation for tests: never the machine's real one.
#[cfg(test)]
pub(crate) fn fake_current(primary: bool) -> Current {
    let identity = installation::Identity {
        id: "aaaa000000000001".to_string(),
        kind: gitpoulpi_config::Kind::Portable,
        root: "/opt/gp/a".to_string(),
        channel: None,
    };
    let mut config = Config::default();
    if primary {
        config.installation.primary = Some(identity.id.clone());
    }
    Current {
        effective: installation::effective(&config, &identity),
        environment: identity.id.clone(),
        ctx: InstallCtx::legacy(),
        config_file: None,
        identity,
        config,
    }
}

/// One other installation of the config.
struct Other {
    id: String,
    name: String,
    kind: String,
    path: String,
    exists: bool,
    effective: Effective,
}

fn is_template_match(reason: &str, template: &str) -> bool {
    // `template` has `\0` where the name goes.
    match template.split_once('\0') {
        Some((pre, post)) => {
            reason.len() >= pre.len() + post.len()
                && reason.starts_with(pre)
                && reason.ends_with(post)
        }
        None => reason == template,
    }
}

/// Whether `reason` is one of these rows' own notes (shown as is).
pub(super) fn is_note(reason: &str) -> bool {
    [t::NAME_PRIMARY, t::SUBMENU_UNAVAILABLE]
        .iter()
        .any(|m| m.get() == reason)
        || is_template_match(reason, &t::alias_used_by("\0"))
}

/// The row of the installation's path, which gives way in its middle.
const INFO: &str = "installation.info";

impl PreferencesState {
    /// The running installation.
    pub(super) fn this(&self) -> &Current {
        self.install
            .this
            .get_or_init(|| Arc::new((self.install.make)()))
    }

    /// Test hook: which installation this is (never the machine's).
    #[cfg(test)]
    pub(super) fn set_this(&mut self, current: Current) {
        self.install.this = std::cell::OnceCell::new();
        let _ = self.install.this.set(Arc::new(current));
    }

    /// Test hook: where legacy launchers were found.
    #[cfg(test)]
    pub(super) fn set_legacy(&mut self, places: Vec<Place>) {
        self.install.legacy = Some(places);
    }

    /// What this installation uses, from the config as it is now.
    pub(super) fn this_effective(&self) -> Effective {
        let this = self.this();
        crate::launcher_sync::effective_in(&self.config, &this.identity, &this.config)
    }

    /// The config as this installation shows it: its own colour keys
    /// (theme, graph colours) over the shared `ui.*`.
    pub(super) fn own_config(&self) -> Config {
        installation::overlay_look(&self.config, &self.this_effective())
    }

    /// The colour this installation wears (never another one's).
    pub(super) fn this_colour(&self) -> String {
        self.this_effective().logo_color
    }

    /// This installation's entry as stored, else the seeded one.
    pub(super) fn own_entry(&self) -> gitpoulpi_config::InstallationEntry {
        let this = self.this();
        self.config
            .installation
            .settings
            .get(&this.identity.id)
            .or_else(|| this.config.installation.settings.get(&this.identity.id))
            .cloned()
            .unwrap_or_default()
    }

    /// Takes the installation table and the look keys older builds read from
    /// the file again (the rest of the page's config stays), and publishes
    /// the look when it changed.
    pub(super) fn reread_installation(&mut self, loaded: Option<Config>) {
        let Some(loaded) = loaded.or_else(|| {
            let file = self.config_file()?;
            Config::load(&file).ok()
        }) else {
            return;
        };
        let before = self.this_effective();
        self.config.installation = loaded.installation;
        self.config.ui.logo_color = loaded.ui.logo_color;
        self.config.ui.command_alias = loaded.ui.command_alias;
        self.config.ui.command_alias_add = loaded.ui.command_alias_add;
        if self.this_effective() != before {
            self.publish_installation();
        }
    }

    fn others(&self) -> Vec<Other> {
        let me = &self.this().identity.id;
        self.config
            .installation
            .settings
            .iter()
            .filter(|(id, _)| *id != me)
            .filter_map(|(id, e)| {
                let effective = installation::effective_of(&self.config, id)?;
                Some(Other {
                    id: id.clone(),
                    name: effective.name.clone(),
                    kind: e.kind.as_str().to_string(),
                    path: e.path.clone(),
                    exists: !e.path.is_empty() && Path::new(&e.path).exists(),
                    effective,
                })
            })
            .collect()
    }

    /// The other installation that is the primary, as `(id, name)`.
    fn other_primary(&self) -> Option<(String, String)> {
        let id = self.config.installation.primary.clone()?;
        if id == self.this().identity.id {
            return None;
        }
        let name = installation::effective_of(&self.config, &id)
            .map_or_else(|| PRIMARY_NAME.to_string(), |e| e.name);
        Some((id, name))
    }

    /// The pool rows: Installation, name, version, colour, primary,
    /// environment and folder-menu submenu.
    pub(super) fn installation_rows(&self) -> Vec<Field> {
        let this = self.this();
        let eff = self.this_effective();
        let entry = self.own_entry();
        let others = self.others();
        let path = this.identity.root.clone();
        let full = t::installation_info(this.identity.kind.as_str(), &path, &this.environment);
        let info = Field::global(t::INSTALLATION.get(), INFO, Control::Info(full.clone()))
            .with_help(&format!(
                "{} {}",
                t::INSTALLATION_HELP.get(),
                t::installation_folder(&path)
            ))
            .with_ellipsis_of(&path)
            .live();
        let mut name = Field::global(t::NAME.get(), "installation.name", Control::Text(eff.name))
            .with_help(t::NAME_HELP.get())
            .live();
        if eff.primary {
            name = name.coming(t::NAME_PRIMARY.get());
        }
        let version = Field::global(
            t::NAME_WITH_VERSION.get(),
            "installation.name_with_version",
            Control::Toggle(eff.name_with_version),
        )
        .with_help(t::NAME_WITH_VERSION_HELP.get())
        .live();
        let colour = Field::global(
            t::COLOR.get(),
            "installation.logo_color",
            Control::Combo {
                options: crate::logo_colors::ways()
                    .iter()
                    .map(|w| {
                        (
                            crate::logo_colors::name(&w.slug).to_string(),
                            w.slug.clone(),
                        )
                    })
                    .collect(),
                value: if eff.logo_color.is_empty() {
                    crate::logo_colors::default_slug().to_string()
                } else {
                    eff.logo_color
                },
                custom: false,
            },
        )
        .with_help(t::COLOR_HELP.get())
        .live();
        let primary = Field::global(
            t::PRIMARY.get(),
            "installation.primary",
            Control::Toggle(eff.primary),
        )
        .with_help(t::PRIMARY_HELP.get())
        .live();
        let mut env_options = vec![(t::ENV_OWN.get().to_string(), String::new())];
        for other in &others {
            let env = other.effective.env.as_str();
            if env != this.identity.id && env_options.iter().all(|(_, v)| v != env) {
                env_options.push((env.to_string(), env.to_string()));
            }
        }
        let env = Field::global(
            t::ENV.get(),
            "installation.env",
            Control::Combo {
                options: env_options,
                value: entry.env.trim().to_string(),
                custom: true,
            },
        )
        .with_help(t::ENV_HELP.get())
        .live();
        let mut submenus = vec![(t::SUBMENU_NONE.get().to_string(), String::new())];
        for other in &others {
            let name = other.effective.folder_menu_submenu.trim();
            if !name.is_empty() && submenus.iter().all(|(_, v)| !v.eq_ignore_ascii_case(name)) {
                submenus.push((name.to_string(), name.to_string()));
            }
        }
        let mut submenu = Field::global(
            t::SUBMENU.get(),
            "installation.folder_menu_submenu",
            Control::Combo {
                options: submenus,
                value: entry.folder_menu_submenu.trim().to_string(),
                custom: true,
            },
        )
        .with_help(t::SUBMENU_HELP.get())
        .live();
        if Os::current() == Os::Macos {
            submenu = submenu.coming(t::SUBMENU_UNAVAILABLE.get());
        } else if self.folder_menu_lacks_submenus() {
            submenu = submenu.with_warning(t::SUBMENU_ROOT_ONLY.get());
        }
        vec![info, name, version, colour, primary, env, submenu]
    }

    /// The Other installations and the Legacy launchers rows.
    pub(super) fn installation_action_rows(&self) -> Vec<Field> {
        let mut fields = Vec::new();
        for (i, other) in self.others().iter().enumerate() {
            let text = if other.exists {
                t::other_installation_info(&other.kind, &other.path)
            } else {
                t::other_installation_gone(&other.path)
            };
            fields.push(
                Field::plain(
                    &t::other_installation(&other.name),
                    Control::Info(text),
                    Target::None,
                )
                .with_ellipsis_of(&other.path),
            );
            if !other.exists {
                fields.push(Field::button(
                    t::FORGET.get(),
                    PrefAction::Installation(InstallButton::Forget(i as u8)),
                ));
            }
        }
        // Without a primary the plain-named launchers of an older build are
        // nobody's: shown, never rewritten, here to take over or remove.
        if self.config.installation.primary.is_none() {
            if let Some(places) = self.install.legacy.as_ref().filter(|p| !p.is_empty()) {
                let places: Vec<&str> = places.iter().map(|p| place_name(*p)).collect();
                fields.push(
                    Field::plain(
                        t::LEGACY.get(),
                        Control::Info(t::legacy_found(places.join(", "))),
                        Target::None,
                    )
                    .with_help(t::LEGACY_HELP.get()),
                );
                fields.push(Field::buttons(
                    "",
                    vec![
                        Btn::new(
                            t::PRIMARY.get(),
                            PrefAction::Installation(InstallButton::BecomePrimary),
                        ),
                        Btn::new(
                            t::REMOVE_LEGACY.get(),
                            PrefAction::Installation(InstallButton::RemoveLegacy),
                        ),
                    ],
                ));
            }
        }
        fields
    }

    // ---------------------------------------------------------------
    // Writing a value
    // ---------------------------------------------------------------

    /// `write_global` for a virtual `installation.<field>` key. `false` when
    /// the value was refused (the message says why).
    pub(super) fn write_installation(&mut self, field: &str, value: &str) -> bool {
        let value = value.trim();
        let Some(file) = self.config_file() else {
            self.message = Some(screen_no_dir());
            return false;
        };
        let this = self.this();
        let (id, identity, seeded) = (
            this.identity.id.clone(),
            this.identity.clone(),
            self.own_entry(),
        );
        let eff = self.this_effective();
        let old_name = installation::launcher_name(&eff.name, eff.name_with_version, VERSION);
        // What the entry takes, after the checks.
        let cleaned = match field {
            "info" => return true,
            "primary" => return self.set_primary(value == "true"),
            "name" => {
                if eff.primary {
                    self.message = Some(t::NAME_PRIMARY.get().to_string());
                    return false;
                }
                if !value.is_empty() && installation::validate_name(value).is_err() {
                    self.message = Some(t::NAME_INVALID.get().to_string());
                    return false;
                }
                let taken = self
                    .others()
                    .iter()
                    .any(|o| o.exists && o.effective.name.trim().eq_ignore_ascii_case(value));
                if taken {
                    self.message = Some(t::name_taken(value));
                    return false;
                }
                value.to_string()
            }
            "name_with_version" => value.to_string(),
            "logo_color" => {
                if !crate::logo_colors::ways().iter().any(|w| w.slug == value) {
                    self.message = Some(format!("installation.logo_color: {value:?}"));
                    return false;
                }
                value.to_string()
            }
            "command_alias" | "command_alias_add" => {
                if !value.is_empty() {
                    if let Err(e) = desktop::alias::validate_name(value) {
                        self.message = Some(super::launcher_alias::name_error_text(e).to_string());
                        return false;
                    }
                }
                value.to_string()
            }
            "env" => {
                let clean = installation::sanitize(value);
                if clean.is_empty() && !value.is_empty() {
                    self.message = Some(format!("installation.env: {value:?}"));
                    return false;
                }
                clean
            }
            "folder_menu_submenu" => value.replace(|c: char| c.is_control(), " "),
            other => {
                self.message = Some(format!("installation.{other}: unknown")); // i18n-english
                return false;
            }
        };
        let flag = value == "true";
        let primary = eff.primary;
        let edit = |i: &mut gitpoulpi_config::InstallationConfig| {
            let entry = i.settings.entry(id.clone()).or_insert_with(|| {
                if seeded.path.is_empty() {
                    gitpoulpi_config::InstallationEntry {
                        path: identity.root.clone(),
                        kind: identity.kind,
                        ..Default::default()
                    }
                } else {
                    seeded.clone()
                }
            });
            match field {
                "name" => entry.name = cleaned.clone(),
                "name_with_version" => entry.name_with_version = flag,
                "logo_color" => entry.logo_color = cleaned.clone(),
                "command_alias" => entry.command_alias = cleaned.clone(),
                "command_alias_add" => entry.command_alias_add = cleaned.clone(),
                "env" => entry.env = cleaned.clone(),
                "folder_menu_submenu" => entry.folder_menu_submenu = cleaned.clone(),
                _ => {}
            }
        };
        if let Err(e) = installation::set_installation_in_file(&file, edit) {
            let text = format!("installation.{field}: {e}");
            crate::activity_log::report_problem(crate::activity_log::ProblemSource::Save, &text);
            self.message = Some(text);
            return false;
        }
        // The primary also owns the keys older builds read.
        if primary {
            let key = match field {
                "logo_color" => Some("ui.logo_color"),
                "command_alias" => Some("ui.command_alias"),
                "command_alias_add" => Some("ui.command_alias_add"),
                _ => None,
            };
            if let Some(key) = key {
                let _ = gitpoulpi_config::set_in_file(&file, key, &cleaned);
            }
        }
        self.config = Config::load(&file).unwrap_or_else(|_| self.config.clone());
        self.publish_installation();
        // The shortcuts carry the launcher name: they move to the new one now,
        // not at the next Create.
        if matches!(field, "name" | "name_with_version") {
            let now = self.this_effective();
            let new_name = installation::launcher_name(&now.name, now.name_with_version, VERSION);
            if new_name != old_name {
                self.install_start(Job::Rename(old_name));
            }
        }
        self.message = Some(if field == "env" {
            t::ENV_RESTART.get().to_string()
        } else {
            format!("installation.{field} = {value}")
        });
        true
    }

    /// Tells the rest of the running TUI (the title bar's octopus, the shortcut
    /// sync) what this installation uses now.
    pub(super) fn publish_installation(&self) {
        crate::about::install(&self.this_colour());
        crate::launcher_sync::install(&self.config);
    }

    /// The Become the primary checkbox: on asks first when another installation
    /// is primary; off hands the role back.
    fn set_primary(&mut self, on: bool) -> bool {
        if on {
            self.install_press(InstallButton::BecomePrimary);
        } else if self.this_effective().primary {
            self.message = Some(t::WORKING.get().to_string());
            self.install_start(Job::Resign);
        }
        true
    }

    // ---------------------------------------------------------------
    // Buttons and jobs
    // ---------------------------------------------------------------

    /// Whether `button` asks before it runs.
    fn install_confirm(&self, button: InstallButton) -> Option<String> {
        match button {
            InstallButton::BecomePrimary => self
                .other_primary()
                .map(|(_, name)| t::primary_confirm(name)),
            InstallButton::RemoveLegacy => Some(
                gitpoulpi_i18n::msg::settings_screen::confirm_action(t::REMOVE_LEGACY.get()),
            ),
            InstallButton::Forget(i) => self
                .others()
                .get(usize::from(i))
                .map(|o| t::forget_confirm(&o.name)),
        }
    }

    /// The text of the question for `action`, when it is one of ours.
    pub(super) fn install_confirm_text(&self, action: PrefAction) -> Option<String> {
        match action {
            PrefAction::Installation(b) => self.install_confirm(b),
            _ => None,
        }
    }

    /// Whether pressing `action` asks first, besides the always destructive ones.
    pub(super) fn install_asks(&self, action: PrefAction) -> bool {
        matches!(action, PrefAction::Installation(b) if self.install_confirm(b).is_some())
    }

    /// Presses `button` as the page's button would: asks, or runs.
    pub(super) fn install_press(&mut self, button: InstallButton) {
        let label = match button {
            InstallButton::BecomePrimary => t::PRIMARY.get(),
            InstallButton::RemoveLegacy => t::REMOVE_LEGACY.get(),
            InstallButton::Forget(_) => t::FORGET.get(),
        };
        let btn = Btn::new(label, PrefAction::Installation(button));
        self.press_button(&btn);
    }

    /// Runs the confirmed or unguarded button.
    pub(super) fn install_button(&mut self, button: InstallButton) {
        let job = match button {
            InstallButton::BecomePrimary => Job::BecomePrimary,
            InstallButton::RemoveLegacy => Job::RemoveLegacy,
            InstallButton::Forget(i) => {
                let Some(other) = self.others().into_iter().nth(usize::from(i)) else {
                    return;
                };
                if other.exists {
                    return;
                }
                Job::Forget(Gone {
                    id: other.id,
                    name: other.name,
                    aliases: vec![
                        other.effective.command_alias.clone(),
                        other.effective.command_alias_add.clone(),
                    ],
                    submenu: other.effective.folder_menu_submenu.clone(),
                })
            }
        };
        self.message = Some(t::WORKING.get().to_string());
        self.install_start(job);
    }

    /// The palette's route: show the installation rows, press the button and
    /// leave its row selected.
    pub(crate) fn installation_command(&mut self, button: InstallButton) {
        let _ = self.show_page(None, Some("installation.name"));
        // Forget has one button per gone installation: the first one.
        if let InstallButton::Forget(_) = button {
            if self.others().iter().all(|o| o.exists) {
                self.message = Some(t::FORGET_NONE.get().to_string());
                return;
            }
        }
        let button = match button {
            InstallButton::Forget(_) => {
                let first = self.others().iter().position(|o| !o.exists).unwrap_or(0);
                InstallButton::Forget(first as u8)
            }
            other => other,
        };
        self.install_press(button);
        self.focus_after_job(&[PrefAction::Installation(button)]);
    }

    /// Reads what the machine holds (legacy launchers) on a thread.
    pub(super) fn install_read(&mut self) {
        self.install_start(Job::Read);
    }

    /// Starts a job on a thread, or queues it behind the one running.
    fn install_start(&mut self, job: Job) {
        if self.install.rx.is_some() {
            if job != Job::Read {
                self.install.pending = Some(job);
            }
            return;
        }
        let (tx, rx) = mpsc::channel();
        self.install.rx = Some(rx);
        let machine = JobMachine {
            file: self.config_file(),
            identity: self.this().identity.clone(),
            seeded: self.this().config.clone(),
            setting: self.config.ui.launch_terminal.clone(),
            launcher: self.launcher_env_factory(),
            folder: self.folder_menu_env_factory(),
            aliases: self.tools_machine_factory(),
        };
        std::thread::spawn(move || {
            let done = match job {
                Job::Read => None,
                job => Some(machine.run(job)),
            };
            let legacy = launcher::legacy_places(&(machine.launcher)(), &machine.setting);
            let loaded = machine.file.as_deref().and_then(load_settled);
            let _ = tx.send_notify(Reply {
                legacy,
                done,
                loaded,
            });
        });
    }

    /// A handover job that finished says who the primary is, whatever the
    /// re-read made of the file.
    fn settle_primary(&mut self, done: &Done) {
        let me = self.this().identity.id.clone();
        let primary = &mut self.config.installation.primary;
        match done {
            Done::Primary(_) if primary.as_deref() != Some(me.as_str()) => *primary = Some(me),
            Done::Resigned(_) if primary.as_deref() == Some(me.as_str()) => *primary = None,
            _ => return,
        }
        self.publish_installation();
    }

    /// Applies a finished job. Never blocks; called once a frame.
    pub(crate) fn poll_install(&mut self) {
        let Some(rx) = &self.install.rx else {
            return;
        };
        let Ok(reply) = rx.try_recv() else { return };
        self.install.rx = None;
        self.install.legacy = Some(reply.legacy);
        // The file is the truth, whatever the job made of it: a handover that
        // fails half way has still written the primary, and another
        // installation may have changed it since the page opened.
        self.reread_installation(reply.loaded);
        if let Some(done) = reply.done {
            // The job's own answer settles the role when the file could not be
            // read back: the row never keeps the state the job just ended.
            self.settle_primary(&done);
            if let Done::Failed(e) = &done {
                crate::activity_log::report_problem(
                    crate::activity_log::ProblemSource::Tool,
                    &t::failed(e),
                );
            }
            if !matches!(
                done,
                Done::Failed(_) | Done::LegacyRemoved(_) | Done::Renamed
            ) {
                self.publish_installation();
            }
            let text = match &done {
                Done::Renamed => None,
                Done::Primary(name) => Some(t::done_primary(name)),
                Done::Resigned(name) => Some(t::done_resigned(name)),
                Done::Forgot(name) => Some(t::done_forgot(name)),
                Done::LegacyRemoved(0) => Some(t::DONE_LEGACY_NOTHING.get().to_string()),
                Done::LegacyRemoved(_) => Some(t::DONE_LEGACY_REMOVED.get().to_string()),
                Done::Failed(e) => Some(t::failed(e)),
            };
            if text.is_some() {
                self.message = text;
            }
        }
        if let Some(job) = self.install.pending.take() {
            self.install_start(job);
        }
        self.apply_pending_focus();
    }
}

fn screen_no_dir() -> String {
    gitpoulpi_i18n::msg::settings_screen::NO_CONFIG_DIR
        .get()
        .into()
}

fn place_name(place: Place) -> &'static str {
    match (place, Os::current()) {
        (Place::Launcher, Os::Macos) => t::PLACE_APPLICATIONS.get(),
        (Place::Launcher, _) => t::PLACE_LAUNCHER.get(),
        (Place::Desktop, _) => t::PLACE_DESKTOP.get(),
        (Place::StartMenu, _) => t::PLACE_START_MENU.get(),
    }
}

/// Everything a job needs, owned, so it can leave the input thread.
struct JobMachine {
    file: Option<PathBuf>,
    identity: installation::Identity,
    seeded: Config,
    setting: String,
    launcher: Arc<dyn Fn() -> Env + Send + Sync>,
    folder: super::launcher_folder_menu::MakeEnv,
    aliases: Arc<dyn Fn() -> super::launcher_alias::Machine + Send + Sync>,
}

impl JobMachine {
    fn run(&self, job: Job) -> Done {
        let Some(file) = self.file.clone() else {
            return Done::Failed(screen_no_dir());
        };
        match job {
            // Only a read: the thread never runs it.
            Job::Read => Done::LegacyRemoved(0),
            Job::BecomePrimary => self.change_primary(&file, true),
            Job::Resign => self.change_primary(&file, false),
            Job::RemoveLegacy => {
                let env = (self.launcher)();
                let logo = self.logo(&file);
                let removed = launcher::remove_legacy(&env, &(self.folder)(&logo));
                Done::LegacyRemoved(removed)
            }
            Job::Forget(gone) => self.forget(&file, &gone),
            Job::Rename(old) => self.rename(&file, old),
        }
    }

    fn config(&self, file: &Path) -> Config {
        Config::load(file).unwrap_or_default()
    }

    fn logo(&self, file: &Path) -> String {
        crate::launcher_sync::effective_in(&self.config(file), &self.identity, &self.seeded)
            .logo_color
    }

    fn ctx(&self, file: &Path) -> InstallCtx {
        launcher::ctx_in(&self.config(file), &self.identity, &self.seeded)
    }

    /// Becomes (or stops being) the primary: the launchers this installation
    /// has move to the names the new role owns, in one go. The previous primary
    /// keeps its own launchers under its own name.
    fn change_primary(&self, file: &Path, becoming: bool) -> Done {
        let mut env = (self.launcher)();
        // The role as the file has it now, whatever the factory captured.
        env.ctx = self.ctx(file);
        let logo = self.logo(file);
        let mut fe = (self.folder)(&logo);
        fe.ctx = env.ctx.clone();
        let label = launcher::folder_menu_label(&env.ctx.name);
        // What this installation has now, under its current names.
        let had: Vec<Place> = launcher::snapshot(&env, &self.setting, &logo)
            .rows
            .iter()
            .filter(|r| {
                matches!(
                    r.status.state,
                    desktop::State::Installed | desktop::State::Outdated
                )
            })
            .map(|r| r.place)
            .collect();
        let had_folder = folder_menu::has_any_ours(&folder_menu::status(&fe, &label));
        // The launchers under the names of the old role are removed only after
        // the ones of the new role exist: a launcher is always there (Explorer
        // takes seconds to settle a delete followed by a create).
        let (old_ctx, old_fe) = (env.ctx.clone(), fe.clone());
        let handover = if becoming {
            installation::become_primary_in_file(file, &self.identity)
        } else {
            installation::resign_primary_in_file(file, &self.identity.id)
        };
        let handover = match handover {
            Ok(h) => h,
            Err(e) => return Done::Failed(e.to_string()),
        };
        // The previous primary's legacy-named launchers go to its own names.
        if let Some(old) = handover.as_ref().and_then(|h| h.demoted.clone()) {
            if old != self.identity.id {
                let known = env.ctx.known.clone();
                let old_ctx = InstallCtx {
                    short: installation::short_id(&old),
                    name: PRIMARY_NAME.to_string(),
                    primary: Some(old.clone()),
                    id: old,
                    known,
                    alias_claims: Vec::new(),
                };
                let _ = desktop::hand_over_shortcuts(&env.dirs, env.os, &old_ctx);
                let mut old_fe = fe.clone();
                old_fe.ctx = old_ctx;
                let _ = folder_menu::demote(&old_fe);
            }
        }
        let ctx = self.ctx(file);
        let logo = self.logo(file);
        env.ctx = ctx.clone();
        fe.ctx = ctx.clone();
        // The shortcuts move to the names of the new role by a rename, before
        // anything is written: there is no moment without one, whether or not
        // the rewrite below works (Explorer settles a delete followed by a
        // create slowly, and a failed create must not leave nothing).
        let _ = desktop::rename_shortcuts(&env.dirs, env.os, &old_ctx, &ctx);
        let mut installed = Vec::new();
        for place in &had {
            if matches!(
                launcher::run(&env, &self.setting, &logo, launcher::Op::Install(*place)),
                launcher::Done::Installed(_)
            ) {
                installed.push(*place);
            }
        }
        // What is left under the old names goes only where the new one is
        // written (macOS bundles are not renamed).
        for place in &installed {
            let _ = desktop::remove(*place, &env.dirs, env.os, &old_ctx);
        }
        if had_folder {
            let label = launcher::folder_menu_label(&ctx.name);
            let written = folder_menu::install(&fe, &label).is_ok()
                && folder_menu::has_any_ours(&folder_menu::status(&fe, &label));
            if written {
                let _ = folder_menu::remove_replaced(&old_fe, &fe);
            }
        }
        // Launchers left under the plain name (no owner tag) are this
        // installation's now: the sync rewrites them.
        let _ = launcher::sync_shortcuts(&env, &self.setting, &logo);
        // The icons of the names left behind (the window's, or a shortcut's
        // that moved) go unless a pin, a legacy shortcut or another place
        // still names them. After the sync, which re-points the pins first.
        if old_ctx.is_primary() != ctx.is_primary() {
            desktop::release_icons(&env.dirs, env.os, &old_ctx);
        }
        // The alias files this installation made carry the role's own text (the
        // legacy marker is the primary's alone): rewritten now, so they read
        // installed and not outdated. Another installation's are never touched.
        super::launcher_alias::rewrite_own(&(self.aliases)(), &ctx);
        if becoming {
            Done::Primary(ctx.name)
        } else {
            Done::Resigned(ctx.name)
        }
    }

    /// The launcher name changed from `old_name`: this installation's
    /// shortcuts and folder-menu entry move to the new one, in place.
    fn rename(&self, file: &Path, old_name: String) -> Done {
        let env = (self.launcher)();
        let logo = self.logo(file);
        let ctx = self.ctx(file);
        let old = InstallCtx {
            name: old_name,
            ..ctx.clone()
        };
        let mut fe = (self.folder)(&logo);
        fe.ctx = ctx.clone();
        launcher::rename_launchers(&env, &fe, &self.setting, &logo, &old, &ctx);
        Done::Renamed
    }

    /// Forgets a gone installation: its launchers, aliases and folder-menu
    /// entries, then its entry.
    fn forget(&self, file: &Path, gone: &Gone) -> Done {
        let env = (self.launcher)();
        let config = self.config(file);
        let was_primary = config.installation.primary.as_deref() == Some(gone.id.as_str());
        let ctx = InstallCtx {
            id: gone.id.clone(),
            short: installation::short_id(&gone.id),
            name: gone.name.clone(),
            primary: config.installation.primary.clone(),
            known: Vec::new(),
            alias_claims: Vec::new(),
        };
        for place in env.dirs.places(env.os) {
            let _ = desktop::remove(place, &env.dirs, env.os, &ctx);
        }
        let logo = self.logo(file);
        let mut fe = (self.folder)(&logo);
        fe.ctx = ctx.clone();
        fe.submenu = gone.submenu.clone();
        let _ = folder_menu::remove(&fe);
        let machine = (self.aliases)();
        let alias_env = machine.alias.clone().with_ctx(ctx);
        for name in gone.aliases.iter().filter(|n| !n.is_empty()) {
            let _ = desktop::alias::remove_with_app_path(&alias_env, &*machine.registry, name);
        }
        let id = gone.id.clone();
        let result = installation::set_installation_in_file(file, |i| {
            i.settings.remove(&id);
            if was_primary {
                i.primary = None;
                i.ui_before = None;
            }
        });
        match result {
            Ok(()) => Done::Forgot(gone.name.clone()),
            Err(e) => Done::Failed(e.to_string()),
        }
    }
}

#[cfg(test)]
mod settle_tests {
    use super::*;

    /// A job that ended says who the primary is even when the file could not
    /// be read back: the checkbox never keeps the state the job just left.
    #[test]
    fn a_finished_handover_settles_the_primary_row_when_the_reread_failed() {
        let tmp = tempfile::tempdir().unwrap();
        let mut st = crate::preferences::tests::state(tmp.path());
        st.set_this(fake_current(true));
        st.config.installation.primary = Some("aaaa000000000001".to_string());
        let checked = |st: &PreferencesState| st.this_effective().primary;
        assert!(checked(&st));
        // The file could not be read back: the job's own answer still counts.
        st.settle_primary(&Done::Resigned("name".to_string()));
        assert!(!checked(&st), "resigned: the box is clear");
        st.settle_primary(&Done::Primary("name".to_string()));
        assert!(checked(&st), "became the primary: the box is checked");
        // Other answers change nothing.
        st.settle_primary(&Done::Renamed);
        assert!(checked(&st));
    }
}
