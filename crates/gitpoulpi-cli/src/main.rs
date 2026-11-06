mod config_cmd;
mod daemon_cmd;
mod desktop_cmd;
mod uninstall_cmd;

#[cfg(feature = "mcp")]
use gitpoulpi_commands::cli::McpAction;
use gitpoulpi_commands::cli::{self, Commands, DaemonAction};
use gitpoulpi_config::{ConfigPaths, DefaultBackend, Overrides, Settings};

/// The command tree is large, and clap's derive code for it builds big
/// frames in debug builds: on Windows' 1 MiB main-thread stack, parsing
/// alone (even `--help`) overflowed. So everything runs on a thread with an
/// explicit, generous stack.
const STACK_BYTES: usize = 32 * 1024 * 1024;

/// The tokio runtime every command runs on. Its worker threads get the same
/// generous stack as the main worker: the MCP server builds its tool list
/// (a walk of the whole command tree, big derive frames in debug builds)
/// on one of them, and the default 2 MiB overflowed once the tree grew.
fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(STACK_BYTES)
        .build()
        .expect("failed to start tokio runtime")
}

fn main() {
    let worker = std::thread::Builder::new()
        .name("gitpoulpi-main".into())
        .stack_size(STACK_BYTES)
        .spawn(real_main)
        .expect("failed to spawn the main worker thread");
    if worker.join().is_err() {
        std::process::exit(101);
    }
}

fn real_main() {
    // User-only app directories (best effort, once per start).
    if let Some(paths) = gitpoulpi_config::ConfigPaths::discover() {
        gitpoulpi_config::secure_fs::harden_app_dirs(&paths);
    }
    let cli = cli::parse_localized();
    if cli.add.is_some() && cli.command.is_some() {
        eprintln!("gitpoulpi: --add opens a folder as a tab and takes no subcommand");
        std::process::exit(2);
    }
    let session = resolve_session(cli.session.as_deref(), cli.repo.as_deref());
    let overrides = Overrides {
        default_backend: cli.backend.map(Into::into),
    };

    match cli.command {
        // `gitpoulpi` / `gp` with no subcommand launches the TUI, like
        // double-clicking a desktop app -- the CLI's read/write subcommands
        // are for scripting/agent use.
        None => {
            // The TUI resolves its own settings (it has no command-line
            // layer, see `gitpoulpi_tui::run`), so a `--backend` typed here
            // would be read by nobody; say so rather than ignore it.
            if cli.backend.is_some() {
                eprintln!("gitpoulpi: {TUI_BACKEND_FLAG_HELP}");
                std::process::exit(2);
            }
            let mut repo = cli.repo.as_ref().map(std::path::PathBuf::from);
            let mut session = session;
            // `--add`: a tab in the GitPoulpi that is already open, else a
            // normal start on that folder.
            if let Some(path) = cli.add.clone() {
                let path = path
                    .or_else(|| cli.repo.clone())
                    .map(std::path::PathBuf::from);
                let wanted = cli
                    .session
                    .as_deref()
                    .map(gitpoulpi_config::sanitize_session_name);
                let args = gitpoulpi_daemon::join::AddArgs {
                    path: path.clone(),
                    session: wanted,
                    no_raise: cli.no_raise,
                };
                match gitpoulpi_daemon::join::run_add(&args, cli.json) {
                    gitpoulpi_daemon::join::AddExit::Exit(code) => std::process::exit(code),
                    gitpoulpi_daemon::join::AddExit::Start => {
                        if path.is_some() {
                            repo = path;
                            session = resolve_session(
                                cli.session.as_deref(),
                                repo.as_deref().and_then(std::path::Path::to_str),
                            );
                        }
                    }
                }
            }
            // `run` has already restored the terminal, explained the
            // error (with its cause chain) and recorded it in
            // `gitpoulpi-tui-exit.log` -- only the exit status is left.
            if gitpoulpi_tui::run_with(
                &session,
                gitpoulpi_tui::RunOptions {
                    cwd_file: cli.cwd_file.clone(),
                    repo,
                    launcher: cli.launcher,
                },
            )
            .is_err()
            {
                std::process::exit(1);
            }
        }
        // Daemon lifecycle is the one command group that is *about* the
        // daemon rather than sent to it, so `run`/`status` never open an
        // IPC client of their own beyond what they need to ask; `stop`/
        // `start`/`restart` do, deliberately -- see `daemon_cmd`.
        Some(Commands::Daemon { action }) => run_daemon_command(
            &session,
            action,
            resolve_backend(&overrides),
            cli.backend,
            cli.json,
        ),
        // `session` answers from the session files rather than a daemon:
        // `gitpoulpi_commands::run` handles it before it would connect.
        // `config` reads and writes a file on this machine: it has to work
        // outside a repository and must not start a daemon to answer.
        Some(Commands::Config { action }) => config_cmd::run(action, &overrides, cli.json),
        Some(Commands::Desktop { action }) => desktop_cmd::run(action, cli.json),
        Some(Commands::Launch {
            new_window,
            command,
        }) => std::process::exit(gitpoulpi_commands::launch::run(
            &command, cli.json, false, new_window,
        )),
        Some(Commands::Uninstall { keep_files }) => uninstall_cmd::run(keep_files, cli.json),
        #[cfg(feature = "mcp")]
        Some(Commands::Mcp { action }) => {
            run_mcp_command(&session, action, resolve_backend(&overrides))
        }
        Some(_) => {
            let backend = resolve_backend(&overrides);
            run_repo_command(cli, session, backend)
        }
    }
}

const TUI_BACKEND_FLAG_HELP: &str =
    "--backend applies to gitpoulpi's subcommands, not to the TUI. Set \
     general.default_backend in config.toml (`gitpoulpi config set general.default_backend \
     git2`) or export GITPOULPI_GENERAL_DEFAULT_BACKEND.";

/// Runs the whole precedence ladder (built-in defaults -> config.toml ->
/// `GITPOULPI_*` -> command-line flags) and hands back the one piece the
/// CLI acts on.
///
/// A config file that will not parse is fatal rather than quietly replaced
/// by the defaults: running on a backend other than the configured one,
/// without saying so, would be a silent bug.
fn resolve_backend(overrides: &Overrides) -> DefaultBackend {
    match Settings::resolve(ConfigPaths::discover().as_ref(), overrides) {
        Ok(settings) => settings.config().general.default_backend,
        Err(err) => {
            eprintln!("gitpoulpi: {err}");
            std::process::exit(1);
        }
    }
}

/// Turns the optional `-s` into the session name every other component
/// uses. Both branches go through `gitpoulpi_config`, as the TUI's own dev
/// binary does; if the two resolved differently, a CLI mutation would stop
/// showing up in an open TUI on the same repo (architecture principle 3).
///
/// The derived default follows `--repo` when given, not the current
/// directory: an agent driving several repositories from one place must
/// land on the *target* repo's daemon, the one an open TUI is using.
fn resolve_session(explicit: Option<&str>, repo: Option<&str>) -> String {
    match explicit {
        // Sanitised even when the user typed it: the name becomes
        // `sessions/<name>.port`, and `-s feature/login` would otherwise
        // be a path into a directory that does not exist (and is illegal
        // outright on Windows).
        Some(name) => gitpoulpi_config::sanitize_session_name(name),
        None => {
            let start = match repo {
                Some(path) => std::path::PathBuf::from(path),
                None => std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
            };
            gitpoulpi_config::default_session_name(&start)
        }
    }
}

fn run_repo_command(cli: cli::Cli, session: String, backend: DefaultBackend) {
    let json = cli.json;
    let runtime = runtime();
    let result = runtime.block_on(gitpoulpi_commands::run(cli, session, backend));
    if let Ok(code) = result {
        if code != 0 {
            std::process::exit(code);
        }
        return;
    }
    if let Err(err) = result {
        // A scripting caller that asked for JSON gets JSON on the error
        // path too, so it never has to parse a human sentence to find out
        // what went wrong.
        if json {
            println!(
                "{}",
                gitpoulpi_commands::build_features::error_object(&err.to_string())
            );
        } else {
            eprintln!("gitpoulpi: {err}");
            if let Some(detail) = gitpoulpi_commands::client::error_detail_for(&err.to_string()) {
                for line in detail.lines() {
                    eprintln!("  {line}");
                }
            }
        }
        std::process::exit(1);
    }
}

#[cfg(feature = "mcp")]
/// `[mcp]` from the ambient config, with `--ui-control` applied on top.
fn mcp_config(ui_control: Option<&str>) -> gitpoulpi_config::McpConfig {
    let mut cfg = Settings::resolve_ambient()
        .map(|s| s.config().mcp.clone())
        .unwrap_or_default();
    if let Some(mode) = ui_control.and_then(|m| m.parse().ok()) {
        cfg.ui_control = mode;
    }
    cfg
}

#[cfg(feature = "mcp")]
fn run_mcp_command(session: &str, action: McpAction, backend: DefaultBackend) {
    match action {
        McpAction::Serve {
            stdio: _,
            http: None,
            ui_control,
            ..
        } => {
            // `--stdio` is accepted but not required: stdio is the only
            // transport implemented, so it is also the default -- see
            // `gitpoulpi_mcp::serve_stdio`.
            let cfg = mcp_config(ui_control.as_deref());
            let runtime = runtime();
            if let Err(err) = runtime.block_on(gitpoulpi_mcp::serve_stdio(
                session.to_string(),
                backend,
                cfg,
            )) {
                eprintln!("gitpoulpi: mcp serve failed: {err}");
                std::process::exit(1);
            }
        }
        McpAction::Serve {
            http: Some(addr),
            allow_remote,
            ui_control,
            ..
        } => {
            let cfg = mcp_config(ui_control.as_deref());
            let runtime = runtime();
            if let Err(err) = runtime.block_on(gitpoulpi_mcp::serve_http(
                &addr,
                allow_remote,
                session.to_string(),
                backend,
                cfg,
            )) {
                eprintln!("gitpoulpi: mcp serve failed: {err}");
                std::process::exit(1);
            }
        }
    }
}

fn run_daemon_command(
    session: &str,
    action: DaemonAction,
    selection: DefaultBackend,
    backend_flag: Option<cli::BackendArg>,
    json: bool,
) {
    match action {
        DaemonAction::Run => {
            // Built through `gitpoulpi_daemon::backend_for` rather than
            // named here, so this long-lived daemon and the short-lived
            // ones the CLI and TUI embed can never disagree about which
            // implementation `general.default_backend` selects.
            let backend = gitpoulpi_daemon::backend_for(selection);
            let daemon = gitpoulpi_daemon::Daemon::new(session);
            // Its log lines go to the same day file the window reads.
            gitpoulpi_log::init_from_settings("daemon");
            let runtime = runtime();
            if let Err(err) = runtime.block_on(daemon.run_foreground(backend)) {
                eprintln!("gitpoulpi: daemon failed: {err}");
                std::process::exit(1);
            }
        }
        DaemonAction::Start => daemon_cmd::start(session, selection, backend_flag, json),
        DaemonAction::Stop { all } => daemon_cmd::stop(session, all, json),
        DaemonAction::Restart => daemon_cmd::restart(session, selection, backend_flag, json),
        DaemonAction::Status => daemon_cmd::status(session, json),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_explicit_session_is_sanitised_before_it_becomes_a_filename() {
        assert_eq!(
            resolve_session(Some("feature/login"), None),
            "feature-login"
        );
        assert_eq!(resolve_session(Some("CON"), None), "_CON");
    }

    #[test]
    fn no_flag_derives_a_non_empty_default() {
        assert!(!resolve_session(None, None).is_empty());
    }

    #[test]
    fn the_derived_default_follows_repo_rather_than_the_current_directory() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("elsewhere");
        std::fs::create_dir_all(repo.join(".git")).unwrap();

        let via_repo = resolve_session(None, Some(&repo.to_string_lossy()));
        assert_eq!(
            via_repo,
            gitpoulpi_config::default_session_name(&repo),
            "--repo must pick the target repo's session, not the cwd's"
        );
        assert_ne!(via_repo, resolve_session(None, None));
    }
}
