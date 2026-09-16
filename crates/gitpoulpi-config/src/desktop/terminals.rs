//! The terminals a shortcut can start GitPoulpi in (`ui.launch_terminal`),
//! how each is told to run a program, and the "auto" choice.
//!
//! Pure: `has` answers "is this program (or `app:Name` bundle on macOS)
//! installed", so the caller decides what to look at (`PATH`, `/Applications`)
//! and tests pass a fake list.

use crate::open_in::Os;

/// The setting's two non-terminal values.
pub const AUTO: &str = "auto";
/// The OS's own terminal: the plain console on Windows, `xdg-terminal-exec`
/// on Linux, Terminal.app on macOS.
pub const DEFAULT: &str = "default";

/// One terminal a shortcut can name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Term {
    pub id: &'static str,
    pub label: &'static str,
    /// What `has` is asked: a program on `PATH`, or `app:Name` for a macOS
    /// bundle. Empty: always present.
    pub probe: &'static str,
    pub os: Os,
}

const fn t(id: &'static str, label: &'static str, probe: &'static str, os: Os) -> Term {
    Term {
        id,
        label,
        probe,
        os,
    }
}

const TERMS: &[Term] = &[
    t(
        "xdg-terminal-exec",
        "xdg-terminal-exec",
        "xdg-terminal-exec",
        Os::Linux,
    ),
    t("kitty", "kitty", "kitty", Os::Linux),
    t("foot", "foot", "foot", Os::Linux),
    t("alacritty", "Alacritty", "alacritty", Os::Linux),
    t("wezterm", "WezTerm", "wezterm", Os::Linux),
    t("ghostty", "Ghostty", "ghostty", Os::Linux),
    t(
        "gnome-terminal",
        "GNOME Terminal",
        "gnome-terminal",
        Os::Linux,
    ),
    t("ptyxis", "Ptyxis", "ptyxis", Os::Linux),
    t("konsole", "Konsole", "konsole", Os::Linux),
    t(
        "xfce4-terminal",
        "Xfce Terminal",
        "xfce4-terminal",
        Os::Linux,
    ),
    t("tilix", "Tilix", "tilix", Os::Linux),
    t("terminator", "Terminator", "terminator", Os::Linux),
    t("mate-terminal", "MATE Terminal", "mate-terminal", Os::Linux),
    t("windows-terminal", "Windows Terminal", "wt", Os::Windows),
    // `wezterm-gui.exe`, not `wezterm.exe`: the latter is a console-subsystem
    // program, so a shortcut to it opens a console window (Windows Terminal
    // when that is the default) that lingers next to the real WezTerm window.
    t("wezterm", "WezTerm", "wezterm-gui", Os::Windows),
    t("alacritty", "Alacritty", "alacritty", Os::Windows),
    t("console", "Windows console", "", Os::Windows),
    t("apple-terminal", "Terminal", "", Os::Macos),
    t("iterm", "iTerm2", "app:iTerm", Os::Macos),
    t("wezterm", "WezTerm", "app:WezTerm", Os::Macos),
    t("kitty", "kitty", "app:kitty", Os::Macos),
    t("ghostty", "Ghostty", "app:Ghostty", Os::Macos),
];

/// Every terminal known for `os`, in the order auto prefers them.
pub fn catalog(os: Os) -> Vec<&'static Term> {
    TERMS.iter().filter(|t| t.os == os).collect()
}

/// The terminal `id` names on `os`.
pub fn find(os: Os, id: &str) -> Option<&'static Term> {
    TERMS.iter().find(|t| t.os == os && t.id == id)
}

/// The terminals of `os` that are installed (the always-present ones included).
pub fn installed(os: Os, has: &dyn Fn(&str) -> bool) -> Vec<&'static Term> {
    catalog(os)
        .into_iter()
        .filter(|t| t.probe.is_empty() || has(t.probe))
        .collect()
}

/// What `auto` means here, from the installed terminals' ids.
///
/// Windows: Windows Terminal, else WezTerm, else Alacritty, else the plain
/// console. Linux: `xdg-terminal-exec` (Omarchy), else the first installed;
/// `None` when nothing is installed. macOS: another detected terminal
/// (iTerm2, Ghostty, WezTerm, kitty), else Terminal.app.
pub fn auto_choice(os: Os, installed_ids: &[&str]) -> Option<&'static str> {
    let first =
        |order: &[&'static str]| order.iter().copied().find(|id| installed_ids.contains(id));
    match os {
        Os::Windows => {
            Some(first(&["windows-terminal", "wezterm", "alacritty"]).unwrap_or("console"))
        }
        Os::Macos => {
            Some(first(&["iterm", "ghostty", "wezterm", "kitty"]).unwrap_or("apple-terminal"))
        }
        Os::Linux => first(&["xdg-terminal-exec"]).or_else(|| {
            catalog(Os::Linux)
                .into_iter()
                .map(|t| t.id)
                .find(|id| installed_ids.contains(id))
        }),
    }
}

/// The terminal id a `ui.launch_terminal` value means on this machine: `auto`
/// and an empty or unknown value are [`auto_choice`], `default` is the OS's
/// own terminal. `None` only on a Linux box with no terminal at all.
pub fn resolve(setting: &str, os: Os, has: &dyn Fn(&str) -> bool) -> Option<&'static str> {
    let setting = setting.trim();
    if setting == DEFAULT {
        return Some(match os {
            Os::Windows => "console",
            Os::Macos => "apple-terminal",
            Os::Linux => "xdg-terminal-exec",
        });
    }
    if let Some(term) = find(os, setting) {
        return Some(term.id);
    }
    let ids: Vec<&str> = installed(os, has).iter().map(|t| t.id).collect();
    auto_choice(os, &ids)
}

/// Whether the environment says we already run inside terminal `id`
/// (`var` answers an environment variable): the terminal's own marker
/// variable is set. Used to not relaunch into the terminal we are in.
pub fn inside(id: &str, var: &dyn Fn(&str) -> bool) -> bool {
    let markers: &[&str] = match id {
        "windows-terminal" => &["WT_SESSION"],
        "wezterm" => &["WEZTERM_PANE", "WEZTERM_EXECUTABLE"],
        "alacritty" => &["ALACRITTY_WINDOW_ID", "ALACRITTY_SOCKET"],
        _ => &[],
    };
    markers.iter().any(|m| var(m))
}

/// The argument every shortcut we write passes after the exe, so GitPoulpi
/// knows it was started from a launcher (`general.startup_tab_launcher`).
/// Shortcuts written before it existed keep working: they just start
/// without it, until `launcher::sync_shortcuts` rewrites them.
pub const LAUNCHER_FLAG: &str = "--launcher";

/// The window class (X11 `WM_CLASS`, Wayland `app_id`) every terminal we
/// start for a shortcut is given, and the `.desktop` entry's
/// `StartupWMClass`: that match is what lets a dock or taskbar put the
/// running window under the launcher's icon instead of under the terminal's.
pub const WM_CLASS: &str = "gitpoulpi";

/// Ghostty's GTK application id must contain a dot, so it cannot use
/// [`WM_CLASS`] as is.
const GHOSTTY_CLASS: &str = "io.github.poulpi_lab.gitpoulpi";

/// The Windows AppUserModelID of our shortcuts and of the terminal window
/// that runs GitPoulpi: the taskbar groups windows and pins by it.
pub const AUMID: &str = "GitPoulpi.GitPoulpi";

/// The class terminal `id` is started with on `os`, which the `.desktop`
/// entry must repeat as `StartupWMClass`. `None`: that terminal has no
/// reliable way to set it (it groups under its own icon), or the OS has no
/// such notion. `xdg-terminal-exec` keeps Omarchy's `TUI.tile` app id: the
/// window rules (floating, size) match it.
pub fn wm_class(os: Os, id: &str) -> Option<&'static str> {
    if os != Os::Linux {
        return None;
    }
    match id {
        "xdg-terminal-exec" => Some("TUI.tile"),
        "kitty" | "foot" | "alacritty" | "wezterm" => Some(WM_CLASS),
        "ghostty" => Some(GHOSTTY_CLASS),
        _ => None,
    }
}

/// The command that runs `inner` through `gitpoulpi launch`: it shows the
/// window of a GitPoulpi that is already open, else runs `inner`. Windows
/// uses the console-less helper (`gitpoulpi-launch.exe`), which has no
/// subcommand word; elsewhere `launch_exe` is `gitpoulpi` itself.
pub fn launch_argv(os: Os, launch_exe: &str, inner: &[String]) -> Vec<String> {
    launch_argv_with(os, launch_exe, inner, false)
}

/// [`launch_argv`], with `--new-window` when `new_window`: the same command
/// that never reuses an open window (the Linux "New Window" desktop action).
pub fn launch_argv_with(
    os: Os,
    launch_exe: &str,
    inner: &[String],
    new_window: bool,
) -> Vec<String> {
    let mut argv = vec![launch_exe.to_string()];
    if os != Os::Windows {
        argv.push("launch".to_string());
    }
    if new_window {
        argv.push("--new-window".to_string());
    }
    argv.push("--".to_string());
    argv.extend(inner.iter().cloned());
    argv
}

/// The program that runs `launch` for shortcuts next to `exe`, if there is
/// one: `gitpoulpi` itself on Linux (when `exe` is it, or sits beside it),
/// `gitpoulpi-launch.exe` beside the exe on Windows (a console-less build,
/// so a click never flashes a console window). `None`: shortcuts run the
/// terminal command directly, as before. macOS has none (the Dock already
/// raises a running app).
pub fn launch_helper(
    os: Os,
    exe: &std::path::Path,
    exists: &dyn Fn(&std::path::Path) -> bool,
) -> Option<std::path::PathBuf> {
    match os {
        Os::Linux => {
            if exe.file_stem().is_some_and(|s| s == "gitpoulpi") {
                return Some(exe.to_path_buf());
            }
            let sibling = exe.with_file_name("gitpoulpi");
            exists(&sibling).then_some(sibling)
        }
        Os::Windows => {
            let helper = exe.with_file_name("gitpoulpi-launch.exe");
            exists(&helper).then_some(helper)
        }
        Os::Macos => None,
    }
}

/// The argument vector that runs `exe` inside terminal `id` (Linux and
/// Windows; macOS goes through `open`, see `macos.rs`). `None` for the plain
/// Windows console (the shortcut points at the exe itself) and unknown ids.
pub fn argv(os: Os, id: &str, exe: &str) -> Option<Vec<String>> {
    let v = |parts: &[&str]| -> Vec<String> {
        parts
            .iter()
            .map(|p| (*p).to_string())
            .chain([exe.to_string(), LAUNCHER_FLAG.to_string()])
            .collect()
    };
    match (os, id) {
        (Os::Linux, "xdg-terminal-exec") => {
            Some(v(&["xdg-terminal-exec", "--app-id=TUI.tile", "-e"]))
        }
        (Os::Linux, "kitty") => Some(v(&["kitty", "--class", WM_CLASS])),
        (Os::Linux, "foot") => Some(v(&["foot", "--app-id=gitpoulpi"])),
        (Os::Linux, "alacritty") => Some(v(&["alacritty", "--class", WM_CLASS, "-e"])),
        (Os::Linux, "wezterm") => Some(v(&["wezterm", "start", "--class", WM_CLASS, "--"])),
        (Os::Linux, "ghostty") => Some(v(&[
            "ghostty",
            "--class=io.github.poulpi_lab.gitpoulpi",
            "-e",
        ])),
        (Os::Linux, "gnome-terminal") => Some(v(&["gnome-terminal", "--"])),
        (Os::Linux, "ptyxis") => Some(v(&["ptyxis", "--"])),
        (Os::Linux, "konsole") => Some(v(&["konsole", "-e"])),
        (Os::Linux, "xfce4-terminal") => Some(v(&["xfce4-terminal", "-x"])),
        (Os::Linux, "tilix") => Some(v(&["tilix", "-e"])),
        (Os::Linux, "terminator") => Some(v(&["terminator", "-x"])),
        (Os::Linux, "mate-terminal") => Some(v(&["mate-terminal", "-x"])),
        (Os::Windows, "windows-terminal") => Some(v(&["wt.exe"])),
        (Os::Windows, "wezterm") => Some(v(&["wezterm-gui.exe", "start", "--"])),
        (Os::Windows, "alacritty") => Some(v(&["alacritty.exe", "-e"])),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_on_windows_prefers_windows_terminal_then_wezterm_then_the_console() {
        assert_eq!(
            auto_choice(Os::Windows, &["wezterm", "windows-terminal"]),
            Some("windows-terminal")
        );
        assert_eq!(
            auto_choice(Os::Windows, &["alacritty", "wezterm"]),
            Some("wezterm")
        );
        assert_eq!(auto_choice(Os::Windows, &[]), Some("console"));
    }

    #[test]
    fn auto_on_linux_prefers_xdg_terminal_exec_then_the_first_installed() {
        assert_eq!(
            auto_choice(Os::Linux, &["kitty", "xdg-terminal-exec"]),
            Some("xdg-terminal-exec")
        );
        assert_eq!(auto_choice(Os::Linux, &["konsole", "foot"]), Some("foot"));
        assert_eq!(auto_choice(Os::Linux, &[]), None);
    }

    #[test]
    fn auto_on_macos_is_terminal_app_unless_another_is_detected() {
        assert_eq!(
            auto_choice(Os::Macos, &["apple-terminal"]),
            Some("apple-terminal")
        );
        assert_eq!(
            auto_choice(Os::Macos, &["apple-terminal", "kitty", "iterm"]),
            Some("iterm")
        );
    }

    #[test]
    fn resolve_maps_settings_to_ids_from_a_fake_machine() {
        let has = |p: &str| matches!(p, "kitty" | "wezterm");
        assert_eq!(resolve("auto", Os::Linux, &has), Some("kitty"));
        assert_eq!(resolve("", Os::Linux, &has), Some("kitty"));
        assert_eq!(resolve("wezterm", Os::Linux, &has), Some("wezterm"));
        assert_eq!(resolve("nonsense", Os::Linux, &has), Some("kitty"));
        assert_eq!(resolve("default", Os::Windows, &has), Some("console"));
        assert_eq!(resolve("auto", Os::Linux, &|_| false), None);
    }

    #[test]
    fn installed_lists_the_probed_programs_and_the_always_present_ones() {
        let ids: Vec<_> = installed(Os::Windows, &|p| p == "wt")
            .iter()
            .map(|t| t.id)
            .collect();
        assert_eq!(ids, ["windows-terminal", "console"]);
    }

    #[test]
    fn wezterm_on_windows_starts_the_gui_program_not_the_console_one() {
        let a = argv(Os::Windows, "wezterm", "C:/gp.exe").unwrap();
        assert_eq!(
            a,
            ["wezterm-gui.exe", "start", "--", "C:/gp.exe", "--launcher"]
        );
        assert_eq!(find(Os::Windows, "wezterm").unwrap().probe, "wezterm-gui");
    }

    #[test]
    fn linux_terminals_that_can_are_started_with_the_class_the_entry_names() {
        for (id, class_arg) in [
            ("kitty", "--class gitpoulpi"),
            ("foot", "--app-id=gitpoulpi"),
            ("alacritty", "--class gitpoulpi"),
            ("wezterm", "--class gitpoulpi"),
            ("ghostty", "--class=io.github.poulpi_lab.gitpoulpi"),
        ] {
            let line = argv(Os::Linux, id, "/x/gp").unwrap().join(" ");
            assert!(line.contains(class_arg), "{id}: {line}");
            let class = wm_class(Os::Linux, id).unwrap();
            assert!(class_arg.ends_with(class), "{id}: {class}");
        }
        // Omarchy's own app id stays: its window rules match it.
        assert_eq!(wm_class(Os::Linux, "xdg-terminal-exec"), Some("TUI.tile"));
        assert_eq!(wm_class(Os::Linux, "konsole"), None);
        assert_eq!(wm_class(Os::Windows, "wezterm"), None);
    }

    #[test]
    fn launch_wraps_the_terminal_command_after_a_double_dash() {
        let inner = argv(Os::Linux, "kitty", "/x/gp").unwrap();
        let line = launch_argv(Os::Linux, "/x/gp", &inner);
        assert_eq!(
            line[..3],
            ["/x/gp".to_string(), "launch".into(), "--".into()]
        );
        assert_eq!(line[3..], inner[..]);
        // The Windows helper has no subcommand word.
        let w = launch_argv(
            Os::Windows,
            r"C:\gp\gitpoulpi-launch.exe",
            &["wt.exe".into()],
        );
        assert_eq!(w, [r"C:\gp\gitpoulpi-launch.exe", "--", "wt.exe"]);
    }

    #[test]
    fn the_launch_helper_is_found_beside_the_exe() {
        use std::path::Path;
        let none = |_: &Path| false;
        let all = |_: &Path| true;
        // Linux: gitpoulpi itself, or the one beside gitpoulpi-tui.
        assert_eq!(
            launch_helper(Os::Linux, Path::new("/o/gitpoulpi"), &none),
            Some("/o/gitpoulpi".into())
        );
        assert_eq!(
            launch_helper(Os::Linux, Path::new("/o/gitpoulpi-tui"), &all),
            Some("/o/gitpoulpi".into())
        );
        assert_eq!(
            launch_helper(Os::Linux, Path::new("/o/gitpoulpi-tui"), &none),
            None
        );
        // Windows: only when the console-less helper is installed.
        assert_eq!(
            launch_helper(Os::Windows, Path::new("C:/gp/gitpoulpi.exe"), &all),
            Some("C:/gp/gitpoulpi-launch.exe".into())
        );
        assert_eq!(
            launch_helper(Os::Windows, Path::new("C:/gp/gitpoulpi.exe"), &none),
            None
        );
        assert_eq!(
            launch_helper(Os::Macos, Path::new("/o/gitpoulpi"), &all),
            None
        );
    }

    #[test]
    fn inside_reads_the_terminals_own_marker_variable() {
        let set = |name: &'static str| move |v: &str| v == name;
        assert!(inside("wezterm", &set("WEZTERM_PANE")));
        assert!(inside("windows-terminal", &set("WT_SESSION")));
        // Another terminal's marker does not count.
        assert!(!inside("wezterm", &set("WT_SESSION")));
        assert!(!inside("console", &set("WT_SESSION")));
    }

    #[test]
    fn every_catalogued_terminal_has_an_argv_except_the_console_and_macos() {
        for term in TERMS
            .iter()
            .filter(|t| t.os != Os::Macos && t.id != "console")
        {
            let a = argv(term.os, term.id, "/x/gp").unwrap_or_else(|| panic!("{}", term.id));
            assert_eq!(a[a.len() - 2..], ["/x/gp", LAUNCHER_FLAG], "{}", term.id);
        }
        assert!(argv(Os::Windows, "console", "x").is_none());
    }
}
