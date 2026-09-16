//! Branch name patterns: literal text around `{token}` placeholders.
//!
//! `feature/{slug}`, `release/{version}`, `{user}/{issue}-{slug}`,
//! `bot/{issue_key}`. A pattern renders fields into a name and parses a name
//! back into fields; for any fields `f` that render, `parse(render(f))`
//! yields the same normalised fields (and `render(parse(n)) == n` for names
//! the pattern itself produces).

use std::collections::BTreeMap;

use super::refname::{check_ref_format, slugify, RefFormatError, DEFAULT_SLUG_MAX};

/// Field values by token name (`slug`, `issue`, ...).
pub type Fields = BTreeMap<String, String>;

/// A placeholder a pattern may contain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Token {
    /// Free text, slugified when rendered (may contain `/` when last).
    Slug,
    /// An issue number (`123`; a leading `#` is dropped when rendering).
    Issue,
    /// A tracker key (`PROJ-123`).
    IssueKey,
    /// A user name (slugified when rendered, never contains `/`).
    User,
    /// `2026-10-16` or `20261016`.
    Date,
    /// `1.2.0`, `v1.2`, `1.2.0-rc1`.
    Version,
    /// A counter (digits).
    N,
}

impl Token {
    pub const ALL: [Token; 7] = [
        Token::Slug,
        Token::Issue,
        Token::IssueKey,
        Token::User,
        Token::Date,
        Token::Version,
        Token::N,
    ];

    /// The name between the braces.
    pub fn name(self) -> &'static str {
        match self {
            Token::Slug => "slug",
            Token::Issue => "issue",
            Token::IssueKey => "issue_key",
            Token::User => "user",
            Token::Date => "date",
            Token::Version => "version",
            Token::N => "n",
        }
    }

    pub fn from_name(name: &str) -> Option<Token> {
        Token::ALL.into_iter().find(|t| t.name() == name)
    }

    /// A value that is valid for this token, used to validate a pattern.
    fn sample(self) -> &'static str {
        match self {
            Token::Slug => "x",
            Token::Issue | Token::N => "1",
            Token::IssueKey => "A-1",
            Token::User => "u",
            Token::Date => "2026-01-01",
            Token::Version => "1.0.0",
        }
    }

    /// Whether `s` (non-empty) is a value of this token. `slash` allows a `/`
    /// (only the last token of a pattern may swallow path separators).
    fn accepts(self, s: &str, slash: bool) -> bool {
        if s.is_empty() {
            return false;
        }
        match self {
            Token::Slug => (slash || !s.contains('/')) && !s.starts_with('-') && ref_ok(s),
            Token::User => !s.contains('/') && !s.starts_with('-') && ref_ok(s),
            Token::Issue | Token::N => s.bytes().all(|b| b.is_ascii_digit()),
            Token::IssueKey => {
                let Some((project, number)) = s.rsplit_once('-') else {
                    return false;
                };
                project.starts_with(|c: char| c.is_ascii_alphabetic())
                    && project.bytes().all(|b| b.is_ascii_alphanumeric())
                    && !number.is_empty()
                    && number.bytes().all(|b| b.is_ascii_digit())
            }
            Token::Date => is_date(s),
            Token::Version => is_version(s),
        }
    }
}

/// A component-wise valid ref (so a `/` inside is fine, a `..` is not).
fn ref_ok(s: &str) -> bool {
    check_ref_format(s).is_ok()
}

fn is_date(s: &str) -> bool {
    let b = s.as_bytes();
    let digits =
        |r: std::ops::Range<usize>| b.get(r).is_some_and(|d| d.iter().all(u8::is_ascii_digit));
    match b.len() {
        8 => digits(0..8),
        10 => digits(0..4) && b[4] == b'-' && digits(5..7) && b[7] == b'-' && digits(8..10),
        _ => false,
    }
}

fn is_version(s: &str) -> bool {
    let s = s.strip_prefix(['v', 'V']).unwrap_or(s);
    let core_end = s
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(s.len());
    let (core, rest) = s.split_at(core_end);
    if core.is_empty() || core.starts_with('.') || core.ends_with('.') || core.contains("..") {
        return false;
    }
    if !core.starts_with(|c: char| c.is_ascii_digit()) {
        return false;
    }
    match rest.chars().next() {
        None => true,
        Some('-') | Some('+') => {
            let tail = &rest[1..];
            !tail.is_empty()
                && tail
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
                && !tail.ends_with('.')
                && !tail.contains("..")
        }
        _ => false,
    }
}

/// Why a pattern could not be built or rendered.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PatternError {
    #[error("the pattern is empty")]
    Empty,
    #[error("unclosed `{{` in the pattern")]
    Unclosed,
    #[error("stray `}}` in the pattern")]
    StrayBrace,
    #[error("unknown token `{{{0}}}` (known: slug, issue, issue_key, user, date, version, n)")]
    UnknownToken(String),
    #[error("`{{{0}}}` appears twice in the pattern")]
    DuplicateToken(String),
    #[error("two tokens need text between them to be told apart")]
    AdjacentTokens,
    #[error("the pattern does not make valid branch names: {0}")]
    InvalidRef(RefFormatError),
    #[error("missing value for `{{{0}}}`")]
    MissingField(String),
    #[error("`{value}` is not a valid `{{{token}}}`")]
    BadValue { token: String, value: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Piece {
    Lit(String),
    Tok(Token),
}

/// A parsed branch name pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchPattern {
    source: String,
    pieces: Vec<Piece>,
}

impl BranchPattern {
    /// Parses `source`. The pattern must be able to produce valid branch
    /// names (checked by rendering sample values).
    pub fn parse(source: &str) -> Result<Self, PatternError> {
        if source.is_empty() {
            return Err(PatternError::Empty);
        }
        let mut pieces = Vec::new();
        let mut lit = String::new();
        let mut rest = source;
        while let Some(c) = rest.chars().next() {
            match c {
                '{' => {
                    let end = rest.find('}').ok_or(PatternError::Unclosed)?;
                    let name = &rest[1..end];
                    let token = Token::from_name(name)
                        .ok_or_else(|| PatternError::UnknownToken(name.to_string()))?;
                    if pieces.contains(&Piece::Tok(token)) {
                        return Err(PatternError::DuplicateToken(name.to_string()));
                    }
                    if !lit.is_empty() {
                        pieces.push(Piece::Lit(std::mem::take(&mut lit)));
                    }
                    if matches!(pieces.last(), Some(Piece::Tok(_))) {
                        return Err(PatternError::AdjacentTokens);
                    }
                    pieces.push(Piece::Tok(token));
                    rest = &rest[end + 1..];
                }
                '}' => return Err(PatternError::StrayBrace),
                _ => {
                    lit.push(c);
                    rest = &rest[c.len_utf8()..];
                }
            }
        }
        if !lit.is_empty() {
            pieces.push(Piece::Lit(lit));
        }
        let pattern = Self {
            source: source.to_string(),
            pieces,
        };
        let sample: Fields = pattern
            .tokens()
            .into_iter()
            .map(|t| (t.name().to_string(), t.sample().to_string()))
            .collect();
        let name = pattern.render_unchecked(&sample)?;
        check_ref_format(&name).map_err(PatternError::InvalidRef)?;
        Ok(pattern)
    }

    pub fn as_str(&self) -> &str {
        &self.source
    }

    /// The tokens, in order of appearance.
    pub fn tokens(&self) -> Vec<Token> {
        self.pieces
            .iter()
            .filter_map(|p| match p {
                Piece::Tok(t) => Some(*t),
                Piece::Lit(_) => None,
            })
            .collect()
    }

    /// How many literal characters the pattern fixes: the classifier prefers
    /// the pattern that says more (`release/{version}` over `{slug}`).
    pub fn specificity(&self) -> usize {
        self.pieces
            .iter()
            .map(|p| match p {
                Piece::Lit(l) => l.chars().count(),
                Piece::Tok(_) => 0,
            })
            .sum()
    }

    /// The leading literal text (`feature/` of `feature/{slug}`).
    pub fn prefix(&self) -> &str {
        match self.pieces.first() {
            Some(Piece::Lit(l)) => l,
            _ => "",
        }
    }

    /// Renders `fields` into a branch name: `slug` and `user` are slugified,
    /// `issue` loses a leading `#`, every value is validated, the result must
    /// be a valid branch name.
    pub fn render(&self, fields: &Fields) -> Result<String, PatternError> {
        let name = self.render_unchecked(fields)?;
        check_ref_format(&name).map_err(PatternError::InvalidRef)?;
        Ok(name)
    }

    fn render_unchecked(&self, fields: &Fields) -> Result<String, PatternError> {
        let mut out = String::new();
        for piece in &self.pieces {
            match piece {
                Piece::Lit(l) => out.push_str(l),
                Piece::Tok(t) => {
                    let raw = fields
                        .get(t.name())
                        .map(|v| v.trim())
                        .filter(|v| !v.is_empty())
                        .ok_or_else(|| PatternError::MissingField(t.name().to_string()))?;
                    let value = match t {
                        Token::Slug | Token::User => slugify(raw, DEFAULT_SLUG_MAX),
                        Token::Issue => raw.trim_start_matches('#').to_string(),
                        _ => raw.to_string(),
                    };
                    if !t.accepts(&value, false) {
                        return Err(PatternError::BadValue {
                            token: t.name().to_string(),
                            value: raw.to_string(),
                        });
                    }
                    out.push_str(&value);
                }
            }
        }
        Ok(out)
    }

    /// The fields of `name` if it matches the whole pattern.
    pub fn parse_name(&self, name: &str) -> Option<Fields> {
        let mut out = Fields::new();
        self.match_from(0, name, &mut out).then_some(out)
    }

    fn match_from(&self, idx: usize, rest: &str, out: &mut Fields) -> bool {
        let Some(piece) = self.pieces.get(idx) else {
            return rest.is_empty();
        };
        match piece {
            Piece::Lit(l) => rest
                .strip_prefix(l.as_str())
                .is_some_and(|r| self.match_from(idx + 1, r, out)),
            Piece::Tok(t) => {
                let last = idx + 1 == self.pieces.len();
                if last {
                    if t.accepts(rest, true) {
                        out.insert(t.name().to_string(), rest.to_string());
                        return true;
                    }
                    return false;
                }
                // Tokens are never adjacent: the next piece is a literal.
                let Some(Piece::Lit(next)) = self.pieces.get(idx + 1) else {
                    return false;
                };
                for (end, _) in rest.match_indices(next.as_str()) {
                    let candidate = &rest[..end];
                    if t.accepts(candidate, false) {
                        out.insert(t.name().to_string(), candidate.to_string());
                        if self.match_from(idx + 1, &rest[end..], out) {
                            return true;
                        }
                        out.remove(t.name());
                    }
                }
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(pairs: &[(&str, &str)]) -> Fields {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn pat(s: &str) -> BranchPattern {
        BranchPattern::parse(s).unwrap()
    }

    #[test]
    fn renders_and_parses_a_prefix_pattern() {
        let p = pat("feature/{slug}");
        let name = p.render(&f(&[("slug", "Add Login Page")])).unwrap();
        assert_eq!(name, "feature/add-login-page");
        assert_eq!(p.parse_name(&name), Some(f(&[("slug", "add-login-page")])));
    }

    #[test]
    fn suffix_and_several_tokens() {
        let p = pat("{user}/{issue}-{slug}-wip");
        let name = p
            .render(&f(&[
                ("user", "Ana Maria"),
                ("issue", "#42"),
                ("slug", "Fix it"),
            ]))
            .unwrap();
        assert_eq!(name, "ana-maria/42-fix-it-wip");
        assert_eq!(
            p.parse_name(&name),
            Some(f(&[
                ("user", "ana-maria"),
                ("issue", "42"),
                ("slug", "fix-it")
            ]))
        );
        assert_eq!(p.parse_name("ana/x-fix-wip"), None, "issue must be digits");
        assert_eq!(p.parse_name("ana/42-fix"), None, "suffix is required");
    }

    #[test]
    fn every_token_round_trips() {
        let cases = [
            ("release/{version}", "version", "1.2.0-rc1"),
            ("release/{version}", "version", "v2.0"),
            ("bot/{issue_key}", "issue_key", "PROJ-123"),
            ("daily/{date}", "date", "2026-10-16"),
            ("daily/{date}", "date", "20261016"),
            ("wip/{n}", "n", "7"),
            ("i/{issue}", "issue", "9"),
            ("{user}/work", "user", "kai"),
        ];
        for (pattern, token, value) in cases {
            let p = pat(pattern);
            let name = p.render(&f(&[(token, value)])).unwrap();
            let back = p.parse_name(&name).unwrap_or_else(|| panic!("{name}"));
            assert_eq!(back.get(token).map(String::as_str), Some(value), "{name}");
            assert_eq!(p.render(&back).unwrap(), name);
        }
    }

    #[test]
    fn render_is_idempotent_through_parse() {
        let p = pat("{user}/{slug}");
        for input in ["Hello World", "  spaced  ", "Ünï-côdé", "a/b", "x..y"] {
            let name = p
                .render(&f(&[("user", "Me"), ("slug", input)]))
                .unwrap_or_else(|e| panic!("{input}: {e}"));
            let fields = p.parse_name(&name).unwrap();
            assert_eq!(p.render(&fields).unwrap(), name, "{input}");
        }
    }

    #[test]
    fn the_last_slug_may_hold_slashes_but_inner_tokens_may_not() {
        let p = pat("feature/{slug}");
        assert_eq!(
            p.parse_name("feature/ui/login"),
            Some(f(&[("slug", "ui/login")]))
        );
        let q = pat("{slug}-end");
        assert_eq!(q.parse_name("a/b-end"), None);
        assert_eq!(q.parse_name("a-b-end"), Some(f(&[("slug", "a-b")])));
    }

    #[test]
    fn backtracking_finds_the_split() {
        let p = pat("{slug}-{n}");
        assert_eq!(
            p.parse_name("fix-login-3"),
            Some(f(&[("slug", "fix-login"), ("n", "3")]))
        );
        assert_eq!(p.parse_name("fix-login-x"), None);
    }

    #[test]
    fn weird_input_never_panics_or_matches_wrongly() {
        let p = pat("feature/{slug}");
        for name in [
            "",
            "feature/",
            "feature",
            "Feature/x",
            "feature//x",
            "feature/-x",
            "feature/a..b",
            "feature/ünï",
            "xfeature/y",
            "feature/a b",
        ] {
            let r = p.parse_name(name);
            if let Some(fields) = r {
                assert!(check_ref_format(name).is_ok(), "{name}");
                assert_eq!(p.render(&fields).ok().as_deref().map(|_| ()), Some(()));
            }
        }
        assert_eq!(p.parse_name("feature/"), None);
        assert_eq!(p.parse_name("feature//x"), None);
        assert_eq!(p.parse_name("feature/-x"), None);
        assert_eq!(p.parse_name("feature/a..b"), None);
        assert_eq!(p.parse_name("Feature/x"), None);
    }

    #[test]
    fn unicode_literals_work() {
        let p = pat("été/{slug}");
        assert_eq!(p.parse_name("été/x"), Some(f(&[("slug", "x")])));
        assert_eq!(p.specificity(), 4);
    }

    #[test]
    fn bad_patterns_are_rejected() {
        let cases: &[(&str, PatternError)] = &[
            ("", PatternError::Empty),
            ("a/{slug", PatternError::Unclosed),
            ("a}/{slug}", PatternError::StrayBrace),
            ("a/{nope}", PatternError::UnknownToken("nope".into())),
            ("{slug}/{slug}", PatternError::DuplicateToken("slug".into())),
            ("{user}{slug}", PatternError::AdjacentTokens),
        ];
        for (src, want) in cases {
            assert_eq!(BranchPattern::parse(src).unwrap_err(), *want, "{src}");
        }
        for src in [
            "a b/{slug}",
            "a:/{slug}",
            "/{slug}",
            "{slug}/",
            "a..{slug}",
            "-{slug}",
        ] {
            assert!(
                matches!(
                    BranchPattern::parse(src),
                    Err(PatternError::InvalidRef(_)) | Err(PatternError::BadValue { .. })
                ),
                "{src}: {:?}",
                BranchPattern::parse(src)
            );
        }
    }

    #[test]
    fn render_errors() {
        let p = pat("release/{version}");
        assert_eq!(
            p.render(&f(&[])).unwrap_err(),
            PatternError::MissingField("version".into())
        );
        assert!(matches!(
            p.render(&f(&[("version", "banana")])).unwrap_err(),
            PatternError::BadValue { .. }
        ));
        let q = pat("feature/{slug}");
        assert!(matches!(
            q.render(&f(&[("slug", "!!!")])).unwrap_err(),
            PatternError::BadValue { .. }
        ));
        assert_eq!(
            q.render(&f(&[("slug", "   ")])).unwrap_err(),
            PatternError::MissingField("slug".into())
        );
    }

    #[test]
    fn versions_and_dates() {
        for v in ["1", "1.2", "1.2.3", "v1.2.3", "1.2.3-rc.1", "1.0.0+build5"] {
            assert!(is_version(v), "{v}");
        }
        for v in [
            "", "v", ".1", "1.", "1..2", "a1", "1.2-", "1.2x", "1.2-a..b",
        ] {
            assert!(!is_version(v), "{v}");
        }
        for d in ["2026-10-16", "20261016"] {
            assert!(is_date(d));
        }
        for d in ["2026-1-16", "2026/10/16", "2026101", "abcd-ef-gh"] {
            assert!(!is_date(d));
        }
    }

    #[test]
    fn issue_key_shape() {
        let p = pat("{issue_key}-{slug}");
        assert_eq!(
            p.parse_name("PROJ-123-fix"),
            Some(f(&[("issue_key", "PROJ-123"), ("slug", "fix")]))
        );
        assert_eq!(p.parse_name("123-fix"), None);
    }

    #[test]
    fn specificity_orders_patterns() {
        assert!(pat("release/{version}").specificity() > pat("{slug}").specificity());
        assert_eq!(pat("{slug}").specificity(), 0);
        assert_eq!(pat("feature/{slug}").prefix(), "feature/");
        assert_eq!(pat("{user}/{slug}").prefix(), "");
    }
}
