//! Reading the reflog and finding lost commits over the system `git`
//! (the undo-safety-net design). The reflog is read from its
//! file (`git rev-parse --git-path logs/<ref>` names it, linked worktrees
//! included), so a page costs one process and the entries are exactly what
//! git stored, `old` included. A repository using the reftable format has
//! no such file (and libgit2 cannot open it): `git log -g` reads it, here and
//! for `Git2Backend`.

use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use gitpoulpi_core::domain::{
    reflog_refname, LostCommit, LostCommitOptions, LostCommitsPage, LostSource, Oid, ReflogEntry,
    ReflogPage, RepoRef, ZERO_OID,
};
use gitpoulpi_core::{Error, Result};

use crate::process::{run, run_bytes};
use crate::stream::{stream_lines, Streamed};

/// The raw bytes of blob `oid`.
pub fn blob_bytes(repo: &RepoRef, oid: Oid) -> Result<Vec<u8>> {
    run_bytes(&repo.root, &["cat-file", "blob", &oid.to_hex()])
}

/// One line of a reflog file: `<old> <new> <Name <email>> <time> <tz>\t<msg>`.
pub fn parse_reflog_line(line: &str) -> Option<ReflogEntry> {
    let (meta, message) = line.split_once('\t').unwrap_or((line, ""));
    let old = Oid::from_hex(meta.get(0..40)?)?;
    let new = Oid::from_hex(meta.get(41..81)?)?;
    let who = meta.get(82..)?;
    // `Name <email> 1700000000 +0200`: the last two words are time and zone.
    let mut tail = who.rsplitn(3, ' ');
    let _zone = tail.next()?;
    let time: i64 = tail.next()?.parse().ok()?;
    let committer = tail.next()?.to_string();
    Some(ReflogEntry {
        old,
        new,
        committer,
        time,
        message: message.trim_end_matches(['\r', '\n']).to_string(),
    })
}

/// The log file of `refname` (`HEAD` or a full ref name), if git has one.
fn log_file(repo: &RepoRef, refname: &str) -> Result<PathBuf> {
    let out = run(
        &repo.root,
        &["rev-parse", "--git-path", &format!("logs/{refname}")],
    )?;
    let path = PathBuf::from(out.trim());
    Ok(if path.is_absolute() {
        path
    } else {
        repo.root.join(path)
    })
}

/// The reflog through `git log -g`, for ref storage with no log files
/// (reftable). `old` is the next older entry's `new` (the first entry's is
/// `ZERO_OID`), so one extra entry is read to give the last one of the page
/// its `old` and to know whether there is `more`. A ref without a reflog (a
/// deleted branch, a tag, an unknown name) is an empty page.
pub fn reflog_via_git(
    repo: &RepoRef,
    refname: &str,
    skip: usize,
    limit: usize,
) -> Result<ReflogPage> {
    let refname = reflog_refname(refname);
    let want = limit.saturating_add(1).to_string();
    let skip_arg = format!("--skip={skip}");
    let out = run(
        &repo.root,
        &[
            "log",
            "-g",
            "--date=unix",
            "--format=%H%x1f%gd%x1f%gn <%ge>%x1f%gs",
            "-n",
            &want,
            &skip_arg,
            &refname,
            "--",
        ],
    )
    // `git log -g` of a name with no reflog is an error ("bad revision").
    .unwrap_or_default();
    let mut rows: Vec<(Oid, i64, String, String)> = Vec::new();
    for line in out.lines() {
        let mut f = line.split('\x1f');
        let (Some(new), Some(gd), Some(who), Some(msg)) = (f.next(), f.next(), f.next(), f.next())
        else {
            continue;
        };
        let (Some(new), Some(time)) = (
            Oid::from_hex(new),
            gd.rsplit_once("@{")
                .and_then(|(_, t)| t.strip_suffix('}'))
                .and_then(|t| t.parse::<i64>().ok()),
        ) else {
            continue;
        };
        rows.push((new, time, who.to_string(), msg.to_string()));
    }
    let more = rows.len() > limit;
    let mut entries = Vec::new();
    for (i, (new, time, committer, message)) in rows.iter().take(limit).enumerate() {
        entries.push(ReflogEntry {
            old: rows.get(i + 1).map_or(ZERO_OID, |r| r.0),
            new: *new,
            committer: committer.clone(),
            time: *time,
            message: message.clone(),
        });
    }
    Ok(ReflogPage {
        refname,
        entries,
        more,
    })
}

pub fn reflog(repo: &RepoRef, refname: &str, skip: usize, limit: usize) -> Result<ReflogPage> {
    if gitpoulpi_core::paths::uses_reftable(&repo.root) {
        return reflog_via_git(repo, refname, skip, limit);
    }
    let refname = reflog_refname(refname);
    let path = log_file(repo, &refname)?;
    let text = match std::fs::read(&path) {
        Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => {
            return Err(Error::Backend(format!(
                "cannot read the reflog {}: {e}",
                path.display()
            )))
        }
    };
    let mut entries: Vec<ReflogEntry> = text.lines().filter_map(parse_reflog_line).collect();
    entries.reverse();
    let more = entries.len() > skip.saturating_add(limit);
    let entries = entries.into_iter().skip(skip).take(limit).collect();
    Ok(ReflogPage {
        refname,
        entries,
        more,
    })
}

/// Everything reachable from a branch, tag, remote-tracking ref, stash or
/// HEAD counts as kept; the undo pins and the reflogs do not.
const NOT_KEPT: [&str; 3] = ["--not", "--exclude=refs/gitpoulpi/*", "--all"];
/// Ceiling on the commits the deep scan hands to git: past it the page is
/// `partial` (a repository with millions of commits).
const DEEP_CAP: usize = 2_000_000;

fn oids_of(
    repo: &RepoRef,
    positives: &[&str],
    deadline: Option<Instant>,
    partial: &mut bool,
) -> Result<HashSet<Oid>> {
    let mut args: Vec<&str> = vec!["rev-list"];
    args.extend(positives);
    args.extend(NOT_KEPT);
    let mut set = HashSet::new();
    let ended = stream_lines(&repo.root, &args, None, deadline, |line| {
        if let Some(oid) = Oid::from_hex(line.trim()) {
            set.insert(oid);
        }
        true
    })?;
    *partial |= ended == Streamed::TimedOut;
    Ok(set)
}

pub fn lost_commits(repo: &RepoRef, opts: &LostCommitOptions) -> Result<LostCommitsPage> {
    let deadline =
        (opts.budget_ms > 0).then(|| Instant::now() + Duration::from_millis(opts.budget_ms));
    let mut partial = false;
    let pins = "--glob=refs/gitpoulpi/undo/*";
    let reflogged = oids_of(repo, &["--reflog"], deadline, &mut partial)?;
    let pinned = oids_of(repo, &[pins], deadline, &mut partial)?;

    // Dangling commits: every commit in the object database, minus what the
    // listing below finds reachable. One `cat-file` pass, streamed.
    let mut stdin: Option<Vec<u8>> = None;
    if opts.deep {
        let mut all: Vec<u8> = Vec::new();
        let mut count = 0usize;
        let ended = stream_lines(
            &repo.root,
            &[
                "cat-file",
                "--batch-all-objects",
                "--unordered",
                "--batch-check=%(objecttype) %(objectname)",
            ],
            None,
            deadline,
            |line| {
                if let Some(hex) = line.strip_prefix("commit ") {
                    count += 1;
                    if count > DEEP_CAP {
                        partial = true;
                        return false;
                    }
                    all.extend_from_slice(hex.as_bytes());
                    all.push(b'\n');
                }
                true
            },
        )?;
        partial |= ended == Streamed::TimedOut;
        if !all.is_empty() {
            stdin = Some(all);
        }
    }

    let want = opts.skip.saturating_add(opts.limit).saturating_add(1);
    let want_text = want.to_string();
    let mut args: Vec<&str> = vec![
        "log",
        "--date-order",
        "--format=%H%x1f%P%x1f%an%x1f%at%x1f%s",
        "-n",
        &want_text,
        "--reflog",
        pins,
    ];
    if stdin.is_some() {
        args.push("--stdin");
    }
    args.extend(NOT_KEPT);
    let mut seen: HashSet<Oid> = HashSet::new();
    let mut commits: Vec<LostCommit> = Vec::new();
    let ended = stream_lines(&repo.root, &args, stdin, deadline, |line| {
        let mut f = line.split('\x1f');
        let (Some(h), Some(p), Some(author), Some(at), Some(summary)) =
            (f.next(), f.next(), f.next(), f.next(), f.next())
        else {
            return true;
        };
        let Some(oid) = Oid::from_hex(h) else {
            return true;
        };
        if !seen.insert(oid) {
            return true;
        }
        let source = if reflogged.contains(&oid) {
            LostSource::Reflog
        } else if pinned.contains(&oid) {
            LostSource::Pin
        } else {
            LostSource::Dangling
        };
        commits.push(LostCommit {
            oid,
            summary: summary.to_string(),
            author: author.to_string(),
            time: at.parse().unwrap_or(0),
            parents: p.split(' ').filter_map(Oid::from_hex).collect(),
            source,
        });
        true
    })?;
    partial |= ended == Streamed::TimedOut;
    let more = commits.len() > opts.skip.saturating_add(opts.limit);
    let commits = commits
        .into_iter()
        .skip(opts.skip)
        .take(opts.limit)
        .collect();
    Ok(LostCommitsPage {
        commits,
        more,
        partial,
    })
}
