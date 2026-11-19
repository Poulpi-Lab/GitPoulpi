//! The coordinator (`run --job`) and the elevated worker (`elevated`).
//!
//! The coordinator is never elevated. It validates the job, takes the locks,
//! waits for the TUI to end, stops every session, does the procedure (itself,
//! or through the elevated worker, or by running the package manager),
//! releases the locks, writes the final status and starts GitPoulpi again as
//! the same user. The elevated worker is the **installed** copy of this
//! program, run through the elevation prompt for the one step that needs
//! rights.

use std::path::{Path, PathBuf};
use std::time::Duration;

use gitpoulpi_config::release;
use gitpoulpi_update::job::{Phase, UpdateJob};
use gitpoulpi_update::managed_names;
use gitpoulpi_update::plan::{Elevation, Procedure};

use crate::env::{same_dir, Env};
use crate::locks;
use crate::ops::{LaunchPlan, RunError};
use crate::outcome::{Outcome, Status};
use crate::{stop, swap};

const REQUESTER_WAIT: Duration = Duration::from_secs(10);

/// How GitPoulpi starts again. Unix replaces the updater in the same
/// terminal; Windows has no such thing, so the launcher (`gitpoulpi-launch`,
/// which opens the user's terminal) starts it detached.
pub fn relaunch_plan(env: &Env, job: &UpdateJob) -> LaunchPlan {
    let r = &job.relaunch;
    let exe = relaunch_exe(env, job);
    if !release::is_windows(&env.host_target) {
        return LaunchPlan {
            program: exe,
            args: r.args.clone(),
            cwd: r.cwd.clone(),
            replace_process: true,
        };
    }
    let launcher = env.exe_in(&job.install_dir, "gitpoulpi-launch");
    if env.fs.exists(&launcher) {
        let mut args = vec![
            "--new-window".to_string(),
            "--".to_string(),
            exe.to_string_lossy().into_owned(),
        ];
        args.extend(r.args.iter().cloned());
        LaunchPlan {
            program: launcher,
            args,
            cwd: r.cwd.clone(),
            replace_process: false,
        }
    } else {
        LaunchPlan {
            program: exe,
            args: r.args.clone(),
            cwd: r.cwd.clone(),
            replace_process: false,
        }
    }
}

/// The program that starts again: the recorded one, unless it is gone or a
/// name the new release retired (a user who skipped versions recorded a
/// program that no longer exists), then `gitpoulpi` in the install folder.
/// The arguments carry over: `gitpoulpi` with no subcommand is the TUI.
fn relaunch_exe(env: &Env, job: &UpdateJob) -> PathBuf {
    let recorded = &job.relaunch.exe;
    let retired = recorded
        .file_stem()
        .is_some_and(|s| release::is_retired_in(env.retired, &s.to_string_lossy()));
    if retired || !env.fs.exists(recorded) {
        env.exe_in(&job.install_dir, "gitpoulpi")
    } else {
        recorded.clone()
    }
}

fn read_job(env: &Env, path: &Path) -> Result<UpdateJob, Outcome> {
    let text = env
        .fs
        .read_to_string(path)
        .map_err(|e| Outcome::BadJob(format!("could not read {}: {e}", path.display())))?;
    UpdateJob::parse(&text).map_err(|e| Outcome::BadJob(e.to_string()))
}

fn updates_dir(job: &UpdateJob) -> PathBuf {
    job.status_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default()
}

fn wait_for_requester(env: &Env, job: &UpdateJob, status: &Status) {
    if !env.proc.is_alive(job.requester_pid) {
        return;
    }
    status.set(Phase::WaitingForApp, "Waiting for GitPoulpi to close");
    stop::wait_until(env, REQUESTER_WAIT, || {
        !env.proc.is_alive(job.requester_pid)
    });
}

/// Runs the elevated worker and turns its exit code into an outcome. The
/// worker leaves its own reason in the status file.
fn through_worker(env: &Env, job: &UpdateJob, job_path: &Path, status: &Status) -> Outcome {
    let worker = env.exe_in(&job.install_dir, "gitpoulpi-updater");
    if !env.fs.exists(&worker) {
        return Outcome::Failed(format!(
            "{} is missing, so the administrator step cannot run",
            worker.display()
        ));
    }
    status.set(Phase::Working, "Waiting for the administrator prompt");
    let argv = vec![
        worker.to_string_lossy().into_owned(),
        "elevated".into(),
        "--job".into(),
        job_path.to_string_lossy().into_owned(),
        "--parent".into(),
        env.own_pid.to_string(),
    ];
    match env.proc.run_elevated(&argv) {
        Err(RunError::Declined) => Outcome::Declined,
        Err(RunError::Failed(e)) => {
            Outcome::Failed(format!("could not start the administrator step: {e}"))
        }
        Ok(code) => {
            let message = status
                .read()
                .filter(|s| s.phase == Phase::Failed || s.phase == Phase::Done)
                .map(|s| s.message);
            match code {
                0 => Outcome::Done(
                    message.unwrap_or_else(|| format!("Updated to {}", job.to_version)),
                ),
                2 => Outcome::Declined,
                3 => Outcome::NothingToDo(message.unwrap_or_default()),
                4 => Outcome::Refused(message.unwrap_or_default()),
                5 => Outcome::WouldNotStop(message.unwrap_or_default()),
                6 => Outcome::BadJob(message.unwrap_or_default()),
                _ => Outcome::Failed(message.unwrap_or_else(|| {
                    "the administrator step failed; the previous version is still installed".into()
                })),
            }
        }
    }
}

/// `run --job <path>`: the whole update. Returns the outcome after the
/// relaunch attempt (on Unix a successful relaunch never returns).
pub fn coordinator(env: &Env, job_path: &Path) -> Outcome {
    let job = match read_job(env, job_path) {
        Ok(j) => j,
        Err(o) => {
            eprintln!("gitpoulpi-updater: {}", o.message());
            return o;
        }
    };
    let status = Status {
        fs: env.fs,
        clock: env.clock,
        path: &job.status_path,
        echo: env.echo,
    };
    status.set(
        Phase::Started,
        &format!("Updating GitPoulpi to {}", job.to_version),
    );

    let (outcome, locked) = run_locked(env, &job, job_path, &status);
    if let Some(held) = locked {
        held.release(env);
    }
    status.finish(&outcome);
    let plan = relaunch_plan(env, &job);
    if let Err(e) = env.proc.relaunch(&plan) {
        eprintln!("gitpoulpi-updater: could not start GitPoulpi again: {e}");
    }
    outcome
}

fn run_locked(
    env: &Env,
    job: &UpdateJob,
    job_path: &Path,
    status: &Status,
) -> (Outcome, Option<locks::Held>) {
    let checked = match env.check(job) {
        Ok(c) => c,
        Err(o) => return (o, None),
    };
    let held = match locks::acquire(env, &updates_dir(job), &job.install_dir) {
        Ok(h) => h,
        Err(why) => return (Outcome::NothingToDo(why), None),
    };

    wait_for_requester(env, job, status);
    status.set(Phase::StoppingProcesses, "Closing GitPoulpi");
    stop::stop_daemons(env, job);
    let files: Vec<PathBuf> = managed_names(&env.host_target)
        .iter()
        .map(|n| job.install_dir.join(n))
        .collect();
    if let Err(why) = stop::stop_processes(env, job, &files, &[env.own_pid], false) {
        return (Outcome::WouldNotStop(why), Some(held));
    }

    let outcome = match &checked.procedure {
        Procedure::Swap {
            elevation: Elevation::None,
            registry,
        } => swap::swap(
            env,
            job,
            checked.resolved.channel,
            *registry,
            &Env::extra_hosts(&checked.effective),
            status,
        ),
        Procedure::Swap {
            elevation: Elevation::Uac,
            ..
        } => through_worker(env, job, job_path, status),
        Procedure::Run { argv, elevation } => {
            swap::manager(env, job, argv, *elevation == Elevation::Uac, status)
        }
        _ => Outcome::BadJob("nothing this updater can run".into()),
    };
    (outcome, Some(held))
}

/// `elevated --job <path> --parent <pid>`: the one step that needs rights.
/// Only ever runs as the installed copy; reads the machine policy only.
pub fn elevated(env: &Env, job_path: &Path, parent: u32) -> Outcome {
    let job = match read_job(env, job_path) {
        Ok(j) => j,
        Err(o) => return o,
    };
    let status = Status {
        fs: env.fs,
        clock: env.clock,
        path: &job.status_path,
        echo: env.echo,
    };
    let outcome = elevated_step(env, &job, parent, &status);
    // The coordinator reads the reason from here.
    status.finish(&outcome);
    outcome
}

fn elevated_step(env: &Env, job: &UpdateJob, parent: u32, status: &Status) -> Outcome {
    if !env.proc.is_alive(parent) {
        return Outcome::BadJob("the update that started this step is gone".into());
    }
    let installed_copy = env
        .own_exe
        .parent()
        .is_some_and(|d| same_dir(d, &job.install_dir));
    if !installed_copy {
        return Outcome::BadJob("the administrator step only runs from the installed copy".into());
    }
    let checked = match env.check(job) {
        Ok(c) => c,
        Err(o) => return o,
    };
    let Procedure::Swap {
        elevation: Elevation::Uac,
        registry,
    } = &checked.procedure
    else {
        return Outcome::BadJob("this install's update does not need administrator rights".into());
    };

    status.set(
        Phase::StoppingProcesses,
        "Closing other GitPoulpi processes",
    );
    let files: Vec<PathBuf> = managed_names(&env.host_target)
        .iter()
        .map(|n| job.install_dir.join(n))
        .collect();
    if let Err(why) = stop::stop_processes(env, job, &files, &[env.own_pid, parent], true) {
        return Outcome::WouldNotStop(why);
    }
    let lock = locks::install_lock(env, &job.install_dir);
    let outcome = swap::swap(
        env,
        job,
        checked.resolved.channel,
        *registry,
        &Env::extra_hosts(&checked.effective),
        status,
    );
    if let Some(p) = lock {
        let _ = env.fs.remove_file(&p);
    }
    outcome
}
