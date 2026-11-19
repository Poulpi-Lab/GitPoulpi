//! The `gitpoulpi-mcp` program as a library function: the same server
//! `gitpoulpi mcp serve --stdio` runs, for MCP clients that register a binary
//! directly. The `gitpoulpi-mcp` `[[bin]]` and the `gitpoulpi-mcp` persona of
//! the `gitpoulpi` executable both call [`main`].

use clap::Parser;
use gitpoulpi_config::{ConfigPaths, Overrides, Settings};

#[derive(Debug, Parser)]
#[command(
    name = "gitpoulpi-mcp",
    version = gitpoulpi_ipc::VERSION_STRING,
    long_version = gitpoulpi_commands::build_features::long_version_static(),
    about = "GitPoulpi MCP server (stdio by default, streamable HTTP with --http)."
)]
struct Args {
    /// Named session whose daemon to use. Defaults to the session derived
    /// from `--repo` (or the current directory), exactly like the
    /// `gitpoulpi` CLI.
    #[arg(long, short = 's')]
    session: Option<String>,

    /// Repository this server's calls default to when a tool call doesn't
    /// pass its own `repo` argument. Defaults to the current directory.
    #[arg(long)]
    repo: Option<String>,

    /// Which `GitBackend` a daemon this process starts runs on, overriding
    /// `general.default_backend`. Same semantics as the CLI's `--backend`.
    #[arg(long, value_enum)]
    backend: Option<BackendArg>,

    /// Serve over streamable HTTP instead of stdio. Optionally the
    /// address to listen on (default `[mcp] bind`, else 127.0.0.1:8808); the endpoint is
    /// `/mcp`. Requests need the bearer token (`gitpoulpi mcp token`) unless
    /// `[mcp] http_auth` is off. Loopback only unless `--allow-remote`.
    #[arg(long, value_name = "ADDR", num_args = 0..=1, default_missing_value = "")]
    http: Option<String>,

    /// Address for `--http` (same as `--http ADDR`).
    #[arg(long, requires = "http")]
    bind: Option<String>,

    /// Let `--http` listen on a non-loopback address. Refused unless
    /// `[mcp] http_auth` is on with a token in place, and the traffic is
    /// encrypted (`[mcp] tls`, or `behind_proxy` for a tunnel or reverse
    /// proxy you vouch for). Keeping the loopback address and using
    /// `ssh -L` is the safest setup.
    #[arg(long, requires = "http")]
    allow_remote: bool,

    /// How far the agent may move the user's window with the `ui_*` tools:
    /// `ask` (default: the window shows a chip the user confirms), `auto`
    /// or `off`. Overrides `[mcp] ui_control`; put it in the agent's
    /// `.mcp.json` args to choose per agent setup.
    #[arg(long, value_enum)]
    ui_control: Option<UiControlArg>,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum UiControlArg {
    Auto,
    Ask,
    Off,
}

impl From<UiControlArg> for gitpoulpi_core::domain::UiControl {
    fn from(arg: UiControlArg) -> Self {
        match arg {
            UiControlArg::Auto => Self::Auto,
            UiControlArg::Ask => Self::Ask,
            UiControlArg::Off => Self::Off,
        }
    }
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum BackendArg {
    Git2,
    System,
}

impl From<BackendArg> for gitpoulpi_config::DefaultBackend {
    fn from(arg: BackendArg) -> Self {
        match arg {
            BackendArg::Git2 => Self::Git2,
            BackendArg::System => Self::System,
        }
    }
}

/// The address `--http` listens on: `--bind` wins over the optional address
/// after `--http` (empty means "use `[mcp] bind`"). Shared with
/// `gitpoulpi mcp serve --http --bind`.
#[must_use]
pub fn http_addr(http: &str, bind: Option<&str>) -> String {
    bind.unwrap_or(http).to_string()
}

/// Runs the program on `args` (without the program name); exits the process
/// on failure, returns on a clean end of the server.
pub fn main(args: Vec<String>) -> std::process::ExitCode {
    // Started by git as the credential helper of an account-authenticated
    // transfer (ADR-0010)? Then this is all the process does.
    gitpoulpi_ipc::git_credential::run_if_requested();
    gitpoulpi_ipc::auth::set_client_name("MCP");
    let args = match Args::try_parse_from(std::iter::once("gitpoulpi-mcp".to_string()).chain(args))
    {
        Ok(args) => args,
        Err(err) => err.exit(),
    };
    // User-only app directories (best effort, once per start).
    if let Some(paths) = gitpoulpi_config::ConfigPaths::discover() {
        gitpoulpi_config::secure_fs::harden_app_dirs(&paths);
    }
    let overrides = Overrides {
        default_backend: args.backend.map(Into::into),
    };
    let settings = match Settings::resolve(ConfigPaths::discover().as_ref(), &overrides) {
        Ok(settings) => settings,
        Err(err) => {
            eprintln!("gitpoulpi-mcp: {err}");
            std::process::exit(1);
        }
    };
    let backend = settings.config().general.default_backend;
    let mut cfg = settings.config().mcp.clone();
    if let Some(mode) = args.ui_control {
        cfg.ui_control = mode.into();
    }

    let start = match &args.repo {
        Some(path) => std::path::PathBuf::from(path),
        None => std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
    };
    let session = match &args.session {
        Some(name) => gitpoulpi_config::sanitize_session_name(name),
        None => gitpoulpi_config::default_session_name(&start),
    };

    // Worker threads need room for the tool list (a walk of the whole
    // command tree, big derive frames in debug builds): 2 MiB overflowed.
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(32 * 1024 * 1024)
        .build()
        .expect("failed to start tokio runtime");
    let result = match &args.http {
        Some(default_addr) => {
            let addr = http_addr(default_addr, args.bind.as_deref());
            runtime.block_on(crate::serve_http(
                &addr,
                args.allow_remote,
                session,
                backend,
                cfg,
            ))
        }
        None => runtime.block_on(crate::serve_stdio(session, backend, cfg)),
    };
    if let Err(err) = result {
        eprintln!("gitpoulpi-mcp: {err}");
        std::process::exit(1);
    }
    std::process::ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Args, clap::Error> {
        Args::try_parse_from(std::iter::once("gitpoulpi-mcp").chain(args.iter().copied()))
    }

    #[test]
    fn the_flag_surface_is_pinned() {
        let args = parse(&[
            "-s",
            "w",
            "--repo",
            "/r",
            "--backend",
            "system",
            "--http",
            "--bind",
            "127.0.0.1:1",
            "--allow-remote",
            "--ui-control",
            "off",
        ])
        .unwrap();
        assert_eq!(args.session.as_deref(), Some("w"));
        assert_eq!(args.repo.as_deref(), Some("/r"));
        assert_eq!(args.http.as_deref(), Some(""));
        assert_eq!(args.bind.as_deref(), Some("127.0.0.1:1"));
        assert!(args.allow_remote);
    }

    #[test]
    fn bind_and_allow_remote_need_http() {
        assert!(parse(&["--bind", "127.0.0.1:1"]).is_err());
        assert!(parse(&["--allow-remote"]).is_err());
        assert!(parse(&["--nonsense"]).is_err());
        assert!(parse(&["--ui-control", "sometimes", "--http"]).is_err());
    }

    #[test]
    fn bind_wins_over_the_http_address() {
        assert_eq!(http_addr("", None), "");
        assert_eq!(http_addr("1.2.3.4:5", None), "1.2.3.4:5");
        assert_eq!(http_addr("1.2.3.4:5", Some("127.0.0.1:9")), "127.0.0.1:9");
    }
}
