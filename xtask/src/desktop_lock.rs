//! A mutual-exclusion lock around the *real desktop*.
//!
//! The pty driver removed the need for this in almost every case, but font
//! metrics, glyph presentation and cursor shape are questions only pixels
//! can answer (see `docs/developer/visual-verification.md`). Those answers
//! come from global state (one pointer, one keyboard focus, one screen)
//! while several agents run at once, so one can capture a sibling's window.
//!
//! Acquire this before any `wtype`/`ydotool`/`grim`/`hyprctl` sequence and
//! hold it for the whole launch-drive-capture run: a lock taken per
//! screenshot would let another agent move the pointer between the click
//! and the capture.
//!
//! The lock wraps a command rather than offering acquire/release, so the
//! release is the child exiting, whatever the reason (a lock you must
//! remember to release gets left held when a tool call times out).
//!
//! Implemented with `create_new` rather than `flock`/`LockFileEx`, which
//! keeps it free of `cfg` and identical on every platform. A hard-killed
//! holder leaves the file behind, so an entry older than `--stale` is
//! assumed abandoned and taken.

use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};

pub const USAGE: &str = "\
Usage: cargo run -p xtask -- desktop-lock [--wait <s>] [--stale <s>] -- <command> [args...]

Runs <command> while holding the machine-wide desktop lock, so two agents
cannot drive the pointer, the keyboard focus or the screen at the same
time. Needed only for real-window verification - the pty driver
(`xtask -- tui`) needs no lock at all, which is most of why it exists.

  --wait <s>   how long to wait for the lock (default 120)
  --stale <s>  age at which a held lock is assumed abandoned (default 600)";

/// Held for as long as the wrapped command runs.
#[derive(Debug)]
struct Guard {
    path: PathBuf,
}

impl Drop for Guard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

pub fn run(args: &[String]) -> Result<i32, String> {
    let mut wait = Duration::from_secs(120);
    let mut stale = Duration::from_secs(600);
    let mut rest = args.iter();
    let mut command: Vec<String> = Vec::new();

    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--wait" => {
                wait = Duration::from_secs(parse_secs(rest.next(), "--wait")?);
            }
            "--stale" => {
                stale = Duration::from_secs(parse_secs(rest.next(), "--stale")?);
            }
            "--" => {
                command.extend(rest.cloned());
                break;
            }
            "-h" | "--help" => return Err(USAGE.to_string()),
            other => return Err(format!("unexpected argument {other:?}\n\n{USAGE}")),
        }
    }

    let (program, arguments) = command
        .split_first()
        .ok_or_else(|| format!("no command given\n\n{USAGE}"))?;

    let _guard = acquire(wait, stale)?;
    let status = std::process::Command::new(program)
        .args(arguments)
        .status()
        .map_err(|e| format!("running {program}: {e}"))?;
    Ok(status.code().unwrap_or(1))
}

fn lock_path() -> PathBuf {
    // The OS temp dir, not the config dir: this is machine-wide state
    // shared by every worktree and every agent, and it must not follow a
    // per-session `GITPOULPI_CONFIG_DIR` override -- two agents with
    // different config dirs still share one pointer.
    std::env::temp_dir().join("gitpoulpi-desktop.lock")
}

fn acquire(wait: Duration, stale: Duration) -> Result<Guard, String> {
    let path = lock_path();
    let deadline = Instant::now() + wait;
    loop {
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut file) => {
                let _ = writeln!(
                    file,
                    "pid {} since {:?}",
                    std::process::id(),
                    SystemTime::now()
                );
                return Ok(Guard { path });
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                if age(&path).is_some_and(|age| age > stale) {
                    eprintln!("desktop-lock: breaking a lock older than {stale:?}");
                    let _ = std::fs::remove_file(&path);
                    continue;
                }
                if Instant::now() >= deadline {
                    let holder = std::fs::read_to_string(&path).unwrap_or_default();
                    return Err(format!(
                        "desktop-lock: still held after {wait:?} by: {}",
                        holder.trim()
                    ));
                }
                std::thread::sleep(Duration::from_millis(250));
            }
            Err(e) => return Err(format!("desktop-lock: {}: {e}", path.display())),
        }
    }
}

fn age(path: &PathBuf) -> Option<Duration> {
    std::fs::metadata(path)
        .ok()?
        .modified()
        .ok()?
        .elapsed()
        .ok()
}

fn parse_secs(value: Option<&String>, flag: &str) -> Result<u64, String> {
    value
        .ok_or_else(|| format!("{flag} needs a number of seconds"))?
        .parse()
        .map_err(|_| format!("{flag} needs a number of seconds"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_second_holder_waits_and_then_gives_up_naming_the_first() {
        let path = lock_path();
        // Serialised with the real lock file on purpose: this *is* the
        // machine-wide lock, and a test using a different path would
        // prove nothing about it.
        let first = acquire(Duration::from_secs(5), Duration::from_secs(600));
        let Ok(first) = first else {
            // Another agent genuinely holds the desktop right now; that
            // is the lock working, not a failure.
            return;
        };
        let err = acquire(Duration::from_millis(50), Duration::from_secs(600)).unwrap_err();
        assert!(err.contains("still held"), "{err}");
        assert!(err.contains(&std::process::id().to_string()), "{err}");
        drop(first);
        assert!(!path.exists(), "the lock file outlived its guard");
    }
}
