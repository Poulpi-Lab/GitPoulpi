//! Running a user-supplied program (not git) to its end, under the same
//! process-tree guard as every network command: the daemon's token command
//! (ADR-0010) is one. The program is run directly (never through a shell),
//! with nothing on stdin, in a given directory, and is killed together with
//! everything it started when it takes longer than the timeout or when the
//! future awaiting it is dropped.

use std::path::Path;
use std::time::Duration;

/// How a finished program ended.
#[derive(Debug, Clone)]
pub struct CommandOutput {
    /// `None` when a signal ended it.
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

/// Why a program gave no output at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandRunError {
    /// It could not be started (not found, not executable, no argv).
    Spawn(String),
    /// It ran past the timeout; it and its descendants were killed.
    TimedOut,
}

/// Runs `argv` (program first) in `cwd`, waits at most `timeout`. On a
/// timeout the whole process tree is killed at once (SIGKILL to its group, or
/// the Windows Job Object terminated), not asked politely.
pub async fn run_command(
    argv: &[String],
    cwd: &Path,
    timeout: Duration,
) -> Result<CommandOutput, CommandRunError> {
    let Some((program, args)) = argv.split_first() else {
        return Err(CommandRunError::Spawn("the command is empty".into()));
    };
    let mut cmd = tokio::process::Command::new(program);
    cmd.args(args).current_dir(cwd);
    crate::tree_kill::captured(&mut cmd);
    let (child, mut guard) = crate::tree_kill::spawn_guarded(&mut cmd)
        .map_err(|e| CommandRunError::Spawn(format!("{program}: {e}")))?;
    match tokio::time::timeout(timeout, child.wait_with_output()).await {
        Ok(Ok(output)) => {
            guard.disarm();
            Ok(CommandOutput {
                exit_code: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            })
        }
        Ok(Err(e)) => Err(CommandRunError::Spawn(format!("{program}: {e}"))),
        Err(_) => {
            guard.kill_now();
            Err(CommandRunError::TimedOut)
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn sh(script: &str) -> Vec<String> {
        vec!["sh".into(), "-c".into(), script.into()]
    }

    #[tokio::test]
    async fn output_exit_code_and_cwd() {
        let dir = tempfile::tempdir().unwrap();
        let out = run_command(
            &sh("pwd; echo bad >&2; exit 3"),
            dir.path(),
            Duration::from_secs(10),
        )
        .await
        .unwrap();
        assert_eq!(out.exit_code, Some(3));
        assert!(out.stderr.contains("bad"));
        let canonical = std::fs::canonicalize(dir.path()).unwrap();
        assert_eq!(std::path::Path::new(out.stdout.trim()), canonical);
    }

    #[tokio::test]
    async fn a_missing_program_is_a_spawn_error() {
        let dir = tempfile::tempdir().unwrap();
        let err = run_command(
            &["/no/such/program".into()],
            dir.path(),
            Duration::from_secs(5),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, CommandRunError::Spawn(_)), "{err:?}");
    }

    /// The timeout kills the grandchild too: a `sleep` started by the script
    /// that ignores SIGTERM would outlive a polite kill.
    #[tokio::test]
    async fn a_timeout_kills_the_whole_process_tree() {
        let dir = tempfile::tempdir().unwrap();
        let pidfile = dir.path().join("pid");
        let script = format!(
            "trap '' TERM; sleep 60 & echo $! > '{}'; wait",
            pidfile.display()
        );
        let err = run_command(&sh(&script), dir.path(), Duration::from_millis(1500))
            .await
            .unwrap_err();
        assert_eq!(err, CommandRunError::TimedOut);
        let pid: i32 = std::fs::read_to_string(&pidfile)
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        // The kill is a signal; give the kernel a moment to reap.
        for _ in 0..50 {
            // SAFETY: signal 0 only probes for existence.
            if unsafe { libc::kill(pid, 0) } != 0 {
                return;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        panic!("the grandchild {pid} is still running");
    }
}

/// The Windows counterpart, with PowerShell 7 (`pwsh`) as the scripting
/// language: a test returns early with a message when it is not installed.
#[cfg(all(test, windows))]
mod windows_tests {
    use super::*;

    fn pwsh_installed() -> bool {
        let found = std::process::Command::new("pwsh")
            .args(["-NoProfile", "-Command", "exit 0"])
            .output()
            .is_ok_and(|o| o.status.success());
        if !found {
            eprintln!("SKIPPED: pwsh is not installed");
        }
        found
    }

    fn pwsh(script: &str) -> Vec<String> {
        ["pwsh", "-NoProfile", "-Command", script]
            .map(String::from)
            .to_vec()
    }

    fn running(pid: u32) -> bool {
        let out = std::process::Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/NH"])
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).contains(&pid.to_string())
    }

    #[tokio::test]
    async fn output_exit_code_and_cwd() {
        if !pwsh_installed() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let out = run_command(
            &pwsh("(Get-Location).Path; [Console]::Error.WriteLine('bad'); exit 3"),
            dir.path(),
            Duration::from_secs(30),
        )
        .await
        .unwrap();
        assert_eq!(out.exit_code, Some(3));
        assert!(out.stderr.contains("bad"));
        assert_eq!(
            std::fs::canonicalize(out.stdout.trim()).unwrap(),
            std::fs::canonicalize(dir.path()).unwrap()
        );
    }

    #[tokio::test]
    async fn a_missing_program_is_a_spawn_error() {
        let dir = tempfile::tempdir().unwrap();
        let err = run_command(
            &["C:\\no\\such\\program.exe".into()],
            dir.path(),
            Duration::from_secs(5),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, CommandRunError::Spawn(_)), "{err:?}");
    }

    /// The timeout kills the grandchild too: the script starts its own
    /// `pwsh` that sleeps, and the Job Object ends both.
    #[tokio::test]
    async fn a_timeout_kills_the_whole_process_tree() {
        if !pwsh_installed() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let pidfile = dir.path().join("pid");
        let script = format!(
            "$p = Start-Process pwsh -ArgumentList '-NoProfile','-Command','Start-Sleep 60' -PassThru -NoNewWindow; \
             Set-Content -LiteralPath '{}' -Value $p.Id; Start-Sleep 60",
            pidfile.display()
        );
        let err = run_command(&pwsh(&script), dir.path(), Duration::from_secs(6))
            .await
            .unwrap_err();
        assert_eq!(err, CommandRunError::TimedOut);
        let pid: u32 = std::fs::read_to_string(&pidfile)
            .expect("the script started its child before the timeout")
            .trim()
            .parse()
            .unwrap();
        for _ in 0..50 {
            if !running(pid) {
                return;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        panic!("the grandchild {pid} is still running");
    }
}
