//! Windows double-click: the exe started from Explorer gets a fresh console
//! of its own (the default one, which draws pictures badly). When
//! `ui.launch_terminal` names another terminal, GitPoulpi starts itself in it
//! and exits, so the first window closes.
//!
//! The decision is pure ([`should_relaunch`], tested everywhere); only
//! counting the console's processes and spawning are `cfg(windows)`. A loop is
//! impossible: the child carries `GITPOULPI_RELAUNCHED=1`, and a failed spawn
//! just stays in the console.

/// Test hook: pretend the console is shared with a shell (see
/// `imp::console_processes`). Set by the PTY driver on every child.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) const SHARED_CONSOLE_ENV: &str = "GITPOULPI_TEST_SHARED_CONSOLE";

/// Set in the relaunched process.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) const RELAUNCHED_ENV: &str = "GITPOULPI_RELAUNCHED";

/// Class of the window a classic console host (`conhost.exe`) draws: what a
/// double-click from Explorer gets. Any terminal that starts us directly
/// (`wezterm-gui start -- gitpoulpi.exe`, `wt.exe gitpoulpi.exe`, Rio,
/// Tabby, Alacritty `-e`) runs us on a pseudo-console instead, whose window
/// class is `PseudoConsoleWindow`, and we are then also the console's only
/// process: counting processes alone cannot tell the two apart.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) const CONSOLE_WINDOW_CLASS: &str = "ConsoleWindowClass";

/// Whether the console window's class (`None`: no window) says that a plain
/// console host owns it, with no terminal in between.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) fn owns_console_window(class: Option<&str>) -> bool {
    class == Some(CONSOLE_WINDOW_CLASS)
}

/// Whether to relaunch: switched on, not already a relaunch, alone in the
/// console (`GetConsoleProcessList` says 1: a double-click; started from a
/// shell it is at least 2), the console is a plain console host's own window
/// (not a terminal's pseudo-console: a terminal that runs us directly is
/// where the user wants to be), and a real terminal chosen (not the plain
/// console).
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) fn should_relaunch(
    enabled: bool,
    already_relaunched: bool,
    console_processes: Option<u32>,
    console_window_class: Option<&str>,
    terminal: Option<&str>,
) -> bool {
    enabled
        && !already_relaunched
        && console_processes == Some(1)
        && owns_console_window(console_window_class)
        && terminal.is_some_and(|t| t != "console")
}

/// Whether we already run inside the terminal `setting` names, decided from
/// environment markers alone (no `PATH` listing, which costs ~100 ms on
/// Windows). A named terminal is looked up directly; `auto` (and an empty or
/// unknown value) prefers Windows Terminal, so its marker answers for it.
/// `false` means "not sure", and the caller resolves the terminal properly.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) fn surely_inside(setting: &str, var: &dyn Fn(&str) -> bool) -> bool {
    use gitpoulpi_config::desktop::terminals;
    if setting.trim() == terminals::DEFAULT {
        return false;
    }
    let id = terminals::find(gitpoulpi_config::open_in::Os::Windows, setting.trim())
        .map_or("windows-terminal", |t| t.id);
    terminals::inside(id, var)
}

/// The command line that starts `exe` again inside terminal `id`: the
/// terminal's own words, the exe, `--launcher`, then the original arguments
/// (the flag is not repeated when the original already had it: the CLI's
/// parser rejects a flag given twice). `None` for an unknown terminal and the
/// plain console.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) fn relaunch_argv(
    os: gitpoulpi_config::open_in::Os,
    id: &str,
    exe: &str,
    original: &[String],
) -> Option<Vec<String>> {
    let mut argv = gitpoulpi_config::desktop::terminals::argv(os, id, exe)?;
    let flag = gitpoulpi_config::desktop::terminals::LAUNCHER_FLAG;
    argv.extend(original.iter().filter(|a| a.as_str() != flag).cloned());
    Some(argv)
}

/// Whether this start counts as a launcher start (`general.startup_tab_launcher`
/// applies) on Windows: the flag was given (a shortcut we wrote, current
/// or relaunched), or the process is alone in its console. A shell shares its
/// console with what it starts (`GetConsoleProcessList` is at least 2), so
/// alone means nothing typed it: Explorer double-click, a taskbar pin of the
/// exe itself, or a terminal profile that runs us. Deliberately not the
/// parent-is-`explorer.exe` test: the parent of a pinned `wt.exe` shortcut is
/// `wt.exe`, and walking the process tree needs a snapshot API for no gain.
/// Read before [`try_relaunch`], whose child is told by the flag.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) fn infer_launcher(flag: bool, relaunched: bool, console_processes: Option<u32>) -> bool {
    flag || relaunched || console_processes == Some(1)
}

#[cfg(windows)]
mod imp {
    use super::*;
    use crate::launcher::Env;
    use gitpoulpi_config::desktop::terminals;
    use gitpoulpi_config::open_in::Os;

    extern "system" {
        fn GetConsoleProcessList(process_list: *mut u32, process_count: u32) -> u32;
        fn GetConsoleWindow() -> *mut core::ffi::c_void;
    }
    #[link(name = "user32")]
    extern "system" {
        fn GetClassNameW(hwnd: *mut core::ffi::c_void, buf: *mut u16, max: i32) -> i32;
    }

    /// Class name of the console's window; `None` when it has none.
    pub(crate) fn console_window_class() -> Option<String> {
        // SAFETY: plain queries; the buffer is valid for the 64 units passed.
        unsafe {
            let hwnd = GetConsoleWindow();
            if hwnd.is_null() {
                return None;
            }
            let mut buf = [0u16; 64];
            let n = GetClassNameW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
            (n > 0).then(|| String::from_utf16_lossy(&buf[..n as usize]))
        }
    }

    pub(crate) fn console_processes() -> Option<u32> {
        // A ConPTY holds the child alone, so every end-to-end test looks like
        // a double-click. The PTY driver sets this to say "a shell shares
        // this console" (what a user typing the command has).
        if std::env::var_os(SHARED_CONSOLE_ENV).is_some() {
            return Some(2);
        }
        let mut ids = [0u32; 4];
        // SAFETY: the buffer is valid for the four ids passed.
        let n = unsafe { GetConsoleProcessList(ids.as_mut_ptr(), ids.len() as u32) };
        (n > 0).then_some(n)
    }

    /// Starts GitPoulpi in the configured terminal; `true`: the caller must exit.
    pub(crate) fn try_relaunch() -> bool {
        let ui = gitpoulpi_config::ConfigPaths::discover()
            .and_then(|p| gitpoulpi_config::Config::load(&p.config_file()).ok())
            .unwrap_or_default()
            .ui;
        let already = std::env::var_os(RELAUNCHED_ENV).is_some();
        // Cheap checks first: the terminal lookup lists `PATH`.
        let class = console_window_class();
        if !should_relaunch(
            ui.relaunch_in_terminal,
            already,
            console_processes(),
            class.as_deref(),
            Some("x"),
        ) {
            return false;
        }
        // Already in the chosen terminal (a shortcut started it there): the
        // answer needs no `PATH` listing, so give it before `Env::real()`.
        if surely_inside(&ui.launch_terminal, &|v| std::env::var_os(v).is_some()) {
            return false;
        }
        let env = Env::real();
        let terminal = env.resolve(&ui.launch_terminal);
        if !should_relaunch(true, false, Some(1), class.as_deref(), terminal) {
            return false;
        }
        let Some(id) = terminal else { return false };
        // Started by a shortcut inside the chosen terminal (a ConPTY counts
        // one process too): already there, relaunching would open a second
        // window and close this one.
        if terminals::inside(id, &|v| std::env::var_os(v).is_some()) {
            return false;
        }
        let exe = env.exe.to_string_lossy().to_string();
        let original: Vec<String> = std::env::args().skip(1).collect();
        let Some(argv) = relaunch_argv(Os::Windows, id, &exe, &original) else {
            return false;
        };
        let program = terminals::find(Os::Windows, id)
            .and_then(|t| (!t.probe.is_empty()).then(|| (env.which)(t.probe)))
            .flatten()
            .unwrap_or_else(|| std::path::PathBuf::from(&argv[0]));
        std::process::Command::new(program)
            .args(&argv[1..])
            .env(RELAUNCHED_ENV, "1")
            .spawn()
            .is_ok()
    }
}

#[cfg(windows)]
pub(crate) use imp::{console_processes, try_relaunch};

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_config::open_in::Os;

    fn words(v: Vec<String>) -> Vec<String> {
        v
    }

    fn args(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn the_terminal_we_are_in_is_known_from_the_environment_alone() {
        let set = |name: &'static str| move |v: &str| v == name;
        assert!(surely_inside("auto", &set("WT_SESSION")));
        assert!(surely_inside("", &set("WT_SESSION")));
        assert!(surely_inside("windows-terminal", &set("WT_SESSION")));
        assert!(surely_inside("wezterm", &set("WEZTERM_PANE")));
        // Another terminal chosen: the marker of the one we are in is no answer.
        assert!(!surely_inside("wezterm", &set("WT_SESSION")));
        assert!(!surely_inside("default", &set("WT_SESSION")));
        assert!(!surely_inside("auto", &|_| false));
    }

    #[test]
    fn the_relaunch_command_line_carries_the_flag_and_the_original_arguments() {
        let exe = r"C:\gp\gitpoulpi.exe";
        // wt: `wt.exe <exe> --launcher <original...>`
        assert_eq!(
            words(
                relaunch_argv(
                    Os::Windows,
                    "windows-terminal",
                    exe,
                    &args(&["--repo", "D:\\r"])
                )
                .unwrap()
            ),
            args(&["wt.exe", exe, "--launcher", "--repo", "D:\\r"])
        );
        // wezterm: after `start --`.
        assert_eq!(
            relaunch_argv(Os::Windows, "wezterm", exe, &args(&["-s", "x"])).unwrap(),
            args(&[
                "wezterm-gui.exe",
                "start",
                "--",
                exe,
                "--launcher",
                "-s",
                "x"
            ])
        );
        assert_eq!(
            relaunch_argv(Os::Windows, "alacritty", exe, &[]).unwrap(),
            args(&["alacritty.exe", "-e", exe, "--launcher"])
        );
        // An original `--launcher` is not doubled.
        assert_eq!(
            relaunch_argv(
                Os::Windows,
                "windows-terminal",
                exe,
                &args(&["--launcher", "--cwd-file", "f"])
            )
            .unwrap(),
            args(&["wt.exe", exe, "--launcher", "--cwd-file", "f"])
        );
        // The plain console and unknown terminals do not relaunch.
        assert!(relaunch_argv(Os::Windows, "console", exe, &[]).is_none());
        assert!(relaunch_argv(Os::Windows, "nope", exe, &[]).is_none());
    }

    #[test]
    fn a_start_with_no_shell_around_it_counts_as_a_launcher_start() {
        // The flag, the relaunch marker, or alone in the console.
        assert!(infer_launcher(true, false, Some(3)));
        assert!(infer_launcher(false, true, Some(3)));
        assert!(infer_launcher(false, false, Some(1)));
        // Typed in a shell (it shares the console), or unknown: a normal start.
        assert!(!infer_launcher(false, false, Some(2)));
        assert!(!infer_launcher(false, false, None));
    }

    const OWN: Option<&str> = Some(CONSOLE_WINDOW_CLASS);
    const WT: Option<&str> = Some("windows-terminal");

    #[test]
    fn only_a_lone_double_click_with_a_real_terminal_relaunches() {
        assert!(should_relaunch(true, false, Some(1), OWN, WT));
        // Started from a shell: the shell shares the console.
        assert!(!should_relaunch(true, false, Some(2), OWN, WT));
        // Unknown console state.
        assert!(!should_relaunch(true, false, None, OWN, WT));
        // The switch is off.
        assert!(!should_relaunch(false, false, Some(1), OWN, WT));
        // The loop guard.
        assert!(!should_relaunch(true, true, Some(1), OWN, WT));
        // The plain console is already where we are; nothing found, stay.
        assert!(!should_relaunch(true, false, Some(1), OWN, Some("console")));
        assert!(!should_relaunch(true, false, Some(1), OWN, None));
    }

    #[test]
    fn a_terminal_that_runs_us_directly_is_not_relaunched() {
        // `wezterm-gui start -- gitpoulpi.exe`, `wt.exe gitpoulpi.exe`, Rio,
        // Tabby, Alacritty `-e`: alone in a pseudo-console, no shell around.
        assert!(!should_relaunch(
            true,
            false,
            Some(1),
            Some("PseudoConsoleWindow"),
            WT
        ));
        // No console window at all: nothing proves a double-click either.
        assert!(!should_relaunch(true, false, Some(1), None, WT));
        assert!(owns_console_window(Some("ConsoleWindowClass")));
        assert!(!owns_console_window(Some("PseudoConsoleWindow")));
        assert!(!owns_console_window(None));
    }
}
