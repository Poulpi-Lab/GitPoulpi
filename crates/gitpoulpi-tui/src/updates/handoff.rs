//! Handing the update to `gitpoulpi-updater` (the install-channels design,
//! 1.3). The TUI never swaps its own files, stops a daemon or waits for a
//! package manager: it writes a job, copies the updater out of the install
//! folder, ends, and the updater does the rest.
//!
//! The steps, in the order the code runs them:
//!
//! 1. [`build_job`] (pure) makes the `UpdateJob` from plain inputs.
//! 2. [`prepare`] (a worker thread, never the render thread) writes
//!    `<data>/updates/job.json` and copies the updater to
//!    `<data>/updates/bin/`, so the install folder's copy is never locked and
//!    a package manager may overwrite it. A staged `gitpoulpi-updater.new`
//!    (the new version's updater) is preferred over the installed one.
//! 3. The window ends and the terminal is restored; [`launch`] then starts
//!    the updater: Unix `exec`s it in the same terminal (its progress shows
//!    there), Windows starts it detached in a console of its own and exits
//!    at once, so no parent waits on a file the updater must replace.
//! 4. On the next start [`startup_status`] turns the updater's
//!    `status.json` into one status line.
//!
//! Tests point the launch at a recording stub through `GITPOULPI_UPDATER_EXE`
//! (compiled in only with `debug_assertions`).

use std::path::{Path, PathBuf};

use gitpoulpi_config::release;
use gitpoulpi_i18n::msg::updates as msg;
use gitpoulpi_update::job::{
    Action, JobElevation, Phase, Relaunch, Staged, UpdateJob, UpdateStatus,
};
use gitpoulpi_update::plan::Elevation;

/// A status file older than this is not news any more.
const STATUS_MAX_AGE_SECS: u64 = 24 * 60 * 60;

/// The test override for the updater to start (debug builds only).
#[cfg(debug_assertions)]
pub(crate) const UPDATER_EXE_ENV: &str = "GITPOULPI_UPDATER_EXE";

/// What a job is made of.
#[derive(Debug, Clone)]
pub(crate) struct Inputs {
    pub(crate) action: Action,
    pub(crate) install_dir: PathBuf,
    pub(crate) from_version: String,
    pub(crate) to_version: String,
    pub(crate) staged: Staged,
    pub(crate) manager_argv: Option<Vec<String>>,
    pub(crate) elevation: Elevation,
    /// This process, to be started again afterwards.
    pub(crate) own_exe: PathBuf,
    pub(crate) args: Vec<String>,
    pub(crate) cwd: PathBuf,
    pub(crate) pid: u32,
    pub(crate) config_dir: PathBuf,
    pub(crate) status_path: PathBuf,
}

pub(crate) fn build_job(i: Inputs) -> UpdateJob {
    UpdateJob {
        schema: gitpoulpi_update::job::SCHEMA,
        action: i.action,
        install_dir: i.install_dir,
        from_version: i.from_version,
        to_version: i.to_version,
        staged: i.staged,
        manager_argv: i.manager_argv,
        elevation: match i.elevation {
            Elevation::None => JobElevation::None,
            Elevation::Uac => JobElevation::Uac,
        },
        relaunch: Relaunch {
            exe: i.own_exe,
            args: i.args,
            cwd: i.cwd,
        },
        requester_pid: i.pid,
        config_dir: i.config_dir,
        status_path: i.status_path,
    }
}

/// The files the handoff uses, under `<data dir>/updates/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Paths {
    pub(crate) dir: PathBuf,
    pub(crate) job: PathBuf,
    pub(crate) status: PathBuf,
    /// Where the updater runs from.
    pub(crate) bin: PathBuf,
    /// Where an archive for an elevated worker waits (all-users installs).
    pub(crate) download: PathBuf,
}

#[cfg(test)]
pub(crate) fn paths_in(data_dir: &Path) -> Paths {
    paths_under(data_dir.join("updates"))
}

/// The files under an installation's own updates folder
/// (`ConfigPaths::updates_dir`).
pub(crate) fn paths_under(dir: PathBuf) -> Paths {
    Paths {
        job: dir.join("job.json"),
        status: dir.join("status.json"),
        bin: dir.join("bin"),
        download: dir.join("download"),
        dir,
    }
}

/// The real location, `None` without a config location.
pub(crate) fn paths() -> Option<Paths> {
    gitpoulpi_config::ConfigPaths::discover().map(|p| paths_under(p.updates_dir()))
}

pub(crate) fn updater_name(target: &str) -> String {
    release::exe_name("gitpoulpi-updater", target)
}

/// The updater to copy: a staged new one, else the installed one. `exists`
/// is the file check, so the choice is tested without files.
pub(crate) fn pick_updater(
    install_dir: &Path,
    target: &str,
    exists: &dyn Fn(&Path) -> bool,
) -> Option<PathBuf> {
    let name = updater_name(target);
    let staged = install_dir.join(format!("{name}.new"));
    let installed = install_dir.join(&name);
    if exists(&staged) {
        Some(staged)
    } else if exists(&installed) {
        Some(installed)
    } else {
        None
    }
}

/// What [`prepare`] made ready for [`launch`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Prepared {
    pub(crate) updater: PathBuf,
    pub(crate) job: PathBuf,
}

/// Writes the job and copies the updater. Blocking file work: a worker
/// thread calls it. The error is English technical text for the status line.
pub(crate) fn prepare(job: &UpdateJob, paths: &Paths, target: &str) -> Result<Prepared, String> {
    #[cfg(debug_assertions)]
    let source = std::env::var_os(UPDATER_EXE_ENV)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| pick_updater(&job.install_dir, target, &|p| p.is_file()));
    #[cfg(not(debug_assertions))]
    let source = pick_updater(&job.install_dir, target, &|p| p.is_file());
    let no_updater = "this installation has no updater program"; // i18n-english
    let source = source.ok_or_else(|| no_updater.to_string())?;
    std::fs::create_dir_all(&paths.bin).map_err(|e| format!("{}: {e}", paths.bin.display()))?;
    let updater = paths.bin.join(updater_name(target));
    // A copy of an earlier update may still run or be locked: replace it by
    // name (a rename aside keeps a running Windows exe going).
    if updater.exists() {
        let aside = updater.with_extension("old");
        let _ = std::fs::remove_file(&aside);
        let _ = std::fs::rename(&updater, &aside);
    }
    let copying = "copying"; // i18n-english
    std::fs::copy(&source, &updater).map_err(|e| format!("{copying} {}: {e}", source.display()))?;
    std::fs::write(&paths.job, job.render())
        .map_err(|e| format!("{}: {e}", paths.job.display()))?;
    // A status of an earlier run must not be read as this one's.
    let _ = std::fs::remove_file(&paths.status);
    Ok(Prepared {
        updater,
        job: paths.job.clone(),
    })
}

/// Starts the updater in place of this process (Unix) or detached, then
/// exits (Windows). Returns only when it could not start.
pub(crate) fn launch(prepared: &Prepared) -> String {
    let mut cmd = std::process::Command::new(&prepared.updater);
    cmd.arg("run").arg("--job").arg(&prepared.job);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = cmd.exec();
        format!("{}: {err}", prepared.updater.display())
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;
        match cmd.creation_flags(CREATE_NEW_CONSOLE).spawn() {
            // No waiting parent: the updater replaces files this process
            // (and its console) would otherwise keep locked.
            Ok(_) => std::process::exit(0),
            Err(err) => format!("{}: {err}", prepared.updater.display()),
        }
    }
    #[cfg(not(any(unix, windows)))]
    {
        match cmd.spawn() {
            Ok(_) => std::process::exit(0),
            Err(err) => format!("{}: {err}", prepared.updater.display()),
        }
    }
}

/// The status line for a finished update, `None` while it is still running
/// or when it is too old to matter. `running` is this build's version: after
/// a successful update it is the new one.
pub(crate) fn status_line(status: &UpdateStatus, running: &str, now: u64) -> Option<String> {
    if !status.is_final() || now.saturating_sub(status.at) > STATUS_MAX_AGE_SECS {
        return None;
    }
    Some(match status.phase {
        Phase::Done => msg::msg_updated(running),
        Phase::Declined => msg::MSG_UPDATE_DECLINED.get().to_string(),
        _ => msg::msg_update_failed_previous(&status.message),
    })
}

/// Reads the updater's last status (once): a final one is consumed, so the
/// line shows on one start only. Blocking file work: not for the render
/// thread.
pub(crate) fn startup_status(status_path: &Path, running: &str, now: u64) -> Option<String> {
    let text = std::fs::read_to_string(status_path).ok()?;
    let status = UpdateStatus::parse(&text)?;
    if status.is_final() {
        let _ = std::fs::remove_file(status_path);
    }
    status_line(&status, running, now)
}

#[cfg(test)]
mod tests;
