//! Which program this process plays. `gitpoulpi` is the one real executable;
//! the old program names (`gitpoulpi-tui`, `gitpoulpi-mcp`, `gitpoulpi-launch`)
//! are personas of it, chosen before clap runs: by the hidden first argument
//! `--gitpoulpi-as <name>` (what the stub programs pass), else by the file
//! stem of the name the process was started under (a symlink or a copy).

use std::ffi::OsStr;

/// The flag that names the persona; hidden, never in `--help`.
pub const FLAG: &str = "--gitpoulpi-as";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Persona {
    /// The scriptable CLI (and the TUI when it gets no subcommand).
    Cli,
    Tui,
    Mcp,
    Launch,
}

impl Persona {
    fn parse(name: &str) -> Option<Self> {
        match name {
            "gitpoulpi-tui" => Some(Self::Tui),
            "gitpoulpi-mcp" => Some(Self::Mcp),
            "gitpoulpi-launch" => Some(Self::Launch),
            _ => None,
        }
    }
}

/// The persona and the arguments left for it (without the program name).
/// `argv0` is the name the process was started under.
///
/// An unknown name after `--gitpoulpi-as` is an `Err` message.
pub fn choose(argv0: Option<&OsStr>, args: Vec<String>) -> Result<(Persona, Vec<String>), String> {
    if args.first().map(String::as_str) == Some(FLAG) {
        let name = args
            .get(1)
            .ok_or_else(|| format!("{FLAG} needs a program name"))?;
        let persona = Persona::parse(name).ok_or_else(|| {
            format!(
                "{FLAG}: unknown program {name:?} (gitpoulpi-tui, gitpoulpi-mcp or gitpoulpi-launch)"
            )
        })?;
        return Ok((persona, args[2..].to_vec()));
    }
    let stem = argv0
        .map(std::path::Path::new)
        .and_then(std::path::Path::file_stem)
        .and_then(OsStr::to_str)
        .map(str::to_ascii_lowercase);
    let persona = stem
        .as_deref()
        .and_then(Persona::parse)
        .unwrap_or(Persona::Cli);
    Ok((persona, args))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn the_hidden_flag_wins_and_is_stripped() {
        let got = choose(
            Some(OsStr::new("gitpoulpi")),
            words(&[FLAG, "gitpoulpi-mcp", "--http"]),
        );
        assert_eq!(got, Ok((Persona::Mcp, words(&["--http"]))));
        let got = choose(
            Some(OsStr::new("gitpoulpi-mcp")),
            words(&[FLAG, "gitpoulpi-tui", "-V"]),
        );
        assert_eq!(got, Ok((Persona::Tui, words(&["-V"]))));
    }

    #[test]
    fn the_file_stem_picks_the_persona() {
        for (name, want) in [
            ("/usr/lib/gitpoulpi/gitpoulpi-tui", Persona::Tui),
            ("gitpoulpi-mcp.exe", Persona::Mcp),
            ("GitPoulpi-Launch.EXE", Persona::Launch),
            ("gitpoulpi", Persona::Cli),
            ("gp", Persona::Cli),
        ] {
            let (persona, rest) = choose(Some(OsStr::new(name)), words(&["a"])).unwrap();
            assert_eq!(persona, want, "{name}");
            assert_eq!(rest, words(&["a"]));
        }
    }

    #[test]
    fn a_bad_name_is_an_error() {
        assert!(choose(None, words(&[FLAG])).is_err());
        assert!(choose(None, words(&[FLAG, "nope"])).is_err());
    }

    #[test]
    fn the_flag_is_only_special_as_the_first_argument() {
        let got = choose(None, words(&["log", FLAG, "gitpoulpi-mcp"]));
        assert_eq!(
            got,
            Ok((Persona::Cli, words(&["log", FLAG, "gitpoulpi-mcp"])))
        );
    }
}
