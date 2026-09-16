use std::io::Write;
use std::process::{Command, Stdio};

use crate::{Credential, CredentialSource, Error, Result};

/// Shells to `git credential fill` (only `fill` is implemented; nothing is
/// approved or rejected back), reusing whatever
/// credential.helper the user already has configured (Git Credential
/// Manager on Windows, libsecret/pass-based helpers on Linux, Keychain on
/// macOS) instead of reinventing OS-specific credential storage.
///
/// ADR-0004: GitPoulpi stores no credentials of its own for hosts a helper
/// already covers, so a user who has run `git push` once has already
/// configured GitPoulpi.
pub struct CredentialHelper;

impl CredentialSource for CredentialHelper {
    /// Asks the configured helper for `host`, over git's own
    /// credential protocol.
    ///
    /// Two details that are the difference between working and hanging:
    ///
    /// - **stdin is closed after the request.** `git credential fill`
    ///   reads key=value lines until a blank line *and EOF*; leaving the
    ///   pipe open leaves it waiting.
    /// - **stderr is piped and drained, not inherited.** The TUI calls
    ///   this while it owns the alternate screen: a line written to an
    ///   inherited stderr (`fatal: could not read Username ...` when
    ///   prompts are off) lands on that screen and its newline scrolls the
    ///   terminal under the TUI, so every later click is one row off.
    ///   `wait_with_output` drains both pipes, so a chatty helper cannot
    ///   fill one and block. A helper that must ask the user does it in
    ///   its own window (GCM) or on the controlling terminal, not stderr.
    ///
    /// `protocol=https` is sent unless `host` is spelled `http://host`
    /// (a self-hosted server on plain http, whose helper entry is keyed
    /// by `http`): a helper keyed by protocol stores API tokens under the
    /// protocol the server is reached over, and this path is never used
    /// for ssh - ssh stays on the system ssh-agent and this crate never
    /// sees a private key.
    fn resolve(&self, host: &str) -> Result<Credential> {
        let host = host.trim();
        let Some(request) = fill_request(host) else {
            return Err(Error::NotFound("<empty host>".to_string()));
        };

        let mut child = Command::new("git")
            .args(["credential", "fill"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                Error::HelperFailed(format!("could not run `git credential fill`: {e}"))
            })?;

        child
            .stdin
            .take()
            .expect("stdin was piped")
            .write_all(request.as_bytes())
            .map_err(|e| Error::HelperFailed(format!("could not write to git credential: {e}")))?;

        let output = child
            .wait_with_output()
            .map_err(|e| Error::HelperFailed(format!("git credential fill failed: {e}")))?;

        // A helper with nothing stored exits non-zero (or, when
        // `credential.helper` is unset entirely, succeeds with no
        // username/password). Both are "not found" rather than an error
        // the caller should surface as a failure -- there is a fallback
        // to try.
        if !output.status.success() {
            return Err(Error::NotFound(host.to_string()));
        }

        parse_fill_output(&String::from_utf8_lossy(&output.stdout))
            .ok_or_else(|| Error::NotFound(host.to_string()))
    }
}

/// The request the helper reads for `host` (`host[:port]`, or
/// `http://host[:port]` for a plain-http server); `None` for an empty host.
fn fill_request(host: &str) -> Option<String> {
    let (protocol, host) = match host.split_once("://") {
        Some((scheme, rest)) if scheme.eq_ignore_ascii_case("http") => ("http", rest),
        Some((_, rest)) => ("https", rest),
        None => ("https", host),
    };
    let host = host.trim_end_matches('/');
    (!host.is_empty()).then(|| format!("protocol={protocol}\nhost={host}\n\n"))
}

/// Parses `git credential fill`'s `key=value` reply.
///
/// A password with `=` in it is normal (tokens contain them), so only the
/// *first* `=` separates the key; splitting on every one silently
/// truncates the token.
fn parse_fill_output(stdout: &str) -> Option<Credential> {
    let mut username = None;
    let mut password = None;
    for line in stdout.lines() {
        let line = line.trim_end_matches('\r');
        match line.split_once('=') {
            Some(("username", value)) => username = Some(value.to_string()),
            Some(("password", value)) => password = Some(value.to_string()),
            _ => {}
        }
    }
    let password = password.filter(|p| !p.is_empty())?;
    Some(Credential {
        // A token-only helper answers with no username. The hosts accept
        // any non-empty one alongside a token, and git's own convention
        // for that case is the literal `x-access-token`.
        username: username.unwrap_or_else(|| "x-access-token".to_string()),
        password,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_request_carries_the_protocol_the_server_is_reached_over() {
        assert_eq!(
            fill_request("gitlab.example.com").as_deref(),
            Some("protocol=https\nhost=gitlab.example.com\n\n")
        );
        assert_eq!(
            fill_request("http://nas:8080").as_deref(),
            Some("protocol=http\nhost=nas:8080\n\n")
        );
        assert_eq!(
            fill_request("https://nas:8443/").as_deref(),
            Some("protocol=https\nhost=nas:8443\n\n")
        );
        assert_eq!(fill_request(""), None);
        assert_eq!(fill_request("http://"), None);
    }

    #[test]
    fn a_normal_reply_is_parsed() {
        let credential = parse_fill_output(
            "protocol=https\nhost=gitlab.example.com\nusername=kai\npassword=glpat-secret\n",
        )
        .unwrap();
        assert_eq!(credential.username, "kai");
        assert_eq!(credential.password, "glpat-secret");
    }

    /// Tokens routinely contain `=` (base64 padding, for one). Splitting
    /// on every `=` silently truncates them.
    #[test]
    fn a_password_containing_equals_survives_intact() {
        let credential = parse_fill_output("username=u\npassword=abc=def==\n").unwrap();
        assert_eq!(credential.password, "abc=def==");
    }

    #[test]
    fn a_token_only_reply_gets_gits_conventional_username() {
        let credential = parse_fill_output("password=ghp_token\n").unwrap();
        assert_eq!(credential.username, "x-access-token");
        assert_eq!(credential.password, "ghp_token");
    }

    /// An empty or absent password is "nothing stored", not a
    /// credential with an empty password - which would be sent to the
    /// host and rejected with a confusing 401.
    #[test]
    fn no_password_is_no_credential() {
        assert!(parse_fill_output("username=u\n").is_none());
        assert!(parse_fill_output("username=u\npassword=\n").is_none());
        assert!(parse_fill_output("").is_none());
    }

    /// Helpers on Windows can emit CRLF.
    #[test]
    fn crlf_line_endings_are_handled() {
        let credential = parse_fill_output("username=u\r\npassword=p\r\n").unwrap();
        assert_eq!(credential.password, "p");
    }

    #[test]
    fn an_empty_host_is_rejected_without_spawning_git() {
        assert!(matches!(
            CredentialHelper.resolve("   "),
            Err(Error::NotFound(_))
        ));
    }
}
