//! Standalone entrypoint for `cargo run -p gitpoulpi-tui` during development.
//! The normal way to launch the TUI is `gitpoulpi` (or `gp`) with no
//! subcommand; see crates/gitpoulpi-cli.
//!
//! It accepts exactly one flag, `-s`/`--session`, spelled like the CLI's
//! global one and resolving an omitted value through the same
//! `gitpoulpi_config::default_session_name`. If the two disagreed about the
//! session, a CLI mutation would stop showing up in the open TUI and look
//! like a UI regression, so the tests below pin the spelling.

fn main() -> std::io::Result<()> {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let cwd_file = match take_cwd_file(&mut args) {
        Ok(file) => file,
        Err(message) => {
            eprintln!("gitpoulpi-tui: {message}\n\n{USAGE}");
            std::process::exit(2);
        }
    };
    let launcher = take_flag(&mut args, "--launcher");
    let mut repo = match take_repo(&mut args) {
        Ok(repo) => repo,
        Err(message) => {
            eprintln!("gitpoulpi-tui: {message}\n\n{USAGE}");
            std::process::exit(2);
        }
    };
    let no_raise = take_flag(&mut args, "--no-raise");
    let add = take_add(&mut args);
    let parsed = parse_session_arg(args.iter().map(String::as_str));
    // `gp -a`: a tab in the GitPoulpi that is already open, else start here.
    if let (Some(path), Ok(SessionArg::Named(_) | SessionArg::Default)) = (&add, &parsed) {
        let path = path.clone().or_else(|| repo.clone());
        let wanted = match &parsed {
            Ok(SessionArg::Named(name)) => Some(name.clone()),
            _ => None,
        };
        let join_args = gitpoulpi_daemon::join::AddArgs {
            path: path.clone(),
            session: wanted,
            no_raise,
        };
        match gitpoulpi_daemon::join::run_add(&join_args, false) {
            gitpoulpi_daemon::join::AddExit::Exit(code) => std::process::exit(code),
            gitpoulpi_daemon::join::AddExit::Start => repo = path,
        }
    }
    let session = match parsed {
        Ok(SessionArg::Named(name)) => name,
        Ok(SessionArg::Default) => {
            let start = match &repo {
                Some(path) => path.clone(),
                None => std::env::current_dir()?,
            };
            gitpoulpi_config::default_session_name(&start)
        }
        Ok(SessionArg::Help) => {
            println!("{USAGE}");
            return Ok(());
        }
        Ok(SessionArg::Version) => {
            println!(
                "gitpoulpi-tui {}",
                gitpoulpi_tui::build_features::long_version()
            );
            return Ok(());
        }
        Err(message) => {
            eprintln!("gitpoulpi-tui: {message}\n\n{USAGE}");
            std::process::exit(2);
        }
    };
    gitpoulpi_tui::run_with(
        &session,
        gitpoulpi_tui::RunOptions {
            cwd_file,
            repo,
            launcher,
        },
    )
}

// i18n-scan: off
const USAGE: &str = "\
Usage: gitpoulpi-tui [-s|--session <name>] [--repo <path>] [--cwd-file <file>] [--launcher]
                     [-a|--add [path]] [--no-raise]

  -s, --session <name>  Named session to target. Omitted, it is derived
                        from the current repository's root, so two repos
                        get two daemons and everything inside one repo
                        shares one.
      --repo <path>     Open this repository instead of the current
                        directory's (also picks the default session).
      --cwd-file <file> Write the active tab's folder to <file> on quit
                        (\"Quit and cd here\"), for `gitpoulpi shell-init`.
      --launcher        Started by a desktop / Start menu shortcut: the tab
                        to open on is `general.startup_tab_launcher`.
  -a, --add [path]      Open the folder (default: the current directory) as
                        a new tab of the GitPoulpi that is already running,
                        instead of starting another one. Several running
                        sessions: pick one from a list, or pass -s <name>.
                        None running: start normally on that folder.
      --no-raise        With --add: do not bring the window forward.
  -V, --version         Print the version and build sha.
  -h, --help            Print this message.

This is the development entrypoint; `gitpoulpi` with no subcommand is the
normal way to launch the TUI and takes the same flag.";
// i18n-scan: on

/// Removes `--cwd-file <file>` / `--cwd-file=<file>` from `args` (it is not
/// part of the session surface the tests below pin).
fn take_cwd_file(args: &mut Vec<String>) -> Result<Option<std::path::PathBuf>, String> {
    let mut file = None;
    let mut rest = Vec::with_capacity(args.len());
    let mut iter = std::mem::take(args).into_iter();
    while let Some(arg) = iter.next() {
        if arg == "--cwd-file" {
            file = Some(
                iter.next()
                    .ok_or_else(|| "--cwd-file needs a file".to_string())?, // i18n-english
            );
        } else if let Some(value) = arg.strip_prefix("--cwd-file=") {
            file = Some(value.to_string());
        } else {
            rest.push(arg);
        }
    }
    *args = rest;
    Ok(file.map(std::path::PathBuf::from))
}

/// Removes `-a` / `--add` (with an optional folder after it) from `args`:
/// `None` when absent, `Some(None)` for a bare flag, `Some(Some(path))` with
/// a folder (`--add=path` too). A following word that starts with `-` is
/// another option, not the folder.
fn take_add(args: &mut Vec<String>) -> Option<Option<std::path::PathBuf>> {
    let mut found = None;
    let mut rest = Vec::with_capacity(args.len());
    let mut iter = std::mem::take(args).into_iter().peekable();
    while let Some(arg) = iter.next() {
        if arg == "-a" || arg == "--add" {
            let path = iter.next_if(|next| !next.starts_with('-'));
            found = Some(path.map(std::path::PathBuf::from));
        } else if let Some(value) = arg.strip_prefix("--add=") {
            found = Some(Some(std::path::PathBuf::from(value)));
        } else {
            rest.push(arg);
        }
    }
    *args = rest;
    found
}

/// Removes every `flag` from `args`; whether there was one.
fn take_flag(args: &mut Vec<String>, flag: &str) -> bool {
    let before = args.len();
    args.retain(|a| a != flag);
    args.len() != before
}

/// Removes `--repo <path>` / `--repo=<path>` from `args`.
fn take_repo(args: &mut Vec<String>) -> Result<Option<std::path::PathBuf>, String> {
    let mut repo = None;
    let mut rest = Vec::with_capacity(args.len());
    let mut iter = std::mem::take(args).into_iter();
    while let Some(arg) = iter.next() {
        if arg == "--repo" {
            repo = Some(
                iter.next()
                    .ok_or_else(|| "--repo needs a path".to_string())?, // i18n-english
            );
        } else if let Some(value) = arg.strip_prefix("--repo=") {
            repo = Some(value.to_string());
        } else {
            rest.push(arg);
        }
    }
    *args = rest;
    Ok(repo.map(std::path::PathBuf::from))
}

#[derive(Debug, PartialEq, Eq)]
enum SessionArg {
    Named(String),
    Default,
    Help,
    Version,
}

/// Parses this binary's whole argument surface. Returns the *raw* name;
/// sanitising happens in `gitpoulpi_config` so the CLI and this binary
/// cannot apply different rules to the same string.
fn parse_session_arg<'a>(args: impl Iterator<Item = &'a str>) -> Result<SessionArg, String> {
    let mut args = args.into_iter();
    let mut session: Option<String> = None;

    while let Some(arg) = args.next() {
        match arg {
            "-h" | "--help" => return Ok(SessionArg::Help),
            "-V" | "--version" => return Ok(SessionArg::Version),
            "-s" | "--session" => {
                let value = args
                    .next()
                    .ok_or_else(|| format!("{arg} needs a session name"))?; // i18n-english
                session = Some(value.to_string());
            }
            _ if arg.starts_with("--session=") => {
                session = Some(arg["--session=".len()..].to_string());
            }
            other => return Err(format!("unrecognized argument {other:?}")), // i18n-english
        }
    }

    Ok(match session {
        Some(name) if name.is_empty() => return Err("session name cannot be empty".to_string()), // i18n-english
        Some(name) => SessionArg::Named(gitpoulpi_config::sanitize_session_name(&name)),
        None => SessionArg::Default,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_flag_is_recognized() {
        for flag in ["-V", "--version"] {
            assert_eq!(
                parse_session_arg([flag].into_iter()),
                Ok(SessionArg::Version)
            );
        }
    }

    #[test]
    fn no_arguments_means_derive_the_default() {
        assert_eq!(parse_session_arg([].into_iter()), Ok(SessionArg::Default));
    }

    #[test]
    fn both_spellings_of_the_flag_are_accepted() {
        for flag in ["-s", "--session"] {
            assert_eq!(
                parse_session_arg([flag, "work"].into_iter()),
                Ok(SessionArg::Named("work".to_string())),
                "{flag}"
            );
        }
        assert_eq!(
            parse_session_arg(["--session=work"].into_iter()),
            Ok(SessionArg::Named("work".to_string()))
        );
    }

    #[test]
    fn the_name_is_sanitised_before_it_ever_becomes_a_filename() {
        assert_eq!(
            parse_session_arg(["-s", "feature/login"].into_iter()),
            Ok(SessionArg::Named("feature-login".to_string()))
        );
    }

    #[test]
    fn a_flag_with_no_value_is_an_error_not_a_silent_default() {
        assert!(parse_session_arg(["-s"].into_iter()).is_err());
        assert!(parse_session_arg(["-s", ""].into_iter()).is_err());
    }

    #[test]
    fn unknown_arguments_are_rejected() {
        assert!(parse_session_arg(["--sesion", "work"].into_iter()).is_err());
        assert!(parse_session_arg(["work"].into_iter()).is_err());
    }

    #[test]
    fn cwd_file_is_taken_out_before_the_session_parse() {
        let mut args: Vec<String> = ["-s", "w", "--cwd-file", "/tmp/x"]
            .map(String::from)
            .to_vec();
        assert_eq!(
            take_cwd_file(&mut args),
            Ok(Some(std::path::PathBuf::from("/tmp/x")))
        );
        assert_eq!(args, ["-s", "w"]);
        let mut args = vec!["--cwd-file=/tmp/y".to_string()];
        assert_eq!(
            take_cwd_file(&mut args),
            Ok(Some(std::path::PathBuf::from("/tmp/y")))
        );
        assert!(args.is_empty());
        assert!(take_cwd_file(&mut vec!["--cwd-file".to_string()]).is_err());
    }

    #[test]
    fn repo_is_taken_out_before_the_session_parse() {
        let mut args: Vec<String> = ["--repo", "/r", "-s", "w"].map(String::from).to_vec();
        assert_eq!(
            take_repo(&mut args),
            Ok(Some(std::path::PathBuf::from("/r")))
        );
        assert_eq!(args, ["-s", "w"]);
        let mut args = vec!["--repo=/x".to_string()];
        assert_eq!(
            take_repo(&mut args),
            Ok(Some(std::path::PathBuf::from("/x")))
        );
        assert!(take_repo(&mut vec!["--repo".to_string()]).is_err());
    }

    #[test]
    fn launcher_flag_is_taken_out_before_the_session_parse() {
        let mut args: Vec<String> = ["-s", "w", "--launcher"].map(String::from).to_vec();
        assert!(take_flag(&mut args, "--launcher"));
        assert_eq!(args, ["-s", "w"]);
        assert!(!take_flag(&mut args, "--launcher"));
    }

    #[test]
    fn add_takes_an_optional_folder_and_leaves_the_other_options_alone() {
        let words = |a: &[&str]| a.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let path = |p: &str| Some(Some(std::path::PathBuf::from(p)));
        let mut a = words(&["-a"]);
        assert_eq!(take_add(&mut a), Some(None));
        assert!(a.is_empty());
        let mut a = words(&["--add", "../x", "-s", "w"]);
        assert_eq!(take_add(&mut a), path("../x"));
        assert_eq!(a, ["-s", "w"]);
        // A following option is not the folder.
        let mut a = words(&["-a", "-s", "w"]);
        assert_eq!(take_add(&mut a), Some(None));
        assert_eq!(a, ["-s", "w"]);
        let mut a = words(&["--add=/p/q"]);
        assert_eq!(take_add(&mut a), path("/p/q"));
        let mut a = words(&["-s", "w"]);
        assert_eq!(take_add(&mut a), None);
        assert_eq!(a, ["-s", "w"]);
    }

    #[test]
    fn help_is_recognized() {
        assert_eq!(parse_session_arg(["-h"].into_iter()), Ok(SessionArg::Help));
        assert_eq!(
            parse_session_arg(["--help"].into_iter()),
            Ok(SessionArg::Help)
        );
    }
}
