//! A credential helper's stderr must not reach the process's own terminal.
//!
//! The TUI resolves tokens through the helper while it owns the alternate
//! screen: a line a helper (or the tool, with prompts disabled) writes to an
//! inherited stderr lands on that screen, and its newline scrolls the
//! terminal under the TUI's feet, so every click after it is one row off.
//!
//! The test runs the resolution in a child copy of this test binary whose
//! stderr is a pipe, and checks the helper's noise did not come out of it.

use gitpoulpi_secrets::{CredentialHelper, CredentialSource};
use std::process::{Command, Stdio};

const NOISE: &str = "NOISE-FROM-THE-HELPER";
const CHILD: &str = "GITPOULPI_SECRETS_TEST_CHILD";

/// Only does anything inside the child process of
/// [`a_helper_that_writes_to_stderr_does_not_reach_ours`].
#[test]
fn child_resolves_through_a_noisy_helper() {
    if std::env::var_os(CHILD).is_none() {
        return;
    }
    let result = CredentialHelper.resolve("example.test");
    assert!(result.is_err(), "the helper has nothing to give");
}

#[test]
fn a_helper_that_writes_to_stderr_does_not_reach_ours() {
    if std::env::var_os(CHILD).is_some() {
        return;
    }
    let null = if cfg!(windows) { "NUL" } else { "/dev/null" };
    let out = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "child_resolves_through_a_noisy_helper",
            "--nocapture",
        ])
        .env(CHILD, "1")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", null)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_CONFIG_COUNT", "2")
        .env("GIT_CONFIG_KEY_0", "credential.helper")
        .env("GIT_CONFIG_VALUE_0", "")
        .env("GIT_CONFIG_KEY_1", "credential.helper")
        .env("GIT_CONFIG_VALUE_1", format!("!echo {NOISE} >&2; exit 1"))
        .stdin(Stdio::null())
        .output()
        .expect("run the child");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "the child failed:\n{stderr}\n{}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        !stderr.contains(NOISE),
        "the helper's stderr reached the process's own:\n{stderr}"
    );
}
