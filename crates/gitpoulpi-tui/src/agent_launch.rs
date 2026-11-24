//! Opening a local AI agent in a visible terminal: shared by Settings'
//! "Open the agent..." (the agent's own working folder) and the palette /
//! sequence command "Tab: Open AI Agent Here" (the active tab's folder).
//!
//! The terminal is the user's own (the same choice as "Terminal here"): the
//! agent's command line is appended to that terminal's launcher where its
//! syntax is known (`wt`, `cmd`, `xdg-terminal-exec`, `$TERMINAL -e`);
//! otherwise (a custom terminal template, macOS Terminal) the terminal opens
//! in the folder and the status line says which command to run. Nothing is
//! sent to the agent and no AI request starts: opening it is the action.

use gitpoulpi_config::{AiConfig, AiProfile, AiProviderKind};
use gitpoulpi_i18n::msg::ai_page as msg;
use std::path::Path;

/// What `program args` (a terminal launcher) must be given to run `agent`
/// in the terminal it opens; `None` when its syntax is not known.
pub(crate) fn launcher_with_agent(
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
pub(crate) fn agent_command_line(profile: &AiProfile) -> Vec<String> {
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

/// What starting the agent in a folder will run, before anything spawns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Plan {
    pub(crate) program: String,
    pub(crate) args: Vec<String>,
    /// The agent is part of the launch (else the terminal only opens there).
    pub(crate) ran: bool,
    /// The agent command line as shown to the user (no environment).
    pub(crate) shown: String,
}

/// Whether `profile` can be opened as an agent at all.
pub(crate) fn is_local_agent(profile: &AiProfile) -> bool {
    profile.kind == AiProviderKind::LocalAgent && !profile.command.trim().is_empty()
}

/// The terminal launch that opens `profile`'s agent in `folder`.
pub(crate) fn plan(profile: &AiProfile, folder: &Path) -> Result<Plan, String> {
    let agent = agent_command_line(profile);
    let templated = crate::tab_dir::terminal_argv(folder)?.is_some();
    let (program, args) = crate::repo_menu::terminal_command(
        folder,
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
        None => match crate::tab_dir::terminal_argv(folder)? {
            Some(argv) => (argv[0].clone(), argv[1..].to_vec(), false),
            None => (program, args, false),
        },
    };
    let shown = agent
        .iter()
        .skip_while(|a| *a == "env" || a.contains('='))
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    Ok(Plan {
        program,
        args,
        ran,
        shown,
    })
}

/// Starts the agent of `profile` in `folder`; the status line's message.
pub(crate) fn start_in(profile: &AiProfile, folder: &Path) -> Result<String, String> {
    let plan = plan(profile, folder)?;
    let label = msg::OPEN_AGENT.get().trim_end_matches('.').to_string();
    let launch = crate::tab_dir::Launch {
        label: &label,
        program: &plan.program,
        args: &plan.args,
        raw_args: None,
        cwd: folder,
        info: &crate::tab_dir::launch_repo_info(folder),
    };
    crate::tab_dir::start(&launch, crate::external_tools::report_late_error)?;
    Ok(if plan.ran {
        msg::opened_agent(folder.display().to_string())
    } else {
        msg::opened_terminal_for_agent(folder.display().to_string(), &plan.shown)
    })
}

/// The local-agent profiles of `ai`: id and display name, in order.
pub(crate) fn local_agents(ai: &AiConfig) -> Vec<(String, String)> {
    ai.profiles
        .iter()
        .filter(|p| is_local_agent(p))
        .map(|p| (p.id.clone(), p.display_name().to_string()))
        .collect()
}

/// `ai`'s profile `id` (empty: the default one) as an agent to open.
pub(crate) fn resolve<'a>(ai: &'a AiConfig, id: &str) -> Result<&'a AiProfile, String> {
    if !ai.enabled {
        return Err(
            gitpoulpi_i18n::msg::palette::WHY_AI_IS_TURNED_OFF_SETTINGS_AI
                .get()
                .to_string(),
        );
    }
    let id = id.trim();
    let profile = if id.is_empty() {
        ai.profile(&ai.default_profile)
            .filter(|p| is_local_agent(p))
            .or_else(|| ai.profiles.iter().find(|p| is_local_agent(p)))
    } else {
        Some(
            ai.profile(id)
                .ok_or_else(|| msg::open_agent_unknown_profile(id))?,
        )
    };
    profile
        .filter(|p| is_local_agent(p))
        .ok_or_else(|| msg::OPEN_AGENT_NO_AGENT.get().to_string())
}

/// Opens profile `id` of `ai` in the tab at `path`. The one function the
/// palette's chooser and a sequence step both reach.
pub(crate) fn open_here_with(ai: &AiConfig, id: &str, path: &Path) -> Result<String, String> {
    let profile = resolve(ai, id)?;
    if !path.is_dir() {
        return Err(gitpoulpi_i18n::msg::tab_dir::gone_from_disk(path.display()));
    }
    start_in(profile, path)
}

fn ambient_ai() -> AiConfig {
    gitpoulpi_config::Settings::resolve_ambient()
        .map(|s| s.config().ai.clone())
        .unwrap_or_default()
}

/// "Tab: Open AI Agent Here" with a profile id (a sequence's `profile`
/// option; empty = the default profile), read from the live settings.
pub(crate) fn open_here(id: &str, path: &Path) -> Result<String, String> {
    open_here_with(&ambient_ai(), id, path)
}

/// What the palette does: no agent says so, one runs, several ask (the
/// chooser lists them).
pub(crate) enum PaletteStep {
    Done(Result<String, String>),
    Choose(Vec<(String, String)>),
}

pub(crate) fn palette_step(path: &Path) -> PaletteStep {
    let ai = ambient_ai();
    if !ai.enabled {
        return PaletteStep::Done(resolve(&ai, "").map(|_| String::new()));
    }
    let agents = local_agents(&ai);
    match agents.len() {
        0 => PaletteStep::Done(Err(msg::OPEN_AGENT_NO_AGENT.get().to_string())),
        1 => PaletteStep::Done(open_here_with(&ai, &agents[0].0, path)),
        _ => PaletteStep::Choose(agents),
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

    fn local(id: &str, command: &str) -> AiProfile {
        let mut p = AiProfile::new(id, id, AiProviderKind::LocalAgent);
        p.command = command.into();
        p.terminal_args = vec!["--hello".into()];
        p
    }

    fn ai(enabled: bool, profiles: Vec<AiProfile>, default: &str) -> AiConfig {
        AiConfig {
            enabled,
            default_profile: default.into(),
            profiles,
            ..AiConfig::default()
        }
    }

    #[test]
    fn a_profile_id_picks_the_agent_and_the_default_is_the_fallback() {
        let cfg = ai(
            true,
            vec![
                AiProfile::new("api", "Api", AiProviderKind::OpenAiCompatible),
                local("one", "agent-one"),
                local("two", "agent-two"),
            ],
            "api",
        );
        assert_eq!(resolve(&cfg, "two").unwrap().command, "agent-two");
        // The default is not an agent: the first local one stands in.
        assert_eq!(resolve(&cfg, "").unwrap().command, "agent-one");
        assert_eq!(local_agents(&cfg).len(), 2);
        let err = resolve(&cfg, "nope").unwrap_err();
        assert!(err.contains("nope"), "{err}");
        // A profile that is not a local agent is refused, not launched.
        assert_eq!(
            resolve(&cfg, "api").unwrap_err(),
            msg::OPEN_AGENT_NO_AGENT.get()
        );
    }

    #[test]
    fn ai_off_or_no_agent_is_a_clear_error() {
        let off = ai(false, vec![local("one", "agent-one")], "one");
        assert!(resolve(&off, "one").is_err());
        let none = ai(true, vec![], "");
        assert_eq!(
            resolve(&none, "").unwrap_err(),
            msg::OPEN_AGENT_NO_AGENT.get()
        );
        let dir = tempfile::tempdir().unwrap();
        assert!(open_here_with(&none, "", dir.path()).is_err());
    }

    #[test]
    fn the_launch_runs_the_agent_in_the_tabs_folder() {
        let _g = crate::tab_dir::TEST_SERIAL
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        crate::tab_dir::reset_for_tests();
        let dir = tempfile::tempdir().unwrap();
        let plan = plan(&local("one", "agent-one"), dir.path());
        let plan = plan.unwrap();
        if cfg!(unix) && !cfg!(target_os = "macos") {
            // A terminal whose syntax is known gets the agent appended.
            assert!(plan.ran, "{plan:?}");
            assert_eq!(&plan.args[plan.args.len() - 2..], ["agent-one", "--hello"]);
        }
        assert_eq!(plan.shown, "agent-one --hello");
    }
}
