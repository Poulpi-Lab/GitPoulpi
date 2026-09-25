//! The code of a request read through the host's API instead of a clone:
//! the diff per file, the commits, the diff of one commit and a file at a
//! revision. The shapes live here, the per-host calls in [`super::github`]
//! and [`super::gitlab`], behind `HostingClient` methods whose default is
//! `Unsupported`.
//!
//! A host's per-file `patch` has hunks but no `diff --git` header, so it is
//! parsed with [`gitpoulpi_core::diff_text::parse_hunks`], the parser the
//! system-git backend uses too.

use serde::{Deserialize, Serialize};

use gitpoulpi_core::domain::FileDiff;

/// One changed file with its hunks.
///
/// `omitted` says why a file that did change carries no hunks (the host
/// did not send its patch: too big, too many files). A binary file, a
/// rename without edits or a mode change has no hunks either and is *not*
/// omitted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiFileDiff {
    pub diff: FileDiff,
    /// The path at the merge base, for a rename.
    pub previous_path: Option<String>,
    pub additions: u32,
    pub deletions: u32,
    pub omitted: Option<String>,
}

/// The reason shown for a file whose patch the host left out.
pub const NOT_SHOWN_BY_THE_HOST: &str = "not shown by the host";

/// One commit of a request, oldest first in a list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiCommit {
    pub sha: String,
    pub parents: Vec<String>,
    pub author: String,
    pub date: String,
    pub summary: String,
}

/// The first line of a commit message.
#[cfg_attr(
    not(any(feature = "github", feature = "gitlab", feature = "gitea")),
    allow(dead_code)
)]
pub(crate) fn summary_of(message: &str) -> String {
    message
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .to_string()
}

/// The added and deleted lines of `diff`'s hunks.
#[cfg_attr(not(feature = "gitea"), allow(dead_code))]
pub(crate) fn hunk_counts(diff: &FileDiff) -> (u32, u32) {
    use gitpoulpi_core::domain::LineOrigin;
    let count = |origin: LineOrigin| {
        diff.hunks
            .iter()
            .flat_map(|h| h.lines.iter())
            .filter(|l| l.origin == origin)
            .count() as u32
    };
    (count(LineOrigin::Add), count(LineOrigin::Delete))
}

/// Decodes RFC 4648 base64 (standard alphabet), ignoring the line breaks
/// both hosts put in the `content` of a file. `None` for any other
/// character or a bad length.
#[cfg_attr(
    not(any(feature = "gitlab", feature = "gitea", feature = "gerrit")),
    allow(dead_code)
)]
pub(crate) fn base64_decode(text: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(text.len() / 4 * 3);
    let (mut acc, mut bits) = (0u32, 0u32);
    let mut padding = 0usize;
    for c in text.bytes() {
        let value = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => {
                padding += 1;
                continue;
            }
            b'\n' | b'\r' | b' ' => continue,
            _ => return None,
        };
        if padding > 0 {
            return None;
        }
        acc = (acc << 6) | u32::from(value);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    (padding <= 2).then_some(out)
}

/// `a/b c/d.rs` as `a/b%20c/d.rs`: each segment percent-encoded, the
/// slashes kept (GitHub's and Gitea's `contents` route).
#[cfg(any(feature = "github", feature = "gitea"))]
pub(crate) fn encode_path(path: &str) -> String {
    path.trim_start_matches('/')
        .split('/')
        .map(crate::client::encode_component)
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_decodes_rfc_vectors_with_and_without_line_breaks() {
        for (plain, encoded) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(base64_decode(encoded).unwrap(), plain.as_bytes());
        }
        assert_eq!(base64_decode("Zm9v\nYmFy\n").unwrap(), b"foobar");
        assert_eq!(base64_decode("AP8=").unwrap(), vec![0u8, 255]);
        assert!(base64_decode("Zm9v!").is_none());
        assert!(base64_decode("Zg==Zg").is_none());
    }

    #[test]
    fn a_summary_is_the_first_line() {
        assert_eq!(summary_of("fix: x\n\nlong body"), "fix: x");
        assert_eq!(summary_of(""), "");
    }
}
