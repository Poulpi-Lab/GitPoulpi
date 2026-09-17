//! The rule stages of the metadata rewrite: each rule runs over every active
//! commit one after the other (so a rule that needs every commit, such as
//! anonymising or "smart hours", sees the result of the rules before it).
//! A scope that limits the rules to some commits (a selection) passes only
//! those commits in.

use std::collections::HashMap;

use gitpoulpi_core::domain::rewrite::{
    dates, identity, trailers, CompiledMatch, CompiledPattern, Ident, Mailmap, PathChange,
    RewriteRefusal, RuleKind, TrailerAction, TrailerRule, WhichDate, Who, WindowInput,
};
use gitpoulpi_core::domain::Oid;
use gitpoulpi_core::{Error, Result};

use super::objects::{Commit, Person};

/// One commit while the rules run.
pub(super) struct Work {
    pub(super) commit: Commit,
    pub(super) author0: Option<Person>,
    pub(super) committer0: Option<Person>,
    pub(super) author: Option<Person>,
    pub(super) committer: Option<Person>,
    /// The message after the rules, `None` while it is the original.
    pub(super) message: Option<Vec<u8>>,
    /// The tree after the content rules, `None` while it is the original.
    pub(super) tree: Option<Oid>,
    /// What the content rules did to files (the plan's row; capped).
    pub(super) paths: std::rc::Rc<Vec<PathChange>>,
    /// The message-encoding rule made this commit UTF-8: its `encoding`
    /// header goes.
    pub(super) drop_encoding: bool,
}

impl Work {
    pub(super) fn new(commit: Commit) -> Self {
        let (author, committer) = (commit.author(), commit.committer());
        Self {
            commit,
            author0: author.clone(),
            committer0: committer.clone(),
            author,
            committer,
            message: None,
            tree: None,
            paths: std::rc::Rc::default(),
            drop_encoding: false,
        }
    }

    pub(super) fn message(&self) -> &[u8] {
        self.message
            .as_deref()
            .unwrap_or_else(|| self.commit.message())
    }
}

fn invalid(detail: impl Into<String>) -> Error {
    Error::RewriteRefused(RewriteRefusal::InvalidRule, detail.into())
}

/// Which people a date rule moves.
fn who_of(which: WhichDate) -> Who {
    match which {
        WhichDate::AuthorDate => Who::Author,
        WhichDate::CommitterDate => Who::Committer,
        WhichDate::Both => Who::Both,
    }
}

/// Runs `f` on the author and / or committer of every commit; the number of
/// commits where it returned `true` at least once.
fn people(works: &mut [&mut Work], who: Who, mut f: impl FnMut(&mut Person) -> bool) -> u64 {
    let mut commits = 0;
    for w in works {
        let mut hit = false;
        if who.author() {
            if let Some(p) = w.author.as_mut() {
                hit |= f(p);
            }
        }
        if who.committer() {
            if let Some(p) = w.committer.as_mut() {
                hit |= f(p);
            }
        }
        commits += u64::from(hit);
    }
    commits
}

/// The `.mailmap` a `Mailmap` rule applies: `file` (relative to the
/// worktree, or absolute), else the worktree's own.
pub(super) fn load_mailmap(root: &std::path::Path, file: Option<&str>) -> Result<Mailmap> {
    let path = root.join(file.unwrap_or(".mailmap"));
    match std::fs::read_to_string(&path) {
        Ok(text) => Ok(Mailmap::parse(&text)),
        Err(e) => Err(invalid(format!("cannot read {}: {e}", path.display()))),
    }
}

/// Every author / committer through `map`; the number of commits changed.
pub(super) fn apply_mailmap(works: &mut [&mut Work], map: &Mailmap, who: Who) -> u64 {
    people(works, who, |p| {
        let old = ident_of(p);
        let new = map.resolve(&old);
        if new == old {
            return false;
        }
        p.name = new.name;
        p.email = new.email;
        true
    })
}

fn ident_of(p: &Person) -> Ident {
    Ident::new(&p.name, &p.email)
}

/// Applies one rule to every commit; the number of commits it changed.
///
/// `taggers` are the identities of the annotated tags that may be re-created:
/// the rules that ask for it (`include_taggers`) change them too, in the same
/// order as the commits' identities, but they are not counted.
pub(super) fn apply_rule(
    works: &mut [&mut Work],
    order: &[Oid],
    kind: &RuleKind,
    taggers: &mut [Person],
) -> Result<u64> {
    Ok(match kind {
        RuleKind::Identity(rule) => {
            let when = CompiledMatch::new(&rule.when).map_err(invalid)?;
            let mut change = |p: &mut Person| {
                let old = ident_of(p);
                if !when.matches(&old) {
                    return false;
                }
                match identity::apply(&rule.set, &old) {
                    Ok(new) if new != old => {
                        p.name = new.name;
                        p.email = new.email;
                        true
                    }
                    _ => false,
                }
            };
            if rule.include_taggers {
                taggers.iter_mut().for_each(|t| {
                    change(t);
                });
            }
            if rule.map_trailers {
                // The trailers whose value is a matching `Name <email>`.
                let mut commits = 0;
                for w in works.iter_mut() {
                    let mut one: [&mut Work; 1] = [&mut **w];
                    let mut hit = people(&mut one, rule.apply_to, &mut change) > 0;
                    let [w] = one;
                    hit |= edit_messages(&mut [w], |text| {
                        trailers::map_identities(text, |old| {
                            when.matches(old)
                                .then(|| identity::apply(&rule.set, old).ok())
                                .flatten()
                        })
                    }) > 0;
                    commits += u64::from(hit);
                }
                commits
            } else {
                people(works, rule.apply_to, &mut change)
            }
        }
        RuleKind::Anonymise(rule) => {
            let mut seen = Vec::new();
            for w in works.iter() {
                if rule.apply_to.author() {
                    seen.extend(w.author.iter().map(ident_of));
                }
                if rule.apply_to.committer() {
                    seen.extend(w.committer.iter().map(ident_of));
                }
            }
            // Taggers are numbered after every commit identity, so adding
            // them never renumbers anybody.
            if rule.include_taggers {
                seen.extend(taggers.iter().map(ident_of));
            }
            let map = identity::mapping(&seen, rule).map_err(invalid)?;
            let mut change = |p: &mut Person| {
                let old = ident_of(p);
                match map.get(&old) {
                    Some(new) if *new != old => {
                        p.name = new.name.clone();
                        p.email = new.email.clone();
                        true
                    }
                    _ => false,
                }
            };
            if rule.include_taggers {
                taggers.iter_mut().for_each(|t| {
                    change(t);
                });
            }
            people(works, rule.apply_to, &mut change)
        }
        RuleKind::DateShift {
            by_seconds,
            apply_to,
        } => people(works, who_of(*apply_to), |p| {
            let new = dates::shift(p.date(), *by_seconds);
            let changed = new != p.date();
            p.set_date(new);
            changed
        }),
        RuleKind::DateTimezone {
            offset_minutes,
            keep,
            apply_to,
        } => people(works, who_of(*apply_to), |p| {
            let new = dates::set_offset(p.date(), *offset_minutes, *keep);
            let changed = new != p.date();
            p.set_date(new);
            changed
        }),
        RuleKind::DateWindow(rule) => window_stage(works, order, rule),
        RuleKind::CommitterFromAuthor => {
            let mut commits = 0;
            for w in works.iter_mut() {
                if let (Some(author), Some(committer)) = (&w.author, w.committer.as_mut()) {
                    if committer != author {
                        *committer = author.clone();
                        commits += 1;
                    }
                }
            }
            commits
        }
        RuleKind::MessageReplace(rule) => {
            let pattern = CompiledPattern::new(&rule.find).map_err(invalid)?;
            edit_messages(works, |text| pattern.replace_all(text, &rule.with))
        }
        RuleKind::MessageDeleteLines { pattern } => {
            let pattern = CompiledPattern::new(pattern).map_err(invalid)?;
            edit_messages(works, |text| pattern.delete_lines(text))
        }
        RuleKind::Trailer(rule) => trailer_stage(works, rule)?,
        _ => unreachable!("check_supported refused every other rule"),
    })
}

/// Messages that are valid UTF-8 go through `f`; the others are left alone
/// byte for byte.
fn edit_messages(works: &mut [&mut Work], f: impl Fn(&str) -> String) -> u64 {
    let mut commits = 0;
    for w in works {
        let Ok(text) = std::str::from_utf8(w.message()) else {
            continue;
        };
        let new = f(text);
        if new != text {
            w.message = Some(new.into_bytes());
            commits += 1;
        }
    }
    commits
}

/// The message edits of a `Trailer` rule; the number of commits changed.
fn trailer_stage(works: &mut [&mut Work], rule: &TrailerRule) -> Result<u64> {
    let find = match &rule.find {
        Some(p) => Some(CompiledPattern::new(p).map_err(invalid)?),
        None => None,
    };
    Ok(match rule.action {
        TrailerAction::Add => edit_messages(works, |text| {
            trailers::add(text, &rule.key, &rule.value, rule.if_exists)
        }),
        TrailerAction::Remove => edit_messages(works, |text| {
            trailers::remove(text, &rule.key, find.as_ref())
        }),
        TrailerAction::Map => {
            let find = find.ok_or_else(|| invalid("a trailer to map needs the text to replace"))?;
            edit_messages(works, |text| {
                trailers::map_text(text, &rule.key, &find, &rule.with)
            })
        }
    })
}

/// "Smart hours" needs every commit's parents as indices.
fn window_stage(works: &mut [&mut Work], order: &[Oid], rule: &dates::WorkWindowRule) -> u64 {
    let index: HashMap<Oid, usize> = order.iter().enumerate().map(|(i, o)| (*o, i)).collect();
    let zero = dates::GitDate::new(0, 0);
    let inputs: Vec<WindowInput> = works
        .iter()
        .map(|w| WindowInput {
            parents: w
                .commit
                .parents()
                .unwrap_or_default()
                .iter()
                .filter_map(|p| index.get(p).copied())
                .collect(),
            author: w.author.as_ref().map_or(zero, Person::date),
            committer: w.committer.as_ref().map_or(zero, Person::date),
        })
        .collect();
    let times = dates::window(&inputs, rule);
    let mut commits = 0;
    for (w, t) in works.iter_mut().zip(times) {
        let mut hit = false;
        if let Some(p) = w.author.as_mut() {
            if p.date() != t.author {
                p.set_date(t.author);
                hit = true;
            }
        }
        if let Some(p) = w.committer.as_mut() {
            if p.date() != t.committer {
                p.set_date(t.committer);
                hit = true;
            }
        }
        commits += u64::from(hit);
    }
    commits
}
