//! Storage for API tokens, keyed by the `token_ref` a
//! `[[integrations]]` entry carries.
//!
//! **Why this exists at all**: a GitHub/GitLab personal access token is a
//! password, and `config.toml` is a plaintext file people paste into bug
//! reports and sync to dotfile repositories. So the config file stores a
//! *reference* (`token_ref = "integration:my-gitlab"`) and the token
//! itself lands here.
//!
//! The one persistent implementation is [`CredentialHelperTokens`], which
//! shells to `git credential approve|fill|reject`: it is the store the
//! user already has configured and trusts (Git Credential Manager,
//! libsecret or `pass`, Keychain), it needs no cryptography or key
//! management of our own, and it is one binary invocation on every
//! platform.
//!
//! There is no encrypted-file fallback: a machine with no credential
//! helper has nowhere to put a token beyond the session's memory, and
//! [`CredentialHelperTokens::store`] says so **at the moment of storing**
//! by writing the token, reading it back and reporting failure if the
//! read does not return what was written.

use std::io::Write;
use std::process::{Command, Stdio};

use crate::{Error, Result};

/// A store of opaque secrets addressed by a `token_ref`.
///
/// A trait so tests can use [`InMemoryTokens`], which touches neither the
/// user's keychain nor their `git` configuration.
pub trait TokenStore {
    /// Stores `token` under `token_ref`, replacing anything already
    /// there. Implementations must verify the write.
    fn store(&self, token_ref: &str, token: &str) -> Result<()>;

    /// `Ok(None)` means "nothing is stored under this reference", which
    /// is a normal answer, not an error.
    fn load(&self, token_ref: &str) -> Result<Option<String>>;

    /// Removing a reference that holds nothing succeeds.
    fn delete(&self, token_ref: &str) -> Result<()>;

    /// Whether a token is present, without bringing it into memory.
    fn has(&self, token_ref: &str) -> bool {
        matches!(self.load(token_ref), Ok(Some(_)))
    }

    /// `Some(why)` when the token under `token_ref` is only held in this
    /// process's memory because the persistent store refused it (`why` is
    /// the store's error). `None` for a persisted token, or a store that
    /// has no such fallback. Never touches the OS store.
    fn session_only_reason(&self, _token_ref: &str) -> Option<String> {
        None
    }
}

/// The synthetic host a `token_ref` is mapped onto.
///
/// `git credential` addresses secrets by protocol/host/path. Helpers such
/// as Git Credential Manager only accept protocols they know (`https`),
/// so we use `https` with a host under the reserved `.invalid` TLD: it
/// can never collide with a real credential, and it parses as a normal
/// host everywhere. The reference itself goes in the percent-encoded
/// path (a raw `:` would be read as host:port), and
/// `credential.useHttpPath=true` makes helpers keep one entry per path.
const HOST: &str = "gitpoulpi.invalid";

/// The fixed username every entry is stored and looked up under.
/// Sending it on `fill` too is what stops git from asking for one.
const USERNAME: &str = "gitpoulpi";

/// Tokens the credential helper could not keep, held for this process
/// only so a machine with a broken or absent helper still works for the
/// session. Never persisted.
/// Each entry is `(token, why the helper refused it)`.
type SessionMap = std::sync::Mutex<std::collections::BTreeMap<String, (String, String)>>;

fn session() -> &'static SessionMap {
    static SESSION: std::sync::OnceLock<SessionMap> = std::sync::OnceLock::new();
    SESSION.get_or_init(Default::default)
}

/// [`TokenStore`] over the user's configured git credential helper, with
/// a session-only in-memory fallback when the helper cannot store.
#[derive(Debug, Default, Clone)]
pub struct CredentialHelperTokens {
    /// Extra `-c` settings (tests use this to pick a helper).
    config: Vec<String>,
}

impl CredentialHelperTokens {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds `-c <setting>` to every git invocation, e.g.
    /// `credential.helper=store --file X`.
    pub fn with_config(mut self, setting: impl Into<String>) -> Self {
        self.config.push(setting.into());
        self
    }

    /// True when the token is only held in memory for this session
    /// because the credential helper refused it: callers should warn the
    /// user it will be gone on restart.
    pub fn is_session_only(&self, token_ref: &str) -> bool {
        session()
            .lock()
            .expect("session map poisoned")
            .contains_key(token_ref)
    }

    /// `integration:my-gitlab` ->
    /// `protocol=https host=gitpoulpi.invalid path=integration%3Amy-gitlab
    /// username=gitpoulpi`, one field per line, no terminating blank line.
    fn fields(token_ref: &str) -> String {
        format!(
            "protocol=https\nhost={HOST}\npath={}\nusername={USERNAME}\n",
            percent_encode(token_ref)
        )
    }

    /// The request for `fill` / `reject`: the fields plus the blank line.
    fn describe(token_ref: &str) -> String {
        format!("{}\n", Self::fields(token_ref))
    }

    /// Builds the `git credential <verb>` invocation, wired so it can
    /// never pop an interactive prompt (there is nobody to answer and
    /// nothing useful a prompt could collect):
    /// `GIT_TERMINAL_PROMPT=0`, `GCM_INTERACTIVE=Never`,
    /// `GCM_GUI_PROMPT=false` and `credential.interactive=false`.
    fn command(&self, verb: &str) -> Command {
        let mut cmd = Command::new("git");
        cmd.args(["-c", "credential.interactive=false"])
            .args(["-c", "credential.useHttpPath=true"]);
        for setting in &self.config {
            cmd.args(["-c", setting]);
        }
        cmd.args(["credential", verb])
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GCM_INTERACTIVE", "Never")
            .env("GCM_GUI_PROMPT", "false");
        cmd
    }

    fn run(&self, verb: &str, input: &str) -> Result<String> {
        let mut child = self
            .command(verb)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                Error::HelperFailed(format!("could not run `git credential {verb}`: {e}"))
            })?;
        child
            .stdin
            .as_mut()
            .expect("stdin was piped")
            .write_all(input.as_bytes())
            .map_err(|e| Error::HelperFailed(e.to_string()))?;
        // Dropping the handle closes the pipe; `git credential fill`
        // blocks for more input until it sees EOF.
        drop(child.stdin.take());
        let output = child
            .wait_with_output()
            .map_err(|e| Error::HelperFailed(e.to_string()))?;
        if !output.status.success() {
            return Err(Error::HelperFailed(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ));
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    /// Stores through the helper and reads it back.
    fn store_in_helper(&self, token_ref: &str, token: &str) -> Result<()> {
        let input = format!("{}password={token}\n\n", Self::fields(token_ref));
        self.run("approve", &input)?;
        // Verify, rather than trust: with no helper configured at all,
        // `git credential approve` succeeds and stores nothing.
        match self.load_from_helper(token_ref)? {
            Some(stored) if stored == token => Ok(()),
            _ => Err(Error::HelperFailed(
                "no git credential helper appears to be configured \
                 (see `git config --global credential.helper`)"
                    .into(),
            )),
        }
    }

    fn load_from_helper(&self, token_ref: &str) -> Result<Option<String>> {
        match self.run("fill", &Self::describe(token_ref)) {
            Ok(output) => Ok(parse_password(&output)),
            // Nothing stored + prompts disabled makes git fail with
            // "could not read Username/Password ... terminal prompts
            // disabled": that is "not found", not a helper failure.
            Err(Error::HelperFailed(msg)) if is_prompt_refusal(&msg) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

fn is_prompt_refusal(stderr: &str) -> bool {
    let lower = stderr.to_ascii_lowercase();
    lower.contains("terminal prompts disabled")
        || lower.contains("could not read username")
        || lower.contains("could not read password")
        || lower.contains("cannot prompt")
        // Newer git, with `credential.interactive=false`.
        || lower.contains("unable to get password")
        || lower.contains("unable to get username")
}

impl TokenStore for CredentialHelperTokens {
    fn store(&self, token_ref: &str, token: &str) -> Result<()> {
        match self.store_in_helper(token_ref, token) {
            Ok(()) => {
                session()
                    .lock()
                    .expect("session map poisoned")
                    .remove(token_ref);
                Ok(())
            }
            // The helper cannot keep it (absent, refusing, broken): keep
            // it for this session rather than failing the connection.
            // `is_session_only` lets the UI warn that it will not survive.
            Err(e) => {
                session()
                    .lock()
                    .expect("session map poisoned")
                    .insert(token_ref.to_string(), (token.to_string(), e.to_string()));
                Ok(())
            }
        }
    }

    fn load(&self, token_ref: &str) -> Result<Option<String>> {
        if let Some(t) = session()
            .lock()
            .expect("session map poisoned")
            .get(token_ref)
        {
            return Ok(Some(t.0.clone()));
        }
        self.load_from_helper(token_ref)
    }

    fn session_only_reason(&self, token_ref: &str) -> Option<String> {
        session()
            .lock()
            .expect("session map poisoned")
            .get(token_ref)
            .map(|t| t.1.clone())
    }

    fn delete(&self, token_ref: &str) -> Result<()> {
        session()
            .lock()
            .expect("session map poisoned")
            .remove(token_ref);
        match self.run("reject", &Self::describe(token_ref)) {
            Err(Error::HelperFailed(msg)) if is_prompt_refusal(&msg) => Ok(()),
            other => other.map(|_| ()),
        }
    }
}

/// Pulls `password=` out of `git credential fill`'s key=value output.
///
/// Split on the *first* `=` only: a base64-ish token can contain one, and
/// splitting on all of them truncates the secret.
pub(crate) fn parse_password(output: &str) -> Option<String> {
    for line in output.lines() {
        if let Some((key, value)) = line.split_once('=') {
            if key == "password" && !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Minimal percent-encoding for the characters a `token_ref` can carry
/// that a URL path must not (`:` above all). Not a general-purpose
/// encoder; the input alphabet is ours.
pub(crate) fn percent_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

/// An in-memory [`TokenStore`], for tests and for a session that
/// deliberately does not want to touch the machine's credential store.
///
/// Explicitly not a persistence mechanism: dropping it forgets
/// everything, which is the point.
#[derive(Debug, Default)]
pub struct InMemoryTokens {
    entries: std::sync::Mutex<std::collections::BTreeMap<String, String>>,
}

impl TokenStore for InMemoryTokens {
    fn store(&self, token_ref: &str, token: &str) -> Result<()> {
        self.entries
            .lock()
            .expect("token map poisoned")
            .insert(token_ref.to_string(), token.to_string());
        Ok(())
    }

    fn load(&self, token_ref: &str) -> Result<Option<String>> {
        Ok(self
            .entries
            .lock()
            .expect("token map poisoned")
            .get(token_ref)
            .cloned())
    }

    fn delete(&self, token_ref: &str) -> Result<()> {
        self.entries
            .lock()
            .expect("token map poisoned")
            .remove(token_ref);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The failure this guards against is silent and remote: a token
    /// truncated at its first `=` authenticates as nothing, and the error
    /// the user sees comes from GitHub, not from here.
    #[test]
    fn a_password_containing_an_equals_sign_survives_parsing() {
        let output = "protocol=gitpoulpi\nhost=token\nusername=gitpoulpi\npassword=abc=def==\n";
        assert_eq!(parse_password(output), Some("abc=def==".to_string()));
    }

    #[test]
    fn an_empty_or_absent_password_reads_as_nothing_stored() {
        assert_eq!(parse_password("username=gitpoulpi\n"), None);
        assert_eq!(parse_password("password=\n"), None);
    }

    #[test]
    fn a_reference_is_encoded_so_git_cannot_read_it_as_a_host_and_port() {
        assert_eq!(
            percent_encode("integration:my-gitlab"),
            "integration%3Amy-gitlab"
        );
        let described = CredentialHelperTokens::describe("integration:my-gitlab");
        assert!(
            described.contains("host=gitpoulpi.invalid\n"),
            "{described}"
        );
        assert!(
            described.contains("path=integration%3Amy-gitlab\n"),
            "{described}"
        );
    }

    /// Every field is on its own line, with a username, for fill/approve
    /// alike (glued-together `path=...username=...` lines never delivered
    /// the username to git).
    #[test]
    fn the_request_is_well_formed_and_always_carries_a_username() {
        assert_eq!(
            CredentialHelperTokens::describe("integration:x"),
            "protocol=https\nhost=gitpoulpi.invalid\npath=integration%3Ax\nusername=gitpoulpi\n\n"
        );
        let approve = format!(
            "{}password=p\n\n",
            CredentialHelperTokens::fields("integration:x")
        );
        assert!(
            approve.contains("\nusername=gitpoulpi\npassword=p\n"),
            "{approve}"
        );
    }

    #[test]
    fn stored_tokens_never_address_a_real_host() {
        let described = CredentialHelperTokens::describe("integration:github");
        assert!(described.contains(".invalid"), "{described}");
        assert!(!described.contains("github.com"), "{described}");
    }

    #[test]
    fn the_command_is_built_fully_non_interactive() {
        let cmd = CredentialHelperTokens::new().command("fill");
        assert_eq!(cmd.get_program().to_string_lossy(), "git");
        let args: Vec<String> = cmd
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            args,
            vec![
                "-c",
                "credential.interactive=false",
                "-c",
                "credential.useHttpPath=true",
                "credential",
                "fill"
            ]
        );
        let envs: std::collections::HashMap<String, String> = cmd
            .get_envs()
            .map(|(k, v)| {
                (
                    k.to_string_lossy().into_owned(),
                    v.map(|v| v.to_string_lossy().into_owned())
                        .unwrap_or_default(),
                )
            })
            .collect();
        assert_eq!(
            envs.get("GIT_TERMINAL_PROMPT").map(String::as_str),
            Some("0")
        );
        assert_eq!(
            envs.get("GCM_INTERACTIVE").map(String::as_str),
            Some("Never")
        );
        assert_eq!(
            envs.get("GCM_GUI_PROMPT").map(String::as_str),
            Some("false")
        );
    }

    fn tmp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("gp-secrets-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn git_available() -> bool {
        Command::new("git").arg("--version").output().is_ok()
    }

    /// Real `git credential` + the `store` helper: store, load, replace,
    /// delete. This is the round trip the Integrations page relies on.
    #[test]
    fn round_trips_through_a_real_git_credential_helper() {
        if !git_available() {
            return;
        }
        let dir = tmp_dir("roundtrip");
        let file = dir.join("creds").to_string_lossy().replace('\\', "/");
        let tokens = CredentialHelperTokens::new()
            // An empty value resets any globally configured helper.
            .with_config("credential.helper=")
            .with_config(format!("credential.helper=store --file {file}"));
        let r = "integration:rt-gitlab-self-managed";
        assert_eq!(tokens.load(r).unwrap(), None, "nothing stored yet");
        tokens.store(r, "glpat-TEST=abc==").unwrap();
        assert!(!tokens.is_session_only(r), "helper should have kept it");
        assert_eq!(tokens.load(r).unwrap(), Some("glpat-TEST=abc==".into()));
        let raw = std::fs::read_to_string(dir.join("creds")).unwrap();
        assert!(
            raw.starts_with("https://gitpoulpi:") && raw.contains("@gitpoulpi.invalid/"),
            "entry keyed by fixed username + synthetic host"
        );
        tokens.store(r, "glpat-TEST-2").unwrap();
        assert_eq!(tokens.load(r).unwrap(), Some("glpat-TEST-2".into()));
        tokens.delete(r).unwrap();
        assert_eq!(tokens.load(r).unwrap(), None);
        tokens.delete(r).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// With nothing stored and prompts disabled git fails with "could not
    /// read Username ... terminal prompts disabled". That must read as
    /// `None`, and a helper that will not store must fall back to the
    /// session instead of failing.
    #[test]
    fn a_helper_that_cannot_store_falls_back_to_the_session() {
        if !git_available() {
            return;
        }
        // A helper that stores nothing and never answers.
        let tokens = CredentialHelperTokens::new()
            .with_config("credential.helper=")
            .with_config("credential.helper=!true");
        let r = "integration:fallback-gitlab-self-managed";
        assert_eq!(tokens.load(r).unwrap(), None);
        tokens.store(r, "glpat-TEST-session").unwrap();
        assert!(tokens.is_session_only(r));
        assert!(
            tokens
                .session_only_reason(r)
                .is_some_and(|why| !why.is_empty()),
            "the refusal is kept for the caller"
        );
        assert_eq!(tokens.load(r).unwrap(), Some("glpat-TEST-session".into()));
        tokens.delete(r).unwrap();
        assert!(!tokens.is_session_only(r));
        assert_eq!(tokens.session_only_reason(r), None);
        assert_eq!(tokens.load(r).unwrap(), None);

        // A helper that hard-fails.
        let failing = CredentialHelperTokens::new()
            .with_config("credential.helper=")
            .with_config("credential.helper=!false");
        let r2 = "integration:fallback2";
        failing.store(r2, "glpat-TEST-2").unwrap();
        assert!(failing.is_session_only(r2));
        failing.delete(r2).unwrap();
    }

    #[test]
    fn the_in_memory_store_round_trips_and_forgets_on_delete() {
        let store = InMemoryTokens::default();
        assert!(!store.has("integration:x"));
        store.store("integration:x", "glpat-secret").unwrap();
        assert_eq!(
            store.load("integration:x").unwrap(),
            Some("glpat-secret".to_string())
        );
        assert!(store.has("integration:x"));
        store.delete("integration:x").unwrap();
        assert_eq!(store.load("integration:x").unwrap(), None);
        // Deleting what is not there is not an error.
        store.delete("integration:x").unwrap();
    }
}
