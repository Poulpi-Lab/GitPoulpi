//! Saved copies of a stopped operation's state, for the undo journal
//! (`<git-dir>/gitpoulpi/opstate/<name>/`): the index file, whose conflict
//! stages no tree can hold, and the state files git keeps while a merge,
//! cherry-pick or revert runs. Restoring one puts back exactly what was
//! saved, deleting the state files it did not have.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Single files git keeps in the git dir while an operation is stopped.
const STATE_FILES: &[&str] = &[
    "MERGE_HEAD",
    "MERGE_MSG",
    "MERGE_MODE",
    "AUTO_MERGE",
    "ORIG_HEAD",
    "CHERRY_PICK_HEAD",
    "REVERT_HEAD",
    "SQUASH_MSG",
];
/// The multi-pick / multi-revert queue.
const STATE_DIRS: &[&str] = &["sequencer"];

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn root(git_dir: &Path) -> PathBuf {
    git_dir.join("gitpoulpi").join("opstate")
}

/// A name no other saved state has.
pub fn fresh_name() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    format!(
        "{nanos:x}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    )
}

/// Saves the index and the state files under `name`.
pub fn save(git_dir: &Path, name: &str) -> std::io::Result<()> {
    let dir = root(git_dir).join(name);
    std::fs::create_dir_all(&dir)?;
    let result = (|| {
        let index = git_dir.join("index");
        if index.is_file() {
            std::fs::copy(&index, dir.join("index"))?;
        }
        for file in STATE_FILES {
            let from = git_dir.join(file);
            if from.is_file() {
                std::fs::copy(&from, dir.join(file))?;
            }
        }
        for sub in STATE_DIRS {
            let from = git_dir.join(sub);
            if from.is_dir() {
                copy_dir(&from, &dir.join(sub))?;
            }
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_dir_all(&dir);
    }
    result
}

/// Puts the state saved under `name` back.
pub fn restore(git_dir: &Path, name: &str) -> std::io::Result<()> {
    let dir = root(git_dir).join(name);
    if !dir.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "the saved state is gone",
        ));
    }
    for file in STATE_FILES {
        let saved = dir.join(file);
        let target = git_dir.join(file);
        if saved.is_file() {
            replace_file(&saved, &target)?;
        } else if target.exists() {
            std::fs::remove_file(&target)?;
        }
    }
    for sub in STATE_DIRS {
        let saved = dir.join(sub);
        let target = git_dir.join(sub);
        if target.exists() {
            std::fs::remove_dir_all(&target)?;
        }
        if saved.is_dir() {
            copy_dir(&saved, &target)?;
        }
    }
    // The index last: the operation's files are back by then.
    let saved = dir.join("index");
    if saved.is_file() {
        replace_file(&saved, &git_dir.join("index"))?;
    }
    Ok(())
}

/// Deletes the state saved under `name`.
pub fn remove(git_dir: &Path, name: &str) {
    let _ = std::fs::remove_dir_all(root(git_dir).join(name));
}

/// Copy beside the target, then rename over it: a reader never sees half
/// a file.
fn replace_file(from: &Path, to: &Path) -> std::io::Result<()> {
    let mut tmp = to.as_os_str().to_owned();
    tmp.push(".gitpoulpi-tmp");
    let tmp = PathBuf::from(tmp);
    std::fs::copy(from, &tmp)?;
    std::fs::rename(&tmp, to).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })
}

fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}
