//! The terminal run mode of a local agent: the agent's interactive screen
//! driven in a virtual terminal, for agents whose headless mode is billed or
//! limited differently from their interactive one (what a provider does with
//! either is the provider's business; GitPoulpi only reproduces what a person
//! typing would do).
//!
//! One run: spawn the agent on a pty in the chosen working folder, wait for
//! its prompt box, paste the prompt followed by a sentinel instruction, wait
//! for the answer, close the agent. The answer comes from Claude Code's own
//! session transcript ([`transcript`]) or from the screen between two marker
//! lines ([`extract`]). **First-run dialogs (folder trust, login, theme) are
//! never answered by GitPoulpi**: the run stops with
//! [`Error::AgentSetup`](crate::Error::AgentSetup) and the user answers them
//! once by hand ("Open the agent").
//!
//! At most two terminal sessions run at once (each is a whole agent
//! process). Dropping the future cancels the run within about 50 ms.

pub mod extract;
pub mod transcript;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use gitpoulpi_config::{AgentWorkdir, AiProfile};
use gitpoulpi_pty::{PtySession, SpawnOptions};

use crate::presets::{self, TerminalExtract, TerminalSpec};
use crate::{AiClient, Error, Result};

const COLS: u16 = 220;
/// Tall on purpose: the answer is read from the screen, and the history a
/// `vt100` screen can look back at is one screenful (see `full_lines`).
const ROWS: u16 = 1000;
const SCROLLBACK: usize = 1000;
/// How long the agent has to show its prompt box.
const STARTUP: Duration = Duration::from_secs(30);
/// How long the screen must hold still before an answer is trusted.
const SETTLE: Duration = Duration::from_millis(1500);
const POLL: Duration = Duration::from_millis(50);
/// The paste is cut into pieces of this size.
const CHUNK: usize = 4096;
/// The grace the agent gets to leave after its exit command.
const EXIT_GRACE: Duration = Duration::from_secs(2);

static SLOTS: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(2);

/// The folder GitPoulpi owns for agents run in the `dedicated` mode.
pub fn dedicated_dir() -> PathBuf {
    gitpoulpi_config::ConfigPaths::discover()
        .map(|p| p.data_dir().join("agent-terminal"))
        .unwrap_or_else(|| std::env::temp_dir().join("gitpoulpi-agent-terminal"))
}

/// The working folder `profile` runs in for an action on `repo_root`.
///
/// `default` is the process's folder for a headless run (`None`) and the
/// dedicated folder for a terminal run.
pub fn workdir_for(profile: &AiProfile, repo_root: Option<&Path>) -> Option<PathBuf> {
    let repo = repo_root.filter(|p| !p.as_os_str().is_empty());
    match profile.workdir {
        AgentWorkdir::Repository => repo
            .map(Path::to_path_buf)
            .or_else(|| Some(dedicated_dir())),
        AgentWorkdir::Dedicated => Some(dedicated_dir()),
        AgentWorkdir::Default => match profile.run_mode {
            gitpoulpi_config::AgentRunMode::Headless => None,
            gitpoulpi_config::AgentRunMode::Terminal => Some(dedicated_dir()),
        },
    }
}

/// A short random hex string (a nonce for the markers).
pub(crate) fn random_hex(bytes: usize) -> String {
    use std::hash::{BuildHasher, Hasher};
    let mut out = String::new();
    while out.len() < bytes * 2 {
        let mut h = std::collections::hash_map::RandomState::new().build_hasher();
        h.write_u128(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_nanos()),
        );
        out.push_str(&format!("{:016x}", h.finish()));
    }
    out.truncate(bytes * 2);
    out
}

/// A random version-4 UUID (a Claude Code session id).
fn random_uuid() -> String {
    let h = random_hex(16);
    let variant = ['8', '9', 'a', 'b'][usize::from(h.as_bytes()[16] % 4)];
    format!(
        "{}-{}-4{}-{variant}{}-{}",
        &h[0..8],
        &h[8..12],
        &h[13..16],
        &h[17..20],
        &h[20..32]
    )
}

struct CancelOnDrop(Arc<AtomicBool>);

impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

/// Everything one run needs, owned so it can move to a blocking thread.
#[derive(Clone)]
struct Run {
    agent: String,
    program: PathBuf,
    args: Vec<String>,
    env: Vec<(String, String)>,
    timeout: Duration,
    spec: &'static TerminalSpec,
    workdir: PathBuf,
}

pub struct TerminalAgentClient {
    run: Run,
}

impl TerminalAgentClient {
    /// `repo_root` is the repository the action runs in (the `repository`
    /// working folder).
    pub fn from_profile(profile: &AiProfile, repo_root: Option<&Path>) -> Result<Self> {
        if profile.command.trim().is_empty() {
            return Err(Error::BadProfile("the local agent has no command".into()));
        }
        let harness = presets::harness_of(profile);
        let spec = harness.and_then(|h| h.terminal()).ok_or_else(|| {
            Error::BadProfile(
                "this agent type has no interactive (virtual terminal) mode: use headless".into(),
            )
        })?;
        let mut args = profile.terminal_args.clone();
        if args.is_empty() {
            if let Some(h) = harness {
                args = h.default_args_for(gitpoulpi_config::AgentRunMode::Terminal);
            }
        }
        let env = profile
            .env
            .iter()
            .filter(|(k, _)| !k.trim().is_empty())
            .map(|(k, v)| {
                (
                    k.trim().to_string(),
                    crate::expand_env_refs(v, |n| std::env::var(n).ok()),
                )
            })
            .collect();
        let mut terminal_profile = profile.clone();
        terminal_profile.run_mode = gitpoulpi_config::AgentRunMode::Terminal;
        Ok(Self {
            run: Run {
                agent: harness.map_or_else(|| profile.command.clone(), |h| h.name.to_string()),
                program: crate::resolve_command(profile.command.trim()),
                args,
                env,
                timeout: Duration::from_secs(terminal_profile.effective_timeout_secs()),
                spec,
                workdir: workdir_for(&terminal_profile, repo_root).unwrap_or_else(dedicated_dir),
            },
        })
    }

    pub fn with_workdir(mut self, dir: PathBuf) -> Self {
        self.run.workdir = dir;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.run.timeout = timeout;
        self
    }
}

#[async_trait::async_trait]
impl AiClient for TerminalAgentClient {
    async fn complete(&self, system: &str, user: &str) -> Result<String> {
        let _slot = SLOTS
            .acquire()
            .await
            .map_err(|e| Error::Agent(e.to_string()))?;
        let cancel = Arc::new(AtomicBool::new(false));
        let _guard = CancelOnDrop(Arc::clone(&cancel));
        let run = self.run.clone();
        let prompt = format!("{system}\n\n---\n\n{user}\n");
        tokio::task::spawn_blocking(move || run.execute(&prompt, &cancel))
            .await
            .map_err(|e| Error::Agent(e.to_string()))?
    }
}

/// Deletes the Claude transcript and ends the session however the run ends.
struct Session {
    pty: PtySession,
    transcript_root: Option<(PathBuf, String)>,
    exit_input: &'static str,
}

impl Session {
    /// Asks the agent to leave, gives it a moment, then kills it.
    fn close(mut self) {
        if self.pty.exited().is_none() {
            let _ = self.pty.write(self.exit_input.as_bytes());
            std::thread::sleep(Duration::from_millis(100));
            let _ = self.pty.write(b"\r");
            let deadline = Instant::now() + EXIT_GRACE;
            while self.pty.exited().is_none() && Instant::now() < deadline {
                std::thread::sleep(POLL);
            }
        }
        self.pty.stop();
        if let Some((root, id)) = &self.transcript_root {
            if let Some(file) = transcript::find(root, id) {
                let _ = std::fs::remove_file(file);
            }
        }
    }
}

impl Run {
    fn fail(&self, what: &str, pty: &PtySession) -> Error {
        let screen = pty.last_screen_lines(20).join("\n");
        Error::Agent(format!(
            "{} (virtual terminal): {what}. Last lines of its screen:\n{screen}",
            self.agent
        ))
    }

    fn execute(&self, prompt: &str, cancel: &AtomicBool) -> Result<String> {
        std::fs::create_dir_all(&self.workdir).map_err(|e| {
            Error::Agent(format!(
                "could not create the working folder {}: {e}",
                self.workdir.display()
            ))
        })?;
        let start = Instant::now();
        let deadline = start + self.timeout;
        let nonce = random_hex(6);

        let mut args = self.args.clone();
        let mut session_id = None;
        if self.spec.extract == TerminalExtract::ClaudeTranscript {
            let id = random_uuid();
            args.push("--session-id".into());
            args.push(id.clone());
            session_id = Some(id);
        }
        let mut options = SpawnOptions::new(&self.program);
        options.args = args.iter().map(Into::into).collect();
        options.cwd = Some(self.workdir.clone());
        options.env = self.env.iter().map(|(k, v)| (k.into(), v.into())).collect();
        options.cols = COLS;
        options.rows = ROWS;
        options.scrollback = SCROLLBACK;
        let pty = PtySession::spawn(options).map_err(|e| {
            Error::Agent(format!(
                "could not start `{}`: {e} (is it installed and on your PATH?)",
                self.program.display()
            ))
        })?;
        let transcript_root = session_id
            .as_ref()
            .and_then(|id| transcript::config_dir(&self.env).map(|dir| (dir, id.clone())));
        let session = Session {
            pty,
            transcript_root,
            exit_input: self.spec.exit_input,
        };
        let result = self.drive(
            &session,
            prompt,
            &nonce,
            session_id.as_deref(),
            cancel,
            deadline,
        );
        session.close();
        result
    }

    fn drive(
        &self,
        session: &Session,
        prompt: &str,
        nonce: &str,
        session_id: Option<&str>,
        cancel: &AtomicBool,
        deadline: Instant,
    ) -> Result<String> {
        let pty = &session.pty;
        let timed_out = |what: &str| {
            self.fail(
                &format!("{what} within {} seconds", self.timeout.as_secs()),
                pty,
            )
        };

        // 1. Wait for the prompt box; stop at a first-run dialog.
        let startup_end = (Instant::now() + STARTUP).min(deadline);
        loop {
            if cancel.load(Ordering::Relaxed) {
                return Err(Error::Agent("canceled".into()));
            }
            let screen = pty.screen_text();
            if let Some(found) = extract::blocker(&screen, self.spec.blockers) {
                return Err(Error::AgentSetup {
                    agent: self.agent.clone(),
                    folder: self.workdir.display().to_string(),
                    detail: found.to_string(),
                });
            }
            if extract::any_marker(&screen, self.spec.ready)
                && pty.idle_for() >= Duration::from_millis(400)
            {
                break;
            }
            if let Some(status) = pty.exited() {
                return Err(self.fail(&format!("it exited before it was ready ({status})"), pty));
            }
            if Instant::now() >= startup_end {
                return Err(timed_out("it did not show its prompt"));
            }
            std::thread::sleep(POLL);
        }

        // 2. Deliver the prompt and the sentinel instruction.
        let text = format!("{}\n\n{}", prompt.trim_end(), extract::instruction(nonce));
        self.deliver(pty, &text, cancel)?;

        // 3. Wait for the answer.
        loop {
            if cancel.load(Ordering::Relaxed) {
                return Err(Error::Agent("canceled".into()));
            }
            let screen = pty.screen_text();
            let busy = extract::any_marker(&screen, self.spec.busy);
            let settled = !busy && pty.idle_for() >= SETTLE;
            if settled {
                if let (Some((root, _)), Some(id)) = (&session.transcript_root, session_id) {
                    // Not written (yet, or never): the screen is the
                    // fallback below.
                    if let Some(file) = transcript::find(root, id) {
                        if let Some(turn) = std::fs::read_to_string(file)
                            .ok()
                            .and_then(|raw| transcript::last_turn(&raw))
                            .filter(|t| t.complete)
                        {
                            return Ok(framed_or_whole(&turn.text, nonce));
                        }
                    }
                }
                if let Some(found) = extract::answer(&pty.full_lines(), nonce) {
                    if found.trim().is_empty() {
                        return Err(Error::Empty);
                    }
                    return Ok(found);
                }
            }
            if let Some(status) = pty.exited() {
                return Err(self.fail(&format!("it exited before it answered ({status})"), pty));
            }
            if Instant::now() >= deadline {
                return Err(timed_out("it did not answer"));
            }
            std::thread::sleep(POLL);
        }
    }

    /// Types or pastes `text`, then Enter.
    fn deliver(&self, pty: &PtySession, text: &str, cancel: &AtomicBool) -> Result<()> {
        let io = |e: std::io::Error| Error::Agent(format!("could not type into the agent: {e}"));
        if pty.bracketed_paste() {
            pty.write(b"\x1b[200~").map_err(io)?;
            for chunk in chunks(&text.replace("\r\n", "\n")) {
                pty.write(chunk.as_bytes()).map_err(io)?;
            }
            pty.write(b"\x1b[201~").map_err(io)?;
            std::thread::sleep(Duration::from_millis(200));
        } else {
            // No bracketed paste: a newline would submit, so lines are
            // joined with Alt+Enter, which agents take as "new line".
            let typed = text.replace("\r\n", "\n").replace('\n', "\x1b\r");
            for chunk in chunks(&typed) {
                if cancel.load(Ordering::Relaxed) {
                    return Err(Error::Agent("canceled".into()));
                }
                pty.write(chunk.as_bytes()).map_err(io)?;
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        pty.write(b"\r").map_err(io)
    }
}

/// `text` cut into pieces of about [`CHUNK`] bytes on character boundaries.
fn chunks(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = text;
    while !rest.is_empty() {
        let mut end = CHUNK.min(rest.len());
        while !rest.is_char_boundary(end) {
            end -= 1;
        }
        let (head, tail) = rest.split_at(end);
        out.push(head);
        rest = tail;
    }
    out
}

/// The text between the markers when the agent kept them, else the whole
/// text (a transcript is exact, so nothing needs guessing).
fn framed_or_whole(text: &str, nonce: &str) -> String {
    let lines: Vec<String> = text.lines().map(str::to_string).collect();
    extract::answer(&lines, nonce).unwrap_or_else(|| text.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunks_cut_on_character_boundaries_and_lose_nothing() {
        let text = "\u{e9}".repeat(5000);
        let parts = chunks(&text);
        assert!(parts.len() >= 3);
        assert!(parts.iter().all(|p| p.len() <= CHUNK));
        assert_eq!(parts.concat(), text);
    }

    #[test]
    fn a_uuid_has_the_v4_shape() {
        let id = random_uuid();
        let parts: Vec<&str> = id.split('-').collect();
        assert_eq!(
            parts.iter().map(|p| p.len()).collect::<Vec<_>>(),
            [8, 4, 4, 4, 12]
        );
        assert!(parts[2].starts_with('4'));
        assert!(id.chars().all(|c| c == '-' || c.is_ascii_hexdigit()));
        assert_ne!(random_uuid(), random_uuid());
    }

    #[test]
    fn the_working_folder_follows_the_profile() {
        use gitpoulpi_config::{AgentRunMode, AiProviderKind};
        let mut p = AiProfile::new("x", "X", AiProviderKind::LocalAgent);
        let repo = Path::new("/some/repo");
        // Headless keeps today's behaviour by default.
        assert_eq!(workdir_for(&p, Some(repo)), None);
        p.workdir = AgentWorkdir::Repository;
        assert_eq!(workdir_for(&p, Some(repo)), Some(repo.to_path_buf()));
        // No repository known: the dedicated folder.
        assert_eq!(workdir_for(&p, None), Some(dedicated_dir()));
        p.workdir = AgentWorkdir::Default;
        p.run_mode = AgentRunMode::Terminal;
        assert_eq!(workdir_for(&p, Some(repo)), Some(dedicated_dir()));
    }
}
