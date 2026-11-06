//! What must not leave the machine by accident: the pure half of the AI
//! privacy and sensitive-staging hardening.
//!
//! Three things live here, none of them doing I/O:
//!
//! - a list of **sensitive path patterns** ([`DEFAULT_SENSITIVE_PATTERNS`])
//!   and a small gitignore-style matcher ([`matching_pattern`]);
//! - [`classify`]: why a file, given its path and (optionally) its content,
//!   counts as sensitive. It uses the secret scanner of the history rewrite,
//!   so the two agree on what a secret is, and **no value ever leaves this
//!   function**: a finding carries a rule id and a masked preview;
//! - [`redact`]: a text with every finding replaced by `[REDACTED:<rule>]`.

use serde::{Deserialize, Serialize};

use crate::domain::rewrite::secrets::{preview, scan_text};

/// The paths the staging guard and the AI exclusion treat as sensitive. A
/// `!` pattern re-allows what an earlier pattern matched; the last pattern
/// that matches a path decides.
pub const DEFAULT_SENSITIVE_PATTERNS: &[&str] = &[
    ".env",
    ".env.*",
    "!.env.example",
    "!.env.sample",
    "!.env.template",
    "*.pem",
    "*.key",
    "id_rsa*",
    "id_ed25519*",
    "id_ecdsa*",
    "*.p12",
    "*.pfx",
    "*.jks",
    "*.keystore",
    "*.crt",
    "*.cer",
    "*.der",
    "*.kdbx",
];

/// [`DEFAULT_SENSITIVE_PATTERNS`] as owned strings (the config default).
pub fn default_sensitive_patterns() -> Vec<String> {
    DEFAULT_SENSITIVE_PATTERNS
        .iter()
        .map(|p| (*p).to_string())
        .collect()
}

/// Why one file is sensitive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Reason {
    /// Its path matches this pattern.
    Pattern(String),
    /// Its content holds something the secret scanner recognises. `preview`
    /// is masked; the value itself is never stored here.
    Secret { rule: String, preview: String },
}

/// A file that should not be staged or sent without a decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sensitive {
    pub path: String,
    pub reason: Reason,
}

impl Sensitive {
    /// A one-line reason for a list, such as `matches .env.*` or
    /// `secret: github-token (ghp_****)`.
    pub fn describe(&self) -> String {
        match &self.reason {
            Reason::Pattern(p) => format!("matches {p}"),
            Reason::Secret { rule, preview } => format!("secret: {rule} ({preview})"),
        }
    }
}

/// The paths of `files` for a message: the first few, then a count.
pub fn summary(files: &[Sensitive]) -> String {
    let mut paths: Vec<&str> = Vec::new();
    for f in files {
        if !paths.contains(&f.path.as_str()) {
            paths.push(&f.path);
        }
    }
    let shown = paths.iter().take(5).copied().collect::<Vec<_>>().join(", ");
    if paths.len() > 5 {
        format!("{shown}, and {} more", paths.len() - 5)
    } else {
        shown
    }
}

/// One secret replaced in a text. No value: a rule and a masked preview.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Redaction {
    pub rule: String,
    pub preview: String,
}

/// The positive pattern that makes `path` sensitive, or `None`. A pattern
/// without a `/` matches the file name at any depth, one with a `/` matches
/// the whole path (a leading `/` anchors it at the root), `!` re-allows,
/// `*` and `?` do not cross a `/`, `**` does. The last matching pattern wins.
/// Matching ignores ASCII case: a guard must not be fooled by `.ENV`.
pub fn matching_pattern<'a, S: AsRef<str>>(path: &str, patterns: &'a [S]) -> Option<&'a str> {
    let path = path.replace('\\', "/");
    let path = path.trim_start_matches("./");
    let mut hit: Option<&'a str> = None;
    for raw in patterns {
        let raw = raw.as_ref().trim();
        if raw.is_empty() || raw.starts_with('#') {
            continue;
        }
        let (negated, body) = match raw.strip_prefix('!') {
            Some(rest) => (true, rest.trim()),
            None => (false, raw),
        };
        if body.is_empty() || !pattern_matches(body, path) {
            continue;
        }
        hit = if negated { None } else { Some(raw) };
    }
    hit
}

fn pattern_matches(pattern: &str, path: &str) -> bool {
    let pattern = pattern.trim_end_matches('/');
    let anchored = pattern.starts_with('/');
    let pattern = pattern.trim_start_matches('/');
    if !anchored && !pattern.contains('/') {
        let name = path.rsplit('/').next().unwrap_or(path);
        return glob(pattern.as_bytes(), name.as_bytes());
    }
    glob(pattern.as_bytes(), path.as_bytes())
}

/// `*`, `?` and `**`, case-insensitive; `/` is matched only by itself or `**`.
fn glob(pat: &[u8], text: &[u8]) -> bool {
    match pat.first() {
        None => text.is_empty(),
        Some(b'*') => {
            let double = pat.get(1) == Some(&b'*');
            let rest = &pat[if double { 2 } else { 1 }..];
            // `**/` also matches no directory at all.
            if double && rest.first() == Some(&b'/') && glob(&rest[1..], text) {
                return true;
            }
            let mut i = 0;
            loop {
                if glob(rest, &text[i..]) {
                    return true;
                }
                if i == text.len() || (!double && text[i] == b'/') {
                    return false;
                }
                i += 1;
            }
        }
        Some(b'?') => match text.first() {
            Some(c) if *c != b'/' => glob(&pat[1..], &text[1..]),
            _ => false,
        },
        Some(p) => match text.first() {
            Some(c) if c.eq_ignore_ascii_case(p) => glob(&pat[1..], &text[1..]),
            _ => false,
        },
    }
}

/// Why `path` (with `content`, when the caller read it) is sensitive. A path
/// that matches a pattern gives one entry; each distinct secret in the
/// content gives one more (a rule and a masked preview, never the value).
pub fn classify<S: AsRef<str>>(
    path: &str,
    content: Option<&[u8]>,
    patterns: &[S],
) -> Vec<Sensitive> {
    let mut out = Vec::new();
    if let Some(p) = matching_pattern(path, patterns) {
        out.push(Sensitive {
            path: path.to_string(),
            reason: Reason::Pattern(p.to_string()),
        });
    }
    if let Some(bytes) = content {
        let mut seen: Vec<(String, String)> = Vec::new();
        for m in scan_text(bytes, path) {
            let key = (m.rule.to_string(), preview(m.rule, &m.value));
            if seen.contains(&key) {
                continue;
            }
            seen.push(key.clone());
            out.push(Sensitive {
                path: path.to_string(),
                reason: Reason::Secret {
                    rule: key.0,
                    preview: key.1,
                },
            });
        }
    }
    out
}

/// `text` with every secret replaced by `[REDACTED:<rule>]`, and what was
/// replaced.
pub fn redact(text: &str) -> (String, Vec<Redaction>) {
    redact_in(text, "")
}

/// [`redact`] for the content of the file at `path` (a lock file is scanned
/// for token prefixes only).
pub fn redact_in(text: &str, path: &str) -> (String, Vec<Redaction>) {
    let bytes = text.as_bytes();
    let found = scan_text(bytes, path);
    if found.is_empty() {
        return (text.to_string(), Vec::new());
    }
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0usize;
    let mut redactions = Vec::new();
    for m in found {
        let end = m.offset + m.value.len();
        if m.offset < at || end > bytes.len() {
            continue;
        }
        out.extend_from_slice(&bytes[at..m.offset]);
        out.extend_from_slice(format!("[REDACTED:{}]", m.rule).as_bytes());
        redactions.push(Redaction {
            rule: m.rule.to_string(),
            preview: preview(m.rule, &m.value),
        });
        at = end;
    }
    out.extend_from_slice(&bytes[at..]);
    (String::from_utf8_lossy(&out).into_owned(), redactions)
}

#[cfg(test)]
mod tests;
