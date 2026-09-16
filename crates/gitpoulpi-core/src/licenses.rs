//! The third-party license notices built into every binary, so a copy that
//! was installed without the archive's files still carries them.
//!
//! The text is `THIRD-PARTY-LICENSES.txt` at the repository root, generated
//! by `cargo run -p xtask -- licenses` (a test fails when it is stale).

/// Credits, bundled-code notices, the crate list and every license text.
pub const THIRD_PARTY: &str = include_str!("../../../THIRD-PARTY-LICENSES.txt");
