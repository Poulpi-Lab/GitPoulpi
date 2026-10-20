//! External scripts (slice S11): a shell, Python or Lua script run by an
//! interpreter installed on the machine, speaking the same contract as the
//! embedded Lua (`contract.rs`) as JSON Lines: one request per line on stdin,
//! one answer per line on stdout, the carried `state` inside the requests.
//!
//! **Not sandboxed: it runs with the user's rights.** What this module does
//! to keep that honest and bounded:
//!
//! - one long-lived process per pass (a fresh one for each pass, so a script
//!   starts with an empty `state`), fed by a writer thread and read by a
//!   reader thread so no pipe can block the engine;
//! - a per-request timeout and a total timeout, polled together with the
//!   job's cancel flag every 50 ms;
//! - on a timeout, a failure, a cancel or the end of the pass the **whole
//!   process tree** is killed (Unix: the process group; Windows: a Job
//!   Object, `tree_kill.rs`), then reaped, then the temporary directory is
//!   removed: nothing is left running;
//! - the environment is cleared except [`ENV_ALLOW`]; the working directory
//!   is a fresh temporary directory holding the script's own copy (the very
//!   bytes that were hashed and trusted, never the file that may change
//!   under us); the repository's path is not given;
//! - stderr is captured (64 KiB, 1 KiB per line) and returned as the commit's
//!   log lines; stdout is the protocol, so a script prints its diagnostics to
//!   stderr.

use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use gitpoulpi_core::Error;
use serde_json::Value;

use crate::tools::retry_busy;
use crate::tree_kill::TreeGuard;

/// The only variables the script sees (when the parent has them).
pub(super) const ENV_ALLOW: &[&str] = &[
    "PATH",
    "HOME",
    "USERPROFILE",
    "SYSTEMROOT",
    "TEMP",
    "TMP",
    "LANG",
];

/// How much stderr is kept per pass.
const STDERR_CAP: usize = 64 * 1024;
/// How much of one stderr line is kept.
const STDERR_LINE_CAP: usize = 1024;
/// The longest answer line read (a line is one commit's answer).
const MAX_LINE: usize = 16 * 1024 * 1024;
/// How often a wait looks at the cancel flag and the clocks.
const POLL: Duration = Duration::from_millis(50);
/// How long a script gets to exit by itself once its stdin is closed.
const EXIT_GRACE: Duration = Duration::from_millis(1000);

/// What to run, built by `stage.rs` from the tool probe.
#[derive(Debug, Clone)]
pub(super) struct ExternalSpec {
    pub program: PathBuf,
    /// Arguments before the script file (`-I -u`, `-NoProfile -File`...).
    pub lead_args: Vec<String>,
    /// The file name of the script's copy in the working directory (the
    /// extension matters to PowerShell and to humans reading a traceback).
    pub file_name: String,
    pub text: String,
    pub timeout: Duration,
    pub total: Duration,
}

/// Why a call did not produce an answer.
#[derive(Debug)]
pub(super) enum Fail {
    /// The script is at fault (timeout, crash, bad output): `ScriptFailed`.
    Script(String),
    /// The job was canceled, or the engine failed.
    Engine(Error),
}

enum Line {
    Text(String),
    TooLong,
    Eof,
}

#[derive(Default)]
struct StderrBuf {
    lines: Vec<String>,
    bytes: usize,
    truncated: bool,
}

/// A running script process and everything that must die with it.
pub(super) struct ExternalProc {
    child: Child,
    guard: TreeGuard,
    to_script: Option<Sender<String>>,
    from_script: Receiver<Line>,
    stderr: Arc<Mutex<StderrBuf>>,
    dir: PathBuf,
    started: Instant,
    timeout: Duration,
    total: Duration,
    ended: bool,
}

static SEQ: AtomicU64 = AtomicU64::new(0);

fn make_dir() -> std::io::Result<PathBuf> {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let dir = std::env::temp_dir().join(format!(
        "gitpoulpi-script-{}-{}-{nanos}",
        std::process::id(),
        SEQ.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(dir)
}

impl ExternalProc {
    /// Starts the interpreter on the script's copy.
    pub(super) fn start(spec: &ExternalSpec) -> Result<Self, String> {
        let dir = make_dir().map_err(|e| format!("cannot create a working directory: {e}"))?;
        let script = dir.join(&spec.file_name);
        if let Err(e) = std::fs::write(&script, &spec.text) {
            let _ = std::fs::remove_dir_all(&dir);
            return Err(format!("cannot write the script: {e}"));
        }
        let mut cmd = Command::new(&spec.program);
        cmd.args(&spec.lead_args)
            // The bare file name, relative to `current_dir`: an absolute
            // Windows path (backslashes) is mangled by a POSIX shell.
            .arg(&spec.file_name)
            .current_dir(&dir)
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for name in ENV_ALLOW {
            if let Some(value) = std::env::var_os(name) {
                cmd.env(name, value);
            }
        }
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0);
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            // CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW
            cmd.creation_flags(0x0000_0200 | 0x0800_0000);
        }
        let mut child = match retry_busy(|| cmd.spawn()) {
            Ok(child) => child,
            Err(e) => {
                let _ = std::fs::remove_dir_all(&dir);
                return Err(format!("cannot start {}: {e}", spec.program.display()));
            }
        };
        let guard = TreeGuard::for_pid(child.id());
        let (stdin, stdout, stderr) =
            (child.stdin.take(), child.stdout.take(), child.stderr.take());
        let (to_script, rx_in) = mpsc::channel::<String>();
        if let Some(mut stdin) = stdin {
            std::thread::spawn(move || {
                for line in rx_in {
                    if stdin.write_all(line.as_bytes()).is_err()
                        || stdin.write_all(b"\n").is_err()
                        || stdin.flush().is_err()
                    {
                        break;
                    }
                }
            });
        }
        let (tx_out, from_script) = mpsc::channel::<Line>();
        if let Some(stdout) = stdout {
            std::thread::spawn(move || read_lines(stdout, &tx_out));
        }
        let buf = Arc::new(Mutex::new(StderrBuf::default()));
        if let Some(stderr) = stderr {
            let buf = Arc::clone(&buf);
            std::thread::spawn(move || read_stderr(stderr, &buf));
        }
        Ok(Self {
            child,
            guard,
            to_script: Some(to_script),
            from_script,
            stderr: buf,
            dir,
            started: Instant::now(),
            timeout: spec.timeout,
            total: spec.total,
            ended: false,
        })
    }

    /// Sends one request and waits for its answer: the JSON value and the
    /// stderr lines the script wrote meanwhile. `cancel` is looked at every
    /// 50 ms and ends the wait with its error.
    pub(super) fn call(
        &mut self,
        request: &Value,
        cancel: &mut dyn FnMut() -> gitpoulpi_core::Result<()>,
    ) -> Result<(Value, Vec<String>), Fail> {
        let result = self.exchange(request, cancel);
        if result.is_err() {
            // At once: a script that failed or hung gets no grace period.
            self.kill();
        }
        result
    }

    fn exchange(
        &mut self,
        request: &Value,
        cancel: &mut dyn FnMut() -> gitpoulpi_core::Result<()>,
    ) -> Result<(Value, Vec<String>), Fail> {
        let line = serde_json::to_string(request)
            .map_err(|e| Fail::Script(format!("cannot encode the request: {e}")))?;
        let sent = self
            .to_script
            .as_ref()
            .is_some_and(|tx| tx.send(line).is_ok());
        if !sent {
            return Err(Fail::Script("the script is no longer running".into()));
        }
        let deadline = Instant::now() + self.timeout;
        loop {
            cancel().map_err(Fail::Engine)?;
            if self.started.elapsed() > self.total {
                return Err(Fail::Script(format!(
                    "the script ran longer than the total timeout of {} s",
                    self.total.as_secs()
                )));
            }
            let now = Instant::now();
            if now >= deadline {
                return Err(Fail::Script(format!(
                    "the script did not answer within {} ms",
                    self.timeout.as_millis()
                )));
            }
            match self.from_script.recv_timeout(POLL.min(deadline - now)) {
                Ok(Line::Text(text)) if text.trim().is_empty() => {}
                Ok(Line::Text(text)) => {
                    let value: Value = serde_json::from_str(text.trim()).map_err(|_| {
                        Fail::Script(format!(
                            "the script printed something that is not JSON on stdout \
                             (print diagnostics to stderr): {}",
                            clip(&text, 120)
                        ))
                    })?;
                    return Ok((value, self.take_stderr()));
                }
                Ok(Line::TooLong) => {
                    return Err(Fail::Script(format!(
                        "the script answered with a line over {MAX_LINE} bytes"
                    )))
                }
                Ok(Line::Eof) | Err(RecvTimeoutError::Disconnected) => {
                    return Err(Fail::Script(self.stopped_text()));
                }
                Err(RecvTimeoutError::Timeout) => {}
            }
        }
    }

    /// "the script exited with status 3: <last stderr line>".
    fn stopped_text(&mut self) -> String {
        let deadline = Instant::now() + Duration::from_millis(500);
        let status = loop {
            match self.child.try_wait() {
                Ok(Some(status)) => break Some(status),
                Ok(None) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(10))
                }
                _ => break None,
            }
        };
        // The stderr thread may not have read the last line yet.
        let wait = Instant::now() + Duration::from_millis(300);
        while self
            .stderr
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .lines
            .is_empty()
            && Instant::now() < wait
        {
            std::thread::sleep(Duration::from_millis(10));
        }
        let last = self.take_stderr().into_iter().rev().find(|l| !l.is_empty());
        let mut text = match status {
            Some(s) => format!("the script stopped answering (it exited: {s})"),
            None => "the script stopped answering".to_string(),
        };
        if let Some(last) = last {
            text.push_str(&format!(": {last}"));
        }
        text
    }

    /// The stderr lines written since the last call, each clipped.
    fn take_stderr(&self) -> Vec<String> {
        let mut buf = self.stderr.lock().unwrap_or_else(|e| e.into_inner());
        let mut lines = std::mem::take(&mut buf.lines);
        if buf.truncated {
            lines.push("(stderr truncated at 64 KiB)".into());
            buf.truncated = false;
            buf.bytes = STDERR_CAP;
        }
        lines
    }

    /// Ends the pass: closes the script's stdin, gives it a moment to exit by
    /// itself, then kills whatever is left of its tree and removes the working
    /// directory. Idempotent; also what every failure does.
    pub(super) fn end(&mut self) {
        if self.ended {
            return;
        }
        self.ended = true;
        self.to_script = None;
        let deadline = Instant::now() + EXIT_GRACE;
        while Instant::now() < deadline {
            if !matches!(self.child.try_wait(), Ok(None)) {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        // Always, also after a clean exit: a background child of the script
        // must not outlive the pass.
        self.guard.kill_now();
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.dir);
    }

    /// Kills at once, without the grace period (a failure or a cancel).
    fn kill(&mut self) {
        if self.ended {
            return;
        }
        self.ended = true;
        self.to_script = None;
        self.guard.kill_now();
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

impl Drop for ExternalProc {
    fn drop(&mut self) {
        self.kill();
    }
}

fn read_lines(stdout: impl Read, out: &Sender<Line>) {
    let mut reader = BufReader::new(stdout);
    loop {
        let mut raw = Vec::new();
        // `take` bounds one line; a longer one is reported once and the
        // reader stops (the engine kills the script).
        let n = (&mut reader)
            .take(MAX_LINE as u64 + 1)
            .read_until(b'\n', &mut raw);
        match n {
            Ok(0) | Err(_) => {
                let _ = out.send(Line::Eof);
                return;
            }
            Ok(_) if raw.len() > MAX_LINE => {
                let _ = out.send(Line::TooLong);
                return;
            }
            Ok(_) => {
                let text = String::from_utf8_lossy(&raw);
                let text = text.trim_end_matches(['\n', '\r']).to_string();
                if out.send(Line::Text(text)).is_err() {
                    return;
                }
            }
        }
    }
}

/// Keeps draining even past the cap: a script blocked on a full stderr pipe
/// would otherwise hang.
fn read_stderr(stderr: impl Read, into: &Mutex<StderrBuf>) {
    let reader = BufReader::new(stderr);
    for raw in reader.split(b'\n') {
        let Ok(raw) = raw else { return };
        let text = String::from_utf8_lossy(&raw);
        let line = clip(text.trim_end_matches('\r'), STDERR_LINE_CAP);
        let mut buf = into.lock().unwrap_or_else(|e| e.into_inner());
        if buf.bytes + line.len() + 1 > STDERR_CAP {
            buf.truncated = true;
            continue;
        }
        buf.bytes += line.len() + 1;
        buf.lines.push(line);
    }
}

fn clip(text: &str, max: usize) -> String {
    if text.len() <= max {
        return text.to_string();
    }
    let mut end = max;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}...", &text[..end])
}

/// Where a script's copy goes, for tests and diagnostics.
#[allow(dead_code)]
pub(super) fn work_dir_of(proc: &ExternalProc) -> &Path {
    &proc.dir
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use serde_json::json;

    fn sh() -> Option<PathBuf> {
        ["/bin/sh", "/usr/bin/sh"]
            .iter()
            .map(PathBuf::from)
            .find(|p| p.exists())
    }

    fn spec(text: &str, timeout_ms: u64) -> Option<ExternalSpec> {
        Some(ExternalSpec {
            program: sh()?,
            lead_args: Vec::new(),
            file_name: "script.sh".into(),
            text: text.into(),
            timeout: Duration::from_millis(timeout_ms),
            total: Duration::from_secs(60),
        })
    }

    /// Running (a zombie waiting for its reaper is dead).
    #[cfg(target_os = "linux")]
    fn alive(pid: u32) -> bool {
        match std::fs::read_to_string(format!("/proc/{pid}/stat")) {
            Ok(stat) => stat
                .rsplit_once(')')
                .is_some_and(|(_, rest)| !rest.trim_start().starts_with('Z')),
            Err(_) => false,
        }
    }

    fn call(p: &mut ExternalProc, v: Value) -> Result<(Value, Vec<String>), Fail> {
        p.call(&v, &mut || Ok(()))
    }

    #[test]
    fn answers_line_by_line_and_the_stderr_comes_back_as_log() {
        let Some(spec) = spec(
            "while read -r line; do echo '{\"log\":\"hi\"}'; echo oops >&2; done\n",
            5000,
        ) else {
            return;
        };
        let mut p = ExternalProc::start(&spec).unwrap();
        let mut lines = Vec::new();
        for _ in 0..3 {
            let (v, log) = call(&mut p, json!({"kind": "commit"})).unwrap();
            assert_eq!(v, json!({"log": "hi"}));
            lines.extend(log);
        }
        // stdout and stderr are two pipes read by two threads, so a line the
        // script wrote to stderr just before (or after) its answer can reach
        // the buffer after that answer was taken: it comes back with the next
        // call. What must hold is that none is lost or doubled, so wait (on
        // the buffer, not on a clock) until the third has landed.
        let deadline = Instant::now() + Duration::from_secs(30);
        while lines.len() < 3 {
            assert!(
                Instant::now() < deadline,
                "only {} of 3 stderr lines arrived: {lines:?}",
                lines.len()
            );
            std::thread::sleep(Duration::from_millis(10));
            lines.extend(p.take_stderr());
        }
        assert_eq!(lines, vec!["oops"; 3]);
        p.end();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn a_timeout_kills_the_whole_tree() {
        let Some(spec) = spec(
            "sleep 300 &\necho $! > pidfile\nwhile read -r line; do sleep 300; done\n",
            300,
        ) else {
            return;
        };
        let mut p = ExternalProc::start(&spec).unwrap();
        let dir = work_dir_of(&p).to_path_buf();
        let pidfile = dir.join("pidfile");
        let waited = Instant::now();
        while !pidfile.exists()
            || std::fs::read_to_string(&pidfile)
                .unwrap_or_default()
                .is_empty()
        {
            assert!(waited.elapsed() < Duration::from_secs(5), "no pidfile");
            std::thread::sleep(Duration::from_millis(10));
        }
        let background: u32 = std::fs::read_to_string(&pidfile)
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        assert!(
            alive(background),
            "the background sleep runs before the call"
        );
        let started = Instant::now();
        let r = call(&mut p, json!({}));
        assert!(matches!(r, Err(Fail::Script(ref m)) if m.contains("did not answer")));
        assert!(started.elapsed() < Duration::from_secs(5));
        // The background `sleep` (a grandchild in the group) is gone, and so
        // is the directory.
        let waited = Instant::now();
        while alive(background) {
            assert!(
                waited.elapsed() < Duration::from_secs(5),
                "the sleep survived"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(!dir.exists(), "the working directory is removed");
    }

    #[test]
    fn the_environment_is_cleared() {
        std::env::set_var("GITPOULPI_PLANTED_SECRET", "hunter2");
        let Some(spec) = spec(
            "while read -r line; do printf '{\"log\":\"[%s]\"}\\n' \"$GITPOULPI_PLANTED_SECRET\"; done\n",
            5000,
        ) else {
            return;
        };
        let mut p = ExternalProc::start(&spec).unwrap();
        let (v, _) = call(&mut p, json!({})).unwrap();
        assert_eq!(v, json!({"log": "[]"}));
        p.end();
    }

    #[test]
    fn a_crash_is_reported_with_the_exit_status_and_the_last_stderr_line() {
        let Some(spec) = spec("read -r line\necho boom >&2\nexit 3\n", 5000) else {
            return;
        };
        let mut p = ExternalProc::start(&spec).unwrap();
        let r = call(&mut p, json!({}));
        match r {
            Err(Fail::Script(m)) => {
                assert!(m.contains("exited"), "{m}");
                assert!(m.contains("boom"), "{m}");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn output_that_is_not_json_is_a_failure_that_says_so() {
        let Some(spec) = spec("while read -r line; do echo hello; done\n", 5000) else {
            return;
        };
        let mut p = ExternalProc::start(&spec).unwrap();
        let r = call(&mut p, json!({}));
        assert!(matches!(r, Err(Fail::Script(ref m)) if m.contains("not JSON")));
    }

    #[test]
    fn cancel_ends_the_wait_and_kills() {
        let Some(spec) = spec("while read -r line; do sleep 300; done\n", 60_000) else {
            return;
        };
        let mut p = ExternalProc::start(&spec).unwrap();
        let mut n = 0;
        let r = p.call(&json!({}), &mut || {
            n += 1;
            if n > 3 {
                Err(Error::Canceled { job: 0 })
            } else {
                Ok(())
            }
        });
        assert!(matches!(r, Err(Fail::Engine(_))));
    }
}
