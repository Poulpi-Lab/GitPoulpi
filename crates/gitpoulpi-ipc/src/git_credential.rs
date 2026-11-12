//! The credential helper git runs for an account-authenticated transfer
//! (ADR-0010), and its wire types.
//!
//! Git is given `credential.helper = !"<this executable>"
//! --gitpoulpi-git-credential`; it appends the action (`get`, `store` or
//! `erase`) and writes its `key=value` request on stdin. Every GitPoulpi
//! binary that can host a daemon calls [`run_if_requested`] first thing in
//! `main`: when it was started that way it does this job and exits, and never
//! reaches the rest of the program.
//!
//! The helper holds **no secret**. It reads the grant id and the session name
//! from its environment (neither is secret: the id answers only over the
//! authenticated IPC, only while its operation runs, only for its host), asks
//! the session's daemon over the token handshake, and prints what the daemon
//! answers. A helper that cannot reach the daemon, or is not answered,
//! prints nothing and exits 0: git then has no credential and fails with its
//! own message (prompts are off), which the daemon turns into a typed error.

use std::io::{Read, Write};

use serde::{Deserialize, Serialize};

/// `argv[1]` of a credential-helper run.
pub const FLAG: &str = "--gitpoulpi-git-credential";
/// The grant id of the transfer (a 128-bit random hex string).
pub const ENV_GRANT: &str = "GITPOULPI_CREDENTIAL_GRANT";
/// The session whose daemon issued the grant.
pub const ENV_SESSION: &str = "GITPOULPI_CREDENTIAL_SESSION";
/// The path of the user-only file that names the issuing daemon's endpoint
/// (its port, then the IPC token): written by the daemon while the grant
/// lives, removed with it. The path is not a secret; the token is only in the
/// file. The helper prefers it to the session's published port file, because
/// the grant exists only in the daemon that issued it and an embedded daemon
/// of a short-lived CLI run publishes nothing.
pub const ENV_ENDPOINT: &str = "GITPOULPI_CREDENTIAL_ENDPOINT";

/// What [`ENV_ENDPOINT`]'s file holds: the port, a newline, the token in hex.
pub fn endpoint_file_text(endpoint: &crate::Endpoint) -> String {
    format!(
        "{}\n{}\n",
        endpoint.port,
        endpoint
            .token
            .as_ref()
            .map(|t| t.to_hex())
            .unwrap_or_default()
    )
}

/// Parses [`endpoint_file_text`]. `None` for anything else (a missing token
/// line is a daemon without tokens).
pub fn parse_endpoint_file(text: &str) -> Option<crate::Endpoint> {
    let mut lines = text.lines();
    let port: u16 = lines.next()?.trim().parse().ok()?;
    let token = lines
        .next()
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .and_then(crate::Token::from_hex);
    Some(crate::Endpoint::new(port, token))
}

/// What git asks of a helper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialAction {
    /// Git needs a credential.
    Get,
    /// The server accepted the credential git was given.
    Store,
    /// The server rejected it (a 401 or 403).
    Erase,
}

impl CredentialAction {
    pub fn parse(word: &str) -> Option<Self> {
        match word {
            "get" => Some(Self::Get),
            "store" => Some(Self::Store),
            "erase" => Some(Self::Erase),
            _ => None,
        }
    }
}

/// The helper's request to the daemon (`OperationKind::CredentialGrant`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialGrantRequest {
    pub grant: String,
    pub action: CredentialAction,
    pub protocol: String,
    /// `host` or `host:port`, as git spells it.
    pub host: String,
    #[serde(default)]
    pub path: String,
}

/// The daemon's answer: both fields empty means "no credential".
#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialGrantAnswer {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

// By hand: a derived `Debug` would print the password.
impl std::fmt::Debug for CredentialGrantAnswer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CredentialGrantAnswer")
            .field("username", &self.username)
            .field("password", &self.password.as_ref().map(|_| "***"))
            .finish()
    }
}

/// The fields of git's request that matter here.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct GitRequest {
    pub protocol: String,
    pub host: String,
    pub path: String,
}

/// Parses git's `key=value` lines up to the blank line (or the end). LF and
/// CRLF both end a line (Git for Windows' `sh` and msys pipes vary); the
/// password git sends with `store` is never read into anything.
pub fn parse_git_request(input: &str) -> GitRequest {
    let mut request = GitRequest::default();
    for line in input.lines() {
        let line = line.trim_end_matches('\r');
        if line.is_empty() {
            break;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key {
            "protocol" => request.protocol = value.to_string(),
            "host" => request.host = value.to_string(),
            "path" => request.path = value.to_string(),
            _ => {}
        }
    }
    request
}

/// What the helper prints for `get`; nothing for anything else or for no
/// credential.
pub fn format_answer(action: CredentialAction, answer: &CredentialGrantAnswer) -> String {
    match (action, &answer.username, &answer.password) {
        (CredentialAction::Get, Some(user), Some(password)) => {
            format!("username={user}\npassword={password}\n")
        }
        _ => String::new(),
    }
}

/// When this process was started as git's credential helper, serves that
/// one request and exits (never returns); otherwise returns at once. Call
/// first thing in `main`, before argument parsing: the flag is not a
/// documented option of any binary.
pub fn run_if_requested() {
    let mut args = std::env::args_os().skip(1);
    if args.next().as_deref() != Some(std::ffi::OsStr::new(FLAG)) {
        return;
    }
    let action = args
        .next()
        .and_then(|a| a.to_str().and_then(CredentialAction::parse));
    // Any failure is "no credential": exit 0 and print nothing.
    if let Some(action) = action {
        let mut input = String::new();
        let _ = std::io::stdin().take(64 * 1024).read_to_string(&mut input);
        let git = parse_git_request(&input);
        let answer = ask_daemon(action, git).unwrap_or_default();
        let text = format_answer(action, &answer);
        let mut out = std::io::stdout();
        let _ = out.write_all(text.as_bytes());
        let _ = out.flush();
    }
    std::process::exit(0);
}

/// How long the helper waits for the daemon: git is waiting for it. It must
/// outlast the longest a token command may run (30 s, `token_command::TIMEOUT`
/// in the daemon, which pins the relation with a test): a helper that gives
/// up first makes git fail with its generic "terminal prompts disabled" and
/// cuts a slow command short, instead of the command's own failure reaching
/// the user.
pub const ASK_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(40);

fn ask_daemon(action: CredentialAction, git: GitRequest) -> Option<CredentialGrantAnswer> {
    let grant = std::env::var(ENV_GRANT).ok().filter(|g| !g.is_empty())?;
    let session = std::env::var(ENV_SESSION).ok().filter(|s| !s.is_empty())?;
    // The session names a file: no path in it.
    if session.contains(['/', '\\']) || session.contains("..") {
        return None;
    }
    // The daemon that issued the grant first (it may be unpublished), then
    // whatever the session's port file names.
    let endpoint = std::env::var_os(ENV_ENDPOINT)
        .filter(|p| !p.is_empty())
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|text| parse_endpoint_file(&text))
        .or_else(|| {
            let paths = gitpoulpi_config::ConfigPaths::discover()?;
            crate::discovery::endpoint(&paths.ipc_port_file(&session))
        })?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .ok()?;
    runtime.block_on(async {
        tokio::time::timeout(ASK_TIMEOUT, async {
            let mut client = crate::Client::connect(&endpoint).await.ok()?;
            let response = client
                .call(&crate::Request {
                    id: 1,
                    session,
                    repo: None,
                    command: gitpoulpi_core::domain::OperationKind::CredentialGrant,
                    args: serde_json::to_value(CredentialGrantRequest {
                        grant,
                        action,
                        protocol: git.protocol,
                        host: git.host,
                        path: git.path,
                    })
                    .ok()?,
                })
                .await
                .ok()?;
            serde_json::from_value(response.result.ok()?).ok()
        })
        .await
        .ok()
        .flatten()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_requests_parse_with_lf_and_crlf_and_stop_at_the_blank_line() {
        let lf = parse_git_request(
            "protocol=https\nhost=git.example:8443\npath=a/b.git\nusername=x\n\nhost=ignored\n",
        );
        assert_eq!(
            lf,
            GitRequest {
                protocol: "https".into(),
                host: "git.example:8443".into(),
                path: "a/b.git".into()
            }
        );
        let crlf = parse_git_request("protocol=http\r\nhost=h\r\n\r\n");
        assert_eq!(crlf.protocol, "http");
        assert_eq!(crlf.host, "h");
        assert_eq!(parse_git_request("garbage\n"), GitRequest::default());
    }

    #[test]
    fn only_a_get_with_a_full_answer_prints_anything() {
        let full = CredentialGrantAnswer {
            username: Some("u".into()),
            password: Some("p".into()),
        };
        assert_eq!(
            format_answer(CredentialAction::Get, &full),
            "username=u\npassword=p\n"
        );
        assert_eq!(format_answer(CredentialAction::Store, &full), "");
        assert_eq!(format_answer(CredentialAction::Erase, &full), "");
        assert_eq!(
            format_answer(CredentialAction::Get, &CredentialGrantAnswer::default()),
            ""
        );
    }

    #[test]
    fn the_answer_never_prints_its_password_in_debug() {
        let answer = CredentialGrantAnswer {
            username: Some("u".into()),
            password: Some("hunter2-hunter2".into()),
        };
        assert!(!format!("{answer:?}").contains("hunter2"));
    }

    #[test]
    fn the_endpoint_file_round_trips() {
        let token = crate::Token::generate().unwrap();
        let endpoint = crate::Endpoint::new(4242, Some(token.clone()));
        let back = parse_endpoint_file(&endpoint_file_text(&endpoint)).unwrap();
        assert_eq!(back.port, 4242);
        assert_eq!(back.token.map(|t| t.to_hex()), Some(token.to_hex()));
        assert!(parse_endpoint_file("not a port\n").is_none());
    }

    #[test]
    fn actions_parse_by_git_s_words() {
        assert_eq!(CredentialAction::parse("get"), Some(CredentialAction::Get));
        assert_eq!(
            CredentialAction::parse("store"),
            Some(CredentialAction::Store)
        );
        assert_eq!(
            CredentialAction::parse("erase"),
            Some(CredentialAction::Erase)
        );
        assert_eq!(CredentialAction::parse("capability"), None);
    }
}
