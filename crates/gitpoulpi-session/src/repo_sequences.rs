//! What a repository's own `.gitpoulpi/sequences.toml` is worth right now:
//! its sequences, their hash and whether the user trusted exactly that.
//!
//! The file can come from other people, so it is read once here and the
//! answer carries both the sequences and the hash they were judged by:
//! what runs is what was trusted, with no gap between check and use. The
//! trust record lives in [`RepoState::trusted_sequences`].

use crate::RepoState;
use gitpoulpi_config::sequences::{
    parse_repo_sequences, repo_file, repo_sequences_hash, SequenceDef,
};
use std::path::Path;

/// Where the repository's file stands against the trust record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SequencesTrust {
    /// No file, or a file with no sequence: nothing to ask.
    None,
    /// The user trusted exactly these sequences.
    Trusted,
    /// Never trusted: ask.
    Untrusted,
    /// Trusted once, then an id, a command, a pause or an option changed:
    /// ask again.
    Changed,
}

/// The repository's sequences file as read from disk.
#[derive(Debug, Clone)]
pub struct RepoSequences {
    /// Every sequence of the file (scope `repo`). Never run these unless
    /// `trust` is [`SequencesTrust::Trusted`].
    pub defs: Vec<SequenceDef>,
    /// `repo_sequences_hash` of `defs` (what trusting records).
    pub hash: String,
    pub trust: SequencesTrust,
    /// Why the file could not be read, when it could not (it then holds no
    /// sequences and asks nothing).
    pub error: Option<String>,
}

impl RepoSequences {
    /// The sequences that may be listed, bound and run: the file's, but only
    /// when trusted.
    pub fn trusted(&self) -> &[SequenceDef] {
        if self.trust == SequencesTrust::Trusted {
            &self.defs
        } else {
            &[]
        }
    }

    /// Whether the user has to be asked.
    pub fn needs_answer(&self) -> bool {
        matches!(
            self.trust,
            SequencesTrust::Untrusted | SequencesTrust::Changed
        )
    }
}

/// Reads `<root>/.gitpoulpi/sequences.toml` and compares it with `state`.
pub fn inspect_repo_sequences(root: &Path, state: &RepoState) -> RepoSequences {
    let path = repo_file(root);
    let empty = |error: Option<String>| RepoSequences {
        defs: Vec::new(),
        hash: String::new(),
        trust: SequencesTrust::None,
        error,
    };
    if !path.exists() {
        return empty(None);
    }
    let defs = match std::fs::read_to_string(&path)
        .map_err(|e| e.to_string())
        .and_then(|text| parse_repo_sequences(&text).map_err(|e| e.to_string()))
    {
        Ok(defs) => defs,
        Err(why) => return empty(Some(why)),
    };
    if defs.is_empty() {
        return empty(None);
    }
    let hash = repo_sequences_hash(&defs);
    let trust = if state.trusts_sequences(&hash) {
        SequencesTrust::Trusted
    } else if state.trusted_sequences.is_empty() {
        SequencesTrust::Untrusted
    } else {
        SequencesTrust::Changed
    };
    RepoSequences {
        defs,
        hash,
        trust,
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(root: &Path, text: &str) {
        let file = repo_file(root);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, text).unwrap();
    }

    const ONE: &str = r#"
[[sequences]]
id = "ship"
name = "Ship it"
[[sequences.steps]]
run = "stage_all"
"#;

    #[test]
    fn a_file_is_untrusted_until_its_hash_is_recorded() {
        let dir = tempfile::tempdir().unwrap();
        let mut state = RepoState::default();
        assert_eq!(
            inspect_repo_sequences(dir.path(), &state).trust,
            SequencesTrust::None,
            "no file"
        );
        write(dir.path(), ONE);
        let seen = inspect_repo_sequences(dir.path(), &state);
        assert_eq!(seen.trust, SequencesTrust::Untrusted);
        assert!(seen.trusted().is_empty(), "never offered while untrusted");
        assert!(seen.needs_answer());
        state.trust_sequences(&seen.hash, vec!["ship".into()], 1);
        let seen = inspect_repo_sequences(dir.path(), &state);
        assert_eq!(seen.trust, SequencesTrust::Trusted);
        assert_eq!(seen.trusted().len(), 1);
    }

    #[test]
    fn a_changed_command_asks_again_but_a_new_name_does_not() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), ONE);
        let mut state = RepoState::default();
        let first = inspect_repo_sequences(dir.path(), &state);
        state.trust_sequences(&first.hash, vec!["ship".into()], 1);
        write(dir.path(), &ONE.replace("Ship it", "Renamed"));
        assert_eq!(
            inspect_repo_sequences(dir.path(), &state).trust,
            SequencesTrust::Trusted,
            "names do not count"
        );
        write(dir.path(), &ONE.replace("stage_all", "push"));
        let changed = inspect_repo_sequences(dir.path(), &state);
        assert_eq!(changed.trust, SequencesTrust::Changed);
        assert!(changed.trusted().is_empty());
        write(dir.path(), &format!("{ONE}pause = \"always\"\n"));
        assert_eq!(
            inspect_repo_sequences(dir.path(), &state).trust,
            SequencesTrust::Changed,
            "a pause counts"
        );
    }

    #[test]
    fn an_unreadable_file_asks_nothing_and_says_why() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "[[sequences]\nbroken");
        let seen = inspect_repo_sequences(dir.path(), &RepoState::default());
        assert_eq!(seen.trust, SequencesTrust::None);
        assert!(seen.error.is_some());
    }
}
