//! A fake interactive agent for the terminal run mode: a small std-only
//! Rust program (`terminal_agent_main.rs`) compiled with the `rustc` that
//! built the tests, once per source version, into the system temp directory.
//! See that file for what it paints and its environment knobs. It stands in
//! for Claude Code, Codex and Gemini so no test needs a real agent, an
//! account or a network.

use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

const SOURCE: &str = include_str!("terminal_agent_main.rs");

fn hash_of(text: &str) -> String {
    // FNV-1a: only to name the compiled file after its source.
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in text.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{h:016x}")
}

fn build() -> PathBuf {
    let exe = if cfg!(windows) { ".exe" } else { "" };
    let dir = std::env::temp_dir();
    let out = dir.join(format!("gitpoulpi-terminal-agent-{}{exe}", hash_of(SOURCE)));
    if out.exists() {
        return out;
    }
    let pid = std::process::id();
    let src = dir.join(format!("gitpoulpi-terminal-agent-{pid}.rs"));
    let tmp = dir.join(format!("gitpoulpi-terminal-agent-{pid}.tmp{exe}"));
    std::fs::write(&src, SOURCE).expect("write the terminal agent source");
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let status = Command::new(rustc)
        .args(["--edition", "2021", "-O", "-o"])
        .arg(&tmp)
        .arg(&src)
        .status()
        .expect("run rustc for the terminal agent");
    assert!(status.success(), "the terminal agent did not compile");
    let _ = std::fs::remove_file(&src);
    // Another test process may have won the race: either file is fine.
    if std::fs::rename(&tmp, &out).is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    assert!(out.exists(), "the terminal agent is missing");
    out
}

/// The compiled fake agent.
pub fn program() -> &'static PathBuf {
    static PROGRAM: OnceLock<PathBuf> = OnceLock::new();
    PROGRAM.get_or_init(build)
}
