//! `gitpoulpi-launch`: the program the Windows shortcuts and the file-manager
//! entries run.
//!
//! A stub: it runs `gitpoulpi(.exe) --gitpoulpi-as gitpoulpi-launch <args>`
//! from its own folder and returns that exit code. The real work (reuse an
//! open window, else start the command after `--`, `--folder DIR`, or the
//! Explorer address-bar start with no argument) is the launch persona of
//! `gitpoulpi` (`src/launch_persona.rs`).
//!
//! It is built for the Windows GUI subsystem so that a click on a shortcut
//! never flashes a console window, and starts `gitpoulpi` with
//! `CREATE_NO_WINDOW` for the same reason. It prints nothing. On other
//! systems it is the same stub with a console, which nothing runs. Std only.
#![cfg_attr(windows, windows_subsystem = "windows")]

use std::path::PathBuf;
use std::process::Command;

#[cfg(windows)]
const TARGET: &str = "gitpoulpi.exe";
#[cfg(not(windows))]
const TARGET: &str = "gitpoulpi";

/// `gitpoulpi` next to this program (symlinks resolved).
fn target() -> Option<PathBuf> {
    let own = std::env::current_exe()
        .and_then(std::fs::canonicalize)
        .ok()?;
    Some(own.parent()?.join(TARGET))
}

fn main() {
    let Some(target) = target() else {
        std::process::exit(127);
    };
    let mut cmd = Command::new(target);
    cmd.arg("--gitpoulpi-as")
        .arg("gitpoulpi-launch")
        .args(std::env::args_os().skip(1));
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let code = match cmd.status() {
        Ok(status) => status.code().unwrap_or(1),
        Err(_) => 127,
    };
    std::process::exit(code);
}
