//! Rename detection for the changes git does not pair itself: a tracked
//! file deleted in the working tree plus an untracked (or intent-to-add)
//! file with similar content is one move. Staged renames and commit diffs
//! are paired by git (or libgit2); this is the part of `status` that `git
//! status` leaves as a deletion plus a new file, shared by both backends so
//! they pair alike.
//!
//! No I/O here: the backends hand in a reader for the two sides.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::{FileStatus, StatusEntry};

/// git's default `-M` threshold: a pair is a rename from 50% similarity.
pub const RENAME_THRESHOLD: u8 = 50;
/// Like `diff.renameLimit`: with more deleted or more new files than this
/// on either side, no pairing is tried (the listing stays a deletion plus
/// additions) so a huge untracked set never costs a full comparison.
pub const RENAME_LIMIT: usize = 400;
/// A file larger than this is not compared (it stays a delete plus an add).
pub const RENAME_MAX_FILE_BYTES: u64 = 512 * 1024;
/// Comparison work (chunks looked at) one `status` may spend.
const RENAME_WORK_BUDGET: u64 = 20_000_000;

/// Which side of a pair a [`pair_unstaged_renames`] reader is asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenameSide {
    /// The tracked file's staged-or-committed content (the index).
    Old,
    /// The file on disk.
    New,
}

/// Similarity of two files in percent: the share of the larger file's
/// bytes that lies in lines (or 64-byte blocks of a long line) present in
/// both. 100 for identical content; empty files score 0 (git never calls
/// two empty files a rename).
pub fn similarity(old: &[u8], new: &[u8]) -> u8 {
    similarity_of(&chunks(old), old.len(), &chunks(new), new.len()).0
}

type Chunks = HashMap<u64, (u64, u32)>;

fn chunks(data: &[u8]) -> Chunks {
    let mut map: Chunks = HashMap::new();
    let mut start = 0;
    let mut add = |piece: &[u8]| {
        // FNV-1a: stable across runs and platforms (no random state).
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for b in piece {
            h ^= u64::from(*b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        let e = map.entry(h).or_insert((piece.len() as u64, 0));
        e.1 += 1;
    };
    for (i, b) in data.iter().enumerate() {
        if *b == b'\n' || i + 1 - start >= 64 {
            add(&data[start..=i]);
            start = i + 1;
        }
    }
    if start < data.len() {
        add(&data[start..]);
    }
    map
}

/// `(percent, comparison work)`.
fn similarity_of(a: &Chunks, a_len: usize, b: &Chunks, b_len: usize) -> (u8, u64) {
    let max = a_len.max(b_len) as u64;
    if a_len == 0 || b_len == 0 || max == 0 {
        return (0, 0);
    }
    let (small, large) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    let mut common = 0u64;
    for (h, (len, n)) in small {
        if let Some((_, m)) = large.get(h) {
            common += len * u64::from((*n).min(*m));
        }
    }
    (((common * 100) / max).min(100) as u8, small.len() as u64)
}

/// The bytes of the regular file `rel` under `root`, for the `New` side of
/// a pair: `None` for a symlink, a directory, a missing or an oversized file.
pub fn read_workdir_candidate(root: &Path, rel: &Path) -> Option<Vec<u8>> {
    let full = root.join(rel);
    let meta = std::fs::symlink_metadata(&full).ok()?;
    if !meta.is_file() || meta.len() > RENAME_MAX_FILE_BYTES {
        return None;
    }
    std::fs::read(full).ok()
}

/// `true` for the entries [`pair_unstaged_renames`] takes as the old side.
pub fn is_unstaged_rename_source(e: &StatusEntry) -> bool {
    is_candidate_old(e)
}

type Loaded = Option<(Chunks, usize)>;

fn is_candidate_old(e: &StatusEntry) -> bool {
    e.index.is_none() && e.worktree == Some(FileStatus::Deleted) && e.renamed_from.is_none()
}

fn is_candidate_new(e: &StatusEntry) -> bool {
    e.index.is_none()
        && matches!(
            e.worktree,
            Some(FileStatus::Untracked) | Some(FileStatus::Added)
        )
        && e.renamed_from.is_none()
}

/// Pairs the unstaged deletions with the new files of `entries`: a new
/// entry (untracked, or intent-to-add, no staged part) whose content is at
/// least [`RENAME_THRESHOLD`] percent like a deleted tracked entry's gets
/// `renamed_from` set to it. The best pairs win, each file is used once, and
/// `entries` keep their statuses and order: the deleted file stays listed
/// (the lists show the pair as one row), so nothing that works from the
/// raw statuses changes.
///
/// `read` answers a path's content or `None` (unreadable): no pair for that
/// file. Returns the similarity of each pair made, by new path.
pub fn pair_unstaged_renames(
    entries: &mut [StatusEntry],
    mut read: impl FnMut(&Path, RenameSide) -> Option<Vec<u8>>,
) -> HashMap<PathBuf, u8> {
    let mut made = HashMap::new();
    let olds: Vec<usize> = (0..entries.len())
        .filter(|&i| is_candidate_old(&entries[i]))
        .collect();
    let news: Vec<usize> = (0..entries.len())
        .filter(|&i| is_candidate_new(&entries[i]))
        .collect();
    if olds.is_empty() || news.is_empty() || olds.len() > RENAME_LIMIT || news.len() > RENAME_LIMIT
    {
        return made;
    }
    let mut load = |path: &Path, side: RenameSide| -> Loaded {
        read(path, side)
            .filter(|b| !b.is_empty() && b.len() as u64 <= RENAME_MAX_FILE_BYTES)
            .map(|b| (chunks(&b), b.len()))
    };
    let old_data: Vec<Loaded> = olds
        .iter()
        .map(|&i| load(&entries[i].path, RenameSide::Old))
        .collect();
    let new_data: Vec<Loaded> = news
        .iter()
        .map(|&i| load(&entries[i].path, RenameSide::New))
        .collect();

    let mut scored: Vec<(u8, usize, usize)> = Vec::new();
    let mut work = 0u64;
    'outer: for (oi, o) in old_data.iter().enumerate() {
        let Some((oc, olen)) = o else { continue };
        for (ni, n) in new_data.iter().enumerate() {
            let Some((nc, nlen)) = n else { continue };
            // Below half the larger size the score cannot reach 50%.
            if (*olen).min(*nlen) * 2 < (*olen).max(*nlen) {
                continue;
            }
            let (score, w) = similarity_of(oc, *olen, nc, *nlen);
            work += w + 1;
            if score >= RENAME_THRESHOLD {
                scored.push((score, oi, ni));
            }
            if work > RENAME_WORK_BUDGET {
                break 'outer;
            }
        }
    }
    // Best first; ties by path so both backends choose alike.
    scored.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| entries[news[a.2]].path.cmp(&entries[news[b.2]].path))
            .then_with(|| entries[olds[a.1]].path.cmp(&entries[olds[b.1]].path))
    });
    let mut old_used = vec![false; olds.len()];
    let mut new_used = vec![false; news.len()];
    for (score, oi, ni) in scored {
        if old_used[oi] || new_used[ni] {
            continue;
        }
        old_used[oi] = true;
        new_used[ni] = true;
        let from = entries[olds[oi]].path.clone();
        let to = news[ni];
        entries[to].renamed_from = Some(from);
        made.insert(entries[to].path.clone(), score);
    }
    made
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(path: &str, worktree: FileStatus) -> StatusEntry {
        StatusEntry {
            path: path.into(),
            index: None,
            worktree: Some(worktree),
            renamed_from: None,
        }
    }

    #[test]
    fn identical_is_100_and_unrelated_is_low() {
        assert_eq!(similarity(b"a\nb\nc\n", b"a\nb\nc\n"), 100);
        assert_eq!(similarity(b"a\nb\nc\n", b"x\ny\nz\n"), 0);
        assert_eq!(similarity(b"", b""), 0);
    }

    #[test]
    fn a_small_edit_stays_above_the_threshold() {
        let old = "one\ntwo\nthree\nfour\n";
        let new = "one\ntwo\nthree\nFOUR\n";
        assert!(similarity(old.as_bytes(), new.as_bytes()) >= RENAME_THRESHOLD);
    }

    #[test]
    fn pairs_the_best_match_once() {
        let mut entries = vec![
            entry("old_a.txt", FileStatus::Deleted),
            entry("old_b.txt", FileStatus::Deleted),
            entry("new_a.txt", FileStatus::Untracked),
            entry("new_c.txt", FileStatus::Untracked),
        ];
        let paired = pair_unstaged_renames(&mut entries, |p, _| {
            Some(
                match p.to_str().unwrap() {
                    "old_a.txt" | "new_a.txt" => "a1\na2\na3\na4\n",
                    "old_b.txt" => "b1\nb2\nb3\nb4\n",
                    _ => "zzz\nyyy\nxxx\n",
                }
                .as_bytes()
                .to_vec(),
            )
        });
        assert_eq!(paired.len(), 1);
        assert_eq!(entries[2].renamed_from, Some(PathBuf::from("old_a.txt")));
        assert_eq!(entries[3].renamed_from, None);
    }
}
