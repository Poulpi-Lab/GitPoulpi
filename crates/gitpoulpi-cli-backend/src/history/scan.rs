//! The secret scan (slice S14): every unique blob reachable in the scope is
//! read once and handed to the pure scanner in
//! `gitpoulpi_core::domain::rewrite::secrets`.
//!
//! The answer per tree is memoised by tree id, and holds only the files
//! that have a finding, so a commit costs one lookup however big its tree:
//! the work is proportional to the unique trees and blobs, not to the
//! commits times the files. First commit, last commit, commit count and
//! blob count of each finding come out of that.
//!
//! **Values stay here.** The public answer is `SecretScan` (previews and
//! counts); the values live in [`Scanned::values`] for the one caller that
//! builds a replacement from finding ids (`resolve`, used by the preview),
//! and are dropped with it.

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::rc::Rc;

use gitpoulpi_core::domain::rewrite::secrets::{
    is_lockfile, preview, scan_text, SecretValue, MAX_SCAN_BLOB,
};
use gitpoulpi_core::domain::rewrite::{
    ContentFind, Recipe, RewriteProgress, RewriteProgressSink, RewriteRefusal, RuleKind, Scope,
    ScopeOptions, SecretFinding, SecretScan,
};
use gitpoulpi_core::domain::{Oid, RepoRef};
use gitpoulpi_core::{Error, Result};
use sha1::{Digest, Sha1};

use super::blobs::is_binary;
use super::objects::Commit;
use super::plumbing::parse_tree;
use super::refs;
use super::walk::{self, refuse, Reader};
use gitpoulpi_core::domain::rewrite::RewritePhase;

/// The id of a secret: `f-` and twelve hex digits of a hash of its value.
/// Stable between scans. It is a hash of the value, so it is as sensitive as
/// the history it came from (which holds the value) and no more.
pub(super) fn finding_id(value: &SecretValue) -> String {
    let mut hasher = Sha1::new();
    hasher.update(b"gitpoulpi-secret\0");
    hasher.update(value.reveal());
    let sum: [u8; 20] = hasher.finalize().into();
    format!("f-{}", &Oid(sum).to_hex()[..12])
}

type Tick<'a> = &'a mut dyn FnMut(RewritePhase, u64, u64) -> Result<()>;

/// A scan's findings and, for the backend only, their values by id.
pub(super) struct Scanned {
    pub(super) scan: SecretScan,
    pub(super) values: HashMap<String, SecretValue>,
}

/// One file of a tree that holds a finding.
struct Hit {
    path: String,
    blob: Oid,
    finding: usize,
}

struct Acc {
    id: String,
    rule: &'static str,
    value: SecretValue,
    path: String,
    paths: HashSet<String>,
    first: usize,
    last: usize,
    last_seen: usize,
    commits: u64,
    blobs: HashSet<Oid>,
}

struct Scanner<'a> {
    reader: Reader,
    tick: Tick<'a>,
    blobs: HashMap<(Oid, bool), Rc<Vec<usize>>>,
    trees: HashMap<Oid, Rc<Vec<Hit>>>,
    finds: Vec<Acc>,
    by_id: HashMap<String, usize>,
    scanned: u64,
    skipped_binary: u64,
    skipped_large: u64,
}

impl Scanner<'_> {
    /// The findings of blob `oid` as indices into `finds`.
    fn blob(&mut self, oid: Oid, path: &str) -> Result<Rc<Vec<usize>>> {
        let lock = is_lockfile(path);
        if let Some(found) = self.blobs.get(&(oid, lock)) {
            return Ok(found.clone());
        }
        let found = match self.reader.blob(oid, MAX_SCAN_BLOB)? {
            (_, None) => {
                self.skipped_large += 1;
                Vec::new()
            }
            (_, Some(body)) if is_binary(&body) => {
                self.skipped_binary += 1;
                Vec::new()
            }
            (_, Some(body)) => {
                self.scanned += 1;
                let mut indices = Vec::new();
                for m in scan_text(&body, path) {
                    let id = finding_id(&m.value);
                    let at = match self.by_id.get(&id) {
                        Some(i) => *i,
                        None => {
                            let i = self.finds.len();
                            self.by_id.insert(id.clone(), i);
                            self.finds.push(Acc {
                                id,
                                rule: m.rule,
                                value: m.value,
                                path: String::new(),
                                paths: HashSet::new(),
                                first: usize::MAX,
                                last: 0,
                                last_seen: usize::MAX,
                                commits: 0,
                                blobs: HashSet::new(),
                            });
                            i
                        }
                    };
                    if !indices.contains(&at) {
                        indices.push(at);
                    }
                }
                indices
            }
        };
        let found = Rc::new(found);
        self.blobs.insert((oid, lock), found.clone());
        Ok(found)
    }

    /// The files of tree `oid` that hold a finding, paths relative to it.
    fn tree(&mut self, oid: Oid) -> Result<Rc<Vec<Hit>>> {
        if let Some(hits) = self.trees.get(&oid) {
            return Ok(hits.clone());
        }
        let (kind, body) = self.reader.get(oid)?;
        let entries = (kind == "tree")
            .then(|| parse_tree(&body))
            .flatten()
            .ok_or_else(|| Error::Backend(format!("{} is not a readable tree", oid.to_hex())))?;
        let mut hits = Vec::new();
        for e in &entries {
            let name = String::from_utf8_lossy(&e.name).into_owned();
            if e.is_tree() {
                for h in self.tree(e.oid)?.iter() {
                    hits.push(Hit {
                        path: format!("{name}/{}", h.path),
                        blob: h.blob,
                        finding: h.finding,
                    });
                }
            } else if e.mode == b"100644" || e.mode == b"100755" {
                for finding in self.blob(e.oid, &name)?.iter() {
                    hits.push(Hit {
                        path: name.clone(),
                        blob: e.oid,
                        finding: *finding,
                    });
                }
            }
        }
        let hits = Rc::new(hits);
        self.trees.insert(oid, hits.clone());
        Ok(hits)
    }
}

/// Scans the history `scope` walks.
pub(super) fn run(
    root: &Path,
    scope: &Scope,
    options: &ScopeOptions,
    tick: Tick<'_>,
) -> Result<Scanned> {
    tick(RewritePhase::Walking, 0, 0)?;
    let head = walk::head_of(root)?;
    let all_refs = refs::list(root)?;
    let resolved = refs::resolve(scope, options, &all_refs, &head)?;
    let order = walk::rev_list(root, &resolved.tips, &resolved.not)?;
    if order.is_empty() {
        return Err(refuse(
            RewriteRefusal::EmptyScope,
            "the scope has no commit",
        ));
    }
    let total = order.len() as u64;
    let mut reader = Reader::new(root)?;
    let raw = reader.commits(&order, |i| tick(RewritePhase::Walking, i, total))?;
    let mut trees = Vec::with_capacity(raw.len());
    for bytes in raw {
        trees.push(Commit::parse(bytes)?.tree());
    }

    let mut s = Scanner {
        reader,
        tick,
        blobs: HashMap::new(),
        trees: HashMap::new(),
        finds: Vec::new(),
        by_id: HashMap::new(),
        scanned: 0,
        skipped_binary: 0,
        skipped_large: 0,
    };
    for (k, tree) in trees.iter().enumerate() {
        if k % 64 == 0 {
            (s.tick)(RewritePhase::Rewriting, k as u64, total)?;
        }
        let Some(tree) = tree else { continue };
        let hits = s.tree(*tree)?;
        for h in hits.iter() {
            let f = &mut s.finds[h.finding];
            if f.first == usize::MAX {
                f.path = h.path.clone();
            }
            if f.last_seen != k {
                f.last_seen = k;
                f.commits += 1;
                f.first = f.first.min(k);
                f.last = k;
            }
            f.blobs.insert(h.blob);
            if f.paths.len() < 10_000 {
                f.paths.insert(h.path.clone());
            }
        }
    }
    (s.tick)(RewritePhase::Rewriting, total, total)?;

    let mut values = HashMap::new();
    let mut findings: Vec<SecretFinding> = Vec::new();
    let mut accs: Vec<&Acc> = s.finds.iter().filter(|f| f.commits > 0).collect();
    accs.sort_by(|a, b| (a.first, &a.path, &a.id).cmp(&(b.first, &b.path, &b.id)));
    for f in accs {
        findings.push(SecretFinding {
            id: f.id.clone(),
            rule: f.rule.to_string(),
            path: f.path.clone(),
            more_paths: f.paths.len().saturating_sub(1) as u32,
            first_commit: order[f.first],
            last_commit: order[f.last],
            commits: f.commits,
            blobs: f.blobs.len() as u64,
            preview: preview(f.rule, &f.value),
        });
        values.insert(f.id.clone(), f.value.clone());
    }
    Ok(Scanned {
        scan: SecretScan {
            findings,
            commits: total,
            blobs_scanned: s.scanned,
            blobs_skipped_binary: s.skipped_binary,
            blobs_skipped_large: s.skipped_large,
        },
        values,
    })
}

fn canceled() -> Error {
    // The daemon puts its job number on it.
    Error::Canceled { job: 0 }
}

/// `rewrite_scan_secrets`: the findings, never a value.
pub(crate) fn scan(
    repo: &RepoRef,
    scope: &Scope,
    sink: RewriteProgressSink<'_>,
) -> Result<SecretScan> {
    let mut tick = |phase, done, total| match sink(RewriteProgress { phase, done, total }) {
        std::ops::ControlFlow::Continue(()) => Ok(()),
        std::ops::ControlFlow::Break(()) => Err(canceled()),
    };
    Ok(run(&repo.root, scope, &ScopeOptions::default(), &mut tick)?.scan)
}

/// What the recipe's scanning secret rules replace, by rule index: the
/// values of the findings they name (all of them when they name none). Empty
/// without such a rule, and then nothing is scanned.
pub(super) fn resolve(
    root: &Path,
    recipe: &Recipe,
    tick: Tick<'_>,
) -> Result<HashMap<usize, Vec<Vec<u8>>>> {
    let wants = |kind: &RuleKind| {
        matches!(
            kind,
            RuleKind::ContentReplace(c) if matches!(c.find, ContentFind::Secrets { scan: true, .. })
        )
    };
    if !recipe.enabled_rules().any(|(_, r)| wants(&r.kind)) {
        return Ok(HashMap::new());
    }
    let scanned = run(root, &recipe.scope, &recipe.scope_options, tick)?;
    let mut out = HashMap::new();
    for (index, rule) in recipe.enabled_rules() {
        let RuleKind::ContentReplace(c) = &rule.kind else {
            continue;
        };
        let ContentFind::Secrets {
            literals,
            scan: true,
        } = &c.find
        else {
            continue;
        };
        let ids: Vec<&str> = literals.iter().filter_map(|l| l.finding_id()).collect();
        let values: Vec<Vec<u8>> = if ids.is_empty() {
            scanned
                .scan
                .findings
                .iter()
                .filter_map(|f| scanned.values.get(&f.id))
                .map(|v| v.reveal().to_vec())
                .collect()
        } else {
            let mut picked = Vec::new();
            for id in ids {
                match scanned.values.get(id) {
                    Some(v) => picked.push(v.reveal().to_vec()),
                    None => {
                        return Err(Error::RewriteRefused(
                            RewriteRefusal::InvalidRule,
                            format!("finding {id} is not in the history any more: scan again"),
                        ))
                    }
                }
            }
            picked
        };
        out.insert(index, values);
    }
    Ok(out)
}
