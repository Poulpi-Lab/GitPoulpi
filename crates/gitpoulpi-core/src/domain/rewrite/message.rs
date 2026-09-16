//! Message rules: search and replace, delete matching lines. Pure functions
//! over text; the engine decodes a message, calls these, and keeps the bytes
//! of a message that is not valid UTF-8 untouched.

use regex::{NoExpand, Regex, RegexBuilder};
use serde::{Deserialize, Serialize};

/// What a text rule looks for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextPattern {
    pub pattern: String,
    /// `false`: `pattern` is a literal string.
    #[serde(default)]
    pub regex: bool,
    #[serde(default = "yes")]
    pub case_sensitive: bool,
}

fn yes() -> bool {
    true
}

impl TextPattern {
    pub fn literal(text: &str) -> Self {
        Self {
            pattern: text.to_string(),
            regex: false,
            case_sensitive: true,
        }
    }

    pub fn regex(pattern: &str) -> Self {
        Self {
            pattern: pattern.to_string(),
            regex: true,
            case_sensitive: true,
        }
    }
}

/// Replace every match with `with`. A deletion is an empty replacement. In
/// regex mode `with` may use `$1` / `${name}`; in literal mode it is verbatim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextReplace {
    pub find: TextPattern,
    #[serde(default)]
    pub with: String,
}

/// A [`TextPattern`] compiled once (the engine runs it per commit).
#[derive(Debug, Clone)]
pub struct CompiledPattern {
    re: Regex,
    literal: bool,
}

impl CompiledPattern {
    /// Fails (with the reason) for an empty pattern or a regex that does not
    /// compile or is too large.
    pub fn new(pattern: &TextPattern) -> Result<Self, String> {
        if pattern.pattern.is_empty() {
            return Err("the pattern is empty".to_string());
        }
        let source = if pattern.regex {
            pattern.pattern.clone()
        } else {
            regex::escape(&pattern.pattern)
        };
        let re = RegexBuilder::new(&source)
            .case_insensitive(!pattern.case_sensitive)
            .size_limit(1 << 20)
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Self {
            re,
            literal: !pattern.regex,
        })
    }

    pub fn is_match(&self, text: &str) -> bool {
        self.re.is_match(text)
    }

    /// `text` with every match replaced.
    pub fn replace_all(&self, text: &str, with: &str) -> String {
        if self.literal {
            self.re.replace_all(text, NoExpand(with)).into_owned()
        } else {
            self.re.replace_all(text, with).into_owned()
        }
    }

    /// `text` without the lines that contain a match. Line endings of the
    /// kept lines (and a missing final newline) are preserved.
    pub fn delete_lines(&self, text: &str) -> String {
        text.split_inclusive('\n')
            .filter(|line| !self.re.is_match(line.trim_end_matches(['\n', '\r'])))
            .collect()
    }
}

/// Applies a replace rule to one message.
pub fn replace(text: &str, rule: &TextReplace) -> Result<String, String> {
    Ok(CompiledPattern::new(&rule.find)?.replace_all(text, &rule.with))
}

/// Removes the lines of `text` that match `pattern`.
pub fn delete_lines(text: &str, pattern: &TextPattern) -> Result<String, String> {
    Ok(CompiledPattern::new(pattern)?.delete_lines(text))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(find: TextPattern, with: &str) -> TextReplace {
        TextReplace {
            find,
            with: with.to_string(),
        }
    }

    #[test]
    fn literal_replace_does_not_interpret_the_pattern_or_the_replacement() {
        let out = replace(
            "fix a.b and a.b ($1)",
            &rule(TextPattern::literal("a.b"), "x$1"),
        )
        .unwrap();
        assert_eq!(out, "fix x$1 and x$1 ($1)");
        assert_eq!(
            replace("axb", &rule(TextPattern::literal("a.b"), "!")).unwrap(),
            "axb"
        );
    }

    #[test]
    fn regex_replace_expands_groups() {
        let out = replace(
            "PROJ-12 and PROJ-7",
            &rule(TextPattern::regex(r"PROJ-(\d+)"), "#$1"),
        )
        .unwrap();
        assert_eq!(out, "#12 and #7");
    }

    #[test]
    fn replacing_with_nothing_deletes_the_match() {
        let out = replace("a secret b", &rule(TextPattern::literal(" secret"), "")).unwrap();
        assert_eq!(out, "a b");
    }

    #[test]
    fn case_insensitive_matching() {
        let mut p = TextPattern::literal("wip");
        p.case_sensitive = false;
        assert_eq!(replace("WIP: x", &rule(p, "done")).unwrap(), "done: x");
    }

    #[test]
    fn delete_lines_keeps_the_rest_byte_for_byte() {
        let text = "subject\n\nbody\nSigned-off-by: Alice Example <alice@example.com>\r\nlast";
        let out = delete_lines(text, &TextPattern::regex("^Signed-off-by:")).unwrap();
        assert_eq!(out, "subject\n\nbody\nlast");
        // Nothing matches: unchanged, including a missing final newline.
        assert_eq!(
            delete_lines("a\nb", &TextPattern::literal("zzz")).unwrap(),
            "a\nb"
        );
    }

    #[test]
    fn bad_patterns_are_reported() {
        assert!(CompiledPattern::new(&TextPattern::literal("")).is_err());
        assert!(CompiledPattern::new(&TextPattern::regex("(")).is_err());
        assert!(replace("x", &rule(TextPattern::regex("["), "")).is_err());
    }
}
