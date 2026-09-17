//! Streaming a long git scan line by line, killable at a deadline.

use std::io::Write;
use std::path::Path;
use std::process::Stdio;

use gitpoulpi_core::Result;

use crate::process::{failed_with, git, spawn_failed};

/// How often a scan with no output checks that git is still running.
const EXIT_POLL: std::time::Duration = std::time::Duration::from_millis(50);
/// How long the output of an exited git is still collected.
const EXIT_GRACE: std::time::Duration = std::time::Duration::from_millis(300);

/// How a [`stream_lines`] run ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Streamed {
    /// git ran to the end.
    Done,
    /// The callback asked to stop; git was killed.
    Stopped,
    /// The deadline passed; git was killed.
    TimedOut,
}

/// Runs `git <args>` and hands each stdout line to `on_line` as it
/// arrives (return `false` to stop), killing git at `deadline`. For scans
/// that may be big (the lost-commit search over a whole object database):
/// nothing is buffered beyond one line, and the render thread's request
/// never waits longer than the deadline. A non-zero exit that was not
/// caused by killing git is an error.
pub(crate) fn stream_lines(
    repo_root: &Path,
    args: &[&str],
    stdin: Option<Vec<u8>>,
    deadline: Option<std::time::Instant>,
    mut on_line: impl FnMut(&str) -> bool,
) -> Result<Streamed> {
    use std::io::{BufRead, Read};
    use std::sync::{Arc, Mutex};

    let mut cmd = git(repo_root, args);
    cmd.stdin(if stdin.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    })
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());
    let mut child = cmd.spawn().map_err(spawn_failed)?;
    let writer = stdin.map(|input| {
        let mut pipe = child.stdin.take().expect("stdin was piped");
        std::thread::spawn(move || pipe.write_all(&input))
    });
    let stdout = child.stdout.take().expect("stdout was piped");
    let mut stderr = child.stderr.take().expect("stderr was piped");
    let (stderr_tx, stderr_rx) = std::sync::mpsc::channel::<String>();
    std::thread::spawn(move || {
        let mut text = String::new();
        let _ = stderr.read_to_string(&mut text);
        let _ = stderr_tx.send(text);
    });
    let child = Arc::new(Mutex::new(child));
    let timed_out = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let (done_tx, done_rx) = std::sync::mpsc::channel::<()>();
    let watchdog = deadline.map(|deadline| {
        let child = Arc::clone(&child);
        let timed_out = Arc::clone(&timed_out);
        std::thread::spawn(move || {
            let wait = deadline.saturating_duration_since(std::time::Instant::now());
            if done_rx.recv_timeout(wait).is_err() {
                timed_out.store(true, std::sync::atomic::Ordering::SeqCst);
                let _ = child.lock().unwrap_or_else(|e| e.into_inner()).kill();
            }
        })
    });

    // The lines come from a reader thread, so the end of the scan is git
    // exiting, not the pipe closing: a process that outlives git holding
    // its stdout (a handle inherited by a sibling spawn on Windows, a
    // backgrounded helper) would otherwise keep this read open until the
    // deadline killed it and the page came back empty and `partial`.
    let (line_tx, line_rx) = std::sync::mpsc::channel::<Vec<u8>>();
    std::thread::spawn(move || {
        let mut reader = std::io::BufReader::new(stdout);
        loop {
            let mut buf = Vec::new();
            match reader.read_until(b'\n', &mut buf) {
                Ok(0) | Err(_) => break,
                Ok(_) => {
                    if line_tx.send(buf).is_err() {
                        break;
                    }
                }
            }
        }
    });
    let mut stopped = false;
    let mut handle = |mut buf: Vec<u8>| -> bool {
        while matches!(buf.last(), Some(b'\n' | b'\r')) {
            buf.pop();
        }
        if on_line(&String::from_utf8_lossy(&buf)) {
            return true;
        }
        stopped = true;
        let _ = child.lock().unwrap_or_else(|e| e.into_inner()).kill();
        false
    };
    'read: loop {
        match line_rx.recv_timeout(EXIT_POLL) {
            Ok(buf) => {
                if !handle(buf) {
                    break;
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                let exited = matches!(
                    child.lock().unwrap_or_else(|e| e.into_inner()).try_wait(),
                    Ok(Some(_)) | Err(_)
                );
                if exited {
                    // What git wrote is in the pipe already: take it, then
                    // stop waiting for an end that may never come.
                    while let Ok(buf) = line_rx.recv_timeout(EXIT_GRACE) {
                        if !handle(buf) {
                            break 'read;
                        }
                    }
                    break;
                }
            }
        }
    }
    drop(line_rx);
    let status = child.lock().unwrap_or_else(|e| e.into_inner()).wait();
    let _ = done_tx.send(());
    if let Some(w) = watchdog {
        let _ = w.join();
    }
    if let Some(w) = writer {
        // A killed git does not read its input to the end.
        let _ = w.join();
    }
    let stderr = stderr_rx.recv_timeout(EXIT_GRACE).unwrap_or_default();
    let timed_out = timed_out.load(std::sync::atomic::Ordering::SeqCst);
    if stopped {
        return Ok(Streamed::Stopped);
    }
    if timed_out {
        return Ok(Streamed::TimedOut);
    }
    let status = status
        .map_err(|e| gitpoulpi_core::Error::Backend(format!("failed to wait for git: {e}")))?;
    if !status.success() {
        return Err(failed_with(args, status.code(), &stderr));
    }
    Ok(Streamed::Done)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    /// A helper that outlives git and keeps its stdout open must not hold
    /// the scan until the deadline.
    #[test]
    fn a_process_holding_the_pipe_open_does_not_hold_the_scan() {
        let dir = tempfile::tempdir().unwrap();
        let started = std::time::Instant::now();
        let mut lines = Vec::new();
        let ended = stream_lines(
            dir.path(),
            &["-c", "alias.bg=!sleep 8 & echo hi", "bg"],
            None,
            None,
            |l| {
                lines.push(l.to_string());
                true
            },
        )
        .unwrap();
        assert_eq!(ended, Streamed::Done);
        assert_eq!(lines, vec!["hi".to_string()]);
        assert!(started.elapsed() < std::time::Duration::from_secs(5));
    }

    #[test]
    fn crlf_lines_lose_their_terminators() {
        let dir = tempfile::tempdir().unwrap();
        let mut lines = Vec::new();
        stream_lines(
            dir.path(),
            &["-c", "alias.crlf=!printf 'a\\r\\nb\\r\\n'", "crlf"],
            None,
            None,
            |l| {
                lines.push(l.to_string());
                true
            },
        )
        .unwrap();
        assert_eq!(lines, vec!["a".to_string(), "b".to_string()]);
    }
}
