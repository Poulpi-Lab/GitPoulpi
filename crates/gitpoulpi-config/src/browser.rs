//! Opening addresses and folders in the OS: the one place that launches a
//! URL handler, shared by the CLI and the TUI.
//!
//! A URL is checked before anything is spawned (scheme allowlist, no control
//! characters or whitespace, a real host) and then handed to the launcher as a
//! single argument. Windows uses `rundll32 url.dll,FileProtocolHandler`, never
//! a shell: `cmd /C start` would run the `&`, `|` or `^` of a query string.

use std::path::Path;
use std::process::{Command, Stdio};

/// Schemes [`open_url`] launches.
pub const WEB_SCHEMES: &[&str] = &["http", "https", "mailto"];

/// Why an address was not launched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenError {
    /// The address is not an allowed, well-formed URL.
    Refused(String),
    /// The OS launcher could not be started.
    Launch(String),
}

impl std::fmt::Display for OpenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenError::Refused(why) => write!(f, "refused to open the address: {why}"),
            OpenError::Launch(why) => write!(f, "could not launch a browser: {why}"),
        }
    }
}

impl std::error::Error for OpenError {}

/// Checks `url` against `schemes`: an allowed scheme, only printable ASCII
/// without whitespace or backslashes, and (for `http`/`https`) a host.
pub fn check_url(url: &str, schemes: &[&str]) -> Result<(), OpenError> {
    let refuse = |why: &str| Err(OpenError::Refused(why.to_string()));
    if url.is_empty() {
        return refuse("empty address");
    }
    if url
        .chars()
        .any(|c| !c.is_ascii() || c.is_ascii_control() || c == ' ' || c == '\\')
    {
        return refuse("it contains control characters, whitespace or a backslash");
    }
    let Some((scheme, rest)) = url.split_once(':') else {
        return refuse("no scheme");
    };
    let scheme = scheme.to_ascii_lowercase();
    if !schemes.contains(&scheme.as_str()) {
        return refuse(&format!("scheme `{scheme}` is not allowed"));
    }
    match scheme.as_str() {
        "http" | "https" => {
            let Some(after) = rest.strip_prefix("//") else {
                return refuse("missing `//` after the scheme");
            };
            let authority = after.split(['/', '?', '#']).next().unwrap_or("");
            let hostport = authority.rsplit('@').next().unwrap_or("");
            if let Some(v6) = hostport.strip_prefix('[') {
                let Some((inner, tail)) = v6.split_once(']') else {
                    return refuse("invalid host");
                };
                if inner.is_empty()
                    || !inner
                        .chars()
                        .all(|c| c.is_ascii_hexdigit() || c == ':' || c == '.')
                {
                    return refuse("invalid host");
                }
                let port_ok = tail.is_empty()
                    || tail
                        .strip_prefix(':')
                        .is_some_and(|p| p.chars().all(|c| c.is_ascii_digit()));
                if !port_ok {
                    return refuse("invalid port");
                }
            } else {
                let (host, port) = match hostport.rsplit_once(':') {
                    Some((h, p)) => (h, Some(p)),
                    None => (hostport, None),
                };
                if port.is_some_and(|p| !p.chars().all(|c| c.is_ascii_digit())) {
                    return refuse("invalid port");
                }
                let host_ok = !host.is_empty()
                    && !host.starts_with(['.', '-'])
                    && host
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_');
                if !host_ok {
                    return refuse("invalid host");
                }
            }
        }
        // `mailto:` and a scheme a caller allowed explicitly (`obsidian`):
        // the scheme and the character rules above are the whole check.
        _ => {
            if rest.is_empty() {
                return refuse("empty address");
            }
        }
    }
    Ok(())
}

/// The program and arguments that open `target` in the OS default handler.
/// Always one argument for the target, never a shell.
pub fn launcher_command(target: &str) -> (String, Vec<String>) {
    if cfg!(windows) {
        (
            "rundll32".into(),
            vec!["url.dll,FileProtocolHandler".into(), target.to_string()],
        )
    } else if cfg!(target_os = "macos") {
        ("open".into(), vec![target.to_string()])
    } else {
        ("xdg-open".into(), vec![target.to_string()])
    }
}

/// `GITPOULPI_OPEN_LOG=<file>` (append the target instead of launching) and
/// `GITPOULPI_NO_BROWSER=1` (launch nothing). `Some` when a hook handled it.
fn test_hook(target: &str) -> Option<Result<(), OpenError>> {
    if let Some(log) = std::env::var_os("GITPOULPI_OPEN_LOG").filter(|v| !v.is_empty()) {
        use std::io::Write as _;
        return Some(
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log)
                .and_then(|mut f| writeln!(f, "{target}"))
                .map_err(|e| OpenError::Launch(e.to_string())),
        );
    }
    if std::env::var_os("GITPOULPI_NO_BROWSER").is_some_and(|v| !v.is_empty()) {
        return Some(Ok(()));
    }
    None
}

fn spawn(target: &str) -> Result<(), OpenError> {
    let (program, args) = launcher_command(target);
    let cwd = std::env::current_dir().unwrap_or_else(|_| ".".into());
    Command::new(program)
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|e| OpenError::Launch(e.to_string()))
}

/// Opens a web address (`http`, `https`, `mailto`) in the default browser,
/// detached and never waited on.
pub fn open_url(url: &str) -> Result<(), OpenError> {
    open_url_with(url, WEB_SCHEMES)
}

/// [`open_url`] for a caller that builds a link of another scheme itself
/// (the `obsidian://` link of a note), with that scheme allowed.
pub fn open_url_with(url: &str, schemes: &[&str]) -> Result<(), OpenError> {
    check_url(url, schemes)?;
    match test_hook(url) {
        Some(done) => done,
        None => spawn(url),
    }
}

/// Opens a local folder or file in the OS file manager / default app. Not a
/// URL: no allowlist, but the same launcher (never `cmd /C start`).
pub fn open_path(path: &Path) -> Result<(), OpenError> {
    let target = path.to_string_lossy().into_owned();
    match test_hook(&target) {
        Some(done) => done,
        None => spawn(&target),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web_urls_pass() {
        for u in [
            "https://github.com/o/r/pull/1",
            "http://127.0.0.1:8080/x?a=1&b=2",
            "https://user@host.example/p#f",
            "https://[::1]:3000/",
            "mailto:me@example.com",
        ] {
            assert!(check_url(u, WEB_SCHEMES).is_ok(), "{u}");
        }
    }

    #[test]
    fn dangerous_or_malformed_urls_are_refused() {
        for u in [
            "javascript:alert(1)",
            "file:///etc/passwd",
            "ms-msdt:/id PCWDiagnostic",
            "ms-officecmd:x",
            "data:text/html;base64,AAAA",
            "obsidian://open?path=x",
            "-https://x",
            "https://",
            "https:///path",
            "https:example.com",
            "https://exa mple.com",
            "https://example.com/\nx",
            "https://example.com/a\\b",
            "https://exa\"mple.com",
            "https://host:port/",
            "https://.bad/",
            "https://h\u{e9}.com/",
            "C:\\Windows\\System32\\calc.exe",
            "",
        ] {
            assert!(
                matches!(check_url(u, WEB_SCHEMES), Err(OpenError::Refused(_))),
                "{u:?}"
            );
        }
    }

    #[test]
    fn extra_scheme_only_when_allowed() {
        assert!(check_url("obsidian://open?path=%2Fa", &["obsidian"]).is_ok());
        assert!(check_url("javascript:1", &["obsidian"]).is_err());
    }

    #[test]
    fn metacharacters_stay_one_argument_and_no_shell() {
        let url = "https://example.com/?a=1&b=2|calc^x";
        assert!(check_url(url, WEB_SCHEMES).is_ok());
        let (program, args) = launcher_command(url);
        assert_ne!(program, "cmd");
        assert_ne!(program, "sh");
        assert_eq!(args.last().map(String::as_str), Some(url));
        assert!(args.iter().all(|a| a != "/C" && a != "-c" && a != "start"));
        if cfg!(windows) {
            assert_eq!(program, "rundll32");
            assert_eq!(args, ["url.dll,FileProtocolHandler", url]);
        }
    }
}
