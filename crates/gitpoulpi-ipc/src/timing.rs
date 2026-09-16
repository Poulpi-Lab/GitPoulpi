//! Opt-in latency tracing for the request path, shared by the daemon and
//! its clients.
//!
//! Off unless `GITPOULPI_TIMING` is set (to anything but `0`). When on,
//! each call to [`record`] appends one line to `gitpoulpi-timing.log` in
//! the system temp directory -- a file rather than stderr, because the
//! TUI owns the terminal and anything printed there lands in the middle
//! of the frame. Nothing here is on by default or costs more than one
//! cached boolean check when off.
//!
//! ```text
//! $env:GITPOULPI_TIMING = 1; gitpoulpi-tui
//! Get-Content -Wait $env:TEMP\gitpoulpi-timing.log
//! ```

use std::io::Write;
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// The switch: `GITPOULPI_TIMING`, read once per process.
pub const ENV: &str = "GITPOULPI_TIMING";

/// Whether timing lines are being written.
pub fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os(ENV).is_some_and(|v| v != "0" && !v.is_empty()))
}

/// Where the lines go.
pub fn log_path() -> std::path::PathBuf {
    std::env::temp_dir().join("gitpoulpi-timing.log")
}

/// Appends `<unix ms> <who> <what>` to the log, when enabled. Failures
/// to write are ignored: this is a diagnostic, never a reason to fail
/// the request it describes.
pub fn record(who: &str, what: impl FnOnce() -> String) {
    if !enabled() {
        return;
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let line = format!("{now} {who} {}\n", what());
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path())
    {
        let _ = file.write_all(line.as_bytes());
    }
}

/// A duration as whole milliseconds with one decimal, for the log.
pub fn ms(d: Duration) -> String {
    format!("{:.1}ms", d.as_secs_f64() * 1000.0)
}
