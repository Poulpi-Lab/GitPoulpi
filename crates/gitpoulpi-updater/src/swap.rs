//! The procedures themselves: swapping GitPoulpi's own files (and walking
//! back when the new ones do not run), and running the package manager.
//!
//! Traps:
//! - The archive a user-level process staged is never trusted by the
//!   elevated worker: it downloads the `.sha256` itself and compares, before
//!   a single file is touched.
//! - A swap is verified by running `<install_dir>/gitpoulpi --version`; a
//!   build that does not report the target version is rolled back.
//! - The Inno `DisplayVersion` follows every swap and every rollback, or
//!   winget and "Apps & features" keep showing the old version.

use std::path::Path;

use gitpoulpi_config::release::{Marker, MARKER_FILE};
use gitpoulpi_hosting::updates::download_host_allowed_with;
use gitpoulpi_update::channel::{Channel, Hive};
use gitpoulpi_update::fetch;
use gitpoulpi_update::install::{self, Replace};
use gitpoulpi_update::job::{Action, Phase, Staged, UpdateJob};
use gitpoulpi_update::{managed_names, UpdateError};

use crate::env::Env;
use crate::ops::RunError;
use crate::outcome::{Outcome, Status};

/// Whether `output` (a `--version` line) names `version`.
pub fn reports(output: &str, version: &str) -> bool {
    let want = version.trim().trim_start_matches('v');
    output
        .split(|c: char| c.is_whitespace() || c == '(' || c == ')')
        .any(|t| t.trim_start_matches('v') == want)
}

fn installed_reports(env: &Env, dir: &Path, version: &str) -> Result<(), String> {
    let exe = env.exe_in(dir, "gitpoulpi");
    match env.proc.version_of(&exe) {
        Some(out) if reports(&out, version) => Ok(()),
        Some(out) => Err(format!(
            "{} reports {:?}, not {version}",
            exe.display(),
            out.trim()
        )),
        None => Err(format!("{} could not be run", exe.display())),
    }
}

/// Re-verifies the staged archive and stages it into the install folder.
fn stage_archive(
    env: &Env,
    job: &UpdateJob,
    archive: &Path,
    sha256_url: &str,
    extra_hosts: &[String],
) -> Result<(), String> {
    let name = archive
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    if !download_host_allowed_with(sha256_url, extra_hosts) {
        return Err(format!("{sha256_url} is not an allowed download host"));
    }
    let bytes = env
        .transfer
        .read_file(archive)
        .map_err(|e| format!("could not read {}: {e}", archive.display()))?;
    let text = env
        .transfer
        .get(sha256_url)
        .map_err(|e| format!("could not download the checksum: {e}"))?;
    let expected =
        fetch::parse_checksum(&String::from_utf8_lossy(&text), name).map_err(|e| e.to_string())?;
    let actual = fetch::sha256_hex(&bytes);
    if actual != expected {
        return Err(UpdateError::ChecksumMismatch { expected, actual }.to_string());
    }
    install::stage_from_archive_into(
        env.fs,
        &job.install_dir,
        &bytes,
        &job.to_version,
        &env.host_target,
    )
    .map(|_| ())
    .map_err(|e| e.to_string())
}

fn set_display_version(env: &Env, hive: Option<Hive>, version: &str) -> Option<String> {
    let hive = hive?;
    env.registry
        .set_display_version(hive, version)
        .err()
        .map(|e| format!("the installed-programs entry was not updated: {e}"))
}

fn installed_version(env: &Env, dir: &Path) -> Option<String> {
    Marker::parse(&env.fs.read_to_string(&dir.join(MARKER_FILE)).ok()?).map(|m| m.version)
}

/// Swaps (or reverts) the install's own files, verifies the result and rolls
/// back when it does not run. `channel` is recorded in a Legacy install's
/// marker; `registry` is the hive whose `DisplayVersion` follows.
pub fn swap(
    env: &Env,
    job: &UpdateJob,
    channel: Channel,
    registry: Option<Hive>,
    extra_hosts: &[String],
    status: &Status,
) -> Outcome {
    let dir = &job.install_dir;
    let names = managed_names(&env.host_target);
    let replace = Replace::for_target(&env.host_target);
    let own = env.own_name_in(dir);
    status.set(Phase::Working, "Replacing GitPoulpi's files");

    let forward = job.action == Action::Apply;
    let done = if forward {
        if let Staged::Archive {
            archive,
            sha256_url,
        } = &job.staged
        {
            if let Err(e) = stage_archive(env, job, archive, sha256_url, extra_hosts) {
                return Outcome::Failed(format!("{e}; nothing was changed"));
            }
        }
        let resolved_channel = match channel {
            Channel::Install(c) => Some(c),
            _ => None,
        };
        install::apply_with_channel(env.fs, dir, &names, replace, &own, resolved_channel)
    } else {
        install::revert(env.fs, dir, &names, replace, &own)
    };
    match done {
        Ok(()) => {}
        Err(UpdateError::NothingStaged) => {
            return Outcome::NothingToDo("no downloaded update is waiting".into())
        }
        Err(UpdateError::NothingToRevert) => {
            return Outcome::NothingToDo("there is no previous version to go back to".into())
        }
        Err(e) => return Outcome::Failed(format!("{e}; the previous version is still installed")),
    }

    let new_version = installed_version(env, dir).unwrap_or_else(|| job.to_version.clone());
    let registry_note = set_display_version(env, registry, &new_version);

    status.set(Phase::Verifying, "Checking the new version");
    if let Err(why) = installed_reports(env, dir, &job.to_version) {
        status.set(
            Phase::RollingBack,
            "The new version does not run, going back",
        );
        // The swap is symmetric: reverting brings the old files back.
        let back = install::revert(env.fs, dir, &names, replace, &own);
        let _ = set_display_version(env, registry, &job.from_version);
        return match back {
            Ok(()) => Outcome::Failed(format!("{why}; the previous version was put back")),
            Err(e) => Outcome::Failed(format!(
                "{why}; the previous version could not be put back: {e}"
            )),
        };
    }
    let mut message = format!("Updated to {}", job.to_version);
    if !forward {
        message = format!("Went back to {}", job.to_version);
    }
    if let Some(note) = registry_note {
        message.push_str(&format!(" ({note})"));
    }
    Outcome::Done(message)
}

/// Runs the package manager's command (the plan's, never the job's) and
/// checks the version it left behind.
pub fn manager(
    env: &Env,
    job: &UpdateJob,
    argv: &[String],
    elevated: bool,
    status: &Status,
) -> Outcome {
    status.set(Phase::Working, &format!("Running {}", argv.join(" ")));
    let ran = if elevated {
        env.proc.run_elevated(argv)
    } else {
        env.proc.run(argv)
    };
    match ran {
        Err(RunError::Declined) => return Outcome::Declined,
        Err(RunError::Failed(e)) => {
            return Outcome::Failed(format!("could not run {}: {e}", argv[0]))
        }
        Ok(0) => {}
        Ok(code) => {
            return Outcome::Failed(format!(
                "{} exited with code {code}; run its own downgrade if the install is broken",
                argv[0]
            ))
        }
    }
    status.set(Phase::Verifying, "Checking the new version");
    match installed_reports(env, &job.install_dir, &job.to_version) {
        Ok(()) => Outcome::Done(format!("Updated to {}", job.to_version)),
        Err(why) => Outcome::Failed(format!(
            "{why}; the package manager's own downgrade puts the previous version back"
        )),
    }
}
