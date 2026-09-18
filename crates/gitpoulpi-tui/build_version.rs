//! The short git hash a development build shows next to its version in the
//! status bar chip (`GITPOULPI_GIT_HASH`, read with `option_env!`).
//!
//! Cheap and never fatal: one `git rev-parse`, and no hash at all when git is
//! absent, the sources are not a checkout (a crates.io or archive build) or the
//! build is a release archive (`xtask dist` sets `GITPOULPI_DIST`). The script
//! reruns when `HEAD` or the branch it names moves, never on every build.

use std::path::PathBuf;
use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn run() {
    println!("cargo:rerun-if-env-changed=GITPOULPI_DIST");
    if std::env::var_os("GITPOULPI_DIST").is_some() {
        return;
    }
    let Some(hash) = git(&["rev-parse", "--short=7", "HEAD"]) else {
        return;
    };
    println!("cargo:rustc-env=GITPOULPI_GIT_HASH={hash}");
    // `--git-path` resolves worktrees too (where `.git` is a file).
    let watch = |what: &str| {
        if let Some(path) = git(&["rev-parse", "--git-path", what]) {
            let path = PathBuf::from(path);
            if path.exists() {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    };
    watch("HEAD");
    if let Some(branch) = git(&["symbolic-ref", "-q", "HEAD"]) {
        watch(&branch);
    }
}
