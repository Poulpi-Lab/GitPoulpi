//! The master gate for tests that reach a real provider (the internet or the
//! Docker bench of `scripts/test-bench.sh`).
//!
//! Such a test is `#[ignore = LIVE_IGNORE]` (so `cargo test --workspace`, the
//! gate, `scripts/check.ps1` and CI never run it) and starts with
//! `if !gitpoulpi_testkit::live::enabled("live_gitea_reads") { return; }`
//! (so `--ignored` alone does not connect either: it needs
//! `GITPOULPI_LIVE_TESTS=1` too, and prints one skip line otherwise). The
//! per-provider variables (`GITPOULPI_LIVE_GITEA`, `_TOKEN`, ...) still say
//! *where* and *as whom*.

/// The master switch: `GITPOULPI_LIVE_TESTS=1`.
pub const ENV: &str = "GITPOULPI_LIVE_TESTS";

/// The `#[ignore]` reason every live test uses (attributes need a literal,
/// so tests repeat this text).
pub const LIVE_IGNORE: &str = "live: set GITPOULPI_LIVE_TESTS=1 and run with --ignored";

/// Whether the master switch is on (`1` or `true`).
pub fn is_on() -> bool {
    parse(std::env::var(ENV).ok().as_deref())
}

fn parse(value: Option<&str>) -> bool {
    matches!(value.map(str::trim), Some("1") | Some("true"))
}

/// `true` when the live test `name` may run. Otherwise prints a single skip
/// line and returns `false`; the caller returns (the test passes, it never
/// fails for lack of a provider).
pub fn enabled(name: &str) -> bool {
    if is_on() {
        return true;
    }
    eprintln!("SKIPPED {name}: live provider test, set {ENV}=1 to run it");
    false
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn only_one_or_true_turns_it_on() {
        assert!(parse(Some("1")) && parse(Some("true")) && parse(Some(" 1 ")));
        assert!(!parse(None) && !parse(Some("")) && !parse(Some("0")) && !parse(Some("yes")));
    }
}
