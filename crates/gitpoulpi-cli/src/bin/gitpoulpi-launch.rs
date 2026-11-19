//! `gitpoulpi-launch`: the program the Windows shortcuts and the file-manager
//! entries run.
//!
//! The same as `gitpoulpi launch -- <command>`, built for the Windows GUI
//! subsystem so that a click on a shortcut never flashes a console window
//! (a console program started from Explorer opens one, even when it only
//! asks a daemon a question and exits). It prints nothing: a window that is
//! already open is shown, else the command after `--` is started
//! (`gitpoulpi_commands::launch`).
//!
//! `--folder DIR` (the Explorer right-click entry) opens that folder, and no
//! argument at all (App Paths `gitpoulpi.exe`: `gitpoulpi` typed in the
//! address bar) opens the working folder Explorer started it in, or just
//! GitPoulpi when that folder means nothing (the install folder, home, the
//! Windows folder). On other systems it is the same program with a console,
//! which nothing runs.
#![cfg_attr(windows, windows_subsystem = "windows")]

#[path = "../launch_persona.rs"]
mod launch_persona;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(launch_persona::run(&args));
}
