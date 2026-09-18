//! `gitpoulpi-launch`: the program the Windows shortcuts run.
//!
//! The same as `gitpoulpi launch -- <command>`, built for the Windows GUI
//! subsystem so that a click on a shortcut never flashes a console window
//! (a console program started from Explorer opens one, even when it only
//! asks a daemon a question and exits). It prints nothing: a window that is
//! already open is shown, else the command after `--` is started
//! (`gitpoulpi_commands::launch`). On other systems it is the same program
//! with a console, which nothing runs.
#![cfg_attr(windows, windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (new_window, rest) = gitpoulpi_commands::launch::split_new_window(&args);
    std::process::exit(gitpoulpi_commands::launch::run(
        rest, false, true, new_window,
    ));
}
