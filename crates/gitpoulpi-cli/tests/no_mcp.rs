//! Custom builds without `mcp`: the subcommand is not in the clap tree.
#![cfg(not(feature = "mcp"))]

use std::process::Command;

#[test]
fn mcp_subcommand_is_absent() {
    let out = Command::new(env!("CARGO_BIN_EXE_gitpoulpi"))
        .args(["mcp", "serve"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let help = Command::new(env!("CARGO_BIN_EXE_gitpoulpi"))
        .arg("--help")
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&help.stdout);
    assert!(!text.lines().any(|l| l.trim_start().starts_with("mcp ")));
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
