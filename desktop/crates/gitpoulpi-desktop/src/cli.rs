//! The command line: `gitpoulpi-desktop [REPO] [--theme SLUG] [--lane-theme SLUG] [--lang CODE] [--session NAME]`.

use std::path::PathBuf;

use gitpoulpi_i18n::Lang;

use crate::app::Overrides;

#[derive(Debug, PartialEq)]
pub struct Args {
    /// Repository (or a folder inside one) to open; the current directory when absent.
    pub repo: PathBuf,
    pub overrides: Overrides,
    /// The daemon session to join; the repository's default session (the one the terminal UI and the
    /// CLI use for it) when absent.
    pub session: Option<String>,
}

/// Parses the arguments after the program name. An unknown option or a missing value is an error
/// message ready to print.
pub fn parse(
    args: impl IntoIterator<Item = String>,
    default_repo: PathBuf,
) -> Result<Args, String> {
    let mut out = Args {
        repo: default_repo,
        overrides: Overrides::default(),
        session: None,
    };
    let mut it = args.into_iter();
    while let Some(arg) = it.next() {
        let mut value = |name: &str| it.next().ok_or_else(|| format!("{name} needs a value"));
        match arg.as_str() {
            "--theme" => out.overrides.theme = Some(value("--theme")?),
            "--lane-theme" => out.overrides.lane_theme = Some(value("--lane-theme")?),
            "--session" => {
                out.session = Some(gitpoulpi_config::sanitize_session_name(&value(
                    "--session",
                )?))
            }
            "--lang" => {
                let code = value("--lang")?;
                out.overrides.lang =
                    Some(Lang::parse(&code).ok_or_else(|| format!("unknown language: {code}"))?);
            }
            other if other.starts_with("--") => return Err(format!("unknown option: {other}")),
            _ => out.repo = PathBuf::from(arg),
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_strs(args: &[&str]) -> Result<Args, String> {
        parse(args.iter().map(|s| s.to_string()), PathBuf::from("."))
    }

    #[test]
    fn no_arguments_opens_the_default_repository() {
        let a = parse_strs(&[]).unwrap();
        assert_eq!(a.repo, PathBuf::from("."));
        assert_eq!(a.overrides, Overrides::default());
    }

    #[test]
    fn options_and_a_path_in_any_order() {
        let a = parse_strs(&["--theme", "nord", "/some/repo", "--lang", "fr"]).unwrap();
        assert_eq!(a.repo, PathBuf::from("/some/repo"));
        assert_eq!(a.overrides.theme.as_deref(), Some("nord"));
        assert_eq!(a.overrides.lang, Some(Lang::Fr));
    }

    #[test]
    fn a_session_can_be_named() {
        assert_eq!(parse_strs(&[]).unwrap().session, None);
        let a = parse_strs(&["--session", "work"]).unwrap();
        assert_eq!(a.session.as_deref(), Some("work"));
        assert!(parse_strs(&["--session"]).is_err());
    }

    #[test]
    fn bad_input_is_reported() {
        assert!(parse_strs(&["--theme"]).is_err());
        assert!(parse_strs(&["--nope"]).is_err());
        assert!(parse_strs(&["--lang", "xx-not-a-language"]).is_err());
    }
}
