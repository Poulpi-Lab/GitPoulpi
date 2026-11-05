//! A network git command must never wait on the terminal. The old failure:
//! ssh asked "Are you sure you want to continue connecting?" on /dev/tty, the
//! child sat in a background process group of the daemon's terminal, got
//! SIGTTIN and stopped, and the fetch hung for hours.
//!
//! No network: a fake ssh (`GIT_SSH_COMMAND`) reads /dev/tty exactly like
//! ssh does and, with no terminal, reports the host-key failure. The test
//! re-runs itself under a pty as its controlling terminal (the situation of
//! a daemon started from a terminal), so it hangs without the fix even when
//! the test runner itself has no terminal.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use gitpoulpi_cli_backend::SystemGitBackend;
use gitpoulpi_core::domain::FetchOptions;
use gitpoulpi_core::{FailureKind, GitBackend};
use gitpoulpi_testkit::FixtureRepoBuilder;

const INNER: &str = "GITPOULPI_TEST_NO_TERMINAL_INNER";
const TEST: &str = "a_fetch_that_would_prompt_on_the_terminal_fails_fast_instead_of_hanging";

#[test]
fn a_fetch_that_would_prompt_on_the_terminal_fails_fast_instead_of_hanging() {
    if std::env::var_os(INNER).is_none() {
        return launch_under_a_pty();
    }
    let dir = tempfile::tempdir().unwrap();
    let ssh = dir.path().join("fake-ssh");
    std::fs::write(
        &ssh,
        "#!/bin/sh\n\
         if { read answer </dev/tty; } 2>/dev/null; then echo prompted; exit 1; fi\n\
         echo 'Host key verification failed.' >&2\n\
         exit 255\n",
    )
    .unwrap();
    std::fs::set_permissions(&ssh, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::env::set_var("GIT_SSH_COMMAND", &ssh);
    std::env::remove_var("GIT_SSH");

    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    repo.git(&["remote", "add", "origin", "ssh://git@example.invalid/r.git"]);
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let result = rt.block_on(async {
        tokio::time::timeout(
            Duration::from_secs(15),
            SystemGitBackend::new().fetch(&repo.repo_ref(), "origin", &FetchOptions::default()),
        )
        .await
    });
    let err = result
        .expect("the fetch hung on the terminal")
        .expect_err("the fake ssh always fails");
    let detail = err.detail().expect("a failure carries its detail");
    assert!(
        detail.stderr.contains("Host key verification failed"),
        "{detail:?}"
    );
    assert_eq!(detail.kind(), FailureKind::HostKeyUnverified);
}

/// Runs this very test again with a fresh pty as controlling terminal, and
/// kills it if it has not finished in 40 s.
fn launch_under_a_pty() {
    let (mut master, mut slave) = (0, 0);
    // SAFETY: plain libc call on fds we own.
    let rc = unsafe {
        libc::openpty(
            &mut master,
            &mut slave,
            std::ptr::null_mut(),
            std::ptr::null(),
            std::ptr::null(),
        )
    };
    assert_eq!(rc, 0, "openpty");
    let mut cmd = Command::new(std::env::current_exe().unwrap());
    cmd.args(["--exact", TEST, "--test-threads=1"])
        .env(INNER, "1")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    // SAFETY: only async-signal-safe libc calls between fork and exec.
    unsafe {
        cmd.pre_exec(move || {
            libc::setsid();
            libc::ioctl(slave, libc::TIOCSCTTY as _, 0);
            Ok(())
        });
    }
    let mut child = cmd.spawn().unwrap();
    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break Some(status);
        }
        if start.elapsed() > Duration::from_secs(40) {
            break None;
        }
        std::thread::sleep(Duration::from_millis(100));
    };
    if status.is_none() {
        // SAFETY: signalling the session we created (its own process group),
        // which also holds the stopped children.
        unsafe { libc::kill(-(child.id() as i32), libc::SIGKILL) };
        let _ = child.wait();
    }
    // SAFETY: closing the fds opened above.
    unsafe {
        libc::close(slave);
        libc::close(master);
    }
    assert!(
        status.is_some_and(|s| s.success()),
        "the inner fetch test hung or failed: {status:?}"
    );
}
