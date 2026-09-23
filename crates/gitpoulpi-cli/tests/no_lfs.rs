//! Custom builds without `lfs`: no `lfs` subcommand, no `clone --no-lfs`.
#![cfg(not(feature = "lfs"))]

use std::process::Command;

#[test]
fn lfs_subcommand_is_absent() {
    let out = Command::new(env!("CARGO_BIN_EXE_gitpoulpi"))
        .args(["lfs", "status"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let help = Command::new(env!("CARGO_BIN_EXE_gitpoulpi"))
        .arg("--help")
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&help.stdout);
    assert!(!text.lines().any(|l| l.trim_start().starts_with("lfs ")));
}

// No test reads or writes the real config, data dir or credentials of whoever runs it.
gitpoulpi_testkit::isolate_binary!();
