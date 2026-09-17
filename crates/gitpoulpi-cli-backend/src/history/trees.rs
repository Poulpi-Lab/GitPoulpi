//! The content rules over trees. A tree is rewritten once per `(tree id,
//! path)` for the whole run: the trees below a commit that changed nothing
//! are the same objects as in the commit before, so "remove `secrets/` from
//! 100 000 commits" re-creates only the trees on the way to `secrets/`.
//! Globs that name a directory prefix tell the walk which subtrees no rule
//! can reach, and those are never read.

use std::collections::HashMap;
use std::rc::Rc;

use gitpoulpi_core::domain::rewrite::lfs as lfs_rewrite;
use gitpoulpi_core::domain::rewrite::textconv::{Attributes, EolAttr};
use gitpoulpi_core::domain::rewrite::{
    GlobSet, PathChange, PathFilter, PathMove, Recipe, RewriteRefusal, RuleKind,
};
use gitpoulpi_core::domain::Oid;
use gitpoulpi_core::{Error, Result};

use super::blobs::{Blobs, Decision};
use super::moves::Mover;
use super::pack::{self, Sink};
use super::plumbing::{parse_tree, TreeEntry};
use super::walk::Reader;

/// Paths a commit's row lists at most (a rule may touch thousands).
const PATH_CAP: usize = 64;

/// The bit of the rule at position `n` among the content rules.
pub(super) fn bit(n: usize) -> u64 {
    1 << n.min(63)
}

/// What a rewritten tree is.
#[derive(Clone)]
pub(super) struct Done {
    /// The new tree; `None` when it became empty.
    pub(super) new: Option<Oid>,
    pub(super) changed: bool,
    pub(super) paths: Rc<Vec<PathChange>>,
    /// The content rules that acted (see [`bit`]).
    pub(super) hit: u64,
    /// The root became empty by a path rule (an extraction of a directory
    /// the commit does not have).
    pub(super) vanished_root: bool,
    /// The first path a rename would have overwritten in this commit.
    pub(super) collision: Option<String>,
}

/// A rewritten tree by its path and the `.gitattributes` blob it was read with.
type TreeEntryDone = (Box<str>, Option<Oid>, Done);

pub(super) struct Content {
    removals: Vec<(u64, GlobSet)>,
    blobs: Blobs,
    /// A rule can reach any directory (a glob without a slash, or a rule on
    /// every file).
    everywhere: bool,
    /// Else the directories the anchored globs lead through.
    prefixes: Vec<String>,
    trees: HashMap<Oid, Vec<TreeEntryDone>>,
    pub(super) sink: Sink,
    /// Index in the recipe of each content rule, in bit order.
    pub(super) rule_ids: Vec<usize>,
    /// S21: the patterns of each LFS import rule (by bit) and of the export
    /// rules, for the `.gitattributes` of every tree they touch.
    lfs_in: Vec<(u64, Vec<String>)>,
    lfs_out: Vec<String>,
    lfs_out_mask: u64,
    /// The `.gitattributes` the root tree gets, by (its blob, rules that acted).
    attrs: HashMap<(Option<Oid>, u64), Option<Oid>>,
    /// The path rules, with the bit of each among the content rules.
    mover: Option<Mover>,
    move_bits: Vec<u64>,
    /// S22: the exec bit leaves the files these rules match.
    strip_exec: Vec<(u64, PathFilter)>,
    /// S22: a line-ending rule follows `.gitattributes`; the root file of the
    /// commit being rewritten (its blob and what it says) is what it reads.
    follow_attrs: bool,
    attrs_parsed: HashMap<Oid, Rc<Attributes>>,
    cur_attrs: Option<(Oid, Rc<Attributes>)>,
    /// A removal or blob rule exists: the trees are walked.
    walks: bool,
    /// A path rule keeps only a subdirectory.
    pub(super) extracts: bool,
}

pub(super) struct Counters {
    pub(super) lfs_blobs: u64,
    pub(super) lfs_bytes: u64,
    pub(super) lfs_objects: u64,
    pub(super) lfs_restored: u64,
    pub(super) blobs_changed: u64,
    pub(super) skipped_binary: u64,
    pub(super) skipped_large: u64,
    pub(super) skipped_undecodable: u64,
}

/// The literal directories an anchored glob leads through: its segments up
/// to the first one with a wildcard. `None` for a glob that matches at any
/// depth.
fn glob_prefix(glob: &str) -> Option<String> {
    let trimmed = glob.trim();
    if !trimmed.trim_end_matches('/').contains('/') {
        return None;
    }
    let body = trimmed.trim_start_matches('/').trim_end_matches('/');
    let literal: Vec<&str> = body
        .split('/')
        .take_while(|seg| !seg.contains(['*', '?', '[']))
        .collect();
    Some(literal.join("/"))
}

fn path_cap(into: &mut Vec<PathChange>, from: &[PathChange]) {
    let room = PATH_CAP.saturating_sub(into.len());
    into.extend(from.iter().take(room).cloned());
}

impl Content {
    /// `None` when the recipe has no content rule.
    #[cfg(test)]
    pub(super) fn new(recipe: &Recipe, sink: Sink) -> Result<Option<Self>> {
        Self::with_secrets(recipe, sink, &HashMap::new())
    }

    /// `found`: the values the scanner found for each scanning secret rule,
    /// by rule index (`scan::resolve`).
    pub(super) fn with_secrets(
        recipe: &Recipe,
        sink: Sink,
        found: &HashMap<usize, Vec<Vec<u8>>>,
    ) -> Result<Option<Self>> {
        let mut me = Self {
            removals: Vec::new(),
            blobs: Blobs::new(),
            everywhere: false,
            prefixes: Vec::new(),
            trees: HashMap::new(),
            sink,
            rule_ids: Vec::new(),
            lfs_in: Vec::new(),
            lfs_out: Vec::new(),
            lfs_out_mask: 0,
            attrs: HashMap::new(),
            mover: None,
            move_bits: Vec::new(),
            strip_exec: Vec::new(),
            follow_attrs: false,
            attrs_parsed: HashMap::new(),
            cur_attrs: None,
            walks: false,
            extracts: false,
        };
        me.blobs.lfs.configure(me.sink.dir().to_path_buf());
        let mut rules = Vec::new();
        for (index, rule) in recipe.enabled_rules() {
            let b = bit(me.rule_ids.len());
            match &rule.kind {
                RuleKind::ContentReplace(c) => {
                    me.blobs
                        .add_replace(b, c, found.get(&index).map(Vec::as_slice))?;
                    if c.paths.is_empty() {
                        me.everywhere = true;
                    }
                    me.reach(&c.paths);
                    me.walks = true;
                }
                RuleKind::RemovePaths { globs } => {
                    let set = GlobSet::new(globs)
                        .map_err(|e| Error::RewriteRefused(RewriteRefusal::InvalidRule, e))?;
                    me.removals.push((b, set));
                    me.reach(globs);
                    me.walks = true;
                }
                RuleKind::RemoveBlobsOver { bytes } => {
                    me.blobs.add_over(b, *bytes);
                    me.everywhere = true;
                    me.walks = true;
                }
                RuleKind::MovePaths(m) => {
                    me.extracts |= matches!(m, PathMove::ExtractSubdir { .. });
                    rules.push(m.clone());
                    me.move_bits.push(b);
                }
                RuleKind::LfsImport(l) => {
                    me.blobs
                        .add_lfs_import(b, &l.patterns, l.min_size.unwrap_or(0))?;
                    me.lfs_in.push((b, l.patterns.clone()));
                    me.reach(&l.patterns);
                    me.walks = true;
                }
                RuleKind::LfsExport(l) => {
                    me.blobs.add_lfs_export(b, &l.patterns)?;
                    me.lfs_out.extend(l.patterns.iter().cloned());
                    me.lfs_out_mask |= b;
                    me.reach(&l.patterns);
                    me.walks = true;
                }
                RuleKind::LineEndings(e) => {
                    me.blobs.add_eol(b, e)?;
                    me.follow_attrs |= e.follow_attributes;
                    if e.paths.is_empty() {
                        me.everywhere = true;
                    }
                    me.reach(&e.paths);
                    me.walks = true;
                }
                RuleKind::Reencode(e) => {
                    me.blobs.add_reencode(b, e)?;
                    if e.paths.is_empty() {
                        me.everywhere = true;
                    }
                    me.reach(&e.paths);
                    me.walks = true;
                }
                RuleKind::StripExec(e) => {
                    let filter = PathFilter::new(&e.paths, &e.exclude)
                        .map_err(|e| Error::RewriteRefused(RewriteRefusal::InvalidRule, e))?;
                    me.strip_exec.push((b, filter));
                    me.reach(&e.paths);
                    me.walks = true;
                }
                _ => continue,
            }
            me.rule_ids.push(index);
        }
        if !rules.is_empty() {
            me.mover = Some(Mover::new(rules));
        }
        Ok((!me.rule_ids.is_empty()).then_some(me))
    }

    fn reach(&mut self, globs: &[String]) {
        for g in globs {
            match glob_prefix(g) {
                Some(p) => self.prefixes.push(p),
                None => self.everywhere = true,
            }
        }
    }

    /// S21: where an export finds the objects.
    pub(super) fn set_lfs_storage(&mut self, storage: std::path::PathBuf) {
        self.blobs.lfs.set_storage(storage);
    }

    pub(super) fn lfs(&self) -> &super::lfs::Lfs {
        &self.blobs.lfs
    }

    pub(super) fn counters(&self) -> Counters {
        Counters {
            lfs_blobs: self.blobs.lfs.blobs,
            lfs_bytes: self.blobs.lfs.bytes,
            lfs_objects: self.blobs.lfs.objects(),
            lfs_restored: self.blobs.lfs.restored,
            blobs_changed: self.blobs.changed,
            skipped_binary: self.blobs.skipped_binary,
            skipped_large: self.blobs.skipped_large,
            skipped_undecodable: self.blobs.skipped_undecodable,
        }
    }

    /// Whether a rule can act somewhere under (or at) the directory `dir`.
    fn may_match(&self, dir: &str) -> bool {
        self.everywhere
            || self.prefixes.iter().any(|p| {
                p.is_empty()
                    || p == dir
                    || p.strip_prefix(dir).is_some_and(|r| r.starts_with('/'))
                    || dir
                        .strip_prefix(p.as_str())
                        .is_some_and(|r| r.starts_with('/'))
            })
    }

    /// The bit of the first path-removal rule that removes `path`.
    fn removed_by(&self, path: &str) -> Option<u64> {
        self.removals
            .iter()
            .find(|(_, set)| set.is_match(path))
            .map(|(b, _)| *b)
    }

    /// The bit of the first rule that strips the exec bit of `path`.
    fn strip_exec_by(&self, path: &str) -> Option<u64> {
        self.strip_exec
            .iter()
            .find(|(_, f)| f.matches(path))
            .map(|(b, _)| *b)
    }

    /// Reads the root `.gitattributes` of the commit whose root tree is
    /// `tree` (only when a line-ending rule follows the attributes).
    fn load_attributes(&mut self, reader: &mut Reader, tree: Oid) -> Result<()> {
        if !self.follow_attrs {
            return Ok(());
        }
        let (kind, body) = reader.get(tree)?;
        let blob = (kind == "tree")
            .then(|| parse_tree(&body))
            .flatten()
            .and_then(|entries| {
                entries
                    .into_iter()
                    .find(|e| e.name == b".gitattributes" && !e.is_tree() && e.mode != b"120000")
            })
            .map(|e| e.oid);
        self.cur_attrs = match blob {
            None => None,
            Some(oid) => {
                let attrs = match self.attrs_parsed.get(&oid) {
                    Some(a) => a.clone(),
                    None => {
                        let text = reader.get(oid)?.1;
                        let a = Rc::new(Attributes::parse(&text));
                        self.attrs_parsed.insert(oid, a.clone());
                        a
                    }
                };
                Some((oid, attrs))
            }
        };
        Ok(())
    }

    fn eol_attr(&self, path: &str) -> EolAttr {
        self.cur_attrs
            .as_ref()
            .map_or(EolAttr::Unset, |(_, a)| a.eol_for(path))
    }

    /// The root tree of a commit, rewritten.
    pub(super) fn rewrite_root(&mut self, reader: &mut Reader, tree: Oid) -> Result<Done> {
        self.load_attributes(reader, tree)?;
        let mut done = if self.walks {
            self.rewrite_tree(reader, tree, "")?
        } else {
            Done {
                new: Some(tree),
                changed: false,
                paths: Rc::default(),
                hit: 0,
                vanished_root: false,
                collision: None,
            }
        };
        if let Some(mover) = self.mover.as_mut() {
            let moved = mover.apply(reader, &mut self.sink, done.new)?;
            if moved.collision.is_some() || moved.root != done.new || !moved.acted.is_empty() {
                let mut paths = done.paths.to_vec();
                path_cap(&mut paths, &moved.paths);
                for k in &moved.acted {
                    done.hit |= self.move_bits[*k];
                }
                done.changed |= moved.root != done.new;
                done.vanished_root = moved.root.is_none() && done.new.is_some();
                done.new = moved.root;
                done.paths = Rc::new(paths);
                done.collision = moved.collision;
            }
        }
        if done.changed && done.new.is_none() {
            done.new = Some(self.sink.add(pack::TREE, b"")?);
        }
        Ok(done)
    }

    fn rewrite_tree(&mut self, reader: &mut Reader, tree: Oid, prefix: &str) -> Result<Done> {
        let attrs_id = self.cur_attrs.as_ref().map(|(o, _)| *o);
        if let Some(done) = self
            .trees
            .get(&tree)
            .and_then(|v| v.iter().find(|(p, a, _)| &**p == prefix && *a == attrs_id))
        {
            return Ok(done.2.clone());
        }
        let (kind, body) = reader.get(tree)?;
        let entries = (kind == "tree")
            .then(|| parse_tree(&body))
            .flatten()
            .ok_or_else(|| Error::Backend(format!("{} is not a readable tree", tree.to_hex())))?;
        let mut out: Vec<u8> = Vec::with_capacity(body.len());
        let (mut changed, mut hit) = (false, 0u64);
        let mut paths: Vec<PathChange> = Vec::new();
        for e in &entries {
            let name = String::from_utf8_lossy(&e.name);
            let path = if prefix.is_empty() {
                name.into_owned()
            } else {
                format!("{prefix}/{name}")
            };
            let mut oid = e.oid;
            let mut mode: &[u8] = &e.mode;
            if let Some(b) = self.removed_by(&path) {
                changed = true;
                hit |= b;
                path_cap(&mut paths, &[PathChange::Removed { path }]);
                continue;
            }
            if e.is_tree() {
                if self.may_match(&path) {
                    let sub = self.rewrite_tree(reader, e.oid, &path)?;
                    hit |= sub.hit;
                    if sub.changed {
                        changed = true;
                        path_cap(&mut paths, &sub.paths);
                        match sub.new {
                            Some(new) => oid = new,
                            // The directory emptied: git has no empty trees.
                            None => continue,
                        }
                    }
                }
            } else if e.mode != b"160000" {
                let mut mode_changed = false;
                if e.mode == b"100755" {
                    if let Some(b) = self.strip_exec_by(&path) {
                        mode = b"100644";
                        mode_changed = true;
                        changed = true;
                        hit |= b;
                    }
                }
                let regular = e.mode == b"100644" || e.mode == b"100755";
                let (decision, bits) = if self.blobs.is_empty() {
                    (Decision::Keep, 0)
                } else {
                    let attr = self.eol_attr(&path);
                    self.blobs
                        .decide(reader, &mut self.sink, e.oid, &path, regular, attr)?
                };
                hit |= bits;
                match decision {
                    Decision::Keep => {
                        if mode_changed {
                            path_cap(&mut paths, &[PathChange::Modified { path }]);
                        }
                    }
                    Decision::Drop => {
                        changed = true;
                        path_cap(&mut paths, &[PathChange::Removed { path }]);
                        continue;
                    }
                    Decision::New(new) => {
                        changed = true;
                        oid = new;
                        let lfs_in = self.lfs_in.iter().any(|(b, _)| bits & b != 0);
                        let change = if lfs_in {
                            PathChange::LfsImported { path }
                        } else if bits & self.lfs_out_mask != 0 {
                            PathChange::LfsExported { path }
                        } else {
                            PathChange::Modified { path }
                        };
                        path_cap(&mut paths, &[change]);
                    }
                }
            }
            out.extend_from_slice(mode);
            out.push(b' ');
            out.extend_from_slice(&e.name);
            out.push(0);
            out.extend_from_slice(&oid.0);
        }
        if prefix.is_empty() && self.has_lfs() {
            if let Some(rebuilt) = self.lfs_attributes(reader, &out, hit)? {
                out = rebuilt;
                changed = true;
                path_cap(
                    &mut paths,
                    &[PathChange::Modified {
                        path: ".gitattributes".into(),
                    }],
                );
            }
        }
        let new = if !changed {
            Some(tree)
        } else if out.is_empty() {
            None
        } else {
            Some(self.sink.add(pack::TREE, &out)?)
        };
        if let (Some(mover), Some(id), true) = (self.mover.as_mut(), new, changed) {
            mover.remember(id, &out);
        }
        let done = Done {
            new,
            changed,
            paths: Rc::new(paths),
            hit,
            vanished_root: false,
            collision: None,
        };
        self.trees
            .entry(tree)
            .or_default()
            .push((prefix.into(), attrs_id, done.clone()));
        Ok(done)
    }

    fn has_lfs(&self) -> bool {
        !self.lfs_in.is_empty() || !self.lfs_out.is_empty()
    }

    /// The root tree `out` (serialised entries) with its `.gitattributes`
    /// brought in line: the lines of the import rules that converted a file in
    /// this tree are added, the lines of the export rules are removed. `None`
    /// when the tree is already right.
    fn lfs_attributes(
        &mut self,
        reader: &mut Reader,
        out: &[u8],
        hit: u64,
    ) -> Result<Option<Vec<u8>>> {
        let mut entries = parse_tree(out).unwrap_or_default();
        let at = entries.iter().position(|e| e.name == b".gitattributes");
        let blob = at.map(|i| entries[i].oid);
        let regular =
            at.is_some_and(|i| entries[i].mode == b"100644" || entries[i].mode == b"100755");
        if at.is_some() && !regular {
            // A symlink or a directory by that name: left alone.
            return Ok(None);
        }
        let key = (blob, hit);
        let new_blob = match self.attrs.get(&key) {
            Some(done) => *done,
            None => {
                let old = match blob {
                    Some(b) => Some(reader.get(b)?.1),
                    None => None,
                };
                let add: Vec<String> = self
                    .lfs_in
                    .iter()
                    .filter(|(b, _)| hit & b != 0)
                    .flat_map(|(_, p)| p.iter().cloned())
                    .collect();
                let mut text = old.clone();
                if !self.lfs_out.is_empty() {
                    if let Some(t) = lfs_rewrite::remove_attributes(
                        text.as_deref().unwrap_or_default(),
                        &self.lfs_out,
                    ) {
                        text = Some(t);
                    }
                }
                if !add.is_empty() {
                    if let Some(t) = lfs_rewrite::add_attributes(text.as_deref(), &add) {
                        text = Some(t);
                    }
                }
                let result = match text {
                    Some(t) if Some(&t) == old.as_ref() => blob,
                    // An export that removed every line leaves no file.
                    Some(t) if t.is_empty() => None,
                    Some(t) => Some(self.sink.add(pack::BLOB, &t)?),
                    None => blob,
                };
                self.attrs.insert(key, result);
                result
            }
        };
        if new_blob == blob {
            return Ok(None);
        }
        match (at, new_blob) {
            (Some(i), Some(n)) => entries[i].oid = n,
            (Some(i), None) => {
                entries.remove(i);
            }
            (None, Some(n)) => {
                entries.push(TreeEntry {
                    mode: b"100644".to_vec(),
                    name: b".gitattributes".to_vec(),
                    oid: n,
                });
                // Git's order: a directory sorts as if its name ended in `/`.
                entries.sort_by_key(|e| {
                    let mut k = e.name.clone();
                    if e.is_tree() {
                        k.push(b'/');
                    }
                    k
                });
            }
            (None, None) => return Ok(None),
        }
        let mut rebuilt = Vec::with_capacity(out.len() + 40);
        for e in &entries {
            rebuilt.extend_from_slice(&e.mode);
            rebuilt.push(b' ');
            rebuilt.extend_from_slice(&e.name);
            rebuilt.push(0);
            rebuilt.extend_from_slice(&e.oid.0);
        }
        Ok(Some(rebuilt))
    }

    /// Whether the rules would change (or remove) the file at `path` of the
    /// tree `root`: how a dirty file is told from one the rewrite leaves alone.
    pub(super) fn would_change(
        &mut self,
        reader: &mut Reader,
        root: Oid,
        path: &str,
    ) -> Result<bool> {
        if self.mover.as_ref().is_some_and(|m| m.would_move(path)) {
            return Ok(true);
        }
        self.load_attributes(reader, root)?;
        let parts: Vec<&str> = path.split('/').collect();
        let mut tree = root;
        for (i, part) in parts.iter().enumerate() {
            let (kind, body) = reader.get(tree)?;
            let entries = (kind == "tree")
                .then(|| parse_tree(&body))
                .flatten()
                .unwrap_or_default();
            let Some(entry) = entries.iter().find(|e| e.name == part.as_bytes()) else {
                return Ok(false);
            };
            if i + 1 < parts.len() {
                if !entry.is_tree() {
                    return Ok(false);
                }
                tree = entry.oid;
                continue;
            }
            if entry.is_tree() || entry.mode == b"160000" {
                return Ok(false);
            }
            if self.removed_by(path).is_some() {
                return Ok(true);
            }
            if path == ".gitattributes" && self.has_lfs() {
                return Ok(true);
            }
            if entry.mode == b"100755" && self.strip_exec_by(path).is_some() {
                return Ok(true);
            }
            if self.blobs.is_empty() {
                return Ok(false);
            }
            let regular = entry.mode == b"100644" || entry.mode == b"100755";
            let attr = self.eol_attr(path);
            let (decision, _) =
                self.blobs
                    .decide(reader, &mut self.sink, entry.oid, path, regular, attr)?;
            return Ok(decision != Decision::Keep);
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn globs_tell_which_directories_matter() {
        assert_eq!(
            glob_prefix("secrets/"),
            None,
            "a slashless glob is any depth"
        );
        assert_eq!(glob_prefix("*.log"), None);
        assert_eq!(glob_prefix("docs/*.md"), Some("docs".into()));
        assert_eq!(glob_prefix("a/b/c.txt"), Some("a/b/c.txt".into()));
        assert_eq!(glob_prefix("/build/out"), Some("build/out".into()));
        assert_eq!(glob_prefix("*/x"), Some(String::new()));
    }

    #[test]
    fn a_prefix_keeps_the_way_to_it_and_what_is_below_it() {
        let recipe = Recipe::new(
            "t",
            gitpoulpi_core::domain::rewrite::Scope::default(),
            vec![gitpoulpi_core::domain::rewrite::Rule::new(
                RuleKind::RemovePaths {
                    globs: vec!["a/b/*.key".into()],
                },
            )],
        );
        let c = Content::new(&recipe, Sink::new(std::path::PathBuf::new()))
            .unwrap()
            .unwrap();
        assert!(c.may_match("a"));
        assert!(c.may_match("a/b"));
        assert!(c.may_match("a/b/deep"));
        assert!(!c.may_match("x"));
        assert!(!c.may_match("a/c"));
        assert!(!c.may_match("ab"));
    }
}
