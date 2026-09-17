//! Notes follow their commits: a note attached to a rewritten commit moves
//! to the new id. The notes tree is rebuilt with the fan-out it had (`ab/cdef...`)
//! and the ref moves to a new notes commit on top of the old one, in the
//! same `update-ref` transaction as everything else.

use std::collections::{BTreeMap, HashMap};

use gitpoulpi_core::domain::rewrite::GitDate;
use gitpoulpi_core::domain::Oid;
use gitpoulpi_core::Result;

use super::objects::{Commit, Person};
use super::pack;
use super::plumbing::{parse_tree, serialise_tree, TreeEntry};
use super::refs::RefLine;
use super::serialize::object_id;
use super::walk::Reader;

/// A notes ref that moves.
pub(crate) struct NoteMove {
    pub(crate) name: String,
    pub(crate) old: Oid,
    pub(crate) new: Oid,
}

fn is_hex(text: &str, len: usize) -> bool {
    text.len() == len && text.bytes().all(|b| b.is_ascii_hexdigit())
}

/// The notes of a tree: `(annotated commit hex, entry)`, the deepest fan-out
/// seen, and the entries that are no note (kept at the root). `None` when the
/// tree has a shape this code does not understand: such notes are left alone.
struct Loaded {
    notes: Vec<(String, TreeEntry)>,
    others: Vec<TreeEntry>,
    depth: usize,
}

fn load(reader: &mut Reader, tree: Oid, prefix: &str, out: &mut Loaded) -> Option<()> {
    let (kind, body) = reader.get(tree).ok()?;
    if kind != "tree" {
        return None;
    }
    for e in parse_tree(&body)? {
        let name = String::from_utf8(e.name.clone()).ok();
        match name {
            Some(name) if e.is_tree() && is_hex(&name, 2) && prefix.len() + 2 < 40 => {
                load(reader, e.oid, &format!("{prefix}{name}"), out)?;
            }
            Some(name) if !e.is_tree() && is_hex(&format!("{prefix}{name}"), 40) => {
                out.depth = out.depth.max(prefix.len() / 2);
                out.notes.push((format!("{prefix}{name}"), e));
            }
            _ if prefix.is_empty() => out.others.push(e),
            _ => return None,
        }
    }
    Some(())
}

/// Writes the trees of `notes` (key to entry) at `depth` levels of fan-out;
/// the id of the tree at `level`.
fn build(
    notes: &[(&str, &TreeEntry)],
    others: &[TreeEntry],
    level: usize,
    depth: usize,
    objects: &mut Vec<(u8, Vec<u8>)>,
) -> Oid {
    let mut entries: Vec<TreeEntry> = others.to_vec();
    if level == depth {
        for (key, e) in notes {
            entries.push(TreeEntry {
                mode: e.mode.clone(),
                name: key.as_bytes()[level * 2..].to_vec(),
                oid: e.oid,
            });
        }
    } else {
        let mut groups: BTreeMap<&str, Vec<(&str, &TreeEntry)>> = BTreeMap::new();
        for (key, e) in notes {
            groups
                .entry(&key[level * 2..level * 2 + 2])
                .or_default()
                .push((key, e));
        }
        for (name, group) in groups {
            entries.push(TreeEntry {
                mode: b"40000".to_vec(),
                name: name.as_bytes().to_vec(),
                oid: build(&group, &[], level + 1, depth, objects),
            });
        }
    }
    let body = serialise_tree(entries);
    let id = object_id("tree", &body);
    objects.push((pack::TREE, body));
    id
}

/// Moves the notes of every rewritten commit. `objects` receives the new
/// trees and notes commits for the pack; `now` stamps the notes commits.
pub(crate) fn remap(
    reader: &mut Reader,
    notes_refs: &[RefLine],
    new_id: &HashMap<Oid, Oid>,
    now: i64,
    objects: &mut Vec<(u8, Vec<u8>)>,
) -> Result<Vec<NoteMove>> {
    let mut moves = Vec::new();
    for r in notes_refs.iter().filter(|r| r.kind == "commit") {
        let Ok((_, raw)) = reader.get(r.oid) else {
            continue;
        };
        let Ok(commit) = Commit::parse(raw) else {
            continue;
        };
        let Some(tree) = commit.tree() else { continue };
        let mut loaded = Loaded {
            notes: Vec::new(),
            others: Vec::new(),
            depth: 0,
        };
        if load(reader, tree, "", &mut loaded).is_none() {
            continue;
        }
        let moved = |key: &str| {
            Oid::from_hex(key)
                .and_then(|k| new_id.get(&k))
                .map(|o| o.to_hex())
        };
        if !loaded.notes.iter().any(|(k, _)| moved(k).is_some()) {
            continue;
        }
        // A note already on the new id wins over one that moves onto it.
        let mut by_key: BTreeMap<String, TreeEntry> = BTreeMap::new();
        for (key, e) in &loaded.notes {
            if moved(key).is_none() {
                by_key.insert(key.clone(), e.clone());
            }
        }
        for (key, e) in &loaded.notes {
            if let Some(new_key) = moved(key) {
                by_key.entry(new_key).or_insert_with(|| e.clone());
            }
        }
        let flat: Vec<(&str, &TreeEntry)> = by_key.iter().map(|(k, e)| (k.as_str(), e)).collect();
        let new_tree = build(&flat, &loaded.others, 0, loaded.depth, objects);

        let mut who = commit.committer().unwrap_or_else(|| {
            Person::parse(b"GitPoulpi <gitpoulpi@localhost> 0 +0000").expect("a valid ident")
        });
        who.set_date(GitDate::new(now, who.offset));
        let line = String::from_utf8_lossy(&who.line()).into_owned();
        let body = format!(
            "tree {}\nparent {}\nauthor {line}\ncommitter {line}\n\nNotes moved by GitPoulpi history rewrite\n",
            new_tree.to_hex(),
            r.oid.to_hex()
        )
        .into_bytes();
        let new = object_id("commit", &body);
        objects.push((pack::COMMIT, body));
        moves.push(NoteMove {
            name: r.name.clone(),
            old: r.oid,
            new,
        });
    }
    Ok(moves)
}
