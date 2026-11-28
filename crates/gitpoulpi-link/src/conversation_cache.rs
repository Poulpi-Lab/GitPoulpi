//! The saved copy of a timeline: what paints at once when an item opens (and
//! what an offline window shows), refreshed behind it.
//!
//! One JSON file per item under `<state dir>/conversations/<integration>/
//! <project>/<kind>-<n>.json`, written atomically and read leniently (a file
//! that does not parse is a miss). It holds the page the host sent and the
//! time it was saved, never a token. A file over [`MAX_BYTES`] is not
//! written, and only the [`KEEP`] most recent files are kept.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use gitpoulpi_hosting::conversation::{ItemKey, ItemKind, TimelinePage};
use serde::{Deserialize, Serialize};

/// The largest file written.
pub const MAX_BYTES: usize = 2 * 1024 * 1024;
/// Files kept across all items of one repository.
pub const KEEP: usize = 50;

#[derive(Serialize, Deserialize)]
struct Saved {
    saved_at: i64,
    page: TimelinePage,
}

pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

/// A path component: only letters, digits, `-` and `_` survive.
fn slug(text: &str) -> String {
    let s: String = text
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect();
    if s.is_empty() {
        "_".to_string()
    } else {
        s
    }
}

/// Where item `key` of `project` on `integration` is saved under `dir`.
pub fn file_for(dir: &Path, integration: &str, project: &str, key: &ItemKey) -> PathBuf {
    let kind = match key.kind {
        ItemKind::Issue => "issue",
        ItemKind::PullRequest => "request",
    };
    let id = if key.id.is_empty() {
        key.number.to_string()
    } else {
        key.id.clone()
    };
    dir.join("conversations")
        .join(slug(integration))
        .join(slug(project))
        .join(format!("{kind}-{}.json", slug(&id)))
}

/// The saved page and when it was saved (unix seconds).
pub fn read(file: &Path) -> Option<(TimelinePage, i64)> {
    let text = std::fs::read_to_string(file).ok()?;
    let saved: Saved = serde_json::from_str(&text).ok()?;
    Some((saved.page, saved.saved_at))
}

/// Saves `page`; failures are silent (a missing copy is only a slower open).
pub fn write(file: &Path, page: &TimelinePage) {
    let Ok(text) = serde_json::to_string(&Saved {
        saved_at: now(),
        page: page.clone(),
    }) else {
        return;
    };
    if text.len() > MAX_BYTES {
        return;
    }
    if let Some(parent) = file.parent() {
        if gitpoulpi_config::secure_fs::create_private_dir_all(parent).is_err() {
            return;
        }
    }
    if gitpoulpi_config::secure_fs::write_private(file, text.as_bytes()).is_ok() {
        prune(file);
    }
}

/// Keeps the [`KEEP`] newest files under `<dir>/conversations`.
fn prune(file: &Path) {
    let Some(root) = file
        .ancestors()
        .find(|p| p.file_name().is_some_and(|n| n == "conversations"))
    else {
        return;
    };
    let mut all: Vec<(SystemTime, PathBuf)> = Vec::new();
    collect(root, &mut all, 0);
    if all.len() <= KEEP {
        return;
    }
    all.sort_by_key(|(modified, _)| std::cmp::Reverse(*modified));
    for (_, path) in all.into_iter().skip(KEEP) {
        let _ = std::fs::remove_file(path);
    }
}

fn collect(dir: &Path, out: &mut Vec<(SystemTime, PathBuf)>, depth: usize) {
    let Ok(read) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in read.flatten() {
        let path = entry.path();
        if path.is_dir() && depth < 3 {
            collect(&path, out, depth + 1);
        } else if path.extension().is_some_and(|e| e == "json") {
            if let Ok(modified) = entry.metadata().and_then(|m| m.modified()) {
                out.push((modified, path));
            }
        }
    }
}
