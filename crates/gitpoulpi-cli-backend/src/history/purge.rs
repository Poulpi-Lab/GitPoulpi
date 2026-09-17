//! Purge (the history-rewrite design, "Purge (S15)"): makes the
//! old objects of rewrite journal entries unreachable and then gone.
//!
//! What is "the old history": every object reachable from the entries' old
//! ref values and from none of their new ones (`T`). It is computed with two
//! full `rev-list --objects` listings and a set difference (the one-command
//! `A --not B` form does not exclude the blobs of B's older commits).
//!
//! **Nothing is changed while anything still reaches `T`.** A candidate that
//! can keep it alive (every ref but the pins being dropped, the stash list,
//! detached worktree HEADs) is checked with `rev-list`; a hit is a typed
//! blocker and the call answers `Blocked` untouched, because dropping the undo
//! pins of an entry whose objects stay would lose undo for nothing.
//!
//! The run: delete the pins, expire the reflogs that name `T`, remove the
//! applied runs' folders, `git gc --prune=now`, then ask `cat-file` for every
//! object of `T`.
//!
//! **Reflogs, conservatively.** `git reflog expire --expire=never
//! --expire-unreachable=<future>` on a ref drops exactly the entries whose old
//! *or* new id is no longer reachable from the ref's tip (the rewrite's own
//! entry has the old tip as its old id, so it goes too) and keeps every other
//! entry. It runs only on the reflogs of the refs the entries moved, HEAD when
//! its branch moved, and any reflog with an entry that names an object of `T`.
//! `refs/stash` is never expired: its reflog *is* the stash list.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use gitpoulpi_core::domain::rewrite::{
    PurgeBlocker, PurgeBlockerKind, PurgeOutcome, PurgeProgressSink, PurgeRequest, PurgeStatus,
    PurgeStep,
};
use gitpoulpi_core::domain::{Oid, RepoRef, UNDO_PIN_PREFIX};
use gitpoulpi_core::{Error, Result};

use super::store;
use super::walk::read_git;
use crate::process::{run, spawn_failed};

type Set = HashSet<Oid>;

/// Runs `git <args>` (replace refs ignored) with `input` on stdin and returns
/// stdout. stdin is written from its own thread, see `process::run_piped_stdin`.
fn piped(root: &Path, args: &[&str], input: String) -> Result<String> {
    let mut child = read_git(root, args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(spawn_failed)?;
    let mut pipe = child.stdin.take().expect("stdin was piped");
    let writer = std::thread::spawn(move || pipe.write_all(input.as_bytes()));
    let out = child
        .wait_with_output()
        .map_err(|e| Error::Backend(format!("failed to wait for git: {e}")))?;
    let _ = writer.join();
    if !out.status.success() {
        return Err(Error::Backend(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// `v` without repeats, in order.
fn unique(v: Vec<Oid>) -> Vec<Oid> {
    let mut seen = Set::new();
    v.into_iter().filter(|o| seen.insert(*o)).collect()
}

fn first_oid(line: &str) -> Option<Oid> {
    Oid::from_hex(line.get(..40)?)
}

fn lines_of(oids: impl IntoIterator<Item = Oid>) -> String {
    oids.into_iter()
        .map(|o| format!("{}\n", o.to_hex()))
        .collect()
}

/// The objects among `oids` that exist, with their type.
fn existing(root: &Path, oids: &[Oid]) -> Result<Vec<(Oid, String)>> {
    if oids.is_empty() {
        return Ok(Vec::new());
    }
    let out = piped(
        root,
        &["cat-file", "--batch-check=%(objectname) %(objecttype)"],
        lines_of(oids.iter().copied()),
    )?;
    Ok(out
        .lines()
        .filter_map(|l| {
            let oid = first_oid(l)?;
            let kind = l.get(41..)?.trim();
            (kind != "missing").then(|| (oid, kind.to_string()))
        })
        .collect())
}

/// Every object reachable from `tips`.
fn objects_of(root: &Path, tips: &[Oid]) -> Result<Set> {
    if tips.is_empty() {
        return Ok(Set::new());
    }
    let out = piped(
        root,
        &["rev-list", "--objects", "--stdin"],
        lines_of(tips.iter().copied()),
    )?;
    Ok(out.lines().filter_map(first_oid).collect())
}

/// The commits reachable from `tips` but not from `excluded`.
fn reach(root: &Path, tips: &[Oid], excluded: &[Oid]) -> Result<Vec<Oid>> {
    if tips.is_empty() {
        return Ok(Vec::new());
    }
    let mut input = lines_of(tips.iter().copied());
    for oid in excluded {
        input.push_str(&format!("^{}\n", oid.to_hex()));
    }
    let out = piped(root, &["rev-list", "--stdin"], input)?;
    Ok(out.lines().filter_map(first_oid).collect())
}

/// What can keep the old history alive.
struct Candidate {
    kind: PurgeBlockerKind,
    name: String,
    tip: Oid,
}

fn kind_of_ref(name: &str) -> PurgeBlockerKind {
    if name.starts_with("refs/heads/") {
        PurgeBlockerKind::Branch
    } else if name.starts_with("refs/tags/") {
        PurgeBlockerKind::Tag
    } else if name.starts_with("refs/remotes/") || name.starts_with("refs/gitpoulpi/remote-tags/") {
        PurgeBlockerKind::RemoteTracking
    } else if name.starts_with(UNDO_PIN_PREFIX) {
        PurgeBlockerKind::UndoPin
    } else if name == "refs/stash" {
        PurgeBlockerKind::Stash
    } else if name.starts_with("refs/notes/") {
        PurgeBlockerKind::Notes
    } else if name.starts_with("refs/replace/") {
        PurgeBlockerKind::Replace
    } else {
        PurgeBlockerKind::Other
    }
}

struct Worktree {
    path: PathBuf,
    head: Option<Oid>,
    detached: bool,
    bare: bool,
}

fn worktrees(root: &Path) -> Vec<Worktree> {
    let Ok(text) = run(root, &["worktree", "list", "--porcelain"]) else {
        return Vec::new();
    };
    let mut out: Vec<Worktree> = Vec::new();
    for line in text.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            out.push(Worktree {
                path: PathBuf::from(path),
                head: None,
                detached: false,
                bare: false,
            });
        } else if let Some(last) = out.last_mut() {
            if let Some(head) = line.strip_prefix("HEAD ") {
                last.head = Oid::from_hex(head.trim());
            } else if line == "detached" {
                last.detached = true;
            } else if line == "bare" {
                last.bare = true;
            }
        }
    }
    out
}

fn same_dir(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

fn candidates(root: &Path, dropped_pins: &HashSet<String>) -> Result<Vec<Candidate>> {
    let mut out = Vec::new();
    let refs = run(
        root,
        &[
            "for-each-ref",
            "--format=%(objectname) %(objecttype) %(refname)",
        ],
    )?;
    let mut have_stash = false;
    for line in refs.lines() {
        let mut parts = line.splitn(3, ' ');
        let (Some(oid), Some(kind), Some(name)) = (parts.next(), parts.next(), parts.next()) else {
            continue;
        };
        let Some(tip) = Oid::from_hex(oid) else {
            continue;
        };
        if matches!(kind, "tree" | "blob") || dropped_pins.contains(name) {
            continue;
        }
        if name == "refs/stash" {
            have_stash = true;
            continue;
        }
        out.push(Candidate {
            kind: kind_of_ref(name),
            name: name.to_string(),
            tip,
        });
    }
    if have_stash {
        // The stash list is the reflog of refs/stash: one entry per stash.
        let log = run(root, &["reflog", "show", "--format=%H", "refs/stash"])?;
        for (i, line) in log.lines().enumerate() {
            if let Some(tip) = first_oid(line) {
                out.push(Candidate {
                    kind: PurgeBlockerKind::Stash,
                    name: format!("stash@{{{i}}}"),
                    tip,
                });
            }
        }
    }
    for wt in worktrees(root) {
        if let (true, false, Some(tip)) = (wt.detached, wt.bare, wt.head) {
            out.push(Candidate {
                kind: PurgeBlockerKind::Worktree,
                name: wt.path.to_string_lossy().into_owned(),
                tip,
            });
        }
    }
    Ok(out)
}

/// What a reflog holds of the old history.
struct Reflog {
    /// Where to run `git reflog expire` (a linked worktree has its own HEAD
    /// log).
    dir: PathBuf,
    refname: String,
    display: String,
    entries: u32,
}

/// `refs/heads/main@{3}` to `refs/heads/main`.
fn ref_of_selector(selector: &str) -> &str {
    selector.rfind("@{").map_or(selector, |at| &selector[..at])
}

fn reflogs(
    root: &Path,
    old: &Set,
    touched: &HashSet<String>,
    head_branch: Option<&str>,
) -> Result<Vec<Reflog>> {
    let out = run(root, &["reflog", "show", "--all", "--format=%H %gD"])?;
    // name -> (entries naming the old history, entries)
    let mut seen: BTreeMap<String, (u32, u32)> = BTreeMap::new();
    for line in out.lines() {
        let Some(sha) = first_oid(line) else { continue };
        let name = ref_of_selector(line.get(41..).unwrap_or("").trim()).to_string();
        let slot = seen.entry(name).or_default();
        slot.1 += 1;
        if old.contains(&sha) {
            slot.0 += 1;
        }
    }
    let head_moved = head_branch.is_some_and(|b| touched.contains(b));
    let mut logs: Vec<Reflog> = Vec::new();
    for (name, (named, total)) in seen {
        // The stash list is the reflog of refs/stash; the other worktrees'
        // HEAD logs are handled one by one below.
        if name == "refs/stash"
            || name.starts_with(UNDO_PIN_PREFIX)
            || name.starts_with("worktrees/")
        {
            continue;
        }
        let moved = touched.contains(&name) || (name == "HEAD" && head_moved);
        if named == 0 && !moved {
            continue;
        }
        let _ = total;
        logs.push(Reflog {
            dir: root.to_path_buf(),
            display: name.clone(),
            refname: name,
            // The rewrite's own entry names the old tip as its old id.
            entries: named + u32::from(moved && named == 0),
        });
    }
    // The other worktrees' HEAD logs.
    for wt in worktrees(root) {
        if wt.bare || same_dir(&wt.path, root) {
            continue;
        }
        let Ok(text) = run(&wt.path, &["reflog", "show", "--format=%H", "HEAD"]) else {
            continue;
        };
        let named = text
            .lines()
            .filter_map(first_oid)
            .filter(|o| old.contains(o))
            .count() as u32;
        if named > 0 {
            logs.push(Reflog {
                dir: wt.path.clone(),
                refname: "HEAD".into(),
                display: format!("HEAD ({})", wt.path.display()),
                entries: named,
            });
        }
    }
    Ok(logs)
}

/// The applied runs whose `moved.json` lists one of the old tips.
fn run_dirs_of(git_dir: &Path, olds: &Set) -> Vec<PathBuf> {
    let dir = git_dir.join("gitpoulpi").join("rewrite");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|e| e.path())
        .filter(|path| {
            store::read_moved(path).is_some_and(|moved| moved.iter().any(|r| olds.contains(&r.old)))
        })
        .collect()
}

fn present(root: &Path, oids: &Set) -> Result<u32> {
    if oids.is_empty() {
        return Ok(0);
    }
    let out = piped(
        root,
        &["cat-file", "--batch-check=%(objectname) %(objecttype)"],
        lines_of(oids.iter().copied()),
    )?;
    Ok(out.lines().filter(|l| !l.ends_with(" missing")).count() as u32)
}

fn now_plus(secs: u64) -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
        + secs
}

pub(crate) fn purge(
    repo: &RepoRef,
    req: &PurgeRequest,
    progress: PurgeProgressSink<'_>,
) -> Result<PurgeOutcome> {
    let root = repo.root.as_path();
    let git_dir = super::git_dir(root)?;
    progress(PurgeStep::Checking);

    // The history to get rid of: reachable from an old value, from no new one.
    let mut old_tips: Vec<Oid> = Vec::new();
    let mut new_tips: Vec<Oid> = Vec::new();
    let mut touched: HashSet<String> = HashSet::new();
    for change in req.entries.iter().flat_map(|e| &e.refs) {
        if change.old != change.new {
            if let Some(old) = change.old {
                old_tips.push(old);
            }
            touched.insert(change.name.clone());
        }
        if let Some(new) = change.new {
            new_tips.push(new);
        }
    }
    let old_tips = unique(old_tips);
    // A value one entry made and another (selected) entry moved on from is
    // part of the old history too: only the values nothing here replaces stay.
    let new_tips = unique(new_tips)
        .into_iter()
        .filter(|n| !old_tips.contains(n))
        .collect::<Vec<_>>();
    let old_tips: Vec<Oid> = existing(root, &old_tips)?
        .into_iter()
        .map(|(o, _)| o)
        .collect();
    let new_tips: Vec<Oid> = existing(root, &new_tips)?
        .into_iter()
        .map(|(o, _)| o)
        .collect();
    let old_set: Set = old_tips.iter().copied().collect();

    let kept = objects_of(root, &new_tips)?;
    let target: Set = objects_of(root, &old_tips)?
        .into_iter()
        .filter(|o| !kept.contains(o))
        .collect();
    let types: HashMap<Oid, String> = {
        let all: Vec<Oid> = target.iter().copied().collect();
        existing(root, &all)?.into_iter().collect()
    };
    let target_commits: Set = types
        .iter()
        .filter(|(_, kind)| kind.as_str() == "commit")
        .map(|(o, _)| *o)
        .collect();
    let old_blobs = types.values().filter(|k| k.as_str() == "blob").count() as u32;

    // Who else keeps it alive.
    let dropped_pins: HashSet<String> = req
        .entries
        .iter()
        .flat_map(|e| e.pins.iter().cloned())
        .collect();
    let candidates = candidates(root, &dropped_pins)?;
    let mut blockers: Vec<PurgeBlocker> = Vec::new();
    if !target_commits.is_empty() {
        let tips = unique(candidates.iter().map(|c| c.tip).collect());
        let hit = reach(root, &tips, &new_tips)?
            .into_iter()
            .any(|c| target_commits.contains(&c));
        if hit {
            let mut cache: HashMap<Oid, u32> = HashMap::new();
            for c in &candidates {
                let n = match cache.get(&c.tip) {
                    Some(n) => *n,
                    None => {
                        let n = reach(root, &[c.tip], &new_tips)?
                            .into_iter()
                            .filter(|c| target_commits.contains(c))
                            .count() as u32;
                        cache.insert(c.tip, n);
                        n
                    }
                };
                if n > 0 {
                    blockers.push(PurgeBlocker {
                        kind: c.kind,
                        name: c.name.clone(),
                        commits: n,
                    });
                }
            }
        }
    }
    blockers.sort_by(|a, b| (a.kind as u8, &a.name).cmp(&(b.kind as u8, &b.name)));

    let head_branch = run(root, &["symbolic-ref", "-q", "HEAD"])
        .ok()
        .map(|s| s.trim().to_string());
    let logs = reflogs(root, &target, &touched, head_branch.as_deref())?;
    let runs = run_dirs_of(&git_dir, &old_set);
    let existing_pins: Vec<String> = {
        let listed = run(
            root,
            &["for-each-ref", "--format=%(refname)", UNDO_PIN_PREFIX],
        )?;
        listed
            .lines()
            .filter(|n| dropped_pins.contains(*n))
            .map(str::to_string)
            .collect()
    };

    let mut outcome = PurgeOutcome {
        status: if blockers.is_empty() {
            PurgeStatus::Ready
        } else {
            PurgeStatus::Blocked
        },
        blockers,
        old_commits: target_commits.len() as u32,
        old_blobs,
        old_objects: target.len() as u32,
        reflog_entries: logs.iter().map(|l| l.entries).sum(),
        reflogs: logs.iter().map(|l| l.display.clone()).collect(),
        pins_dropped: 0,
        runs_removed: 0,
        remaining: 0,
    };
    if req.dry_run || outcome.status == PurgeStatus::Blocked {
        outcome.pins_dropped = existing_pins.len() as u32;
        outcome.runs_removed = runs.len() as u32;
        return Ok(outcome);
    }

    // ---- from here on nothing is undone

    progress(PurgeStep::Pins);
    if !existing_pins.is_empty() {
        let script: String = existing_pins
            .iter()
            .map(|n| format!("delete {n}\n"))
            .collect();
        piped(root, &["update-ref", "--stdin"], script)?;
    }
    outcome.pins_dropped = existing_pins.len() as u32;

    progress(PurgeStep::Reflogs);
    let until = format!("--expire-unreachable={}", now_plus(3600));
    let mut by_dir: BTreeMap<PathBuf, Vec<&str>> = BTreeMap::new();
    for log in &logs {
        by_dir
            .entry(log.dir.clone())
            .or_default()
            .push(log.refname.as_str());
    }
    for (dir, names) in by_dir {
        let mut args = vec!["reflog", "expire", "--expire=never", until.as_str()];
        args.extend(names);
        run(&dir, &args)?;
    }
    for dir in &runs {
        store::remove_run(dir)?;
    }
    outcome.runs_removed = runs.len() as u32;

    progress(PurgeStep::Collecting);
    run(
        root,
        &[
            "-c",
            "gc.reflogExpire=never",
            "-c",
            "gc.reflogExpireUnreachable=never",
            "gc",
            "--prune=now",
            "--quiet",
        ],
    )?;

    progress(PurgeStep::Verifying);
    outcome.remaining = present(root, &target)?;
    outcome.status = if outcome.remaining == 0 {
        PurgeStatus::Purged
    } else {
        PurgeStatus::Incomplete
    };
    Ok(outcome)
}
