//! `cargo run -p xtask -- perf --rewrite`: the history rewrite content
//! engine against the plan's targets (the history-rewrite design,
//! "Performance targets"; docs/developer/perf-budgets.md).
//!
//! The measuring is the ignored test `rewrite_content_perf` of
//! `gitpoulpi-git2-backend` (it builds a history with `import_content_history`
//! and times a path removal and a text replace through the backend); this
//! only runs it in release mode with the size the caller asks for, so the
//! numbers are the ones a user gets.

use std::process::Command;

pub const USAGE: &str = "\
Usage: cargo run -p xtask -- perf --rewrite [options]

  --commits <n>   history size, default 10000 (the plan's table has 10000
                  and 100000; other sizes are interpolated)";

pub fn run(args: &[String]) -> Result<i32, String> {
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("{USAGE}");
        return Ok(0);
    }
    let mut commits = 10_000usize;
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--commits" => {
                commits = it
                    .next()
                    .and_then(|v| v.parse().ok())
                    .ok_or("--commits needs a number")?;
            }
            other => return Err(format!("unknown option {other}\n\n{USAGE}")),
        }
    }
    let status = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .args([
            "test",
            "--release",
            "-p",
            "gitpoulpi-git2-backend",
            "--test",
            "rewrite_content_perf",
            "--",
            "--ignored",
            "--nocapture",
        ])
        .env("GITPOULPI_REWRITE_PERF_COMMITS", commits.to_string())
        .current_dir(crate::perf::workspace_root())
        .status()
        .map_err(|e| format!("cannot run cargo: {e}"))?;
    Ok(status.code().unwrap_or(1))
}
