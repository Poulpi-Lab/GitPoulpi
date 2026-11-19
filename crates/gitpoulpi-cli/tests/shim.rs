//! The stubs shipped as `gitpoulpi-tui` / `gitpoulpi-mcp` (`gitpoulpi-shim`):
//! a copy of the shim next to `gitpoulpi` answers like the old program,
//! passes stdin/stdout and the exit code through, and says so when
//! `gitpoulpi` is missing.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

fn gitpoulpi() -> &'static str {
    env!("CARGO_BIN_EXE_gitpoulpi")
}

/// The built `gitpoulpi-shim`, next to the built `gitpoulpi` (built here, always:
/// cargo does not build another package's binary for this test, and a stale
/// one would test old code).
fn shim() -> &'static Path {
    static SHIM: OnceLock<PathBuf> = OnceLock::new();
    SHIM.get_or_init(|| {
        let profile_dir = Path::new(gitpoulpi()).parent().unwrap();
        let path = profile_dir.join(format!("gitpoulpi-shim{}", std::env::consts::EXE_SUFFIX));
        let mut cmd = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()));
        cmd.args(["build", "-p", "gitpoulpi-shim"]);
        if profile_dir.file_name().is_some_and(|n| n == "release") {
            cmd.arg("--release");
        }
        let status = cmd.status().unwrap();
        assert!(status.success(), "building gitpoulpi-shim failed");
        path
    })
}

/// An install folder with the real `gitpoulpi` and the shim as `name`.
fn install(name: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let exe = std::env::consts::EXE_SUFFIX;
    std::fs::copy(gitpoulpi(), dir.path().join(format!("gitpoulpi{exe}"))).unwrap();
    let stub = dir.path().join(format!("{name}{exe}"));
    std::fs::copy(shim(), &stub).unwrap();
    (dir, stub)
}

#[test]
fn the_tui_stub_answers_version_like_the_tui() {
    let (_dir, stub) = install("gitpoulpi-tui");
    let out = Command::new(stub).arg("--version").output().unwrap();
    assert!(out.status.success(), "{out:?}");
    let line = String::from_utf8_lossy(&out.stdout);
    assert!(line.starts_with("gitpoulpi-tui "), "{line}");
}

#[test]
fn the_exit_code_passes_through() {
    let (_dir, stub) = install("gitpoulpi-tui");
    // The TUI persona refuses an unknown flag with exit code 2.
    let out = Command::new(stub).arg("--sesion").output().unwrap();
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert!(String::from_utf8_lossy(&out.stderr).contains("gitpoulpi-tui: unrecognized argument"));
}

#[cfg(unix)]
#[test]
fn a_symlinked_stub_finds_gitpoulpi_next_to_the_real_file() {
    let (dir, stub) = install("gitpoulpi-tui");
    let elsewhere = tempfile::tempdir().unwrap();
    let link = elsewhere.path().join("gitpoulpi-tui");
    std::os::unix::fs::symlink(&stub, &link).unwrap();
    let out = Command::new(link).arg("--version").output().unwrap();
    assert!(out.status.success(), "{out:?}");
    drop(dir);
}

#[test]
fn a_missing_gitpoulpi_is_a_clear_error() {
    let (dir, stub) = install("gitpoulpi-tui");
    std::fs::remove_file(
        dir.path()
            .join(format!("gitpoulpi{}", std::env::consts::EXE_SUFFIX)),
    )
    .unwrap();
    let out = Command::new(stub).arg("--version").output().unwrap();
    assert_eq!(out.status.code(), Some(127), "{out:?}");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("was not found next to"), "{err}");
}

#[cfg(feature = "mcp")]
#[test]
fn the_mcp_stub_passes_stdin_and_stdout_through() {
    use std::io::Write;
    use std::process::Stdio;
    let (_dir, stub) = install("gitpoulpi-mcp");
    let config = tempfile::tempdir().unwrap();
    let mut child = Command::new(stub)
        .args(["--session", "shim-mcp"])
        .env("GITPOULPI_CONFIG_DIR", config.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let request = serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {
            "protocolVersion": "2025-06-18", "capabilities": {},
            "clientInfo": { "name": "shim-test", "version": "0" },
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
    let response: serde_json::Value = serde_json::from_str(line).unwrap();
    assert_eq!(response["id"], serde_json::Value::from(1));
    assert_eq!(response["result"]["serverInfo"]["name"], "gitpoulpi-mcp");
}

/// `gitpoulpi-launch` (the cli crate's own stub) next to `gitpoulpi` runs the
/// launch persona: with no window open it starts the command after `--`.
#[cfg(unix)]
#[test]
fn the_launch_stub_starts_the_command_through_the_persona() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(gitpoulpi(), dir.path().join("gitpoulpi")).unwrap();
    let stub = dir.path().join("gitpoulpi-launch");
    std::fs::copy(env!("CARGO_BIN_EXE_gitpoulpi-launch"), &stub).unwrap();
    let script = dir.path().join("terminal.sh");
    std::fs::write(&script, "#!/bin/sh\ntouch \"$1\"\n").unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    let marker = dir.path().join("started");
    let config = tempfile::tempdir().unwrap();
    let out = Command::new(stub)
        .env("GITPOULPI_CONFIG_DIR", config.path())
        .env("HOME", config.path())
        .env("XDG_CONFIG_HOME", config.path())
        .env("XDG_DATA_HOME", config.path())
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .arg("--")
        .arg(&script)
        .arg(&marker)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !marker.exists() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    assert!(marker.exists(), "the command after `--` was not started");
}
