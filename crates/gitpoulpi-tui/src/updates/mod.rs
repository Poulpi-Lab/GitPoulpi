//! Knowing when a newer GitPoulpi exists (the updates-and-install design,
//! slice A): the status bar's version chip, the Release notes tab and the
//! check behind them, and [`auto`] (slice B: the self-update).
//!
//! - `mod.rs` (this file): the process-wide state, the check and its cache.
//! - [`chip`]: the version chip right of the Activity button.
//! - [`notes`]: the Release notes tab (a view of the Home window).
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
pub(crate) mod notes;

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use gitpoulpi_config::{UpdateChannel, UpdateMode};
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
    /// `Updates: Update now`: download, verify and stage the release.
    UpdateNow,
    /// `Updates: Restart to update`: swap the files, restart.
    Restart,
    /// `Updates: Revert to the previous version`.
    Revert,
    /// `Release notes: Close tab`.
    CloseTab,
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
    /// The self-update (slice B).
    install: auto::InstallState,
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

pub(crate) fn snapshot() -> Snapshot {
    let s = shared();
    let known = (s.mode != UpdateMode::Off)
        .then(|| s.known.clone())
        .flatten();
    Snapshot {
        mode: s.mode,
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
}

/// The palette's view of the self-update commands.
pub(crate) fn whys() -> Whys {
    use auto::Phase;
    let s = snapshot();
    let busy = matches!(
        s.install.phase,
        Phase::Downloading(_) | Phase::Installing(_)
    );
    let busy_why = busy.then(|| msg::WHY_UPDATE_BUSY.get());
    let update_now = if !s.install.updatable {
        Some(msg::WHY_NOT_UPDATABLE.get())
    } else if busy {
        busy_why
    } else if matches!(s.install.phase, Phase::Ready(_)) {
        Some(msg::WHY_UPDATE_READY.get())
    } else if s.available.is_none() && s.skipped.is_none() {
        Some(msg::WHY_NO_UPDATE.get())
    } else {
        None
    };
    let restart = if !s.install.updatable {
        Some(msg::WHY_NOT_UPDATABLE.get())
    } else if busy {
        busy_why
    } else if !matches!(s.install.phase, Phase::Ready(_)) {
        Some(msg::WHY_NOTHING_STAGED.get())
    } else {
        None
    };
    let revert = if !s.install.updatable {
        Some(msg::WHY_NOT_UPDATABLE.get())
    } else if busy {
        busy_why
    } else if s.install.revert_to.is_none() {
        Some(msg::WHY_NO_REVERT.get())
    } else {
        None
    };
    Whys {
        update_now,
        restart,
        revert,
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
    s.known = unskipped.available(RUNNING, s.channel);
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
        (s.mode, due)
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
        if s.mode == UpdateMode::Off {
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
#[cfg_attr(not(feature = "hosting"), allow(dead_code))]
pub(crate) fn perform(
    transport: &dyn gitpoulpi_hosting::HttpTransport,
    base: &str,
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
    let answer = runtime.block_on(gitpoulpi_hosting::updates::check(transport, base, params));
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
    let (state_path, channel, repo) = {
        let s = shared();
        (s.state_path.clone(), s.channel, s.repo.clone())
    };
    let Some(state_path) = state_path else {
        return Outcome::NotDue;
    };
    let transport = match gitpoulpi_hosting::ReqwestTransport::new() {
        Ok(t) => t,
        Err(e) => return Outcome::Failed(e.to_string()),
    };
    let params = CheckParams {
        repo,
        running: RUNNING.to_string(),
        channel,
        target: gitpoulpi_config::release::host_target(),
        token: github_token(),
        pin: None,
    };
    perform(
        &transport,
        &gitpoulpi_hosting::updates::api_base(),
        &params,
        &state_path,
        force,
        unix_now(),
    )
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

/// Serialises the tests that touch the process-wide state.
#[cfg(test)]
pub(crate) static TEST_LOCK: Mutex<()> = Mutex::new(());

#[cfg(test)]
mod tests;

#[cfg(test)]
mod auto_tests;
