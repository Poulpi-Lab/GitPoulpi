//! How to open a GitPoulpi window when an agent wants to show something and
//! none is open (`[ui] launch`, `[ui] launch_command`).
//!
//! A launch is an **argv template**: split into arguments first, then the
//! placeholders `{repo}`, `{session}` and `{gitpoulpi}` are substituted
//! inside each argument. Nothing goes through a shell unless the template
//! itself starts one (`sh -c '...'`), so a repository path with spaces or
//! quotes stays one argument. Pure functions, no process spawning (that is
//! `gitpoulpi-commands`'s job).
//!
//! `verified` on a preset means its flags were checked against the installed
//! program's own `--help`/man page while writing this; the others follow the
//! program's published documentation and have never been run here.

/// One built-in launch template.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LaunchPreset {
    pub id: &'static str,
    pub label: &'static str,
    /// `terminal` (a new window) or `multiplexer` (a tab/pane/workspace in
    /// one that is already running).
    pub kind: &'static str,
    pub template: &'static str,
    /// `linux`, `macos`, `windows`: where it is offered.
    pub platforms: &'static [&'static str],
    pub verified: bool,
}

const ALL: &[&str] = &["linux", "macos", "windows"];
const UNIX: &[&str] = &["linux", "macos"];
const LINUX: &[&str] = &["linux"];
const WINDOWS: &[&str] = &["windows"];

/// Every preset, in the order Settings lists them.
pub const PRESETS: &[LaunchPreset] = &[
    LaunchPreset {
        id: "xdg-terminal-exec",
        label: "Default terminal (xdg-terminal-exec)",
        kind: "terminal",
        template: "xdg-terminal-exec {gitpoulpi} --session {session}",
        platforms: LINUX,
        verified: false,
    },
    LaunchPreset {
        id: "kitty",
        label: "kitty",
        kind: "terminal",
        template: "kitty --directory {repo} {gitpoulpi} --session {session}",
        platforms: UNIX,
        verified: true,
    },
    LaunchPreset {
        id: "foot",
        label: "foot",
        kind: "terminal",
        template: "foot --working-directory {repo} {gitpoulpi} --session {session}",
        platforms: LINUX,
        verified: true,
    },
    LaunchPreset {
        id: "alacritty",
        label: "Alacritty",
        kind: "terminal",
        template: "alacritty --working-directory {repo} -e {gitpoulpi} --session {session}",
        platforms: ALL,
        verified: false,
    },
    LaunchPreset {
        id: "wezterm",
        label: "WezTerm (new window)",
        kind: "terminal",
        template: "wezterm start --cwd {repo} -- {gitpoulpi} --session {session}",
        platforms: ALL,
        verified: false,
    },
    LaunchPreset {
        id: "ghostty",
        label: "Ghostty",
        kind: "terminal",
        template: "ghostty --working-directory={repo} -e {gitpoulpi} --session {session}",
        platforms: UNIX,
        verified: false,
    },
    LaunchPreset {
        id: "gnome-terminal",
        label: "GNOME Terminal",
        kind: "terminal",
        template: "gnome-terminal --working-directory={repo} -- {gitpoulpi} --session {session}",
        platforms: LINUX,
        verified: false,
    },
    LaunchPreset {
        id: "windows-terminal",
        label: "Windows Terminal (new window)",
        kind: "terminal",
        template: "wt.exe -d {repo} {gitpoulpi} --session {session}",
        platforms: WINDOWS,
        verified: false,
    },
    LaunchPreset {
        id: "windows-terminal-tab",
        label: "Windows Terminal (new tab, current window)",
        kind: "terminal",
        template: "wt.exe -w 0 new-tab -d {repo} {gitpoulpi} --session {session}",
        platforms: WINDOWS,
        verified: false,
    },
    LaunchPreset {
        id: "tmux",
        label: "tmux (new window)",
        kind: "multiplexer",
        template: "tmux new-window -c {repo} {gitpoulpi} --session {session}",
        platforms: UNIX,
        verified: true,
    },
    LaunchPreset {
        id: "zellij",
        label: "zellij (new pane)",
        kind: "multiplexer",
        template: "zellij run --cwd {repo} -- {gitpoulpi} --session {session}",
        platforms: UNIX,
        verified: false,
    },
    LaunchPreset {
        id: "wezterm-tab",
        label: "WezTerm (new tab)",
        kind: "multiplexer",
        template: "wezterm cli spawn --cwd {repo} -- {gitpoulpi} --session {session}",
        platforms: ALL,
        verified: false,
    },
    LaunchPreset {
        id: "cmux",
        label: "cmux (new workspace)",
        kind: "multiplexer",
        template: "cmux new-workspace --cwd {repo} --command \"{gitpoulpi} --session {session}\"",
        platforms: UNIX,
        verified: false,
    },
];

/// The values `ui.launch` takes besides a preset id.
pub const LAUNCH_OFF: &str = "off";
pub const LAUNCH_CUSTOM: &str = "custom";

fn current_platform() -> &'static str {
    if cfg!(windows) {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    }
}

/// The preset with this id.
pub fn preset(id: &str) -> Option<&'static LaunchPreset> {
    PRESETS.iter().find(|p| p.id == id)
}

/// The presets offered on this OS.
pub fn presets_for_this_os() -> Vec<&'static LaunchPreset> {
    let os = current_platform();
    PRESETS
        .iter()
        .filter(|p| p.platforms.contains(&os))
        .collect()
}

/// `off`, this OS's preset ids, then `custom`: the options of the Settings
/// combobox. The configured value is kept even when it is not offered here
/// (a config shared between machines), so the caller appends it when absent.
pub fn choices_for_this_os() -> Vec<String> {
    let mut out = vec![LAUNCH_OFF.to_string()];
    out.extend(presets_for_this_os().iter().map(|p| p.id.to_string()));
    out.push(LAUNCH_CUSTOM.to_string());
    out
}

/// The template `ui.launch` + `ui.launch_command` select: `Ok(None)` for
/// `off`, an error for an unknown id or an empty custom template.
pub fn resolve_template(launch: &str, custom: &str) -> Result<Option<String>, String> {
    match launch.trim() {
        "" | LAUNCH_OFF => Ok(None),
        LAUNCH_CUSTOM => {
            if custom.trim().is_empty() {
                Err("ui.launch is \"custom\" but ui.launch_command is empty".into())
            } else {
                Ok(Some(custom.to_string()))
            }
        }
        id => preset(id)
            .map(|p| Some(p.template.to_string()))
            .ok_or_else(|| {
                format!("ui.launch = \"{id}\" is not \"off\", \"custom\" or a preset id")
            }),
    }
}

/// The value of `ui.terminal_launch` that follows `ui.launch`, else the
/// platform's own terminal.
pub const TERMINAL_AUTO: &str = "auto";

/// The template that opens a terminal or pane **running the user's shell**
/// in `{repo}`, derived from a launch template (which runs gitpoulpi): the
/// `{gitpoulpi} --session {session}` command and the `-e` / `--` /
/// `--command` that introduced it are dropped. The program is always
/// started with the folder as its working directory too, so a template with
/// no `{repo}` (`xdg-terminal-exec`) still opens there.
pub fn shell_template_from(launch_template: &str) -> Result<String, String> {
    let args = split_argv(launch_template)?;
    let mut out: Vec<String> = Vec::new();
    for arg in args {
        if arg.contains("{gitpoulpi}") || arg.contains("{session}") || arg == "--session" {
            // The command's own words, and the flag that introduced it.
            if arg.contains("{session}") && matches!(out.last().map(String::as_str), Some("-s")) {
                out.pop();
            }
            while matches!(
                out.last().map(String::as_str),
                Some("-e") | Some("--") | Some("--command")
            ) {
                out.pop();
            }
            continue;
        }
        out.push(arg);
    }
    if out.is_empty() {
        return Err("the launch command is empty".into());
    }
    Ok(out
        .iter()
        .map(|a| {
            if a.is_empty() || a.contains(char::is_whitespace) {
                format!("'{a}'")
            } else {
                a.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" "))
}

/// The shell-in-`{repo}` template of a preset. `zellij run` needs a
/// command, so its pane comes from `zellij action new-pane`.
pub fn terminal_template(id: &str) -> Option<String> {
    let p = preset(id)?;
    match p.id {
        "zellij" => Some("zellij action new-pane --cwd {repo}".to_string()),
        _ => shell_template_from(p.template).ok(),
    }
}

/// `auto`, this OS's preset ids, then `custom`: the options of the
/// `ui.terminal_launch` combobox.
pub fn terminal_choices_for_this_os() -> Vec<String> {
    let mut out = vec![TERMINAL_AUTO.to_string()];
    out.extend(presets_for_this_os().iter().map(|p| p.id.to_string()));
    out.push(LAUNCH_CUSTOM.to_string());
    out
}

/// The shell template that `ui.terminal_launch`, `ui.terminal_launch_command`
/// and (for `auto`) `ui.launch` select. `Ok(None)` means "no template: use the
/// platform's own terminal" (`auto` with `ui.launch` off); an error names the
/// setting.
pub fn resolve_terminal_template(
    terminal_launch: &str,
    terminal_custom: &str,
    ui_launch: &str,
    ui_launch_custom: &str,
) -> Result<Option<String>, String> {
    match terminal_launch.trim() {
        "" | TERMINAL_AUTO => match resolve_template(ui_launch, ui_launch_custom) {
            Ok(Some(t)) => shell_template_from(&t).map(Some),
            // An unusable `ui.launch` does not stop "Open terminal here".
            Ok(None) | Err(_) => Ok(None),
        },
        LAUNCH_CUSTOM => {
            if terminal_custom.trim().is_empty() {
                Err(
                    "ui.terminal_launch is \"custom\" but ui.terminal_launch_command is empty"
                        .into(),
                )
            } else {
                Ok(Some(terminal_custom.to_string()))
            }
        }
        id => terminal_template(id).map(Some).ok_or_else(|| {
            format!("ui.terminal_launch = \"{id}\" is not \"auto\", \"custom\" or a preset id")
        }),
    }
}

/// What the placeholders stand for.
#[derive(Debug, Clone, Copy)]
pub struct LaunchVars<'a> {
    pub repo: &'a str,
    pub session: &'a str,
    pub gitpoulpi: &'a str,
}

/// Splits `template` into arguments: whitespace separates, `'...'` is
/// literal, `"..."` allows `\"` and `\\`. A backslash outside quotes is
/// literal, so Windows paths survive.
pub fn split_argv(template: &str) -> Result<Vec<String>, String> {
    let mut args = Vec::new();
    let mut cur = String::new();
    let mut started = false;
    let mut chars = template.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            c if c.is_whitespace() => {
                if started {
                    args.push(std::mem::take(&mut cur));
                    started = false;
                }
            }
            '\'' => {
                started = true;
                loop {
                    match chars.next() {
                        Some('\'') => break,
                        Some(ch) => cur.push(ch),
                        None => return Err("unterminated ' in the launch command".into()),
                    }
                }
            }
            '"' => {
                started = true;
                loop {
                    match chars.next() {
                        Some('"') => break,
                        Some('\\') if matches!(chars.peek(), Some('"') | Some('\\')) => {
                            cur.push(chars.next().unwrap_or('\\'));
                        }
                        Some(ch) => cur.push(ch),
                        None => return Err("unterminated \" in the launch command".into()),
                    }
                }
            }
            ch => {
                started = true;
                cur.push(ch);
            }
        }
    }
    if started {
        args.push(cur);
    }
    Ok(args)
}

/// Splits the template, then substitutes the placeholders inside each
/// argument (so a value with spaces stays one argument).
pub fn expand(template: &str, vars: &LaunchVars) -> Result<Vec<String>, String> {
    let args = split_argv(template)?;
    if args.is_empty() {
        return Err("the launch command is empty".into());
    }
    Ok(args
        .into_iter()
        .map(|a| {
            a.replace("{repo}", vars.repo)
                .replace("{session}", vars.session)
                .replace("{gitpoulpi}", vars.gitpoulpi)
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    const VARS: LaunchVars = LaunchVars {
        repo: "/home/u/my repo",
        session: "s 1",
        gitpoulpi: "/opt/gp/gitpoulpi",
    };

    #[test]
    fn split_handles_quotes_and_empty_arguments() {
        assert_eq!(
            split_argv(r#"a  'b c' "d \"e\"" "" f\g"#).unwrap(),
            vec!["a", "b c", r#"d "e""#, "", r"f\g"]
        );
        assert!(split_argv("a 'b").is_err());
        assert!(split_argv("a \"b").is_err());
        assert!(split_argv("   ").unwrap().is_empty());
    }

    #[test]
    fn expansion_keeps_substituted_values_as_single_arguments() {
        let argv = expand(
            "kitty --directory {repo} {gitpoulpi} --session {session}",
            &VARS,
        )
        .unwrap();
        assert_eq!(
            argv,
            vec![
                "kitty",
                "--directory",
                "/home/u/my repo",
                "/opt/gp/gitpoulpi",
                "--session",
                "s 1"
            ]
        );
        let argv = expand("x --command \"{gitpoulpi} -s {session}\"", &VARS).unwrap();
        assert_eq!(argv[2], "/opt/gp/gitpoulpi -s s 1");
        assert!(expand("  ", &VARS).is_err());
    }

    #[test]
    fn every_preset_parses_and_mentions_the_session() {
        let ids: std::collections::HashSet<_> = PRESETS.iter().map(|p| p.id).collect();
        assert_eq!(ids.len(), PRESETS.len(), "duplicate preset id");
        for p in PRESETS {
            let argv = expand(p.template, &VARS).unwrap();
            assert!(!argv[0].is_empty(), "{}", p.id);
            assert!(argv.iter().any(|a| a.contains("s 1")), "{}", p.id);
            assert!(
                argv.iter().any(|a| a.contains("/opt/gp/gitpoulpi")),
                "{}",
                p.id
            );
            assert!(!p.platforms.is_empty());
            assert!(p.kind == "terminal" || p.kind == "multiplexer");
            assert_ne!(p.id, LAUNCH_OFF);
            assert_ne!(p.id, LAUNCH_CUSTOM);
        }
    }

    #[test]
    fn resolve_template_covers_off_custom_preset_and_errors() {
        assert_eq!(resolve_template("off", "x").unwrap(), None);
        assert_eq!(resolve_template("", "").unwrap(), None);
        assert_eq!(
            resolve_template("custom", " my-term {repo} ")
                .unwrap()
                .as_deref(),
            Some(" my-term {repo} ")
        );
        assert!(resolve_template("custom", "  ").is_err());
        assert_eq!(
            resolve_template("tmux", "").unwrap().as_deref(),
            Some(preset("tmux").unwrap().template)
        );
        assert!(resolve_template("nope", "").is_err());
    }

    #[test]
    fn every_preset_has_a_shell_template_without_gitpoulpi() {
        for p in PRESETS {
            let t = terminal_template(p.id).unwrap_or_else(|| panic!("{}", p.id));
            assert!(!t.contains("gitpoulpi") && !t.contains("session"), "{t}");
            assert!(
                t.contains("{repo}") || p.id == "xdg-terminal-exec",
                "{}: {t}",
                p.id
            );
            let argv = expand(&t, &VARS).unwrap();
            assert!(
                !argv.iter().any(|a| a == "-e" || a == "--"),
                "{}: {t}",
                p.id
            );
        }
        assert_eq!(
            terminal_template("kitty").as_deref(),
            Some("kitty --directory {repo}")
        );
        assert_eq!(
            terminal_template("wezterm").as_deref(),
            Some("wezterm start --cwd {repo}")
        );
        assert_eq!(
            terminal_template("cmux").as_deref(),
            Some("cmux new-workspace --cwd {repo}")
        );
        assert_eq!(
            terminal_template("zellij").as_deref(),
            Some("zellij action new-pane --cwd {repo}")
        );
    }

    #[test]
    fn terminal_resolution_follows_ui_launch_in_auto() {
        assert_eq!(
            resolve_terminal_template("auto", "", "off", "").unwrap(),
            None
        );
        assert_eq!(
            resolve_terminal_template("auto", "", "tmux", "")
                .unwrap()
                .as_deref(),
            Some("tmux new-window -c {repo}")
        );
        assert_eq!(
            resolve_terminal_template(
                "auto",
                "",
                "custom",
                "my-term -x {repo} {gitpoulpi} -s {session}"
            )
            .unwrap()
            .as_deref(),
            Some("my-term -x {repo}")
        );
        assert_eq!(
            resolve_terminal_template("foot", "", "tmux", "")
                .unwrap()
                .as_deref(),
            Some("foot --working-directory {repo}")
        );
        assert!(resolve_terminal_template("custom", " ", "off", "").is_err());
        assert!(resolve_terminal_template("nope", "", "off", "").is_err());
        assert!(terminal_choices_for_this_os().len() > 3);
    }

    #[test]
    fn choices_start_with_off_and_end_with_custom() {
        let c = choices_for_this_os();
        assert_eq!(c.first().map(String::as_str), Some("off"));
        assert_eq!(c.last().map(String::as_str), Some("custom"));
        assert!(c.len() > 3);
    }
}
