//! Whether the index holds unmerged (conflict-stage) entries, read straight
//! from `.git/index` so "unstage all" stays at one `git` spawn when there is
//! no conflict (the common case). Index format:
//! <https://git-scm.com/docs/index-format>.

use std::path::Path;

/// `Some(true)` when an entry has a non-zero stage, `Some(false)` when none
/// does, `None` when this reader cannot tell (no index, index v4, a split
/// index, `GIT_INDEX_FILE`, anything unexpected): the caller asks git then.
pub fn has_unmerged(git_dir: &Path) -> Option<bool> {
    if std::env::var_os("GIT_INDEX_FILE").is_some() {
        return None;
    }
    scan(&std::fs::read(git_dir.join("index")).ok()?)
}

fn be32(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_be_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

fn scan(b: &[u8]) -> Option<bool> {
    if b.get(..4)? != b"DIRC" {
        return None;
    }
    let version = be32(b, 4)?;
    // v4 prefix-compresses names (no padding): not worth decoding here.
    if !(2..=3).contains(&version) {
        return None;
    }
    let count = be32(b, 8)? as usize;
    // Entry: ctime, mtime, dev, ino, mode, uid, gid, size (40 bytes), the
    // object id (20 bytes, SHA-1 -- a SHA-256 repository gives up below),
    // then 16-bit flags whose bits 12-13 are the stage.
    const FIXED: usize = 62;
    let mut pos = 12;
    let mut unmerged = false;
    for _ in 0..count {
        let flags = u16::from_be_bytes(b.get(pos + 60..pos + 62)?.try_into().ok()?);
        if (flags >> 12) & 0x3 != 0 {
            unmerged = true;
        }
        let extended = version >= 3 && flags & 0x4000 != 0;
        let fixed = FIXED + if extended { 2 } else { 0 };
        let name_start = pos + fixed;
        let name_len = match (flags & 0x0fff) as usize {
            0x0fff => b.get(name_start..)?.iter().position(|&c| c == 0)?,
            n => n,
        };
        if *b.get(name_start + name_len)? != 0 {
            // Not a NUL after the name: not SHA-1, or not an index we read.
            return None;
        }
        // The entry is padded with 1-8 NULs to a multiple of 8 bytes.
        pos += (fixed + name_len + 8) & !7;
    }
    // A split index keeps entries in a shared file this reader never opens.
    let end = b.len().checked_sub(20)?;
    while pos + 8 <= end {
        let sig = b.get(pos..pos + 4)?;
        if sig == b"link" {
            return None;
        }
        if !sig.iter().all(u8::is_ascii_alphabetic) {
            break;
        }
        pos += 8 + be32(b, pos + 4)? as usize;
    }
    Some(unmerged)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn git(dir: &Path, args: &[&str]) -> bool {
        Command::new("git")
            .current_dir(dir)
            .args([
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(args)
            .output()
            .expect("git runs")
            .status
            .success()
    }

    /// A repository on `main` with `feature` branched off and both
    /// changing `f.txt`; `long` adds a file whose path is over 4095 bytes.
    fn repo(long: bool) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let d = dir.path();
        assert!(git(d, &["init", "-q", "-b", "main"]));
        std::fs::write(d.join("f.txt"), "base\n").unwrap();
        std::fs::write(d.join("other.txt"), "x\n").unwrap();
        assert!(git(d, &["add", "-A"]));
        if long {
            // Too long for the file system (PATH_MAX): an index-only entry.
            let deep = (0..45)
                .map(|_| "d".repeat(100))
                .collect::<Vec<_>>()
                .join("/");
            let blob = Command::new("git")
                .current_dir(d)
                .args(["hash-object", "-w", "other.txt"])
                .output()
                .unwrap();
            let blob = String::from_utf8(blob.stdout).unwrap();
            let info = format!("100644,{},{deep}/leaf.txt", blob.trim());
            assert!(git(d, &["update-index", "--add", "--cacheinfo", &info]));
        }
        assert!(git(d, &["commit", "-qm", "base"]));
        assert!(git(d, &["checkout", "-qb", "feature"]));
        std::fs::write(d.join("f.txt"), "theirs\n").unwrap();
        assert!(git(d, &["commit", "-qam", "theirs"]));
        assert!(git(d, &["checkout", "-q", "main"]));
        std::fs::write(d.join("f.txt"), "ours\n").unwrap();
        assert!(git(d, &["commit", "-qam", "ours"]));
        dir
    }

    #[test]
    fn a_clean_index_has_no_unmerged_entry() {
        let dir = repo(false);
        assert_eq!(has_unmerged(&dir.path().join(".git")), Some(false));
    }

    #[test]
    fn a_stopped_merge_has_unmerged_entries_and_resolving_clears_them() {
        let dir = repo(false);
        let d = dir.path();
        assert!(!git(d, &["merge", "-q", "feature"]), "the merge conflicts");
        assert_eq!(has_unmerged(&d.join(".git")), Some(true));
        assert!(git(d, &["add", "f.txt"]));
        assert_eq!(has_unmerged(&d.join(".git")), Some(false));
    }

    // The entry is index-only because the path is over PATH_MAX, and Windows
    // cannot commit it. The parser reads the same bytes on every OS, so the
    // unix run covers it.
    #[cfg_attr(
        windows,
        ignore = "an over-long path cannot be committed on Windows; the parser is OS-independent"
    )]
    #[test]
    fn a_name_longer_than_the_flags_field_is_still_read() {
        let dir = repo(true);
        let d = dir.path();
        assert_eq!(has_unmerged(&d.join(".git")), Some(false));
        assert!(!git(d, &["merge", "-q", "feature"]));
        assert_eq!(has_unmerged(&d.join(".git")), Some(true));
    }

    #[test]
    fn index_v4_and_no_index_are_left_to_git() {
        let dir = repo(false);
        let d = dir.path();
        assert!(git(d, &["update-index", "--index-version", "4"]));
        assert_eq!(has_unmerged(&d.join(".git")), None);
        assert_eq!(has_unmerged(&d.join("nowhere")), None);
    }
}
