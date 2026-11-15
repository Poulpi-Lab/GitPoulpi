//! The local-agent client: an agent CLI (`claude -p`, ...) run as a
//! subprocess. The prompt goes on stdin, the answer is read from stdout.
//!
//! Dropping the future returned by [`LocalAgentClient::complete`] kills
//! the process (`kill_on_drop`), which is how a canceled request stops
//! costing anything. On Windows that is the whole tree: an agent is often a
//! `.cmd` shim that starts `node`, and killing only the shim would leave the
//! real work running and the output pipe open.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use gitpoulpi_config::AiProfile;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::{AiClient, Error, Result, DEFAULT_TIMEOUT_SECS};

/// Kills a Windows process tree when dropped (`TerminateProcess` reaches only
/// the process it is given, and `kill_on_drop` stops there). Disarmed once
/// the agent has exited by itself, so a recycled pid is never hit.
struct TreeKill(Option<u32>);

impl TreeKill {
    fn disarm(&mut self) {
        self.0 = None;
    }
}

impl Drop for TreeKill {
    fn drop(&mut self) {
        #[cfg(windows)]
        if let Some(pid) = self.0 {
            use std::os::windows::process::CommandExt;
            let _ = std::process::Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/T", "/F"])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .creation_flags(0x0800_0000)
                .status();
        }
    }
}

pub struct LocalAgentClient {
    command: String,
    args: Vec<String>,
    /// Set for this subprocess only; values are never logged (this type
    /// has no `Debug`).
    env: Vec<(String, String)>,
    timeout: Duration,
    /// The folder the agent runs in; `None` is this process's.
    workdir: Option<PathBuf>,
    /// Start without the hosting credentials of this process's environment.
    scrub_env: bool,
}

/// Variables of the environment that carry a hosting or tracker credential,
/// or a way to get one (the askpass helpers, the ssh agent): an agent that
/// reads a repository for a drafted reply does not get them, so even a
/// misconfigured one cannot publish. Exact names, plus every `BITBUCKET_*`.
const CREDENTIAL_ENV: &[&str] = &[
    "GH_TOKEN",
    "GITHUB_TOKEN",
    "GH_ENTERPRISE_TOKEN",
    "GITHUB_ENTERPRISE_TOKEN",
    "GITLAB_TOKEN",
    "GL_TOKEN",
    "CI_JOB_TOKEN",
    "AZURE_DEVOPS_EXT_PAT",
    "SYSTEM_ACCESSTOKEN",
    "JIRA_API_TOKEN",
    "JIRA_TOKEN",
    "LINEAR_API_KEY",
    "YOUTRACK_TOKEN",
    "REDMINE_API_KEY",
    "TRELLO_TOKEN",
    "TRELLO_API_KEY",
    "GITEA_TOKEN",
    "FORGEJO_TOKEN",
    "CODEBERG_TOKEN",
    "GIT_ASKPASS",
    "SSH_ASKPASS",
    "SSH_AUTH_SOCK",
];

/// The names among `names` that carry a credential (see [`CREDENTIAL_ENV`]).
pub fn credential_env<I, S>(names: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    names
        .into_iter()
        .map(|n| n.as_ref().to_string())
        .filter(|name| {
            let upper = name.to_ascii_uppercase();
            CREDENTIAL_ENV.contains(&upper.as_str()) || upper.starts_with("BITBUCKET_")
        })
        .collect()
}

impl LocalAgentClient {
    pub fn new(command: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            command: command.into(),
            args,
            env: Vec::new(),
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            workdir: None,
            scrub_env: false,
        }
    }

    pub fn from_profile(profile: &AiProfile) -> Result<Self> {
        if profile.command.trim().is_empty() {
            return Err(Error::BadProfile("the local agent has no command".into()));
        }
        let mut client = Self::new(profile.command.trim(), profile.args.clone());
        client.timeout = Duration::from_secs(profile.effective_timeout_secs());
        client.env = profile
            .env
            .iter()
            .filter(|(k, _)| !k.trim().is_empty())
            .map(|(k, v)| {
                (
                    k.trim().to_string(),
                    expand_env_refs(v, |n| std::env::var(n).ok()),
                )
            })
            .collect();
        Ok(client)
    }

    /// Extra environment variables for the subprocess (already expanded).
    pub fn with_env(mut self, env: Vec<(String, String)>) -> Self {
        self.env = env;
        self
    }

    /// Runs the agent in `dir` (`None`: this process's folder).
    pub fn in_workdir(mut self, dir: Option<PathBuf>) -> Self {
        self.workdir = dir;
        self
    }

    /// Starts the agent without the hosting credentials of this process's
    /// environment (see [`credential_env`]); variables the profile sets
    /// itself are still passed.
    pub fn scrubbing_env(mut self, scrub: bool) -> Self {
        self.scrub_env = scrub;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

#[async_trait::async_trait]
impl AiClient for LocalAgentClient {
    async fn complete(&self, system: &str, user: &str) -> Result<String> {
        let program = resolve_command(&self.command);
        let mut command = Command::new(&program);
        if self.scrub_env {
            let names = std::env::vars_os().filter_map(|(k, _)| k.into_string().ok());
            for name in credential_env(names) {
                command.env_remove(name);
            }
        }
        command
            .args(&self.args)
            .envs(self.env.iter().map(|(k, v)| (k, v)))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        if let Some(dir) = &self.workdir {
            std::fs::create_dir_all(dir).map_err(|e| {
                Error::Agent(format!(
                    "could not create the working folder {}: {e}",
                    dir.display()
                ))
            })?;
            command.current_dir(dir);
        }
        #[cfg(windows)]
        {
            // No console window flashing over the TUI.
            command.creation_flags(0x0800_0000);
        }
        let mut child = spawn_retrying_busy(&mut command).await.map_err(|e| {
            Error::Agent(format!(
                "could not start `{}`: {e} (is it installed and on your PATH?)",
                self.command
            ))
        })?;

        let mut tree = TreeKill(child.id());
        // An agent CLI has no separate system-prompt channel over stdin,
        // so the two are one document.
        let prompt = format!("{system}\n\n---\n\n{user}\n");
        let mut stdin = child.stdin.take().expect("stdin was piped");
        // Written concurrently: a big diff would otherwise fill the pipe
        // while the agent waits for us to read its output. A write error
        // (the agent answered without reading everything) is not fatal.
        let writer = tokio::spawn(async move {
            let _ = stdin.write_all(prompt.as_bytes()).await;
            let _ = stdin.shutdown().await;
        });

        let output = tokio::time::timeout(self.timeout, child.wait_with_output())
            .await
            .map_err(|_| Error::Timeout(self.timeout.as_secs()))?
            .map_err(|e| Error::Agent(e.to_string()));
        writer.abort();
        tree.disarm();
        let output = output?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stderr = stderr.trim();
            let detail = if stderr.is_empty() {
                format!("exit status {}", output.status)
            } else {
                stderr.chars().take(400).collect()
            };
            return Err(Error::Agent(detail));
        }
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if text.is_empty() {
            Err(Error::Empty)
        } else {
            Ok(text)
        }
    }
}

/// Replaces every `${env:NAME}` in `value` with `lookup(NAME)` (empty when
/// unset); anything else, including a lone `$` or an unclosed `${env:`,
/// is kept as typed.
pub fn expand_env_refs(value: &str, lookup: impl Fn(&str) -> Option<String>) -> String {
    let mut out = String::new();
    let mut rest = value;
    while let Some(at) = rest.find("${env:") {
        out.push_str(&rest[..at]);
        let after = &rest[at + 6..];
        match after.find('}') {
            Some(end) => {
                out.push_str(&lookup(after[..end].trim()).unwrap_or_default());
                rest = &after[end + 1..];
            }
            None => {
                out.push_str(&rest[at..]);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}

/// Finds the executable for `command`.
///
/// On Windows an agent CLI is often an npm shim (`claude.cmd`) that
/// `CreateProcess` will not find from a bare `claude`; this walks `PATH`
/// with the `PATHEXT` extensions like a shell. Anything that already names
/// a file, or is not found, is returned unchanged so the spawn error names
/// what the user configured.
pub fn resolve_command(command: &str) -> PathBuf {
    let direct = Path::new(command);
    if direct.components().count() > 1 || direct.is_absolute() {
        return direct.to_path_buf();
    }
    if !cfg!(windows) {
        return direct.to_path_buf();
    }
    let path = std::env::var_os("PATH").unwrap_or_default();
    let exts: Vec<String> = std::env::var("PATHEXT")
        .unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".into())
        .split(';')
        .filter(|e| !e.is_empty())
        .map(|e| e.to_ascii_lowercase())
        .collect();
    let has_ext = direct.extension().is_some();
    for dir in std::env::split_paths(&path) {
        if has_ext {
            let candidate = dir.join(command);
            if candidate.is_file() {
                return candidate;
            }
        }
        for ext in &exts {
            let candidate = dir.join(format!("{command}{ext}"));
            if candidate.is_file() {
                return candidate;
            }
        }
    }
    direct.to_path_buf()
}

/// Spawns `command`, retrying a few times on ETXTBSY ("Text file busy").
/// A script written just before it is run (a freshly generated launcher, a
/// test fixture) can still be open for writing in a concurrently forked
/// child for a moment; the condition clears on its own within milliseconds.
async fn spawn_retrying_busy(command: &mut Command) -> std::io::Result<tokio::process::Child> {
    const MAX_ATTEMPTS: u32 = 8;
    let mut attempt = 0;
    loop {
        match command.spawn() {
            Err(e) if attempt + 1 < MAX_ATTEMPTS && is_text_file_busy(&e) => {
                attempt += 1;
                tokio::time::sleep(Duration::from_millis(25 * u64::from(attempt))).await;
            }
            other => return other,
        }
    }
}

fn is_text_file_busy(e: &std::io::Error) -> bool {
    // ETXTBSY is 26 on Linux, macOS and the BSDs.
    cfg!(unix) && e.raw_os_error() == Some(26)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run<F: std::future::Future>(f: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(f)
    }

    /// Writes a fake agent that prints `stdout` and exits with `code`.
    fn fake_agent(dir: &Path, stdout: &str, code: i32) -> PathBuf {
        if cfg!(windows) {
            let path = dir.join("agent.cmd");
            std::fs::write(
                &path,
                format!("@echo off\r\necho {stdout}\r\nexit /b {code}\r\n"),
            )
            .unwrap();
            path
        } else {
            let path = dir.join("agent.sh");
            std::fs::write(
                &path,
                format!("#!/bin/sh\ncat >/dev/null\necho '{stdout}'\nexit {code}\n"),
            )
            .unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
            }
            path
        }
    }

    #[test]
    fn the_agents_stdout_is_the_answer() {
        let dir = tempfile::tempdir().unwrap();
        let agent = fake_agent(dir.path(), "feat: from the agent", 0);
        let client = LocalAgentClient::new(agent.to_string_lossy(), vec![]);
        assert_eq!(
            run(client.complete("sys", "user")).unwrap(),
            "feat: from the agent"
        );
    }

    /// A fake agent that prints the value of environment variable `var`.
    fn env_echo_agent(dir: &Path, var: &str) -> PathBuf {
        if cfg!(windows) {
            let path = dir.join("env-agent.cmd");
            std::fs::write(&path, format!("@echo off\r\nmore >nul\r\necho %{var}%\r\n")).unwrap();
            path
        } else {
            let path = dir.join("env-agent.sh");
            std::fs::write(
                &path,
                format!("#!/bin/sh\ncat >/dev/null\necho \"${var}\"\n"),
            )
            .unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
            }
            path
        }
    }

    #[test]
    fn a_profiles_env_reaches_the_subprocess_and_only_it() {
        let dir = tempfile::tempdir().unwrap();
        let agent = env_echo_agent(dir.path(), "GITPOULPI_TEST_AGENT_VAR");
        let mut profile = AiProfile {
            kind: gitpoulpi_config::AiProviderKind::LocalAgent,
            command: agent.to_string_lossy().into_owned(),
            ..AiProfile::default()
        };
        profile
            .env
            .insert("GITPOULPI_TEST_AGENT_VAR".into(), "from-the-profile".into());
        let client = LocalAgentClient::from_profile(&profile).unwrap();
        assert_eq!(run(client.complete("s", "u")).unwrap(), "from-the-profile");
        // Not leaked into this process.
        assert!(std::env::var("GITPOULPI_TEST_AGENT_VAR").is_err());
        // Without it the agent sees nothing.
        profile.env.clear();
        let client = LocalAgentClient::from_profile(&profile).unwrap();
        assert!(!run(client.complete("s", "u"))
            .map(|t| t.contains("from-the-profile"))
            .unwrap_or(false));
    }

    #[test]
    fn env_references_are_expanded_from_the_given_lookup() {
        let lookup = |n: &str| (n == "HOME").then(|| "/home/me".to_string());
        assert_eq!(expand_env_refs("${env:HOME}/x", lookup), "/home/me/x");
        assert_eq!(expand_env_refs("a${env:NOPE}b", lookup), "ab");
        assert_eq!(
            expand_env_refs("plain $ ${env:HOME", lookup),
            "plain $ ${env:HOME"
        );
        assert_eq!(expand_env_refs("${env: HOME }", lookup), "/home/me");
    }

    #[test]
    fn a_failing_agent_is_an_error_not_an_answer() {
        let dir = tempfile::tempdir().unwrap();
        let agent = fake_agent(dir.path(), "boom", 3);
        let client = LocalAgentClient::new(agent.to_string_lossy(), vec![]);
        assert!(matches!(
            run(client.complete("s", "u")),
            Err(Error::Agent(_))
        ));
    }

    #[test]
    fn a_missing_agent_says_what_was_configured() {
        let client = LocalAgentClient::new("definitely-not-an-agent-xyz", vec![]);
        let err = run(client.complete("s", "u")).unwrap_err();
        assert!(
            err.to_string().contains("definitely-not-an-agent-xyz"),
            "{err}"
        );
    }

    #[test]
    fn a_profile_without_a_command_is_refused() {
        let profile = AiProfile {
            kind: gitpoulpi_config::AiProviderKind::LocalAgent,
            ..AiProfile::default()
        };
        assert!(matches!(
            LocalAgentClient::from_profile(&profile),
            Err(Error::BadProfile(_))
        ));
    }
}
