//! Slice B of the updates-and-install design: replacing the
//! running build by the newer release, driven from the UI. The engine
//! (download, checksum, unpack, stage, swap, revert) is `gitpoulpi-update`;
//! this module is the state, the background threads and the restart.
//!
//! **When it applies.** Only a build installed from a release archive: the
//! marker `.gitpoulpi-dist` (written by `xtask dist`) sits next to its exe. A
//! `cargo` build never updates itself; the Release notes tab says so.
//!
//! **Flow.** `Update now` (or `auto` mode, once a check found a release) runs
//! a tracked, cancellable job on a thread of its own: download, verify the
//! `.sha256`, unpack, stage `<name>.new` beside the old files. Nothing
//! running is touched and nothing restarts. The chip and the tab then offer
//! `Restart to update`: on that action the session's standalone daemon is
//! stopped, the files are swapped (all or nothing), and the TUI quits and
//! re-executes itself with the same arguments once its terminal is restored
//! ([`relaunch_if_requested`]).
//!
//! **Revert.** The swap keeps the previous files as `<name>.old`. `Revert to
//! <previous>` (tab button and palette) swaps them back, until the new
//! version has been running for [`CLEANUP_AFTER`]: then the `.old` files are
//! deleted. A Windows exe that was running at the time of the swap cannot be
//! deleted until its process ends, so that cleanup simply runs again on a
//! later start. None of this is journaled in git undo: it changes GitPoulpi's
//! own files, not a repository.
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
use gitpoulpi_update::fsops::{FileOps, RealFs};
use gitpoulpi_update::install::{self, Replace};
use gitpoulpi_update::{archive, fetch, managed_names, Updatable, UpdateError};

use super::shared;

/// How long the new version must have been running before the previous
/// version's `.old` files are deleted (the "successful start").
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
    /// The swap (or the revert) is running.
    Installing(String),
    /// The last attempt failed (the reason).
    Failed(String),
}

/// What to run again once the window is gone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Relaunch {
    pub(crate) exe: PathBuf,
    pub(crate) args: Vec<std::ffi::OsString>,
}

#[derive(Default)]
pub(super) struct InstallState {
    probed: bool,
    updatable: Option<Updatable>,
    /// This process's exe, read before any swap (Linux reports a replaced
    /// exe as "... (deleted)" afterwards).
    own_exe: Option<PathBuf>,
    pub(super) phase: Phase,
    /// Bytes received and the total, while downloading.
    progress: Option<(u64, Option<u64>)>,
    cancel: Option<Arc<AtomicBool>>,
    /// The version `Revert` brings back.
    revert_to: Option<String>,
    relaunch: Option<Relaunch>,
    quit: bool,
    session: String,
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
            relaunch: None,
            quit: false,
            session: String::new(),
        }
    }
}

/// What the screen needs of the install state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InstallView {
    pub(crate) updatable: bool,
    pub(crate) phase: Phase,
    pub(crate) revert_to: Option<String>,
}

pub(super) fn view(s: &super::Shared) -> InstallView {
    InstallView {
        updatable: s.install.updatable.is_some(),
        phase: s.install.phase.clone(),
        revert_to: s.install.revert_to.clone(),
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

/// Names the session whose standalone daemon is stopped before a swap.
pub(crate) fn set_session(session: &str) {
    shared().install.session = session.to_string();
}

/// Looks at the install once: is this a release build, is a download staged,
/// is there a previous version. Reads three tiny files; later calls only
/// refresh what an action changed.
pub(super) fn probe(s: &mut super::Shared) {
    if s.install.probed {
        return;
    }
    s.install.probed = true;
    let exe = std::env::current_exe().ok();
    s.install.own_exe = exe.clone();
    s.install.updatable = exe
        .as_deref()
        .and_then(|e| gitpoulpi_update::detect(&RealFs, e, &release::host_target()));
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

/// Test seam: pretends the build is (not) an installed release.
#[cfg(test)]
pub(super) fn set_updatable(s: &mut super::Shared, updatable: Option<Updatable>, fs: &dyn FileOps) {
    s.install = InstallState::new();
    s.install.probed = true;
    s.install.updatable = updatable;
    refresh(s, fs);
}

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
    let done = install::cleanup(fs, &updatable.dir, &names);
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

/// `Updates: Update now`.
pub(crate) fn update_now() -> Option<String> {
    let release = {
        let s = shared();
        if s.mode == gitpoulpi_config::UpdateMode::Off {
            return Some(msg::MSG_OFF.get().to_string());
        }
        if s.install.updatable.is_none() {
            return Some(msg::WHY_NOT_UPDATABLE.get().to_string());
        }
        if busy(&s.install.phase) {
            return Some(msg::WHY_UPDATE_BUSY.get().to_string());
        }
        if matches!(s.install.phase, Phase::Ready(_)) {
            return Some(msg::WHY_UPDATE_READY.get().to_string());
        }
        match s.known.clone() {
            Some(r) => r,
            None => return Some(msg::WHY_NO_UPDATE.get().to_string()),
        }
    };
    if release.asset.is_none() {
        // Nothing to download for this platform: the release page is the way.
        return Some(msg::msg_update_failed(
            "this release has no archive for this platform", // i18n-english
        ));
    }
    start_download(release, Duration::ZERO)
}

/// `auto` mode: a check (or the cache) knows a release this build can take.
/// Quiet: no status line until it is ready.
pub(super) fn auto_start(delay: Duration) {
    let release = {
        let s = shared();
        if s.mode != gitpoulpi_config::UpdateMode::Auto
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
    start_download(release, delay);
}

fn start_download(release: Release, delay: Duration) -> Option<String> {
    let version = release.version.clone();
    let cancel = Arc::new(AtomicBool::new(false));
    let updatable = {
        let mut s = shared();
        let updatable = s.install.updatable.clone()?;
        if busy(&s.install.phase) {
            return Some(msg::WHY_UPDATE_BUSY.get().to_string());
        }
        s.install.phase = Phase::Downloading(version.clone());
        s.install.progress = None;
        s.install.cancel = Some(cancel.clone());
        updatable
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
                let result = download_and_stage(&release, &updatable, &cancel, &on_progress);
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
    cancel: &AtomicBool,
    on_progress: fetch::OnProgress<'_>,
) -> Result<(), UpdateError> {
    let transport = gitpoulpi_hosting::ReqwestTransport::new()
        .map_err(|e| UpdateError::Download(e.to_string()))?;
    fetch_and_stage(&transport, &RealFs, release, updatable, cancel, on_progress)
}

#[cfg(not(feature = "hosting"))]
fn download_and_stage(
    _release: &Release,
    _updatable: &Updatable,
    _cancel: &AtomicBool,
    _on_progress: fetch::OnProgress<'_>,
) -> Result<(), UpdateError> {
    Err(UpdateError::Download(
        "this build has no network support".into(), // i18n-english
    ))
}

/// Download, verify, unpack and stage `release` beside `updatable`'s files.
/// Plain inputs, so the tests drive it with a recorded transport and a temp
/// folder.
#[cfg_attr(not(feature = "hosting"), allow(dead_code))]
pub(super) fn fetch_and_stage(
    transport: &dyn gitpoulpi_hosting::HttpTransport,
    fs: &dyn FileOps,
    release: &Release,
    updatable: &Updatable,
    cancel: &AtomicBool,
    on_progress: fetch::OnProgress<'_>,
) -> Result<(), UpdateError> {
    let target = &updatable.marker.target;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| UpdateError::Download(e.to_string()))?;
    let bytes = runtime.block_on(fetch::download_verified(
        transport,
        release,
        target,
        cancel,
        on_progress,
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

pub(super) fn finish_download(job: u64, version: &str, result: Result<(), UpdateError>) {
    let (state, message) = {
        let mut s = shared();
        s.install.progress = None;
        s.install.cancel = None;
        match result {
            Ok(()) => {
                s.install.phase = Phase::Ready(version.to_string());
                (JobState::Done, msg::msg_update_ready(version))
            }
            Err(UpdateError::Canceled) => {
                s.install.phase = Phase::Idle;
                (
                    JobState::Canceled,
                    msg::MSG_UPDATE_CANCELED.get().to_string(),
                )
            }
            Err(e) => {
                tracing::warn!(target: "updates", "update download failed: {e}");
                s.install.phase = Phase::Failed(e.to_string());
                (JobState::Failed, msg::msg_update_failed(e.to_string()))
            }
        }
    };
    shared().message = Some(message);
    crate::jobs_ui::local_finish(job, state);
    crate::redraw::poke();
}

/// `Updates: Restart to update`: swaps the files, then quits and re-executes.
pub(crate) fn restart_now() -> Option<String> {
    swap(false)
}

/// `Updates: Revert to the previous version`.
pub(crate) fn revert_now() -> Option<String> {
    swap(true)
}

fn swap(revert: bool) -> Option<String> {
    let (updatable, version) = {
        let mut s = shared();
        let Some(updatable) = s.install.updatable.clone() else {
            return Some(msg::WHY_NOT_UPDATABLE.get().to_string());
        };
        if busy(&s.install.phase) {
            return Some(msg::WHY_UPDATE_BUSY.get().to_string());
        }
        let version = if revert {
            match s.install.revert_to.clone() {
                Some(v) => v,
                None => return Some(msg::WHY_NO_REVERT.get().to_string()),
            }
        } else {
            match &s.install.phase {
                Phase::Ready(v) => v.clone(),
                _ => return Some(msg::WHY_NOTHING_STAGED.get().to_string()),
            }
        };
        s.install.phase = Phase::Installing(version.clone());
        (updatable, version)
    };
    let job = crate::jobs_ui::local_start_sent(msg::job_install(&version));
    crate::redraw::poke();
    let session = shared().install.session.clone();
    let spawned = std::thread::Builder::new()
        .name("update-swap".into())
        .spawn({
            let version = version.clone();
            move || {
                stop_standalone_daemon(&session);
                let result = swap_files(&RealFs, &updatable, revert);
                finish_swap(job, &version, revert, result);
            }
        });
    if spawned.is_err() {
        finish_swap(
            job,
            &version,
            revert,
            Err(UpdateError::Download("could not start a thread".into())), // i18n-english
        );
    }
    None
}

/// The file part of a restart or a revert.
pub(super) fn swap_files(
    fs: &dyn FileOps,
    updatable: &Updatable,
    revert: bool,
) -> Result<(), UpdateError> {
    let names = managed_names(&updatable.marker.target);
    let replace = Replace::for_target(&updatable.marker.target);
    if revert {
        install::revert(fs, &updatable.dir, &names, replace, &updatable.exe_name)
    } else {
        install::apply(fs, &updatable.dir, &names, replace, &updatable.exe_name)
    }
}

pub(super) fn finish_swap(job: u64, version: &str, revert: bool, result: Result<(), UpdateError>) {
    let ok = result.is_ok();
    {
        let mut s = shared();
        match &result {
            Ok(()) => {
                // The files are in place; the window now ends and the new
                // binary starts (`relaunch_if_requested`).
                s.install.relaunch = s.install.own_exe.clone().map(|exe| Relaunch {
                    exe,
                    args: std::env::args_os().skip(1).collect(),
                });
                s.install.quit = true;
                s.message = Some(msg::msg_restarting(version));
            }
            Err(e) => {
                tracing::warn!(target: "updates", "update swap failed: {e}");
                // Rolled back: the download (or the previous version) is
                // still there to try again.
                s.install.phase = if revert {
                    Phase::Idle
                } else {
                    Phase::Ready(version.to_string())
                };
                s.message = Some(msg::msg_update_failed(e.to_string()));
            }
        }
        refresh(&mut s, &RealFs);
    }
    crate::jobs_ui::local_finish(job, if ok { JobState::Done } else { JobState::Failed });
    crate::redraw::poke();
}

/// Whether the window must quit now (the swap is done and the user asked for
/// the restart). Read by the loop's tick.
pub(crate) fn take_quit_request() -> bool {
    std::mem::take(&mut shared().install.quit)
}

/// After the window ended and the terminal is restored: starts the new
/// binary in place of this process (Unix `exec`; Windows runs it and exits
/// with its status), with the same arguments. No-op unless a swap asked for it.
pub(crate) fn relaunch_if_requested() {
    let Some(plan) = shared().install.relaunch.take() else {
        return;
    };
    let what = "gitpoulpi: could not restart"; // i18n-english
    let mut cmd = std::process::Command::new(&plan.exe);
    cmd.args(&plan.args)
        // Windows: not a double-click, do not relaunch into another terminal.
        .env("GITPOULPI_RELAUNCHED", "1");
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = cmd.exec();
        eprintln!("{what} {}: {err}", plan.exe.display());
    }
    #[cfg(not(unix))]
    match cmd.status() {
        Ok(status) => std::process::exit(status.code().unwrap_or(0)),
        Err(err) => {
            eprintln!("{what} {}: {err}", plan.exe.display());
        }
    }
}

/// Asks the session's daemon to shut down when it is a standalone process
/// (an embedded one lives in this process and goes with it). Best effort: a
/// binary a daemon still holds makes the swap refuse and say which.
fn stop_standalone_daemon(session: &str) {
    use gitpoulpi_core::domain::OperationKind;
    use gitpoulpi_ipc::{Client, Request};
    if session.is_empty() {
        return;
    }
    let Some(paths) = gitpoulpi_config::ConfigPaths::discover() else {
        return;
    };
    if !paths.ipc_pid_file(session).exists() {
        return;
    }
    let port_file = paths.ipc_port_file(session);
    let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    else {
        return;
    };
    runtime.block_on(async {
        let ask = async {
            let mut client = Client::connect(&port_file).await.ok()?;
            client
                .call(&Request {
                    id: 1,
                    session: session.to_string(),
                    repo: None,
                    command: OperationKind::Shutdown,
                    args: serde_json::Value::Null,
                })
                .await
                .ok()
        };
        if tokio::time::timeout(Duration::from_secs(5), ask)
            .await
            .is_err()
        {
            tracing::warn!(target: "updates", "the session daemon did not stop in time");
        }
        // The daemon removes its port file when it is gone.
        for _ in 0..30 {
            if !port_file.exists() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });
}
