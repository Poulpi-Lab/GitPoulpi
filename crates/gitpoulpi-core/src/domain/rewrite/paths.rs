//! Path rules: glob matching (`RemovePaths`, content rules) and the
//! rename / into-subdirectory / extract-subdirectory mapping. Paths are
//! repository-relative with `/` separators.

use regex::Regex;
use serde::{Deserialize, Serialize};

/// A set of git-style globs. A pattern without a `/` matches at any depth
/// (like `.gitignore`); with one it is anchored at the repository root. `*`
/// and `?` stay inside a path segment, `**` crosses segments, `[abc]` and
/// `[!abc]` are classes. A pattern that matches a directory matches
/// everything below it.
#[derive(Debug, Clone)]
pub struct GlobSet {
    regexes: Vec<Regex>,
}

impl GlobSet {
    pub fn new<S: AsRef<str>>(globs: &[S]) -> Result<Self, String> {
        let regexes = globs
            .iter()
            .map(|g| glob_regex(g.as_ref()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { regexes })
    }

    pub fn is_match(&self, path: &str) -> bool {
        let path = path.trim_start_matches('/');
        self.regexes.iter().any(|re| re.is_match(path))
    }

    pub fn is_empty(&self) -> bool {
        self.regexes.is_empty()
    }
}

/// Checks one glob and returns the reason it is refused.
pub fn validate_glob(glob: &str) -> Result<(), String> {
    glob_regex(glob).map(|_| ())
}

fn glob_regex(glob: &str) -> Result<Regex, String> {
    let trimmed = glob.trim();
    if trimmed.is_empty() {
        return Err("the path pattern is empty".to_string());
    }
    if trimmed.contains('\0') || trimmed.contains('\n') {
        return Err("the path pattern has a control character".to_string());
    }
    if trimmed.split('/').any(|seg| seg == "..") {
        return Err("a path pattern cannot contain `..`".to_string());
    }
    let anchored_by_slash = trimmed.trim_end_matches('/').contains('/');
    let body = trimmed.trim_start_matches('/').trim_end_matches('/');
    if body.is_empty() {
        return Err("the path pattern is empty".to_string());
    }
    let mut re = String::from("^");
    if !anchored_by_slash {
        re.push_str("(?:.*/)?");
    }
    let chars: Vec<char> = body.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '*' if chars.get(i + 1) == Some(&'*') => {
                i += 2;
                if chars.get(i) == Some(&'/') {
                    i += 1;
                    re.push_str("(?:.*/)?");
                } else {
                    re.push_str(".*");
                }
                continue;
            }
            '*' => re.push_str("[^/]*"),
            '?' => re.push_str("[^/]"),
            '[' => {
                let Some(close) = chars[i + 1..].iter().position(|c| *c == ']') else {
                    return Err("a `[` in the path pattern is not closed".to_string());
                };
                let class: String = chars[i + 1..i + 1 + close].iter().collect();
                if class.is_empty() {
                    return Err("an empty `[]` in the path pattern".to_string());
                }
                re.push('[');
                match class.strip_prefix('!') {
                    Some(rest) => {
                        re.push('^');
                        re.push_str(&escape_class(rest));
                    }
                    None => re.push_str(&escape_class(&class)),
                }
                re.push(']');
                i += close + 2;
                continue;
            }
            c => re.push_str(&regex::escape(&c.to_string())),
        }
        i += 1;
    }
    re.push_str("(?:/.*)?$");
    Regex::new(&re).map_err(|e| e.to_string())
}

fn escape_class(class: &str) -> String {
    class
        .chars()
        .map(|c| match c {
            '\\' | '^' | '[' | '&' | '~' => format!("\\{c}"),
            c => c.to_string(),
        })
        .collect()
}

/// How an extracted subdirectory ends up. Asked each time: there is no
/// default, [`PathMove::ExtractSubdir`] carries an `Option` and validation
/// refuses `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtractMode {
    /// A new local clone holding only the subdirectory's history, opened in
    /// a new tab; the original repository is not touched.
    Clone,
    /// This repository's history is rewritten to the subdirectory.
    InPlace,
    /// The subdirectory becomes its own repository and the original
    /// references it as a submodule.
    Submodule,
}

/// One path-moving rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "move", rename_all = "snake_case")]
pub enum PathMove {
    /// `from` (a file or directory) becomes `to`.
    Rename { from: String, to: String },
    /// Everything moves under `dir`.
    IntoSubdir { dir: String },
    /// Keep only `dir`, which becomes the root.
    ExtractSubdir {
        dir: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mode: Option<ExtractMode>,
    },
}

/// A directory-ish path with no empty segment, `.`, `..` or edge slashes.
pub fn clean_dir(path: &str) -> Option<String> {
    let trimmed = path.trim().trim_matches('/');
    if trimmed.is_empty() || trimmed.contains('\0') || trimmed.contains('\\') {
        return None;
    }
    if trimmed
        .split('/')
        .any(|seg| seg.is_empty() || seg == "." || seg == "..")
    {
        return None;
    }
    Some(trimmed.to_string())
}

impl PathMove {
    /// Why this rule is not valid, if it is not.
    pub fn problem(&self) -> Option<String> {
        match self {
            PathMove::Rename { from, to } => {
                let (Some(f), Some(t)) = (clean_dir(from), clean_dir(to)) else {
                    return Some("both paths must be non-empty relative paths".to_string());
                };
                if f == t {
                    return Some("the old and new paths are the same".to_string());
                }
                None
            }
            PathMove::IntoSubdir { dir } => clean_dir(dir)
                .is_none()
                .then(|| "the directory must be a non-empty relative path".to_string()),
            PathMove::ExtractSubdir { dir, mode } => {
                if clean_dir(dir).is_none() {
                    Some("the directory must be a non-empty relative path".to_string())
                } else if mode.is_none() {
                    Some("choose how to extract: clone, in place or submodule".to_string())
                } else {
                    None
                }
            }
        }
    }

    /// Where `path` ends up, or `None` when the path is dropped (outside an
    /// extracted subdirectory).
    pub fn map_path(&self, path: &str) -> Option<String> {
        match self {
            PathMove::Rename { from, to } => {
                let (f, t) = (clean_dir(from)?, clean_dir(to)?);
                if path == f {
                    Some(t)
                } else {
                    Some(
                        path.strip_prefix(&f)
                            .and_then(|rest| rest.strip_prefix('/'))
                            .map_or_else(|| path.to_string(), |rest| format!("{t}/{rest}")),
                    )
                }
            }
            PathMove::IntoSubdir { dir } => Some(format!("{}/{path}", clean_dir(dir)?)),
            PathMove::ExtractSubdir { dir, .. } => {
                let d = clean_dir(dir)?;
                path.strip_prefix(&d)
                    .and_then(|rest| rest.strip_prefix('/'))
                    .map(str::to_string)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(globs: &[&str]) -> GlobSet {
        GlobSet::new(globs).unwrap()
    }

    #[test]
    fn a_slashless_glob_matches_at_any_depth() {
        let s = set(&["*.log"]);
        assert!(s.is_match("a.log"));
        assert!(s.is_match("deep/er/a.log"));
        assert!(!s.is_match("a.logx"));
        assert!(!s.is_match("a.log.txt"));
    }

    #[test]
    fn a_glob_with_a_slash_is_anchored_and_stars_stay_in_a_segment() {
        let s = set(&["docs/*.md"]);
        assert!(s.is_match("docs/a.md"));
        assert!(!s.is_match("docs/sub/a.md"));
        assert!(!s.is_match("x/docs/a.md"));
        let d = set(&["docs/**/*.md"]);
        assert!(d.is_match("docs/a.md"));
        assert!(d.is_match("docs/sub/deep/a.md"));
    }

    #[test]
    fn a_directory_pattern_matches_what_is_below_it() {
        let s = set(&["secrets/", "build"]);
        assert!(s.is_match("secrets/key.pem"));
        assert!(s.is_match("a/b/build/out.o"));
        assert!(!s.is_match("secretsx/key"));
    }

    #[test]
    fn classes_and_question_marks() {
        let s = set(&["file?.[ch]", "x[!a]"]);
        assert!(s.is_match("file1.c"));
        assert!(s.is_match("sub/fileZ.h"));
        assert!(!s.is_match("file12.c"));
        assert!(s.is_match("xb"));
        assert!(!s.is_match("xa"));
    }

    #[test]
    fn bad_globs_are_refused() {
        for bad in ["", "/", "a/../b", "[abc", "[]"] {
            assert!(validate_glob(bad).is_err(), "{bad:?}");
        }
        assert!(validate_glob("**/node_modules").is_ok());
    }

    #[test]
    fn rename_moves_a_file_or_a_whole_directory() {
        let m = PathMove::Rename {
            from: "src/old".into(),
            to: "lib/new".into(),
        };
        assert_eq!(m.map_path("src/old"), Some("lib/new".into()));
        assert_eq!(m.map_path("src/old/a.rs"), Some("lib/new/a.rs".into()));
        assert_eq!(m.map_path("src/older/a.rs"), Some("src/older/a.rs".into()));
    }

    #[test]
    fn into_subdir_and_extract_are_inverse() {
        let into = PathMove::IntoSubdir { dir: "app".into() };
        assert_eq!(into.map_path("a/b.rs"), Some("app/a/b.rs".into()));
        let out = PathMove::ExtractSubdir {
            dir: "app".into(),
            mode: Some(ExtractMode::Clone),
        };
        assert_eq!(out.map_path("app/a/b.rs"), Some("a/b.rs".into()));
        assert_eq!(out.map_path("other/b.rs"), None);
        assert_eq!(out.map_path("app"), None);
    }

    #[test]
    fn extract_needs_an_explicit_mode() {
        let ask = PathMove::ExtractSubdir {
            dir: "app".into(),
            mode: None,
        };
        assert!(ask.problem().unwrap().contains("choose how"));
        for mode in [
            ExtractMode::Clone,
            ExtractMode::InPlace,
            ExtractMode::Submodule,
        ] {
            let ok = PathMove::ExtractSubdir {
                dir: "app".into(),
                mode: Some(mode),
            };
            assert_eq!(ok.problem(), None);
        }
        assert!(PathMove::IntoSubdir { dir: "../x".into() }
            .problem()
            .is_some());
        assert!(PathMove::Rename {
            from: "a".into(),
            to: "a/".into()
        }
        .problem()
        .is_some());
    }
}
