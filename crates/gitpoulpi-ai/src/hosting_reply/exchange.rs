//! An exchange: the requests of one "Write with AI" sitting (a first draft,
//! then "shorter", then another instruction...), and what GitPoulpi gives back
//! to the model on the next one.
//!
//! A local Claude session that is resumed remembers all of it. Everything
//! else (an API profile, Codex, any other agent) does not, so the earlier
//! instructions and drafts go along again as proper turns: the first turn is
//! the conversation message (the same text every time, so a provider's prefix
//! cache keeps it), then each answer and the instruction that followed. They
//! are held to the conversation budget, oldest follow-ups first, and the
//! newest draft is always kept.
//!
//! The TUI keeps the list with the composer (dropped when it closes, and on
//! "Refresh context"); the CLI keeps it in a small file of its state folder,
//! for `--continue` ([`ExchangeStore`]).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::Turn;

/// Most exchanges kept in a list (the first and the newest ones).
pub const MAX_EXCHANGES: usize = 24;

/// One finished request of an exchange: what the user asked, the text they
/// asked to improve, and the answer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Exchange {
    pub hint: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub draft: Option<String>,
    pub answer: String,
}

/// Adds `exchange`, keeping the first (it fixes the conversation message and
/// the repository context) and the newest [`MAX_EXCHANGES`] - 1.
pub fn push_capped(list: &mut Vec<Exchange>, exchange: Exchange) {
    list.push(exchange);
    while list.len() > MAX_EXCHANGES {
        list.remove(1);
    }
}

fn bytes(turn: &Turn) -> usize {
    turn.user.len() + turn.assistant.len()
}

/// `turns` held to `budget` bytes: the follow-ups in the middle go first, the
/// oldest first. The first turn (the conversation) and the last (the newest
/// draft) always stay. Returns the turns and how many were left out.
pub fn trim(mut turns: Vec<Turn>, budget: usize) -> (Vec<Turn>, usize) {
    let mut total: usize = turns.iter().map(bytes).sum();
    let mut left_out = 0;
    while total > budget && turns.len() > 2 {
        total -= bytes(&turns.remove(1));
        left_out += 1;
    }
    (turns, left_out)
}

/// The exchange of the CLI, in a file of its state folder.
#[derive(Debug, Clone)]
pub struct ExchangeStore {
    path: PathBuf,
}

impl ExchangeStore {
    /// The store for `key` (see [`key_of`]) under `dir`.
    pub fn on_disk(dir: &Path, key: &str) -> Self {
        Self {
            path: dir.join(format!("{key}.json")),
        }
    }

    /// What was kept; nothing (or a file that does not read) is an empty list.
    pub fn load(&self) -> Vec<Exchange> {
        std::fs::read_to_string(&self.path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    /// Keeps `list` (user-only file); an empty list removes the file.
    pub fn save(&self, list: &[Exchange]) {
        if list.is_empty() {
            let _ = std::fs::remove_file(&self.path);
            return;
        }
        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(text) = serde_json::to_string(list) {
            let _ = gitpoulpi_config::secure_fs::write_private(&self.path, text.as_bytes());
        }
    }
}

/// A stable file name for what an exchange is about (the repository, the
/// target and what picks it: a thread, a path, a title).
pub fn key_of(parts: &[&str]) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in parts.join("\u{1}").bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn turn(user: &str, assistant: &str) -> Turn {
        Turn {
            user: user.into(),
            assistant: assistant.into(),
        }
    }

    #[test]
    fn trimming_drops_the_oldest_follow_ups_and_keeps_the_ends() {
        let turns = vec![
            turn("conversation", "a0"),
            turn("u1", "a1"),
            turn("u2", "a2"),
            turn("u3", "a3"),
        ];
        let all: usize = turns.iter().map(bytes).sum();
        assert_eq!(trim(turns.clone(), all), (turns.clone(), 0));
        // One byte under: the oldest middle turn goes.
        let (kept, left_out) = trim(turns.clone(), all - 1);
        assert_eq!(left_out, 1);
        assert_eq!(
            kept,
            vec![turns[0].clone(), turns[2].clone(), turns[3].clone()]
        );
        // Nothing fits: the conversation and the newest draft stay.
        let (kept, left_out) = trim(turns.clone(), 0);
        assert_eq!(left_out, 2);
        assert_eq!(kept, vec![turns[0].clone(), turns[3].clone()]);
    }

    #[test]
    fn the_list_keeps_its_first_and_its_newest_entries() {
        let mut list = Vec::new();
        for i in 0..(MAX_EXCHANGES + 3) {
            push_capped(
                &mut list,
                Exchange {
                    hint: format!("h{i}"),
                    draft: None,
                    answer: format!("a{i}"),
                },
            );
        }
        assert_eq!(list.len(), MAX_EXCHANGES);
        assert_eq!(list[0].hint, "h0");
        assert_eq!(list[1].hint, "h4");
        assert_eq!(list.last().unwrap().hint, format!("h{}", MAX_EXCHANGES + 2));
    }

    #[test]
    fn the_store_round_trips_and_an_empty_list_removes_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let store = ExchangeStore::on_disk(dir.path(), &key_of(&["/r", "comment"]));
        assert!(store.load().is_empty());
        let list = vec![Exchange {
            hint: "shorter".into(),
            draft: Some("text".into()),
            answer: "t".into(),
        }];
        store.save(&list);
        assert_eq!(store.load(), list);
        store.save(&[]);
        assert!(store.load().is_empty());
        assert_ne!(key_of(&["/r", "comment"]), key_of(&["/r", "thread_reply"]));
    }
}
