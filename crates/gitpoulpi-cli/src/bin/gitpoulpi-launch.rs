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

use gitpoulpi_commands::launch::{self, LaunchArgs};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (new_window, parsed) = launch::split_launch_args(&args);
    let start = &gitpoulpi_tui::desktop_shortcuts::start_command;
    let code = match parsed {
        LaunchArgs::Command(command) => launch::run(&command, false, true, new_window),
        LaunchArgs::Folder(folder) => {
            launch::run_folder(Some(&folder), false, true, new_window, start)
        }
        LaunchArgs::Nothing => {
            launch::run_folder(working_folder().as_deref(), false, true, new_window, start)
        }
    };
    std::process::exit(code);
}

/// The folder Explorer's address bar started this in, when it means one.
fn working_folder() -> Option<std::path::PathBuf> {
    use gitpoulpi_config::desktop::terminals::address_bar_folder;
    let cwd = std::env::current_dir().ok()?;
    let exe = std::env::current_exe().ok();
    let home = gitpoulpi_config::home_dir();
    let windir = std::env::var_os("WINDIR").map(std::path::PathBuf::from);
    address_bar_folder(
        gitpoulpi_config::open_in::Os::current(),
        &cwd,
        exe.as_deref().and_then(std::path::Path::parent),
        home.as_deref(),
        windir.as_deref(),
        &|p| p.is_dir(),
    )
}
