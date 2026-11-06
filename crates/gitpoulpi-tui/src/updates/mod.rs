//! Knowing when a newer GitPoulpi exists and what to do about it (the
//! updates-and-install and install-channels designs): the title bar's version
//! chip, the update panel, the Release notes tab and the check behind them.
//!
//! - `mod.rs` (this file): the process-wide state, the check and its cache.
//! - [`chip`]: the version chip in the title bar; a click opens the panel.
//! - [`panel`]: the update panel (version, "Installed with", the actions of
//!   the install channel's procedure).
//! - [`notes`]: the Release notes tab (a view of the Home window).
//! - [`plans`]: what this install is and what may be done about an update.
//! - [`auto`]: download and stage; [`handoff`]: the job for
//!   `gitpoulpi-updater`; [`terminal`]: "Open in terminal".
//!
//! The check runs on a thread of its own, a couple of seconds after startup
//! has settled, never on the render thread, and only when the cache file says
//! a day has passed (`updates.json` in the data folder keeps the last answer,
//! when it was asked and the version the user skipped). Its pieces (semver,
//! channel, asset choice, throttle) live in `gitpoulpi_hosting::updates`; this
//! module wires them to the config, the token store and the screen.
//!
//! A failed background check is silent: one line in the application log. A
//! check the user asked for (palette) says so on the status line. `auto` mode
//! downloads and stages the release by itself when this build came from a
//! release archive ([`auto`]); otherwise it behaves like `notify`.

pub(crate) mod auto;
pub(crate) mod chip;
pub(crate) mod handoff;
pub(crate) mod notes;
pub(crate) mod panel;
pub(crate) mod plans;
pub(crate) mod terminal;

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use gitpoulpi_config::{UpdateChannel, UpdateMode};
use gitpoulpi_hosting::updates::sources::SourceSpec;
use gitpoulpi_hosting::updates::{CheckParams, Release, UpdateState};
use gitpoulpi_i18n::msg::updates as msg;

/// The running version.
pub(crate) const RUNNING: &str = env!("CARGO_PKG_VERSION");

/// The short git hash of a development build (`build.rs`; empty for a
/// release archive and when git was not there).
pub(crate) const GIT_HASH: &str = match option_env!("GITPOULPI_GIT_HASH") {
    Some(hash) => hash,
    None => "",
};

/// How long the background check waits after the window starts, so the first
/// frames and the repository load are not competing with it.
const STARTUP_DELAY: std::time::Duration = std::time::Duration::from_secs(2);

/// The palette's (and the tab's) actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UpdatesCommand {
    /// `Help: Release notes`: opens the tab.
    ReleaseNotes,
    /// `Updates: Check for updates now`.
    CheckNow,
    /// `Updates: Download`: the release page in the browser.
    Download,
    /// `Updates: Skip this version`.
    Skip,
    /// `Updates: Update now`: download, verify and stage the release (a
    /// package-manager channel runs its command).
    UpdateNow,
    /// `Updates: Restart to update`: hand the staged release to the updater.
    Restart,
    /// `Updates: Revert to the previous version`.
    Revert,
    /// `Release notes: Close tab`.
    CloseTab,
    /// `Updates: Show update`: opens the update panel.
    ShowPanel,
    /// `Updates: Copy update command`.
    CopyCommand,
    /// `Updates: Run update command in a terminal`.
    RunInTerminal,
}

#[derive(Default)]
struct Shared {
    mode: UpdateMode,
    channel: UpdateChannel,
    repo: String,
    /// Where `updates.json` lives; `None` until [`configure`] (no config dir).
    state_path: Option<PathBuf>,
    /// The newest release known to be newer than the running build, skipped
    /// or not.
    known: Option<Release>,
    /// The user chose to skip `known`.
    skipped: bool,
    checking: bool,
    /// A status line for the loop to pick up ([`take_message`]).
    message: Option<String>,
    /// The self-update.
    install: auto::InstallState,
    /// How GitPoulpi was installed and the machine policy; `None` until the
    /// background read ([`load_facts`]) has finished.
    facts: Option<plans::Facts>,
    facts_started: bool,
}

static SHARED: Mutex<Shared> = Mutex::new(Shared {
    mode: UpdateMode::Notify,
    channel: UpdateChannel::Stable,
    repo: String::new(),
    state_path: None,
    known: None,
    skipped: false,
    checking: false,
    message: None,
    install: auto::InstallState::new(),
    facts: None,
    facts_started: false,
});

fn shared() -> MutexGuard<'static, Shared> {
    SHARED.lock().unwrap_or_else(|e| e.into_inner())
}

/// The check's file, under the data folder of the config location.
pub(crate) fn state_path() -> Option<PathBuf> {
    gitpoulpi_config::ConfigPaths::discover().map(|p| p.data_dir().join("updates.json"))
}

/// What the screen needs of the state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Snapshot {
    pub(crate) mode: UpdateMode,
    /// A newer release the user has not skipped (and checks are not off).
    pub(crate) available: Option<Release>,
    /// A newer release the user skipped.
    pub(crate) skipped: Option<Release>,
    pub(crate) checking: bool,
    /// Whether this build may replace itself, and where the update stands.
    pub(crate) install: auto::InstallView,
}

/// `config.toml [updates]` as the state holds it.
fn user_config(s: &Shared) -> gitpoulpi_config::UpdatesConfig {
    gitpoulpi_config::UpdatesConfig {
        mode: s.mode,
        channel: s.channel,
        repo: s.repo.clone(),
    }
}

/// The plans for the release the chip announces (the running version when
/// none is known).
fn plans_of(s: &Shared) -> plans::Plans {
    let to = s.known.as_ref().map_or(RUNNING, |r| r.version.as_str());
    plans::plans(
        s.facts.as_ref(),
        &user_config(s),
        to,
        s.install.revert_to.as_deref(),
    )
}

/// The user's settings and the machine policy combined.
fn effective(s: &Shared) -> gitpoulpi_config::policy::EffectiveUpdates {
    gitpoulpi_config::policy::merge(
        &s.facts
            .as_ref()
            .map(|f| f.policy.clone())
            .unwrap_or_default(),
        &user_config(s),
    )
}

/// What Settings needs: how this build was installed, and which of the
/// update rows the administrator's policy sets.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct SettingsView {
    /// The "Installed with" text; `None` until the install is read.
    pub(crate) installed_with: Option<String>,
    pub(crate) mode_locked: bool,
    pub(crate) channel_locked: bool,
    pub(crate) repo_locked: bool,
}

pub(crate) fn settings_view() -> SettingsView {
    let s = shared();
    let eff = effective(&s);
    SettingsView {
        installed_with: s.facts.as_ref().map(|f| plans::channel_label(f.channel())),
        mode_locked: eff.mode_locked,
        channel_locked: eff.release_channel_locked,
        repo_locked: eff.repo_locked,
    }
}

pub(crate) fn snapshot() -> Snapshot {
    let s = shared();
    let eff = effective(&s);
    // A development build never updates: its chip is the plain version, even
    // when the cache another build wrote knows a release.
    let dev = s
        .facts
        .as_ref()
        .is_some_and(|f| f.channel() == gitpoulpi_update::channel::Channel::Dev);
    let known = (eff.mode != UpdateMode::Off && !dev)
        .then(|| s.known.clone())
        .flatten();
    Snapshot {
        mode: eff.mode,
        available: known.clone().filter(|_| !s.skipped),
        skipped: known.filter(|_| s.skipped),
        checking: s.checking,
        install: auto::view(&s),
    }
}

/// Why each self-update command is unavailable now (`None`: available).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Whys {
    pub(crate) update_now: Option<&'static str>,
    pub(crate) restart: Option<&'static str>,
    pub(crate) revert: Option<&'static str>,
    /// `Updates: Copy update command`.
    pub(crate) copy: Option<&'static str>,
    /// `Updates: Run update command in a terminal`.
    pub(crate) terminal: Option<&'static str>,
}

/// The palette's view of the self-update commands.
pub(crate) fn whys() -> Whys {
    use auto::Phase;
    use gitpoulpi_update::plan::Procedure;
    let s = snapshot();
    let busy = matches!(
        s.install.phase,
        Phase::Downloading(_) | Phase::Installing(_)
    );
    let busy_why = busy.then(|| msg::WHY_UPDATE_BUSY.get());
    let known = s.available.is_some() || s.skipped.is_some();
    let swap = matches!(s.install.procedure, Procedure::Swap { .. });
    let run = matches!(s.install.procedure, Procedure::Run { .. });
    // Why the swap-only commands do not apply: a build with no install
    // record cannot update itself; every other channel updates its own way.
    let manual = if matches!(s.install.procedure, Procedure::Nothing(_)) {
        msg::WHY_NOT_UPDATABLE.get()
    } else {
        msg::WHY_MANUAL.get()
    };
    let update_now = if !(swap && s.install.updatable) && !run {
        Some(manual)
    } else if busy {
        busy_why
    } else if swap && matches!(s.install.phase, Phase::Ready(_)) {
        Some(msg::WHY_UPDATE_READY.get())
    } else if !known {
        Some(msg::WHY_NO_UPDATE.get())
    } else {
        None
    };
    let restart = if !(swap && s.install.updatable) {
        Some(manual)
    } else if busy {
        busy_why
    } else if !matches!(s.install.phase, Phase::Ready(_)) {
        Some(msg::WHY_NOTHING_STAGED.get())
    } else {
        None
    };
    let revert = if s.install.revert_to.is_none() {
        Some(if swap || run {
            msg::WHY_NO_REVERT.get()
        } else {
            manual
        })
    } else if busy {
        busy_why
    } else {
        None
    };
    let commands = panel::commands(&s.install.procedure);
    Whys {
        update_now,
        restart,
        revert,
        copy: commands.is_empty().then(|| msg::WHY_NO_COMMAND.get()),
        terminal: (!matches!(s.install.procedure, Procedure::Show { .. }) || commands.is_empty())
            .then(|| msg::WHY_NO_TERMINAL.get()),
    }
}

/// Publishes `updates.mode` / `updates.channel` / `updates.repo` and re-reads
/// the cache for the new channel. Called at start and by the Settings page
/// (live: no restart).
pub(crate) fn configure(config: &gitpoulpi_config::Config, state_path: Option<PathBuf>) {
    let mut s = shared();
    s.mode = config.updates.mode;
    s.channel = config.updates.channel;
    s.repo = config.updates.repo.clone();
    if state_path.is_some() {
        s.state_path = state_path;
    }
    reload_cache(&mut s);
    auto::probe(&mut s);
    drop(s);
    load_facts();
    crate::redraw::poke();
}

/// Reads how GitPoulpi was installed and the machine policy, once, on a
/// thread of its own (it reads files, writes a probe file and walks `PATH`).
/// The result also applies the cache and starts what `auto` mode waits for;
/// the updater's status from the last update becomes one status line.
fn load_facts() {
    {
        let mut s = shared();
        if s.facts_started {
            return;
        }
        s.facts_started = true;
    }
    // Tests inject their own facts (`auto::set_updatable`).
    #[cfg(not(test))]
    {
        let spawned = std::thread::Builder::new()
            .name("update-facts".into())
            .spawn(|| {
                let facts = plans::Facts::detect();
                let status = handoff::paths()
                    .and_then(|p| handoff::startup_status(&p.status, RUNNING, unix_now()));
                {
                    let mut s = shared();
                    auto::apply_facts(&mut s, facts);
                    reload_cache(&mut s);
                    if status.is_some() {
                        s.message = status;
                    }
                }
                crate::redraw::poke();
                auto::auto_start(STARTUP_DELAY);
            });
        if spawned.is_err() {
            shared().facts_started = false;
        }
    }
}

/// A line for the status bar, from any thread.
pub(super) fn say(line: String) {
    shared().message = Some(line);
    crate::redraw::poke();
}

/// Derives `known` / `skipped` from the cache file; no request.
fn reload_cache(s: &mut Shared) {
    let Some(path) = s.state_path.clone() else {
        return;
    };
    let state = UpdateState::load(&path);
    // The skip is applied below, so the release stays known (the tab still
    // shows it, as skipped).
    let unskipped = UpdateState {
        skipped: None,
        ..state.clone()
    };
    let eff = effective(s);
    s.known = unskipped
        .available(RUNNING, eff.release_channel)
        .filter(|r| eff.pin.as_ref().map_or(true, |p| p.allows(&r.version)));
    s.skipped = s
        .known
        .as_ref()
        .is_some_and(|k| state.skipped.as_deref() == Some(k.version.as_str()));
}

/// Starts the once-a-day check when one is due. Returns at once: the thread
/// waits [`STARTUP_DELAY`], then asks.
pub(crate) fn start_background_check() {
    // Once per process: every window restart (a tab switch) runs `init` again.
    static STARTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if STARTED.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    let (mode, due) = {
        let s = shared();
        let due = s
            .state_path
            .as_deref()
            .map_or(true, |p| UpdateState::load(p).due(unix_now(), false));
        (effective(&s).mode, due)
    };
    if mode == UpdateMode::Off {
        return;
    }
    // Not due: the cache may already know a release `auto` has not fetched.
    auto::auto_start(STARTUP_DELAY);
    if !due {
        return;
    }
    spawn_check(false, STARTUP_DELAY);
}

/// `Updates: Check for updates now`: asks at once, ignoring the throttle.
/// The status line answers (right away, then when the host has).
pub(crate) fn check_now() -> Option<String> {
    {
        let mut s = shared();
        if effective(&s).mode == UpdateMode::Off {
            return Some(msg::MSG_OFF.get().to_string());
        }
        if s.checking {
            return Some(msg::MSG_CHECKING.get().to_string());
        }
        s.checking = true;
    }
    crate::redraw::poke();
    spawn_check(true, std::time::Duration::ZERO);
    Some(msg::MSG_CHECKING.get().to_string())
}

/// The status line a finished manual check left, once.
pub(crate) fn take_message() -> Option<String> {
    shared().message.take()
}

/// `Updates: Skip this version`: remembered in the cache file.
pub(crate) fn skip() -> Option<String> {
    let mut s = shared();
    let known = s.known.clone()?;
    if let Some(path) = &s.state_path {
        let mut state = UpdateState::load(path);
        state.skipped = Some(known.version.clone());
        if let Err(e) = state.save(path) {
            tracing::warn!(target: "updates", "could not save the skipped version: {e}");
        }
    }
    s.skipped = true;
    drop(s);
    crate::redraw::poke();
    Some(msg::msg_skipped(&known.version))
}

/// `Updates: Download`: the release page in the browser.
pub(crate) fn download() -> Option<String> {
    let known = shared().known.clone()?;
    let url = known.page_url.clone();
    if url.is_empty() {
        return None;
    }
    Some(match crate::repo_menu::open_url(&url) {
        Ok(()) => msg::msg_opened(&url),
        Err(_) => msg::msg_open_failed(&url),
    })
}

/// Runs the palette's update actions that need no view.
pub(crate) fn run(command: UpdatesCommand) -> Option<String> {
    match command {
        UpdatesCommand::CheckNow => check_now(),
        UpdatesCommand::Download => download(),
        UpdatesCommand::Skip => skip(),
        UpdatesCommand::UpdateNow => auto::update_now(),
        UpdatesCommand::Restart => auto::restart_now(),
        UpdatesCommand::Revert => auto::revert_now(),
        UpdatesCommand::ShowPanel => {
            panel::open();
            None
        }
        UpdatesCommand::CopyCommand => panel::copy_command(),
        UpdatesCommand::RunInTerminal => panel::run_in_terminal(),
        UpdatesCommand::ReleaseNotes | UpdatesCommand::CloseTab => None,
    }
}

/// What a check did.
#[derive(Debug)]
#[cfg_attr(not(feature = "hosting"), allow(dead_code))]
pub(crate) enum Outcome {
    /// Not due (checked within the day, `force` off): no request was made.
    NotDue,
    /// The host answered (the cache now holds what it said).
    Answered,
    Failed(String),
}

/// One check against `base` through `transport`: the throttle, the request,
/// the cache file. Plain inputs, so the tests drive it with a recorded
/// transport and a temp folder.
#[cfg(test)]
pub(crate) fn perform(
    transport: &dyn gitpoulpi_hosting::HttpTransport,
    base: &str,
    params: &CheckParams,
    state_path: &Path,
    force: bool,
    now: u64,
) -> Outcome {
    let spec = SourceSpec::GitHubReleases {
        api_base: base.to_string(),
        repo: params.repo.clone(),
        token: params.token.clone(),
    };
    perform_source(transport, &spec, params, state_path, force, now)
}

/// [`perform`] against any check source (the install channel's and the
/// policy's choice, `plans::choose_source`).
#[cfg_attr(not(feature = "hosting"), allow(dead_code))]
pub(crate) fn perform_source(
    transport: &dyn gitpoulpi_hosting::HttpTransport,
    spec: &SourceSpec,
    params: &CheckParams,
    state_path: &Path,
    force: bool,
    now: u64,
) -> Outcome {
    let mut state = UpdateState::load(state_path);
    if !state.due(now, force) {
        return Outcome::NotDue;
    }
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => return Outcome::Failed(e.to_string()),
    };
    let answer = runtime.block_on(spec.check(transport, params));
    let outcome = match &answer {
        Ok(_) => Outcome::Answered,
        Err(e) => Outcome::Failed(e.to_string()),
    };
    state.record(now, Some(answer));
    if let Err(e) = state.save(state_path) {
        tracing::warn!(target: "updates", "could not save the update check: {e}");
    }
    outcome
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn spawn_check(force: bool, delay: std::time::Duration) {
    let spawned = std::thread::Builder::new()
        .name("update-check".into())
        .spawn(move || {
            if !delay.is_zero() {
                std::thread::sleep(delay);
            }
            let outcome = run_check(force);
            finish(outcome, force);
        });
    if spawned.is_err() {
        shared().checking = false;
    }
}

#[cfg(feature = "hosting")]
fn run_check(force: bool) -> Outcome {
    // Reads the install and the policy here when the background read has
    // not finished (this is a background thread, blocking is fine).
    let facts = {
        let known = shared().facts.clone();
        known.or_else(|| {
            let facts = plans::Facts::detect();
            let mut s = shared();
            if s.facts.is_none() {
                auto::apply_facts(&mut s, facts.clone());
            }
            Some(facts)
        })
    };
    let (state_path, eff) = {
        let s = shared();
        (s.state_path.clone(), effective(&s))
    };
    let Some(state_path) = state_path else {
        return Outcome::NotDue;
    };
    let transport = match gitpoulpi_hosting::ReqwestTransport::new() {
        Ok(t) => t,
        Err(e) => return Outcome::Failed(e.to_string()),
    };
    let params = CheckParams {
        repo: eff.repo.clone(),
        running: RUNNING.to_string(),
        channel: eff.release_channel,
        target: gitpoulpi_config::release::host_target(),
        token: github_token(),
        pin: eff.pin.as_ref().and_then(plans::pin_text),
    };
    let spec = plans::choose_source(
        facts.as_ref(),
        &eff,
        gitpoulpi_hosting::updates::api_base(),
        params.token.clone(),
    );
    perform_source(&transport, &spec, &params, &state_path, force, unix_now())
}

#[cfg(not(feature = "hosting"))]
fn run_check(_force: bool) -> Outcome {
    Outcome::NotDue
}

/// The stored github.com token, when the user has connected one. Asked on the
/// check's thread: resolving it may start the credential helper.
#[cfg(feature = "hosting")]
fn github_token() -> Option<String> {
    use gitpoulpi_hosting::TokenSource;
    let integrations = crate::pull_request::load_integrations();
    let github = integrations.iter().find(|i| {
        i.enabled
            && i.kind == gitpoulpi_config::IntegrationKind::Github
            && i.host()
                .is_some_and(|h| h.eq_ignore_ascii_case("github.com"))
    })?;
    gitpoulpi_hosting::DefaultTokenSource::for_integrations(&integrations)
        .token(github)
        .ok()
}

/// Publishes what a check found: the cache is re-read (the file is the one
/// truth), a manual check leaves a status line, a failure one log line.
fn finish(outcome: Outcome, manual: bool) {
    let mut s = shared();
    s.checking = false;
    match &outcome {
        Outcome::NotDue => {}
        Outcome::Answered => reload_cache(&mut s),
        // A private repository answers 404 until a token is stored: expected,
        // so the background check only logs it.
        Outcome::Failed(why) => {
            tracing::info!(target: "updates", "update check failed: {why}");
        }
    }
    if manual {
        s.message = Some(match &outcome {
            Outcome::Failed(_) => msg::MSG_FAILED.get().to_string(),
            _ => match (&s.known, s.skipped) {
                (Some(k), false) => msg::msg_available(&k.version),
                _ => msg::msg_up_to_date(RUNNING),
            },
        });
    }
    drop(s);
    crate::redraw::poke();
    // `auto` mode: a release this check just found is fetched right away.
    if matches!(outcome, Outcome::Answered) {
        auto::auto_start(std::time::Duration::ZERO);
    }
}

/// The chip's text for the running build: `v0.1.0`, and for a development
/// build `v0.1.0 (abc1234)`.
pub(crate) fn running_label() -> String {
    if GIT_HASH.is_empty() {
        format!("v{RUNNING}")
    } else {
        format!("v{RUNNING} ({GIT_HASH})")
    }
}

/// What the Settings tests pretend the machine reads as.
#[cfg(test)]
pub(crate) struct TestFacts {
    pub(crate) channel: gitpoulpi_config::release::InstallChannel,
    pub(crate) mode: Option<UpdateMode>,
    pub(crate) release_channel: Option<UpdateChannel>,
}

/// Test seam: the machine reads as `facts` (`None`: not read yet).
#[cfg(test)]
pub(crate) fn set_test_facts(facts: Option<TestFacts>) {
    use gitpoulpi_update::channel::{Channel, How, Resolved};
    let facts = facts.map(|t| plans::Facts {
        resolved: Resolved {
            channel: Channel::Install(t.channel),
            how: How::Marker,
            dir: PathBuf::from("/opt/gp"),
            exe_name: "gitpoulpi-tui".into(),
            marker: None,
        },
        policy: gitpoulpi_config::policy::Policy {
            mode: t.mode,
            release_channel: t.release_channel,
            ..Default::default()
        },
        apt_source_present: false,
        aur_helper: None,
        binstall: false,
    });
    auto::set_facts(&mut shared(), facts);
}

/// Serialises the tests that touch the process-wide state.
#[cfg(test)]
pub(crate) static TEST_LOCK: Mutex<()> = Mutex::new(());

#[cfg(test)]
mod tests;

#[cfg(test)]
mod auto_tests;
