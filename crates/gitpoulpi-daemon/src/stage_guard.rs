//! The sensitive staging guard: `StageFile`, `StageFolder` and `StageAll`
//! refuse, with `TypedError::SensitiveStage`, to put a file in the index that
//! looks like a secret unless the request says `allow_sensitive`.
//!
//! The guard collects the paths the request would stage (the target, or the
//! changed paths of `backend.status()` under it), reads their working-tree
//! content and asks `gitpoulpi_core::privacy::classify`. A path is judged by
//! its name whatever its size; its content is scanned only when it is a
//! regular text file of at most [`MAX_FILE`] bytes and the files read so far
//! sum to at most [`MAX_TOTAL`]. A value found never leaves `classify`: the
//! refusal carries a rule and a masked preview.
//!
//! `[privacy] staging_guard` and `sensitive_patterns` are read the way the
//! undo settings are: kept until the config file's stamp moves.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

use gitpoulpi_core::domain::{FileStatus, OperationKind, RepoRef, StageTarget};
use gitpoulpi_core::privacy::{classify, Sensitive};
use gitpoulpi_core::{Error, GitBackend};
use serde_json::Value;

use crate::dispatch::Failure;

/// The most of one file's content that is scanned.
pub const MAX_FILE: u64 = 1 << 20;
/// The most content scanned for one request.
pub const MAX_TOTAL: u64 = 32 << 20;

/// `[privacy]` as the guard reads it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuardSettings {
    pub enabled: bool,
    pub patterns: Vec<String>,
}

impl Default for GuardSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            patterns: gitpoulpi_core::privacy::default_sensitive_patterns(),
        }
    }
}

type Stamp = (Option<PathBuf>, Option<(SystemTime, u64)>);

#[derive(PartialEq, Eq)]
struct Key {
    file: Stamp,
    env_guard: Option<String>,
    env_patterns: Option<String>,
}

static CACHE: Mutex<Option<(Key, GuardSettings)>> = Mutex::new(None);

fn key() -> Key {
    let file = gitpoulpi_config::ConfigPaths::discover().map(|p| p.config_file());
    let stamp = file
        .as_ref()
        .and_then(|f| std::fs::metadata(f).ok())
        .and_then(|m| Some((m.modified().ok()?, m.len())));
    Key {
        file: (file, stamp),
        env_guard: std::env::var("GITPOULPI_PRIVACY_STAGING_GUARD").ok(),
        env_patterns: std::env::var("GITPOULPI_PRIVACY_SENSITIVE_PATTERNS").ok(),
    }
}

/// The settings now. Anything unreadable is the default (guard on).
pub fn settings() -> GuardSettings {
    let key = key();
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((cached, s)) = cache.as_ref() {
        if *cached == key {
            return s.clone();
        }
    }
    let s = gitpoulpi_config::Settings::resolve(
        gitpoulpi_config::ConfigPaths::discover().as_ref(),
        &gitpoulpi_config::Overrides::default(),
    )
    .map(|s| GuardSettings {
        enabled: s.config().privacy.staging_guard,
        patterns: s.config().privacy.sensitive_patterns.clone(),
    })
    .unwrap_or_default();
    *cache = Some((key, s.clone()));
    s
}

/// `Ok` when the request may go on; a refusal as the dispatcher carries it.
/// Only `StageFile`, `StageFolder` and `StageAll` are checked.
pub(crate) fn check(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    command: OperationKind,
    target: &StageTarget,
    args: &Value,
) -> Result<(), Failure> {
    if !matches!(
        command,
        OperationKind::StageFile | OperationKind::StageFolder | OperationKind::StageAll
    ) || args
        .get("allow_sensitive")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Ok(());
    }
    let s = settings();
    match scan(backend, repo, target, &s) {
        Ok(files) if files.is_empty() => Ok(()),
        Ok(files) => {
            let e = Error::SensitiveStage { files };
            Err((e.to_string(), e.typed(), None))
        }
        // A status that cannot be read is the stage's own failure to report.
        Err(_) => Ok(()),
    }
}

/// What staging `target` would add that looks sensitive.
pub fn scan(
    backend: &dyn GitBackend,
    repo: &RepoRef,
    target: &StageTarget,
    settings: &GuardSettings,
) -> gitpoulpi_core::Result<Vec<Sensitive>> {
    if !settings.enabled {
        return Ok(Vec::new());
    }
    let paths: Vec<PathBuf> = match target {
        StageTarget::File(p) => vec![p.clone()],
        StageTarget::Folder(dir) => changed(backend, repo)?
            .into_iter()
            .filter(|p| p.starts_with(dir))
            .collect(),
        StageTarget::All => changed(backend, repo)?,
        // Hunks and lines are parts of a file that is already being worked on.
        StageTarget::Hunk(..) | StageTarget::Line(..) => return Ok(Vec::new()),
    };
    let mut found = Vec::new();
    let mut total = 0u64;
    for path in paths {
        let shown = path.to_string_lossy().replace('\\', "/");
        let content = read_text(&repo.root.join(&path), &mut total);
        found.extend(classify(&shown, content.as_deref(), &settings.patterns));
    }
    Ok(found)
}

/// The paths with a working-tree change to stage (a deletion has no content
/// to put in the index, but its name is still judged).
fn changed(backend: &dyn GitBackend, repo: &RepoRef) -> gitpoulpi_core::Result<Vec<PathBuf>> {
    Ok(backend
        .status(repo)?
        .files
        .into_iter()
        .filter(|f| f.worktree.is_some())
        .filter(|f| f.worktree != Some(FileStatus::Deleted))
        .map(|f| f.path)
        .collect())
}

/// The content of a regular, non-binary file within the caps, else `None`.
fn read_text(path: &Path, total: &mut u64) -> Option<Vec<u8>> {
    let meta = std::fs::symlink_metadata(path).ok()?;
    if !meta.is_file() || meta.len() > MAX_FILE || *total + meta.len() > MAX_TOTAL {
        return None;
    }
    let mut bytes = Vec::with_capacity(meta.len() as usize);
    std::fs::File::open(path)
        .ok()?
        .take(MAX_FILE)
        .read_to_end(&mut bytes)
        .ok()?;
    *total += bytes.len() as u64;
    // A NUL in the first 8 KiB: a binary file, matched by name only.
    if bytes.iter().take(8192).any(|b| *b == 0) {
        return None;
    }
    Some(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_binary_file_and_a_big_file_are_not_read() {
        let dir = tempfile::tempdir().unwrap();
        let bin = dir.path().join("a.bin");
        std::fs::write(&bin, b"ghp_\0abc").unwrap();
        let big = dir.path().join("big.txt");
        std::fs::write(&big, vec![b'a'; (MAX_FILE + 1) as usize]).unwrap();
        let ok = dir.path().join("ok.txt");
        std::fs::write(&ok, b"hello").unwrap();
        let mut total = 0;
        assert!(read_text(&bin, &mut total).is_none());
        assert!(read_text(&big, &mut total).is_none());
        assert_eq!(read_text(&ok, &mut total).unwrap(), b"hello");
        assert_eq!(total, 5 + 8);
        assert!(read_text(&dir.path().join("missing"), &mut total).is_none());
    }

    #[test]
    fn the_total_cap_stops_reading_content() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("f.txt");
        std::fs::write(&f, b"x").unwrap();
        let mut total = MAX_TOTAL;
        assert!(read_text(&f, &mut total).is_none());
    }
}
