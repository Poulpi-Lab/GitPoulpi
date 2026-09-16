//! `.mailmap` (slice S16): the file git reads to show one canonical name and
//! email for a person who committed under several. Pure: the parser and the
//! lookup (used by the `Mailmap` rewrite rule, whose result is checked
//! against `git log --use-mailmap`), the lines "Merge into..." writes, and
//! the types of the Identities list.
//!
//! The lookup follows git's `mailmap.c`: an entry is keyed by the old email
//! (case-insensitive) and optionally the old name (case-insensitive); an
//! entry that names the old name wins over one that only names the email,
//! and of two equal keys the later line wins.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::identity::{is_storable, Ident};

/// One parsed line: what an identity matching `old_*` becomes.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Entry {
    old_name: Option<String>,
    old_email: String,
    new_name: Option<String>,
    new_email: Option<String>,
}

/// A parsed `.mailmap`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Mailmap {
    entries: Vec<Entry>,
    /// Positions in `entries` by lower-cased old email: a lookup (one per
    /// commit in a 100 000-commit Log) does not scan a long file.
    by_email: HashMap<String, Vec<usize>>,
}

/// `name <email>` at the start of `text`: the name (trimmed, may be empty),
/// the email, and what follows the closing `>`.
fn pair(text: &str) -> Option<(&str, &str, &str)> {
    let open = text.find('<')?;
    let close = open + text[open..].find('>')?;
    Some((
        text[..open].trim(),
        text[open + 1..close].trim(),
        &text[close + 1..],
    ))
}

impl Mailmap {
    /// Parses the text of a `.mailmap`. Lines git would not understand are
    /// skipped, as git does.
    pub fn parse(text: &str) -> Self {
        let mut entries = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((name1, email1, rest)) = pair(line) else {
                continue;
            };
            match pair(rest) {
                Some((name2, email2, _)) => entries.push(Entry {
                    old_name: (!name2.is_empty()).then(|| name2.to_string()),
                    old_email: email2.to_string(),
                    new_name: (!name1.is_empty()).then(|| name1.to_string()),
                    new_email: Some(email1.to_string()),
                }),
                None => {
                    // `Proper Name <commit@email>`: no name, no mapping.
                    if name1.is_empty() {
                        continue;
                    }
                    entries.push(Entry {
                        old_name: None,
                        old_email: email1.to_string(),
                        new_name: Some(name1.to_string()),
                        new_email: None,
                    });
                }
            }
        }
        let mut map = Self {
            entries,
            by_email: HashMap::new(),
        };
        map.reindex();
        map
    }

    fn reindex(&mut self) {
        self.by_email.clear();
        for (i, e) in self.entries.iter().enumerate() {
            self.by_email
                .entry(e.old_email.to_lowercase())
                .or_default()
                .push(i);
        }
    }

    /// `other`'s lines after this one's: where both name the same identity
    /// the later line wins, which is how git layers `mailmap.file` over
    /// the worktree's `.mailmap`.
    pub fn extend(&mut self, other: Mailmap) {
        self.entries.extend(other.entries);
        self.reindex();
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn find(&self, ident: &Ident) -> Option<&Entry> {
        let mut simple = None;
        let mut exact = None;
        let at = self.by_email.get(&ident.email.to_lowercase())?;
        for e in at.iter().map(|i| &self.entries[*i]) {
            match &e.old_name {
                Some(n) if n.to_lowercase() == ident.name.to_lowercase() => exact = Some(e),
                Some(_) => {}
                None => simple = Some(e),
            }
        }
        exact.or(simple)
    }

    /// The canonical identity for `ident` (the same one when nothing maps it).
    pub fn resolve(&self, ident: &Ident) -> Ident {
        match self.find(ident) {
            Some(e) => Ident {
                name: e.new_name.clone().unwrap_or_else(|| ident.name.clone()),
                email: e.new_email.clone().unwrap_or_else(|| ident.email.clone()),
            },
            None => ident.clone(),
        }
    }

    /// `true` when some line maps `ident` to something else.
    pub fn maps(&self, ident: &Ident) -> bool {
        self.resolve(ident) != *ident
    }
}

/// The line that maps `old` to `canonical`, in git's four-part form so it
/// matches that name and email only.
pub fn line_for(canonical: &Ident, old: &Ident) -> String {
    if old.name.is_empty() {
        format!("{} <{}> <{}>", canonical.name, canonical.email, old.email)
    } else {
        format!(
            "{} <{}> {} <{}>",
            canonical.name, canonical.email, old.name, old.email
        )
    }
}

/// Why `canonical` or an `old` identity cannot be written to a `.mailmap`.
pub fn problem(canonical: &Ident, from: &[Ident]) -> Option<&'static str> {
    let ok = |i: &Ident| is_storable(&i.name) && is_storable(&i.email);
    if canonical.name.trim().is_empty() || canonical.email.trim().is_empty() {
        return Some("the canonical identity needs a name and an email");
    }
    if !ok(canonical) || !from.iter().all(ok) {
        return Some("a name or email cannot contain `<`, `>` or a line break");
    }
    if from.iter().any(|i| i.email.trim().is_empty()) {
        return Some("an identity to merge has no email");
    }
    None
}

/// `existing` with the lines mapping each of `from` to `canonical` appended
/// (once each: a line already in the file, and the canonical identity
/// itself, are skipped). The number of lines added comes back with the text.
pub fn merged(existing: &str, canonical: &Ident, from: &[Ident]) -> (String, usize) {
    let mut text = existing.to_string();
    let mut added = 0;
    for old in from {
        if old == canonical {
            continue;
        }
        let line = line_for(canonical, old);
        if existing.lines().any(|l| l.trim() == line) {
            continue;
        }
        if !text.is_empty() && !text.ends_with('\n') {
            text.push('\n');
        }
        text.push_str(&line);
        text.push('\n');
        added += 1;
    }
    (text, added)
}

/// One name and email seen in the history, with how often.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityCount {
    pub name: String,
    pub email: String,
    /// Commits of the scope this identity authored.
    pub as_author: u64,
    /// Commits of the scope this identity committed.
    pub as_committer: u64,
    /// What the `.mailmap` already maps it to, when it does.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mapped_to: Option<Ident>,
}

impl IdentityCount {
    pub fn ident(&self) -> Ident {
        Ident::new(&self.name, &self.email)
    }
}

/// Every identity of a scope, most commits first, and the state of the
/// repository's `.mailmap`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityList {
    pub identities: Vec<IdentityCount>,
    /// Commits the counts were read from.
    pub commits: u64,
    /// The worktree has a `.mailmap` file.
    pub has_mailmap: bool,
}

/// What "Merge into..." wrote.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MailmapAdded {
    /// Lines appended (0: all were already there).
    pub added: usize,
    /// The file written, relative to the repository root.
    pub file: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: &str, e: &str) -> Ident {
        Ident::new(n, e)
    }

    #[test]
    fn the_four_forms() {
        let m = Mailmap::parse(
            "# comment\n\
             Proper One <old1@x.org>\n\
             <proper2@x.org> <old2@x.org>\n\
             Proper Three <proper3@x.org> <old3@x.org>\n\
             Proper Four <proper4@x.org> Old Four <old4@x.org>\n",
        );
        assert_eq!(
            m.resolve(&id("Anyone", "OLD1@x.org")),
            id("Proper One", "OLD1@x.org")
        );
        assert_eq!(
            m.resolve(&id("Anyone", "old2@x.org")),
            id("Anyone", "proper2@x.org")
        );
        assert_eq!(
            m.resolve(&id("Anyone", "old3@x.org")),
            id("Proper Three", "proper3@x.org")
        );
        assert_eq!(
            m.resolve(&id("old four", "old4@x.org")),
            id("Proper Four", "proper4@x.org")
        );
        // The name has to match for the four-part form.
        assert_eq!(
            m.resolve(&id("Someone Else", "old4@x.org")),
            id("Someone Else", "old4@x.org")
        );
    }

    #[test]
    fn a_named_entry_beats_an_email_only_one_and_the_later_line_wins() {
        let m = Mailmap::parse(
            "General <g@x.org> <o@x.org>\n\
             Special <s@x.org> Sam <o@x.org>\n\
             Later <l@x.org> <o@x.org>\n",
        );
        assert_eq!(m.resolve(&id("Sam", "o@x.org")), id("Special", "s@x.org"));
        assert_eq!(m.resolve(&id("Pat", "o@x.org")), id("Later", "l@x.org"));
    }

    #[test]
    fn merged_appends_each_line_once() {
        let canon = id("Alex Doe", "alex@x.org");
        let from = [id("alex", "alex@work.com"), id("Alex Doe", "alex@x.org")];
        let (text, n) = merged("# people\nOther <o@x.org> <p@x.org>", &canon, &from);
        assert_eq!(n, 1);
        assert_eq!(
            text,
            "# people\nOther <o@x.org> <p@x.org>\nAlex Doe <alex@x.org> alex <alex@work.com>\n"
        );
        let (again, n) = merged(&text, &canon, &from);
        assert_eq!((again, n), (text, 0));
        // What was written maps what it says.
        assert!(Mailmap::parse(&merged("", &canon, &from).0).maps(&from[0]));
    }

    #[test]
    fn problems() {
        let ok = id("A", "a@x.org");
        assert!(problem(&ok, &[id("B", "b@x.org")]).is_none());
        assert!(problem(&id("", "a@x.org"), &[]).is_some());
        assert!(problem(&ok, &[id("B<", "b@x.org")]).is_some());
        assert!(problem(&ok, &[id("B", "")]).is_some());
    }
}
