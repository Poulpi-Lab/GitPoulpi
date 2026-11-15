//! Drafts of the texts being written, one per target, so a reply that was
//! not sent is still there the next time the same target is opened.
//!
//! One file in the config directory (`remote-conversations.toml`, the plan's
//! file for targets without a clone; kept for every target, one place to
//! look), `[drafts]` keyed by `integration|project|kind#n|target`. Saved
//! after [`IDLE`] without typing and when the editor goes away; removed once
//! the text is sent or thrown away. Best effort everywhere: a draft that
//! cannot be written is a lost draft, never an error in the form.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Typing pauses this long before the draft is written.
pub(crate) const IDLE: Duration = Duration::from_millis(500);

const FILE: &str = "remote-conversations.toml";

#[derive(Debug, Default, Serialize, Deserialize)]
struct Drafts {
    #[serde(default)]
    drafts: BTreeMap<String, String>,
}

/// The key of a target from its parts (`integration`, `project`,
/// `issue#12`, `reply:thread-3`).
pub(crate) fn key(parts: &[&str]) -> String {
    parts.join("|")
}

/// The directory the drafts live in. Never the real config under test.
pub(crate) fn config_dir() -> Option<PathBuf> {
    if cfg!(test) {
        return None;
    }
    crate::session_store::config_dir()
}

fn read(dir: &Path) -> Drafts {
    std::fs::read_to_string(dir.join(FILE))
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default()
}

fn write(dir: &Path, drafts: &Drafts) {
    let Ok(text) = toml::to_string(drafts) else {
        return;
    };
    let path = dir.join(FILE);
    let tmp = dir.join(format!("{FILE}.tmp"));
    if std::fs::create_dir_all(dir).is_err() || std::fs::write(&tmp, text).is_err() {
        return;
    }
    if std::fs::rename(&tmp, &path).is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
}

/// The saved draft of `key`.
pub(crate) fn load(dir: &Path, key: &str) -> Option<String> {
    read(dir)
        .drafts
        .remove(key)
        .filter(|t| !t.trim().is_empty())
}

/// Saves `text` as the draft of `key` (a blank text removes it).
pub(crate) fn save(dir: &Path, key: &str, text: &str) {
    if text.trim().is_empty() {
        return remove(dir, key);
    }
    let mut all = read(dir);
    if all.drafts.get(key).map(String::as_str) == Some(text) {
        return;
    }
    all.drafts.insert(key.to_string(), text.to_string());
    write(dir, &all);
}

/// Forgets the draft of `key`.
pub(crate) fn remove(dir: &Path, key: &str) {
    let mut all = read(dir);
    if all.drafts.remove(key).is_some() {
        write(dir, &all);
    }
}
