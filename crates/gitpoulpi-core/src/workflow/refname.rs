//! Branch-name hygiene: `git check-ref-format` rules and slugs.

/// Why a name is not a valid branch name.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RefFormatError {
    #[error("the name is empty")]
    Empty,
    #[error("`@` alone is not a valid branch name")]
    At,
    #[error("a branch name cannot start with `-`")]
    LeadingDash,
    #[error("a branch name cannot start or end with `/`, or contain `//`")]
    Slash,
    #[error("a branch name cannot end with `.`")]
    TrailingDot,
    #[error("a branch name cannot contain `..` or `@{{`")]
    ForbiddenSequence,
    #[error("a component cannot start with `.` or end with `.lock`")]
    BadComponent,
    #[error("`{0}` is not allowed in a branch name")]
    ForbiddenChar(char),
}

/// `git check-ref-format --branch`, reimplemented so the pattern engine stays
/// pure (no process): the rules are those of `git-check-ref-format(1)`.
pub fn check_ref_format(name: &str) -> Result<(), RefFormatError> {
    if name.is_empty() {
        return Err(RefFormatError::Empty);
    }
    if name == "@" {
        return Err(RefFormatError::At);
    }
    if name.starts_with('-') {
        return Err(RefFormatError::LeadingDash);
    }
    if name.starts_with('/') || name.ends_with('/') || name.contains("//") {
        return Err(RefFormatError::Slash);
    }
    if name.ends_with('.') {
        return Err(RefFormatError::TrailingDot);
    }
    if name.contains("..") || name.contains("@{") {
        return Err(RefFormatError::ForbiddenSequence);
    }
    if let Some(c) = name
        .chars()
        .find(|c| c.is_control() || matches!(c, ' ' | '~' | '^' | ':' | '?' | '*' | '[' | '\\'))
    {
        return Err(RefFormatError::ForbiddenChar(c));
    }
    if name
        .split('/')
        .any(|part| part.starts_with('.') || part.ends_with(".lock"))
    {
        return Err(RefFormatError::BadComponent);
    }
    Ok(())
}

/// The longest slug [`slugify`] produces by default.
pub const DEFAULT_SLUG_MAX: usize = 50;

/// Folds the common Latin letters to ASCII (`é` -> `e`, `ß` -> `ss`).
/// Anything not in the table is `None` (it becomes a separator).
fn fold(c: char) -> Option<&'static str> {
    Some(match c {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' | 'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å'
        | 'Ā' | 'Ă' | 'Ą' => "a",
        'æ' | 'Æ' => "ae",
        'ç' | 'ć' | 'č' | 'Ç' | 'Ć' | 'Č' => "c",
        'ď' | 'đ' | 'ð' | 'Ď' | 'Đ' | 'Ð' => "d",
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ě' | 'ę' | 'È' | 'É' | 'Ê' | 'Ë' | 'Ē' | 'Ě' | 'Ę' => {
            "e"
        }
        'ğ' | 'Ğ' => "g",
        'ì' | 'í' | 'î' | 'ï' | 'ī' | 'ı' | 'Ì' | 'Í' | 'Î' | 'Ï' | 'Ī' | 'İ' => "i",
        'ł' | 'ĺ' | 'ľ' | 'Ł' | 'Ĺ' | 'Ľ' => "l",
        'ñ' | 'ń' | 'ň' | 'Ñ' | 'Ń' | 'Ň' => "n",
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' | 'ő' | 'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'Ø' | 'Ō'
        | 'Ő' => "o",
        'œ' | 'Œ' => "oe",
        'ř' | 'ŕ' | 'Ř' | 'Ŕ' => "r",
        'ś' | 'š' | 'ş' | 'Ś' | 'Š' | 'Ş' => "s",
        'ß' => "ss",
        'ť' | 'ţ' | 'Ť' | 'Ţ' => "t",
        'þ' | 'Þ' => "th",
        'ù' | 'ú' | 'û' | 'ü' | 'ū' | 'ů' | 'ű' | 'ų' | 'Ù' | 'Ú' | 'Û' | 'Ü' | 'Ū' | 'Ů' | 'Ű'
        | 'Ų' => "u",
        'ý' | 'ÿ' | 'Ý' | 'Ÿ' => "y",
        'ź' | 'ż' | 'ž' | 'Ź' | 'Ż' | 'Ž' => "z",
        _ => return None,
    })
}

/// A branch-name-safe slug: lowercase ASCII letters and digits joined by single
/// `-`, accents folded, at most `max` characters, no leading or trailing `-`.
/// Idempotent. Empty when the input has no letter or digit at all.
pub fn slugify(input: &str, max: usize) -> String {
    let mut out = String::new();
    let mut pending_dash = false;
    for c in input.chars() {
        let folded: Option<String> = if c.is_ascii_alphanumeric() {
            Some(c.to_ascii_lowercase().to_string())
        } else {
            fold(c).map(str::to_string)
        };
        match folded {
            Some(s) => {
                if pending_dash && !out.is_empty() {
                    out.push('-');
                }
                pending_dash = false;
                out.push_str(&s);
            }
            None => pending_dash = true,
        }
    }
    // Output is ASCII, so byte truncation is on a character boundary.
    out.truncate(max.min(out.len()));
    while out.ends_with('-') {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_names() {
        for n in [
            "main",
            "feature/login",
            "release/1.2.0",
            "a/b/c",
            "x-y_z",
            "feat/é",
            "v1.0",
        ] {
            assert_eq!(check_ref_format(n), Ok(()), "{n}");
        }
    }

    #[test]
    fn invalid_names_follow_git() {
        let bad = [
            "",
            "@",
            "-x",
            "/x",
            "x/",
            "a//b",
            "x.",
            "a..b",
            "a@{b",
            "a b",
            "a~b",
            "a^b",
            "a:b",
            "a?b",
            "a*b",
            "a[b",
            "a\\b",
            "a\tb",
            ".hidden",
            "a/.b",
            "a.lock",
            "a/b.lock/c",
            "a\u{7f}",
        ];
        for n in bad {
            assert!(check_ref_format(n).is_err(), "{n:?} should be invalid");
        }
    }

    #[test]
    fn slugs() {
        assert_eq!(slugify("Fix the Login page!", 50), "fix-the-login-page");
        assert_eq!(slugify("  --Café  crème--  ", 50), "cafe-creme");
        assert_eq!(slugify("Straße ÆØ", 50), "strasse-aeo");
        assert_eq!(slugify("日本語 only", 50), "only");
        assert_eq!(slugify("日本語", 50), "");
        assert_eq!(slugify("", 50), "");
        assert_eq!(slugify("a/b\\c..d", 50), "a-b-c-d");
        assert_eq!(slugify("x\u{0}y", 50), "x-y");
    }

    #[test]
    fn slug_cap_never_leaves_a_trailing_dash_and_is_idempotent() {
        let s = slugify("aaaa bbbb cccc", 5);
        assert_eq!(s, "aaaa");
        assert_eq!(slugify(&s, 5), s);
        let long = "word ".repeat(40);
        let s = slugify(&long, DEFAULT_SLUG_MAX);
        assert!(s.len() <= DEFAULT_SLUG_MAX && !s.ends_with('-'));
        assert_eq!(slugify(&s, DEFAULT_SLUG_MAX), s);
        assert_eq!(slugify("abc", 0), "");
    }

    #[test]
    fn every_slug_is_a_valid_branch_name_when_non_empty() {
        for input in ["--x", "..", ".lock", "a.lock", "@{", "-", "x y z", "é.lock"] {
            let s = slugify(input, 50);
            if !s.is_empty() {
                assert_eq!(check_ref_format(&s), Ok(()), "{input:?} -> {s:?}");
            }
        }
    }
}
