//! The path rules (`MovePaths`) over root trees: rename a file or folder,
//! move everything into a subdirectory, keep only a subdirectory as the new
//! root. They work on tree objects, never on blobs, so every blob keeps its
//! id and a moved folder is the very same tree object at its new place.
//!
//! A rule runs on the root a commit has after the content rules (so a
//! removal glob names the paths of the original layout), in recipe order
//! among the path rules. A rename never overwrites: when its target exists
//! in a commit (a file, a folder, or a file where a folder would have to
//! be) the commit is reported with the path and the preview is refused.

use std::collections::HashMap;

use gitpoulpi_core::domain::rewrite::paths::clean_dir;
use gitpoulpi_core::domain::rewrite::{PathChange, PathMove};
use gitpoulpi_core::domain::Oid;
use gitpoulpi_core::{Error, Result};

use super::pack::{self, Sink};
use super::plumbing::{parse_tree, serialise_tree, TreeEntry};
use super::walk::Reader;

/// What one root became.
pub(super) struct Moved {
    /// The new root; `None` when nothing is left (an extraction of a
    /// directory the commit does not have).
    pub(super) root: Option<Oid>,
    /// The path rules that acted, as bits of the rule's position among the
    /// path rules (the caller maps them).
    pub(super) acted: Vec<usize>,
    pub(super) paths: Vec<PathChange>,
    /// The first path a rename would have overwritten.
    pub(super) collision: Option<String>,
}

type Entry = (Vec<u8>, Oid);
/// A tree without an entry, and that entry.
type Taken = (Option<Oid>, Option<Entry>);

pub(super) struct Mover {
    rules: Vec<PathMove>,
    /// Bodies of the trees created in this run: they are in the run's pack,
    /// not yet in the repository, so `cat-file` cannot read them.
    made: HashMap<Oid, Vec<u8>>,
    /// `(tree, rule, depth)` to the tree without the entry at the rule's
    /// `from` path and that entry.
    taken: HashMap<(Oid, usize, usize), Taken>,
}

fn segments(path: &str) -> Vec<Vec<u8>> {
    clean_dir(path)
        .unwrap_or_default()
        .split('/')
        .map(|s| s.as_bytes().to_vec())
        .collect()
}

fn text(name: &[u8]) -> String {
    String::from_utf8_lossy(name).into_owned()
}

impl Mover {
    pub(super) fn new(rules: Vec<PathMove>) -> Self {
        Self {
            rules,
            made: HashMap::new(),
            taken: HashMap::new(),
        }
    }

    /// Whether a file at `path` of the checked-out tree ends up elsewhere
    /// (or nowhere): how an edited file is told from one the rules leave be.
    pub(super) fn would_move(&self, path: &str) -> bool {
        let mut at = Some(path.to_string());
        for rule in &self.rules {
            at = at.and_then(|p| rule.map_path(&p));
        }
        at.as_deref() != Some(path)
    }

    /// A tree another stage created (its body is not in the repository).
    pub(super) fn remember(&mut self, id: Oid, body: &[u8]) {
        self.made.insert(id, body.to_vec());
    }

    fn entries(&self, reader: &mut Reader, tree: Oid) -> Result<Vec<TreeEntry>> {
        let body = match self.made.get(&tree) {
            Some(body) => body.clone(),
            None => reader.get(tree)?.1,
        };
        parse_tree(&body)
            .ok_or_else(|| Error::Backend(format!("{} is not a readable tree", tree.to_hex())))
    }

    fn write(&mut self, sink: &mut Sink, entries: Vec<TreeEntry>) -> Result<Option<Oid>> {
        if entries.is_empty() {
            return Ok(None);
        }
        let body = serialise_tree(entries);
        let id = sink.add(pack::TREE, &body)?;
        self.made.insert(id, body);
        Ok(Some(id))
    }

    /// `tree` without the entry at `segs[depth..]`, and that entry.
    fn take(
        &mut self,
        reader: &mut Reader,
        sink: &mut Sink,
        (rule, tree): (usize, Oid),
        segs: &[Vec<u8>],
        depth: usize,
    ) -> Result<Taken> {
        if let Some(done) = self.taken.get(&(tree, rule, depth)) {
            return Ok(done.clone());
        }
        let mut entries = self.entries(reader, tree)?;
        let result = match entries.iter().position(|e| e.name == segs[depth]) {
            None => (Some(tree), None),
            Some(i) if depth + 1 == segs.len() => {
                let gone = entries.remove(i);
                (
                    self.write(sink, entries)?,
                    Some((gone.mode.clone(), gone.oid)),
                )
            }
            Some(i) if entries[i].is_tree() => {
                let below = entries[i].oid;
                let (rest, gone) = self.take(reader, sink, (rule, below), segs, depth + 1)?;
                if gone.is_none() {
                    (Some(tree), None)
                } else {
                    match rest {
                        Some(new) => entries[i].oid = new,
                        None => {
                            entries.remove(i);
                        }
                    }
                    (self.write(sink, entries)?, gone)
                }
            }
            // A file where a folder is named: nothing there to take.
            Some(_) => (Some(tree), None),
        };
        self.taken.insert((tree, rule, depth), result.clone());
        Ok(result)
    }

    /// `tree` with `entry` at `segs[depth..]`; `Err(path)` names what is
    /// already there.
    fn put(
        &mut self,
        reader: &mut Reader,
        sink: &mut Sink,
        tree: Option<Oid>,
        segs: &[Vec<u8>],
        depth: usize,
        entry: &Entry,
    ) -> Result<std::result::Result<Oid, String>> {
        let mut entries = match tree {
            Some(t) => self.entries(reader, t)?,
            None => Vec::new(),
        };
        let name = &segs[depth];
        let here = || text(&segs[..=depth].join(&b'/'));
        let at = entries.iter().position(|e| &e.name == name);
        if depth + 1 == segs.len() {
            if at.is_some() {
                return Ok(Err(here()));
            }
            entries.push(TreeEntry {
                mode: entry.0.clone(),
                name: name.clone(),
                oid: entry.1,
            });
        } else {
            match at {
                Some(i) if entries[i].is_tree() => {
                    let below = entries[i].oid;
                    match self.put(reader, sink, Some(below), segs, depth + 1, entry)? {
                        Ok(new) => entries[i].oid = new,
                        Err(path) => return Ok(Err(path)),
                    }
                }
                // A file is in the way of a folder.
                Some(_) => return Ok(Err(here())),
                None => match self.put(reader, sink, None, segs, depth + 1, entry)? {
                    Ok(new) => entries.push(TreeEntry {
                        mode: b"40000".to_vec(),
                        name: name.clone(),
                        oid: new,
                    }),
                    Err(path) => return Ok(Err(path)),
                },
            }
        }
        let id = self
            .write(sink, entries)?
            .expect("a tree with an entry is not empty");
        Ok(Ok(id))
    }

    /// Runs every path rule over `root`.
    pub(super) fn apply(
        &mut self,
        reader: &mut Reader,
        sink: &mut Sink,
        root: Option<Oid>,
    ) -> Result<Moved> {
        let mut out = Moved {
            root,
            acted: Vec::new(),
            paths: Vec::new(),
            collision: None,
        };
        for k in 0..self.rules.len() {
            let Some(current) = out.root else { break };
            match self.rules[k].clone() {
                PathMove::Rename { from, to } => {
                    let (from_segs, to_segs) = (segments(&from), segments(&to));
                    let (rest, gone) = self.take(reader, sink, (k, current), &from_segs, 0)?;
                    let Some(entry) = gone else { continue };
                    match self.put(reader, sink, rest, &to_segs, 0, &entry)? {
                        Ok(new) => out.root = Some(new),
                        Err(path) => {
                            out.collision = Some(path);
                            return Ok(out);
                        }
                    }
                    out.acted.push(k);
                    out.paths.push(PathChange::Renamed {
                        from: clean_dir(&from).unwrap_or(from),
                        to: clean_dir(&to).unwrap_or(to),
                    });
                }
                PathMove::IntoSubdir { dir } => {
                    let mut tree = current;
                    for name in segments(&dir).into_iter().rev() {
                        tree = self
                            .write(
                                sink,
                                vec![TreeEntry {
                                    mode: b"40000".to_vec(),
                                    name,
                                    oid: tree,
                                }],
                            )?
                            .expect("one entry");
                    }
                    out.root = Some(tree);
                    out.acted.push(k);
                    out.paths.push(PathChange::Renamed {
                        from: ".".into(),
                        to: clean_dir(&dir).unwrap_or(dir),
                    });
                }
                PathMove::ExtractSubdir { dir, .. } => {
                    let mut at = Some(current);
                    for name in segments(&dir) {
                        let Some(tree) = at else { break };
                        at = self
                            .entries(reader, tree)?
                            .into_iter()
                            .find(|e| e.name == name && e.is_tree())
                            .map(|e| e.oid);
                    }
                    out.root = at;
                    out.acted.push(k);
                    if at.is_some() {
                        out.paths.push(PathChange::Renamed {
                            from: clean_dir(&dir).unwrap_or(dir),
                            to: ".".into(),
                        });
                    }
                }
            }
        }
        Ok(out)
    }
}
