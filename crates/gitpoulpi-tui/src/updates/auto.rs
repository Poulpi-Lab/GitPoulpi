//! The self-update's side of the UI: download, verify, unpack and stage the
//! release, then hand the swap to `gitpoulpi-updater` (the install-channels
//! design). The engine (download, checksum, unpack, stage) is
//! `gitpoulpi-update`; the swap, the daemon stop and the restart are the
//! updater's ([`super::handoff`]): this module only prepares the job and
//! asks the window to end.
//!
//! **What may happen** is the install channel's [`Procedure`] (`plans`):
//! `Swap` channels (archive, the Inno installers) download and stage here,
//! then the updater replaces the files; `Run` channels (Chocolatey, winget,
//! Scoop, Homebrew) hand the package manager's command to the updater;
//! everything else (apt, AUR, cargo, Docker, managed installs, a folder this
//! user cannot write) is shown in the panel and never run by the app.
//!
//! **Flow of a swap.** `Update now` (or `auto` mode, once a check found a
//! release) runs a tracked, cancellable job on a thread of its own: download,
//! verify the `.sha256`, unpack, stage `<name>.new` beside the old files (for
//! an all-users install the verified archive waits in the user's cache and
//! the elevated worker re-verifies it). Nothing running is touched and
//! nothing restarts. The panel then offers `Update and restart`, which writes
//! the job and ends the window; `Update and restart` on a release that is not
//! downloaded yet downloads it first and then does the same.
//!
//! **Revert.** The updater keeps the previous files as `<name>.old`. `Revert
//! to <previous>` hands a revert job over, until the new version has been
//! running for [`CLEANUP_AFTER`]: then the `.old` files are deleted. A
//! Windows exe that was running at the time of the swap cannot be deleted
//! until its process ends, so that cleanup simply runs again on a later
//! start. None of this is journaled in git undo: it changes GitPoulpi's own
//! files, not a repository.
//!
//! The threads never touch the render thread: it reads the state under a
//! short lock and asks for a redraw when it changes.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use gitpoulpi_config::release;
use gitpoulpi_hosting::updates::{Release, Version};
use gitpoulpi_i18n::msg::updates as msg;
use gitpoulpi_ipc::JobState;
use gitpoulpi_update::channel::Channel;
use gitpoulpi_update::fsops::{FileOps, RealFs};
use gitpoulpi_update::install;
use gitpoulpi_update::job::{Action, Staged};
use gitpoulpi_update::plan::{self, Elevation, Procedure};
use gitpoulpi_update::{archive, fetch, managed_names, Updatable, UpdateError};

use super::{handoff, plans, shared};

/// How long the new version must have been running before the previous
/// version's `.old` files are deleted (the "successful start").
#[cfg_attr(not(feature = "hosting"), allow(dead_code))]
pub(crate) const CLEANUP_AFTER: Duration = Duration::from_secs(60);

/// Where the update stands.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) enum Phase {
    #[default]
    Idle,
    /// The archive is coming down (the version).
    Downloading(String),
    /// Staged beside the old files; waiting for the user's restart.
    Ready(String),
    /// The job is being handed to the updater (or the swap is running).
    Installing(String),
    /// The last attempt failed (the reason).
    Failed(String),
}

#[derive(Default)]
pub(super) struct InstallState {
    probed: bool,
    pub(super) updatable: Option<Updatable>,
    /// This process's exe, read before any swap (Linux reports a replaced
    /// exe as "... (deleted)" afterwards).
    own_exe: Option<PathBuf>,
    pub(super) phase: Phase,
    /// Bytes received and the total, while downloading.
    progress: Option<(u64, Option<u64>)>,
    cancel: Option<Arc<AtomicBool>>,
    /// The version `Revert` brings back.
    pub(super) revert_to: Option<String>,
    /// `Update and restart` was asked before the download was there: the
    /// handoff follows the download.
    then_restart: bool,
    /// An all-users install: the verified archive waiting in the user's
    /// cache and its checksum URL (the elevated worker checks it again).
    /// The archive in the cache, its `.sha256` URL and its `.minisig` URL.
    cached: Option<(PathBuf, String, String)>,
    /// The job is ready: the window ends, then the updater starts.
    handoff: Option<handoff::Prepared>,
    quit: bool,
}

impl InstallState {
    pub(super) const fn new() -> Self {
        Self {
            probed: false,
            updatable: None,
            own_exe: None,
            phase: Phase::Idle,
            progress: None,
            cancel: None,
            revert_to: None,
            then_restart: false,
            cached: None,
            handoff: None,
            quit: false,
        }
    }
}

/// What the screen needs of the install state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InstallView {
    /// This build swaps its own files (a `Swap` procedure, files to replace).
    pub(crate) updatable: bool,
    pub(crate) phase: Phase,
    /// The version a revert brings back, when this install can go back.
    pub(crate) revert_to: Option<String>,
    /// What an update does here; `Nothing` until the install is known.
    pub(crate) procedure: Procedure,
    /// What going back does.
    pub(crate) revert: Procedure,
    /// How GitPoulpi was installed; `None` until read.
    pub(crate) channel: Option<Channel>,
    /// The administrator's message, shown in the panel.
    pub(crate) message: Option<String>,
}

pub(super) fn view(s: &super::Shared) -> InstallView {
    let plans = super::plans_of(s);
    let swap = matches!(plans.procedure, Procedure::Swap { .. });
    let can_revert = matches!(plans.revert, Procedure::Swap { .. } | Procedure::Run { .. });
    InstallView {
        updatable: s.install.updatable.is_some() && swap,
        phase: s.install.phase.clone(),
        revert_to: s.install.revert_to.clone().filter(|_| can_revert),
        procedure: plans.procedure,
        revert: plans.revert,
        channel: s.facts.as_ref().map(plans::Facts::channel),
        message: plans.effective.message,
    }
}

/// Bytes received so far and the total, while a download runs.
pub(crate) fn progress() -> Option<(u64, Option<u64>)> {
    shared().install.progress
}

#[cfg(test)]
pub(super) fn set_progress(done: u64, total: Option<u64>) {
    shared().install.progress = Some((done, total));
}

/// Remembers this process's exe (before any swap changes it).
pub(super) fn probe(s: &mut super::Shared) {
    if s.install.probed {
        return;
    }
    s.install.probed = true;
    s.install.own_exe = std::env::current_exe().ok();
}

/// The files this install can swap: its folder, exe and marker. Only the
/// channels the updater replaces itself have them.
fn updatable_from(facts: &plans::Facts) -> Option<Updatable> {
    use gitpoulpi_config::release::InstallChannel as C;
    match facts.channel() {
        Channel::Install(C::Archive | C::InnoUser | C::InnoMachine) => {
            facts.resolved.marker.clone().map(|marker| Updatable {
                dir: facts.resolved.dir.clone(),
                exe_name: facts.resolved.exe_name.clone(),
                marker,
            })
        }
        _ => None,
    }
}

/// Stores what the background read found and looks at the install folder:
/// is a download staged, is there a previous version.
#[cfg_attr(not(feature = "hosting"), allow(dead_code))]
pub(super) fn apply_facts(s: &mut super::Shared, facts: plans::Facts) {
    s.install.updatable = updatable_from(&facts);
    s.facts = Some(facts);
    refresh(s, &RealFs);
    if s.install.revert_to.is_some() {
        schedule_cleanup(CLEANUP_AFTER);
    }
}

/// Re-reads the staged and previous versions from the install folder.
fn refresh(s: &mut super::Shared, fs: &dyn FileOps) {
    let Some(updatable) = s.install.updatable.clone() else {
        return;
    };
    let names = managed_names(&updatable.marker.target);
    s.install.revert_to = install::previous_version(fs, &updatable.dir);
    if let Phase::Idle | Phase::Failed(_) = s.install.phase {
        let staged = install::staged_version(fs, &updatable.dir);
        let newer = staged.filter(|v| {
            matches!((Version::parse(v), Version::parse(super::RUNNING)), (Some(a), Some(b)) if a > b)
        });
        match newer {
            Some(version) => s.install.phase = Phase::Ready(version),
            // A leftover of an older download: not this build's next version.
            None => install::discard(fs, &updatable.dir, &names),
        }
    }
}

/// Test seam: pretends the build is (not) an installed release, as an
/// archive install would be.
#[cfg(test)]
pub(super) fn set_updatable(s: &mut super::Shared, updatable: Option<Updatable>, fs: &dyn FileOps) {
    use gitpoulpi_config::release::InstallChannel;
    use gitpoulpi_update::channel::{How, Resolved};
    s.install = InstallState::new();
    s.install.probed = true;
    s.facts = updatable.as_ref().map(|u| plans::Facts {
        resolved: Resolved {
            channel: Channel::Install(InstallChannel::Archive),
            how: How::Marker,
            dir: u.dir.clone(),
            exe_name: u.exe_name.clone(),
            marker: Some(u.marker.clone()),
        },
        policy: Default::default(),
        apt_source_present: false,
        aur_helper: None,
        binstall: false,
    });
    s.install.updatable = updatable;
    refresh(s, fs);
}

/// Test seam: pretends the machine reads as `facts` (a channel, a policy).
#[cfg(test)]
pub(super) fn set_facts(s: &mut super::Shared, facts: Option<plans::Facts>) {
    s.install = InstallState::new();
    s.install.probed = true;
    s.install.updatable = facts.as_ref().and_then(updatable_from);
    s.facts = facts;
}

/// What the updater does to the files (the tests stand in for it): the swap
/// or the revert of an install folder.
#[cfg(test)]
pub(super) fn swap_files(
    fs: &dyn FileOps,
    updatable: &Updatable,
    revert: bool,
) -> Result<(), UpdateError> {
    use gitpoulpi_update::install::Replace;
    let names = managed_names(&updatable.marker.target);
    let replace = Replace::for_target(&updatable.marker.target);
    if revert {
        install::revert(fs, &updatable.dir, &names, replace, &updatable.exe_name)
    } else {
        install::apply(fs, &updatable.dir, &names, replace, &updatable.exe_name)
    }
}

#[cfg_attr(not(feature = "hosting"), allow(dead_code))]
fn schedule_cleanup(after: Duration) {
    let _ = std::thread::Builder::new()
        .name("update-cleanup".into())
        .spawn(move || {
            std::thread::sleep(after);
            cleanup_now(&RealFs);
        });
}

/// Deletes the previous version's `.old` files: the running version proved
/// itself. Files still in use stay for a later start.
pub(super) fn cleanup_now(fs: &dyn FileOps) {
    let Some(updatable) = shared().install.updatable.clone() else {
        return;
    };
    let names = managed_names(&updatable.marker.target);
    let mut done = install::cleanup(fs, &updatable.dir, &names);
    // Programs this release retired: no longer started by anything.
    let retired = release::retired_names(&updatable.marker.target);
    let gone = install::cleanup_retired(fs, &updatable.dir, &retired);
    done.removed.extend(gone.removed);
    done.kept.extend(gone.kept);
    if !done.removed.is_empty() {
        tracing::info!(target: "updates", "removed the previous version's files: {:?}", done.removed);
    }
    let mut s = shared();
    s.install.revert_to = install::previous_version(fs, &updatable.dir);
    drop(s);
    crate::redraw::poke();
}

fn busy(phase: &Phase) -> bool {
    matches!(phase, Phase::Downloading(_) | Phase::Installing(_))
}

/// What an action needs of the state, read under one lock.
struct Now {
    plans: plans::Plans,
    release: Option<Release>,
    phase: Phase,
    has_files: bool,
}

fn now() -> Now {
    let s = shared();
    Now {
        plans: super::plans_of(&s),
        release: s.known.clone(),
        phase: s.install.phase.clone(),
        has_files: s.install.updatable.is_some(),
    }
}

/// Why a swap or run action does not apply: a build with no install record
/// cannot update itself; every other install updates its own way.
fn manual_why(procedure: &Procedure) -> String {
    if matches!(procedure, Procedure::Nothing(_)) {
        msg::WHY_NOT_UPDATABLE.get().to_string()
    } else {
        msg::WHY_MANUAL.get().to_string()
    }
}

/// The message for a release that has nothing to download for this platform.
fn no_archive() -> String {
    msg::msg_update_failed("this release has no archive for this platform") // i18n-english
}

/// The checks every swap action starts with. `Err` is the status line.
fn swap_ready(n: &Now) -> Result<(), String> {
    if !n.has_files {
        return Err(msg::WHY_NOT_UPDATABLE.get().to_string());
    }
    if busy(&n.phase) {
        return Err(msg::WHY_UPDATE_BUSY.get().to_string());
    }
    Ok(())
}

/// `Updates: Update now`. A swap channel downloads and stages the release (the
/// restart is a separate action); a package-manager channel runs its
/// command; everything else points at the update panel.
pub(crate) fn update_now() -> Option<String> {
    let n = now();
    if n.plans.effective.mode == gitpoulpi_config::UpdateMode::Off {
        return Some(msg::MSG_OFF.get().to_string());
    }
    match n.plans.procedure {
        Procedure::Swap { .. } => {
            if let Err(why) = swap_ready(&n) {
                return Some(why);
            }
            if matches!(n.phase, Phase::Ready(_)) {
                return Some(msg::WHY_UPDATE_READY.get().to_string());
            }
            let Some(release) = n.release else {
                return Some(msg::WHY_NO_UPDATE.get().to_string());
            };
            if release.asset.is_none() {
                // Nothing to download for this platform: the release page is the way.
                return Some(no_archive());
            }
            start_download(release, Duration::ZERO, false)
        }
        Procedure::Run { .. } => run_manager(),
        other => Some(manual_why(&other)),
    }
}

/// The panel's main button: a swap channel downloads (when it has not) and
/// then hands over to the updater; a package-manager channel runs its
/// command.
pub(crate) fn update_and_restart() -> Option<String> {
    let n = now();
    if n.plans.effective.mode == gitpoulpi_config::UpdateMode::Off {
        return Some(msg::MSG_OFF.get().to_string());
    }
    match n.plans.procedure {
        Procedure::Swap { .. } => {
            if let Err(why) = swap_ready(&n) {
                return Some(why);
            }
            if matches!(n.phase, Phase::Ready(_)) {
                return restart_now();
            }
            let Some(release) = n.release else {
                return Some(msg::WHY_NO_UPDATE.get().to_string());
            };
            if release.asset.is_none() {
                return Some(no_archive());
            }
            start_download(release, Duration::ZERO, true)
        }
        Procedure::Run { .. } => run_manager(),
        other => Some(manual_why(&other)),
    }
}

/// `auto` mode: a check (or the cache) knows a release this build can take.
/// Quiet: no status line until it is ready. Only where the updater swaps
/// files; a package-manager channel behaves like `notify`.
pub(super) fn auto_start(delay: Duration) {
    let release = {
        let s = shared();
        let plans = super::plans_of(&s);
        if !plan::stages_in_background(&plans.procedure, &plans.effective)
            || s.install.updatable.is_none()
            || s.skipped
            || !matches!(s.install.phase, Phase::Idle)
        {
            return;
        }
        match s.known.clone() {
            Some(r) => r,
            None => return,
        }
    };
    // Nothing to download for this platform: the notice stays "available".
    if release.asset.is_none() {
        return;
    }
    start_download(release, delay, false);
}

fn start_download(release: Release, delay: Duration, then_restart: bool) -> Option<String> {
    let version = release.version.clone();
    let cancel = Arc::new(AtomicBool::new(false));
    let (updatable, cache) = {
        let mut s = shared();
        let updatable = s.install.updatable.clone()?;
        if busy(&s.install.phase) {
            return Some(msg::WHY_UPDATE_BUSY.get().to_string());
        }
        // An all-users install stages in the user's cache: the folder is not
        // ours to write.
        let cache = matches!(
            super::plans_of(&s).procedure,
            Procedure::Swap {
                elevation: Elevation::Uac,
                ..
            }
        );
        s.install.phase = Phase::Downloading(version.clone());
        s.install.progress = None;
        s.install.cancel = Some(cancel.clone());
        s.install.cached = None;
        s.install.then_restart = then_restart;
        (updatable, cache)
    };
    let stop = cancel.clone();
    let job = crate::jobs_ui::local_start(
        msg::job_download(&version),
        Arc::new(move || stop.store(true, Ordering::SeqCst)),
    );
    crate::redraw::poke();
    let spawned = std::thread::Builder::new()
        .name("update-download".into())
        .spawn({
            let version = version.clone();
            move || {
                if !delay.is_zero() {
                    std::thread::sleep(delay);
                }
                let last = std::sync::atomic::AtomicU64::new(u64::MAX);
                let on_progress = |done: u64, total: Option<u64>| {
                    shared().install.progress = Some((done, total));
                    // Redraw (and relabel the job) when the percentage moves,
                    // not for every chunk.
                    let pct = total.filter(|t| *t > 0).map_or(0, |t| done * 100 / t);
                    if last.swap(pct, Ordering::SeqCst) != pct {
                        crate::jobs_ui::local_relabel(
                            job,
                            format!("{} ({pct}%)", msg::job_download(&version)),
                        );
                        crate::redraw::poke();
                    }
                };
                let result = download_and_stage(&release, &updatable, cache, &cancel, &on_progress);
                finish_download(job, &version, result);
            }
        });
    if spawned.is_err() {
        finish_download(
            job,
            &version,
            Err(UpdateError::Download("could not start a thread".into())), // i18n-english
        );
    }
    Some(msg::msg_update_started(&version))
}

#[cfg(feature = "hosting")]
fn download_and_stage(
    release: &Release,
    updatable: &Updatable,
    cache: bool,
    cancel: &AtomicBool,
    on_progress: fetch::OnProgress<'_>,
) -> Result<(), UpdateError> {
    // The policy's `download_base` mirror is the one extra host allowed.
    let mirror = super::extra_download_hosts();
    // Production: the compiled-in release keys, nothing else.
    let keys = gitpoulpi_update::signing::release_keys();
    let transport = gitpoulpi_hosting::ReqwestTransport::new()
        .map_err(|e| UpdateError::Download(e.to_string()))?
        .with_download_hosts(mirror.clone());
    if cache {
        let dir = handoff::paths()
            .map(|p| p.download)
            .ok_or_else(|| UpdateError::Download("no data folder for the download".into()))?; // i18n-english
        let path = fetch_to_cache(
            &transport,
            release,
            updatable,
            &dir,
            cancel,
            on_progress,
            &mirror,
            &keys,
        )?;
        let sha_url = release
            .checksum
            .as_ref()
            .map(|c| c.url.clone())
            .ok_or(UpdateError::NoChecksum)?;
        let sig_url = release
            .asset
            .as_ref()
            .map(|a| fetch::minisig_url(&a.url))
            .ok_or(UpdateError::NoSignature)?;
        shared().install.cached = Some((path, sha_url, sig_url));
        return Ok(());
    }
    fetch_and_stage(
        &transport,
        &RealFs,
        release,
        updatable,
        cancel,
        on_progress,
        &mirror,
        &keys,
    )
}

#[cfg(not(feature = "hosting"))]
fn download_and_stage(
    _release: &Release,
    _updatable: &Updatable,
    _cache: bool,
    _cancel: &AtomicBool,
    _on_progress: fetch::OnProgress<'_>,
) -> Result<(), UpdateError> {
    Err(UpdateError::Download(
        "this build has no network support".into(), // i18n-english
    ))
}

fn runtime() -> Result<tokio::runtime::Runtime, UpdateError> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| UpdateError::Download(e.to_string()))
}

/// Download, verify, unpack and stage `release` beside `updatable`'s files.
/// Plain inputs, so the tests drive it with a recorded transport and a temp
/// folder.
#[cfg_attr(not(feature = "hosting"), allow(dead_code))]
#[allow(clippy::too_many_arguments)]
pub(super) fn fetch_and_stage(
    transport: &dyn gitpoulpi_hosting::HttpTransport,
    fs: &dyn FileOps,
    release: &Release,
    updatable: &Updatable,
    cancel: &AtomicBool,
    on_progress: fetch::OnProgress<'_>,
    extra_hosts: &[String],
    keys: &[gitpoulpi_update::signing::PublicKey],
) -> Result<(), UpdateError> {
    let target = &updatable.marker.target;
    let bytes = runtime()?.block_on(fetch::download_verified_with(
        transport,
        release,
        target,
        cancel,
        on_progress,
        extra_hosts,
        keys,
    ))?;
    let files = archive::extract(&bytes, &release.version, target)?;
    if !files.iter().any(|f| f.name == updatable.exe_name) {
        return Err(UpdateError::Archive(format!(
            "the archive has no {}", // i18n-english
            updatable.exe_name
        )));
    }
    let names = managed_names(target);
    // A download of an older attempt must not mix with this one.
    install::discard(fs, &updatable.dir, &names);
    install::stage(fs, &updatable.dir, &files)?;
    Ok(())
}

/// Download and verify `release`, check it unpacks, and keep the archive in
/// `dir` for the elevated worker (an all-users install: the install folder
/// cannot be written without the administrator's approval). Returns the file.
#[cfg_attr(not(feature = "hosting"), allow(dead_code))]
#[allow(clippy::too_many_arguments)]
pub(super) fn fetch_to_cache(
    transport: &dyn gitpoulpi_hosting::HttpTransport,
    release: &Release,
    updatable: &Updatable,
    dir: &std::path::Path,
    cancel: &AtomicBool,
    on_progress: fetch::OnProgress<'_>,
    extra_hosts: &[String],
    keys: &[gitpoulpi_update::signing::PublicKey],
) -> Result<PathBuf, UpdateError> {
    let target = &updatable.marker.target;
    let bytes = runtime()?.block_on(fetch::download_verified_with(
        transport,
        release,
        target,
        cancel,
        on_progress,
        extra_hosts,
        keys,
    ))?;
    let files = archive::extract(&bytes, &release.version, target)?;
    if !files.iter().any(|f| f.name == updatable.exe_name) {
        return Err(UpdateError::Archive(format!(
            "the archive has no {}", // i18n-english
            updatable.exe_name
        )));
    }
    let io = |what: &str, e: std::io::Error| UpdateError::Io {
        what: what.to_string(),
        error: e.to_string(),
    };
    let creating = "creating the download folder"; // i18n-english
    let saving = "saving the download"; // i18n-english
    std::fs::create_dir_all(dir).map_err(|e| io(creating, e))?;
    let path = dir.join(release::archive_name(&release.version, target));
    std::fs::write(&path, &bytes).map_err(|e| io(saving, e))?;
    Ok(path)
}

pub(super) fn finish_download(job: u64, version: &str, result: Result<(), UpdateError>) {
    let (state, message, restart) = {
        let mut s = shared();
        s.install.progress = None;
        s.install.cancel = None;
        let restart = std::mem::take(&mut s.install.then_restart);
        match result {
            Ok(()) => {
                s.install.phase = Phase::Ready(version.to_string());
                (JobState::Done, msg::msg_update_ready(version), restart)
            }
            Err(UpdateError::Canceled) => {
                s.install.phase = Phase::Idle;
                (
                    JobState::Canceled,
                    msg::MSG_UPDATE_CANCELED.get().to_string(),
                    false,
                )
            }
            Err(e) => {
                tracing::warn!(target: "updates", "update download failed: {e}");
                s.install.phase = Phase::Failed(e.to_string());
                (
                    JobState::Failed,
                    msg::msg_update_failed(e.to_string()),
                    false,
                )
            }
        }
    };
    shared().message = Some(message);
    crate::jobs_ui::local_finish(job, state);
    crate::redraw::poke();
    // `Update and restart` was pressed before the download was there.
    if restart {
        if let Some(line) = restart_now() {
            shared().message = Some(line);
        }
    }
}

/// `Updates: Restart to update`: hands the staged release to the updater.
pub(crate) fn restart_now() -> Option<String> {
    let n = now();
    if !matches!(n.plans.procedure, Procedure::Swap { .. }) {
        return Some(manual_why(&n.plans.procedure));
    }
    if let Err(why) = swap_ready(&n) {
        return Some(why);
    }
    match n.phase {
        Phase::Ready(version) => begin_handoff(Kind::Apply, version),
        _ => Some(msg::WHY_NOTHING_STAGED.get().to_string()),
    }
}

/// `Updates: Revert to the previous version`.
pub(crate) fn revert_now() -> Option<String> {
    let n = now();
    let previous = shared().install.revert_to.clone();
    match n.plans.revert {
        Procedure::Swap { .. } => {
            if let Err(why) = swap_ready(&n) {
                return Some(why);
            }
            match previous {
                Some(version) => begin_handoff(Kind::Revert, version),
                None => Some(msg::WHY_NO_REVERT.get().to_string()),
            }
        }
        Procedure::Run { argv, elevation } => {
            if busy(&n.phase) {
                return Some(msg::WHY_UPDATE_BUSY.get().to_string());
            }
            match previous {
                Some(version) => begin_handoff(Kind::Manager(argv, elevation), version),
                None => Some(msg::WHY_NO_REVERT.get().to_string()),
            }
        }
        _ => Some(msg::WHY_NO_REVERT.get().to_string()),
    }
}

/// Runs the package manager's update command through the updater.
pub(crate) fn run_manager() -> Option<String> {
    let n = now();
    let Procedure::Run { argv, elevation } = n.plans.procedure else {
        return Some(manual_why(&n.plans.procedure));
    };
    if busy(&n.phase) {
        return Some(msg::WHY_UPDATE_BUSY.get().to_string());
    }
    let Some(release) = n.release else {
        return Some(msg::WHY_NO_UPDATE.get().to_string());
    };
    begin_handoff(Kind::Manager(argv, elevation), release.version)
}

/// What the updater is asked to do.
enum Kind {
    Apply,
    Revert,
    Manager(Vec<String>, Elevation),
}

/// Starts the handoff: the job and the updater copy are made on a worker
/// thread, then the window is asked to end ([`take_quit_request`]) and
/// [`launch_if_requested`] starts the updater once the terminal is restored.
fn begin_handoff(kind: Kind, to_version: String) -> Option<String> {
    let (install_dir, own_exe, staged, elevation, restore) = {
        let mut s = shared();
        if busy(&s.install.phase) {
            return Some(msg::WHY_UPDATE_BUSY.get().to_string());
        }
        let plans = super::plans_of(&s);
        let install_dir = s
            .install
            .updatable
            .as_ref()
            .map(|u| u.dir.clone())
            .or_else(|| s.facts.as_ref().map(|f| f.resolved.dir.clone()))?;
        let own_exe = s
            .install
            .own_exe
            .clone()
            .or_else(|| std::env::current_exe().ok())?;
        let swap_elevation = |p: &Procedure| match p {
            Procedure::Swap { elevation, .. } => *elevation,
            _ => Elevation::None,
        };
        let staged = match (&kind, s.install.cached.clone()) {
            (Kind::Apply, Some((archive, sha256_url, minisig_url))) => Staged::Archive {
                archive,
                sha256_url,
                minisig_url,
            },
            _ => Staged::InPlace,
        };
        let elevation = match &kind {
            Kind::Apply => swap_elevation(&plans.procedure),
            Kind::Revert => swap_elevation(&plans.revert),
            Kind::Manager(_, e) => *e,
        };
        let restore = match kind {
            Kind::Apply => Phase::Ready(to_version.clone()),
            _ => Phase::Idle,
        };
        s.install.phase = Phase::Installing(to_version.clone());
        (install_dir, own_exe, staged, elevation, restore)
    };
    crate::redraw::poke();
    let (action, manager_argv) = match kind {
        Kind::Apply => (Action::Apply, None),
        Kind::Revert => (Action::Revert, None),
        Kind::Manager(argv, _) => (Action::Manager, Some(argv)),
    };
    let work = {
        let (to_version, restore) = (to_version.clone(), restore.clone());
        move || {
            let result = (|| {
                let paths = handoff::paths().ok_or_else(|| "no data folder".to_string())?; // i18n-english
                let config = gitpoulpi_config::ConfigPaths::discover()
                    .ok_or_else(|| "no config folder".to_string())?; // i18n-english
                let config_dir = config.config_dir().to_path_buf();
                let job = handoff::build_job(handoff::Inputs {
                    action,
                    install_dir,
                    from_version: super::RUNNING.to_string(),
                    to_version: to_version.clone(),
                    staged,
                    manager_argv,
                    elevation,
                    own_exe,
                    args: std::env::args_os()
                        .skip(1)
                        .map(|a| a.to_string_lossy().into_owned())
                        .collect(),
                    cwd: std::env::current_dir().unwrap_or_default(),
                    pid: std::process::id(),
                    config_dir,
                    status_path: paths.status.clone(),
                    environment: config.env().map(str::to_string),
                    install_id: config.install_id().map(str::to_string),
                });
                handoff::prepare(&job, &paths, &release::host_target())
            })();
            finish_handoff(result, &to_version, restore);
        }
    };
    if std::thread::Builder::new()
        .name("update-handoff".into())
        .spawn(work)
        .is_err()
    {
        finish_handoff(
            Err("could not start a thread".into()), // i18n-english
            &to_version,
            restore,
        );
    }
    None
}

pub(super) fn finish_handoff(
    result: Result<handoff::Prepared, String>,
    version: &str,
    restore: Phase,
) {
    {
        let mut s = shared();
        match result {
            Ok(prepared) => {
                // The job is written: the window now ends and the updater
                // starts (`launch_if_requested`).
                s.install.handoff = Some(prepared);
                s.install.quit = true;
                s.message = Some(msg::msg_restarting(version));
            }
            Err(why) => {
                tracing::warn!(target: "updates", "could not hand the update over: {why}");
                s.install.phase = restore;
                s.message = Some(msg::msg_handoff_failed(why));
            }
        }
    }
    crate::redraw::poke();
}

/// Whether the window must quit now (the job is written and the user asked
/// for the update). Read by the loop's tick.
pub(crate) fn take_quit_request() -> bool {
    std::mem::take(&mut shared().install.quit)
}

/// After the window ended and the terminal is restored: starts the updater
/// in place of this process (Unix `exec`; Windows starts it detached and
/// exits). No-op unless a handoff asked for it. Returns only when the
/// updater could not start.
pub(crate) fn launch_if_requested() {
    let Some(prepared) = shared().install.handoff.take() else {
        return;
    };
    let why = handoff::launch(&prepared);
    eprintln!("gitpoulpi: could not start the updater: {why}"); // i18n-english
}
