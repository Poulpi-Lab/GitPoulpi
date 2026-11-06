//! "Open in terminal" for a command the panel shows (the install-channels
//! design: a `Show` procedure is never run by the app, only handed to the
//! user's own terminal, where `sudo` can ask for its password).
//!
//! The terminal is the one `ui.launch_terminal` would pick on this machine
//! (`gitpoulpi_config::desktop::terminals`), so Omarchy's `xdg-terminal-exec`
//! and Windows Terminal are used when they are there. The shell waits for
//! Enter at the end, so the output can be read before the window closes.
//! [`invocation`] is pure; [`open`] looks at `PATH` and starts the terminal
//! on a thread of its own (a spawn costs hundreds of milliseconds on
//! Windows), and the answer comes back as a status line.

use gitpoulpi_config::desktop::terminals;
use gitpoulpi_config::open_in::Os;
use gitpoulpi_i18n::msg::updates as msg;

/// The program and arguments that run `command` in a new terminal window.
pub(crate) fn invocation(
    os: Os,
    has: &dyn Fn(&str) -> bool,
    command: &str,
) -> (String, Vec<String>) {
    match os {
        Os::Macos => {
            let script = command.replace('\\', "\\\\").replace('"', "\\\"");
            // AppleScript, not text for the user.
            let run = format!("tell application \"Terminal\" to do script \"{script}\""); // i18n-english
            let show = "tell application \"Terminal\" to activate"; // i18n-english
            (
                "osascript".into(),
                vec!["-e".into(), run, "-e".into(), show.into()],
            )
        }
        Os::Windows => {
            let id = terminals::resolve(terminals::AUTO, os, has);
            let inner = ["cmd".to_string(), "/K".into(), command.to_string()];
            match id.and_then(|id| prefix(os, id)) {
                Some(mut argv) => {
                    let program = argv.remove(0);
                    argv.extend(inner);
                    (program, argv)
                }
                None => {
                    let mut args: Vec<String> =
                        ["/C", "start", ""].iter().map(|s| s.to_string()).collect();
                    args.extend(inner);
                    ("cmd".into(), args)
                }
            }
        }
        Os::Linux => {
            // Shell text the user sees in the new terminal.
            let hold = format!("{command}; echo; printf '%s' 'Press Enter to close... '; read _"); // i18n-english
            let inner = ["sh".to_string(), "-c".into(), hold];
            let id = terminals::resolve(terminals::AUTO, os, has);
            match id.and_then(|id| prefix(os, id)) {
                Some(mut argv) => {
                    let program = argv.remove(0);
                    argv.extend(inner);
                    (program, argv)
                }
                None => {
                    let mut args = vec!["-e".to_string()];
                    args.extend(inner);
                    ("x-terminal-emulator".into(), args)
                }
            }
        }
    }
}

/// The terminal's own command words, up to where the program to run goes:
/// the launcher table with its program and flag taken off.
fn prefix(os: Os, id: &str) -> Option<Vec<String>> {
    let mut argv = terminals::argv(os, id, "")?;
    // `argv` ends with the program and the launcher flag it appends.
    argv.truncate(argv.len().checked_sub(2)?);
    (!argv.is_empty()).then_some(argv)
}

/// Starts a terminal running `command`. Returns at once; the thread reports
/// through the updates status line.
pub(crate) fn open(command: String) {
    let spawned = std::thread::Builder::new()
        .name("update-terminal".into())
        .spawn(move || {
            let lookup = crate::repo_menu::path_lookup();
            let (program, args) = invocation(Os::current(), &lookup, &command);
            let line = match std::process::Command::new(&program)
                .args(&args)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
            {
                Ok(_) => msg::MSG_TERMINAL_OPENED.get().to_string(),
                Err(e) => msg::msg_terminal_failed(format!("{program}: {e}")),
            };
            super::say(line);
        });
    if spawned.is_err() {
        super::say(msg::msg_terminal_failed("could not start a thread")); // i18n-english
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linux_uses_the_terminal_auto_would_pick_and_waits_for_enter() {
        let has = |p: &str| p == "xdg-terminal-exec";
        let (program, args) = invocation(Os::Linux, &has, "sudo apt update");
        assert_eq!(program, "xdg-terminal-exec");
        assert_eq!(&args[..2], ["--app-id=TUI.tile", "-e"]);
        assert_eq!(args[2], "sh");
        assert_eq!(args[3], "-c");
        assert!(args[4].starts_with("sudo apt update; "), "{}", args[4]);
        assert!(args[4].contains("read _"));
    }

    #[test]
    fn linux_without_a_known_terminal_falls_back_to_the_debian_alternative() {
        let (program, args) = invocation(Os::Linux, &|_| false, "ls");
        assert_eq!(program, "x-terminal-emulator");
        assert_eq!(args[0], "-e");
    }

    #[test]
    fn windows_prefers_windows_terminal_else_a_new_console() {
        let (program, args) = invocation(Os::Windows, &|p| p == "wt", "choco upgrade gitpoulpi");
        assert_eq!(program, "wt.exe");
        assert_eq!(args, ["cmd", "/K", "choco upgrade gitpoulpi"]);
        let (program, args) = invocation(Os::Windows, &|_| false, "winget list");
        assert_eq!(program, "cmd");
        assert_eq!(&args[..3], ["/C", "start", ""]);
        assert_eq!(&args[3..], ["cmd", "/K", "winget list"]);
    }

    #[test]
    fn macos_asks_terminal_app_to_run_the_command_with_quotes_escaped() {
        let (program, args) = invocation(Os::Macos, &|_| false, "echo \"hi\"");
        assert_eq!(program, "osascript");
        assert!(
            args[1].contains("do script \"echo \\\"hi\\\"\""),
            "{}",
            args[1]
        );
    }
}
