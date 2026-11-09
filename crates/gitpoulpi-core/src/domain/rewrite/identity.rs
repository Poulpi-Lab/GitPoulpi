//! Identity rules (author / committer / tagger name and email) and the
//! anonymisation numbering. Pure functions; the engine feeds them the
//! identities it reads.

use std::collections::HashMap;

use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};

/// A name and an email, as git stores them in an author, committer or tagger
/// line. (Non-UTF-8 identities are not modelled: the engine leaves a line it
/// cannot decode untouched.)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Ident {
    pub name: String,
    pub email: String,
}

impl Ident {
    pub fn new(name: &str, email: &str) -> Self {
        Self {
            name: name.to_string(),
            email: email.to_string(),
        }
    }
}

/// Which identity line(s) a rule touches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Who {
    Author,
    Committer,
    #[default]
    Both,
}

impl Who {
    pub fn author(self) -> bool {
        matches!(self, Who::Author | Who::Both)
    }
    pub fn committer(self) -> bool {
        matches!(self, Who::Committer | Who::Both)
    }
}

/// Which identities a rule applies to. Emails compare case-insensitively and
/// names case-sensitively (the `.mailmap` convention). With `regex`, both
/// fields are regular expressions that must match the whole value. No field
/// set: matches everyone ("replace all authors").
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct IdentityMatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(default)]
    pub regex: bool,
}

impl IdentityMatch {
    pub fn everyone() -> Self {
        Self::default()
    }

    pub fn email(email: &str) -> Self {
        Self {
            email: Some(email.to_string()),
            ..Self::default()
        }
    }

    pub fn name(name: &str) -> Self {
        Self {
            name: Some(name.to_string()),
            ..Self::default()
        }
    }
}

/// An [`IdentityMatch`] compiled once.
#[derive(Debug, Clone)]
pub struct CompiledMatch {
    name: Option<Field>,
    email: Option<Field>,
}

#[derive(Debug, Clone)]
enum Field {
    Exact(String),
    ExactNoCase(String),
    Pattern(Regex),
}

impl Field {
    fn is_match(&self, value: &str) -> bool {
        match self {
            Field::Exact(s) => s == value,
            Field::ExactNoCase(s) => s.to_lowercase() == value.to_lowercase(),
            Field::Pattern(re) => re.is_match(value),
        }
    }
}

impl CompiledMatch {
    pub fn new(m: &IdentityMatch) -> Result<Self, String> {
        let build = |value: &Option<String>, no_case: bool| -> Result<Option<Field>, String> {
            let Some(value) = value else { return Ok(None) };
            if value.is_empty() {
                return Err("an empty name or email to match".to_string());
            }
            if m.regex {
                let re = RegexBuilder::new(&format!("^(?:{value})$"))
                    .case_insensitive(no_case)
                    .size_limit(1 << 20)
                    .build()
                    .map_err(|e| e.to_string())?;
                Ok(Some(Field::Pattern(re)))
            } else if no_case {
                Ok(Some(Field::ExactNoCase(value.clone())))
            } else {
                Ok(Some(Field::Exact(value.clone())))
            }
        };
        Ok(Self {
            name: build(&m.name, false)?,
            email: build(&m.email, true)?,
        })
    }

    pub fn matches(&self, ident: &Ident) -> bool {
        self.name.as_ref().is_none_or(|f| f.is_match(&ident.name))
            && self.email.as_ref().is_none_or(|f| f.is_match(&ident.email))
    }
}

/// `true` when `m` matches `ident` (compiles `m` on every call; the engine
/// uses [`CompiledMatch`]). A match that does not compile matches nothing.
pub fn matches(m: &IdentityMatch, ident: &Ident) -> bool {
    CompiledMatch::new(m).is_ok_and(|c| c.matches(ident))
}

/// What a matched identity becomes. A field left `None` keeps its value.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct IdentitySet {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

/// One identity-replacement rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityRule {
    pub when: IdentityMatch,
    pub set: IdentitySet,
    #[serde(default)]
    pub apply_to: Who,
    /// Also rewrite the tagger of annotated tags.
    #[serde(default)]
    pub include_taggers: bool,
    /// Also map the identity trailers (`Signed-off-by: Name <email>`, any
    /// key whose value is a `Name <email>`) of the messages. Off by default:
    /// a trailer records who signed off, so changing it is a decision.
    #[serde(default)]
    pub map_trailers: bool,
}

/// `false` for a name or email git cannot store in an identity line: `<`,
/// `>` or a line break.
pub fn is_storable(value: &str) -> bool {
    !value.contains(['<', '>', '\n', '\r', '\0'])
}

/// Why `set` cannot be applied, independent of any commit.
pub fn set_problem(set: &IdentitySet) -> Option<&'static str> {
    if set.name.is_none() && set.email.is_none() {
        return Some("the rule sets neither a name nor an email");
    }
    if let Some(name) = &set.name {
        if name.trim().is_empty() {
            return Some("the new name is empty");
        }
        if !is_storable(name) {
            return Some("a name cannot contain `<`, `>` or a line break");
        }
    }
    if let Some(email) = &set.email {
        if email.trim().is_empty() {
            return Some("the new email is empty");
        }
        if !is_storable(email) {
            return Some("an email cannot contain `<`, `>` or a line break");
        }
    }
    None
}

/// `ident` with `set` applied. Refuses a result that git could not store
/// (see [`set_problem`]).
pub fn apply(set: &IdentitySet, ident: &Ident) -> Result<Ident, &'static str> {
    if let Some(problem) = set_problem(set) {
        return Err(problem);
    }
    Ok(Ident {
        name: set.name.clone().unwrap_or_else(|| ident.name.clone()),
        email: set.email.clone().unwrap_or_else(|| ident.email.clone()),
    })
}

/// Replace identities by numbered placeholders.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnonymiseRule {
    /// Identities left as they are (a maintainer who opted out, a bot).
    #[serde(default)]
    pub keep: Vec<IdentityMatch>,
    /// `{n}` is the contributor number.
    #[serde(default = "default_name_pattern")]
    pub name_pattern: String,
    #[serde(default = "default_email_pattern")]
    pub email_pattern: String,
    #[serde(default)]
    pub apply_to: Who,
    #[serde(default)]
    pub include_taggers: bool,
}

fn default_name_pattern() -> String {
    "Contributor {n}".to_string()
}

fn default_email_pattern() -> String {
    "contributor-{n}@anonymous.invalid".to_string()
}

impl Default for AnonymiseRule {
    fn default() -> Self {
        Self {
            keep: Vec::new(),
            name_pattern: default_name_pattern(),
            email_pattern: default_email_pattern(),
            apply_to: Who::Both,
            include_taggers: false,
        }
    }
}

/// Why `rule` is unusable.
pub fn anonymise_problem(rule: &AnonymiseRule) -> Option<String> {
    if !rule.name_pattern.contains("{n}") || !rule.email_pattern.contains("{n}") {
        return Some("both patterns need `{n}` so contributors stay distinct".to_string());
    }
    let sample = |p: &str| p.replace("{n}", "1");
    if !is_storable(&sample(&rule.name_pattern)) || !is_storable(&sample(&rule.email_pattern)) {
        return Some("a pattern cannot contain `<`, `>` or a line break".to_string());
    }
    for m in &rule.keep {
        if let Err(e) = CompiledMatch::new(m) {
            return Some(e);
        }
    }
    None
}

/// The old identity -> new identity table of an anonymisation run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnonymiseMap {
    pairs: Vec<(Ident, Ident)>,
    by_key: HashMap<String, usize>,
    kept: Vec<Ident>,
}

/// The identity a person is known by: the lower-cased email, or the name
/// when there is no email. Two names with one email are one contributor.
fn key(ident: &Ident) -> String {
    if ident.email.is_empty() {
        format!("name:{}", ident.name)
    } else {
        format!("mail:{}", ident.email.to_lowercase())
    }
}

impl AnonymiseMap {
    /// The replacement of `ident`: itself when kept, `None` when it was not
    /// in the list the map was built from.
    pub fn get(&self, ident: &Ident) -> Option<&Ident> {
        if let Some(kept) = self.kept.iter().find(|k| *k == ident) {
            return Some(kept);
        }
        self.by_key.get(&key(ident)).map(|i| &self.pairs[*i].1)
    }

    /// Every anonymised person in numbering order: the first identity seen
    /// for them and their placeholder.
    pub fn pairs(&self) -> &[(Ident, Ident)] {
        &self.pairs
    }
}

/// Numbers contributors by first appearance in `identities_in_order`
/// (oldest commit first). Kept identities map to themselves and use no
/// number. `Contributor 1 <contributor-1@anonymous.invalid>` is the first.
pub fn mapping(
    identities_in_order: &[Ident],
    rule: &AnonymiseRule,
) -> Result<AnonymiseMap, String> {
    let keeps = rule
        .keep
        .iter()
        .map(CompiledMatch::new)
        .collect::<Result<Vec<_>, _>>()?;
    let mut map = AnonymiseMap {
        pairs: Vec::new(),
        by_key: HashMap::new(),
        kept: Vec::new(),
    };
    for ident in identities_in_order {
        if keeps.iter().any(|k| k.matches(ident)) {
            if !map.kept.contains(ident) {
                map.kept.push(ident.clone());
            }
            continue;
        }
        let k = key(ident);
        if map.by_key.contains_key(&k) {
            continue;
        }
        let n = (map.pairs.len() + 1).to_string();
        let new = Ident {
            name: rule.name_pattern.replace("{n}", &n),
            email: rule.email_pattern.replace("{n}", &n),
        };
        map.by_key.insert(k, map.pairs.len());
        map.pairs.push((ident.clone(), new));
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alice() -> Ident {
        Ident::new("Alice Example", "alice@example.com")
    }

    fn bob() -> Ident {
        Ident::new("Bob Example", "bob@example.com")
    }

    #[test]
    fn matching_by_email_ignores_case_and_by_name_does_not() {
        assert!(matches(
            &IdentityMatch::email("ALICE@Example.com"),
            &alice()
        ));
        assert!(!matches(&IdentityMatch::email("bob@example.com"), &alice()));
        assert!(matches(&IdentityMatch::name("Alice Example"), &alice()));
        assert!(!matches(&IdentityMatch::name("alice example"), &alice()));
        assert!(matches(&IdentityMatch::everyone(), &bob()));
    }

    #[test]
    fn both_fields_must_match_and_regex_must_match_the_whole_value() {
        let both = IdentityMatch {
            name: Some("Alice Example".into()),
            email: Some("other@example.com".into()),
            regex: false,
        };
        assert!(!matches(&both, &alice()));
        let re = IdentityMatch {
            email: Some(r".*@example\.com".into()),
            regex: true,
            ..IdentityMatch::default()
        };
        assert!(matches(&re, &alice()) && matches(&re, &bob()));
        let partial = IdentityMatch {
            email: Some("alice".into()),
            regex: true,
            ..IdentityMatch::default()
        };
        assert!(!matches(&partial, &alice()));
        // A regex that does not compile matches nothing and compiles to an error.
        let broken = IdentityMatch {
            name: Some("(".into()),
            regex: true,
            ..IdentityMatch::default()
        };
        assert!(!matches(&broken, &alice()));
        assert!(CompiledMatch::new(&broken).is_err());
    }

    #[test]
    fn apply_replaces_only_the_fields_that_are_set() {
        let only_email = IdentitySet {
            name: None,
            email: Some("new@example.com".into()),
        };
        assert_eq!(
            apply(&only_email, &alice()).unwrap(),
            Ident::new("Alice Example", "new@example.com")
        );
        let both = IdentitySet {
            name: Some("Carol Example".into()),
            email: Some("carol@example.com".into()),
        };
        assert_eq!(
            apply(&both, &alice()).unwrap(),
            Ident::new("Carol Example", "carol@example.com")
        );
    }

    #[test]
    fn identities_git_cannot_store_are_refused() {
        for (name, email) in [
            (Some("A <b>"), None),
            (Some("A\nB"), None),
            (None, Some("a>b@example.com")),
            (None, Some("")),
            (Some(" "), None),
            (None, None),
        ] {
            let set = IdentitySet {
                name: name.map(String::from),
                email: email.map(String::from),
            };
            assert!(apply(&set, &alice()).is_err(), "{name:?} {email:?}");
        }
    }

    #[test]
    fn anonymise_numbers_contributors_by_first_appearance() {
        let seen = [
            alice(),
            bob(),
            // Same person under another name and a different email case.
            Ident::new("A. Example", "ALICE@example.com"),
            Ident::new("Carol Example", "carol@example.com"),
        ];
        let map = mapping(&seen, &AnonymiseRule::default()).unwrap();
        assert_eq!(map.pairs().len(), 3);
        let first = map.get(&alice()).unwrap();
        assert_eq!(first.name, "Contributor 1");
        assert_eq!(first.email, "contributor-1@anonymous.invalid");
        assert_eq!(map.get(&bob()).unwrap().name, "Contributor 2");
        assert_eq!(map.get(&seen[2]).unwrap(), first);
        assert_eq!(map.get(&seen[3]).unwrap().name, "Contributor 3");
        assert!(map.get(&Ident::new("Zed", "zed@example.com")).is_none());
        // Same input, same table.
        assert_eq!(map, mapping(&seen, &AnonymiseRule::default()).unwrap());
    }

    #[test]
    fn kept_identities_stay_and_use_no_number() {
        let rule = AnonymiseRule {
            keep: vec![IdentityMatch::email("alice@example.com")],
            ..AnonymiseRule::default()
        };
        let map = mapping(&[alice(), bob()], &rule).unwrap();
        assert_eq!(map.get(&alice()).unwrap(), &alice());
        assert_eq!(map.get(&bob()).unwrap().name, "Contributor 1");
        assert_eq!(map.pairs().len(), 1);
    }

    #[test]
    fn anonymise_patterns_must_stay_distinct_and_storable() {
        assert!(anonymise_problem(&AnonymiseRule::default()).is_none());
        let fixed = AnonymiseRule {
            email_pattern: "x@anonymous.invalid".into(),
            ..AnonymiseRule::default()
        };
        assert!(anonymise_problem(&fixed).is_some());
        let bad = AnonymiseRule {
            name_pattern: "<{n}>".into(),
            ..AnonymiseRule::default()
        };
        assert!(anonymise_problem(&bad).is_some());
    }

    #[test]
    fn who_selects_lines() {
        assert!(Who::Both.author() && Who::Both.committer());
        assert!(Who::Author.author() && !Who::Author.committer());
        assert!(!Who::Committer.author() && Who::Committer.committer());
    }
}
