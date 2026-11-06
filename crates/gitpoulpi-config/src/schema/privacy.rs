//! `[privacy]`: what the staging guard treats as sensitive.

use serde::{Deserialize, Serialize};

/// `[privacy]`. The AI side of privacy is `[ai.privacy]` and
/// `[ai.sent_preview]`; the MCP side is `[mcp] allow_sensitive_staging`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PrivacyConfig {
    /// Gitignore-style path patterns for files that look sensitive (a `!`
    /// re-allows what an earlier pattern matched; the last match wins). The
    /// staging guard asks before staging a match, and the AI never receives
    /// the diff of a match while `[ai.privacy] exclude_sensitive_files` is on.
    pub sensitive_patterns: Vec<String>,
    /// `true` (the default): staging a file that matches a pattern, or whose
    /// content holds a recognised secret, asks first.
    pub staging_guard: bool,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            sensitive_patterns: gitpoulpi_core::privacy::default_sensitive_patterns(),
            staging_guard: true,
        }
    }
}
