//! "Restore only this": brings back what one journal entry removed, without
//! undoing the entries after it. Pure, like the rest of the journal: it
//! looks at the entry and at the facts the daemon read, and either plans
//! actions or refuses with a typed reason. The daemon runs the actions as a
//! NEW journaled operation (`OperationKind::JournalRestore`), so Ctrl+Z
//! takes the restore back.
//!
//! Safety rules (each has a test):
//! * nothing is overwritten: a file that changed since is written next to
//!   itself as `<name>.restored` (unique), a taken ref name is a refusal
//!   that offers a free one, a configuration section that changed since is
//!   left alone;
//! * the current branch is never moved: a history operation (reset, amend,
//!   rebase, merge, commit...) is restored as a new branch at the old tip;
//! * when nothing can be brought back without guessing the answer is a
//!   typed reason, never a partial guess.

use std::collections::{BTreeMap, BTreeSet};

use super::{short, JournalEntry};
use crate::domain::{
    EntryState, EntrySummary, FileHash, Oid, PathHash, RestoreItem, RestoreItemKind, RestoreOffer,
    UndoRefusal, UNDO_PIN_PREFIX,
};

/// What the daemon read about the repository for a restore.
#[derive(Debug, Clone, Default)]
pub struct RestoreFacts {
    /// Raw ref values: every local branch, tag and remote-tracking ref.
    pub refs: BTreeMap<String, Oid>,
    /// Symbolic refs (`refs/remotes/origin/HEAD` -> its target).
    pub symrefs: BTreeMap<String, String>,
    /// Current key/value pairs of every config section the entry touched.
    pub config: BTreeMap<String, Vec<(String, String)>>,
    /// The commits on the stash list now.
    pub stashes: Vec<Oid>,
    /// Current state of the paths of the entry's [`PathChange`], sorted.
    pub files: Vec<PathHash>,
}

/// One thing a restore does. Each is also what the new journal entry
/// records, so that undoing the restore takes it back exactly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestoreAction {
    /// A ref that does not exist now gets this value (`kind` says why).
    CreateRef {
        name: String,
        oid: Oid,
        kind: RestoreItemKind,
    },
    /// A symbolic ref that does not exist now.
    SetSymref { name: String, target: String },
    /// A config section that is as the entry left it goes back to `to`.
    SetConfig {
        section: String,
        to: Vec<(String, String)>,
        from: Vec<(String, String)>,
    },
    /// The stash commit goes on top of the stash list.
    StashStore { commit: Oid, message: String },
    /// Files that are exactly as the entry left them are written back from
    /// the entry's safety snapshot (index and working tree).
    RestorePaths { paths: Vec<String>, snapshot: Oid },
    /// A file that changed since: the kept bytes are written to `dest`, a
    /// new file next to it. The original is not touched.
    SideWrite {
        path: String,
        dest: String,
        file: FileHash,
    },
    /// Lines the entry removed from a `.gitignore` / `.gitattributes` (or,
    /// when it only added some, those lines) are appended to the file as it
    /// is now, unless it has them already.
    ReAddLines {
        path: String,
        before: Option<FileHash>,
        after: Option<FileHash>,
    },
}

impl RestoreAction {
    pub fn describe(&self) -> String {
        match self {
            RestoreAction::CreateRef { name, oid, .. } => {
                format!("create {} at {}", display_ref(name), short(*oid))
            }
            RestoreAction::SetSymref { name, target } => {
                format!("point {name} at {target}")
            }
            RestoreAction::SetConfig { section, to, .. } => {
                format!("restore the configuration section {section} ({} keys)", to.len())
            }
            RestoreAction::StashStore { commit, message } => {
                format!("put the stash \"{message}\" ({}) on top of the stash list", short(*commit))
            }
            RestoreAction::RestorePaths { paths, .. } => format!(
                "write back {} (they are as the operation left them)",
                super::paths_in_words(paths)
            ),
            RestoreAction::SideWrite { path, dest, .. } => format!(
                "{path} changed since: write the kept version next to it as {dest}, {path} is not touched"
            ),
            RestoreAction::ReAddLines { path, .. } => {
                format!("add the missing lines back to {path}")
            }
        }
    }

    /// The repository-relative paths this action writes.
    pub fn written_paths(&self) -> Vec<String> {
        match self {
            RestoreAction::RestorePaths { paths, .. } => paths.clone(),
            RestoreAction::SideWrite { dest, .. } => vec![dest.clone()],
            RestoreAction::ReAddLines { path, .. } => vec![path.clone()],
            _ => Vec::new(),
        }
    }
}

/// What `plan_restore` decided.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestorePlan {
    pub entry: EntrySummary,
    /// The label of the new journal entry.
    pub label: String,
    pub actions: Vec<RestoreAction>,
}

impl RestorePlan {
    pub fn describe(&self) -> Vec<String> {
        self.actions.iter().map(RestoreAction::describe).collect()
    }
}

fn display_ref(name: &str) -> String {
    name.strip_prefix("refs/heads/")
        .map(str::to_string)
        .or_else(|| name.strip_prefix("refs/tags/").map(|t| format!("tag {t}")))
        .or_else(|| name.strip_prefix("refs/remotes/").map(str::to_string))
        .unwrap_or_else(|| name.to_string())
}

/// `.gitignore` / `.gitattributes` (in any folder): restored line by line,
/// never replaced as a whole.
fn is_rule_file(path: &str) -> bool {
    let base = path.rsplit('/').next().unwrap_or(path);
    base == ".gitignore" || base == ".gitattributes"
}

/// The lines to append to `current` so that the lines `before` had and
/// `after` lost are back (or, when the entry only added lines, so that the
/// lines it added are back). `None`: nothing to add.
pub fn readd_lines(before: &str, after: &str, current: &str) -> Option<String> {
    let lines = |text: &str| -> Vec<String> {
        text.lines()
            .map(|l| l.trim_end().to_string())
            .filter(|l| !l.is_empty())
            .collect()
    };
    let (b, a, c) = (lines(before), lines(after), lines(current));
    let mut wanted: Vec<String> = b.iter().filter(|l| !a.contains(l)).cloned().collect();
    if wanted.is_empty() {
        wanted = a.iter().filter(|l| !b.contains(l)).cloned().collect();
    }
    let mut missing: Vec<String> = Vec::new();
    for line in wanted {
        if !c.contains(&line) && !missing.contains(&line) {
            missing.push(line);
        }
    }
    if missing.is_empty() {
        return None;
    }
    let mut out = current.to_string();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    for line in missing {
        out.push_str(&line);
        out.push('\n');
    }
    Some(out)
}

/// A ref name component git accepts (the common cases; git has the last
/// word when the ref is created).
pub fn valid_ref_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('-')
        && !name.starts_with('/')
        && !name.ends_with('/')
        && !name.ends_with('.')
        && !name.ends_with(".lock")
        && !name.contains("..")
        && !name.contains("//")
        && !name.contains("@{")
        && !name
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || "~^:?*[\\".contains(c))
}

/// What one entry could bring back, before looking at the repository.
#[derive(Debug, Default)]
struct Candidates<'a> {
    /// Refs that existed and are gone: (full name, old value, kind).
    deleted: Vec<(&'a str, Oid, RestoreItemKind)>,
    /// Local branches a history operation moved: (full name, old tip).
    old_tips: Vec<(&'a str, Oid)>,
    config: Vec<&'a crate::journal::ConfigChange>,
    symrefs: Vec<&'a crate::journal::SymrefChange>,
    stashes: Vec<&'a crate::journal::StashRecord>,
    /// Paths whose working-tree bytes the entry removed or replaced.
    files: Vec<FileCandidate<'a>>,
}

#[derive(Debug)]
struct FileCandidate<'a> {
    path: &'a str,
    /// The working-tree bytes the entry removed or replaced. None only for
    /// a `.gitignore` / `.gitattributes` line the entry added.
    kept: Option<FileHash>,
    after: Option<PathHash>,
    after_worktree: Option<FileHash>,
}

fn candidates(entry: &JournalEntry) -> Candidates<'_> {
    let mut c = Candidates::default();
    for change in &entry.refs {
        if change.name.starts_with(UNDO_PIN_PREFIX) {
            continue;
        }
        match (change.old, change.new) {
            (Some(old), None) => {
                let kind = if change.name.starts_with("refs/tags/") {
                    RestoreItemKind::Tag
                } else if change.name.starts_with("refs/heads/") {
                    RestoreItemKind::Branch
                } else {
                    // refs/remotes/<r>/..., our remote-tag mirror, ...
                    RestoreItemKind::RemoteRef
                };
                c.deleted.push((change.name.as_str(), old, kind));
            }
            (Some(old), Some(new)) if old != new && change.name.starts_with("refs/heads/") => {
                c.old_tips.push((change.name.as_str(), old));
            }
            _ => {}
        }
    }
    c.config = entry
        .config
        .iter()
        .filter(|ch| sorted(&ch.old) != sorted(&ch.new))
        .collect();
    c.symrefs = entry
        .symrefs
        .iter()
        .filter(|s| s.old.is_some() && s.old != s.new)
        .collect();
    if let Some(st) = &entry.stash {
        c.stashes = st.removed.iter().collect();
    }
    if let Some(f) = &entry.files {
        for path in &f.paths {
            let hash = |snap: &crate::domain::Snapshot| {
                snap.hashes.iter().find(|h| &h.path == path).cloned()
            };
            let before = hash(&f.before);
            let after = hash(&f.after);
            let kept = before.and_then(|b| b.worktree);
            let after_worktree = after.as_ref().and_then(|a| a.worktree);
            if after_worktree == kept || (kept.is_none() && !is_rule_file(path)) {
                continue;
            }
            c.files.push(FileCandidate {
                path,
                kept,
                after,
                after_worktree,
            });
        }
    }
    c
}

fn sorted(pairs: &[(String, String)]) -> Vec<(String, String)> {
    let mut v = pairs.to_vec();
    v.sort();
    v
}

/// Why an entry that is not `Done` offers nothing to restore.
fn not_restorable_reason(state: EntryState) -> &'static str {
    match state {
        EntryState::Purged => "its old history was purged, so there is nothing left to restore",
        _ => "the operation never completed, so it is not known what it removed",
    }
}

impl JournalEntry {
    /// What "restore only this" would bring back, from the entry alone.
    pub fn restore_offer(&self) -> RestoreOffer {
        if self.state != EntryState::Done {
            return RestoreOffer {
                items: Vec::new(),
                not_restorable: Some(not_restorable_reason(self.state).into()),
            };
        }
        let c = candidates(self);
        let mut items = Vec::new();
        for (name, oid, kind) in &c.deleted {
            items.push(RestoreItem {
                kind: *kind,
                label: format!("{} ({})", display_ref(name), short(*oid)),
            });
        }
        for (name, oid) in &c.old_tips {
            items.push(RestoreItem {
                kind: RestoreItemKind::OldTip,
                label: format!("old tip of {} ({})", display_ref(name), short(*oid)),
            });
        }
        for r in &c.stashes {
            items.push(RestoreItem {
                kind: RestoreItemKind::Stash,
                label: format!("stash \"{}\" ({})", r.message, short(r.commit)),
            });
        }
        for f in &c.files {
            items.push(RestoreItem {
                kind: if is_rule_file(f.path) {
                    RestoreItemKind::Lines
                } else {
                    RestoreItemKind::File
                },
                label: f.path.to_string(),
            });
        }
        for ch in &c.config {
            // A branch's section follows its branch; a remote or an
            // upstream setting is its own item.
            if ch.section.starts_with("branch.") && !ch.old.is_empty() && !c.deleted.is_empty() {
                continue;
            }
            items.push(RestoreItem {
                kind: RestoreItemKind::Config,
                label: ch.section.clone(),
            });
        }
        let not_restorable = items.is_empty().then(|| self.nothing_reason());
        RestoreOffer {
            items,
            not_restorable,
        }
    }

    fn nothing_reason(&self) -> String {
        if self.index.is_some() || self.opstate.is_some() {
            "it only changed the index (staging), nothing was removed".into()
        } else if self.files.is_some() {
            "the files it changed held nothing that is lost".into()
        } else if self.refs.is_empty() && self.head.is_some() {
            "it only moved HEAD; the reflog lists the earlier positions".into()
        } else if !self.refs.is_empty() || self.stash.is_some() {
            "it only added things, nothing was removed".into()
        } else {
            "it removed nothing that can be brought back without guessing".into()
        }
    }
}

fn unique(base: &str, taken: &dyn Fn(&str) -> bool) -> String {
    if !taken(base) {
        return base.to_string();
    }
    (2..)
        .map(|n| format!("{base}-{n}"))
        .find(|c| !taken(c))
        .expect("an unbounded range has a free name")
}

fn unique_side(path: &str, taken: &dyn Fn(&str) -> bool) -> String {
    let first = format!("{path}.restored");
    if !taken(&first) {
        return first;
    }
    (2..)
        .map(|n| format!("{path}.restored.{n}"))
        .find(|c| !taken(c))
        .expect("an unbounded range has a free name")
}

/// Plans "restore only this" for `entry`. `name` renames the one branch or
/// tag that comes back. `path_exists` says whether a repository-relative
/// path is taken (to pick an unused `<name>.restored`).
pub fn plan_restore(
    entry: &JournalEntry,
    facts: &RestoreFacts,
    name: Option<&str>,
    path_exists: &dyn Fn(&str) -> bool,
) -> Result<RestorePlan, UndoRefusal> {
    if entry.state != EntryState::Done {
        return Err(UndoRefusal::NotRestorable {
            reason: not_restorable_reason(entry.state).into(),
        });
    }
    let c = candidates(entry);
    if c.deleted.is_empty()
        && c.old_tips.is_empty()
        && c.config.is_empty()
        && c.stashes.is_empty()
        && c.files.is_empty()
    {
        return Err(UndoRefusal::NotRestorable {
            reason: entry.nothing_reason(),
        });
    }

    // `name` renames the single branch or tag that comes back.
    let nameable = c
        .deleted
        .iter()
        .filter(|(_, _, k)| matches!(k, RestoreItemKind::Branch | RestoreItemKind::Tag))
        .count()
        + c.old_tips.len();
    if let Some(n) = name {
        if nameable != 1 {
            return Err(UndoRefusal::NotRestorable {
                reason: "--name needs an operation that brings back exactly one branch or tag"
                    .into(),
            });
        }
        if !valid_ref_name(n) {
            return Err(UndoRefusal::NotRestorable {
                reason: format!("{n} is not a valid branch or tag name"),
            });
        }
    }

    let ref_taken = |full: &str| facts.refs.contains_key(full);
    let mut actions: Vec<RestoreAction> = Vec::new();
    let mut already = false;
    // Branches coming back, by their original name: `branch.<n>` sections
    // follow them.
    let mut returning: BTreeSet<String> = BTreeSet::new();

    for (full, oid, kind) in &c.deleted {
        let prefix = match kind {
            RestoreItemKind::Tag => "refs/tags/",
            RestoreItemKind::Branch => "refs/heads/",
            _ => "",
        };
        let target = match (name, prefix.is_empty()) {
            (Some(n), false) => format!("{prefix}{n}"),
            _ => (*full).to_string(),
        };
        match facts.refs.get(&target) {
            Some(now) if now == oid => already = true,
            Some(_) => {
                let short_name = target.strip_prefix(prefix).unwrap_or(&target).to_string();
                let suggested = unique(&format!("{short_name}-restored"), &|n| {
                    ref_taken(&format!("{prefix}{n}"))
                });
                return Err(UndoRefusal::NameTaken {
                    name: short_name,
                    suggested,
                });
            }
            None => {
                if let (Some(orig), true) = (full.strip_prefix("refs/heads/"), target == *full) {
                    returning.insert(orig.to_string());
                }
                actions.push(RestoreAction::CreateRef {
                    name: target,
                    oid: *oid,
                    kind: *kind,
                });
            }
        }
    }
    for (full, oid) in &c.old_tips {
        let branch = full.strip_prefix("refs/heads/").unwrap_or(full);
        let target = match name {
            Some(n) => format!("refs/heads/{n}"),
            None => {
                let base = format!("restore/{branch}-{}", short(*oid));
                format!(
                    "refs/heads/{}",
                    unique(&base, &|n| ref_taken(&format!("refs/heads/{n}")))
                )
            }
        };
        match facts.refs.get(&target) {
            Some(now) if now == oid => already = true,
            Some(_) => {
                let n = target.strip_prefix("refs/heads/").unwrap_or(&target);
                return Err(UndoRefusal::NameTaken {
                    name: n.to_string(),
                    suggested: unique(&format!("{n}-restored"), &|c| {
                        ref_taken(&format!("refs/heads/{c}"))
                    }),
                });
            }
            None => actions.push(RestoreAction::CreateRef {
                name: target,
                oid: *oid,
                kind: RestoreItemKind::OldTip,
            }),
        }
    }

    for s in &c.symrefs {
        let (Some(old), true) = (&s.old, !facts.symrefs.contains_key(&s.name)) else {
            continue;
        };
        // Its target must exist once the refs above are back.
        let target_back = facts.refs.contains_key(old)
            || actions
                .iter()
                .any(|a| matches!(a, RestoreAction::CreateRef { name, .. } if name == old));
        if target_back {
            actions.push(RestoreAction::SetSymref {
                name: s.name.clone(),
                target: old.clone(),
            });
        }
    }

    for ch in &c.config {
        let now = facts.config.get(&ch.section).cloned().unwrap_or_default();
        if sorted(&now) == sorted(&ch.old) {
            already = true;
            continue;
        }
        // A branch's settings only come back with their branch.
        if let Some(branch) = ch.section.strip_prefix("branch.") {
            let exists = facts.refs.contains_key(&format!("refs/heads/{branch}"));
            if !exists && !returning.contains(branch) {
                continue;
            }
        }
        if sorted(&now) != sorted(&ch.new) {
            return Err(UndoRefusal::ConfigChanged {
                section: ch.section.clone(),
            });
        }
        actions.push(RestoreAction::SetConfig {
            section: ch.section.clone(),
            to: ch.old.clone(),
            from: now,
        });
    }

    for r in &c.stashes {
        if facts.stashes.contains(&r.commit) {
            already = true;
        } else {
            actions.push(RestoreAction::StashStore {
                commit: r.commit,
                message: r.message.clone(),
            });
        }
    }

    let history_moving = entry.effect == super::Effect::Files;
    let mut in_place: Vec<String> = Vec::new();
    let mut snapshot = None;
    let mut dests: BTreeSet<String> = BTreeSet::new();
    for f in &c.files {
        let now = facts.files.iter().find(|h| h.path == f.path);
        let now_wt = now.and_then(|h| h.worktree);
        if is_rule_file(f.path) {
            // Whether lines are missing is decided on the contents, when it runs.
            actions.push(RestoreAction::ReAddLines {
                path: f.path.to_string(),
                before: f.kept,
                after: f.after_worktree,
            });
            continue;
        }
        let Some(kept) = f.kept else { continue };
        if now_wt == Some(kept) {
            already = true;
            continue;
        }
        let as_left = {
            let (a_wt, a_ix) = f
                .after
                .as_ref()
                .map(|a| (a.worktree, a.index))
                .unwrap_or((None, None));
            let (n_wt, n_ix) = now.map(|n| (n.worktree, n.index)).unwrap_or((None, None));
            (a_wt, a_ix) == (n_wt, n_ix)
        };
        if as_left && !history_moving {
            in_place.push(f.path.to_string());
            snapshot = entry.files.as_ref().map(|p| p.before.tree);
        } else {
            let dest = unique_side(f.path, &|d| path_exists(d) || dests.contains(d));
            dests.insert(dest.clone());
            actions.push(RestoreAction::SideWrite {
                path: f.path.to_string(),
                dest,
                file: kept,
            });
        }
    }
    if let (false, Some(snapshot)) = (in_place.is_empty(), snapshot) {
        actions.push(RestoreAction::RestorePaths {
            paths: in_place,
            snapshot,
        });
    }

    if actions.is_empty() {
        return Err(if already {
            UndoRefusal::AlreadyRestored
        } else {
            UndoRefusal::NotRestorable {
                reason: entry.nothing_reason(),
            }
        });
    }
    Ok(RestorePlan {
        entry: entry.summary(),
        label: format!("Restore \"{}\"", entry.label),
        actions,
    })
}
