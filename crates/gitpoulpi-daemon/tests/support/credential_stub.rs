//! Stands in for a GitPoulpi binary as git's credential helper in the daemon's
//! tests (a test executable has no helper entry of its own): the same one
//! call every real binary makes first thing in `main`.
//!
//! When `GITPOULPI_STUB_LOG` names a file, the stub first appends what it was
//! run with (its arguments, then its environment, then `END`), so a test can
//! check what the helper saw without a shell wrapper (the same on every OS).

use std::io::Write;

fn log_call(path: &std::ffi::OsStr) {
    let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    else {
        return;
    };
    let mut text = String::from("ARGV");
    for arg in std::env::args().skip(1) {
        text.push_str(&format!(" [{arg}]"));
    }
    text.push('\n');
    for (key, value) in std::env::vars() {
        text.push_str(&format!("{key}={value}\n"));
    }
    text.push_str("END\n");
    let _ = file.write_all(text.as_bytes());
}

fn main() {
    if let Some(path) = std::env::var_os("GITPOULPI_STUB_LOG") {
        log_call(&path);
    }
    gitpoulpi_ipc::git_credential::run_if_requested();
}
