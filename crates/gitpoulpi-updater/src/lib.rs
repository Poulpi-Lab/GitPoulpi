//! `gitpoulpi-updater`: the program that replaces GitPoulpi while GitPoulpi is
//! not running (the install-channels design, 1.3).
//!
//! ```text
//! gitpoulpi-updater run --job <path>                       coordinator, never elevated
//! gitpoulpi-updater elevated --job <path> --parent <pid>   the privileged step only
//! gitpoulpi-updater --version
//! exit: 0 done | 1 failed, rolled back | 2 elevation declined | 3 nothing to do
//!       4 refused by policy | 5 processes would not stop | 6 bad job
//! ```
//!
//! The TUI writes the job and exits; this program stops every session,
//! swaps the files (or runs the package manager), checks `gitpoulpi
//! --version`, rolls back on failure and starts GitPoulpi again as the same
//! user. Everything it does to the machine goes through the traits in
//! [`ops`], so [`flow`] is tested against fakes.

pub mod env;
pub mod flow;
pub mod locks;
pub mod ops;
pub mod outcome;
pub mod real;
#[cfg(windows)]
mod real_windows;
pub mod stop;
pub mod swap;

#[cfg(test)]
mod fakes;
#[cfg(test)]
mod tests;

use std::path::PathBuf;

use gitpoulpi_update::channel::RealEnv;
use gitpoulpi_update::fsops::RealFs;
use gitpoulpi_update::job::exit;
use gitpoulpi_update::policy_source::SystemPolicySource;

/// What the command line asked for.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Version,
    Run { job: PathBuf },
    Elevated { job: PathBuf, parent: u32 },
}

/// Parses `args` (without the program name).
pub fn parse_args(args: &[String]) -> Result<Command, String> {
    let value = |flag: &str| {
        args.iter()
            .position(|a| a == flag)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    match args.first().map(String::as_str) {
        Some("--version" | "-V") => Ok(Command::Version),
        Some("run") => Ok(Command::Run {
            job: value("--job").ok_or("run needs --job <path>")?.into(),
        }),
        Some("elevated") => Ok(Command::Elevated {
            job: value("--job").ok_or("elevated needs --job <path>")?.into(),
            parent: value("--parent")
                .and_then(|p| p.parse().ok())
                .ok_or("elevated needs --parent <pid>")?,
        }),
        _ => Err("usage: gitpoulpi-updater run --job <path> | --version".into()),
    }
}

/// The process entry point: returns the exit code.
pub fn main_with(args: &[String]) -> i32 {
    let command = match parse_args(args) {
        Ok(c) => c,
        Err(why) => {
            eprintln!("gitpoulpi-updater: {why}");
            return exit::BAD_JOB;
        }
    };
    if command == Command::Version {
        println!("gitpoulpi-updater {}", gitpoulpi_ipc::VERSION_STRING);
        return exit::DONE;
    }
    let (fs, proc, daemons, registry, clock, transfer, chan_env, policy) = (
        RealFs,
        real::RealProc,
        real::RealDaemons,
        real::RealRegistry,
        real::RealClock,
        real::RealTransfer,
        RealEnv,
        SystemPolicySource,
    );
    let env = env::Env {
        fs: &fs,
        proc: &proc,
        daemons: &daemons,
        registry: &registry,
        clock: &clock,
        transfer: &transfer,
        channel_env: &chan_env,
        policy: &policy,
        host_target: gitpoulpi_config::release::host_target(),
        keys: gitpoulpi_update::signing::release_keys(),
        own_exe: std::env::current_exe().unwrap_or_default(),
        own_pid: std::process::id(),
        echo: true,
        retired: &gitpoulpi_config::release::RETIRED,
    };
    let outcome = match command {
        Command::Run { job } => flow::coordinator(&env, &job),
        Command::Elevated { job, parent } => flow::elevated(&env, &job, parent),
        Command::Version => unreachable!("handled above"),
    };
    if outcome.code() != exit::DONE {
        eprintln!("gitpoulpi-updater: {}", outcome.message());
    }
    outcome.code()
}
