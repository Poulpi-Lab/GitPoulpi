//! "Open the agent": starts the profile's agent once, in a visible terminal,
//! in the working folder GitPoulpi will run it in, so the user answers its
//! first-run questions (folder trust, login, theme...) by hand. GitPoulpi
//! never answers them.
//!
//! The terminal is the user's own (the same choice as "Open in terminal"):
//! the agent's command line is appended to that terminal's launcher where
//! its syntax is known (`wt`, `cmd`, `xdg-terminal-exec`, `$TERMINAL -e`);
//! otherwise (a custom terminal template, macOS Terminal) the terminal opens
//! in the folder and the status line says which command to run.

use super::*;
use gitpoulpi_i18n::msg::ai_page as msg;
use std::path::{Path, PathBuf};

/// What `program args` (a terminal launcher) must be given to run `agent`
/// in the terminal it opens; `None` when its syntax is not known.
pub(super) fn launcher_with_agent(
    program: &str,
    args: &[String],
    agent: &[String],
) -> Option<(String, Vec<String>)> {
    let stem = program
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(program)
        .trim_end_matches(".exe")
        .to_ascii_lowercase();
    let mut out = args.to_vec();
    match stem.as_str() {
        "wt" | "xdg-terminal-exec" => {}
        // Bare `cmd` in a new console: `/K` keeps the shell open on the agent.
        "cmd" => out.push("/K".into()),
        "open" => return None,
        _ => out.push("-e".into()),
    }
    out.extend(agent.iter().cloned());
    Some((program.to_string(), out))
}

/// The command line that starts the agent of `profile` interactively: its
/// command, the terminal arguments, and (Unix, non-secret values only) its
/// environment through `env`.
pub(super) fn agent_command_line(profile: &AiProfile) -> Vec<String> {
    use gitpoulpi_ai::presets;
    let mut line = Vec::new();
    if cfg!(unix) {
        let vars: Vec<String> = profile
            .env
            .iter()
            .filter(|(k, v)| !k.trim().is_empty() && !presets::env_value_is_secret(k, v))
            .map(|(k, v)| {
                format!(
                    "{}={}",
                    k.trim(),
                    gitpoulpi_ai::expand_env_refs(v, |n| std::env::var(n).ok())
                )
            })
            .collect();
        if !vars.is_empty() {
            line.push("env".to_string());
            line.extend(vars);
        }
    }
    line.push(profile.command.trim().to_string());
    let args = if profile.terminal_args.is_empty() {
        presets::harness_of(profile)
            .map(|h| h.default_args_for(gitpoulpi_config::AgentRunMode::Terminal))
            .unwrap_or_default()
    } else {
        profile.terminal_args.clone()
    };
    line.extend(args);
    line
}

/// Opens the agent of `profile`; the message for the status line.
pub(super) fn open_agent(profile: &AiProfile, repo_root: &Path) -> Result<String, String> {
    if profile.kind != AiProviderKind::LocalAgent || profile.command.trim().is_empty() {
        return Err(msg::OPEN_AGENT_NO_AGENT.get().to_string());
    }
    let mut terminal_profile = profile.clone();
    terminal_profile.run_mode = gitpoulpi_config::AgentRunMode::Terminal;
    let folder: PathBuf =
        gitpoulpi_ai::agent_terminal::workdir_for(&terminal_profile, Some(repo_root))
            .unwrap_or_else(gitpoulpi_ai::agent_terminal::dedicated_dir);
    std::fs::create_dir_all(&folder).map_err(|e| msg::open_agent_failed(e.to_string()))?;
    let agent = agent_command_line(profile);

    let templated = crate::tab_dir::terminal_argv(&folder)?.is_some();
    let (program, args) = crate::repo_menu::terminal_command(
        &folder,
        crate::repo_menu::on_path,
        std::env::var("TERMINAL").ok(),
    );
    let launcher = if templated {
        None
    } else {
        launcher_with_agent(&program, &args, &agent)
    };
    let (program, args, ran) = match launcher {
        Some((p, a)) => (p, a, true),
        None => match crate::tab_dir::terminal_argv(&folder)? {
            Some(argv) => (argv[0].clone(), argv[1..].to_vec(), false),
            None => (program, args, false),
        },
    };
    let label = msg::OPEN_AGENT.get().trim_end_matches('.').to_string();
    let launch = crate::tab_dir::Launch {
        label: &label,
        program: &program,
        args: &args,
        cwd: &folder,
        info: &crate::tab_dir::launch_repo_info(&folder),
    };
    crate::tab_dir::start(&launch, crate::external_tools::report_late_error)?;
    let shown = agent
        .iter()
        .skip_while(|a| *a == "env" || a.contains('='))
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    Ok(if ran {
        msg::opened_agent(folder.display().to_string())
    } else {
        msg::opened_terminal_for_agent(folder.display().to_string(), &shown)
    })
}

impl PreferencesState {
    pub(super) fn ai_open_agent(&mut self) {
        let Some(i) = self.selected_ai_profile() else {
            return;
        };
        let profile = self.config.ai.profiles[i].clone();
        self.message = Some(match open_agent(&profile, &self.repo_root) {
            Ok(done) => done,
            Err(why) => why,
        });
    }

    /// The palette's "AI: Open the Agent": the default profile's agent.
    pub(crate) fn ai_open_default_agent(&mut self) {
        let ai = &self.config.ai;
        let profile = ai
            .profile(&ai.default_profile)
            .or_else(|| ai.profiles.first())
            .cloned();
        self.message = Some(match profile {
            Some(profile) => match open_agent(&profile, &self.repo_root) {
                Ok(done) => done,
                Err(why) => why,
            },
            None => msg::OPEN_AGENT_NO_AGENT.get().to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn agent() -> Vec<String> {
        vec!["claude".into(), "--effort".into(), "low".into()]
    }

    #[test]
    fn the_agent_is_appended_where_the_terminals_syntax_is_known() {
        let (_, a) = launcher_with_agent("wt", &["-d".into(), "/x".into()], &agent()).unwrap();
        assert_eq!(a, ["-d", "/x", "claude", "--effort", "low"]);
        let (_, a) = launcher_with_agent("xdg-terminal-exec", &[], &agent()).unwrap();
        assert_eq!(a, ["claude", "--effort", "low"]);
        let (_, a) = launcher_with_agent("/usr/bin/alacritty", &[], &agent()).unwrap();
        assert_eq!(a, ["-e", "claude", "--effort", "low"]);
        let (_, a) = launcher_with_agent("cmd", &[], &agent()).unwrap();
        assert_eq!(a, ["/K", "claude", "--effort", "low"]);
        assert!(launcher_with_agent("open", &[], &agent()).is_none());
    }

    #[test]
    fn the_command_line_uses_the_terminal_arguments_and_hides_secrets() {
        let mut p = AiProfile::new("x", "X", AiProviderKind::LocalAgent);
        p.command = "claude".into();
        p.terminal_args = vec!["--effort".into(), "low".into()];
        p.env.insert("CLAUDE_CONFIG_DIR".into(), "/cfg".into());
        p.env.insert("ANTHROPIC_API_KEY".into(), "sk-secret".into());
        let line = agent_command_line(&p);
        assert!(!line.iter().any(|a| a.contains("sk-secret")), "{line:?}");
        assert_eq!(&line[line.len() - 3..], ["claude", "--effort", "low"]);
        if cfg!(unix) {
            assert_eq!(&line[..2], ["env", "CLAUDE_CONFIG_DIR=/cfg"]);
        }
        p.terminal_args.clear();
        assert_eq!(
            agent_command_line(&p).last().map(String::as_str),
            Some("claude")
        );
    }
}
