//! How an object id is shown in short form (pure; re-exported by `ui_kit`).

use gitpoulpi_core::domain::Oid;

/// Abbreviated object id, the 7 hex chars git itself shows by default.
pub fn short_oid(oid: Oid) -> String {
    oid.to_hex()[..7].to_string()
}
