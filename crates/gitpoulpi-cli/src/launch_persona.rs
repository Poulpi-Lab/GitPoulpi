//! What the `gitpoulpi-launch` program does, as a function: the
//! `gitpoulpi-launch` bin and the `gitpoulpi-launch` persona of `gitpoulpi`
//! both call [`run`]. See the module docs of `src/bin/gitpoulpi-launch.rs`.

use gitpoulpi_commands::launch::{self, LaunchArgs};

/// Runs on `args` (without the program name); the exit code.
pub fn run(args: &[String]) -> i32 {
    let (new_window, parsed) = launch::split_launch_args(args);
    let start = &gitpoulpi_tui::desktop_shortcuts::start_command;
    match parsed {
        LaunchArgs::Command(command) => launch::run(&command, false, true, new_window),
        LaunchArgs::Folder(folder) => {
            launch::run_folder(Some(&folder), false, true, new_window, start)
        }
        LaunchArgs::Nothing => {
            launch::run_folder(working_folder().as_deref(), false, true, new_window, start)
        }
    }
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
