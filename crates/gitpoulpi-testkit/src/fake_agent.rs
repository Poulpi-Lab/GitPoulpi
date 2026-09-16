//! A fake local AI agent for tests: a tiny script that reads stdin and
//! prints a canned answer. AI tests use it as a `local_agent` profile's
//! command, so they exercise the real subprocess path and never open a
//! socket.

use std::path::{Path, PathBuf};

/// Writes an executable script into `dir` that prints `answer` (one
/// output line per element; an empty element is a blank line) and exits
/// 0. Keep the text to plain words and punctuation: on Windows it goes
/// through `cmd`'s `echo`, which has opinions about `& | < > % ^`.
///
/// If `dir/agent.calls` can be appended to, every run also appends one
/// line there, so a test can tell whether the agent was called.
pub fn write_fake_agent(dir: &Path, answer: &[&str]) -> PathBuf {
    if cfg!(windows) {
        let path = dir.join("fake-agent.cmd");
        let mut body = String::from("@echo off\r\n");
        body.push_str("echo called>>\"%~dp0agent.calls\"\r\n");
        for line in answer {
            if line.is_empty() {
                body.push_str("echo.\r\n");
            } else {
                body.push_str(&format!("echo {line}\r\n"));
            }
        }
        std::fs::write(&path, body).expect("write the fake agent");
        path
    } else {
        let path = dir.join("fake-agent.sh");
        let mut body = String::from(
            "#!/bin/sh\ncat >/dev/null\necho called >> \"$(dirname \"$0\")/agent.calls\"\n",
        );
        for line in answer {
            body.push_str(&format!(
                "printf '%s\\n' '{}'\n",
                line.replace('\'', "'\\''")
            ));
        }
        std::fs::write(&path, body).expect("write the fake agent");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
                .expect("make the fake agent executable");
        }
        path
    }
}

/// A `config.toml` body turning AI on with one `local_agent` profile
/// running `agent`.
pub fn ai_config_toml(agent: &Path) -> String {
    format!(
        "[ai]\nenabled = true\n\n[[ai.profiles]]\nid = \"fake\"\nname = \"Fake agent\"\nkind = \"local_agent\"\ncommand = '{}'\nargs = []\n",
        agent.display()
    )
}

/// Like [`write_fake_agent`] but the agent sleeps `seconds` before it
/// answers, so a test can cancel a request that is still running.
pub fn write_slow_fake_agent(dir: &Path, seconds: u32, answer: &[&str]) -> PathBuf {
    let path = write_fake_agent(dir, answer);
    let body = std::fs::read_to_string(&path).expect("read the fake agent");
    let body = if cfg!(windows) {
        body.replacen(
            "@echo off\r\n",
            &format!("@echo off\r\nping -n {} 127.0.0.1 >nul\r\n", seconds + 1),
            1,
        )
    } else {
        body.replacen(
            "cat >/dev/null\n",
            &format!("cat >/dev/null\nsleep {seconds}\n"),
            1,
        )
    };
    std::fs::write(&path, body).expect("rewrite the fake agent");
    path
}

/// How many times the fake agent at `agent` ran.
pub fn agent_calls(agent: &Path) -> usize {
    let calls = agent.with_file_name("agent.calls");
    std::fs::read_to_string(calls)
        .map(|s| s.lines().count())
        .unwrap_or(0)
}

/// Like [`write_fake_agent`] but the agent also keeps the prompt it was
/// given on stdin (the last run's) in `agent.prompt`, read back with
/// [`agent_prompt`]. Unix only: the Windows `.cmd` agent cannot capture
/// stdin portably.
#[cfg(unix)]
pub fn write_capturing_fake_agent(dir: &Path, answer: &[&str]) -> PathBuf {
    let path = write_fake_agent(dir, answer);
    let body = std::fs::read_to_string(&path).expect("read the fake agent");
    let body = body.replacen(
        "cat >/dev/null\n",
        "cat > \"$(dirname \"$0\")/agent.prompt\"\n",
        1,
    );
    std::fs::write(&path, body).expect("rewrite the fake agent");
    path
}

/// The prompt the capturing agent at `agent` last received.
pub fn agent_prompt(agent: &Path) -> String {
    std::fs::read_to_string(agent.with_file_name("agent.prompt")).unwrap_or_default()
}
