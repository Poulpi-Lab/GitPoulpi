//! Secret detection for history rewrites (slice S14): the pure part. The
//! backend reads the blobs, this decides what in one of them is a secret.
//!
//! **Not exhaustive.** It finds the common shapes: well-known token
//! prefixes (the table is shared with `gitpoulpi_core::redact`, so the
//! application log and the scanner mask the same things; a test pins it),
//! private key blocks, JWTs, passwords in URLs, and `password = value` style
//! assignments whose value looks random enough ([`ENTROPY_MIN`]).
//!
//! **A value never leaves this module by accident.** [`SecretValue`] has a
//! redacting `Debug`, no `Display` and no `Serialize`: the only way to the
//! bytes is [`SecretValue::reveal`], which the backend calls to build the
//! replacement. Everything that crosses a wire or lands in a log is a
//! [`SecretFinding`], which holds a redacted preview.

use std::fmt;
use std::ops::Range;
use std::sync::OnceLock;

use regex::bytes::Regex;
use serde::{Deserialize, Serialize};

use crate::domain::Oid;

/// Shannon entropy (bits per character) a generic `key = value` value needs
/// to count as a secret. English words sit near 2.5 to 3.2; random base64
/// or hex sits at 4 to 6.
pub const ENTROPY_MIN: f64 = 3.0;

/// Letters-only values (identifiers such as `passwordField`) need more.
pub const ENTROPY_MIN_ALPHA: f64 = 3.5;

/// Blobs over this size are not scanned (they are data, not source).
pub const MAX_SCAN_BLOB: u64 = 5 << 20;

/// Lines longer than this are minified or encoded data: the generic
/// assignment rule does not read them.
const MAX_GENERIC_LINE: usize = 4000;

/// A secret shape that announces itself by a prefix. One table feeds the
/// scanner and `redact`.
#[derive(Debug, Clone, Copy)]
pub struct TokenShape {
    /// Stable rule id (`github-token`).
    pub rule: &'static str,
    pub prefixes: &'static [&'static str],
    /// Regex for what follows the prefix.
    pub body: &'static str,
}

/// The shared table. Order matters: when two shapes match the same text the
/// earlier one wins (`sk-ant-` before the generic `sk-`).
pub const TOKEN_SHAPES: &[TokenShape] = &[
    TokenShape {
        rule: "github-token",
        prefixes: &["ghp_", "gho_", "ghu_", "ghs_", "ghr_"],
        body: "[A-Za-z0-9]{36,255}",
    },
    TokenShape {
        rule: "github-pat",
        prefixes: &["github_pat_"],
        body: "[A-Za-z0-9_]{40,255}",
    },
    TokenShape {
        rule: "gitlab-token",
        prefixes: &["glpat-", "glptt-"],
        body: "[A-Za-z0-9_-]{20,255}",
    },
    TokenShape {
        rule: "slack-token",
        prefixes: &["xoxb-", "xoxp-", "xoxa-", "xoxr-", "xoxs-"],
        body: "[A-Za-z0-9-]{10,255}",
    },
    TokenShape {
        rule: "stripe-key",
        prefixes: &["sk_live_", "rk_live_"],
        body: "[0-9A-Za-z]{24,255}",
    },
    TokenShape {
        rule: "anthropic-key",
        prefixes: &["sk-ant-"],
        body: "[A-Za-z0-9_-]{32,255}",
    },
    TokenShape {
        rule: "openai-key",
        prefixes: &["sk-"],
        body: "[A-Za-z0-9_-]{32,255}",
    },
    TokenShape {
        rule: "google-api-key",
        prefixes: &["AIza"],
        body: "[0-9A-Za-z_-]{35}",
    },
    TokenShape {
        rule: "npm-token",
        prefixes: &["npm_"],
        body: "[A-Za-z0-9]{36}",
    },
    TokenShape {
        rule: "aws-access-key",
        prefixes: &["AKIA", "ASIA"],
        body: "[0-9A-Z]{16}",
    },
];

/// Every prefix of [`TOKEN_SHAPES`]: what `redact` masks words starting with.
pub fn token_prefixes() -> impl Iterator<Item = &'static str> + Clone {
    TOKEN_SHAPES.iter().flat_map(|s| s.prefixes.iter().copied())
}

pub const RULE_PRIVATE_KEY: &str = "private-key";
pub const RULE_JWT: &str = "jwt";
pub const RULE_URL_CREDENTIALS: &str = "url-credentials";
pub const RULE_GENERIC: &str = "generic-secret";

/// Every rule id the scanner can report.
pub fn rule_ids() -> Vec<&'static str> {
    let mut ids: Vec<&'static str> = TOKEN_SHAPES.iter().map(|s| s.rule).collect();
    ids.extend([
        RULE_PRIVATE_KEY,
        RULE_JWT,
        RULE_URL_CREDENTIALS,
        RULE_GENERIC,
    ]);
    ids
}

/// The bytes of a secret. No `Display`, no `Serialize`, a redacted `Debug`;
/// [`SecretValue::reveal`] is the one way out and its callers are the places
/// that build a replacement.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SecretValue(Vec<u8>);

impl SecretValue {
    pub fn new(bytes: impl Into<Vec<u8>>) -> Self {
        Self(bytes.into())
    }

    /// The bytes. Only the backend's replacement builder calls this.
    pub fn reveal(&self) -> &[u8] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Debug for SecretValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretValue(<redacted, {} bytes>)", self.0.len())
    }
}

/// One secret found in a text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretMatch {
    pub rule: &'static str,
    pub value: SecretValue,
    /// Where it starts in the text.
    pub offset: usize,
}

/// What a scan of a repository's history found about one secret: never the
/// value. `id` is stable (the same secret has the same id in every scan), so
/// a rule can name it and the daemon, which holds the value, resolves it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretFinding {
    /// `f-` and twelve hex digits.
    pub id: String,
    /// The rule that found it (see [`rule_ids`]).
    pub rule: String,
    /// The first file it was seen in, in history order.
    pub path: String,
    /// How many other paths hold the same value.
    pub more_paths: u32,
    pub first_commit: Oid,
    pub last_commit: Oid,
    /// Commits of the scope whose tree holds it.
    pub commits: u64,
    /// Distinct blobs that hold it.
    pub blobs: u64,
    /// `ghp_****abcd`: the start and the end at most, see [`preview`].
    pub preview: String,
}

/// A scan's answer: the findings and how much was read.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretScan {
    pub findings: Vec<SecretFinding>,
    /// Commits walked.
    pub commits: u64,
    /// Unique blobs read.
    pub blobs_scanned: u64,
    pub blobs_skipped_binary: u64,
    /// Over [`MAX_SCAN_BLOB`].
    pub blobs_skipped_large: u64,
}

/// Whether `text` has the shape of a finding id.
pub fn is_finding_id(text: &str) -> bool {
    text.len() == 14
        && text.starts_with("f-")
        && text[2..]
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

// ------------------------------------------------------------------ entropy

/// Shannon entropy of `bytes`, in bits per byte.
pub fn shannon_entropy(bytes: &[u8]) -> f64 {
    if bytes.is_empty() {
        return 0.0;
    }
    let mut counts = [0u32; 256];
    for b in bytes {
        counts[*b as usize] += 1;
    }
    let n = bytes.len() as f64;
    counts
        .iter()
        .filter(|c| **c > 0)
        .map(|c| {
            let p = f64::from(*c) / n;
            -p * p.log2()
        })
        .sum()
}

// ----------------------------------------------------------------- patterns

struct Compiled {
    shapes: Vec<(&'static TokenShape, Regex)>,
    private_key: Regex,
    jwt: Regex,
    url: Regex,
    generic: Regex,
}

fn compiled() -> &'static Compiled {
    static C: OnceLock<Compiled> = OnceLock::new();
    C.get_or_init(|| {
        let shapes = TOKEN_SHAPES
            .iter()
            .map(|s| {
                let alt = s
                    .prefixes
                    .iter()
                    .map(|p| regex::escape(p))
                    .collect::<Vec<_>>()
                    .join("|");
                let re = Regex::new(&format!("(?-u)(?:{alt}){}", s.body))
                    .expect("token shape compiles");
                (s, re)
            })
            .collect();
        Compiled {
            shapes,
            private_key: Regex::new(
                r"(?-u)(?s)-----BEGIN [A-Z ]*PRIVATE KEY(?: BLOCK)?-----.*?-----END [A-Z ]*PRIVATE KEY(?: BLOCK)?-----",
            )
            .expect("private key compiles"),
            jwt: Regex::new(r"(?-u)eyJ[A-Za-z0-9_-]{8,}\.eyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}")
                .expect("jwt compiles"),
            url: Regex::new(
                r#"(?-u)(?i)[a-z][a-z0-9+.-]{1,15}://[^\s:/@"'<>]{1,64}:([^\s@/"'<>]{6,})@[^\s/"'<>]+"#,
            )
            .expect("url compiles"),
            generic: Regex::new(
                r#"(?-u)(?i)[\w.-]*(?:password|passwd|secret|token|api[_-]?key|access[_-]?key|private[_-]?key|credentials?)[\w.-]*["']?[ \t]*(?::=|[:=])[ \t]*["']?([^\s"'`,;&)}\]]{8,})"#,
            )
            .expect("generic compiles"),
        }
    })
}

/// Whether a token would be the middle of a longer word or of base64.
fn glued_before(text: &[u8], start: usize) -> bool {
    start > 0
        && matches!(
            text[start - 1],
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'+' | b'/' | b'_' | b'-'
        )
}

fn glued_after(text: &[u8], end: usize) -> bool {
    text.get(end).is_some_and(u8::is_ascii_alphanumeric)
}

const PLACEHOLDERS: &[&str] = &[
    "example",
    "changeme",
    "change_me",
    "change-me",
    "placeholder",
    "your_",
    "your-",
    "yourtoken",
    "yourpassword",
    "xxxxxxxx",
    "********",
    "redacted",
    "***removed***",
    "dummy",
    "sample",
    "insert",
    "replace",
    "todo",
    "<",
    "secret_here",
    "password123",
];

fn is_placeholder(value: &[u8]) -> bool {
    let lower = String::from_utf8_lossy(value).to_lowercase();
    PLACEHOLDERS.iter().any(|p| lower.contains(p))
        || [
            "${",
            "$(",
            "{{",
            "%(",
            "@",
            "$",
            "process.env",
            "os.environ",
            "env(",
            "getenv",
        ]
        .iter()
        .any(|p| lower.starts_with(p))
        || lower.contains('(')
}

fn is_uuid(value: &[u8]) -> bool {
    value.len() == 36
        && value.iter().enumerate().all(|(i, b)| match i {
            8 | 13 | 18 | 23 => *b == b'-',
            _ => b.is_ascii_hexdigit(),
        })
}

/// Whether a generic `key = value` value looks like a secret.
fn generic_value_ok(value: &[u8]) -> bool {
    if value.len() < 8 || is_placeholder(value) || is_uuid(value) {
        return false;
    }
    let lower = String::from_utf8_lossy(value).to_lowercase();
    if [
        "sha512-", "sha384-", "sha256-", "sha1-", "data:", "http://", "https://", "file:",
    ]
    .iter()
    .any(|p| lower.starts_with(p))
    {
        return false;
    }
    let letters_only = value.iter().all(u8::is_ascii_alphabetic);
    let floor = if letters_only {
        ENTROPY_MIN_ALPHA
    } else {
        ENTROPY_MIN
    };
    shannon_entropy(value) >= floor
}

/// Lock files list thousands of hashes under keys such as `integrity`; the
/// generic rule leaves them alone (the token prefixes still apply).
pub fn is_lockfile(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path).to_ascii_lowercase();
    name.ends_with(".lock")
        || name.ends_with("-lock.json")
        || name.ends_with("-lock.yaml")
        || name == "go.sum"
}

type Hit = (Range<usize>, &'static str, usize);

/// The secrets in `text`, which is the content of the file at `path`.
/// Overlapping matches keep the most specific rule (a token prefix beats the
/// generic assignment it sits in).
pub fn scan_text(text: &[u8], path: &str) -> Vec<SecretMatch> {
    let c = compiled();
    let mut found: Vec<Hit> = Vec::new();
    for (priority, (shape, re)) in c.shapes.iter().enumerate() {
        for m in re.find_iter(text) {
            if glued_before(text, m.start()) {
                continue;
            }
            if shape.rule == "aws-access-key" && glued_after(text, m.end()) {
                continue;
            }
            let body = &text[m.range()];
            // One repeated letter, or a documented example, is not a key.
            let distinct = body.iter().collect::<std::collections::HashSet<_>>().len();
            if distinct < 6 || is_placeholder(body) {
                continue;
            }
            found.push((m.range(), shape.rule, priority));
        }
    }
    let base = c.shapes.len();
    for m in c.private_key.find_iter(text) {
        if m.len() >= 80 {
            found.push((m.range(), RULE_PRIVATE_KEY, base));
        }
    }
    for m in c.jwt.find_iter(text) {
        if !glued_before(text, m.start()) {
            found.push((m.range(), RULE_JWT, base + 1));
        }
    }
    for caps in c.url.captures_iter(text) {
        if let Some(g) = caps.get(1) {
            if generic_value_ok(g.as_bytes()) {
                found.push((g.range(), RULE_URL_CREDENTIALS, base + 2));
            }
        }
    }
    if !is_lockfile(path) {
        for caps in c.generic.captures_iter(text) {
            let (Some(g), Some(whole)) = (caps.get(1), caps.get(0)) else {
                continue;
            };
            if line_len(text, whole.start()) > MAX_GENERIC_LINE {
                continue;
            }
            if generic_value_ok(g.as_bytes()) {
                found.push((g.range(), RULE_GENERIC, base + 3));
            }
        }
    }
    // Earliest first; at the same start the most specific, then the longest.
    found.sort_by(|a, b| {
        a.0.start
            .cmp(&b.0.start)
            .then(a.2.cmp(&b.2))
            .then(b.0.end.cmp(&a.0.end))
    });
    let mut out: Vec<SecretMatch> = Vec::new();
    let mut covered = 0usize;
    for (range, rule, _) in found {
        if range.start < covered {
            continue;
        }
        covered = range.end;
        out.push(SecretMatch {
            rule,
            value: SecretValue::new(&text[range.clone()]),
            offset: range.start,
        });
    }
    out
}

/// Length of the line holding `at`.
fn line_len(text: &[u8], at: usize) -> usize {
    let start = text[..at]
        .iter()
        .rposition(|b| *b == b'\n')
        .map_or(0, |p| p + 1);
    let end = text[at..]
        .iter()
        .position(|b| *b == b'\n')
        .map_or(text.len(), |p| at + p);
    end - start
}

// ------------------------------------------------------------------ preview

/// The prefix of one of `rule`'s shapes that `value` starts with.
fn prefix_of(rule: &str, value: &[u8]) -> Option<&'static str> {
    TOKEN_SHAPES
        .iter()
        .filter(|s| s.rule == rule)
        .flat_map(|s| s.prefixes.iter().copied())
        .find(|p| value.starts_with(p.as_bytes()))
}

/// What may be shown about a secret: a known prefix, `****`, and for a long
/// value its last four characters (`ghp_****abcd`). A value that is not long
/// shows only its length. Never more than eight characters of a value.
pub fn preview(rule: &str, value: &SecretValue) -> String {
    let bytes = value.reveal();
    if rule == RULE_PRIVATE_KEY {
        return "-----BEGIN PRIVATE KEY-----****".to_string();
    }
    let text = String::from_utf8_lossy(bytes);
    let chars: Vec<char> = text.chars().collect();
    let tail = |n: usize| chars[chars.len() - n..].iter().collect::<String>();
    if let Some(prefix) = prefix_of(rule, bytes) {
        return if chars.len() >= prefix.len() + 16 {
            format!("{prefix}****{}", tail(4))
        } else {
            format!("{prefix}****")
        };
    }
    if rule == RULE_JWT {
        return format!("eyJ****{}", tail(4));
    }
    if chars.len() >= 20 {
        format!("****{} ({} chars)", tail(4), chars.len())
    } else {
        format!("**** ({} chars)", chars.len())
    }
}

#[cfg(test)]
mod tests;
