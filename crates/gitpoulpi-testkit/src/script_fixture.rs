//! A stand-in for Python and an installed Lua, so the external script tests
//! need no interpreter on the machine.
//!
//! The program is a small Rust file (`script_fixture_main.rs`) compiled with
//! the `rustc` that built the tests, once per source version, into the
//! system temp directory (`gitpoulpi-script-fixture-<hash>`). [`install`]
//! points `GITPOULPI_TOOL_PYTHON_PATH` and `GITPOULPI_TOOL_LUA_PATH` at it,
//! the seam of `cli-backend`'s tool probe, so a backend resolves "python"
//! and "lua" to this program. It is run like an interpreter (`program FILE`)
//! and reads `mode=...` lines of FILE to decide what to answer.

use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

const SOURCE: &str = include_str!("script_fixture_main.rs");

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
    let out = dir.join(format!("gitpoulpi-script-fixture-{}{exe}", hash_of(SOURCE)));
    if out.exists() {
        return out;
    }
    let pid = std::process::id();
    let src = dir.join(format!("gitpoulpi-script-fixture-{pid}.rs"));
    let tmp = dir.join(format!("gitpoulpi-script-fixture-{pid}.tmp{exe}"));
    std::fs::write(&src, SOURCE).expect("write the fixture source");
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let status = Command::new(rustc)
        .args(["--edition", "2021", "-O", "-o"])
        .arg(&tmp)
        .arg(&src)
        .status()
        .expect("run rustc for the script fixture");
    assert!(status.success(), "the script fixture did not compile");
    let _ = std::fs::remove_file(&src);
    // Another test process may have won the race: either file is fine.
    if std::fs::rename(&tmp, &out).is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    assert!(out.exists(), "the script fixture is missing");
    out
}

/// The compiled fixture program.
pub fn program() -> &'static PathBuf {
    static PROGRAM: OnceLock<PathBuf> = OnceLock::new();
    PROGRAM.get_or_init(build)
}

/// Makes "python" and an installed "lua" resolve to the fixture for the rest
/// of this process. Idempotent; call it before the first script check.
pub fn install() {
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| {
        let program = program();
        std::env::set_var("GITPOULPI_TOOL_PYTHON_PATH", program);
        std::env::set_var("GITPOULPI_TOOL_LUA_PATH", program);
    });
}
