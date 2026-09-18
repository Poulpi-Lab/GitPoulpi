//! The Start form's pure part: the fields a topic type's pattern asks for,
//! what each one suggests, and the branch name they render to.

use std::collections::BTreeMap;

use gitpoulpi_core::workflow::{
    check_ref_format, slugify, BranchPattern, Fields, Token, DEFAULT_SLUG_MAX,
};

/// What the form can prefill besides the issue and the slug.
#[derive(Debug, Clone, Default)]
pub(crate) struct Suggestions {
    /// `{user}`: the active profile's name, slugified by the pattern.
    pub(crate) user: Option<String>,
    /// `{date}`: today, `YYYY-MM-DD`.
    pub(crate) today: String,
    /// `{version}`: the next version after the repository's tags.
    pub(crate) version: Option<String>,
}

/// One input of the form, in pattern order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FormField {
    pub(crate) token: Token,
    pub(crate) value: String,
}

/// The form of one topic type.
#[derive(Debug, Clone)]
pub(crate) struct StartForm {
    pattern: Option<BranchPattern>,
    pub(crate) fields: Vec<FormField>,
    /// `{slug}` still holds what an issue's title put there (a pick may
    /// replace it; a typed slug is never replaced).
    slug_from_issue: bool,
}

impl StartForm {
    /// The form for `pattern` (the type's rendering pattern), prefilled.
    pub(crate) fn new(pattern: &str, suggest: &Suggestions) -> Self {
        let parsed = BranchPattern::parse(pattern).ok();
        let fields = parsed
            .as_ref()
            .map(|p| p.tokens())
            .unwrap_or_default()
            .into_iter()
            .map(|token| FormField {
                token,
                value: match token {
                    Token::User => suggest.user.clone().unwrap_or_default(),
                    Token::Date => suggest.today.clone(),
                    Token::Version => suggest.version.clone().unwrap_or_default(),
                    _ => String::new(),
                },
            })
            .collect();
        Self {
            pattern: parsed,
            fields,
            slug_from_issue: false,
        }
    }

    pub(crate) fn has(&self, token: Token) -> bool {
        self.fields.iter().any(|f| f.token == token)
    }

    pub(crate) fn value(&self, token: Token) -> &str {
        self.fields
            .iter()
            .find(|f| f.token == token)
            .map_or("", |f| f.value.as_str())
    }

    /// Typing into field `index`.
    pub(crate) fn edited(&mut self, index: usize) {
        if self
            .fields
            .get(index)
            .is_some_and(|f| f.token == Token::Slug)
        {
            self.slug_from_issue = false;
        }
    }

    /// An issue was picked: `reference` is its reference (`#12`, `ABC-7`),
    /// `title` its title. Fills `{issue}` / `{issue_key}` and, unless the
    /// user typed one, `{slug}` from the title.
    pub(crate) fn pick_issue(&mut self, reference: &str, title: &str) {
        let digits: String = reference
            .trim_start_matches('#')
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        for f in &mut self.fields {
            match f.token {
                Token::Issue if !digits.is_empty() => f.value = digits.clone(),
                Token::IssueKey if reference.contains('-') => f.value = reference.to_string(),
                _ => {}
            }
        }
        let slug = slugify(title, DEFAULT_SLUG_MAX);
        let replace = self.slug_from_issue || self.value(Token::Slug).is_empty();
        if replace && !slug.is_empty() {
            if let Some(f) = self.fields.iter_mut().find(|f| f.token == Token::Slug) {
                f.value = slug;
                self.slug_from_issue = true;
            }
        }
    }

    /// The token values, by name.
    pub(crate) fn values(&self) -> Fields {
        self.fields
            .iter()
            .filter(|f| !f.value.trim().is_empty())
            .map(|f| (f.token.name().to_string(), f.value.trim().to_string()))
            .collect::<BTreeMap<_, _>>()
    }

    /// The branch name the fields render to, or why they do not.
    /// `exists` says whether a branch of that name is already there.
    pub(crate) fn name(&self, exists: impl Fn(&str) -> bool) -> Result<String, String> {
        let pattern = self
            .pattern
            .as_ref()
            .ok_or_else(|| gitpoulpi_i18n::msg::flow::BAD_PATTERN.get().to_string())?;
        let name = pattern.render(&self.values()).map_err(|e| e.to_string())?;
        check_ref_format(&name).map_err(|e| e.to_string())?;
        if exists(&name) {
            return Err(gitpoulpi_i18n::msg::flow::name_taken(&name));
        }
        Ok(name)
    }
}

/// How the next version is derived from the highest tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Bump {
    Minor,
    Patch,
}

fn parse_version(tag: &str) -> Option<Vec<u64>> {
    let core = tag.strip_prefix(['v', 'V']).unwrap_or(tag);
    // Pre-releases and builds are not releases: skip them.
    if core.contains(['-', '+']) {
        return None;
    }
    let parts: Vec<u64> = core
        .split('.')
        .map(|p| p.parse::<u64>().ok())
        .collect::<Option<_>>()?;
    (!parts.is_empty() && parts.len() <= 3).then_some(parts)
}

/// The version after the highest release tag (`v1.4.2` gives `1.4.3` for a
/// patch bump, `1.5.0` for a minor one); `None` when no tag is a version.
pub(crate) fn next_version(tags: &[String], bump: Bump) -> Option<String> {
    let mut best = tags
        .iter()
        .filter_map(|t| parse_version(t))
        .map(|mut v| {
            v.resize(3, 0);
            v
        })
        .max()?;
    match bump {
        Bump::Patch => best[2] += 1,
        Bump::Minor => {
            best[1] += 1;
            best[2] = 0;
        }
    }
    Some(format!("{}.{}.{}", best[0], best[1], best[2]))
}

/// Today (UTC) as `YYYY-MM-DD`.
pub(crate) fn today_iso() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    iso_date((secs / 86_400) as i64)
}

/// The civil date of `days` since 1970-01-01 (Howard Hinnant's algorithm).
fn iso_date(days: i64) -> String {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { year + 1 } else { year };
    format!("{year:04}-{month:02}-{day:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn suggest() -> Suggestions {
        Suggestions {
            user: Some("Ada Lovelace".into()),
            today: "2026-10-16".into(),
            version: Some("1.3.0".into()),
        }
    }

    #[test]
    fn fields_follow_the_pattern_and_prefill_what_they_can() {
        let form = StartForm::new("{user}/{issue}-{slug}", &suggest());
        let tokens: Vec<Token> = form.fields.iter().map(|f| f.token).collect();
        assert_eq!(tokens, [Token::User, Token::Issue, Token::Slug]);
        assert_eq!(form.value(Token::User), "Ada Lovelace");
        assert_eq!(form.value(Token::Issue), "");
        let release = StartForm::new("release/{version}", &suggest());
        assert_eq!(release.value(Token::Version), "1.3.0");
        let dated = StartForm::new("daily/{date}", &suggest());
        assert_eq!(dated.value(Token::Date), "2026-10-16");
    }

    #[test]
    fn the_name_is_the_rendered_pattern_and_says_why_not() {
        let mut form = StartForm::new("feature/{slug}", &suggest());
        assert!(form.name(|_| false).is_err(), "an empty slug has no name");
        form.fields[0].value = "Add a login page".into();
        assert_eq!(form.name(|_| false).unwrap(), "feature/add-a-login-page");
        assert!(form.name(|n| n == "feature/add-a-login-page").is_err());
    }

    #[test]
    fn a_picked_issue_fills_the_number_and_the_slug_once() {
        let mut form = StartForm::new("feature/{issue}-{slug}", &suggest());
        form.pick_issue("#42", "Fix the Login Bug");
        assert_eq!(form.value(Token::Issue), "42");
        assert_eq!(form.value(Token::Slug), "fix-the-login-bug");
        assert_eq!(
            form.name(|_| false).unwrap(),
            "feature/42-fix-the-login-bug"
        );
        // A later pick replaces the slug it filled itself...
        form.pick_issue("#43", "Other thing");
        assert_eq!(form.value(Token::Slug), "other-thing");
        // ...but never one the user typed.
        form.fields[1].value = "mine".into();
        form.edited(1);
        form.pick_issue("#44", "Third");
        assert_eq!(form.value(Token::Slug), "mine");
        assert_eq!(form.value(Token::Issue), "44");
    }

    #[test]
    fn a_tracker_key_fills_the_key_field() {
        let mut form = StartForm::new("{issue_key}-{slug}", &suggest());
        form.pick_issue("ABC-7", "Login");
        assert_eq!(form.value(Token::IssueKey), "ABC-7");
        assert_eq!(form.name(|_| false).unwrap(), "ABC-7-login");
    }

    #[test]
    fn the_next_version_follows_the_highest_release_tag() {
        let tags: Vec<String> = ["v1.4.2", "v1.10.0", "v1.9.9", "v2.0.0-rc1", "nightly"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(next_version(&tags, Bump::Patch).as_deref(), Some("1.10.1"));
        assert_eq!(next_version(&tags, Bump::Minor).as_deref(), Some("1.11.0"));
        assert_eq!(next_version(&["nightly".to_string()], Bump::Minor), None);
        assert_eq!(
            next_version(&["1.2".to_string()], Bump::Patch).as_deref(),
            Some("1.2.1")
        );
    }

    #[test]
    fn dates_are_civil() {
        assert_eq!(iso_date(0), "1970-01-01");
        assert_eq!(iso_date(20_742), "2026-10-16");
        assert_eq!(iso_date(11_016), "2000-02-29");
    }
}
