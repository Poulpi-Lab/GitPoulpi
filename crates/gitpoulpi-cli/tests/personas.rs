//! One executable, several programs: `gitpoulpi --gitpoulpi-as <name>` (and a
//! copy or symlink named like the old program) behaves like the old
//! `gitpoulpi-tui` / `gitpoulpi-mcp` binaries.

use std::path::{Path, PathBuf};
use std::process::Command;

fn gitpoulpi() -> &'static str {
    env!("CARGO_BIN_EXE_gitpoulpi")
}

/// A copy of the built `gitpoulpi` under `name` in `dir`.
fn copy_as(dir: &Path, name: &str) -> PathBuf {
    let target = dir.join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
    std::fs::copy(gitpoulpi(), &target).unwrap();
    target
}

fn version_line(out: &std::process::Output) -> String {
    assert!(out.status.success(), "{out:?}");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn the_tui_persona_answers_version_like_the_tui() {
    let out = Command::new(gitpoulpi())
        .args(["--gitpoulpi-as", "gitpoulpi-tui", "--version"])
        .output()
        .unwrap();
    let line = version_line(&out);
    assert!(line.starts_with("gitpoulpi-tui "), "{line}");
}

#[test]
fn a_copy_named_gitpoulpi_tui_is_the_tui() {
    let dir = tempfile::tempdir().unwrap();
    let exe = copy_as(dir.path(), "gitpoulpi-tui");
    let out = Command::new(exe).arg("--version").output().unwrap();
    assert!(version_line(&out).starts_with("gitpoulpi-tui "));
}

#[cfg(unix)]
#[test]
fn a_symlink_named_gitpoulpi_tui_is_the_tui() {
    let dir = tempfile::tempdir().unwrap();
    let link = dir.path().join("gitpoulpi-tui");
    std::os::unix::fs::symlink(gitpoulpi(), &link).unwrap();
    let out = Command::new(link).arg("--version").output().unwrap();
    assert!(version_line(&out).starts_with("gitpoulpi-tui "));
}

#[test]
fn the_tui_persona_keeps_the_old_flag_parser() {
    let out = Command::new(gitpoulpi())
        .args(["--gitpoulpi-as", "gitpoulpi-tui", "--sesion", "x"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert!(String::from_utf8_lossy(&out.stderr).contains("gitpoulpi-tui: unrecognized argument"));
}

#[test]
fn the_plain_program_still_says_gitpoulpi() {
    let out = Command::new(gitpoulpi()).arg("--version").output().unwrap();
    assert!(version_line(&out).starts_with("gitpoulpi"));
    assert!(!version_line(&out).starts_with("gitpoulpi-tui"));
}

#[test]
fn an_unknown_persona_is_refused() {
    let out = Command::new(gitpoulpi())
        .args(["--gitpoulpi-as", "gitpoulpi-nope"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
}

#[cfg(feature = "mcp")]
mod mcp {
    use super::*;
    use std::io::Write;
    use std::process::Stdio;

    fn initialize(mut cmd: Command, config: &Path) -> serde_json::Value {
        let mut child = cmd
            .env("GITPOULPI_CONFIG_DIR", config)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let request = serde_json::json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {
                "protocolVersion": "2025-06-18", "capabilities": {},
                "clientInfo": { "name": "persona-test", "version": "0" },
            }
        });
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(format!("{request}\n").as_bytes())
            .unwrap();
        drop(child.stdin.take());
        let output = child.wait_with_output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let line = stdout.lines().next().unwrap_or_else(|| {
            panic!(
                "no stdout; stderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            )
        });
        serde_json::from_str(line).unwrap()
    }

    #[test]
    fn the_mcp_persona_answers_initialize() {
        let config = tempfile::tempdir().unwrap();
        let mut cmd = Command::new(gitpoulpi());
        cmd.args([
            "--gitpoulpi-as",
            "gitpoulpi-mcp",
            "--session",
            "persona-mcp",
        ]);
        let response = initialize(cmd, config.path());
        assert_eq!(response["id"], serde_json::Value::from(1));
        assert_eq!(response["result"]["serverInfo"]["name"], "gitpoulpi-mcp");
    }

    #[test]
    fn a_copy_named_gitpoulpi_mcp_is_the_mcp_server() {
        let config = tempfile::tempdir().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let mut cmd = Command::new(copy_as(dir.path(), "gitpoulpi-mcp"));
        cmd.args(["--session", "persona-mcp-copy"]);
        let response = initialize(cmd, config.path());
        assert_eq!(response["result"]["serverInfo"]["name"], "gitpoulpi-mcp");
    }

    /// Flag parity with the old `gitpoulpi-mcp` and `mcp serve`: `--bind`
    /// needs `--http` in both.
    #[test]
    fn bind_needs_http_in_both_spellings() {
        let config = tempfile::tempdir().unwrap();
        for args in [
            &["--gitpoulpi-as", "gitpoulpi-mcp", "--bind", "127.0.0.1:0"][..],
            &["mcp", "serve", "--bind", "127.0.0.1:0"][..],
        ] {
            let out = Command::new(gitpoulpi())
                .env("GITPOULPI_CONFIG_DIR", config.path())
                .args(args)
                .output()
                .unwrap();
            assert_eq!(out.status.code(), Some(2), "{args:?}: {out:?}");
        }
    }

    #[test]
    fn the_mcp_persona_rejects_unknown_flags_like_the_old_binary() {
        let out = Command::new(gitpoulpi())
            .args(["--gitpoulpi-as", "gitpoulpi-mcp", "--nonsense"])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2));
    }
}

#[cfg(not(feature = "mcp"))]
#[test]
fn the_mcp_persona_says_this_build_has_no_mcp_server() {
    let out = Command::new(gitpoulpi())
        .args(["--gitpoulpi-as", "gitpoulpi-mcp"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("this build has no MCP server"));
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
