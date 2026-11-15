//! Searching the files of a revision (`GitBackend::grep_tree`): what a hit is
//! and the rules both backends apply identically.
//!
//! The rules live here so the two backends cannot drift: a term is a plain
//! string (case-sensitive, no pattern), a hit is one line, a binary file, an
//! LFS pointer and a file over [`MAX_GREP_FILE_BYTES`] are never searched,
//! a hit's text is cut to [`MAX_HIT_TEXT_BYTES`], and the hits come out
//! sorted by path then line, at most `max_hits` for each term.

use serde::{Deserialize, Serialize};

use super::{decode_text_lossy, LfsPointer};

/// Files above this many bytes are not searched.
pub const MAX_GREP_FILE_BYTES: u64 = 1024 * 1024;

/// A hit's text is cut to this many bytes (on a character boundary).
pub const MAX_HIT_TEXT_BYTES: usize = 240;

/// How many leading bytes the binary sniff looks at (git's own heuristic).
const BINARY_SNIFF_BYTES: usize = 8000;

/// One line that contains a searched term.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrepHit {
    /// The term that matched.
    pub term: String,
    /// Repository-relative, `/` separated.
    pub path: String,
    /// 1-based.
    pub line: u32,
    /// The line without its line ending, cut to [`MAX_HIT_TEXT_BYTES`].
    pub text: String,
}

/// The terms worth searching: not blank, on one line, each once.
pub fn usable_terms(terms: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for term in terms {
        let term = term.trim();
        if term.is_empty() || term.contains('\n') || term.contains('\r') || term.contains('\0') {
            continue;
        }
        if !out.iter().any(|t| t == term) {
            out.push(term.to_string());
        }
    }
    out
}

/// A line cut to [`MAX_HIT_TEXT_BYTES`] without its `\r`.
pub fn hit_text(line: &str) -> String {
    let line = line.strip_suffix('\r').unwrap_or(line);
    if line.len() <= MAX_HIT_TEXT_BYTES {
        return line.to_string();
    }
    let mut end = MAX_HIT_TEXT_BYTES;
    while !line.is_char_boundary(end) {
        end -= 1;
    }
    line[..end].to_string()
}

/// Whether `raw` is searched at all: text (no NUL in the first 8000 bytes)
/// that is not an LFS pointer and not over [`MAX_GREP_FILE_BYTES`].
pub fn is_searchable(raw: &[u8]) -> bool {
    if raw.len() as u64 > MAX_GREP_FILE_BYTES {
        return false;
    }
    if raw[..raw.len().min(BINARY_SNIFF_BYTES)].contains(&0) {
        return false;
    }
    LfsPointer::parse(raw).is_none()
}

/// The hits of `terms` (already [`usable_terms`]) in one file's bytes; empty
/// for a file that is not [`is_searchable`].
pub fn grep_bytes(path: &str, raw: &[u8], terms: &[String]) -> Vec<GrepHit> {
    if !is_searchable(raw) {
        return Vec::new();
    }
    let text = decode_text_lossy(raw);
    let mut hits = Vec::new();
    for (index, line) in text.split('\n').enumerate() {
        for term in terms {
            if line.contains(term.as_str()) {
                hits.push(GrepHit {
                    term: term.clone(),
                    path: path.to_string(),
                    line: index as u32 + 1,
                    text: hit_text(line),
                });
            }
        }
    }
    hits
}

/// Sorts `hits` by path, line, then the order of `terms`, and keeps at most
/// `max_hits` of each term.
pub fn finish_hits(mut hits: Vec<GrepHit>, terms: &[String], max_hits: usize) -> Vec<GrepHit> {
    let rank = |term: &str| terms.iter().position(|t| t == term).unwrap_or(usize::MAX);
    hits.sort_by(|a, b| {
        (a.path.as_str(), a.line, rank(&a.term)).cmp(&(b.path.as_str(), b.line, rank(&b.term)))
    });
    let mut count: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    hits.retain(|hit| {
        let n = count.entry(hit.term.clone()).or_insert(0);
        *n += 1;
        *n <= max_hits
    });
    hits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terms_are_trimmed_deduplicated_and_single_line() {
        let terms = usable_terms(&[
            " a ".into(),
            "a".into(),
            "".into(),
            "x\ny".into(),
            "b".into(),
        ]);
        assert_eq!(terms, ["a", "b"]);
    }

    #[test]
    fn binary_and_pointer_files_are_not_searched() {
        assert!(grep_bytes("a.bin", b"foo\0bar", &["foo".into()]).is_empty());
        let pointer = format!(
            "version https://git-lfs.github.com/spec/v1\noid sha256:{}\nsize 12\n",
            "a".repeat(64)
        );
        assert!(grep_bytes("p", pointer.as_bytes(), &["oid".into()]).is_empty());
        assert_eq!(
            grep_bytes("t", b"x foo\r\ny", &["foo".into()])[0].text,
            "x foo"
        );
    }

    #[test]
    fn hits_are_capped_per_term_after_sorting() {
        let terms = vec!["a".to_string(), "b".to_string()];
        let mut hits = Vec::new();
        for path in ["z", "m", "c"] {
            hits.extend(grep_bytes(path, b"a b\n", &terms));
        }
        let hits = finish_hits(hits, &terms, 2);
        let keys: Vec<(&str, &str)> = hits
            .iter()
            .map(|h| (h.path.as_str(), h.term.as_str()))
            .collect();
        assert_eq!(keys, [("c", "a"), ("c", "b"), ("m", "a"), ("m", "b")]);
    }
}
