//! The "Open in" launcher (`[ui] open_in`, `[ui] open_in_default`): a list of
//! labeled commands the tab bar offers for the active tab's folder.
//!
//! Pure functions only, nothing is spawned and nothing reads the process
//! environment (the caller passes an `env` and an `on_path` closure), so every
//! rule is table-testable:
//!
//! - **Expansion**: the command is split into arguments first
//!   ([`launch::split_argv`]), then the variables are substituted inside each
//!   argument, so a folder with spaces or quotes stays one argument. There is no
//!   implicit shell. Variables: `${cwd}`, `${repo}`, `${name}`, `${branch}`,
//!   `${env:NAME}`; the legacy `{repo}` of `ui.terminal_launch_command` means
//!   `${cwd}`. An unknown `${x}` is an error naming it; a `$` that does not
//!   start `${` (a shell snippet's `"$GITPOULPI_CWD"`) is literal.
//! - **Detection** ([`detect`]): which multiplexer, terminal, desktop and
//!   editors the user seems to have, from environment variables and `PATH`.
//! - **Presets** ([`presets`]): the catalog "Add preset" offers. `verified`
//!   means the flags were checked against the installed program's own help
//!   while writing this; the others follow published documentation. The UI
//!   shows no flag either way.
//! - **Effective list** ([`effective_list`]): while `ui.open_in` is absent, the
//!   list is computed from the migration of the legacy keys plus detection.

use serde::{Deserialize, Serialize};

use crate::launch;

/// One entry of `[[ui.open_in]]`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct OpenInEntry {
    pub label: String,
    pub command: String,
}

impl OpenInEntry {
    pub fn new(label: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            command: command.into(),
        }
    }
}

/// The operating systems presets and detection distinguish.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Windows,
    Macos,
    Linux,
}

impl Os {
    pub fn current() -> Self {
        if cfg!(windows) {
            Os::Windows
        } else if cfg!(target_os = "macos") {
            Os::Macos
        } else {
            Os::Linux
        }
    }

    fn name(self) -> &'static str {
        match self {
            Os::Windows => "windows",
            Os::Macos => "macos",
            Os::Linux => "linux",
        }
    }
}

// ---------------------------------------------------------------- expansion

/// What the variables stand for. Empty strings are legal (`branch` of a
/// detached HEAD).
#[derive(Debug, Clone, Copy, Default)]
pub struct OpenInVars<'a> {
    /// The tab's folder.
    pub cwd: &'a str,
    /// The repository root (differs from `cwd` for a worktree tab).
    pub repo: &'a str,
    /// The folder's base name.
    pub name: &'a str,
    /// The checked-out branch, empty when detached.
    pub branch: &'a str,
    /// The web page of the origin remote, empty unless it is a GitHub or
    /// GitLab (incl. self-managed) project.
    pub web_url: &'a str,
}

fn substitute(
    arg: &str,
    vars: &OpenInVars,
    env: &dyn Fn(&str) -> Option<String>,
) -> Result<String, String> {
    let mut out = String::with_capacity(arg.len());
    let mut rest = arg;
    while let Some(i) = rest.find(['$', '{']) {
        out.push_str(&rest[..i]);
        rest = &rest[i..];
        if let Some(after) = rest.strip_prefix("${") {
            let Some(end) = after.find('}') else {
                return Err(format!("unterminated ${{ in \"{arg}\""));
            };
            let name = &after[..end];
            match name {
                "cwd" => out.push_str(vars.cwd),
                "repo" => out.push_str(vars.repo),
                "name" => out.push_str(vars.name),
                "branch" => out.push_str(vars.branch),
                "web_url" => out.push_str(vars.web_url),
                _ => match name.strip_prefix("env:") {
                    Some(var) if !var.is_empty() => out.push_str(&env(var).unwrap_or_default()),
                    _ => return Err(format!("unknown variable ${{{name}}}")),
                },
            }
            rest = &after[end + 1..];
        } else if let Some(after) = rest.strip_prefix("{repo}") {
            // Legacy `ui.terminal_launch_command`.
            out.push_str(vars.cwd);
            rest = after;
        } else {
            let c = rest.chars().next().unwrap_or('$');
            out.push(c);
            rest = &rest[c.len_utf8()..];
        }
    }
    out.push_str(rest);
    Ok(out)
}

/// Splits `command`, then substitutes the variables inside each argument.
/// An empty command or an unknown variable is an error.
pub fn expand(
    command: &str,
    vars: &OpenInVars,
    env: &dyn Fn(&str) -> Option<String>,
) -> Result<Vec<String>, String> {
    let args = launch::split_argv(command)?;
    if args.is_empty() {
        return Err("the command is empty".into());
    }
    args.iter().map(|a| substitute(a, vars, env)).collect()
}

/// Checks a command without running it: it splits, is not empty and names only
/// known variables. What Settings shows next to the Command field.
pub fn validate(command: &str) -> Result<(), String> {
    expand(command, &OpenInVars::default(), &|_| None).map(|_| ())
}

// ------------------------------------------------------------------- presets

/// One built-in entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenInPreset {
    pub id: &'static str,
    pub label: &'static str,
    /// One of [`GROUPS`].
    pub group: &'static str,
    pub command: &'static str,
    /// `windows`, `macos`, `linux`: where it is offered.
    pub platforms: &'static [&'static str],
    /// The detection id (multiplexer or terminal) that proposes it when the
    /// process runs inside it, or `""`.
    pub host: &'static str,
    /// The program the command starts (looked up on `PATH`), or `""` when it
    /// cannot be looked up (macOS apps, `explorer.exe`).
    pub bin: &'static str,
    /// For "run a tool in a new pane": the tool that must also be installed.
    pub tool: &'static str,
    /// Checked by running it, or against the installed program's own help.
    pub verified: bool,
}

/// The categories of the Add preset picker, in the order it lists them.
/// `explorer` is the OS's own file manager, `filemanager` the others
/// (including the TUI ones run in a pane).
pub const GROUPS: &[&str] = &[
    "explorer",
    "filemanager",
    "multiplexer",
    "terminal",
    "editor",
    "git",
    "agent",
    "web",
];

const ALL: &[&str] = &["linux", "macos", "windows"];
const UNIX: &[&str] = &["linux", "macos"];
const LINUX: &[&str] = &["linux"];
const MACOS: &[&str] = &["macos"];
const WINDOWS: &[&str] = &["windows"];

#[allow(clippy::too_many_arguments)]
const fn p(
    id: &'static str,
    label: &'static str,
    group: &'static str,
    command: &'static str,
    platforms: &'static [&'static str],
    host: &'static str,
    bin: &'static str,
    verified: bool,
) -> OpenInPreset {
    OpenInPreset {
        id,
        label,
        group,
        command,
        platforms,
        host,
        bin,
        tool: "",
        verified,
    }
}

/// The hand-written presets, in the order Add preset lists them within a
/// group. The tool-in-a-pane variants are generated by [`presets`].
#[rustfmt::skip]
const BASE: &[OpenInPreset] = &[
    // The OS's own file manager: `explorer.exe` exits 1 on success, the runner treats it as such.
    p("explorer-windows", "File explorer", "explorer", "explorer.exe ${cwd}", WINDOWS, "", "", true),
    p("explorer-macos", "Finder", "explorer", "open ${cwd}", MACOS, "", "", true),
    p("explorer-linux", "File manager", "explorer", "xdg-open ${cwd}", LINUX, "", "xdg-open", true),
    // Other graphical file managers.
    p("nautilus", "Files (Nautilus)", "filemanager", "nautilus ${cwd}", LINUX, "", "nautilus", false),
    p("dolphin", "Dolphin", "filemanager", "dolphin ${cwd}", LINUX, "", "dolphin", false),
    p("thunar", "Thunar", "filemanager", "thunar ${cwd}", LINUX, "", "thunar", false),
    p("nemo", "Nemo", "filemanager", "nemo ${cwd}", LINUX, "", "nemo", false),
    p("pcmanfm", "PCManFM", "filemanager", "pcmanfm ${cwd}", LINUX, "", "pcmanfm", false),
    // Multiplexers.
    p("herdr-tab", "Herdr: new tab", "multiplexer", "herdr tab create --workspace ${env:HERDR_WORKSPACE_ID} --cwd ${cwd} --focus", ALL, "herdr", "herdr", true),
    p("herdr-workspace", "Herdr: new workspace", "multiplexer", "herdr workspace create --cwd ${cwd} --label ${name} --focus", ALL, "herdr", "herdr", true),
    p("herdr-split", "Herdr: split pane", "multiplexer", "herdr pane split --current --direction right --cwd ${cwd} --focus", ALL, "herdr", "herdr", true),
    p("herdr-split-down", "Herdr: split pane (down)", "multiplexer", "herdr pane split --current --direction down --cwd ${cwd} --focus", ALL, "herdr", "herdr", true),
    p("tmux-window", "tmux: new window", "multiplexer", "tmux new-window -c ${cwd}", UNIX, "tmux", "tmux", true),
    p("tmux-split", "tmux: split pane", "multiplexer", "tmux split-window -h -c ${cwd}", UNIX, "tmux", "tmux", true),
    p("tmux-split-down", "tmux: split pane (down)", "multiplexer", "tmux split-window -v -c ${cwd}", UNIX, "tmux", "tmux", true),
    p("tmux-session", "tmux: new detached session", "multiplexer", "tmux new-session -d -c ${cwd} -s ${name}", UNIX, "tmux", "tmux", true),
    p("zellij-pane", "zellij: new pane", "multiplexer", "zellij action new-pane --cwd ${cwd}", UNIX, "zellij", "zellij", false),
    p("zellij-tab", "zellij: new tab", "multiplexer", "zellij action new-tab --cwd ${cwd}", UNIX, "zellij", "zellij", false),
    p("zellij-floating", "zellij: floating pane", "multiplexer", "zellij action new-pane --floating --cwd ${cwd}", UNIX, "zellij", "zellij", false),
    p("wezterm-tab", "WezTerm: new tab", "multiplexer", "wezterm cli spawn --cwd ${cwd}", ALL, "wezterm", "wezterm", false),
    p("wezterm-split", "WezTerm: split pane", "multiplexer", "wezterm cli split-pane --right --cwd ${cwd}", ALL, "wezterm", "wezterm", false),
    p("wezterm-split-down", "WezTerm: split pane (down)", "multiplexer", "wezterm cli split-pane --bottom --cwd ${cwd}", ALL, "wezterm", "wezterm", false),
    p("wezterm-newwin", "WezTerm: new window (same instance)", "multiplexer", "wezterm cli spawn --new-window --cwd ${cwd}", ALL, "wezterm", "wezterm", false),
    p("kitty-tab", "kitty: new tab (remote control)", "multiplexer", "kitten @ launch --type=tab --cwd=${cwd}", UNIX, "kitty", "kitten", true),
    p("kitty-window", "kitty: split window (remote control)", "multiplexer", "kitten @ launch --type=window --cwd=${cwd}", UNIX, "kitty", "kitten", true),
    p("kitty-oswindow", "kitty: new OS window (remote control)", "multiplexer", "kitten @ launch --type=os-window --cwd=${cwd}", UNIX, "kitty", "kitten", true),
    p("screen-window", "screen: new window", "multiplexer", "screen -X eval 'chdir ${cwd}' screen", UNIX, "", "screen", false),
    p("cmux-workspace", "cmux: new workspace", "multiplexer", "cmux new-workspace --cwd ${cwd}", UNIX, "cmux", "cmux", false),
    p("psmux-window", "psmux: new window", "multiplexer", "psmux new-window -c ${cwd}", WINDOWS, "", "psmux", false),
    // Terminals (a new window running the user's shell in the folder).
    p("xdg-terminal-exec", "Default terminal (xdg-terminal-exec)", "terminal", "xdg-terminal-exec --dir=${cwd}", LINUX, "", "xdg-terminal-exec", true),
    p("kitty", "kitty", "terminal", "kitty --directory ${cwd}", UNIX, "kitty", "kitty", true),
    p("foot", "foot", "terminal", "foot --working-directory ${cwd}", LINUX, "foot", "foot", true),
    p("alacritty", "Alacritty", "terminal", "alacritty --working-directory ${cwd}", ALL, "alacritty", "alacritty", false),
    p("alacritty-msg", "Alacritty: new window (existing daemon)", "terminal", "alacritty msg create-window --working-directory ${cwd}", ALL, "alacritty", "alacritty", false),
    p("wezterm-window", "WezTerm: new window", "terminal", "wezterm start --cwd ${cwd}", ALL, "wezterm", "wezterm", false),
    p("ghostty", "Ghostty", "terminal", "ghostty --working-directory=${cwd}", LINUX, "ghostty", "ghostty", false),
    p("ghostty-macos", "Ghostty", "terminal", "open -na Ghostty --args --working-directory=${cwd}", MACOS, "ghostty", "", false),
    p("gnome-terminal", "GNOME Terminal", "terminal", "gnome-terminal --working-directory=${cwd}", LINUX, "gnome-terminal", "gnome-terminal", false),
    p("ptyxis", "Ptyxis", "terminal", "ptyxis --new-window --working-directory=${cwd}", LINUX, "", "ptyxis", false),
    p("konsole", "Konsole", "terminal", "konsole --workdir ${cwd}", LINUX, "konsole", "konsole", false),
    p("xfce4-terminal", "Xfce Terminal", "terminal", "xfce4-terminal --working-directory=${cwd}", LINUX, "", "xfce4-terminal", false),
    p("tilix", "Tilix", "terminal", "tilix --working-directory=${cwd}", LINUX, "", "tilix", false),
    p("terminator", "Terminator", "terminal", "terminator --working-directory=${cwd}", LINUX, "", "terminator", false),
    p("mate-terminal", "MATE Terminal", "terminal", "mate-terminal --working-directory=${cwd}", LINUX, "", "mate-terminal", false),
    p("iterm", "iTerm2", "terminal", "open -a iTerm ${cwd}", MACOS, "iterm", "", false),
    p("terminal-app", "Terminal", "terminal", "open -a Terminal ${cwd}", MACOS, "apple-terminal", "", false),
    p("wt-window", "Windows Terminal: new window", "terminal", "wt.exe -d ${cwd}", WINDOWS, "windows-terminal", "wt", false),
    p("wt-tab", "Windows Terminal: new tab", "terminal", "wt.exe -w 0 new-tab -d ${cwd}", WINDOWS, "windows-terminal", "wt", false),
    p("wt-split", "Windows Terminal: split pane", "terminal", "wt.exe -w 0 split-pane -d ${cwd}", WINDOWS, "windows-terminal", "wt", false),
    p("wt-powershell", "Windows Terminal: PowerShell 7 (tab)", "terminal", "wt.exe -w 0 new-tab -d ${cwd} pwsh.exe", WINDOWS, "windows-terminal", "wt", false),
    p("wt-winps", "Windows Terminal: Windows PowerShell (tab)", "terminal", "wt.exe -w 0 new-tab -d ${cwd} powershell.exe", WINDOWS, "windows-terminal", "wt", false),
    p("wt-cmd", "Windows Terminal: Command Prompt (tab)", "terminal", "wt.exe -w 0 new-tab -d ${cwd} cmd.exe", WINDOWS, "windows-terminal", "wt", false),
    p("wt-wsl", "Windows Terminal: WSL (tab)", "terminal", "wt.exe -w 0 new-tab -d ${cwd} wsl.exe", WINDOWS, "windows-terminal", "wt", false),
    p("pwsh-window", "PowerShell 7 window", "terminal", "cmd /C start \"\" /D ${cwd} pwsh.exe", WINDOWS, "", "pwsh", false),
    p("powershell-window", "Windows PowerShell window", "terminal", "cmd /C start \"\" /D ${cwd} powershell.exe", WINDOWS, "", "powershell", false),
    p("cmd-window", "Command Prompt window", "terminal", "cmd /C start \"\" /D ${cwd} cmd.exe", WINDOWS, "", "", false),
    p("wsl-window", "WSL window", "terminal", "cmd /C start \"\" wsl.exe --cd ${cwd}", WINDOWS, "", "wsl", false),
    p("git-bash", "Git Bash", "terminal", "git-bash.exe --cd=${cwd}", WINDOWS, "", "git-bash", false),
    p("warp-windows", "Warp: new tab", "terminal", "cmd /C start \"\" warp://action/new_tab?path=${cwd}", WINDOWS, "warp", "", false),
    p("warp-macos", "Warp: new tab", "terminal", "open warp://action/new_tab?path=${cwd}", MACOS, "warp", "", false),
    p("warp-linux", "Warp: new tab", "terminal", "xdg-open warp://action/new_tab?path=${cwd}", LINUX, "warp", "", false),
    // Editors and IDEs. `zeditor` is Zed's name on Arch.
    p("code", "VS Code", "editor", "code ${cwd}", ALL, "code", "code", true),
    p("vscodium", "VSCodium", "editor", "codium ${cwd}", ALL, "", "codium", false),
    p("cursor", "Cursor", "editor", "cursor ${cwd}", ALL, "cursor", "cursor", false),
    p("windsurf", "Windsurf", "editor", "windsurf ${cwd}", ALL, "", "windsurf", false),
    p("zed", "Zed", "editor", "zed ${cwd}", UNIX, "zed", "zed", false),
    p("zeditor", "Zed", "editor", "zeditor ${cwd}", LINUX, "zeditor", "zeditor", false),
    p("sublime", "Sublime Text", "editor", "subl ${cwd}", ALL, "", "subl", false),
    p("emacs", "Emacs", "editor", "emacs ${cwd}", ALL, "", "emacs", false),
    p("idea", "IntelliJ IDEA", "editor", "idea ${cwd}", ALL, "", "idea", false),
    p("rustrover", "RustRover", "editor", "rustrover ${cwd}", ALL, "", "rustrover", false),
    p("pycharm", "PyCharm", "editor", "pycharm ${cwd}", ALL, "", "pycharm", false),
    p("webstorm", "WebStorm", "editor", "webstorm ${cwd}", ALL, "", "webstorm", false),
    p("clion", "CLion", "editor", "clion ${cwd}", ALL, "", "clion", false),
    p("goland", "GoLand", "editor", "goland ${cwd}", ALL, "", "goland", false),
    p("rider", "Rider", "editor", "rider ${cwd}", ALL, "", "rider", false),
    p("phpstorm", "PhpStorm", "editor", "phpstorm ${cwd}", ALL, "", "phpstorm", false),
    p("studio", "Android Studio", "editor", "studio ${cwd}", ALL, "", "studio", false),
    p("devenv", "Visual Studio", "editor", "devenv ${cwd}", WINDOWS, "", "devenv", false),
    // Git tools with their own window.
    p("gitk", "gitk", "git", "gitk --all", UNIX, "", "gitk", false),
    p("git-gui", "git gui", "git", "git gui", ALL, "", "git", false),
    p("gitg", "gitg", "git", "gitg ${cwd}", LINUX, "", "gitg", false),
    p("smerge", "Sublime Merge", "git", "smerge ${cwd}", ALL, "", "smerge", false),
    p("gitkraken", "GitKraken", "git", "gitkraken --path ${cwd}", ALL, "", "gitkraken", false),
    p("github-desktop", "GitHub Desktop", "git", "github ${cwd}", ALL, "", "github", false),
    // The repository's page (needs a GitHub / GitLab origin for ${web_url}).
    p("web-xdg", "Repository page on the web", "web", "xdg-open ${web_url}", LINUX, "", "xdg-open", false),
    p("web-macos", "Repository page on the web", "web", "open ${web_url}", MACOS, "", "", false),
    p("web-windows", "Repository page on the web", "web", "rundll32 url.dll,FileProtocolHandler ${web_url}", WINDOWS, "", "", false),
    p("gh-web", "GitHub: open repository (gh)", "web", "gh repo view --web", ALL, "", "gh", true),
    p("glab-web", "GitLab: open repository (glab)", "web", "glab repo view --web", ALL, "", "glab", false),
];

/// A program run inside a pane, tab or terminal.
struct Tool {
    id: &'static str,
    label: &'static str,
    group: &'static str,
    /// What the pane's shell runs; starts in the folder.
    run: &'static str,
    /// The program to look for (`""`: always offered).
    bin: &'static str,
    /// Herdr's `--kind` when it can start the tool as an agent.
    agent_kind: &'static str,
}

const fn tool(
    id: &'static str,
    label: &'static str,
    group: &'static str,
    run: &'static str,
    bin: &'static str,
    agent_kind: &'static str,
) -> Tool {
    Tool {
        id,
        label,
        group,
        run,
        bin,
        agent_kind,
    }
}

#[rustfmt::skip]
const TOOLS: &[Tool] = &[
    tool("lazygit", "lazygit", "git", "lazygit", "lazygit", ""),
    tool("tig", "tig", "git", "tig", "tig", ""),
    tool("gitui", "gitui", "git", "gitui", "gitui", ""),
    tool("yazi", "yazi", "filemanager", "yazi", "yazi", ""),
    tool("superfile", "superfile", "filemanager", "spf .", "spf", ""),
    tool("nnn", "nnn", "filemanager", "nnn", "nnn", ""),
    tool("lf", "lf", "filemanager", "lf", "lf", ""),
    tool("ranger", "ranger", "filemanager", "ranger", "ranger", ""),
    tool("nvim", "Neovim", "editor", "nvim .", "nvim", ""),
    tool("vim", "Vim", "editor", "vim .", "vim", ""),
    tool("helix", "Helix", "editor", "hx .", "hx", ""),
    tool("editor", "$EDITOR", "editor", "${env:EDITOR}", "", ""),
    tool("claude", "Claude Code", "agent", "claude", "claude", "claude"),
    tool("codex", "Codex", "agent", "codex", "codex", "codex"),
    tool("gemini", "Gemini CLI", "agent", "gemini", "gemini", "gemini"),
    tool("opencode", "opencode", "agent", "opencode", "opencode", "opencode"),
];

/// Where a tool can be launched.
struct Place {
    id: &'static str,
    label: &'static str,
    platforms: &'static [&'static str],
    host: &'static str,
    bin: &'static str,
    /// The command takes a whole shell command line as one argument (so
    /// `$EDITOR` with its own flags works).
    shell: bool,
    /// `herdr`: the new tab / pane is created, then the tool typed into it.
    herdr: Option<&'static str>,
    /// Command words before the tool.
    prefix: &'static str,
    verified: bool,
}

#[rustfmt::skip]
const PLACES: &[Place] = &[
    Place { id: "herdr-tab", label: "Herdr tab", platforms: UNIX, host: "herdr", bin: "herdr", shell: true, herdr: Some("tab"), prefix: "", verified: true },
    Place { id: "herdr-split", label: "Herdr split", platforms: UNIX, host: "herdr", bin: "herdr", shell: true, herdr: Some("split"), prefix: "", verified: true },
    Place { id: "tmux-window", label: "tmux window", platforms: UNIX, host: "tmux", bin: "tmux", shell: true, herdr: None, prefix: "tmux new-window -c ${cwd}", verified: true },
    Place { id: "tmux-split", label: "tmux split", platforms: UNIX, host: "tmux", bin: "tmux", shell: true, herdr: None, prefix: "tmux split-window -h -c ${cwd}", verified: true },
    Place { id: "zellij-pane", label: "zellij pane", platforms: UNIX, host: "zellij", bin: "zellij", shell: false, herdr: None, prefix: "zellij run --cwd ${cwd} --", verified: false },
    Place { id: "zellij-floating", label: "zellij floating pane", platforms: UNIX, host: "zellij", bin: "zellij", shell: false, herdr: None, prefix: "zellij run --floating --cwd ${cwd} --", verified: false },
    Place { id: "wezterm-tab", label: "WezTerm tab", platforms: ALL, host: "wezterm", bin: "wezterm", shell: false, herdr: None, prefix: "wezterm cli spawn --cwd ${cwd} --", verified: false },
    Place { id: "wezterm-split", label: "WezTerm split", platforms: ALL, host: "wezterm", bin: "wezterm", shell: false, herdr: None, prefix: "wezterm cli split-pane --right --cwd ${cwd} --", verified: false },
    Place { id: "kitty-tab", label: "kitty tab", platforms: UNIX, host: "kitty", bin: "kitten", shell: false, herdr: None, prefix: "kitten @ launch --type=tab --cwd=${cwd}", verified: true },
    Place { id: "kitty-oswindow", label: "kitty window", platforms: UNIX, host: "kitty", bin: "kitten", shell: false, herdr: None, prefix: "kitten @ launch --type=os-window --cwd=${cwd}", verified: true },
    Place { id: "xdg-terminal", label: "new terminal", platforms: LINUX, host: "", bin: "xdg-terminal-exec", shell: false, herdr: None, prefix: "xdg-terminal-exec --dir=${cwd} --", verified: true },
    Place { id: "wt-window", label: "Windows Terminal window", platforms: WINDOWS, host: "windows-terminal", bin: "wt", shell: false, herdr: None, prefix: "wt.exe -d ${cwd}", verified: false },
    Place { id: "wt-tab", label: "Windows Terminal tab", platforms: WINDOWS, host: "windows-terminal", bin: "wt", shell: false, herdr: None, prefix: "wt.exe -w 0 new-tab -d ${cwd}", verified: false },
];

/// `herdr` creates the tab or split, prints JSON naming its first pane; the
/// pane id is cut out of it and the tool typed into that pane. `$1` is the
/// workspace (a new tab lands in the focused one otherwise), `$2` the folder,
/// `$3` the command line.
macro_rules! herdr_pane {
    ($create:literal, $then:literal) => {
        concat!(
            r#"sh -c 'p=$("#,
            $create,
            r#" | sed -n "s/.*\"pane_id\":\"\([^\"]*\)\".*/\1/p") && "#,
            $then,
            r#"' sh ${env:HERDR_WORKSPACE_ID} ${cwd} "#
        )
    };
}
const HERDR_TAB_RUN: &str = herdr_pane!(
    r#"herdr tab create --workspace "$1" --cwd "$2" --focus"#,
    r#"herdr pane run "$p" "$3""#
);
const HERDR_SPLIT_RUN: &str = herdr_pane!(
    r#"herdr pane split --current --direction right --cwd "$2" --focus"#,
    r#"herdr pane run "$p" "$3""#
);
const HERDR_TAB_EDITOR: &str = herdr_pane!(
    r#"herdr tab create --workspace "$1" --cwd "$2" --focus"#,
    r#"{ t=$3; [ -n "$t" ] || t=vi; herdr pane run "$p" "$t"; }"#
);
const HERDR_SPLIT_EDITOR: &str = herdr_pane!(
    r#"herdr pane split --current --direction right --cwd "$2" --focus"#,
    r#"{ t=$3; [ -n "$t" ] || t=vi; herdr pane run "$p" "$t"; }"#
);
const HERDR_TAB_AGENT: &str = herdr_pane!(
    r#"herdr tab create --workspace "$1" --cwd "$2" --focus"#,
    r#"herdr agent start "$3-$$" --kind "$3" --pane "$p""#
);
const HERDR_SPLIT_AGENT: &str = herdr_pane!(
    r#"herdr pane split --current --direction right --cwd "$2" --focus"#,
    r#"herdr agent start "$3-$$" --kind "$3" --pane "$p""#
);

fn leak(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn variants() -> Vec<OpenInPreset> {
    let mut out = Vec::new();
    for place in PLACES {
        for tool in TOOLS {
            let is_editor_var = tool.id == "editor";
            if is_editor_var && !place.shell {
                continue;
            }
            let command = match place.herdr {
                Some(kind) => {
                    let (script, arg) = if !tool.agent_kind.is_empty() {
                        (
                            if kind == "tab" {
                                HERDR_TAB_AGENT
                            } else {
                                HERDR_SPLIT_AGENT
                            },
                            tool.agent_kind,
                        )
                    } else if is_editor_var {
                        (
                            if kind == "tab" {
                                HERDR_TAB_EDITOR
                            } else {
                                HERDR_SPLIT_EDITOR
                            },
                            tool.run,
                        )
                    } else {
                        (
                            if kind == "tab" {
                                HERDR_TAB_RUN
                            } else {
                                HERDR_SPLIT_RUN
                            },
                            tool.run,
                        )
                    };
                    if arg.contains(' ') {
                        format!("{script}'{arg}'")
                    } else {
                        format!("{script}{arg}")
                    }
                }
                None => format!("{} {}", place.prefix, tool.run),
            };
            // The "$EDITOR" label reads "$EDITOR in a tmux window".
            out.push(OpenInPreset {
                id: leak(format!("{}-{}", place.id, tool.id)),
                label: leak(format!("{} in {}", tool.label, place.label)),
                group: tool.group,
                command: leak(command),
                platforms: place.platforms,
                host: place.host,
                bin: place.bin,
                tool: tool.bin,
                // Herdr's agent start was run for real with claude (it
                // starts the agent; it reports a trust prompt as not ready).
                verified: place.verified,
            });
        }
    }
    out
}

/// Every preset: [`BASE`] then the generated tool-in-a-pane variants.
pub fn presets() -> &'static [OpenInPreset] {
    static ALL_PRESETS: std::sync::OnceLock<Vec<OpenInPreset>> = std::sync::OnceLock::new();
    ALL_PRESETS.get_or_init(|| {
        let mut v = BASE.to_vec();
        v.extend(variants());
        v
    })
}

/// The preset with this id.
pub fn preset(id: &str) -> Option<&'static OpenInPreset> {
    presets().iter().find(|p| p.id == id)
}

/// The presets offered on `os`: the others are never listed.
pub fn presets_for(os: Os) -> Vec<&'static OpenInPreset> {
    presets()
        .iter()
        .filter(|p| p.platforms.contains(&os.name()))
        .collect()
}

impl OpenInPreset {
    pub fn entry(&self) -> OpenInEntry {
        OpenInEntry::new(self.label, self.command)
    }

    /// Whether this machine has what the preset needs: its program (and its
    /// tool) on `PATH`.
    pub fn installed(&self, d: &Detected) -> bool {
        let has = |b: &str| b.is_empty() || d.installed.contains(&b);
        has(self.bin) && has(self.tool)
    }

    /// 0: running inside its host (or needing none) with everything
    /// installed; 1: installed; 2: not found. A tool-in-a-pane variant whose
    /// tool or host is missing is not offered at all ([`Self::offered`]).
    pub fn rank(&self, d: &Detected) -> u8 {
        let inside = !self.host.is_empty()
            && (d.multiplexers.contains(&self.host) || d.terminals.contains(&self.host));
        if self.installed(d) && inside {
            0
        } else if self.installed(d) {
            1
        } else {
            2
        }
    }

    /// Hand-written presets are always offered (ranked); a generated variant
    /// only when both its host program and its tool are installed.
    pub fn offered(&self, d: &Detected) -> bool {
        self.tool.is_empty() || self.installed(d)
    }
}

/// What Add preset lists on `os`: only that OS's presets, by category in
/// [`GROUPS`] order, the ones this machine can use before the others.
pub fn picker_order(os: Os, d: &Detected) -> Vec<&'static OpenInPreset> {
    let mut out = Vec::new();
    for group in GROUPS {
        let mut in_group: Vec<_> = presets_for(os)
            .into_iter()
            .filter(|p| p.group == *group && p.offered(d))
            .collect();
        in_group.sort_by_key(|p| p.rank(d)); // stable: definition order within a rank
        out.extend(in_group);
    }
    out
}

/// "nvim hosted in" the detected multiplexer or terminal: an editor that needs
/// a terminal to draw in.
pub fn nvim_entries(detected: &Detected) -> Vec<OpenInEntry> {
    let mut out = Vec::new();
    for host in &detected.multiplexers {
        match *host {
            "tmux" => out.push(OpenInEntry::new(
                "nvim in tmux",
                "tmux new-window -c ${cwd} nvim",
            )),
            "zellij" => out.push(OpenInEntry::new(
                "nvim in zellij",
                "zellij action new-pane --cwd ${cwd} -- nvim",
            )),
            _ => {}
        }
    }
    for host in &detected.terminals {
        match *host {
            "wezterm" => out.push(OpenInEntry::new(
                "nvim in WezTerm",
                "wezterm cli spawn --cwd ${cwd} -- nvim",
            )),
            "windows-terminal" => out.push(OpenInEntry::new(
                "nvim in Windows Terminal",
                "wt.exe -d ${cwd} nvim",
            )),
            _ => {}
        }
    }
    out
}

// ----------------------------------------------------------------- detection

/// What [`detect`] found. Ids match [`OpenInPreset::host`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Detected {
    /// The multiplexers the process runs inside.
    pub multiplexers: Vec<&'static str>,
    /// The outer terminal emulator or host.
    pub terminals: Vec<&'static str>,
    /// `omarchy`, `hyprland`, `wayland`.
    pub desktop: Vec<&'static str>,
    /// Editors found on `PATH`: `code`, `cursor`, `zed`, `zeditor`, `nvim`, `vim`.
    pub editors: Vec<&'static str>,
    /// Every program a preset needs (or runs) that is on `PATH`.
    pub installed: Vec<&'static str>,
}

/// Looks at `env` and `on_path`; spawns nothing. Multiplexers are listed
/// apart from the outer terminal (herdr inside WezTerm is both).
///
/// `cmux` is recognized by `CMUX_WORKSPACE_ID` / `CMUX_SOCKET_PATH`, a guess
/// nobody has checked; psmux is not detected (it is assumed to set `TMUX`).
pub fn detect(
    env: &dyn Fn(&str) -> Option<String>,
    os: Os,
    on_path: &dyn Fn(&str) -> bool,
) -> Detected {
    let set = |k: &str| env(k).is_some_and(|v| !v.is_empty());
    let is = |k: &str, want: &str| env(k).is_some_and(|v| v == want);
    let mut d = Detected::default();

    if is("HERDR_ENV", "1") || set("HERDR_PANE_ID") {
        d.multiplexers.push("herdr");
    }
    if set("TMUX") {
        d.multiplexers.push("tmux");
    }
    if set("ZELLIJ") {
        d.multiplexers.push("zellij");
    }
    if set("CMUX_WORKSPACE_ID") || set("CMUX_SOCKET_PATH") {
        d.multiplexers.push("cmux");
    }

    let term_program = env("TERM_PROGRAM").unwrap_or_default();
    let term = env("TERM").unwrap_or_default();
    let mut term_push = |id: &'static str, hit: bool| {
        if hit && !d.terminals.contains(&id) {
            d.terminals.push(id);
        }
    };
    term_push(
        "wezterm",
        set("WEZTERM_PANE") || set("WEZTERM_UNIX_SOCKET") || term_program == "WezTerm",
    );
    term_push("kitty", set("KITTY_WINDOW_ID") || set("KITTY_LISTEN_ON"));
    term_push("windows-terminal", set("WT_SESSION"));
    term_push(
        "alacritty",
        set("ALACRITTY_WINDOW_ID") || set("ALACRITTY_SOCKET"),
    );
    term_push(
        "ghostty",
        set("GHOSTTY_RESOURCES_DIR") || term_program == "ghostty",
    );
    term_push("foot", term.starts_with("foot"));
    term_push("konsole", set("KONSOLE_VERSION"));
    term_push("gnome-terminal", set("GNOME_TERMINAL_SCREEN"));
    term_push("warp", term_program == "WarpTerminal");
    term_push("vscode", term_program == "vscode");
    term_push("iterm", term_program == "iTerm.app");
    term_push("apple-terminal", term_program == "Apple_Terminal");

    if os == Os::Linux {
        // Omarchy ships kitty as its terminal; its scripts sit on PATH.
        if set("OMARCHY_PATH") || on_path("omarchy-menu") {
            d.desktop.push("omarchy");
        }
        if set("HYPRLAND_INSTANCE_SIGNATURE") {
            d.desktop.push("hyprland");
        }
        if set("WAYLAND_DISPLAY") {
            d.desktop.push("wayland");
        }
    }

    for editor in ["code", "cursor", "zed", "zeditor", "nvim", "vim"] {
        if on_path(editor) {
            d.editors.push(editor);
        }
    }
    for p in presets() {
        for bin in [p.bin, p.tool] {
            if !bin.is_empty() && !d.installed.contains(&bin) && on_path(bin) {
                d.installed.push(bin);
            }
        }
    }
    d
}

// ---------------------------------------------------------------- migration

/// The `terminal_launch` command with `{repo}` spelled `${cwd}`.
fn to_cwd(template: &str) -> String {
    template.replace("{repo}", "${cwd}")
}

/// The entry the legacy `ui.terminal_launch*` keys stand for: `custom` ->
/// "Terminal (custom)", a preset id -> that preset, `auto` -> "Default
/// terminal" (the `ui.launch` preset's shell form when one is set, else the
/// platform's own terminal). `Ok(None)` for an unusable `custom` (empty
/// command); an unknown preset id is an error.
pub fn migrate_terminal(
    terminal_launch: &str,
    terminal_custom: &str,
    ui_launch: &str,
    ui_launch_custom: &str,
    os: Os,
) -> Result<Option<OpenInEntry>, String> {
    match terminal_launch.trim() {
        "" | launch::TERMINAL_AUTO => {
            let from_launch = launch::resolve_template(ui_launch, ui_launch_custom)
                .ok()
                .flatten()
                .and_then(|t| launch::shell_template_from(&t).ok());
            let command = from_launch.map(|t| to_cwd(&t)).unwrap_or_else(|| {
                match os {
                    Os::Windows => "wt.exe -d ${cwd}",
                    Os::Macos => "open -a Terminal ${cwd}",
                    Os::Linux => "xdg-terminal-exec --dir=${cwd}",
                }
                .to_string()
            });
            Ok(Some(OpenInEntry::new("Default terminal", command)))
        }
        launch::LAUNCH_CUSTOM => {
            if terminal_custom.trim().is_empty() {
                Ok(None)
            } else {
                Ok(Some(OpenInEntry::new(
                    "Terminal (custom)",
                    to_cwd(terminal_custom),
                )))
            }
        }
        id => {
            let template = launch::terminal_template(id)
                .ok_or_else(|| format!("ui.terminal_launch = \"{id}\" is not a preset id"))?;
            let label = launch::preset(id).map_or(id, |p| p.label);
            let command = to_cwd(&template);
            if !command.contains("${cwd}") {
                // `xdg-terminal-exec` takes the folder from its own flag.
                if let Some(p) = preset(id) {
                    return Ok(Some(p.entry()));
                }
            }
            Ok(Some(OpenInEntry::new(label, command)))
        }
    }
}

/// The list that applies while `ui.open_in` is absent: the legacy terminal
/// entry (give it only when `terminal_launch` is not `auto`), the OS explorer,
/// the detected multiplexers' entries, the detected terminals' entries, then
/// editors on `PATH` (with nvim hosted in what was detected). Labels are
/// unique.
pub fn effective_list(
    legacy_terminal: Option<OpenInEntry>,
    detected: &Detected,
    os: Os,
) -> Vec<OpenInEntry> {
    let mut out: Vec<OpenInEntry> = Vec::new();
    let mut add = |e: OpenInEntry| {
        if !out.iter().any(|x| x.label == e.label) {
            out.push(e);
        }
    };
    if let Some(e) = legacy_terminal {
        add(e);
    }
    let offered = presets_for(os);
    // Inside herdr, its "new tab here" leads (so it is the default); on
    // Omarchy, whose terminal is kitty, kitty's "new window here" follows it
    // (and leads when not in herdr), even from another terminal.
    if detected.multiplexers.contains(&"herdr") {
        if let Some(p) = offered.iter().find(|p| p.id == "herdr-tab") {
            add(p.entry());
        }
    }
    if detected.desktop.contains(&"omarchy") {
        if let Some(p) = offered.iter().find(|p| p.id == "kitty") {
            add(p.entry());
        }
    }
    // Only the OS's own file manager: the other presets are picked by hand.
    for p in offered.iter().filter(|p| p.group == "explorer") {
        add(p.entry());
    }
    for p in offered.iter().filter(|p| {
        p.group == "multiplexer"
            && DEFAULT_MULTIPLEXER.contains(&p.id)
            && (detected.multiplexers.contains(&p.host) || detected.terminals.contains(&p.host))
    }) {
        add(p.entry());
    }
    for p in offered.iter().filter(|p| {
        p.group == "terminal" && !p.host.is_empty() && detected.terminals.contains(&p.host)
    }) {
        add(p.entry());
    }
    for p in offered
        .iter()
        .filter(|p| p.group == "editor" && detected.editors.contains(&p.host))
    {
        add(p.entry());
    }
    if detected.editors.contains(&"nvim") {
        for e in nvim_entries(detected) {
            add(e);
        }
    }
    out
}

/// The multiplexer presets the computed list offers; the rest are for Add preset.
const DEFAULT_MULTIPLEXER: &[&str] = &[
    "herdr-tab",
    "herdr-workspace",
    "herdr-split",
    "tmux-window",
    "tmux-split",
    "zellij-pane",
    "zellij-tab",
    "wezterm-tab",
    "wezterm-split",
    "cmux-workspace",
    "kitty-tab",
];

/// The index of the default entry: the one labeled `default_label`, else the
/// first (an empty or stale label falls back quietly).
pub fn default_index(entries: &[OpenInEntry], default_label: &str) -> usize {
    entries
        .iter()
        .position(|e| e.label == default_label)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    const VARS: OpenInVars = OpenInVars {
        cwd: "/home/u/my repo",
        repo: "/home/u/root",
        name: "my repo",
        branch: "feat/x",
        web_url: "https://github.com/o/r",
    };

    fn no_env(_: &str) -> Option<String> {
        None
    }

    #[test]
    fn expansion_keeps_values_with_spaces_as_one_argument() {
        let argv = expand(
            "tool --dir ${cwd} --label ${name} -b ${branch} ${repo}",
            &VARS,
            &no_env,
        )
        .unwrap();
        assert_eq!(
            argv,
            [
                "tool",
                "--dir",
                "/home/u/my repo",
                "--label",
                "my repo",
                "-b",
                "feat/x",
                "/home/u/root"
            ]
        );
    }

    #[test]
    fn expansion_handles_quotes_env_legacy_and_literal_dollars() {
        let env = |k: &str| (k == "EDITOR").then(|| "vi m".to_string());
        assert_eq!(
            expand(
                "sh -c 'cd \"$GITPOULPI_CWD\" && ${env:EDITOR}'",
                &VARS,
                &env
            )
            .unwrap(),
            ["sh", "-c", "cd \"$GITPOULPI_CWD\" && vi m"]
        );
        assert_eq!(
            expand("a x=${env:MISSING}.", &VARS, &env).unwrap(),
            ["a", "x=."]
        );
        assert_eq!(
            expand("my-term --cd {repo} {other} $5", &VARS, &no_env).unwrap(),
            ["my-term", "--cd", "/home/u/my repo", "{other}", "$5"]
        );
        assert_eq!(
            expand("x \"${cwd}/sub dir\"", &VARS, &no_env).unwrap(),
            ["x", "/home/u/my repo/sub dir"]
        );
        // Substituted values are not re-scanned.
        let v = OpenInVars {
            cwd: "${name}",
            ..VARS
        };
        assert_eq!(expand("x ${cwd}", &v, &no_env).unwrap(), ["x", "${name}"]);
    }

    #[test]
    fn expansion_errors_name_the_problem() {
        let err = expand("x ${nope}", &VARS, &no_env).unwrap_err();
        assert!(err.contains("${nope}"), "{err}");
        assert!(expand("x ${env:}", &VARS, &no_env).is_err());
        assert!(expand("x ${cwd", &VARS, &no_env).is_err());
        assert!(expand("x 'unterminated", &VARS, &no_env).is_err());
        assert!(expand("   ", &VARS, &no_env).is_err());
        assert!(validate("tool ${cwd} ${env:HOME}").is_ok());
        assert!(validate("tool ${bogus}").is_err());
    }

    fn env_of(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |k| map.get(k).cloned()
    }

    #[test]
    fn detect_table() {
        let none = |_: &str| false;
        // herdr running inside WezTerm: multiplexer and terminal both found.
        let d = detect(
            &env_of(&[
                ("HERDR_ENV", "1"),
                ("HERDR_PANE_ID", "p1"),
                ("WEZTERM_PANE", "3"),
            ]),
            Os::Windows,
            &none,
        );
        assert_eq!(d.multiplexers, ["herdr"]);
        assert_eq!(d.terminals, ["wezterm"]);
        // tmux inside kitty, with editors on PATH.
        let d = detect(
            &env_of(&[
                ("TMUX", "/tmp/tmux-1000/default,1,0"),
                ("KITTY_WINDOW_ID", "1"),
            ]),
            Os::Linux,
            &|p| p == "code" || p == "nvim",
        );
        assert_eq!(d.multiplexers, ["tmux"]);
        assert_eq!(d.terminals, ["kitty"]);
        assert_eq!(d.editors, ["code", "nvim"]);
        // Windows Terminal alone.
        let d = detect(&env_of(&[("WT_SESSION", "abc")]), Os::Windows, &none);
        assert!(d.multiplexers.is_empty());
        assert_eq!(d.terminals, ["windows-terminal"]);
        // TERM_PROGRAM and TERM spellings; an empty variable is unset.
        let d = detect(
            &env_of(&[("TERM_PROGRAM", "ghostty"), ("TERM", "foot"), ("TMUX", "")]),
            Os::Linux,
            &none,
        );
        assert_eq!(d.terminals, ["ghostty", "foot"]);
        assert!(d.multiplexers.is_empty());
        // Hyprland only counts on Linux.
        let hypr = env_of(&[
            ("HYPRLAND_INSTANCE_SIGNATURE", "x"),
            ("WAYLAND_DISPLAY", "w-0"),
        ]);
        assert_eq!(
            detect(&hypr, Os::Linux, &none).desktop,
            ["hyprland", "wayland"]
        );
        assert!(detect(&hypr, Os::Macos, &none).desktop.is_empty());
        // Nothing at all.
        assert_eq!(detect(&no_env, Os::Linux, &none), Detected::default());
    }

    /// Runs the tmux presets against a private tmux server (own socket, no
    /// config): skipped when tmux is missing.
    #[cfg(unix)]
    #[test]
    fn tmux_presets_open_in_the_folder_on_a_private_server() {
        use std::process::{Command, Stdio};
        if Command::new("tmux").arg("-V").output().is_err() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let folder = std::fs::canonicalize(dir.path()).unwrap();
        let start = tempfile::tempdir().unwrap();
        let sock = format!("gp-test-{}", std::process::id());
        let tmux = |args: &[&str]| {
            Command::new("tmux")
                .args(["-L", &sock, "-f", "/dev/null"])
                .args(args)
                .env_remove("TMUX")
                .stderr(Stdio::null())
                .output()
                .unwrap()
        };
        let ok = tmux(&[
            "new-session",
            "-d",
            "-s",
            "t",
            "-c",
            start.path().to_str().unwrap(),
        ]);
        if !ok.status.success() {
            return;
        }
        let server =
            String::from_utf8(tmux(&["display-message", "-p", "#{socket_path},#{pid},0"]).stdout)
                .unwrap();
        let tmux_env = server.trim().to_string();
        for id in ["tmux-window", "tmux-split"] {
            let entry = preset(id).unwrap().entry();
            let vars = OpenInVars {
                cwd: folder.to_str().unwrap(),
                repo: folder.to_str().unwrap(),
                name: "x",
                branch: "",
                web_url: "",
            };
            let argv = expand(&entry.command, &vars, &no_env).unwrap();
            let mut cmd = Command::new(&argv[0]);
            // Targets the private server through $TMUX, as inside a pane.
            let st = cmd
                .args(&argv[1..])
                .env("TMUX", &tmux_env)
                .stderr(Stdio::null())
                .status()
                .unwrap();
            assert!(st.success(), "{id} failed");
        }
        let paths =
            String::from_utf8(tmux(&["list-panes", "-a", "-F", "#{pane_current_path}"]).stdout)
                .unwrap();
        tmux(&["kill-server"]);
        let hits = paths
            .lines()
            .filter(|l| std::fs::canonicalize(l).ok().as_deref() == Some(folder.as_path()))
            .count();
        assert_eq!(hits, 2, "{paths}");
    }

    #[test]
    fn omarchy_puts_kitty_first_and_hyprland_follows() {
        let hypr = env_of(&[("HYPRLAND_INSTANCE_SIGNATURE", "x")]);
        let d = detect(&hypr, Os::Linux, &|p| p == "omarchy-menu");
        assert_eq!(d.desktop, ["omarchy", "hyprland"]);
        let d2 = detect(
            &env_of(&[("OMARCHY_PATH", "/usr/share/omarchy")]),
            Os::Linux,
            &|_| false,
        );
        assert_eq!(d2.desktop, ["omarchy"]);
        // Even inside another terminal, kitty is the default entry.
        let d3 = Detected {
            terminals: vec!["alacritty"],
            ..d.clone()
        };
        let list = effective_list(None, &d3, Os::Linux);
        assert_eq!(list[0].label, "kitty");
        assert_eq!(list[0].command, "kitty --directory ${cwd}");
        assert_eq!(default_index(&list, ""), 0);
        // Inside herdr (on Omarchy) herdr leads, kitty is second.
        let d4 = Detected {
            multiplexers: vec!["herdr"],
            ..d3.clone()
        };
        let list = effective_list(None, &d4, Os::Linux);
        assert_eq!(list[0].label, "Herdr: new tab");
        assert_eq!(list[1].label, "kitty");
        // Off Omarchy the file manager leads, as before.
        let plain = effective_list(None, &Detected::default(), Os::Linux);
        assert_eq!(plain[0].label, "File manager");
        assert!(!detect(&hypr, Os::Macos, &|p| p == "omarchy-menu")
            .desktop
            .contains(&"omarchy"));
    }

    #[test]
    fn every_preset_parses_and_names_the_folder() {
        let mut ids = std::collections::HashSet::new();
        // Run in the tab's folder without naming it, or naming the web page.
        let implicit = [
            "gitk",
            "git-gui",
            "gh-web",
            "glab-web",
            "web-xdg",
            "web-macos",
        ];
        for p in presets() {
            assert!(ids.insert(p.id), "duplicate preset id {}", p.id);
            validate(p.command).unwrap_or_else(|e| panic!("{}: {e}", p.id));
            assert!(
                p.command.contains("${cwd}")
                    || p.command.contains("${env:")
                    || p.command.contains("${web_url}")
                    || implicit.contains(&p.id),
                "{} does not use the folder",
                p.id
            );
            assert!(!p.platforms.is_empty(), "{} is offered nowhere", p.id);
            assert!(GROUPS.contains(&p.group), "{}", p.id);
            assert!(!p.label.is_empty());
        }
        for e in nvim_entries(&Detected {
            multiplexers: vec!["tmux", "zellij"],
            terminals: vec!["wezterm", "windows-terminal"],
            ..Detected::default()
        }) {
            validate(&e.command).unwrap();
            assert!(e.command.contains("${cwd}"));
        }
        // Each OS gets an explorer.
        for os in [Os::Windows, Os::Macos, Os::Linux] {
            assert!(presets_for(os).iter().any(|p| p.group == "explorer"));
        }
        // The verified ones are the plan's.
        assert!(preset("herdr-tab").unwrap().verified);
        assert!(!preset("zellij-pane").unwrap().verified);
        // Labels of the defaults stay what saved configs may name.
        assert_eq!(preset("herdr-split").unwrap().label, "Herdr: split pane");
    }

    fn all_installed() -> Detected {
        let mut d = Detected::default();
        for p in presets() {
            for b in [p.bin, p.tool] {
                if !b.is_empty() && !d.installed.contains(&b) {
                    d.installed.push(b);
                }
            }
        }
        d
    }

    #[test]
    fn only_the_current_os_is_ever_listed() {
        for os in [Os::Windows, Os::Macos, Os::Linux] {
            let name = os.name();
            let listed = picker_order(os, &all_installed());
            assert!(!listed.is_empty());
            for p in &listed {
                assert!(p.platforms.contains(&name), "{} on {name}", p.id);
            }
            for p in presets_for(os) {
                assert!(p.platforms.contains(&name));
            }
            // The computed defaults too, whatever was detected.
            let d = Detected {
                multiplexers: vec!["herdr", "tmux", "zellij"],
                terminals: vec!["kitty", "windows-terminal", "iterm", "wezterm"],
                editors: vec!["code", "zed", "zeditor"],
                ..all_installed()
            };
            for e in effective_list(None, &d, os) {
                let hit = presets()
                    .iter()
                    .any(|p| p.label == e.label && p.platforms.contains(&name));
                let nvim = e.label.starts_with("nvim in ");
                assert!(hit || nvim, "{} on {name}", e.label);
            }
        }
        let on_linux: Vec<_> = picker_order(Os::Linux, &all_installed());
        assert!(on_linux.iter().all(|p| !p.id.starts_with("wt-")));
        assert!(picker_order(Os::Windows, &all_installed())
            .iter()
            .all(|p| p.id != "explorer-linux" && p.id != "nautilus"));
    }

    #[test]
    fn the_picker_groups_by_category_and_ranks_what_is_installed() {
        let d = Detected {
            multiplexers: vec!["herdr"],
            installed: vec!["herdr", "lazygit", "tmux", "yazi", "gh", "xdg-open"],
            ..Detected::default()
        };
        let list = picker_order(Os::Linux, &d);
        // Categories come in GROUPS order, never interleaved.
        let mut last = 0;
        for p in &list {
            let at = GROUPS.iter().position(|g| *g == p.group).unwrap();
            assert!(at >= last, "{} breaks the grouping", p.id);
            last = at;
        }
        let pos = |id: &str| list.iter().position(|p| p.id == id);
        // Inside herdr with lazygit: its herdr variants lead the tmux ones.
        assert!(pos("herdr-tab-lazygit").unwrap() < pos("tmux-window-lazygit").unwrap());
        // Installed hand-written presets precede the ones not found.
        assert!(pos("herdr-tab").unwrap() < pos("zellij-pane").unwrap());
        assert!(pos("tmux-window").unwrap() < pos("zellij-pane").unwrap());
        // Variants of an absent tool or an absent host are not offered.
        assert!(pos("herdr-tab-tig").is_none());
        assert!(pos("zellij-pane-lazygit").is_none());
        assert!(pos("tmux-window-claude").is_none());
        // $EDITOR needs no installed tool.
        assert!(pos("herdr-tab-editor").is_some());
        // Categories present.
        for g in [
            "explorer",
            "multiplexer",
            "terminal",
            "editor",
            "git",
            "web",
        ] {
            assert!(list.iter().any(|p| p.group == g), "no {g}");
        }
        assert!(pos("herdr-tab-yazi").is_some());
        assert!(list.iter().any(|p| p.group == "filemanager"));
    }

    #[test]
    fn herdr_chains_keep_the_workspace_and_expand_to_one_script() {
        let env = env_of(&[("HERDR_WORKSPACE_ID", "w3"), ("EDITOR", "my ed --x")]);
        let vars = OpenInVars {
            cwd: "/home/u/my repo",
            ..VARS
        };
        let plain = preset("herdr-tab").unwrap();
        assert!(plain
            .command
            .contains("--workspace ${env:HERDR_WORKSPACE_ID}"));
        for (id, last) in [
            ("herdr-tab-lazygit", "lazygit"),
            ("herdr-split-lazygit", "lazygit"),
            ("herdr-tab-superfile", "spf ."),
            ("herdr-tab-editor", "my ed --x"),
            ("herdr-tab-claude", "claude"),
        ] {
            let argv = expand(preset(id).unwrap().command, &vars, &env).unwrap();
            assert_eq!(&argv[..2], ["sh", "-c"], "{id}");
            assert_eq!(argv[3], "sh");
            assert_eq!(argv[4], "w3", "{id}");
            assert_eq!(argv[5], "/home/u/my repo", "{id}");
            assert_eq!(argv[6], last, "{id}");
            assert_eq!(argv.len(), 7);
            assert!(argv[2].contains("herdr"), "{id}");
        }
        let agent = expand(preset("herdr-tab-claude").unwrap().command, &vars, &env).unwrap();
        assert!(agent[2].contains("agent start"));
        assert!(agent[2].contains("--kind \"$3\""));
        // Nothing but the tab chain differs between tab and split.
        let split = expand(preset("herdr-split-lazygit").unwrap().command, &vars, &env).unwrap();
        assert!(split[2].contains("pane split --current"));
        // Plain multiplexer variants.
        let t = expand(preset("tmux-window-lazygit").unwrap().command, &vars, &env).unwrap();
        assert_eq!(
            t,
            ["tmux", "new-window", "-c", "/home/u/my repo", "lazygit"]
        );
        let z = expand(preset("zellij-pane-yazi").unwrap().command, &vars, &env).unwrap();
        assert_eq!(
            z,
            ["zellij", "run", "--cwd", "/home/u/my repo", "--", "yazi"]
        );
        let k = expand(preset("kitty-tab-lazygit").unwrap().command, &vars, &env).unwrap();
        assert_eq!(
            k,
            [
                "kitten",
                "@",
                "launch",
                "--type=tab",
                "--cwd=/home/u/my repo",
                "lazygit"
            ]
        );
        // web_url
        let w = expand(preset("web-xdg").unwrap().command, &vars, &env).unwrap();
        assert_eq!(w, ["xdg-open", "https://github.com/o/r"]);
        // The tools that need a shell line are not offered where there is none.
        assert!(preset("zellij-pane-editor").is_none());
    }

    /// A stub program `name` that prints a marker and the folder it runs in.
    #[cfg(unix)]
    fn stub(dir: &std::path::Path, name: &str) {
        use std::os::unix::fs::PermissionsExt;
        let path = dir.join(name);
        std::fs::write(
            &path,
            format!("#!/bin/sh\necho STUB-RAN-{name}:$PWD\nsleep 20\n"),
        )
        .unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    /// The tmux "run a tool in a window / split" variants against a private
    /// server, the tool being a stub on `PATH`.
    #[cfg(unix)]
    #[test]
    fn tmux_tool_variants_run_the_tool_in_the_folder() {
        use std::process::{Command, Stdio};
        if Command::new("tmux").arg("-V").output().is_err() {
            return;
        }
        let bin = tempfile::tempdir().unwrap();
        stub(bin.path(), "lazygit");
        let path = format!(
            "{}:{}",
            bin.path().display(),
            std::env::var("PATH").unwrap_or_default()
        );
        let dir = tempfile::tempdir().unwrap();
        let folder = std::fs::canonicalize(dir.path()).unwrap();
        let sock = format!("gp-test-tool-{}", std::process::id());
        let tmux = |args: &[&str]| {
            Command::new("tmux")
                .args(["-L", &sock, "-f", "/dev/null"])
                .args(args)
                .env_remove("TMUX")
                .env("PATH", &path)
                .stderr(Stdio::null())
                .output()
                .unwrap()
        };
        if !tmux(&["new-session", "-d", "-s", "t"]).status.success() {
            return;
        }
        let server =
            String::from_utf8(tmux(&["display-message", "-p", "#{socket_path},#{pid},0"]).stdout)
                .unwrap();
        let vars = OpenInVars {
            cwd: folder.to_str().unwrap(),
            repo: folder.to_str().unwrap(),
            name: "x",
            branch: "",
            web_url: "",
        };
        for id in ["tmux-window-lazygit", "tmux-split-lazygit"] {
            let argv = expand(preset(id).unwrap().command, &vars, &no_env).unwrap();
            let st = Command::new(&argv[0])
                .args(&argv[1..])
                .env("TMUX", server.trim())
                .env("PATH", &path)
                .stderr(Stdio::null())
                .status()
                .unwrap();
            assert!(st.success(), "{id} failed");
        }
        std::thread::sleep(std::time::Duration::from_millis(600));
        let cmds = String::from_utf8(
            tmux(&[
                "list-panes",
                "-a",
                "-F",
                "#{pane_start_command}|#{pane_current_path}",
            ])
            .stdout,
        )
        .unwrap();
        // The new-session preset, too.
        let argv = expand(preset("tmux-session").unwrap().command, &vars, &no_env).unwrap();
        let st = Command::new(&argv[0])
            .args(&argv[1..])
            .env("TMUX", server.trim())
            .stderr(Stdio::null())
            .status()
            .unwrap();
        let sessions =
            String::from_utf8(tmux(&["list-sessions", "-F", "#{session_name}"]).stdout).unwrap();
        tmux(&["kill-server"]);
        assert!(st.success() && sessions.contains('x'), "{sessions}");
        let hits = cmds
            .lines()
            .filter(|l| {
                let (cmd, p) = l.split_once('|').unwrap_or_default();
                cmd == "lazygit"
                    && std::fs::canonicalize(p).ok().as_deref() == Some(folder.as_path())
            })
            .count();
        assert_eq!(hits, 2, "{cmds}");
    }

    /// The herdr tool variants against a private named session (own server
    /// and socket, never the default one), the tool being a stub on `PATH`.
    #[cfg(unix)]
    #[test]
    fn herdr_tool_variants_run_the_tool_in_a_new_pane() {
        use std::process::{Command, Stdio};
        if Command::new("herdr").arg("--version").output().is_err() {
            return;
        }
        let bin = tempfile::tempdir().unwrap();
        stub(bin.path(), "lazygit");
        stub(bin.path(), "ed-stub");
        let path = format!(
            "{}:{}",
            bin.path().display(),
            std::env::var("PATH").unwrap_or_default()
        );
        let dir = tempfile::tempdir().unwrap();
        let folder = std::fs::canonicalize(dir.path()).unwrap();
        let session = format!("gp-catalog-{}", std::process::id());
        let clean = |c: &mut Command| {
            for k in [
                "HERDR_ENV",
                "HERDR_PANE_ID",
                "HERDR_TAB_ID",
                "HERDR_WORKSPACE_ID",
                "HERDR_SOCKET_PATH",
                "HERDR_SESSION",
            ] {
                c.env_remove(k);
            }
            c.env("PATH", &path);
            // Panes start `$SHELL`; a login-style rc file (the dev machine's
            // zsh/fish) would put the real tools ahead of the stubs on PATH.
            c.env("SHELL", "/bin/sh");
        };
        // The server reports its socket on startup (stdout or stderr
        // depending on the version): read both through a file.
        let log = dir.path().join("server.log");
        let log_file = std::fs::File::create(&log).unwrap();
        let mut server = Command::new("herdr");
        server
            .args(["--session", &session, "server"])
            .stdout(Stdio::from(log_file.try_clone().unwrap()))
            .stderr(Stdio::from(log_file));
        clean(&mut server);
        let Ok(mut child) = server.spawn() else {
            return;
        };
        let mut socket = None;
        for _ in 0..100 {
            let text = std::fs::read_to_string(&log).unwrap_or_default();
            if let Some(line) = text.lines().find_map(|l| l.strip_prefix("api socket: ")) {
                socket = Some(line.trim().to_string());
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        let stop = |child: &mut std::process::Child| {
            let mut c = Command::new("herdr");
            c.args(["session", "stop", &session])
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            clean(&mut c);
            let _ = c.status();
            let _ = child.kill();
            let _ = child.wait();
            let mut c = Command::new("herdr");
            c.args(["session", "delete", &session])
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            clean(&mut c);
            let _ = c.status();
        };
        let Some(socket) = socket else {
            stop(&mut child);
            return;
        };
        let herdr = |args: &[&str], extra: &[(&str, &str)]| {
            let mut c = Command::new("herdr");
            c.args(args).stderr(Stdio::null());
            clean(&mut c);
            c.env("HERDR_SOCKET_PATH", &socket);
            for (k, v) in extra {
                c.env(k, v);
            }
            String::from_utf8_lossy(&c.output().unwrap().stdout).to_string()
        };
        let field = |json: &str, key: &str| -> String {
            let pat = format!("\"{key}\":\"");
            let at = json.find(&pat).map(|i| i + pat.len()).unwrap_or(0);
            json[at..].split('"').next().unwrap_or_default().to_string()
        };
        let created = herdr(
            &[
                "workspace",
                "create",
                "--cwd",
                folder.to_str().unwrap(),
                "--label",
                "t",
            ],
            &[],
        );
        let workspace = field(&created, "workspace_id");
        let root = field(&created, "pane_id");
        let mut failure = None;
        if workspace.is_empty() || root.is_empty() {
            failure = Some(format!("no workspace: {created}"));
        }
        let editor = format!("{} --flag", bin.path().join("ed-stub").display());
        let env = |k: &str| match k {
            "HERDR_WORKSPACE_ID" => Some(workspace.clone()),
            "EDITOR" => Some(editor.clone()),
            _ => None,
        };
        let vars = OpenInVars {
            cwd: folder.to_str().unwrap(),
            repo: folder.to_str().unwrap(),
            name: "x",
            branch: "",
            web_url: "",
        };
        let mut markers = Vec::new();
        if failure.is_none() {
            for (id, marker) in [
                ("herdr-tab-lazygit", "STUB-RAN-lazygit"),
                ("herdr-split-lazygit", "STUB-RAN-lazygit"),
                ("herdr-tab-editor", "STUB-RAN-ed-stub"),
            ] {
                let argv = expand(preset(id).unwrap().command, &vars, &env).unwrap();
                let mut c = Command::new(&argv[0]);
                c.args(&argv[1..])
                    .stderr(Stdio::null())
                    .stdout(Stdio::null());
                clean(&mut c);
                c.env("HERDR_SOCKET_PATH", &socket)
                    .env("HERDR_PANE_ID", &root)
                    .env("HERDR_WORKSPACE_ID", &workspace);
                if !c.status().unwrap().success() {
                    failure = Some(format!("{id} failed"));
                    break;
                }
                markers.push((id, marker));
            }
        }
        let mut panes_text = String::new();
        if failure.is_none() {
            std::thread::sleep(std::time::Duration::from_millis(2500));
            let list = herdr(&["pane", "list"], &[]);
            for part in list.split("\"pane_id\":\"").skip(1) {
                let id = part.split('"').next().unwrap_or_default();
                panes_text.push_str(&herdr(&["pane", "read", id], &[]));
            }
        }
        stop(&mut child);
        if let Some(f) = failure {
            panic!("{f}");
        }
        let want = |m: &str| format!("{m}:{}", folder.display());
        assert_eq!(
            panes_text.matches(&want("STUB-RAN-lazygit")).count(),
            2,
            "{panes_text}"
        );
        assert!(
            panes_text.contains(&want("STUB-RAN-ed-stub")),
            "{panes_text}"
        );
    }

    #[test]
    fn migration_covers_each_legacy_value() {
        let m = |t: &str, c: &str, l: &str, os| migrate_terminal(t, c, l, "", os);
        let e = m("custom", "my-term --cd {repo}", "off", Os::Linux)
            .unwrap()
            .unwrap();
        assert_eq!(
            e,
            OpenInEntry::new("Terminal (custom)", "my-term --cd ${cwd}")
        );
        assert_eq!(m("custom", "  ", "off", Os::Linux).unwrap(), None);
        let e = m("kitty", "", "off", Os::Linux).unwrap().unwrap();
        assert_eq!(e, OpenInEntry::new("kitty", "kitty --directory ${cwd}"));
        let e = m("zellij", "", "off", Os::Linux).unwrap().unwrap();
        assert_eq!(e.command, "zellij action new-pane --cwd ${cwd}");
        assert!(m("nope", "", "off", Os::Linux).is_err());
        // auto: the platform terminal, or the ui.launch preset's shell form.
        let e = m("auto", "", "off", Os::Windows).unwrap().unwrap();
        assert_eq!(e, OpenInEntry::new("Default terminal", "wt.exe -d ${cwd}"));
        let e = m("auto", "", "tmux", Os::Linux).unwrap().unwrap();
        assert_eq!(
            e,
            OpenInEntry::new("Default terminal", "tmux new-window -c ${cwd}")
        );
        // Every launch preset migrates to a valid entry naming the folder.
        for lp in launch::PRESETS {
            let e = m(lp.id, "", "off", Os::Linux).unwrap().unwrap();
            validate(&e.command).unwrap_or_else(|er| panic!("{}: {er}", lp.id));
            assert!(e.command.contains("${cwd}"), "{}: {}", lp.id, e.command);
        }
    }

    #[test]
    fn effective_list_follows_detection_and_dedups() {
        let d = detect(
            &env_of(&[("HERDR_ENV", "1"), ("WEZTERM_PANE", "1")]),
            Os::Windows,
            &|p| p == "code" || p == "nvim",
        );
        let list = effective_list(None, &d, Os::Windows);
        let labels: Vec<&str> = list.iter().map(|e| e.label.as_str()).collect();
        assert_eq!(labels[0], "Herdr: new tab");
        assert_eq!(labels[1], "File explorer");
        assert!(labels.contains(&"Herdr: new tab"));
        assert!(labels.contains(&"WezTerm: new tab"));
        assert!(labels.contains(&"VS Code"));
        assert!(labels.contains(&"nvim in WezTerm"));
        assert!(!labels.contains(&"tmux: new window"));
        // Multiplexers before terminals before editors.
        let pos = |l: &str| labels.iter().position(|x| *x == l).unwrap();
        assert!(pos("Herdr: new tab") < pos("WezTerm: new window"));
        assert!(pos("WezTerm: new window") < pos("VS Code"));
        // The legacy terminal comes first.
        let legacy = OpenInEntry::new("Terminal (custom)", "t ${cwd}");
        let list = effective_list(Some(legacy.clone()), &d, Os::Windows);
        assert_eq!(list[0], legacy);
        // Nothing detected: just the explorer.
        let list = effective_list(None, &Detected::default(), Os::Linux);
        assert_eq!(list.len(), 1);
        assert_eq!(default_index(&list, "nope"), 0);
        let two = vec![
            OpenInEntry::new("a", "x ${cwd}"),
            OpenInEntry::new("b", "y ${cwd}"),
        ];
        assert_eq!(default_index(&two, "b"), 1);
        assert_eq!(default_index(&two, ""), 0);
    }
}
