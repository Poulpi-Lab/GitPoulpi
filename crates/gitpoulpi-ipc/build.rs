//! Bakes the checkout's short commit sha into the build as
//! `GITPOULPI_GIT_SHA`, so the daemon and the TUI report the same value
//! without spawning `git` at run time. This crate sits under both of them,
//! so they always agree. Outside a git checkout (a source tarball, no `git`
//! installed) the value is `unknown`, which the version-mismatch check
//! treats as matching.

use std::path::PathBuf;
use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!s.is_empty()).then_some(s)
}

fn main() {
    println!("cargo:rerun-if-env-changed=GITPOULPI_GIT_SHA");
    println!("cargo:rerun-if-changed=build.rs");
    if let Ok(v) = std::env::var("GITPOULPI_GIT_SHA") {
        if !v.is_empty() {
            // Explicit override (packagers): the env var is already visible
            // to `option_env!` for this crate.
            return;
        }
    }
    let sha = git(&["rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "unknown".into());
    // Re-run when HEAD moves: the reflog changes on every commit/checkout.
    // `--git-path` resolves correctly inside linked worktrees too.
    for p in ["HEAD", "logs/HEAD"] {
        if let Some(path) = git(&["rev-parse", "--git-path", p]) {
            let path = PathBuf::from(path);
            if path.exists() {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }
    println!("cargo:rustc-env=GITPOULPI_GIT_SHA={sha}");
}
