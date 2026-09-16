//! The size analysis (slice S18): what weighs in a repository's history, in
//! the spirit of `git filter-repo --analyze`. Pure data; the walk lives in
//! the backends (`GitBackend::rewrite_analyze`).
//!
//! Sizes are those of the unique blobs (an identical file at ten paths or in
//! ten commits counts once). Any "frees" figure is an **estimate**: the space
//! comes back only after the rewrite, a purge of the old objects and a `gc`.

use serde::{Deserialize, Serialize};

use crate::domain::Oid;

/// How many rows each list keeps unless the caller asks for another number.
pub const DEFAULT_TOP: u32 = 50;
/// The most rows a caller can ask for.
pub const MAX_TOP: u32 = 5000;

/// Where a blob stands today.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlobState {
    /// This very content is in HEAD.
    InHead,
    /// Its path is in HEAD, with other content: an old version.
    Replaced,
    /// No path of this blob is in HEAD any more.
    Deleted,
}

/// One of the largest blobs of the history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlobRow {
    /// A path the blob had (the first one seen).
    pub path: String,
    /// Other paths that held the same content.
    pub more_paths: u32,
    pub blob: Oid,
    /// Uncompressed bytes.
    pub size: u64,
    /// Bytes in the object database (a packed delta counts as the delta), a
    /// rough figure of what it costs on disk.
    pub disk_size: u64,
    /// The commit that introduced it (the oldest in the walk).
    pub first_commit: Oid,
    /// The last commit that touched it: the one that removed it, else the one
    /// that introduced it.
    pub last_commit: Oid,
    pub state: BlobState,
}

/// A folder and the unique blobs ever stored below it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FolderRow {
    pub path: String,
    pub size: u64,
    pub blobs: u64,
    /// Whether HEAD still has files in it.
    pub in_head: bool,
}

/// A path that is not in HEAD any more but is still in the history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeletedRow {
    pub path: String,
    /// Unique blobs it ever held.
    pub versions: u64,
    /// Their sizes added up.
    pub size: u64,
    /// The commit that first added it and the last one that touched it.
    pub first_commit: Oid,
    pub last_commit: Oid,
}

/// The answer of `rewrite_analyze`: capped lists and the totals they come from.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SizeAnalysis {
    /// Largest first.
    pub blobs: Vec<BlobRow>,
    /// Largest first.
    pub folders: Vec<FolderRow>,
    /// Largest first.
    pub deleted: Vec<DeletedRow>,
    /// Rows asked for in each list.
    pub top: u32,
    pub commits: u64,
    /// Unique blobs seen in the scope's history.
    pub blobs_total: u64,
    /// Their sizes added up, uncompressed.
    pub bytes_total: u64,
    /// Their sizes in the object database.
    pub disk_total: u64,
    /// Unique bytes of the files HEAD has now (what stays whatever you remove).
    pub bytes_in_head: u64,
    /// Unique bytes held only by history: removing every deleted path and old
    /// version would free about this much.
    pub bytes_history_only: u64,
    /// Paths missing from HEAD, over all (the `deleted` list is capped).
    pub deleted_paths: u64,
    /// Rows cut from each list by the cap.
    pub blobs_cut: u64,
    pub folders_cut: u64,
    pub deleted_cut: u64,
}

/// A glob that matches exactly `path` (and what is below it): the glob
/// characters in the name are escaped with a class, and a name without a
/// slash gets a leading one so it is anchored to the root.
#[must_use]
pub fn glob_for(path: &str) -> String {
    let mut out = String::with_capacity(path.len() + 3);
    if !path.contains('/') {
        out.push('/');
    }
    for c in path.chars() {
        match c {
            '*' | '?' | '[' => {
                out.push('[');
                out.push(c);
                out.push(']');
            }
            c => out.push(c),
        }
    }
    out
}

/// A size as people read it, 1024-based (the rule forms parse `MiB`).
#[must_use]
pub fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if value >= 100.0 {
        format!("{value:.0} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::rewrite::GlobSet;

    fn set(p: &str) -> GlobSet {
        GlobSet::new(&[glob_for(p)]).unwrap()
    }

    #[test]
    fn a_glob_for_a_path_matches_that_path_only() {
        let g = set("big.bin");
        assert!(g.is_match("big.bin") && !g.is_match("sub/big.bin"));
        let g = set("data/a[1].bin");
        assert!(g.is_match("data/a[1].bin") && !g.is_match("data/a1.bin"));
        let g = set("data/dump*.sql");
        assert!(g.is_match("data/dump*.sql") && !g.is_match("data/dump1.sql"));
        let g = set("data");
        assert!(g.is_match("data/x/y.bin"));
    }

    #[test]
    fn sizes_read_as_people_expect() {
        assert_eq!(human_size(10), "10 B");
        assert_eq!(human_size(1536), "1.5 KiB");
        assert_eq!(human_size(5 * 1024 * 1024), "5.0 MiB");
        assert_eq!(human_size(300 * 1024 * 1024), "300 MiB");
    }
}
