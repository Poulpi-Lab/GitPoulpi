//! The pure side of the LFS import / export rules: the `.gitattributes`
//! lines a rewritten tree carries. The pointer format itself is
//! `domain::LfsPointer` (spec v1).
//!
//! The rest of a `.gitattributes` file is never re-written: lines are added
//! at the end or removed one by one and every other byte stays as it was.

/// What a tracked pattern's line says after the pattern: the attributes
/// `git lfs track` writes.
pub const LFS_ATTRIBUTES: &str = "filter=lfs diff=lfs merge=lfs -text";

/// The pattern as a `.gitattributes` token: attribute files split on
/// whitespace, so a space in a pattern is spelled `[[:space:]]`.
pub fn attribute_token(pattern: &str) -> String {
    let mut out = String::with_capacity(pattern.len());
    for c in pattern.trim().chars() {
        if c.is_whitespace() {
            out.push_str("[[:space:]]");
        } else {
            out.push(c);
        }
    }
    out
}

/// The line a pattern adds.
pub fn attribute_line(pattern: &str) -> String {
    format!("{} {LFS_ATTRIBUTES}", attribute_token(pattern))
}

/// Whether `line` routes `token` through the LFS filter.
fn is_lfs_line(line: &str, token: &str) -> bool {
    let mut parts = line.split_whitespace();
    parts.next() == Some(token) && parts.any(|a| a == "filter=lfs")
}

/// `existing` (a `.gitattributes`, `None` when there is none) with a line for
/// every pattern that has none yet. `None` when nothing needs adding. A
/// missing final newline is added before the new lines; nothing else of the
/// old text moves.
pub fn add_attributes(existing: Option<&[u8]>, patterns: &[String]) -> Option<Vec<u8>> {
    let old = existing.unwrap_or_default();
    let text = String::from_utf8_lossy(old);
    let mut out = old.to_vec();
    let mut changed = false;
    let mut seen: Vec<String> = Vec::new();
    for pattern in patterns {
        let token = attribute_token(pattern);
        if seen.contains(&token) || text.lines().any(|l| is_lfs_line(l, &token)) {
            continue;
        }
        if !out.is_empty() && out.last() != Some(&b'\n') {
            out.push(b'\n');
        }
        out.extend_from_slice(attribute_line(pattern).as_bytes());
        out.push(b'\n');
        seen.push(token);
        changed = true;
    }
    changed.then_some(out)
}

/// `existing` without the LFS lines of `patterns`. `None` when it had none.
/// The rest of the file is byte for byte as it was.
pub fn remove_attributes(existing: &[u8], patterns: &[String]) -> Option<Vec<u8>> {
    let tokens: Vec<String> = patterns.iter().map(|p| attribute_token(p)).collect();
    let mut out = Vec::with_capacity(existing.len());
    let mut changed = false;
    for line in existing.split_inclusive(|b| *b == b'\n') {
        let text = String::from_utf8_lossy(line);
        if tokens.iter().any(|t| is_lfs_line(&text, t)) {
            changed = true;
        } else {
            out.extend_from_slice(line);
        }
    }
    changed.then_some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pats(p: &[&str]) -> Vec<String> {
        p.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn a_missing_file_gets_one_line_per_pattern() {
        let out = add_attributes(None, &pats(&["*.bin", "assets/**"])).unwrap();
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "*.bin filter=lfs diff=lfs merge=lfs -text\nassets/** filter=lfs diff=lfs merge=lfs -text\n"
        );
    }

    #[test]
    fn the_rest_of_the_file_stays_byte_identical_and_lines_are_not_duplicated() {
        let old = b"* text=auto\r\n# keep\n*.bin filter=lfs diff=lfs merge=lfs -text\n*.png -diff";
        // `*.bin` is there already; `*.psd` is new; no final newline before.
        let out = add_attributes(Some(old), &pats(&["*.bin", "*.psd", "*.psd"])).unwrap();
        let mut want = old.to_vec();
        want.extend_from_slice(b"\n*.psd filter=lfs diff=lfs merge=lfs -text\n");
        assert_eq!(out, want);
        assert_eq!(add_attributes(Some(&out), &pats(&["*.bin", "*.psd"])), None);
    }

    #[test]
    fn a_pattern_with_a_space_is_written_as_a_class() {
        assert_eq!(attribute_token("my file.bin"), "my[[:space:]]file.bin");
    }

    #[test]
    fn remove_drops_only_the_lfs_lines_of_the_patterns() {
        let old = b"* text=auto\r\n*.bin filter=lfs diff=lfs merge=lfs -text\n*.bin -diff\n*.psd filter=lfs diff=lfs merge=lfs -text\n";
        let out = remove_attributes(old, &pats(&["*.bin"])).unwrap();
        assert_eq!(
            out,
            b"* text=auto\r\n*.bin -diff\n*.psd filter=lfs diff=lfs merge=lfs -text\n"
        );
        assert_eq!(remove_attributes(&out, &pats(&["*.bin"])), None);
    }
}
