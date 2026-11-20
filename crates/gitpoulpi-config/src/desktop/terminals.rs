//! The terminals a shortcut can start GitPoulpi in (`ui.launch_terminal`),
//! how each is told to run a program, and the "auto" choice.
//!
//! Pure: `has` answers "is this program (or `app:Name` bundle on macOS)
//! installed", so the caller decides what to look at (`PATH`, `/Applications`)
//! and tests pass a fake list.

use super::InstallCtx;
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

/// 16 lower-case hex digits for an installation id: the id itself when it
/// is one, else its FNV-1a-64 (a pinned `p-<name>` id).
fn hex16(id: &str) -> String {
    if id.len() == 16 && id.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) {
        id.to_string()
    } else {
        format!("{:016x}", crate::installation::fnv1a64(id.as_bytes()))
    }
}

/// The AppUserModelID of `ctx`'s shortcuts and terminal window: the primary
/// keeps [`AUMID`], every other installation gets `GitPoulpi.I<16hex>` so its
/// windows and pins group apart on the taskbar.
pub fn aumid_for(ctx: &InstallCtx) -> String {
    if ctx.is_primary() {
        AUMID.to_string()
    } else {
        format!("GitPoulpi.I{}", hex16(&ctx.id))
    }
}

/// [`wm_class`] for `ctx`: the primary keeps today's class, the others get
/// `gitpoulpi-i<8hex>` (Ghostty: `io.github.poulpi_lab.gitpoulpi.i<8hex>`, a
/// valid GTK application id). `xdg-terminal-exec` keeps `TUI.tile` for all.
pub fn wm_class_for(os: Os, id: &str, ctx: &InstallCtx) -> Option<String> {
    let base = wm_class(os, id)?;
    if ctx.is_primary() || base == "TUI.tile" {
        return Some(base.to_string());
    }
    Some(match id {
        "ghostty" => format!("{base}.{}", ctx.short),
        _ => format!("{base}-{}", ctx.short),
    })
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

/// `argv` for opening `folder`: the same command ending in
/// `exe --launcher --repo <folder>`. `None` for an unknown terminal.
///
/// - Windows `console`: the exe itself.
/// - `windows-terminal`: `wt` reads `;` as a command separator, so it is
///   escaped as `\;` in the folder word.
/// - macOS Terminal and iTerm2 are driven through `osascript` (the command is
///   shell-quoted, then escaped for an AppleScript string); WezTerm, kitty
///   and Ghostty use the `open -na ... --args` forms of the app launcher.
pub fn folder_argv(os: Os, id: &str, exe: &str, folder: &str) -> Option<Vec<String>> {
    let tail = |mut v: Vec<String>| {
        v.push("--repo".to_string());
        v.push(folder.to_string());
        v
    };
    let words = |parts: &[&str]| -> Vec<String> {
        parts
            .iter()
            .map(|p| (*p).to_string())
            .chain([exe.to_string(), LAUNCHER_FLAG.to_string()])
            .collect()
    };
    match (os, id) {
        (Os::Windows, "console") => Some(tail(words(&[]))),
        (Os::Windows, "windows-terminal") => {
            let mut v = argv(os, id, exe)?;
            v.push("--repo".to_string());
            v.push(folder.replace(';', "\\;"));
            Some(v)
        }
        (Os::Linux | Os::Windows, _) => argv(os, id, exe).map(tail),
        (Os::Macos, "apple-terminal") => {
            let esc = mac_script_arg(exe, folder);
            Some(vec![
                "osascript".to_string(),
                "-e".to_string(),
                format!("tell application \"Terminal\" to do script \"{esc}\""),
                "-e".to_string(),
                "tell application \"Terminal\" to activate".to_string(),
            ])
        }
        (Os::Macos, "iterm") => {
            let esc = mac_script_arg(exe, folder);
            Some(vec![
                "osascript".to_string(),
                "-e".to_string(),
                format!(
                    "tell application \"iTerm\" to create window with default profile command \"{esc}\""
                ),
            ])
        }
        (Os::Macos, "wezterm") => Some(tail(words(&[
            "/usr/bin/open",
            "-na",
            "WezTerm",
            "--args",
            "start",
            "--",
        ]))),
        (Os::Macos, "kitty") => Some(tail(words(&["/usr/bin/open", "-na", "kitty", "--args"]))),
        (Os::Macos, "ghostty") => Some(tail(words(&[
            "/usr/bin/open",
            "-na",
            "Ghostty",
            "--args",
            "-e",
        ]))),
        (Os::Macos, _) => None,
    }
}

/// The shell command for macOS, quoted for a shell and then escaped for an
/// AppleScript string literal.
fn mac_script_arg(exe: &str, folder: &str) -> String {
    let cmd = format!(
        "{} {} --repo {}",
        super::sh_quote(exe),
        LAUNCHER_FLAG,
        super::sh_quote(folder)
    );
    cmd.replace('\\', "\\\\").replace('"', "\\\"")
}

/// The program a file-manager entry runs (`<helper> [launch] --folder DIR`):
/// the console-less `gitpoulpi-launch.exe` beside the exe on Windows;
/// `gitpoulpi` itself, or the one beside the exe, on Linux and macOS.
pub fn folder_helper(
    os: Os,
    exe: &std::path::Path,
    exists: &dyn Fn(&std::path::Path) -> bool,
) -> Option<std::path::PathBuf> {
    match os {
        Os::Windows => launch_helper(os, exe, exists),
        Os::Linux | Os::Macos => launch_helper(Os::Linux, exe, exists),
    }
}

/// A `--folder` value as the shell handed it over. Explorer's `"%V"` on a
/// drive root is `"C:\"`, and argv parsing reads `\"` as an escaped quote, so
/// the program receives `C:"`: the quote goes and a bare drive gets its `\`.
pub fn normalize_folder_arg(s: &str) -> std::path::PathBuf {
    let t = s.strip_suffix('"').unwrap_or(s);
    let b = t.as_bytes();
    if b.len() == 2 && b[0].is_ascii_alphabetic() && b[1] == b':' {
        return std::path::PathBuf::from(format!("{t}\\"));
    }
    std::path::PathBuf::from(t)
}

fn folded(os: Os, p: &std::path::Path) -> String {
    let s = p.to_string_lossy();
    let t = s.trim_end_matches(['/', '\\']);
    if os == Os::Windows {
        t.replace('/', "\\").to_lowercase()
    } else {
        t.to_string()
    }
}

/// The folder an address-bar start means (Explorer starts the program in the
/// folder it shows): `cwd`, unless it is not a directory, is the install
/// folder or the home folder (where a Run dialog starts), or is inside the
/// Windows folder (where a program started without a context lands).
pub fn address_bar_folder(
    os: Os,
    cwd: &std::path::Path,
    exe_dir: Option<&std::path::Path>,
    home: Option<&std::path::Path>,
    windir: Option<&std::path::Path>,
    is_dir: &dyn Fn(&std::path::Path) -> bool,
) -> Option<std::path::PathBuf> {
    if !is_dir(cwd) {
        return None;
    }
    let c = folded(os, cwd);
    let sep = if os == Os::Windows { '\\' } else { '/' };
    for p in [exe_dir, home].into_iter().flatten() {
        if folded(os, p) == c {
            return None;
        }
    }
    if let Some(w) = windir {
        let w = folded(os, w);
        if c == w || c.starts_with(&format!("{w}{sep}")) {
            return None;
        }
    }
    Some(cwd.to_path_buf())
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    #[test]
    fn folder_argv_ends_in_the_repo_for_every_terminal() {
        for (os, id) in [
            (Os::Linux, "kitty"),
            (Os::Linux, "xdg-terminal-exec"),
            (Os::Linux, "gnome-terminal"),
            (Os::Linux, "ghostty"),
            (Os::Windows, "windows-terminal"),
            (Os::Windows, "wezterm"),
            (Os::Windows, "alacritty"),
            (Os::Windows, "console"),
        ] {
            let v = folder_argv(os, id, "/x/gp", "/work/dir").unwrap();
            let n = v.len();
            assert_eq!(
                v[n - 4..],
                ["/x/gp", "--launcher", "--repo", "/work/dir"],
                "{id}"
            );
        }
        assert_eq!(
            folder_argv(Os::Windows, "console", "gp.exe", "D:\\a").unwrap(),
            ["gp.exe", "--launcher", "--repo", "D:\\a"]
        );
        assert!(folder_argv(Os::Linux, "nope", "x", "y").is_none());
    }

    #[test]
    fn windows_terminal_gets_the_semicolon_escaped() {
        let v = folder_argv(Os::Windows, "windows-terminal", "gp.exe", "C:\\a;b").unwrap();
        assert_eq!(v.last().unwrap(), "C:\\a\\;b");
        let v = folder_argv(Os::Windows, "wezterm", "gp.exe", "C:\\a;b").unwrap();
        assert_eq!(v.last().unwrap(), "C:\\a;b");
    }

    #[test]
    fn macos_terminal_and_iterm_go_through_escaped_applescript() {
        let v = folder_argv(Os::Macos, "apple-terminal", "/App's/gp", "/a \"b\"").unwrap();
        assert_eq!(v[0], "osascript");
        assert!(v[2].starts_with("tell application \"Terminal\" to do script \""));
        assert!(v[2].contains("--repo '/a \\\"b\\\"'"), "{}", v[2]);
        assert!(v[2].contains("'/App'\\\\''s/gp'"), "{}", v[2]);
        assert_eq!(v[4], "tell application \"Terminal\" to activate");
        let v = folder_argv(Os::Macos, "iterm", "/gp", "/d").unwrap();
        assert!(v[2].contains("create window with default profile command"));
        let v = folder_argv(Os::Macos, "kitty", "/gp", "/d").unwrap();
        assert_eq!(
            v,
            [
                "/usr/bin/open",
                "-na",
                "kitty",
                "--args",
                "/gp",
                "--launcher",
                "--repo",
                "/d"
            ]
        );
    }

    #[test]
    fn the_folder_helper_is_the_launch_exe_on_windows_and_the_cli_elsewhere() {
        let yes = |_: &Path| true;
        assert_eq!(
            folder_helper(Os::Windows, Path::new("/g/gitpoulpi.exe"), &yes),
            Some(PathBuf::from("/g/gitpoulpi-launch.exe"))
        );
        assert_eq!(
            folder_helper(Os::Macos, Path::new("/a/gitpoulpi-tui"), &yes),
            Some(PathBuf::from("/a/gitpoulpi"))
        );
        assert_eq!(
            folder_helper(Os::Linux, Path::new("/a/gitpoulpi-tui"), &|_| false),
            None
        );
    }

    #[test]
    fn a_drive_root_argument_loses_the_stray_quote_and_regains_its_backslash() {
        assert_eq!(normalize_folder_arg("C:\""), PathBuf::from("C:\\"));
        assert_eq!(normalize_folder_arg("C:"), PathBuf::from("C:\\"));
        assert_eq!(normalize_folder_arg("C:\\a b"), PathBuf::from("C:\\a b"));
        assert_eq!(normalize_folder_arg("/x/y"), PathBuf::from("/x/y"));
    }

    #[test]
    fn the_address_bar_folder_skips_install_home_windows_and_non_directories() {
        let dir = |_: &Path| true;
        let win = |cwd: &str| {
            address_bar_folder(
                Os::Windows,
                Path::new(cwd),
                Some(Path::new("C:\\Tools\\GitPoulpi")),
                Some(Path::new("C:\\Users\\Me")),
                Some(Path::new("C:\\Windows")),
                &dir,
            )
        };
        assert_eq!(win("D:\\Work\\repo"), Some(PathBuf::from("D:\\Work\\repo")));
        assert_eq!(win("c:\\tools\\gitpoulpi\\"), None);
        assert_eq!(win("C:\\USERS\\me"), None);
        assert_eq!(win("c:\\windows\\SYSTEM32"), None);
        assert_eq!(win("C:\\Windows"), None);
        assert_eq!(
            win("C:\\WindowsApps"),
            Some(PathBuf::from("C:\\WindowsApps"))
        );
        assert_eq!(
            address_bar_folder(Os::Linux, Path::new("/x"), None, None, None, &|_| false),
            None
        );
        assert_eq!(
            address_bar_folder(
                Os::Linux,
                Path::new("/Home"),
                None,
                Some(Path::new("/home")),
                None,
                &dir
            ),
            Some(PathBuf::from("/Home"))
        );
    }

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

    fn ctx(id: &str, primary: Option<&str>) -> InstallCtx {
        InstallCtx {
            id: id.into(),
            short: crate::installation::short_id(id),
            name: "n".into(),
            primary: primary.map(str::to_string),
            known: Vec::new(),
        }
    }

    #[test]
    fn primary_keeps_gitpoulpi_class_and_aumid_others_get_their_own() {
        let id = "0123456789abcdef";
        let primary = ctx(id, Some(id));
        let other = ctx(id, Some("ffffffffffffffff"));
        let nobody = ctx(id, None);
        assert_eq!(aumid_for(&primary), "GitPoulpi.GitPoulpi");
        assert_eq!(aumid_for(&other), "GitPoulpi.I0123456789abcdef");
        assert_eq!(aumid_for(&nobody), aumid_for(&other));
        assert_eq!(
            wm_class_for(Os::Linux, "kitty", &primary).unwrap(),
            "gitpoulpi"
        );
        assert_eq!(
            wm_class_for(Os::Linux, "kitty", &other).unwrap(),
            "gitpoulpi-i01234567"
        );
        assert_eq!(
            wm_class_for(Os::Linux, "ghostty", &primary).unwrap(),
            "io.github.poulpi_lab.gitpoulpi"
        );
        // Omarchy's window rules match TUI.tile whoever asks.
        assert_eq!(
            wm_class_for(Os::Linux, "xdg-terminal-exec", &other).unwrap(),
            "TUI.tile"
        );
        assert_eq!(wm_class_for(Os::Windows, "wezterm", &other), None);
        assert_eq!(wm_class_for(Os::Linux, "konsole", &other), None);
        // A pinned id hashes to 16 hex too.
        assert_eq!(
            aumid_for(&ctx("p-ci", None)).len(),
            "GitPoulpi.I".len() + 16
        );
    }

    #[test]
    fn ghostty_ids_stay_valid_gtk_ids() {
        // GTK: two or more dot-separated elements of [A-Za-z0-9_-], none empty,
        // none starting with a digit.
        fn valid(id: &str) -> bool {
            let parts: Vec<&str> = id.split('.').collect();
            parts.len() >= 2
                && parts.iter().all(|p| {
                    !p.is_empty()
                        && !p.starts_with(|c: char| c.is_ascii_digit())
                        && p.chars()
                            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
                })
        }
        for c in [
            ctx("0123456789abcdef", Some("0123456789abcdef")),
            ctx("0123456789abcdef", None),
            ctx("9999999999999999", None),
            ctx("p-ci_1", None),
        ] {
            let class = wm_class_for(Os::Linux, "ghostty", &c).unwrap();
            assert!(valid(&class), "{class}");
        }
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
