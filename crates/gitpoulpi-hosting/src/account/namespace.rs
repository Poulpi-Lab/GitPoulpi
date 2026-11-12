//! The profile's Namespaces filter: which repositories the account-wide
//! lists keep. Applied to every list the Radar shows (review requests,
//! assigned issues, every open request), because the host's own queries for
//! the first two are global and ignore the namespaces.
//!
//! Plain mode: a line matches a repository's full path (`group/sub/proj`)
//! when it equals the path or is a path prefix ending at a `/` boundary
//! (`ptm` matches `ptm/produits/x`, never `ptmx/y`), ignoring case. Regex
//! mode: each line is a regular expression run against the full path as the
//! user wrote it (unanchored unless the pattern says `^` / `$`).

use regex::Regex;

/// A compiled Namespaces filter; empty = keep everything.
#[derive(Debug, Clone)]
pub struct NamespaceFilter {
    plain: Vec<String>,
    regexes: Vec<Regex>,
}

impl NamespaceFilter {
    /// Compiles `lines`; in regex mode the first invalid line is an error
    /// naming it and the regex engine's reason.
    pub fn new(lines: &[String], regex: bool) -> Result<Self, String> {
        let lines = lines.iter().map(|l| l.trim()).filter(|l| !l.is_empty());
        if regex {
            let mut regexes = Vec::new();
            for line in lines {
                regexes.push(compile(line)?);
            }
            Ok(Self {
                plain: Vec::new(),
                regexes,
            })
        } else {
            Ok(Self {
                plain: lines
                    .map(|l| l.trim_matches('/').to_lowercase())
                    .filter(|l| !l.is_empty())
                    .collect(),
                regexes: Vec::new(),
            })
        }
    }

    /// Whether no line is set (the filter keeps every repository).
    pub fn is_empty(&self) -> bool {
        self.plain.is_empty() && self.regexes.is_empty()
    }

    /// Whether the repository `path` (`owner/name`, `group/sub/proj`) is kept.
    pub fn matches(&self, path: &str) -> bool {
        if self.is_empty() {
            return true;
        }
        let lower = path.trim_matches('/').to_lowercase();
        self.plain.iter().any(|ns| {
            lower == *ns
                || lower
                    .strip_prefix(ns.as_str())
                    .is_some_and(|rest| rest.starts_with('/'))
        }) || self.regexes.iter().any(|re| re.is_match(path))
    }
}

/// One regex line, or `"<line>: <why>"`.
pub fn compile(line: &str) -> Result<Regex, String> {
    Regex::new(line).map_err(|e| format!("{line}: {}", headline(&e.to_string())))
}

/// The regex error's headline, without its multi-line source drawing.
fn headline(error: &str) -> String {
    error
        .lines()
        .rev()
        .find(|l| l.trim_start().starts_with("error:"))
        .map(|l| {
            l.trim_start()
                .trim_start_matches("error:")
                .trim()
                .to_string()
        })
        .unwrap_or_else(|| error.lines().next().unwrap_or("").to_string())
}

/// The first invalid line of `lines` in regex mode, as `"<line>: <why>"`.
pub fn first_regex_error(lines: &[String]) -> Option<String> {
    lines
        .iter()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .find_map(|l| compile(l).err())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    fn plain(items: &[&str]) -> NamespaceFilter {
        NamespaceFilter::new(&lines(items), false).unwrap()
    }

    #[test]
    fn a_plain_namespace_is_a_path_prefix_at_a_slash() {
        let f = plain(&["ptm"]);
        assert!(f.matches("ptm/produits/x"));
        assert!(f.matches("ptm/x"));
        assert!(f.matches("ptm"));
        assert!(f.matches("PTM/x"));
        assert!(!f.matches("ptmx/y"));
        assert!(!f.matches("formation/ptm/x"));
        assert!(!f.matches("formation/a"));
    }

    #[test]
    fn a_nested_namespace_and_a_full_project_path_match() {
        let f = plain(&["ptm/produits", "other/repo"]);
        assert!(f.matches("ptm/produits/x/y"));
        assert!(f.matches("other/repo"));
        assert!(!f.matches("ptm/formation/x"));
        assert!(!f.matches("other/repo2"));
        assert!(plain(&["/ptm/"]).matches("ptm/x"));
    }

    #[test]
    fn several_lines_are_alternatives_and_none_keeps_everything() {
        let f = plain(&["ptm", "acme"]);
        assert!(f.matches("acme/a") && f.matches("ptm/b") && !f.matches("formation/c"));
        let none = plain(&[]);
        assert!(none.is_empty() && none.matches("anything/at/all"));
        assert!(plain(&["  ", ""]).is_empty());
    }

    #[test]
    fn regex_lines_match_the_full_path_as_written() {
        let f = NamespaceFilter::new(&lines(&["^ptm/(produits|infra)/", "docs$"]), true).unwrap();
        assert!(f.matches("ptm/produits/x"));
        assert!(f.matches("ptm/infra/y"));
        assert!(f.matches("formation/docs"));
        assert!(!f.matches("ptm/formation/x"));
        assert!(!f.matches("formation/docs2"));
        // Unanchored: a bare word matches anywhere in the path.
        let loose = NamespaceFilter::new(&lines(&["ptm"]), true).unwrap();
        assert!(loose.matches("formation/ptmx/y"));
    }

    #[test]
    fn an_invalid_regex_is_an_error_naming_the_line_never_a_panic() {
        let bad = lines(&["^ok/", "(unclosed"]);
        let err = NamespaceFilter::new(&bad, true).unwrap_err();
        assert!(err.starts_with("(unclosed: "), "{err}");
        assert_eq!(first_regex_error(&bad), Some(err));
        assert_eq!(first_regex_error(&lines(&["^ok/"])), None);
        // The same text is a fine plain namespace.
        assert!(NamespaceFilter::new(&bad, false).is_ok());
    }
}
