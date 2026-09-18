//! `gitpoulpi ui show ...` and the MCP `ui_show_*` tools against a real TUI:
//! the `gitpoulpi` binary *is* the TUI when it gets no subcommand, so it
//! is launched once in a pty with its embedded daemon, and the CLI and an
//! MCP stdio server run as separate processes against the same session and
//! config directory.

use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command as StdCommand, Stdio};
use std::time::Duration;

use assert_cmd::Command;
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use serde_json::{json, Value};
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(40);
const SESSION: &str = "cli-pty-ui-show";

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial commit")
        .unwrap()
        .build()
}

fn cli(config: &Path, repo: &Path) -> Command {
    let mut cmd = Command::cargo_bin("gitpoulpi").unwrap();
    cmd.env("GITPOULPI_CONFIG_DIR", config)
        .arg("--json")
        .args(["--session", SESSION])
        .args(["--repo", &repo.to_string_lossy()]);
    cmd
}

/// Waits until the TUI's window is attached, as `ui viewers` reports.
fn wait_for_viewer(config: &Path, repo: &Path) {
    let deadline = std::time::Instant::now() + STEP;
    loop {
        let out = cli(config, repo).args(["ui", "viewers"]).output().unwrap();
        if let Ok(v) = serde_json::from_slice::<Value>(&out.stdout) {
            if v.as_array().is_some_and(|a| !a.is_empty()) {
                return;
            }
        }
        assert!(std::time::Instant::now() < deadline, "no window attached");
        std::thread::sleep(Duration::from_millis(150));
    }
}

/// One MCP stdio session: initialize, then one `tools/call`.
fn mcp_call(config: &Path, repo: &Path, tool: &str, arguments: Value) -> Value {
    let mut child = StdCommand::new(env!("CARGO_BIN_EXE_gitpoulpi"))
        .env("GITPOULPI_CONFIG_DIR", config)
        .args(["--session", SESSION])
        .args(["--repo", &repo.to_string_lossy()])
        .args(["mcp", "serve", "--stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
    let mut send = |v: Value| writeln!(stdin, "{v}").unwrap();
    send(
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
        "protocolVersion": "2025-06-18", "capabilities": {},
        "clientInfo": {"name": "claude-code", "version": "0"}}}),
    );
    lines.next().unwrap().unwrap();
    send(json!({"jsonrpc": "2.0", "method": "notifications/initialized"}));
    send(json!({"jsonrpc": "2.0", "id": 2, "method": "tools/call",
        "params": {"name": tool, "arguments": arguments}}));
    let answer = loop {
        let line = lines.next().expect("an MCP answer").unwrap();
        let v: Value = serde_json::from_str(&line).unwrap();
        if v["id"] == json!(2) {
            break v;
        }
    };
    drop(stdin);
    let _ = child.wait();
    answer
}

#[test]
fn the_cli_and_the_mcp_server_show_settings_in_the_running_tui() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    // The agent side's choice: move the window at once.
    std::fs::write(
        config.path().join("config.toml"),
        "[mcp]\nui_control = \"auto\"\n",
    )
    .unwrap();
    let term: Terminal = Terminal::builder(env!("CARGO_BIN_EXE_gitpoulpi"))
        .args(["-s", SESSION])
        .cwd(repo.root())
        .env("GITPOULPI_CONFIG_DIR", config.path())
        .env("GITPOULPI_TERMINAL_IMAGE", "off")
        .env("GITPOULPI_GENERAL_FETCH_ON_TAB_OPEN", "false")
        .env("GITPOULPI_GENERAL_AUTO_FETCH_INTERVAL_MINUTES", "0")
        // Alone in its pty console the Windows build would relaunch itself
        // in Windows Terminal (`relaunch.rs`) and exit: not in a test.
        .env("GITPOULPI_RELAUNCHED", "1")
        .size(200, 50)
        .spawn()
        .expect("spawn the TUI");
    term.wait_for("the Log", STEP, |s| s.text().contains("initial commit"))
        .unwrap_or_else(|e| panic!("{e}"));
    wait_for_viewer(config.path(), repo.root());
    std::thread::sleep(Duration::from_millis(1700));

    // The CLI.
    let out = cli(config.path(), repo.root())
        .args(["ui", "show", "settings", "ui-customization"])
        .args(["--setting", "ui.minimap"])
        .output()
        .unwrap();
    let outcome: Value = serde_json::from_slice(&out.stdout)
        .unwrap_or_else(|e| panic!("{e}: {}", String::from_utf8_lossy(&out.stdout)));
    assert_eq!(outcome["outcome"], "applied", "{outcome}");
    assert_eq!(outcome["state"]["settings"]["page"], "diff-editor");
    assert_eq!(outcome["state"]["settings"]["setting"], "ui.minimap");
    assert!(out.status.success());
    term.wait_for("the Minimap row on the page", STEP, |s| {
        let t = s.text();
        t.contains("Diff & editor") && t.contains("Minimap")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Close it, let the "user is busy" window pass, then the MCP tool.
    let mut term = term;
    term.send_keys("Esc").unwrap();
    term.wait_for("Settings closed", STEP, |s| !s.text().contains("Minimap"))
        .unwrap_or_else(|e| panic!("{e}"));
    std::thread::sleep(Duration::from_millis(1700));
    let answer = mcp_call(
        config.path(),
        repo.root(),
        "ui_show_settings",
        json!({"page": "git-encoding"}),
    );
    assert!(answer["error"].is_null(), "{answer}");
    let result = &answer["result"];
    assert_ne!(result["isError"], json!(true), "{answer}");
    let structured = &result["structuredContent"];
    assert_eq!(structured["outcome"], "applied", "{answer}");
    assert_eq!(structured["state"]["settings"]["page"], "git-encoding");
    term.wait_for("the Git & encoding page", STEP, |s| {
        s.text().contains("Git & encoding")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
